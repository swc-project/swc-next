use std::vec::Vec as RawVec;

use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    Ast, AstUtf8, AstWtf8, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationKindData, ExportNamedDeclaration, ExportSpecifier, ImportOrExportKind,
    NodeId, NodeKindData, ReferenceId, SymbolId,
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
    /// Every export occurrence in source order.
    pub fn exports(&self) -> &[ExportEntry] {
        &self.exports
    }
}

/// A module name together with the AST node that spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleName {
    pub name: ModuleNameValue,
    pub node: NodeId,
}

/// String representation of a module name without assuming UTF-8 handles are flat WTF-8 ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleNameValue {
    Utf8(AstUtf8),
    Wtf8(AstWtf8),
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

/// One export occurrence and its finalized semantic target.
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
        let expression = value.is_expr(tree);
        let binding = match tree.export_default_declaration_kind_data(value) {
            ExportDefaultDeclarationKindData::Function(function) => {
                function.id(tree).map(|id| id.node_id())
            }
            ExportDefaultDeclarationKindData::Class(class) => class.id(tree).map(|id| id.node_id()),
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(ts_interface_declaration) => {
                Some(ts_interface_declaration.id(tree).node_id())
            }
            ExportDefaultDeclarationKindData::TsFunction(_)
            | ExportDefaultDeclarationKindData::Expr(_) => None,
        };
        self.exports.push(RawExportEntry {
            node: declaration.node_id(),
            statement: declaration.node_id(),
            exported: Some(ExportName::Default {
                node: declaration.node_id(),
            }),
            origin: if expression {
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
                    } else if expression {
                        let reference = node_references[value.index()];
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
        expression: bool,
    },
}

fn module_name(tree: &Ast, node: NodeId) -> ModuleName {
    let name = match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) => ModuleNameValue::Utf8(identifier.name(tree)),
        NodeKindData::IdentifierReference(identifier) => {
            ModuleNameValue::Utf8(identifier.name(tree))
        }
        NodeKindData::BindingIdentifier(identifier) => ModuleNameValue::Utf8(identifier.name(tree)),
        NodeKindData::StringLiteral(string) => ModuleNameValue::Wtf8(string.value(tree)),
        _ => unreachable!("module export name must be an identifier or string literal"),
    };
    ModuleName { name, node }
}

fn string_name(tree: &Ast, node: NodeId) -> ModuleName {
    let NodeKindData::StringLiteral(string) = tree.kind_data(node) else {
        unreachable!("module source must be a string literal")
    };
    ModuleName {
        name: ModuleNameValue::Wtf8(string.value(tree)),
        node,
    }
}
