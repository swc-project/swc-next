use std::mem::take;

use swc_core::atoms::wtf8::{CodePoint, Wtf8};
use swc_experimental_ecma_ast::{EsVersion, Span};
// use swc_core::atoms::wtf8::CodePoint;
use swc_core::common::BytePos;

use super::{Context, Lexer};
use crate::{
    error::{Error, SyntaxError},
    input::Tokens,
    lexer::{
        LexResult,
        char_ext::CharExt,
        comments_buffer::{BufferedCommentKind, CommentsBufferCheckpoint},
        token::{Token, TokenAndSpan, TokenValue},
    },
    string_alloc::{MaybeSubUtf8, MaybeSubWtf8},
    syntax::SyntaxFlags,
};

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct TokenFlags: u8 {
        const UNICODE = 1 << 0;
    }
}

/// State of lexer.
///
/// Ported from babylon.
#[derive(Clone)]
pub struct State {
    /// if line break exists between previous token and new token?
    pub had_line_break: bool,
    pub next_regexp: Option<BytePos>,
    pub prev_hi: BytePos,

    pub(super) token_value: Option<TokenValue>,
    token_type: Option<Token>,
}

pub struct LexerCheckpoint {
    comments_buffer: CommentsBufferCheckpoint,
    state: State,
    ctx: Context,
    input_cur_pos: BytePos,
}

impl crate::input::Tokens for Lexer<'_> {
    type Checkpoint = LexerCheckpoint;

    fn checkpoint_save(&self) -> LexerCheckpoint {
        LexerCheckpoint {
            state: self.state.clone(),
            ctx: self.ctx,
            input_cur_pos: self.input.cur_pos(),
            comments_buffer: self
                .comments_buffer
                .as_ref()
                .map(|cb| cb.checkpoint_save())
                .unwrap_or_default(),
        }
    }

    fn checkpoint_load(&mut self, checkpoint: LexerCheckpoint) {
        self.state = checkpoint.state;
        self.ctx = checkpoint.ctx;
        unsafe { self.input.reset_to(checkpoint.input_cur_pos) };
        if let Some(comments_buffer) = self.comments_buffer.as_mut() {
            comments_buffer.checkpoint_load(checkpoint.comments_buffer);
        }
    }

    #[inline]
    fn get_maybe_sub_utf8(&self, maybe: MaybeSubUtf8) -> &str {
        match maybe {
            MaybeSubUtf8::Empty => "",
            MaybeSubUtf8::Inline((start, end)) => unsafe { self.input.slice(start, end) },
            MaybeSubUtf8::Alloc((start, end)) => {
                self.string_allocator.get_allocated_utf8(start, end)
            }
        }
    }

    #[inline]
    fn get_maybe_sub_wtf8(&self, maybe: MaybeSubWtf8) -> &Wtf8 {
        match maybe {
            MaybeSubWtf8::Empty => Wtf8::from_str(""),
            MaybeSubWtf8::Inline((start, end)) => {
                Wtf8::from_str(unsafe { self.input.slice(start, end) })
            }
            MaybeSubWtf8::Alloc((start, end)) => {
                self.string_allocator.get_allocated_wtf8(start, end)
            }
        }
    }

    #[inline]
    fn set_ctx(&mut self, ctx: Context) {
        self.ctx = ctx
    }

    #[inline]
    fn ctx(&self) -> Context {
        self.ctx
    }

    #[inline]
    fn ctx_mut(&mut self) -> &mut Context {
        &mut self.ctx
    }

    #[inline]
    fn syntax(&self) -> SyntaxFlags {
        self.syntax
    }

    #[inline]
    fn target(&self) -> EsVersion {
        self.target
    }

    #[inline]
    fn start_pos(&self) -> BytePos {
        self.start_pos
    }

    #[inline]
    fn set_next_regexp(&mut self, start: Option<BytePos>) {
        self.state.next_regexp = start;
    }

    fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn add_module_mode_error(&mut self, error: Error) {
        if self.ctx.contains(Context::Module) {
            self.add_error(error);
            return;
        }
        self.module_errors.push(error);
    }

    #[inline]
    fn take_errors(&mut self) -> Vec<Error> {
        let mut errors = take(&mut self.errors);
        if self.ctx.contains(Context::Module) && !self.module_errors.is_empty() {
            errors.append(&mut self.module_errors);
        }
        errors
    }

    #[inline]
    fn take_script_module_errors(&mut self) -> Vec<Error> {
        take(&mut self.module_errors)
    }

    #[inline]
    fn end_pos(&self) -> BytePos {
        self.input.end_pos()
    }

    #[inline]
    fn update_token_flags(&mut self, f: impl FnOnce(&mut TokenFlags)) {
        f(&mut self.token_flags)
    }

    #[inline]
    fn token_flags(&self) -> TokenFlags {
        self.token_flags
    }

    fn get_token_value(&self) -> Option<&TokenValue> {
        self.state.token_value.as_ref()
    }

    fn set_token_value(&mut self, token_value: Option<TokenValue>) {
        self.state.token_value = token_value;
    }

    fn take_token_value(&mut self) -> Option<TokenValue> {
        self.state.token_value.take()
    }

    fn first_token(&mut self) -> TokenAndSpan {
        let mut start = self.cur_pos();
        let token = match self.read_shebang() {
            Ok(Some(shebang)) => {
                self.state.set_token_value(TokenValue::Word(shebang));
                Ok(Token::Shebang)
            }
            // Fallback to other tokens
            Ok(None) => {
                self.state.had_line_break = true;
                self.skip_space();
                start = self.input.cur_pos();
                self.read_token()
            }
            Err(error) => Err(error),
        };

        let token = match token {
            Ok(token) => token,
            Err(error) => {
                self.state.set_token_value(TokenValue::Error(error));
                Token::Error
            }
        };
        self.finish_next_token(self.span(start), token)
    }

    fn next_token(&mut self) -> TokenAndSpan {
        let mut start = self.cur_pos();

        let token = match self.read_next_token(&mut start) {
            Ok(res) => res,
            Err(error) => {
                self.state.set_token_value(TokenValue::Error(error));
                Token::Error
            }
        };

        self.finish_next_token(self.span(start), token)
    }

    fn rescan_jsx_token(&mut self, allow_multiline_jsx_text: bool, reset: BytePos) -> TokenAndSpan {
        unsafe {
            self.input.reset_to(reset);
        }
        Tokens::scan_jsx_token(self, allow_multiline_jsx_text)
    }

    fn rescan_jsx_open_el_terminal_token(&mut self, reset: BytePos) -> TokenAndSpan {
        unsafe {
            self.input.reset_to(reset);
        }
        Tokens::scan_jsx_open_el_terminal_token(self)
    }

    fn scan_jsx_token(&mut self, allow_multiline_jsx_text: bool) -> TokenAndSpan {
        let start = self.cur_pos();
        let res = match self.scan_jsx_token(allow_multiline_jsx_text) {
            Ok(res) => Ok(res),
            Err(error) => {
                self.state.set_token_value(TokenValue::Error(error));
                Err(Token::Error)
            }
        };
        let token = match res {
            Ok(t) => t,
            Err(e) => e,
        };
        self.finish_next_token(self.span(start), token)
    }

    fn scan_jsx_open_el_terminal_token(&mut self) -> TokenAndSpan {
        self.skip_space();
        let start = self.input.cur_pos();
        let res = match self.scan_jsx_attrs_terminal_token() {
            Ok(res) => Ok(res),
            Err(error) => {
                self.state.set_token_value(TokenValue::Error(error));
                Err(Token::Error)
            }
        };
        let token = match res {
            Ok(t) => t,
            Err(e) => e,
        };
        self.finish_next_token(self.span(start), token)
    }

    fn scan_jsx_identifier(&mut self, start: BytePos) -> TokenAndSpan {
        let token = self.state.token_type.unwrap();
        debug_assert!(token.is_word());
        let mut v = String::with_capacity(16);
        while let Some(ch) = self.input().peek() {
            if ch == b'-' {
                v.push(ch as char);
                self.bump(1);
            } else {
                let old_pos = self.cur_pos();
                v.push_str(&self.scan_identifier_parts());
                if self.cur_pos() == old_pos {
                    break;
                }
            }
        }
        let v = if !v.is_empty() {
            let mut builder = self.string_allocator.alloc_utf8();
            if token.is_known_ident() || token.is_keyword() {
                builder.push_str(&mut self.string_allocator, &token.to_string());
                builder.push_str(&mut self.string_allocator, &v);
            } else if let Some(TokenValue::Word(value)) = self.state.token_value.take() {
                let value = self.get_maybe_sub_utf8(value).to_string();
                builder.push_str(&mut self.string_allocator, &value);
                builder.push_str(&mut self.string_allocator, &v);
            } else {
                builder.push_str(&mut self.string_allocator, &token.to_string());
                builder.push_str(&mut self.string_allocator, &v);
            };
            builder.finish(&mut self.string_allocator)
        } else if token.is_known_ident() || token.is_keyword() {
            let mut builder = self.string_allocator.alloc_utf8();
            builder.push_str(&mut self.string_allocator, &token.to_string());
            builder.finish(&mut self.string_allocator)
        } else if let Some(TokenValue::Word(value)) = self.state.token_value.take() {
            value
        } else {
            unreachable!(
                "`token_value` should be a word, but got: {:?}",
                self.state.token_value
            )
        };
        self.state.set_token_value(TokenValue::Word(v));
        TokenAndSpan {
            token: Token::JSXName,
            had_line_break: self.state.had_line_break,
            span: self.span(start),
        }
    }

    fn scan_jsx_attribute_value(&mut self) -> TokenAndSpan {
        let Some(cur) = self.peek() else {
            let start = self.cur_pos();
            return TokenAndSpan {
                token: Token::Eof,
                had_line_break: self.state.had_line_break,
                span: self.span(start),
            };
        };
        let start = self.cur_pos();

        match cur {
            b'\'' | b'"' => {
                let token = self.read_jsx_str(cur);
                let token = match token {
                    Ok(token) => token,
                    Err(e) => {
                        self.state.set_token_value(TokenValue::Error(e));
                        return TokenAndSpan {
                            token: Token::Error,
                            had_line_break: self.state.had_line_break,
                            span: self.span(start),
                        };
                    }
                };
                debug_assert!(
                    self.get_token_value()
                        .is_some_and(|t| matches!(t, TokenValue::Str { .. }))
                );
                debug_assert!(token == Token::Str);
                TokenAndSpan {
                    token,
                    had_line_break: self.state.had_line_break,
                    span: self.span(start),
                }
            }
            _ => self.next_token(),
        }
    }

    fn rescan_template_token(
        &mut self,
        start: BytePos,
        start_with_back_tick: bool,
    ) -> TokenAndSpan {
        unsafe { self.input.reset_to(start) };
        let res = self.scan_template_token(start, start_with_back_tick);
        let token = match res.map_err(|e| {
            self.state.set_token_value(TokenValue::Error(e));
            Token::Error
        }) {
            Ok(t) => t,
            Err(e) => e,
        };
        let span = if start_with_back_tick {
            self.span(start)
        } else {
            // `+ BytePos(1)` is used to skip `{`
            self.span(start + BytePos(1))
        };
        self.finish_next_token(span, token)
    }
}

impl Lexer<'_> {
    fn read_next_token(&mut self, start: &mut BytePos) -> Result<Token, Error> {
        if let Some(next_regexp) = self.state.next_regexp {
            *start = next_regexp;
            return self.read_regexp(next_regexp);
        }

        self.state.had_line_break = false;
        self.skip_space();
        *start = self.input.cur_pos();

        self.read_token()
    }

    #[inline(always)]
    fn finish_next_token(&mut self, span: Span, token: Token) -> TokenAndSpan {
        if token == Token::Eof {
            self.consume_pending_comments();
        } else if let Some(comments) = self.comments_buffer.as_mut() {
            comments.pending_to_comment(BufferedCommentKind::Leading, span.lo);
        }

        self.state.set_token_type(token);
        self.state.prev_hi = self.last_pos();
        TokenAndSpan {
            token,
            had_line_break: self.state.had_line_break,
            span,
        }
    }

    fn scan_jsx_token(&mut self, allow_multiline_jsx_text: bool) -> Result<Token, Error> {
        debug_assert!(self.syntax.jsx());

        if self.input_mut().as_str().is_empty() {
            return Ok(Token::Eof);
        };

        if self.input.eat_byte(b'<') {
            return Ok(if self.input.eat_byte(b'/') {
                Token::LessSlash
            } else {
                Token::Lt
            });
        } else if self.input.eat_byte(b'{') {
            return Ok(Token::LBrace);
        }

        let start = self.input.cur_pos();
        let mut first_non_whitespace = 0;
        let mut chunk_start = start;
        let mut value = self.string_allocator.alloc_utf8();

        while let Some(ch) = self.input_mut().peek() {
            if ch == b'{' {
                break;
            } else if ch == b'<' {
                // TODO: check git conflict mark
                break;
            }

            if ch == b'>' {
                self.emit_error(
                    self.input().cur_pos(),
                    SyntaxError::UnexpectedTokenWithSuggestions {
                        candidate_list: vec!["`{'>'}`", "`&gt;`"],
                    },
                );
            } else if ch == b'}' {
                self.emit_error(
                    self.input().cur_pos(),
                    SyntaxError::UnexpectedTokenWithSuggestions {
                        candidate_list: vec!["`{'}'}`", "`&rbrace;`"],
                    },
                );
            }

            if first_non_whitespace == 0 && ch.is_line_terminator() {
                first_non_whitespace = -1;
            } else if !allow_multiline_jsx_text
                && ch.is_line_terminator()
                && first_non_whitespace > 0
            {
                break;
            } else if (ch as char).is_whitespace() {
                first_non_whitespace = self.cur_pos().0 as i32;
            }

            if ch == b'&' {
                let s = unsafe {
                    // Safety: We already checked for the range
                    self.input_slice_to_cur(chunk_start)
                };
                value.push_str(&mut self.string_allocator, s);

                if let Ok(jsx_entity) = self.read_jsx_entity() {
                    value.push(&mut self.string_allocator, jsx_entity.0);
                    chunk_start = self.input.cur_pos();
                }
            } else {
                self.bump(1);
            }
        }

        let value = if value.is_empty(&self.string_allocator) {
            MaybeSubUtf8::new_from_source(start, self.cur_pos())
        } else {
            let s = unsafe {
                // Safety: We already checked for the range
                self.input_slice_to_cur(chunk_start)
            };
            value.push_str(&mut self.string_allocator, s);
            value.finish(&mut self.string_allocator)
        };

        self.state.set_token_value(TokenValue::JsxText(value));
        Ok(Token::JSXText)
    }

    fn scan_jsx_attrs_terminal_token(&mut self) -> LexResult<Token> {
        if self.input.eat_byte(b'>') {
            Ok(Token::Gt)
        } else if self.input.eat_byte(b'/') {
            Ok(Token::Slash)
        } else {
            self.read_token()
        }
    }

    fn scan_identifier_parts(&mut self) -> String {
        let mut v = String::with_capacity(16);
        while let Some(ch) = self.input().peek() {
            if ch.is_ident_part() {
                v.push(ch as char);
                self.bump(1);
            } else if ch == b'\\' {
                self.bump(1); // bump '\'
                if !self.is(b'u') {
                    self.emit_error(self.cur_pos(), SyntaxError::InvalidUnicodeEscape);
                    continue;
                }
                self.bump(1); // bump 'u'
                let Ok(value) = self.read_unicode_escape() else {
                    self.emit_error(self.cur_pos(), SyntaxError::InvalidUnicodeEscape);
                    break;
                };
                if let Some(c) = CodePoint::from(value).to_char() {
                    v.push(c);
                } else {
                    self.emit_error(self.cur_pos(), SyntaxError::InvalidUnicodeEscape);
                }
                self.token_flags |= TokenFlags::UNICODE;
            } else {
                break;
            }
        }
        v
    }
}

impl State {
    pub fn new(start_pos: BytePos) -> Self {
        State {
            had_line_break: false,
            next_regexp: None,
            prev_hi: start_pos,
            token_value: None,
            token_type: None,
        }
    }

    pub(crate) fn set_token_value(&mut self, token_value: TokenValue) {
        self.token_value = Some(token_value);
    }
}

impl State {
    #[inline(always)]
    pub fn set_token_type(&mut self, token_type: Token) {
        self.token_type = Some(token_type);
    }

    #[inline(always)]
    pub fn token_type(&self) -> Option<Token> {
        self.token_type
    }

    pub fn can_have_trailing_line_comment(&self) -> bool {
        let Some(t) = self.token_type() else {
            return true;
        };
        !t.is_bin_op()
    }

    pub fn can_have_trailing_comment(&self) -> bool {
        self.token_type().is_some_and(|t| {
            !t.is_keyword()
                && (t == Token::Semi
                    || t == Token::LBrace
                    || t.is_other_and_can_have_trailing_comment())
        })
    }
}
