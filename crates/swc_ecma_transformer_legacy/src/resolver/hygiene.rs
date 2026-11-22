use std::collections::HashMap;
use swc_next_ecma_ast::*;

use super::{
    Resolver,
    analyzer::{self, Id},
    utils::for_each_binding_id,
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/hygiene/mod.rs#L68>.
pub(super) fn apply(ast: &mut Ast<'_>, resolver: &mut Resolver) {
    let renames = analyzer::analyze(ast, resolver)
        .into_iter()
        .map(|(id, name)| (id, ast.add_utf8(&name)))
        .collect();
    rename(ast, resolver, renames);
}

/// SWC's explicit rename pass, also used by Flow's `await` normalization.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/rename/mod.rs#L1>.
pub(super) fn rename(ast: &mut Ast<'_>, resolver: &mut Resolver, renames: HashMap<Id, AstUtf8>) {
    let program = ast.root_program();
    let mut hygiene = Hygiene {
        ast,
        resolver,
        renames,
        exports: HashMap::new(),
    };
    let _ = program.visit_mut_with(&mut hygiene);
}

struct Hygiene<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    resolver: &'visit mut Resolver,
    renames: HashMap<Id, AstUtf8>,
    exports: HashMap<NodeId, Vec<(BindingIdentifier, AstUtf8)>>,
}

impl Hygiene<'_, '_> {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L649>.
    fn name(&self, node: NodeId, name: AstUtf8) -> Option<AstUtf8> {
        self.renames
            .get(&self.resolver.id(node, name, self.ast))
            .copied()
    }
}

impl<'a> VisitMut<'a> for Hygiene<'_, 'a> {
    fn ast(&mut self) -> &mut Ast<'a> {
        self.ast
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L240>.
    fn visit_mut_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
    ) -> Option<ExportNamedDeclaration> {
        if node.source(self.ast).is_some() {
            return None;
        }
        if let Some(declaration) = node.declaration(self.ast) {
            let mut bindings = Vec::new();
            match self.ast.decl_data(declaration) {
                DeclData::VariableDeclaration(declaration) => {
                    for id in declaration.declarators(self.ast).iter() {
                        let pattern = self.ast.get_node_in_sub_range(id).id(self.ast);
                        for_each_binding_id(self.ast, pattern, |binding| bindings.push(binding));
                    }
                }
                DeclData::Function(function) => bindings.extend(function.id(self.ast)),
                DeclData::Class(class) => bindings.extend(class.id(self.ast)),
                _ => {}
            }
            let renamed = bindings
                .into_iter()
                .filter(|binding| {
                    self.name(binding.node_id(), binding.name(self.ast))
                        .is_some()
                })
                .map(|binding| (binding, binding.name(self.ast)))
                .collect::<Vec<_>>();
            if !renamed.is_empty() {
                self.exports.insert(node.node_id(), renamed);
            }
        }
        node.visit_mut_children_with(self)
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L396>.
    fn visit_mut_program(&mut self, program: Program) -> Option<Program> {
        let _ = program.visit_mut_children_with(self);
        if self.exports.is_empty() {
            return None;
        }
        let statements = program
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect::<Vec<_>>();
        let mut body = Vec::new();
        for statement in statements {
            if let Some(bindings) = self.exports.remove(&statement.node_id()) {
                let export = statement.as_export_named_declaration(self.ast).unwrap();
                body.push(Stmt::Declaration(export.declaration(self.ast).unwrap()));
                let specifiers = bindings
                    .into_iter()
                    .map(|(binding, exported)| {
                        let local = self.ast.identifier_name(DUMMY_SP, binding.name(self.ast));
                        self.resolver.register(self.ast);
                        self.resolver.node_scopes[local.node_id()] =
                            self.resolver.node_scopes[binding.node_id()];
                        let exported = ModuleExportName::IdentifierName(
                            self.ast.identifier_name(DUMMY_SP, exported),
                        );
                        self.ast.export_specifier(
                            DUMMY_SP,
                            ModuleExportName::IdentifierName(local),
                            exported,
                            ImportOrExportKind::Value,
                        )
                    })
                    .collect::<Vec<_>>();
                let specifiers = self.ast.add_sub_range(specifiers);
                body.push(self.ast.stmt_export_named_declaration(
                    export.span(self.ast),
                    None,
                    specifiers,
                    None,
                    SubRange::empty(),
                    ImportOrExportKind::Value,
                ));
            } else {
                body.push(statement);
            }
        }
        let body = self.ast.add_sub_range(body);
        program.set_body(self.ast, body);
        self.resolver.register(self.ast);
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L154>.
    fn visit_mut_export_specifier(&mut self, node: ExportSpecifier) -> Option<ExportSpecifier> {
        if let Some(local) = node.local(self.ast).as_identifier_name(self.ast)
            && let Some(name) = self.name(local.node_id(), local.name(self.ast))
        {
            let scope = self.resolver.node_scopes[local.node_id()];
            let local = self.ast.identifier_name(local.span(self.ast), name);
            self.resolver.register(self.ast);
            self.resolver.node_scopes[local.node_id()] = scope;
            node.set_local(self.ast, ModuleExportName::IdentifierName(local));
        }
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L196>.
    fn visit_mut_binding_identifier(
        &mut self,
        node: BindingIdentifier,
    ) -> Option<BindingIdentifier> {
        if let Some(name) = self.name(node.node_id(), node.name(self.ast)) {
            node.set_name(self.ast, name);
        }
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L196>.
    fn visit_mut_identifier_reference(
        &mut self,
        node: IdentifierReference,
    ) -> Option<IdentifierReference> {
        if let Some(name) = self.name(node.node_id(), node.name(self.ast)) {
            node.set_name(self.ast, name);
        }
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L498>.
    fn visit_mut_object_property(&mut self, node: ObjectProperty) -> Option<ObjectProperty> {
        let _ = node.visit_mut_children_with(self);
        if node.shorthand(self.ast) {
            let key = node.key(self.ast).as_identifier_name(self.ast).unwrap();
            let value = node
                .value(self.ast)
                .as_identifier_reference(self.ast)
                .unwrap();
            node.set_shorthand(
                self.ast,
                self.ast.get_utf8(key.name(self.ast)) == self.ast.get_utf8(value.name(self.ast)),
            );
        }
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L465>.
    fn visit_mut_binding_property(&mut self, node: BindingProperty) -> Option<BindingProperty> {
        let _ = node.visit_mut_children_with(self);
        if node.shorthand(self.ast) {
            let key = node.key(self.ast).as_identifier_name(self.ast).unwrap();
            let mut value = node.value(self.ast);
            if let Some(assignment) = value.as_assignment_pattern(self.ast) {
                value = assignment.left(self.ast);
            }
            let name = match self.ast.binding_pattern_data(value) {
                BindingPatternData::BindingIdentifier(binding) => binding.name(self.ast),
                BindingPatternData::SimpleAssignmentTarget(target) => target
                    .as_identifier_reference(self.ast)
                    .unwrap()
                    .name(self.ast),
                _ => unreachable!("shorthand property must contain an identifier"),
            };
            node.set_shorthand(
                self.ast,
                self.ast.get_utf8(key.name(self.ast)) == self.ast.get_utf8(name),
            );
        }
        None
    }

    /// JSX identifiers are renamed by SWC's identifier visitor: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L196>.
    fn visit_mut_jsx_element_name(&mut self, node: JsxElementName) -> Option<JsxElementName> {
        match self.ast.jsx_element_name_data(node) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                if let Some(name) = self.name(identifier.node_id(), identifier.name(self.ast)) {
                    identifier.set_name(self.ast, name);
                }
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                let _ = member.object(self.ast).visit_mut_with(self);
            }
            JsxElementNameData::JsxNamespacedName(_) => {}
        }
        None
    }

    /// JSX object roots use SWC's identifier visitor: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L196>.
    fn visit_mut_jsx_member_expression_object(
        &mut self,
        node: JsxMemberExpressionObject,
    ) -> Option<JsxMemberExpressionObject> {
        match self.ast.jsx_member_expression_object_data(node) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                if let Some(name) = self.name(identifier.node_id(), identifier.name(self.ast)) {
                    identifier.set_name(self.ast, name);
                }
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                let _ = member.object(self.ast).visit_mut_with(self);
            }
        }
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/ops.rs#L101>.
    fn visit_mut_ts_type(&mut self, _: TsType) -> Option<TsType> {
        None
    }
}
