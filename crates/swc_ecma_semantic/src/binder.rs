use std::{hash::BuildHasher, iter::successors, mem::swap};

use swc_next_allocator::{
    Allocator, IndexSlice,
    hash_map::HashMap as ArenaHashMap,
    vec::{ArenaIndexVec, ArenaVec},
};

use swc_next_ecma_ast::{
    Ast, AstUtf8, AstWtf8, BindingIdentifier, Class, ClassType, ExportNamedDeclaration,
    ExportSpecifier, Function, FunctionType, IdentifierReference, ImportDeclaration,
    ImportOrExportKind, ImportSpecifier, JsxClosingElement, JsxElementName, JsxElementNameData,
    JsxIdentifier, JsxMemberExpressionObjectData, JsxOpeningElement, NodeId, NodeKind, ReferenceId,
    ScopeId, SymbolId, VariableDeclaration, VariableKind,
};

use crate::{
    DeclarationId, HoistingMap, NodeSemanticId, Redeclarations, ScopeMap, Semantic,
    scope::{Scope, ScopeKind, ScopeTree},
};

mod annex_b;
mod lookup;
mod typescript;

use typescript::{
    PendingTsNamespaceExportAlias, TsBuildState, TsDefaultLocal, TsExportAlias,
    TsExportAliasResolution, TsMemberEntry, TsMemberEnvironmentId, TsReferenceValues,
    TsValueTargetLookup, lookup_lexical_name_with_import_aliases,
    symbol_flags_clear_const_enum_only, ts_module_export_name,
};
pub(crate) use typescript::{TsAliasIssue, TsBindings, TsSourceBindingIterator};

use lookup::{LexicalLookup, NameLookupQuery, lookup_lexical_name};

use annex_b::AnnexBFunction;

/// A declared binding.
///
/// ## Example
/// ```ts
/// interface Shape { kind: string }
/// class Shape { kind = "circle" }
/// //    ^^^^^ one symbol with two declarations, occupying
/// //          both value space and type space
/// ```
///
/// Declarations that legally share a name merge into one symbol,
/// such as `var` redeclarations, TS function overloads, and `class`
/// with `interface` merging. [`Semantic::declarations`] lists every declaration
/// site.
///
/// `flags` is the aggregate of every compatible declaration's kinds,
/// modifiers, and spaces. Inspect [`Semantic::declarations`] when the
/// contribution of an individual declaration matters.
#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    /// Same-named declarations that shadow an internal name reuse its name
    /// handle, so lexical lookup can compare names without reading the AST.
    pub name: AstWtf8,
    pub flags: SymbolFlags,
    /// The first declaration contributing to this symbol.
    pub declaration: DeclarationId,
}

/// One declaration contributing to a [`Symbol`].
///
/// Unlike the aggregate flags on a symbol, `flags` describes only this
/// declaration. This preserves the distinct type, value, and namespace
/// meanings of declarations that merge into one symbol.
///
/// Annex B functions can also contribute a lexical declaration and a
/// companion var declaration to distinct symbols. Both retain the same node;
/// the node's symbol identity belongs to the lexical declaration.
#[derive(Debug, Clone, Copy)]
pub struct Declaration {
    pub node: NodeId,
    pub symbol: SymbolId,
    /// The lexical context containing this declaration. UMD global exports
    /// retain their module context without introducing a lexical binding.
    /// Canonical symbols
    /// merged across TypeScript container bodies can therefore have bindings
    /// in several scopes without assigning the symbol itself an arbitrary
    /// primary scope.
    pub scope: ScopeId,
    pub flags: SymbolFlags,
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct SymbolFlags: u32 {
        const FUNCTION_SCOPED_VAR = 1 << 0;
        const BLOCK_SCOPED_VAR = 1 << 1;
        const FUNCTION = 1 << 2;
        const CLASS = 1 << 3;
        const REGULAR_ENUM = 1 << 4;
        const CONST_ENUM = 1 << 5;
        const VALUE_MODULE = 1 << 6;
        const INTERFACE = 1 << 7;
        const TYPE_ALIAS = 1 << 8;
        const TYPE_PARAMETER = 1 << 9;
        const NAMESPACE_MODULE = 1 << 10;
        const IMPORT = 1 << 11;
        const TYPE_IMPORT = 1 << 12;
        const CONST_VAR = 1 << 13;
        const AMBIENT = 1 << 14;
        const PARAMETER = 1 << 15;
        const CATCH_VAR = 1 << 16;
        const EXPORTED = 1 << 17;
        const IS_DEFAULT = 1 << 18;
        const ENUM_MEMBER = 1 << 19;
        /// An async or generator implementation, which cannot participate in
        /// Annex B's ordinary function duplicate exception.
        const NON_ANNEX_B_FUNCTION = 1 << 20;
        /// A value module whose runtime declarations are exclusively const
        /// enums or nested const-enum-only modules. It remains visible in
        /// value space even when its default emit is erased.
        const CONST_ENUM_ONLY_MODULE = 1 << 21;
        /// A file-level `export as namespace` alias. It does not introduce a
        /// lexical binding or inherit the meanings of a same-named local.
        const UMD_EXPORT = 1 << 22;
        /// A TypeScript import-equals declaration, including type-only and
        /// external `require` aliases. Independent of target resolution.
        const IMPORT_EQUALS = 1 << 23;
        /// A named function expression's internal name. Same-named parameters
        /// and local declarations replace its scope binding with a new symbol.
        const FUNCTION_EXPRESSION = 1 << 24;
        /// A named class expression's internal name, local to its class scope.
        const CLASS_EXPRESSION = 1 << 25;
        /// The branch-local name of a sloppy if/else function declaration.
        /// Shares the function scope and is shadowed by parameters and locals.
        const ANNEX_B_IF_FUNCTION = 1 << 26;
        /// A function with a body, distinct from erased overload signatures.
        /// Used to reject duplicate implementations in lexical scopes.
        const FUNCTION_IMPLEMENTATION = 1 << 27;

        const EXPRESSION_NAME = Self::FUNCTION_EXPRESSION.bits() | Self::CLASS_EXPRESSION.bits();
        const INTERNAL_NAME = Self::EXPRESSION_NAME.bits() | Self::ANNEX_B_IF_FUNCTION.bits();

        /// `var` / `let` / `const`, parameters and catch bindings included.
        const VARIABLE = Self::FUNCTION_SCOPED_VAR.bits() | Self::BLOCK_SCOPED_VAR.bits();

        /// Any import binding, value (`import x`) or type-only (`import type x`).
        const ANY_IMPORT = Self::IMPORT.bits() | Self::TYPE_IMPORT.bits();

        /// Declarations that live in JS value space (visible at runtime).
        const VALUE_SPACE = Self::FUNCTION_SCOPED_VAR.bits()
            | Self::BLOCK_SCOPED_VAR.bits()
            | Self::FUNCTION.bits()
            | Self::CLASS.bits()
            | Self::REGULAR_ENUM.bits()
            | Self::CONST_ENUM.bits()
            | Self::VALUE_MODULE.bits()
            | Self::ENUM_MEMBER.bits();

        /// Declarations that live in TS type space.
        const TYPE_SPACE = Self::CLASS.bits()
            | Self::REGULAR_ENUM.bits()
            | Self::CONST_ENUM.bits()
            | Self::INTERFACE.bits()
            | Self::TYPE_ALIAS.bits()
            | Self::TYPE_PARAMETER.bits()
            | Self::ENUM_MEMBER.bits();

        /// Declarations a dotted type name can start from.
        const NAMESPACE_SPACE = Self::VALUE_MODULE.bits()
            | Self::NAMESPACE_MODULE.bits()
            | Self::REGULAR_ENUM.bits()
            | Self::CONST_ENUM.bits();

        const BLOCK_SCOPED_LIKE = Self::VALUE_SPACE.bits() & !Self::FUNCTION_SCOPED_VAR.bits();
    }
}

impl SymbolFlags {
    pub const NONE: Self = Self::empty();

    /// The union of two flag sets. Used when merging two compatible
    /// declarations into a single symbol.
    pub fn merge(self, other: Self) -> Self {
        self | other
    }

    fn with(mut self, flag: Self, value: bool) -> Self {
        self.set(flag, value);
        self
    }

    /// True for a `var` that hoists past intermediate blocks.
    /// Parameters and catch variables are function-scoped but do
    /// not hoist, so they return false here.
    pub fn is_hoisting_var(self) -> bool {
        self.contains(Self::FUNCTION_SCOPED_VAR)
            && !self.intersects(Self::PARAMETER | Self::CATCH_VAR)
    }

    /// True for a JavaScript value-space binding visible at runtime,
    /// meaning `var`/`let`/`const`, function, class, enum, or an
    /// instantiated namespace.
    pub fn in_value_space(self) -> bool {
        self.intersects(Self::VALUE_SPACE)
    }

    /// True for a TypeScript type-space binding, meaning interface,
    /// type alias, type parameter, plus class and enum, which exist in
    /// both spaces.
    pub fn in_type_space(self) -> bool {
        self.intersects(Self::TYPE_SPACE)
    }

    /// True for a binding a dotted type name can start from:
    /// namespaces and enums.
    pub fn in_namespace_space(self) -> bool {
        self.intersects(Self::NAMESPACE_SPACE)
    }

    /// Conservative visibility based on declaration flags alone. Resolved
    /// aliases and namespace/enum member views can narrow this result;
    /// use [`Semantic::lookup`] for name resolution in a scope.
    ///
    /// ## Example
    /// ```ts
    /// type T = string;
    /// function f() {
    ///   const T = 1;
    ///   let x: T;
    ///   // type space finds the outer alias, value space the local const
    /// }
    /// ```
    ///
    /// Value imports alias symbols of unknowable space and are visible in
    /// every space. Type-only imports are not visible to runtime value uses,
    /// but remain valid in type, namespace, and `typeof` positions.
    pub fn visible_in(self, space: ReferenceSpace) -> bool {
        match space {
            ReferenceSpace::Value => self.contains(Self::IMPORT) || self.in_value_space(),
            ReferenceSpace::Typeof => self.intersects(Self::ANY_IMPORT) || self.in_value_space(),
            ReferenceSpace::Type => self.intersects(Self::ANY_IMPORT) || self.in_type_space(),
            ReferenceSpace::Namespace => {
                self.intersects(Self::ANY_IMPORT) || self.in_namespace_space()
            }
            ReferenceSpace::Any => true,
        }
    }

    /// True for declarations a hoisting `var` is forbidden to pass
    /// through: block-scoped bindings (`let`, `const`), classes,
    /// functions, enums, and instantiated namespaces.
    pub fn is_block_scoped_like(self) -> bool {
        self.intersects(Self::BLOCK_SCOPED_LIKE)
    }

    /// Human-readable category for diagnostics.
    pub fn diagnostic_kind(self) -> &'static str {
        if self.contains(Self::FUNCTION) {
            "function"
        } else if self.contains(Self::CLASS) {
            "class"
        } else if self.intersects(Self::REGULAR_ENUM | Self::CONST_ENUM) {
            "enum"
        } else if self.intersects(Self::VALUE_MODULE | Self::NAMESPACE_MODULE) {
            "namespace"
        } else if self.contains(Self::INTERFACE) {
            "interface"
        } else if self.contains(Self::TYPE_ALIAS) {
            "type alias"
        } else if self.contains(Self::TYPE_IMPORT) {
            "type import"
        } else if self.contains(Self::IMPORT) {
            "import"
        } else if self.contains(Self::UMD_EXPORT) {
            "global export alias"
        } else if self.contains(Self::PARAMETER) {
            "parameter"
        } else if self.contains(Self::CATCH_VAR) {
            "catch parameter"
        } else if self.contains(Self::TYPE_PARAMETER) {
            "type parameter"
        } else if self.contains(Self::ENUM_MEMBER) {
            "enum member"
        } else {
            "variable"
        }
    }
}

/// Per-declaration redeclaration excludes. A new declaration using one
/// of these sets conflicts with any matching existing flag. Otherwise
/// both declarations merge into one symbol.
pub struct Excludes;

impl Excludes {
    pub const BLOCK_SCOPED_VAR: SymbolFlags = SymbolFlags::VALUE_SPACE;
    pub const FUNCTION_SCOPED_VAR: SymbolFlags = SymbolFlags::BLOCK_SCOPED_LIKE;
    /// TS functions allow overloads and namespace/class merges. JS hoist
    /// scopes additionally permit merging with `var`.
    pub const FUNCTION: SymbolFlags = SymbolFlags::VALUE_SPACE.difference(
        SymbolFlags::FUNCTION
            .union(SymbolFlags::VALUE_MODULE)
            .union(SymbolFlags::CLASS),
    );
    /// TypeScript binds class/function declarations into one symbol before
    /// checking whether their ambient/body combination is legal.
    pub const CLASS: SymbolFlags = SymbolFlags::VALUE_SPACE
        .union(SymbolFlags::TYPE_SPACE)
        .difference(
            SymbolFlags::VALUE_MODULE
                .union(SymbolFlags::INTERFACE)
                .union(SymbolFlags::FUNCTION),
        );
    /// JavaScript has no declaration-merging phase for classes and functions.
    pub const JS_CLASS: SymbolFlags = Self::CLASS.union(SymbolFlags::FUNCTION);
    pub const INTERFACE: SymbolFlags = SymbolFlags::from_bits_retain(
        SymbolFlags::REGULAR_ENUM.bits()
            | SymbolFlags::CONST_ENUM.bits()
            | SymbolFlags::TYPE_ALIAS.bits()
            | SymbolFlags::TYPE_PARAMETER.bits()
            | SymbolFlags::ENUM_MEMBER.bits(),
    );
    pub const TYPE_ALIAS: SymbolFlags = SymbolFlags::TYPE_SPACE;
    pub const REGULAR_ENUM: SymbolFlags = SymbolFlags::from_bits_retain(
        SymbolFlags::FUNCTION_SCOPED_VAR.bits()
            | SymbolFlags::BLOCK_SCOPED_VAR.bits()
            | SymbolFlags::FUNCTION.bits()
            | SymbolFlags::CLASS.bits()
            | SymbolFlags::CONST_ENUM.bits()
            | SymbolFlags::INTERFACE.bits()
            | SymbolFlags::TYPE_ALIAS.bits()
            | SymbolFlags::TYPE_PARAMETER.bits()
            | SymbolFlags::ENUM_MEMBER.bits(),
    );
    pub const CONST_ENUM: SymbolFlags = SymbolFlags::from_bits_retain(
        SymbolFlags::FUNCTION_SCOPED_VAR.bits()
            | SymbolFlags::BLOCK_SCOPED_VAR.bits()
            | SymbolFlags::FUNCTION.bits()
            | SymbolFlags::CLASS.bits()
            | SymbolFlags::REGULAR_ENUM.bits()
            | SymbolFlags::VALUE_MODULE.bits()
            | SymbolFlags::INTERFACE.bits()
            | SymbolFlags::TYPE_ALIAS.bits()
            | SymbolFlags::TYPE_PARAMETER.bits()
            | SymbolFlags::ENUM_MEMBER.bits(),
    );
    pub const VALUE_MODULE: SymbolFlags = SymbolFlags::from_bits_retain(
        SymbolFlags::FUNCTION_SCOPED_VAR.bits()
            | SymbolFlags::BLOCK_SCOPED_VAR.bits()
            | SymbolFlags::CONST_ENUM.bits()
            | SymbolFlags::ENUM_MEMBER.bits(),
    );
    pub const NAMESPACE_MODULE: SymbolFlags = SymbolFlags::NONE;
    pub const IMPORT_BINDING: SymbolFlags = SymbolFlags::ANY_IMPORT;
    pub const PARAMETER: SymbolFlags =
        SymbolFlags::VALUE_SPACE.difference(SymbolFlags::FUNCTION_SCOPED_VAR);
    pub const CATCH_VAR: SymbolFlags = SymbolFlags::VALUE_SPACE;
    // Type parameters merge (multiple `infer T` in one conditional
    // unify). Duplicate explicit `<T, T>` is caught structurally.
    pub const TYPE_PARAMETER: SymbolFlags =
        SymbolFlags::TYPE_SPACE.difference(SymbolFlags::TYPE_PARAMETER);
    pub const ENUM_MEMBER: SymbolFlags = SymbolFlags::from_bits_retain(
        SymbolFlags::VALUE_SPACE.bits()
            | SymbolFlags::INTERFACE.bits()
            | SymbolFlags::TYPE_ALIAS.bits()
            | SymbolFlags::TYPE_PARAMETER.bits(),
    );
}

/// A use of a name. One is recorded for every `identifier_reference`
/// in the source, for the local name in an `export { name }` specifier,
/// for JSX component tag names (`<Foo>`), and for TS type-predicate parameters
/// (`x is T`). Re-export names in `export { name } from "module"` are external
/// names, not references.
///
/// ## Example
/// ```ts
/// let a = b; a = 2;
/// //      ^ a read of `b`
/// //         ^ a write of `a`, flags.is_write()
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Reference {
    pub name: AstUtf8,
    /// The scope the reference appears in.
    pub scope: ScopeId,
    /// The referencing node.
    pub node: NodeId,
    /// The nearest canonical name-resolution candidate. `None` for names with
    /// no lexical binding or TypeScript container member visible in the
    /// reference's space.
    ///
    /// When [`ReferenceFlags::is_uncertain`] is true, this candidate may not
    /// occupy the requested space. [`Semantic::reference_candidates`] includes
    /// the other possible targets; [`Reference::resolution`] exposes certainty.
    pub symbol: Option<SymbolId>,
    pub flags: ReferenceFlags,
}

impl Reference {
    /// Name-resolution certainty.
    pub fn resolution(self) -> NameResolution {
        match self.symbol {
            Some(candidate) if self.flags.is_uncertain() => NameResolution::Uncertain { candidate },
            Some(symbol) => NameResolution::Resolved(symbol),
            None => NameResolution::Unresolved,
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct ReferenceProperties: u8 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const UNCERTAIN = 1 << 2;
    }
}

/// Whether evaluating a reference reads its previous value, writes a new
/// value, or both. Type-only references do neither.
///
/// Syntactic forms such as assignments, updates, destructuring targets, and
/// for-in/of targets are normalized to these access operations rather than
/// being exposed as separate reference kinds. Alias positions retain a read
/// only when their resolved symbol can contribute a runtime value; unresolved
/// aliases conservatively retain it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceFlags {
    properties: ReferenceProperties,
    /// The declaration space this position resolves in.
    pub space: ReferenceSpace,
}

impl Default for ReferenceFlags {
    fn default() -> Self {
        Self {
            properties: ReferenceProperties::READ,
            space: ReferenceSpace::Value,
        }
    }
}

impl ReferenceFlags {
    /// True when evaluation reads the binding's value.
    pub fn is_read(self) -> bool {
        self.properties.contains(ReferenceProperties::READ)
    }

    /// True when evaluation assigns to the binding.
    pub fn is_write(self) -> bool {
        self.properties.contains(ReferenceProperties::WRITE)
    }

    /// True when the same reference both reads and writes the binding,
    /// as in `value += 1` or `value++`.
    pub fn is_read_write(self) -> bool {
        self.properties
            .contains(ReferenceProperties::READ | ReferenceProperties::WRITE)
    }

    /// True when evaluation only reads the binding.
    pub fn is_read_only(self) -> bool {
        self.is_read() && !self.is_write()
    }

    /// True when evaluation only writes the binding.
    pub fn is_write_only(self) -> bool {
        self.is_write() && !self.is_read()
    }

    /// True when opaque TypeScript alias or property facts leave more than
    /// one possible lookup outcome, including the absence of a local target.
    /// Such a reference is included in every possible target's use index.
    pub fn is_uncertain(self) -> bool {
        self.properties.contains(ReferenceProperties::UNCERTAIN)
    }

    /// Changes name lookup without changing runtime access properties.
    pub(crate) fn with_space(mut self, space: ReferenceSpace) -> Self {
        self.space = space;
        self
    }

    pub fn write_only(mut self) -> Self {
        self.properties = ReferenceProperties::WRITE;
        self
    }

    pub(crate) fn read_write(mut self) -> Self {
        self.properties = ReferenceProperties::READ | ReferenceProperties::WRITE;
        self
    }

    pub(crate) fn no_access(mut self) -> Self {
        self.properties
            .remove(ReferenceProperties::READ | ReferenceProperties::WRITE);
        self
    }

    fn with_uncertain(mut self) -> Self {
        self.properties.insert(ReferenceProperties::UNCERTAIN);
        self
    }
}

/// The declaration space a syntactic position resolves in,
/// matching TypeScript name resolution. A binding outside a reference's
/// space does not shadow.
///
/// ## Example
/// ```ts
/// let a: T = v;
/// //     ^ type
/// //         ^ value
/// let b: N.T;
/// //     ^ namespace
/// let c: typeof v;
/// //            ^ typeof
/// export { T };
/// //       ^ any
/// ```
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum ReferenceSpace {
    /// a runtime use
    #[default]
    Value,
    /// a type use, annotations, heritage clauses, type arguments
    Type,
    /// the qualifier of a dotted type name, `ns` in `ns.T`, or the root
    /// of an internal import-equals target
    Namespace,
    /// a value use inside a type, the entity of a `typeof` query or a
    /// type-predicate parameter
    Typeof,
    /// an alias position that accepts every space: `export { x }` and
    /// `import a = x`
    Any,
}

impl ReferenceSpace {
    pub(crate) fn value_target_space(self) -> Self {
        match self {
            Self::Typeof => Self::Typeof,
            Self::Value | Self::Type | Self::Namespace | Self::Any => Self::Value,
        }
    }

    // None denotes the unrestricted alias space, including empty flag sets.
    fn symbol_mask(self) -> Option<SymbolFlags> {
        Some(match self {
            Self::Value => SymbolFlags::IMPORT | SymbolFlags::VALUE_SPACE,
            Self::Typeof => SymbolFlags::ANY_IMPORT | SymbolFlags::VALUE_SPACE,
            Self::Type => SymbolFlags::ANY_IMPORT | SymbolFlags::TYPE_SPACE,
            Self::Namespace => SymbolFlags::ANY_IMPORT | SymbolFlags::NAMESPACE_SPACE,
            Self::Any => return None,
        })
    }

    /// Spaces used when traversing type syntax. This alone does not determine
    /// runtime access: `Namespace` also occurs in internal import aliases.
    pub fn in_type_position(self) -> bool {
        matches!(self, Self::Type | Self::Namespace | Self::Typeof)
    }
}

/// The certainty of a name lookup in its requested declaration space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameResolution {
    /// No lexical binding or container member is visible in this space.
    Unresolved,
    /// The candidate is known to occupy the requested space.
    Resolved(SymbolId),
    /// An opaque alias or computed name may occupy this space. The nearest
    /// candidate is retained, but outer bindings may also be referenced.
    Uncertain { candidate: SymbolId },
}

impl NameResolution {
    /// The definite target, or `None` when lookup is unresolved or uncertain.
    pub fn symbol(self) -> Option<SymbolId> {
        match self {
            Self::Resolved(symbol) => Some(symbol),
            Self::Unresolved | Self::Uncertain { .. } => None,
        }
    }

    /// The nearest possible target, including an uncertain candidate.
    pub fn candidate(self) -> Option<SymbolId> {
        match self {
            Self::Resolved(symbol) | Self::Uncertain { candidate: symbol } => Some(symbol),
            Self::Unresolved => None,
        }
    }
}

/// A `(start, len)` window into a backing slice.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Range {
    pub(crate) start: u32,
    pub(crate) len: u32,
}

type UseRanges = IndexSlice<SymbolId, [Range]>;

/// Additional targets of uncertain references. The nearest candidate stays
/// on Reference, so ordinary references need no entry or per-reference slice.
#[derive(Debug)]
pub(crate) struct ReferenceCandidates<'a> {
    ranges: ArenaHashMap<'a, ReferenceId, Range>,
    symbols: ArenaVec<'a, SymbolId>,
}

impl<'a> ReferenceCandidates<'a> {
    fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            ranges: ArenaHashMap::new_in(allocator),
            symbols: ArenaVec::new_in(allocator),
        }
    }

    pub(crate) fn additional(&self, reference: ReferenceId) -> &[SymbolId] {
        let Some(range) = self.ranges.get(&reference) else {
            return &[];
        };
        let start = range.start as usize;
        &self.symbols[start..start + range.len as usize]
    }

    #[inline]
    fn record(
        &mut self,
        id: ReferenceId,
        reference: &mut Reference,
        resolution: NameResolution,
        candidates: &mut Vec<SymbolId>,
        use_ranges: &mut UseRanges,
    ) {
        reference.symbol = resolution.candidate();
        if let Some(symbol) = reference.symbol {
            use_ranges[symbol].len += 1;
        }
        if let NameResolution::Uncertain { candidate } = resolution {
            reference.flags = reference.flags.with_uncertain();
            self.record_additional(id, candidate, candidates, use_ranges);
        }
    }

    #[cold]
    fn record_additional(
        &mut self,
        id: ReferenceId,
        candidate: SymbolId,
        candidates: &mut Vec<SymbolId>,
        use_ranges: &mut UseRanges,
    ) {
        candidates.retain(|&symbol| symbol != candidate);
        // A merged symbol can be encountered through both lexical and
        // member views. Count each reference once per possible target.
        candidates.sort_unstable();
        candidates.dedup();
        if candidates.is_empty() {
            return;
        }
        self.ranges.insert(
            id,
            Range {
                start: self.symbols.len() as u32,
                len: candidates.len() as u32,
            },
        );
        self.symbols.extend(candidates.iter().copied());
        for &symbol in candidates.iter() {
            use_ranges[symbol].len += 1;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SymbolLink {
    parent: SymbolId,
    next: SymbolId,
    size: usize,
}

impl SymbolLink {
    fn singleton(symbol: SymbolId) -> Self {
        Self {
            parent: symbol,
            next: symbol,
            size: 1,
        }
    }
}

#[derive(Debug)]
pub(crate) struct SymbolLinks<'a> {
    links: ArenaHashMap<'a, SymbolId, SymbolLink>,
}

impl<'a> SymbolLinks<'a> {
    pub(crate) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            links: ArenaHashMap::new_in(allocator),
        }
    }

    pub(crate) fn symbols(&self, symbol: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        successors(Some(symbol), move |current| {
            self.links
                .get(current)
                .map(|link| link.next)
                .filter(|&next| next != symbol)
        })
    }

    pub(crate) fn link(&mut self, left: SymbolId, right: SymbolId) {
        if left == right {
            return;
        }
        let mut left_root = self.root(left);
        let mut right_root = self.root(right);
        if left_root == right_root {
            return;
        }

        // Splice the two circular member lists without visiting their members.
        let left_next = self.links.get(&left).map_or(left, |link| link.next);
        let right_next = self.links.get(&right).map_or(right, |link| link.next);
        self.links
            .entry(left)
            .or_insert_with(|| SymbolLink::singleton(left))
            .next = right_next;
        self.links
            .entry(right)
            .or_insert_with(|| SymbolLink::singleton(right))
            .next = left_next;

        // Union by size bounds future root lookups; ordinary singleton symbols
        // have no entry and allocate no constraint storage.
        if self.links[&left_root].size < self.links[&right_root].size {
            swap(&mut left_root, &mut right_root);
        }
        let right_size = self.links[&right_root].size;
        self.links.get_mut(&left_root).unwrap().size += right_size;
        self.links.get_mut(&right_root).unwrap().parent = left_root;
    }

    fn root(&self, mut symbol: SymbolId) -> SymbolId {
        while let Some(link) = self.links.get(&symbol) {
            if link.parent == symbol {
                break;
            }
            symbol = link.parent;
        }
        symbol
    }
}

/// Collects symbols and references during the AST walk and finalizes
/// them into a [`Semantic`].
pub(crate) struct SymbolTracker<'a> {
    allocator: &'a Allocator,
    internal_names: crate::InternalNames<'a>,
    symbols: ArenaIndexVec<'a, SymbolId, Symbol>,
    symbol_links: SymbolLinks<'a>,
    references: ArenaIndexVec<'a, ReferenceId, Reference>,
    jsx_usages: ArenaVec<'a, crate::JsxUsage>,
    declarations: ArenaIndexVec<'a, DeclarationId, Declaration>,
    redeclarations: Redeclarations<'a>,
    node_semantic: ArenaIndexVec<'a, NodeId, NodeSemanticId>,
    scope_maps: ArenaIndexVec<'a, ScopeId, ScopeMap<'a>>,
    hoisting_variables: ArenaIndexVec<'a, ScopeId, HoistingMap<'a>>,
    annex_b_functions: ArenaVec<'a, AnnexBFunction>,
    ts: TsBindings<'a>,
    ts_build: TsBuildState<'a>,

    /// What the next `binding_identifier` will declare: its flags,
    /// redeclaration excludes, and destination scope. Valid inside an
    /// enter hook on a `binding_identifier`.
    pub(crate) pending: PendingBinding,
    /// Whether the next `binding_identifier` is the directly-exported
    /// name of an `export` declaration.
    pub(crate) export_state: ExportState,
    // true inside an ambient (`declare`) context
    ambient: bool,
    // snapshots taken before declaration nodes rewrite the context,
    // restored on exit
    saved_stack: Vec<SavedContext>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ExportState {
    #[default]
    None,
    Named,
    Default,
}

impl ExportState {
    fn is_exported(self) -> bool {
        !matches!(self, Self::None)
    }

    fn is_default(self) -> bool {
        matches!(self, Self::Default)
    }

    pub(crate) fn is_named(self) -> bool {
        matches!(self, Self::Named)
    }
}

/// The declaration context for the next `binding_identifier`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingBinding {
    pub flags: SymbolFlags,
    pub excludes: SymbolFlags,
    pub scope: ScopeId,
}

#[derive(Clone, Copy)]
struct PendingDeclaration {
    name: AstWtf8,
    node: NodeId,
    binding: PendingBinding,
}

#[derive(Clone, Copy)]
enum DeclarationDestination {
    Scope,
    TsDefaultExport,
    TsTypeParameter {
        owner: SymbolId,
    },
    TsMember {
        environment: TsMemberEnvironmentId,
        // Synthetic qualified-name members have no lexical binding scope.
        // Ordinary namespace and enum members retain their body scope on the
        // declaration occurrence.
        body_scope: Option<ScopeId>,
        // Global augmentations are canonical members and global bindings at
        // the same time. Their environment and root scope therefore share a
        // symbol instead of adding a root lookup overlay.
        projection_scope: Option<ScopeId>,
    },
}

#[derive(Clone, Copy)]
pub(crate) struct PreparedDeclaration {
    declaration: PendingDeclaration,
    destination: DeclarationDestination,
    // Keep the body-local and canonical member candidates separate. They can
    // legally occupy disjoint TypeScript declaration spaces under one name.
    local: Option<SymbolId>,
    shared: Option<SymbolId>,
    // The symbol that receives this declaration after conflict checking.
    existing: Option<SymbolId>,
    default_local: Option<(SymbolId, TsDefaultLocal)>,
}

#[derive(Clone, Copy)]
pub(crate) struct DeclarationConflictCandidate {
    pub(crate) symbol: SymbolId,
    kind: DeclarationConflictKind,
}

#[derive(Clone, Copy)]
enum DeclarationConflictKind {
    Symbol,
    // A private lexical declaration in a namespace body only conflicts with
    // the canonical member contributions declared in that same body.
    Scope(ScopeId),
    DefaultLocal(TsDefaultLocal),
}

impl PreparedDeclaration {
    pub(crate) fn name(self) -> AstWtf8 {
        self.declaration.name
    }

    pub(crate) fn node(self) -> NodeId {
        self.declaration.node
    }

    pub(crate) fn target(self) -> ScopeId {
        self.declaration.binding.scope
    }

    pub(crate) fn flags(self) -> SymbolFlags {
        self.declaration.binding.flags
    }

    pub(crate) fn excludes(self) -> SymbolFlags {
        self.declaration.binding.excludes
    }

    pub(crate) fn conflict_candidates(self) -> impl Iterator<Item = DeclarationConflictCandidate> {
        let local = self.local.map(|symbol| DeclarationConflictCandidate {
            symbol,
            kind: DeclarationConflictKind::Symbol,
        });
        // A shared default export key is not a lexical redeclaration. TS
        // duplicate exports belong to type checking; the local-name candidate
        // below still checks declarations that actually share a binding name.
        let distinct_shared = self.shared.filter(|&shared| {
            Some(shared) != self.local
                && !matches!(self.destination, DeclarationDestination::TsDefaultExport)
        });
        let shared_scope = matches!(self.destination, DeclarationDestination::Scope)
            .then_some(self.declaration.binding.scope);
        let shared = distinct_shared.map(|symbol| DeclarationConflictCandidate {
            symbol,
            kind: shared_scope.map_or(
                DeclarationConflictKind::Symbol,
                DeclarationConflictKind::Scope,
            ),
        });
        let default_local =
            self.default_local
                .map(|(symbol, local)| DeclarationConflictCandidate {
                    symbol,
                    kind: DeclarationConflictKind::DefaultLocal(local),
                });
        local.into_iter().chain(shared).chain(default_local)
    }
}

impl Default for PendingBinding {
    fn default() -> Self {
        Self {
            flags: SymbolFlags::NONE,
            excludes: SymbolFlags::NONE,
            scope: ScopeId::ROOT,
        }
    }
}

#[derive(Clone, Copy)]
struct SavedContext {
    pending: PendingBinding,
    export_state: ExportState,
    ambient: bool,
}

impl<'a> SymbolTracker<'a> {
    pub fn init(tree: &Ast<'a>) -> Self {
        let allocator = tree.allocator();
        let nodes = tree.node_count();
        let mut node_semantic = ArenaIndexVec::with_capacity_in(nodes, allocator);
        node_semantic.resize(nodes, NodeSemanticId::None);
        let mut tracker = Self {
            allocator,
            internal_names: crate::InternalNames::new_in(allocator),
            symbols: ArenaIndexVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            symbol_links: SymbolLinks::new_in(allocator),
            references: ArenaIndexVec::with_capacity_in(16usize.max(nodes / 4), allocator),
            jsx_usages: ArenaVec::new_in(allocator),
            declarations: ArenaIndexVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            redeclarations: Redeclarations::new_in(allocator),
            node_semantic,
            scope_maps: ArenaIndexVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            hoisting_variables: ArenaIndexVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            annex_b_functions: ArenaVec::new_in(allocator),
            ts: TsBindings::new_in(allocator),
            ts_build: TsBuildState::new_in(allocator),
            pending: PendingBinding::default(),
            export_state: ExportState::None,
            // A declaration file is ambient in its entirety.
            ambient: tree.lang() == swc_next_ecma_ast::Lang::Dts,
            saved_stack: Vec::with_capacity(32),
        };
        if tree.is_ts() && tree.is_module() {
            tracker.init_ts_source_exports(tree);
        }
        tracker
    }

    pub fn enter_export_named_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: ExportNamedDeclaration,
    ) {
        // The declaration form always exports its binding, even when
        // `export_kind` is type. The pure re-export form has no inner
        // binding and remains dependent on its export kind.
        if declaration.declaration(tree).is_some()
            || declaration.export_kind(tree) != ImportOrExportKind::Type
        {
            self.export_state = ExportState::Named;
        }
    }

    pub fn enter_export_default_declaration(&mut self) {
        self.export_state = ExportState::Default;
    }

    pub fn exit_export_declaration(&mut self) {
        self.export_state = ExportState::None;
    }

    pub fn enter_variable_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: VariableDeclaration,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        let kind = declaration.kind(tree);
        self.pending = match kind {
            VariableKind::Var => {
                let target = scope.get(current_scope).hoist_target;
                let mut excludes = Excludes::FUNCTION_SCOPED_VAR;
                // JS permits var/function merges in hoist scopes, except modules.
                if !tree.is_ts() && scope.get(target).kind != ScopeKind::Module {
                    excludes.remove(SymbolFlags::FUNCTION);
                }
                PendingBinding {
                    flags: SymbolFlags::FUNCTION_SCOPED_VAR.with(
                        SymbolFlags::AMBIENT,
                        declaration.declare(tree) || self.ambient,
                    ),
                    excludes,
                    scope: target,
                }
            }
            VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing => {
                PendingBinding {
                    flags: (SymbolFlags::BLOCK_SCOPED_VAR | SymbolFlags::CONST_VAR).with(
                        SymbolFlags::AMBIENT,
                        declaration.declare(tree) || self.ambient,
                    ),
                    excludes: Excludes::BLOCK_SCOPED_VAR,
                    scope: current_scope,
                }
            }
            VariableKind::Let => PendingBinding {
                flags: SymbolFlags::BLOCK_SCOPED_VAR.with(
                    SymbolFlags::AMBIENT,
                    declaration.declare(tree) || self.ambient,
                ),
                excludes: Excludes::BLOCK_SCOPED_VAR,
                scope: current_scope,
            },
        };
    }

    pub fn enter_function(
        &mut self,
        tree: &Ast<'a>,
        function: Function,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
        annex_b_if_body: bool,
    ) {
        self.push_saved_context();
        let is_declaration = function.function_type(tree) == FunctionType::FunctionDeclaration;
        let enclosing = scope
            .get(current_scope)
            .parent
            .expect("a function scope has an enclosing scope");
        // Declaration placement follows the enclosing code, independently
        // of a "use strict" directive in the function's own body.
        let strict = scope.get(enclosing).flags.strict;
        let ordinary = !function.generator(tree) && !function.r#async(tree);
        // Preserve the branch-local binding without allocating an implicit
        // block. Like an expression name, it is distinct from parameters,
        // body declarations and same-named declarations in the outer scope.
        let branch_local = annex_b_if_body && !strict && ordinary;
        let target = if is_declaration && !branch_local {
            enclosing
        } else {
            current_scope
        };

        let target_kind = scope.get(target).kind;
        let annex_b_block_function = is_declaration
            && (target_kind == ScopeKind::Block || branch_local)
            && !strict
            && ordinary;
        if annex_b_block_function
            && !self.ambient
            && let Some(name) = function.id(tree)
        {
            self.annex_b_functions.push(AnnexBFunction {
                name,
                scope: enclosing,
            });
        }
        let hoist_scope = target_kind.is_hoist_target() && target_kind != ScopeKind::Module;
        // TS signatures merge with implementations. Duplicate implementations
        // follow the same hoist-scope and Annex B rules as JavaScript.
        self.pending = PendingBinding {
            flags: (SymbolFlags::FUNCTION | SymbolFlags::FUNCTION_IMPLEMENTATION)
                .with(SymbolFlags::FUNCTION_EXPRESSION, !is_declaration)
                .with(SymbolFlags::ANNEX_B_IF_FUNCTION, branch_local)
                .with(SymbolFlags::NON_ANNEX_B_FUNCTION, !ordinary)
                .with(SymbolFlags::AMBIENT, self.ambient),
            excludes: Excludes::FUNCTION
                .with(SymbolFlags::CLASS, !tree.is_ts())
                .with(
                    SymbolFlags::FUNCTION_SCOPED_VAR,
                    tree.is_ts() || !hoist_scope,
                )
                .with(
                    SymbolFlags::FUNCTION_IMPLEMENTATION,
                    !hoist_scope && !annex_b_block_function,
                )
                .with(SymbolFlags::NON_ANNEX_B_FUNCTION, annex_b_block_function),
            scope: target,
        };
        // Expression names are local.
        if !is_declaration {
            self.export_state = ExportState::None;
        } else if function.id(tree).is_none() {
            self.declare_anonymous_default(tree, function.node_id());
        }
    }

    pub fn enter_class(
        &mut self,
        tree: &Ast<'a>,
        class: Class,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        let is_declaration = class.class_type(tree) == ClassType::ClassDeclaration;
        self.pending = PendingBinding {
            flags: SymbolFlags::CLASS
                .with(SymbolFlags::CLASS_EXPRESSION, !is_declaration)
                .with(SymbolFlags::AMBIENT, class.declare(tree) || self.ambient),
            excludes: if tree.is_ts() {
                Excludes::CLASS
            } else {
                Excludes::JS_CLASS
            },
            scope: if is_declaration {
                scope
                    .get(current_scope)
                    .parent
                    .expect("a class declaration scope has a parent")
            } else {
                current_scope
            },
        };
        // Expression names are local.
        if !is_declaration {
            self.export_state = ExportState::None;
        } else if class.id(tree).is_none() {
            self.declare_anonymous_default(tree, class.node_id());
        }
    }

    pub fn enter_formal_parameters(&mut self, current_scope: ScopeId) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: (SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER).with(
                SymbolFlags::AMBIENT,
                self.pending.flags.contains(SymbolFlags::AMBIENT),
            ),
            excludes: Excludes::PARAMETER,
            scope: current_scope,
        };
        self.export_state = ExportState::None;
    }

    /// Restores the context to what it was before the matching enter.
    pub fn exit_formal_parameters(&mut self) {
        let saved = self
            .saved_stack
            .pop()
            .expect("formal parameters have a saved binding context");
        self.pending = saved.pending;
        self.ambient = saved.ambient;
    }

    pub fn enter_arrow_function_expression(&mut self) -> ExportState {
        std::mem::take(&mut self.export_state)
    }

    pub fn exit_arrow_function_expression(&mut self, saved: ExportState) {
        debug_assert_eq!(self.export_state, ExportState::None);
        self.export_state = saved;
    }

    // Members are not the exported binding.
    pub fn enter_class_body(&mut self) {
        self.export_state = ExportState::None;
    }

    pub fn enter_import_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: ImportDeclaration,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        self.pending = import_pending(declaration.import_kind(tree), current_scope);
    }

    pub fn enter_import_specifier(&mut self, tree: &Ast, specifier: ImportSpecifier) {
        self.push_saved_context();
        if specifier.import_kind(tree) == ImportOrExportKind::Type {
            self.pending.flags = SymbolFlags::TYPE_IMPORT;
            self.pending.excludes = Excludes::IMPORT_BINDING;
        }
    }

    pub fn enter_catch_clause(&mut self, current_scope: ScopeId) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::CATCH_VAR,
            excludes: Excludes::CATCH_VAR,
            scope: current_scope,
        };
    }

    pub fn exit_saved_context(&mut self) {
        let saved = self
            .saved_stack
            .pop()
            .expect("binding context entry and exit are paired");
        self.pending = saved.pending;
        self.export_state = saved.export_state;
        self.ambient = saved.ambient;
    }

    fn push_saved_context(&mut self) {
        self.saved_stack.push(SavedContext {
            pending: self.pending,
            export_state: self.export_state,
            ambient: self.ambient,
        });
    }

    pub(crate) fn prepare_declaration(
        &self,
        tree: &Ast<'a>,
        name: AstWtf8,
        node: NodeId,
    ) -> PreparedDeclaration {
        let name_text = tree.get_wtf8(name).as_bytes();
        let local = self
            .own_binding(self.pending.scope, name_text)
            .filter(|&symbol| {
                !self.symbols[symbol]
                    .flags
                    .intersects(SymbolFlags::INTERNAL_NAME)
            });
        let (destination, shared, existing) = if self.pending_is_default_export(tree) {
            let shared = self
                .ts
                .source_exports
                .as_ref()
                .and_then(|source| source.default.as_ref())
                .map(|default| default.symbol);
            (DeclarationDestination::TsDefaultExport, shared, shared)
        } else if let Some(owner) = self.pending_type_parameter_owner() {
            let shared = self
                .ts_build
                .type_parameters
                .get(&owner)
                .and_then(|parameters| parameters.get(name_text).copied());
            (
                DeclarationDestination::TsTypeParameter { owner },
                shared,
                local.or(shared),
            )
        } else if let Some(environment) = self.pending_ts_source_destination() {
            let shared = self.ts.environments.member(environment, name_text);
            (
                DeclarationDestination::TsMember {
                    environment,
                    body_scope: Some(ScopeId::MODULE),
                    projection_scope: None,
                },
                shared,
                shared,
            )
        } else if let Some((environment, projection_scope)) =
            self.pending_ts_namespace_destination()
        {
            let shared = self
                .ts
                .environments
                .member(environment, name_text)
                .or_else(|| projection_scope.and_then(|scope| self.binding(scope, name_text)));
            (
                DeclarationDestination::TsMember {
                    environment,
                    body_scope: Some(self.pending.scope),
                    projection_scope,
                },
                shared,
                shared,
            )
        } else {
            let shared = self.current_ts_body_member(self.pending.scope, name_text);
            (DeclarationDestination::Scope, shared, local)
        };

        let mut flags = self.pending.flags;
        let exported = self.export_state.is_exported()
            || matches!(destination, DeclarationDestination::TsMember { .. });
        flags.set(SymbolFlags::EXPORTED, exported);
        flags.set(SymbolFlags::IS_DEFAULT, self.export_state.is_default());
        let declaration = PendingDeclaration {
            name,
            node,
            binding: PendingBinding {
                flags,
                ..self.pending
            },
        };
        let default_local = if self.pending.scope == ScopeId::MODULE {
            self.ts.source_exports.as_ref().and_then(|export| {
                let local = self.ts.local_exports.get(&(ScopeId::MODULE, name_text))?;
                let declarations = local.default_declarations?;
                let symbol = export.default.as_ref()?.symbol;
                Some((symbol, declarations))
            })
        } else {
            None
        };
        PreparedDeclaration {
            declaration,
            destination,
            local,
            shared,
            existing,
            default_local,
        }
    }

    pub fn declare_binding_identifier(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        let Some((node, name)) = self.binding_identifier_declaration(tree, identifier, scope)
        else {
            return;
        };
        let declaration_id = self.declare(tree, name.into(), node);
        self.finish_binding_identifier(tree, name, declaration_id, scope, current_scope);
    }

    pub(crate) fn declare_prepared_binding_identifier(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
        prepared: PreparedDeclaration,
    ) {
        let Some((node, name)) = self.binding_identifier_declaration(tree, identifier, scope)
        else {
            return;
        };
        debug_assert_eq!(prepared.node(), node);
        debug_assert_eq!(prepared.name(), name.into());
        let declaration_id = self.declare_prepared(tree, prepared);
        self.finish_binding_identifier(tree, name, declaration_id, scope, current_scope);
    }

    /// Declares the pending binding for `name` at `node`. A compatible
    /// existing binding is merged. On conflict, its flags remain unchanged
    /// because the caller reports the diagnostic. Every node is still stored
    /// as a declaration for error recovery.
    // Keep map-entry temporaries out of the visitor shared with syntax checking.
    #[inline(never)]
    pub fn declare(&mut self, tree: &Ast<'a>, name: AstWtf8, node: NodeId) -> DeclarationId {
        debug_assert!(node.index() < tree.node_count());

        let name_text = tree.get_wtf8(name).as_bytes();
        if self.ts_build.containers.is_empty()
            && self.pending_type_parameter_owner().is_none()
            && (self.pending.scope != ScopeId::MODULE
                || (!self.pending_is_default_export(tree)
                    && self.pending_ts_source_destination().is_none()))
        {
            let target = self.pending.scope;
            let mut declaration_flags = self.pending.flags;
            declaration_flags.set(SymbolFlags::EXPORTED, self.export_state.is_exported());
            declaration_flags.set(SymbolFlags::IS_DEFAULT, self.export_state.is_default());
            let declaration_id = self.declarations.next_idx();
            let bindings = &mut self.scope_maps[target];
            let hash = bindings.hasher().hash_one(name_text);
            let entry = bindings
                .raw_entry_mut()
                .from_key_hashed_nocheck(hash, name_text);
            // The internal name shares the function scope, but is shadowed
            // rather than redeclared by parameters and local declarations.
            // Keep its symbol and AST identity while replacing the name map.
            let (entry, symbol_name) = match entry {
                hashbrown::hash_map::RawEntryMut::Occupied(entry)
                    if self.symbols[*entry.get()]
                        .flags
                        .intersects(SymbolFlags::INTERNAL_NAME) =>
                {
                    let symbol_name = self.symbols[*entry.get()].name;
                    entry.remove_entry();
                    (
                        bindings
                            .raw_entry_mut()
                            .from_key_hashed_nocheck(hash, name_text),
                        symbol_name,
                    )
                }
                entry => (entry, name),
            };
            let id = match entry {
                hashbrown::hash_map::RawEntryMut::Occupied(entry) => {
                    let existing = *entry.get();
                    let symbol = &mut self.symbols[existing];
                    let merges = !symbol.flags.intersects(self.pending.excludes);
                    if merges {
                        symbol.flags = merge_symbol_flags(symbol.flags, declaration_flags);
                    }
                    self.redeclarations
                        .entry(existing)
                        .or_insert_with(|| ArenaVec::with_capacity_in(1, self.allocator))
                        .push(declaration_id);
                    existing
                }
                hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                    let new_id = self.symbols.next_idx();
                    self.symbols.push(Symbol {
                        name: symbol_name,
                        flags: declaration_flags,
                        declaration: declaration_id,
                    });
                    vacant.insert_hashed_nocheck(hash, name_text, new_id);
                    new_id
                }
            };
            self.declarations.push(Declaration {
                node,
                symbol: id,
                scope: target,
                flags: declaration_flags,
            });
            self.record_internal_name(id);
            self.node_semantic[node] = NodeSemanticId::Symbol(id);
            return declaration_id;
        }

        self.declare_ts_context_binding(tree, name, node)
    }

    #[inline]
    fn binding_identifier_declaration(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        scope: &ScopeTree<'a>,
    ) -> Option<(NodeId, AstUtf8)> {
        self.sync_scope_maps_from(scope);
        // The declaration role comes from the enclosing syntax, independently
        // of whether references in that syntax produce runtime access.
        if self.pending.flags.is_empty() {
            return None;
        }
        Some((identifier.node_id(), identifier.name(tree)))
    }

    fn finish_binding_identifier(
        &mut self,
        tree: &Ast<'a>,
        name: AstUtf8,
        declaration_id: DeclarationId,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        // Register a hoisting var in every block it passes through so
        // block-scoped redeclarations see it.
        let declaration = self.declarations[declaration_id];
        if declaration.flags.is_hoisting_var() && current_scope != declaration.scope {
            let name = tree.get_utf8(name).as_bytes();
            for scope_id in scope.ancestors(Some(current_scope)) {
                if scope_id == declaration.scope {
                    break;
                }
                self.hoisting_variables[scope_id]
                    .get_or_insert_with(|| {
                        self.allocator.boxed(ArenaHashMap::new_in(self.allocator))
                    })
                    .entry(name)
                    .or_insert(declaration_id);
                if tree.node_kind(scope.get(scope_id).node) == NodeKind::CatchClause
                    && let Some(catch) = self.own_binding(scope_id, name)
                    && self.symbols[catch].flags.contains(SymbolFlags::CATCH_VAR)
                {
                    self.symbol_links.link(declaration.symbol, catch);
                }
            }
        }
    }

    pub(crate) fn link_parameter_vars(&mut self, scopes: &ScopeTree<'a>, body: ScopeId) {
        self.sync_scope_maps_from(scopes);
        let parameters = scopes.get(body).parent.expect("function body has a parent");
        // Inspect the completed body so a function implementation suppresses
        // parameter-to-var initialization independently of declaration order.
        for (&name, &parameter) in &self.scope_maps[parameters] {
            if self.symbols[parameter]
                .flags
                .contains(SymbolFlags::PARAMETER)
                && let Some(var) = self.own_binding(body, name)
                && self.symbols[var]
                    .flags
                    .contains(SymbolFlags::FUNCTION_SCOPED_VAR)
                && !self.symbols[var]
                    .flags
                    .contains(SymbolFlags::FUNCTION_IMPLEMENTATION)
            {
                self.symbol_links.link(parameter, var);
            }
        }
    }

    fn declare_prepared(&mut self, tree: &Ast<'a>, plan: PreparedDeclaration) -> DeclarationId {
        match plan.destination {
            DeclarationDestination::TsDefaultExport => {
                self.declare_default_export(tree, plan.declaration, true)
            }
            DeclarationDestination::TsTypeParameter { owner } => {
                let declaration = plan.declaration;
                let (id, declaration_id) =
                    self.declare_symbol(tree, declaration, plan.existing, true);
                if plan.local.is_none() {
                    self.insert_scope_binding(
                        tree,
                        declaration.binding.scope,
                        declaration.name,
                        id,
                    );
                }
                let parameters = self
                    .ts_build
                    .type_parameters
                    .entry(owner)
                    .or_insert_with(|| ScopeMap::new_in(self.allocator));
                let name = tree.get_wtf8(declaration.name).as_bytes();
                if !parameters.contains_key(name) {
                    parameters.insert(name, id);
                }
                declaration_id
            }
            DeclarationDestination::Scope => {
                let (id, declaration_id) =
                    self.declare_symbol(tree, plan.declaration, plan.existing, true);
                if plan.existing.is_none() {
                    let declaration = plan.declaration;
                    self.insert_scope_binding(
                        tree,
                        declaration.binding.scope,
                        declaration.name,
                        id,
                    );
                }
                declaration_id
            }
            DeclarationDestination::TsMember {
                environment,
                body_scope,
                projection_scope,
            } => self.declare_ts_member_in_environment(
                tree,
                environment,
                body_scope,
                projection_scope,
                plan.declaration,
                plan.existing,
            ),
        }
    }

    fn declare_symbol(
        &mut self,
        tree: &Ast<'a>,
        declaration: PendingDeclaration,
        existing: Option<SymbolId>,
        record_node: bool,
    ) -> (SymbolId, DeclarationId) {
        let PendingDeclaration {
            name,
            node,
            binding,
        } = declaration;
        let declaration_flags = binding.flags;
        debug_assert!(node.index() < tree.node_count());
        let declaration_id = self.declarations.next_idx();
        let id = if let Some(existing) = existing {
            let merges = self.declaration_merges(existing, binding);
            let symbol = &mut self.symbols[existing];
            if merges {
                symbol.flags = merge_symbol_flags(symbol.flags, declaration_flags);
            }
            self.redeclarations
                .entry(existing)
                .or_insert_with(|| ArenaVec::with_capacity_in(1, self.allocator))
                .push(declaration_id);
            existing
        } else {
            let new_id = self.symbols.next_idx();
            self.symbols.push(Symbol {
                name: self
                    .scope_maps
                    .get(binding.scope)
                    .and_then(|bindings| bindings.get(tree.get_wtf8(name).as_bytes()))
                    .map(|&symbol| self.symbols[symbol])
                    .filter(|symbol| symbol.flags.intersects(SymbolFlags::INTERNAL_NAME))
                    .map_or(name, |symbol| symbol.name),
                flags: declaration_flags,
                declaration: declaration_id,
            });
            new_id
        };

        self.declarations.push(Declaration {
            node,
            symbol: id,
            scope: binding.scope,
            flags: declaration_flags,
        });
        self.record_internal_name(id);
        if record_node {
            debug_assert!(matches!(
                self.node_semantic[node],
                NodeSemanticId::None | NodeSemanticId::Symbol(_)
            ));
            self.node_semantic[node] = NodeSemanticId::Symbol(id);
        }
        (id, declaration_id)
    }

    fn record_internal_name(&mut self, id: SymbolId) {
        let symbol = self.symbols[id];
        if symbol.flags.intersects(SymbolFlags::INTERNAL_NAME) {
            let scope = self.declarations[symbol.declaration].scope;
            self.internal_names
                .entry((scope, symbol.name))
                .or_insert(id);
        }
    }

    #[inline]
    fn insert_scope_binding(
        &mut self,
        tree: &Ast<'a>,
        scope: ScopeId,
        name: AstWtf8,
        id: SymbolId,
    ) {
        let name = tree.get_wtf8(name).as_bytes();
        self.scope_maps[scope].insert(name, id);
    }

    fn sync_scope_maps_from(&mut self, scope: &ScopeTree<'a>) {
        if self.scope_maps.len() < scope.list.len() {
            self.sync_scope_maps(scope.list.len());
        }
    }

    fn sync_scope_maps(&mut self, count: usize) {
        debug_assert_eq!(self.scope_maps.len(), self.hoisting_variables.len());
        let additional = count - self.scope_maps.len();
        self.scope_maps.reserve(additional);
        self.hoisting_variables.reserve(additional);
        for _ in 0..additional {
            self.scope_maps.push(ScopeMap::new_in(self.allocator));
            self.hoisting_variables.push(None);
        }
    }

    pub fn add_identifier_reference(
        &mut self,
        tree: &Ast<'a>,
        identifier: IdentifierReference,
        current_scope: ScopeId,
        flags: ReferenceFlags,
    ) {
        self.add_reference(
            tree,
            identifier.name(tree),
            current_scope,
            identifier.node_id(),
            flags,
        );
    }

    pub fn add_local_export_reference(
        &mut self,
        tree: &Ast<'a>,
        specifier: ExportSpecifier,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
        flags: ReferenceFlags,
        type_only: bool,
    ) {
        let Some(local) = specifier.local(tree).as_identifier_name(tree) else {
            return;
        };
        self.sync_scope_maps_from(scope);
        let local_reference = self.add_reference(
            tree,
            local.name(tree),
            current_scope,
            local.node_id(),
            flags,
        );
        let Some(environment) = self.current_ts_namespace_export_environment(current_scope) else {
            return;
        };
        let exported_name = ts_module_export_name(tree, specifier.exported(tree));
        let exported_name = tree.get_wtf8(exported_name).as_bytes();
        let members = &mut self.ts.environments;
        let id = members.aliases.push(TsExportAlias {
            type_only,
            resolution: TsExportAliasResolution::Pending,
            node: specifier.node_id(),
        });
        let member_environment = &mut members.environments[environment];
        member_environment.has_aliases = true;
        let previous = match member_environment.members.entry(exported_name) {
            hashbrown::hash_map::Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                let previous = entry.alias.map(|id| members.aliases[id].node);
                entry.alias.get_or_insert(id);
                previous
            }
            hashbrown::hash_map::Entry::Vacant(entry) => {
                entry.insert(TsMemberEntry {
                    direct: None,
                    alias: Some(id),
                });
                None
            }
        };
        self.ts_build
            .export_aliases
            .push(PendingTsNamespaceExportAlias {
                environment,
                exported_name,
                local_reference,
                previous,
            });
    }

    pub fn record_jsx_usage(&mut self, node: NodeId, scope: ScopeId, is_fragment: bool) {
        self.jsx_usages.push(crate::JsxUsage {
            node,
            scope,
            is_fragment,
        });
    }

    pub fn add_jsx_opening_element_reference(
        &mut self,
        tree: &Ast<'a>,
        element: JsxOpeningElement,
        current_scope: ScopeId,
    ) {
        // The root of a component tag is a JS binding
        // reference. Lowercase tags are intrinsic element names; the
        // rest of a member chain is property syntax.
        self.add_jsx_reference(tree, element.name(tree), current_scope);
    }

    pub fn add_jsx_closing_element_reference(
        &mut self,
        tree: &Ast<'a>,
        element: JsxClosingElement,
        current_scope: ScopeId,
    ) {
        self.add_jsx_reference(tree, element.name(tree), current_scope);
    }

    fn add_jsx_reference(&mut self, tree: &Ast<'a>, name: JsxElementName, scope: ScopeId) {
        let element_name = tree.jsx_element_name_data(name);
        let Some(identifier) = jsx_tag_root(tree, element_name) else {
            return;
        };
        let name = identifier.name(tree);
        let text = tree.get_utf8(name).as_bytes();
        if matches!(element_name, JsxElementNameData::JsxIdentifier(_))
            && (text.first().is_some_and(u8::is_ascii_lowercase) || text.contains(&b'-'))
        {
            return;
        }
        if text == b"this" {
            return;
        }
        self.add_reference(
            tree,
            name,
            scope,
            identifier.node_id(),
            ReferenceFlags::default(),
        );
    }

    /// Records an identifier reference at `node` in `scope`.
    pub fn add_reference(
        &mut self,
        tree: &Ast<'a>,
        name: AstUtf8,
        scope: ScopeId,
        node: NodeId,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        debug_assert!(node.index() < tree.node_count());
        // Ambient syntax still resolves through the same declaration space,
        // but its erased expressions do not access the binding at runtime.
        // Normalize only actual references instead of checking ambient state
        // while entering every AST node.
        let flags = if self.in_ambient_context() {
            flags.no_access()
        } else {
            flags
        };
        let id = self.references.next_idx();
        self.references.push(Reference {
            name,
            scope,
            node,
            symbol: None,
            flags,
        });
        id
    }

    #[inline]
    fn declaration_merges(&self, existing: SymbolId, binding: PendingBinding) -> bool {
        let existing = self.symbols[existing].flags;
        !existing.intersects(binding.excludes)
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id]
    }

    pub(crate) fn conflict_candidate_info(
        &self,
        candidate: DeclarationConflictCandidate,
    ) -> (SymbolFlags, NodeId) {
        match candidate.kind {
            DeclarationConflictKind::DefaultLocal(local) => return (local.flags, local.node),
            DeclarationConflictKind::Symbol => {
                let symbol = self.symbols[candidate.symbol];
                let declaration = self.declarations[symbol.declaration];
                return (symbol.flags, declaration.node);
            }
            DeclarationConflictKind::Scope(_) => {}
        }
        let mut declarations = self.conflict_candidate_declarations(candidate);
        let first = declarations
            .next()
            .expect("a scoped conflict candidate has a declaration in that scope");
        let flags = declarations.fold(first.flags, |flags, declaration| {
            merge_symbol_flags(flags, declaration.flags)
        });
        (flags, first.node)
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn has_non_ambient_class_declaration(
        &self,
        candidate: DeclarationConflictCandidate,
    ) -> bool {
        self.conflict_candidate_declarations(candidate)
            .any(|declaration| {
                declaration.flags.contains(SymbolFlags::CLASS)
                    && !declaration.flags.contains(SymbolFlags::AMBIENT)
            })
    }

    fn conflict_candidate_declarations(
        &self,
        candidate: DeclarationConflictCandidate,
    ) -> impl Iterator<Item = Declaration> + '_ {
        let symbol = self.symbols[candidate.symbol];
        let first = std::iter::once(symbol.declaration);
        let rest = self
            .redeclarations
            .get(&candidate.symbol)
            .into_iter()
            .flatten()
            .copied();
        first
            .chain(rest)
            .map(|declaration| self.declarations[declaration])
            .filter(move |declaration| match candidate.kind {
                DeclarationConflictKind::Scope(scope) => declaration.scope == scope,
                _ => true,
            })
    }

    pub fn symbol_of_node(&self, node: NodeId) -> Option<SymbolId> {
        self.node_semantic[node].symbol()
    }

    #[inline]
    pub(crate) fn record_scope(&mut self, node: NodeId, id: ScopeId) {
        let slot = &mut self.node_semantic[node];
        debug_assert!(matches!(
            *slot,
            NodeSemanticId::None | NodeSemanticId::Scope(_)
        ));
        *slot = NodeSemanticId::Scope(id);
    }

    /// The `binding_identifier` node of the earliest declaration of `id`.
    /// Later declarations merged into the symbol do not change it.
    pub fn first_decl_of(&self, id: SymbolId) -> NodeId {
        let declaration_id = self.symbols[id].declaration;
        self.declarations[declaration_id].node
    }

    /// The binding of `name` declared directly in `scope`, excluding
    /// hoisting `var`s passing through.
    pub fn own_binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps
            .get(scope)
            .and_then(|map| map.get(name).copied())
    }

    /// The binding of `name` at `scope`, including a hoisting `var` passing
    /// through on its way to its hoist target.
    pub fn binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.own_binding(scope, name).or_else(|| {
            self.hoisting_declaration(scope, name)
                .map(|declaration| self.declarations[declaration].symbol)
        })
    }

    fn hoisting_declaration(&self, scope: ScopeId, name: &[u8]) -> Option<DeclarationId> {
        self.hoisting_variables
            .get(scope)
            .and_then(Option::as_deref)
            .and_then(|map| map.get(name).copied())
    }

    pub(crate) fn hoisting_conflict(
        &self,
        prepared: PreparedDeclaration,
        name: &[u8],
    ) -> Option<Declaration> {
        // Passing vars are conflict-only declarations, never binding candidates.
        // Only declarations that exclude var need the scope's hoisting map.
        if !prepared
            .excludes()
            .contains(SymbolFlags::FUNCTION_SCOPED_VAR)
        {
            return None;
        }
        let scope = match prepared.destination {
            DeclarationDestination::TsMember { body_scope, .. } => body_scope?,
            _ => prepared.target(),
        };
        let declaration = self.hoisting_declaration(scope, name)?;
        Some(self.declarations[declaration])
    }

    /// Resolves references and builds use and node indexes, then moves the completed
    /// records into [`Semantic`]. The result remains valid for the source tree's arena lifetime.
    pub fn finalize(mut self, tree: &Ast<'a>, scopes: ScopeTree<'a>) -> Semantic<'a> {
        // The walk must have ended balanced.
        debug_assert!(self.saved_stack.is_empty());
        debug_assert_eq!(self.export_state, ExportState::None);
        debug_assert!(self.ts_build.containers.is_empty());
        debug_assert!(self.ts_build.enums.is_empty());
        self.sync_scope_maps(scopes.list.len());
        let ts_scope_views = {
            let recorded = self.ts.scope_views.take();
            recorded.or_else(|| self.finalize_ts_scope_views(&scopes.list))
        };

        // ======================= Resolve references =======================
        let (mut use_ranges, reference_candidates, reference_values) = {
            let symbol_count = self.symbols.len();

            let mut use_ranges = ArenaIndexVec::with_capacity_in(symbol_count, self.allocator);
            use_ranges.resize(symbol_count, Range::default());

            let mut reference_candidates = ReferenceCandidates::new_in(self.allocator);
            let mut reference_values = TsReferenceValues::new_in(self.allocator);
            let precise_import_aliases = !self.ts.import_aliases.spaces.is_empty();
            if let Some(scope_views) = ts_scope_views.as_ref() {
                if precise_import_aliases {
                    reference_values = self.resolve_references_with_ts_views::<true>(
                        tree,
                        &scopes,
                        scope_views,
                        &mut use_ranges,
                        &mut reference_candidates,
                    );
                } else {
                    reference_values = self.resolve_references_with_ts_views::<false>(
                        tree,
                        &scopes,
                        scope_views,
                        &mut use_ranges,
                        &mut reference_candidates,
                    );
                }
            } else if precise_import_aliases {
                self.resolve_lexical_references::<true, true, true>(
                    tree,
                    &scopes,
                    &mut use_ranges,
                    &mut reference_candidates,
                );
            } else if scopes
                .list
                .iter()
                .any(|scope| scope.kind.is_ts_lookup_view())
            {
                self.resolve_lexical_references::<false, true, true>(
                    tree,
                    &scopes,
                    &mut use_ranges,
                    &mut reference_candidates,
                );
            } else if tree.is_ts() {
                // No signature or conditional-type view can alter lexical lookup.
                // Select the policy once instead of testing every ancestor edge.
                self.resolve_lexical_references::<false, false, true>(
                    tree,
                    &scopes,
                    &mut use_ranges,
                    &mut reference_candidates,
                );
            } else {
                // No signature or conditional-type view can alter lexical lookup.
                // Select the policy once instead of testing every ancestor edge.
                self.resolve_lexical_references::<false, false, false>(
                    tree,
                    &scopes,
                    &mut use_ranges,
                    &mut reference_candidates,
                );
            }
            (use_ranges, reference_candidates, reference_values)
        };

        // ======================= Reverse uses =======================
        let use_ids = {
            // Use index: prefix-sum and fill. Resolution counted uses above;
            // `len` now doubles as the write cursor during the fill.
            let mut use_offset = 0u32;
            for range in &mut use_ranges {
                range.start = use_offset;
                use_offset += range.len;
                range.len = 0;
            }
            let mut use_ids = ArenaVec::with_capacity_in(use_offset as usize, self.allocator);
            use_ids.resize(use_offset as usize, ReferenceId::from_usize(0));
            if reference_candidates.ranges.is_empty() {
                for (id, reference) in self.references.iter_enumerated() {
                    if let Some(symbol) = reference.symbol {
                        let range = &mut use_ranges[symbol];
                        use_ids[(range.start + range.len) as usize] = id;
                        range.len += 1;
                    }
                }
            } else {
                for (id, reference) in self.references.iter_enumerated() {
                    let targets = reference
                        .symbol
                        .into_iter()
                        .chain(reference_candidates.additional(id).iter().copied());
                    for symbol in targets {
                        let range = &mut use_ranges[symbol];
                        use_ids[(range.start + range.len) as usize] = id;
                        range.len += 1;
                    }
                }
            }
            use_ids
        };

        // ======================= Publish =======================
        for (id, reference) in self.references.iter_enumerated() {
            let slot = &mut self.node_semantic[reference.node];
            debug_assert!(matches!(
                *slot,
                NodeSemanticId::None | NodeSemanticId::Reference(_)
            ));
            *slot = NodeSemanticId::Reference(id);
        }
        self.ts.scope_views = ts_scope_views;
        self.ts.reference_values = if reference_values.ranges.is_empty() {
            None
        } else {
            Some(self.allocator.boxed(reference_values))
        };

        Semantic {
            scopes,
            symbols: self.symbols,
            symbol_links: self.symbol_links,
            references: self.references,
            jsx_usages: self.jsx_usages,
            reference_candidates,
            declarations: self.declarations,
            redeclarations: self.redeclarations,
            use_ids,
            use_ranges,
            scope_maps: self.scope_maps,
            internal_names: self.internal_names,
            hoisting_variables: self.hoisting_variables,
            node_semantic: self.node_semantic,
            typescript: self.ts,
        }
    }

    fn resolve_lexical_references<
        const PRECISE_IMPORT_ALIASES: bool,
        const TS_LOOKUP: bool,
        const FILTER_BY_SPACE: bool,
    >(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'_>,
        use_ranges: &mut UseRanges,
        additional_targets: &mut ReferenceCandidates<'a>,
    ) {
        let import_alias_spaces = self.ts.import_aliases.spaces.as_slice();
        let lexical = LexicalLookup {
            scopes: &scopes.list,
            symbols: &self.symbols,
            internal_names: &self.internal_names,
            scope_maps: &self.scope_maps,
        };
        let mut candidates = Vec::new();
        for (id, reference) in self.references.iter_mut_enumerated() {
            candidates.clear();
            // JavaScript and JSX only produce runtime-value references here;
            // `Any` is the name-only form used by local export specifiers.
            debug_assert!(
                FILTER_BY_SPACE
                    || matches!(
                        reference.flags.space,
                        ReferenceSpace::Value | ReferenceSpace::Any
                    )
            );
            let query = NameLookupQuery {
                scope: reference.scope,
                name: tree.get_utf8(reference.name).as_bytes(),
                space: reference.flags.space,
            };
            let resolved = if PRECISE_IMPORT_ALIASES {
                lookup_lexical_name_with_import_aliases::<true>(
                    lexical,
                    import_alias_spaces,
                    query,
                    |symbol| candidates.push(symbol),
                )
            } else {
                lookup_lexical_name::<true, TS_LOOKUP, FILTER_BY_SPACE>(lexical, query)
            };
            if reference.flags.space == ReferenceSpace::Any {
                reference.flags = {
                    let value_lookup = TsValueTargetLookup {
                        locals: &self.ts.local_exports,
                        symbols: &self.symbols,
                        declarations: &self.declarations,
                        scope_maps: &self.scope_maps,
                        import_alias_spaces,
                    };
                    value_lookup
                        .reference_flags::<PRECISE_IMPORT_ALIASES>(tree, *reference, resolved)
                };
            }
            additional_targets.record(id, reference, resolved, &mut candidates, use_ranges);
        }
    }
}

fn import_pending(kind: ImportOrExportKind, scope: ScopeId) -> PendingBinding {
    PendingBinding {
        flags: if kind == ImportOrExportKind::Type {
            SymbolFlags::TYPE_IMPORT
        } else {
            SymbolFlags::IMPORT
        },
        excludes: Excludes::IMPORT_BINDING,
        scope,
    }
}

// Walks a JSX tag name to its leftmost JsxIdentifier. Returns None for
// namespaced names (`<svg:path>`) and non-tag shapes.
fn jsx_tag_root(tree: &Ast<'_>, name: JsxElementNameData) -> Option<JsxIdentifier> {
    let mut current = name;
    loop {
        match current {
            JsxElementNameData::JsxIdentifier(identifier) => return Some(identifier),
            JsxElementNameData::JsxMemberExpression(member) => {
                match tree.jsx_member_expression_object_data(member.object(tree)) {
                    JsxMemberExpressionObjectData::JsxIdentifier(jsx_identifier) => {
                        current = JsxElementNameData::JsxIdentifier(jsx_identifier)
                    }
                    JsxMemberExpressionObjectData::JsxMemberExpression(jsx_member_expression) => {
                        current = JsxElementNameData::JsxMemberExpression(jsx_member_expression)
                    }
                }
            }
            JsxElementNameData::JsxNamespacedName(_) => return None,
        }
    }
}

fn merge_symbol_flags(existing: SymbolFlags, incoming: SymbolFlags) -> SymbolFlags {
    let mut merged = existing.merge(incoming);
    if merged.contains(SymbolFlags::CONST_ENUM_ONLY_MODULE)
        && (symbol_flags_clear_const_enum_only(existing)
            || symbol_flags_clear_const_enum_only(incoming))
    {
        merged.remove(SymbolFlags::CONST_ENUM_ONLY_MODULE);
    }
    // An overload implementation emits at runtime, so the merged symbol is
    // non-ambient.
    merged.set(
        SymbolFlags::AMBIENT,
        existing.contains(SymbolFlags::AMBIENT) && incoming.contains(SymbolFlags::AMBIENT),
    );
    merged
}
