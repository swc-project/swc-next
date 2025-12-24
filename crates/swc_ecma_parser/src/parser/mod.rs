#![allow(clippy::let_unit_value)]
#![deny(non_snake_case)]

use swc_core::common::{BytePos, Span, comments::Comments};
use swc_experimental_ecma_ast::*;

use crate::{
    Context, ParseRet, Syntax,
    error::SyntaxError,
    input::Buffer,
    lexer::{Token, TokenAndSpan, source::StringSource},
    parser::{
        input::Tokens,
        state::{State, WithState},
        util::{ExprExt, ScratchIndex},
    },
    string_alloc::{MaybeSubUtf8, MaybeSubWtf8},
    syntax::SyntaxFlags,
};

use crate::error::Error;

#[macro_use]
mod macros;
pub mod input;
mod js;
mod jsx;
mod state;
// #[cfg(feature = "typescript")]
// mod typescript;
pub(crate) mod util;

pub type PResult<T> = Result<T, crate::error::Error>;

#[allow(unused)]
pub struct ParserCheckpoint<I: Tokens> {
    lexer: I::Checkpoint,
    buffer_prev_span: Span,
    buffer_cur: TokenAndSpan,
    buffer_next: Option<crate::lexer::NextTokenAndSpan>,
}

/// EcmaScript parser.
pub struct Parser<I: self::input::Tokens> {
    ast: Ast,
    /// Don't mutable this directly. Use Parser::scratch_xxx instead for safety.
    scratch: Vec<NodeId>,
    state: State,
    input: self::input::Buffer<I>,
    found_module_item: bool,
}

impl<I: Tokens> Parser<I> {
    #[inline(always)]
    pub fn input(&self) -> &Buffer<I> {
        &self.input
    }

    #[inline(always)]
    pub fn input_mut(&mut self) -> &mut Buffer<I> {
        &mut self.input
    }

    #[inline(always)]
    fn state(&self) -> &State {
        &self.state
    }

    #[inline(always)]
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    #[allow(unused)]
    fn checkpoint_save(&self) -> ParserCheckpoint<I> {
        ParserCheckpoint {
            lexer: self.input.iter.checkpoint_save(),
            buffer_cur: self.input.cur,
            buffer_next: self.input.next.clone(),
            buffer_prev_span: self.input.prev_span,
        }
    }

    #[allow(unused)]
    fn checkpoint_load(&mut self, checkpoint: ParserCheckpoint<I>) {
        self.input.iter.checkpoint_load(checkpoint.lexer);
        self.input.cur = checkpoint.buffer_cur;
        self.input.next = checkpoint.buffer_next;
        self.input.prev_span = checkpoint.buffer_prev_span;
    }

    #[inline(always)]
    fn mark_found_module_item(&mut self) {
        self.found_module_item = true;
    }

    #[inline]
    fn scratch_start<
        N: NodeIdTrait,
        F: FnOnce(&mut Self, &mut ScratchIndex<N>) -> PResult<R>,
        R,
    >(
        &mut self,
        f: F,
    ) -> PResult<TypedSubRange<N>> {
        let mut scratch = ScratchIndex::new(self.scratch.len());
        let ret = f(self, &mut scratch);
        let range = scratch.end(self);
        ret.map(|_| range)
    }

    #[inline]
    fn to_utf8_ref(&mut self, maybe: MaybeSubUtf8) -> Utf8Ref {
        self.ast.add_utf8(self.input.iter.get_maybe_sub_utf8(maybe))
    }

    #[inline]
    fn to_wtf8_ref(&mut self, maybe: MaybeSubWtf8) -> Wtf8Ref {
        self.ast.add_wtf8(self.input.iter.get_maybe_sub_wtf8(maybe))
    }
}

impl<'a> Parser<crate::lexer::Lexer<'a>> {
    pub fn new(
        syntax: Syntax,
        input: StringSource<'a>,
        comments: Option<&'a dyn Comments>,
    ) -> Self {
        let lexer = crate::lexer::Lexer::new(syntax, Default::default(), input, comments);
        Self::new_from(lexer)
    }
}

impl<I: Tokens> Parser<I> {
    pub fn new_from(mut input: I) -> Self {
        let in_declare = input.syntax().dts();
        let mut ctx = input.ctx() | Context::TopLevel;
        ctx.set(Context::InDeclare, in_declare);
        input.set_ctx(ctx);

        let mut p = Parser {
            ast: Ast::new((input.end_pos().0 - input.start_pos().0) as usize),
            scratch: Vec::new(),
            state: Default::default(),
            input: crate::parser::input::Buffer::new(input),
            found_module_item: false,
        };

        // consume EOF
        p.input.bump();
        // This is a workaround to make comments work
        if p.input.cur.token == Token::Eof {
            p.input.cur.span = Span::new_with_checked(BytePos(0), BytePos(0));
        }

        p
    }

    pub fn take_errors(&mut self) -> Vec<Error> {
        self.input.iter.take_errors()
    }

    pub fn take_script_module_errors(&mut self) -> Vec<Error> {
        self.input.iter.take_script_module_errors()
    }

    pub fn parse_script(mut self) -> PResult<ParseRet<Script, I>> {
        trace_cur!(self, parse_script);

        let ctx = (self.ctx() & !Context::Module) | Context::TopLevel;
        self.set_ctx(ctx);

        let start = self.cur_pos();
        let shebang = self.parse_shebang()?;
        let body = self.parse_stmt_block_body(true, None)?;
        let ret = self.ast.script(self.span(start), body, shebang);

        debug_assert!(self.input().cur() == Token::Eof);
        self.input_mut().bump();

        let errors = self.take_errors();
        Ok(ParseRet {
            ast: self.ast,
            root: ret,
            input: self.input.iter,
            errors,
        })
    }

    pub fn parse_commonjs(mut self) -> PResult<ParseRet<Script, I>> {
        trace_cur!(self, parse_commonjs);

        // CommonJS module is acctually in a function scope
        let ctx = (self.ctx() & !Context::Module)
            | Context::InFunction
            | Context::InsideNonArrowFunctionScope;
        self.set_ctx(ctx);

        let start = self.cur_pos();
        let shebang = self.parse_shebang()?;

        let body = self.parse_stmt_block_body(true, None)?;
        let ret = self.ast.script(self.span(start), body, shebang);

        debug_assert!(self.input().cur() == Token::Eof);
        self.input_mut().bump();

        let errors = self.take_errors();
        Ok(ParseRet {
            ast: self.ast,
            root: ret,
            input: self.input.iter,
            errors,
        })
    }

    pub fn parse_typescript_module(mut self) -> PResult<ParseRet<Module, I>> {
        trace_cur!(self, parse_typescript_module);

        debug_assert!(self.syntax().typescript());

        //TODO: parse() -> PResult<Program>
        let ctx = (self.ctx() | Context::Module | Context::TopLevel) & !Context::Strict;
        // Module code is always in strict mode
        self.set_ctx(ctx);

        let start = self.cur_pos();
        let shebang = self.parse_shebang()?;

        let body = self.parse_module_item_block_body(true, None)?;
        let ret = self.ast.module(self.span(start), body, shebang);

        debug_assert!(self.input().cur() == Token::Eof);
        self.input_mut().bump();

        let errors = self.take_errors();
        Ok(ParseRet {
            ast: self.ast,
            root: ret,
            input: self.input.iter,
            errors,
        })
    }

    /// Returns [Module] if it's a module and returns [Script] if it's not a
    /// module.
    ///
    /// Note: This is not perfect yet. It means, some strict mode violations may
    /// not be reported even if the method returns [Module].
    pub fn parse_program(mut self) -> PResult<ParseRet<Program, I>> {
        let start = self.cur_pos();
        let shebang = self.parse_shebang()?;

        let body = self
            .do_inside_of_context(Context::CanBeModule.union(Context::TopLevel), |p| {
                p.parse_module_item_block_body(true, None)
            })?;
        let has_module_item = self.found_module_item
            || body
                .iter()
                .map(|id| self.ast.get_node_in_sub_range(id))
                .any(|item| matches!(item, ModuleItem::ModuleDecl(..)));
        if has_module_item && !self.ctx().contains(Context::Module) {
            let ctx = self.ctx()
                | Context::Module
                | Context::CanBeModule
                | Context::TopLevel
                | Context::Strict;
            // Emit buffered strict mode / module code violations
            self.input.set_ctx(ctx);
        }

        let ret = if has_module_item {
            self.ast.program_module(self.span(start), body, shebang)
        } else {
            let body = self.scratch_start(|p, stmts| {
                for item in body.iter() {
                    let item = p.ast.get_node_in_sub_range(item);
                    match item {
                        ModuleItem::Stmt(stmt) => stmts.push(p, stmt),
                        ModuleItem::ModuleDecl(_) => {
                            unreachable!("module is handled above")
                        }
                        #[cfg(swc_ast_unknown)]
                        _ => unreachable!(),
                    }
                }
                Ok(())
            })?;
            self.ast.program_script(self.span(start), body, shebang)
        };

        debug_assert!(self.input().cur() == Token::Eof);
        self.input_mut().bump();

        let errors = self.take_errors();
        Ok(ParseRet {
            ast: self.ast,
            root: ret,
            input: self.input.iter,
            errors,
        })
    }

    pub fn parse_module(mut self) -> PResult<ParseRet<Module, I>> {
        let ctx = self.ctx()
            | Context::Module
            | Context::CanBeModule
            | Context::TopLevel
            | Context::Strict;
        // Module code is always in strict mode
        self.set_ctx(ctx);

        let start = self.cur_pos();
        let shebang = self.parse_shebang()?;

        let body = self.parse_module_item_block_body(true, None)?;
        let ret = self.ast.module(self.span(start), body, shebang);

        debug_assert!(self.input().cur() == Token::Eof);
        self.input_mut().bump();

        let errors = self.take_errors();
        Ok(ParseRet {
            ast: self.ast,
            root: ret,
            input: self.input.iter,
            errors,
        })
    }

    pub fn parse_expr(mut self) -> PResult<ParseRet<Expr, I>> {
        // This allow to parse `import.meta`
        let ctx = self.ctx();
        self.set_ctx(ctx.union(Context::CanBeModule));

        let expr = self.parse_expr_inner()?;
        let errors = self.take_errors();
        Ok(ParseRet {
            ast: self.ast,
            root: expr,
            input: self.input.iter,
            errors,
        })
    }
}

impl<I: Tokens> Parser<I> {
    #[inline(always)]
    pub fn with_state<'w>(&'w mut self, state: State) -> WithState<'w, I> {
        let orig_state = std::mem::replace(self.state_mut(), state);
        WithState {
            orig_state,
            inner: self,
        }
    }

    #[inline(always)]
    pub fn ctx(&self) -> Context {
        self.input().get_ctx()
    }

    #[inline(always)]
    pub fn set_ctx(&mut self, ctx: Context) {
        self.input_mut().set_ctx(ctx);
    }

    #[inline]
    pub fn do_inside_of_context<T>(
        &mut self,
        context: Context,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let ctx = self.ctx();
        let new_ctx = ctx.union(context);
        self.set_ctx(new_ctx);
        let result = f(self);
        self.set_ctx(ctx);
        result
    }

    #[inline]
    pub fn do_outside_of_context<T>(
        &mut self,
        context: Context,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let ctx = self.ctx();
        let new_ctx = ctx.difference(context);
        self.set_ctx(new_ctx);
        let result = f(self);
        self.set_ctx(ctx);
        result
    }

    #[inline(always)]
    pub fn strict_mode<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.do_inside_of_context(Context::Strict, f)
    }

    /// Original context is restored when returned guard is dropped.
    #[inline(always)]
    pub fn in_type<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.do_inside_of_context(Context::InType, f)
    }

    #[inline(always)]
    pub fn allow_in_expr<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.do_inside_of_context(Context::IncludeInExpr, f)
    }

    #[inline(always)]
    pub fn disallow_in_expr<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.do_outside_of_context(Context::IncludeInExpr, f)
    }

    #[inline(always)]
    pub fn syntax(&self) -> SyntaxFlags {
        self.input().syntax()
    }

    #[cold]
    pub fn emit_err(&mut self, span: Span, error: SyntaxError) {
        if self.ctx().contains(Context::IgnoreError) || !self.syntax().early_errors() {
            return;
        }
        self.emit_error(crate::error::Error::new(span, error))
    }

    #[cold]
    pub fn emit_error(&mut self, error: crate::error::Error) {
        if self.ctx().contains(Context::IgnoreError) || !self.syntax().early_errors() {
            return;
        }
        let cur = self.input().cur();
        if cur == Token::Error {
            let err = self.input_mut().expect_error_token_and_bump();
            self.input_mut().iter_mut().add_error(err);
        }
        self.input_mut().iter_mut().add_error(error);
    }

    #[cold]
    pub fn emit_strict_mode_err(&mut self, span: Span, error: SyntaxError) {
        if self.ctx().contains(Context::IgnoreError) {
            return;
        }
        let error = crate::error::Error::new(span, error);
        if self.ctx().contains(Context::Strict) {
            self.input_mut().iter_mut().add_error(error);
        } else {
            self.input_mut().iter_mut().add_module_mode_error(error);
        }
    }

    pub fn verify_expr(&mut self, expr: Expr) -> PResult<Expr> {
        #[cfg(feature = "verify")]
        {
            use swc_ecma_visit::Visit;
            let mut v = self::verifier::Verifier { errors: Vec::new() };
            v.visit_expr(&expr);
            for (span, error) in v.errors {
                self.emit_err(span, error);
            }
        }
        Ok(expr)
    }

    #[inline(always)]
    pub fn cur_pos(&self) -> BytePos {
        self.input().cur_pos()
    }

    #[inline(always)]
    pub fn last_pos(&self) -> BytePos {
        self.input().prev_span().hi
    }

    #[inline]
    pub fn is_general_semi(&mut self) -> bool {
        let cur = self.input().cur();
        matches!(cur, Token::Semi | Token::RBrace | Token::Eof)
            || self.input().had_line_break_before_cur()
    }

    pub fn eat_general_semi(&mut self) -> bool {
        if cfg!(feature = "debug") {
            tracing::trace!("eat(';'): cur={:?}", self.input().cur());
        }
        let cur = self.input().cur();
        if cur == Token::Semi {
            self.bump();
            true
        } else {
            cur == Token::RBrace || self.input().had_line_break_before_cur() || cur == Token::Eof
        }
    }

    #[inline]
    pub fn expect_general_semi(&mut self) -> PResult<()> {
        if !self.eat_general_semi() {
            let span = self.input().cur_span();
            let cur = self.input_mut().dump_cur();
            syntax_error!(self, span, SyntaxError::Expected(";".to_string(), cur))
        }
        Ok(())
    }

    #[inline]
    pub fn expect(&mut self, t: Token) -> PResult<()> {
        if !self.input_mut().eat(t) {
            let span = self.input().cur_span();
            let cur = self.input_mut().dump_cur();
            syntax_error!(self, span, SyntaxError::Expected(format!("{t:?}"), cur))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn expect_without_advance(&mut self, t: Token) -> PResult<()> {
        if !self.input_mut().is(t) {
            let span = self.input().cur_span();
            let cur = self.input_mut().dump_cur();
            syntax_error!(self, span, SyntaxError::Expected(format!("{t:?}"), cur))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn bump(&mut self) {
        debug_assert!(
            self.input().cur() != Token::Eof,
            "parser should not call bump() without knowing current token"
        );
        self.input_mut().bump()
    }

    #[inline]
    pub fn span(&self, start: BytePos) -> Span {
        let end = self.last_pos();
        debug_assert!(
            start <= end,
            "assertion failed: (span.start <= span.end). start = {start:?}, end = {end:?}",
        );
        Span::new_with_checked(start, end)
    }

    #[inline(always)]
    pub fn assert_and_bump(&mut self, token: Token) {
        debug_assert!(
            self.input().is(token),
            "assertion failed: expected {token:?}, got {:?}",
            self.input().cur()
        );
        self.bump();
    }

    pub fn check_assign_target(&mut self, expr: Expr, _deny_call: bool) {
        if !expr.is_valid_simple_assignment_target(&self.ast, self.ctx().contains(Context::Strict))
        {
            self.emit_err(expr.span(&self.ast), SyntaxError::TS2406);
        }

        // We follow behavior of tsc
        // if self.input().syntax().typescript() && self.syntax().early_errors() {
        //     let is_eval_or_arguments = match expr {
        //         Expr::Ident(i) => i.is_reserved_in_strict_bind(&self.ast),
        //         _ => false,
        //     };

        //     if is_eval_or_arguments {
        //         self.emit_strict_mode_err(expr.span(&self.ast), SyntaxError::TS1100);
        //     }

        //     fn should_deny(e: &Expr, deny_call: bool) -> bool {
        //         match e {
        //             Expr::Lit(..) => false,
        //             Expr::Call(..) => deny_call,
        //             Expr::Bin(..) => false,
        //             Expr::Paren(ref p) => should_deny(&p.expr, deny_call),

        //             _ => true,
        //         }
        //     }

        //     // It is an early Reference Error if LeftHandSideExpression is neither
        //     // an ObjectLiteral nor an ArrayLiteral and
        //     // IsValidSimpleAssignmentTarget of LeftHandSideExpression is false.
        //     if !is_eval_or_arguments
        //         && !expr.is_valid_simple_assignment_target(self.ctx().contains(Context::Strict))
        //         && should_deny(expr, deny_call)
        //     {
        //         self.emit_err(expr.span(&self.ast), SyntaxError::TS2406);
        //     }
        // }
    }

    #[inline]
    pub fn is_ident_ref(&mut self) -> bool {
        let cur = self.input().cur();
        cur.is_word() && !cur.is_reserved(self.ctx())
    }

    #[inline]
    pub fn peek_is_ident_ref(&mut self) -> bool {
        let ctx = self.ctx();
        peek!(self).is_some_and(|peek| peek.is_word() && !peek.is_reserved(ctx))
    }

    #[inline(always)]
    pub fn eat_ident_ref(&mut self) -> bool {
        if self.is_ident_ref() {
            self.bump();
            true
        } else {
            false
        }
    }

    #[cold]
    #[inline(never)]
    pub fn eof_error(&mut self) -> Error {
        debug_assert!(
            self.input().cur() == Token::Eof,
            "Parser should not call throw_eof_error() without knowing current token"
        );
        let pos = self.input().end_pos();
        let last = Span { lo: pos, hi: pos };
        Error::new(last, SyntaxError::Eof)
    }
}
