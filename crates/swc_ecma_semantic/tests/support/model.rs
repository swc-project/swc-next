use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use swc_next_ecma_ast::{
    Ast, ExportSpecifier, ModuleExportNameData, NodeId, ReferenceId, ScopeId, SymbolId,
    TsEnumDeclaration, TsGlobalDeclaration, TsImportEqualsDeclaration, TsModuleBodyData,
    TsModuleDeclaration, TsModuleNameData, Visit, VisitWith,
};
use swc_next_ecma_semantic::{NameResolution, Semantic, SymbolFlags};

pub(crate) fn span(ast: &Ast<'_>, node: NodeId) -> String {
    let span = ast.span(node);
    format!("{}..{}", span.start, span.end)
}

#[derive(Serialize)]
pub(crate) enum TestResolution {
    Unresolved,
    Resolved(usize),
    Uncertain { candidate: usize },
}

impl From<NameResolution> for TestResolution {
    fn from(resolution: NameResolution) -> Self {
        let (snapshot, symbol, candidate) = match resolution {
            NameResolution::Unresolved => (Self::Unresolved, None, None),
            NameResolution::Resolved(id) => (Self::Resolved(id.index()), Some(id), Some(id)),
            NameResolution::Uncertain { candidate } => (
                Self::Uncertain {
                    candidate: candidate.index(),
                },
                None,
                Some(candidate),
            ),
        };
        assert_eq!(resolution.symbol(), symbol);
        assert_eq!(resolution.candidate(), candidate);
        snapshot
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
    pub(crate) container_owners: BTreeMap<ScopeId, SymbolId>,
    pub(crate) export_names: BTreeMap<SymbolId, BTreeSet<String>>,
    pub(crate) initializers: BTreeMap<SymbolId, Vec<Option<ReferenceId>>>,
    pub(crate) import_references: BTreeSet<ReferenceId>,
}

impl<'a, 's> TestNodes<'a, 's> {
    pub(crate) fn new(ast: &'s Ast<'a>, semantic: &'s Semantic<'a>) -> Self {
        let mut nodes = Self {
            ast,
            semantic,
            owner: semantic.ts_source_module_symbol(),
            container_owners: BTreeMap::new(),
            export_names: BTreeMap::new(),
            initializers: BTreeMap::new(),
            import_references: BTreeSet::new(),
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
        if let Some(owner) = owner
            && let Some(body) = node.body(self.ast)
            && let TsModuleBodyData::TsModuleBlock(body) = self.ast.ts_module_body_data(body)
            && let Some(scope) = self.semantic.scope_of(body.node_id())
        {
            self.container_owners.insert(scope, owner);
        }
        let previous = std::mem::replace(&mut self.owner, owner);
        node.visit_children_with(self);
        self.owner = previous;
    }

    fn visit_ts_enum_declaration(&mut self, node: TsEnumDeclaration) {
        let ast = self.ast;
        if let Some(owner) = self.semantic.symbol_of(node.id(ast).node_id())
            && let Some(scope) = self.semantic.scope_of(node.body(ast).node_id())
        {
            self.container_owners.insert(scope, owner);
        }
        node.visit_children_with(self);
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
        let reference = self.semantic.import_equals_reference(self.ast, node);
        if let Some(symbol) = self.semantic.symbol_of(node.id(self.ast).node_id()) {
            self.initializers.entry(symbol).or_default().push(reference);
        }
        self.import_references.extend(reference);
        node.visit_children_with(self);
    }
}
