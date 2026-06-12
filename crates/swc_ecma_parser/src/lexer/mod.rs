//! ECMAScript lexer.

use std::{borrow::Cow, char};

use either::Either::{self, Left, Right};
use num_bigint::BigInt as NumBigInt;
use rustc_hash::FxHashMap;
use smartstring::{LazyCompact, SmartString};
use swc_experimental_allocator::{
    Allocator,
    atom::{Atom, Wtf8Atom},
    wtf8::{CodePoint, Wtf8Buf},
};
use swc_experimental_ecma_ast::{
    Comment, CommentKind, Comments, EsVersion, Span, is_valid_ascii_continue, is_valid_ascii_start,
    is_valid_non_ascii_continue, is_valid_non_ascii_start,
};

use self::table::{BYTE_HANDLERS, ByteHandler};
use crate::{
    Context, Syntax, byte_search,
    error::{Error, SyntaxError},
    input::Tokens,
    lexer::{
        char_ext::CharExt,
        comments_buffer::{BufferedComment, BufferedCommentKind, CommentsBuffer},
        jsx::xhtml,
        number::{LazyInteger, parse_integer},
        search::SafeByteMatchTable,
        source::StringSource,
        state::State,
    },
    safe_byte_match_table,
    syntax::SyntaxFlags,
};

#[cfg(feature = "unstable")]
pub(crate) mod capturing;
mod char_ext;
mod comments_buffer;
mod jsx;
mod number;
pub(crate) mod search;
pub(crate) mod source;
mod state;
mod table;
pub(crate) mod token;
mod whitespace;

pub(crate) use state::TokenFlags;
pub(crate) use token::{NextTokenAndSpan, Token, TokenAndSpan, TokenValue};

// ===== Byte match tables for comment scanning =====
// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xe2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xa8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xa9];

static LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\n' | b'\r' | LS_OR_PS_FIRST));

static BLOCK_COMMENT_SCAN_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| { matches!(b, b'*' | b'\n' | b'\r' | LS_OR_PS_FIRST) });

static DOUBLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'"' | b'\n' | b'\\' | b'\r'));
static SINGLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\'' | b'\n' | b'\\' | b'\r'));

static NOT_ASCII_ID_CONTINUE_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !(b.is_ascii_alphanumeric() || b == b'_' || b == b'$'));

/// Converts UTF-16 surrogate pair to Unicode code point.
/// `https://tc39.es/ecma262/#sec-utf16decodesurrogatepair`
#[inline]
const fn pair_to_code_point(high: u32, low: u32) -> u32 {
    (high - 0xd800) * 0x400 + low - 0xdc00 + 0x10000
}

/// A Unicode escape sequence.
///
/// `\u Hex4Digits`, `\u Hex4Digits \u Hex4Digits`, or `\u{ HexDigits }`.
#[derive(Debug)]
pub enum UnicodeEscape {
    // `\u Hex4Digits` or `\u{ HexDigits }`, which forms a valid Unicode code point.
    // Char cannot be in range 0xD800..=0xDFFF.
    CodePoint(char),
    // `\u Hex4Digits \u Hex4Digits`, which forms a valid Unicode astral code point.
    // Char is in the range 0x10000..=0x10FFFF.
    SurrogatePair(char),
    // `\u Hex4Digits` or `\u{ HexDigits }`, which forms an invalid Unicode code point.
    // Code unit is in the range 0xD800..=0xDFFF.
    LoneSurrogate(u32),
}

impl From<UnicodeEscape> for CodePoint {
    fn from(value: UnicodeEscape) -> Self {
        match value {
            UnicodeEscape::CodePoint(c) | UnicodeEscape::SurrogatePair(c) => {
                CodePoint::from_char(c)
            }
            UnicodeEscape::LoneSurrogate(u) => unsafe { CodePoint::from_u32_unchecked(u) },
        }
    }
}

pub type LexResult<T> = Result<T, crate::error::Error>;
type NumberEither<'a> = Either<f64, Atom<'a>>;

fn remove_underscore(s: &str, has_underscore: bool) -> Cow<'_, str> {
    if has_underscore {
        debug_assert!(s.contains('_'));
        s.chars().filter(|&c| c != '_').collect::<String>().into()
    } else {
        debug_assert!(!s.contains('_'));
        Cow::Borrowed(s)
    }
}

fn bigint_atom_from_digits<'a>(
    s: &'a str,
    radix: u32,
    has_underscore: bool,
    allocator: &'a Allocator,
) -> Option<Atom<'a>> {
    if radix == 10 && !has_underscore {
        return Some(Atom::from(s));
    }

    let s = remove_underscore(s, has_underscore);
    if radix == 10 {
        return Some(Atom::new_in(s.as_ref(), allocator));
    }

    let value = NumBigInt::parse_bytes(s.as_bytes(), radix)?;
    Some(Atom::new_in(&value.to_string(), allocator))
}

pub struct Lexer<'a, 'cmt> {
    allocator: &'a Allocator,
    comments: Option<&'cmt mut Comments<'a>>,
    /// [Some] if comment comment parsing is enabled. Otherwise [None]
    comments_buffer: Option<CommentsBuffer<'a>>,

    pub ctx: Context,
    input: StringSource<'a>,
    start_pos: u32,

    state: State<'a>,
    token_flags: TokenFlags,
    pub(crate) syntax: SyntaxFlags,
    pub(crate) target: EsVersion,

    errors: Vec<Error>,
    module_errors: Vec<Error>,
}

impl<'a, 'cmt> Lexer<'a, 'cmt> {
    #[inline(always)]
    fn input(&self) -> &StringSource<'a> {
        &self.input
    }

    #[inline(always)]
    fn input_mut(&mut self) -> &mut StringSource<'a> {
        &mut self.input
    }

    #[inline(always)]
    fn push_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    #[inline(always)]
    fn state(&self) -> &State<'a> {
        &self.state
    }

    #[inline(always)]
    fn comments_buffer(&self) -> Option<&CommentsBuffer<'a>> {
        self.comments_buffer.as_ref()
    }

    #[inline(always)]
    unsafe fn input_slice_to_cur(&mut self, start: u32) -> &'a str {
        unsafe { self.input.slice_to_cur(start) }
    }

    #[inline(always)]
    unsafe fn input_slice(&mut self, start: u32, end: u32) -> &'a str {
        unsafe { self.input.slice(start, end) }
    }

    #[inline(always)]
    fn atom_from_slice(&self, start: u32, end: u32) -> Atom<'a> {
        Atom::from(unsafe { self.input.slice(start, end) })
    }

    #[inline(always)]
    fn wtf8_atom_from_slice(&self, start: u32, end: u32) -> Wtf8Atom<'a> {
        Wtf8Atom::from(unsafe { self.input.slice(start, end) })
    }
}

impl<'a, 'cmt> Lexer<'a, 'cmt> {
    pub fn new(
        allocator: &'a Allocator,
        syntax: Syntax,
        target: EsVersion,
        input: StringSource<'a>,
        comments: Option<&'cmt mut Comments<'a>>,
    ) -> Self {
        let start_pos = input.cur_pos();
        let comments_buffer = comments.is_some().then(CommentsBuffer::new);
        Lexer {
            allocator,
            comments,
            comments_buffer,
            ctx: Default::default(),
            input,
            start_pos,
            state: State::new(start_pos),
            syntax: syntax.into_flags(),
            target,
            errors: Default::default(),
            module_errors: Default::default(),
            token_flags: TokenFlags::empty(),
        }
    }

    /// babel: `getTokenFromCode`
    fn read_token(&mut self) -> LexResult<Token> {
        self.token_flags = TokenFlags::empty();
        let byte = match self.input.peek() {
            Some(v) => v,
            None => return Ok(Token::Eof),
        };

        let handler = unsafe { *(&BYTE_HANDLERS as *const ByteHandler).offset(byte as isize) };
        handler(self)
    }

    fn read_token_plus_minus<const C: u8>(&mut self) -> LexResult<Token> {
        let start = self.cur_pos();

        self.bump(1);
        // '++', '--'
        Ok(if self.input.peek() == Some(C) {
            self.bump(1);
            // Handle -->
            if self.state.had_line_break && C == b'-' && self.eat(b'>') {
                self.emit_module_mode_error(start, SyntaxError::LegacyCommentInModule);
                self.skip_line_comment(0);
                self.skip_space();
                return self.read_token();
            }

            if C == b'+' {
                Token::PlusPlus
            } else {
                Token::MinusMinus
            }
        } else if self.input.eat_byte(b'=') {
            if C == b'+' {
                Token::PlusEq
            } else {
                Token::MinusEq
            }
        } else if C == b'+' {
            Token::Plus
        } else {
            Token::Minus
        })
    }

    fn read_token_bang_or_eq<const C: u8>(&mut self) -> LexResult<Token> {
        let start = self.cur_pos();
        let had_line_break_before_last = self.state.had_line_break;

        self.bump(1);

        Ok(if self.input.eat_byte(b'=') {
            // "=="

            if self.input.eat_byte(b'=') {
                if C == b'!' {
                    Token::NotEqEq
                } else {
                    // =======
                    //    ^
                    if had_line_break_before_last && self.is_str("====") {
                        self.emit_error_span(fixed_len_span(start, 7), SyntaxError::TS1185);
                        self.skip_line_comment(4);
                        self.skip_space();
                        return self.read_token();
                    }

                    Token::EqEqEq
                }
            } else if C == b'!' {
                Token::NotEq
            } else {
                Token::EqEq
            }
        } else if C == b'=' && self.input.eat_byte(b'>') {
            // "=>"

            Token::Arrow
        } else if C == b'!' {
            Token::Bang
        } else {
            Token::Eq
        })
    }
}

impl<'a> Lexer<'a, '_> {
    fn read_token_lt_gt<const C: u8>(&mut self) -> LexResult<Token> {
        let had_line_break_before_last = self.state.had_line_break;
        let start = self.cur_pos();
        self.bump(1);

        if self.syntax.typescript()
            && self.ctx.contains(Context::InType)
            && !self.ctx.contains(Context::ShouldNotLexLtOrGtAsType)
        {
            if C == b'<' {
                return Ok(Token::Lt);
            } else if C == b'>' {
                return Ok(Token::Gt);
            }
        }

        // XML style comment. `<!--`
        if C == b'<' && self.is(b'!') && self.peek_2() == Some(b'-') && self.peek_3() == Some(b'-')
        {
            self.skip_line_comment(3);
            self.skip_space();
            self.emit_module_mode_error(start, SyntaxError::LegacyCommentInModule);

            return self.read_token();
        }

        let mut op = if C == b'<' { Token::Lt } else { Token::Gt };

        // '<<', '>>'
        if self.peek() == Some(C) {
            self.bump(1);
            op = if C == b'<' {
                Token::LShift
            } else {
                Token::RShift
            };

            //'>>>'
            if C == b'>' && self.peek() == Some(C) {
                self.bump(1);
                op = Token::ZeroFillRShift;
            }
        }

        let token = if self.eat(b'=') {
            match op {
                Token::Lt => Token::LtEq,
                Token::Gt => Token::GtEq,
                Token::LShift => Token::LShiftEq,
                Token::RShift => Token::RShiftEq,
                Token::ZeroFillRShift => Token::ZeroFillRShiftEq,
                _ => unreachable!(),
            }
        } else {
            op
        };

        // All conflict markers consist of the same character repeated seven times.
        // If it is a <<<<<<< or >>>>>>> marker then it is also followed by a space.
        // <<<<<<<
        //   ^
        // >>>>>>>
        //    ^
        if had_line_break_before_last
            && match op {
                Token::LShift if self.is_str("<<<<< ") => true,
                Token::ZeroFillRShift if self.is_str(">>>> ") => true,
                _ => false,
            }
        {
            self.emit_error_span(fixed_len_span(start, 7), SyntaxError::TS1185);
            self.skip_line_comment(5);
            self.skip_space();
            return self.read_token();
        }

        Ok(token)
    }

    fn read_token_back_quote(&mut self) -> LexResult<Token> {
        let start = self.cur_pos();
        self.scan_template_token(start, true)
    }

    fn scan_template_token(&mut self, start: u32, started_with_backtick: bool) -> LexResult<Token> {
        debug_assert!(self.peek() == Some(if started_with_backtick { b'`' } else { b'}' }));
        let mut cooked = Ok(Wtf8Buf::new());
        self.bump(1); // `}` or `\``
        let cooked_start = self.cur_pos();
        let mut cooked_slice_start = self.cur_pos();
        let mut cooked_needs_alloc = false;
        macro_rules! consume_cooked {
            () => {{
                if cooked_needs_alloc {
                    if let Ok(cooked) = &mut cooked {
                        let last_pos = self.cur_pos();
                        let s = unsafe {
                            // Safety: Both of start and last_pos are valid position because we got them
                            // from `self.input`
                            self.input.slice(cooked_slice_start, last_pos)
                        };
                        cooked.push_str(s);
                    }
                }
            }};
        }
        macro_rules! finish_cooked {
            () => {{
                if cooked_needs_alloc {
                    cooked.map(|atom| Wtf8Atom::new_in(atom, self.allocator))
                } else {
                    Ok(self.wtf8_atom_from_slice(cooked_start, self.cur_pos()))
                }
            }};
        }

        while let Some(c) = self.peek() {
            if c == b'`' {
                consume_cooked!();
                let cooked = finish_cooked!();
                self.bump(1);
                return Ok(if started_with_backtick {
                    self.set_token_value(Some(TokenValue::Template(cooked)));
                    Token::NoSubstitutionTemplateLiteral
                } else {
                    self.set_token_value(Some(TokenValue::Template(cooked)));
                    Token::TemplateTail
                });
            } else if c == b'$' && self.input.peek_2() == Some(b'{') {
                consume_cooked!();
                let cooked = finish_cooked!();

                // Safety: `self.input.peek_2() == Some(b'{')`
                self.bump(2);
                return Ok(if started_with_backtick {
                    self.set_token_value(Some(TokenValue::Template(cooked)));
                    Token::TemplateHead
                } else {
                    self.set_token_value(Some(TokenValue::Template(cooked)));
                    Token::TemplateMiddle
                });
            } else if c == b'\\' {
                cooked_needs_alloc = true;
                consume_cooked!();

                match self.read_escaped_char(true) {
                    Ok(Some(escaped)) => {
                        if let Ok(ref mut cooked) = cooked {
                            cooked.push(escaped);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        cooked = Err(error);
                    }
                }

                cooked_slice_start = self.cur_pos();
            } else if c.is_line_terminator() {
                cooked_needs_alloc = true;
                consume_cooked!();

                // For line terminators, we need the full char (can be multi-byte UTF-8)
                let c_char = if c <= 0x7f {
                    c as char
                } else {
                    self.input.peek_char().unwrap()
                };

                let c = if c == b'\r' && self.peek_2() == Some(b'\n') {
                    self.bump(1); // '\r'
                    '\n'
                } else {
                    match c_char {
                        '\n' => '\n',
                        '\r' => '\n',
                        '\u{2028}' => '\u{2028}',
                        '\u{2029}' => '\u{2029}',
                        _ => unreachable!(),
                    }
                };

                self.bump(c_char.len_utf8());

                if let Ok(ref mut cooked) = cooked {
                    cooked.push_char(c);
                }
                cooked_slice_start = self.cur_pos();
            } else if c <= 0x7f {
                self.bump(1);
            } else {
                let char = self.input.peek_char().unwrap();
                self.bump(char.len_utf8());
            };
        }

        self.error(start, SyntaxError::UnterminatedTpl)?
    }
}

impl<'a> Lexer<'a, '_> {
    #[inline(always)]
    fn span(&self, start: u32) -> Span {
        let end = self.last_pos();
        if cfg!(debug_assertions) && start > end {
            unreachable!(
                "assertion failed: (span.start <= span.end).
 start = {}, end = {}",
                start, end
            )
        }
        Span::new(start, end)
    }

    /// Advances the input by `len` bytes.
    ///
    /// For ASCII characters, use `bump(1)`.
    /// For unknown character length, use `c.len_utf8()` where c is a char.
    #[inline(always)]
    fn bump(&mut self, len: usize) {
        unsafe {
            self.input_mut().bump_bytes(len);
        }
    }

    #[inline(always)]
    fn is(&self, c: u8) -> bool {
        self.input().peek() == Some(c)
    }

    #[inline(always)]
    fn is_str(&self, s: &str) -> bool {
        self.input().as_str().starts_with(s)
    }

    #[inline(always)]
    fn eat(&mut self, c: u8) -> bool {
        self.input_mut().eat_byte(c)
    }

    #[inline(always)]
    fn peek(&self) -> Option<u8> {
        self.input().peek()
    }

    #[inline(always)]
    fn peek_2(&self) -> Option<u8> {
        self.input().peek_2()
    }

    #[inline(always)]
    fn peek_3(&self) -> Option<u8> {
        self.input().peek_3()
    }

    #[inline(always)]
    fn cur_pos(&self) -> u32 {
        self.input().cur_pos()
    }

    #[inline(always)]
    fn last_pos(&self) -> u32 {
        self.input().cur_pos()
    }

    /// Shorthand for `let span = self.span(start); self.error_span(span)`
    #[cold]
    #[inline(never)]
    fn error<T>(&self, start: u32, kind: SyntaxError) -> LexResult<T> {
        let span = self.span(start);
        self.error_span(span, kind)
    }

    #[cold]
    #[inline(never)]
    fn error_span<T>(&self, span: Span, kind: SyntaxError) -> LexResult<T> {
        Err(crate::error::Error::new(span, kind))
    }

    #[cold]
    #[inline(never)]
    fn emit_error(&mut self, start: u32, kind: SyntaxError) {
        let span = self.span(start);
        self.emit_error_span(span, kind)
    }

    #[cold]
    #[inline(never)]
    fn emit_error_span(&mut self, span: Span, kind: SyntaxError) {
        if self.ctx().contains(Context::IgnoreError) {
            return;
        }
        tracing::warn!("Lexer error at {:?}", span);
        let err = crate::error::Error::new(span, kind);
        self.push_error(err);
    }

    #[cold]
    #[inline(never)]
    fn emit_strict_mode_error(&mut self, start: u32, kind: SyntaxError) {
        let span = self.span(start);
        if self.ctx().contains(Context::Strict) {
            self.emit_error_span(span, kind);
        } else {
            let err = crate::error::Error::new(span, kind);
            self.add_module_mode_error(err);
        }
    }

    #[cold]
    #[inline(never)]
    fn emit_module_mode_error(&mut self, start: u32, kind: SyntaxError) {
        let span = self.span(start);
        let err = crate::error::Error::new(span, kind);
        self.add_module_mode_error(err);
    }

    #[inline(never)]
    fn skip_line_comment(&mut self, start_skip: usize) {
        // Position after the initial `//` (or similar)
        let start = self.cur_pos();
        unsafe { self.input_mut().bump_bytes(start_skip) };
        let slice_start = self.cur_pos();

        // foo // comment for foo
        // bar
        //
        // foo
        // // comment for bar
        // bar
        //
        let is_for_next =
            self.state.had_line_break || !self.state().can_have_trailing_line_comment();

        // Fast search for line-terminator
        byte_search! {
            lexer: self,
            table: LINE_BREAK_TABLE,
            continue_if: (matched_byte, pos_offset) {
                if matched_byte != LS_OR_PS_FIRST {
                    // '\r' or '\n' - definitely a line terminator
                    false
                } else {
                    // 0xE2 - could be LS/PS or some other Unicode character
                    // Check the next 2 bytes to see if it's really LS/PS
                    let current_slice = self.input().as_str();
                    let byte_pos = pos_offset;
                    if byte_pos + 2 < current_slice.len() {
                        let bytes = current_slice.as_bytes();
                        let next2 = [bytes[byte_pos + 1], bytes[byte_pos + 2]];
                        if next2 == LS_BYTES_2_AND_3 || next2 == PS_BYTES_2_AND_3 {
                            // It's a real line terminator
                            false
                        } else {
                            // Some other Unicode character starting with 0xE2
                            true
                        }
                    } else {
                        // Not enough bytes for full LS/PS sequence
                        true
                    }
                }
            },
            handle_eof: {
                // Reached EOF – entire remainder is comment
                let end = self.input().end_pos();

                if self.comments_buffer().is_some() {
                    let s = unsafe { self.input_slice(slice_start, end) };
                    let cmt = Comment::new(CommentKind::Line, Span::new(start, end), Atom::from(s));

                    if is_for_next {
                        self.comments_buffer.as_mut().unwrap().push_pending(cmt);
                    } else {
                        let pos = self.state.prev_hi;
                        self.comments_buffer.as_mut().unwrap().push_comment(BufferedComment {
                            kind: BufferedCommentKind::Trailing,
                            pos,
                            comment: cmt,
                        });
                    }
                }

                return;
            }
        };

        // Current position is at the line terminator
        let end = self.cur_pos();

        // Create and process slice only if comments need to be stored
        if self.comments_buffer().is_some() {
            let s = unsafe {
                // Safety: We know that the start and the end are valid
                self.input_slice_to_cur(slice_start)
            };
            let cmt = Comment::new(CommentKind::Line, Span::new(start, end), Atom::from(s));

            if is_for_next {
                self.comments_buffer.as_mut().unwrap().push_pending(cmt);
            } else {
                let pos = self.state.prev_hi;
                self.comments_buffer
                    .as_mut()
                    .unwrap()
                    .push_comment(BufferedComment {
                        kind: BufferedCommentKind::Trailing,
                        pos,
                        comment: cmt,
                    });
            }
        }

        unsafe {
            // Safety: We got end from self.input
            self.input_mut().reset_to(end);
        }
    }

    /// Expects current char to be '/' and next char to be '*'.
    fn skip_block_comment(&mut self) {
        let start = self.cur_pos();

        debug_assert_eq!(self.peek(), Some(b'/'));
        debug_assert_eq!(self.peek_2(), Some(b'*'));

        // Consume initial "/*"
        self.bump(2);

        // jsdoc
        let slice_start = self.cur_pos();

        let had_line_break_before_last = self.state.had_line_break;

        byte_search! {
            lexer: self,
            table: BLOCK_COMMENT_SCAN_TABLE,
            continue_if: (matched_byte, pos_offset) {
                match matched_byte {
                    LS_OR_PS_FIRST => {
                        // 0xE2 - could be LS/PS or some other Unicode character
                        let current_slice = self.input().as_str();
                        let byte_pos = pos_offset;
                        if byte_pos + 2 < current_slice.len() {
                            let bytes = current_slice.as_bytes();
                            let next2 = [bytes[byte_pos + 1], bytes[byte_pos + 2]];
                            if next2 == LS_BYTES_2_AND_3 || next2 == PS_BYTES_2_AND_3 {
                                self.state.had_line_break = true;
                                pos_offset += 2;
                            }
                        }
                        true
                    }
                    b'*' => {
                        let bytes = self.input().as_str().as_bytes();
                        if bytes.get(pos_offset + 1) == Some(&b'/') {
                            // Consume "*/"
                            pos_offset += 2;

                            // Decide trailing / leading
                            let mut is_for_next =
                                had_line_break_before_last || !self.state().can_have_trailing_comment();

                            // If next char is ';' without newline, treat as trailing
                            if !had_line_break_before_last && bytes.get(pos_offset) == Some(&b';') {
                                is_for_next = false;
                            }

                            if self.comments_buffer().is_some() {
                                let s = unsafe {
                                    let cur = self.input().cur_pos();
                                    // Safety: We got slice_start and end from self.input so those are
                                    // valid.
                                    let s = self.input_mut().slice(slice_start, slice_start + (pos_offset - 2) as u32);
                                    self.input_mut().reset_to(cur);
                                    s
                                };
                                let cmt = Comment::new(
                                    CommentKind::Block,
                                    Span::new(
                                        start,
                                        slice_start + pos_offset as u32,
                                    ),
                                    Atom::from(s),
                                );

                                if is_for_next {
                                    self.comments_buffer.as_mut().unwrap().push_pending(cmt);
                                } else {
                                    let pos = self.state.prev_hi;
                                    self.comments_buffer
                                        .as_mut()
                                        .unwrap()
                                        .push_comment(BufferedComment {
                                            kind: BufferedCommentKind::Trailing,
                                            pos,
                                            comment: cmt,
                                        });
                                }
                            }

                            false
                        } else {
                            true
                        }
                    }
                    _ => {
                        self.state.had_line_break = true;
                        true
                    },
                }
            },
            handle_eof: {
                let end_pos = self.input().end_pos();
                let span = Span::new(end_pos, end_pos);
                self.emit_error_span(span, SyntaxError::UnterminatedBlockComment);
                return;
            }
        };
    }

    /// Ensure that ident cannot directly follow numbers.
    fn ensure_not_ident(&mut self) -> LexResult<()> {
        match self.peek() {
            Some(c) if c.is_ident_start() => {
                let span = pos_span(self.cur_pos());
                self.error_span(span, SyntaxError::IdentAfterNum)?
            }
            _ => Ok(()),
        }
    }

    fn make_legacy_octal(&mut self, start: u32, val: f64) -> LexResult<f64> {
        self.ensure_not_ident()?;
        if self.syntax().typescript() && self.target() >= EsVersion::Es5 {
            self.emit_error(start, SyntaxError::TS1085);
        }
        self.emit_strict_mode_error(start, SyntaxError::LegacyOctal);
        Ok(val)
    }

    /// `op`- |total, radix, value| -> (total * radix + value, continue)
    fn read_digits<F, Ret, const RADIX: u8>(
        &mut self,
        mut op: F,
        allow_num_separator: bool,
        has_underscore: &mut bool,
    ) -> LexResult<Ret>
    where
        F: FnMut(Ret, u8, u32) -> LexResult<(Ret, bool)>,
        Ret: Copy + Default,
    {
        debug_assert!(
            RADIX == 2 || RADIX == 8 || RADIX == 10 || RADIX == 16,
            "radix for read_int should be one of 2, 8, 10, 16, but got {RADIX}"
        );

        if cfg!(feature = "debug") {
            tracing::trace!("read_digits(radix = {}), cur = {:?}", RADIX, self.peek());
        }

        let start = self.cur_pos();
        let mut total: Ret = Default::default();
        let mut prev = None;

        while let Some(c) = self.peek() {
            if c == b'_' {
                *has_underscore = true;
                if allow_num_separator {
                    let is_allowed = |c: Option<u8>| {
                        let Some(c) = c else {
                            return false;
                        };
                        (c as char).is_digit(RADIX as _)
                    };
                    let is_forbidden = |c: Option<u8>| {
                        let Some(c) = c else {
                            return false;
                        };

                        if RADIX == 16 {
                            matches!(c, b'.' | b'X' | b'_' | b'x')
                        } else {
                            matches!(c, b'.' | b'B' | b'E' | b'O' | b'_' | b'b' | b'e' | b'o')
                        }
                    };

                    let next = self.input().peek_2();

                    if !is_allowed(next) || is_forbidden(prev) || is_forbidden(next) {
                        self.emit_error(
                            start,
                            SyntaxError::NumericSeparatorIsAllowedOnlyBetweenTwoDigits,
                        );
                    }

                    self.bump(1);
                    continue;
                }
            }

            // e.g. (val for a) = 10  where radix = 16
            let val = if let Some(val) = (c as char).to_digit(RADIX as _) {
                val
            } else {
                return Ok(total);
            };

            self.bump(1);

            let (t, cont) = op(total, RADIX, val)?;

            total = t;

            if !cont {
                return Ok(total);
            }

            prev = Some(c);
        }

        Ok(total)
    }

    /// This can read long integers like
    /// "13612536612375123612312312312312312312312".
    ///
    /// - Returned `bool` is `true` is there was `8` or `9`.
    fn read_number_no_dot_as_str<const RADIX: u8>(&mut self) -> LexResult<LazyInteger> {
        debug_assert!(
            RADIX == 2 || RADIX == 8 || RADIX == 10 || RADIX == 16,
            "radix for read_number_no_dot should be one of 2, 8, 10, 16, but got {RADIX}"
        );
        let start = self.cur_pos();

        let mut not_octal = false;
        let mut read_any = false;
        let mut has_underscore = false;

        self.read_digits::<_, (), RADIX>(
            |_, _, v| {
                read_any = true;

                if v == 8 || v == 9 {
                    not_octal = true;
                }

                Ok(((), true))
            },
            true,
            &mut has_underscore,
        )?;

        if !read_any {
            self.error(start, SyntaxError::ExpectedDigit { radix: RADIX })?;
        }

        Ok(LazyInteger {
            start,
            end: self.cur_pos(),
            not_octal,
            has_underscore,
        })
    }

    /// Reads an integer, octal integer, or floating-point number
    fn read_number<const START_WITH_DOT: bool, const START_WITH_ZERO: bool>(
        &mut self,
    ) -> LexResult<NumberEither<'a>> {
        debug_assert!(!(START_WITH_DOT && START_WITH_ZERO));
        debug_assert!(self.peek().is_some());

        let start = self.cur_pos();
        let mut has_underscore = false;

        let lazy_integer = if START_WITH_DOT {
            // first char is '.'
            debug_assert!(
                self.peek().is_some_and(|c| c == b'.'),
                "read_number<START_WITH_DOT = true> expects current char to be '.'"
            );
            LazyInteger {
                start,
                end: start,
                not_octal: true,
                has_underscore: false,
            }
        } else {
            debug_assert!(!START_WITH_DOT);
            debug_assert!(!START_WITH_ZERO || self.peek().unwrap() == b'0');

            // Use read_number_no_dot to support long numbers.
            let lazy_integer = self.read_number_no_dot_as_str::<10>()?;
            let s = unsafe {
                // Safety: We got both start and end position from `self.input`
                self.input_slice_to_cur(lazy_integer.start)
            };

            // legacy octal number is not allowed in bigint.
            if (!START_WITH_ZERO || lazy_integer.end - lazy_integer.start == 1) && self.eat(b'n') {
                let bigint_value =
                    bigint_atom_from_digits(s, 10, lazy_integer.has_underscore, self.allocator)
                        .unwrap();
                return Ok(Either::Right(bigint_value));
            }

            if START_WITH_ZERO {
                // TODO: I guess it would be okay if I don't use -ffast-math
                // (or something like that), but needs review.
                if s.as_bytes().iter().all(|&c| c == b'0') {
                    // If only one zero is used, it's decimal.
                    // And if multiple zero is used, it's octal.
                    //
                    // e.g. `0` is decimal (so it can be part of float)
                    //
                    // e.g. `000` is octal
                    if start != self.last_pos() - 1 {
                        return self.make_legacy_octal(start, 0f64).map(Either::Left);
                    }
                } else if lazy_integer.not_octal {
                    // if it contains '8' or '9', it's decimal.
                    self.emit_strict_mode_error(start, SyntaxError::LegacyDecimal);
                } else {
                    // It's Legacy octal, and we should reinterpret value.
                    let s = remove_underscore(s, lazy_integer.has_underscore);
                    let val = parse_integer::<8>(&s);
                    return self.make_legacy_octal(start, val).map(Either::Left);
                }
            }

            lazy_integer
        };

        has_underscore |= lazy_integer.has_underscore;
        // At this point, number cannot be an octal literal.

        let has_dot = self.peek() == Some(b'.');
        //  `0.a`, `08.a`, `102.a` are invalid.
        //
        // `.1.a`, `.1e-4.a` are valid,
        if has_dot {
            self.bump(1);

            // equal: if START_WITH_DOT { debug_assert!(xxxx) }
            debug_assert!(!START_WITH_DOT || self.peek().is_some_and(|cur| cur.is_ascii_digit()));

            // Read numbers after dot
            self.read_digits::<_, (), 10>(|_, _, _| Ok(((), true)), true, &mut has_underscore)?;
        }

        let has_e = self.peek().is_some_and(|c| c == b'e' || c == b'E');
        // Handle 'e' and 'E'
        //
        // .5e1 = 5
        // 1e2 = 100
        // 1e+2 = 100
        // 1e-2 = 0.01
        if has_e {
            self.bump(1); // `e`/`E`

            let next = match self.peek() {
                Some(next) => next,
                None => {
                    let pos = self.cur_pos();
                    self.error(pos, SyntaxError::NumLitTerminatedWithExp)?
                }
            };

            if next == b'+' || next == b'-' {
                self.bump(1); // remove '+', '-'
            }

            let lazy_integer = self.read_number_no_dot_as_str::<10>()?;
            has_underscore |= lazy_integer.has_underscore;
        }

        let val = if has_dot || has_e {
            let raw = unsafe {
                // Safety: We got both start and end position from `self.input`
                self.input_slice_to_cur(start)
            };

            let raw = remove_underscore(raw, has_underscore);
            raw.parse().expect("failed to parse float literal")
        } else {
            let s = unsafe { self.input_slice(lazy_integer.start, lazy_integer.end) };
            let s = remove_underscore(s, has_underscore);
            parse_integer::<10>(&s)
        };

        self.ensure_not_ident()?;

        Ok(Either::Left(val))
    }

    fn read_int_u32<const RADIX: u8>(&mut self, len: u8) -> LexResult<Option<u32>> {
        let start = self.cur_pos();

        let mut count = 0;
        let v = self.read_digits::<_, Option<u32>, RADIX>(
            |opt: Option<u32>, radix, val| {
                count += 1;

                let total = opt
                    .unwrap_or_default()
                    .checked_mul(radix as u32)
                    .and_then(|v| v.checked_add(val))
                    .ok_or_else(|| {
                        let span = Span::new(start, start);
                        crate::error::Error::new(span, SyntaxError::InvalidUnicodeEscape)
                    })?;

                Ok((Some(total), count != len))
            },
            true,
            &mut false,
        )?;
        if len != 0 && count != len {
            Ok(None)
        } else {
            Ok(v)
        }
    }

    /// Returns `Left(value)` or `Right(BigInt)`
    fn read_radix_number<const RADIX: u8>(&mut self) -> LexResult<NumberEither<'a>> {
        debug_assert!(
            RADIX == 2 || RADIX == 8 || RADIX == 16,
            "radix should be one of 2, 8, 16, but got {RADIX}"
        );

        debug_assert_eq!(self.peek(), Some(b'0'));
        self.bump(1);

        debug_assert!(
            self.peek()
                .is_some_and(|c| matches!(c, b'b' | b'B' | b'o' | b'O' | b'x' | b'X'))
        );
        self.bump(1);

        let lazy_integer = self.read_number_no_dot_as_str::<RADIX>()?;
        let has_underscore = lazy_integer.has_underscore;

        let s = unsafe {
            // Safety: We got both start and end position from `self.input`
            self.input_slice_to_cur(lazy_integer.start)
        };
        if self.eat(b'n') {
            if s.starts_with('_') {
                return Err(Error::new(
                    Span::new(lazy_integer.start, lazy_integer.end),
                    SyntaxError::NumericSeparatorIsAllowedOnlyBetweenTwoDigits,
                ));
            }

            let Some(bigint_value) =
                bigint_atom_from_digits(s, RADIX as _, has_underscore, self.allocator)
            else {
                // just a fallback in case there is anything we did not catch
                return Err(Error::new(
                    Span::new(lazy_integer.start, lazy_integer.end),
                    SyntaxError::ExpectedDigit { radix: RADIX },
                ));
            };
            return Ok(Either::Right(bigint_value));
        }
        let s = remove_underscore(s, has_underscore);
        let val = parse_integer::<RADIX>(&s);

        self.ensure_not_ident()?;

        Ok(Either::Left(val))
    }

    /// Consume pending comments.
    ///
    /// This is called when the input is exhausted.
    #[cold]
    #[inline(never)]
    fn consume_pending_comments(&mut self) {
        let last = self.state.prev_hi;
        let start_pos = self.start_pos();
        if let (Some(comments), Some(comments_buffer)) =
            (self.comments.as_mut(), self.comments_buffer.as_mut())
        {
            // if the file had no tokens and no shebang, then treat any
            // comments in the leading comments buffer as leading.
            // Otherwise treat them as trailing.
            let kind = if last == start_pos {
                BufferedCommentKind::Leading
            } else {
                BufferedCommentKind::Trailing
            };
            // move the pending to the leading or trailing
            comments_buffer.pending_to_comment(kind, last);

            // now fill the user's passed in comments
            for comment in comments_buffer.take_comments() {
                match comment.kind {
                    BufferedCommentKind::Leading => comments
                        .leading
                        .entry(comment.pos)
                        .or_default()
                        .push(comment.comment),
                    BufferedCommentKind::Trailing => comments
                        .trailing
                        .entry(comment.pos)
                        .or_default()
                        .push(comment.comment),
                }
            }
        }
    }

    fn read_jsx_entity(&mut self) -> LexResult<(char, String)> {
        debug_assert!(self.syntax().jsx());

        fn from_code(s: &str, radix: u32) -> char {
            // TODO(kdy1): unwrap -> Err
            char::from_u32(u32::from_str_radix(s, radix).expect("failed to parse string as number"))
                .expect("failed to parse number as char")
        }

        fn is_hex(s: &str) -> bool {
            s.chars().all(|c| c.is_ascii_hexdigit())
        }

        fn is_dec(s: &str) -> bool {
            s.chars().all(|c| c.is_ascii_digit())
        }

        let mut s = SmartString::<LazyCompact>::default();

        debug_assert!(self.input().peek().is_some_and(|c| c == b'&'));
        self.bump(1);

        let start_pos = self.input().cur_pos();

        for _ in 0..10 {
            let c = match self.input().peek() {
                Some(c) => c,
                None => break,
            };
            self.bump(1);

            if c == b';' {
                if let Some(stripped) = s.strip_prefix('#') {
                    if stripped.starts_with('x') {
                        if is_hex(&s[2..]) {
                            let value = from_code(&s[2..], 16);

                            return Ok((value, format!("&{s};")));
                        }
                    } else if is_dec(stripped) {
                        let value = from_code(stripped, 10);

                        return Ok((value, format!("&{s};")));
                    }
                } else if let Some(entity) = xhtml(&s) {
                    return Ok((entity, format!("&{s};")));
                }

                break;
            }

            s.push(c as char)
        }

        unsafe {
            // Safety: start_pos is a valid position because we got it from self.input
            self.input_mut().reset_to(start_pos);
        }

        Ok(('&', "&".to_string()))
    }

    fn read_jsx_new_line(&mut self, normalize_crlf: bool) -> LexResult<Either<&'static str, char>> {
        debug_assert!(self.syntax().jsx());
        let ch = self.input().peek_char().unwrap();
        self.bump(1);

        let out = if ch == '\r' && self.input().peek() == Some(b'\n') {
            self.bump(1); // `\n`
            Either::Left(if normalize_crlf { "\n" } else { "\r\n" })
        } else {
            Either::Right(ch)
        };
        Ok(out)
    }

    fn read_jsx_str(&mut self, quote: u8) -> LexResult<Token> {
        debug_assert!(self.syntax().jsx());
        let start = self.input().cur_pos();
        self.bump(1); // `quote`
        let mut out = Wtf8Buf::new();
        let mut chunk_start = self.input().cur_pos();
        loop {
            let ch = match self.input().peek() {
                Some(c) => c,
                None => {
                    self.emit_error(start, SyntaxError::UnterminatedStrLit);
                    break;
                }
            };
            let cur_pos = self.input().cur_pos();
            if ch == b'\\' {
                let value = unsafe {
                    // Safety: We already checked for the range
                    self.input_slice_to_cur(chunk_start)
                };

                out.push_str(value);
                out.push_char('\\');

                self.bump(1);

                chunk_start = self.input().cur_pos();

                continue;
            }

            if ch == quote {
                break;
            }

            if ch == b'&' {
                let value = unsafe {
                    // Safety: We already checked for the range
                    self.input_slice_to_cur(chunk_start)
                };

                out.push_str(value);

                let jsx_entity = self.read_jsx_entity()?;

                out.push_char(jsx_entity.0);

                chunk_start = self.input().cur_pos();
            } else if ch.is_line_terminator() {
                let value = unsafe {
                    // Safety: We already checked for the range
                    self.input_slice_to_cur(chunk_start)
                };

                out.push_str(value);

                match self.read_jsx_new_line(false)? {
                    Either::Left(s) => {
                        out.push_str(s);
                    }
                    Either::Right(c) => {
                        out.push_char(c);
                    }
                }

                chunk_start = cur_pos + 1;
            } else {
                self.bump(1);
            }
        }
        let value = if out.is_empty() {
            // Fast path: We don't need to allocate
            self.wtf8_atom_from_slice(chunk_start, self.cur_pos())
        } else {
            let s = unsafe {
                // Safety: We already checked for the range
                self.input_slice_to_cur(chunk_start)
            };
            out.push_str(s);
            Wtf8Atom::new_in(out, self.allocator)
        };

        // it might be at the end of the file when
        // the string literal is unterminated
        if self.input().peek_3().is_some() {
            self.bump(1);
        }

        Ok(Token::str(value, self))
    }

    // Modified based on <https://github.com/oxc-project/oxc/blob/f0e1510b44efdb1b0d9a09f950181b0e4c435abe/crates/oxc_parser/src/lexer/unicode.rs#L237>
    /// Unicode code unit (`\uXXXX`).
    ///
    /// The opening `\u` must already have been consumed before calling this
    /// method.
    ///
    /// See background info on surrogate pairs:
    ///   * `https://mathiasbynens.be/notes/javascript-encoding#surrogate-formulae`
    ///   * `https://mathiasbynens.be/notes/javascript-identifiers-es6`
    fn read_unicode_code_unit(&mut self) -> LexResult<Option<UnicodeEscape>> {
        const MIN_HIGH: u32 = 0xd800;
        const MAX_HIGH: u32 = 0xdbff;
        const MIN_LOW: u32 = 0xdc00;
        const MAX_LOW: u32 = 0xdfff;

        let Some(high) = self.read_int_u32::<16>(4)? else {
            return Ok(None);
        };
        if let Some(ch) = char::from_u32(high) {
            return Ok(Some(UnicodeEscape::CodePoint(ch)));
        }

        // The first code unit of a surrogate pair is always in the range from 0xD800 to
        // 0xDBFF, and is called a high surrogate or a lead surrogate.
        // Note: `high` must be >= `MIN_HIGH`, otherwise `char::from_u32` would have
        // returned `Some`, and already exited.
        debug_assert!(high >= MIN_HIGH);
        let is_pair = high <= MAX_HIGH
            && self.input().peek() == Some(b'\\')
            && self.input().peek_2() == Some(b'u');
        if !is_pair {
            return Ok(Some(UnicodeEscape::LoneSurrogate(high)));
        }

        let before_second = self.input().cur_pos();

        // Bump `\u`
        self.bump(2);

        let Some(low) = self.read_int_u32::<16>(4)? else {
            return Ok(None);
        };

        // The second code unit of a surrogate pair is always in the range from 0xDC00
        // to 0xDFFF, and is called a low surrogate or a trail surrogate.
        // If this isn't a valid pair, rewind to before the 2nd, and return the first
        // only. The 2nd could be the first part of a valid pair.
        if !(MIN_LOW..=MAX_LOW).contains(&low) {
            unsafe {
                // Safety: state is valid position because we got it from cur_pos()
                self.input_mut().reset_to(before_second);
            }
            return Ok(Some(UnicodeEscape::LoneSurrogate(high)));
        }

        let code_point = pair_to_code_point(high, low);
        // SAFETY: `high` and `low` have been checked to be in ranges which always yield
        // a `code_point` which is a valid `char`
        let ch = unsafe { char::from_u32_unchecked(code_point) };
        Ok(Some(UnicodeEscape::SurrogatePair(ch)))
    }

    fn read_unicode_escape(&mut self) -> LexResult<UnicodeEscape> {
        debug_assert_eq!(self.peek(), Some(b'u'));

        let mut is_curly = false;

        self.bump(1); // 'u'

        if self.eat(b'{') {
            is_curly = true;
        }

        let state = self.input().cur_pos();
        let c = match self.read_int_u32::<16>(if is_curly { 0 } else { 4 }) {
            Ok(Some(val)) => {
                if 0x0010_ffff >= val {
                    char::from_u32(val)
                } else {
                    let start = self.cur_pos();

                    self.error(
                        start,
                        SyntaxError::BadCharacterEscapeSequence {
                            expected: if is_curly {
                                "1-6 hex characters in the range 0 to 10FFFF."
                            } else {
                                "4 hex characters"
                            },
                        },
                    )?
                }
            }
            _ => {
                let start = self.cur_pos();

                self.error(
                    start,
                    SyntaxError::BadCharacterEscapeSequence {
                        expected: if is_curly {
                            "1-6 hex characters"
                        } else {
                            "4 hex characters"
                        },
                    },
                )?
            }
        };

        match c {
            Some(c) => {
                if is_curly && !self.eat(b'}') {
                    self.error(state, SyntaxError::InvalidUnicodeEscape)?
                }

                Ok(UnicodeEscape::CodePoint(c))
            }
            _ => {
                unsafe {
                    // Safety: state is valid position because we got it from cur_pos()
                    self.input_mut().reset_to(state);
                }

                let Some(value) = self.read_unicode_code_unit()? else {
                    self.error(
                        state,
                        SyntaxError::BadCharacterEscapeSequence {
                            expected: if is_curly {
                                "1-6 hex characters"
                            } else {
                                "4 hex characters"
                            },
                        },
                    )?
                };

                if is_curly && !self.eat(b'}') {
                    self.error(state, SyntaxError::InvalidUnicodeEscape)?
                }

                Ok(value)
            }
        }
    }

    #[cold]
    fn read_shebang(&mut self) -> LexResult<Option<Atom<'a>>> {
        if self.input().peek() != Some(b'#') || self.input().peek_2() != Some(b'!') {
            return Ok(None);
        }
        self.bump(2); // `#!`

        let start_pos = self.cur_pos();
        while let Some(c) = self.input.peek_char() {
            if c.is_line_terminator() {
                break;
            }
            self.bump(c.len_utf8());
        }

        Ok(Some(self.atom_from_slice(start_pos, self.cur_pos())))
    }

    /// Read an escaped character for string literal.
    ///
    /// In template literal, we should preserve raw string.
    fn read_escaped_char(&mut self, in_template: bool) -> LexResult<Option<CodePoint>> {
        debug_assert_eq!(self.peek(), Some(b'\\'));

        let start = self.cur_pos();

        self.bump(1); // '\'

        let c = match self.input.peek_char() {
            Some(c) => c,
            None => self.error_span(pos_span(start), SyntaxError::InvalidStrEscape)?,
        };

        let c = match c {
            '\\' => '\\',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'b' => '\u{0008}',
            'v' => '\u{000b}',
            'f' => '\u{000c}',
            '\r' => {
                self.bump(1); // remove '\r'

                self.eat(b'\n');

                return Ok(None);
            }
            '\n' | '\u{2028}' | '\u{2029}' => {
                self.bump(c.len_utf8());

                return Ok(None);
            }

            // read hexadecimal escape sequences
            'x' => {
                self.bump(1); // 'x'

                match self.read_int_u32::<16>(2)? {
                    Some(val) => return Ok(CodePoint::from_u32(val)),
                    None => self.error(
                        start,
                        SyntaxError::BadCharacterEscapeSequence {
                            expected: "2 hex characters",
                        },
                    )?,
                }
            }

            // read unicode escape sequences
            'u' => match self.read_unicode_escape() {
                Ok(value) => {
                    return Ok(Some(value.into()));
                }
                Err(err) => self.error(start, err.into_kind())?,
            },

            // octal escape sequences
            '0'..='7' => {
                self.bump(1);

                let first_c = if c == '0' {
                    match self.peek() {
                        Some(next) if (next as char).is_digit(8) => c,
                        // \0 is not an octal literal nor decimal literal.
                        _ => return Ok(Some(CodePoint::from_char('\u{0000}'))),
                    }
                } else {
                    c
                };

                // TODO: Show template instead of strict mode
                if in_template {
                    self.error(start, SyntaxError::LegacyOctal)?
                }

                self.emit_strict_mode_error(start, SyntaxError::LegacyOctal);

                let mut value: u8 = first_c.to_digit(8).unwrap() as u8;

                macro_rules! one {
                    ($check:expr) => {{
                        let cur = self.peek();

                        match cur.and_then(|c| (c as char).to_digit(8)) {
                            Some(v) => {
                                value = if $check {
                                    let new_val = value
                                        .checked_mul(8)
                                        .and_then(|value| value.checked_add(v as u8));
                                    match new_val {
                                        Some(val) => val,
                                        None => return Ok(CodePoint::from_u32(value as u32)),
                                    }
                                } else {
                                    value * 8 + v as u8
                                };

                                self.bump(1);
                            }
                            _ => return Ok(CodePoint::from_u32(value as u32)),
                        }
                    }};
                }

                one!(false);
                one!(true);

                return Ok(CodePoint::from_u32(value as u32));
            }
            _ => c,
        };

        self.bump(c.len_utf8());
        Ok(CodePoint::from_u32(c as u32))
    }

    /// Expects current char to be '/'
    fn read_regexp(&mut self, start: u32) -> LexResult<Token> {
        unsafe {
            // Safety: start is valid position, and peek() is Some('/')
            self.input_mut().reset_to(start);
        }

        debug_assert_eq!(self.peek(), Some(b'/'));

        let start = self.cur_pos();

        self.bump(1); // bump '/'

        let (mut escaped, mut in_class) = (false, false);

        while let Some(c) = self.input.peek_char() {
            // This is ported from babel.
            // Seems like regexp literal cannot contain linebreak.
            if c.is_line_terminator() {
                let span = self.span(start);

                return Err(crate::error::Error::new(
                    span,
                    SyntaxError::UnterminatedRegExp,
                ));
            }

            if escaped {
                escaped = false;
            } else {
                match c {
                    '[' => in_class = true,
                    ']' if in_class => in_class = false,
                    // Terminates content part of regex literal
                    '/' if !in_class => break,
                    _ => {}
                }

                escaped = c == '\\';
            }

            self.bump(c.len_utf8());
        }

        let exp_end = self.cur_pos();

        // input is terminated without following `/`
        if !self.is(b'/') {
            let span = self.span(start);
            return Err(crate::error::Error::new(
                span,
                SyntaxError::UnterminatedRegExp,
            ));
        }

        self.bump(1); // '/'

        // Spec says "It is a Syntax Error if IdentifierPart contains a Unicode escape
        // sequence." TODO: check for escape

        // Need to use `read_word` because '\uXXXX' sequences are allowed
        // here (don't ask).
        // let flags_start = self.cur_pos();
        let flags = {
            match self.peek() {
                Some(c) if c.is_ident_start() => self.read_word_as_str_with().map(|(s, _)| s),
                _ => Ok(Atom::default()),
            }
        }?;

        if !flags.as_str().is_empty() {
            let mut flags_count = flags.as_str().chars().fold(
                FxHashMap::<char, usize>::default(),
                |mut map, flag| {
                    let key = match flag {
                        // https://tc39.es/ecma262/#sec-isvalidregularexpressionliteral
                        'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y' => flag,
                        _ => '\u{0000}', // special marker for unknown flags
                    };
                    map.entry(key).and_modify(|count| *count += 1).or_insert(1);
                    map
                },
            );

            if flags_count.remove(&'\u{0000}').is_some() {
                let span = self.span(start);
                self.emit_error_span(span, SyntaxError::UnknownRegExpFlags);
            }

            if let Some((flag, _)) = flags_count.iter().find(|(_, count)| **count > 1) {
                let span = self.span(start);
                self.emit_error_span(span, SyntaxError::DuplicatedRegExpFlags(*flag));
            }
        }

        Ok(Token::regexp(exp_end, self))
    }

    /// This method is optimized for texts without escape sequences.
    fn read_word_as_str_with(&mut self) -> LexResult<(Atom<'a>, bool)> {
        debug_assert!(self.peek().is_some());
        let slice_start = self.cur_pos();

        // Fast path: try to scan ASCII identifier using byte_search
        if let Some(c) = self.input().peek_ascii()
            && is_valid_ascii_start(c)
        {
            // Use byte_search to quickly scan to end of ASCII identifier
            let next_byte = byte_search! {
                lexer: self,
                table: NOT_ASCII_ID_CONTINUE_TABLE,
                start_at: 1,
                handle_eof: {
                    // Reached EOF, entire remainder is identifier
                    return Ok((self.atom_from_slice(slice_start, self.cur_pos()), false))
                },
            };

            // Check if we hit end of identifier or need to fall back to slow path
            if !next_byte.is_ascii() {
                // Hit Unicode character, fall back to slow path from current position
                return self.read_word_as_str_with_slow_path(slice_start);
            } else if next_byte == b'\\' {
                // Hit escape sequence, fall back to slow path from current position
                return self.read_word_as_str_with_slow_path(slice_start);
            } else {
                // Hit end of identifier (non-continue ASCII char)
                let s = self.atom_from_slice(slice_start, self.cur_pos());
                return Ok((s, false));
            }
        }

        // Fall back to slow path for non-ASCII start or complex cases
        self.read_word_as_str_with_slow_path(slice_start)
    }

    /// Slow path for identifier parsing that handles Unicode and escapes
    #[cold]
    fn read_word_as_str_with_slow_path(
        &mut self,
        mut slice_start: u32,
    ) -> LexResult<(Atom<'a>, bool)> {
        let mut first = true;
        let mut has_escape = false;

        let mut buf = String::new();
        loop {
            if let Some(c) = self.input().peek_ascii() {
                if is_valid_ascii_continue(c) {
                    self.bump(1);
                    continue;
                } else if first && is_valid_ascii_start(c) {
                    self.bump(1);
                    first = false;
                    continue;
                }

                // unicode escape
                if c == b'\\' {
                    first = false;
                    has_escape = true;
                    let start = self.cur_pos();
                    self.bump(1);

                    if !self.is(b'u') {
                        self.error_span(pos_span(start), SyntaxError::ExpectedUnicodeEscape)?
                    }

                    {
                        let end = self.input().cur_pos();
                        let s = unsafe {
                            // Safety: start and end are valid position because we got them from
                            // `self.input`
                            self.input_slice(slice_start, start)
                        };
                        buf.push_str(s);
                        unsafe {
                            // Safety: We got end from `self.input`
                            self.input_mut().reset_to(end);
                        }
                    }

                    let value = self.read_unicode_escape()?;

                    match value {
                        UnicodeEscape::CodePoint(ch) => {
                            let valid = if first {
                                ch.is_ident_start()
                            } else {
                                ch.is_ident_part()
                            };
                            if !valid {
                                self.emit_error(start, SyntaxError::InvalidIdentChar);
                            }
                            buf.push(ch);
                        }
                        UnicodeEscape::SurrogatePair(ch) => {
                            buf.push(ch);
                            self.emit_error(start, SyntaxError::InvalidIdentChar);
                        }
                        UnicodeEscape::LoneSurrogate(code_point) => {
                            buf.push_str(format!("\\u{code_point:04X}").as_str());
                            self.emit_error(start, SyntaxError::InvalidIdentChar);
                        }
                    };

                    slice_start = self.cur_pos();
                    continue;
                }

                // ASCII but not a valid identifier
                break;
            } else if let Some(c) = self.input().peek_char() {
                if is_valid_non_ascii_continue(c) {
                    self.bump(c.len_utf8());
                    continue;
                } else if first && is_valid_non_ascii_start(c) {
                    self.bump(c.len_utf8());
                    first = false;
                    continue;
                }
            }

            break;
        }

        let end = self.cur_pos();
        let value = if !has_escape {
            // Fast path: raw slice is enough if there's no escape.
            self.atom_from_slice(slice_start, end)
        } else {
            let s = unsafe {
                // Safety: slice_start and end are valid position because we got them from
                // `self.input`
                self.input_slice(slice_start, end)
            };

            buf.push_str(s);
            Atom::new_in(buf, self.allocator)
        };

        Ok((value, has_escape))
    }

    /// `#`
    fn read_token_number_sign(&mut self) -> LexResult<Token> {
        debug_assert!(self.peek().is_some_and(|c| c == b'#'));

        self.bump(1); // '#'

        // `#` can also be a part of shebangs, however they should have been
        // handled by `read_shebang()`
        debug_assert!(
            !self.input().is_start() || self.peek() != Some(b'!'),
            "#! should have already been handled by read_shebang()"
        );
        Ok(Token::Hash)
    }

    /// Read a token given `.`.
    ///
    /// This is extracted as a method to reduce size of `read_token`.
    fn read_token_dot(&mut self) -> LexResult<Token> {
        debug_assert!(self.peek().is_some_and(|c| c == b'.'));
        // Check for eof
        let next = match self.input().peek_2() {
            Some(next) => next,
            None => {
                self.bump(1); // '.'
                return Ok(Token::Dot);
            }
        };
        if next.is_ascii_digit() {
            return self.read_number::<true, false>().map(|v| match v {
                Left(value) => Token::num(value, self),
                Right(_) => unreachable!("read_number should not return bigint for leading dot"),
            });
        }

        self.bump(1); // 1st `.`

        if next == b'.' && self.input().peek_2() == Some(b'.') {
            self.bump(2); // 2nd `.` and 3rd `.`
            return Ok(Token::DotDotDot);
        }

        Ok(Token::Dot)
    }

    /// Read a token given `?`.
    ///
    /// This is extracted as a method to reduce size of `read_token`.
    fn read_token_question_mark(&mut self) -> LexResult<Token> {
        debug_assert!(self.peek().is_some_and(|c| c == b'?'));
        self.bump(1);
        if self.input_mut().eat_byte(b'?') {
            if self.input_mut().eat_byte(b'=') {
                Ok(Token::NullishEq)
            } else {
                Ok(Token::NullishCoalescing)
            }
        } else {
            Ok(Token::QuestionMark)
        }
    }

    /// Read a token given `:`.
    ///
    /// This is extracted as a method to reduce size of `read_token`.
    fn read_token_colon(&mut self) -> LexResult<Token> {
        debug_assert!(self.peek().is_some_and(|c| c == b':'));
        self.bump(1); // ':'
        Ok(Token::Colon)
    }

    /// Read a token given `0`.
    ///
    /// This is extracted as a method to reduce size of `read_token`.
    fn read_token_zero(&mut self) -> LexResult<Token> {
        debug_assert_eq!(self.peek(), Some(b'0'));
        let next = self.input().peek_2();

        let bigint = match next {
            Some(b'x') | Some(b'X') => self.read_radix_number::<16>(),
            Some(b'o') | Some(b'O') => self.read_radix_number::<8>(),
            Some(b'b') | Some(b'B') => self.read_radix_number::<2>(),
            _ => {
                return self.read_number::<false, true>().map(|v| match v {
                    Left(value) => Token::num(value, self),
                    Right(value) => Token::bigint(value, self),
                });
            }
        };

        bigint.map(|v| match v {
            Left(value) => Token::num(value, self),
            Right(value) => Token::bigint(value, self),
        })
    }

    /// Read a token given `|` or `&`.
    ///
    /// This is extracted as a method to reduce size of `read_token`.
    fn read_token_logical<const C: u8>(&mut self) -> LexResult<Token> {
        debug_assert!(C == b'|' || C == b'&');
        let is_bit_and = C == b'&';
        let had_line_break_before_last = self.state.had_line_break;
        let start = self.cur_pos();

        self.bump(1);
        let token = if is_bit_and {
            Token::Ampersand
        } else {
            Token::Pipe
        };

        // '|=', '&='
        if self.input_mut().eat_byte(b'=') {
            return Ok(if is_bit_and {
                Token::BitAndEq
            } else {
                debug_assert!(token == Token::Pipe);
                Token::BitOrEq
            });
        }

        // '||', '&&'
        if self.input().peek() == Some(C) {
            self.bump(1);
            if self.input().peek() == Some(b'=') {
                self.bump(1);
                return Ok(if is_bit_and {
                    Token::LogicalAndEq
                } else {
                    debug_assert!(token == Token::Pipe);
                    Token::LogicalOrEq
                });
            }

            // |||||||
            //   ^
            if had_line_break_before_last && !is_bit_and && self.is_str("||||| ") {
                let span = fixed_len_span(start, 7);
                self.emit_error_span(span, SyntaxError::TS1185);
                self.skip_line_comment(5);
                self.skip_space();
                return self.error_span(span, SyntaxError::TS1185);
            }

            return Ok(if is_bit_and {
                Token::LogicalAnd
            } else {
                debug_assert!(token == Token::Pipe);
                Token::LogicalOr
            });
        }

        Ok(token)
    }

    /// Read a token given `*` or `%`.
    ///
    /// This is extracted as a method to reduce size of `read_token`.
    fn read_token_mul_mod<const IS_MUL: bool>(&mut self) -> LexResult<Token> {
        debug_assert!(self.peek().is_some_and(|c| c == b'*' || c == b'%'));
        self.bump(1);
        let token = if IS_MUL {
            if self.input_mut().eat_byte(b'*') {
                // `**`
                Token::Exp
            } else {
                Token::Asterisk
            }
        } else {
            Token::Percent
        };

        Ok(if self.input_mut().eat_byte(b'=') {
            if token == Token::Asterisk {
                Token::MulEq
            } else if token == Token::Percent {
                Token::ModEq
            } else {
                debug_assert!(token == Token::Exp);
                Token::ExpEq
            }
        } else {
            token
        })
    }

    fn read_slash(&mut self) -> LexResult<Token> {
        debug_assert_eq!(self.peek(), Some(b'/'));
        self.bump(1); // '/'
        Ok(if self.eat(b'=') {
            Token::DivEq
        } else {
            Token::Slash
        })
    }

    /// This can be used if there's no keyword starting with the first
    /// character.
    fn read_ident_unknown(&mut self) -> LexResult<Token> {
        debug_assert!(self.peek().is_some());

        let (atom, has_escape) = self.read_word_as_str_with()?;
        let word = Token::unknown_ident(atom, self);

        if has_escape {
            self.update_token_flags(|flags| *flags |= TokenFlags::UNICODE);
        }

        Ok(word)
    }

    /// See https://tc39.github.io/ecma262/#sec-literals-string-literals
    // TODO: merge `read_str_lit` and `read_jsx_str`
    fn read_str_lit(&mut self) -> LexResult<Token> {
        debug_assert!(self.peek() == Some(b'\'') || self.peek() == Some(b'"'));
        let start = self.cur_pos();
        let quote = self.peek().unwrap();

        self.bump(1); // '"' or '\''

        let mut slice_start = self.input().cur_pos();

        let mut buf: Option<Wtf8Buf> = None;

        loop {
            let table = if quote == b'"' {
                &DOUBLE_QUOTE_STRING_END_TABLE
            } else {
                &SINGLE_QUOTE_STRING_END_TABLE
            };

            let fast_path_result = byte_search! {
                lexer: self,
                table: table,
                handle_eof: {
                    let value_end = self.cur_pos();
                    let s = self.wtf8_atom_from_slice(slice_start, value_end);

                    self.emit_error(start, SyntaxError::UnterminatedStrLit);

                    return Ok(Token::str(s, self));
                },
            };
            // dbg!(char::from_fast_path_result as u32);

            match fast_path_result {
                b'"' | b'\'' if fast_path_result == quote => {
                    let value_end = self.cur_pos();

                    let value = if let Some(mut buf) = buf.take() {
                        // `buf` only exist when there has escape.
                        debug_assert!(unsafe { self.input_slice(start, value_end).contains('\\') });
                        let s = unsafe {
                            // Safety: slice_start and value_end are valid position because we
                            // got them from `self.input`
                            self.input_slice(slice_start, value_end)
                        };
                        buf.push_str(s);
                        Wtf8Atom::new_in(buf, self.allocator)
                    } else {
                        self.wtf8_atom_from_slice(slice_start, value_end)
                    };

                    self.bump(1);

                    return Ok(Token::str(value, self));
                }
                b'\\' => {
                    let end = self.cur_pos();
                    let s = unsafe {
                        // Safety: start and end are valid position because we got them from
                        // `self.input`
                        self.input_slice(slice_start, end)
                    };

                    if buf.is_none() {
                        let mut builder = Wtf8Buf::new();
                        builder.push_str(s);
                        buf = Some(builder);
                    } else {
                        buf.as_mut().unwrap().push_str(s);
                    }

                    if let Some(escaped) = self.read_escaped_char(false)? {
                        buf.as_mut().unwrap().push(escaped);
                    }

                    slice_start = self.cur_pos();
                    continue;
                }
                b'\n' | b'\r' => {
                    let end = self.cur_pos();
                    let s = self.wtf8_atom_from_slice(slice_start, end);

                    self.emit_error(start, SyntaxError::UnterminatedStrLit);

                    return Ok(Token::str(s, self));
                }
                _ => self.bump(1),
            }
        }
    }

    fn read_keyword_with(&mut self, convert: fn(&str) -> Option<Token>) -> LexResult<Token> {
        debug_assert!(self.peek().is_some());

        let start = self.cur_pos();
        let (s, has_escape) = self.read_keyword_as_str_with()?;
        if let Some(word) = convert(s.as_str()) {
            // Note: ctx is store in lexer because of this error.
            // 'await' and 'yield' may have semantic of reserved word, which means lexer
            // should know context or parser should handle this error. Our approach to this
            // problem is former one.
            if has_escape && word.is_reserved(self.ctx()) {
                self.error(
                    start,
                    SyntaxError::EscapeInReservedWord {
                        word: s.to_string(),
                    },
                )
            } else {
                Ok(word)
            }
        } else {
            Ok(Token::unknown_ident(s, self))
        }
    }

    /// This is a performant version of [Lexer::read_word_as_str_with] for
    /// reading keywords. We should make sure the first byte is a valid
    /// ASCII.
    fn read_keyword_as_str_with(&mut self) -> LexResult<(Atom<'a>, bool)> {
        let slice_start = self.cur_pos();

        // Fast path: try to scan ASCII identifier using byte_search
        // Performance optimization: check if first char disqualifies as keyword
        // Use byte_search to quickly scan to end of ASCII identifier
        let next_byte = byte_search! {
            lexer: self,
            table: NOT_ASCII_ID_CONTINUE_TABLE,
            start_at: 1,
            handle_eof: {
                // Reached EOF, entire remainder is identifier
                let s = self.atom_from_slice(slice_start, self.cur_pos());
                return Ok((s, false));
            },
        };

        // Check if we hit end of identifier or need to fall back to slow path
        if !next_byte.is_ascii() || next_byte == b'\\' {
            // Hit Unicode character or escape sequence, fall back to slow path from current
            // position
            self.read_word_as_str_with_slow_path(slice_start)
        } else {
            // Hit end of identifier (non-continue ASCII char)
            let s = self.atom_from_slice(slice_start, self.cur_pos());
            Ok((s, false))
        }
    }
}

fn pos_span(p: u32) -> Span {
    Span::new(p, p)
}

fn fixed_len_span(p: u32, len: u32) -> Span {
    Span::new(p, p + len)
}
