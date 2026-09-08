use hashbrown::HashMap;
use rustc_hash::FxBuildHasher;
use swc_next_ecma_ast::{
    Ast, ExportDefaultDeclarationKindData, ExportNamedDeclaration, ModuleExportNameData, NodeId,
    NodeKindData, TsModuleDeclaration, TsModuleName, TsModuleNameData, TsQualifiedNameLeftData,
};

use super::TsContainerContext;

/// TypeScript's final module-instance states. `ConstEnumOnly` has value
/// meaning during binding, while remaining distinguishable from a namespace
/// that necessarily emits a runtime object.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum TsModuleInstanceState {
    NonInstantiated,
    ConstEnumOnly,
    Instantiated,
}

impl TsModuleInstanceState {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Instantiated, _) | (_, Self::Instantiated) => Self::Instantiated,
            (Self::ConstEnumOnly, _) | (_, Self::ConstEnumOnly) => Self::ConstEnumOnly,
            (Self::NonInstantiated, Self::NonInstantiated) => Self::NonInstantiated,
        }
    }
}

pub(super) fn ts_module_instance_state(
    tree: &Ast<'_>,
    declaration: TsModuleDeclaration,
    outer_contexts: &[TsContainerContext],
    module_states: &mut HashMap<NodeId, TsModuleInstanceMemo, FxBuildHasher>,
) -> TsModuleInstanceState {
    // Keep the allocation across declarations, but never reuse answers from
    // another root: cycle results depend on the current traversal.
    module_states.clear();
    TsModuleInstanceAnalyzer {
        tree,
        module_states,
    }
    .module_in_contexts(declaration, outer_contexts, None)
}

/// Computes the same three final states used by TypeScript's
/// `getModuleInstanceState`. The traversal is syntactic because the state is
/// needed while the module symbol itself is being bound. Local export
/// specifiers search the current and enclosing module blocks, matching the
/// binder's forward-reference behavior.
struct TsModuleInstanceAnalyzer<'tree, 'ast, 'scratch> {
    tree: &'tree Ast<'ast>,
    module_states: &'scratch mut HashMap<NodeId, TsModuleInstanceMemo, FxBuildHasher>,
}

#[derive(Clone, Copy)]
pub(super) enum TsModuleInstanceMemo {
    Computing,
    Complete(TsModuleInstanceState),
}

impl TsModuleInstanceAnalyzer<'_, '_, '_> {
    fn module_in_contexts(
        &mut self,
        declaration: TsModuleDeclaration,
        contexts: &[TsContainerContext],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        let Some((context, remaining)) = contexts.split_first() else {
            return self.module(declaration, blocks);
        };
        let Some(body) = context.body() else {
            return self.module_in_contexts(declaration, remaining, blocks);
        };
        let blocks = TsModuleBlockPath {
            node: body,
            parent: blocks,
        };
        self.module_in_contexts(declaration, remaining, Some(&blocks))
    }

    fn module(
        &mut self,
        declaration: TsModuleDeclaration,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        let node = declaration.node_id();
        if let Some(state) = self.module_states.get(&node) {
            // An in-progress dependency is a cycle, while completed states
            // are shared across every path through the current dependency
            // graph.
            return match state {
                TsModuleInstanceMemo::Computing => TsModuleInstanceState::NonInstantiated,
                TsModuleInstanceMemo::Complete(state) => *state,
            };
        }
        self.module_states
            .insert(node, TsModuleInstanceMemo::Computing);
        let state = declaration
            .body(self.tree)
            .map_or(TsModuleInstanceState::Instantiated, |body| {
                self.module_body(body.node_id(), blocks)
            });
        self.module_states
            .insert(node, TsModuleInstanceMemo::Complete(state));
        state
    }

    fn module_body(
        &mut self,
        body: NodeId,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        match self.tree.kind_data(body) {
            NodeKindData::TsModuleBlock(block) => {
                let blocks = TsModuleBlockPath {
                    node: body,
                    parent: blocks,
                };
                let mut state = TsModuleInstanceState::NonInstantiated;
                for statement in block.body(self.tree).iter() {
                    let statement = self.tree.get_node_in_sub_range(statement).node_id();
                    let child = self.statement(statement, Some(&blocks));
                    state = state.combine(child);
                    if state == TsModuleInstanceState::Instantiated {
                        break;
                    }
                }
                state
            }
            NodeKindData::TsModuleDeclaration(declaration) => {
                match self.tree.ts_module_name_data(declaration.id(self.tree)) {
                    TsModuleNameData::StringLiteral(_) => TsModuleInstanceState::NonInstantiated,
                    _ => self.module(declaration, blocks),
                }
            }
            _ => TsModuleInstanceState::Instantiated,
        }
    }

    fn statement(
        &mut self,
        node: NodeId,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        match self.tree.kind_data(node) {
            NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::ImportDeclaration(_)
            | NodeKindData::TsImportEqualsDeclaration(_) => TsModuleInstanceState::NonInstantiated,
            NodeKindData::TsEnumDeclaration(declaration) => {
                if declaration.is_const(self.tree) {
                    TsModuleInstanceState::ConstEnumOnly
                } else {
                    TsModuleInstanceState::Instantiated
                }
            }
            NodeKindData::TsModuleDeclaration(declaration) => self.module(declaration, blocks),
            NodeKindData::ExportDefaultDeclaration(export) => {
                match self
                    .tree
                    .export_default_declaration_kind_data(export.declaration(self.tree))
                {
                    ExportDefaultDeclarationKindData::TsInterfaceDeclaration(_) => {
                        TsModuleInstanceState::NonInstantiated
                    }
                    ExportDefaultDeclarationKindData::Function(_)
                    | ExportDefaultDeclarationKindData::TsFunction(_)
                    | ExportDefaultDeclarationKindData::Class(_)
                    | ExportDefaultDeclarationKindData::Expr(_) => {
                        TsModuleInstanceState::Instantiated
                    }
                }
            }
            NodeKindData::ExportNamedDeclaration(export) => self.export_declaration(export, blocks),
            _ => TsModuleInstanceState::Instantiated,
        }
    }

    fn export_declaration(
        &mut self,
        export: ExportNamedDeclaration,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        if let Some(declaration) = export.declaration(self.tree) {
            if matches!(
                self.tree.kind_data(declaration.node_id()),
                NodeKindData::TsImportEqualsDeclaration(_)
            ) {
                return TsModuleInstanceState::Instantiated;
            }
            return self.statement(declaration.node_id(), blocks);
        }
        if export.source(self.tree).is_some() {
            return TsModuleInstanceState::Instantiated;
        }

        let mut state = TsModuleInstanceState::NonInstantiated;
        for specifier in export.specifiers(self.tree).iter() {
            let specifier = self.tree.get_node_in_sub_range(specifier);
            let local = specifier.local(self.tree);
            let ModuleExportNameData::IdentifierName(local) =
                self.tree.module_export_name_data(local)
            else {
                return TsModuleInstanceState::Instantiated;
            };
            let name = self.tree.get_utf8(local.name(self.tree)).as_bytes();
            let target = self.local_export_target(name, blocks);
            state = state.combine(target);
            if state == TsModuleInstanceState::Instantiated {
                break;
            }
        }
        state
    }

    fn local_export_target(
        &mut self,
        name: &[u8],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        let mut block = blocks;
        while let Some(path) = block {
            if let Some(state) = self.named_statement_state(path.node, name, Some(path)) {
                return state;
            }
            block = path.parent;
        }
        if let Some(state) = self.named_program_statement_state(name) {
            return state;
        }
        // Like TypeScript, an unresolved syntactic target is conservatively
        // treated as a possible value.
        TsModuleInstanceState::Instantiated
    }

    fn named_program_statement_state(&mut self, name: &[u8]) -> Option<TsModuleInstanceState> {
        let statements = self
            .tree
            .root_program()
            .body(self.tree)
            .iter()
            .map(|statement| self.tree.get_node_in_sub_range(statement).node_id());
        self.named_statements_state(statements, name, None)
    }

    fn named_statement_state(
        &mut self,
        body: NodeId,
        name: &[u8],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> Option<TsModuleInstanceState> {
        let NodeKindData::TsModuleBlock(block) = self.tree.kind_data(body) else {
            return None;
        };
        let statements = block
            .body(self.tree)
            .iter()
            .map(|statement| self.tree.get_node_in_sub_range(statement).node_id());
        self.named_statements_state(statements, name, blocks)
    }

    fn named_statements_state(
        &mut self,
        statements: impl Iterator<Item = NodeId>,
        name: &[u8],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> Option<TsModuleInstanceState> {
        let mut found = None;
        for statement in statements {
            let declaration = self.exported_declaration(statement);
            if !ts_module_statement_has_name(self.tree, declaration, name) {
                continue;
            }
            let mut state = self.statement(declaration, blocks);
            if matches!(
                self.tree.kind_data(declaration),
                NodeKindData::TsImportEqualsDeclaration(_)
            ) {
                state = TsModuleInstanceState::Instantiated;
            }
            found = Some(found.map_or(state, |current: TsModuleInstanceState| {
                current.combine(state)
            }));
            if found == Some(TsModuleInstanceState::Instantiated) {
                break;
            }
        }
        found
    }

    fn exported_declaration(&self, node: NodeId) -> NodeId {
        match self.tree.kind_data(node) {
            NodeKindData::ExportNamedDeclaration(export) => export
                .declaration(self.tree)
                .map_or(node, |declaration| declaration.node_id()),
            NodeKindData::ExportDefaultDeclaration(export) => {
                let declaration = export.declaration(self.tree);
                match self.tree.export_default_declaration_kind_data(declaration) {
                    ExportDefaultDeclarationKindData::Function(function) => function.node_id(),
                    ExportDefaultDeclarationKindData::TsFunction(function) => function.node_id(),
                    ExportDefaultDeclarationKindData::Class(class) => class.node_id(),
                    ExportDefaultDeclarationKindData::TsInterfaceDeclaration(interface) => {
                        interface.node_id()
                    }
                    ExportDefaultDeclarationKindData::Expr(_) => node,
                }
            }
            _ => node,
        }
    }
}

struct TsModuleBlockPath<'a> {
    node: NodeId,
    parent: Option<&'a TsModuleBlockPath<'a>>,
}

// Match TS7 NodeHasName for module instantiation. Deliberately inspect only
// direct declaration names, not BoundNames or identifiers in type annotations.
fn ts_module_statement_has_name(tree: &Ast<'_>, node: NodeId, name: &[u8]) -> bool {
    let identifier = match tree.kind_data(node) {
        NodeKindData::Function(function) => function.id(tree),
        NodeKindData::TsFunction(function) => function.id(tree),
        NodeKindData::Class(class) => class.id(tree),
        NodeKindData::TsInterfaceDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsTypeAliasDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsEnumDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsImportEqualsDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsModuleDeclaration(declaration) => {
            return ts_module_declares_name(tree, declaration.id(tree), name);
        }
        NodeKindData::VariableDeclaration(declaration) => {
            return declaration.declarators(tree).iter().any(|declarator| {
                let declarator = tree.get_node_in_sub_range(declarator);
                match tree.kind_data(declarator.id(tree).node_id()) {
                    NodeKindData::BindingIdentifier(identifier) => {
                        tree.get_utf8(identifier.name(tree)).as_bytes() == name
                    }
                    _ => false,
                }
            });
        }
        _ => None,
    };
    identifier.is_some_and(|identifier| tree.get_utf8(identifier.name(tree)).as_bytes() == name)
}

fn ts_module_declares_name(tree: &Ast<'_>, name: TsModuleName, expected: &[u8]) -> bool {
    match tree.ts_module_name_data(name) {
        TsModuleNameData::BindingIdentifier(identifier) => {
            tree.get_utf8(identifier.name(tree)).as_bytes() == expected
        }
        TsModuleNameData::TsQualifiedName(qualified) => {
            let mut left = qualified.left(tree);
            loop {
                match tree.ts_qualified_name_left_data(left) {
                    TsQualifiedNameLeftData::BindingIdentifier(identifier) => {
                        return tree.get_utf8(identifier.name(tree)).as_bytes() == expected;
                    }
                    TsQualifiedNameLeftData::TsQualifiedName(qualified) => {
                        left = qualified.left(tree);
                    }
                    TsQualifiedNameLeftData::IdentifierReference(_)
                    | TsQualifiedNameLeftData::IdentifierName(_)
                    | TsQualifiedNameLeftData::ThisExpression(_) => return false,
                }
            }
        }
        TsModuleNameData::StringLiteral(_) => false,
    }
}
