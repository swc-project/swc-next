use swc_core::common::{DUMMY_SP, Span};
use swc_experimental_ecma_ast::*;

use crate::lexer::MaybeSubUtf8;
use crate::parser::js::is_not_this;
use crate::{Context, PResult, Parser, error::SyntaxError, input::Tokens, lexer::Token};

impl<I: Tokens> Parser<I> {
    pub(crate) fn parse_object<Object, ObjectProp: ExtraDataCompact>(
        &mut self,
        parse_prop: impl Fn(&mut Self) -> PResult<ObjectProp>,
        make_object: impl Fn(
            &mut Self,
            Span,
            TypedSubRange<ObjectProp>,
            Option<Span>,
        ) -> PResult<Object>,
    ) -> PResult<Object> {
        self.do_outside_of_context(Context::WillExpectColonForCond, |p| {
            trace_cur!(p, parse_object);

            let start = p.cur_pos();
            let mut trailing_comma = None;
            p.assert_and_bump(Token::LBrace);

            let props = p.scratch_start(|p, props| {
                while !p.input_mut().eat(Token::RBrace) {
                    let prop = parse_prop(p)?;
                    props.push(p, prop);

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
    pub(crate) fn parse_binding_object_prop(&mut self) -> PResult<ObjectPatProp> {
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

            return Ok(self.ast.object_pat_prop_key_value_pat_prop(
                Span::new_with_checked(key.span_lo(&self.ast), key.span_hi(&self.ast)),
                key,
                value,
            ));
        }
        let key = match key {
            PropName::Ident(ident) => ident,
            _ => unexpected!(self, "an identifier"),
        };

        let value = if self.input_mut().eat(Token::Eq) {
            self.allow_in_expr(Self::parse_assignment_expr).map(Some)?
        } else {
            if self
                .ctx()
                .is_reserved_word(self.ast.get_utf8(key.sym(&self.ast)))
            {
                self.emit_err(
                    key.span(&self.ast),
                    SyntaxError::ReservedWordInObjShorthandOrPat,
                );
            }

            None
        };

        let key_ident = self
            .ast
            .ident(key.span(&self.ast), key.sym(&self.ast), false);
        let key_ident = self.ast.binding_ident(key_ident.span(&self.ast), key_ident);
        Ok(self
            .ast
            .object_pat_prop_assign_pat_prop(self.span(start), key_ident, value))
    }

    fn make_binding_object(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        trailing_comma: Option<Span>,
    ) -> PResult<Pat> {
        let len = props.len();
        for (i, prop) in props.iter().enumerate() {
            let prop = self.ast.get_node_in_sub_range(prop);
            if i == len - 1 {
                if let ObjectPatProp::Rest(rest) = prop {
                    match rest.arg(&self.ast) {
                        Pat::Ident(..) => {
                            if let Some(trailing_comma) = trailing_comma {
                                self.emit_err(trailing_comma, SyntaxError::CommaAfterRestElement);
                            }
                        }
                        _ => syntax_error!(
                            self,
                            prop.span(&self.ast),
                            SyntaxError::DotsWithoutIdentifier
                        ),
                    }
                }
                continue;
            }

            if let ObjectPatProp::Rest(..) = prop {
                self.emit_err(prop.span(&self.ast), SyntaxError::NonLastRestParam)
            }
        }

        let optional = (self.input().syntax().dts() || self.ctx().contains(Context::InDeclare))
            && self.input_mut().eat(Token::QuestionMark);

        Ok(self.ast.pat_object_pat(span, props, optional))
    }

    pub(super) fn parse_object_pat(&mut self) -> PResult<Pat> {
        self.parse_object(Self::parse_binding_object_prop, Self::make_binding_object)
    }

    fn make_expr_object(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
        trailing_comma: Option<Span>,
    ) -> PResult<Expr> {
        if let Some(trailing_comma) = trailing_comma {
            self.state_mut()
                .trailing_commas
                .insert(span.lo, trailing_comma);
        }

        Ok(self.ast.expr_object_lit(span, props))
    }

    fn parse_expr_object_prop(&mut self) -> PResult<PropOrSpread> {
        trace_cur!(self, parse_object_prop);

        let start = self.cur_pos();
        // Parse as 'MethodDefinition'

        if self.input_mut().eat(Token::DotDotDot) {
            // spread element
            let dot3_token = self.span(start);

            let expr = self.allow_in_expr(Self::parse_assignment_expr)?;

            return Ok(self
                .ast
                .prop_or_spread_spread_element(dot3_token, dot3_token, expr));
        }

        if self.input_mut().eat(Token::Asterisk) {
            let name = self.parse_prop_name()?;
            return self
                .do_inside_of_context(Context::AllowDirectSuper, |p| {
                    p.do_outside_of_context(Context::InClassField, |p| {
                        p.parse_fn_args_body(
                            // no decorator in an object literal
                            TypedSubRange::empty(),
                            start,
                            Self::parse_unique_formal_params,
                            false,
                            true,
                        )
                    })
                })
                .map(|function| {
                    self.ast.prop_or_spread_prop_method_prop(
                        function.span(&self.ast),
                        name,
                        function,
                    )
                });
        }

        let has_modifiers = false;
        // let has_modifiers = self.eat_any_ts_modifier()?;
        let modifiers_span = self.input().prev_span();

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
            let value = self.ast.invalid(self.span(start));
            return Ok(self.ast.prop_or_spread_prop_key_value_prop(
                Span::new_with_checked(key.span_lo(&self.ast), self.last_pos()),
                key,
                Expr::Invalid(value),
            ));
        }
        //
        // {[computed()]: a,}
        // { 'a': a, }
        // { 0: 1, }
        // { a: expr, }
        if self.input_mut().eat(Token::Colon) {
            let value = self.allow_in_expr(Self::parse_assignment_expr)?;
            let span = Span::new_with_checked(key.span_lo(&self.ast), value.span_hi(&self.ast));
            return Ok(self
                .ast
                .prop_or_spread_prop_key_value_prop(span, key, value));
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
                            TypedSubRange::empty(),
                            start,
                            Self::parse_unique_formal_params,
                            false,
                            false,
                        )
                    })
                })
                .map(|function| {
                    self.ast.prop_or_spread_prop_method_prop(
                        function.span(&self.ast),
                        key,
                        function,
                    )
                });
        }

        let ident = match key {
            PropName::Ident(ident) => ident,
            // TODO
            _ => unexpected!(self, "identifier"),
        };
        let ident_span = ident.span(&self.ast);
        let ident_sym = ident.sym(&self.ast);

        if self.input_mut().eat(Token::QuestionMark) {
            self.emit_err(self.input().prev_span(), SyntaxError::TS1162);
        }

        // `ident` from parse_prop_name is parsed as 'IdentifierName'
        // It means we should check for invalid expressions like { for, }
        let cur = self.input().cur();
        if matches!(cur, Token::Eq | Token::Comma | Token::RBrace) {
            if self.ctx().is_reserved_word(self.ast.get_utf8(ident_sym)) {
                self.emit_err(ident_span, SyntaxError::ReservedWordInObjShorthandOrPat);
            }

            let ident = self.ast.ident(ident_span, ident_sym, false);
            if self.input_mut().eat(Token::Eq) {
                let value = self.allow_in_expr(Self::parse_assignment_expr)?;
                let span = self.span(start);
                return Ok(self.ast.prop_or_spread_prop_assign_prop(span, ident, value));
            }

            return Ok(PropOrSpread::Prop(Prop::Shorthand(ident)));
        }

        // get a(){}
        // set a(v){}
        // async a(){}

        let ident = self.ast.get_utf8(ident_sym);
        let is_get = ident == "get";
        let is_set = ident == "set";
        let is_async = ident == "async";
        if is_get || is_set || is_async {
            trace_cur!(self, parse_object_prop__after_accessor);

            if has_modifiers {
                self.emit_err(modifiers_span, SyntaxError::TS1042);
            }

            let is_generator = is_async && self.input_mut().eat(Token::Asterisk);
            let key = self.parse_prop_name()?;
            let key_span = key.span(&self.ast);
            self.do_inside_of_context(Context::AllowDirectSuper, |p| {
                p.do_outside_of_context(Context::InClassField, |p| {
                    if is_get {
                        return p
                            .parse_fn_args_body(
                                // no decorator in an object literal
                                TypedSubRange::empty(),
                                start,
                                |p| {
                                    let params = p.parse_formal_params()?;

                                    if params.iter().any(|param| {
                                        is_not_this(&p.ast, p.ast.get_node_in_sub_range(param))
                                    }) {
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

                                let body = function.body(&p.ast);
                                p.ast
                                    .prop_or_spread_prop_getter_prop(p.span(start), key, body)
                            });
                    }

                    if is_set {
                        return p
                            .parse_fn_args_body(
                                // no decorator in an object literal
                                TypedSubRange::empty(),
                                start,
                                |p| {
                                    let params = p.parse_formal_params()?;

                                    if params
                                        .iter()
                                        .filter(|param| {
                                            is_not_this(&p.ast, p.ast.get_node_in_sub_range(*param))
                                        })
                                        .count()
                                        != 1
                                    {
                                        p.emit_err(key_span, SyntaxError::SetterParam);
                                    }

                                    if !params.is_empty()
                                        && let Pat::Rest(rest) = p
                                            .ast
                                            .get_node_in_sub_range(params.get(0).unwrap())
                                            .pat(&p.ast)
                                    {
                                        p.emit_err(rest.span(&p.ast), SyntaxError::RestPatInSetter);
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
                                let mut params = function.params(&p.ast);
                                if params.len() >= 2 {
                                    this = Some(
                                        p.ast
                                            .get_node_in_sub_range(params.remove_first())
                                            .pat(&p.ast),
                                    );
                                }

                                if params.len() != 1 {
                                    p.emit_err(key_span, SyntaxError::SetterParam);
                                }

                                let param = match params.iter().next() {
                                    Some(param) => {
                                        let param = p.ast.get_node_in_sub_range(param);
                                        param.pat(&p.ast)
                                    }
                                    None => {
                                        p.emit_err(key_span, SyntaxError::SetterParam);
                                        p.ast.pat_invalid(DUMMY_SP)
                                    }
                                };

                                let body = function.body(&p.ast);
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
                                TypedSubRange::empty(),
                                start,
                                Self::parse_unique_formal_params,
                                true,
                                is_generator,
                            )
                            .map(|function| {
                                let span = Span::new_with_checked(
                                    key.span_lo(&p.ast),
                                    key.span_hi(&p.ast),
                                );
                                p.ast.prop_or_spread_prop_method_prop(span, key, function)
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

    pub(crate) fn parse_object_expr(&mut self) -> PResult<Expr> {
        self.parse_object(Self::parse_expr_object_prop, Self::make_expr_object)
    }

    /// spec: 'PropertyName'
    pub fn parse_prop_name(&mut self) -> PResult<PropName> {
        trace_cur!(self, parse_prop_name);
        self.do_inside_of_context(Context::InPropertyName, |p| {
            let start = p.input().cur_pos();
            let cur = p.input().cur();
            let v = if cur == Token::Str {
                PropName::Str(p.parse_str_lit())
            } else if cur == Token::Num {
                let raw = p.to_utf8_ref(MaybeSubUtf8::new_from_span(p.input.cur_span()));
                let value = p.input_mut().expect_number_token_value();
                p.bump();
                p.ast.prop_name_number(p.span(start), value, raw.into())
            } else if cur == Token::BigInt {
                let raw = p.to_utf8_ref(MaybeSubUtf8::new_from_span(p.input.cur_span()));
                let value = p.input_mut().expect_bigint_token_value();
                let value = p.ast.add_bigint(*value);
                p.bump();
                p.ast.prop_name_big_int(p.span(start), value, raw.into())
            } else if cur.is_word() {
                let w = p.input_mut().expect_word_token_and_bump();
                let w = p.to_utf8_ref(w);
                p.ast.prop_name_ident_name(p.span(start), w)
            } else if cur == Token::LBracket {
                p.bump();
                let inner_start = p.input().cur_pos();
                let mut expr = p.allow_in_expr(Self::parse_assignment_expr)?;
                if p.syntax().typescript() && p.input().is(Token::Comma) {
                    let exprs = p.scratch_start(|p, exprs| {
                        exprs.push(p, expr);
                        while p.input_mut().eat(Token::Comma) {
                            let expr = p.allow_in_expr(Self::parse_assignment_expr)?;
                            exprs.push(p, expr);
                        }
                        p.emit_err(p.span(inner_start), SyntaxError::TS1171);
                        Ok(())
                    })?;
                    expr = p.ast.expr_seq_expr(p.span(inner_start), exprs);
                }
                expect!(p, Token::RBracket);
                p.ast.prop_name_computed_prop_name(p.span(start), expr)
            } else {
                unexpected!(
                    p,
                    "identifier, string literal, numeric literal or [ for the computed key"
                )
            };
            Ok(v)
        })
    }
}
