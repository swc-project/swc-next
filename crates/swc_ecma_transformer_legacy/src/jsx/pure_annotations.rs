use std::collections::{HashMap, HashSet};

use swc_next_ecma_ast::*;

use crate::common::resolver::context::Resolver;

/// SWC's annotations for calls to known pure React and React DOM APIs.
/// <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/pure_annotations.rs>.
pub(super) fn hook(ast: &mut Ast<'_>, resolver: &Resolver, pure_positions: &mut HashSet<u32>) {
    let program = ast.root_program();
    let mut imports = HashMap::new();
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
                        ModuleExportNameData::IdentifierName(name) => ast.get_utf8(name.name(ast)),
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
            imports.insert(id, (src == "react-dom", name.to_owned()));
        }
    }
    if !imports.is_empty() {
        program.visit_mut_with(&mut PureAnnotations {
            ast,
            resolver,
            imports,
            pure_positions,
        });
    }
}

struct PureAnnotations<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a Resolver,
    imports: HashMap<SymbolId, (bool, String)>,
    pure_positions: &'a mut HashSet<u32>,
}

impl PureAnnotations<'_, '_> {
    fn import(&self, expression: Expr) -> Option<&(bool, String)> {
        let id = expression.as_identifier_reference(self.ast)?;
        let id = self.resolver.symbol(id.node_id())?;
        self.imports.get(&id)
    }

    fn is_pure_call(&self, call: CallExpression) -> bool {
        if call.optional(self.ast) {
            return false;
        }
        let callee = call.callee(self.ast);
        if let Some((react_dom, specifier)) = self.import(callee) {
            return is_pure(*react_dom, specifier);
        }
        let Some(member) = callee.as_member_expression(self.ast) else {
            return false;
        };
        if member.optional(self.ast) || member.computed(self.ast) {
            return false;
        }
        let Some((react_dom, specifier)) = self.import(member.object(self.ast)) else {
            return false;
        };
        matches!(specifier.as_str(), "default" | "*")
            && member
                .property(self.ast)
                .as_identifier_name(self.ast)
                .is_some_and(|property| {
                    is_pure(*react_dom, self.ast.get_utf8(property.name(self.ast)))
                })
    }
}

impl<'ast> VisitMut<'ast> for PureAnnotations<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_call_expression(&mut self, call: CallExpression) -> Option<CallExpression> {
        if self.is_pure_call(call) {
            call.set_pure(self.ast, true);
            self.pure_positions.insert(call.span(self.ast).start);
        }
        call.visit_mut_children_with(self)
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
