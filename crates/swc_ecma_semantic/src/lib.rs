//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use oxc_index::Idx;
use swc_next_allocator::{
    hash_map::{HashMap as ArenaHashMap, Iter, Values},
    vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{
    Ast, AstInternedUtf8, AstWtf8, Diagnostic, NodeId, ScopeId, SymbolId, VisitWith,
};

use crate::{binder::Range, checker::Checker, scope::ScopeTree};

mod binder;
mod checker;
mod context;
mod ecmascript;
mod module_record;
pub mod name_resolver;
pub mod scope;
mod used_names;

pub use binder::{Declaration, Reference, ReferenceFlags, ReferenceSpace, Symbol, SymbolFlags};
pub use module_record::{
    ExportEntry, ExportName, ExportOrigin, ExportTarget, ExternalExportName, ModuleName,
    ModuleRecord,
};
pub use swc_next_ecma_ast::GeneratedName;
pub use swc_next_ecma_ast::ReferenceId;
pub use used_names::{GeneratedNameState, UsedNameState};

oxc_index::define_nonmax_u32_index_type! {
    /// Identifier for a declaration in a completed [`Semantic`] model.
    pub struct DeclarationId;
}

/// Canonical name stored for a semantic symbol.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SymbolName {
    Utf8(AstInternedUtf8),
    Wtf8(AstWtf8),
}

impl SymbolName {
    pub fn as_utf8(self) -> Option<AstInternedUtf8> {
        match self {
            Self::Utf8(name) => Some(name),
            Self::Wtf8(_) => None,
        }
    }

    pub fn as_wtf8(self) -> Option<AstWtf8> {
        match self {
            Self::Utf8(_) => None,
            Self::Wtf8(name) => Some(name),
        }
    }
}

type ScopeName = SymbolName;

type ScopeMap<'a> = ArenaHashMap<'a, AstInternedUtf8, SymbolId>;
type Wtf8ScopeMap<'a> = ArenaHashMap<'a, (ScopeId, AstWtf8), SymbolId>;
type Redeclarations<'a> = ArenaHashMap<'a, SymbolId, ArenaVec<'a, DeclarationId>>;

/// The complete semantic model of a tree, with every scope, symbol,
/// and reference fully resolved and cross-indexed. Backed by the
/// tree's arena and valid for the lifetime of the tree.
///
/// The query methods connect declaration nodes, reference nodes,
/// containing scopes, uses, and name lookup without rebuilding
/// indexes outside the arena-backed model.
#[derive(Debug)]
pub struct Semantic<'a> {
    /// Every scope, indexed by [`ScopeId`].
    scopes: ScopeTree<'a>,
    /// Every symbol, in declaration order, indexed by [`SymbolId`].
    symbols: ArenaVec<'a, Symbol>,
    /// Every reference, in source order, indexed by [`ReferenceId`].
    references: ArenaVec<'a, Reference>,
    /// Every declaration in source order, indexed by [`DeclarationId`].
    declarations: ArenaVec<'a, Declaration>,
    /// Additional declaration ids for redeclared symbols, in source order.
    redeclarations: Redeclarations<'a>,

    use_ids: ArenaVec<'a, ReferenceId>,
    use_ranges: ArenaVec<'a, Range>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, ScopeMap<'a>>,
    wtf8_bindings: Wtf8ScopeMap<'a>,

    /// The symbol declared by each node, indexed by [`NodeId`].
    node_symbols: ArenaVec<'a, Option<SymbolId>>,
    node_references: ArenaVec<'a, Option<ReferenceId>>,

    module_record: ModuleRecord<'a>,
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
    /// Empty unless [`AnalyzeOptions::build_module_record`] was enabled.
    pub fn module_record(&self) -> &ModuleRecord<'a> {
        &self.module_record
    }

    /// The scope with the given id.
    pub fn scope(&self, id: ScopeId) -> scope::Scope {
        self.scopes.get(id)
    }

    /// The symbol declared at `node`, or `None` when the node is not a
    /// declaration site.
    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.node_symbols[node.index()]
    }

    /// The reference recorded at `node`, or `None` when the node is
    /// not a reference site.
    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        self.node_references[node.index()]
    }

    /// The scope of the given symbol id.
    pub fn scope_of(&self, id: SymbolId) -> ScopeId {
        self.symbols[id.index()].scope
    }

    /// Iterates the ids of every declaration of `id`, in source order.
    pub fn declaration_ids(&self, id: SymbolId) -> DeclarationIdIterator<'_> {
        let additional = self
            .redeclarations
            .get(&id)
            .map_or(&[][..], |declarations| declarations.as_slice());
        DeclarationIdIterator {
            first: Some(self.symbols[id.index()].declaration),
            additional: additional.iter().copied(),
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
    pub fn declarations(
        &self,
        id: SymbolId,
    ) -> impl ExactSizeIterator<Item = Declaration> + DoubleEndedIterator + '_ {
        self.declaration_ids(id)
            .map(|declaration_id| self.declaration(declaration_id))
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
    pub fn binding(&self, scope_id: ScopeId, name: AstInternedUtf8) -> Option<SymbolId> {
        self.scope_maps[scope_id.index()]
            .get(&name)
            .copied()
            .or_else(|| {
                self.hoisting_variables[scope_id.index()]
                    .get(&name)
                    .copied()
            })
    }

    /// Every symbol declared directly in `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through.
    pub fn bindings(&self, scope_id: ScopeId) -> BindingIterator<'_> {
        BindingIterator {
            inner: self.scope_maps[scope_id.index()].values(),
            wtf8: self.wtf8_bindings.iter(),
            scope: scope_id,
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
        name: AstInternedUtf8,
        space: ReferenceSpace,
    ) -> Option<SymbolId> {
        for ancestor in self.scopes.ancestors(Some(scope_id)) {
            let Some(&id) = self.scope_maps[ancestor.index()].get(&name) else {
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

    /// Iterates every `(id, declaration)` pair in source order.
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

/// Declaration ids for one symbol, in source order.
#[derive(Clone)]
pub struct DeclarationIdIterator<'a> {
    first: Option<DeclarationId>,
    additional: std::iter::Copied<std::slice::Iter<'a, DeclarationId>>,
}

impl Iterator for DeclarationIdIterator<'_> {
    type Item = DeclarationId;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.additional.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for DeclarationIdIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.additional.next_back().or_else(|| self.first.take())
    }
}

impl ExactSizeIterator for DeclarationIdIterator<'_> {
    fn len(&self) -> usize {
        usize::from(self.first.is_some()) + self.additional.len()
    }
}

impl std::iter::FusedIterator for DeclarationIdIterator<'_> {}

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

pub struct BindingIterator<'a> {
    inner: Values<'a, AstInternedUtf8, SymbolId>,
    wtf8: Iter<'a, (ScopeId, AstWtf8), SymbolId>,
    scope: ScopeId,
}

impl Iterator for BindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.inner.next() {
            return Some(*id);
        }
        self.wtf8
            .find_map(|(&(scope, _), &id)| (scope == self.scope).then_some(id))
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
    /// Whether to collect the source text module's export occurrences and
    /// resolved targets in [`Semantic::module_record`].
    ///
    /// Disabled by default.
    pub build_module_record: bool,
}

/// Runs semantic analysis on a tree and builds the requested [`Semantic`] model.
///
/// When [`AnalyzeOptions::check_syntax`] is enabled, scope-dependent early
/// errors are returned separately from parser diagnostics. Their messages use
/// the tree's arena, so the result is valid as long as the tree is alive.
/// When [`AnalyzeOptions::build_module_record`] is disabled, the returned
/// module record is empty.
pub fn analyze<'a>(ast: &mut Ast<'a>, options: AnalyzeOptions) -> SemanticReturn<'a> {
    let root = ast.root_program();
    let mut visitor = Checker::new(ast, options.check_syntax, options.build_module_record);
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
    use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType, Span};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::{
        AnalyzeOptions, ExportOrigin, ExportTarget, ReferenceSpace, Semantic, SymbolFlags,
        SymbolName, analyze, scope::ScopeKind,
    };

    fn analyze_source<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        lang: Lang,
        source_type: SourceType,
    ) -> (Ast<'a>, Semantic<'a>) {
        analyze_source_with_options(
            allocator,
            source,
            lang,
            source_type,
            AnalyzeOptions::default(),
        )
    }

    fn analyze_source_with_options<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        lang: Lang,
        source_type: SourceType,
        options: AnalyzeOptions,
    ) -> (Ast<'a>, Semantic<'a>) {
        let parse = Parser::init_copying(
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
        let mut ast = parse.ast;
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let result = analyze(&mut ast, options);
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
        (ast, result.semantic)
    }

    fn syntax_diagnostics(source: &str) -> Vec<(String, Span)> {
        let allocator = Allocator::new();
        let mut parse = Parser::init_copying(
            &allocator,
            source,
            Options {
                lang: Lang::Js,
                source_type: SourceType::Script,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        analyze(
            &mut parse.ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .diagnostics
        .into_iter()
        .map(|diagnostic| (diagnostic.message.into_owned(), diagnostic.span))
        .collect()
    }

    #[test]
    fn strict_directive_octal_escapes() {
        let before_directive = syntax_diagnostics(r#"'\1'; 'use strict';"#);
        assert_eq!(before_directive.len(), 1, "{before_directive:#?}");
        assert_eq!(before_directive[0].1, Span::new(0, 4));

        let after_directive = syntax_diagnostics(r#"'use strict'; '\1';"#);
        assert_eq!(after_directive.len(), 1, "{after_directive:#?}");
        assert_eq!(after_directive[0].1, Span::new(14, 18));

        let comment = syntax_diagnostics(r#"'use strict'; "ok" /* \1 */;"#);
        assert!(comment.is_empty(), "{comment:#?}");

        let escaped_backslash = syntax_diagnostics(r#""\\1"; "use strict";"#);
        assert!(escaped_backslash.is_empty(), "{escaped_backslash:#?}");
    }

    #[test]
    fn symbols() {
        let allocator = Allocator::new();
        let (mut ast, semantic) = analyze_source(
            &allocator,
            "interface A {} interface B {} interface A {} interface \\u0041 {}",
            Lang::Ts,
            SourceType::Script,
        );
        let a_name = ast.intern_utf8("A");
        let b_name = ast.intern_utf8("B");
        let a = semantic.binding(ScopeId::ROOT, a_name).unwrap();
        let b = semantic.binding(ScopeId::ROOT, b_name).unwrap();
        let mut a_declarations = semantic.declaration_ids(a);
        assert_eq!(a_declarations.len(), 3);
        let first_a = a_declarations.next().unwrap();
        let last_a = a_declarations.next_back().unwrap();
        assert!(
            ast.span(semantic.declaration(first_a).node).start
                < ast.span(semantic.declaration(last_a).node).start
        );

        let mut declarations = semantic.iter_declarations();
        let first = declarations.next().unwrap();
        let second = declarations.next().unwrap();
        let third = declarations.next().unwrap();
        let fourth = declarations.next().unwrap();
        assert!(declarations.next().is_none());
        assert_eq!(
            [
                first.declaration.symbol,
                second.declaration.symbol,
                third.declaration.symbol,
                fourth.declaration.symbol,
            ],
            [a, b, a, a]
        );
        assert_eq!(semantic.symbol(a).declaration, first.id);
        assert_eq!(
            semantic.declaration_ids(a).collect::<Vec<_>>(),
            [first.id, third.id, fourth.id]
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
            .find(|entry| {
                matches!(
                    entry.symbol.name,
                    SymbolName::Utf8(name) if ast.utf8_eq_bytes(name.into(), b"T")
                )
            })
            .unwrap();
        assert!(symbol.symbol.flags.contains(SymbolFlags::TYPE_IMPORT));

        let reference = semantic
            .iter_references()
            .find(|entry| ast.utf8_eq_bytes(entry.reference.name.into(), b"T"))
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
            .find(|entry| ast.utf8_eq_bytes(entry.reference.name.into(), b"T"))
            .unwrap()
            .reference;
        assert_eq!(reference.flags.space, ReferenceSpace::Value);
        assert!(reference.symbol.is_none());
        assert!(reference.flags.is_read_only());
    }

    #[test]
    fn modules() {
        let allocator = Allocator::new();

        let (_, semantic) = analyze_source(
            &allocator,
            "export const value = 1; export { value }; export * from 'dep'; export default value;",
            Lang::Js,
            SourceType::Module,
        );
        assert!(semantic.module_record().exports().is_empty());

        let (ast, semantic) = analyze_source_with_options(
            &allocator,
            "export { value }; const value = 1;",
            Lang::Js,
            SourceType::Module,
            AnalyzeOptions {
                build_module_record: true,
                ..AnalyzeOptions::default()
            },
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
                reference: Some(local_reference),
            }
        );

        let (ast, semantic) = analyze_source_with_options(
            &allocator,
            "export { external as alias } from 'dep';",
            Lang::Js,
            SourceType::Module,
            AnalyzeOptions {
                build_module_record: true,
                ..AnalyzeOptions::default()
            },
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

        let (mut ast, semantic) = analyze_source(
            &allocator,
            "let value; with ({}) { value; }",
            Lang::Js,
            SourceType::Script,
        );
        let value_name = ast.intern_utf8("value");
        let value_symbol = semantic.binding(ScopeId::ROOT, value_name).unwrap();
        let value = semantic
            .iter_references()
            .find(|entry| ast.utf8_eq_bytes(entry.reference.name.into(), b"value"))
            .unwrap()
            .reference;
        assert!(value.flags.is_dynamic());
        assert_eq!(value.symbol, Some(value_symbol));

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
            .find(|entry| ast.utf8_eq_bytes(entry.reference.name.into(), b"value"))
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

        let reference_id = semantic.reference_of(declaration.declaration.node).unwrap();
        let reference = semantic.reference(reference_id);
        let symbol_id = declaration.declaration.symbol;
        assert!(reference.flags.is_write_only());
        assert!(reference.flags.is_dynamic());
        assert_eq!(reference.symbol, Some(symbol_id));
        assert_eq!(semantic.uses(symbol_id), &[reference_id]);
        assert!(matches!(
            semantic.symbol(symbol_id).name,
            SymbolName::Utf8(name) if ast.utf8_eq_bytes(name.into(), b"value")
        ));

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

    #[test]
    fn canonicalizes_semantic_name_handles() {
        let allocator = Allocator::new();
        let (mut ast, semantic) = analyze_source(
            &allocator,
            "let value; value; v\\u0061lue;",
            Lang::Js,
            SourceType::Script,
        );

        let references: Vec<_> = semantic
            .iter_references()
            .map(|entry| entry.reference)
            .collect();
        assert_eq!(references.len(), 2);
        assert_eq!(references[0].name, references[1].name);
        assert!(ast.utf8_eq_bytes(references[0].name.into(), b"value"));

        let value_name = ast.intern_utf8("value");
        let symbol = semantic.binding(ScopeId::ROOT, value_name).unwrap();
        assert_eq!(
            semantic.symbol(symbol).name,
            SymbolName::Utf8(references[0].name)
        );
    }
}
