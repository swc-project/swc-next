//! Module import and export records.

use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    Ast, AstNode, ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration,
    ExportSpecifier, ImportDeclaration, ImportOrExportKind, ImportPhase, NodeId, NodeKind, ScopeId,
    String as AstString, SymbolId, TsExportAssignment, TsImportEqualsDeclaration,
    TsNamespaceExportDeclaration,
};

use crate::{Semantic, scope::scope_id};

/// One dependency edge of the module, static or dynamic.
#[derive(Debug, Clone, Copy)]
pub struct Import {
    /// The import form.
    pub kind: ImportKind,
    /// Imported name of a `.named` record, "default" for default imports.
    pub name: AstString,
    /// Local binding, `None` when the form binds nothing.
    pub symbol: Option<SymbolId>,
    /// Decoded module specifier.
    pub specifier: AstString,
    /// True for type only imports.
    pub type_only: bool,
    /// Phase modifier, `None` for regular imports.
    pub phase: Option<ImportPhase>,
    /// The smallest node identifying the record.
    pub node: NodeId,
}

/// The import form.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImportKind {
    /// `import d from "m"` and `import { a as b } from "m"`
    Named,
    /// `import * as ns from "m"`
    Namespace,
    /// `import "m"`
    SideEffect,
    /// `import x = require("m")`
    ImportEquals,
    /// `import("m")` at any depth
    Dynamic,
    /// `require("m")` at any depth, only when `require` is a free name
    Require,
}

/// One exported name of the module.
#[derive(Debug, Clone, Copy)]
pub struct Export {
    /// The export form.
    pub kind: ExportKind,
    /// Exported name, the global name for `.global`, empty otherwise.
    pub name: AstString,
    /// Backing local binding, `None` when nothing local backs the export.
    pub symbol: Option<SymbolId>,
    /// Name taken from the source module by a `.re_export`.
    pub from_name: AstString,
    /// Decoded specifier of the `from` clause, empty for local exports.
    pub specifier: AstString,
    /// True for type only exports.
    pub type_only: bool,
    /// The smallest node identifying the record.
    pub node: NodeId,
}

/// The export form.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExportKind {
    /// `export { x }` and exported declarations, `export default` included
    Named,
    /// `export { a as b } from "m"`
    ReExport,
    /// `export * as ns from "m"`
    Namespace,
    /// `export * from "m"`
    Star,
    /// `export = expr`
    Equals,
    /// `export as namespace N`
    Global,
}

/// CommonJS classification signals, from free value references at any depth.
#[derive(Debug, Default, Clone, Copy)]
pub struct Flags {
    /// A free `require` reference appears.
    pub uses_require: bool,
    /// A free `module` reference appears.
    pub uses_module: bool,
    /// A free `exports` reference appears.
    pub uses_exports: bool,
    /// `import.meta` appears.
    pub uses_import_meta: bool,
    pub _reserved: u8,
}

/// The records of one module, allocated in the tree arena.
#[derive(Debug, Clone, Copy)]
pub struct Records<'a> {
    pub imports: &'a [Import],
    pub exports: &'a [Export],
    pub flags: Flags,
}

impl Records<'static> {
    pub const EMPTY: Self = Self {
        imports: &[],
        exports: &[],
        flags: Flags {
            uses_require: false,
            uses_module: false,
            uses_exports: false,
            uses_import_meta: false,
            _reserved: 0,
        },
    };
}

/// Collects the records of an analyzed tree. Static records come first
/// in source order, then dynamic and require records in source order.
pub fn collect<'a>(tree: &mut Ast<'a>, semantic: &Semantic<'_>) -> Records<'a> {
    assert!(tree.node_count() > 0);
    let default_name = tree.add_string(b"default");
    let top_scope = if tree.is_module() {
        scope_id(1)
    } else {
        scope_id(0)
    };
    let mut collector = Collector {
        tree,
        semantic,
        allocator: tree.allocator(),
        default_name,
        top_scope,
        imports: ArenaVec::new_in(tree.allocator()),
        exports: ArenaVec::new_in(tree.allocator()),
        flags: Flags::default(),
    };

    let program = unsafe { typed::<swc_next_ecma_ast::Program>(NodeId::from_usize_unchecked(0)) };
    for statement in program.body(collector.tree()).iter() {
        let statement = collector.tree().get_node_in_sub_range(statement).node_id();
        collector.statement(statement);
    }
    collector.sweep();

    Records {
        imports: collector.imports.into_bump_slice(),
        exports: collector.exports.into_bump_slice(),
        flags: collector.flags,
    }
}

struct Collector<'a, 'semantic, 'data> {
    tree: *mut Ast<'a>,
    semantic: &'semantic Semantic<'data>,
    allocator: &'a Allocator,
    default_name: AstString,
    top_scope: ScopeId,
    imports: ArenaVec<'a, Import>,
    exports: ArenaVec<'a, Export>,
    flags: Flags,
}

impl<'a> Collector<'a, '_, '_> {
    #[inline]
    fn tree(&self) -> &Ast<'a> {
        unsafe { &*self.tree }
    }

    fn add_import(&mut self, record: Import) {
        let _allocator = self.allocator;
        self.imports.push(record);
    }

    fn add_export(&mut self, record: Export) {
        let _allocator = self.allocator;
        self.exports.push(record);
    }

    fn statement(&mut self, node: NodeId) {
        match self.tree().node_kind(node) {
            NodeKind::ImportDeclaration => self.import_declaration(unsafe { typed(node) }, node),
            NodeKind::TsImportEqualsDeclaration => self.import_equals(unsafe { typed(node) }, node),
            NodeKind::ExportNamedDeclaration => self.export_named(unsafe { typed(node) }),
            NodeKind::ExportDefaultDeclaration => self.export_default(unsafe { typed(node) }, node),
            NodeKind::ExportAllDeclaration => self.export_all(unsafe { typed(node) }, node),
            NodeKind::TsExportAssignment => self.export_assignment(unsafe { typed(node) }, node),
            NodeKind::TsNamespaceExportDeclaration => {
                self.namespace_export(unsafe { typed(node) }, node)
            }
            _ => {}
        }
    }

    fn import_declaration(&mut self, declaration: ImportDeclaration, node: NodeId) {
        let tree = unsafe { &*self.tree };
        let specifier = literal_string(tree, declaration.source(tree).node_id());
        let type_only = declaration.import_kind(tree) == ImportOrExportKind::Type;
        let specifiers = declaration.specifiers(tree);
        if specifiers.is_empty() {
            self.add_import(Import {
                kind: ImportKind::SideEffect,
                name: AstString::EMPTY,
                symbol: None,
                specifier,
                type_only,
                phase: declaration.phase(tree),
                node,
            });
            return;
        }

        for specifier_node in specifiers.iter() {
            let specifier_node = tree.get_node_in_sub_range(specifier_node).node_id();
            match tree.node_kind(specifier_node) {
                NodeKind::ImportSpecifier => {
                    let item =
                        unsafe { typed::<swc_next_ecma_ast::ImportSpecifier>(specifier_node) };
                    self.add_import(Import {
                        kind: ImportKind::Named,
                        name: self.string_of(item.imported(tree).node_id()),
                        symbol: self.binding_symbol(item.local(tree).node_id()),
                        specifier,
                        type_only: type_only || item.import_kind(tree) == ImportOrExportKind::Type,
                        phase: declaration.phase(tree),
                        node: specifier_node,
                    });
                }
                NodeKind::ImportDefaultSpecifier => {
                    let item = unsafe {
                        typed::<swc_next_ecma_ast::ImportDefaultSpecifier>(specifier_node)
                    };
                    self.add_import(Import {
                        kind: ImportKind::Named,
                        name: self.default_name,
                        symbol: self.binding_symbol(item.local(tree).node_id()),
                        specifier,
                        type_only,
                        phase: declaration.phase(tree),
                        node: specifier_node,
                    });
                }
                NodeKind::ImportNamespaceSpecifier => {
                    let item = unsafe {
                        typed::<swc_next_ecma_ast::ImportNamespaceSpecifier>(specifier_node)
                    };
                    self.add_import(Import {
                        kind: ImportKind::Namespace,
                        name: AstString::EMPTY,
                        symbol: self.binding_symbol(item.local(tree).node_id()),
                        specifier,
                        type_only,
                        phase: declaration.phase(tree),
                        node: specifier_node,
                    });
                }
                _ => {}
            }
        }
    }

    /// Only the external module reference form is a dependency edge.
    /// The qualified name form aliases a namespace, not a module.
    fn import_equals(&mut self, declaration: TsImportEqualsDeclaration, node: NodeId) {
        let tree = unsafe { &*self.tree };
        let reference = declaration.module_reference(tree).node_id();
        if tree.node_kind(reference) != NodeKind::TsExternalModuleReference {
            return;
        }
        let reference = unsafe { typed::<swc_next_ecma_ast::TsExternalModuleReference>(reference) };
        self.add_import(Import {
            kind: ImportKind::ImportEquals,
            name: AstString::EMPTY,
            symbol: self.binding_symbol(declaration.id(tree).node_id()),
            specifier: literal_string(tree, reference.expression(tree).node_id()),
            type_only: declaration.import_kind(tree) == ImportOrExportKind::Type,
            phase: None,
            node,
        });
    }

    fn export_named(&mut self, declaration: ExportNamedDeclaration) {
        let tree = unsafe { &*self.tree };
        if let Some(inner) = declaration.declaration(tree) {
            self.declaration_names(
                inner.node_id(),
                declaration.export_kind(tree) == ImportOrExportKind::Type,
            );
            return;
        }

        let source = declaration.source(tree);
        let has_source = source.is_some();
        let specifier = source
            .map(|source| literal_string(tree, source.node_id()))
            .unwrap_or(AstString::EMPTY);
        for specifier_node in declaration.specifiers(tree).iter() {
            let specifier_node = tree.get_node_in_sub_range(specifier_node).node_id();
            if tree.node_kind(specifier_node) != NodeKind::ExportSpecifier {
                continue;
            }
            let export = unsafe { typed::<ExportSpecifier>(specifier_node) };
            let type_only = declaration.export_kind(tree) == ImportOrExportKind::Type
                || export.export_kind(tree) == ImportOrExportKind::Type;
            if has_source {
                self.add_export(Export {
                    kind: ExportKind::ReExport,
                    name: self.string_of(export.exported(tree).node_id()),
                    symbol: None,
                    from_name: self.string_of(export.local(tree).node_id()),
                    specifier,
                    type_only,
                    node: specifier_node,
                });
            } else {
                let local = self.string_of(export.local(tree).node_id());
                self.add_export(Export {
                    kind: ExportKind::Named,
                    name: self.string_of(export.exported(tree).node_id()),
                    symbol: self.module_binding(local),
                    from_name: AstString::EMPTY,
                    specifier: AstString::EMPTY,
                    type_only,
                    node: specifier_node,
                });
            }
        }
    }

    /// Every name bound by an exported declaration is exported under
    /// its own name.
    fn declaration_names(&mut self, declaration: NodeId, type_only: bool) {
        let tree = unsafe { &*self.tree };
        match tree.node_kind(declaration) {
            NodeKind::VariableDeclaration => {
                let declaration =
                    unsafe { typed::<swc_next_ecma_ast::VariableDeclaration>(declaration) };
                for declarator in declaration.declarators(tree).iter() {
                    let declarator = tree.get_node_in_sub_range(declarator);
                    self.pattern_names(declarator.id(tree).node_id(), type_only, 0);
                }
            }
            NodeKind::Function => {
                let function = unsafe { typed::<swc_next_ecma_ast::Function>(declaration) };
                if let Some(id) = function.id(tree) {
                    self.export_local(id.node_id(), type_only);
                }
            }
            NodeKind::TsFunction => {
                let function = unsafe { typed::<swc_next_ecma_ast::TsFunction>(declaration) };
                if let Some(id) = function.id(tree) {
                    self.export_local(id.node_id(), type_only);
                }
            }
            NodeKind::Class => {
                let class = unsafe { typed::<swc_next_ecma_ast::Class>(declaration) };
                if let Some(id) = class.id(tree) {
                    self.export_local(id.node_id(), type_only);
                }
            }
            NodeKind::TsTypeAliasDeclaration => {
                let declaration =
                    unsafe { typed::<swc_next_ecma_ast::TsTypeAliasDeclaration>(declaration) };
                self.export_local(declaration.id(tree).node_id(), true);
            }
            NodeKind::TsInterfaceDeclaration => {
                let declaration =
                    unsafe { typed::<swc_next_ecma_ast::TsInterfaceDeclaration>(declaration) };
                self.export_local(declaration.id(tree).node_id(), true);
            }
            NodeKind::TsEnumDeclaration => {
                let declaration =
                    unsafe { typed::<swc_next_ecma_ast::TsEnumDeclaration>(declaration) };
                self.export_local(declaration.id(tree).node_id(), type_only);
            }
            NodeKind::TsModuleDeclaration => {
                let declaration =
                    unsafe { typed::<swc_next_ecma_ast::TsModuleDeclaration>(declaration) };
                let id = declaration.id(tree).node_id();
                if tree.node_kind(id) == NodeKind::BindingIdentifier {
                    self.export_local(id, type_only);
                }
            }
            _ => {}
        }
    }

    /// Exports every identifier bound by a binding pattern.
    fn pattern_names(&mut self, pattern: NodeId, type_only: bool, depth: u32) {
        assert!(depth < 256);
        let tree = unsafe { &*self.tree };
        match tree.node_kind(pattern) {
            NodeKind::BindingIdentifier => self.export_local(pattern, type_only),
            NodeKind::AssignmentPattern => {
                let pattern = unsafe { typed::<swc_next_ecma_ast::AssignmentPattern>(pattern) };
                self.pattern_names(pattern.left(tree).node_id(), type_only, depth + 1);
            }
            NodeKind::BindingRestElement => {
                let rest = unsafe { typed::<swc_next_ecma_ast::BindingRestElement>(pattern) };
                self.pattern_names(rest.argument(tree).node_id(), type_only, depth + 1);
            }
            NodeKind::ArrayPattern => {
                let array = unsafe { typed::<swc_next_ecma_ast::ArrayPattern>(pattern) };
                for element in array.elements(tree).iter() {
                    if let Some(element) = tree.get_node_in_sub_range(element) {
                        self.pattern_names(element.node_id(), type_only, depth + 1);
                    }
                }
                if let Some(rest) = array.rest(tree) {
                    self.pattern_names(rest.node_id(), type_only, depth + 1);
                }
            }
            NodeKind::ObjectPattern => {
                let object = unsafe { typed::<swc_next_ecma_ast::ObjectPattern>(pattern) };
                for property in object.properties(tree).iter() {
                    let property = tree.get_node_in_sub_range(property);
                    self.pattern_names(property.value(tree).node_id(), type_only, depth + 1);
                }
                if let Some(rest) = object.rest(tree) {
                    self.pattern_names(rest.node_id(), type_only, depth + 1);
                }
            }
            _ => {}
        }
    }

    fn export_local(&mut self, binding: NodeId, type_only: bool) {
        let tree = unsafe { &*self.tree };
        let identifier = unsafe { typed::<swc_next_ecma_ast::BindingIdentifier>(binding) };
        let name = identifier.name(tree);
        self.add_export(Export {
            kind: ExportKind::Named,
            name,
            symbol: self.module_binding(name),
            from_name: AstString::EMPTY,
            specifier: AstString::EMPTY,
            type_only,
            node: binding,
        });
    }

    /// Only named declarations and identifiers bind a symbol.
    fn export_default(&mut self, declaration: ExportDefaultDeclaration, node: NodeId) {
        let tree = unsafe { &*self.tree };
        let inner = declaration.declaration(tree).node_id();
        let symbol = match tree.node_kind(inner) {
            NodeKind::Function => unsafe { typed::<swc_next_ecma_ast::Function>(inner) }
                .id(tree)
                .and_then(|id| self.binding_symbol(id.node_id())),
            NodeKind::TsFunction => unsafe { typed::<swc_next_ecma_ast::TsFunction>(inner) }
                .id(tree)
                .and_then(|id| self.binding_symbol(id.node_id())),
            NodeKind::Class => unsafe { typed::<swc_next_ecma_ast::Class>(inner) }
                .id(tree)
                .and_then(|id| self.binding_symbol(id.node_id())),
            NodeKind::IdentifierReference => {
                let identifier = unsafe { typed::<swc_next_ecma_ast::IdentifierReference>(inner) };
                self.module_binding(identifier.name(tree))
            }
            _ => None,
        };
        self.add_export(Export {
            kind: ExportKind::Named,
            name: self.default_name,
            symbol,
            from_name: AstString::EMPTY,
            specifier: AstString::EMPTY,
            type_only: false,
            node,
        });
    }

    fn export_all(&mut self, declaration: ExportAllDeclaration, node: NodeId) {
        let tree = unsafe { &*self.tree };
        let exported = declaration.exported(tree);
        self.add_export(Export {
            kind: if exported.is_some() {
                ExportKind::Namespace
            } else {
                ExportKind::Star
            },
            name: exported
                .map(|name| self.string_of(name.node_id()))
                .unwrap_or(AstString::EMPTY),
            symbol: None,
            from_name: AstString::EMPTY,
            specifier: literal_string(tree, declaration.source(tree).node_id()),
            type_only: declaration.export_kind(tree) == ImportOrExportKind::Type,
            node,
        });
    }

    fn export_assignment(&mut self, declaration: TsExportAssignment, node: NodeId) {
        let tree = unsafe { &*self.tree };
        let expression = declaration.expression(tree).node_id();
        let symbol = if tree.node_kind(expression) == NodeKind::IdentifierReference {
            let identifier = unsafe { typed::<swc_next_ecma_ast::IdentifierReference>(expression) };
            self.module_binding(identifier.name(tree))
        } else {
            None
        };
        self.add_export(Export {
            kind: ExportKind::Equals,
            name: AstString::EMPTY,
            symbol,
            from_name: AstString::EMPTY,
            specifier: AstString::EMPTY,
            type_only: false,
            node,
        });
    }

    fn namespace_export(&mut self, declaration: TsNamespaceExportDeclaration, node: NodeId) {
        self.add_export(Export {
            kind: ExportKind::Global,
            name: declaration.id(self.tree()).name(self.tree()),
            symbol: None,
            from_name: AstString::EMPTY,
            specifier: AstString::EMPTY,
            type_only: false,
            node,
        });
    }

    /// Collects the records and flags that can appear at any depth.
    fn sweep(&mut self) {
        for index in 0..self.tree().node_count() {
            let node = NodeId::from_usize_unchecked(index);
            match self.tree().node_kind(node) {
                NodeKind::ImportExpression => self.dynamic_import(unsafe { typed(node) }, node),
                NodeKind::CallExpression => self.require_call(unsafe { typed(node) }, node),
                NodeKind::MetaProperty => self.meta_property(unsafe { typed(node) }),
                _ => {}
            }
        }

        for reference in self.semantic.references {
            if reference.symbol.is_some() || reference.flags.space.in_type_position() {
                continue;
            }
            match self.tree().string(reference.name) {
                b"require" => self.flags.uses_require = true,
                b"module" => self.flags.uses_module = true,
                b"exports" => self.flags.uses_exports = true,
                _ => {}
            }
        }
    }

    /// Only a literal specifier is a statically known edge.
    fn dynamic_import(&mut self, expression: swc_next_ecma_ast::ImportExpression, node: NodeId) {
        let Some(specifier) = self.literal(expression.source(self.tree()).node_id()) else {
            return;
        };
        self.add_import(Import {
            kind: ImportKind::Dynamic,
            name: AstString::EMPTY,
            symbol: None,
            specifier,
            type_only: false,
            phase: expression.phase(self.tree()),
            node,
        });
    }

    /// A local binding named `require` makes the call an ordinary call.
    fn require_call(&mut self, call: swc_next_ecma_ast::CallExpression, node: NodeId) {
        let tree = unsafe { &*self.tree };
        let callee = call.callee(tree).node_id();
        if tree.node_kind(callee) != NodeKind::IdentifierReference {
            return;
        }
        let identifier = unsafe { typed::<swc_next_ecma_ast::IdentifierReference>(callee) };
        if tree.string(identifier.name(tree)) != b"require" {
            return;
        }
        if self.semantic.symbol_of(callee).is_some() {
            return;
        }
        let arguments = call.arguments(tree);
        if arguments.len() != 1 {
            return;
        }
        let argument = tree
            .get_node_in_sub_range(arguments.iter().next().expect("one argument"))
            .node_id();
        let Some(specifier) = self.literal(argument) else {
            return;
        };
        self.add_import(Import {
            kind: ImportKind::Require,
            name: AstString::EMPTY,
            symbol: None,
            specifier,
            type_only: false,
            phase: None,
            node,
        });
    }

    fn meta_property(&mut self, property: swc_next_ecma_ast::MetaProperty) {
        if self
            .tree()
            .string(property.meta(self.tree()).name(self.tree()))
            == b"import"
        {
            self.flags.uses_import_meta = true;
        }
    }

    /// The symbol bound by a binding identifier in the top level scope.
    fn binding_symbol(&self, binding: NodeId) -> Option<SymbolId> {
        let binding = unsafe { typed::<swc_next_ecma_ast::BindingIdentifier>(binding) };
        self.module_binding(binding.name(self.tree()))
    }

    /// Looks up a name in the top level scope, hoisted vars included.
    fn module_binding(&self, name: AstString) -> Option<SymbolId> {
        self.semantic
            .binding(self.top_scope, self.tree().string(name))
    }

    /// String literal text, or `None` for anything computed.
    fn literal(&self, node: NodeId) -> Option<AstString> {
        (self.tree().node_kind(node) == NodeKind::StringLiteral)
            .then(|| unsafe { typed::<swc_next_ecma_ast::StringLiteral>(node) }.value(self.tree()))
    }

    /// Name text of an identifier or string literal shaped node.
    fn string_of(&self, node: NodeId) -> AstString {
        match self.tree().node_kind(node) {
            NodeKind::IdentifierName => {
                unsafe { typed::<swc_next_ecma_ast::IdentifierName>(node) }.name(self.tree())
            }
            NodeKind::IdentifierReference => {
                unsafe { typed::<swc_next_ecma_ast::IdentifierReference>(node) }.name(self.tree())
            }
            NodeKind::BindingIdentifier => {
                unsafe { typed::<swc_next_ecma_ast::BindingIdentifier>(node) }.name(self.tree())
            }
            NodeKind::StringLiteral => {
                unsafe { typed::<swc_next_ecma_ast::StringLiteral>(node) }.value(self.tree())
            }
            _ => AstString::EMPTY,
        }
    }
}

fn literal_string(tree: &Ast<'_>, node: NodeId) -> AstString {
    if tree.node_kind(node) == NodeKind::StringLiteral {
        unsafe { typed::<swc_next_ecma_ast::StringLiteral>(node) }.value(tree)
    } else {
        AstString::EMPTY
    }
}

#[inline]
unsafe fn typed<N: AstNode>(node: NodeId) -> N {
    unsafe { N::from_node_id_unchecked(node) }
}
