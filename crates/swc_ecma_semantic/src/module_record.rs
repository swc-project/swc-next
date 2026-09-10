use std::vec::Vec as RawVec;

use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    Ast, AstWtf8, ExportAllDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKindData,
    ExportNamedDeclaration, ExportSpecifier, ImportOrExportKind, NodeId, NodeKindData, ReferenceId,
    SymbolId,
};

/// The ECMAScript export occurrences of a source text module, in source
/// order. TypeScript's CommonJS-style `export =` and UMD
/// `export as namespace` declarations are outside this record.
///
/// Each entry preserves both its syntactic origin and its semantic target
/// state. Local names that do not yet have a reference remain explicitly
/// unresolved instead of being conflated with external exports.
#[derive(Debug)]
pub struct ModuleRecord<'a> {
    exports: ArenaVec<'a, ExportEntry>,
}

impl ModuleRecord<'_> {
    pub(crate) fn replace_target(
        &mut self,
        statement: NodeId,
        new: NodeId,
        declaration: bool,
        reference: Option<ReferenceId>,
        symbol: Option<SymbolId>,
        declaration_symbol: Option<SymbolId>,
    ) {
        for entry in &mut self.exports {
            if entry.statement != statement {
                continue;
            }
            if matches!(entry.target, ExportTarget::Expression { .. }) || !declaration {
                entry.origin = ExportOrigin::DefaultExpression;
                entry.target = ExportTarget::Expression {
                    node: new,
                    reference,
                    symbol,
                };
            } else if let Some(symbol) = declaration_symbol {
                entry.target = ExportTarget::Local {
                    symbol,
                    reference: None,
                };
            } else {
                entry.target = ExportTarget::AnonymousDeclaration { node: new };
            }
        }
    }

    pub(crate) fn normalize_targets(
        &mut self,
        ast: &Ast<'_>,
        node_references: &[Option<ReferenceId>],
        references: &[crate::Reference],
    ) {
        for entry in &mut self.exports {
            if !matches!(entry.target, ExportTarget::Expression { .. }) {
                continue;
            }
            let NodeKindData::ExportDefaultDeclaration(declaration) =
                ast.kind_data(entry.statement)
            else {
                continue;
            };
            let node = declaration.declaration(ast).node_id();
            let reference = node_references[node.index()];
            let symbol = reference.and_then(|id| references[id.index()].symbol);
            entry.target = ExportTarget::Expression {
                node,
                reference,
                symbol,
            };
        }
    }

    pub(crate) fn refresh_references(&mut self, references: &[crate::Reference]) {
        for entry in &mut self.exports {
            match entry.target {
                ExportTarget::Local {
                    reference: Some(id),
                    ..
                }
                | ExportTarget::UnresolvedLocal {
                    reference: Some(id),
                    ..
                } => {
                    let reference = references[id.index()];
                    entry.target = if let Some(symbol) = reference.symbol {
                        ExportTarget::Local {
                            symbol,
                            reference: Some(id),
                        }
                    } else {
                        ExportTarget::UnresolvedLocal {
                            name: ModuleName {
                                name: reference.name.into(),
                                node: reference.node,
                            },
                            reference: Some(id),
                        }
                    };
                }
                ExportTarget::Expression {
                    reference: Some(id),
                    ref mut symbol,
                    ..
                } => {
                    *symbol = references[id.index()].symbol;
                }
                _ => {}
            }
        }
    }

    /// Every export occurrence in source order.
    pub fn exports(&self) -> &[ExportEntry] {
        &self.exports
    }
}

/// A module name together with the AST node that spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleName {
    pub name: AstWtf8,
    pub node: NodeId,
}

/// The name introduced into this module's export namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportName {
    Named(ModuleName),
    Default { node: NodeId },
}

/// The syntactic form that created an [`ExportEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportOrigin {
    DirectDeclaration,
    LocalSpecifier,
    ReExportSpecifier,
    NamespaceReExport,
    StarReExport,
    DefaultDeclaration,
    DefaultExpression,
}

/// The name requested from an external module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalExportName {
    Named(ModuleName),
    All,
    AllButDefault,
}

/// The target state of an export occurrence.
/// When a target carries a reference, its symbol is the nearest candidate.
/// Inspect [`crate::Reference::resolution`] and
/// [`crate::Semantic::reference_candidates`] for that reference's certainty
/// and additional possible targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportTarget {
    Local {
        symbol: SymbolId,
        reference: Option<ReferenceId>,
    },
    External {
        source: ModuleName,
        imported: ExternalExportName,
    },
    AnonymousDeclaration {
        node: NodeId,
    },
    Expression {
        node: NodeId,
        reference: Option<ReferenceId>,
        symbol: Option<SymbolId>,
    },
    UnresolvedLocal {
        name: ModuleName,
        reference: Option<ReferenceId>,
    },
}

/// One export occurrence and its finalized semantic target state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportEntry {
    pub node: NodeId,
    pub statement: NodeId,
    pub exported: Option<ExportName>,
    pub origin: ExportOrigin,
    pub target: ExportTarget,
    /// Whether a specifier or re-export explicitly restricts its target to
    /// type meanings. Direct declarations do not set this flag: their syntax
    /// determines the meanings they contribute. Consumers that need
    /// declaration-specific metadata can match [`ExportEntry::node`] against
    /// the target symbol's declarations.
    pub type_only: bool,
}

pub(crate) struct ModuleRecordBuilder<'a> {
    allocator: &'a Allocator,
    enabled: bool,
    exports: RawVec<RawExportEntry>,
}

impl<'a> ModuleRecordBuilder<'a> {
    pub fn new(allocator: &'a Allocator, enabled: bool) -> Self {
        Self {
            allocator,
            enabled,
            exports: RawVec::new(),
        }
    }

    pub fn add_direct_export(&mut self, tree: &Ast, binding: NodeId, statement: NodeId) {
        if !self.enabled {
            return;
        }
        let name = module_name(tree, binding);
        self.exports.push(RawExportEntry {
            node: binding,
            statement,
            exported: Some(ExportName::Named(name)),
            origin: ExportOrigin::DirectDeclaration,
            target: RawExportTarget::Direct { binding },
            type_only: false,
        });
    }

    pub fn add_specifier(
        &mut self,
        tree: &Ast,
        declaration: ExportNamedDeclaration,
        specifier: ExportSpecifier,
    ) {
        if !self.enabled {
            return;
        }
        let type_only = declaration.export_kind(tree) == ImportOrExportKind::Type
            || specifier.export_kind(tree) == ImportOrExportKind::Type;
        let exported = ExportName::Named(module_name(tree, specifier.exported(tree).node_id()));
        let (origin, target) = if let Some(source) = declaration.source(tree) {
            (
                ExportOrigin::ReExportSpecifier,
                RawExportTarget::External {
                    source: string_name(tree, source.node_id()),
                    imported: ExternalExportName::Named(module_name(
                        tree,
                        specifier.local(tree).node_id(),
                    )),
                },
            )
        } else {
            (
                ExportOrigin::LocalSpecifier,
                RawExportTarget::Local {
                    name: module_name(tree, specifier.local(tree).node_id()),
                },
            )
        };
        self.exports.push(RawExportEntry {
            node: specifier.node_id(),
            statement: declaration.node_id(),
            exported: Some(exported),
            origin,
            target,
            type_only,
        });
    }

    pub fn add_export_all(&mut self, tree: &Ast, declaration: ExportAllDeclaration) {
        if !self.enabled {
            return;
        }
        let source = string_name(tree, declaration.source(tree).node_id());
        let type_only = declaration.export_kind(tree) == ImportOrExportKind::Type;
        let (exported, origin, target) = if let Some(exported) = declaration.exported(tree) {
            (
                Some(ExportName::Named(module_name(tree, exported.node_id()))),
                ExportOrigin::NamespaceReExport,
                RawExportTarget::External {
                    source,
                    imported: ExternalExportName::All,
                },
            )
        } else {
            (
                None,
                ExportOrigin::StarReExport,
                RawExportTarget::External {
                    source,
                    imported: ExternalExportName::AllButDefault,
                },
            )
        };
        self.exports.push(RawExportEntry {
            node: declaration.node_id(),
            statement: declaration.node_id(),
            exported,
            origin,
            target,
            type_only,
        });
    }

    pub fn add_default_export(&mut self, tree: &Ast, declaration: ExportDefaultDeclaration) {
        if !self.enabled {
            return;
        }
        let value = declaration.declaration(tree);
        let expression = value
            .as_expr(tree)
            .map(|expression| crate::ecmascript::unwrap_parentheses(tree, expression.node_id()));
        let binding = match tree.export_default_declaration_kind_data(value) {
            ExportDefaultDeclarationKindData::Function(function) => {
                function.id(tree).map(|id| id.node_id())
            }
            ExportDefaultDeclarationKindData::Class(class) => class.id(tree).map(|id| id.node_id()),
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(ts_interface_declaration) => {
                Some(ts_interface_declaration.id(tree).node_id())
            }
            ExportDefaultDeclarationKindData::TsFunction(function) => {
                function.id(tree).map(|id| id.node_id())
            }
            ExportDefaultDeclarationKindData::Expr(_) => None,
        };
        self.exports.push(RawExportEntry {
            node: declaration.node_id(),
            statement: declaration.node_id(),
            exported: Some(ExportName::Default {
                node: declaration.node_id(),
            }),
            origin: if expression.is_some() {
                ExportOrigin::DefaultExpression
            } else {
                ExportOrigin::DefaultDeclaration
            },
            target: RawExportTarget::Default {
                value: value.node_id(),
                binding,
                expression,
            },
            type_only: false,
        });
    }

    pub fn finalize(
        self,
        node_symbols: &[Option<SymbolId>],
        references: &[crate::Reference],
        node_references: &[Option<ReferenceId>],
    ) -> ModuleRecord<'a> {
        let mut exports = ArenaVec::with_capacity_in(self.exports.len(), self.allocator);
        for raw in self.exports {
            let target = match raw.target {
                RawExportTarget::Direct { binding } => {
                    let symbol =
                        node_symbols[binding.index()].expect("direct export binding has a symbol");
                    ExportTarget::Local {
                        symbol,
                        reference: None,
                    }
                }
                RawExportTarget::Local { name } => {
                    let reference = node_references[name.node.index()];
                    let symbol = reference.and_then(|id| references[id.index()].symbol);
                    if let Some(symbol) = symbol {
                        ExportTarget::Local { symbol, reference }
                    } else {
                        ExportTarget::UnresolvedLocal { name, reference }
                    }
                }
                RawExportTarget::External { source, imported } => {
                    ExportTarget::External { source, imported }
                }
                RawExportTarget::Default {
                    value,
                    binding,
                    expression,
                } => {
                    if let Some(binding) = binding {
                        let symbol = node_symbols[binding.index()]
                            .expect("named default export has a symbol");
                        ExportTarget::Local {
                            symbol,
                            reference: None,
                        }
                    } else if let Some(expression) = expression {
                        let reference = node_references[expression.index()];
                        let symbol = reference.and_then(|id| references[id.index()].symbol);
                        ExportTarget::Expression {
                            node: value,
                            reference,
                            symbol,
                        }
                    } else {
                        ExportTarget::AnonymousDeclaration { node: value }
                    }
                }
            };
            exports.push(ExportEntry {
                node: raw.node,
                statement: raw.statement,
                exported: raw.exported,
                origin: raw.origin,
                target,
                type_only: raw.type_only,
            });
        }
        ModuleRecord { exports }
    }
}

#[derive(Clone, Copy)]
struct RawExportEntry {
    node: NodeId,
    statement: NodeId,
    exported: Option<ExportName>,
    origin: ExportOrigin,
    target: RawExportTarget,
    type_only: bool,
}

#[derive(Clone, Copy)]
enum RawExportTarget {
    Direct {
        binding: NodeId,
    },
    Local {
        name: ModuleName,
    },
    External {
        source: ModuleName,
        imported: ExternalExportName,
    },
    Default {
        value: NodeId,
        binding: Option<NodeId>,
        expression: Option<NodeId>,
    },
}

fn module_name(tree: &Ast, node: NodeId) -> ModuleName {
    let name = match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) => identifier.name(tree).into(),
        NodeKindData::IdentifierReference(identifier) => identifier.name(tree).into(),
        NodeKindData::BindingIdentifier(identifier) => identifier.name(tree).into(),
        NodeKindData::StringLiteral(string) => string.value(tree),
        _ => unreachable!("module export name must be an identifier or string literal"),
    };
    ModuleName { name, node }
}

fn string_name(tree: &Ast, node: NodeId) -> ModuleName {
    let NodeKindData::StringLiteral(string) = tree.kind_data(node) else {
        unreachable!("module source must be a string literal")
    };
    ModuleName {
        name: string.value(tree),
        node,
    }
}
