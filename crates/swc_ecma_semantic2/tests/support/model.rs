use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use swc_next_allocator::Idx;
use swc_next_ecma_ast::{
    Ast, ExportSpecifier, ModuleExportNameData, NodeId, ReferenceId, ScopeId, SymbolId,
    TsGlobalDeclaration, TsImportEqualsDeclaration, TsModuleDeclaration, TsModuleNameData, Visit,
    VisitWith,
};
use swc_next_ecma_semantic2::{NameResolution, Semantic, SymbolFlags};

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
}

#[derive(Serialize)]
pub(crate) enum TestResolution {
    Unresolved,
    Resolved(usize),
    Uncertain { candidate: usize },
}

impl From<NameResolution> for TestResolution {
    fn from(resolution: NameResolution) -> Self {
        match resolution {
            NameResolution::Unresolved => Self::Unresolved,
            NameResolution::Resolved(id) => Self::Resolved(id.index()),
            NameResolution::Uncertain { candidate } => Self::Uncertain {
                candidate: candidate.index(),
            },
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

/// Public export names are scoped to their source module or merged namespace.
/// Both local and exported names are retained so private/type-only negative
/// value-target queries remain visible alongside exported values.
pub(crate) struct TestNodes<'a, 's> {
    ast: &'s Ast<'a>,
    semantic: &'s Semantic<'a>,
    owner: Option<SymbolId>,
    pub(crate) export_names: BTreeMap<SymbolId, BTreeSet<String>>,
    pub(crate) imports: Vec<(Option<SymbolId>, Option<ReferenceId>)>,
}

impl<'a, 's> TestNodes<'a, 's> {
    pub(crate) fn new(ast: &'s Ast<'a>, semantic: &'s Semantic<'a>) -> Self {
        let mut nodes = Self {
            ast,
            semantic,
            owner: semantic.ts_source_module_symbol(),
            export_names: BTreeMap::new(),
            imports: Vec::new(),
        };
        ast.root_program().visit_with(&mut nodes);
        nodes
    }
}

impl<'a> Visit<'a> for TestNodes<'a, '_> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn visit_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        let owner = match self.ast.ts_module_name_data(node.id(self.ast)) {
            TsModuleNameData::BindingIdentifier(name) => self.semantic.symbol_of(name.node_id()),
            TsModuleNameData::TsQualifiedName(name) => {
                self.semantic.symbol_of(name.right(self.ast).node_id())
            }
            TsModuleNameData::StringLiteral(_) => None,
        };
        let previous = std::mem::replace(&mut self.owner, owner);
        node.visit_children_with(self);
        self.owner = previous;
    }

    fn visit_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        let previous = self.owner.take();
        node.visit_children_with(self);
        self.owner = previous;
    }

    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        let Some(owner) = self.owner else { return };
        for name in [node.local(self.ast), node.exported(self.ast)] {
            let name = match self.ast.module_export_name_data(name) {
                ModuleExportNameData::IdentifierName(name) => {
                    self.ast.get_utf8(name.name(self.ast)).to_owned()
                }
                ModuleExportNameData::StringLiteral(name) => {
                    String::from_utf8_lossy(self.ast.get_wtf8(name.value(self.ast)).as_bytes())
                        .into_owned()
                }
            };
            self.export_names.entry(owner).or_default().insert(name);
        }
    }

    fn visit_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        self.imports.push((
            self.semantic.symbol_of(node.id(self.ast).node_id()),
            self.semantic.import_equals_reference(self.ast, node),
        ));
        node.visit_children_with(self);
    }
}
