//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use oxc_index::Idx;
use swc_next_allocator::{
    hash_map::{HashMap as ArenaHashMap, Values},
    vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{Ast, Diagnostic, NodeId, ScopeId, SymbolId, VisitWith};

use crate::{binder::Range, checker::Checker, scope::ScopeTree};

mod binder;
mod checker;
mod context;
mod ecmascript;
mod module_record;
pub mod scope;

pub use binder::{Declaration, Reference, ReferenceFlags, ReferenceSpace, Symbol, SymbolFlags};
pub use module_record::{
    ExportEntry, ExportName, ExportOrigin, ExportTarget, ExternalExportName, ModuleName,
    ModuleRecord,
};
pub use swc_next_ecma_ast::ReferenceId;

oxc_index::define_nonmax_u32_index_type! {
    /// Identifier for a declaration in a completed [`Semantic`] model.
    pub struct DeclarationId;
}

type ScopeMap<'a> = ArenaHashMap<'a, &'a [u8], SymbolId>;

/// The complete semantic model of a tree, with every scope, symbol,
/// and reference fully resolved and cross-indexed. Backed by the
/// tree's arena and valid for the lifetime of the tree.
///
/// The query methods connect declaration nodes, reference nodes,
/// containing scopes, parents, uses, and name lookup without rebuilding
/// indexes outside the arena-backed model.
#[derive(Debug)]
pub struct Semantic<'a> {
    /// Every scope, indexed by [`ScopeId`].
    scopes: ScopeTree<'a>,
    /// Every symbol, in declaration order, indexed by [`SymbolId`].
    symbols: ArenaVec<'a, Symbol>,
    /// Every reference, in source order, indexed by [`ReferenceId`].
    references: ArenaVec<'a, Reference>,
    /// Every declaration, grouped by symbol and indexed by [`DeclarationId`].
    declarations: ArenaVec<'a, Declaration>,
    module_record: ModuleRecord<'a>,
    use_ids: ArenaVec<'a, ReferenceId>,
    use_ranges: ArenaVec<'a, Range>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, ScopeMap<'a>>,
    node_scopes: ArenaVec<'a, ScopeId>,
    node_parents: ArenaVec<'a, Option<NodeId>>,
    node_declarations: ArenaVec<'a, Option<DeclarationId>>,
    node_references: ArenaVec<'a, Option<ReferenceId>>,
}

impl<'a> Semantic<'a> {
    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id.index()]
    }

    /// The reference with the given id.
    pub fn reference(&self, id: ReferenceId) -> Reference {
        self.references[id.index()]
    }

    /// The declaration with the given id.
    pub fn declaration(&self, id: DeclarationId) -> Declaration {
        self.declarations[id.index()]
    }

    /// The source text module's export occurrences and resolved targets.
    pub fn module_record(&self) -> &ModuleRecord<'a> {
        &self.module_record
    }

    /// The scope with the given id.
    pub fn scope(&self, id: ScopeId) -> scope::Scope {
        self.scopes.get(id)
    }

    /// The symbol declared at `node`, or the lexical fallback symbol of the
    /// reference at `node`. `None` when the node neither declares nor
    /// references a binding. Inspect [`ReferenceFlags::is_dynamic`] before
    /// treating a reference's fallback as its definite runtime binding.
    ///
    /// ## Example
    /// ```ts
    /// let a = 1; a;
    /// //  ^ the declared symbol
    /// //         ^ the same symbol, through the reference
    /// ```
    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.node_declarations[node.index()]
            .map(|id| self.declaration(id).symbol)
            .or_else(|| self.node_references[node.index()].and_then(|id| self.reference(id).symbol))
    }

    /// The declaration recorded at `node`, or `None` when the node is not a
    /// declaration site.
    pub fn declaration_of(&self, node: NodeId) -> Option<DeclarationId> {
        self.node_declarations[node.index()]
    }

    /// The reference recorded at `node`, or `None` when the node is
    /// not a reference site.
    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        self.node_references[node.index()]
    }

    /// The innermost lexical scope containing `node`. A scope-creating
    /// node (function, block, class, ...) maps to the scope it creates;
    /// that scope's parent is the enclosing scope.
    pub fn scope_of(&self, node: NodeId) -> ScopeId {
        self.node_scopes[node.index()]
    }

    /// The structural parent of `node`, or `None` at the root.
    pub fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        self.node_parents[node.index()]
    }

    /// Walks from `node` up to the root, yielding `node` first.
    pub fn ancestors(&self, node: Option<NodeId>) -> SemanticAncestorIterator<'_> {
        SemanticAncestorIterator {
            node_parents: &self.node_parents,
            current: node,
        }
    }

    /// Every declaration of `id`, in source order. Usually a single element.
    /// Symbols whose declarations merge (`var` redeclarations, TS overloads,
    /// `class` + `interface`) have one entry per declaration.
    ///
    /// ## Example
    /// ```ts
    /// var a = 1; var a = 2;
    /// //  ^ declarations[0]
    /// //             ^ declarations[1], same symbol
    /// ```
    ///
    /// A conflicting redeclaration is recorded here too, aliased onto
    /// the existing symbol so tooling on broken code still maps the node.
    pub fn declarations(&self, id: SymbolId) -> &[Declaration] {
        let range = self.symbol(id).declarations;
        let start = range.start as usize;
        let end = start + range.len as usize;
        &self.declarations[start..end]
    }

    /// Every use site of `id`, in source order. Declaration sites are
    /// not uses; those are returned by [`Semantic::declarations`]. A dynamic
    /// `with` reference is included under its potential lexical fallback.
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        let range = self.use_ranges[id.index()];
        let start = range.start as usize;
        &self.use_ids[start..start + range.len as usize]
    }

    /// The binding of `name` at `scope` alone, including a hoisting
    /// `var` passing through on its way to its hoist target. This does
    /// not walk the scope chain; see [`Semantic::lookup`].
    ///
    /// ## Example
    /// ```ts
    /// function f() { { var a; } }
    /// //             ^ binding here finds `a`, whose symbol
    /// //               lives in the function scope
    /// ```
    pub fn binding(&self, scope_id: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps[scope_id.index()]
            .get(name)
            .copied()
            .or_else(|| self.hoisting_variables[scope_id.index()].get(name).copied())
    }

    /// Every symbol declared directly in `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through.
    pub fn bindings(&self, scope_id: ScopeId) -> BindingIterator<'_> {
        BindingIterator {
            inner: self.scope_maps[scope_id.index()].values(),
        }
    }

    /// The nearest lexical fallback binding of `name` visible in `space`
    /// from `scope`. A binding outside the space does not shadow.
    /// [`ReferenceSpace::Any`] matches by name alone. This does not query
    /// runtime object properties when the path crosses [`scope::ScopeKind::With`].
    ///
    /// ## Example
    /// ```ts
    /// type T = string;
    /// function f() {
    ///   const T = 1;
    ///   // type space finds the outer alias, value space the local const
    /// }
    /// ```
    pub fn lookup(
        &self,
        scope_id: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> Option<SymbolId> {
        for ancestor in self.scopes.ancestors(Some(scope_id)) {
            let Some(&id) = self.scope_maps[ancestor.index()].get(name) else {
                continue;
            };
            if self.symbol(id).flags.visible_in(space) {
                return Some(id);
            }
        }
        None
    }

    /// Iterates every `(id, scope)` pair in creation order.
    pub fn iter_scopes(&self) -> ScopeIterator<'_> {
        ScopeIterator {
            list: &self.scopes.list,
            index: 0,
        }
    }

    /// Iterates every `(id, symbol)` pair in declaration order.
    pub fn iter_symbols(&self) -> SymbolIterator<'_> {
        SymbolIterator {
            symbols: &self.symbols,
            index: 0,
        }
    }

    /// Iterates every `(id, reference)` pair in source order.
    pub fn iter_references(&self) -> ReferenceIterator<'_> {
        ReferenceIterator {
            references: &self.references,
            index: 0,
        }
    }

    /// Iterates every `(id, declaration)` pair, grouped by symbol and in
    /// source order within each symbol.
    pub fn iter_declarations(&self) -> DeclarationIterator<'_> {
        DeclarationIterator {
            declarations: &self.declarations,
            index: 0,
        }
    }
}

/// A `(id, scope)` pair yielded by [`Semantic::iter_scopes`].
pub struct ScopeEntry {
    pub id: ScopeId,
    pub scope: scope::Scope,
}

/// A `(id, symbol)` pair yielded by [`Semantic::iter_symbols`].
pub struct SymbolEntry {
    pub id: SymbolId,
    pub symbol: Symbol,
}

/// A `(id, reference)` pair yielded by [`Semantic::iter_references`].
pub struct ReferenceEntry {
    pub id: ReferenceId,
    pub reference: Reference,
}

/// A `(id, declaration)` pair yielded by [`Semantic::iter_declarations`].
pub struct DeclarationEntry {
    pub id: DeclarationId,
    pub declaration: Declaration,
}

pub struct ScopeIterator<'a> {
    list: &'a [scope::Scope],
    index: u32,
}

impl Iterator for ScopeIterator<'_> {
    type Item = ScopeEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let scope = *self.list.get(index)?;
        self.index += 1;
        Some(ScopeEntry {
            id: ScopeId::from_usize(index),
            scope,
        })
    }
}

pub struct SemanticAncestorIterator<'a> {
    node_parents: &'a [Option<NodeId>],
    current: Option<NodeId>,
}

impl Iterator for SemanticAncestorIterator<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = self.node_parents[node.index()];
        Some(node)
    }
}

pub struct BindingIterator<'a> {
    inner: Values<'a, &'a [u8], SymbolId>,
}

impl Iterator for BindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}

pub struct SymbolIterator<'a> {
    symbols: &'a [Symbol],
    index: u32,
}

impl Iterator for SymbolIterator<'_> {
    type Item = SymbolEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let symbol = *self.symbols.get(index)?;
        self.index += 1;
        Some(SymbolEntry {
            id: SymbolId::from_usize(index),
            symbol,
        })
    }
}

pub struct ReferenceIterator<'a> {
    references: &'a [Reference],
    index: u32,
}

pub struct DeclarationIterator<'a> {
    declarations: &'a [Declaration],
    index: u32,
}

impl Iterator for ReferenceIterator<'_> {
    type Item = ReferenceEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let reference = *self.references.get(index)?;
        self.index += 1;
        Some(ReferenceEntry {
            id: ReferenceId::from_usize(index),
            reference,
        })
    }
}

impl Iterator for DeclarationIterator<'_> {
    type Item = DeclarationEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let declaration = *self.declarations.get(index)?;
        self.index += 1;
        Some(DeclarationEntry {
            id: DeclarationId::from_usize(index),
            declaration,
        })
    }
}

/// Semantic analysis output and any requested scope-dependent early errors.
pub struct SemanticReturn<'a> {
    pub semantic: Semantic<'a>,
    pub diagnostics: Vec<Diagnostic<'a>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AnalyzeOptions {
    /// Whether to check and report scope-dependent syntax errors.
    ///
    /// Disabled by default.
    pub check_syntax: bool,
}

/// Runs semantic analysis on a tree and builds the complete [`Semantic`] model.
///
/// When [`AnalyzeOptions::check_syntax`] is enabled, scope-dependent early
/// errors are returned separately from parser diagnostics. Their messages use
/// the tree's arena, so the result is valid as long as the tree is alive.
pub fn analyze<'a>(ast: &Ast<'a>, options: AnalyzeOptions) -> SemanticReturn<'a> {
    let root = ast.root_program();
    let mut visitor = Checker::new(ast, options.check_syntax);
    root.visit_with(&mut visitor);
    let (semantic, diagnostics) = visitor.finish();
    SemanticReturn {
        semantic,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::{
        AnalyzeOptions, ExportOrigin, ExportTarget, ReferenceSpace, Semantic, SymbolFlags, analyze,
        scope::ScopeKind,
    };

    fn analyze_source<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        lang: Lang,
        source_type: SourceType,
    ) -> (Ast<'a>, Semantic<'a>) {
        let parse = Parser::init(
            allocator,
            source,
            Options {
                lang,
                source_type,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        let ast = parse.ast;
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let result = analyze(&ast, AnalyzeOptions::default());
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
        (ast, result.semantic)
    }

    #[test]
    fn symbols() {
        let allocator = Allocator::new();
        let (ast, semantic) = analyze_source(
            &allocator,
            "interface A {} interface B {} interface A {}",
            Lang::Ts,
            SourceType::Script,
        );
        let a = semantic.binding(ScopeId::ROOT, b"A").unwrap();
        let b = semantic.binding(ScopeId::ROOT, b"B").unwrap();
        let a_declarations = semantic.declarations(a);
        assert_eq!(a_declarations.len(), 2);
        assert!(ast.span(a_declarations[0].node).start < ast.span(a_declarations[1].node).start);

        let mut declarations = semantic.iter_declarations();
        let first = declarations.next().unwrap();
        let second = declarations.next().unwrap();
        let third = declarations.next().unwrap();
        assert!(declarations.next().is_none());
        assert_eq!(
            [
                first.declaration.symbol,
                second.declaration.symbol,
                third.declaration.symbol,
            ],
            [a, a, b]
        );
        assert_eq!(
            semantic.declaration_of(first.declaration.node),
            Some(first.id)
        );
        assert_eq!(
            semantic.declaration_of(second.declaration.node),
            Some(second.id)
        );
        assert_eq!(
            semantic.declaration_of(third.declaration.node),
            Some(third.id)
        );
    }

    #[test]
    fn references() {
        let allocator = Allocator::new();

        let (_, semantic) = analyze_source(
            &allocator,
            "let value; value;",
            Lang::Js,
            SourceType::Script,
        );
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(reference.flags.is_read_only());

        let (_, semantic) = analyze_source(
            &allocator,
            "let value; value = 1;",
            Lang::Js,
            SourceType::Script,
        );
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(reference.flags.is_write_only());

        let (_, semantic) = analyze_source(
            &allocator,
            "let value; value++;",
            Lang::Js,
            SourceType::Script,
        );
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(reference.flags.is_read_write());

        let (_, semantic) = analyze_source(
            &allocator,
            "let value; delete value;",
            Lang::Js,
            SourceType::Script,
        );
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(!reference.flags.is_read());
        assert!(!reference.flags.is_write());

        let (ast, semantic) = analyze_source(
            &allocator,
            "import type { T } from 'mod'; type U = T;",
            Lang::Ts,
            SourceType::Module,
        );
        let symbol = semantic
            .iter_symbols()
            .find(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"T")
            .unwrap();
        assert!(symbol.symbol.flags.contains(SymbolFlags::TYPE_IMPORT));

        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"T")
            .unwrap()
            .reference;
        assert_eq!(reference.flags.space, ReferenceSpace::Type);
        assert_eq!(reference.symbol, Some(symbol.id));
        assert!(!reference.flags.is_read());

        let (ast, semantic) = analyze_source(
            &allocator,
            "import type { T } from 'mod'; T;",
            Lang::Ts,
            SourceType::Module,
        );
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"T")
            .unwrap()
            .reference;
        assert_eq!(reference.flags.space, ReferenceSpace::Value);
        assert!(reference.symbol.is_none());
        assert!(reference.flags.is_read_only());
    }

    #[test]
    fn modules() {
        let allocator = Allocator::new();

        let (ast, semantic) = analyze_source(
            &allocator,
            "export { value }; const value = 1;",
            Lang::Js,
            SourceType::Module,
        );
        let body = ast.root_program().body(&ast);
        let statement = body.get_node(&ast, 0).unwrap();
        let declaration = statement.as_export_named_declaration(&ast).unwrap();
        let local_specifier = declaration.specifiers(&ast).get_node(&ast, 0).unwrap();
        let local_node = local_specifier.local(&ast).node_id();

        let local_reference = semantic.reference_of(local_node).unwrap();
        let local_reference_data = semantic.reference(local_reference);
        let local_symbol = local_reference_data.symbol.unwrap();
        assert_eq!(local_reference_data.flags.space, ReferenceSpace::Any);

        let local_export = semantic
            .module_record()
            .exports()
            .iter()
            .find(|entry| entry.node == local_specifier.node_id())
            .unwrap();
        assert_eq!(local_export.origin, ExportOrigin::LocalSpecifier);
        assert_eq!(
            local_export.target,
            ExportTarget::Local {
                symbol: local_symbol,
                declaration: None,
                reference: Some(local_reference),
            }
        );

        let (ast, semantic) = analyze_source(
            &allocator,
            "export { external as alias } from 'dep';",
            Lang::Js,
            SourceType::Module,
        );
        let body = ast.root_program().body(&ast);
        let statement = body.get_node(&ast, 0).unwrap();
        let declaration = statement.as_export_named_declaration(&ast).unwrap();
        let re_export_specifier = declaration.specifiers(&ast).get_node(&ast, 0).unwrap();
        let re_export_node = re_export_specifier.local(&ast).node_id();
        assert_eq!(semantic.reference_of(re_export_node), None);

        let re_export = semantic
            .module_record()
            .exports()
            .iter()
            .find(|entry| entry.node == re_export_specifier.node_id())
            .unwrap();
        assert_eq!(re_export.origin, ExportOrigin::ReExportSpecifier);
        assert!(matches!(re_export.target, ExportTarget::External { .. }));
    }

    #[test]
    fn dynamic_scopes() {
        let allocator = Allocator::new();

        let (ast, semantic) = analyze_source(
            &allocator,
            "let value; with ({}) { value; }",
            Lang::Js,
            SourceType::Script,
        );
        let value_symbol = semantic.binding(ScopeId::ROOT, b"value").unwrap();
        let value = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"value")
            .unwrap()
            .reference;
        assert!(value.flags.is_dynamic());
        assert_eq!(value.symbol, Some(value_symbol));

        let with_scope = semantic
            .iter_scopes()
            .find(|entry| entry.scope.kind == ScopeKind::With)
            .unwrap();
        assert_eq!(semantic.scope_of(with_scope.scope.node), with_scope.id);

        let (_, semantic) = analyze_source(
            &allocator,
            "with ({}) { missing; }",
            Lang::Js,
            SourceType::Script,
        );
        let missing = semantic.iter_references().next().unwrap().reference;
        assert!(missing.flags.is_dynamic());
        assert!(missing.symbol.is_none());

        let (ast, semantic) = analyze_source(
            &allocator,
            "with ({}) { let value; value; }",
            Lang::Js,
            SourceType::Script,
        );
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"value")
            .unwrap()
            .reference;
        assert!(!reference.flags.is_dynamic());
        assert!(reference.symbol.is_some());

        let (ast, semantic) = analyze_source(
            &allocator,
            "with ({}) { var value = 1; }",
            Lang::Js,
            SourceType::Script,
        );
        let declaration = semantic.iter_declarations().next().unwrap();
        assert_eq!(
            semantic.declaration_of(declaration.declaration.node),
            Some(declaration.id)
        );

        let reference_id = semantic.reference_of(declaration.declaration.node).unwrap();
        let reference = semantic.reference(reference_id);
        assert!(reference.flags.is_write_only());
        assert!(reference.flags.is_dynamic());
        assert_eq!(reference.symbol, Some(declaration.declaration.symbol));
        assert_eq!(
            semantic.uses(declaration.declaration.symbol),
            &[reference_id]
        );
        assert_eq!(
            ast.get_wtf8(semantic.symbol(declaration.declaration.symbol).name)
                .as_bytes(),
            b"value"
        );

        let (_, semantic) = analyze_source(
            &allocator,
            "function f() { eval('direct'); }",
            Lang::Js,
            SourceType::Script,
        );
        let function = semantic
            .iter_scopes()
            .find(|entry| entry.scope.kind == ScopeKind::Function)
            .unwrap();
        assert!(function.scope.flags.contains_direct_eval());
        assert!(semantic.scope(ScopeId::ROOT).flags.contains_direct_eval());

        let (_, semantic) = analyze_source(
            &allocator,
            "function f() { (0, eval)('indirect'); }",
            Lang::Js,
            SourceType::Script,
        );
        let function = semantic
            .iter_scopes()
            .find(|entry| entry.scope.kind == ScopeKind::Function)
            .unwrap();
        assert!(!function.scope.flags.contains_direct_eval());
        assert!(!semantic.scope(ScopeId::ROOT).flags.contains_direct_eval());
    }
}
