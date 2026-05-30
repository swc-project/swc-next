use std::borrow::Cow;

use swc_experimental_allocator::vec::Vec;
use swc_experimental_allocator::{atom::Atom, boxed::Box};
use swc_experimental_ecma_ast::*;

use crate::{Context, PResult, Parser, error::SyntaxError, input::Tokens, lexer::Token};

impl<'a, I: Tokens<'a>> Parser<'a, I> {
    pub fn parse_module_item(&mut self) -> PResult<ModuleItem<'a>> {
        self.do_inside_of_context(Context::TopLevel, |p| {
            p.parse_stmt_like(true, handle_import_export)
        })
    }

    pub(crate) fn parse_module_item_block_body(
        &mut self,
        allow_directives: bool,
        end: Option<Token>,
    ) -> PResult<Vec<'a, ModuleItem<'a>>> {
        self.parse_block_body(allow_directives, end, handle_import_export)
    }

    /// Parses `from 'foo.js' with {};` or `from 'foo.js' assert {};`
    fn parse_from_clause_and_semi(
        &mut self,
    ) -> PResult<(Box<'a, Str<'a>>, Option<Box<'a, ObjectLit<'a>>>)> {
        expect!(self, Token::From);

        let cur = self.input().cur();
        let src = if cur == Token::Str {
            let src = self.parse_str_lit();
            self.boxed(src)
        } else {
            unexpected!(self, "a string literal")
        };
        let with = if self.input().syntax().import_attributes()
            && !self.input().had_line_break_before_cur()
            && (self.input_mut().eat(Token::Assert) || self.input_mut().eat(Token::With))
        {
            match self.parse_object_expr()? {
                Expr::Object(v) => Some(v),
                _ => unreachable!(),
            }
        } else {
            None
        };
        self.expect_general_semi()?;
        Ok((src, with))
    }

    fn parse_named_export_specifier(
        &mut self,
        type_only: bool,
    ) -> PResult<ExportNamedSpecifier<'a>> {
        let start = self.cur_pos();

        let mut is_type_only = false;

        let orig_token = self.input().cur();
        let orig = match self.parse_module_export_name()? {
            ModuleExportName::Ident(orig_ident) => {
                // Handle:
                // `export { type xx }`
                // `export { type xx as yy }`
                // `export { type as }`
                // `export { type as as }`
                // `export { type as as as }`
                if self.syntax().typescript()
                    && orig_token == Token::Type
                    && self.input().cur().is_word()
                {
                    let possibly_orig_token = self.input().cur();
                    let possibly_orig = self
                        .parse_ident_name()
                        .map(|(span, sym)| self.ast.ident(span, sym, false))?;
                    if possibly_orig_token == Token::As {
                        // `export { type as }`
                        if !self.input().cur().is_word() {
                            if type_only {
                                self.emit_err(orig_ident.span(), SyntaxError::TS2207);
                            }

                            return Ok(self.ast.export_named_specifier(
                                self.span(start),
                                ModuleExportName::Ident(self.boxed(possibly_orig)),
                                None,
                                true,
                            ));
                        }

                        let maybe_as_token = self.input().cur();
                        let maybe_as = self
                            .parse_ident_name()
                            .map(|(span, sym)| self.ast.ident(span, sym, false))?;
                        if maybe_as_token == Token::As {
                            if self.input().cur().is_word() {
                                // `export { type as as as }`
                                // `export { type as as foo }`
                                let exported = self
                                    .parse_ident_name()
                                    .map(|(span, sym)| self.ast.ident(span, sym, false))?;

                                if type_only {
                                    self.emit_err(orig_ident.span(), SyntaxError::TS2207);
                                }

                                debug_assert!(start <= orig_ident.span_hi());
                                return Ok(self.ast.export_named_specifier(
                                    Span::new(start, orig_ident.span_hi()),
                                    ModuleExportName::Ident(self.boxed(possibly_orig)),
                                    Some(ModuleExportName::Ident(self.boxed(exported))),
                                    true,
                                ));
                            } else {
                                // `export { type as as }`
                                return Ok(self.ast.export_named_specifier(
                                    Span::new(start, orig_ident.span_hi()),
                                    ModuleExportName::Ident(orig_ident),
                                    Some(ModuleExportName::Ident(self.boxed(maybe_as))),
                                    false,
                                ));
                            }
                        } else {
                            // `export { type as xxx }`
                            return Ok(self.ast.export_named_specifier(
                                Span::new(start, orig_ident.span_hi()),
                                ModuleExportName::Ident(orig_ident),
                                Some(ModuleExportName::Ident(self.boxed(maybe_as))),
                                false,
                            ));
                        }
                    } else {
                        // `export { type xx }`
                        // `export { type xx as yy }`
                        if type_only {
                            self.emit_err(orig_ident.span(), SyntaxError::TS2207);
                        }

                        is_type_only = true;
                        ModuleExportName::Ident(self.boxed(possibly_orig))
                    }
                } else {
                    ModuleExportName::Ident(orig_ident)
                }
            }
            module_export_name => module_export_name,
        };

        let exported = if self.input_mut().eat(Token::As) {
            Some(self.parse_module_export_name()?)
        } else {
            None
        };

        Ok(self
            .ast
            .export_named_specifier(self.span(start), orig, exported, is_type_only))
    }

    fn parse_imported_binding(&mut self) -> PResult<Ident<'a>> {
        let ident = self
            .do_outside_of_context(Context::InAsync.union(Context::InGenerator), |p| {
                p.parse_binding_ident(false)
            })?;
        Ok(ident)
    }

    fn parse_imported_default_binding(&mut self) -> PResult<Ident<'a>> {
        self.parse_imported_binding()
    }

    /// Parse `foo`, `foo2 as bar` in `import { foo, foo2 as bar }`
    fn parse_import_specifier(&mut self, type_only: bool) -> PResult<ImportSpecifier<'a>> {
        let start = self.cur_pos();
        let orig_token = self.input().cur();
        match self.parse_module_export_name()? {
            ModuleExportName::Ident(mut orig_name) => {
                let mut is_type_only = false;
                // Handle:
                // `import { type xx } from 'mod'`
                // `import { type xx as yy } from 'mod'`
                // `import { type as } from 'mod'`
                // `import { type as as } from 'mod'`
                // `import { type as as as } from 'mod'`
                if self.syntax().typescript()
                    && orig_token == Token::Type
                    && self.input().cur().is_word()
                {
                    let possibly_orig_token = self.input().cur();
                    let possibly_orig_name = self
                        .parse_ident_name()
                        .map(|(span, sym)| self.ast.ident(span, sym, false))?;

                    let possibly_orig_name_str = possibly_orig_name.sym.as_str();
                    if possibly_orig_token == Token::As {
                        // `import { type as } from 'mod'`
                        if !self.input().cur().is_word() {
                            if self.ctx().is_reserved_word(possibly_orig_name_str) {
                                syntax_error!(
                                    self,
                                    possibly_orig_name.span(),
                                    SyntaxError::ReservedWordInImport
                                )
                            }

                            if type_only {
                                self.emit_err(orig_name.span(), SyntaxError::TS2206);
                            }

                            return Ok(self.ast.import_specifier_import_named_specifier(
                                self.span(start),
                                self.boxed(possibly_orig_name),
                                None,
                                true,
                            ));
                        }

                        let maybe_as_token = self.input().cur();
                        let maybe_as: Ident = self.parse_binding_ident(false)?;
                        if maybe_as_token == Token::As {
                            if self.input().cur().is_word() {
                                // `import { type as as as } from 'mod'`
                                // `import { type as as foo } from 'mod'`
                                let local: Ident = self.parse_binding_ident(false)?;

                                if type_only {
                                    self.emit_err(orig_name.span(), SyntaxError::TS2206);
                                }

                                return Ok(self.ast.import_specifier_import_named_specifier(
                                    Span::new(start, orig_name.span_hi()),
                                    self.boxed(local),
                                    Some(ModuleExportName::Ident(self.boxed(possibly_orig_name))),
                                    true,
                                ));
                            } else {
                                // `import { type as as } from 'mod'`
                                return Ok(self.ast.import_specifier_import_named_specifier(
                                    Span::new(start, maybe_as.span_hi()),
                                    self.boxed(maybe_as),
                                    Some(ModuleExportName::Ident(orig_name)),
                                    false,
                                ));
                            }
                        } else {
                            // `import { type as xxx } from 'mod'`
                            return Ok(self.ast.import_specifier_import_named_specifier(
                                Span::new(start, orig_name.span_hi()),
                                self.boxed(maybe_as),
                                Some(ModuleExportName::Ident(orig_name)),
                                false,
                            ));
                        }
                    } else {
                        // `import { type xx } from 'mod'`
                        // `import { type xx as yy } from 'mod'`
                        if type_only {
                            self.emit_err(orig_name.span(), SyntaxError::TS2206);
                        }

                        orig_name = self.boxed(possibly_orig_name);
                        is_type_only = true;
                    }
                }

                if self.input_mut().eat(Token::As) {
                    let local: Ident = self.parse_binding_ident(false)?;
                    return Ok(self.ast.import_specifier_import_named_specifier(
                        Span::new(start, local.span_hi()),
                        self.boxed(local),
                        Some(ModuleExportName::Ident(orig_name)),
                        is_type_only,
                    ));
                }

                // Handle difference between
                //
                // 'ImportedBinding'
                // 'IdentifierName' as 'ImportedBinding'
                if self.ctx().is_reserved_word(orig_name.sym.as_str()) {
                    syntax_error!(self, orig_name.span(), SyntaxError::ReservedWordInImport)
                }

                let local = orig_name;
                Ok(self.ast.import_specifier_import_named_specifier(
                    self.span(start),
                    local,
                    None,
                    is_type_only,
                ))
            }
            ModuleExportName::Str(orig_str) => {
                if self.input_mut().eat(Token::As) {
                    let local: Ident = self.parse_binding_ident(false)?;
                    Ok(self.ast.import_specifier_import_named_specifier(
                        Span::new(start, local.span_hi()),
                        self.boxed(local),
                        Some(ModuleExportName::Str(orig_str)),
                        false,
                    ))
                } else {
                    syntax_error!(
                        self,
                        orig_str.span(),
                        SyntaxError::ImportBindingIsString(orig_str.value.to_string_lossy().into())
                    )
                }
            }
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        }
    }

    pub(crate) fn parse_export(
        &mut self,
        mut decorators: Vec<'a, Decorator<'a>>,
    ) -> PResult<ModuleDecl<'a>> {
        if !self.ctx().contains(Context::Module) && self.ctx().contains(Context::TopLevel) {
            // Switch to module mode
            let ctx = self.ctx() | Context::Module | Context::Strict;
            self.set_ctx(ctx);
        }

        let start = self.cur_pos();
        self.assert_and_bump(Token::Export);

        let cur = self.input().cur();
        if cur == Token::Eof {
            return Err(self.eof_error());
        }

        // let after_export_start = self.cur_pos();

        // "export declare" is equivalent to just "export".
        // let declare = self.input().syntax().typescript() && self.input_mut().eat(Token::Declare);

        // if declare {
        //     // TODO: Remove
        //     if let Some(decl) = self.try_parse_ts_declare(after_export_start, decorators.clone())? {
        //         return Ok(ExportDecl {
        //             span: self.span(start),
        //             decl,
        //         }
        //         .into());
        //     }
        // }

        // if self.input().syntax().typescript() {
        //     let cur = self.input().cur();
        //     if cur.is_word() {
        //         let sym = cur.take_word(self.input()).unwrap();
        //         // TODO: remove clone
        //         if let Some(decl) = self.try_parse_ts_export_decl(decorators.clone(), sym) {
        //             return Ok(ExportDecl {
        //                 span: self.span(start),
        //                 decl,
        //             }
        //             .into());
        //         }
        //     }

        //     if self.input_mut().eat(Token::Import) {
        //         let is_type_only =
        //             self.input().is(Token::Type) && peek!(self).is_some_and(|p| p.is_word());

        //         if is_type_only {
        //             self.assert_and_bump(Token::Type);
        //         }

        //         let id = self.parse_ident_name()?;

        //         // export import A = B
        //         return self
        //             .parse_ts_import_equals_decl(
        //                 start,
        //                 id.into(),
        //                 /* is_export */ true,
        //                 is_type_only,
        //             )
        //             .map(From::from);
        //     }

        //     if self.input_mut().eat(Token::Eq) {
        //         // `export = x;`
        //         let expr = self.parse_expr()?;
        //         self.expect_general_semi()?;
        //         return Ok(TsExportAssignment {
        //             span: self.span(start),
        //             expr,
        //         }
        //         .into());
        //     }

        //     if self.input_mut().eat(Token::As) {
        //         // `export as namespace A;`
        //         // See `parseNamespaceExportDeclaration` in TypeScript's own parser
        //         expect!(self, Token::Namespace);
        //         let id = self.parse_ident(false, false)?;
        //         self.expect_general_semi()?;
        //         return Ok(TsNamespaceExportDecl {
        //             span: self.span(start),
        //             id,
        //         }
        //         .into());
        //     }
        // }

        let ns_export_specifier_start = self.cur_pos();

        let type_only = self.input().syntax().typescript() && self.input_mut().eat(Token::Type);

        // Some("default") if default is exported from 'src'
        let mut export_default = None;

        if !type_only && self.input_mut().eat(Token::Default) {
            if self.input().is(Token::At) {
                let start = self.cur_pos();
                let after_decorators = self.parse_decorators(false)?;

                if !decorators.is_empty() {
                    syntax_error!(self, self.span(start), SyntaxError::TS8038);
                }

                decorators = after_decorators;
            }

            if self.input().syntax().typescript() {
                if self.input().is(Token::Abstract)
                    && peek!(self).is_some_and(|cur| cur == Token::Class)
                    && !self.input_mut().has_linebreak_between_cur_and_peeked()
                {
                    let class_start = self.cur_pos();
                    self.assert_and_bump(Token::Abstract);
                    let cur = self.input().cur();
                    if cur == Token::Error {
                        let err = self.input_mut().expect_error_token_and_bump();
                        return Err(err);
                    }

                    return self
                        .parse_default_class(start, class_start, decorators, true)
                        .map(|decl| ModuleDecl::ExportDefaultDecl(self.boxed(decl)));
                }
                if self.input().is(Token::Abstract)
                    && peek!(self).is_some_and(|cur| cur == Token::Interface)
                {
                    self.emit_err(self.input().cur_span(), SyntaxError::TS1242);
                    self.assert_and_bump(Token::Abstract);
                }

                // if self.input().is(Token::Interface) {
                //     let interface_start = self.cur_pos();
                //     self.assert_and_bump(Token::Interface);
                //     let decl = self
                //         .parse_ts_interface_decl(interface_start)
                //         .map(DefaultDecl::from)?;
                //     return Ok(ExportDefaultDecl {
                //         span: self.span(start),
                //         decl,
                //     }
                //     .into());
                // }
            }

            if self.input().is(Token::Class) {
                let class_start = self.cur_pos();
                let decl = self.parse_default_class(start, class_start, decorators, false)?;
                return Ok(ModuleDecl::ExportDefaultDecl(self.boxed(decl)));
            } else if self.input().is(Token::Async)
                && peek!(self).is_some_and(|cur| cur == Token::Function)
                && !self.input_mut().has_linebreak_between_cur_and_peeked()
            {
                let decl = self.parse_default_async_fn(start, decorators)?;
                return Ok(ModuleDecl::ExportDefaultDecl(self.boxed(decl)));
            } else if self.input().is(Token::Function) {
                let decl = self.parse_default_fn(start, decorators)?;
                return Ok(ModuleDecl::ExportDefaultDecl(self.boxed(decl)));
            } else if self.input().syntax().export_default_from()
                && ((self.input().is(Token::From)
                    && peek!(self).is_some_and(|peek| peek == Token::Str))
                    || (self.input().is(Token::Comma)
                        && (peek!(self)
                            .is_some_and(|peek| matches!(peek, Token::Asterisk | Token::LBrace)))))
            {
                let sym = Atom::new_const("default");
                export_default = Some(self.ast.ident(self.input().prev_span(), sym, false));
            } else {
                let expr = self.allow_in_expr(Self::parse_assignment_expr)?;
                self.expect_general_semi()?;
                return Ok(self
                    .ast
                    .module_decl_export_default_expr(self.span(start), expr));
            }
        }

        if self.input().is(Token::At) {
            let start = self.cur_pos();
            let after_decorators = self.parse_decorators(false)?;

            if !decorators.is_empty() {
                syntax_error!(self, self.span(start), SyntaxError::TS8038);
            }

            decorators = after_decorators;
        }

        let decl = if !type_only && self.input().is(Token::Class) {
            let class_start = self.cur_pos();
            self.parse_class_decl(start, class_start, decorators, false)?
        } else if !type_only
            && self.input().is(Token::Async)
            && peek!(self).is_some_and(|cur| cur == Token::Function)
            && !self.input_mut().has_linebreak_between_cur_and_peeked()
        {
            self.parse_async_fn_decl(decorators)?
        } else if !type_only && self.input().is(Token::Function) {
            self.parse_fn_decl(decorators)?
        // } else if !type_only
        //     && self.input().syntax().typescript()
        //     && self.input().is(Token::Const)
        //     && peek!(self).is_some_and(|cur| cur == Token::Enum)
        // {
        //     let enum_start = self.cur_pos();
        //     self.assert_and_bump(Token::Const);
        //     self.assert_and_bump(Token::Enum);
        //     return self
        //         .parse_ts_enum_decl(enum_start, /* is_const */ true)
        //         .map(Decl::from)
        //         .map(|decl| {
        //             ExportDecl {
        //                 span: self.span(start),
        //                 decl,
        //             }
        //             .into()
        //         });
        } else if !type_only
            && (self.input().is(Token::Var)
                || self.input().is(Token::Const)
                || (self.input().is(Token::Let))
                    && peek!(self)
                        .map(|t| t.follows_keyword_let())
                        .unwrap_or(false))
        {
            let decl = self.parse_var_stmt(false)?;
            Decl::Var(self.boxed(decl))
        } else {
            // ```javascript
            // export foo, * as bar, { baz } from "mod"; // *
            // export      * as bar, { baz } from "mod"; // *
            // export foo,           { baz } from "mod"; // *
            // export foo, * as bar          from "mod"; // *
            // export foo                    from "mod"; // *
            // export      * as bar          from "mod"; //
            // export                { baz } from "mod"; //
            // export                { baz }           ; //
            // export      *                 from "mod"; //
            // ```

            // export default
            // export foo
            let default = match export_default {
                Some(default) => Some(default),
                None => {
                    if self.input().syntax().export_default_from() && self.input().cur().is_word() {
                        Some(self.parse_ident(false, false)?)
                    } else {
                        None
                    }
                }
            };

            if default.is_none()
                && self.input().is(Token::Asterisk)
                && !peek!(self).is_some_and(|cur| cur == Token::As)
            {
                self.assert_and_bump(Token::Asterisk);

                // improve error message for `export * from foo`
                let (src, with) = self.parse_from_clause_and_semi()?;
                return Ok(self
                    .ast
                    .module_decl_export_all(self.span(start), src, type_only, with));
            }

            let mut has_default = false;
            let mut has_ns = false;

            let specifiers = self.collect_vec(|p, specifiers| {
                if let Some(default) = default {
                    has_default = true;

                    let specifier = p
                        .ast
                        .export_specifier_export_default_specifier(p.boxed(default));
                    specifiers.push(specifier);
                }

                // export foo, * as bar
                //           ^
                if has_default
                    && p.input().is(Token::Comma)
                    && peek!(p).is_some_and(|cur| cur == Token::Asterisk)
                {
                    p.assert_and_bump(Token::Comma);

                    has_ns = true;
                }
                // export     * as bar
                //            ^
                else if !has_default && p.input().is(Token::Asterisk) {
                    has_ns = true;
                }

                if has_ns {
                    p.assert_and_bump(Token::Asterisk);
                    expect!(p, Token::As);
                    let name = p.parse_module_export_name()?;
                    let specifier = p.ast.export_specifier_export_namespace_specifier(
                        p.span(ns_export_specifier_start),
                        name,
                    );
                    specifiers.push(specifier);
                }

                if has_default || has_ns {
                    if p.input().is(Token::From) {
                        return Ok(());
                    } else if !p.input().syntax().export_default_from() {
                        // emit error
                        expect!(p, Token::From);
                    }

                    expect!(p, Token::Comma);
                }

                expect!(p, Token::LBrace);

                while !p.input().is(Token::RBrace) {
                    let specifier = p.parse_named_export_specifier(type_only)?;
                    specifiers.push(ExportSpecifier::Named(p.boxed(specifier)));

                    if p.input().is(Token::RBrace) {
                        break;
                    } else {
                        expect!(p, Token::Comma);
                    }
                }
                expect!(p, Token::RBrace);

                Ok(())
            })?;

            if has_default || has_ns {
                let (src, with) = self.parse_from_clause_and_semi()?;
                return Ok(self.ast.module_decl_named_export(
                    self.span(start),
                    specifiers,
                    Some(src),
                    type_only,
                    with,
                ));
            }

            let opt = if self.input().is(Token::From) {
                Some(self.parse_from_clause_and_semi()?)
            } else {
                for s in specifiers.iter() {
                    match s {
                        ExportSpecifier::Default(default) => {
                            self.emit_err(
                                default.exported.span(),
                                SyntaxError::ExportExpectFrom(default.exported.sym.to_string()),
                            );
                        }
                        ExportSpecifier::Namespace(namespace) => {
                            let export_name = match &namespace.name {
                                ModuleExportName::Ident(i) => Cow::Borrowed(i.sym.as_str()),
                                ModuleExportName::Str(s) => s.value.to_string_lossy(),
                                #[cfg(swc_ast_unknown)]
                                _ => unreachable!(),
                            };
                            self.emit_err(
                                namespace.span(),
                                SyntaxError::ExportExpectFrom(export_name.into_owned()),
                            );
                        }
                        ExportSpecifier::Named(named) => match &named.orig {
                            ModuleExportName::Ident(id) if id.is_reserved() => {
                                self.emit_err(
                                    id.span(),
                                    SyntaxError::ExportExpectFrom(id.sym.to_string()),
                                );
                            }
                            ModuleExportName::Str(s) => {
                                self.emit_err(s.span(), SyntaxError::ExportBindingIsString);
                            }
                            _ => {}
                        },
                        #[cfg(swc_ast_unknown)]
                        _ => unreachable!(),
                    }
                }

                self.eat_general_semi();

                None
            };
            let (src, with) = match opt {
                Some(v) => (Some(v.0), v.1),
                None => (None, None),
            };

            return Ok(self.ast.module_decl_named_export(
                self.span(start),
                specifiers,
                src,
                type_only,
                with,
            ));
        };

        Ok(self.ast.module_decl_export_decl(self.span(start), decl))
    }

    pub(crate) fn parse_import(&mut self) -> PResult<ModuleItem<'a>> {
        let start = self.cur_pos();

        if peek!(self).is_some_and(|cur| cur == Token::Dot) {
            let expr = self.parse_expr_inner()?;

            self.eat_general_semi();

            return Ok(self.ast.module_item_stmt_expr_stmt(self.span(start), expr));
        }

        if peek!(self).is_some_and(|cur| cur == Token::LParen) {
            let expr = self.parse_expr_inner()?;

            self.eat_general_semi();

            return Ok(self.ast.module_item_stmt_expr_stmt(self.span(start), expr));
        }

        // It's now import statement

        if !self.ctx().contains(Context::Module) {
            // Switch to module mode
            let ctx = self.ctx() | Context::Module | Context::Strict;
            self.set_ctx(ctx);
        }

        expect!(self, Token::Import);

        // Handle import 'mod.js'
        if self.input().cur() == Token::Str {
            let src = self.parse_str_lit();
            let with = if self.input().syntax().import_attributes()
                && !self.input().had_line_break_before_cur()
                && (self.input_mut().eat(Token::Assert) || self.input_mut().eat(Token::With))
            {
                match self.parse_object_expr()? {
                    Expr::Object(v) => Some(v),
                    _ => unreachable!(),
                }
            } else {
                None
            };
            self.eat_general_semi();

            return Ok(self.ast.module_item_module_decl_import_decl(
                self.span(start),
                self.vec(),
                self.boxed(src),
                false,
                with,
                ImportPhase::default(),
            ));
        }

        let mut type_only = false;
        let mut phase = ImportPhase::Evaluation;
        let specifiers = self.collect_vec(|p, specifiers| {
            'import_maybe_ident: {
                if p.is_ident_ref() {
                    let local_token = p.input().cur();
                    let mut local = p.parse_imported_default_binding()?;
                    let local_sym = local.sym.as_str();
                    let is_source = local_sym == "source";
                    let is_defer = local_sym == "defer";

                    if p.input().syntax().typescript() && local_token == Token::Type {
                        let cur = p.input().cur();
                        if cur == Token::LBrace || cur == Token::Asterisk {
                            type_only = true;
                            break 'import_maybe_ident;
                        }

                        if p.is_ident_ref() {
                            if !p.input().is(Token::From)
                                || peek!(p).is_some_and(|cur| cur == Token::From)
                            {
                                type_only = true;
                                local = p.parse_imported_default_binding()?;
                            } else if peek!(p).is_some_and(|cur| cur == Token::Eq) {
                                type_only = true;
                                local = p
                                    .parse_ident_name()
                                    .map(|(span, sym)| p.ast.ident(span, sym, false))?;
                            }
                        }
                    }

                    // if p.input().syntax().typescript() && p.input().is(Token::Eq) {
                    //     return p
                    //         .parse_ts_import_equals_decl(start, local, false, type_only)
                    //         .map(ModuleDecl::from)
                    //         .map(ModuleItem::from);
                    // }

                    if is_source || is_defer {
                        let new_phase = if is_source {
                            ImportPhase::Source
                        } else {
                            ImportPhase::Defer
                        };

                        let cur = p.input().cur();
                        if cur == Token::LBrace || cur == Token::Asterisk {
                            phase = new_phase;
                            break 'import_maybe_ident;
                        }

                        if p.is_ident_ref() && !p.input().is(Token::From)
                            || peek!(p).is_some_and(|cur| cur == Token::From)
                        {
                            // For defer phase, we expect only namespace imports, so break here
                            // and let the subsequent code handle validation
                            if new_phase == ImportPhase::Defer {
                                break 'import_maybe_ident;
                            }
                            phase = new_phase;

                            local = p.parse_imported_default_binding()?;
                        }
                    }

                    //TODO: Better error reporting
                    if !p.input().is(Token::From) {
                        expect!(p, Token::Comma);
                    }

                    let specifier = p
                        .ast
                        .import_specifier_import_default_specifier(local.span(), p.boxed(local));
                    specifiers.push(specifier);
                }
            }

            {
                let import_spec_start = p.cur_pos();
                // Namespace imports are not allowed in source phase.
                if phase != ImportPhase::Source && p.input_mut().eat(Token::Asterisk) {
                    expect!(p, Token::As);
                    let local = p.parse_imported_binding()?;
                    let specifier = p.ast.import_specifier_import_star_as_specifier(
                        p.span(import_spec_start),
                        p.boxed(local),
                    );
                    specifiers.push(specifier);
                    // Named imports are only allowed in evaluation phase.
                } else if phase == ImportPhase::Evaluation && p.input_mut().eat(Token::LBrace) {
                    while !p.input().is(Token::RBrace) {
                        let specifier = p.parse_import_specifier(type_only)?;
                        specifiers.push(specifier);

                        if p.input().is(Token::RBrace) {
                            break;
                        } else {
                            expect!(p, Token::Comma);
                        }
                    }
                    expect!(p, Token::RBrace);
                }
            }

            Ok(())
        })?;

        let src = {
            expect!(self, Token::From);
            if self.input().cur() == Token::Str {
                self.parse_str_lit()
            } else {
                unexpected!(self, "a string literal")
            }
        };

        let with = if self.input().syntax().import_attributes()
            && !self.input().had_line_break_before_cur()
            && (self.input_mut().eat(Token::Assert) || self.input_mut().eat(Token::With))
        {
            match self.parse_object_expr()? {
                Expr::Object(v) => Some(v),
                _ => unreachable!(),
            }
        } else {
            None
        };

        self.expect_general_semi()?;

        Ok(self.ast.module_item_module_decl_import_decl(
            self.span(start),
            specifiers,
            self.boxed(src),
            type_only,
            with,
            phase,
        ))
    }
}

fn handle_import_export<'a, I: Tokens<'a>>(
    p: &mut Parser<'a, I>,
    decorators: Vec<'a, Decorator<'a>>,
) -> PResult<ModuleItem<'a>> {
    if !p
        .ctx()
        .intersects(Context::TopLevel.union(Context::TsModuleBlock))
    {
        syntax_error!(p, SyntaxError::NonTopLevelImportExport);
    }

    let decl = if p.input().is(Token::Import) {
        p.parse_import()?
    } else if p.input().is(Token::Export) {
        let decl = p.parse_export(decorators)?;
        ModuleItem::ModuleDecl(p.boxed(decl))
    } else {
        unreachable!(
            "handle_import_export should not be called if current token isn't import nor export"
        )
    };

    Ok(decl)
}
