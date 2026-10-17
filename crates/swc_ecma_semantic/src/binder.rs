use std::hash::BuildHasher;

use hashbrown::HashMap;
use oxc_index::Idx;
use rustc_hash::FxBuildHasher;
use swc_next_allocator::{
    Allocator, boxed::Box as ArenaBox, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, Ast, AstUtf8, AstWtf8, BindingIdentifier, Class,
    ClassType, ExportNamedDeclaration, ExportSpecifier, Function, FunctionType,
    IdentifierReference, ImportDeclaration, ImportOrExportKind, ImportSpecifier, JsxClosingElement,
    JsxElementName, JsxElementNameData, JsxIdentifier, JsxMemberExpressionObjectData,
    JsxOpeningElement, MemberExpression, ModuleExportName, ModuleExportNameData, NodeId, NodeKind,
    NodeKindData, ReferenceId, ScopeId, SymbolId, TsEnumDeclaration, TsEnumMember, TsFunction,
    TsGlobalDeclaration, TsImportEqualsDeclaration, TsInterfaceDeclaration, TsModuleDeclaration,
    TsModuleName, TsModuleNameData, TsNamespaceExportDeclaration, TsQualifiedName,
    TsQualifiedNameLeftData, TsTypeAliasDeclaration, TsTypePredicate, VariableDeclaration,
    VariableKind,
};

use crate::{
    DeclarationId, HoistingMap, NodeSemanticId, Redeclarations, ScopeMap, Semantic,
    module_record::ModuleRecordBuilder,
    scope::{Scope, ScopeKind, ScopeTracker, ScopeTree},
};

mod aliases;
mod annex_b;
mod lookup;
mod member_facts;
mod module_instance;

use lookup::{
    LexicalLookup, NameBinding, NameLookupQuery, TsNameLookup, lookup_lexical_name,
    lookup_lexical_name_with_import_aliases, lookup_name_in_scopes, lookup_name_with_ts_views,
    precise_import_alias_spaces, symbol_visibility,
};

use annex_b::AnnexBFunction;
use member_facts::{PropertyValue, TsExpandoBindingSite, text_member_name};
use module_instance::{TsModuleInstanceMemo, TsModuleInstanceState, ts_module_instance_state};

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

oxc_index::define_nonmax_u32_index_type! {
    struct TsMemberEnvironmentId;
}

type TsMemberMap<'a> = ArenaHashMap<'a, &'a [u8], TsMemberEntry>;
type TsNamespaceExportAliasMap<'a> = ArenaHashMap<'a, &'a [u8], TsNamespaceExportAliasEntry<'a>>;

#[derive(Debug)]
struct TsMemberEnvironment<'a> {
    // Canonical declarations visible from other bodies merged into the same
    // namespace or enum.
    members: TsMemberMap<'a>,
    // Public aliases do not become lexical bindings in merged namespace
    // bodies. TypeScript likewise keeps container locals and exports apart.
    export_aliases: TsNamespaceExportAliasMap<'a>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsMemberKinds: u8 {
        const MODULE = 1 << 0;
        const ENUM = 1 << 1;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsMemberSpaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
        const TYPEOF = 1 << 3;
    }
}

impl TsMemberSpaces {
    fn from_flags(flags: SymbolFlags) -> Self {
        let mut spaces = Self::empty();
        spaces.set(Self::VALUE, flags.visible_in(ReferenceSpace::Value));
        spaces.set(Self::TYPE, flags.visible_in(ReferenceSpace::Type));
        spaces.set(Self::NAMESPACE, flags.visible_in(ReferenceSpace::Namespace));
        spaces.set(Self::TYPEOF, flags.visible_in(ReferenceSpace::Typeof));
        spaces
    }

    fn visible_in(self, space: ReferenceSpace) -> bool {
        match space {
            ReferenceSpace::Value => self.contains(Self::VALUE),
            ReferenceSpace::Type => self.contains(Self::TYPE),
            ReferenceSpace::Namespace => self.contains(Self::NAMESPACE),
            ReferenceSpace::Typeof => self.contains(Self::TYPEOF),
            ReferenceSpace::Any => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TsMemberEntry {
    symbol: SymbolId,
    kinds: TsMemberKinds,
    module_spaces: TsMemberVisibility,
    enum_spaces: TsMemberSpaces,
    // Excludes enum-only and import-alias contributions to the shared symbol.
    module_declaration: bool,
}

#[derive(Debug, Clone, Copy)]
struct TsMemberVisibility {
    possible: TsMemberSpaces,
    certain: TsMemberSpaces,
}

impl TsMemberVisibility {
    fn certain(spaces: TsMemberSpaces) -> Self {
        Self {
            possible: spaces,
            certain: spaces,
        }
    }

    fn insert(&mut self, other: Self) {
        self.possible |= other.possible;
        self.certain |= other.certain;
    }

    fn in_space(self, space: ReferenceSpace) -> BindingVisibility {
        if self.certain.visible_in(space) {
            BindingVisibility::Present
        } else if self.possible.visible_in(space) {
            BindingVisibility::Unknown
        } else {
            BindingVisibility::Absent
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BindingVisibility {
    Absent,
    Present,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
struct TsNamespaceExportAliasEntry<'a> {
    type_only: bool,
    resolution: TsExportAliasResolution<'a>,
    node: NodeId,
}

#[derive(Debug, Clone, Copy)]
enum TsExportAliasResolution<'a> {
    Resolved(TsResolvedExportAliasTarget<'a>),
    Unresolved,
    NonLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TsMemberViewKind {
    Module,
    AmbientModule,
    SourceFile,
    Enum,
}

impl TsMemberViewKind {
    fn accepts(self, kinds: TsMemberKinds) -> bool {
        match self {
            Self::Module | Self::AmbientModule | Self::SourceFile => {
                kinds.contains(TsMemberKinds::MODULE)
            }
            Self::Enum => kinds.contains(TsMemberKinds::ENUM),
        }
    }

    fn spaces(self, entry: TsMemberEntry) -> TsMemberVisibility {
        match self {
            Self::Module | Self::AmbientModule | Self::SourceFile => entry.module_spaces,
            Self::Enum => TsMemberVisibility::certain(entry.enum_spaces),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TsMemberView {
    environment: TsMemberEnvironmentId,
    kind: TsMemberViewKind,
}

impl TsMemberView {
    #[inline(always)]
    fn candidates<'lookup, 'a>(
        self,
        environments: &'lookup TsMemberEnvironments<'a>,
        hash: u64,
        name: &[u8],
    ) -> Option<TsMemberCandidates<'lookup, 'a>> {
        let environment = &environments.environments[self.environment.index()];
        let direct = environment
            .members
            .raw_entry()
            .from_hash(hash, |candidate| *candidate == name)
            .map(|(_, entry)| *entry)
            .filter(|entry| self.kind.accepts(entry.kinds));
        let alias = if matches!(self.kind, TsMemberViewKind::Enum)
            || environment.export_aliases.is_empty()
        {
            None
        } else {
            environment
                .export_aliases
                .raw_entry()
                .from_hash(hash, |candidate| *candidate == name)
                .map(|(_, entry)| entry)
        };
        // TS7 excludes pure export specifiers in source files and ambient module bodies.
        // A symbol with its own declaration meaning still exposes its alias
        // contribution, even when the direct declaration occupies another space.
        if matches!(
            self.kind,
            TsMemberViewKind::AmbientModule | TsMemberViewKind::SourceFile
        ) && alias.is_some()
        {
            let has_declaration = direct.is_some_and(|entry| entry.module_declaration);
            if !has_declaration {
                return None;
            }
        }
        Some(TsMemberCandidates { direct, alias })
    }

    #[inline(always)]
    fn lookup<'a>(
        self,
        environments: &TsMemberEnvironments<'a>,
        hash: u64,
        name: &[u8],
    ) -> Option<TsMemberLookup<'a>> {
        let TsMemberCandidates { direct, alias } = self.candidates(environments, hash, name)?;
        let target = TsResolvedImportTarget::from_members(direct.map(|entry| entry.symbol), alias)?;
        let direct_spaces = direct
            .map(|entry| self.kind.spaces(entry))
            .unwrap_or_else(|| TsMemberVisibility::certain(TsMemberSpaces::empty()));
        Some(TsMemberLookup {
            target,
            direct_spaces,
        })
    }
}

struct TsMemberCandidates<'lookup, 'a> {
    direct: Option<TsMemberEntry>,
    alias: Option<&'lookup TsNamespaceExportAliasEntry<'a>>,
}

struct TsMemberLookup<'a> {
    target: TsResolvedImportTarget<'a>,
    direct_spaces: TsMemberVisibility,
}

impl TsMemberLookup<'_> {
    fn visit_candidates<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        symbols: &[Symbol],
        import_alias_spaces: &[Option<TsResolvedAliasSpaces>],
        space: ReferenceSpace,
        mut accept: impl FnMut(SymbolId, BindingVisibility) -> bool,
    ) -> bool {
        if let Some(symbol) = self.target.direct
            && accept(symbol, self.direct_spaces.in_space(space))
        {
            return true;
        }
        let Some(alias) = self.target.alias else {
            return false;
        };
        for contribution in alias
            .target
            .public_contributions(self.target.type_only_export)
        {
            if contribution.type_only_export.is_some() && space == ReferenceSpace::Value {
                continue;
            }
            let visibility = if PRECISE_IMPORT_ALIASES
                && let Some(spaces) =
                    precise_import_alias_spaces(symbols, import_alias_spaces, contribution.symbol)
            {
                // Namespace-body lookup uses ModuleMember, excluding property
                // values even when a qualified alias could access them.
                spaces
                    .difference(
                        TsResolvedAliasSpaces::PROPERTY_VALUE
                            | TsResolvedAliasSpaces::UNKNOWN_PROPERTY_VALUE,
                    )
                    .visibility(space, symbols[contribution.symbol.index()].flags)
            } else {
                symbol_visibility::<PRECISE_IMPORT_ALIASES>(
                    symbols,
                    import_alias_spaces,
                    contribution.symbol,
                    space,
                )
            };
            if accept(contribution.symbol, visibility) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
struct TsMemberEnvironments<'a> {
    owners: ArenaHashMap<'a, SymbolId, TsMemberEnvironmentId>,
    environments: ArenaVec<'a, TsMemberEnvironment<'a>>,
}

impl<'a> TsMemberEnvironments<'a> {
    fn needs_value_projections(&self) -> bool {
        self.environments.iter().any(|environment| {
            environment.export_aliases.iter().any(|(name, alias)| {
                let direct = environment.members.get(name).map(|entry| entry.symbol);
                TsResolvedImportTarget::from_members(direct, Some(alias))
                    .is_some_and(TsResolvedImportTarget::needs_value_projection)
            })
        })
    }

    fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            owners: ArenaHashMap::new_in(allocator),
            environments: ArenaVec::new_in(allocator),
        }
    }

    fn ensure(&mut self, owner: SymbolId, allocator: &'a Allocator) -> TsMemberEnvironmentId {
        *self.owners.entry(owner).or_insert_with(|| {
            let environment = TsMemberEnvironmentId::from_usize(self.environments.len());
            self.environments.push(TsMemberEnvironment {
                members: TsMemberMap::new_in(allocator),
                export_aliases: TsNamespaceExportAliasMap::new_in(allocator),
            });
            environment
        })
    }

    fn get(&self, owner: SymbolId) -> Option<&TsMemberEnvironment<'a>> {
        let environment = *self.owners.get(&owner)?;
        Some(&self.environments[environment.index()])
    }

    fn members(&self, owner: SymbolId) -> Option<&TsMemberMap<'a>> {
        self.get(owner).map(|environment| &environment.members)
    }

    fn environment(&self, environment: TsMemberEnvironmentId) -> &TsMemberMap<'a> {
        &self.environments[environment.index()].members
    }

    fn environment_mut(&mut self, environment: TsMemberEnvironmentId) -> &mut TsMemberMap<'a> {
        &mut self.environments[environment.index()].members
    }

    fn environment_export_aliases(
        &self,
        environment: TsMemberEnvironmentId,
    ) -> &TsNamespaceExportAliasMap<'a> {
        &self.environments[environment.index()].export_aliases
    }

    fn export_aliases_mut(
        &mut self,
        environment: TsMemberEnvironmentId,
    ) -> &mut TsNamespaceExportAliasMap<'a> {
        &mut self.environments[environment.index()].export_aliases
    }

    fn member(&self, environment: TsMemberEnvironmentId, name: &[u8]) -> Option<SymbolId> {
        self.environment(environment)
            .get(name)
            .map(|entry| entry.symbol)
    }
}

#[derive(Debug)]
struct TsImportAliasIndex<'a> {
    spaces: ArenaVec<'a, Option<TsResolvedAliasSpaces>>,
    targets: ArenaVec<'a, Option<SymbolId>>,
}

#[derive(Debug)]
pub(crate) struct TsMemberIndex<'a> {
    source_exports: Option<TsSourceExports<'a>>,
    local_exports: TsLocalExportMap<'a>,
    environments: TsMemberEnvironments<'a>,
    scope_views: Option<TsScopeViewIndex<'a>>,
    views: ArenaVec<'a, TsMemberView>,
    import_aliases: Option<TsImportAliasIndex<'a>>,
    reference_values: Option<ArenaBox<'a, TsReferenceValues<'a>>>,
}

impl TsMemberIndex<'_> {
    pub(crate) fn reference_values(&self, reference: ReferenceId) -> Option<&[NameResolution]> {
        self.reference_values.as_ref()?.get(reference)
    }

    pub(crate) fn import_alias_target(&self, alias: SymbolId) -> Option<SymbolId> {
        self.import_aliases
            .as_ref()?
            .targets
            .get(alias.index())
            .copied()
            .flatten()
    }

    pub(crate) fn value_target(
        &self,
        symbols: &[Symbol],
        declarations: &[Declaration],
        scope_maps: &[ScopeMap<'_>],
        resolution: NameResolution,
        name: &[u8],
        space: ReferenceSpace,
    ) -> Option<NameResolution> {
        let spaces = self
            .import_aliases
            .as_ref()
            .map(|aliases| aliases.spaces.as_slice())
            .unwrap_or_default();
        let symbol = resolution.candidate()?;
        let target = TsValueTargetLookup {
            locals: &self.local_exports,
            symbols,
            declarations,
            scope_maps,
            import_alias_spaces: spaces,
        }
        .target::<true>(symbol, name, space);
        let candidate = target.candidate()?;
        if matches!(resolution, NameResolution::Uncertain { .. }) {
            Some(NameResolution::Uncertain { candidate })
        } else {
            Some(target)
        }
    }

    pub(crate) fn source_module_symbol(&self) -> Option<SymbolId> {
        self.source_exports.as_ref().map(|source| source.owner)
    }

    pub(crate) fn default_export_symbol(&self) -> Option<SymbolId> {
        self.source_exports
            .as_ref()?
            .default
            .as_ref()
            .map(|default| default.symbol)
    }

    pub(crate) fn source_binding(&self, name: &[u8]) -> Option<SymbolId> {
        if name == b"default" {
            return None;
        }
        let source = self.source_exports.as_ref()?;
        self.environments.member(source.environment, name)
    }

    pub(crate) fn source_bindings<'s>(
        &'s self,
        locals: &'s ScopeMap<'s>,
    ) -> Option<TsSourceBindingIterator<'s>> {
        let source = self.source_exports.as_ref()?;
        Some(TsSourceBindingIterator {
            inner: self.environments.environment(source.environment).iter(),
            locals,
        })
    }

    pub(crate) fn namespace_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.members(owner, TsMemberViewKind::Module)
    }

    pub(crate) fn namespace_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.member(owner, name, TsMemberViewKind::Module)
    }

    pub(crate) fn namespace_has_export_aliases(&self, owner: SymbolId) -> bool {
        self.environments
            .get(owner)
            .is_some_and(|environment| !environment.export_aliases.is_empty())
    }

    pub(crate) fn namespace_value_targets<'s>(
        &'s self,
        symbols: &'s [Symbol],
        declarations: &'s [Declaration],
        scope_maps: &'s [ScopeMap<'_>],
        owner: SymbolId,
        name: &'s [u8],
    ) -> impl Iterator<Item = NameResolution> + 's {
        let target = self.environments.get(owner).and_then(|environment| {
            let direct = environment.members.get(name).map(|entry| entry.symbol);
            let alias = environment.export_aliases.get(name);
            TsResolvedImportTarget::from_members(direct, alias)
        });
        target
            .into_iter()
            .flat_map(TsResolvedImportTarget::public_contributions)
            .filter_map(move |contribution| {
                if contribution.type_only_export.is_some() {
                    return None;
                }
                self.value_target(
                    symbols,
                    declarations,
                    scope_maps,
                    NameResolution::Resolved(contribution.symbol),
                    name,
                    ReferenceSpace::Value,
                )
            })
    }

    pub(crate) fn enum_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.members(owner, TsMemberViewKind::Enum)
    }

    pub(crate) fn enum_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.member(owner, name, TsMemberViewKind::Enum)
    }

    fn members(
        &self,
        owner: SymbolId,
        kind: TsMemberViewKind,
    ) -> impl Iterator<Item = SymbolId> + '_ {
        self.environments
            .members(owner)
            .into_iter()
            .flat_map(|members| members.values())
            .filter(move |entry| kind.accepts(entry.kinds))
            .map(|entry| entry.symbol)
    }

    fn member(&self, owner: SymbolId, name: &[u8], kind: TsMemberViewKind) -> Option<SymbolId> {
        self.environments
            .members(owner)
            .and_then(|members| members.get(name))
            .filter(|entry| kind.accepts(entry.kinds))
            .map(|entry| entry.symbol)
    }

    pub(crate) fn resolve_name(
        &self,
        semantic: &Semantic<'_>,
        scope: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        let query = NameLookupQuery { scope, name, space };
        let lexical = LexicalLookup {
            scopes: &semantic.scopes.list,
            symbols: &semantic.symbols,
            internal_names: &semantic.internal_names,
            scope_maps: &semantic.scope_maps,
        };
        let ts = TsNameLookup {
            source_exports: self.source_exports.as_ref(),
            local_exports: &self.local_exports,
            environments: &self.environments,
            views: &self.views,
            import_alias_spaces: self
                .import_aliases
                .as_ref()
                .map(|aliases| aliases.spaces.as_slice())
                .unwrap_or_default(),
        };
        if self.import_aliases.is_some() {
            lookup_name_in_scopes::<true, true>(
                lexical,
                &ts,
                self.scope_views.as_ref(),
                query,
                |_| {},
            )
            .resolution()
        } else {
            lookup_name_in_scopes::<true, false>(
                lexical,
                &ts,
                self.scope_views.as_ref(),
                query,
                |_| {},
            )
            .resolution()
        }
    }
}

pub(crate) struct TsSourceBindingIterator<'a> {
    inner: swc_next_allocator::hash_map::Iter<'a, &'a [u8], TsMemberEntry>,
    locals: &'a ScopeMap<'a>,
}

impl Iterator for TsSourceBindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|(&name, member)| {
            (name != b"default" && !self.locals.contains_key(name)).then_some(member.symbol)
        })
    }
}

#[derive(Clone, Copy)]
enum TsContainerContext {
    UnsupportedExternalModule {
        body: Option<NodeId>,
    },
    Member {
        environment: TsMemberEnvironmentId,
        views: Range,
        projection_scope: Option<ScopeId>,
        body: Option<NodeId>,
        body_scope: Option<ScopeId>,
        implicit_exports: bool,
        public_exports: bool,
    },
}

impl TsContainerContext {
    fn body(self) -> Option<NodeId> {
        match self {
            Self::UnsupportedExternalModule { body } | Self::Member { body, .. } => body,
        }
    }
}

#[derive(Clone, Copy)]
struct PendingTsNamespaceExportAlias<'a> {
    type_only: bool,
    environment: TsMemberEnvironmentId,
    exported_name: &'a [u8],
    local_reference: ReferenceId,
    node: NodeId,
    previous: Option<NodeId>,
}

#[derive(Clone, Copy)]
struct TsImportAliasTarget {
    alias: SymbolId,
    declaration: TsImportEqualsDeclaration,
    root_reference: ReferenceId,
}

// Like TS7's merged export symbols, these nodes retain the public name at
// which alias resolution meets declaration meaning. Pure forwarding aliases
// share their target; only a direct-plus-alias combination allocates a node.
#[derive(Debug, Clone, Copy)]
enum TsExportTarget<'a> {
    // Public access may cross an exported-value bridge; conflict checks keep
    // the lexical declaration meaning. Both stay compact for simple targets.
    Symbols { lexical: SymbolId, public: SymbolId },
    Member(&'a TsExportMember<'a>),
}

#[derive(Debug)]
struct TsExportMember<'a> {
    direct: SymbolId,
    alias: TsResolvedExportAliasTarget<'a>,
}

#[derive(Debug, Clone, Copy)]
struct TsResolvedExportAliasTarget<'a> {
    target: TsExportTarget<'a>,
    type_only_export: Option<NodeId>,
}

impl<'a> TsResolvedExportAliasTarget<'a> {
    fn target(target: TsExportTarget<'a>) -> Self {
        Self {
            target,
            type_only_export: None,
        }
    }

    fn symbol(symbol: SymbolId) -> Self {
        Self::target(TsExportTarget::Symbols {
            lexical: symbol,
            public: symbol,
        })
    }
}

#[derive(Clone, Copy)]
struct TsExportContribution {
    symbol: SymbolId,
    type_only_export: Option<NodeId>,
}

struct TsExportContributions<'a> {
    target: Option<TsExportTarget<'a>>,
    conflict: bool,
    type_only_export: Option<NodeId>,
}

impl<'a> TsExportTarget<'a> {
    fn public_contributions(self, type_only_export: Option<NodeId>) -> TsExportContributions<'a> {
        TsExportContributions {
            target: Some(self),
            conflict: false,
            type_only_export,
        }
    }

    fn conflict_contributions(self) -> TsExportContributions<'a> {
        TsExportContributions {
            target: Some(self),
            conflict: true,
            type_only_export: None,
        }
    }
}

impl Iterator for TsExportContributions<'_> {
    type Item = TsExportContribution;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.target.take()?;
        let type_only_export = self.type_only_export;
        let symbol = match target {
            TsExportTarget::Symbols { lexical, public } => {
                if self.conflict {
                    lexical
                } else {
                    public
                }
            }
            TsExportTarget::Member(member) => {
                self.target = Some(member.alias.target);
                // The member is already a public export symbol. Its alias edge
                // crosses the exported-value bridge, even when the caller is
                // checking a new alias against the member's declaration spaces.
                self.conflict = false;
                self.type_only_export = self.type_only_export.or(member.alias.type_only_export);
                member.direct
            }
        };
        Some(TsExportContribution {
            symbol,
            type_only_export,
        })
    }
}

#[derive(Clone, Copy)]
struct TsResolvedImportTarget<'a> {
    direct: Option<SymbolId>,
    alias: Option<TsResolvedExportAliasTarget<'a>>,
    type_only_export: Option<NodeId>,
    // Includes implicit members such as prototype; no lexical binding or
    // explicit member AST node is required to preserve this value meaning.
    property_value: PropertyValue,
}

impl<'a> TsResolvedImportTarget<'a> {
    fn needs_value_projection(self) -> bool {
        self.alias.is_some_and(|alias| match alias.target {
            TsExportTarget::Member(_) => true,
            TsExportTarget::Symbols { public, .. } => match self.direct {
                Some(direct) => direct != public,
                None => self.type_only_export.is_some(),
            },
        })
    }

    fn from_members(
        direct: Option<SymbolId>,
        alias: Option<&TsNamespaceExportAliasEntry<'a>>,
    ) -> Option<Self> {
        let alias = alias.and_then(|entry| match entry.resolution {
            TsExportAliasResolution::Resolved(target) => Some(target),
            TsExportAliasResolution::Unresolved | TsExportAliasResolution::NonLocal => None,
        });
        if direct.is_none() && alias.is_none() {
            return None;
        }
        Some(Self {
            direct,
            alias,
            type_only_export: alias.and_then(|alias| alias.type_only_export),
            property_value: PropertyValue::Absent,
        })
    }

    fn symbol(symbol: SymbolId) -> Self {
        Self {
            direct: Some(symbol),
            alias: None,
            type_only_export: None,
            property_value: PropertyValue::Absent,
        }
    }

    fn property_value(value: PropertyValue) -> Self {
        Self {
            direct: None,
            alias: None,
            type_only_export: None,
            property_value: value,
        }
    }

    fn non_symbol_spaces(self) -> TsResolvedAliasSpaces {
        match self.property_value {
            PropertyValue::Absent => TsResolvedAliasSpaces::empty(),
            PropertyValue::Present => TsResolvedAliasSpaces::PROPERTY_VALUE,
            PropertyValue::Unknown => TsResolvedAliasSpaces::UNKNOWN_PROPERTY_VALUE,
        }
    }

    // Symbol contributions exclude static members represented by non_symbol_spaces.
    fn public_contributions(self) -> impl Iterator<Item = TsExportContribution> + 'a {
        let direct = self.direct.map(|symbol| TsExportContribution {
            symbol,
            type_only_export: None,
        });
        direct.into_iter().chain(
            self.alias
                .into_iter()
                .flat_map(move |alias| alias.target.public_contributions(self.type_only_export)),
        )
    }

    fn public_symbols(self) -> impl Iterator<Item = SymbolId> + 'a {
        self.public_contributions()
            .map(|contribution| contribution.symbol)
    }

    fn conflict_symbols(self) -> impl Iterator<Item = SymbolId> + 'a {
        // A pure alias still checks its immediate lexical target. A name with
        // declarations of its own is already a public symbol: subsequent alias
        // edges contribute their exported meanings, not private bridge sources.
        let immediate_lexical_target =
            self.direct.is_none() && self.property_value == PropertyValue::Absent;
        self.direct
            .into_iter()
            .chain(self.alias.into_iter().flat_map(move |alias| {
                let contributions = if immediate_lexical_target {
                    alias.target.conflict_contributions()
                } else {
                    alias.target.public_contributions(None)
                };
                contributions.map(|contribution| contribution.symbol)
            }))
    }

    fn with_type_only_export(mut self, node: Option<NodeId>) -> Self {
        self.type_only_export = self.type_only_export.or(node);
        self
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TsNamespaceExportAliasIssue {
    Missing { name: AstUtf8, node: NodeId },
    NonLocal { name: AstUtf8, node: NodeId },
    Duplicate { node: NodeId, previous: NodeId },
    Conflict { node: NodeId, previous: NodeId },
}

#[derive(Clone, Copy)]
pub(crate) struct TsImportAliasConflictIssue {
    pub(crate) name: AstWtf8,
    pub(crate) node: NodeId,
    pub(crate) previous: NodeId,
    pub(crate) previous_flags: SymbolFlags,
}

#[derive(Clone, Copy)]
pub(crate) enum TsImportAliasIssue {
    Conflict(TsImportAliasConflictIssue),
    TypeOnlyExport { node: NodeId, export: NodeId },
}

enum TsNamespaceExportAliasLookup<'a> {
    Resolved(TsResolvedExportAliasTarget<'a>),
    Missing,
    NonLocal,
}

bitflags::bitflags! {
    /// TypeScript checker declaration spaces used when validating that merged
    /// declarations are either all exported or all local. These are narrower
    /// than reference/export capabilities: in particular, an unresolved
    /// import does not claim every space merely because it can be referenced
    /// in every space.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsDeclarationSpaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
    }
}

bitflags::bitflags! {
    /// Value, type, and namespace spaces contributed by a resolved internal
    /// TypeScript alias, plus uncertainty from opaque dependencies. Known bits
    /// prove contributions; UNKNOWN only permits conservative reference access.
    /// These results must not be written back into [`SymbolFlags`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsResolvedAliasSpaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
        // Opaque dependencies may contribute any reference space, but do not
        // prove a declaration-space conflict. Propagates with the known bits.
        const UNKNOWN = 1 << 3;
        // Properties and methods have value meaning, but are excluded by
        // TypeScript's ModuleMember mask inside merged namespace bodies.
        const PROPERTY_VALUE = 1 << 4;
        // Unresolved property names may add value meaning, but do not prove
        // existence, a conflict, or type/namespace contributions.
        const UNKNOWN_PROPERTY_VALUE = 1 << 5;
    }
}

impl TsResolvedAliasSpaces {
    fn known(self) -> Self {
        self.difference(Self::UNKNOWN | Self::UNKNOWN_PROPERTY_VALUE)
    }

    fn visibility(self, space: ReferenceSpace, flags: SymbolFlags) -> BindingVisibility {
        if self.known().visible_in(space, flags) {
            BindingVisibility::Present
        } else if self.visible_in(space, flags) {
            BindingVisibility::Unknown
        } else {
            BindingVisibility::Absent
        }
    }

    fn may_have(self, space: Self) -> bool {
        self.intersects(space | Self::UNKNOWN)
            || (space.contains(Self::PROPERTY_VALUE) && self.contains(Self::UNKNOWN_PROPERTY_VALUE))
    }

    fn may_have_value(self) -> bool {
        self.may_have(Self::VALUE | Self::PROPERTY_VALUE)
    }

    fn declaration_spaces(mut self) -> Self {
        if self.contains(Self::PROPERTY_VALUE) {
            self.remove(Self::PROPERTY_VALUE);
            self.insert(Self::VALUE);
        }
        self.intersection(Self::VALUE | Self::TYPE | Self::NAMESPACE)
    }

    fn visible_in(self, space: ReferenceSpace, flags: SymbolFlags) -> bool {
        match space {
            ReferenceSpace::Value => {
                // Type-only restricts the import contribution, not an
                // independent value declaration merged into the same symbol.
                flags.in_value_space()
                    || (flags.contains(SymbolFlags::IMPORT) && self.may_have_value())
            }
            ReferenceSpace::Typeof => self.may_have_value(),
            ReferenceSpace::Type => self.may_have(Self::TYPE),
            ReferenceSpace::Namespace => self.may_have(Self::NAMESPACE),
            ReferenceSpace::Any => true,
        }
    }
}

/// A `(start, len)` window into a backing slice.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Range {
    pub(crate) start: u32,
    pub(crate) len: u32,
}

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
        use_ranges: &mut [Range],
    ) {
        reference.symbol = resolution.candidate();
        if let Some(symbol) = reference.symbol {
            use_ranges[symbol.index()].len += 1;
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
        use_ranges: &mut [Range],
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
            use_ranges[symbol.index()].len += 1;
        }
    }
}

/// Value projections for references that selected an alias-backed member.
/// Other references use their canonical candidates without additional storage.
#[derive(Debug)]
struct TsReferenceValues<'a> {
    ranges: ArenaHashMap<'a, ReferenceId, Range>,
    targets: ArenaVec<'a, NameResolution>,
}

impl<'a> TsReferenceValues<'a> {
    fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            ranges: ArenaHashMap::new_in(allocator),
            targets: ArenaVec::new_in(allocator),
        }
    }

    fn get(&self, reference: ReferenceId) -> Option<&[NameResolution]> {
        let range = self.ranges.get(&reference)?;
        let start = range.start as usize;
        Some(&self.targets[start..start + range.len as usize])
    }

    fn record(&mut self, reference: ReferenceId, targets: &[NameResolution]) {
        self.ranges.insert(
            reference,
            Range {
                start: self.targets.len() as u32,
                len: targets.len() as u32,
            },
        );
        self.targets.extend(targets.iter().copied());
    }
}

#[derive(Debug)]
struct TsScopeViewIndex<'a> {
    ranges: ArenaVec<'a, Range>,
    lookup: ArenaVec<'a, TsScopeLookup>,
}

#[derive(Debug, Default, Clone, Copy)]
struct TsScopeLookup {
    // The nearest member overlay also bounds the ordinary lexical prefix.
    members: Option<ScopeId>,
    // An ancestor signature or conditional-type view can restrict lexical lookup.
    restricted_lexical: bool,
}

/// Source-file locals and exports have different identities. Named exports
/// use the common member environment; default declarations share its `default`
/// entry even when their local names differ. Local export links preserve TS7's
/// ExportValue and LocalSymbol.ExportSymbol relationship without duplicating
/// declaration ownership.
#[derive(Debug)]
struct TsSourceExports<'a> {
    owner: SymbolId,
    environment: TsMemberEnvironmentId,
    implicit_exports: bool,
    default: Option<TsDefaultExport<'a>>,
}

#[derive(Debug)]
struct TsDefaultExport<'a> {
    symbol: SymbolId,
    first_local: Option<&'a [u8]>,
    first_local_is_function: bool,
}

#[derive(Debug, Clone, Copy)]
struct TsExportLocal {
    target: SymbolId,
    target_is_function: bool,
    exported_value: bool,
    default_declarations: Option<TsDefaultLocal>,
}

#[derive(Debug, Clone, Copy)]
struct TsDefaultLocal {
    node: NodeId,
    flags: SymbolFlags,
}

/// TS7 local export links belong to a declaration body, even when its public
/// member environment is shared across several namespace bodies. Recording
/// them during binding prevents same-named members in other bodies from
/// acquiring a local ExportValue relationship.
type TsLocalExportMap<'a> = ArenaHashMap<'a, (ScopeId, &'a [u8]), TsExportLocal>;

impl TsExportLocal {
    fn exported_value(self) -> Option<SymbolId> {
        self.exported_value.then_some(self.target)
    }
}

fn ts_exported_value(
    locals: &TsLocalExportMap<'_>,
    symbols: &[Symbol],
    declarations: &[Declaration],
    scope_maps: &[ScopeMap<'_>],
    lexical: SymbolId,
    name: &[u8],
) -> Option<SymbolId> {
    let first = symbols[lexical.index()].declaration;
    let scope = declarations[first.index()].scope;
    let target = locals.get(&(scope, name))?.exported_value()?;
    (scope_maps[scope.index()].get(name).copied() == Some(lexical)).then_some(target)
}

struct TsValueTargetLookup<'lookup, 'arena> {
    locals: &'lookup TsLocalExportMap<'arena>,
    symbols: &'lookup [Symbol],
    declarations: &'lookup [Declaration],
    scope_maps: &'lookup [ScopeMap<'arena>],
    import_alias_spaces: &'lookup [Option<TsResolvedAliasSpaces>],
}

impl TsValueTargetLookup<'_, '_> {
    fn target<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        symbol: SymbolId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        debug_assert!(matches!(
            space,
            ReferenceSpace::Value | ReferenceSpace::Typeof
        ));
        let symbol = ts_exported_value(
            self.locals,
            self.symbols,
            self.declarations,
            self.scope_maps,
            symbol,
            name,
        )
        .unwrap_or(symbol);
        match symbol_visibility::<PRECISE_IMPORT_ALIASES>(
            self.symbols,
            self.import_alias_spaces,
            symbol,
            space,
        ) {
            BindingVisibility::Absent => NameResolution::Unresolved,
            BindingVisibility::Present => NameResolution::Resolved(symbol),
            BindingVisibility::Unknown => NameResolution::Uncertain { candidate: symbol },
        }
    }
}

impl TsSourceExports<'_> {
    fn lookup(
        &self,
        name: &[u8],
        space: ReferenceSpace,
        symbols: &[Symbol],
        locals: &TsLocalExportMap<'_>,
    ) -> Option<SymbolId> {
        if matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof)
            && let Some(symbol) = locals
                .get(&(ScopeId::MODULE, name))
                .and_then(|local| local.exported_value())
            && symbols[symbol.index()].flags.visible_in(space)
        {
            return Some(symbol);
        }
        let default = self.default.as_ref()?;
        (default.first_local == Some(name)
            && symbols[default.symbol.index()].flags.visible_in(space))
        .then_some(default.symbol)
    }
}

/// Collects symbols and references during the AST walk and finalizes
/// them into a [`Semantic`].
pub(crate) struct SymbolTracker<'a> {
    allocator: &'a Allocator,
    internal_names: crate::InternalNames<'a>,
    symbols: ArenaVec<'a, Symbol>,
    references: ArenaVec<'a, Reference>,
    jsx_usages: ArenaVec<'a, crate::JsxUsage>,
    declarations: ArenaVec<'a, Declaration>,
    redeclarations: Redeclarations<'a>,
    node_semantic: ArenaVec<'a, NodeSemanticId>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, HoistingMap<'a>>,
    annex_b_functions: ArenaVec<'a, AnnexBFunction>,
    ts_type_parameter_owners: ArenaHashMap<'a, ScopeId, SymbolId>,
    ts_type_parameters: ArenaHashMap<'a, SymbolId, ScopeMap<'a>>,
    ts_source_exports: Option<TsSourceExports<'a>>,
    ts_local_exports: TsLocalExportMap<'a>,
    ts_member_environments: TsMemberEnvironments<'a>,
    ts_global_owner: Option<SymbolId>,
    ts_member_views: ArenaVec<'a, TsMemberView>,
    ts_scope_view_records: ArenaVec<'a, (ScopeId, Range)>,
    ts_scope_view_index: Option<TsScopeViewIndex<'a>>,
    pending_ts_namespace_export_aliases: ArenaVec<'a, PendingTsNamespaceExportAlias<'a>>,
    ts_import_alias_targets: ArenaVec<'a, TsImportAliasTarget>,
    ts_expando_assignments: ArenaVec<'a, (MemberExpression, TsExpandoBindingSite)>,
    ts_import_aliases: Option<TsImportAliasIndex<'a>>,
    ts_container_stack: Vec<TsContainerContext>,
    ts_module_instance_scratch: HashMap<NodeId, TsModuleInstanceMemo, FxBuildHasher>,
    ts_enum_owner_stack: Vec<SymbolId>,

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
    Named {
        statement: NodeId,
    },
    Default,
}

impl ExportState {
    fn is_exported(self) -> bool {
        !matches!(self, Self::None)
    }

    fn is_default(self) -> bool {
        matches!(self, Self::Default)
    }

    pub(crate) fn named(self) -> Option<NodeId> {
        match self {
            Self::Named { statement } => Some(statement),
            Self::None | Self::Default => None,
        }
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
        let mut node_semantic = ArenaVec::with_capacity_in(nodes, allocator);
        node_semantic.resize(nodes, NodeSemanticId::None);
        let mut tracker = Self {
            allocator,
            internal_names: crate::InternalNames::new_in(allocator),
            symbols: ArenaVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            references: ArenaVec::with_capacity_in(16usize.max(nodes / 4), allocator),
            jsx_usages: ArenaVec::new_in(allocator),
            declarations: ArenaVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            redeclarations: Redeclarations::new_in(allocator),
            node_semantic,
            scope_maps: ArenaVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            hoisting_variables: ArenaVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            annex_b_functions: ArenaVec::new_in(allocator),
            ts_type_parameter_owners: ArenaHashMap::new_in(allocator),
            ts_type_parameters: ArenaHashMap::new_in(allocator),
            ts_source_exports: None,
            ts_local_exports: ArenaHashMap::new_in(allocator),
            ts_member_environments: TsMemberEnvironments::new_in(allocator),
            ts_global_owner: None,
            ts_member_views: ArenaVec::new_in(allocator),
            ts_scope_view_records: ArenaVec::new_in(allocator),
            ts_scope_view_index: None,
            pending_ts_namespace_export_aliases: ArenaVec::new_in(allocator),
            ts_import_alias_targets: ArenaVec::new_in(allocator),
            ts_expando_assignments: ArenaVec::new_in(allocator),
            ts_import_aliases: None,
            ts_container_stack: Vec::new(),
            ts_module_instance_scratch: HashMap::with_hasher(FxBuildHasher),
            ts_enum_owner_stack: Vec::new(),
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

    fn init_ts_source_exports(&mut self, tree: &Ast<'a>) {
        let program = tree.root_program();
        let (owner, _) = self.declare_symbol(
            tree,
            PendingDeclaration {
                name: tree.empty_wtf8(),
                node: program.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::VALUE_MODULE | SymbolFlags::NAMESPACE_MODULE,
                    excludes: SymbolFlags::empty(),
                    scope: ScopeId::ROOT,
                },
            },
            None,
            false,
        );
        let environment = self.ts_member_environments.ensure(owner, self.allocator);
        let start = self.ts_member_views.len();
        self.ts_member_views.push(TsMemberView {
            environment,
            kind: TsMemberViewKind::SourceFile,
        });
        self.record_ts_scope_views(
            ScopeId::MODULE,
            Range {
                start: start as u32,
                len: 1,
            },
        );
        let implicit_exports = self.ambient
            && !program.body(tree).iter().any(|statement| {
                let statement = tree.get_node_in_sub_range(statement).node_id();
                ts_statement_is_export_declaration(tree, statement)
            });
        self.ts_source_exports = Some(TsSourceExports {
            owner,
            environment,
            implicit_exports,
            default: None,
        });
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
            self.export_state = ExportState::Named {
                statement: declaration.node_id(),
            };
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
        scope: &ScopeTracker<'a>,
    ) {
        self.push_saved_context();
        let kind = declaration.kind(tree);
        self.pending = match kind {
            VariableKind::Var => {
                let target = scope.hoist_target();
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
                    scope: scope.current,
                }
            }
            VariableKind::Let => PendingBinding {
                flags: SymbolFlags::BLOCK_SCOPED_VAR.with(
                    SymbolFlags::AMBIENT,
                    declaration.declare(tree) || self.ambient,
                ),
                excludes: Excludes::BLOCK_SCOPED_VAR,
                scope: scope.current,
            },
        };
    }

    pub fn enter_function(
        &mut self,
        tree: &Ast<'a>,
        function: Function,
        scope: &ScopeTracker<'a>,
        annex_b_if_body: bool,
    ) {
        self.push_saved_context();
        let is_declaration = function.function_type(tree) == FunctionType::FunctionDeclaration;
        let enclosing = scope
            .current_scope()
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
            scope.current
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

    pub fn enter_ts_function(
        &mut self,
        tree: &Ast<'a>,
        function: TsFunction,
        scope: &ScopeTracker<'a>,
    ) {
        self.push_saved_context();
        let target = scope
            .current_scope()
            .parent
            .expect("a TypeScript function scope has a parent");
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION | SymbolFlags::AMBIENT,
            excludes: Excludes::FUNCTION,
            scope: target,
        };
        if function.id(tree).is_none() {
            self.declare_anonymous_default(tree, function.node_id());
        }
    }

    pub fn enter_class(&mut self, tree: &Ast<'a>, class: Class, scope: &ScopeTracker<'a>) {
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
                    .current_scope()
                    .parent
                    .expect("a class declaration scope has a parent")
            } else {
                scope.current
            },
        };
        // Expression names are local.
        if !is_declaration {
            self.export_state = ExportState::None;
        } else if class.id(tree).is_none() {
            self.declare_anonymous_default(tree, class.node_id());
        }
    }

    pub fn enter_formal_parameters(&mut self, scope: &ScopeTracker<'a>) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: (SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER).with(
                SymbolFlags::AMBIENT,
                self.pending.flags.contains(SymbolFlags::AMBIENT),
            ),
            excludes: Excludes::PARAMETER,
            scope: scope.current,
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

    pub fn enter_ts_module_block(&mut self, scope: &ScopeTracker<'a>) {
        self.export_state = ExportState::None;
        let context = self
            .ts_container_stack
            .last_mut()
            .expect("a TypeScript module block has a context");
        let TsContainerContext::Member {
            views, body_scope, ..
        } = context
        else {
            return;
        };
        *body_scope = Some(scope.current);
        let views = *views;
        self.record_ts_scope_views(scope.current, views);
    }

    pub fn enter_ts_enum_body(&mut self, scope: &ScopeTracker<'a>) {
        self.export_state = ExportState::None;
        let owner = *self
            .ts_enum_owner_stack
            .last()
            .expect("an enum body has an owner");
        let environment = self.ts_member_environments.ensure(owner, self.allocator);
        let start = self.ts_member_views.len();
        self.ts_member_views.push(TsMemberView {
            environment,
            kind: TsMemberViewKind::Enum,
        });
        self.ts_scope_view_records.push((
            scope.current,
            Range {
                start: start as u32,
                len: 1,
            },
        ));
    }

    pub fn enter_ts_enum_members(&mut self, owner: SymbolId) {
        self.ts_enum_owner_stack.push(owner);
    }

    pub fn exit_ts_enum_members(&mut self, owner: SymbolId) {
        let exited = self.ts_enum_owner_stack.pop();
        debug_assert_eq!(exited, Some(owner));
    }

    pub fn enter_ts_module_body(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsModuleDeclaration,
    ) -> Vec<PreparedDeclaration> {
        let name = declaration.id(tree);
        let (declarations, views) = self.prepare_ts_module_views(tree, name);
        let body = declaration.body(tree).map(|body| body.node_id());
        let context = if views.len == 0 {
            TsContainerContext::UnsupportedExternalModule { body }
        } else {
            // The body's own environment precedes qualified-name ancestors.
            let environment = self.ts_member_views[views.start as usize].environment;
            TsContainerContext::Member {
                environment,
                views,
                projection_scope: None,
                body,
                body_scope: None,
                implicit_exports: self.ambient
                    && !ts_module_has_export_declarations(tree, declaration.body(tree)),
                public_exports: true,
            }
        };
        self.ts_container_stack.push(context);
        declarations
    }

    pub fn exit_ts_module_body(&mut self) {
        self.ts_container_stack
            .pop()
            .expect("a TypeScript module body owner exists");
    }

    pub fn enter_import_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: ImportDeclaration,
        scope: &ScopeTracker<'a>,
    ) {
        self.push_saved_context();
        self.pending = import_pending(declaration.import_kind(tree), scope.current);
    }

    pub fn enter_ts_import_equals_declaration(
        &mut self,
        import_kind: ImportOrExportKind,
        scope: &ScopeTracker<'a>,
    ) {
        self.push_saved_context();
        self.pending = import_pending(import_kind, scope.current);
        self.pending.flags |= SymbolFlags::IMPORT_EQUALS;
    }

    pub fn enter_import_specifier(&mut self, tree: &Ast, specifier: ImportSpecifier) {
        self.push_saved_context();
        if specifier.import_kind(tree) == ImportOrExportKind::Type {
            self.pending.flags = SymbolFlags::TYPE_IMPORT;
            self.pending.excludes = Excludes::IMPORT_BINDING;
        }
    }

    pub fn enter_catch_clause(&mut self, scope: &ScopeTracker<'a>) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::CATCH_VAR,
            excludes: Excludes::CATCH_VAR,
            scope: scope.current,
        };
    }

    pub fn enter_ts_interface_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsInterfaceDeclaration,
        scope: &ScopeTracker<'a>,
    ) {
        // The id binds in the surrounding scope. Type-parameter and
        // body bindings live in a scope pushed by the tracker.
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::INTERFACE.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: Excludes::INTERFACE,
            scope: scope
                .current_scope()
                .parent
                .expect("an interface scope has a parent"),
        };
    }

    pub fn enter_ts_type_alias_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsTypeAliasDeclaration,
        scope: &ScopeTracker<'a>,
    ) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_ALIAS.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: Excludes::TYPE_ALIAS,
            scope: scope
                .current_scope()
                .parent
                .expect("a type alias scope has a parent"),
        };
    }

    pub fn enter_ts_enum_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsEnumDeclaration,
        scope: &ScopeTracker<'a>,
    ) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: if declaration.is_const(tree) {
                SymbolFlags::CONST_ENUM
            } else {
                SymbolFlags::REGULAR_ENUM
            }
            .with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: if declaration.is_const(tree) {
                Excludes::CONST_ENUM
            } else {
                Excludes::REGULAR_ENUM
            },
            scope: scope.current,
        };
    }

    pub fn enter_ts_module_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsModuleDeclaration,
        scope: &ScopeTracker<'a>,
    ) {
        self.push_saved_context();
        if matches!(
            tree.ts_module_name_data(declaration.id(tree)),
            TsModuleNameData::StringLiteral(_)
        ) {
            if declaration.declare(tree) {
                self.ambient = true;
            }
            return;
        }
        let state = ts_module_instance_state(
            tree,
            declaration,
            &self.ts_container_stack,
            &mut self.ts_module_instance_scratch,
        );
        let instantiated = state != TsModuleInstanceState::NonInstantiated;
        let mut flags = SymbolFlags::NAMESPACE_MODULE.with(SymbolFlags::VALUE_MODULE, instantiated);
        flags.set(
            SymbolFlags::CONST_ENUM_ONLY_MODULE,
            state == TsModuleInstanceState::ConstEnumOnly,
        );
        self.pending = PendingBinding {
            flags: flags.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: if instantiated {
                Excludes::VALUE_MODULE
            } else {
                Excludes::NAMESPACE_MODULE
            },
            scope: scope.current,
        };
        // `declare` makes the whole body ambient.
        if declaration.declare(tree) {
            self.ambient = true;
        }
    }

    pub fn enter_ts_global_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsGlobalDeclaration,
    ) {
        self.push_saved_context();
        let identifier = declaration.id(tree);
        let existing = self.ts_global_owner;
        let (owner, _) = self.declare_symbol(
            tree,
            PendingDeclaration {
                name: identifier.name(tree).into(),
                node: identifier.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::NAMESPACE_MODULE | SymbolFlags::AMBIENT,
                    excludes: Excludes::NAMESPACE_MODULE,
                    scope: ScopeId::ROOT,
                },
            },
            existing,
            true,
        );
        let views = self.append_ts_namespace_views([owner], TsMemberViewKind::Module);
        if existing.is_none() {
            self.ts_global_owner = Some(owner);
        }
        self.ambient = true;
        let environment = self.ts_member_views[views.start as usize].environment;
        self.ts_container_stack.push(TsContainerContext::Member {
            environment,
            views,
            projection_scope: Some(ScopeId::ROOT),
            body: Some(declaration.body(tree).node_id()),
            body_scope: None,
            implicit_exports: true,
            public_exports: false,
        });
    }

    pub fn exit_ts_global_declaration(&mut self) {
        self.ts_container_stack
            .pop()
            .expect("a TypeScript global body context exists");
        self.exit_saved_context();
    }

    pub fn enter_ts_type_parameter(
        &mut self,
        tree: &Ast<'a>,
        parent: Option<NodeId>,
        scope: &ScopeTracker<'a>,
    ) {
        // `TsTypeParameter` wraps `<T>` and `infer U`.
        self.push_saved_context();
        // `infer` declares in the enclosing conditional scope so the
        // true branch sees it, even from inside a signature type.
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            infer_owner(scope)
        } else if scope.current_scope().kind == ScopeKind::TsTypeParameters {
            scope
                .current_scope()
                .parent
                .expect("type parameters have an owning declaration scope")
        } else {
            scope.current
        };
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_PARAMETER,
            excludes: Excludes::TYPE_PARAMETER,
            scope: target,
        };
        // TS7 stores class/interface parameters in the merged owner's Members,
        // but only exposes parameters declared by the current occurrence. Keep
        // that identity index separate from both lexical scopes and exports.
        let owner = match tree.kind_data(scope.get(target).node) {
            NodeKindData::Class(class) if class.class_type(tree) == ClassType::ClassDeclaration => {
                if let Some(id) = class.id(tree) {
                    self.symbol_of_node(id.node_id())
                } else {
                    self.default_export_symbol()
                }
            }
            NodeKindData::TsInterfaceDeclaration(interface) => {
                self.symbol_of_node(interface.id(tree).node_id())
            }
            _ => None,
        };
        if let Some(owner) = owner {
            self.ts_type_parameter_owners.insert(target, owner);
        }
        self.export_state = ExportState::None;
    }

    pub fn enter_ts_mapped_type(
        &mut self,
        tree: &Ast<'a>,
        parent: Option<NodeId>,
        scope: &ScopeTracker<'a>,
    ) {
        // Mapped type keys `[K in T]` are bare binding identifiers, so the
        // same pending context as a type parameter applies.
        self.push_saved_context();
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            infer_owner(scope)
        } else {
            scope.current
        };
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_PARAMETER,
            excludes: Excludes::TYPE_PARAMETER,
            scope: target,
        };
        self.export_state = ExportState::None;
    }

    pub fn enter_ts_function_type(&mut self, scope: &ScopeTracker<'a>) {
        self.enter_signature_type(scope);
    }

    pub fn enter_ts_constructor_type(&mut self, scope: &ScopeTracker<'a>) {
        self.enter_signature_type(scope);
    }

    pub fn enter_ts_method_signature(&mut self, scope: &ScopeTracker<'a>) {
        self.enter_signature_type(scope);
    }

    pub fn enter_ts_call_signature_declaration(&mut self, scope: &ScopeTracker<'a>) {
        self.enter_signature_type(scope);
    }

    pub fn enter_ts_construct_signature_declaration(&mut self, scope: &ScopeTracker<'a>) {
        self.enter_signature_type(scope);
    }

    pub fn enter_ts_index_signature(&mut self, scope: &ScopeTracker<'a>) {
        self.enter_signature_type(scope);
    }

    fn enter_signature_type(&mut self, scope: &ScopeTracker<'a>) {
        // Erasure does not remove signature parameters from the value namespace.
        // FormalParameters uses the same role; index signatures bind directly.
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
            excludes: Excludes::PARAMETER,
            scope: scope.current,
        };
        self.export_state = ExportState::None;
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

    #[inline(always)]
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
                !self.symbols[symbol.index()]
                    .flags
                    .intersects(SymbolFlags::INTERNAL_NAME)
            });
        let (destination, shared, existing) = if self.pending_is_default_export(tree) {
            let shared = self
                .ts_source_exports
                .as_ref()
                .and_then(|source| source.default.as_ref())
                .map(|default| default.symbol);
            (DeclarationDestination::TsDefaultExport, shared, shared)
        } else if let Some(owner) = self.pending_type_parameter_owner() {
            let shared = self
                .ts_type_parameters
                .get(&owner)
                .and_then(|parameters| parameters.get(name_text).copied());
            (
                DeclarationDestination::TsTypeParameter { owner },
                shared,
                local.or(shared),
            )
        } else if let Some(environment) = self.pending_ts_source_destination() {
            let shared = self.ts_member_environments.member(environment, name_text);
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
                .ts_member_environments
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
            self.ts_source_exports.as_ref().and_then(|export| {
                let local = self.ts_local_exports.get(&(ScopeId::MODULE, name_text))?;
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
        scope: &ScopeTracker<'a>,
    ) -> bool {
        let Some((node, name)) = self.binding_identifier_declaration(tree, identifier, scope)
        else {
            return false;
        };
        let declaration_id = self.declare(tree, name.into(), node);
        self.finish_binding_identifier(tree, name, declaration_id, scope);
        true
    }

    pub(crate) fn declare_prepared_binding_identifier(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        scope: &ScopeTracker<'a>,
        prepared: PreparedDeclaration,
    ) -> bool {
        let Some((node, name)) = self.binding_identifier_declaration(tree, identifier, scope)
        else {
            return false;
        };
        debug_assert_eq!(prepared.node(), node);
        debug_assert_eq!(prepared.name(), name.into());
        let declaration_id = self.declare_prepared(tree, prepared);
        self.finish_binding_identifier(tree, name, declaration_id, scope);
        true
    }

    pub(crate) fn prepare_ts_enum_member(
        &mut self,
        tree: &Ast<'a>,
        member: TsEnumMember,
        scope: &ScopeTracker<'a>,
    ) -> Option<PreparedDeclaration> {
        self.sync_scope_maps_from(scope);
        // Members are lexically visible inside the enum body:
        //
        //   enum E { a, b = a }
        //   //              ^ resolves to the member
        //
        // Member names are not BindingIdentifiers, so they declare here.
        // Literal computed names denote the same member as their unwrapped
        // string or no-substitution template form.
        let member_name = member.id(tree).node_id();
        let name = text_member_name(tree, member_name, false)?;
        let owner = *self
            .ts_enum_owner_stack
            .last()
            .expect("an enum member has an owner");
        let flags = SymbolFlags::ENUM_MEMBER.with(SymbolFlags::AMBIENT, self.ambient);
        let environment = self.ts_member_environments.ensure(owner, self.allocator);
        let name_text = tree.get_wtf8(name).as_bytes();
        let shared = self
            .ts_member_environments
            .environment(environment)
            .get(name_text)
            .map(|entry| entry.symbol);
        let local = self.own_binding(scope.current, name_text);
        let prepared = PreparedDeclaration {
            declaration: PendingDeclaration {
                name,
                node: member.node_id(),
                binding: PendingBinding {
                    flags,
                    excludes: Excludes::ENUM_MEMBER,
                    scope: scope.current,
                },
            },
            destination: DeclarationDestination::TsMember {
                environment,
                body_scope: Some(scope.current),
                projection_scope: None,
            },
            local,
            shared,
            existing: shared,
            default_local: None,
        };
        Some(prepared)
    }

    pub(crate) fn declare_prepared_ts_enum_member(
        &mut self,
        tree: &Ast<'a>,
        prepared: PreparedDeclaration,
    ) {
        debug_assert!(prepared.flags().contains(SymbolFlags::ENUM_MEMBER));
        self.declare_prepared(tree, prepared);
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
        if self.ts_container_stack.is_empty()
            && self.pending_type_parameter_owner().is_none()
            && (self.pending.scope != ScopeId::MODULE
                || (!self.pending_is_default_export(tree)
                    && self.pending_ts_source_destination().is_none()))
        {
            let target = self.pending.scope;
            let mut declaration_flags = self.pending.flags;
            declaration_flags.set(SymbolFlags::EXPORTED, self.export_state.is_exported());
            declaration_flags.set(SymbolFlags::IS_DEFAULT, self.export_state.is_default());
            let declaration_id = DeclarationId::from_usize(self.declarations.len());
            let bindings = &mut self.scope_maps[target.index()];
            let hash = bindings.hasher().hash_one(name_text);
            let entry = bindings
                .raw_entry_mut()
                .from_key_hashed_nocheck(hash, name_text);
            // The internal name shares the function scope, but is shadowed
            // rather than redeclared by parameters and local declarations.
            // Keep its symbol and AST identity while replacing the name map.
            let (entry, symbol_name) = match entry {
                hashbrown::hash_map::RawEntryMut::Occupied(entry)
                    if self.symbols[entry.get().index()]
                        .flags
                        .intersects(SymbolFlags::INTERNAL_NAME) =>
                {
                    let symbol_name = self.symbols[entry.get().index()].name;
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
                    let symbol = &mut self.symbols[existing.index()];
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
                    let new_id = SymbolId::from_usize(self.symbols.len());
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
            self.node_semantic[node.index()] = NodeSemanticId::Symbol(id);
            return declaration_id;
        }

        self.declare_ts_context_binding(tree, name, node)
    }

    fn record_ts_scope_views(&mut self, scope: ScopeId, views: Range) {
        self.ts_scope_view_records.push((scope, views));
    }

    fn append_ts_namespace_views(
        &mut self,
        owners: impl IntoIterator<Item = SymbolId>,
        kind: TsMemberViewKind,
    ) -> Range {
        let start = self.ts_member_views.len();
        for owner in owners {
            let environment = self.ts_member_environments.ensure(owner, self.allocator);
            self.ts_member_views
                .push(TsMemberView { environment, kind });
        }
        Range {
            start: start as u32,
            len: (self.ts_member_views.len() - start) as u32,
        }
    }

    #[inline(always)]
    fn pending_ts_namespace_destination(&self) -> Option<(TsMemberEnvironmentId, Option<ScopeId>)> {
        let TsContainerContext::Member {
            environment,
            projection_scope,
            body_scope,
            implicit_exports,
            ..
        } = *self.ts_container_stack.last()?
        else {
            return None;
        };
        let implicitly_exported = implicit_exports
            && body_scope == Some(self.pending.scope)
            && !self.pending.flags.intersects(SymbolFlags::ANY_IMPORT);
        (self.export_state.is_exported() || implicitly_exported)
            .then_some((environment, projection_scope))
    }

    fn pending_ts_source_destination(&self) -> Option<TsMemberEnvironmentId> {
        if self.pending.scope != ScopeId::MODULE {
            return None;
        }
        let source = self.ts_source_exports.as_ref()?;
        let implicit =
            source.implicit_exports && !self.pending.flags.intersects(SymbolFlags::ANY_IMPORT);
        (self.export_state.is_exported() || implicit).then_some(source.environment)
    }

    fn current_ts_namespace_export_environment(
        &self,
        scope: ScopeId,
    ) -> Option<TsMemberEnvironmentId> {
        if scope == ScopeId::MODULE
            && let Some(source) = &self.ts_source_exports
        {
            return Some(source.environment);
        }
        let TsContainerContext::Member {
            environment,
            body_scope,
            public_exports,
            ..
        } = *self.ts_container_stack.last()?
        else {
            return None;
        };
        (public_exports && body_scope == Some(scope)).then_some(environment)
    }

    #[inline(always)]
    fn current_ts_body_member(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        if scope == ScopeId::MODULE
            && let Some(source) = &self.ts_source_exports
        {
            return self.ts_member_environments.member(source.environment, name);
        }
        let TsContainerContext::Member {
            environment,
            body_scope,
            ..
        } = *self.ts_container_stack.last()?
        else {
            return None;
        };
        if body_scope != Some(scope) {
            return None;
        }
        let member = self.ts_member_environments.member(environment, name)?;
        self.symbol_declared_in_scope(member, scope)
            .then_some(member)
    }

    #[inline(always)]
    fn prepare_ts_module_views(
        &mut self,
        tree: &Ast<'a>,
        name: TsModuleName,
    ) -> (Vec<PreparedDeclaration>, Range) {
        let kind = if self.ambient {
            TsMemberViewKind::AmbientModule
        } else {
            TsMemberViewKind::Module
        };
        match tree.ts_module_name_data(name) {
            TsModuleNameData::BindingIdentifier(identifier) => {
                let owner = self.node_semantic[identifier.node_id().index()].symbol();
                let views = owner.map_or_else(Range::default, |owner| {
                    self.append_ts_namespace_views([owner], kind)
                });
                (Vec::new(), views)
            }
            TsModuleNameData::StringLiteral(_) => (Vec::new(), Range::default()),
            TsModuleNameData::TsQualifiedName(qualified) => {
                let mut members = Vec::new();
                let Some(root) = collect_ts_module_path(tree, qualified, &mut members) else {
                    return (Vec::new(), Range::default());
                };
                let Some(mut owner) = self.node_semantic[root.index()].symbol() else {
                    return (Vec::new(), Range::default());
                };
                let mut owners = Vec::with_capacity(members.len() + 1);
                owners.push(owner);
                let mut declarations = Vec::new();
                for (name, node) in members {
                    let parent = owner;
                    let mut flags = self.pending.flags;
                    flags.insert(SymbolFlags::EXPORTED);
                    let declaration = PendingDeclaration {
                        name,
                        node,
                        binding: PendingBinding {
                            flags,
                            ..self.pending
                        },
                    };
                    let (next_owner, prepared) =
                        self.declare_ts_namespace_member(tree, parent, declaration, None);
                    owner = next_owner;
                    if prepared.shared.is_some() {
                        declarations.push(prepared);
                    }
                    owners.push(owner);
                }
                let views = self.append_ts_namespace_views(owners.into_iter().rev(), kind);
                (declarations, views)
            }
        }
    }

    pub fn record_ts_global_export(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsNamespaceExportDeclaration,
        scope: &ScopeTracker<'a>,
    ) {
        if scope.current != ScopeId::MODULE || tree.lang() != swc_next_ecma_ast::Lang::Dts {
            return;
        }
        let identifier = declaration.id(tree);
        self.declare_symbol(
            tree,
            PendingDeclaration {
                name: identifier.name(tree).into(),
                node: identifier.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::UMD_EXPORT,
                    excludes: SymbolFlags::empty(),
                    scope: scope.current,
                },
            },
            None,
            true,
        );
    }

    fn pending_type_parameter_owner(&self) -> Option<SymbolId> {
        if !self.pending.flags.contains(SymbolFlags::TYPE_PARAMETER) {
            return None;
        }
        self.ts_type_parameter_owners
            .get(&self.pending.scope)
            .copied()
    }

    fn pending_is_default_export(&self, tree: &Ast) -> bool {
        tree.is_ts()
            && self.ts_source_exports.is_some()
            && self.export_state.is_default()
            && self.pending.scope == ScopeId::MODULE
            && self
                .pending
                .flags
                .intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS | SymbolFlags::INTERFACE)
    }

    fn declare_anonymous_default(&mut self, tree: &Ast<'a>, node: NodeId) {
        if !self.pending_is_default_export(tree) {
            return;
        }
        let declaration = PendingDeclaration {
            name: tree.empty_wtf8(),
            node,
            binding: PendingBinding {
                flags: self.pending.flags | SymbolFlags::EXPORTED | SymbolFlags::IS_DEFAULT,
                ..self.pending
            },
        };
        self.declare_default_export(tree, declaration, false);
    }

    fn declare_default_export(
        &mut self,
        tree: &Ast<'a>,
        declaration: PendingDeclaration,
        named: bool,
    ) -> DeclarationId {
        let mut source = self
            .ts_source_exports
            .take()
            .expect("a default export belongs to a source module");
        let environment = source.environment;
        let existing = source.default.as_ref().map(|default| default.symbol);
        let merges =
            existing.is_none_or(|existing| self.declaration_merges(existing, declaration.binding));
        let (symbol, declaration_id) = self.declare_symbol(tree, declaration, existing, named);
        let default = source.default.get_or_insert(TsDefaultExport {
            symbol,
            first_local: None,
            first_local_is_function: false,
        });
        if named {
            let name = tree.get_wtf8(declaration.name).as_bytes();
            let function = declaration.binding.flags.contains(SymbolFlags::FUNCTION);
            if default.first_local.is_none() || function && !default.first_local_is_function {
                default.first_local = Some(name);
                default.first_local_is_function = function;
            }
        }
        self.ts_source_exports = Some(source);
        if merges {
            self.record_ts_member(
                environment,
                b"default",
                symbol,
                TsMemberKinds::MODULE,
                declaration.binding.flags,
            );
        }
        if named {
            self.record_ts_export_local(tree, declaration, symbol);
        }
        declaration_id
    }

    fn record_ts_export_local(
        &mut self,
        tree: &Ast<'a>,
        declaration: PendingDeclaration,
        symbol: SymbolId,
    ) {
        let flags = declaration.binding.flags;
        // TS7 declares exported import-equals aliases only in Exports.
        if flags.intersects(SymbolFlags::ANY_IMPORT) {
            return;
        }
        let name = tree.get_wtf8(declaration.name).as_bytes();
        let function = flags.contains(SymbolFlags::FUNCTION);
        let scope = declaration.binding.scope;
        let local = self
            .ts_local_exports
            .entry((scope, name))
            .or_insert(TsExportLocal {
                target: symbol,
                target_is_function: function,
                exported_value: false,
                default_declarations: None,
            });
        local.exported_value |= flags.in_value_space();
        // declareModuleMember replaces LocalSymbol.ExportSymbol on every
        // export declaration. TS7 visits statement-list functions first, then
        // all other statements, each in source order.
        if local.target_is_function || !function {
            local.target = symbol;
            local.target_is_function = function;
        }
        if flags.contains(SymbolFlags::IS_DEFAULT) && scope == ScopeId::MODULE {
            match &mut local.default_declarations {
                Some(local) => local.flags = merge_symbol_flags(local.flags, flags),
                local @ None => {
                    *local = Some(TsDefaultLocal {
                        node: declaration.node,
                        flags,
                    })
                }
            }
        }
    }

    /// An infer parameter is visible in its own constraint as well as the
    /// conditional true branch, without exposing its siblings' infer names.
    pub fn expose_ts_infer_parameter(
        &mut self,
        tree: &Ast<'a>,
        name: BindingIdentifier,
        scope: &ScopeTracker<'a>,
    ) {
        self.sync_scope_maps_from(scope);
        let symbol = self
            .symbol_of_node(name.node_id())
            .expect("the infer parameter is already declared");
        self.insert_scope_binding(tree, scope.current, name.name(tree).into(), symbol);
    }

    #[inline(always)]
    fn binding_identifier_declaration(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        scope: &ScopeTracker<'a>,
    ) -> Option<(NodeId, AstUtf8)> {
        self.sync_scope_maps_from(scope);
        // The declaration role comes from the enclosing syntax, independently
        // of whether references in that syntax produce runtime access.
        if self.pending.flags.is_empty() {
            return None;
        }
        Some((identifier.node_id(), identifier.name(tree)))
    }

    #[inline(always)]
    fn finish_binding_identifier(
        &mut self,
        tree: &Ast<'a>,
        name: AstUtf8,
        declaration_id: DeclarationId,
        scope: &ScopeTracker<'a>,
    ) {
        // Register a hoisting var in every block it passes through so
        // block-scoped redeclarations see it.
        let declaration = self.declarations[declaration_id.index()];
        if declaration.flags.is_hoisting_var() && scope.current != declaration.scope {
            let name = tree.get_utf8(name).as_bytes();
            for scope_id in scope.ancestors(Some(scope.current)) {
                if scope_id == declaration.scope {
                    break;
                }
                self.hoisting_variables[scope_id.index()]
                    .get_or_insert_with(|| {
                        self.allocator.boxed(ArenaHashMap::new_in(self.allocator))
                    })
                    .entry(name)
                    .or_insert(declaration_id);
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn declare_ts_context_binding(
        &mut self,
        tree: &Ast<'a>,
        name: AstWtf8,
        node: NodeId,
    ) -> DeclarationId {
        let plan = self.prepare_declaration(tree, name, node);
        self.declare_prepared(tree, plan)
    }

    #[inline(always)]
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
                    .ts_type_parameters
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

    fn declare_ts_namespace_member(
        &mut self,
        tree: &Ast<'a>,
        owner: SymbolId,
        declaration: PendingDeclaration,
        actual_scope: Option<ScopeId>,
    ) -> (SymbolId, PreparedDeclaration) {
        let name = declaration.name;
        let name_text = tree.get_wtf8(name).as_bytes();
        let environment = self.ts_member_environments.ensure(owner, self.allocator);
        let shared_existing = self.ts_member_environments.member(environment, name_text);
        let local_existing = actual_scope.and_then(|scope| self.own_binding(scope, name_text));
        let target = actual_scope.unwrap_or(declaration.binding.scope);
        let mut declaration = declaration;
        declaration.binding.scope = target;
        let prepared = PreparedDeclaration {
            declaration,
            destination: DeclarationDestination::TsMember {
                environment,
                body_scope: actual_scope,
                projection_scope: None,
            },
            local: local_existing,
            shared: shared_existing,
            existing: shared_existing,
            default_local: None,
        };
        let declaration_id = self.declare_prepared(tree, prepared);
        let symbol = self.declarations[declaration_id.index()].symbol;
        (symbol, prepared)
    }

    fn declare_ts_member_in_environment(
        &mut self,
        tree: &Ast<'a>,
        environment: TsMemberEnvironmentId,
        body_scope: Option<ScopeId>,
        projection_scope: Option<ScopeId>,
        mut declaration: PendingDeclaration,
        existing: Option<SymbolId>,
    ) -> DeclarationId {
        let name = declaration.name;
        let name_text = tree.get_wtf8(name).as_bytes();
        let kind = if declaration.binding.flags.contains(SymbolFlags::ENUM_MEMBER) {
            TsMemberKinds::ENUM
        } else {
            TsMemberKinds::MODULE
        };
        let symbol_scope = body_scope.unwrap_or(declaration.binding.scope);
        declaration.binding.scope = symbol_scope;
        let merges =
            existing.is_none_or(|existing| self.declaration_merges(existing, declaration.binding));
        let (id, declaration_id) = self.declare_symbol(tree, declaration, existing, true);

        if merges {
            self.record_ts_member(environment, name_text, id, kind, declaration.binding.flags);
            if let Some(scope) = projection_scope {
                if let Some(projected) = self.binding(scope, name_text) {
                    debug_assert_eq!(projected, id);
                } else {
                    self.insert_scope_binding(tree, scope, name, id);
                }
            }
        }
        if kind == TsMemberKinds::MODULE && body_scope.is_some() {
            self.record_ts_export_local(tree, declaration, id);
        }
        declaration_id
    }

    fn record_ts_member(
        &mut self,
        environment: TsMemberEnvironmentId,
        name: &'a [u8],
        symbol: SymbolId,
        kind: TsMemberKinds,
        flags: SymbolFlags,
    ) {
        let spaces = TsMemberSpaces::from_flags(flags);
        let module_declaration =
            kind == TsMemberKinds::MODULE && !flags.intersects(SymbolFlags::ANY_IMPORT);
        let members = self.ts_member_environments.environment_mut(environment);
        let hash = members.hasher().hash_one(name);
        match members.raw_entry_mut().from_key_hashed_nocheck(hash, name) {
            hashbrown::hash_map::RawEntryMut::Occupied(mut entry) => {
                let entry = entry.get_mut();
                debug_assert_eq!(entry.symbol, symbol);
                entry.kinds.insert(kind);
                entry.module_declaration |= module_declaration;
                if kind == TsMemberKinds::MODULE {
                    entry
                        .module_spaces
                        .insert(TsMemberVisibility::certain(spaces));
                } else {
                    entry.enum_spaces.insert(spaces);
                }
            }
            hashbrown::hash_map::RawEntryMut::Vacant(entry) => {
                let (module_spaces, enum_spaces) = if kind == TsMemberKinds::MODULE {
                    (spaces, TsMemberSpaces::empty())
                } else {
                    (TsMemberSpaces::empty(), spaces)
                };
                entry.insert_hashed_nocheck(
                    hash,
                    name,
                    TsMemberEntry {
                        symbol,
                        kinds: kind,
                        module_spaces: TsMemberVisibility::certain(module_spaces),
                        enum_spaces,
                        module_declaration,
                    },
                );
            }
        }
    }

    #[inline(always)]
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
        let declaration_id = DeclarationId::from_usize(self.declarations.len());
        let id = if let Some(existing) = existing {
            let merges = self.declaration_merges(existing, binding);
            let symbol = &mut self.symbols[existing.index()];
            if merges {
                symbol.flags = merge_symbol_flags(symbol.flags, declaration_flags);
            }
            self.redeclarations
                .entry(existing)
                .or_insert_with(|| ArenaVec::with_capacity_in(1, self.allocator))
                .push(declaration_id);
            existing
        } else {
            let new_id = SymbolId::from_usize(self.symbols.len());
            self.symbols.push(Symbol {
                name: self
                    .scope_maps
                    .get(binding.scope.index())
                    .and_then(|bindings| bindings.get(tree.get_wtf8(name).as_bytes()))
                    .map(|&symbol| self.symbols[symbol.index()])
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
                self.node_semantic[node.index()],
                NodeSemanticId::None | NodeSemanticId::Symbol(_)
            ));
            self.node_semantic[node.index()] = NodeSemanticId::Symbol(id);
        }
        (id, declaration_id)
    }

    fn record_internal_name(&mut self, id: SymbolId) {
        let symbol = self.symbols[id.index()];
        if symbol.flags.intersects(SymbolFlags::INTERNAL_NAME) {
            let scope = self.declarations[symbol.declaration.index()].scope;
            self.internal_names
                .entry((scope, symbol.name))
                .or_insert(id);
        }
    }

    #[inline(always)]
    fn insert_scope_binding(
        &mut self,
        tree: &Ast<'a>,
        scope: ScopeId,
        name: AstWtf8,
        id: SymbolId,
    ) {
        let name = tree.get_wtf8(name).as_bytes();
        self.scope_maps[scope.index()].insert(name, id);
    }

    fn sync_scope_maps_from(&mut self, scope: &ScopeTracker<'a>) {
        if self.scope_maps.len() < scope.scopes.len() {
            self.sync_scope_maps(scope.scopes.len());
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

    pub fn record_ts_import_equals_target(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsImportEqualsDeclaration,
    ) {
        let alias = self
            .symbol_of_node(declaration.id(tree).node_id())
            .expect("an import-equals binding has a symbol");
        let Some(root) = crate::ecmascript::import_equals_reference_node(tree, declaration) else {
            return;
        };
        // The module reference has just been visited, so the root is normally
        // the last reference. Match the node instead of relying on that order.
        let root_reference = self
            .references
            .iter()
            .rposition(|reference| reference.node == root)
            .map(ReferenceId::from_usize);
        debug_assert!(
            root_reference.is_some(),
            "an internal import target has a root reference"
        );
        let Some(root_reference) = root_reference else {
            return;
        };
        self.ts_import_alias_targets.push(TsImportAliasTarget {
            alias,
            declaration,
            root_reference,
        });
    }

    pub fn add_identifier_reference(
        &mut self,
        tree: &Ast<'a>,
        identifier: IdentifierReference,
        scope: &ScopeTracker<'a>,
        flags: ReferenceFlags,
    ) {
        self.add_reference(
            tree,
            identifier.name(tree),
            scope.current,
            identifier.node_id(),
            flags,
        );
    }

    pub fn add_local_export_reference(
        &mut self,
        tree: &Ast<'a>,
        specifier: ExportSpecifier,
        scope: &ScopeTracker<'a>,
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
            scope.current,
            local.node_id(),
            flags,
        );
        let Some(environment) = self.current_ts_namespace_export_environment(scope.current) else {
            return;
        };
        let exported_name = ts_module_export_name(tree, specifier.exported(tree));
        let exported_name = tree.get_wtf8(exported_name).as_bytes();
        let aliases = self.ts_member_environments.export_aliases_mut(environment);
        let previous = aliases.get(exported_name).map(|entry| entry.node);
        if previous.is_none() {
            aliases.insert(
                exported_name,
                TsNamespaceExportAliasEntry {
                    type_only,
                    resolution: TsExportAliasResolution::Unresolved,
                    node: specifier.node_id(),
                },
            );
        }
        self.pending_ts_namespace_export_aliases
            .push(PendingTsNamespaceExportAlias {
                type_only,
                environment,
                exported_name,
                local_reference,
                node: specifier.node_id(),
                previous,
            });
    }

    /// Save property assignments with TS7's binder containers. These are not
    /// lexical queries: crossing a function or namespace changes ownership.
    pub fn record_ts_expando_assignment(
        &mut self,
        tree: &Ast<'a>,
        node: AssignmentExpression,
        scope: &ScopeTracker<'a>,
        ancestors: impl Iterator<Item = NodeId>,
    ) {
        if !tree.lang().is_typescript() || node.operator(tree) != AssignmentOperator::Assign {
            return;
        }
        let left = crate::ecmascript::unwrap_parentheses(tree, node.left(tree).node_id());
        let NodeKindData::MemberExpression(member) = tree.kind_data(left) else {
            return;
        };
        if !member_facts::is_expando_assignment(tree, member) {
            return;
        }
        let site = TsExpandoBindingSite::new(tree, scope, ancestors);
        self.ts_expando_assignments.push((member, site));
    }

    pub fn add_ts_type_predicate_reference(
        &mut self,
        tree: &Ast<'a>,
        predicate: TsTypePredicate,
        scope: &ScopeTracker<'a>,
    ) {
        self.sync_scope_maps_from(scope);
        // `param is T` parses `param` as an IdentifierName but it
        // references the enclosing function or signature parameter. Its
        // binding identity is preserved even when the signature is erased.
        let parameter_name = predicate.parameter_name(tree);
        let parameter_node = parameter_name.node_id();
        let NodeKindData::IdentifierName(parameter_name) = tree.kind_data(parameter_node) else {
            return;
        };
        let name = parameter_name.name(tree);
        let parameter_scope = if scope.current_scope().kind == ScopeKind::TsSignatureReturn {
            scope
                .current_scope()
                .parent
                .expect("a return annotation has an owning signature")
        } else {
            scope.current
        };
        let Some(parameter) = self.own_binding(parameter_scope, tree.get_utf8(name).as_bytes())
        else {
            return;
        };
        if !self
            .symbol(parameter)
            .flags
            .contains(SymbolFlags::PARAMETER)
        {
            return;
        }
        self.add_reference(
            tree,
            name,
            scope.current,
            parameter_node,
            ReferenceFlags::default()
                .with_space(ReferenceSpace::Typeof)
                .no_access(),
        );
    }

    pub fn record_jsx_usage(&mut self, scope: ScopeId, is_fragment: bool) {
        // Adjacent JSX in one environment has the same implicit dependencies.
        // Keep scope summaries instead of allocating an entry for every tag.
        if let Some(usage) = self.jsx_usages.last_mut()
            && usage.scope == scope
        {
            usage.is_fragment |= is_fragment;
            return;
        }
        self.jsx_usages.push(crate::JsxUsage { scope, is_fragment });
    }

    pub fn add_jsx_opening_element_reference(
        &mut self,
        tree: &Ast<'a>,
        element: JsxOpeningElement,
        scope: &ScopeTracker<'a>,
    ) {
        // The root of a component tag is a JS binding
        // reference. Lowercase tags are intrinsic element names; the
        // rest of a member chain is property syntax.
        self.add_jsx_reference(tree, element.name(tree), scope.current);
    }

    pub fn add_jsx_closing_element_reference(
        &mut self,
        tree: &Ast<'a>,
        element: JsxClosingElement,
        scope: &ScopeTracker<'a>,
    ) {
        self.add_jsx_reference(tree, element.name(tree), scope.current);
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
        let id = ReferenceId::from_usize(self.references.len());
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
    pub fn in_ambient_context(&self) -> bool {
        self.ambient || self.pending.flags.contains(SymbolFlags::AMBIENT)
    }

    pub(crate) fn in_unsupported_ts_external_module(&self) -> bool {
        matches!(
            self.ts_container_stack.last(),
            Some(TsContainerContext::UnsupportedExternalModule { .. })
        )
    }

    #[inline(always)]
    fn declaration_merges(&self, existing: SymbolId, binding: PendingBinding) -> bool {
        let existing = self.symbols[existing.index()].flags;
        !existing.intersects(binding.excludes)
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id.index()]
    }

    #[inline]
    pub(crate) fn conflict_candidate_info(
        &self,
        candidate: DeclarationConflictCandidate,
    ) -> (SymbolFlags, NodeId) {
        match candidate.kind {
            DeclarationConflictKind::DefaultLocal(local) => return (local.flags, local.node),
            DeclarationConflictKind::Symbol => {
                let symbol = self.symbols[candidate.symbol.index()];
                let declaration = self.declarations[symbol.declaration.index()];
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
        let symbol = self.symbols[candidate.symbol.index()];
        let first = std::iter::once(symbol.declaration);
        let rest = self
            .redeclarations
            .get(&candidate.symbol)
            .into_iter()
            .flatten()
            .copied();
        first
            .chain(rest)
            .map(|declaration| self.declarations[declaration.index()])
            .filter(move |declaration| match candidate.kind {
                DeclarationConflictKind::Scope(scope) => declaration.scope == scope,
                _ => true,
            })
    }

    pub fn symbol_of_node(&self, node: NodeId) -> Option<SymbolId> {
        self.node_semantic[node.index()].symbol()
    }

    fn default_export_symbol(&self) -> Option<SymbolId> {
        self.ts_source_exports
            .as_ref()?
            .default
            .as_ref()
            .map(|default| default.symbol)
    }

    /// The `binding_identifier` node of the earliest declaration of `id`.
    /// Later declarations merged into the symbol do not change it.
    pub fn first_decl_of(&self, id: SymbolId) -> NodeId {
        let declaration_id = self.symbols[id.index()].declaration;
        self.declarations[declaration_id.index()].node
    }

    /// The binding of `name` declared directly in `scope`, excluding
    /// hoisting `var`s passing through.
    pub fn own_binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps
            .get(scope.index())
            .and_then(|map| map.get(name).copied())
    }

    fn symbol_declared_in_scope(&self, symbol: SymbolId, scope: ScopeId) -> bool {
        let first = self.symbols[symbol.index()].declaration;
        if self.declarations[first.index()].scope == scope {
            return true;
        }
        self.redeclarations
            .get(&symbol)
            .is_some_and(|declarations| {
                declarations
                    .iter()
                    .any(|declaration| self.declarations[declaration.index()].scope == scope)
            })
    }

    /// Checks whether an existing symbol has an overlapping declaration in
    /// the same binding scope whose exported/local visibility differs from
    /// the incoming declaration.
    /// TypeScript permits that difference across merged namespace bodies and
    /// across disjoint declaration spaces, but not within one body and space.
    #[cold]
    #[inline(never)]
    pub(crate) fn has_same_scope_export_visibility_conflict(
        &self,
        prepared: PreparedDeclaration,
        symbol: SymbolId,
    ) -> bool {
        // Default exports are checked by TypeScript's separate TS2652 rule.
        // This path implements only the named-export/local TS2395 invariant.
        let flags = prepared.flags();
        let target = prepared.target();
        if flags.contains(SymbolFlags::IS_DEFAULT) {
            return false;
        }
        let incoming_exported = flags.contains(SymbolFlags::EXPORTED);
        let incoming_spaces = ts_declaration_spaces(flags);
        if incoming_spaces.is_empty() {
            return false;
        }
        let first = self.symbols[symbol.index()].declaration;
        if self.declaration_has_export_visibility_conflict(
            first,
            target,
            incoming_exported,
            incoming_spaces,
        ) {
            return true;
        }
        self.redeclarations
            .get(&symbol)
            .is_some_and(|declarations| {
                declarations.iter().copied().any(|declaration| {
                    self.declaration_has_export_visibility_conflict(
                        declaration,
                        target,
                        incoming_exported,
                        incoming_spaces,
                    )
                })
            })
    }

    fn declaration_has_export_visibility_conflict(
        &self,
        declaration: DeclarationId,
        scope: ScopeId,
        incoming_exported: bool,
        incoming_spaces: TsDeclarationSpaces,
    ) -> bool {
        let declaration = self.declarations[declaration.index()];
        declaration.scope == scope
            && !declaration.flags.contains(SymbolFlags::IS_DEFAULT)
            && declaration.flags.contains(SymbolFlags::EXPORTED) != incoming_exported
            && ts_declaration_spaces(declaration.flags).intersects(incoming_spaces)
    }

    /// The binding of `name` at `scope`, including a hoisting `var` passing
    /// through on its way to its hoist target.
    pub fn binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.own_binding(scope, name).or_else(|| {
            self.hoisting_declaration(scope, name)
                .map(|declaration| self.declarations[declaration.index()].symbol)
        })
    }

    fn hoisting_declaration(&self, scope: ScopeId, name: &[u8]) -> Option<DeclarationId> {
        self.hoisting_variables
            .get(scope.index())
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
        Some(self.declarations[declaration.index()])
    }

    /// Follow a recorded local ExportValue link without changing the local
    /// symbol used for conflict checks or qualified namespace ownership.
    fn ts_exported_value(&self, lexical: SymbolId, name: &[u8]) -> Option<SymbolId> {
        ts_exported_value(
            &self.ts_local_exports,
            &self.symbols,
            &self.declarations,
            &self.scope_maps,
            lexical,
            name,
        )
    }

    fn ts_member_declared_in_scope(
        &self,
        scope: ScopeId,
        name: &[u8],
        scope_views: &TsScopeViewIndex<'_>,
    ) -> Option<SymbolId> {
        let range = scope_views.ranges[scope.index()];
        if range.len == 0 {
            return None;
        }
        // The body's own container view precedes qualified-name ancestors.
        // Restrict the result to a contribution declared in this body, since
        // the environment itself is shared by all merged bodies.
        let view = self.ts_member_views[range.start as usize];
        self.ts_member_environments
            .environment(view.environment)
            .get(name)
            .filter(|entry| {
                view.kind.accepts(entry.kinds) && self.symbol_declared_in_scope(entry.symbol, scope)
            })
            .map(|entry| entry.symbol)
    }

    #[cold]
    #[inline(never)]
    fn ts_public_member(&self, owner: SymbolId, name: &[u8]) -> Option<TsResolvedImportTarget<'a>> {
        // TypeScript's public export symbol can combine a direct member and
        // an export alias under one name. Keep both contributions here rather
        // than mutating either canonical symbol with the other's meanings.
        let environment = self.ts_member_environments.get(owner)?;
        let direct = environment.members.get(name).map(|entry| entry.symbol);
        let alias = environment.export_aliases.get(name);
        TsResolvedImportTarget::from_members(direct, alias)
    }

    /// Finalizes the tracker into a complete [`Semantic`]. Builds the
    /// declaration, use, and node indexes and resolves every reference
    /// to its declaring symbol. The result aliases tracker storage and
    /// remains valid for the source tree's arena lifetime.
    pub fn finalize(
        mut self,
        tree: &Ast<'a>,
        scopes: ScopeTree<'a>,
        module_record: ModuleRecordBuilder<'a>,
    ) -> Semantic<'a> {
        // The walk must have ended balanced.
        debug_assert!(self.saved_stack.is_empty());
        debug_assert_eq!(self.export_state, ExportState::None);
        debug_assert!(self.ts_container_stack.is_empty());
        debug_assert!(self.ts_enum_owner_stack.is_empty());
        self.sync_scope_maps(scopes.list.len());
        let ts_scope_views = self
            .ts_scope_view_index
            .take()
            .or_else(|| self.finalize_ts_scope_views(&scopes.list));
        let symbol_count = self.symbols.len();

        let mut use_ranges = ArenaVec::with_capacity_in(symbol_count, self.allocator);
        use_ranges.resize(symbol_count, Range::default());

        let mut reference_candidates = ReferenceCandidates::new_in(self.allocator);
        let mut reference_values = TsReferenceValues::new_in(self.allocator);
        let precise_import_aliases = self.ts_import_aliases.is_some();
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
            for (index, reference) in self.references.iter().enumerate() {
                if let Some(symbol) = reference.symbol {
                    let range = &mut use_ranges[symbol.index()];
                    use_ids[(range.start + range.len) as usize] = ReferenceId::from_usize(index);
                    range.len += 1;
                }
            }
        } else {
            for (index, reference) in self.references.iter().enumerate() {
                let id = ReferenceId::from_usize(index);
                let targets = reference
                    .symbol
                    .into_iter()
                    .chain(reference_candidates.additional(id).iter().copied());
                for symbol in targets {
                    let range = &mut use_ranges[symbol.index()];
                    use_ids[(range.start + range.len) as usize] = id;
                    range.len += 1;
                }
            }
        }

        for (index, reference) in self.references.iter().enumerate() {
            let slot = &mut self.node_semantic[reference.node.index()];
            debug_assert!(matches!(
                *slot,
                NodeSemanticId::None | NodeSemanticId::Reference(_)
            ));
            *slot = NodeSemanticId::Reference(ReferenceId::from_usize(index));
        }
        for (index, scope) in scopes.list.iter().enumerate() {
            let slot = &mut self.node_semantic[scope.node.index()];
            debug_assert!(matches!(
                *slot,
                NodeSemanticId::None | NodeSemanticId::Scope(_)
            ));
            *slot = NodeSemanticId::Scope(ScopeId::from_usize(index));
        }
        let module_record = module_record.finalize(&self.node_semantic, &self.references);
        let declarations = self.declarations;
        let redeclarations = self.redeclarations;
        let node_semantic = self.node_semantic;

        let reference_values = if reference_values.ranges.is_empty() {
            None
        } else {
            Some(self.allocator.boxed(reference_values))
        };

        Semantic {
            scopes,
            symbols: self.symbols,
            references: self.references,
            jsx_usages: self.jsx_usages,
            reference_candidates,
            declarations,
            redeclarations,
            module_record,
            use_ids,
            use_ranges,
            scope_maps: self.scope_maps,
            internal_names: self.internal_names,
            hoisting_variables: self.hoisting_variables,
            node_semantic,
            ts_members: TsMemberIndex {
                source_exports: self.ts_source_exports,
                local_exports: self.ts_local_exports,
                environments: self.ts_member_environments,
                scope_views: ts_scope_views,
                views: self.ts_member_views,
                import_aliases: self.ts_import_aliases,
                reference_values,
            },
        }
    }

    #[cold]
    #[inline(never)]
    fn finalize_ts_scope_views(&self, scopes: &[Scope]) -> Option<TsScopeViewIndex<'a>> {
        if self.ts_scope_view_records.is_empty() {
            return None;
        }
        let scope_count = scopes.len();
        let mut ranges = ArenaVec::with_capacity_in(scope_count, self.allocator);
        ranges.resize(scope_count, Range::default());
        for &(scope, range) in &self.ts_scope_view_records {
            debug_assert_eq!(ranges[scope.index()].len, 0);
            ranges[scope.index()] = range;
        }
        let mut lookup: ArenaVec<TsScopeLookup> =
            ArenaVec::with_capacity_in(scope_count, self.allocator);
        for (index, scope) in scopes.iter().enumerate() {
            let parent = scope
                .parent
                .map(|parent| lookup[parent.index()])
                .unwrap_or_default();
            lookup.push(TsScopeLookup {
                members: if ranges[index].len != 0 {
                    Some(ScopeId::from_usize(index))
                } else {
                    parent.members
                },
                restricted_lexical: scope.kind.is_ts_lookup_view() || parent.restricted_lexical,
            });
        }
        Some(TsScopeViewIndex { ranges, lookup })
    }

    fn resolve_references_with_ts_views<const PRECISE_IMPORT_ALIASES: bool>(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'_>,
        scope_views: &TsScopeViewIndex<'_>,
        use_ranges: &mut [Range],
        additional_targets: &mut ReferenceCandidates<'a>,
    ) -> TsReferenceValues<'a> {
        if self.ts_member_environments.needs_value_projections() {
            self.resolve_references_with_ts_views_policy::<PRECISE_IMPORT_ALIASES, true>(
                tree,
                scopes,
                scope_views,
                use_ranges,
                additional_targets,
            )
        } else {
            self.resolve_references_with_ts_views_policy::<PRECISE_IMPORT_ALIASES, false>(
                tree,
                scopes,
                scope_views,
                use_ranges,
                additional_targets,
            )
        }
    }

    #[cold]
    #[inline(never)]
    fn resolve_references_with_ts_views_policy<
        const PRECISE_IMPORT_ALIASES: bool,
        const MEMBER_VALUES: bool,
    >(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'_>,
        scope_views: &TsScopeViewIndex<'_>,
        use_ranges: &mut [Range],
        additional_targets: &mut ReferenceCandidates<'a>,
    ) -> TsReferenceValues<'a> {
        let mut reference_values = TsReferenceValues::new_in(self.allocator);
        let import_alias_spaces = self
            .ts_import_aliases
            .as_ref()
            .map(|aliases| aliases.spaces.as_slice())
            .unwrap_or_default();
        let mut candidates = Vec::new();
        let mut bindings = Vec::new();
        let mut values = Vec::new();
        for reference_index in 0..self.references.len() {
            candidates.clear();
            bindings.clear();
            let reference = self.references[reference_index];
            let id = ReferenceId::from_usize(reference_index);
            let name = tree.get_utf8(reference.name).as_bytes();
            let query = NameLookupQuery {
                scope: reference.scope,
                name,
                space: reference.flags.space,
            };
            let result = lookup_name_with_ts_views::<true, PRECISE_IMPORT_ALIASES, MEMBER_VALUES>(
                LexicalLookup {
                    scopes: &scopes.list,
                    symbols: &self.symbols,
                    internal_names: &self.internal_names,
                    scope_maps: &self.scope_maps,
                },
                &TsNameLookup {
                    source_exports: self.ts_source_exports.as_ref(),
                    local_exports: &self.ts_local_exports,
                    environments: &self.ts_member_environments,
                    views: &self.ts_member_views,
                    import_alias_spaces,
                },
                scope_views,
                query,
                |binding| {
                    if MEMBER_VALUES {
                        bindings.push(binding);
                    } else {
                        candidates.push(binding.symbol);
                    }
                },
            );
            let resolved = result.resolution();
            let has_member = MEMBER_VALUES
                && (result
                    .binding
                    .is_some_and(|binding| binding.member.is_some())
                    || bindings.iter().any(|binding| binding.member.is_some()));
            let selected = result.binding.into_iter().chain(bindings.iter().copied());
            if has_member {
                values.clear();
                self.collect_binding_values::<PRECISE_IMPORT_ALIASES>(
                    name,
                    selected,
                    matches!(resolved, NameResolution::Uncertain { .. }),
                    reference.flags.space.value_target_space(),
                    &mut values,
                );
                if reference.flags.space == ReferenceSpace::Any && values.is_empty() {
                    self.references[reference_index].flags = reference.flags.no_access();
                }
                reference_values.record(id, &values);
            } else if reference.flags.space == ReferenceSpace::Any {
                self.references[reference_index].flags = self
                    .resolved_reference_flags::<PRECISE_IMPORT_ALIASES>(tree, reference, resolved);
            }
            if MEMBER_VALUES {
                candidates.extend(bindings.iter().map(|binding| binding.symbol));
            }
            additional_targets.record(
                id,
                &mut self.references[reference_index],
                resolved,
                &mut candidates,
                use_ranges,
            );
        }
        reference_values
    }

    fn resolve_lexical_references<
        const PRECISE_IMPORT_ALIASES: bool,
        const TS_LOOKUP: bool,
        const FILTER_BY_SPACE: bool,
    >(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'_>,
        use_ranges: &mut [Range],
        additional_targets: &mut ReferenceCandidates<'a>,
    ) {
        let import_alias_spaces = self
            .ts_import_aliases
            .as_ref()
            .map(|aliases| aliases.spaces.as_slice())
            .unwrap_or_default();
        let mut candidates = Vec::new();
        for reference_index in 0..self.references.len() {
            candidates.clear();
            let reference = self.references[reference_index];
            let id = ReferenceId::from_usize(reference_index);
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
            let lexical = LexicalLookup {
                scopes: &scopes.list,
                symbols: &self.symbols,
                internal_names: &self.internal_names,
                scope_maps: &self.scope_maps,
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
                self.references[reference_index].flags = self
                    .resolved_reference_flags::<PRECISE_IMPORT_ALIASES>(tree, reference, resolved);
            }
            additional_targets.record(
                id,
                &mut self.references[reference_index],
                resolved,
                &mut candidates,
                use_ranges,
            );
        }
    }

    fn collect_binding_values<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        name: &[u8],
        bindings: impl Iterator<Item = NameBinding>,
        uncertain: bool,
        space: ReferenceSpace,
        targets: &mut Vec<NameResolution>,
    ) {
        let spaces = self
            .ts_import_aliases
            .as_ref()
            .map(|aliases| aliases.spaces.as_slice())
            .unwrap_or_default();
        let hash = self.scope_maps[ScopeId::ROOT.index()]
            .hasher()
            .hash_one(name);
        let value_targets = TsValueTargetLookup {
            locals: &self.ts_local_exports,
            symbols: &self.symbols,
            declarations: &self.declarations,
            scope_maps: &self.scope_maps,
            import_alias_spaces: spaces,
        };
        let mut accept = |symbol, visibility| {
            let value = value_targets.target::<PRECISE_IMPORT_ALIASES>(symbol, name, space);
            if let Some(candidate) = value.candidate() {
                let value = if uncertain || visibility == BindingVisibility::Unknown {
                    NameResolution::Uncertain { candidate }
                } else {
                    value
                };
                if !targets.contains(&value) {
                    targets.push(value);
                }
            }
            visibility == BindingVisibility::Present
        };
        for binding in bindings {
            if let Some(view) = binding.member {
                let member = view
                    .lookup(&self.ts_member_environments, hash, name)
                    .expect("the selected member remains in its environment");
                member.visit_candidates::<PRECISE_IMPORT_ALIASES>(
                    &self.symbols,
                    spaces,
                    space,
                    &mut accept,
                );
            } else {
                accept(binding.symbol, BindingVisibility::Present);
            }
        }
    }

    fn resolved_reference_flags<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        tree: &Ast<'a>,
        reference: Reference,
        resolved: NameResolution,
    ) -> ReferenceFlags {
        debug_assert_eq!(reference.flags.space, ReferenceSpace::Any);
        let Some(symbol) = resolved.candidate() else {
            return reference.flags;
        };
        let name = tree.get_utf8(reference.name).as_bytes();
        let spaces = self
            .ts_import_aliases
            .as_ref()
            .map(|aliases| aliases.spaces.as_slice())
            .unwrap_or_default();
        let target = TsValueTargetLookup {
            locals: &self.ts_local_exports,
            symbols: &self.symbols,
            declarations: &self.declarations,
            scope_maps: &self.scope_maps,
            import_alias_spaces: spaces,
        }
        .target::<PRECISE_IMPORT_ALIASES>(symbol, name, ReferenceSpace::Value);
        if target.candidate().is_some() {
            reference.flags
        } else {
            reference.flags.no_access()
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

fn infer_owner(scope: &ScopeTracker<'_>) -> ScopeId {
    // Like TS7's getInferTypeContainer, select the nearest conditional whose
    // extends subtree contains the declaration, even across nested branches.
    for id in scope.ancestors(Some(scope.current)) {
        let region = scope.get(id);
        if region.kind == ScopeKind::TsConditionalExtends {
            return region
                .parent
                .expect("an extends view has a conditional owner");
        }
    }
    scope.current
}

fn collect_ts_module_path(
    tree: &Ast<'_>,
    qualified: TsQualifiedName,
    members: &mut Vec<(AstWtf8, NodeId)>,
) -> Option<NodeId> {
    let root = match tree.ts_qualified_name_left_data(qualified.left(tree)) {
        TsQualifiedNameLeftData::BindingIdentifier(identifier) => identifier.node_id(),
        TsQualifiedNameLeftData::TsQualifiedName(left) => {
            collect_ts_module_path(tree, left, members)?
        }
        TsQualifiedNameLeftData::IdentifierReference(_)
        | TsQualifiedNameLeftData::IdentifierName(_)
        | TsQualifiedNameLeftData::ThisExpression(_) => return None,
    };
    let right = qualified.right(tree);
    members.push((right.name(tree).into(), right.node_id()));
    Some(root)
}

fn ts_module_export_name(tree: &Ast<'_>, name: ModuleExportName) -> AstWtf8 {
    match tree.module_export_name_data(name) {
        ModuleExportNameData::IdentifierName(identifier) => identifier.name(tree).into(),
        ModuleExportNameData::StringLiteral(literal) => literal.value(tree),
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

// TypeScript gives an ambient namespace body an implicit export context unless
// it contains an export declaration or export assignment. `export as
// namespace` does not change that context.
fn ts_module_has_export_declarations(
    tree: &Ast<'_>,
    body: Option<swc_next_ecma_ast::TsModuleBody>,
) -> bool {
    let Some(body) = body else {
        return false;
    };
    match tree.kind_data(body.node_id()) {
        NodeKindData::TsModuleBlock(block) => block.body(tree).iter().any(|statement| {
            let statement = tree.get_node_in_sub_range(statement).node_id();
            ts_statement_is_export_declaration(tree, statement)
        }),
        NodeKindData::TsModuleDeclaration(declaration) => {
            ts_module_has_export_declarations(tree, declaration.body(tree))
        }
        _ => false,
    }
}

fn ts_statement_is_export_declaration(tree: &Ast, statement: NodeId) -> bool {
    match tree.kind_data(statement) {
        NodeKindData::ExportNamedDeclaration(export) => export.declaration(tree).is_none(),
        NodeKindData::ExportDefaultDeclaration(export) => {
            export.declaration(tree).as_expr(tree).is_some()
        }
        NodeKindData::ExportAllDeclaration(_) | NodeKindData::TsExportAssignment(_) => true,
        _ => false,
    }
}

fn symbol_flags_clear_const_enum_only(flags: SymbolFlags) -> bool {
    flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS | SymbolFlags::REGULAR_ENUM)
        || flags.contains(SymbolFlags::VALUE_MODULE)
            && !flags.contains(SymbolFlags::CONST_ENUM_ONLY_MODULE)
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

fn ts_declaration_spaces(flags: SymbolFlags) -> TsDeclarationSpaces {
    if flags.intersects(SymbolFlags::ANY_IMPORT) {
        // TypeScript derives import-clause and namespace-import spaces from
        // the resolved target. This single-file binder intentionally does not
        // invent those target meanings when module resolution is unavailable.
        return TsDeclarationSpaces::empty();
    }

    let mut spaces = TsDeclarationSpaces::empty();
    if flags.intersects(SymbolFlags::INTERFACE | SymbolFlags::TYPE_ALIAS | SymbolFlags::ENUM_MEMBER)
    {
        spaces.insert(TsDeclarationSpaces::TYPE);
    }
    if flags.intersects(SymbolFlags::CLASS | SymbolFlags::REGULAR_ENUM | SymbolFlags::CONST_ENUM) {
        spaces.insert(TsDeclarationSpaces::VALUE | TsDeclarationSpaces::TYPE);
    }
    if flags.contains(SymbolFlags::VALUE_MODULE) {
        spaces.insert(TsDeclarationSpaces::VALUE | TsDeclarationSpaces::NAMESPACE);
    } else if flags.contains(SymbolFlags::NAMESPACE_MODULE) {
        spaces.insert(TsDeclarationSpaces::NAMESPACE);
    }
    if flags.intersects(
        SymbolFlags::FUNCTION_SCOPED_VAR
            | SymbolFlags::BLOCK_SCOPED_VAR
            | SymbolFlags::FUNCTION
            | SymbolFlags::ENUM_MEMBER,
    ) {
        spaces.insert(TsDeclarationSpaces::VALUE);
    }
    spaces
}

fn ts_resolved_alias_spaces(flags: SymbolFlags) -> TsResolvedAliasSpaces {
    let spaces = ts_declaration_spaces(flags);
    let mut resolved = TsResolvedAliasSpaces::empty();
    resolved.set(
        TsResolvedAliasSpaces::VALUE,
        spaces.contains(TsDeclarationSpaces::VALUE),
    );
    resolved.set(
        TsResolvedAliasSpaces::TYPE,
        spaces.contains(TsDeclarationSpaces::TYPE),
    );
    resolved.set(
        TsResolvedAliasSpaces::NAMESPACE,
        spaces.contains(TsDeclarationSpaces::NAMESPACE),
    );
    if flags.intersects(SymbolFlags::REGULAR_ENUM | SymbolFlags::CONST_ENUM) {
        resolved.insert(TsResolvedAliasSpaces::NAMESPACE);
    }
    resolved
}
