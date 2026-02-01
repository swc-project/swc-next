use swc_core::atoms::Atom;
use swc_core::common::BytePos;
use swc_experimental_ecma_ast::*;

use crate::{
    Context, PResult,
    error::{Error, SyntaxError},
    input::Tokens,
    lexer::Token,
    parser::{
        Parser,
        js::pat::PatType,
        util::{FromStmt, ScratchIndex},
    },
};

#[allow(clippy::enum_variant_names)]
enum TempForHead {
    For {
        init: Option<VarDeclOrExpr>,
        test: Option<Expr>,
        update: Option<Expr>,
    },
    ForIn {
        left: ForHead,
        right: Expr,
    },
    ForOf {
        left: ForHead,
        right: Expr,
    },
}

impl<I: Tokens> Parser<I> {
    fn parse_normal_for_head(&mut self, init: Option<VarDeclOrExpr>) -> PResult<TempForHead> {
        let test = if self.input_mut().eat(Token::Semi) {
            None
        } else {
            let test = self.allow_in_expr(|p| p.parse_expr_inner()).map(Some)?;
            self.input_mut().eat(Token::Semi);
            test
        };

        let update = if self.input().is(Token::RParen) {
            None
        } else {
            self.allow_in_expr(|p| p.parse_expr_inner()).map(Some)?
        };

        Ok(TempForHead::For { init, test, update })
    }

    fn parse_for_each_head(&mut self, left: ForHead) -> PResult<TempForHead> {
        let is_of = self.input().cur() == Token::Of;
        self.bump();
        if is_of {
            let right = self.allow_in_expr(Self::parse_assignment_expr)?;
            Ok(TempForHead::ForOf { left, right })
        } else {
            if let ForHead::UsingDecl(d) = &left {
                self.emit_err(
                    d.span(&self.ast),
                    SyntaxError::UsingDeclNotAllowedForForInLoop,
                )
            }
            let right = self.allow_in_expr(|p| p.parse_expr_inner())?;
            Ok(TempForHead::ForIn { left, right })
        }
    }

    fn parse_return_stmt(&mut self) -> PResult<Stmt> {
        let start = self.cur_pos();

        self.assert_and_bump(Token::Return);

        let arg = if self.is_general_semi() {
            None
        } else {
            self.allow_in_expr(|p| p.parse_expr_inner()).map(Some)?
        };
        self.expect_general_semi()?;
        let stmt = self.ast.stmt_return_stmt(self.span(start), arg);

        if !self.ctx().contains(Context::InFunction)
            && !self.input().syntax().allow_return_outside_function()
        {
            self.emit_err(self.span(start), SyntaxError::ReturnNotAllowed);
        }

        Ok(stmt)
    }

    fn parse_var_declarator(
        &mut self,
        for_loop: bool,
        kind: VarDeclKind,
    ) -> PResult<VarDeclarator> {
        let start = self.cur_pos();

        let is_let_or_const = matches!(kind, VarDeclKind::Let | VarDeclKind::Const);

        let name = self.parse_binding_pat_or_ident(is_let_or_const)?;

        // let definite = if self.input().syntax().typescript() {
        //     match name {
        //         Pat::Ident(..) => self.input_mut().eat(Token::Bang),
        //         _ => false,
        //     }
        // } else {
        //     false
        // };

        // Typescript extension
        // if self.input().syntax().typescript() && self.input().is(Token::Colon) {
        //     let type_annotation = self.try_parse_ts_type_ann()?;
        //     match name {
        //         Pat::Array(ArrayPat {
        //             ref mut type_ann, ..
        //         })
        //         | Pat::Ident(BindingIdent {
        //             ref mut type_ann, ..
        //         })
        //         | Pat::Object(ObjectPat {
        //             ref mut type_ann, ..
        //         })
        //         | Pat::Rest(RestPat {
        //             ref mut type_ann, ..
        //         }) => {
        //             *type_ann = type_annotation;
        //         }
        //         _ => unreachable!("invalid syntax: Pat: {:?}", name),
        //     }
        // }

        //FIXME: This is wrong. Should check in/of only on first looself.
        let cur = self.input().cur();
        let init = if !for_loop || !(cur == Token::In || cur == Token::Of) {
            if self.input_mut().eat(Token::Eq) {
                let expr = self.parse_assignment_expr()?;
                let expr = self.verify_expr(expr)?;

                Some(expr)
            } else {
                // Destructuring bindings require initializers, but
                // typescript allows `declare` vars not to have initializers.
                if self.ctx().contains(Context::InDeclare) {
                    None
                } else if kind == VarDeclKind::Const
                    && !for_loop
                    && !self.ctx().contains(Context::InDeclare)
                {
                    self.emit_err(
                        self.span(start),
                        SyntaxError::ConstDeclarationsRequireInitialization,
                    );

                    None
                } else {
                    match name {
                        Pat::Ident(..) => None,
                        _ => {
                            syntax_error!(self, self.span(start), SyntaxError::PatVarWithoutInit)
                        }
                    }
                }
            }
        } else {
            // e.g. for(let a;;)
            None
        };

        Ok(self.ast.var_declarator(self.span(start), name, init))
    }

    pub(crate) fn parse_var_stmt(&mut self, for_loop: bool) -> PResult<VarDecl> {
        let start = self.cur_pos();
        let t = self.input().cur();
        let kind = if t == Token::Const {
            VarDeclKind::Const
        } else if t == Token::Let {
            VarDeclKind::Let
        } else if t == Token::Var {
            VarDeclKind::Var
        } else {
            unreachable!()
        };
        self.bump();
        let var_span = self.span(start);
        let should_include_in = kind != VarDeclKind::Var || !for_loop;

        // if self.syntax().typescript() && for_loop {
        //     let cur = self.input().cur();
        //     let res: PResult<bool> = if cur == Token::In || cur == Token::Of {
        //         self.ts_look_ahead(|p| {
        //             //
        //             if !p.input_mut().eat(Token::Of) && !p.input_mut().eat(Token::In) {
        //                 return Ok(false);
        //             }

        //             p.parse_assignment_expr()?;
        //             expect!(p, Token::RParen);

        //             Ok(true)
        //         })
        //     } else {
        //         Ok(false)
        //     };

        //     match res {
        //         Ok(true) => {
        //             let pos = var_span.hi();
        //             let span = Span::new_with_checked(pos, pos);
        //             self.emit_err(span, SyntaxError::TS1123);

        //             return Ok(Box::new(VarDecl {
        //                 span: self.span(start),
        //                 kind,
        //                 declare: false,
        //                 decls: Vec::new(),
        //                 ..Default::default()
        //             }));
        //         }
        //         Err(..) => {}
        //         _ => {}
        //     }
        // }

        let decls = self.scratch_start(|p, decls| {
            loop {
                // Handle
                //      var a,;
                //
                // NewLine is ok
                if p.input().is(Token::Semi) {
                    let prev_span = p.input().prev_span();
                    let span = if prev_span == var_span {
                        Span::new_with_checked(prev_span.hi, prev_span.hi)
                    } else {
                        prev_span
                    };
                    p.emit_err(span, SyntaxError::TS1009);
                    break;
                }

                let decl = if should_include_in {
                    p.do_inside_of_context(Context::IncludeInExpr, |p| {
                        p.parse_var_declarator(for_loop, kind)
                    })
                } else {
                    p.parse_var_declarator(for_loop, kind)
                }?;

                decls.push(p, decl);

                if !p.input_mut().eat(Token::Comma) {
                    break;
                }
            }

            if !for_loop && !p.eat_general_semi() {
                p.emit_err(p.input().cur_span(), SyntaxError::TS1005);

                let _ = p.parse_expr_inner();

                while !p.eat_general_semi() {
                    p.bump();

                    if p.input().cur() == Token::Error {
                        break;
                    }
                }
            }

            Ok(())
        })?;
        Ok(self.ast.var_decl(self.span(start), kind, false, decls))
    }

    fn parse_using_decl(&mut self, start: BytePos, is_await: bool) -> PResult<Option<UsingDecl>> {
        // using
        // reader = init()

        // is two statements
        if self.input_mut().has_linebreak_between_cur_and_peeked() {
            return Ok(None);
        }

        if !self.peek_is_ident_ref() {
            return Ok(None);
        }

        self.assert_and_bump(Token::Using);

        let decls = self.scratch_start(|p, decls| {
            loop {
                // Handle
                //      var a,;
                //
                // NewLine is ok
                if p.input().is(Token::Semi) {
                    let span = p.input().prev_span();
                    p.emit_err(span, SyntaxError::TS1009);
                    break;
                }

                let decl = p.parse_var_declarator(false, VarDeclKind::Var)?;
                decls.push(p, decl);
                if !p.input_mut().eat(Token::Comma) {
                    break;
                }
            }

            if !p.syntax().explicit_resource_management() {
                p.emit_err(p.span(start), SyntaxError::UsingDeclNotEnabled);
            }

            if !p.ctx().contains(Context::AllowUsingDecl) {
                p.emit_err(p.span(start), SyntaxError::UsingDeclNotAllowed);
            }

            Ok(())
        })?;
        for decl in decls.iter() {
            let decl = self.ast.get_node_in_sub_range(decl);
            match decl.name(&self.ast) {
                Pat::Ident(..) => {}
                _ => {
                    self.emit_err(self.span(start), SyntaxError::InvalidNameInUsingDecl);
                }
            }

            if decl.init(&self.ast).is_none() {
                self.emit_err(self.span(start), SyntaxError::InitRequiredForUsingDecl);
            }
        }

        self.expect_general_semi()?;

        Ok(Some(self.ast.using_decl(self.span(start), is_await, decls)))
    }

    fn parse_for_head(&mut self) -> PResult<TempForHead> {
        // let strict = self.ctx().contains(Context::Strict);

        let cur = self.input().cur();
        if cur == Token::Const
            || cur == Token::Var
            || (self.input().is(Token::Let)
                && peek!(self).map_or(false, |v| v.follows_keyword_let()))
        {
            let decl = self.parse_var_stmt(true)?;

            let cur = self.input().cur();
            if cur == Token::Of || cur == Token::In {
                if decl.decls(&self.ast).len() != 1 {
                    for d in decl.decls(&self.ast).iter().skip(1) {
                        let d = self.ast.get_node_in_sub_range(d);
                        self.emit_err(
                            d.name(&self.ast).span(&self.ast),
                            SyntaxError::TooManyVarInForInHead,
                        );
                    }
                } else {
                    let decls = decl.decls(&self.ast);
                    let decl = self.ast.get_node_in_sub_range(decls.get(0));
                    if (self.ctx().contains(Context::Strict) || self.input().is(Token::Of))
                        && decl.init(&self.ast).is_some()
                    {
                        self.emit_err(
                            decl.name(&self.ast).span(&self.ast),
                            SyntaxError::VarInitializerInForInHead,
                        );
                    }

                    // if self.syntax().typescript() {
                    //     let type_ann = match decl.decls[0].name {
                    //         Pat::Ident(ref v) => Some(&v.type_ann),
                    //         Pat::Array(ref v) => Some(&v.type_ann),
                    //         Pat::Rest(ref v) => Some(&v.type_ann),
                    //         Pat::Object(ref v) => Some(&v.type_ann),
                    //         _ => None,
                    //     };

                    //     if let Some(type_ann) = type_ann {
                    //         if type_ann.is_some() {
                    //             self.emit_err(decl.decls[0].name.span(), SyntaxError::TS2483);
                    //         }
                    //     }
                    // }
                }

                return self.parse_for_each_head(ForHead::VarDecl(decl));
            }

            expect!(self, Token::Semi);
            return self.parse_normal_for_head(Some(VarDeclOrExpr::VarDecl(decl)));
        }

        if self.input_mut().eat(Token::Semi) {
            return self.parse_normal_for_head(None);
        }

        let start = self.cur_pos();
        let init = self.disallow_in_expr(Self::parse_for_head_prefix)?;

        let mut is_using_decl = false;
        let mut is_await_using_decl = false;

        if self.input().syntax().explicit_resource_management() {
            // using foo
            let mut maybe_using_decl = init.is_ident_ref_to(&self.ast, "using");
            let mut maybe_await_using_decl = false;

            // await using foo
            if !maybe_using_decl
                && init
                    .as_await()
                    .filter(|e| e.arg(&self.ast).is_ident_ref_to(&self.ast, "using"))
                    .is_some()
            {
                maybe_using_decl = true;
                maybe_await_using_decl = true;
            }

            if maybe_using_decl
                && !self.input().is(Token::Of)
                && (peek!(self).is_some_and(|peek| peek == Token::Of || peek == Token::In))
            {
                is_using_decl = maybe_using_decl;
                is_await_using_decl = maybe_await_using_decl;
            }
        }

        if is_using_decl {
            let name = self.parse_binding_ident(false)?;
            let name = self.ast.binding_ident(self.span(start), name);
            let decl = self
                .ast
                .var_declarator(self.span(start), Pat::Ident(name), Some(init));

            let decls = self.scratch_start(|p, decls| {
                decls.push(p, decl);
                Ok(())
            })?;

            let pat = self
                .ast
                .using_decl(self.span(start), is_await_using_decl, decls);

            let cur = self.input().cur();
            if cur == Token::Error {
                let err = self.input_mut().expect_error_token_and_bump();
                return Err(err);
            } else if cur == Token::Eof {
                return Err(self.eof_error());
            }

            return self.parse_for_each_head(ForHead::UsingDecl(pat));
        }

        // for (a of b)
        let cur = self.input().cur();
        if cur == Token::Of || cur == Token::In {
            let is_in = self.input().is(Token::In);

            let pat = self.reparse_expr_as_pat(PatType::AssignPat, init)?;

            // for ({} in foo) is invalid
            if self.input().syntax().typescript() && is_in {
                match pat {
                    Pat::Ident(..) => {}
                    Pat::Expr(..) => {}
                    ref v => self.emit_err(v.span(&self.ast), SyntaxError::TS2491),
                }
            }

            return self.parse_for_each_head(ForHead::Pat(pat));
        }

        expect!(self, Token::Semi);

        let init = self.verify_expr(init)?;
        self.parse_normal_for_head(Some(VarDeclOrExpr::Expr(init)))
    }

    fn parse_for_stmt(&mut self) -> PResult<Stmt> {
        let start = self.cur_pos();

        self.assert_and_bump(Token::For);
        let await_start = self.cur_pos();
        let await_token = if self.input_mut().eat(Token::Await) {
            Some(self.span(await_start))
        } else {
            None
        };
        expect!(self, Token::LParen);

        let head = self.do_inside_of_context(Context::ForLoopInit, |p| {
            if await_token.is_some() {
                p.do_inside_of_context(Context::ForAwaitLoopInit, Self::parse_for_head)
            } else {
                p.do_outside_of_context(Context::ForAwaitLoopInit, Self::parse_for_head)
            }
        })?;

        expect!(self, Token::RParen);

        let body = self.do_inside_of_context(
            Context::IsBreakAllowed.union(Context::IsContinueAllowed),
            |p| p.do_outside_of_context(Context::TopLevel, Self::parse_stmt),
        )?;

        let span = self.span(start);
        Ok(match head {
            TempForHead::For { init, test, update } => {
                if let Some(await_token) = await_token {
                    syntax_error!(self, await_token, SyntaxError::AwaitForStmt);
                }

                self.ast.stmt_for_stmt(span, init, test, update, body)
            }
            TempForHead::ForIn { left, right } => {
                if let Some(await_token) = await_token {
                    syntax_error!(self, await_token, SyntaxError::AwaitForStmt);
                }

                self.ast.stmt_for_in_stmt(span, left, right, body)
            }
            TempForHead::ForOf { left, right } => {
                self.ast
                    .stmt_for_of_stmt(span, await_token.is_some(), left, right, body)
            }
        })
    }

    pub fn parse_stmt(&mut self) -> PResult<Stmt> {
        trace_cur!(self, parse_stmt);
        self.parse_stmt_like(false, handle_import_export)
    }

    fn parse_if_stmt(&mut self) -> PResult<IfStmt> {
        let start = self.cur_pos();

        self.assert_and_bump(Token::If);
        let if_token = self.input().prev_span();

        expect!(self, Token::LParen);

        let test = self
            .allow_in_expr(|p| p.parse_expr_inner())
            .map_err(|err| {
                Error::new(
                    err.span(),
                    SyntaxError::WithLabel {
                        inner: Box::new(err),
                        span: if_token,
                        note: "Tried to parse the condition for an if statement",
                    },
                )
            })?;
        expect!(self, Token::RParen);

        let cons = self.parse_stmt()?;
        let alt = self
            .input_mut()
            .eat(Token::Else)
            .then(|| self.parse_stmt())
            .transpose()?;

        let span = self.span(start);
        Ok(self.ast.if_stmt(span, test, cons, alt))
    }

    fn parse_throw_stmt(&mut self) -> PResult<Stmt> {
        let start = self.cur_pos();

        self.assert_and_bump(Token::Throw);

        if self.input().had_line_break_before_cur() {
            // TODO: Suggest throw arg;
            syntax_error!(self, SyntaxError::LineBreakInThrow);
        }

        let arg = self.allow_in_expr(|p| p.parse_expr_inner())?;
        self.expect_general_semi()?;

        let span = self.span(start);
        Ok(self.ast.stmt_throw_stmt(span, arg))
    }

    fn parse_with_stmt(&mut self) -> PResult<Stmt> {
        if self.syntax().typescript() {
            let span = self.input().cur_span();
            self.emit_err(span, SyntaxError::TS2410);
        }

        {
            let span = self.input().cur_span();
            self.emit_strict_mode_err(span, SyntaxError::WithInStrict);
        }

        let start = self.cur_pos();

        self.assert_and_bump(Token::With);

        expect!(self, Token::LParen);
        let obj = self.allow_in_expr(|p| p.parse_expr_inner())?;
        expect!(self, Token::RParen);

        let body = self.do_inside_of_context(Context::InFunction, |p| {
            p.do_outside_of_context(Context::TopLevel, Self::parse_stmt)
        })?;

        let span = self.span(start);
        Ok(self.ast.stmt_with_stmt(span, obj, body))
    }

    fn parse_while_stmt(&mut self) -> PResult<Stmt> {
        let start = self.cur_pos();

        self.assert_and_bump(Token::While);

        expect!(self, Token::LParen);
        let test = self.allow_in_expr(|p| p.parse_expr_inner())?;
        expect!(self, Token::RParen);

        let body = self.do_inside_of_context(
            Context::IsBreakAllowed.union(Context::IsContinueAllowed),
            |p| p.do_outside_of_context(Context::TopLevel, Self::parse_stmt),
        )?;

        let span = self.span(start);
        Ok(self.ast.stmt_while_stmt(span, test, body))
    }

    /// It's optional since es2019
    fn parse_catch_param(&mut self) -> PResult<Option<Pat>> {
        if self.input_mut().eat(Token::LParen) {
            let pat = self.parse_binding_pat_or_ident(false)?;

            // let type_ann_start = self.cur_pos();
            // if self.syntax().typescript() && self.input_mut().eat(Token::Colon) {
            //     let ty = self.do_inside_of_context(Context::InType, Self::parse_ts_type)?;
            //     // self.emit_err(ty.span(), SyntaxError::TS1196);

            //     match &mut pat {
            //         Pat::Ident(BindingIdent { type_ann, .. })
            //         | Pat::Array(ArrayPat { type_ann, .. })
            //         | Pat::Rest(RestPat { type_ann, .. })
            //         | Pat::Object(ObjectPat { type_ann, .. }) => {
            //             *type_ann = Some(Box::new(TsTypeAnn {
            //                 span: self.span(type_ann_start),
            //                 type_ann: ty,
            //             }));
            //         }
            //         Pat::Assign(..) => {}
            //         Pat::Invalid(_) => {}
            //         Pat::Expr(_) => {}
            //         #[cfg(swc_ast_unknown)]
            //         _ => unreachable!(),
            //     }
            // }
            expect!(self, Token::RParen);
            Ok(Some(pat))
        } else {
            Ok(None)
        }
    }

    fn parse_do_stmt(&mut self) -> PResult<Stmt> {
        let start = self.cur_pos();

        self.assert_and_bump(Token::Do);

        let body = self.do_inside_of_context(
            Context::IsBreakAllowed.union(Context::IsContinueAllowed),
            |p| p.do_outside_of_context(Context::TopLevel, Self::parse_stmt),
        )?;

        expect!(self, Token::While);
        expect!(self, Token::LParen);

        let test = self.allow_in_expr(|p| p.parse_expr_inner())?;

        expect!(self, Token::RParen);

        // We *may* eat semicolon.
        let _ = self.eat_general_semi();

        let span = self.span(start);

        Ok(self.ast.stmt_do_while_stmt(span, test, body))
    }

    fn parse_labelled_stmt(&mut self, l: Ident) -> PResult<Stmt> {
        self.do_inside_of_context(Context::IsBreakAllowed, |p| {
            p.do_outside_of_context(Context::AllowUsingDecl, |p| {
                let start = l.span_lo(&p.ast);
                let sym = l.sym(&p.ast);
                let atom = p.ast.get_utf8(sym);

                let mut errors = Vec::new();
                for lb in &p.state().labels {
                    let lb = p.ast.get_utf8(*lb);
                    if atom == lb {
                        errors.push(Error::new(
                            l.span(&p.ast),
                            SyntaxError::DuplicateLabel(Atom::new(atom)),
                        ));
                    }
                }
                p.state_mut().labels.push(sym);

                let body = if p.input().is(Token::Function) {
                    let f = p.parse_fn_decl(TypedSubRange::empty())?;
                    if let Decl::Fn(fn_decl) = &f {
                        let function = fn_decl.function(&p.ast);
                        if p.ctx().contains(Context::Strict) {
                            p.emit_err(function.span(&p.ast), SyntaxError::LabelledFunctionInStrict)
                        }
                        if function.is_generator(&p.ast) || function.is_async(&p.ast) {
                            p.emit_err(function.span(&p.ast), SyntaxError::LabelledGeneratorOrAsync)
                        }
                    }

                    Stmt::Decl(f)
                } else {
                    p.do_outside_of_context(Context::TopLevel, Self::parse_stmt)?
                };

                for err in errors {
                    p.emit_error(err);
                }

                {
                    let pos = p
                        .state()
                        .labels
                        .iter()
                        .position(|v| p.ast.get_utf8(*v) == p.ast.get_utf8(sym));
                    if let Some(pos) = pos {
                        p.state_mut().labels.remove(pos);
                    }
                }

                Ok(p.ast.stmt_labeled_stmt(p.span(start), l, body))
            })
        })
    }

    pub(crate) fn parse_block(&mut self, allow_directives: bool) -> PResult<BlockStmt> {
        let start = self.cur_pos();

        expect!(self, Token::LBrace);

        let stmts = self.do_outside_of_context(Context::TopLevel, |p| {
            p.parse_stmt_block_body(allow_directives, Some(Token::RBrace))
        })?;

        let span = self.span(start);
        Ok(self.ast.block_stmt(span, stmts))
    }

    fn parse_finally_block(&mut self) -> PResult<Option<BlockStmt>> {
        Ok(if self.input_mut().eat(Token::Finally) {
            self.parse_block(false).map(Some)?
        } else {
            None
        })
    }

    fn parse_catch_clause(&mut self) -> PResult<Option<CatchClause>> {
        let start = self.cur_pos();
        Ok(if self.input_mut().eat(Token::Catch) {
            let param = self.parse_catch_param()?;
            self.parse_block(false)
                .map(|body| self.ast.catch_clause(self.span(start), param, body))
                .map(Some)?
        } else {
            None
        })
    }

    fn parse_try_stmt(&mut self) -> PResult<Stmt> {
        let start = self.cur_pos();
        self.assert_and_bump(Token::Try);

        let block = self.parse_block(false)?;

        let catch_start = self.cur_pos();
        let handler = self.parse_catch_clause()?;
        let finalizer = self.parse_finally_block()?;

        if handler.is_none() && finalizer.is_none() {
            self.emit_err(
                Span::new_with_checked(catch_start, catch_start),
                SyntaxError::TS1005,
            );
        }

        let span = self.span(start);
        Ok(self.ast.stmt_try_stmt(span, block, handler, finalizer))
    }

    fn parse_switch_stmt(&mut self) -> PResult<Stmt> {
        let switch_start = self.cur_pos();

        self.assert_and_bump(Token::Switch);

        expect!(self, Token::LParen);
        let discriminant = self.allow_in_expr(|p| p.parse_expr_inner())?;
        expect!(self, Token::RParen);

        expect!(self, Token::LBrace);

        let mut span_of_previous_default = None;
        let cases = self.scratch_start(|p, cases| {
            p.do_inside_of_context(Context::IsBreakAllowed, |p| {
                while {
                    let cur = p.input().cur();
                    cur == Token::Case || cur == Token::Default
                } {
                    let is_case = p.input().is(Token::Case);
                    let case_start = p.cur_pos();
                    p.bump();
                    let test = if is_case {
                        p.allow_in_expr(|p| p.parse_expr_inner()).map(Some)?
                    } else {
                        if let Some(previous) = span_of_previous_default {
                            syntax_error!(p, SyntaxError::MultipleDefault { previous });
                        }
                        span_of_previous_default = Some(p.span(case_start));

                        None
                    };
                    expect!(p, Token::Colon);

                    let cons = p.scratch_start(|p, cons| {
                        while {
                            let cur = p.input().cur();
                            !(cur == Token::Case || cur == Token::Default || cur == Token::RBrace)
                        } {
                            let con = p.do_outside_of_context(
                                Context::TopLevel,
                                Self::parse_stmt_list_item,
                            )?;
                            cons.push(p, con);
                        }
                        Ok(())
                    })?;

                    let case = p.ast.switch_case(
                        Span::new_with_checked(case_start, p.input().prev_span().hi),
                        test,
                        cons,
                    );
                    cases.push(p, case);
                }
                Ok(())
            })?;
            Ok(())
        })?;

        // eof or rbrace
        expect!(self, Token::RBrace);

        Ok(self
            .ast
            .stmt_switch_stmt(self.span(switch_start), discriminant, cases))
    }

    /// Parse a statement and maybe a declaration.
    pub fn parse_stmt_list_item(&mut self) -> PResult<Stmt> {
        trace_cur!(self, parse_stmt_list_item);
        self.parse_stmt_like(true, handle_import_export)
    }

    /// Parse a statement, declaration or module item.
    #[inline(always)]
    pub(crate) fn parse_stmt_like<Type: FromStmt>(
        &mut self,
        include_decl: bool,
        handle_import_export: impl Fn(&mut Self, TypedSubRange<Decorator>) -> PResult<Type>,
    ) -> PResult<Type> {
        trace_cur!(self, parse_stmt_like);

        debug_tracing!(self, "parse_stmt_like");

        let start = self.cur_pos();
        let decorators = if self.input().get_cur().token == Token::At {
            self.parse_decorators(true)?
        } else {
            TypedSubRange::empty()
        };

        let cur = self.input().cur();
        if cur == Token::Import || cur == Token::Export {
            return handle_import_export(self, decorators);
        }

        self.do_outside_of_context(Context::WillExpectColonForCond, |p| {
            p.do_inside_of_context(Context::AllowUsingDecl, |p| {
                p.parse_stmt_internal(start, include_decl, decorators)
            })
        })
        .map(Type::from_stmt)
    }

    /// `parseStatementContent`
    fn parse_stmt_internal(
        &mut self,
        start: BytePos,
        include_decl: bool,
        decorators: TypedSubRange<Decorator>,
    ) -> PResult<Stmt> {
        trace_cur!(self, parse_stmt_internal);

        // let is_typescript = self.input().syntax().typescript();
        // if is_typescript
        //     && self.input().is(Token::Const)
        //     && peek!(self).is_some_and(|peek| peek == Token::Enum)
        // {
        //     self.assert_and_bump(Token::Const);
        //     self.assert_and_bump(Token::Enum);
        //     return self
        //         .parse_ts_enum_decl(start, true)
        //         .map(Decl::from)
        //         .map(Stmt::from);
        // }

        let top_level = self.ctx().contains(Context::TopLevel);

        let cur = self.input().cur();

        if cur == Token::Await && (include_decl || top_level) {
            if top_level {
                self.mark_found_module_item();
                if !self.ctx().contains(Context::CanBeModule) {
                    self.emit_err(self.input().cur_span(), SyntaxError::TopLevelAwaitInScript);
                }
            }

            if peek!(self).is_some_and(|peek| peek == Token::Using) {
                let eaten_await = Some(self.input().cur_pos());
                self.assert_and_bump(Token::Await);
                let v = self.parse_using_decl(start, true)?;
                if let Some(v) = v {
                    return Ok(Stmt::Decl(Decl::Using(v)));
                }

                let expr = self.parse_await_expr(eaten_await)?;
                let expr = self.allow_in_expr(|p| p.parse_bin_op_recursively(expr, 0))?;
                self.eat_general_semi();

                let span = self.span(start);
                return Ok(self.ast.stmt_expr_stmt(span, expr));
            }
        } else if cur == Token::Break || cur == Token::Continue {
            let is_break = self.input().is(Token::Break);
            self.bump();
            let label = if self.eat_general_semi() {
                None
            } else {
                let i = self.parse_label_ident().map(Some)?;
                self.expect_general_semi()?;
                i
            };
            let span = self.span(start);
            if is_break {
                if label.is_some()
                    && !self.state().labels.iter().any(|l| {
                        self.ast.get_utf8(*l)
                            == self.ast.get_utf8(label.as_ref().unwrap().sym(&self.ast))
                    })
                {
                    self.emit_err(span, SyntaxError::TS1116);
                } else if !self.ctx().contains(Context::IsBreakAllowed) {
                    self.emit_err(span, SyntaxError::TS1105);
                }
            } else if !self.ctx().contains(Context::IsContinueAllowed) {
                self.emit_err(span, SyntaxError::TS1115);
            } else if label.is_some()
                && !self.state().labels.iter().any(|l| {
                    self.ast.get_utf8(*l)
                        == self.ast.get_utf8(label.as_ref().unwrap().sym(&self.ast))
                })
            {
                self.emit_err(span, SyntaxError::TS1107);
            }
            return Ok(if is_break {
                self.ast.stmt_break_stmt(span, label)
            } else {
                self.ast.stmt_continue_stmt(span, label)
            });
        } else if cur == Token::Debugger {
            self.bump();
            self.expect_general_semi()?;
            return Ok(self.ast.stmt_debugger_stmt(self.span(start)));
        } else if cur == Token::Do {
            return self.parse_do_stmt();
        } else if cur == Token::For {
            return self.parse_for_stmt();
        } else if cur == Token::Function {
            if !include_decl {
                self.emit_err(self.input().cur_span(), SyntaxError::DeclNotAllowed);
            }
            return self.parse_fn_decl(decorators).map(Stmt::Decl);
        } else if cur == Token::Class {
            if !include_decl {
                self.emit_err(self.input().cur_span(), SyntaxError::DeclNotAllowed);
            }
            return self
                .parse_class_decl(start, start, decorators, false)
                .map(Stmt::Decl);
        } else if cur == Token::If {
            return self.parse_if_stmt().map(Stmt::If);
        } else if cur == Token::Return {
            return self.parse_return_stmt();
        } else if cur == Token::Switch {
            return self.parse_switch_stmt();
        } else if cur == Token::Throw {
            return self.parse_throw_stmt();
        } else if cur == Token::Catch {
            // Error recovery
            let span = self.input().cur_span();
            self.emit_err(span, SyntaxError::TS1005);

            let _ = self.parse_catch_clause();
            let _ = self.parse_finally_block();

            let invalid = self.ast.expr_invalid(span);
            return Ok(self.ast.stmt_expr_stmt(span, invalid));
        } else if cur == Token::Finally {
            // Error recovery
            let span = self.input().cur_span();
            self.emit_err(span, SyntaxError::TS1005);

            let _ = self.parse_finally_block();

            let invalud = self.ast.expr_invalid(span);
            return Ok(self.ast.stmt_expr_stmt(span, invalud));
        } else if cur == Token::Try {
            return self.parse_try_stmt();
        } else if cur == Token::With {
            return self.parse_with_stmt();
        } else if cur == Token::While {
            return self.parse_while_stmt();
        } else if cur == Token::Var || (cur == Token::Const && include_decl) {
            let v = self.parse_var_stmt(false)?;
            return Ok(Stmt::Decl(Decl::Var(v)));
        } else if cur == Token::Let && include_decl {
            // 'let' can start an identifier reference.
            let is_keyword = match peek!(self) {
                Some(t) => t.follows_keyword_let(),
                _ => false,
            };

            if is_keyword {
                let v = self.parse_var_stmt(false)?;
                return Ok(Stmt::Decl(Decl::Var(v)));
            }
        } else if cur == Token::Using && include_decl {
            let v = self.parse_using_decl(start, false)?;
            if let Some(v) = v {
                return Ok(Stmt::Decl(Decl::Using(v)));
            }
        // } else if cur == Token::Interface
        //     && is_typescript
        //     && peek!(self).is_some_and(|peek| peek.is_word())
        //     && !self.input_mut().has_linebreak_between_cur_and_peeked()
        // {
        //     let start = self.input().cur_pos();
        //     self.bump();
        //     return Ok(self.parse_ts_interface_decl(start)?.into());
        // } else if cur == Token::Type
        //     && is_typescript
        //     && peek!(self).is_some_and(|peek| peek.is_word())
        //     && !self.input_mut().has_linebreak_between_cur_and_peeked()
        // {
        //     let start = self.input().cur_pos();
        //     self.bump();
        //     return Ok(self.parse_ts_type_alias_decl(start)?.into());
        // } else if cur == Token::Enum
        //     && is_typescript
        //     && peek!(self).is_some_and(|peek| peek.is_word())
        //     && !self.input_mut().has_linebreak_between_cur_and_peeked()
        // {
        //     let start = self.input().cur_pos();
        //     self.bump();
        //     return Ok(self.parse_ts_enum_decl(start, false)?.into());
        } else if cur == Token::LBrace {
            return self
                .do_inside_of_context(Context::AllowUsingDecl, |p| p.parse_block(false))
                .map(Stmt::Block);
        } else if cur == Token::Semi {
            self.bump();
            return Ok(self.ast.stmt_empty_stmt(self.span(start)));
        }

        // Handle async function foo() {}
        if self.input().is(Token::Async)
            && peek!(self).is_some_and(|peek| peek == Token::Function)
            && !self.input_mut().has_linebreak_between_cur_and_peeked()
        {
            return self.parse_async_fn_decl(decorators).map(Stmt::Decl);
        }

        // If the statement does not start with a statement keyword or a
        // brace, it's an ExpressionStatement or LabeledStatement. We
        // simply start parsing an expression, and afterwards, if the
        // next token is a colon and the expression was a simple
        // Identifier node, we switch to interpreting it as a label.
        let expr = self.allow_in_expr(|p| p.parse_expr_inner())?;

        let expr = match expr {
            Expr::Ident(ident) => {
                if self.input_mut().eat(Token::Colon) {
                    return self.parse_labelled_stmt(ident);
                }
                Expr::Ident(ident)
            }
            _ => self.verify_expr(expr)?,
        };

        if let Expr::Ident(ref ident) = expr {
            let ident_sym = self.ast.get_utf8(ident.sym(&self.ast));
            if ident_sym == "interface" && self.input().had_line_break_before_cur() {
                self.emit_strict_mode_err(
                    ident.span(&self.ast),
                    SyntaxError::InvalidIdentInStrict(Atom::new(ident_sym)),
                );

                self.eat_general_semi();

                return Ok(self.ast.stmt_expr_stmt(self.span(start), expr));
            }

            // if self.input().syntax().typescript() {
            //     if let Some(decl) = self.parse_ts_expr_stmt(decorators, ident.clone())? {
            //         return Ok(decl.into());
            //     }
            // }
        }

        // if self.syntax().typescript() {
        //     if let Expr::Ident(ref i) = expr {
        //         match &*i.sym {
        //             "public" | "static" | "abstract" => {
        //                 if self.input_mut().eat(Token::Interface) {
        //                     self.emit_err(i.span, SyntaxError::TS2427);
        //                     return self
        //                         .parse_ts_interface_decl(start)
        //                         .map(Decl::from)
        //                         .map(Stmt::from);
        //                 }
        //             }
        //             _ => {}
        //         }
        //     }
        // }

        if self.eat_general_semi() {
            Ok(self.ast.stmt_expr_stmt(self.span(start), expr))
        } else {
            let cur = self.input().cur();
            if cur.is_bin_op() {
                self.emit_err(self.input().cur_span(), SyntaxError::TS1005);
                let expr = self.parse_bin_op_recursively(expr, 0)?;
                return Ok(self.ast.stmt_expr_stmt(self.span(start), expr));
            }

            syntax_error!(
                self,
                SyntaxError::ExpectedSemiForExprStmt {
                    expr: expr.span(&self.ast)
                }
            );
        }
    }

    pub(crate) fn parse_stmt_block_body(
        &mut self,
        allow_directives: bool,
        end: Option<Token>,
    ) -> PResult<TypedSubRange<Stmt>> {
        self.parse_block_body(allow_directives, end, handle_import_export)
    }

    pub(crate) fn parse_block_body<Type: FromStmt>(
        &mut self,
        allow_directives: bool,
        end: Option<Token>,
        handle_import_export: impl Fn(&mut Self, TypedSubRange<Decorator>) -> PResult<Type>,
    ) -> PResult<TypedSubRange<Type>> {
        trace_cur!(self, parse_block_body);

        let cur_str = self.input.iter.read_string(self.input.cur_span());
        let has_strict_directive =
            allow_directives && (cur_str == "\"use strict\"" || cur_str == "'use strict'");

        let stmts = self.scratch_start(|p, stmts| {
            let parse_stmts = |p: &mut Self, stmts: &mut ScratchIndex<Type>| -> PResult<()> {
                let is_stmt_start = |p: &mut Self| {
                    let cur = p.input().cur();
                    match end {
                        Some(end) => {
                            if cur == Token::Eof {
                                let eof_text = p.input_mut().dump_cur();
                                p.emit_err(
                                    p.input().cur_span(),
                                    SyntaxError::Expected(format!("{end:?}"), eof_text),
                                );
                                false
                            } else {
                                cur != end
                            }
                        }
                        None => cur != Token::Eof,
                    }
                };
                while is_stmt_start(p) {
                    let stmt = p.parse_stmt_like(true, &handle_import_export)?;
                    stmts.push(p, stmt);
                }
                Ok(())
            };

            if has_strict_directive {
                p.do_inside_of_context(Context::Strict, |p| parse_stmts(p, stmts))?;
            } else {
                parse_stmts(p, stmts)?;
            };

            Ok(())
        })?;

        if self.input().cur() != Token::Eof && end.is_some() {
            self.bump();
        }

        Ok(stmts)
    }

    pub(crate) fn parse_shebang(&mut self) -> PResult<OptionalUtf8Ref> {
        let cur = self.input().cur();
        Ok(if cur == Token::Shebang {
            let atom = self.input_mut().expect_shebang_token_and_bump();
            self.to_utf8_ref(atom).into()
        } else {
            OptionalUtf8Ref::none()
        })
    }
}

fn handle_import_export<I: Tokens>(
    p: &mut Parser<I>,
    _: TypedSubRange<Decorator>,
) -> PResult<Stmt> {
    let start = p.cur_pos();
    if p.input().is(Token::Import) && peek!(p).is_some_and(|peek| peek == Token::LParen) {
        let expr = p.parse_expr_inner()?;

        p.eat_general_semi();

        return Ok(p.ast.stmt_expr_stmt(p.span(start), expr));
    }

    if p.input().is(Token::Import) && peek!(p).is_some_and(|peek| peek == Token::Dot) {
        let expr = p.parse_expr_inner()?;

        p.eat_general_semi();

        return Ok(p.ast.stmt_expr_stmt(p.span(start), expr));
    }

    syntax_error!(p, SyntaxError::ImportExportInScript);
}
