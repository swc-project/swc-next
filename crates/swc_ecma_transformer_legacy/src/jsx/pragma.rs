use swc_next_ecma_ast::*;

use crate::resolver::Resolver;

use super::{Jsx, JsxRuntime, build::member};

/// SWC comment validation; dotted paths are the agreed option restriction: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L475>.
pub(super) fn valid_path(value: &str) -> bool {
    value.split('.').all(|part| is_identifier(part.as_bytes()))
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L143>.
pub(super) fn expression(ast: &mut Ast<'_>, resolver: &mut Resolver, value: &str) -> Expr {
    let mut parts = value.split('.');
    let root = parts.next().unwrap();
    let mut result = if root == "this" {
        ast.expr_this_expression(DUMMY_SP)
    } else {
        resolver.marked_reference(ast, root, false)
    };
    for part in parts {
        result = member(ast, result, part, DUMMY_SP);
    }
    result
}

impl Jsx<'_> {
    /// SWC directive parsing: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L320>.
    /// SWC directive application: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1143>.
    pub(super) fn parse_directives(&mut self, ast: &Ast<'_>, offset: u32) -> bool {
        let Some(comments) = self.comments.get(&offset) else {
            return false;
        };
        let mut runtime = None;
        let mut import_source = None;
        let mut pragma = None;
        let mut pragma_frag = None;
        for comment in comments {
            if comment.kind != CommentKind::Block || comment.position != CommentPosition::Leading {
                continue;
            }
            let text = ast.get_utf8(comment.value(ast.source().as_bytes()));
            for line in text.lines() {
                let line = line.trim();
                let line = line.strip_prefix('*').map_or(line, str::trim);
                if !line.starts_with("@jsx") {
                    continue;
                }
                let mut words = line.split_whitespace();
                while let Some(directive) = words.next() {
                    let value = words.next();
                    match (directive, value) {
                        ("@jsxRuntime", Some("classic")) => runtime = Some(JsxRuntime::Classic),
                        ("@jsxRuntime", Some("automatic")) => runtime = Some(JsxRuntime::Automatic),
                        ("@jsxRuntime", Some(_)) => self.diagnostics.push(Diagnostic::error(
                            "Runtime must be either `classic` or `automatic`.",
                            comment.span,
                        )),
                        ("@jsxImportSource", Some(value)) => {
                            runtime = Some(JsxRuntime::Automatic);
                            import_source = Some(value.to_owned());
                        }
                        ("@jsx", Some(value)) if valid_path(value) => {
                            pragma = Some((value.to_owned(), comment.span))
                        }
                        ("@jsxFrag", Some(value)) if valid_path(value) => {
                            pragma_frag = Some((value.to_owned(), comment.span))
                        }
                        _ => {}
                    }
                }
            }
        }
        let found = runtime.is_some()
            || import_source.is_some()
            || pragma.is_some()
            || pragma_frag.is_some();
        if let Some(runtime) = runtime {
            self.runtime = runtime;
        }
        if let Some(import_source) = import_source {
            self.import_source = import_source;
        }
        if let Some((pragma, span)) = pragma {
            if self.runtime == JsxRuntime::Automatic {
                self.diagnostics.push(Diagnostic::error(
                    "pragma cannot be set when runtime is automatic",
                    span,
                ));
            }
            self.pragma = pragma;
        }
        if let Some((pragma, span)) = pragma_frag {
            if self.runtime == JsxRuntime::Automatic {
                self.diagnostics.push(Diagnostic::error(
                    "pragmaFrag cannot be set when runtime is automatic",
                    span,
                ));
            }
            self.pragma_frag = pragma;
        }
        found
    }
}
