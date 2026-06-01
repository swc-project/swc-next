use swc_experimental_allocator::vec::Vec;
use swc_experimental_ecma_ast::*;

use crate::parser::js::is_not_this;
use crate::{Context, PResult, Parser, error::SyntaxError, input::Tokens, lexer::Token};

impl<'a, I: Tokens<'a>> Parser<'a, I> {
    pub(crate) fn parse_object<Object, ObjectProp: 'a>(
        &mut self,
        parse_prop: impl Fn(&mut Self) -> PResult<ObjectProp>,
        make_object: impl Fn(&mut Self, Span, Vec<'a, ObjectProp>, Option<Span>) -> PResult<Object>,
    ) -> PResult<Object> {
        self.do_outside_of_context(Context::WillExpectColonForCond, |p| {
            trace_cur!(p, parse_object);

            let start = p.cur_pos();
            let mut trailing_comma = None;
            p.assert_and_bump(Token::LBrace);

            let props = p.collect_vec(|p, props| {
                while !p.input_mut().eat(Token::RBrace) {
                    let prop = parse_prop(p)?;
                    props.push(prop);

                    if !p.input().is(Token::RBrace) {
                        expect!(p, Token::Comma);
                        if p.input().is(Token::RBrace) {
                            trailing_comma = Some(p.input().prev_span());
                        }
                    }
                }
                Ok(())
            })?;

            let span = p.span(start);
            make_object(p, span, props, trailing_comma)
        })
    }

    /// Production 'BindingProperty'
    pub(crate) fn parse_binding_object_prop(&mut self) -> PResult<ObjectPatProp<'a>> {
        let start = self.cur_pos();

        if self.input_mut().eat(Token::DotDotDot) {
            // spread element
            let dot3_token = self.span(start);

            let arg = self.parse_binding_pat_or_ident(false)?;

            return Ok(self
                .ast
                .object_pat_prop_rest_pat(self.span(start), dot3_token, arg));
        }

        let key = self.parse_prop_name()?;
        if self.input_mut().eat(Token::Colon) {
            let value = self.parse_binding_element()?;

            return Ok(self.ast.object_pat_prop_key_value_pat_prop(key, value));
        }
        let key = match key {
            PropName::Ident(ident) => ident,
            _ => unexpected!(self, "an identifier"),
        };

        let value = if self.input_mut().eat(Token::Eq) {
            self.allow_in_expr(Self::parse_assignment_expr).map(Some)?
        } else {
            if self.ctx().is_reserved_word(key.sym.as_str()) {
                self.emit_err(key.span(), SyntaxError::ReservedWordInObjShorthandOrPat);
            }

            None
        };

        let key_ident = self.ast.box_ident(key.span(), key.sym, false);
        let key_ident = self.ast.box_binding_ident(key_ident);
        Ok(self
            .ast
            .object_pat_prop_assign_pat_prop(self.span(start), key_ident, value))
    }

    fn make_binding_object(
        &mut self,
        span: Span,
        props: Vec<'a, ObjectPatProp<'a>>,
        trailing_comma: Option<Span>,
    ) -> PResult<Pat<'a>> {
        let len = props.len();
        for (i, prop) in props.iter().enumerate() {
            if i == len - 1 {
                if let ObjectPatProp::Rest(rest) = prop {
                    match &rest.arg {
                        Pat::Ident(..) => {
                            if let Some(trailing_comma) = trailing_comma {
                                self.emit_err(trailing_comma, SyntaxError::CommaAfterRestElement);
                            }
                        }
                        _ => syntax_error!(self, prop.span(), SyntaxError::DotsWithoutIdentifier),
                    }
                }
                continue;
            }

            if let ObjectPatProp::Rest(..) = prop {
                self.emit_err(prop.span(), SyntaxError::NonLastRestParam)
            }
        }

        let optional = (self.input().syntax().dts() || self.ctx().contains(Context::InDeclare))
            && self.input_mut().eat(Token::QuestionMark);

        Ok(self.ast.pat_object_pat(span, props, optional))
    }

    pub(super) fn parse_object_pat(&mut self) -> PResult<Pat<'a>> {
        self.parse_object(
            |p| p.parse_binding_object_prop(),
            |p, span, props, trailing_comma| p.make_binding_object(span, props, trailing_comma),
        )
    }

    fn make_expr_object(
        &mut self,
        span: Span,
        props: Vec<'a, PropOrSpread<'a>>,
        trailing_comma: Option<Span>,
    ) -> PResult<Expr<'a>> {
        if let Some(trailing_comma) = trailing_comma {
            self.state_mut()
                .trailing_commas
                .insert(span.start, trailing_comma);
        }

        Ok(self.ast.expr_object_lit(span, props))
    }

    fn parse_expr_object_prop(&mut self) -> PResult<PropOrSpread<'a>> {
        trace_cur!(self, parse_object_prop);

        let start = self.cur_pos();
        // Parse as 'MethodDefinition'

        if self.input_mut().eat(Token::DotDotDot) {
            // spread element
            let dot3_token = self.span(start);

            let expr = self.allow_in_expr(Self::parse_assignment_expr)?;

            return Ok(self.ast.prop_or_spread_spread_element(dot3_token, expr));
        }

        if self.input_mut().eat(Token::Asterisk) {
            let name = self.parse_prop_name()?;
            return self
                .do_inside_of_context(Context::AllowDirectSuper, |p| {
                    p.do_outside_of_context(Context::InClassField, |p| {
                        p.parse_fn_args_body(
                            // no decorator in an object literal
                            p.vec(),
                            start,
                            Self::parse_unique_formal_params,
                            false,
                            true,
                        )
                    })
                })
                .map(|function| {
                    self.ast
                        .prop_or_spread_prop_method_prop(name, self.boxed(function))
                });
        }

        let has_modifiers = false;
        // let has_modifiers = self.eat_any_ts_modifier()?;
        let modifiers_span = self.input().prev_span();

        let key_token = self.input().cur();
        let key = self.parse_prop_name()?;

        let cur = self.input().cur();
        if self.input().syntax().typescript()
            && !(matches!(
                cur,
                Token::LParen
                    | Token::LBracket
                    | Token::Colon
                    | Token::Comma
                    | Token::QuestionMark
                    | Token::Eq
                    | Token::Asterisk
            ) || cur == Token::Str
                || cur == Token::Num
                || cur.is_word())
            && !(self.input().syntax().typescript() && self.input().is(Token::Lt))
            && !(self.input().is(Token::RBrace) && matches!(key, PropName::Ident(..)))
        {
            trace_cur!(self, parse_object_prop_error);

            self.emit_err(self.input().cur_span(), SyntaxError::TS1005);
            let value = self.ast.invalid();
            return Ok(self
                .ast
                .prop_or_spread_prop_key_value_prop(key, Expr::Invalid(self.boxed(value))));
        }
        //
        // {[computed()]: a,}
        // { 'a': a, }
        // { 0: 1, }
        // { a: expr, }
        if self.input_mut().eat(Token::Colon) {
            let value = self.allow_in_expr(Self::parse_assignment_expr)?;
            return Ok(self.ast.prop_or_spread_prop_key_value_prop(key, value));
        }

        // Handle `a(){}` (and async(){} / get(){} / set(){})
        if (self.input().syntax().typescript() && self.input().is(Token::Lt))
            || self.input().is(Token::LParen)
        {
            return self
                .do_inside_of_context(Context::AllowDirectSuper, |p| {
                    p.do_outside_of_context(Context::InClassField, |p| {
                        p.parse_fn_args_body(
                            // no decorator in an object literal
                            p.vec(),
                            start,
                            Self::parse_unique_formal_params,
                            false,
                            false,
                        )
                    })
                })
                .map(|function| {
                    self.ast
                        .prop_or_spread_prop_method_prop(key, self.boxed(function))
                });
        }

        let ident = match key {
            PropName::Ident(ident) => ident,
            // TODO
            _ => unexpected!(self, "identifier"),
        };
        let ident_span = ident.span();
        let ident_sym = ident.sym;

        if self.input_mut().eat(Token::QuestionMark) {
            self.emit_err(self.input().prev_span(), SyntaxError::TS1162);
        }

        // `ident` from parse_prop_name is parsed as 'IdentifierName'
        // It means we should check for invalid expressions like { for, }
        let cur = self.input().cur();
        if matches!(cur, Token::Eq | Token::Comma | Token::RBrace) {
            if self.ctx().is_reserved_word(ident_sym.as_str()) {
                self.emit_err(ident_span, SyntaxError::ReservedWordInObjShorthandOrPat);
            }

            let ident = self.ast.box_ident(ident_span, ident_sym, false);
            if self.input_mut().eat(Token::Eq) {
                let value = self.allow_in_expr(Self::parse_assignment_expr)?;
                let span = self.span(start);
                return Ok(self.ast.prop_or_spread_prop_assign_prop(span, ident, value));
            }

            return Ok(self
                .ast
                .prop_or_spread_prop_ident(ident_span, ident_sym, false));
        }

        // get a(){}
        // set a(v){}
        // async a(){}

        let is_get = key_token == Token::Get;
        let is_set = key_token == Token::Set;
        let is_async = key_token == Token::Async;
        if is_get || is_set || is_async {
            trace_cur!(self, parse_object_prop__after_accessor);

            if has_modifiers {
                self.emit_err(modifiers_span, SyntaxError::TS1042);
            }

            let is_generator = is_async && self.input_mut().eat(Token::Asterisk);
            let key = self.parse_prop_name()?;
            let key_span = key.span();
            self.do_inside_of_context(Context::AllowDirectSuper, |p| {
                p.do_outside_of_context(Context::InClassField, |p| {
                    if is_get {
                        return p
                            .parse_fn_args_body(
                                // no decorator in an object literal
                                p.vec(),
                                start,
                                |p| {
                                    let params = p.parse_formal_params()?;

                                    if params.iter().any(is_not_this) {
                                        p.emit_err(key_span, SyntaxError::GetterParam);
                                    }

                                    Ok(params)
                                },
                                false,
                                false,
                            )
                            .map(|function| {
                                if p.input().syntax().typescript()
                                    && p.input().target() == EsVersion::Es3
                                {
                                    p.emit_err(key_span, SyntaxError::TS1056);
                                }

                                let body = function.body;
                                p.ast
                                    .prop_or_spread_prop_getter_prop(p.span(start), key, body)
                            });
                    }

                    if is_set {
                        return p
                            .parse_fn_args_body(
                                // no decorator in an object literal
                                p.vec(),
                                start,
                                |p| {
                                    let params = p.parse_formal_params()?;

                                    if params.iter().filter(|param| is_not_this(param)).count() != 1
                                    {
                                        p.emit_err(key_span, SyntaxError::SetterParam);
                                    }

                                    if !params.is_empty()
                                        && let Pat::Rest(rest) = &params.first().unwrap().pat
                                    {
                                        p.emit_err(rest.span(), SyntaxError::RestPatInSetter);
                                    }

                                    if p.input().syntax().typescript()
                                        && p.input().target() == EsVersion::Es3
                                    {
                                        p.emit_err(key_span, SyntaxError::TS1056);
                                    }

                                    Ok(params)
                                },
                                false,
                                false,
                            )
                            .map(|function| {
                                let mut this = None;
                                let mut params = function.params;
                                if params.len() >= 2 {
                                    this = Some(params.remove(0).pat);
                                }

                                let param =
                                    params.into_iter().next().map(|v| v.pat).unwrap_or_else(|| {
                                        p.emit_err(key_span, SyntaxError::SetterParam);
                                        p.ast.pat_invalid()
                                    });

                                let body = function.body;
                                p.ast.prop_or_spread_prop_setter_prop(
                                    p.span(start),
                                    key,
                                    this,
                                    param,
                                    body,
                                )
                            });
                    }

                    if is_async {
                        return p
                            .parse_fn_args_body(
                                // no decorator in an object literal
                                p.vec(),
                                start,
                                Self::parse_unique_formal_params,
                                true,
                                is_generator,
                            )
                            .map(|function| {
                                p.ast
                                    .prop_or_spread_prop_method_prop(key, p.boxed(function))
                            });
                    }

                    unreachable!()
                })
            })
        } else if self.input().syntax().typescript() {
            unexpected!(
                self,
                "... , *,  (, [, :, , ?, =, an identifier, public, protected, private, \
                         readonly, <."
            )
        } else {
            unexpected!(self, "... , *,  (, [, :, , ?, = or an identifier")
        }
    }

    pub(crate) fn parse_object_expr(&mut self) -> PResult<Expr<'a>> {
        self.parse_object(
            |p| p.parse_expr_object_prop(),
            |p, span, props, trailing_comma| p.make_expr_object(span, props, trailing_comma),
        )
    }

    /// spec: 'PropertyName'
    pub fn parse_prop_name(&mut self) -> PResult<PropName<'a>> {
        trace_cur!(self, parse_prop_name);
        let start = self.input().cur_pos();
        let cur = self.input().cur();
        let v = if cur == Token::Str {
            let str_lit = self.parse_str_lit();
            PropName::Str(self.boxed(str_lit))
        } else if cur == Token::Num {
            let raw = self.atom_from_span(self.input.cur_span());
            let value = self.input_mut().expect_number_token_value();
            self.bump();
            self.ast
                .prop_name_number(self.span(start), value, raw.into())
        } else if cur == Token::BigInt {
            let raw = self.atom_from_span(self.input.cur_span());
            let value = self.input_mut().expect_bigint_token_value();
            self.bump();
            self.ast
                .prop_name_big_int(self.span(start), value, raw.into())
        } else if cur.is_word() {
            let w = self.input_mut().expect_word_token_and_bump();
            self.ast.prop_name_ident_name(self.span(start), w)
        } else if cur == Token::LBracket {
            self.bump();
            let inner_start = self.input().cur_pos();
            let mut expr = self.allow_in_expr(Self::parse_assignment_expr)?;
            if self.syntax().typescript() && self.input().is(Token::Comma) {
                let exprs = self.collect_vec(|p, exprs| {
                    exprs.push(expr);
                    while p.input_mut().eat(Token::Comma) {
                        let expr = p.allow_in_expr(Self::parse_assignment_expr)?;
                        exprs.push(expr);
                    }
                    p.emit_err(p.span(inner_start), SyntaxError::TS1171);
                    Ok(())
                })?;
                expr = self.ast.expr_seq_expr(self.span(inner_start), exprs);
            }
            expect!(self, Token::RBracket);
            self.ast
                .prop_name_computed_prop_name(self.span(start), expr)
        } else {
            unexpected!(
                self,
                "identifier, string literal, numeric literal or [ for the computed key"
            )
        };
        Ok(v)
    }
}
