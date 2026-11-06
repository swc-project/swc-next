use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingProperty, DUMMY_SP, Decl, Function, ImportDeclarationSpecifierData,
    ImportOrExportKind, ModuleExportName, NodeId, NodeKindData, ObjectProperty, ScopeId,
    SourceType, Stmt, SymbolId, Visit, VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SymbolFlags, analyze};

use super::{BoundIdentifier, TransformCtx};
use crate::helpers::{HelperKind, PreparedHelper};

impl TransformCtx<'_, '_> {
    pub(super) fn prepare_inline_helper(&mut self, helper: HelperKind) -> PreparedHelper {
        let allocator = Allocator::new();
        let parsed = Parser::init(
            &allocator,
            helper.code(),
            Options {
                source_type: SourceType::Module,
                comments: CommentMode::None,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parsed.diagnostics.is_empty(), "invalid packaged helper");
        let source = parsed.ast;
        let analyzed = analyze(&source, AnalyzeOptions::default());
        assert!(analyzed.diagnostics.is_empty(), "invalid helper semantics");
        let semantic = analyzed.semantic;
        let program = source.root_program();
        let mut function = None;
        let mut imports = Vec::new();
        for slot in program.body(&source).iter() {
            match source.kind_data(source.get_node_in_sub_range(slot).node_id()) {
                NodeKindData::Function(node) => {
                    assert!(
                        function.replace(node).is_none(),
                        "one function per helper module"
                    );
                }
                NodeKindData::ImportDeclaration(import) => imports.push(import),
                NodeKindData::ExportNamedDeclaration(export) => {
                    assert!(
                        export.declaration(&source).is_none() && export.source(&source).is_none()
                    );
                }
                _ => panic!("helper modules contain imports, one function and its export"),
            }
        }
        let function: Function = function.expect("helper function");
        let name = function.id(&source).expect("named helper function");
        assert_eq!(source.get_utf8(name.name(&source)), helper.name());
        let mut live = HelperNodes {
            ast: &source,
            nodes: FxHashSet::default(),
        };
        function.visit_with(&mut live);

        // Imported dependency names may acquire suffixes. Reserve every local
        // name before choosing them so those references cannot be captured.
        for entry in semantic.iter_declarations() {
            if entry.declaration.scope != ScopeId::MODULE {
                let name = source.get_wtf8(semantic.symbol(entry.declaration.symbol).name);
                self.reserve_name(name.as_str().expect("helper identifier"));
            }
        }
        let mut symbols = FxHashMap::default();
        for import in imports {
            let path = source.get_wtf8(import.source(&source).value(&source));
            let dependency = HelperKind::from_import(path.as_str().expect("helper import path"))
                .expect("packaged helper dependency");
            let binding = self.ensure_helper(dependency);
            for slot in import.specifiers(&source).iter() {
                let specifier = source.get_node_in_sub_range(slot);
                let ImportDeclarationSpecifierData::ImportSpecifier(specifier) =
                    source.import_declaration_specifier_data(specifier)
                else {
                    panic!("helpers use named imports");
                };
                let imported = specifier
                    .imported(&source)
                    .as_identifier_name(&source)
                    .expect("named helper export");
                assert_eq!(source.get_utf8(imported.name(&source)), "_");
                let symbol = semantic
                    .symbol_of(specifier.local(&source).node_id())
                    .unwrap();
                symbols.insert(symbol, binding);
            }
        }

        let mapping = self.add_node(|ast| ast.import_synthesized(&source));
        let scope = self.program_scope();
        let mut scopes = FxHashMap::default();
        scopes.insert(ScopeId::ROOT, scope);
        scopes.insert(ScopeId::MODULE, scope);
        for entry in semantic.iter_scopes() {
            if !live.nodes.contains(&entry.scope.node) {
                continue;
            }
            let parent = scopes[&entry.scope.parent.expect("helper scope parent")];
            let child = self.create_child_scope(
                parent,
                mapping.node_id(entry.scope.node),
                entry.scope.kind,
                entry.scope.flags,
            );
            scopes.insert(entry.id, child);
        }
        let generated_name = self.generate_name(helper.name(), 2);
        for entry in semantic.iter_declarations() {
            let declaration = entry.declaration;
            if !live.nodes.contains(&declaration.node) {
                continue;
            }
            let NodeKindData::BindingIdentifier(original) = source.kind_data(declaration.node)
            else {
                panic!("helper declaration must bind an identifier");
            };
            let identifier = mapping.node(original);
            if original == name {
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
            if !live.nodes.contains(&reference.node) {
                continue;
            }
            let NodeKindData::IdentifierReference(original) = source.kind_data(reference.node)
            else {
                panic!("helper reference must be an identifier");
            };
            let node = mapping.node(original);
            let target = reference.symbol.map(|symbol| symbols[&symbol]);
            if let Some(binding) = target {
                node.set_name(self.ast, binding.identifier.name(self.ast));
            } else {
                self.state
                    .helpers
                    .globals
                    .insert(source.get_utf8(reference.name).to_owned());
            }
            self.register_reference(
                scopes[&reference.scope],
                node,
                target.map(|binding| binding.symbol),
                reference.flags,
            );
        }
        let imported = mapping.node(function);
        // ESM helper sources are strict even when inserted into a sloppy script.
        if !self.semantic().scope(scope).flags.strict {
            let body = imported.body(self.ast);
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
            statement: Stmt::Declaration(Decl::Function(imported)),
        }
    }

    pub(super) fn protect_helper_globals(&mut self) {
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
                let mut nodes = HelperNodes {
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

struct HelperNodes<'s, 'a> {
    ast: &'s Ast<'a>,
    nodes: FxHashSet<NodeId>,
}

impl<'a> Visit<'a> for HelperNodes<'_, 'a> {
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
