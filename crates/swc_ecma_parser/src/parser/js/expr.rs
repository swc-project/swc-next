use either::Either;
use swc_core::common::{BytePos, Span};
use swc_experimental_ecma_ast::*;

use crate::parser::util::ExprExt;
use crate::string_alloc::MaybeSubUtf8;
use crate::{
    Context, PResult,
    error::{Error, SyntaxError},
    input::Tokens,
    lexer::Token,
    parser::{Parser, js::pat::PatType, util::IsSimpleParameterList},
};

pub(crate) enum AssignTargetOrSpread {
    ExprOrSpread(ExprOrSpread),
    #[allow(unused)]
    Pat(Pat),
}

impl AssignTargetOrSpread {
    pub(crate) fn span(&self, ast: &Ast) -> Span {
        match self {
            AssignTargetOrSpread::ExprOrSpread(expr_or_spread) => expr_or_spread.span(ast),
            AssignTargetOrSpread::Pat(pat) => pat.span(ast),
        }
    }
}

impl CloneIn for AssignTargetOrSpread {
    type Cloned = Self;

    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            AssignTargetOrSpread::ExprOrSpread(it) => {
                AssignTargetOrSpread::ExprOrSpread(it.clone_in(ast))
            }
            AssignTargetOrSpread::Pat(it) => AssignTargetOrSpread::Pat(it.clone_in(ast)),
        }
    }
}

impl<I: Tokens> Parser<I> {
    pub(crate) fn parse_expr_inner(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_expr);
        debug_tracing!(self, "parse_expr");
        let expr = self.parse_assignment_expr()?;
        let start = expr.span(&self.ast).lo;

        if self.input_mut().is(Token::Comma) {
            let exprs = self.scratch_start(|p, exprs| {
                exprs.push(p, expr);

                while p.input_mut().eat(Token::Comma) {
                    let expr = p.parse_assignment_expr()?;
                    exprs.push(p, expr);
                }
                Ok(())
            })?;
            return Ok(self.ast.expr_seq_expr(self.span(start), exprs));
        }

        Ok(expr)
    }

    /// AssignmentExpression[+In, ?Yield, ?Await]
    /// ...AssignmentExpression[+In, ?Yield, ?Await]
    fn parse_expr_or_spread(&mut self) -> PResult<ExprOrSpread> {
        trace_cur!(self, parse_expr_or_spread);
        let start = self.input().cur_pos();
        if self.input_mut().eat(Token::DotDotDot) {
            let spread_span = self.span(start);
            self.allow_in_expr(Self::parse_assignment_expr)
                .map_err(|err| {
                    Error::new(
                        err.span(),
                        SyntaxError::WithLabel {
                            inner: Box::new(err),
                            span: spread_span,
                            note: "An expression should follow '...'",
                        },
                    )
                })
                .map(|expr| {
                    let spread = self.ast.spread_dot_3_token(spread_span);
                    self.ast.expr_or_spread(
                        expr.span(&self.ast).with_lo(spread_span.lo()),
                        Some(spread),
                        expr,
                    )
                })
        } else {
            self.parse_assignment_expr()
                .map(|expr| self.ast.expr_or_spread(expr.span(&self.ast), None, expr))
        }
    }

    ///`parseMaybeAssign` (overridden)
    #[cfg_attr(
        feature = "tracing-spans",
        tracing::instrument(level = "debug", skip_all)
    )]
    pub(crate) fn parse_assignment_expr(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_assignment_expr);

        // if self.input().syntax().typescript() && self.input().is(Token::JSXTagStart) {
        //     // Note: When the JSX plugin is on, type assertions (`<T> x`) aren't valid
        //     // syntax.
        //     let res = self.try_parse_ts(|p| p.parse_assignment_expr_base().map(Some));
        //     if let Some(res) = res {
        //         return Ok(res);
        //     }
        // }

        self.parse_assignment_expr_base()
    }

    /// Parse an assignment expression. This includes applications of
    /// operators like `+=`.
    ///
    /// `parseMaybeAssign`
    #[cfg_attr(feature = "tracing-spans", tracing::instrument(skip_all))]
    fn parse_assignment_expr_base(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_assignment_expr_base);
        // let start = self.input().cur_span();

        // if self.input().syntax().typescript()
        //     && (self.input().cur() == Token::Lt || self.input().cur() == Token::JSXTagStart)
        //     && (peek!(self).is_some_and(|peek| peek.is_word() || peek == Token::JSXName))
        // {
        //     let res = self.do_outside_of_context(Context::WillExpectColonForCond, |p| {
        //         p.try_parse_ts(|p| {
        //             let type_parameters = p.parse_ts_type_params(false, true)?;
        //             let mut arrow = p.parse_assignment_expr_base()?;
        //             match *arrow {
        //                 Expr::Arrow(ArrowExpr {
        //                     ref mut span,
        //                     ref mut type_params,
        //                     ..
        //                 }) => {
        //                     *span = Span::new_with_checked(type_parameters.span.lo, span.hi);
        //                     *type_params = Some(type_parameters);
        //                 }
        //                 _ => unexpected!(p, "("),
        //             }
        //             Ok(Some(arrow))
        //         })
        //     });
        //     if let Some(res) = res {
        //         if self.input().syntax().disallow_ambiguous_jsx_like() {
        //             self.emit_err(start, SyntaxError::ReservedArrowTypeParam);
        //         }
        //         return Ok(res);
        //     }
        // }

        if self.ctx().contains(Context::InGenerator) && self.input().is(Token::Yield) {
            return self.parse_yield_expr();
        }

        let cur = self.input().cur();

        if cur == Token::Error {
            let err = self.input_mut().expect_error_token_and_bump();
            return Err(err);
        }

        self.state_mut().potential_arrow_start =
            if cur.is_known_ident() || matches!(cur, Token::Ident | Token::Yield | Token::LParen) {
                Some(self.cur_pos())
            } else {
                None
            };

        let start = self.cur_pos();

        // Try to parse conditional expression.
        let cond = self.parse_cond_expr()?;

        return_if_arrow!(self, cond);

        match cond {
            // if cond is conditional expression but not left-hand-side expression,
            // just return it.
            Expr::Cond(..) | Expr::Bin(..) | Expr::Unary(..) | Expr::Update(..) => return Ok(cond),
            _ => {}
        }

        self.finish_assignment_expr(start, cond)
    }

    pub(super) fn parse_unary_expr(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_unary_expr);

        let token_and_span = self.input().get_cur();
        let start = token_and_span.span.lo;
        let cur = token_and_span.token;

        if cur == Token::Lt && self.input().syntax().typescript() && !self.input().syntax().jsx() {
            // self.bump(); // consume `<`
            // return if self.input_mut().eat(Token::Const) {
            //     self.expect(Token::Gt)?;
            //     let expr = self.parse_unary_expr()?;
            //     Ok(TsConstAssertion {
            //         span: self.span(start),
            //         expr,
            //     }
            //     .into())
            // } else {
            //     self.parse_ts_type_assertion(start)
            //         .map(Expr::from)
            //         .map(Box::new)
            // };
        } else if cur == Token::Lt
            && self.input().syntax().jsx()
            && self.input_mut().peek().is_some_and(|peek| {
                peek.is_word() || peek == Token::Gt || peek.should_rescan_into_gt_in_jsx()
            })
        {
            fn into_expr(e: Either<JSXFragment, JSXElement>) -> Expr {
                match e {
                    Either::Left(l) => Expr::JSXFragment(l),
                    Either::Right(r) => Expr::JSXElement(r),
                }
            }
            return self.parse_jsx_element(true).map(into_expr);
        } else if matches!(cur, Token::PlusPlus | Token::MinusMinus) {
            // Parse update expression
            let op = if cur == Token::PlusPlus {
                UpdateOp::PlusPlus
            } else {
                UpdateOp::MinusMinus
            };
            self.bump();

            let arg = self.parse_unary_expr()?;
            let span = Span::new_with_checked(start, arg.span_hi(&self.ast));
            self.check_assign_target(arg, false);

            return Ok(self.ast.expr_update_expr(span, op, true, arg));
        } else if cur == Token::Delete
            || cur == Token::Void
            || cur == Token::TypeOf
            || cur == Token::Plus
            || cur == Token::Minus
            || cur == Token::Tilde
            || cur == Token::Bang
        {
            // Parse unary expression
            let op = if cur == Token::Delete {
                UnaryOp::Delete
            } else if cur == Token::Void {
                UnaryOp::Void
            } else if cur == Token::TypeOf {
                UnaryOp::TypeOf
            } else if cur == Token::Plus {
                UnaryOp::Plus
            } else if cur == Token::Minus {
                UnaryOp::Minus
            } else if cur == Token::Tilde {
                UnaryOp::Tilde
            } else {
                debug_assert!(cur == Token::Bang);
                UnaryOp::Bang
            };
            self.bump();
            let arg_start = self.cur_pos() - BytePos(1);
            let arg = match self.parse_unary_expr() {
                Ok(expr) => expr,
                Err(err) => {
                    self.emit_error(err);
                    self.ast
                        .expr_invalid(Span::new_with_checked(arg_start, arg_start))
                }
            };

            if op == UnaryOp::Delete
                && let Expr::Ident(ref i) = arg
            {
                self.emit_strict_mode_err(i.span(&self.ast), SyntaxError::TS1102)
            }

            return Ok(self.ast.expr_unary_expr(
                Span::new_with_checked(start, arg.span_hi(&self.ast)),
                op,
                arg,
            ));
        } else if cur == Token::Await {
            return self.parse_await_expr(None);
        }

        // UpdateExpression
        let expr = self.parse_lhs_expr()?;
        if let Expr::Arrow(_) = expr {
            return Ok(expr);
        }

        // Line terminator isn't allowed here.
        if self.input_mut().had_line_break_before_cur() {
            return Ok(expr);
        }

        let cur = self.input().cur();
        if cur == Token::PlusPlus || cur == Token::MinusMinus {
            let op = if cur == Token::PlusPlus {
                UpdateOp::PlusPlus
            } else {
                UpdateOp::MinusMinus
            };

            self.check_assign_target(expr, false);
            self.bump();

            return Ok(self.ast.expr_update_expr(
                self.span(expr.span_lo(&self.ast)),
                op,
                false,
                expr,
            ));
        }
        Ok(expr)
    }

    #[inline(always)]
    pub(super) fn parse_primary_expr(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_primary_expr);
        let start = self.input().cur_pos();
        let can_be_arrow = self
            .state
            .potential_arrow_start
            .map(|s| s == start)
            .unwrap_or(false);
        let tok = self.input.cur();
        match tok {
            Token::This => return self.parse_this_expr(start),
            Token::Async => {
                if let Some(res) = self.try_parse_async_start(can_be_arrow) {
                    return res;
                }
            }
            Token::LBracket => {
                return self
                    .do_outside_of_context(Context::WillExpectColonForCond, Self::parse_array_lit);
            }
            Token::LBrace => {
                return self.parse_object_expr();
            }
            // Handle FunctionExpression and GeneratorExpression
            Token::Function => {
                return self.parse_fn_expr();
            }
            // Literals
            Token::Null | Token::True | Token::False | Token::Num | Token::BigInt | Token::Str => {
                return self.parse_lit().map(Expr::Lit);
            }
            // Regexp
            Token::Slash | Token::DivEq => {
                if let Some(res) = self.try_parse_regexp(start) {
                    return Ok(res);
                }
            }
            Token::LParen => return self.parse_paren_expr_or_arrow_fn(can_be_arrow, None),
            Token::NoSubstitutionTemplateLiteral => {
                return Ok(Expr::Tpl(
                    self.parse_no_substitution_template_literal(false)?,
                ));
            }
            Token::TemplateHead => {
                // parse template literal
                return Ok(Expr::Tpl(
                    self.do_outside_of_context(Context::WillExpectColonForCond, |p| {
                        p.parse_tpl(false)
                    })?,
                ));
            }
            _ => {}
        }

        self.parse_primary_expr_rest(start, can_be_arrow)
    }

    /// Parse call, dot, and `[]`-subscript expressions.
    #[cfg_attr(
        feature = "tracing-spans",
        tracing::instrument(level = "debug", skip_all)
    )]
    pub(crate) fn parse_lhs_expr(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_lhs_expr);

        let token_and_span = self.input().get_cur();
        let start = token_and_span.span.lo;
        let cur = token_and_span.token;

        // `super()` can't be handled from parse_new_expr()
        if cur == Token::Super {
            self.bump(); // eat `super`
            let obj = self.ast.callee_super(self.span(start));
            return self.parse_subscripts(obj, false, false);
        } else if cur == Token::Import {
            self.bump(); // eat `import`
            return self.parse_dynamic_import_or_import_meta(start, false);
        }

        let callee = self.parse_new_expr()?;
        return_if_arrow!(self, callee);

        // let type_args = None;
        // let type_args = if self.input().syntax().typescript() && {
        //     let cur = self.input().cur();
        //     cur == Token::Lt || cur == Token::LShift
        // } {
        //     self.try_parse_ts(|p| {
        //         let type_args = p.parse_ts_type_args()?;
        //         p.assert_and_bump(Token::Gt);
        //         if p.input().is(Token::LParen) {
        //             Ok(Some(type_args))
        //         } else {
        //             Ok(None)
        //         }
        //     })
        // } else {
        //     None
        // };

        if let Expr::New(new) = callee
            && new.args(&self.ast).is_none()
        {
            // If this is parsed using 'NewExpression' rule, just return it.
            // Because it's not left-recursive.
            // if type_args.is_some() {
            //     // This fails with `expected (`
            //     expect!(self, Token::LParen);
            // }
            debug_assert!(
                self.input().cur() != Token::LParen,
                "parse_new_expr() should eat paren if it exists"
            );
            return Ok(Expr::New(new));
        }
        // 'CallExpr' rule contains 'MemberExpr (...)',
        // and 'MemberExpr' rule contains 'new MemberExpr (...)'

        if self.input().is(Token::LParen) {
            // This is parsed using production MemberExpression,
            // which is left-recursive.
            let (callee, is_import) = match callee {
                _ if callee.is_ident_ref_to(&self.ast, "import") => (
                    self.ast
                        .callee_import(callee.span(&self.ast), Default::default()),
                    true,
                ),
                _ => (Callee::Expr(callee), false),
            };
            let args = self.parse_args(is_import)?;

            let call_expr = match callee {
                Callee::Expr(e) if unwrap_ts_non_null(e).is_opt_chain() => {
                    let base = self.ast.opt_chain_base_opt_call(self.span(start), e, args);
                    self.ast.expr_opt_chain_expr(self.span(start), false, base)
                }
                _ => self.ast.expr_call_expr(self.span(start), callee, args),
            };

            return self.parse_subscripts(Callee::Expr(call_expr), false, false);
        }
        // if type_args.is_some() {
        //     // This fails
        //     expect!(self, Token::LParen);
        // }

        // This is parsed using production 'NewExpression', which contains
        // 'MemberExpression'
        Ok(callee)
    }

    #[cfg_attr(
        feature = "tracing-spans",
        tracing::instrument(level = "debug", skip_all)
    )]
    fn parse_array_lit(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_array_lit);

        let start = self.input().cur_pos();

        self.assert_and_bump(Token::LBracket);

        let mut elems = Vec::with_capacity(8);

        while !self.input().is(Token::RBracket) {
            if self.input().is(Token::Comma) {
                expect!(self, Token::Comma);
                elems.push(None);
                continue;
            }

            let elem = self.allow_in_expr(|p| p.parse_expr_or_spread())?;
            elems.push(Some(elem));

            if !self.input().is(Token::RBracket) {
                expect!(self, Token::Comma);
                if self.input().is(Token::RBracket) {
                    let prev_span = self.input().prev_span();
                    self.state_mut().trailing_commas.insert(start, prev_span);
                }
            }
        }

        expect!(self, Token::RBracket);

        let span = self.span(start);
        let elems = self.ast.add_typed_sub_range(elems);
        Ok(self.ast.expr_array_lit(span, elems))
    }

    #[allow(unused)]
    fn at_possible_async(&mut self, expr: Expr) -> bool {
        // TODO(kdy1): !this.state.containsEsc &&
        self.state().potential_arrow_start == Some(expr.span_lo(&self.ast))
            && expr.is_ident_ref_to(&self.ast, "async")
    }

    fn parse_yield_expr(&mut self) -> PResult<Expr> {
        let start = self.input().cur_pos();
        self.assert_and_bump(Token::Yield);
        debug_assert!(self.ctx().contains(Context::InGenerator));

        // Spec says
        // YieldExpression cannot be used within the FormalParameters of a generator
        // function because any expressions that are part of FormalParameters are
        // evaluated before the resulting generator object is in a resumable state.
        if self.ctx().contains(Context::InParameters) && !self.ctx().contains(Context::InFunction) {
            syntax_error!(self, self.input().prev_span(), SyntaxError::YieldParamInGen)
        }

        let parse_with_arg = |p: &mut Self| {
            let has_star = p.input_mut().eat(Token::Asterisk);
            let err_span = p.span(start);
            let arg = p.parse_assignment_expr().map_err(|err| {
                Error::new(
                    err.span(),
                    SyntaxError::WithLabel {
                        inner: Box::new(err),
                        span: err_span,
                        note: "Tried to parse an argument of yield",
                    },
                )
            })?;
            Ok(p.ast.expr_yield_expr(p.span(start), Some(arg), has_star))
        };

        if self.is_general_semi() || {
            let cur = self.input().cur();
            cur != Token::Lt
                && cur != Token::Asterisk
                && cur != Token::Slash
                && cur != Token::DivEq
                && !cur.starts_expr()
        } {
            Ok(self.ast.expr_yield_expr(self.span(start), None, false))
        } else {
            parse_with_arg(self)
        }
    }

    fn parse_tpl_elements(
        &mut self,
        is_tagged_tpl: bool,
    ) -> PResult<(TypedSubRange<Expr>, TypedSubRange<TplElement>)> {
        trace_cur!(self, parse_tpl_elements);

        let mut exprs = Vec::new();
        let cur_elem = self.parse_template_head(is_tagged_tpl)?;
        let mut is_tail = cur_elem.tail(&self.ast);
        let mut quasis = vec![cur_elem];

        while !is_tail {
            exprs.push(self.allow_in_expr(|p| p.parse_expr_inner())?);
            let elem = self.parse_tpl_element(is_tagged_tpl)?;
            is_tail = elem.tail(&self.ast);
            quasis.push(elem);
        }

        let exprs = self.ast.add_typed_sub_range(exprs);
        let quasis = self.ast.add_typed_sub_range(quasis);
        Ok((exprs, quasis))
    }

    fn parse_tagged_tpl(
        &mut self,
        tag: Expr,
        // type_params: Option<Box<TsTypeParamInstantiation>>,
    ) -> PResult<TaggedTpl> {
        let tagged_tpl_start = tag.span_lo(&self.ast);
        trace_cur!(self, parse_tagged_tpl);

        let tpl = if self.input_mut().is(Token::NoSubstitutionTemplateLiteral) {
            self.input_mut().rescan_template_token(true);
            self.parse_no_substitution_template_literal(true)?
        } else {
            self.parse_tpl(true)?
        };

        let span = self.span(tagged_tpl_start);

        if tag.is_opt_chain() {
            self.emit_err(span, SyntaxError::TaggedTplInOptChain);
        }

        Ok(self.ast.tagged_tpl(span, tag, tpl))
    }

    pub(super) fn parse_no_substitution_template_literal(
        &mut self,
        is_tagged_tpl: bool,
    ) -> PResult<Tpl> {
        let start = self.input.cur_pos();
        let cur = self.input.cur();
        let token_span = self.input.cur_span();
        debug_assert!(matches!(cur, Token::NoSubstitutionTemplateLiteral));

        // exclude the leading and the trailing `\``
        let span = Span::new_with_checked(token_span.lo + BytePos(1), token_span.hi - BytePos(1));
        let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(span));
        let cooked = self.input.expect_template_token_value();
        let cooked = match cooked {
            Ok(cooked) => self.to_wtf8_ref(cooked).into(),
            Err(err) => {
                if is_tagged_tpl {
                    OptionalWtf8Ref::new_none()
                } else {
                    return Err(err);
                }
            }
        };
        self.bump();

        let tpl_element = self.ast.tpl_element(span, true, cooked, raw);
        let quasis = self.scratch_start(|p, quasis| {
            quasis.push(p, tpl_element);
            Ok(())
        })?;
        let exprs = self.ast.add_typed_sub_range([]);
        Ok(self.ast.tpl(self.span(start), exprs, quasis))
    }

    fn parse_template_head(&mut self, is_tagged_tpl: bool) -> PResult<TplElement> {
        let cur = self.input().cur();
        let token_span = self.input.cur_span();
        debug_assert!(matches!(cur, Token::TemplateHead));

        // exclude the leading `\`` and the trailing `${`
        let span = Span::new_with_checked(token_span.lo + BytePos(1), token_span.hi - BytePos(2));
        let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(span));
        let cooked = self.input.expect_template_token_value();
        let cooked = match cooked {
            Ok(cooked) => self.to_wtf8_ref(cooked).into(),
            Err(err) => {
                if is_tagged_tpl {
                    OptionalWtf8Ref::new_none()
                } else {
                    return Err(err);
                }
            }
        };

        self.bump();
        Ok(self.ast.tpl_element(span, false, cooked, raw))
    }

    pub(super) fn parse_tpl(&mut self, is_tagged_tpl: bool) -> PResult<Tpl> {
        trace_cur!(self, parse_tpl);
        debug_assert!(matches!(self.input.cur(), Token::TemplateHead));

        let start = self.cur_pos();

        let (exprs, quasis) = self.parse_tpl_elements(is_tagged_tpl)?;
        Ok(self.ast.tpl(self.span(start), exprs, quasis))
    }

    pub(crate) fn parse_tpl_element(&mut self, is_tagged_tpl: bool) -> PResult<TplElement> {
        if self.input_mut().is(Token::RBrace) {
            self.input_mut().rescan_template_token(false);
        }
        let cur = self.input_mut().cur();
        let token_span = self.input.cur_span();

        let (raw, cooked, tail, span) = match cur {
            Token::TemplateMiddle => {
                let span = Span::new_with_checked(token_span.lo, token_span.hi - BytePos(2));
                let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(span));
                let cooked = self.input.expect_template_token_value();
                self.bump();

                match cooked {
                    Ok(cooked) => (raw, self.to_wtf8_ref(cooked).into(), false, span),
                    Err(err) => {
                        if is_tagged_tpl {
                            (raw, OptionalWtf8Ref::new_none(), false, span)
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
            Token::TemplateTail => {
                let span = Span::new_with_checked(token_span.lo, token_span.hi - BytePos(1));
                let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(span));
                let cooked = self.input.expect_template_token_value();
                self.bump();

                match cooked {
                    Ok(cooked) => (raw, self.to_wtf8_ref(cooked).into(), true, span),
                    Err(err) => {
                        if is_tagged_tpl {
                            (raw, OptionalWtf8Ref::new_none(), true, span)
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
            Token::Error => {
                let err = cur.take_error(self.input_mut());
                self.input_mut().bump();
                return Err(err);
            }
            _ => {
                unexpected!(self, "`}`")
            }
        };

        Ok(self.ast.tpl_element(span, tail, cooked, raw))
    }

    // fn parse_tpl_ty_elements(&mut self) -> PResult<(Vec<Box<TsType>>, Vec<TplElement>)> {
    //     trace_cur!(self, parse_tpl_elements);

    //     let mut tys = Vec::new();
    //     let cur_elem = self.parse_template_head(false)?;
    //     let mut is_tail = cur_elem.tail(self.ast);
    //     let mut quasis = vec![cur_elem];

    //     while !is_tail {
    //         tys.push(self.parse_ts_type()?);
    //         let elem = self.parse_tpl_element(false)?;
    //         is_tail = elem.tail(&self.ast);
    //         quasis.push(elem);
    //     }
    //     Ok((tys, quasis))
    // }

    // fn parse_no_substitution_template_ty(&mut self) -> PResult<TsTplLitType> {
    //     let start = self.input.cur_pos();
    //     let cur = self.input.cur();
    //     debug_assert!(matches!(cur, Token::NoSubstitutionTemplateLiteral));

    //     let (cooked, raw) = cur.take_template(self.input_mut());
    //     let (raw, cooked) = match cooked {
    //         Ok(cooked) => (raw, Some(cooked)),
    //         Err(_) => (raw, None),
    //     };
    //     self.bump();
    //     let pos = self.input.prev_span().hi;
    //     debug_assert!(start.0 <= pos.0);
    //     let span = Span::new_with_checked(start, pos);
    //     Ok(TsTplLitType {
    //         span,
    //         types: vec![],
    //         quasis: vec![TplElement {
    //             span: {
    //                 debug_assert!(start.0 <= pos.0 - 2);
    //                 // `____`
    //                 // `start.0 + 1` means skip the first backtick
    //                 // `pos.0 - 1` means skip the last backtick
    //                 Span::new_with_checked(
    //                     BytePos::from_u32(start.0 + 1),
    //                     BytePos::from_u32(pos.0 - 1),
    //                 )
    //             },
    //             tail: true,
    //             raw,
    //             cooked,
    //         }],
    //     })
    // }

    // fn parse_tpl_ty(&mut self) -> PResult<TsTplLitType> {
    //     trace_cur!(self, parse_tpl_ty);
    //     debug_assert!(matches!(self.input.cur(), Token::TemplateHead));

    //     let start = self.cur_pos();

    //     let (types, quasis) = self.parse_tpl_ty_elements()?;

    //     let _ = self.input.cur();

    //     Ok(TsTplLitType {
    //         span: self.span(start),
    //         types,
    //         quasis,
    //     })
    // }

    // pub(super) fn parse_tagged_tpl_ty(&mut self) -> PResult<TsLitType> {
    //     let start = self.cur_pos();
    //     debug_assert!(self.input().syntax().typescript());
    //     trace_cur!(self, parse_tagged_tpl);
    //     let tpl_ty = if self.input_mut().is(Token::NoSubstitutionTemplateLiteral) {
    //         self.parse_no_substitution_template_ty()
    //     } else {
    //         self.parse_tpl_ty()
    //     };
    //     tpl_ty.map(|tpl_ty| {
    //         let lit = TsLit::Tpl(tpl_ty);
    //         TsLitType {
    //             span: self.span(start),
    //             lit,
    //         }
    //     })
    // }

    pub(crate) fn parse_str_lit(&mut self) -> Str {
        debug_assert!(self.input().cur() == Token::Str);
        let token_and_span = self.input().get_cur();
        let start = token_and_span.span.lo;

        let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(token_and_span.span));
        let value = self.input.expect_string_token_value();
        let value = self.to_wtf8_ref(value);
        self.bump();
        self.ast.str(self.span(start), value, raw.into())
    }

    pub(crate) fn parse_lit(&mut self) -> PResult<Lit> {
        let token_and_span = self.input().get_cur();
        let start = token_and_span.span.lo;
        let cur = token_and_span.token;
        let v = if cur == Token::Null {
            self.bump();
            let span = self.span(start);
            self.ast.lit_null(span)
        } else if cur == Token::True || cur == Token::False {
            let value = cur == Token::True;
            self.bump();
            let span = self.span(start);
            self.ast.lit_bool(span, value)
        } else if cur == Token::Str {
            Lit::Str(self.parse_str_lit())
        } else if cur == Token::Num {
            let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(token_and_span.span));
            let value = self.input_mut().expect_number_token_value();
            self.bump();
            self.ast.lit_number(self.span(start), value, raw.into())
        } else if cur == Token::BigInt {
            let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(token_and_span.span));
            let value = self.input_mut().expect_bigint_token_value();
            self.bump();

            let value = self.ast.add_bigint(*value);
            self.ast.lit_big_int(self.span(start), value, raw.into())
        } else if cur == Token::Error {
            let err = self.input_mut().expect_error_token_and_bump();
            return Err(err);
        } else if cur == Token::Eof {
            return Err(self.eof_error());
        } else {
            unreachable!("parse_lit should not be called for {:?}", cur)
        };
        Ok(v)
    }

    /// Parse `Arguments[Yield, Await]`
    #[cfg_attr(feature = "tracing-spans", tracing::instrument(skip_all))]
    pub(crate) fn parse_args(
        &mut self,
        is_dynamic_import: bool,
    ) -> PResult<TypedSubRange<ExprOrSpread>> {
        trace_cur!(self, parse_args);

        self.do_outside_of_context(Context::WillExpectColonForCond, |p| {
            let start = p.cur_pos();
            expect!(p, Token::LParen);

            let mut first = true;
            p.scratch_start(|p, expr_or_spreads| {
                while !p.input().is(Token::RParen) {
                    if first {
                        first = false;
                    } else {
                        expect!(p, Token::Comma);
                        // Handle trailing comma.
                        if p.input().is(Token::RParen) {
                            if is_dynamic_import && !p.input().syntax().import_attributes() {
                                syntax_error!(
                                    p,
                                    p.span(start),
                                    SyntaxError::TrailingCommaInsideImport
                                )
                            }

                            break;
                        }
                    }

                    let expr_or_spread = p.allow_in_expr(|p| p.parse_expr_or_spread())?;
                    expr_or_spreads.push(p, expr_or_spread);
                }

                expect!(p, Token::RParen);
                Ok(())
            })
        })
    }

    fn finish_assignment_expr(&mut self, start: BytePos, cond: Expr) -> PResult<Expr> {
        trace_cur!(self, finish_assignment_expr);

        if let Some(op) = self.input().cur().as_assign_op() {
            let left = if op == AssignOp::Assign {
                let pat = self.reparse_expr_as_pat(PatType::AssignPat, cond)?;
                match AssignTarget::try_from_pat(&mut self.ast, pat) {
                    Ok(pat) => pat,
                    Err(expr) => {
                        syntax_error!(self, expr.span(&self.ast), SyntaxError::InvalidAssignTarget)
                    }
                }
            } else {
                // It is an early Reference Error if IsValidSimpleAssignmentTarget of
                // LeftHandSideExpression is false.
                if !cond.is_valid_simple_assignment_target(
                    &self.ast,
                    self.ctx().contains(Context::Strict),
                ) {
                    if self.input().syntax().typescript() {
                        self.emit_err(cond.span(&self.ast), SyntaxError::TS2406);
                    } else {
                        self.emit_err(cond.span(&self.ast), SyntaxError::NotSimpleAssign)
                    }
                }
                if self.input().syntax().typescript()
                    && cond
                        .as_ident()
                        .map(|i| i.is_reserved_in_strict_bind(&self.ast))
                        .unwrap_or(false)
                {
                    self.emit_strict_mode_err(cond.span(&self.ast), SyntaxError::TS1100);
                }

                // TODO
                match AssignTarget::try_from_expr(&mut self.ast, cond) {
                    Ok(v) => v,
                    Err(v) => {
                        syntax_error!(self, v.span(&self.ast), SyntaxError::InvalidAssignTarget);
                    }
                }
            };

            self.bump();
            let right = self.parse_assignment_expr()?;

            Ok(self.ast.expr_assign_expr(self.span(start), op, left, right))
        } else {
            Ok(cond)
        }
    }

    /// Spec: 'ConditionalExpression'
    #[cfg_attr(
        feature = "tracing-spans",
        tracing::instrument(level = "debug", skip_all)
    )]
    fn parse_cond_expr(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_cond_expr);

        let start = self.cur_pos();

        let test = self.parse_bin_expr()?;
        return_if_arrow!(self, test);

        if self.input_mut().eat(Token::QuestionMark) {
            let cons = self.do_inside_of_context(
                Context::InCondExpr
                    .union(Context::WillExpectColonForCond)
                    .union(Context::IncludeInExpr),
                Self::parse_assignment_expr,
            )?;

            expect!(self, Token::Colon);

            let alt = self.do_inside_of_context(Context::InCondExpr, |p| {
                p.do_outside_of_context(
                    Context::WillExpectColonForCond,
                    Self::parse_assignment_expr,
                )
            })?;

            let span = Span::new_with_checked(start, alt.span_hi(&self.ast));
            Ok(self.ast.expr_cond_expr(span, test, cons, alt))
        } else {
            Ok(test)
        }
    }

    #[cfg_attr(feature = "tracing-spans", tracing::instrument(skip_all))]
    pub(crate) fn parse_subscripts(
        &mut self,
        obj: Callee,
        no_call: bool,
        no_computed_member: bool,
    ) -> PResult<Expr> {
        let start = obj.span(&self.ast).lo;
        let mut expr = match obj {
            Callee::Import(import) => self.parse_subscript_import_call(start, import)?,
            Callee::Super(s) => self.parse_subscript_super(start, s, no_call)?,
            Callee::Expr(expr) => expr,
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        };

        loop {
            expr = match self.parse_subscript(start, expr, no_call, no_computed_member)? {
                (expr, false) => return Ok(expr),
                (expr, true) => expr,
            }
        }
    }

    /// returned bool is true if this method should be called again.
    #[cfg_attr(feature = "tracing-spans", tracing::instrument(skip_all))]
    fn parse_subscript(
        &mut self,
        start: BytePos,
        callee: Expr,
        no_call: bool,
        no_computed_member: bool,
    ) -> PResult<(Expr, bool)> {
        trace_cur!(self, parse_subscript);

        // if self.input().syntax().typescript() {
        //     if !self.input().had_line_break_before_cur() && self.input().is(Token::Bang) {
        //         self.assert_and_bump(Token::Bang);

        //         let expr = Box::new(Expr::TsNonNull(TsNonNullExpr {
        //             span: self.span(start),
        //             expr: callee,
        //         }));

        //         return Ok((expr, true));
        //     }

        //     if self.input().is(Token::Lt) {
        //         // tsTryParseAndCatch is expensive, so avoid if not necessary.
        //         // There are number of things we are going to "maybe" parse, like type arguments
        //         // on tagged template expressions. If any of them fail, walk it back and
        //         // continue.

        //         let result = None;
        //         let result = self.do_inside_of_context(Context::ShouldNotLexLtOrGtAsType, |p| {
        //             p.try_parse_ts(|p| {
        //                 if !no_call && p.at_possible_async(&callee) {
        //                     // Almost certainly this is a generic async function `async <T>() =>
        //                     // ... But it might be a call with a
        //                     // type argument `async<T>();`
        //                     let async_arrow_fn = p.try_parse_ts_generic_async_arrow_fn(start)?;
        //                     if let Some(async_arrow_fn) = async_arrow_fn {
        //                         return Ok(Some((async_arrow_fn.into(), true)));
        //                     }
        //                 }

        //                 let type_args = p.parse_ts_type_args()?;
        //                 p.assert_and_bump(Token::Gt);
        //                 let cur = p.input().cur();

        //                 if !no_call && cur == Token::LParen {
        //                     // possibleAsync always false here, because we would have handled it
        //                     // above. (won't be any undefined arguments)
        //                     let args = p.parse_args(false)?;

        //                     let expr = if callee.is_opt_chain() {
        //                         Expr::OptChain(OptChainExpr {
        //                             span: p.span(start),
        //                             base: Box::new(OptChainBase::Call(OptCall {
        //                                 span: p.span(start),
        //                                 callee: callee.take(),
        //                                 type_args: Some(type_args),
        //                                 args,
        //                                 ..Default::default()
        //                             })),
        //                             optional: false,
        //                         })
        //                     } else {
        //                         Expr::Call(CallExpr {
        //                             span: p.span(start),
        //                             callee: Callee::Expr(callee.take()),
        //                             type_args: Some(type_args),
        //                             args,
        //                             ..Default::default()
        //                         })
        //                     };

        //                     Ok(Some((Box::new(expr), true)))
        //                 } else if matches!(
        //                     cur,
        //                     Token::NoSubstitutionTemplateLiteral
        //                         | Token::TemplateHead
        //                         | Token::BackQuote
        //                 ) {
        //                     p.parse_tagged_tpl(callee.take(), Some(type_args))
        //                         .map(|expr| (expr.into(), true))
        //                         .map(Some)
        //                 } else if matches!(cur, Token::Eq | Token::As | Token::Satisfies) {
        //                     let expr = Expr::TsInstantiation(TsInstantiation {
        //                         span: p.span(start),
        //                         expr: callee.take(),
        //                         type_args,
        //                     });
        //                     Ok(Some((Box::new(expr), false)))
        //                 } else if no_call {
        //                     unexpected!(p, "`")
        //                 } else {
        //                     unexpected!(p, "( or `")
        //                 }
        //             })
        //         });

        //         if let Some(expr) = result {
        //             return Ok(expr);
        //         }
        //     }
        // }

        // let ts_instantiation = None;
        // let ts_instantiation = if self.syntax().typescript() && self.input().is(Token::Lt) {
        //     self.try_parse_ts_type_args()
        // } else {
        //     None
        // };

        let question_dot_token = if self.input().is(Token::QuestionMark)
            && peek!(self).is_some_and(|peek| peek == Token::Dot)
        {
            let start = self.cur_pos();
            self.bump();

            let span = Some(self.span(start));
            self.bump();

            span
        } else {
            None
        };

        // If question_dot_token is Some, then `self.cur == Token::Dot`
        let question_dot = question_dot_token.is_some();

        // $obj[name()]
        if !no_computed_member && self.input_mut().eat(Token::LBracket) {
            let bracket_lo = self.input().prev_span().lo;
            let prop = self.allow_in_expr(|p| p.parse_expr_inner())?;
            expect!(self, Token::RBracket);
            let span = Span::new_with_checked(callee.span_lo(&self.ast), self.input().last_pos());
            debug_assert_eq!(callee.span_lo(&self.ast), span.lo());
            let prop = self.ast.computed_prop_name(
                Span::new_with_checked(bracket_lo, self.input().last_pos()),
                prop,
            );

            // let type_args = None;
            // let type_args = if self.syntax().typescript() && self.input().is(Token::Lt) {
            //     self.try_parse_ts_type_args()
            // } else {
            //     None
            // };

            let is_opt_chain = unwrap_ts_non_null(callee).is_opt_chain();
            let expr = self
                .ast
                .member_expr(span, callee, MemberProp::Computed(prop));
            let expr = if is_opt_chain || question_dot {
                self.ast
                    .expr_opt_chain_expr(span, question_dot, OptChainBase::Member(expr))
            } else {
                Expr::Member(expr)
            };

            // let expr = if let Some(type_args) = type_args {
            //     Expr::TsInstantiation(TsInstantiation {
            //         expr: Box::new(expr),
            //         type_args,
            //         span: self.span(start),
            //     })
            // } else {
            //     expr
            // };
            return Ok((expr, true));
        }

        // let type_args = None;
        // let type_args = if self.syntax().typescript() && self.input().is(Token::Lt) && question_dot
        // {
        //     let ret = self.parse_ts_type_args()?;
        //     self.assert_and_bump(Token::Gt);
        //     Some(ret)
        // } else {
        //     None
        // };

        // if (self.input.is(Token::LParen) && (!no_call || question_dot)) || type_args.is_some() {
        if self.input.is(Token::LParen) && (!no_call || question_dot) {
            let args = self.parse_args(false)?;

            let span = self.span(start);
            return if question_dot || unwrap_ts_non_null(callee).is_opt_chain() {
                let base = self
                    .ast
                    .opt_chain_base_opt_call(self.span(start), callee, args);
                let expr = self.ast.opt_chain_expr(span, question_dot, base);
                Ok((Expr::OptChain(expr), true))
            } else {
                let expr = self
                    .ast
                    .call_expr(self.span(start), Callee::Expr(callee), args);
                Ok((Expr::Call(expr), true))
            };
        }

        // member expression
        // $obj.name
        if question_dot || self.input_mut().eat(Token::Dot) {
            let prop = self.parse_maybe_private_name().map(|e| match e {
                Either::Left(p) => MemberProp::PrivateName(p),
                Either::Right((span, sym)) => {
                    let sym = self.to_utf8_ref(sym);
                    MemberProp::Ident(self.ast.ident_name(span, sym))
                }
            })?;
            let span = self.span(callee.span_lo(&self.ast));
            debug_assert_eq!(callee.span_lo(&self.ast), span.lo());
            debug_assert_eq!(prop.span_hi(&self.ast), span.hi());

            // let type_args = None;
            // let type_args = if self.syntax().typescript() && self.input().is(Token::Lt) {
            //     self.try_parse_ts_type_args()
            // } else {
            //     None
            // };

            let expr = self.ast.member_expr(span, callee, prop);
            let expr = if unwrap_ts_non_null(expr.obj(&self.ast)).is_opt_chain() || question_dot {
                self.ast.expr_opt_chain_expr(
                    self.span(start),
                    question_dot,
                    OptChainBase::Member(expr),
                )
            } else {
                Expr::Member(expr)
            };

            // let expr = if let Some(type_args) = type_args {
            //     Expr::TsInstantiation(TsInstantiation {
            //         expr: Box::new(expr),
            //         type_args,
            //         span: self.span(start),
            //     })
            // } else {
            //     expr
            // };

            return Ok((expr, true));
        }

        let expr = callee;
        // let expr = if let Some(type_args) = ts_instantiation {
        //     TsInstantiation {
        //         expr: callee,
        //         type_args,
        //         span: self.span(start),
        //     }
        //     .into()
        // } else {
        //     callee
        // };

        // MemberExpression[?Yield, ?Await] TemplateLiteral[?Yield, ?Await, +Tagged]
        let cur = self.input().cur();
        if matches!(
            cur,
            Token::TemplateHead | Token::NoSubstitutionTemplateLiteral | Token::BackQuote
        ) {
            let tpl = self.do_outside_of_context(Context::WillExpectColonForCond, |p| {
                p.parse_tagged_tpl(expr)
            })?;
            return Ok((Expr::TaggedTpl(tpl), true));
        }

        Ok((expr, false))
    }

    /// Section 13.3 ImportCall
    #[cfg_attr(feature = "tracing-spans", tracing::instrument(skip_all))]
    fn parse_subscript_super(
        &mut self,
        start: BytePos,
        lhs: Super,
        no_call: bool,
    ) -> PResult<Expr> {
        trace_cur!(self, parse_subscript_super);
        match self.input().cur() {
            Token::LBracket => {
                self.bump();
                let bracket_lo = self.input().prev_span().lo;
                let prop = self.allow_in_expr(|p| p.parse_expr_inner())?;
                expect!(self, Token::RBracket);
                let span = Span::new_with_checked(lhs.span_lo(&self.ast), self.input().last_pos());
                debug_assert_eq!(lhs.span_lo(&self.ast), span.lo());
                let prop = self.ast.computed_prop_name(
                    Span::new_with_checked(bracket_lo, self.input().last_pos()),
                    prop,
                );

                if !self.ctx().contains(Context::AllowDirectSuper)
                    && !self.input().syntax().allow_super_outside_method()
                {
                    syntax_error!(self, lhs.span(&self.ast), SyntaxError::InvalidSuper)
                } else {
                    Ok(self
                        .ast
                        .expr_super_prop_expr(span, lhs, SuperProp::Computed(prop)))
                }
            }
            Token::LParen if !no_call => {
                let args = self.parse_args(false)?;
                Ok(self
                    .ast
                    .expr_call_expr(self.span(start), Callee::Super(lhs), args))
            }
            Token::Dot => {
                self.bump();
                let prop = self.parse_maybe_private_name().map(|e| match e {
                    Either::Left(p) => MemberProp::PrivateName(p),
                    Either::Right((span, sym)) => {
                        let sym = self.to_utf8_ref(sym);
                        MemberProp::Ident(self.ast.ident_name(span, sym))
                    }
                })?;
                let span = self.span(lhs.span_lo(&self.ast));
                debug_assert_eq!(lhs.span_lo(&self.ast), span.lo());
                debug_assert_eq!(prop.span_hi(&self.ast), span.hi());

                if !self.ctx().contains(Context::AllowDirectSuper)
                    && !self.input().syntax().allow_super_outside_method()
                {
                    syntax_error!(self, lhs.span(&self.ast), SyntaxError::InvalidSuper);
                } else {
                    let expr = match prop {
                        MemberProp::Ident(ident) => {
                            self.ast.super_prop_expr(span, lhs, SuperProp::Ident(ident))
                        }
                        MemberProp::PrivateName(..) => {
                            syntax_error!(
                                self,
                                self.input().cur_span(),
                                SyntaxError::InvalidSuperCall
                            )
                        }
                        MemberProp::Computed(..) => unreachable!(),
                        #[cfg(swc_ast_unknown)]
                        _ => unreachable!(),
                    };

                    Ok(Expr::SuperProp(expr))
                }
            }
            _ => {
                if no_call {
                    syntax_error!(self, self.input().cur_span(), SyntaxError::InvalidSuperCall)
                } else {
                    syntax_error!(self, self.input().cur_span(), SyntaxError::InvalidSuper)
                }
            }
        }
    }

    /// Section 13.3 ImportCall
    #[cfg_attr(feature = "tracing-spans", tracing::instrument(skip_all))]
    fn parse_subscript_import_call(&mut self, start: BytePos, lhs: Import) -> PResult<Expr> {
        trace_cur!(self, parse_subscript_import);

        if self.input().is(Token::LParen) {
            let args = self.parse_args(true)?;
            let expr = self
                .ast
                .expr_call_expr(self.span(start), Callee::Import(lhs), args);
            return Ok(expr);
        }

        syntax_error!(self, self.input().cur_span(), SyntaxError::InvalidImport);
    }

    fn parse_dynamic_import_or_import_meta(
        &mut self,
        start: BytePos,
        no_call: bool,
    ) -> PResult<Expr> {
        if self.input_mut().eat(Token::Dot) {
            self.mark_found_module_item();

            let (_, sym) = self.parse_ident_name()?;
            match self.input.iter.get_maybe_sub_utf8(sym) {
                "meta" => {
                    let span = self.span(start);
                    if !self.ctx().contains(Context::CanBeModule) {
                        self.emit_err(span, SyntaxError::ImportMetaInScript);
                    }
                    let expr = self.ast.meta_prop_expr(span, MetaPropKind::ImportMeta);
                    self.parse_subscripts(Callee::Expr(Expr::MetaProp(expr)), no_call, false)
                }
                "defer" => self.parse_dynamic_import_call(start, ImportPhase::Defer),
                "source" => self.parse_dynamic_import_call(start, ImportPhase::Source),
                _ => unexpected!(self, "meta"),
            }
        } else {
            self.parse_dynamic_import_call(start, ImportPhase::Evaluation)
        }
    }

    fn parse_dynamic_import_call(&mut self, start: BytePos, phase: ImportPhase) -> PResult<Expr> {
        let import = self.ast.callee_import(self.span(start), phase);
        self.parse_subscripts(import, false, false)
    }

    /// `is_new_expr`: true iff we are parsing production 'NewExpression'.
    #[cfg_attr(
        feature = "tracing-spans",
        tracing::instrument(level = "debug", skip_all)
    )]
    fn parse_member_expr_or_new_expr(&mut self, is_new_expr: bool) -> PResult<Expr> {
        self.do_inside_of_context(Context::ShouldNotLexLtOrGtAsType, |p| {
            p.parse_member_expr_or_new_expr_inner(is_new_expr)
        })
    }

    fn parse_member_expr_or_new_expr_inner(&mut self, is_new_expr: bool) -> PResult<Expr> {
        trace_cur!(self, parse_member_expr_or_new_expr);

        let start = self.cur_pos();
        if self.input_mut().eat(Token::New) {
            if self.input_mut().eat(Token::Dot) {
                if self.input_mut().eat(Token::Target) {
                    let span = self.span(start);
                    let expr = self.ast.expr_meta_prop_expr(span, MetaPropKind::NewTarget);

                    let ctx = self.ctx();
                    if !ctx.contains(Context::InsideNonArrowFunctionScope)
                        && !ctx.contains(Context::InParameters)
                        && !ctx.contains(Context::InClass)
                    {
                        self.emit_err(span, SyntaxError::InvalidNewTarget);
                    }

                    return self.parse_subscripts(Callee::Expr(expr), true, false);
                }

                unexpected!(self, "target")
            }

            // 'NewExpression' allows new call without paren.
            let callee = self.parse_member_expr_or_new_expr(is_new_expr)?;
            return_if_arrow!(self, callee);

            if is_new_expr {
                match callee {
                    Expr::OptChain(opt) if opt.optional(&self.ast) => {
                        syntax_error!(
                            self,
                            opt.span(&self.ast),
                            SyntaxError::OptChainCannotFollowConstructorCall
                        )
                    }
                    Expr::Member(member) => {
                        if let Expr::OptChain(opt) = member.obj(&self.ast)
                            && opt.optional(&self.ast)
                        {
                            syntax_error!(
                                self,
                                opt.span(&self.ast),
                                SyntaxError::OptChainCannotFollowConstructorCall
                            )
                        }
                    }
                    _ => {}
                }
            }

            // let type_args = None;
            // let type_args = if self.input().syntax().typescript() && {
            //     let cur = self.input().cur();
            //     cur == Token::Lt || cur == Token::LShift
            // } {
            //     self.try_parse_ts(|p| {
            //         let args = p.do_outside_of_context(
            //             Context::ShouldNotLexLtOrGtAsType,
            //             Self::parse_ts_type_args,
            //         )?;
            //         p.assert_and_bump(Token::Gt);
            //         if !p.input().is(Token::LParen) {
            //             let span = p.input().cur_span();
            //             let cur = p.input_mut().dump_cur();
            //             syntax_error!(p, span, SyntaxError::Expected('('.to_string(), cur))
            //         }
            //         Ok(Some(args))
            //     })
            // } else {
            //     None
            // };

            if !is_new_expr || self.input().is(Token::LParen) {
                // Parsed with 'MemberExpression' production.
                let args = self.parse_args(false)?;
                let new_expr = self.ast.expr_new_expr(self.span(start), callee, Some(args));

                // We should parse subscripts for MemberExpression.
                // Because it's left recursive.
                return self.parse_subscripts(Callee::Expr(new_expr), true, false);
            }

            // Parsed with 'NewExpression' production.

            return Ok(self.ast.expr_new_expr(self.span(start), callee, None));
        }

        if self.input_mut().eat(Token::Super) {
            let base = self.ast.callee_super(self.span(start));
            return self.parse_subscripts(base, true, false);
        } else if self.input_mut().eat(Token::Import) {
            return self.parse_dynamic_import_or_import_meta(start, true);
        }
        let obj = self.parse_primary_expr()?;
        return_if_arrow!(self, obj);

        // let type_args = None;
        // let type_args = if self.syntax().typescript() && self.input().is(Token::Lt) {
        //     self.try_parse_ts_type_args()
        // } else {
        //     None
        // };
        // let obj = if let Some(type_args) = type_args {
        //     trace_cur!(self, parse_member_expr_or_new_expr__with_type_args);
        //     TsInstantiation {
        //         expr: obj,
        //         type_args,
        //         span: self.span(start),
        //     }
        //     .into()
        // } else {
        //     obj
        // };

        self.parse_subscripts(Callee::Expr(obj), true, false)
    }

    /// Parse `NewExpression`.
    /// This includes `MemberExpression`.
    #[cfg_attr(feature = "tracing-spans", tracing::instrument(skip_all))]
    pub(crate) fn parse_new_expr(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_new_expr);
        self.parse_member_expr_or_new_expr(true)
    }

    /// Name from spec: 'LogicalORExpression'
    pub(crate) fn parse_bin_expr(&mut self) -> PResult<Expr> {
        trace_cur!(self, parse_bin_expr);

        let left = match self.parse_unary_expr() {
            Ok(v) => v,
            Err(err) => {
                trace_cur!(self, parse_bin_expr__recovery_unary_err);

                let cur = self.input().cur();
                if cur == Token::Error {
                    let err = self.input_mut().expect_error_token_and_bump();
                    return Err(err);
                } else if (cur == Token::In && self.ctx().contains(Context::IncludeInExpr))
                    || cur == Token::InstanceOf
                    || cur.is_bin_op()
                {
                    self.emit_err(self.input().cur_span(), SyntaxError::TS1109);
                    self.ast.expr_invalid(err.span())
                } else {
                    return Err(err);
                }
            }
        };

        return_if_arrow!(self, left);
        self.parse_bin_op_recursively(left, 0)
    }

    /// Parse binary operators with the operator precedence parsing
    /// algorithm. `left` is the left-hand side of the operator.
    /// `minPrec` provides context that allows the function to stop and
    /// defer further parser to one of its callers when it encounters an
    /// operator that has a lower precedence than the set it is parsing.
    ///
    /// `parseExprOp`
    pub(crate) fn parse_bin_op_recursively(
        &mut self,
        mut left: Expr,
        mut min_prec: u8,
    ) -> PResult<Expr> {
        loop {
            let (next_left, next_prec) = self.parse_bin_op_recursively_inner(left, min_prec)?;

            if let Expr::Bin(bin) = next_left
                && let Expr::Bin(bin) = bin.left(&self.ast)
                && matches!(
                    bin.op(&self.ast),
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr
                )
                && matches!(bin.op(&self.ast), BinaryOp::NullishCoalescing)
            {
                self.emit_err(
                    bin.span(&self.ast),
                    SyntaxError::NullishCoalescingWithLogicalOp,
                );
            }

            min_prec = match next_prec {
                Some(v) => v,
                None => return Ok(next_left),
            };

            left = next_left;
        }
    }

    /// Returns `(left, Some(next_prec))` or `(expr, None)`.
    fn parse_bin_op_recursively_inner(
        &mut self,
        left: Expr,
        min_prec: u8,
    ) -> PResult<(Expr, Option<u8>)> {
        #[allow(unused)]
        const PREC_OF_IN: u8 = 7;

        // if self.input().syntax().typescript() && !self.input().had_line_break_before_cur() {
        //     if PREC_OF_IN > min_prec && self.input().is(Token::As) {
        //         let start = left.span_lo(&self.ast);
        //         let expr = left;
        //         let node = if peek!(self).is_some_and(|cur| cur == Token::Const) {
        //             self.bump(); // as
        //             self.bump(); // const
        //             TsConstAssertion {
        //                 span: self.span(start),
        //                 expr,
        //             }
        //             .into()
        //         } else {
        //             let type_ann = self.next_then_parse_ts_type()?;
        //             TsAsExpr {
        //                 span: self.span(start),
        //                 expr,
        //                 type_ann,
        //             }
        //             .into()
        //         };

        //         return self.parse_bin_op_recursively_inner(node, min_prec);
        //     } else if self.input().is(Token::Satisfies) {
        //         let start = left.span_lo(&self.ast);
        //         let expr = left;
        //         let node = {
        //             let type_ann = self.next_then_parse_ts_type()?;
        //             TsSatisfiesExpr {
        //                 span: self.span(start),
        //                 expr,
        //                 type_ann,
        //             }
        //             .into()
        //         };

        //         return self.parse_bin_op_recursively_inner(node, min_prec);
        //     }
        // }

        // Return left on eof
        let cur = self.input().cur();
        let op = if cur == Token::In && self.ctx().contains(Context::IncludeInExpr) {
            BinaryOp::In
        } else if cur == Token::InstanceOf {
            BinaryOp::InstanceOf
        } else if let Some(op) = cur.as_bin_op() {
            op
        } else {
            return Ok((left, None));
        };

        if op.precedence() <= min_prec {
            if cfg!(feature = "debug") {
                tracing::trace!(
                    "returning {:?} without parsing {:?} because min_prec={}, prec={}",
                    // left,
                    "left",
                    op,
                    min_prec,
                    op.precedence()
                );
            }

            return Ok((left, None));
        }
        self.bump();
        if cfg!(feature = "debug") {
            tracing::trace!(
                "parsing binary op {:?} min_prec={}, prec={}",
                op,
                min_prec,
                op.precedence()
            );
        }

        if op == BinaryOp::Exp && (left.is_unary() || left.is_await()) {
            // This is invalid syntax.
            // Correct implementation would be returning Ok(left) and
            // returning "unexpected token '**'" on next.
            // But it's not useful error message.
            syntax_error!(
                self,
                SyntaxError::UnaryInExp {
                    // FIXME: Use display
                    // left: format!("{left:?}"),
                    left: "left".to_string(),
                    left_span: left.span(&self.ast),
                }
            )
        }

        let right = {
            let left_of_right = self.parse_unary_expr()?;
            self.parse_bin_op_recursively(
                left_of_right,
                if op == BinaryOp::Exp {
                    // exponential operator is right associative
                    op.precedence() - 1
                } else {
                    op.precedence()
                },
            )?
        };
        /* this check is for all ?? operators
         * a ?? b && c for this example
         * b && c => This is considered as a logical expression in the ast tree
         * a => Identifier
         * so for ?? operator we need to check in this case the right expression to
         * have parenthesis second case a && b ?? c
         * here a && b => This is considered as a logical expression in the ast tree
         * c => identifier
         * so now here for ?? operator we need to check the left expression to have
         * parenthesis if the parenthesis is missing we raise an error and
         * throw it
         */
        if op == BinaryOp::NullishCoalescing {
            if let Some(left) = left.as_bin() {
                let op = left.op(&self.ast);
                if op == BinaryOp::LogicalAnd || op == BinaryOp::LogicalOr {
                    self.emit_err(
                        left.span(&self.ast),
                        SyntaxError::NullishCoalescingWithLogicalOp,
                    );
                }
            }

            if let Some(right) = right.as_bin() {
                let op = right.op(&self.ast);
                if op == BinaryOp::LogicalAnd || op == BinaryOp::LogicalOr {
                    self.emit_err(
                        right.span(&self.ast),
                        SyntaxError::NullishCoalescingWithLogicalOp,
                    );
                }
            }
        }

        let node = self.ast.expr_bin_expr(
            Span::new_with_checked(left.span_lo(&self.ast), right.span_hi(&self.ast)),
            op,
            left,
            right,
        );

        Ok((node, Some(min_prec)))
    }

    pub(crate) fn parse_await_expr(
        &mut self,
        start_of_await_token: Option<BytePos>,
    ) -> PResult<Expr> {
        let start = start_of_await_token.unwrap_or_else(|| self.cur_pos());

        if start_of_await_token.is_none() {
            self.assert_and_bump(Token::Await);
        }

        let await_token = self.span(start);

        if self.input().is(Token::Asterisk) {
            syntax_error!(self, SyntaxError::AwaitStar);
        }

        let ctx = self.ctx();

        let span = self.span(start);

        if !ctx.contains(Context::InAsync)
            && (self.is_general_semi() || {
                let cur = self.input().cur();
                matches!(cur, Token::RParen | Token::RBracket | Token::Comma)
            })
        {
            if ctx.contains(Context::Module) {
                self.emit_err(span, SyntaxError::InvalidIdentInAsync);
            }

            let sym = self.ast.add_utf8("await");
            return Ok(self.ast.expr_ident(span, sym, false));
        }

        // This has been checked if start_of_await_token == true,
        if start_of_await_token.is_none() && ctx.contains(Context::TopLevel) {
            self.mark_found_module_item();
            if !ctx.contains(Context::CanBeModule) {
                self.emit_err(await_token, SyntaxError::TopLevelAwaitInScript);
            }
        }

        if ctx.contains(Context::InFunction) && !ctx.contains(Context::InAsync) {
            self.emit_err(await_token, SyntaxError::AwaitInFunction);
        }

        if ctx.contains(Context::InParameters) && !ctx.contains(Context::InFunction) {
            self.emit_err(span, SyntaxError::AwaitParamInAsync);
        }

        let arg = self.parse_unary_expr()?;
        Ok(self.ast.expr_await_expr(self.span(start), arg))
    }

    pub(crate) fn parse_for_head_prefix(&mut self) -> PResult<Expr> {
        self.parse_expr_inner()
    }

    // Returns (args_or_pats, trailing_comma)
    #[cfg_attr(
        feature = "tracing-spans",
        tracing::instrument(level = "debug", skip_all)
    )]
    fn parse_args_or_pats(&mut self) -> PResult<(Vec<AssignTargetOrSpread>, Option<Span>)> {
        self.do_outside_of_context(
            Context::WillExpectColonForCond,
            Self::parse_args_or_pats_inner,
        )
    }

    fn parse_args_or_pats_inner(&mut self) -> PResult<(Vec<AssignTargetOrSpread>, Option<Span>)> {
        trace_cur!(self, parse_args_or_pats);

        expect!(self, Token::LParen);

        let mut items = Vec::new();
        let mut trailing_comma = None;

        // TODO(kdy1): optimize (once we parsed a pattern, we can parse everything else
        // as a pattern instead of reparsing)
        while !self.input().is(Token::RParen) {
            // https://github.com/swc-project/swc/issues/410
            let is_async = self.input().is(Token::Async)
                && peek!(self)
                    .is_some_and(|t| t == Token::LParen || t == Token::Function || t.is_word());

            let start = self.cur_pos();
            self.state_mut().potential_arrow_start = Some(start);
            // let modifier_start = start;

            // let has_modifier = self.eat_any_ts_modifier()?;
            // let pat_start = self.cur_pos();

            let arg = {
                if self.input().syntax().typescript()
                    && (self.is_ident_ref()
                        || (self.input().is(Token::DotDotDot) && self.peek_is_ident_ref()))
                {
                    let spread = if self.input_mut().eat(Token::DotDotDot) {
                        Some(self.input().prev_span())
                    } else {
                        None
                    };

                    // At here, we use parse_bin_expr() instead of parse_assignment_expr()
                    // because `x?: number` should not be parsed as a conditional expression
                    let expr = if spread.is_some() {
                        self.parse_bin_expr()?
                    } else {
                        let mut expr = self.parse_bin_expr()?;

                        if self.input().cur().is_assign_op() {
                            expr = self.finish_assignment_expr(start, expr)?
                        }

                        expr
                    };

                    self.ast.expr_or_spread(expr.span(&self.ast), None, expr)
                } else {
                    self.allow_in_expr(|p| p.parse_expr_or_spread())?
                }
            };

            // let optional = if self.input().syntax().typescript() {
            //     if self.input().is(Token::QuestionMark) {
            //         if peek!(self).is_some_and(|peek| {
            //             matches!(
            //                 peek,
            //                 Token::Comma | Token::Eq | Token::RParen | Token::Colon
            //             )
            //         }) {
            //             self.assert_and_bump(Token::QuestionMark);
            //             if arg.spread.is_some() {
            //                 self.emit_err(self.input().prev_span(), SyntaxError::TS1047);
            //             }
            //             match *arg.expr {
            //                 Expr::Ident(..) => {}
            //                 _ => {
            //                     syntax_error!(
            //                         self,
            //                         arg.span(&self.ast),
            //                         SyntaxError::TsBindingPatCannotBeOptional
            //                     )
            //                 }
            //             }
            //             true
            //         } else if let ExprOrSpread::Expr(test) = arg {
            //             expect!(self, Token::QuestionMark);

            //             let cons = self.do_inside_of_context(
            //                 Context::InCondExpr
            //                     .union(Context::WillExpectColonForCond)
            //                     .union(Context::IncludeInExpr),
            //                 Self::parse_assignment_expr,
            //             )?;
            //             expect!(self, Token::Colon);

            //             let alt = self.do_inside_of_context(Context::InCondExpr, |p| {
            //                 p.do_outside_of_context(
            //                     Context::WillExpectColonForCond,
            //                     Self::parse_assignment_expr,
            //                 )
            //             })?;

            //             arg = ExprOrSpread::Expr(self.ast.expr_cond_expr(
            //                 Span::new_with_checked(start, alt.span_hi(&self.ast)),
            //                 test,
            //                 cons,
            //                 alt,
            //             ));

            //             false
            //         } else {
            //             false
            //         }
            //     } else {
            //         false
            //     }
            // } else {
            //     false
            // };

            // if optional || (self.input().syntax().typescript() && self.input().is(Token::Colon)) {
            //     // TODO: `async(...args?: any[]) : any => {}`
            //     //
            //     // if self.input().syntax().typescript() && optional && arg.spread.is_some() {
            //     //     self.emit_err(self.input().prev_span(), SyntaxError::TS1047)
            //     // }

            //     let mut pat = self.reparse_expr_as_pat(PatType::BindingPat, arg.expr)?;
            //     if optional {
            //         match pat {
            //             Pat::Ident(ref mut i) => i.id(&self.ast).set_optional(&mut self.ast, true),
            //             _ => unreachable!(),
            //         }
            //     }
            //     if let Some(spread) = arg.as_spread_element() {
            //         pat = self.ast.pat_rest_pat(self.span(pat_start), span, arg);
            //     }
            //     match pat {
            //         Pat::Ident(BindingIdent {
            //             id: Ident { ref mut span, .. },
            //             ref mut type_ann,
            //             ..
            //         })
            //         | Pat::Array(ArrayPat {
            //             ref mut type_ann,
            //             ref mut span,
            //             ..
            //         })
            //         | Pat::Object(ObjectPat {
            //             ref mut type_ann,
            //             ref mut span,
            //             ..
            //         })
            //         | Pat::Rest(RestPat {
            //             ref mut type_ann,
            //             ref mut span,
            //             ..
            //         }) => {
            //             let new_type_ann = self.try_parse_ts_type_ann()?;
            //             if new_type_ann.is_some() {
            //                 *span = Span::new_with_checked(pat_start, self.input().prev_span().hi);
            //             }
            //             *type_ann = new_type_ann;
            //         }
            //         Pat::Expr(ref expr) => unreachable!("invalid pattern: Expr({:?})", expr),
            //         Pat::Assign(..) | Pat::Invalid(..) => {
            //             // We don't have to panic here.
            //             // See: https://github.com/swc-project/swc/issues/1170
            //             //
            //             // Also, as an exact error is added to the errors while
            //             // creating `Invalid`, we don't have to emit a new
            //             // error.
            //         }
            //         #[cfg(swc_ast_unknown)]
            //         _ => unreachable!(),
            //     }

            //     if self.input_mut().eat(Token::Eq) {
            //         let right = self.parse_assignment_expr()?;
            //         pat = self.ast.pat_assign_pat(self.span(pat_start), pat, right);
            //     }

            //     if has_modifier {
            //         self.emit_err(self.span(modifier_start), SyntaxError::TS2369);
            //     }

            //     items.push(AssignTargetOrSpread::Pat(pat))
            // } else {
            //     if has_modifier {
            //         self.emit_err(self.span(modifier_start), SyntaxError::TS2369);
            //     }

            //     items.push(AssignTargetOrSpread::ExprOrSpread(arg));
            // }
            items.push(AssignTargetOrSpread::ExprOrSpread(arg));

            // https://github.com/swc-project/swc/issues/433
            if self.input_mut().eat(Token::Arrow) && {
                debug_assert_eq!(items.len(), 1);
                match &items[0] {
                    AssignTargetOrSpread::ExprOrSpread(expr_or_spread) => {
                        expr_or_spread.expr(&self.ast).is_ident()
                    }
                    AssignTargetOrSpread::Pat(Pat::Expr(expr)) => expr.is_ident(),
                    AssignTargetOrSpread::Pat(Pat::Ident(..)) => true,
                    _ => false,
                }
            } {
                let params = {
                    let items = items.clone_in(&mut self.ast);
                    self.parse_paren_items_as_params(items, None)?
                };

                let body: BlockStmtOrExpr = self.parse_fn_block_or_expr_body(
                    false,
                    false,
                    true,
                    params.is_simple_parameter_list(&self.ast),
                )?;
                let span = self.span(start);

                let arrow = self
                    .ast
                    .expr_arrow_expr(span, params, body, is_async, false);
                let expr = self.ast.expr_or_spread(span, None, arrow);
                items.push(AssignTargetOrSpread::ExprOrSpread(expr));
            }

            if !self.input().is(Token::RParen) {
                expect!(self, Token::Comma);
                if self.input().is(Token::RParen) {
                    trailing_comma = Some(self.input().prev_span());
                }
            }
        }

        expect!(self, Token::RParen);
        Ok((items, trailing_comma))
    }

    #[cfg_attr(
        feature = "tracing-spans",
        tracing::instrument(level = "debug", skip_all)
    )]
    fn parse_paren_expr_or_arrow_fn(
        &mut self,
        can_be_arrow: bool,
        async_span: Option<Span>,
    ) -> PResult<Expr> {
        trace_cur!(self, parse_paren_expr_or_arrow_fn);

        let expr_start = async_span.map(|x| x.lo()).unwrap_or_else(|| self.cur_pos());

        // At this point, we can't know if it's parenthesized
        // expression or head of arrow function.
        // But as all patterns of javascript is subset of
        // expressions, we can parse both as expression.

        let (paren_items, trailing_comma) = self
            .do_outside_of_context(Context::WillExpectColonForCond, |p| {
                p.allow_in_expr(Self::parse_args_or_pats)
            })?;

        let has_pattern = paren_items
            .iter()
            .any(|item| matches!(item, AssignTargetOrSpread::Pat(..)));

        // let will_expect_colon_for_cond = self.ctx().contains(Context::WillExpectColonForCond);
        // This is slow path. We handle arrow in conditional expression.
        // if self.syntax().typescript()
        //     && self.ctx().contains(Context::InCondExpr)
        //     && self.input().is(Token::Colon)
        // {
        //     // TODO: Remove clone
        //     let items_ref = &paren_items;
        //     if let Some(expr) = self.try_parse_ts(|p| {
        //         let return_type = p.parse_ts_type_or_type_predicate_ann(Token::Colon)?;

        //         expect!(p, Token::Arrow);

        //         let params: Vec<Pat> =
        //             p.parse_paren_items_as_params(items_ref.clone(), trailing_comma)?;

        //         let body: Box<BlockStmtOrExpr> = p.parse_fn_block_or_expr_body(
        //             async_span.is_some(),
        //             false,
        //             true,
        //             params.is_simple_parameter_list(),
        //         )?;

        //         if will_expect_colon_for_cond && !p.input().is(Token::Colon) {
        //             trace_cur!(p, parse_arrow_in_cond__fail);
        //             unexpected!(p, "fail")
        //         }

        //         Ok(Some(
        //             ArrowExpr {
        //                 span: p.span(expr_start),
        //                 is_async: async_span.is_some(),
        //                 is_generator: false,
        //                 params,
        //                 body,
        //                 return_type: Some(return_type),
        //                 ..Default::default()
        //             }
        //             .into(),
        //         ))
        //     }) {
        //         return Ok(expr);
        //     }
        // }

        // let return_type = None;
        // let return_type = if !self.ctx().contains(Context::WillExpectColonForCond)
        //     && self.input().syntax().typescript()
        //     && self.input().is(Token::Colon)
        // {
        //     self.try_parse_ts(|p| {
        //         let return_type = p.parse_ts_type_or_type_predicate_ann(Token::Colon)?;

        //         if !p.input().is(Token::Arrow) {
        //             unexpected!(p, "fail")
        //         }

        //         Ok(Some(return_type))
        //     })
        // } else {
        //     None
        // };

        // we parse arrow function at here, to handle it efficiently.
        // if has_pattern || return_type.is_some() || self.input().is(Token::Arrow) {
        if has_pattern || self.input().is(Token::Arrow) {
            if self.input().had_line_break_before_cur() {
                syntax_error!(
                    self,
                    self.span(expr_start),
                    SyntaxError::LineBreakBeforeArrow
                );
            }

            if !can_be_arrow {
                syntax_error!(self, self.span(expr_start), SyntaxError::ArrowNotAllowed);
            }
            expect!(self, Token::Arrow);

            let params = self.parse_paren_items_as_params(paren_items, trailing_comma)?;

            let body: BlockStmtOrExpr = self.parse_fn_block_or_expr_body(
                async_span.is_some(),
                false,
                true,
                params.is_simple_parameter_list(&self.ast),
            )?;

            let arrow_expr = self.ast.arrow_expr(
                self.span(expr_start),
                params,
                body,
                async_span.is_some(),
                false,
            );
            if arrow_expr.body(&self.ast).is_block_stmt() && self.input().cur().is_bin_op() {
                // ) is required
                self.emit_err(self.input().cur_span(), SyntaxError::TS1005);
                let errorred_expr = self.parse_bin_op_recursively(Expr::Arrow(arrow_expr), 0)?;

                if !self.is_general_semi() {
                    // ; is required
                    self.emit_err(self.input().cur_span(), SyntaxError::TS1005);
                }

                return Ok(errorred_expr);
            }
            return Ok(Expr::Arrow(arrow_expr));
        } else {
            // If there's no arrow function, we have to check there's no
            // AssignProp in lhs to check against assignment in object literals
            // like (a, {b = 1});
            for expr_or_spread in paren_items.iter() {
                if let AssignTargetOrSpread::ExprOrSpread(e) = expr_or_spread
                    && let Expr::Object(o) = e.expr(&self.ast)
                {
                    for prop in o.props(&self.ast).iter() {
                        let prop = self.ast.get_node_in_sub_range(prop);
                        if let PropOrSpread::Prop(prop) = prop
                            && prop.is_assign()
                        {
                            self.emit_err(prop.span(&self.ast), SyntaxError::AssignProperty);
                        }
                    }
                }
            }
        }

        let expr_or_spreads = self.scratch_start(|p, expr_or_spreads| {
            for item in paren_items {
                match item {
                    AssignTargetOrSpread::ExprOrSpread(e) => expr_or_spreads.push(p, e),
                    AssignTargetOrSpread::Pat(pat) => {
                        syntax_error!(p, pat.span(&p.ast), SyntaxError::InvalidExpr)
                    }
                }
            }
            Ok(())
        })?;

        if let Some(async_span) = async_span {
            // It's a call expression
            let sym = self.ast.add_utf8("async");
            let callee = self.ast.callee_expr_ident(async_span, sym, false);
            return Ok(self.ast.expr_call_expr(
                self.span(async_span.lo()),
                callee,
                expr_or_spreads,
            ));
        }

        // It was not head of arrow function.
        if expr_or_spreads.is_empty() {
            syntax_error!(
                self,
                Span::new_with_checked(expr_start, self.last_pos()),
                SyntaxError::EmptyParenExpr
            );
        }

        // TODO: Verify that invalid expression like {a = 1} does not exists.

        // ParenthesizedExpression cannot contain spread.
        if expr_or_spreads.len() == 1 {
            let expr_or_spread = self
                .ast
                .get_node_in_sub_range(expr_or_spreads.iter().next().unwrap());
            let expr = match expr_or_spread.spread(&self.ast) {
                Some(_) => {
                    syntax_error!(
                        self,
                        expr_or_spread.span(&self.ast),
                        SyntaxError::SpreadInParenExpr
                    )
                }
                None => expr_or_spread.expr(&self.ast),
            };

            if self.syntax().no_paren() {
                return Ok(expr);
            }
            Ok(self.ast.expr_paren_expr(self.span(expr_start), expr))
        } else {
            debug_assert!(expr_or_spreads.len() >= 2);

            let exprs = self.scratch_start(|p, exprs| {
                for expr in expr_or_spreads.iter() {
                    let expr_or_spread = p.ast.get_node_in_sub_range(expr);
                    match expr_or_spread.spread(&p.ast) {
                        Some(_) => {
                            syntax_error!(
                                p,
                                expr_or_spread.span(&p.ast),
                                SyntaxError::SpreadInParenExpr
                            )
                        }
                        None => exprs.push(p, expr_or_spread.expr(&p.ast)),
                    }
                }
                Ok(())
            })?;

            debug_assert!(exprs.len() >= 2);

            // span of sequence expression should not include '(', ')'
            let span_lo = self
                .ast
                .get_node_in_sub_range(exprs.first().unwrap())
                .span_lo(&self.ast);
            let span_hi = self
                .ast
                .get_node_in_sub_range(exprs.last().unwrap())
                .span_hi(&self.ast);
            let seq_expr = self
                .ast
                .expr_seq_expr(Span::new_with_checked(span_lo, span_hi), exprs);

            if self.syntax().no_paren() {
                return Ok(seq_expr);
            }
            Ok(self.ast.expr_paren_expr(self.span(expr_start), seq_expr))
        }
    }

    fn parse_primary_expr_rest(&mut self, start: BytePos, can_be_arrow: bool) -> PResult<Expr> {
        let decorators = if self.input().is(Token::At) {
            Some(self.parse_decorators(false)?)
        } else {
            None
        };

        let token_and_span = self.input().get_cur();
        let cur = token_and_span.token;

        if cur == Token::Class {
            return self.parse_class_expr(start, decorators.unwrap_or(TypedSubRange::empty()));
        }

        let try_parse_arrow_expr = |p: &mut Self, id: Ident, id_is_async| -> PResult<Expr> {
            if can_be_arrow && !p.input().had_line_break_before_cur() {
                if id_is_async && p.is_ident_ref() {
                    // see https://github.com/tc39/ecma262/issues/2034
                    // ```js
                    // for(async of
                    // for(async of x);
                    // for(async of =>{};;);
                    // ```
                    let ctx = p.ctx();
                    if ctx.contains(Context::ForLoopInit)
                        && p.input().is(Token::Of)
                        && !peek!(p).is_some_and(|peek| peek == Token::Arrow)
                    {
                        // ```spec https://tc39.es/ecma262/#prod-ForInOfStatement
                        // for ( [lookahead ∉ { let, async of }] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
                        // [+Await] for await ( [lookahead ≠ let] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
                        // ```

                        if !ctx.contains(Context::ForAwaitLoopInit) {
                            p.emit_err(p.input().prev_span(), SyntaxError::TS1106);
                        }

                        return Ok(Expr::Ident(id));
                    }

                    let ident = p.parse_binding_ident(false)?;
                    // if p.input().syntax().typescript()
                    //     && self
                    //         .ast
                    //         .get_atom(ident.id(&self.ast).sym(&self.ast))
                    //         .as_str()
                    //         == "as"
                    //     && !p.input().is(Token::Arrow)
                    // {
                    //     // async as type
                    //     let type_ann = p.in_type(Self::parse_ts_type)?;
                    //     return Ok(TsAsExpr {
                    //         span: p.span(start),
                    //         expr: Box::new(id.into()),
                    //         type_ann,
                    //     }
                    //     .into());
                    // }

                    // async a => body
                    let arg = Pat::Ident(p.ast.binding_ident(id.span(&p.ast), ident));
                    let params = p.scratch_start(|p, params| {
                        params.push(p, arg);
                        Ok(())
                    })?;

                    expect!(p, Token::Arrow);
                    let body = p.parse_fn_block_or_expr_body(
                        true,
                        false,
                        true,
                        params.is_simple_parameter_list(&p.ast),
                    )?;

                    return Ok(p
                        .ast
                        .expr_arrow_expr(p.span(start), params, body, true, false));
                } else if p.input_mut().eat(Token::Arrow) {
                    if p.ctx().contains(Context::Strict) && id.is_reserved_in_strict_bind(&p.ast) {
                        p.emit_strict_mode_err(
                            id.span(&p.ast),
                            SyntaxError::EvalAndArgumentsInStrict,
                        )
                    }

                    let params = p.scratch_start(|p, params| {
                        let pat = Pat::Ident(p.ast.binding_ident(id.span(&p.ast), id));
                        params.push(p, pat);
                        Ok(())
                    })?;

                    let body = p.parse_fn_block_or_expr_body(
                        false,
                        false,
                        true,
                        params.is_simple_parameter_list(&p.ast),
                    )?;

                    return Ok(p
                        .ast
                        .expr_arrow_expr(p.span(start), params, body, false, false));
                }
            }

            Ok(Expr::Ident(id))
        };

        let token_start = token_and_span.span.lo;
        if cur == Token::Let || (self.input().syntax().typescript() && cur == Token::Await) {
            let ctx = self.ctx();
            let id = self.parse_ident(
                !ctx.contains(Context::InGenerator),
                !ctx.contains(Context::InAsync),
            )?;
            try_parse_arrow_expr(self, id, false)
        } else if cur == Token::Hash {
            self.bump(); // consume `#`
            let (_, sym) = self.parse_ident_name()?;
            let sym = self.to_utf8_ref(sym);
            Ok(self.ast.expr_private_name(self.span(start), sym))
        } else if cur == Token::Ident {
            let word = self.input_mut().expect_word_token_and_bump();
            if self.ctx().contains(Context::InClassField)
                && self.input.iter.get_maybe_sub_utf8(word) == "arguments"
            {
                self.emit_err(self.input().prev_span(), SyntaxError::ArgumentsInClassField)
            };

            let word = self.to_utf8_ref(word);
            let id = self.ast.ident(self.span(token_start), word, false);
            try_parse_arrow_expr(self, id, false)
        } else if self.is_ident_ref() {
            let id_is_async = self.input().cur() == Token::Async;
            let word = self.input_mut().expect_word_token_and_bump();
            let word = self.to_utf8_ref(word);
            let id = self.ast.ident(self.span(token_start), word, false);
            try_parse_arrow_expr(self, id, id_is_async)
        } else {
            syntax_error!(self, self.input().cur_span(), SyntaxError::TS1109)
        }
    }

    fn try_parse_regexp(&mut self, start: BytePos) -> Option<Expr> {
        // Regexp
        debug_assert!(self.input().cur() == Token::Slash || self.input().cur() == Token::DivEq);

        self.input_mut().set_next_regexp(Some(start));

        self.bump(); // `/` or `/=`

        let cur = self.input().cur();
        if cur == Token::Regex {
            self.input_mut().set_next_regexp(None);
            let token_span = self.input.cur_span();
            let exp_end = self.input_mut().expect_regex_token_value();

            let exp_start = token_span.lo + BytePos(1); // +1 to exclude left `/`
            let exp = self.to_utf8_ref(MaybeSubUtf8::new_from_span(Span::new_with_checked(
                exp_start, exp_end,
            )));
            let flags_start = exp_end + BytePos(1); // +1 to exclude right `/`
            let flags = if flags_start <= token_span.hi {
                self.to_utf8_ref(MaybeSubUtf8::new_from_span(Span::new_with_checked(
                    flags_start,
                    token_span.hi,
                )))
            } else {
                Utf8Ref::new_empty()
            };

            self.bump();
            let span = self.span(start);
            Some(self.ast.expr_lit_regex(span, exp, flags))
        } else {
            None
        }
    }

    fn try_parse_async_start(&mut self, can_be_arrow: bool) -> Option<PResult<Expr>> {
        if peek!(self).is_some_and(|peek| peek == Token::Function)
            && !self.input_mut().has_linebreak_between_cur_and_peeked()
        {
            // handle `async function` expression
            return Some(self.parse_async_fn_expr());
        }

        // if can_be_arrow
        //     && self.input().syntax().typescript()
        //     && peek!(self).is_some_and(|peek| peek == Token::Lt)
        // {
        //     // try parsing `async<T>() => {}`
        //     if let Some(res) = self.try_parse_ts(|p| {
        //         let start = p.cur_pos();
        //         p.assert_and_bump(Token::Async);
        //         p.try_parse_ts_generic_async_arrow_fn(start)
        //     }) {
        //         return Some(Ok(res.into()));
        //     }
        // }

        if can_be_arrow
            && peek!(self).is_some_and(|peek| peek == Token::LParen)
            && !self.input_mut().has_linebreak_between_cur_and_peeked()
        {
            if let Err(e) = self.expect(Token::Async) {
                return Some(Err(e));
            }
            let async_span = self.input().prev_span();
            return Some(self.parse_paren_expr_or_arrow_fn(can_be_arrow, Some(async_span)));
        }

        None
    }

    fn parse_this_expr(&mut self, start: BytePos) -> PResult<Expr> {
        debug_assert!(self.input().cur() == Token::This);
        self.input_mut().bump();
        Ok(self.ast.expr_this_expr(self.span(start)))
    }

    #[allow(unused)]
    pub(crate) fn is_start_of_left_hand_side_expr(&mut self) -> bool {
        let cur = self.input().cur();
        matches!(
            cur,
            Token::This
                | Token::Null
                | Token::Super
                | Token::True
                | Token::False
                | Token::Num
                | Token::BigInt
                | Token::Str
                | Token::NoSubstitutionTemplateLiteral
                | Token::TemplateHead
                | Token::LParen
                | Token::LBracket
                | Token::Function
                | Token::Class
                | Token::New
                | Token::Regex
                | Token::Import
        ) || cur.is_ident_ref(self.ctx())
            || cur == Token::BackQuote && {
                peek!(self)
                    .is_some_and(|peek| matches!(peek, Token::LParen | Token::Lt | Token::Dot))
            }
    }
}

fn unwrap_ts_non_null(expr: Expr) -> Expr {
    // while let Expr::TsNonNull(ts_non_null) = expr {
    //     expr = &ts_non_null.expr;
    // }

    expr
}
