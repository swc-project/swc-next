//! 13.3.3 Destructuring Binding Patterns

use crate::{
    Context, PResult,
    error::SyntaxError,
    input::Tokens,
    lexer::Token,
    parser::{Parser, js::expr::AssignTargetOrSpread, util::ExprExt},
};
use swc_core::common::BytePos;
use swc_experimental_ecma_ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PatType {
    BindingPat,
    BindingElement,
    /// AssignmentPattern
    AssignPat,
    AssignElement,
}

impl PatType {
    fn element(self) -> Self {
        match self {
            PatType::BindingPat | PatType::BindingElement => PatType::BindingElement,
            PatType::AssignPat | PatType::AssignElement => PatType::AssignElement,
        }
    }
}

impl<I: Tokens> Parser<I> {
    pub fn parse_pat(&mut self) -> PResult<Pat> {
        self.parse_binding_pat_or_ident(false)
    }

    /// argument of arrow is pattern, although idents in pattern is already
    /// checked if is a keyword, it should also be checked if is arguments or
    /// eval
    fn pat_is_valid_argument_in_strict(&mut self, pat: Pat) {
        debug_assert!(self.ctx().contains(Context::Strict));
        match pat {
            Pat::Ident(i) => {
                if i.is_reserved_in_strict_bind(&self.ast) {
                    self.emit_strict_mode_err(
                        i.span(&self.ast),
                        SyntaxError::EvalAndArgumentsInStrict,
                    )
                }
            }
            Pat::Array(arr) => {
                for pat in arr.elems(&self.ast).iter() {
                    let pat = self.ast.get_node_in_sub_range(pat);
                    if let Some(pat) = pat {
                        self.pat_is_valid_argument_in_strict(pat);
                    }
                }
            }
            Pat::Rest(r) => self.pat_is_valid_argument_in_strict(r.arg(&self.ast)),
            Pat::Object(obj) => {
                for prop in obj.props(&self.ast).iter() {
                    let prop = self.ast.get_node_in_sub_range(prop);
                    match prop {
                        ObjectPatProp::KeyValue(key_value) => {
                            self.pat_is_valid_argument_in_strict(key_value.value(&self.ast))
                        }
                        ObjectPatProp::Rest(rest) => {
                            self.pat_is_valid_argument_in_strict(rest.arg(&self.ast))
                        }
                        ObjectPatProp::Assign(assign) => {
                            let key = assign.key(&self.ast);
                            if key.is_reserved_in_strict_bind(&self.ast) {
                                self.emit_strict_mode_err(
                                    key.span(&self.ast),
                                    SyntaxError::EvalAndArgumentsInStrict,
                                )
                            }
                        }
                        #[cfg(swc_ast_unknown)]
                        _ => unreachable!(),
                    }
                }
            }
            Pat::Assign(a) => self.pat_is_valid_argument_in_strict(a.left(&self.ast)),
            Pat::Invalid(_) | Pat::Expr(_) => (),
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        }
    }

    /// This does not return 'rest' pattern because non-last parameter cannot be
    /// rest.
    pub(super) fn reparse_expr_as_pat(&mut self, pat_ty: PatType, expr: Expr) -> PResult<Pat> {
        if let Expr::Invalid(i) = expr {
            return Ok(Pat::Invalid(i));
        }
        if pat_ty == PatType::AssignPat {
            match expr {
                Expr::Object(..) | Expr::Array(..) => {
                    // It is a Syntax Error if LeftHandSideExpression is either
                    // an ObjectLiteral or an ArrayLiteral
                    // and LeftHandSideExpression cannot
                    // be reparsed as an AssignmentPattern.
                }
                _ => {
                    self.check_assign_target(expr, true);
                }
            }
        }
        self.reparse_expr_as_pat_inner(pat_ty, expr)
    }

    fn reparse_expr_as_pat_inner(&mut self, pat_ty: PatType, expr: Expr) -> PResult<Pat> {
        // In dts, we do not reparse.
        debug_assert!(!self.input().syntax().dts());
        let span = expr.span(&self.ast);
        if pat_ty == PatType::AssignPat {
            match expr {
                Expr::Object(..) | Expr::Array(..) => {
                    // It is a Syntax Error if LeftHandSideExpression is either
                    // an ObjectLiteral or an ArrayLiteral
                    // and LeftHandSideExpression cannot
                    // be reparsed as an AssignmentPattern.
                }

                _ => match expr {
                    // It is a Syntax Error if the LeftHandSideExpression is
                    // CoverParenthesizedExpressionAndArrowParameterList:(Expression) and
                    // Expression derives a phrase that would produce a Syntax Error according
                    // to these rules if that phrase were substituted for
                    // LeftHandSideExpression. This rule is recursively applied.
                    Expr::Paren(..) => {
                        return Ok(Pat::Expr(expr));
                    }
                    Expr::Ident(i) => {
                        let i = self.ast.binding_ident(i.span(&self.ast), i);
                        return Ok(Pat::Ident(i));
                    }
                    _ => {
                        return Ok(Pat::Expr(expr));
                    }
                },
            }
        }

        // AssignmentElement:
        //      DestructuringAssignmentTarget Initializer[+In]?
        //
        // DestructuringAssignmentTarget:
        //      LeftHandSideExpression
        if pat_ty == PatType::AssignElement {
            match expr {
                Expr::Array(..) | Expr::Object(..) => {}
                Expr::Member(..)
                | Expr::SuperProp(..)
                | Expr::Call(..)
                | Expr::New(..)
                | Expr::Lit(..)
                | Expr::Ident(..)
                | Expr::Fn(..)
                | Expr::Class(..)
                | Expr::Paren(..)
                // | Expr::TsAs(..) 
                | Expr::Tpl(..)=> {
                    if !expr.is_valid_simple_assignment_target(&self.ast, self.ctx().contains(Context::Strict))
                    {
                        self.emit_err(span, SyntaxError::NotSimpleAssign)
                    }
                    match expr {
                        Expr::Ident(i) => {
                            let i = self.ast.binding_ident(i.span(&self.ast), i);
                            return Ok(Pat::Ident(i))
                        },
                        _ => {
                            return Ok(Pat::Expr(expr));
                        }
                    }
                }
                // It's special because of optional initializer
                Expr::Assign(..) => {}
                _ => self.emit_err(span, SyntaxError::InvalidPat),
            }
        }

        match expr {
            Expr::Paren(..) => {
                self.emit_err(span, SyntaxError::InvalidPat);
                Ok(self.ast.pat_invalid(span))
            }
            Expr::Assign(assign_expr) => {
                let span = assign_expr.span(&self.ast);
                if assign_expr.op(&self.ast) != AssignOp::Assign {
                    self.emit_err(span, SyntaxError::InvalidPat);
                    return Ok(self.ast.pat_invalid(span));
                }

                let left = match assign_expr.left(&self.ast) {
                    AssignTarget::Simple(left) => {
                        let left = match left {
                            SimpleAssignTarget::Ident(binding_ident) => {
                                let sym = binding_ident.id(&self.ast);
                                Expr::Ident(sym)
                            }
                            SimpleAssignTarget::Member(member_expr) => Expr::Member(member_expr),
                            SimpleAssignTarget::SuperProp(super_prop_expr) => {
                                Expr::SuperProp(super_prop_expr)
                            }
                            SimpleAssignTarget::Paren(paren_expr) => Expr::Paren(paren_expr),
                            SimpleAssignTarget::OptChain(opt_chain_expr) => {
                                Expr::OptChain(opt_chain_expr)
                            }
                            SimpleAssignTarget::Invalid(invalid) => Expr::Invalid(invalid),
                        };
                        self.reparse_expr_as_pat(pat_ty, left)?
                    }
                    AssignTarget::Pat(pat) => match pat {
                        AssignTargetPat::Array(array_pat) => Pat::Array(array_pat),
                        AssignTargetPat::Object(object_pat) => Pat::Object(object_pat),
                        AssignTargetPat::Invalid(invalid) => Pat::Invalid(invalid),
                    },
                    #[cfg(swc_ast_unknown)]
                    _ => unreachable!(),
                };
                let right = assign_expr.right(&self.ast);
                Ok(self.ast.pat_assign_pat(span, left, right))
            }
            Expr::Object(object) => {
                // {}
                let object_span = object.span(&self.ast);
                let props = object.props(&self.ast);
                let len = props.len();

                let props = self.scratch_start(|p, obj_props| {
                    for (idx, prop) in props.iter().enumerate() {
                        let prop = p.ast.get_node_in_sub_range(prop);
                        let span = prop.span(&p.ast);
                        let prop = match prop {
                            PropOrSpread::Prop(prop) => match prop {
                                Prop::Shorthand(id) => {
                                    let binding_ident = p.ast.binding_ident(id.span(&p.ast), id);
                                    p.ast.object_pat_prop_assign_pat_prop(
                                        id.span(&p.ast),
                                        binding_ident,
                                        None,
                                    )
                                }
                                Prop::KeyValue(kv_prop) => {
                                    let pat = p.reparse_expr_as_pat(
                                        pat_ty.element(),
                                        kv_prop.value(&p.ast),
                                    )?;
                                    p.ast.object_pat_prop_key_value_pat_prop(
                                        span,
                                        kv_prop.key(&p.ast),
                                        pat,
                                    )
                                }
                                Prop::Assign(assign_prop) => {
                                    let id = assign_prop.key(&p.ast);
                                    let key = p.ast.binding_ident(id.span(&p.ast), id);
                                    let value = assign_prop.value(&p.ast);
                                    p.ast
                                        .object_pat_prop_assign_pat_prop(span, key, Some(value))
                                }
                                _ => syntax_error!(p, prop.span(&p.ast), SyntaxError::InvalidPat),
                            },

                            PropOrSpread::SpreadElement(spread) => {
                                let dot_3_token = spread.dot_3_token(&p.ast);
                                let expr = spread.expr(&p.ast);
                                if idx != len - 1 {
                                    p.emit_err(span, SyntaxError::NonLastRestParam)
                                } else if let Some(trailing_comma) =
                                    p.state().trailing_commas.get(&object_span.lo)
                                {
                                    p.emit_err(*trailing_comma, SyntaxError::CommaAfterRestElement);
                                };

                                let element_pat_ty = pat_ty.element();
                                let pat = if let PatType::BindingElement = element_pat_ty {
                                    if let Expr::Ident(i) = expr {
                                        Pat::Ident(p.ast.binding_ident(i.span(&p.ast), i))
                                    } else {
                                        p.emit_err(span, SyntaxError::DotsWithoutIdentifier);
                                        p.ast.pat_invalid(span)
                                    }
                                } else {
                                    p.reparse_expr_as_pat(element_pat_ty, expr)?
                                };
                                if let Pat::Assign(_) = pat {
                                    p.emit_err(span, SyntaxError::TS1048)
                                };
                                p.ast.object_pat_prop_rest_pat(span, dot_3_token, pat)
                            }
                            #[cfg(swc_ast_unknown)]
                            _ => unreachable!(),
                        };

                        obj_props.push(p, prop);
                    }

                    Ok(())
                })?;

                Ok(self.ast.pat_object_pat(object_span, props, false))
            }
            Expr::Ident(ident) => Ok(Pat::Ident(
                self.ast.binding_ident(ident.span(&self.ast), ident),
            )),
            Expr::Array(array) => {
                let mut exprs = array.elems(&self.ast);
                if exprs.is_empty() {
                    return Ok(self.ast.pat_array_pat(span, TypedSubRange::empty(), false));
                }
                // Trailing comma may exist. We should remove those commas.
                let count_of_trailing_comma = exprs
                    .iter()
                    .rev()
                    .take_while(|e| self.ast.get_node_in_sub_range(*e).is_none())
                    .count();
                let len = exprs.len();
                let mut params = Vec::with_capacity(exprs.len() - count_of_trailing_comma);
                // Comma or other pattern cannot follow a rest pattern.
                let idx_of_rest_not_allowed = if count_of_trailing_comma == 0 {
                    len - 1
                } else {
                    // last element is comma, so rest is not allowed for every pattern element.
                    len - count_of_trailing_comma
                };

                let after = exprs.split_off(idx_of_rest_not_allowed);
                for expr in exprs.iter() {
                    let expr = self.ast.get_node_in_sub_range(expr);
                    match expr {
                        Some(expr_or_spread) => match expr_or_spread.spread(&self.ast) {
                            Some(_) => self.emit_err(
                                expr_or_spread.expr(&self.ast).span(&self.ast),
                                SyntaxError::NonLastRestParam,
                            ),
                            None => {
                                let expr = expr_or_spread.expr(&self.ast);
                                params.push(Some(self.reparse_expr_as_pat(pat_ty.element(), expr)?))
                            }
                        },
                        None => params.push(None),
                    }
                }

                let exprs = after;
                if count_of_trailing_comma == 0 {
                    let expr = self.ast.get_node_in_sub_range(exprs.iter().next().unwrap());
                    let last = match expr {
                        // Rest
                        Some(expr_or_spread) => {
                            let spread = expr_or_spread.spread(&self.ast);
                            let expr = expr_or_spread.expr(&self.ast);
                            match spread {
                                Some(spread) => {
                                    // TODO: is BindingPat correct?
                                    if let Expr::Assign(_) = expr {
                                        self.emit_err(expr.span(&self.ast), SyntaxError::TS1048);
                                    };
                                    if let Some(trailing_comma) =
                                        self.state().trailing_commas.get(&span.lo)
                                    {
                                        self.emit_err(
                                            *trailing_comma,
                                            SyntaxError::CommaAfterRestElement,
                                        );
                                    }
                                    let expr_span = expr.span(&self.ast);
                                    let spread_span = spread.span(&self.ast);

                                    Some(self.reparse_expr_as_pat(pat_ty.element(), expr).map(
                                        |pat| self.ast.pat_rest_pat(expr_span, spread_span, pat),
                                    )?)
                                }
                                None => {
                                    // TODO: is BindingPat correct?
                                    Some(self.reparse_expr_as_pat(pat_ty.element(), expr)?)
                                }
                            }
                        }
                        // TODO: syntax error if last element is ellison and ...rest exists.
                        None => None,
                    };
                    params.push(last);
                }

                let params = self.ast.add_typed_sub_range(params);
                Ok(self.ast.pat_array_pat(span, params, false))
            }

            // Invalid patterns.
            // Note that assignment expression with '=' is valid, and handled above.
            Expr::Lit(..) => {
                self.emit_err(span, SyntaxError::InvalidPat);
                Ok(self.ast.pat_invalid(span))
            }

            Expr::Yield(..) if self.ctx().contains(Context::InGenerator) => {
                self.emit_err(span, SyntaxError::InvalidPat);
                Ok(self.ast.pat_invalid(span))
            }

            _ => {
                self.emit_err(span, SyntaxError::InvalidPat);

                Ok(self.ast.pat_invalid(span))
            }
        }
    }

    pub(super) fn parse_binding_element(&mut self) -> PResult<Pat> {
        trace_cur!(self, parse_binding_element);

        let start = self.cur_pos();
        let left = self.parse_binding_pat_or_ident(false)?;

        if self.input_mut().eat(Token::Eq) {
            let right = self.allow_in_expr(Self::parse_assignment_expr)?;

            if self.ctx().contains(Context::InDeclare) {
                self.emit_err(self.span(start), SyntaxError::TS2371);
            }

            return Ok(self.ast.pat_assign_pat(self.span(start), left, right));
        }

        Ok(left)
    }

    pub(crate) fn parse_binding_pat_or_ident(&mut self, disallow_let: bool) -> PResult<Pat> {
        trace_cur!(self, parse_binding_pat_or_ident);

        let cur = self.input().cur();
        if cur.is_word() {
            let ident = self.parse_binding_ident(disallow_let)?;
            Ok(Pat::Ident(
                self.ast.binding_ident(ident.span(&self.ast), ident),
            ))
        } else if cur == Token::LBracket {
            self.parse_array_binding_pat()
        } else if cur == Token::LBrace {
            self.parse_object_pat()
        } else if cur == Token::Error {
            let err = self.input_mut().expect_error_token_and_bump();
            Err(err)
        } else {
            unexpected!(self, "yield, an identifier, [ or {")
        }
    }

    fn parse_array_binding_pat(&mut self) -> PResult<Pat> {
        let start = self.cur_pos();

        self.assert_and_bump(Token::LBracket);

        let mut elems = Vec::new();

        let mut rest_span = Span::default();

        while !self.input().is(Token::RBracket) {
            if self.input_mut().eat(Token::Comma) {
                elems.push(None);
                continue;
            }

            if !rest_span.is_dummy() {
                self.emit_err(rest_span, SyntaxError::NonLastRestParam);
            }

            let start = self.cur_pos();

            let mut is_rest = false;
            if self.input_mut().eat(Token::DotDotDot) {
                is_rest = true;
                let dot3_token = self.span(start);

                let pat = self.parse_binding_pat_or_ident(false)?;
                rest_span = self.span(start);

                let pat = self.ast.pat_rest_pat(rest_span, dot3_token, pat);
                elems.push(Some(pat));
            } else {
                elems.push(Some(self.parse_binding_element()?));
            }

            if !self.input().is(Token::RBracket) {
                expect!(self, Token::Comma);
                if is_rest && self.input().is(Token::RBracket) {
                    self.emit_err(self.input().prev_span(), SyntaxError::CommaAfterRestElement);
                }
            }
        }

        expect!(self, Token::RBracket);
        let optional = (self.input().syntax().dts() || self.ctx().contains(Context::InDeclare))
            && self.input_mut().eat(Token::QuestionMark);

        let elems = self.ast.add_typed_sub_range(elems);
        Ok(self.ast.pat_array_pat(self.span(start), elems, optional))
    }

    /// spec: 'FormalParameter'
    ///
    /// babel: `parseAssignableListItem`
    fn parse_formal_param_pat(&mut self) -> PResult<Pat> {
        let start = self.cur_pos();

        // let has_modifier = self.eat_any_ts_modifier()?;

        // let pat_start = self.cur_pos();
        let pat = self.parse_binding_element()?;
        let opt = false;

        // if self.input().syntax().typescript() {
        //     if self.input_mut().eat(Token::QuestionMark) {
        //         match pat {
        //             Pat::Ident(BindingIdent {
        //                 id:
        //                     Ident {
        //                         ref mut optional, ..
        //                     },
        //                 ..
        //             })
        //             | Pat::Array(ArrayPat {
        //                 ref mut optional, ..
        //             })
        //             | Pat::Object(ObjectPat {
        //                 ref mut optional, ..
        //             }) => {
        //                 *optional = true;
        //                 opt = true;
        //             }
        //             _ if self.input().syntax().dts() || self.ctx().contains(Context::InDeclare) => {
        //             }
        //             _ => {
        //                 syntax_error!(
        //                     self,
        //                     self.input().prev_span(),
        //                     SyntaxError::TsBindingPatCannotBeOptional
        //                 );
        //             }
        //         }
        //     }

        //     match pat {
        //         Pat::Array(ArrayPat {
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

        //         Pat::Ident(BindingIdent {
        //             ref mut type_ann, ..
        //         }) => {
        //             let new_type_ann = self.try_parse_ts_type_ann()?;
        //             *type_ann = new_type_ann;
        //         }

        //         Pat::Assign(AssignPat { ref mut span, .. }) => {
        //             if (self.try_parse_ts_type_ann()?).is_some() {
        //                 *span = Span::new_with_checked(pat_start, self.input().prev_span().hi);
        //                 self.emit_err(*span, SyntaxError::TSTypeAnnotationAfterAssign);
        //             }
        //         }
        //         Pat::Invalid(..) => {}
        //         // _ => unreachable!("invalid syntax: Pat: {:?}", pat),
        //         _ => unreachable!("invalid syntax: Pat: {:?}", "pat"),
        //     }
        // }

        let pat = if self.input_mut().eat(Token::Eq) {
            // `=` cannot follow optional parameter.
            if opt {
                self.emit_err(pat.span(&self.ast), SyntaxError::TS1015);
            }

            let right = self.parse_assignment_expr()?;
            if self.ctx().contains(Context::InDeclare) {
                self.emit_err(self.span(start), SyntaxError::TS2371);
            }

            return Ok(self.ast.pat_assign_pat(self.span(start), pat, right));
        } else {
            pat
        };

        // if has_modifier {
        //     self.emit_err(self.span(start), SyntaxError::TS2369);
        //     return Ok(pat);
        // }

        Ok(pat)
    }

    fn parse_constructor_param(
        &mut self,
        param_start: BytePos,
        decorators: TypedSubRange<Decorator>,
    ) -> PResult<ParamOrTsParamProp> {
        // let (accessibility, is_override, readonly) = if self.input().syntax().typescript() {
        //     let accessibility = self.parse_access_modifier()?;
        //     (
        //         accessibility,
        //         self.parse_ts_modifier(&["override"], false)?.is_some(),
        //         self.parse_ts_modifier(&["readonly"], false)?.is_some(),
        //     )
        // } else {
        //     (None, false, false)
        // };
        // if accessibility.is_none() && !is_override && !readonly {
        //     let pat = self.parse_formal_param_pat()?;
        //     Ok(ParamOrTsParamProp::Param(Param {
        //         span: self.span(param_start),
        //         decorators,
        //         pat,
        //     }))
        // } else {
        //     let param = match self.parse_formal_param_pat()? {
        //         Pat::Ident(i) => TsParamPropParam::Ident(i),
        //         Pat::Assign(a) => TsParamPropParam::Assign(a),
        //         node => syntax_error!(self, node.span(), SyntaxError::TsInvalidParamPropPat),
        //     };
        //     Ok(ParamOrTsParamProp::TsParamProp(TsParamProp {
        //         span: self.span(param_start),
        //         accessibility,
        //         is_override,
        //         readonly,
        //         decorators,
        //         param,
        //     }))
        // }
        let pat = self.parse_formal_param_pat()?;
        Ok(self
            .ast
            .param_or_ts_param_prop_param(self.span(param_start), decorators, pat))
    }

    pub(crate) fn parse_constructor_params(
        &mut self,
    ) -> PResult<TypedSubRange<ParamOrTsParamProp>> {
        self.scratch_start(|p, params| {
            let mut rest_span = Span::default();

            while !p.input().is(Token::RParen) {
                if !rest_span.is_dummy() {
                    p.emit_err(rest_span, SyntaxError::TS1014);
                }

                let param_start = p.cur_pos();
                let decorators = p.parse_decorators(false)?;
                let pat_start = p.cur_pos();

                let mut is_rest = false;
                if p.input_mut().eat(Token::DotDotDot) {
                    is_rest = true;
                    let dot3_token = p.span(pat_start);

                    let pat = p.parse_binding_pat_or_ident(false)?;
                    // let type_ann =
                    //     if p.input().syntax().typescript() && p.input().is(Token::Colon) {
                    //         let cur_pos = p.cur_pos();
                    //         Some(p.parse_ts_type_ann(/* eat_colon */ true, cur_pos)?)
                    //     } else {
                    //         None
                    //     };

                    rest_span = p.span(pat_start);
                    let pat = p.ast.pat_rest_pat(rest_span, dot3_token, pat);
                    let param =
                        p.ast
                            .param_or_ts_param_prop_param(p.span(param_start), decorators, pat);
                    params.push(p, param);
                } else {
                    let param = p.parse_constructor_param(param_start, decorators)?;
                    params.push(p, param);
                }

                if !p.input().is(Token::RParen) {
                    expect!(p, Token::Comma);
                    if p.input().is(Token::RParen) && is_rest {
                        p.emit_err(p.input().prev_span(), SyntaxError::CommaAfterRestElement);
                    }
                }
            }
            Ok(())
        })
    }

    pub(crate) fn parse_formal_params(&mut self) -> PResult<TypedSubRange<Param>> {
        self.scratch_start(|p, params| {
            let mut rest_span = Span::default();

            while !p.input().is(Token::RParen) {
                if !rest_span.is_dummy() {
                    p.emit_err(rest_span, SyntaxError::TS1014);
                }

                let param_start = p.cur_pos();
                let decorators = p.parse_decorators(false)?;
                let pat_start = p.cur_pos();

                let pat = if p.input_mut().eat(Token::DotDotDot) {
                    let dot3_token = p.span(pat_start);

                    let mut pat = p.parse_binding_pat_or_ident(false)?;

                    if p.input_mut().eat(Token::Eq) {
                        let right = p.parse_assignment_expr()?;
                        p.emit_err(pat.span(&p.ast), SyntaxError::TS1048);
                        pat = p.ast.pat_assign_pat(p.span(pat_start), pat, right);
                    }

                    // let type_ann =
                    //     if p.input().syntax().typescript() && p.input().is(Token::Colon) {
                    //         let cur_pos = p.cur_pos();
                    //         let ty = p.parse_ts_type_ann(/* eat_colon */ true, cur_pos)?;
                    //         Some(ty)
                    //     } else {
                    //         None
                    //     };

                    rest_span = p.span(pat_start);
                    let pat = p.ast.pat_rest_pat(rest_span, dot3_token, pat);

                    if p.syntax().typescript() && p.input_mut().eat(Token::QuestionMark) {
                        p.emit_err(p.input().prev_span(), SyntaxError::TS1047);
                        //
                    }

                    pat
                } else {
                    p.parse_formal_param_pat()?
                };
                let is_rest = matches!(pat, Pat::Rest(_));

                let param = p.ast.param(p.span(param_start), decorators, pat);
                params.push(p, param);

                if !p.input().is(Token::RParen) {
                    expect!(p, Token::Comma);
                    if is_rest && p.input().is(Token::RParen) {
                        p.emit_err(p.input().prev_span(), SyntaxError::CommaAfterRestElement);
                    }
                }
            }

            Ok(())
        })
    }

    pub(crate) fn parse_unique_formal_params(&mut self) -> PResult<TypedSubRange<Param>> {
        // FIXME: This is wrong
        self.parse_formal_params()
    }

    pub(super) fn parse_paren_items_as_params(
        &mut self,
        mut exprs: Vec<AssignTargetOrSpread>,
        trailing_comma: Option<Span>,
    ) -> PResult<TypedSubRange<Pat>> {
        let pat_ty = PatType::BindingPat;

        let len = exprs.len();
        if len == 0 {
            return Ok(TypedSubRange::empty());
        }

        let params = self.scratch_start(|p, params| {
            for expr in exprs.drain(..len - 1) {
                match expr {
                    AssignTargetOrSpread::ExprOrSpread(expr_or_spread) => {
                        match expr_or_spread.spread(&p.ast) {
                            Some(_) => p.emit_err(expr.span(&p.ast), SyntaxError::TS1014),
                            None => {
                                let param =
                                    p.reparse_expr_as_pat(pat_ty, expr_or_spread.expr(&p.ast))?;
                                params.push(p, param);
                            }
                        }
                    }
                    AssignTargetOrSpread::Pat(Pat::Rest(..)) => {
                        p.emit_err(expr.span(&p.ast), SyntaxError::TS1014)
                    }
                    AssignTargetOrSpread::Pat(pat) => params.push(p, pat),
                }
            }

            debug_assert_eq!(exprs.len(), 1);
            let expr = exprs.pop().unwrap();
            let outer_expr_span = expr.span(&p.ast);
            let last = match expr {
                // Rest
                AssignTargetOrSpread::ExprOrSpread(expr_or_spread) => {
                    match expr_or_spread.spread(&p.ast) {
                        Some(dot3_token) => {
                            let expr = expr_or_spread.expr(&p.ast);
                            if let Expr::Assign(_) = expr {
                                p.emit_err(outer_expr_span, SyntaxError::TS1048)
                            };
                            if let Some(trailing_comma) = trailing_comma {
                                p.emit_err(trailing_comma, SyntaxError::CommaAfterRestElement);
                            }

                            let expr_span = expr.span(&p.ast);
                            let dot3_span = dot3_token.span(&p.ast);
                            p.reparse_expr_as_pat(pat_ty, expr)
                                .map(|pat| p.ast.pat_rest_pat(expr_span, dot3_span, pat))?
                        }
                        None => p.reparse_expr_as_pat(pat_ty, expr_or_spread.expr(&p.ast))?,
                    }
                }
                AssignTargetOrSpread::Pat(pat) => {
                    if let Some(trailing_comma) = trailing_comma
                        && let Pat::Rest(..) = pat
                    {
                        p.emit_err(trailing_comma, SyntaxError::CommaAfterRestElement);
                    }
                    pat
                }
            };
            params.push(p, last);

            Ok(())
        })?;

        if self.ctx().contains(Context::Strict) {
            for param in params.iter() {
                let param = self.ast.get_node_in_sub_range(param);
                self.pat_is_valid_argument_in_strict(param)
            }
        }
        Ok(params)
    }
}
