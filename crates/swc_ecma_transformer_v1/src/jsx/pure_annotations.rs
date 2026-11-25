use rustc_hash::{FxHashMap, FxHashSet};

use swc_next_ecma_ast::*;

use crate::common::resolver::context::Resolver;

/// SWC's annotations for calls to known pure React and React DOM APIs.
/// <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/pure_annotations.rs>.
#[derive(Default)]
pub(super) struct PureAnnotations<'ast> {
    imports: FxHashMap<SymbolId, (bool, &'ast str)>,
}

impl<'ast> PureAnnotations<'ast> {
    pub(super) fn collect_imports(
        &mut self,
        ast: &Ast<'ast>,
        resolver: &Resolver,
        program: Program,
    ) {
        for slot in program.body(ast).iter() {
            let Some(import) = ast.get_node_in_sub_range(slot).as_import_declaration(ast) else {
                continue;
            };
            let Some(src @ ("react" | "react-dom")) =
                ast.get_wtf8(import.source(ast).value(ast)).as_str()
            else {
                continue;
            };
            for slot in import.specifiers(ast).iter() {
                let specifier = ast.get_node_in_sub_range(slot);
                let (local, name) = match ast.import_declaration_specifier_data(specifier) {
                    ImportDeclarationSpecifierData::ImportSpecifier(named) => {
                        let name = match ast.module_export_name_data(named.imported(ast)) {
                            ModuleExportNameData::IdentifierName(name) => {
                                ast.get_utf8(name.name(ast))
                            }
                            ModuleExportNameData::StringLiteral(name) => {
                                let Some(name) = ast.get_wtf8(name.value(ast)).as_str() else {
                                    continue;
                                };
                                name
                            }
                        };
                        (named.local(ast), name)
                    }
                    ImportDeclarationSpecifierData::ImportDefaultSpecifier(default) => {
                        (default.local(ast), "default")
                    }
                    ImportDeclarationSpecifierData::ImportNamespaceSpecifier(namespace) => {
                        (namespace.local(ast), "*")
                    }
                };
                let id = resolver.symbol_of(local.node_id());
                self.imports.insert(id, (src == "react-dom", name));
            }
        }
    }

    pub(super) fn annotate(
        &self,
        ast: &mut Ast<'ast>,
        resolver: &Resolver,
        call: CallExpression,
        pure_positions: &mut FxHashSet<u32>,
    ) {
        if !self.imports.is_empty() && self.is_pure_call(ast, resolver, call) {
            call.set_pure(ast, true);
            pure_positions.insert(call.span(ast).start);
        }
    }

    fn import(
        &self,
        ast: &Ast<'ast>,
        resolver: &Resolver,
        expression: Expr,
    ) -> Option<(bool, &'ast str)> {
        let id = expression.as_identifier_reference(ast)?;
        let id = resolver.symbol(id.node_id())?;
        self.imports.get(&id).copied()
    }

    fn is_pure_call(&self, ast: &Ast<'ast>, resolver: &Resolver, call: CallExpression) -> bool {
        if call.optional(ast) {
            return false;
        }
        let callee = call.callee(ast);
        if let Some((react_dom, specifier)) = self.import(ast, resolver, callee) {
            return is_pure(react_dom, specifier);
        }
        let Some(member) = callee.as_member_expression(ast) else {
            return false;
        };
        if member.optional(ast) || member.computed(ast) {
            return false;
        }
        let Some((react_dom, specifier)) = self.import(ast, resolver, member.object(ast)) else {
            return false;
        };
        matches!(specifier, "default" | "*")
            && member
                .property(ast)
                .as_identifier_name(ast)
                .is_some_and(|property| is_pure(react_dom, ast.get_utf8(property.name(ast))))
    }
}

fn is_pure(react_dom: bool, specifier: &str) -> bool {
    if react_dom {
        specifier == "createPortal"
    } else {
        matches!(
            specifier,
            "cloneElement"
                | "createContext"
                | "createElement"
                | "createFactory"
                | "createRef"
                | "forwardRef"
                | "isValidElement"
                | "memo"
                | "lazy"
        )
    }
}
