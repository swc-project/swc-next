use std::collections::BTreeMap;

use serde::Serialize;
use swc_next_allocator::Idx;
use swc_next_ecma_ast::{
    Ast, NodeId, NodeKindData, ReferenceId, ScopeId, SymbolId, TsImportEqualsDeclaration,
    TsModuleReferenceData, Visit, VisitWith,
};
use swc_next_ecma_semantic2::{Semantic, SymbolFlags, TsImportAlias};

pub(crate) fn span(ast: &Ast<'_>, node: NodeId) -> String {
    let span = ast.span(node);
    format!("{}..{}", span.start, span.end)
}

/// Scope IDs refer to this table instead of repeating scope details at every use.
#[derive(Default, Serialize)]
#[serde(transparent)]
pub(crate) struct TestScopes(BTreeMap<usize, TestScope>);

impl TestScopes {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn insert(&mut self, semantic: &Semantic<'_>, id: ScopeId) -> usize {
        self.0.entry(id.index()).or_insert_with(|| {
            let scope = semantic.scope(id);
            let mut flags = Vec::new();
            if scope.flags.strict {
                flags.push("Strict");
            }
            if scope.flags.arguments_barrier {
                flags.push("ArgumentsBarrier");
            }
            TestScope {
                kind: format!("{:?}", scope.kind),
                parent: scope.parent.map(Idx::index),
                flags: flags.join(" | "),
                hoist_target: (scope.hoist_target != id).then_some(scope.hoist_target.index()),
                function_body: scope.function_body.map(Idx::index),
                container_owner: semantic.ts_container_owner(id).map(Idx::index),
            }
        });
        id.index()
    }
}

/// Omitted flags are empty, omitted parents/bodies are absent, and an omitted
/// hoist target is the scope itself. Non-default results remain visible.
#[derive(Serialize)]
struct TestScope {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<usize>,
    #[serde(skip_serializing_if = "String::is_empty")]
    flags: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist_target: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_body: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_owner: Option<usize>,
}

#[derive(Serialize)]
pub(crate) enum TestAlias {
    Target(usize),
    Unknown(Vec<&'static str>),
}

impl From<TsImportAlias> for TestAlias {
    fn from(alias: TsImportAlias) -> Self {
        match alias {
            TsImportAlias::Target(id) => Self::Target(id.index()),
            TsImportAlias::Unknown(spaces) => {
                Self::Unknown(spaces.iter_names().map(|(name, _)| name).collect())
            }
        }
    }
}

pub(crate) fn flags(flags: SymbolFlags) -> String {
    if flags.is_empty() {
        return "0x0".to_owned();
    }
    flags
        .iter_names()
        .map(|(name, _)| name)
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Syntax-to-reference associations for import-equals initializers.
pub(crate) struct TestImportEquals<'a, 's> {
    ast: &'s Ast<'a>,
    semantic: &'s Semantic<'a>,
    pub(crate) entries: Vec<(Option<SymbolId>, Option<ReferenceId>)>,
}

impl<'a, 's> TestImportEquals<'a, 's> {
    pub(crate) fn new(ast: &'s Ast<'a>, semantic: &'s Semantic<'a>) -> Self {
        let mut imports = Self {
            ast,
            semantic,
            entries: Vec::new(),
        };
        ast.root_program().visit_with(&mut imports);
        imports
    }
}

impl<'a> Visit<'a> for TestImportEquals<'a, '_> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn visit_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        let ast = self.ast;
        let reference = match ast.ts_module_reference_data(node.module_reference(ast)) {
            TsModuleReferenceData::TsTypeName(name) => {
                let mut root = name.node_id();
                while let NodeKindData::TsQualifiedName(qualified) = ast.kind_data(root) {
                    root = qualified.left(ast).node_id();
                }
                self.semantic.reference_of(root)
            }
            TsModuleReferenceData::TsExternalModuleReference(_) => None,
        };
        self.entries
            .push((self.semantic.symbol_of(node.id(ast).node_id()), reference));
        node.visit_children_with(self);
    }
}
