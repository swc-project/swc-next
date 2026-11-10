use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    DUMMY_SP, Decl, ImportDeclarationSpecifierData, NodeKindData, ScopeId, Stmt,
};
use swc_next_ecma_parser::Parser;
use swc_next_ecma_semantic::{AnalyzeOptions, SymbolFlags, analyze_synthetic_module};

use super::{BoundIdentifier, TransformCtx};
use crate::helpers::{HelperKind, PreparedHelper};

impl TransformCtx<'_, '_> {
    pub(super) fn prepare_inline_helper(&mut self, helper: HelperKind) -> PreparedHelper {
        let first_node = self.ast.node_count();
        let program = self
            .add_node(|ast| Parser::parse_synthetic_module(ast, helper.code()))
            .expect("valid packaged helper");
        let (semantic, diagnostics) =
            analyze_synthetic_module(self.ast, program, first_node, AnalyzeOptions::default());
        assert!(diagnostics.is_empty(), "invalid helper semantics");
        let mut function = None;
        let mut imports = Vec::new();
        for slot in program.body(self.ast).iter() {
            match self
                .ast
                .kind_data(self.ast.get_node_in_sub_range(slot).node_id())
            {
                NodeKindData::Function(node) => {
                    assert!(
                        function.replace(node).is_none(),
                        "one function per helper module"
                    );
                }
                NodeKindData::ImportDeclaration(import) => imports.push(import),
                NodeKindData::ExportNamedDeclaration(export) => {
                    assert!(
                        export.declaration(self.ast).is_none() && export.source(self.ast).is_none()
                    );
                }
                _ => panic!("helper modules contain imports, one function and its export"),
            }
        }
        let function = function.expect("helper function");
        let name = function.id(self.ast).expect("named helper function");
        assert_eq!(self.ast.get_utf8(name.name(self.ast)), helper.name());
        // Imported dependency names may acquire suffixes. Reserve every local
        // name before choosing them so those references cannot be captured.
        for entry in semantic.iter_declarations() {
            if entry.declaration.scope != ScopeId::MODULE {
                let name = self
                    .ast
                    .get_wtf8(semantic.symbol(entry.declaration.symbol).name);
                self.reserve_name(name.as_str().expect("helper identifier"));
            }
        }
        let mut symbols = FxHashMap::default();
        for import in imports {
            let path = self.ast.get_wtf8(import.source(self.ast).value(self.ast));
            let dependency = HelperKind::from_import(path.as_str().expect("helper import path"))
                .expect("packaged helper dependency");
            let binding = self.ensure_helper(dependency);
            for slot in import.specifiers(self.ast).iter() {
                let specifier = self.ast.get_node_in_sub_range(slot);
                let ImportDeclarationSpecifierData::ImportSpecifier(specifier) =
                    self.ast.import_declaration_specifier_data(specifier)
                else {
                    panic!("helpers use named imports");
                };
                let imported = specifier
                    .imported(self.ast)
                    .as_identifier_name(self.ast)
                    .expect("named helper export");
                assert_eq!(self.ast.get_utf8(imported.name(self.ast)), "_");
                let symbol = semantic
                    .symbol_of(specifier.local(self.ast).node_id())
                    .unwrap();
                symbols.insert(symbol, binding);
            }
        }

        let scope = self.program_scope();
        let mut scopes = FxHashMap::default();
        scopes.insert(ScopeId::ROOT, scope);
        scopes.insert(ScopeId::MODULE, scope);
        for entry in semantic.iter_scopes() {
            if entry.scope.node == program.node_id() {
                continue;
            }
            let parent = scopes[&entry.scope.parent.expect("helper scope parent")];
            let child = self.create_child_scope(
                parent,
                entry.scope.node,
                entry.scope.kind,
                entry.scope.flags,
            );
            scopes.insert(entry.id, child);
        }
        let generated_name = self.generate_name(helper.name(), 2);
        for entry in semantic.iter_declarations() {
            let declaration = entry.declaration;
            if declaration.scope == ScopeId::MODULE && declaration.node != name.node_id() {
                continue;
            }
            let NodeKindData::BindingIdentifier(identifier) = self.ast.kind_data(declaration.node)
            else {
                panic!("helper declaration must bind an identifier");
            };
            if identifier == name {
                identifier.set_name(self.ast, generated_name);
            }
            let symbol = self.register_binding(
                scopes[&declaration.scope],
                identifier,
                declaration.flags - SymbolFlags::EXPORTED,
            );
            symbols.insert(declaration.symbol, BoundIdentifier { identifier, symbol });
        }
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            if reference.scope == ScopeId::MODULE {
                continue;
            }
            let NodeKindData::IdentifierReference(node) = self.ast.kind_data(reference.node) else {
                panic!("helper reference must be an identifier");
            };
            let target = reference.symbol.map(|symbol| symbols[&symbol]);
            if let Some(binding) = target {
                let name = binding.identifier.name(self.ast);
                if node.name(self.ast) != name {
                    node.set_name(self.ast, name);
                }
            } else {
                self.state
                    .helpers
                    .globals
                    .insert(self.ast.get_utf8(reference.name).to_owned());
            }
            self.register_reference(
                scopes[&reference.scope],
                node,
                target.map(|binding| binding.symbol),
                reference.flags,
            );
        }
        // ESM helper sources are strict even when inserted into a sloppy script.
        if !self.semantic().scope(scope).flags.strict {
            let body = function.body(self.ast);
            let value = self.ast.add_utf8("use strict");
            let expression =
                self.add_node(|ast| ast.string_literal(DUMMY_SP, value.into(), ast.empty_utf8()));
            let directive = self.add_node(|ast| ast.directive(DUMMY_SP, expression, value));
            let directives = self.ast.add_sub_range(
                std::iter::once(directive).chain(
                    body.directives(self.ast)
                        .iter()
                        .map(|slot| self.ast.get_node_in_sub_range(slot))
                        .collect::<Vec<_>>(),
                ),
            );
            body.set_directives(self.ast, directives);
        }
        PreparedHelper {
            bound_identifier: symbols[&semantic.symbol_of(name.node_id()).unwrap()],
            statement: Stmt::Declaration(Decl::Function(function)),
        }
    }
}
