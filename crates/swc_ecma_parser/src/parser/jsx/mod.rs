use std::borrow::Cow;

use swc_core::atoms::Atom;
use swc_core::common::{BytePos, Span};
use swc_experimental_ecma_ast::*;

use super::{Parser, input::Tokens};
use crate::string_alloc::MaybeSubUtf8;
use crate::{
    Context, PResult,
    error::SyntaxError,
    lexer::{Token, TokenFlags},
};

impl<I: Tokens> Parser<I> {
    /// Parses JSX expression enclosed into curly brackets.
    fn parse_jsx_expr_container(&mut self) -> PResult<JSXExprContainer> {
        debug_assert!(self.input().syntax().jsx());
        debug_assert!(self.input().is(Token::LBrace));

        let start = self.input().cur_pos();
        self.bump(); // bump "{"
        let expr = if self.input().is(Token::RBrace) {
            JSXExpr::JSXEmptyExpr(self.parse_jsx_empty_expr())
        } else {
            self.parse_expr_inner().map(JSXExpr::Expr)?
        };
        expect!(self, Token::RBrace);

        Ok(self.ast.jsx_expr_container(self.span(start), expr))
    }

    /// JSXEmptyExpression is unique type since it doesn't actually parse
    /// anything, and so it should start at the end of last read token (left
    /// brace) and finish at the beginning of the next one (right brace).
    fn parse_jsx_empty_expr(&mut self) -> JSXEmptyExpr {
        debug_assert!(self.input().syntax().jsx());
        let start = self.input().cur_pos();

        self.ast
            .jsx_empty_expr(Span::new_with_checked(start, start))
    }

    fn jsx_expr_container_to_jsx_attr_value(
        &mut self,
        start: BytePos,
        node: JSXExprContainer,
    ) -> PResult<JSXAttrValue> {
        match node.expr(&self.ast) {
            JSXExpr::JSXEmptyExpr(..) => {
                syntax_error!(self, self.span(start), SyntaxError::EmptyJSXAttr)
            }
            JSXExpr::Expr(..) => Ok(JSXAttrValue::JSXExprContainer(node)),
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        }
    }

    fn parse_jsx_text(&mut self) -> JSXText {
        debug_assert!(self.input().syntax().jsx());
        let cur = self.input_mut().cur();
        debug_assert!(cur == Token::JSXText);

        let raw = self.to_utf8_ref(MaybeSubUtf8::new_from_span(self.input.cur_span()));
        let value = self.input.expect_jsx_token_value();
        let value = self.to_utf8_ref(value);

        self.input_mut().scan_jsx_token(true);
        let span = self.input().prev_span();
        self.ast.jsx_text(span, value, raw)
    }

    fn parse_jsx_ident(&mut self) -> PResult<(Span, Utf8Ref)> {
        debug_assert!(self.input().syntax().jsx());
        trace_cur!(self, parse_jsx_ident);
        let cur = self.input().cur();
        if cur == Token::JSXName || cur == Token::Ident {
            if self.input().token_flags().contains(TokenFlags::UNICODE) {
                syntax_error!(
                    self,
                    self.input().cur_span(),
                    SyntaxError::InvalidUnicodeEscape
                );
            }
            let name = cur.take_jsx_name(self.input_mut());
            self.bump();
            let span = self.input().prev_span();

            let name = self.to_utf8_ref(name);
            Ok((span, name))
        } else {
            unexpected!(self, "jsx identifier")
        }
    }

    fn parse_jsx_tag_name(&mut self) -> PResult<JSXAttrName> {
        debug_assert!(self.input().syntax().jsx());
        trace_cur!(self, parse_jsx_tag_name);
        let start = self.input().cur_pos();
        self.input_mut().scan_jsx_identifier();

        let (span, sym) = self.parse_jsx_ident()?;
        let ns = self.ast.ident_name(span, sym);

        Ok(if self.input_mut().eat(Token::Colon) {
            self.input_mut().scan_jsx_identifier();
            let (span, sym) = self.parse_jsx_ident()?;
            let name = self.ast.ident_name(span, sym);

            self.ast.jsx_attr_name_jsx_namespaced_name(
                Span::new_with_checked(start, name.span(&self.ast).hi),
                ns,
                name,
            )
        } else {
            JSXAttrName::Ident(ns)
        })
    }

    fn parse_jsx_element_name(&mut self) -> PResult<JSXElementName> {
        debug_assert!(self.input().syntax().jsx());
        trace_cur!(self, parse_jsx_element_name);
        let start = self.input().cur_pos();
        let mut node = match self.parse_jsx_tag_name()? {
            JSXAttrName::Ident(i) => {
                let name =
                    self.ast
                        .jsx_element_name_ident(i.span(&self.ast), i.sym(&self.ast), false);
                self.ast.free_node(i.node_id());
                name
            }
            JSXAttrName::JSXNamespacedName(i) => JSXElementName::JSXNamespacedName(i),
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        };
        while self.input_mut().eat(Token::Dot) {
            self.input_mut().scan_jsx_identifier();
            let (span, sym) = self.parse_jsx_ident()?;

            let obj = match node {
                JSXElementName::Ident(i) => JSXObject::Ident(i),
                JSXElementName::JSXMemberExpr(i) => JSXObject::JSXMemberExpr(i),
                _ => unreachable!("JSXNamespacedName -> JSXObject"),
            };

            let prop = self.ast.ident_name(span, sym);
            let new_node = self
                .ast
                .jsx_element_name_jsx_member_expr(self.span(start), obj, prop);

            node = new_node;
        }
        Ok(node)
    }

    fn parse_jsx_closing_element(
        &mut self,
        in_expr_context: bool,
        open_name: JSXElementName,
    ) -> PResult<JSXClosingElement> {
        let start = self.cur_pos();
        self.expect(Token::LessSlash)?;
        let tagname = self.parse_jsx_element_name()?;

        // Handle JSX closing tag followed by '=': '</tag>='
        // When lexer sees '>=' it combines into GtEq, but JSX only needs '>'
        // Use rescan_jsx_open_el_terminal_token to split >= back into >
        self.input_mut().rescan_jsx_open_el_terminal_token();
        self.expect_without_advance(Token::Gt)?;

        if in_expr_context {
            self.bump();
        } else {
            self.input_mut().scan_jsx_token(true);
        }

        if get_qualified_jsx_name(&self.ast, open_name)
            != get_qualified_jsx_name(&self.ast, tagname)
        {
            syntax_error!(
                self,
                tagname.span(&self.ast),
                SyntaxError::JSXExpectedClosingTag {
                    tag: Atom::new(get_qualified_jsx_name(&self.ast, open_name)),
                }
            )
        }

        let span = self.span(start);
        Ok(self.ast.jsx_closing_element(span, tagname))
    }

    fn parse_jsx_closing_fragment(&mut self, in_expr_context: bool) -> PResult<JSXClosingFragment> {
        let start = self.cur_pos();
        self.expect(Token::LessSlash)?;

        // Handle JSX closing fragment followed by '=': '</>=
        // When lexer sees '>=' it combines into GtEq, but JSX only needs '>'
        // Use rescan_jsx_open_el_terminal_token to split >= back into >
        self.input_mut().rescan_jsx_open_el_terminal_token();
        self.expect_without_advance(Token::Gt)?;

        if in_expr_context {
            self.bump();
        } else {
            self.input_mut().scan_jsx_token(true);
        }
        let span = self.span(start);
        Ok(self.ast.jsx_closing_fragment(span))
    }

    fn parse_jsx_children(&mut self) -> TypedSubRange<JSXElementChild> {
        self.scratch_start(|p, list| {
            loop {
                p.input_mut().rescan_jsx_token(true);
                let Ok(Some(child)) = p.parse_jsx_child(p.input().get_cur().token) else {
                    return Ok(());
                };
                list.push(p, child);
            }
        })
        .unwrap()
    }

    fn parse_jsx_child(&mut self, t: Token) -> PResult<Option<JSXElementChild>> {
        debug_assert!(self.input().syntax().jsx());

        match t {
            Token::LessSlash => Ok(None),
            Token::LBrace => Ok(Some({
                self.do_outside_of_context(
                    Context::InCondExpr.union(Context::WillExpectColonForCond),
                    |p| {
                        let start = p.cur_pos();
                        p.bump(); // bump "{"
                        let ret = if p.input().cur() == Token::DotDotDot {
                            p.bump(); // bump "..."
                            let expr = p.parse_expr_inner()?;
                            p.expect_without_advance(Token::RBrace)?;
                            p.input_mut().scan_jsx_token(true);
                            p.ast
                                .jsx_element_child_jsx_spread_child(p.span(start), expr)
                        } else {
                            let expr = if p.input().cur() == Token::RBrace {
                                JSXExpr::JSXEmptyExpr(p.parse_jsx_empty_expr())
                            } else {
                                p.parse_expr_inner().map(JSXExpr::Expr)?
                            };
                            p.expect_without_advance(Token::RBrace)?;
                            p.input_mut().scan_jsx_token(true);
                            p.ast
                                .jsx_element_child_jsx_expr_container(p.span(start), expr)
                        };
                        Ok(ret)
                    },
                )?
            })),
            Token::Lt => {
                let ele = self.parse_jsx_element(false)?;
                match ele {
                    either::Either::Left(frag) => Ok(Some(JSXElementChild::JSXFragment(frag))),
                    either::Either::Right(ele) => Ok(Some(JSXElementChild::JSXElement(ele))),
                }
            }
            Token::JSXText => Ok(Some(JSXElementChild::JSXText(self.parse_jsx_text()))),
            Token::Eof => {
                unexpected!(self, "< (jsx tag start), jsx text or {")
            }
            _ => unreachable!(),
        }
    }

    fn parse_jsx_attr_name(&mut self) -> PResult<JSXAttrName> {
        debug_assert!(self.input().syntax().jsx());
        trace_cur!(self, parse_jsx_attr_name);
        let start = self.input().cur_pos();
        self.input_mut().scan_jsx_identifier();

        let (attr_span, attr_name) = self.parse_jsx_ident()?;
        if self.input_mut().eat(Token::Colon) {
            self.input_mut().scan_jsx_identifier();
            let (span, sym) = self.parse_jsx_ident()?;

            let ns = self.ast.ident_name(attr_span, attr_name);
            let name = self.ast.ident_name(span, sym);
            Ok(self.ast.jsx_attr_name_jsx_namespaced_name(
                Span::new_with_checked(start, span.hi),
                ns,
                name,
            ))
        } else {
            Ok(JSXAttrName::Ident(
                self.ast.ident_name(attr_span, attr_name),
            ))
        }
    }

    fn parse_jsx_attr_value(&mut self) -> PResult<Option<JSXAttrValue>> {
        debug_assert!(self.input().syntax().jsx());
        trace_cur!(self, parse_jsx_attr_value);
        if self.input().is(Token::Eq) {
            self.input_mut().scan_jsx_attribute_value();
            let cur = self.input().get_cur();
            match cur.token {
                Token::Str => {
                    let value = self.parse_str_lit();
                    Ok(Some(JSXAttrValue::Str(value)))
                }
                Token::LBrace => {
                    let start = self.cur_pos();
                    let node = self.parse_jsx_expr_container()?;
                    self.jsx_expr_container_to_jsx_attr_value(start, node)
                        .map(Some)
                }
                Token::Lt => match self.parse_jsx_element(true)? {
                    either::Either::Left(frag) => Ok(Some(JSXAttrValue::JSXFragment(frag))),
                    either::Either::Right(ele) => Ok(Some(JSXAttrValue::JSXElement(ele))),
                },
                _ => {
                    let span = self.input().cur_span();
                    syntax_error!(self, span, SyntaxError::InvalidJSXValue)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn parse_jsx_attr(&mut self) -> PResult<JSXAttrOrSpread> {
        debug_assert!(self.input().syntax().jsx());
        trace_cur!(self, parse_jsx_attr);
        if self.input_mut().eat(Token::LBrace) {
            let dot3_start = self.input().cur_pos();
            self.expect(Token::DotDotDot)?;
            let dot3_token = self.span(dot3_start);
            let expr = self.parse_assignment_expr()?;
            self.expect(Token::RBrace)?;

            Ok(self.ast.jsx_attr_or_spread_spread_element(
                Span::new_with_checked(dot3_start, expr.span_hi(&self.ast)),
                dot3_token,
                expr,
            ))
        } else {
            let start = self.input().cur_pos();
            let name = self.parse_jsx_attr_name()?;
            let value = self.do_outside_of_context(
                Context::InCondExpr.union(Context::WillExpectColonForCond),
                |p| p.parse_jsx_attr_value(),
            )?;

            Ok(self
                .ast
                .jsx_attr_or_spread_jsx_attr(self.span(start), name, value))
        }
    }

    fn parse_jsx_attrs(&mut self) -> PResult<TypedSubRange<JSXAttrOrSpread>> {
        self.scratch_start(|p, attrs| {
            loop {
                trace_cur!(p, parse_jsx_opening__attrs_loop);
                p.input_mut().rescan_jsx_open_el_terminal_token();
                let cur = p.input().get_cur();
                if matches!(cur.token, Token::Gt | Token::Slash) {
                    return Ok(());
                }
                let attr = p.parse_jsx_attr()?;
                attrs.push(p, attr);
            }
        })
    }

    pub(crate) fn parse_jsx_element(
        &mut self,
        in_expr_context: bool,
    ) -> PResult<either::Either<JSXFragment, JSXElement>> {
        debug_assert!(self.input().syntax().jsx());
        trace_cur!(self, parse_jsx_element);

        let start = self.cur_pos();

        self.do_outside_of_context(Context::ShouldNotLexLtOrGtAsType, |p| {
            p.expect(Token::Lt)?;

            // Handle JSX fragment opening followed by '=': '<>='
            // When lexer sees '>=' it combines into GtEq, but JSX fragment only needs '>'
            // Use rescan_jsx_open_el_terminal_token to split >= back into >
            p.input_mut().rescan_jsx_open_el_terminal_token();

            if p.input().cur() == Token::Gt {
                // <>xxxxxx</>
                p.input_mut().scan_jsx_token(true);
                let opening = p.ast.jsx_opening_fragment(p.span(start));
                let children = p.parse_jsx_children();
                let closing = p.parse_jsx_closing_fragment(in_expr_context)?;
                let span = p.span(start);
                Ok(either::Either::Left(
                    p.ast.jsx_fragment(span, opening, children, closing),
                ))
            } else {
                let name = p.do_outside_of_context(Context::ShouldNotLexLtOrGtAsType, |p| {
                    p.parse_jsx_element_name()
                })?;
                // let type_args = if p.input().syntax().typescript() && p.input().is(Token::Lt) {
                //     p.try_parse_ts(|p| {
                //         let ret = p.parse_ts_type_args()?;
                //         p.assert_and_bump(Token::Gt);
                //         Ok(Some(ret))
                //     })
                // } else {
                //     None
                // };
                let attrs = p.parse_jsx_attrs()?;
                if p.input().cur() == Token::Gt {
                    // <xxxxx>xxxxx</xxxxx>
                    p.input_mut().scan_jsx_token(true);
                    let span = Span::new_with_checked(start, p.input.get_cur().span.lo);
                    let opening = p.ast.jsx_opening_element(span, name, attrs, false);
                    let children = p.parse_jsx_children();
                    let closing =
                        p.parse_jsx_closing_element(in_expr_context, opening.name(&p.ast))?;
                    let span = if in_expr_context {
                        Span::new_with_checked(start, p.last_pos())
                    } else {
                        Span::new_with_checked(start, p.cur_pos())
                    };
                    Ok(either::Either::Right(p.ast.jsx_element(
                        span,
                        opening,
                        children,
                        Some(closing),
                    )))
                } else {
                    // <xxxxx/>
                    p.expect(Token::Slash)?;

                    // Handle JSX self-closing tag followed by '=': '<tag/>='
                    // When lexer sees '>=' it combines into GtEq, but JSX only needs '>'
                    // Use rescan_jsx_open_el_terminal_token to split >= back into >
                    p.input_mut().rescan_jsx_open_el_terminal_token();
                    p.expect_without_advance(Token::Gt)?;

                    if in_expr_context {
                        p.bump();
                    } else {
                        p.input_mut().scan_jsx_token(true);
                    }
                    let span = if in_expr_context {
                        p.span(start)
                    } else {
                        Span::new_with_checked(start, p.cur_pos())
                    };

                    let opening = p.ast.jsx_opening_element(span, name, attrs, true);
                    Ok(either::Either::Right(p.ast.jsx_element(
                        span,
                        opening,
                        TypedSubRange::empty(),
                        None,
                    )))
                }
            }
        })
    }
}

fn get_qualified_jsx_name(ast: &'_ Ast, name: JSXElementName) -> Cow<'_, str> {
    fn get_qualified_obj_name(ast: &'_ Ast, obj: JSXObject) -> Cow<'_, str> {
        match obj {
            JSXObject::Ident(i) => Cow::Borrowed(ast.get_utf8(i.sym(ast))),
            JSXObject::JSXMemberExpr(member) => Cow::Owned(format!(
                "{}.{}",
                get_qualified_obj_name(ast, member.obj(ast)),
                ast.get_utf8(member.prop(ast).sym(ast))
            )),
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        }
    }
    match name {
        JSXElementName::Ident(i) => Cow::Borrowed(ast.get_utf8(i.sym(ast))),
        JSXElementName::JSXNamespacedName(item) => Cow::Owned(format!(
            "{}:{}",
            ast.get_utf8(item.ns(ast).sym(ast)),
            ast.get_utf8(item.name(ast).sym(ast))
        )),
        JSXElementName::JSXMemberExpr(item) => Cow::Owned(format!(
            "{}.{}",
            get_qualified_obj_name(ast, item.obj(ast)),
            ast.get_utf8(item.prop(ast).sym(ast))
        )),
        #[cfg(swc_ast_unknown)]
        _ => unreachable!(),
    }
}
