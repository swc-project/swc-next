//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! TypeScript analysis resolves declaration identities, spaces, and visibility.
//! It does not infer expression types or evaluate conditional types. Member
//! lookup uses declarations and direct property-name syntax. Unresolved computed
//! names preserve possible property value contributions without opening type or
//! namespace lookup, inferring member names, or proving declaration conflicts.
//! References expose this uncertainty and retain every possible binding in the
//! use index. Parentheses are transparent when collecting property-name facts,
//! including where TypeScript's own member binding distinguishes their syntax.
//! Merged class/interface type parameters share declaration identity while
//! retaining occurrence-local visibility. This does not model TypeScript's
//! shared identities between type parameters and same-named instance members.
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

use crate::{
    binder::{Range, TsMemberIndex},
    checker::Checker,
    scope::ScopeTree,
};

mod binder;
mod checker;
mod context;
mod ecmascript;
mod module_record;
pub mod name_resolver;
pub mod scope;

pub use binder::{
    Declaration, NameResolution, Reference, ReferenceFlags, ReferenceSpace, Symbol, SymbolFlags,
};
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
// Most scopes have no var declarations hoisting through them.
type HoistingMap<'a> = Option<swc_next_allocator::boxed::Box<'a, ScopeMap<'a>>>;
type Redeclarations<'a> = ArenaHashMap<'a, SymbolId, ArenaVec<'a, DeclarationId>>;

/// The semantic model of a tree, with scopes, symbols, references, and their
/// lookup certainty cross-indexed. Opaque TypeScript facts may leave multiple
/// possible reference targets. Backed by the
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
    reference_candidates: binder::ReferenceCandidates<'a>,
    /// Every declaration in source order, indexed by [`DeclarationId`].
    declarations: ArenaVec<'a, Declaration>,
    /// Additional declaration ids for redeclared symbols, in source order.
    redeclarations: Redeclarations<'a>,

    use_ids: ArenaVec<'a, ReferenceId>,
    use_ranges: ArenaVec<'a, Range>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, HoistingMap<'a>>,

    /// The symbol declared by each node, indexed by [`NodeId`].
    node_symbols: ArenaVec<'a, Option<SymbolId>>,
    node_references: ArenaVec<'a, Option<ReferenceId>>,

    module_record: ModuleRecord<'a>,
    ts_members: TsMemberIndex<'a>,
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

    /// The binding scope of a symbol in JavaScript semantic analysis only.
    /// Returns the scope of the symbol's first declaration.
    ///
    /// TypeScript symbols can merge declarations across multiple scopes;
    /// use the relevant [`Declaration::scope`] for TypeScript analysis.
    pub fn scope_of(&self, id: SymbolId) -> ScopeId {
        let declaration = self.symbol(id).declaration;
        self.declaration(declaration).scope
    }

    /// The definite terminal symbol of an internal TypeScript import alias.
    /// Follows forwarding aliases without changing their binding identities.
    /// Returns `None` for non-aliases, opaque or cyclic targets, property-only
    /// targets, and merged contributions without a unique terminal symbol.
    /// The target retains its declaration kind, including `CONST_ENUM`, regardless
    /// of JavaScript emit options. This does not decide runtime access or emission.
    pub fn ts_import_alias_target(&self, alias: SymbolId) -> Option<SymbolId> {
        self.ts_members.import_alias_target(alias)
    }

    /// The initializer's root reference in an internal `import A = N.Member`.
    /// Its access flags account for the whole alias target, not just `N`:
    /// a known type-only target does not read the root, while opaque targets
    /// conservatively retain a read. Ambient and explicit type-only aliases
    /// also have no runtime access. External `require` targets return `None`.
    ///
    /// This describes initializer access, not whether the alias is used or
    /// whether an emit option preserves its target. Use ordinary reference
    /// queries for declaration dependencies even when this reference is unread.
    /// `ast` must be the tree analyzed to produce this model.
    pub fn import_equals_reference(
        &self,
        ast: &Ast<'_>,
        declaration: swc_next_ecma_ast::TsImportEqualsDeclaration,
    ) -> Option<ReferenceId> {
        let node = ecmascript::import_equals_reference_node(ast, declaration)?;
        self.reference_of(node)
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

    /// Every definite or possible use site of `id`, once each in source order. Declaration sites are
    /// not uses; those are returned by [`Semantic::declarations`]. A dynamic
    /// `with` reference is included under its potential lexical fallback.
    /// An uncertain TypeScript reference is included under every candidate,
    /// even when [`Reference::symbol`] names a different, nearer candidate.
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        let range = self.use_ranges[id.index()];
        let start = range.start as usize;
        &self.use_ids[start..start + range.len as usize]
    }

    /// Possible bindings for this reference, with the nearest candidate first
    /// and any additional targets in declaration order. Each symbol occurs once.
    /// A candidate is not proof that lookup succeeds: an opaque alias may not
    /// contribute to the reference's space. Inspect [`Reference::resolution`]
    /// before an operation that requires a unique target.
    pub fn reference_candidates(&self, id: ReferenceId) -> impl Iterator<Item = SymbolId> + '_ {
        self.reference(id)
            .symbol
            .into_iter()
            .chain(self.reference_candidates.additional(id).iter().copied())
    }

    /// Value bindings associated with a reference's selected bindings.
    /// Includes the alias contributions of a selected public member and follows
    /// local TypeScript ExportValue links without changing lexical identity.
    /// Alias space information determines whether each target is a definite
    /// value or only a possible one. Results are never Unresolved.
    ///
    /// This queries value meaning, not runtime usage: a `typeof` or explicit
    /// type-only export can still name a value. Access flags and emit options
    /// determine whether it is needed at runtime. Import aliases remain bindings;
    /// this does not replace them with their final property or declaration target.
    /// `ast` must be the tree analyzed to produce this model.
    pub fn reference_value_targets<'s>(
        &'s self,
        ast: &'s Ast<'_>,
        id: ReferenceId,
    ) -> impl Iterator<Item = NameResolution> + 's {
        let reference = self.reference(id);
        let name = ast.get_utf8(reference.name).as_bytes();
        let member_values = self.ts_members.reference_values(id);
        let lexical_values = member_values
            .is_none()
            .then_some(())
            .into_iter()
            .flat_map(move |_| self.reference_candidates(id))
            .filter_map(move |symbol| {
                let resolution = if reference.flags.is_uncertain() {
                    NameResolution::Uncertain { candidate: symbol }
                } else {
                    NameResolution::Resolved(symbol)
                };
                self.ts_members.value_target(
                    &self.symbols,
                    &self.declarations,
                    &self.scope_maps,
                    resolution,
                    name,
                    reference.flags.space.value_target_space(),
                )
            });
        member_values
            .into_iter()
            .flatten()
            .copied()
            .chain(lexical_values)
    }

    /// Members declared by any declaration merged into a TypeScript namespace.
    pub fn ts_namespace_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.ts_members.namespace_members(owner)
    }

    /// Finds a member declared by any declaration merged into a TypeScript namespace.
    /// This returns its identity; its declaration flags do not describe its
    /// resolved visibility in every reference space or member view.
    pub fn ts_namespace_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.ts_members.namespace_member(owner, name)
    }

    /// Members declared by any declaration merged into a TypeScript enum.
    pub fn ts_enum_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.ts_members.enum_members(owner)
    }

    /// Finds a member declared by any declaration merged into a TypeScript enum.
    /// This returns its identity; a merged symbol can also have declarations
    /// whose visibility belongs to a different member view.
    pub fn ts_enum_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.ts_members.enum_member(owner, name)
    }

    /// The lexical binding of `name` at `scope` alone, including a hoisting
    /// `var` passing through on its way to its hoist target. This does not walk
    /// the scope chain; see [`Semantic::lookup`]. TypeScript container members
    /// are owned by their merged container environment and are therefore
    /// queried through [`Semantic::lookup`] or the container APIs instead.
    /// Source-file named exports are returned when no separate local binding
    /// owns the name; use [`Semantic::lookup`] to select a declaration space.
    /// Source-file default declarations likewise expose their local names
    /// through [`Semantic::lookup`], with separate visibility in each space.
    ///
    /// ## Example
    /// ```ts
    /// function f() { { var a; } }
    /// //             ^ binding here finds the occurrence owned by
    /// //               the function scope
    /// ```
    pub fn binding(&self, scope_id: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps[scope_id.index()]
            .get(name)
            .copied()
            .or_else(|| {
                self.hoisting_variables[scope_id.index()]
                    .as_deref()
                    .and_then(|bindings| bindings.get(name).copied())
            })
            .or_else(|| {
                (scope_id == ScopeId::MODULE)
                    .then(|| self.ts_members.source_binding(name))
                    .flatten()
            })
    }

    /// Every lexical symbol owned directly by `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through. Merged
    /// TypeScript container members are returned by the container APIs rather
    /// than projected into every declaration body. Source-file default
    /// declarations are also owned by their shared export identity. Named
    /// source-file exports are included when no separate local owns their name.
    pub fn bindings(&self, scope_id: ScopeId) -> BindingIterator<'_> {
        BindingIterator {
            inner: self.scope_maps[scope_id.index()].values(),
            source: if scope_id == ScopeId::MODULE {
                self.ts_members
                    .source_bindings(&self.scope_maps[scope_id.index()])
            } else {
                None
            },
        }
    }

    /// The nearest name-resolution candidate for `name` visible in `space`
    /// from `scope`. At each lexical layer, body-local binding candidates are
    /// considered before TypeScript container member views. A candidate
    /// outside the requested space does not shadow.
    /// [`ReferenceSpace::Any`] matches by name alone. This does not query
    /// runtime object properties when the path crosses [`scope::ScopeKind::With`].
    ///
    /// Unresolved internal TypeScript aliases remain conservative candidates.
    /// A result is not proof of their target's type or of a computed member's
    /// existence. Unknown property names permit value candidates only; they do
    /// not create type or namespace contributions. Consumers must not interpret
    /// missing member facts as absence.
    /// This query does not distinguish definite bindings from possible candidates.
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
        self.resolve_name(scope_id, name, space).candidate()
    }

    /// Looks up a name and reports whether its target is known or only possible.
    /// Unknown contributions do not prove that an outer binding is shadowed.
    /// This is name resolution; runtime interception by `with` is represented
    /// separately by [`ReferenceFlags::is_dynamic`].
    fn resolve_name(
        &self,
        scope_id: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        self.ts_members.resolve_name(
            &self.scopes,
            &self.symbols,
            &self.scope_maps,
            scope_id,
            name,
            space,
        )
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
    inner: Values<'a, &'a [u8], SymbolId>,
    source: Option<binder::TsSourceBindingIterator<'a>>,
}

impl Iterator for BindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .copied()
            .or_else(|| self.source.as_mut()?.next())
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
pub fn analyze<'a>(ast: &Ast<'a>, options: AnalyzeOptions) -> SemanticReturn<'a> {
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
    use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::{
        AnalyzeOptions, ExportOrigin, ExportTarget, ReferenceSpace, Semantic, SymbolFlags, analyze,
        scope::ScopeKind,
    };

    fn analyze_source<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        parser_options: Options,
    ) -> (Ast<'a>, Semantic<'a>) {
        analyze_with_options(allocator, source, parser_options, AnalyzeOptions::default())
    }

    fn analyze_with_options<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        parser_options: Options,
        analyze_options: AnalyzeOptions,
    ) -> (Ast<'a>, Semantic<'a>) {
        let parse = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
        let ast = parse.ast;
        assert!(parse.diagnostics.is_empty());

        let result = analyze(&ast, analyze_options);
        assert!(result.diagnostics.is_empty());
        (ast, result.semantic)
    }

    #[test]
    fn symbols() {
        let allocator = Allocator::new();
        let ts_script = Options {
            lang: Lang::Ts,
            source_type: SourceType::Script,
            ..Options::default()
        };

        let source = "interface A {} interface B {} interface A {} interface A {}";
        let (ast, semantic) = analyze_source(&allocator, source, ts_script);
        let a = semantic.binding(ScopeId::ROOT, b"A").unwrap();
        let b = semantic.binding(ScopeId::ROOT, b"B").unwrap();
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
        let js_script = Options {
            source_type: SourceType::Script,
            ..Options::default()
        };
        let ts_module = Options {
            lang: Lang::Ts,
            ..Options::default()
        };

        let source = "let value; value;";
        let (_, semantic) = analyze_source(&allocator, source, js_script);
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(reference.flags.is_read_only());

        let source = "let value; value = 1;";
        let (_, semantic) = analyze_source(&allocator, source, js_script);
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(reference.flags.is_write_only());

        let source = "let value; value++;";
        let (_, semantic) = analyze_source(&allocator, source, js_script);
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(reference.flags.is_read_write());

        let source = "let value; delete value;";
        let (_, semantic) = analyze_source(&allocator, source, js_script);
        let reference = semantic.iter_references().next().unwrap().reference;
        assert!(!reference.flags.is_read());
        assert!(!reference.flags.is_write());

        let source = "import type { T } from 'mod'; type U = T;";
        let (ast, semantic) = analyze_source(&allocator, source, ts_module);
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

        let source = "import type { T } from 'mod'; T;";
        let (ast, semantic) = analyze_source(&allocator, source, ts_module);
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
        let js_module = Options::default();
        let module_options = AnalyzeOptions {
            build_module_record: true,
            ..AnalyzeOptions::default()
        };

        let source = "export const value = 1;";
        let (_, semantic) = analyze_source(&allocator, source, js_module);
        assert!(semantic.module_record().exports().is_empty());

        let source = "export { value }; const value = 1;";
        let (ast, semantic) = analyze_with_options(&allocator, source, js_module, module_options);
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

        let source = "export { external as alias } from 'dep';";
        let (ast, semantic) = analyze_with_options(&allocator, source, js_module, module_options);
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
        let js_script = Options {
            source_type: SourceType::Script,
            ..Options::default()
        };

        let source = "let value; with ({}) { value; }";
        let (ast, semantic) = analyze_source(&allocator, source, js_script);
        let value_symbol = semantic.binding(ScopeId::ROOT, b"value").unwrap();
        let value = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"value")
            .unwrap()
            .reference;
        assert!(value.flags.is_dynamic());
        assert_eq!(value.symbol, Some(value_symbol));

        let source = "with ({}) { missing; }";
        let (_, semantic) = analyze_source(&allocator, source, js_script);
        let missing = semantic.iter_references().next().unwrap().reference;
        assert!(missing.flags.is_dynamic());
        assert!(missing.symbol.is_none());

        let source = "with ({}) { let value; value; }";
        let (ast, semantic) = analyze_source(&allocator, source, js_script);
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"value")
            .unwrap()
            .reference;
        assert!(!reference.flags.is_dynamic());
        assert!(reference.symbol.is_some());

        let source = "with ({}) { var value = 1; }";
        let (ast, semantic) = analyze_source(&allocator, source, js_script);
        let declaration = semantic.iter_declarations().next().unwrap();
        let reference_id = semantic.reference_of(declaration.declaration.node).unwrap();
        let reference = semantic.reference(reference_id);
        let symbol_id = declaration.declaration.symbol;
        assert!(reference.flags.is_write_only());
        assert!(reference.flags.is_dynamic());
        assert_eq!(reference.symbol, Some(symbol_id));
        assert_eq!(semantic.uses(symbol_id), &[reference_id]);
        assert_eq!(
            ast.get_wtf8(semantic.symbol(symbol_id).name).as_bytes(),
            b"value"
        );

        let source = "function f() { eval('direct'); }";
        let (_, semantic) = analyze_source(&allocator, source, js_script);
        let function = semantic
            .iter_scopes()
            .find(|entry| entry.scope.kind == ScopeKind::Function)
            .unwrap();
        assert!(function.scope.flags.contains_direct_eval());
        assert!(semantic.scope(ScopeId::ROOT).flags.contains_direct_eval());

        let source = "function f() { (0, eval)('indirect'); }";
        let (_, semantic) = analyze_source(&allocator, source, js_script);
        let function = semantic
            .iter_scopes()
            .find(|entry| entry.scope.kind == ScopeKind::Function)
            .unwrap();
        assert!(!function.scope.flags.contains_direct_eval());
        assert!(!semantic.scope(ScopeId::ROOT).flags.contains_direct_eval());
    }
}
