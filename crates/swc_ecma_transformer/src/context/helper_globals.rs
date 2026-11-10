//! Protect free names in inline helpers from bindings in the surrounding program.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingProperty, DUMMY_SP, ImportOrExportKind, ModuleExportName, NodeId,
    NodeKindData, ObjectProperty, Stmt, SymbolId, Visit, VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_semantic::SymbolFlags;

use super::TransformCtx;

impl TransformCtx<'_, '_> {
    pub(crate) fn protect_helper_globals(&mut self) {
        let scope = self.program_scope();
        let mut renamed = FxHashMap::default();
        for global in std::mem::take(&mut self.state.helpers.globals) {
            let Some(symbol) = self.semantic().binding(scope, global.as_bytes()) else {
                continue;
            };
            if !self
                .semantic()
                .symbol(symbol)
                .flags
                .intersects(SymbolFlags::VALUE_SPACE)
            {
                continue;
            }
            let name = self.generate_name(&format!("_{global}"), 2);
            let linked: Vec<_> = self.semantic().linked_symbols(symbol).collect();
            for symbol in linked {
                if renamed.contains_key(&symbol) {
                    continue;
                }
                let original = self.ast.add_utf8(&global);
                renamed.insert(symbol, original);
                self.rename_symbol(symbol, name);
            }
        }
        if renamed.is_empty() {
            return;
        }
        self.preserve_helper_shadow_exports(&renamed);
        self.ast
            .root_program()
            .visit_mut_with(&mut ExpandedShorthands { ast: self.ast });
        self.preserve_renamed_function_names(&renamed);
        // Restoring an inferred name can request SetFunctionName for the first time.
        if !self.state.helpers.globals.is_empty() {
            self.protect_helper_globals();
        }
    }

    fn preserve_helper_shadow_exports(&mut self, renamed: &FxHashMap<SymbolId, AstUtf8>) {
        let program = self.ast.root_program();
        let scope = self.program_scope();
        let mut statements = Vec::new();
        for slot in program.body(self.ast).iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            if let Some(export) = statement.as_export_named_declaration(self.ast) {
                for slot in export.specifiers(self.ast).iter() {
                    let specifier = self.ast.get_node_in_sub_range(slot);
                    let local = specifier.local(self.ast).node_id();
                    // Parser shorthand exports can share the local/exported node.
                    if local == specifier.exported(self.ast).node_id()
                        && let Some(reference) = self.semantic().reference_of(local)
                        && let Some(symbol) = self.semantic().reference(reference).symbol
                        && let Some(&name) = renamed.get(&symbol)
                    {
                        let exported = ModuleExportName::IdentifierName(
                            self.add_node(|ast| ast.identifier_name(DUMMY_SP, name)),
                        );
                        specifier.set_exported(self.ast, exported);
                    }
                }
            }
            if let Some(export) = statement.as_export_named_declaration(self.ast)
                && let Some(declaration) = export.declaration(self.ast)
            {
                let mut nodes = DeclarationNodes {
                    ast: self.ast,
                    nodes: FxHashSet::default(),
                };
                declaration.visit_with(&mut nodes);
                let mut seen = FxHashSet::default();
                let bindings: Vec<_> = self
                    .semantic()
                    .iter_declarations()
                    .filter(|entry| {
                        entry.declaration.scope == scope
                            && nodes.nodes.contains(&entry.declaration.node)
                            && seen.insert(entry.declaration.symbol)
                    })
                    .map(|entry| entry.declaration)
                    .collect();
                if bindings
                    .iter()
                    .any(|binding| renamed.contains_key(&binding.symbol))
                {
                    let mut specifiers = Vec::new();
                    for binding in bindings {
                        let NodeKindData::BindingIdentifier(identifier) =
                            self.ast.kind_data(binding.node)
                        else {
                            unreachable!()
                        };
                        let name = renamed
                            .get(&binding.symbol)
                            .copied()
                            .unwrap_or(identifier.name(self.ast));
                        let exported = ModuleExportName::IdentifierName(
                            self.add_node(|ast| ast.identifier_name(DUMMY_SP, name)),
                        );
                        let local_name = identifier.name(self.ast);
                        let local = self.add_node(|ast| ast.identifier_name(DUMMY_SP, local_name));
                        self.register_export_reference(scope, local, binding.symbol);
                        let local = ModuleExportName::IdentifierName(local);
                        specifiers.push(self.add_node(|ast| {
                            ast.export_specifier(
                                DUMMY_SP,
                                local,
                                exported,
                                ImportOrExportKind::Value,
                            )
                        }));
                    }
                    export.set_declaration(self.ast, None);
                    let specifiers = self.ast.add_sub_range(specifiers);
                    export.set_specifiers(self.ast, specifiers);
                    statements.push(Stmt::Declaration(declaration));
                }
            }
            statements.push(statement);
        }
        let body = self.ast.add_sub_range(statements);
        program.set_body(self.ast, body);
    }
}

struct DeclarationNodes<'s, 'a> {
    ast: &'s Ast<'a>,
    nodes: FxHashSet<NodeId>,
}

impl<'a> Visit<'a> for DeclarationNodes<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }
    fn enter_node(&mut self, node: NodeKindData) {
        self.nodes.insert(node.node_id());
    }
}

struct ExpandedShorthands<'s, 'a> {
    ast: &'s mut Ast<'a>,
}

impl<'a> VisitMut<'a> for ExpandedShorthands<'_, 'a> {
    fn ast(&mut self) -> &mut Ast<'a> {
        self.ast
    }

    fn visit_mut_object_property(&mut self, node: ObjectProperty) -> Option<ObjectProperty> {
        node.visit_mut_children_with(self);
        if node.shorthand(self.ast)
            && let Some(key) = node.key(self.ast).as_identifier_name(self.ast)
            && let Some(value) = node.value(self.ast).as_identifier_reference(self.ast)
            && self.ast.get_utf8(key.name(self.ast)) != self.ast.get_utf8(value.name(self.ast))
        {
            node.set_shorthand(self.ast, false);
        }
        None
    }

    fn visit_mut_binding_property(&mut self, node: BindingProperty) -> Option<BindingProperty> {
        node.visit_mut_children_with(self);
        let value = node.value(self.ast);
        let binding = value.as_binding_identifier(self.ast).or_else(|| {
            value
                .as_assignment_pattern(self.ast)?
                .left(self.ast)
                .as_binding_identifier(self.ast)
        });
        if node.shorthand(self.ast)
            && let Some(key) = node.key(self.ast).as_identifier_name(self.ast)
            && let Some(binding) = binding
            && self.ast.get_utf8(key.name(self.ast)) != self.ast.get_utf8(binding.name(self.ast))
        {
            node.set_shorthand(self.ast, false);
        }
        None
    }
}
