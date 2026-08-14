use std::hint::cold_path;

use memchr::{memchr, memchr3};
use swc_next_allocator::Allocator;
use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{
    Comment, CommentKind, CommentPosition, Span, is_ident_continue_ascii, is_ident_start_ascii,
};

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
    // UnterminatedRegex,
    NonTerminatedTemplateLiteral,
    UnterminatedRegexLiteral,
    InvalidRegexLineTerminator,
    // InvalidRegex,
    InvalidRegexFlag,
    DuplicateRegexFlag,
    IncompatibleRegexFlags,
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
    // MultipleDecimalPoints,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LexerState {
    pub token_flags: u8,
    had_line_break: bool,
    previous_token_tag: Option<TokenTag>,
    /// End offset of the most recently consumed token.
    pub(super) prev_token_end: u32,
}

impl Default for LexerState {
    fn default() -> Self {
        Self {
            token_flags: 0,
            had_line_break: true,
            previous_token_tag: None,
            prev_token_end: 0,
        }
    }
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

    pub fn next_token(&mut self, target: &mut Token) -> Result<(), LexicalError> {
        self.read_next_token(target)?;
        self.handle_token(*target, HandleTokenMode::Append);
        Ok(())
    }

    fn read_next_token(&mut self, target: &mut Token) -> Result<(), LexicalError> {
        debug_assert!((self.cursor as usize) <= self.source.len());
        self.clear_token_flags();
        self.skip_ws_and_comments()?;

        if (self.cursor as usize) >= self.source.len() {
            *target = self.create_token(TokenTag::Eof, self.cursor, self.cursor);
            return Ok(());
        }

        let current_char = self.source[self.cursor as usize];
        if is_ident_start_ascii(current_char) && self.mode == LexerMode::Normal {
            let start = self.cursor;
            let pos = scan_ascii_identifier_continue(self.source, start + 1);
            if (pos as usize) >= self.source.len()
                || (self.source[pos as usize] != b'\\' && self.source[pos as usize] < 0x80)
            {
                self.cursor = pos;
                *target = self.create_token(
                    get_keyword_type(&self.source[start as usize..pos as usize]),
                    start,
                    pos,
                );
                return Ok(());
            }
            *target = self.scan_identifier_or_keyword()?;
            return Ok(());
        }

        *target = match current_char {
            b'+' | b'*' | b'-' | b'!' | b'<' | b'>' | b'=' | b'|' | b'&' | b'^' | b'%' | b'/'
            | b'?' => self.scan_punctuation()?,
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
        Ok(())
    }

    #[inline]
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
        self.state.previous_token_tag = Some(token.tag);
        self.state.had_line_break = false;
    }

    /// Restores a compound token removed by a failed speculative rescan.
    #[inline]
    pub(crate) fn rewrite_last_collected_token(&mut self, token: Token) {
        if self.config.tokens() {
            if let Some(last) = self.tokens.last_mut() {
                debug_assert_eq!(last.span.start, token.span.start);
                *last = token;
            } else {
                debug_assert!(!self.config.tokens());
            }
        }
    }

    fn scan_simple_punctuation(&mut self) -> Token {
        debug_assert!((self.cursor as usize) < self.source.len());
        let start = self.cursor;
        let tag = match self.source[self.cursor as usize] {
            b'~' => TokenTag::BitwiseNot,
            b'(' => TokenTag::LeftParen,
            b')' => TokenTag::RightParen,
            b'{' => TokenTag::LeftBrace,
            b'[' => TokenTag::LeftBracket,
            b']' => TokenTag::RightBracket,
            b';' => TokenTag::Semicolon,
            b',' => TokenTag::Comma,
            b':' => TokenTag::Colon,
            b'@' => TokenTag::At,
            _ => unreachable!(),
        };
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
                let c = self.source[pos as usize];
                if c == quote {
                    pos += 1;
                    self.cursor = pos;
                    return Ok(self.create_token(TokenTag::StringLiteral, start, pos));
                }
                if c == b'\\' {
                    self.cursor = pos;
                    self.consume_escape(EscapeContext::String)?;
                    pos = self.cursor;
                    continue;
                }
                if c == b'\n' || c == b'\r' {
                    self.cursor = pos;
                    return Err(LexicalError::UnterminatedString);
                }
                pos += 1;
            }
        } else {
            while (pos as usize) < self.source.len() {
                if self.source[pos as usize] == quote {
                    pos += 1;
                    self.cursor = pos;
                    return Ok(self.create_token(TokenTag::StringLiteral, start, pos));
                }
                pos += 1;
            }
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

    fn consume_escape(&mut self, context: EscapeContext) -> Result<(), LexicalError> {
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

    fn scan_punctuation(&mut self) -> Result<Token, LexicalError> {
        debug_assert!((self.cursor as usize) < self.source.len());
        let start = self.cursor;
        let c0 = self.source[self.cursor as usize];
        let c1 = self.peek(1);
        let c2 = self.peek(2);
        let c3 = self.peek(3);

        let token = match c0 {
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
            b'*' if c1 == b'*' && c2 == b'=' => self.punc_token(3, TokenTag::ExponentAssign, start),
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
            b'<' if c1 == b'<' && c2 == b'=' => {
                self.punc_token(3, TokenTag::LeftShiftAssign, start)
            }
            b'<' => match c1 {
                b'<' => self.punc_token(2, TokenTag::LeftShift, start),
                b'=' => self.punc_token(2, TokenTag::LessThanEqual, start),
                _ => self.punc_token(1, TokenTag::LessThan, start),
            },
            b'>' if c1 == b'>' && c2 == b'=' => {
                self.punc_token(3, TokenTag::RightShiftAssign, start)
            }
            b'>' if c1 == b'>' && c2 == b'>' && c3 == b'=' => {
                self.punc_token(4, TokenTag::UnsignedRightShiftAssign, start)
            }
            b'>' if c1 == b'>' && c2 == b'>' => {
                self.punc_token(3, TokenTag::UnsignedRightShift, start)
            }
            b'>' => match c1 {
                b'>' if self.mode == LexerMode::JsxTag => {
                    self.punc_token(1, TokenTag::GreaterThan, start)
                }
                b'>' => self.punc_token(2, TokenTag::RightShift, start),
                b'=' => self.punc_token(2, TokenTag::GreaterThanEqual, start),
                _ => self.punc_token(1, TokenTag::GreaterThan, start),
            },
            b'=' if c1 == b'=' && c2 == b'=' => self.punc_token(3, TokenTag::StrictEqual, start),
            b'=' => match c1 {
                b'=' => self.punc_token(2, TokenTag::Equal, start),
                b'>' => self.punc_token(2, TokenTag::Arrow, start),
                _ => self.punc_token(1, TokenTag::Assign, start),
            },
            b'!' if c1 == b'=' && c2 == b'=' => self.punc_token(3, TokenTag::StrictNotEqual, start),
            b'!' => match c1 {
                b'=' => self.punc_token(2, TokenTag::NotEqual, start),
                _ => self.punc_token(1, TokenTag::LogicalNot, start),
            },
            b'&' if c1 == b'&' && c2 == b'=' => {
                self.punc_token(3, TokenTag::LogicalAndAssign, start)
            }
            b'&' => match c1 {
                b'&' => self.punc_token(2, TokenTag::LogicalAnd, start),
                b'=' => self.punc_token(2, TokenTag::BitwiseAndAssign, start),
                _ => self.punc_token(1, TokenTag::BitwiseAnd, start),
            },
            b'|' if c1 == b'|' && c2 == b'=' => {
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
            b'?' if c1 == b'?' && c2 == b'=' => self.punc_token(3, TokenTag::NullishAssign, start),
            b'?' => match c1 {
                b'?' => self.punc_token(2, TokenTag::NullishCoalescing, start),
                b'.' if c2.is_ascii_digit() => self.punc_token(1, TokenTag::Question, start),
                b'.' => self.punc_token(2, TokenTag::OptionalChaining, start),
                _ => self.punc_token(1, TokenTag::Question, start),
            },
            _ => unreachable!(),
        };
        Ok(token)
    }

    fn punc_token(&mut self, len: u32, tag: TokenTag, start: u32) -> Token {
        debug_assert!(len > 0);
        debug_assert_eq!(start, self.cursor);
        debug_assert!((start + len) as usize <= self.source.len());
        self.cursor += len;
        self.create_token(tag, start, self.cursor)
    }

    fn peek(&self, offset: u32) -> u8 {
        let idx = (self.cursor + offset) as usize;
        self.source.get(idx).copied().unwrap_or(0)
    }

    fn set_token_flag(&mut self, flag: TokenFlag) {
        self.state.token_flags |= flag_mask(flag);
    }

    fn has_token_flag(&self, flag: TokenFlag) -> bool {
        (self.state.token_flags & flag_mask(flag)) != 0
    }

    fn clear_token_flags(&mut self) {
        self.state.token_flags = 0;
    }

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
                while (self.cursor as usize) < self.source.len() {
                    match self.source[self.cursor as usize] {
                        b'<' | b'{' => break,
                        _ => self.cursor += 1,
                    }
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
                let mut flags_seen = 0u32;

                loop {
                    let flag = self.peek(0);
                    if !flag.is_ascii_alphabetic() {
                        break;
                    }
                    if !matches!(flag, b'g' | b'i' | b'm' | b's' | b'u' | b'y' | b'd' | b'v') {
                        return Err(LexicalError::InvalidRegexFlag);
                    }
                    let bit = flag - b'a';
                    if (flags_seen & (1u32 << bit)) != 0 {
                        return Err(LexicalError::DuplicateRegexFlag);
                    }
                    flags_seen |= 1u32 << bit;
                    self.cursor += 1;
                }

                let u_bit = 1u32 << (b'u' - b'a');
                let v_bit = 1u32 << (b'v' - b'a');
                if flags_seen & u_bit != 0 && flags_seen & v_bit != 0 {
                    return Err(LexicalError::IncompatibleRegexFlags);
                }

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
                        self.state.had_line_break = true;
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
                    self.state.had_line_break = true;
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

    #[inline]
    fn can_have_trailing_line_comment(&self) -> bool {
        !self.state.had_line_break
            && self.state.previous_token_tag.is_some_and(|tag| {
                (self.mode == LexerMode::JsxTag && tag == TokenTag::Slash)
                    || tag.can_have_trailing_line_comment()
            })
    }

    #[inline]
    fn can_have_trailing_block_comment(&self, had_line_break: bool, next: u32) -> bool {
        !had_line_break
            && (self.source.get(next as usize) == Some(&b';')
                || self
                    .state
                    .previous_token_tag
                    .is_some_and(TokenTag::can_have_trailing_comment))
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
        let (position, attached_to) = if token.tag == TokenTag::Eof {
            cold_path();
            if self.state.prev_token_end == 0 {
                (CommentPosition::Leading, 0)
            } else {
                (CommentPosition::Trailing, self.state.prev_token_end)
            }
        } else {
            (CommentPosition::Leading, token.span.start)
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
        self.record_comment(CommentKind::Line, start, pos as u32, trailing)?;
        self.state.had_line_break = true;
        Ok(())
    }

    fn scan_block_comment(&mut self) -> Result<(), LexicalError> {
        debug_assert!(((self.cursor + 1) as usize) < self.source.len());
        debug_assert_eq!(self.source[self.cursor as usize], b'/');
        debug_assert_eq!(self.source[self.cursor as usize + 1], b'*');
        let start = self.cursor;
        let had_line_break = self.state.had_line_break;
        let src = self.source;
        let mut pos = start as usize + 2;
        while pos < src.len() {
            let rest = &src[pos..];
            let ascii_offset = memchr3(b'*', b'\n', b'\r', rest).unwrap_or(rest.len());
            let unicode_offset = memchr(0xE2, &rest[..ascii_offset]).unwrap_or(ascii_offset);
            pos += ascii_offset.min(unicode_offset);
            if pos == src.len() {
                break;
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
                    self.state.had_line_break = true;
                    pos += 1;
                }
                0x80..=0xFF => {
                    let lt_len = unicode_separator_len(src, pos) as usize;
                    if lt_len > 0 {
                        self.set_token_flag(TokenFlag::LineTerminatorBefore);
                        self.state.had_line_break = true;
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
            if c == b'-' && self.peek(1) == b'-' && self.peek(2) == b'>' {
                self.cursor += 3;
                self.record_comment(CommentKind::Line, start, self.cursor, trailing)?;
                self.state.had_line_break = true;
                return Ok(());
            }
            if self.is_line_terminator(c) {
                break;
            }
            self.cursor += 1;
        }
        self.record_comment(CommentKind::Line, start, self.cursor, trailing)?;
        self.state.had_line_break = true;
        Ok(())
    }

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
        self.state.had_line_break = true;
        Ok(())
    }

    pub fn create_token(&mut self, tag: TokenTag, start: u32, end: u32) -> Token {
        Token {
            span: Span { start, end },
            tag,
            flags: self.consume_token_flags(),
        }
    }
}

const IDENTIFIER_BATCH_SIZE: usize = 32;

const NOT_ASCII_IDENTIFIER_CONTINUE: [bool; 256] = {
    let mut table = [true; 256];
    let mut byte = b'0' as usize;
    while byte <= b'9' as usize {
        table[byte] = false;
        byte += 1;
    }
    byte = b'A' as usize;
    while byte <= b'Z' as usize {
        table[byte] = false;
        byte += 1;
    }
    byte = b'a' as usize;
    while byte <= b'z' as usize {
        table[byte] = false;
        byte += 1;
    }
    table[b'_' as usize] = false;
    table[b'$' as usize] = false;
    table
};

#[inline(always)]
fn scan_ascii_identifier_continue(source: &[u8], start: u32) -> u32 {
    debug_assert!(u32::try_from(source.len()).is_ok());
    debug_assert!((start as usize) <= source.len());
    let mut position = start as usize;

    'search: while source.len() - position >= IDENTIFIER_BATCH_SIZE {
        let batch = &source[position..position + IDENTIFIER_BATCH_SIZE];
        for (offset, byte) in batch.iter().enumerate() {
            if NOT_ASCII_IDENTIFIER_CONTINUE[*byte as usize] {
                position += offset;
                break 'search;
            }
        }
        position += IDENTIFIER_BATCH_SIZE;
    }
    while position < source.len() && !NOT_ASCII_IDENTIFIER_CONTINUE[source[position] as usize] {
        position += 1;
    }

    debug_assert!(position >= start as usize);
    debug_assert!(position <= source.len());
    position as u32
}

fn is_surrogate_codepoint(cp: u32) -> bool {
    (0xD800..=0xDFFF).contains(&cp)
}

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
        // LexicalError::UnterminatedRegex => "Unterminated regular expression",
        LexicalError::NonTerminatedTemplateLiteral => "Unterminated template literal",
        LexicalError::UnterminatedRegexLiteral => "Unterminated regular expression literal",
        LexicalError::InvalidRegexLineTerminator => {
            "Line terminator not allowed in regular expression literal"
        }
        // LexicalError::InvalidRegex => "Invalid regular expression",
        LexicalError::InvalidRegexFlag => "Invalid regular expression flag",
        LexicalError::DuplicateRegexFlag => "Duplicate regular expression flag",
        LexicalError::IncompatibleRegexFlags => {
            "The 'u' and 'v' regular expression flags cannot be used together"
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
        // LexicalError::MultipleDecimalPoints => {
        //     "Numeric literal cannot contain multiple decimal points"
        // }
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
        // LexicalError::UnterminatedRegex => {
        //     "Try adding a closing slash (/) here to complete the regex"
        // }
        LexicalError::NonTerminatedTemplateLiteral => {
            "Try adding a closing backtick (`) here to complete the template"
        }
        LexicalError::UnterminatedRegexLiteral => {
            "Try adding a closing slash (/) here, optionally followed by flags (g, i, m, etc.)"
        }
        LexicalError::InvalidRegexLineTerminator => {
            "Try removing the line break here or escaping it within the regex pattern"
        }
        // LexicalError::InvalidRegex => {
        //     "Try checking the regex syntax here for unclosed groups, invalid escapes, or malformed patterns"
        // }
        LexicalError::InvalidRegexFlag => {
            "Valid regex flags are: `g` (global), `i` (ignoreCase), `m` (multiline), `s` (dotAll), `u` (unicode), `y` (sticky), `d` (hasIndices), `v` (setNotation)"
        }
        LexicalError::DuplicateRegexFlag => {
            "Remove the duplicate flag; each flag can only appear once"
        }
        LexicalError::IncompatibleRegexFlags => {
            "The 'u' (unicode) and 'v' (unicodeSets) flags are mutually exclusive; use one or the other"
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
        // LexicalError::MultipleDecimalPoints => "Try removing the extra decimal point here",
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
        assert_eq!(token.span, Span::new(4, 8));
        assert!(!token.has_line_terminator_before());

        let token = next_after_trivia("\r\nname", false);
        assert_eq!(token.span, Span::new(2, 6));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("// line\nname", false);
        assert_eq!(token.span, Span::new(8, 12));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("/* block\ncomment */name", false);
        assert_eq!(token.span, Span::new(19, 23));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("\u{2028}name", false);
        assert_eq!(token.span, Span::new(3, 7));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("\u{00A0}name", false);
        assert_eq!(token.span, Span::new(2, 6));
        assert!(!token.has_line_terminator_before());

        let token = next_after_trivia("<!-- comment\nname", true);
        assert_eq!(token.span, Span::new(13, 17));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia("--> comment\nname", true);
        assert_eq!(token.span, Span::new(12, 16));
        assert!(token.has_line_terminator_before());

        let token = next_after_trivia(" \t--> comment\nname", true);
        assert_eq!(token.span, Span::new(14, 18));
        assert!(token.has_line_terminator_before());
    }

    #[test]
    fn template() {
        let token = next_template("`plain $ value`").unwrap();
        assert_eq!(token.tag, TokenTag::NoSubstitutionTemplate);
        assert_eq!(token.span, Span::new(0, 15));

        let token = next_template("`head ${value}`").unwrap();
        assert_eq!(token.tag, TokenTag::TemplateHead);
        assert_eq!(token.span, Span::new(0, 8));

        let token = next_template("`escaped \\` value`").unwrap();
        assert_eq!(token.tag, TokenTag::NoSubstitutionTemplate);
        assert!(token.is_escaped());

        let token = next_template("`first\r\nsecond`").unwrap();
        assert_eq!(token.tag, TokenTag::NoSubstitutionTemplate);
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
        assert_eq!(token.tag, TokenTag::TemplateMiddle);
        assert_eq!(token.span, Span::new(0, 10));

        let mut lexer = Lexer::init(&allocator, "}tail`", false, false, NoTokenParserConfig);
        let token = lexer.re_scan_template_continuation(0).unwrap();
        assert_eq!(token.tag, TokenTag::TemplateTail);
        assert_eq!(token.span, Span::new(0, 6));

        assert_eq!(
            next_template("`unterminated"),
            Err(LexicalError::NonTerminatedTemplateLiteral)
        );
    }
}
