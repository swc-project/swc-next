use std::hint::cold_path;

use memchr::{memchr, memchr2, memchr3};
use swc_next_allocator::Allocator;
use swc_next_allocator::vec::ArenaVec;
use swc_next_ecma_ast::{
    Comment, CommentKind, CommentPosition, Span, is_ident_continue_ascii, is_ident_start_ascii,
};

use swc_next_simd::memchr::memchr4;
use swc_next_simd::prelude::*;

use super::config::{NoTokenParserConfig, ParserConfig};
use super::token::{Token, TokenFlag, TokenTag, flag_mask};
use super::unicode_id::{can_continue_id, can_start_id};
use super::util::{
    code_point_at, is_multi_byte_space, parse_hex2, parse_octal, parse_unicode_escape,
    unicode_separator_len,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LexicalError {
    UnterminatedString,
    NonTerminatedTemplateLiteral,
    UnterminatedRegexLiteral,
    InvalidRegexLineTerminator,
    InvalidIdentifierStart,
    InvalidIdentifierContinue,
    UnterminatedMultiLineComment,
    InvalidUnicodeEscape,
    InvalidHexEscape,
    InvalidOctalEscape,
    InvalidOctalLiteralDigit,
    InvalidBinaryLiteral,
    InvalidHexLiteral,
    InvalidExponentPart,
    NumericSeparatorMisuse,
    ConsecutiveNumericSeparators,
    InvalidBigIntSuffix,
    IdentifierAfterNumericLiteral,
    InvalidUtf8,
    JsxIdentifierCannotContainEscapes,
    JsxIdentifierCannotStartWithBackslash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum LexerMode {
    #[default]
    Normal,
    JsxTag,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandleTokenMode {
    Append,
    Replace,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LexerState {
    pub token_flags: u8,
    /// Tag of the token before the one being read, used only by the trivia
    /// scanner. Fast paths do not update it; [`Lexer::next_token`] takes it
    /// from its target instead. An `Eof` target is a placeholder unless
    /// [`Lexer::handle_token`] already recorded `Eof` here.
    previous_token_tag: Option<TokenTag>,
    /// End offset of the most recently consumed token.
    pub(super) prev_token_end: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hashbang {
    pub start: u32,
    pub len: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegexResult {
    pub span: Span,
    pub pattern: Span,
    pub flags: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EscapeContext {
    String,
    Template,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConsumeUnicodeContext {
    IdentifierStart,
    IdentifierContinue,
    Normal,
}

#[derive(Clone, Debug)]
pub struct Lexer<'a, C: ParserConfig = NoTokenParserConfig> {
    source: &'a [u8],

    /// Whether Annex B HTML-like comments are recognized.
    allow_html_like_comments: bool,
    collect_comments: bool,
    pub(crate) hashbang: Option<Hashbang>,

    pub(crate) config: C,
    pub(crate) tokens: ArenaVec<'a, Token>,
    pub(crate) comments: ArenaVec<'a, Comment>,
    /// Annex B comments which become errors if an unambiguous program resolves
    /// to a module.
    pub(crate) module_mode_errors: ArenaVec<'a, Span>,
    pub(crate) processed_comments: usize,

    pub(crate) cursor: u32,
    pub(crate) state: LexerState,
    pub(crate) mode: LexerMode,
}

impl<'a, C: ParserConfig> Lexer<'a, C> {
    pub fn init(
        allocator: &'a Allocator,
        source: &'a str,
        allow_html_like_comments: bool,
        collect_comments: bool,
        config: C,
    ) -> Self {
        debug_assert!(u32::try_from(source.len()).is_ok());
        // Large generated files commonly contain enough comments for repeated
        // growth to show up in parser profiles. Keep the estimate proportional
        // to the source while bounding the reservation to a small fraction of
        // its size. Below 2 MiB, the bounded growth cost is lower than the
        // up-front allocation on comment-light sources.
        let comment_capacity = if collect_comments && source.len() >= 2 * 1024 * 1024 {
            source.len() / 256
        } else {
            0
        };

        let tokens = if config.tokens() {
            ArenaVec::with_capacity_in(source.len() + 1, allocator)
        } else {
            ArenaVec::new_in(allocator)
        };

        let mut lexer = Self {
            config,
            comments: ArenaVec::with_capacity_in(comment_capacity, allocator),
            module_mode_errors: ArenaVec::new_in(allocator),
            processed_comments: 0,
            tokens,
            collect_comments,
            source: source.as_bytes(),
            cursor: 0,
            allow_html_like_comments,
            hashbang: None,
            state: LexerState::default(),
            mode: LexerMode::Normal,
        };
        lexer.skip_hashbang();
        if lexer.hashbang.is_some() {
            lexer.state.prev_token_end = lexer.cursor;
        }
        lexer
    }

    fn skip_hashbang(&mut self) {
        debug_assert_eq!(self.cursor, 0);
        if self.source.len() >= 2 && self.source[0] == b'#' && self.source[1] == b'!' {
            let mut end = 2u32;
            // HashbangComment uses SingleLineCommentChars, so all four
            // ECMAScript line terminators end it: LF, CR, LS, and PS.
            while (end as usize) < self.source.len() {
                let byte = self.source[end as usize];
                if byte == b'\n'
                    || byte == b'\r'
                    || (byte == 0xE2 && unicode_separator_len(self.source, end as usize) > 0)
                {
                    break;
                }
                end += 1;
            }
            self.hashbang = Some(Hashbang {
                start: 2,
                len: (end - 2) as u16,
            });
            self.cursor = end;
        }
    }

    /// Reads the next token into `target`. `target` must hold the last token
    /// read, or an `Eof` placeholder before the first token.
    #[inline]
    pub fn next_token(&mut self, target: &mut Token) -> Result<(), LexicalError> {
        self.read_next_token(target)?;
        self.handle_token(*target, HandleTokenMode::Append);
        Ok(())
    }

    /// Reads common tokens without entering the full trivia and token scanner.
    /// On failure, the cursor is unchanged so trivia keeps its original context.
    ///
    /// Token flags are kept in a local instead of [`LexerState::token_flags`],
    /// which stays clear between tokens.
    #[inline]
    pub(crate) fn try_next_token(&mut self, target: &mut Token) -> bool {
        if self.mode != LexerMode::Normal {
            return self.try_next_jsx_tag_token(target);
        }

        debug_assert_eq!(self.state.token_flags, 0);
        let mut start = self.cursor;
        let Some(mut byte) = self.source.get(start as usize).copied() else {
            return false;
        };
        if byte == b' ' {
            start += 1;
            let Some(next) = self.source.get(start as usize).copied() else {
                return false;
            };
            byte = next;
        } else if byte == b'\n' {
            let Some((next_start, next)) = self.skip_line_feed_indentation(start) else {
                return false;
            };
            // Every scanner below leaves the cursor unchanged on failure, so a line
            // feed is rescanned by the trivia scanner with its original context.
            return self.try_next_token_at::<true>(target, next_start, next);
        }

        self.try_next_token_at::<false>(target, start, byte)
    }

    /// Reads a fast-path token at `start`. `LINE_BREAK` is a constant so each
    /// copy builds its token flags without keeping them in a register.
    #[inline(always)]
    fn try_next_token_at<const LINE_BREAK: bool>(
        &mut self,
        target: &mut Token,
        start: u32,
        byte: u8,
    ) -> bool {
        let flags = fast_token_flags(LINE_BREAK);
        match FAST_CLASS[byte as usize] {
            FastClass::Identifier => return self.try_identifier_token::<LINE_BREAK>(target, start),
            FastClass::Punctuation => {
                self.cursor = start + 1;
                *target = Token::new(start, start + 1, SIMPLE_PUNCT_TAG[byte as usize], flags);
            }
            FastClass::Digit => {
                if !self.scan_decimal_integer(start, byte, flags, target) {
                    return false;
                }
            }
            FastClass::Dot => {
                if self
                    .source
                    .get(start as usize + 1)
                    .is_some_and(|byte| *byte == b'.' || byte.is_ascii_digit())
                {
                    return false;
                }
                self.cursor = start + 1;
                *target = Token::new(start, start + 1, TokenTag::Dot, flags);
            }
            FastClass::Operator => return self.try_operator_token::<LINE_BREAK>(target, start),
            FastClass::Slow => return false,
        }
        self.handle_fast_token(*target);
        true
    }

    /// Reads an ASCII identifier or keyword at `start`. Identifier and operator scans need more
    /// registers than the other fast paths, so they are tail-called to keep callee-saved register
    /// saves out of [`Self::try_next_token`].
    #[inline(never)]
    fn try_identifier_token<const LINE_BREAK: bool>(
        &mut self,
        target: &mut Token,
        start: u32,
    ) -> bool {
        let flags = fast_token_flags(LINE_BREAK);
        if !self.scan_ascii_identifier(start, flags, target) {
            return false;
        }
        self.handle_fast_token(*target);
        true
    }

    /// Reads an operator at `start`. See [`Self::try_identifier_token`].
    #[inline(never)]
    fn try_operator_token<const LINE_BREAK: bool>(
        &mut self,
        target: &mut Token,
        start: u32,
    ) -> bool {
        // HTML comments still need the trivia scanner.
        if self.source[start as usize..].starts_with(b"<!--") {
            return false;
        }
        self.cursor = start;
        self.state.token_flags = fast_token_flags(LINE_BREAK);
        *target = self.scan_punctuation();
        self.handle_fast_token(*target);
        true
    }

    /// Reads common tokens inside a JSX tag: names, `=` before a value, `>`,
    /// `/>`, `{`, and attribute strings. On failure, the cursor is unchanged.
    #[inline(never)]
    fn try_next_jsx_tag_token(&mut self, target: &mut Token) -> bool {
        debug_assert_eq!(self.mode, LexerMode::JsxTag);
        let src = self.source;
        let mut start = self.cursor;
        let Some(mut byte) = src.get(start as usize).copied() else {
            return false;
        };
        while matches!(byte, b' ' | b'\t') {
            start += 1;
            let Some(&next) = src.get(start as usize) else {
                return false;
            };
            byte = next;
        }
        let line_break = byte == b'\n';
        if line_break {
            let Some((next_start, next)) = self.skip_line_feed_indentation(start) else {
                return false;
            };
            start = next_start;
            byte = next;
        }

        let start = start as usize;
        let (tag, end) = match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                // JSX names may contain hyphens between identifier parts.
                let Some(mut end) = scan_ascii_identifier_continue(src, start as u32 + 1) else {
                    return false;
                };
                while src.get(end as usize) == Some(&b'-') {
                    let Some(next) = scan_ascii_identifier_continue(src, end + 1) else {
                        return false;
                    };
                    end = next;
                }
                (TokenTag::JsxIdentifier, end as usize)
            }
            b'>' => (TokenTag::GreaterThan, start + 1),
            b'{' => (TokenTag::LeftBrace, start + 1),
            b'=' if matches!(src.get(start + 1), Some(b'{' | b'"' | b'\'')) => {
                (TokenTag::Assign, start + 1)
            }
            b'/' if src.get(start + 1) == Some(&b'>') => (TokenTag::Slash, start + 1),
            b'"' | b'\'' => {
                let Some(offset) = memchr(byte, &src[start + 1..]) else {
                    return false;
                };
                (TokenTag::StringLiteral, start + offset + 2)
            }
            _ => return false,
        };

        self.clear_token_flags();
        if line_break {
            self.set_token_flag(TokenFlag::LineTerminatorBefore);
        }
        self.cursor = end as u32;
        *target = self.create_token(tag, start as u32, end as u32);
        self.handle_fast_token(*target);
        true
    }

    /// Skips a line feed, blank lines, and space or tab indentation. Returns the
    /// position and value of the next byte, or `None` at the end of the source.
    #[inline(always)]
    fn skip_line_feed_indentation(&self, line_feed: u32) -> Option<(u32, u8)> {
        let mut start = line_feed as usize + 1;
        if self.source.get(start) == Some(&b' ') {
            // Skip a run of spaces eight bytes at a time. The first byte that differs from a
            // space ends the run, wherever it falls in the word.
            while let Some(word) = self.source.get(start..start + 8) {
                let differing =
                    u64::from_le_bytes(word.try_into().unwrap()) ^ 0x2020_2020_2020_2020;
                start += (differing.trailing_zeros() / 8) as usize;
                if differing != 0 {
                    break;
                }
            }
        }
        while matches!(self.source.get(start), Some(b' ' | b'\t' | b'\n')) {
            start += 1;
        }
        let byte = *self.source.get(start)?;
        Some((start as u32, byte))
    }

    /// Scans an ASCII identifier or keyword and builds its token with `flags`.
    #[inline(always)]
    fn scan_ascii_identifier(&mut self, start: u32, flags: u8, target: &mut Token) -> bool {
        let Some(pos) = scan_ascii_identifier_continue(self.source, start + 1) else {
            return false;
        };
        // SAFETY: `start` indexes the identifier start byte, and the scan returns a position
        // after it that is at most `source.len()`.
        let lexeme = unsafe { self.source.get_unchecked(start as usize..pos as usize) };
        self.cursor = pos;
        *target = Token::new(start, pos, get_keyword_type(lexeme), flags);
        true
    }

    /// Scans a decimal integer without a fraction, exponent, separator,
    /// suffix, or legacy octal digits. Other numbers use [`Self::scan_number`].
    /// `byte` is the digit at `start`.
    #[inline(always)]
    fn scan_decimal_integer(
        &mut self,
        start: u32,
        byte: u8,
        flags: u8,
        target: &mut Token,
    ) -> bool {
        debug_assert_eq!(self.source[start as usize], byte);
        let mut end = start as usize + 1;
        if byte != b'0' {
            while self.source.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
        }
        if let Some(&next) = self.source.get(end)
            && (ASCII_IDENTIFIER_CLASS[next as usize] != IdentifierClass::End || next == b'.')
        {
            return false;
        }
        self.cursor = end as u32;
        *target = Token::new(start, end as u32, TokenTag::NumericLiteral, flags);
        true
    }

    #[inline]
    fn read_next_token(&mut self, target: &mut Token) -> Result<(), LexicalError> {
        debug_assert!((self.cursor as usize) <= self.source.len());
        if target.tag() != TokenTag::Eof {
            self.state.previous_token_tag = Some(target.tag());
        }
        self.clear_token_flags();
        self.skip_ws_and_comments()?;

        if (self.cursor as usize) >= self.source.len() {
            *target = self.create_token(TokenTag::Eof, self.cursor, self.cursor);
            return Ok(());
        }

        let current_char = self.source[self.cursor as usize];
        if is_ident_start_ascii(current_char) && self.mode == LexerMode::Normal {
            if self.scan_ascii_identifier(self.cursor, self.state.token_flags, target) {
                self.clear_token_flags();
            } else {
                *target = self.scan_identifier_or_keyword()?;
            }
            return Ok(());
        }

        let token = match current_char {
            b'+' | b'*' | b'-' | b'!' | b'<' | b'>' | b'=' | b'|' | b'&' | b'^' | b'%' | b'/'
            | b'?' => self.scan_punctuation(),
            b'.' => self.scan_dot()?,
            b'0'..=b'9' => self.scan_number()?,
            b'"' | b'\'' => self.scan_string()?,
            b'`' => self.scan_template_literal()?,
            b'~' | b'(' | b')' | b'{' | b'[' | b']' | b';' | b',' | b':' | b'@' => {
                self.scan_simple_punctuation()
            }
            b'}' => self.handle_right_brace(),
            _ => self.scan_identifier_or_keyword()?,
        };
        *target = token;
        Ok(())
    }

    // Make sure `mode` is always inlined to eliminate the branch.
    #[inline(always)]
    pub(crate) fn handle_token(&mut self, token: Token, mode: HandleTokenMode) {
        if self.config.tokens() {
            match mode {
                HandleTokenMode::Append => self.tokens.push(token),
                HandleTokenMode::Replace => {
                    if let Some(last_token) = self.tokens.last_mut() {
                        *last_token = token;
                    }
                }
            }
        }

        if self.processed_comments < self.comments.len() {
            self.attach_pending_comments(token);
        }
        self.state.previous_token_tag = Some(token.tag());
    }

    /// Records a token read by a fast path. Fast paths never skip comments,
    /// and the scanner that does attaches its comments to the token it reads.
    #[inline(always)]
    fn handle_fast_token(&mut self, token: Token) {
        debug_assert_eq!(self.processed_comments, self.comments.len());
        if self.config.tokens() {
            self.tokens.push(token);
        }
    }

    /// Restores a compound token removed by a failed speculative rescan.
    #[inline]
    pub(crate) fn rewrite_last_collected_token(&mut self, token: Token) {
        if self.config.tokens() {
            if let Some(last) = self.tokens.last_mut() {
                debug_assert_eq!(last.start(), token.start());
                *last = token;
            } else {
                debug_assert!(!self.config.tokens());
            }
        }
    }

    #[inline]
    pub(crate) fn rewind_module_mode_errors(&mut self, cursor: u32) {
        if self
            .module_mode_errors
            .last()
            .is_some_and(|span| span.start >= cursor)
        {
            let len = self
                .module_mode_errors
                .partition_point(|span| span.start < cursor);
            self.module_mode_errors.truncate(len);
        }
    }

    #[inline]
    fn scan_simple_punctuation(&mut self) -> Token {
        debug_assert!((self.cursor as usize) < self.source.len());
        let start = self.cursor;
        let tag = SIMPLE_PUNCT_TAG[self.source[self.cursor as usize] as usize];
        debug_assert_ne!(tag, TokenTag::Eof);
        self.cursor += 1;
        self.create_token(tag, start, self.cursor)
    }

    fn handle_right_brace(&mut self) -> Token {
        debug_assert_eq!(self.source[self.cursor as usize], b'}');
        let start = self.cursor;
        self.cursor += 1;
        self.create_token(TokenTag::RightBrace, start, self.cursor)
    }

    fn scan_dot(&mut self) -> Result<Token, LexicalError> {
        debug_assert_eq!(self.source[self.cursor as usize], b'.');
        let start = self.cursor;
        let c1 = self.peek(1);
        let c2 = self.peek(2);
        if c1.is_ascii_digit() {
            return self.scan_number();
        }
        if c1 == b'.' && c2 == b'.' {
            return Ok(self.punc_token(3, TokenTag::Spread, start));
        }
        Ok(self.punc_token(1, TokenTag::Dot, start))
    }

    fn scan_string(&mut self) -> Result<Token, LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        let start = self.cursor;
        let quote = self.source[start as usize];
        debug_assert!(quote == b'"' || quote == b'\'');
        let mut pos = self.cursor + 1;

        if self.mode == LexerMode::Normal {
            while (pos as usize) < self.source.len() {
                let rest = &self.source[pos as usize..];
                let Some(offset) = memchr4(quote, b'\\', b'\n', b'\r', rest) else {
                    pos = self.source.len() as u32;
                    break;
                };
                pos += offset as u32;
                let byte = self.source[pos as usize];
                if byte == quote {
                    pos += 1;
                    self.cursor = pos;
                    return Ok(self.create_token(TokenTag::StringLiteral, start, pos));
                }
                self.cursor = pos;
                if byte == b'\\' {
                    self.consume_escape(EscapeContext::String)?;
                    pos = self.cursor;
                } else {
                    return Err(LexicalError::UnterminatedString);
                }
            }
        } else {
            if let Some(offset) = memchr(quote, &self.source[pos as usize..]) {
                pos += offset as u32 + 1;
                self.cursor = pos;
                return Ok(self.create_token(TokenTag::StringLiteral, start, pos));
            }
            pos = self.source.len() as u32;
        }

        self.cursor = pos;
        Err(LexicalError::UnterminatedString)
    }

    fn scan_template_literal(&mut self) -> Result<Token, LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        debug_assert_eq!(self.source[self.cursor as usize], b'`');
        let start = self.cursor;
        self.cursor += 1;

        self.scan_template_segment(
            start,
            TokenTag::NoSubstitutionTemplate,
            TokenTag::TemplateHead,
        )
    }

    fn scan_template_segment(
        &mut self,
        start: u32,
        tail_tag: TokenTag,
        middle_tag: TokenTag,
    ) -> Result<Token, LexicalError> {
        loop {
            let cursor = self.cursor as usize;
            let source = &self.source[cursor..];
            let special = memchr3(b'\\', b'`', b'$', source);
            let plain_len = special.unwrap_or(source.len());
            let has_carriage_return = !self.has_token_flag(TokenFlag::Escaped)
                && memchr(b'\r', &source[..plain_len]).is_some();

            self.cursor += plain_len as u32;
            if has_carriage_return {
                self.set_token_flag(TokenFlag::Escaped);
            }

            let Some(_) = special else {
                return Err(LexicalError::NonTerminatedTemplateLiteral);
            };

            match self.source[self.cursor as usize] {
                b'\\' => self.consume_escape(EscapeContext::Template)?,
                b'`' => {
                    self.cursor += 1;
                    return Ok(self.create_token(tail_tag, start, self.cursor));
                }
                b'$' if self.peek(1) == b'{' => {
                    self.cursor += 2;
                    return Ok(self.create_token(middle_tag, start, self.cursor));
                }
                b'$' => self.cursor += 1,
                _ => unreachable!(),
            }
        }
    }

    // Inline `context` to eliminate the branch
    #[inline(always)]
    fn consume_escape(&mut self, context: EscapeContext) -> Result<(), LexicalError> {
        if matches!(
            self.source.get(self.cursor as usize + 1),
            Some(b'\\' | b'"' | b'\'' | b'n' | b'r' | b't' | b'b' | b'f' | b'v')
        ) {
            self.set_token_flag(TokenFlag::Escaped);
            self.cursor += 2;
            return Ok(());
        }
        if context == EscapeContext::Template {
            match self.consume_escape_impl(context) {
                Ok(()) => Ok(()),
                Err(_) => {
                    self.set_token_flag(TokenFlag::InvalidEscape);
                    if (self.cursor as usize) < self.source.len() {
                        self.cursor += 1;
                    }
                    Ok(())
                }
            }
        } else {
            self.consume_escape_impl(context)
        }
    }

    fn consume_escape_impl(&mut self, context: EscapeContext) -> Result<(), LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        debug_assert_eq!(self.source[self.cursor as usize], b'\\');
        self.cursor += 1;
        self.set_token_flag(TokenFlag::Escaped);

        if (self.cursor as usize) >= self.source.len() {
            return Err(LexicalError::UnterminatedString);
        }

        match self.source[self.cursor as usize] {
            b'0' => {
                let c1 = self.peek(1);
                if !c1.is_ascii_digit() {
                    self.cursor += 1;
                    return Ok(());
                }
                if context == EscapeContext::Template {
                    return Err(LexicalError::InvalidOctalEscape);
                }
                self.consume_octal()
            }
            b'x' => self.consume_hex(),
            b'u' => {
                let cp = self.consume_unicode_escape(ConsumeUnicodeContext::Normal)?;
                if is_surrogate_codepoint(cp) {
                    let is_high_paired = is_utf16_high_surrogate(cp)
                        && self.peek(0) == b'\\'
                        && self.peek(1) == b'u';
                    if is_high_paired {
                        self.cursor += 1;
                        let next_cp = self.consume_unicode_escape(ConsumeUnicodeContext::Normal)?;
                        if !(0xDC00..=0xDFFF).contains(&next_cp) {
                            self.set_token_flag(TokenFlag::LoneSurrogates);
                        }
                    } else {
                        self.set_token_flag(TokenFlag::LoneSurrogates);
                    }
                }
                Ok(())
            }
            b'1'..=b'7' => {
                if context == EscapeContext::Template {
                    return Err(LexicalError::InvalidOctalEscape);
                }
                self.consume_octal()
            }
            b'8' | b'9' => {
                if context == EscapeContext::Template {
                    return Err(LexicalError::InvalidOctalEscape);
                }
                self.cursor += 1;
                Ok(())
            }
            b'\n' => {
                self.cursor += 1;
                Ok(())
            }
            b'\r' => {
                self.cursor += 1;
                if (self.cursor as usize) < self.source.len()
                    && self.source[self.cursor as usize] == b'\n'
                {
                    self.cursor += 1;
                }
                Ok(())
            }
            _ => {
                let us_len = unicode_separator_len(self.source, self.cursor as usize);
                if us_len > 0 {
                    self.cursor += u32::from(us_len);
                    return Ok(());
                }
                self.cursor += 1;
                Ok(())
            }
        }
    }

    fn consume_octal(&mut self) -> Result<(), LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        let result = parse_octal(self.source, self.cursor as usize);
        if result.end == self.cursor as usize {
            return Err(LexicalError::InvalidOctalEscape);
        }
        self.cursor = result.end as u32;
        Ok(())
    }

    fn consume_hex(&mut self) -> Result<(), LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        if let Some(parsed) = parse_hex2(self.source, self.cursor as usize + 1) {
            self.cursor = parsed.end as u32;
            Ok(())
        } else {
            Err(LexicalError::InvalidHexEscape)
        }
    }

    // Inline `consume_unicode_escape` to eliminate the branches
    #[inline(always)]
    fn consume_unicode_escape(
        &mut self,
        context: ConsumeUnicodeContext,
    ) -> Result<u32, LexicalError> {
        if (self.cursor as usize) >= self.source.len() || self.source[self.cursor as usize] != b'u'
        {
            return Err(LexicalError::InvalidUnicodeEscape);
        }
        self.set_token_flag(TokenFlag::Escaped);
        let parsed = parse_unicode_escape(self.source, self.cursor as usize + 1)
            .ok_or(LexicalError::InvalidUnicodeEscape)?;
        match context {
            ConsumeUnicodeContext::IdentifierStart => {
                if !can_start_id(parsed.value.to_u32()) {
                    return Err(LexicalError::InvalidIdentifierStart);
                }
            }
            ConsumeUnicodeContext::IdentifierContinue => {
                if !can_continue_id(parsed.value.to_u32()) {
                    return Err(LexicalError::InvalidIdentifierContinue);
                }
            }
            ConsumeUnicodeContext::Normal => {}
        }
        self.cursor = parsed.end as u32;
        Ok(parsed.value.to_u32())
    }

    fn scan_number(&mut self) -> Result<Token, LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        let lead = self.source[self.cursor as usize];
        debug_assert!(lead == b'.' || lead.is_ascii_digit());

        let start = self.cursor;
        let mut tag = TokenTag::NumericLiteral;
        let mut has_decimal_or_exponent = false;
        let mut is_leading_zero = false;

        if self.peek(0) == b'0' {
            self.cursor += 1;

            match self.peek(0) {
                b'x' | b'X' => {
                    tag = TokenTag::HexLiteral;
                    self.cursor += 1;
                    let digits_start = self.cursor;
                    self.consume_hex_digits()?;
                    if self.cursor == digits_start {
                        return Err(LexicalError::InvalidHexLiteral);
                    }
                }
                b'o' | b'O' => {
                    tag = TokenTag::OctalLiteral;
                    self.cursor += 1;
                    let digits_start = self.cursor;
                    self.consume_octal_digits()?;
                    if self.cursor == digits_start {
                        return Err(LexicalError::InvalidOctalLiteralDigit);
                    }
                }
                b'b' | b'B' => {
                    tag = TokenTag::BinaryLiteral;
                    self.cursor += 1;
                    let digits_start = self.cursor;
                    self.consume_binary_digits()?;
                    if self.cursor == digits_start {
                        return Err(LexicalError::InvalidBinaryLiteral);
                    }
                }
                b'0'..=b'9' => {
                    is_leading_zero = true;
                    let mut is_legacy_octal = true;
                    while (self.cursor as usize) < self.source.len() {
                        let c = self.source[self.cursor as usize];
                        if c.is_ascii_digit() {
                            if c >= b'8' {
                                is_legacy_octal = false;
                            }
                            self.cursor += 1;
                        } else {
                            break;
                        }
                    }
                    tag = if is_legacy_octal {
                        TokenTag::OctalLiteral
                    } else {
                        TokenTag::NumericLiteral
                    };
                }
                _ => {}
            }
        } else {
            self.consume_decimal_digits()?;
        }

        if tag == TokenTag::NumericLiteral && self.peek(0) == b'.' {
            let next = self.peek(1);
            if next == b'_' {
                return Err(LexicalError::NumericSeparatorMisuse);
            }
            self.cursor += 1;
            has_decimal_or_exponent = true;
            if next.is_ascii_digit() {
                self.consume_decimal_digits()?;
            }
        }

        if tag == TokenTag::NumericLiteral {
            let exp = self.peek(0);
            if exp == b'e' || exp == b'E' {
                has_decimal_or_exponent = true;
                self.consume_exponent()?;
            }
        }

        if !is_leading_zero && self.peek(0) == b'n' {
            if has_decimal_or_exponent {
                return Err(LexicalError::InvalidBigIntSuffix);
            }
            self.cursor += 1;
            tag = TokenTag::BigIntLiteral;
        }

        let c = self.peek(0);
        if is_ident_start_ascii(c) || c == b'\\' {
            return Err(LexicalError::IdentifierAfterNumericLiteral);
        }
        debug_assert!(self.cursor > start);
        Ok(self.create_token(tag, start, self.cursor))
    }

    fn consume_digits<F>(&mut self, is_valid_digit: F) -> Result<(), LexicalError>
    where
        F: Fn(u8) -> bool,
    {
        debug_assert!((self.cursor as usize) <= self.source.len());

        if (self.cursor as usize) < self.source.len() && self.source[self.cursor as usize] == b'_' {
            return Err(LexicalError::NumericSeparatorMisuse);
        }

        let mut last_was_separator = false;

        while (self.cursor as usize) < self.source.len() {
            let c = self.source[self.cursor as usize];
            if is_valid_digit(c) {
                self.cursor += 1;
                last_was_separator = false;
            } else if c == b'_' {
                if last_was_separator {
                    return Err(LexicalError::ConsecutiveNumericSeparators);
                }
                self.cursor += 1;
                last_was_separator = true;
            } else {
                break;
            }
        }

        if last_was_separator {
            return Err(LexicalError::NumericSeparatorMisuse);
        }
        Ok(())
    }

    fn consume_decimal_digits(&mut self) -> Result<(), LexicalError> {
        self.consume_digits(|c| c.is_ascii_digit())
    }

    fn consume_hex_digits(&mut self) -> Result<(), LexicalError> {
        self.consume_digits(|c| c.is_ascii_hexdigit())
    }

    fn consume_octal_digits(&mut self) -> Result<(), LexicalError> {
        self.consume_digits(super::util::is_octal_digit)
    }

    fn consume_binary_digits(&mut self) -> Result<(), LexicalError> {
        self.consume_digits(|c| c == b'0' || c == b'1')
    }

    fn consume_exponent(&mut self) -> Result<(), LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        debug_assert!(
            self.source[self.cursor as usize] == b'e' || self.source[self.cursor as usize] == b'E'
        );

        self.cursor += 1;

        let c = self.peek(0);
        if c == b'+' || c == b'-' {
            self.cursor += 1;
        }

        let exp_start = self.cursor;
        self.consume_decimal_digits()?;

        if self.cursor == exp_start {
            return Err(LexicalError::InvalidExponentPart);
        }
        Ok(())
    }

    fn scan_identifier_body(&mut self, is_jsx_tag: bool) -> Result<bool, LexicalError> {
        debug_assert!((self.cursor as usize) <= self.source.len());

        let mut has_escape = false;
        let src = self.source;
        let mut pos = self.cursor as usize;
        let entry_pos = pos;

        loop {
            while pos < src.len()
                && if is_jsx_tag {
                    let byte = src[pos];
                    is_ident_continue_ascii(byte) || byte == b'-'
                } else {
                    is_ident_continue_ascii(src[pos])
                }
            {
                pos += 1;
            }

            if pos >= src.len() {
                break;
            }

            let c = src[pos];

            if c == b'\\' {
                if is_jsx_tag {
                    self.cursor = pos as u32;
                    return Err(LexicalError::JsxIdentifierCannotContainEscapes);
                }

                has_escape = true;
                self.cursor = pos as u32 + 1;
                self.consume_unicode_escape(ConsumeUnicodeContext::IdentifierContinue)?;

                pos = self.cursor as usize;
                continue;
            }

            if c >= 0x80 {
                self.cursor = pos as u32;
                let cp = code_point_at(src, pos).map_err(|_| LexicalError::InvalidUtf8)?;

                if can_continue_id(cp.value.to_u32()) {
                    pos += cp.len as usize;
                    continue;
                }
            }

            break;
        }

        debug_assert!(pos >= entry_pos);
        self.cursor = pos as u32;
        Ok(has_escape)
    }

    fn scan_identifier_or_keyword(&mut self) -> Result<Token, LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());

        let start = self.cursor;
        let is_jsx_tag = self.mode == LexerMode::JsxTag;
        let mut has_escape = false;
        let is_private = self.peek(0) == b'#';

        if is_private {
            self.cursor += 1;
        }

        let first_char = self.peek(0);

        if first_char.is_ascii() {
            if first_char == b'\\' {
                if is_jsx_tag {
                    return Err(LexicalError::JsxIdentifierCannotStartWithBackslash);
                }

                has_escape = true;
                self.cursor += 1;
                self.consume_unicode_escape(ConsumeUnicodeContext::IdentifierStart)?;
            } else {
                if !is_ident_start_ascii(first_char) {
                    return Err(LexicalError::InvalidIdentifierStart);
                }
                self.cursor += 1;
            }

            let body_has_escape = self.scan_identifier_body(is_jsx_tag)?;
            has_escape = has_escape || body_has_escape;
        } else {
            let c_cp = code_point_at(self.source, self.cursor as usize)
                .map_err(|_| LexicalError::InvalidUtf8)?;

            if !can_start_id(c_cp.value.to_u32()) {
                return Err(LexicalError::InvalidIdentifierStart);
            }

            self.cursor += u32::from(c_cp.len);
            has_escape = self.scan_identifier_body(is_jsx_tag)?;
        }

        debug_assert!(self.cursor > start);
        let lexeme = &self.source[start as usize..self.cursor as usize];

        let tag = if is_jsx_tag {
            TokenTag::JsxIdentifier
        } else if is_private {
            TokenTag::PrivateIdentifier
        } else if has_escape {
            get_escaped_keyword_type(lexeme)
        } else {
            get_keyword_type(lexeme)
        };

        Ok(self.create_token(tag, start, self.cursor))
    }

    #[inline(always)]
    fn scan_punctuation(&mut self) -> Token {
        debug_assert!((self.cursor as usize) < self.source.len());
        let start = self.cursor;
        let c0 = self.source[self.cursor as usize];
        let c1 = self.peek(1);

        match c0 {
            b'+' => match c1 {
                b'+' => self.punc_token(2, TokenTag::Increment, start),
                b'=' => self.punc_token(2, TokenTag::PlusAssign, start),
                _ => self.punc_token(1, TokenTag::Plus, start),
            },
            b'-' => match c1 {
                b'-' => self.punc_token(2, TokenTag::Decrement, start),
                b'=' => self.punc_token(2, TokenTag::MinusAssign, start),
                _ => self.punc_token(1, TokenTag::Minus, start),
            },
            b'*' if c1 == b'*' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::ExponentAssign, start)
            }
            b'*' => match c1 {
                b'*' => self.punc_token(2, TokenTag::Exponent, start),
                b'=' => self.punc_token(2, TokenTag::StarAssign, start),
                _ => self.punc_token(1, TokenTag::Star, start),
            },
            b'/' if c1 == b'=' => self.punc_token(2, TokenTag::SlashAssign, start),
            b'/' => self.punc_token(1, TokenTag::Slash, start),
            b'%' => match c1 {
                b'=' => self.punc_token(2, TokenTag::PercentAssign, start),
                _ => self.punc_token(1, TokenTag::Percent, start),
            },
            b'<' if c1 == b'<' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::LeftShiftAssign, start)
            }
            b'<' => match c1 {
                b'<' => self.punc_token(2, TokenTag::LeftShift, start),
                b'=' => self.punc_token(2, TokenTag::LessThanEqual, start),
                _ => self.punc_token(1, TokenTag::LessThan, start),
            },
            // Inside a JSX tag `>` only closes the tag, so `<div>=</div>` and the
            // trailing `>>` of `<div attr=<e></e>></div>` must not merge.
            b'>' if self.mode == LexerMode::JsxTag => {
                self.punc_token(1, TokenTag::GreaterThan, start)
            }
            b'>' if c1 == b'>' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::RightShiftAssign, start)
            }
            b'>' if c1 == b'>' && self.peek(2) == b'>' && self.peek(3) == b'=' => {
                self.punc_token(4, TokenTag::UnsignedRightShiftAssign, start)
            }
            b'>' if c1 == b'>' && self.peek(2) == b'>' => {
                self.punc_token(3, TokenTag::UnsignedRightShift, start)
            }
            b'>' => match c1 {
                b'>' => self.punc_token(2, TokenTag::RightShift, start),
                b'=' => self.punc_token(2, TokenTag::GreaterThanEqual, start),
                _ => self.punc_token(1, TokenTag::GreaterThan, start),
            },
            b'=' if c1 == b'=' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::StrictEqual, start)
            }
            b'=' => match c1 {
                b'=' => self.punc_token(2, TokenTag::Equal, start),
                b'>' => self.punc_token(2, TokenTag::Arrow, start),
                _ => self.punc_token(1, TokenTag::Assign, start),
            },
            b'!' if c1 == b'=' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::StrictNotEqual, start)
            }
            b'!' => match c1 {
                b'=' => self.punc_token(2, TokenTag::NotEqual, start),
                _ => self.punc_token(1, TokenTag::LogicalNot, start),
            },
            b'&' if c1 == b'&' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::LogicalAndAssign, start)
            }
            b'&' => match c1 {
                b'&' => self.punc_token(2, TokenTag::LogicalAnd, start),
                b'=' => self.punc_token(2, TokenTag::BitwiseAndAssign, start),
                _ => self.punc_token(1, TokenTag::BitwiseAnd, start),
            },
            b'|' if c1 == b'|' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::LogicalOrAssign, start)
            }
            b'|' => match c1 {
                b'|' => self.punc_token(2, TokenTag::LogicalOr, start),
                b'=' => self.punc_token(2, TokenTag::BitwiseOrAssign, start),
                _ => self.punc_token(1, TokenTag::BitwiseOr, start),
            },
            b'^' => match c1 {
                b'=' => self.punc_token(2, TokenTag::BitwiseXorAssign, start),
                _ => self.punc_token(1, TokenTag::BitwiseXor, start),
            },
            b'?' if c1 == b'?' && self.peek(2) == b'=' => {
                self.punc_token(3, TokenTag::NullishAssign, start)
            }
            b'?' => match c1 {
                b'?' => self.punc_token(2, TokenTag::NullishCoalescing, start),
                b'.' if self.peek(2).is_ascii_digit() => {
                    self.punc_token(1, TokenTag::Question, start)
                }
                b'.' => self.punc_token(2, TokenTag::OptionalChaining, start),
                _ => self.punc_token(1, TokenTag::Question, start),
            },
            _ => unreachable!(),
        }
    }

    #[inline]
    fn punc_token(&mut self, len: u32, tag: TokenTag, start: u32) -> Token {
        debug_assert!(len > 0);
        debug_assert_eq!(start, self.cursor);
        debug_assert!((start + len) as usize <= self.source.len());
        self.cursor += len;
        self.create_token(tag, start, self.cursor)
    }

    #[inline]
    fn peek(&self, offset: u32) -> u8 {
        let idx = (self.cursor + offset) as usize;
        self.source.get(idx).copied().unwrap_or(0)
    }

    #[inline]
    fn set_token_flag(&mut self, flag: TokenFlag) {
        self.state.token_flags |= flag_mask(flag);
    }

    #[inline]
    fn has_token_flag(&self, flag: TokenFlag) -> bool {
        (self.state.token_flags & flag_mask(flag)) != 0
    }

    #[inline]
    fn clear_token_flags(&mut self) {
        self.state.token_flags = 0;
    }

    #[inline]
    fn consume_token_flags(&mut self) -> u8 {
        let flags = self.state.token_flags;
        self.state.token_flags = 0;
        flags
    }

    pub fn rewind_to(&mut self, position: u32) {
        debug_assert!((position as usize) <= self.source.len());
        self.cursor = position;
        self.clear_token_flags();
    }

    pub fn re_scan_template_continuation(
        &mut self,
        right_brace_start: u32,
    ) -> Result<Token, LexicalError> {
        debug_assert!((right_brace_start as usize) < self.source.len());
        debug_assert_eq!(self.source[right_brace_start as usize], b'}');

        self.rewind_to(right_brace_start);

        let start = self.cursor;
        self.cursor += 1;

        self.scan_template_segment(start, TokenTag::TemplateTail, TokenTag::TemplateMiddle)
    }

    pub fn re_scan_greater_than(&mut self, token_start: u32) -> Token {
        debug_assert!((token_start as usize) < self.source.len());
        debug_assert_eq!(self.source[token_start as usize], b'>');
        self.rewind_to(token_start + 1);
        self.create_token(TokenTag::GreaterThan, token_start, token_start + 1)
    }

    pub fn re_scan_less_than(&mut self, token_start: u32) -> Token {
        debug_assert!((token_start as usize) < self.source.len());
        debug_assert_eq!(self.source[token_start as usize], b'<');
        self.rewind_to(token_start + 1);
        self.create_token(TokenTag::LessThan, token_start, token_start + 1)
    }

    pub fn scan_jsx_text(&mut self) -> Token {
        let start = self.cursor;
        let token = match self.source.get(self.cursor as usize).copied() {
            Some(b'<') => {
                self.cursor += 1;
                self.create_token(TokenTag::LessThan, start, self.cursor)
            }
            Some(b'{') => {
                self.cursor += 1;
                self.create_token(TokenTag::LeftBrace, start, self.cursor)
            }
            Some(_) => {
                self.cursor += 1;
                let rest = &self.source[self.cursor as usize..];
                if let Some(offset) = memchr2(b'<', b'{', rest) {
                    self.cursor += offset as u32;
                } else {
                    self.cursor = self.source.len() as u32;
                }
                self.create_token(TokenTag::JsxText, start, self.cursor)
            }
            _ => self.create_token(TokenTag::Eof, start, start),
        };
        debug_assert!(self.cursor >= start);
        self.handle_token(token, HandleTokenMode::Append);
        token
    }

    pub fn re_scan_as_regex(
        &mut self,
        slash_token_start: u32,
    ) -> Result<RegexResult, LexicalError> {
        debug_assert!((slash_token_start as usize) < self.source.len());
        debug_assert_eq!(self.source[slash_token_start as usize], b'/');
        self.rewind_to(slash_token_start);

        let start = self.cursor;
        self.cursor += 1;
        let mut in_class = false;

        while (self.cursor as usize) < self.source.len() {
            let c = self.source[self.cursor as usize];
            if self.is_line_terminator(c) {
                return Err(LexicalError::InvalidRegexLineTerminator);
            }

            if c == b'\\' {
                self.cursor += 1;
                if (self.cursor as usize) >= self.source.len() {
                    return Err(LexicalError::UnterminatedRegexLiteral);
                }
                if self.is_line_terminator(self.source[self.cursor as usize]) {
                    return Err(LexicalError::InvalidRegexLineTerminator);
                }
                self.cursor += 1;
                continue;
            }

            if c == b'[' {
                in_class = true;
                self.cursor += 1;
                continue;
            }
            if c == b']' && in_class {
                in_class = false;
                self.cursor += 1;
                continue;
            }
            if c == b'/' && !in_class {
                self.cursor += 1;
                let closing_delimiter_pos = self.cursor;
                // Preserve the full IdentifierPart sequence, including Unicode
                // and escapes. Semantic syntax checks validate the raw flags.
                self.scan_identifier_body(false)?;

                let end = self.cursor;
                return Ok(RegexResult {
                    span: Span { start, end },
                    pattern: Span {
                        start: start + 1,
                        end: closing_delimiter_pos - 1,
                    },
                    flags: Span {
                        start: closing_delimiter_pos,
                        end,
                    },
                });
            }

            self.cursor += 1;
        }

        Err(LexicalError::UnterminatedRegexLiteral)
    }

    #[inline]
    fn is_line_terminator(&self, c: u8) -> bool {
        c == b'\n'
            || c == b'\r'
            || (c == 0xE2 && unicode_separator_len(self.source, self.cursor as usize) > 0)
    }

    #[inline]
    fn skip_ws_and_comments(&mut self) -> Result<(), LexicalError> {
        debug_assert!((self.cursor as usize) <= self.source.len());
        let at_start_of_source = self.cursor == 0;

        loop {
            let src = self.source;
            let mut pos = self.cursor as usize;

            while pos < src.len() {
                match src[pos] {
                    b' ' | b'\t' | 0x0B | 0x0C => pos += 1,
                    b'\n' | b'\r' => {
                        self.set_token_flag(TokenFlag::LineTerminatorBefore);
                        pos += 1;
                    }
                    _ => break,
                }
            }

            self.cursor = pos as u32;
            if pos == src.len()
                || !matches!(src[pos], b'/' | b'<' | b'-' | 0x80..=0xFF)
                || !self.skip_ws_and_comments_slow(at_start_of_source)?
            {
                return Ok(());
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn skip_ws_and_comments_slow(
        &mut self,
        at_start_of_source: bool,
    ) -> Result<bool, LexicalError> {
        let src = self.source;
        let pos = self.cursor as usize;
        debug_assert!(pos < src.len());

        match src[pos] {
            b'/' => match src.get(pos + 1).copied() {
                Some(b'/') => self.scan_line_comment()?,
                Some(b'*') => self.scan_block_comment()?,
                _ => return Ok(false),
            },
            b'<' => {
                if !self.allow_html_like_comments
                    || pos + 3 >= src.len()
                    || src[pos + 1] != b'!'
                    || src[pos + 2] != b'-'
                    || src[pos + 3] != b'-'
                {
                    return Ok(false);
                }
                self.scan_html_comment()?;
            }
            b'-' => {
                let can_be_html_close_comment =
                    at_start_of_source || self.has_token_flag(TokenFlag::LineTerminatorBefore);
                if !self.allow_html_like_comments
                    || !can_be_html_close_comment
                    || pos + 2 >= src.len()
                    || src[pos + 1] != b'-'
                    || src[pos + 2] != b'>'
                {
                    return Ok(false);
                }
                self.scan_html_close_comment()?;
            }
            0x80..=0xFF => {
                let separator_len = unicode_separator_len(src, pos) as usize;
                if separator_len > 0 {
                    self.set_token_flag(TokenFlag::LineTerminatorBefore);
                    self.cursor += separator_len as u32;
                    return Ok(true);
                }

                let code_point = code_point_at(src, pos).map_err(|_| LexicalError::InvalidUtf8)?;
                if !is_multi_byte_space(code_point.value.to_u32()) {
                    return Ok(false);
                }
                self.cursor += u32::from(code_point.len);
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    /// Line and HTML-like comments end at a line terminator or at the end of the
    /// source, so [`TokenFlag::LineTerminatorBefore`] records every line break
    /// since the previous token that can precede another comment.
    #[inline]
    fn can_have_trailing_line_comment(&self) -> bool {
        !self.has_token_flag(TokenFlag::LineTerminatorBefore)
            && self.state.previous_token_tag.is_some_and(|tag| {
                (self.mode == LexerMode::JsxTag && tag == TokenTag::Slash)
                    || tag.can_have_trailing_line_comment()
            })
    }

    #[inline]
    fn can_have_trailing_block_comment(&self, had_line_break: bool, next: u32) -> bool {
        !had_line_break
            && self.state.previous_token_tag.is_some_and(|tag| {
                self.source.get(next as usize) == Some(&b';') || tag.can_have_trailing_comment()
            })
    }

    #[inline]
    fn record_comment(
        &mut self,
        kind: CommentKind,
        start: u32,
        end: u32,
        trailing: bool,
    ) -> Result<(), LexicalError> {
        debug_assert!(start < end);
        debug_assert!((end as usize) <= self.source.len());
        if !self.collect_comments {
            return Ok(());
        }
        let mut comment = Comment::new(Span::new(start, end), kind);
        if trailing {
            comment.position = CommentPosition::Trailing;
            comment.attached_to = self.state.prev_token_end;
            for comment in &mut self.comments[self.processed_comments..] {
                comment.attached_to = self.state.prev_token_end;
            }
            self.processed_comments = self.comments.len() + 1;
        }
        self.comments.push(comment);
        Ok(())
    }

    fn attach_pending_comments(&mut self, token: Token) {
        let (position, attached_to) = if token.tag() == TokenTag::Eof {
            cold_path();
            if self.state.prev_token_end == 0 {
                (CommentPosition::Leading, 0)
            } else {
                (CommentPosition::Trailing, self.state.prev_token_end)
            }
        } else {
            (CommentPosition::Leading, token.start())
        };

        for comment in &mut self.comments[self.processed_comments..] {
            comment.position = position;
            comment.attached_to = attached_to;
        }
        self.processed_comments = self.comments.len();
    }

    fn scan_line_comment(&mut self) -> Result<(), LexicalError> {
        debug_assert!(((self.cursor + 1) as usize) < self.source.len());
        debug_assert_eq!(self.source[self.cursor as usize], b'/');
        debug_assert_eq!(self.source[self.cursor as usize + 1], b'/');
        let start = self.cursor;
        let src = self.source;
        let mut pos = start as usize + 2;
        while pos < src.len() {
            let rest = &src[pos..];
            let Some(offset) = memchr3(b'\n', b'\r', 0xE2, rest) else {
                pos = src.len();
                break;
            };
            pos += offset;
            if src[pos] != 0xE2 || unicode_separator_len(src, pos) > 0 {
                break;
            }
            pos += 1;
        }
        self.cursor = pos as u32;
        let trailing = self.can_have_trailing_line_comment();
        self.record_comment(CommentKind::Line, start, pos as u32, trailing)
    }

    fn scan_block_comment(&mut self) -> Result<(), LexicalError> {
        debug_assert!(((self.cursor + 1) as usize) < self.source.len());
        debug_assert_eq!(self.source[self.cursor as usize], b'/');
        debug_assert_eq!(self.source[self.cursor as usize + 1], b'*');
        let start = self.cursor;
        let had_line_break = self.has_token_flag(TokenFlag::LineTerminatorBefore);
        let src = self.source;
        let mut pos = start as usize + 2;
        while pos < src.len() {
            // Preserve line-terminator bookkeeping while searching for */.
            // The overlapping load also catches a terminator across chunks.
            while src.len() - pos > 16 {
                let bytes = u8x16::from_slice(&src[pos..]);
                let next = u8x16::from_slice(&src[pos + 1..]);
                let end = bytes.simd_eq(u8x16::splat(b'*')) & next.simd_eq(u8x16::splat(b'/'));
                // Once recorded, further line breaks cannot change the next token's flags.
                let special = if self.has_token_flag(TokenFlag::LineTerminatorBefore) {
                    end
                } else {
                    end | bytes.simd_eq(u8x16::splat(b'\n'))
                        | bytes.simd_eq(u8x16::splat(b'\r'))
                        | bytes.simd_eq(u8x16::splat(0xE2))
                };
                if let Some(offset) = special.first_set() {
                    pos += offset;
                    break;
                }
                pos += 16;
            }

            match src[pos] {
                b'*' if pos + 1 < src.len() && src[pos + 1] == b'/' => {
                    pos += 2;
                    self.cursor = pos as u32;
                    let trailing = self.can_have_trailing_block_comment(had_line_break, pos as u32);
                    return self.record_comment(CommentKind::Block, start, pos as u32, trailing);
                }
                b'*' => pos += 1,
                b'\n' | b'\r' => {
                    self.set_token_flag(TokenFlag::LineTerminatorBefore);
                    pos += 1;
                }
                0x80..=0xFF => {
                    let lt_len = unicode_separator_len(src, pos) as usize;
                    if lt_len > 0 {
                        self.set_token_flag(TokenFlag::LineTerminatorBefore);
                        pos += lt_len;
                    } else {
                        pos += 1;
                    }
                }
                _ => pos += 1,
            }
        }
        self.cursor = pos as u32;
        Err(LexicalError::UnterminatedMultiLineComment)
    }

    #[cold]
    #[inline(never)]
    fn scan_html_comment(&mut self) -> Result<(), LexicalError> {
        debug_assert!(self.allow_html_like_comments);
        debug_assert!(((self.cursor + 3) as usize) < self.source.len());
        debug_assert_eq!(self.source[self.cursor as usize], b'<');
        debug_assert_eq!(self.source[self.cursor as usize + 1], b'!');
        let start = self.cursor;
        let trailing = self.can_have_trailing_line_comment();
        self.cursor += 4;
        while (self.cursor as usize) < self.source.len() {
            let c = self.source[self.cursor as usize];
            if self.is_line_terminator(c) {
                break;
            }
            self.cursor += 1;
        }
        self.record_comment(CommentKind::Line, start, self.cursor, trailing)?;
        self.module_mode_errors.push(Span::new(start, self.cursor));
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn scan_html_close_comment(&mut self) -> Result<(), LexicalError> {
        debug_assert!(self.allow_html_like_comments);
        debug_assert!(((self.cursor + 2) as usize) < self.source.len());
        debug_assert_eq!(self.source[self.cursor as usize], b'-');
        debug_assert_eq!(self.source[self.cursor as usize + 1], b'-');
        debug_assert_eq!(self.source[self.cursor as usize + 2], b'>');
        let start = self.cursor;
        let trailing = self.can_have_trailing_line_comment();
        self.cursor += 3;
        while (self.cursor as usize) < self.source.len() {
            if self.is_line_terminator(self.source[self.cursor as usize]) {
                break;
            }
            self.cursor += 1;
        }
        self.record_comment(CommentKind::Line, start, self.cursor, trailing)?;
        self.module_mode_errors.push(Span::new(start, self.cursor));
        Ok(())
    }

    #[inline]
    pub fn create_token(&mut self, tag: TokenTag, start: u32, end: u32) -> Token {
        Token::new(start, end, tag, self.consume_token_flags())
    }
}

const SIMPLE_PUNCT_TAG: [TokenTag; 256] = {
    let mut table = [TokenTag::Eof; 256];
    table[b'~' as usize] = TokenTag::BitwiseNot;
    table[b'(' as usize] = TokenTag::LeftParen;
    table[b')' as usize] = TokenTag::RightParen;
    table[b'{' as usize] = TokenTag::LeftBrace;
    table[b'}' as usize] = TokenTag::RightBrace;
    table[b'[' as usize] = TokenTag::LeftBracket;
    table[b']' as usize] = TokenTag::RightBracket;
    table[b';' as usize] = TokenTag::Semicolon;
    table[b',' as usize] = TokenTag::Comma;
    table[b':' as usize] = TokenTag::Colon;
    table[b'@' as usize] = TokenTag::At;
    table
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FastClass {
    Slow,
    Identifier,
    Punctuation,
    Digit,
    Dot,
    Operator,
}

const FAST_CLASS: [FastClass; 256] = {
    let mut table = [FastClass::Slow; 256];
    let mut byte = 0;
    while byte < 256 {
        if (byte >= b'a' as usize && byte <= b'z' as usize)
            || (byte >= b'A' as usize && byte <= b'Z' as usize)
            || byte == b'_' as usize
            || byte == b'$' as usize
        {
            table[byte] = FastClass::Identifier;
        } else if !matches!(SIMPLE_PUNCT_TAG[byte], TokenTag::Eof) {
            table[byte] = FastClass::Punctuation;
        }
        byte += 1;
    }
    let mut digit = b'0' as usize;
    while digit <= b'9' as usize {
        table[digit] = FastClass::Digit;
        digit += 1;
    }
    table[b'.' as usize] = FastClass::Dot;
    // Slash and minus can start comments and retain the trivia path.
    let operators = b"+*%<>=!&|^?";
    let mut operator = 0;
    while operator < operators.len() {
        table[operators[operator] as usize] = FastClass::Operator;
        operator += 1;
    }
    table
};

const IDENTIFIER_BATCH_SIZE: usize = 32;

/// How a byte after an ASCII identifier prefix affects the identifier.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum IdentifierClass {
    /// An ASCII identifier part.
    Continue,
    /// A byte that ends the identifier.
    End,
    /// An escape or non-ASCII byte that needs the full identifier scanner.
    Slow,
}

const ASCII_IDENTIFIER_CLASS: [IdentifierClass; 256] = {
    let mut table = [IdentifierClass::End; 256];
    let mut byte = 0;
    while byte < 256 {
        if (byte >= b'0' as usize && byte <= b'9' as usize)
            || (byte >= b'A' as usize && byte <= b'Z' as usize)
            || (byte >= b'a' as usize && byte <= b'z' as usize)
            || byte == b'_' as usize
            || byte == b'$' as usize
        {
            table[byte] = IdentifierClass::Continue;
        } else if byte == b'\\' as usize || byte >= 0x80 {
            table[byte] = IdentifierClass::Slow;
        }
        byte += 1;
    }
    table
};

/// Returns the token flags of a fast-path token, which may only follow a line break.
#[inline(always)]
fn fast_token_flags(line_break: bool) -> u8 {
    if line_break {
        flag_mask(TokenFlag::LineTerminatorBefore)
    } else {
        0
    }
}

// This is a very hot path. Inline this loop too
// to avoid losing the single-call-site bonus and
// introducing a new helper call.
/// Returns the end of the ASCII identifier part at `start`, or `None` if an
/// escape or a non-ASCII byte follows it and the full scanner is needed.
#[inline(always)]
fn scan_ascii_identifier_continue(source: &[u8], start: u32) -> Option<u32> {
    debug_assert!(u32::try_from(source.len()).is_ok());
    debug_assert!((start as usize) <= source.len());
    // `scan_ascii_identifier` passes the byte after a start byte it already read from `source`.
    let mut position = start as usize;

    // Comparing the batch end instead of the remaining length keeps no extra counter live, so
    // the scan fits in caller-saved registers. `position + IDENTIFIER_BATCH_SIZE` cannot overflow
    // because a slice length is at most `isize::MAX`.
    while position + IDENTIFIER_BATCH_SIZE <= source.len() {
        let batch_end = position + IDENTIFIER_BATCH_SIZE;
        // SAFETY: `position <= source.len()` by the caller invariant above, and the loop condition
        // guarantees that `batch_end` is within `source`.
        let batch = unsafe { source.get_unchecked(position..batch_end) };
        for (offset, byte) in batch.iter().enumerate() {
            if ASCII_IDENTIFIER_CLASS[*byte as usize] != IdentifierClass::Continue {
                return ascii_identifier_end(*byte, position + offset);
            }
        }
        position += IDENTIFIER_BATCH_SIZE;
    }
    while let Some(byte) = source.get(position) {
        if ASCII_IDENTIFIER_CLASS[*byte as usize] != IdentifierClass::Continue {
            return ascii_identifier_end(*byte, position);
        }
        position += 1;
    }

    debug_assert!(position >= start as usize);
    debug_assert!(position <= source.len());
    Some(position as u32)
}

/// Ends an ASCII identifier part at `position`, where `byte` is not an
/// identifier part. Returns `None` for `\\` and non-ASCII bytes, which are the
/// [`IdentifierClass::Slow`] bytes. Testing the byte instead of the table keeps
/// the scan loop from holding the class of each byte.
#[inline(always)]
fn ascii_identifier_end(byte: u8, position: usize) -> Option<u32> {
    // `\\` maps to zero and non-ASCII bytes stay negative.
    let slow = (byte ^ b'\\') as i8 <= 0;
    (!slow).then_some(position as u32)
}

#[inline]
fn is_surrogate_codepoint(cp: u32) -> bool {
    (0xD800..=0xDFFF).contains(&cp)
}

#[inline]
fn is_utf16_high_surrogate(cp: u32) -> bool {
    (0xD800..=0xDBFF).contains(&cp)
}

fn get_escaped_keyword_type(lexeme: &[u8]) -> TokenTag {
    debug_assert!(!lexeme.is_empty());
    let mut buf = [0u8; 11];
    let mut out = 0usize;
    let mut i = 0usize;

    while i < lexeme.len() {
        if out == buf.len() {
            return TokenTag::Identifier;
        }
        if lexeme[i] == b'\\' && i + 1 < lexeme.len() && lexeme[i + 1] == b'u' {
            let Some(parsed) = parse_unicode_escape(lexeme, i + 2) else {
                return TokenTag::Identifier;
            };
            if parsed.value.to_u32() >= 0x80 {
                return TokenTag::Identifier;
            }
            buf[out] = parsed.value.to_u32() as u8;
            out += 1;
            i = parsed.end;
        } else {
            if !lexeme[i].is_ascii() {
                return TokenTag::Identifier;
            }
            buf[out] = lexeme[i];
            out += 1;
            i += 1;
        }
    }

    get_keyword_type(&buf[..out])
}

pub fn lexical_error_message(error_type: LexicalError) -> &'static str {
    match error_type {
        LexicalError::InvalidHexEscape => "Invalid hexadecimal escape sequence",
        LexicalError::UnterminatedString => "Unterminated string literal",
        LexicalError::NonTerminatedTemplateLiteral => "Unterminated template literal",
        LexicalError::UnterminatedRegexLiteral => "Unterminated regular expression literal",
        LexicalError::InvalidRegexLineTerminator => {
            "Line terminator not allowed in regular expression literal"
        }
        LexicalError::InvalidIdentifierStart => "Invalid character at start of identifier",
        LexicalError::InvalidIdentifierContinue => "Invalid character in identifier",
        LexicalError::UnterminatedMultiLineComment => "Unterminated multi-line comment",
        LexicalError::InvalidUnicodeEscape => "Invalid Unicode escape sequence",
        LexicalError::InvalidOctalEscape => "Invalid octal escape sequence",
        LexicalError::InvalidOctalLiteralDigit => {
            "Octal literal must contain at least one octal digit"
        }
        LexicalError::InvalidBinaryLiteral => {
            "Binary literal must contain at least one binary digit"
        }
        LexicalError::InvalidHexLiteral => {
            "Hexadecimal literal must contain at least one hex digit"
        }
        LexicalError::InvalidExponentPart => "Exponent part is missing a number",
        LexicalError::NumericSeparatorMisuse => {
            "Numeric separator cannot appear at the end of a numeric literal"
        }
        LexicalError::ConsecutiveNumericSeparators => {
            "Numeric literal cannot contain consecutive separators"
        }
        LexicalError::InvalidBigIntSuffix => {
            "BigInt literal cannot contain decimal point or exponent"
        }
        LexicalError::IdentifierAfterNumericLiteral => {
            "Identifier cannot immediately follow a numeric literal"
        }
        LexicalError::InvalidUtf8 => "Invalid UTF-8 byte sequence",
        LexicalError::JsxIdentifierCannotContainEscapes => {
            "JSX tag names cannot contain escape sequences"
        }
        LexicalError::JsxIdentifierCannotStartWithBackslash => {
            "JSX tag names cannot start with a backslash"
        }
    }
}

pub fn lexical_error_help(error_type: LexicalError) -> &'static str {
    match error_type {
        LexicalError::InvalidHexEscape => {
            "Try adding two hexadecimal digits here (e.g., \\x41 for 'A')"
        }
        LexicalError::UnterminatedString => {
            "Try adding a closing quote here to complete the string"
        }
        LexicalError::NonTerminatedTemplateLiteral => {
            "Try adding a closing backtick (`) here to complete the template"
        }
        LexicalError::UnterminatedRegexLiteral => {
            "Try adding a closing slash (/) here, optionally followed by flags (g, i, m, etc.)"
        }
        LexicalError::InvalidRegexLineTerminator => {
            "Try removing the line break here or escaping it within the regex pattern"
        }
        LexicalError::InvalidIdentifierStart => {
            "Try starting the identifier here with a letter (a-z, A-Z), underscore (_), or dollar sign ($)"
        }
        LexicalError::InvalidIdentifierContinue => {
            "Try using a valid identifier character here (letters, digits, underscore, or dollar sign)"
        }
        LexicalError::UnterminatedMultiLineComment => {
            "Try adding the closing delimiter (*/) here to complete the comment"
        }
        LexicalError::InvalidUnicodeEscape => {
            "Try using \\uHHHH (4 hex digits) or \\u{HHHHHH} (1-6 hex digits) here"
        }
        LexicalError::InvalidOctalEscape => {
            "Try using a valid octal sequence here (\\0-7, \\00-77, or \\000-377)"
        }
        LexicalError::InvalidOctalLiteralDigit => {
            "Try adding at least one octal digit (0-7) here after '0o'"
        }
        LexicalError::InvalidBinaryLiteral => {
            "Try adding at least one binary digit (0 or 1) here after '0b'"
        }
        LexicalError::InvalidHexLiteral => {
            "Try adding at least one hex digit (0-9, a-f, A-F) here after '0x'"
        }
        LexicalError::InvalidExponentPart => {
            "Try adding digits here after the exponent (e.g., e10, e-5, E+2)"
        }
        LexicalError::NumericSeparatorMisuse => {
            "Try removing the trailing underscore here or adding more digits after it"
        }
        LexicalError::ConsecutiveNumericSeparators => {
            "Try removing one of the consecutive underscores here"
        }
        LexicalError::InvalidBigIntSuffix => {
            "Try removing the 'n' suffix here, or remove the decimal point/exponent from the number"
        }
        LexicalError::IdentifierAfterNumericLiteral => {
            "Try adding whitespace here between the number and identifier"
        }
        LexicalError::InvalidUtf8 => {
            "The source file contains invalid UTF-8 encoding. Ensure the file is saved with valid UTF-8 encoding"
        }
        LexicalError::JsxIdentifierCannotContainEscapes => {
            "Remove the escape sequence and use the literal character instead"
        }
        LexicalError::JsxIdentifierCannotStartWithBackslash => {
            "JSX tag names must be plain identifiers without escape sequences"
        }
    }
}

// INLINE: This is a very hot path. Reuse identifier bytes
// already loaded by the caller and avoid preserving
// token state across a separate keyword-lookup call.
#[inline(always)]
fn get_keyword_type(lexeme: &[u8]) -> TokenTag {
    match lexeme {
        b"if" => TokenTag::If,
        b"of" => TokenTag::Of,
        b"in" => TokenTag::In,
        b"do" => TokenTag::Do,
        b"as" => TokenTag::As,
        b"is" => TokenTag::Is,
        b"any" => TokenTag::Any,
        b"for" => TokenTag::For,
        b"get" => TokenTag::Get,
        b"let" => TokenTag::Let,
        b"new" => TokenTag::New,
        b"out" => TokenTag::Out,
        b"set" => TokenTag::Set,
        b"try" => TokenTag::Try,
        b"var" => TokenTag::Var,
        b"case" => TokenTag::Case,
        b"this" => TokenTag::This,
        b"else" => TokenTag::Else,
        b"enum" => TokenTag::Enum,
        b"void" => TokenTag::Void,
        b"with" => TokenTag::With,
        b"null" => TokenTag::NullLiteral,
        b"type" => TokenTag::Type,
        b"true" => TokenTag::True,
        b"from" => TokenTag::From,
        b"await" => TokenTag::Await,
        b"async" => TokenTag::Async,
        b"break" => TokenTag::Break,
        b"const" => TokenTag::Const,
        b"class" => TokenTag::Class,
        b"catch" => TokenTag::Catch,
        b"defer" => TokenTag::Defer,
        b"false" => TokenTag::False,
        b"infer" => TokenTag::Infer,
        b"keyof" => TokenTag::Keyof,
        b"never" => TokenTag::Never,
        b"super" => TokenTag::Super,
        b"throw" => TokenTag::Throw,
        b"using" => TokenTag::Using,
        b"while" => TokenTag::While,
        b"yield" => TokenTag::Yield,
        b"assert" => TokenTag::Assert,
        b"bigint" => TokenTag::Bigint,
        b"delete" => TokenTag::Delete,
        b"export" => TokenTag::Export,
        b"global" => TokenTag::Global,
        b"import" => TokenTag::Import,
        b"module" => TokenTag::Module,
        b"number" => TokenTag::Number,
        b"object" => TokenTag::Object,
        b"public" => TokenTag::Public,
        b"return" => TokenTag::Return,
        b"source" => TokenTag::Source,
        b"static" => TokenTag::Static,
        b"string" => TokenTag::String,
        b"switch" => TokenTag::Switch,
        b"symbol" => TokenTag::Symbol,
        b"typeof" => TokenTag::Typeof,
        b"unique" => TokenTag::Unique,
        b"boolean" => TokenTag::Boolean,
        b"declare" => TokenTag::Declare,
        b"default" => TokenTag::Default,
        b"extends" => TokenTag::Extends,
        b"finally" => TokenTag::Finally,
        b"package" => TokenTag::Package,
        b"private" => TokenTag::Private,
        b"require" => TokenTag::Require,
        b"unknown" => TokenTag::Unknown,
        b"abstract" => TokenTag::Abstract,
        b"continue" => TokenTag::Continue,
        b"debugger" => TokenTag::Debugger,
        b"function" => TokenTag::Function,
        b"readonly" => TokenTag::Readonly,
        b"satisfies" => TokenTag::Satisfies,
        b"undefined" => TokenTag::Undefined,
        b"accessor" => TokenTag::Accessor,
        b"interface" => TokenTag::Interface,
        b"namespace" => TokenTag::Namespace,
        b"protected" => TokenTag::Protected,
        b"asserts" => TokenTag::Asserts,
        b"intrinsic" => TokenTag::Intrinsic,
        b"override" => TokenTag::Override,
        b"constructor" => TokenTag::Constructor,
        b"implements" => TokenTag::Implements,
        b"instanceof" => TokenTag::Instanceof,
        _ => TokenTag::Identifier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn next_template(source: &str) -> Result<Token, LexicalError> {
        let allocator = Allocator::new();
        let mut lexer = Lexer::init(&allocator, source, false, false, NoTokenParserConfig);
        let mut token = Token::eof(0);
        lexer.next_token(&mut token)?;
        Ok(token)
    }

    fn next_after_trivia(source: &str, allow_html_like_comments: bool) -> Token {
        let allocator = Allocator::new();
        let mut lexer = Lexer::init(
            &allocator,
            source,
            allow_html_like_comments,
            false,
            NoTokenParserConfig,
        );
        let mut token = Token::eof(0);
        lexer.next_token(&mut token).unwrap();
        token
    }

    #[test]
    fn initialization() {
        let allocator = Allocator::new();
        let small_source = " ".repeat(512);

        let small = Lexer::init(&allocator, &small_source, false, true, NoTokenParserConfig);
        assert_eq!(small.comments.capacity(), 0);

        let large_source = " ".repeat(2 * 1024 * 1024);
        let collecting = Lexer::init(&allocator, &large_source, false, true, NoTokenParserConfig);
        assert_eq!(collecting.comments.capacity(), large_source.len() / 256);

        let ignoring = Lexer::init(&allocator, &large_source, false, false, NoTokenParserConfig);
        assert_eq!(ignoring.comments.capacity(), 0);
    }

    #[test]
    fn trivia() {
        let token = next_after_trivia(" \t\x0B\x0Cname", false);
        assert_eq!(token.span(), Span::new(4, 8));
        assert!(!token.has_line_terminator_before());

        let token = next_after_trivia("\r\nname", false);
        assert_eq!(token.span(), Span::new(2, 6));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("// line\nname", false);
        assert_eq!(token.span(), Span::new(8, 12));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("/* block\ncomment */name", false);
        assert_eq!(token.span(), Span::new(19, 23));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("\u{2028}name", false);
        assert_eq!(token.span(), Span::new(3, 7));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("\u{00A0}name", false);
        assert_eq!(token.span(), Span::new(2, 6));
        assert!(!token.has_line_terminator_before());

        let token = next_after_trivia("<!-- comment\nname", true);
        assert_eq!(token.span(), Span::new(13, 17));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("<!-- --> \"\nname", true);
        assert_eq!(token.span(), Span::new(11, 15));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("<!-- --> \"\r\nname", true);
        assert_eq!(token.span(), Span::new(12, 16));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("<!-- --> \"\u{2028}name", true);
        assert_eq!(token.span(), Span::new(13, 17));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("<!-- --> \"\u{2029}name", true);
        assert_eq!(token.span(), Span::new(13, 17));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("<!-- --> \"", true);
        assert_eq!(token.tag(), TokenTag::Eof);
        assert_eq!(token.span(), Span::new(10, 10));

        let token = next_after_trivia("--> comment\nname", true);
        assert_eq!(token.span(), Span::new(12, 16));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia(" \t--> comment\nname", true);
        assert_eq!(token.span(), Span::new(14, 18));
        assert!(token.has_line_terminator_before());
    }

    #[test]
    fn template() {
        let token = next_template("`plain $ value`").unwrap();
        assert_eq!(token.tag(), TokenTag::NoSubstitutionTemplate);
        assert_eq!(token.span(), Span::new(0, 15));

        let token = next_template("`head ${value}`").unwrap();
        assert_eq!(token.tag(), TokenTag::TemplateHead);
        assert_eq!(token.span(), Span::new(0, 8));

        let token = next_template("`escaped \\` value`").unwrap();
        assert_eq!(token.tag(), TokenTag::NoSubstitutionTemplate);
        assert!(token.is_escaped());

        let token = next_template("`first\r\nsecond`").unwrap();
        assert_eq!(token.tag(), TokenTag::NoSubstitutionTemplate);
        assert!(token.is_escaped());

        let allocator = Allocator::new();

        let mut lexer = Lexer::init(
            &allocator,
            "}middle ${value}",
            false,
            false,
            NoTokenParserConfig,
        );
        let token = lexer.re_scan_template_continuation(0).unwrap();
        assert_eq!(token.tag(), TokenTag::TemplateMiddle);
        assert_eq!(token.span(), Span::new(0, 10));

        let mut lexer = Lexer::init(&allocator, "}tail`", false, false, NoTokenParserConfig);
        let token = lexer.re_scan_template_continuation(0).unwrap();
        assert_eq!(token.tag(), TokenTag::TemplateTail);
        assert_eq!(token.span(), Span::new(0, 6));

        assert_eq!(
            next_template("`unterminated"),
            Err(LexicalError::NonTerminatedTemplateLiteral)
        );
    }
}
