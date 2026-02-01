use either::Either;
use swc_core::atoms::Atom;
use swc_core::common::BytePos;
use swc_experimental_ecma_ast::*;

use crate::{
    Context, PResult, Parser,
    error::SyntaxError,
    input::Tokens,
    lexer::{MaybeSubUtf8, Token},
};

impl<I: Tokens> Parser<I> {
    // https://tc39.es/ecma262/#prod-ModuleExportName
    pub(crate) fn parse_module_export_name(&mut self) -> PResult<ModuleExportName> {
        let cur = self.input().cur();
        let module_export_name = if cur == Token::Str {
            ModuleExportName::Str(self.parse_str_lit())
        } else if cur.is_word() {
            let (span, sym) = self.parse_ident_name()?;
            let sym = self.to_utf8_ref(sym);
            ModuleExportName::Ident(self.ast.ident(span, sym, false))
        } else {
            unexpected!(self, "identifier or string");
        };
        Ok(module_export_name)
    }

    /// Use this when spec says "IdentifierName".
    /// This allows idents like `catch`.
    pub(crate) fn parse_ident_name(&mut self) -> PResult<(Span, MaybeSubUtf8)> {
        let token_and_span = self.input().get_cur();
        let start = token_and_span.span.lo;
        let cur = token_and_span.token;
        let w = if cur.is_word() {
            self.input_mut().expect_word_token_and_bump()
        } else if cur == Token::JSXName && self.ctx().contains(Context::InType) {
            self.input_mut().expect_jsx_name_token_and_bump()
        } else {
            syntax_error!(self, SyntaxError::ExpectedIdent)
        };
        Ok((self.span(start), w))
    }

    pub(crate) fn parse_maybe_private_name(
        &mut self,
    ) -> PResult<Either<PrivateName, (Span, MaybeSubUtf8)>> {
        let is_private = self.input().is(Token::Hash);
        if is_private {
            self.parse_private_name().map(Either::Left)
        } else {
            self.parse_ident_name().map(Either::Right)
        }
    }

    pub(crate) fn parse_private_name(&mut self) -> PResult<PrivateName> {
        let start = self.cur_pos();
        self.assert_and_bump(Token::Hash);
        let hash_end = self.input().prev_span().hi;
        if self.input().cur_pos() - hash_end != BytePos(0) {
            syntax_error!(
                self,
                self.span(start),
                SyntaxError::SpaceBetweenHashAndIdent
            );
        }
        let (_, sym) = self.parse_ident_name()?;
        let sym = self.to_utf8_ref(sym);
        Ok(self.ast.private_name(self.span(start), sym))
    }

    /// IdentifierReference
    #[inline]
    fn parse_ident_ref(&mut self) -> PResult<Ident> {
        let ctx = self.ctx();
        self.parse_ident(
            !ctx.contains(Context::InGenerator),
            !ctx.contains(Context::InAsync),
        )
    }

    /// LabelIdentifier
    #[inline]
    pub(crate) fn parse_label_ident(&mut self) -> PResult<Ident> {
        self.parse_ident_ref()
    }

    /// Different from legacy SWC: This function returns [Ident] instead of [BindingIdent]
    /// Legacy SWC transforms [BindingIdent] back to [Ident], leading to memory hole in ast context
    /// https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#prod-BindingIdentifier
    pub(crate) fn parse_binding_ident(&mut self, disallow_let: bool) -> PResult<Ident> {
        trace_cur!(self, parse_binding_ident);

        let cur = self.input().cur();
        if disallow_let && cur == Token::Let {
            unexpected!(self, "let is reserved in const, let, class declaration")
        } else if cur == Token::Ident {
            let span = self.input().cur_span();
            let word = self.input_mut().expect_word_token_and_bump();
            let word = self.to_utf8_ref(word);

            let word_str = self.ast.get_utf8(word);
            if "arguments" == word_str || "eval" == word_str {
                self.emit_strict_mode_err(span, SyntaxError::EvalAndArgumentsInStrict);
            }

            let ident = self.ast.ident(span, word, false);
            return Ok(ident);
        }

        // "yield" and "await" is **lexically** accepted.
        let ident = self.parse_ident(true, true)?;
        let ctx = self.ctx();
        if (ctx.intersects(Context::InAsync.union(Context::InStaticBlock))
            && self.ast.get_utf8(ident.sym(&self.ast)) == "await")
            || (ctx.contains(Context::InGenerator)
                && self.ast.get_utf8(ident.sym(&self.ast)) == "yield")
        {
            self.emit_err(ident.span(&self.ast), SyntaxError::ExpectedIdent);
        }

        Ok(ident)
    }

    pub(crate) fn parse_opt_binding_ident(&mut self, disallow_let: bool) -> PResult<Option<Ident>> {
        trace_cur!(self, parse_opt_binding_ident);
        let token_and_span = self.input().get_cur();
        let cur = token_and_span.token;
        if cur == Token::This && self.input().syntax().typescript() {
            let start = token_and_span.span.lo;
            let sym = self.ast.add_utf8("this");
            let ident = self.ast.ident(self.span(start), sym, false);
            Ok(Some(ident))
        } else if cur.is_word() && !cur.is_reserved(self.ctx()) {
            self.parse_binding_ident(disallow_let).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Identifier
    ///
    /// In strict mode, "yield" is SyntaxError if matched.
    pub(crate) fn parse_ident(&mut self, incl_yield: bool, incl_await: bool) -> PResult<Ident> {
        trace_cur!(self, parse_ident);

        let token_and_span = self.input().get_cur();
        if !token_and_span.token.is_word() {
            syntax_error!(self, SyntaxError::ExpectedIdent)
        }
        let span = token_and_span.span;
        let start = span.lo;
        let t = token_and_span.token;

        // Spec:
        // It is a Syntax Error if this phrase is contained in strict mode code and the
        // StringValue of IdentifierName is: "implements", "interface", "let",
        // "package", "private", "protected", "public", "static", or "yield".
        if t == Token::Enum {
            let word = self.input_mut().expect_word_token_and_bump();
            let word = self.to_utf8_ref(word);
            let word_str = self.ast.get_utf8(word);
            self.emit_err(span, SyntaxError::InvalidIdentInStrict(Atom::new(word_str)));

            return Ok(self.ast.ident(span, word, false));
        } else if t == Token::Yield
            || t == Token::Let
            || t == Token::Static
            || t == Token::Implements
            || t == Token::Interface
            || t == Token::Package
            || t == Token::Private
            || t == Token::Protected
            || t == Token::Public
        {
            let word = self.input_mut().expect_word_token_and_bump();
            let word = self.to_utf8_ref(word);
            let word_str = self.ast.get_utf8(word);
            self.emit_strict_mode_err(span, SyntaxError::InvalidIdentInStrict(Atom::new(word_str)));

            return Ok(self.ast.ident(span, word, false));
        };

        let word;

        // Spec:
        // It is a Syntax Error if StringValue of IdentifierName is the same String
        // value as the StringValue of any ReservedWord except for yield or await.
        if t == Token::Await {
            let ctx = self.ctx();
            if ctx.contains(Context::InDeclare) {
                word = MaybeSubUtf8::new_from_span(span)
            } else if ctx.contains(Context::InStaticBlock) {
                syntax_error!(self, span, SyntaxError::ExpectedIdent)
            } else if ctx.contains(Context::Module) | ctx.contains(Context::InAsync) {
                syntax_error!(self, span, SyntaxError::InvalidIdentInAsync)
            } else if incl_await {
                word = MaybeSubUtf8::new_from_span(span)
            } else {
                syntax_error!(self, span, SyntaxError::ExpectedIdent)
            }
        } else if (t == Token::This && self.input().syntax().typescript())
            || t == Token::Let
            || t.is_known_ident()
        {
            word = MaybeSubUtf8::new_from_span(span)
        } else if t == Token::Ident {
            let word = self.input_mut().expect_word_token_and_bump();
            let word = self.to_utf8_ref(word);
            let word_str = self.ast.get_utf8(word);
            if self.ctx().contains(Context::InClassField) && word_str == "arguments" {
                self.emit_err(span, SyntaxError::ArgumentsInClassField)
            }

            return Ok(self.ast.ident(self.span(start), word, false));
        } else if t == Token::Yield && incl_yield {
            word = MaybeSubUtf8::new_from_span(span)
        } else if t == Token::Null || t == Token::True || t == Token::False || t.is_keyword() {
            syntax_error!(self, span, SyntaxError::ExpectedIdent)
        } else {
            unreachable!()
        }
        self.bump();

        let word = self.to_utf8_ref(word);
        Ok(self.ast.ident(self.span(start), word, false))
    }
}
