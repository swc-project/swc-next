use std::{hash::BuildHasher, iter::successors, mem::swap};

use swc_next_allocator::{
    Allocator, CloneIn,
    hash_map::HashMap as ArenaHashMap,
    vec::{ArenaIndexVec, ArenaVec},
};
use swc_next_ecma_ast::{
    Ast, AstUtf8, AstWtf8, BindingIdentifier, Class, ClassType, ExportNamedDeclaration,
    ExportSpecifier, Function, FunctionType, IdentifierReference, ImportDeclaration,
    ImportOrExportKind, ImportSpecifier, JsxClosingElement, JsxElementName, JsxElementNameData,
    JsxIdentifier, JsxMemberExpressionObjectData, JsxOpeningElement, NodeId, NodeKind,
    NodeKindData, ReferenceId, ScopeId, SymbolId, TsEnumDeclaration, TsEnumMember,
    TsInterfaceDeclaration, TsModuleDeclaration, TsTypeAliasDeclaration, TsTypePredicate,
    VariableDeclaration, VariableKind,
};

use crate::{
    DeclarationId, Declarations, HoistingMap, NodeSemanticId, Redeclarations, ScopeMap, ScopeMaps,
    Scopes, Semantic, Symbols,
    scope::{Scope, ScopeKind, ScopeTree},
};

mod annex_b;

use annex_b::AnnexBFunction;

/// A `(start, len)` window into a backing slice.
#[derive(Debug, Default, Clone, Copy)]
pub struct Range {
    pub start: u32,
    pub len: u32,
}

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
    /// The binding scope, including the hoist target for a `var`.
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

    /// Whether these declaration flags are visible in the requested reference space.
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
    pub const CLASS: SymbolFlags = SymbolFlags::VALUE_SPACE
        .union(SymbolFlags::TYPE_SPACE)
        .difference(SymbolFlags::VALUE_MODULE.union(SymbolFlags::INTERFACE));
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
    /// The lexical binding visible in the reference space.
    pub symbol: Option<SymbolId>,
    pub flags: ReferenceFlags,
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct ReferenceProperties: u8 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
    }
}

/// Whether evaluating a reference reads its previous value, writes a new
/// value, or both. Type-only references do neither.
///
/// Syntactic forms such as assignments, updates, destructuring targets, and
/// for-in/of targets are normalized to these access operations rather than
/// being exposed as separate reference kinds.
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

    /// Changes lookup space and clears runtime access for type positions.
    pub(crate) fn with_space(mut self, space: ReferenceSpace) -> Self {
        self.space = space;
        if space.in_type_position() {
            self.properties = ReferenceProperties::empty();
        }
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
        self.properties = ReferenceProperties::empty();
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
    /// Spaces used when traversing type syntax. This alone does not determine
    /// runtime access: `Namespace` also occurs in internal import aliases.
    pub fn in_type_position(self) -> bool {
        matches!(self, Self::Type | Self::Namespace | Self::Typeof)
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
        successors(Some(symbol), move |current| match self.links.get(current) {
            Some(link) if link.next != symbol => Some(link.next),
            _ => None,
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
        let left_next = match self.links.get(&left) {
            Some(link) => link.next,
            None => left,
        };
        let right_next = match self.links.get(&right) {
            Some(link) => link.next,
            None => right,
        };
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
    symbols: ArenaIndexVec<'a, SymbolId, Symbol>,
    symbol_links: SymbolLinks<'a>,
    references: ArenaIndexVec<'a, ReferenceId, Reference>,
    declarations: ArenaIndexVec<'a, DeclarationId, Declaration>,
    redeclarations: Redeclarations<'a>,
    node_semantic: ArenaIndexVec<'a, NodeId, NodeSemanticId>,
    scope_maps: ArenaIndexVec<'a, ScopeId, ScopeMap<'a>>,
    hoisting_variables: ArenaIndexVec<'a, ScopeId, HoistingMap<'a>>,
    annex_b_functions: ArenaVec<'a, AnnexBFunction>,

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
        Self {
            allocator,
            symbols: ArenaIndexVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            symbol_links: SymbolLinks::new_in(allocator),
            references: ArenaIndexVec::with_capacity_in(16usize.max(nodes / 4), allocator),
            declarations: ArenaIndexVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            redeclarations: Redeclarations::new_in(allocator),
            node_semantic,
            scope_maps: ArenaIndexVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            hoisting_variables: ArenaIndexVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            annex_b_functions: ArenaVec::new_in(allocator),
            pending: PendingBinding::default(),
            export_state: ExportState::None,
            // A declaration file is ambient in its entirety.
            ambient: tree.lang() == swc_next_ecma_ast::Lang::Dts,
            saved_stack: Vec::with_capacity(32),
        }
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
        self.pending = {
            let mut binding = match declaration.kind(tree) {
                VariableKind::Var => {
                    let target = scope.get(current_scope).hoist_target;
                    let mut excludes = Excludes::FUNCTION_SCOPED_VAR;
                    // JS permits var/function merges in hoist scopes, except modules.
                    if !tree.is_ts() && scope.get(target).kind != ScopeKind::Module {
                        excludes.remove(SymbolFlags::FUNCTION);
                    }
                    PendingBinding {
                        flags: SymbolFlags::FUNCTION_SCOPED_VAR,
                        excludes,
                        scope: target,
                    }
                }
                VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing => {
                    PendingBinding {
                        flags: SymbolFlags::BLOCK_SCOPED_VAR | SymbolFlags::CONST_VAR,
                        excludes: Excludes::BLOCK_SCOPED_VAR,
                        scope: current_scope,
                    }
                }
                VariableKind::Let => PendingBinding {
                    flags: SymbolFlags::BLOCK_SCOPED_VAR,
                    excludes: Excludes::BLOCK_SCOPED_VAR,
                    scope: current_scope,
                },
            };
            binding.flags.set(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            );
            binding
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
        // TS signatures merge with implementations. Duplicate implementations
        // follow the same hoist-scope and Annex B rules as JavaScript.
        self.pending = PendingBinding {
            flags: (SymbolFlags::FUNCTION | SymbolFlags::FUNCTION_IMPLEMENTATION)
                .with(SymbolFlags::FUNCTION_EXPRESSION, !is_declaration)
                .with(SymbolFlags::ANNEX_B_IF_FUNCTION, branch_local)
                .with(SymbolFlags::NON_ANNEX_B_FUNCTION, !ordinary)
                .with(SymbolFlags::AMBIENT, self.ambient),
            excludes: {
                let hoist_scope = target_kind.is_hoist_target() && target_kind != ScopeKind::Module;
                let unique_implementation = !hoist_scope && !annex_b_block_function;
                let mut excludes = Excludes::FUNCTION;
                excludes.set(SymbolFlags::CLASS, !tree.is_ts());
                excludes.set(
                    SymbolFlags::FUNCTION_SCOPED_VAR,
                    tree.is_ts() || !hoist_scope,
                );
                excludes.set(SymbolFlags::FUNCTION_IMPLEMENTATION, unique_implementation);
                excludes.set(SymbolFlags::NON_ANNEX_B_FUNCTION, annex_b_block_function);
                excludes
            },
            scope: target,
        };
        // Expression names are local.
        if !is_declaration {
            self.export_state = ExportState::None;
        }
    }

    pub fn enter_ts_function(&mut self, scope: &ScopeTree<'a>, current_scope: ScopeId) {
        self.push_saved_context();
        let target = scope
            .get(current_scope)
            .parent
            .expect("a TypeScript function scope has a parent");
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION | SymbolFlags::AMBIENT,
            excludes: Excludes::FUNCTION,
            scope: target,
        };
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
            excludes: Excludes::CLASS,
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

    pub fn enter_ts_module_block(&mut self) {
        self.export_state = ExportState::None;
    }

    pub fn enter_ts_enum_body(&mut self) {
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

    pub fn enter_ts_import_equals_declaration(
        &mut self,
        import_kind: ImportOrExportKind,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        self.pending = import_pending(import_kind, current_scope);
        self.pending.flags |= SymbolFlags::IMPORT_EQUALS;
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

    pub fn enter_ts_interface_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsInterfaceDeclaration,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        // The id binds in the surrounding scope. Type-parameter and
        // body bindings live in a scope pushed by the builder.
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::INTERFACE.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: Excludes::INTERFACE,
            scope: scope
                .get(current_scope)
                .parent
                .expect("an interface scope has a parent"),
        };
    }

    pub fn enter_ts_type_alias_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsTypeAliasDeclaration,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_ALIAS.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: Excludes::TYPE_ALIAS,
            scope: scope
                .get(current_scope)
                .parent
                .expect("a type alias scope has a parent"),
        };
    }

    pub fn enter_ts_enum_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsEnumDeclaration,
        current_scope: ScopeId,
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
            scope: current_scope,
        };
    }

    pub fn enter_ts_module_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsModuleDeclaration,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        let instantiated = namespace_is_instantiated(tree, declaration.body(tree));
        self.pending = PendingBinding {
            flags: SymbolFlags::NAMESPACE_MODULE
                .with(SymbolFlags::VALUE_MODULE, instantiated)
                .with(
                    SymbolFlags::AMBIENT,
                    declaration.declare(tree) || self.ambient,
                ),
            excludes: if instantiated {
                Excludes::VALUE_MODULE
            } else {
                Excludes::NAMESPACE_MODULE
            },
            scope: current_scope,
        };
        if declaration.declare(tree) {
            self.ambient = true;
        }
    }

    pub fn enter_ts_global_declaration(&mut self) {
        self.push_saved_context();
        self.ambient = true;
    }

    pub fn enter_ts_namespace_export_declaration(&mut self, current_scope: ScopeId) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::NAMESPACE_MODULE,
            excludes: Excludes::NAMESPACE_MODULE,
            scope: current_scope,
        };
    }

    pub fn enter_ts_type_parameter(
        &mut self,
        tree: &Ast<'a>,
        parent: Option<NodeId>,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            nearest_conditional_scope(tree, scope, current_scope)
        } else {
            current_scope
        };
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_PARAMETER,
            excludes: Excludes::TYPE_PARAMETER,
            scope: target,
        };
        self.export_state = ExportState::None;
    }

    pub fn enter_ts_mapped_type(
        &mut self,
        tree: &Ast<'a>,
        parent: Option<NodeId>,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        // Mapped type keys `[K in T]` are bare binding identifiers, so the
        // same pending context as a type parameter applies.
        self.push_saved_context();
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            nearest_conditional_scope(tree, scope, current_scope)
        } else {
            current_scope
        };
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_PARAMETER,
            excludes: Excludes::TYPE_PARAMETER,
            scope: target,
        };
        self.export_state = ExportState::None;
    }

    pub fn enter_ts_function_type(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_constructor_type(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_method_signature(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_call_signature_declaration(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_construct_signature_declaration(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_index_signature(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    fn enter_signature_type(&mut self, current_scope: ScopeId) {
        // Signature parameter names are labels rather than declarations.
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::NONE,
            excludes: SymbolFlags::NONE,
            scope: current_scope,
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

    pub fn declare_binding_identifier(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.sync_scope_maps_from(scope);
        // Type-only signature parameter labels do not declare bindings.
        if self.pending.flags.is_empty() {
            return;
        }
        let name = identifier.name(tree);
        let declaration_id = self.declare(tree, name.into(), identifier.node_id());
        let declaration = self.declarations[declaration_id];
        if !declaration.flags.is_hoisting_var() || current_scope == declaration.scope {
            return;
        }

        // Register a hoisting var in every block it passes through so
        // block-scoped redeclarations see it.
        let name = tree.get_utf8(name).as_bytes().clone_in(self.allocator);
        for scope_id in scope.ancestors(Some(current_scope)) {
            if scope_id == declaration.scope {
                break;
            }
            self.hoisting_variables[scope_id]
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

    pub fn declare_ts_enum_member(
        &mut self,
        tree: &Ast<'a>,
        member: TsEnumMember,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.sync_scope_maps_from(scope);
        if member.computed(tree) {
            return;
        }
        let name = match tree.kind_data(member.id(tree).node_id()) {
            NodeKindData::IdentifierName(identifier) => identifier.name(tree).into(),
            NodeKindData::StringLiteral(literal) => literal.value(tree),
            _ => return,
        };
        let saved = self.pending;
        self.pending = PendingBinding {
            flags: SymbolFlags::ENUM_MEMBER.with(SymbolFlags::AMBIENT, self.ambient),
            excludes: Excludes::ENUM_MEMBER,
            scope: current_scope,
        };
        self.declare(tree, name, member.node_id());
        self.pending = saved;
    }

    fn insert_scope_binding(
        &mut self,
        tree: &Ast<'a>,
        scope: ScopeId,
        name: AstWtf8,
        id: SymbolId,
    ) {
        let name = tree.get_wtf8(name).as_bytes().clone_in(self.allocator);
        self.scope_maps[scope].insert(name, id);
    }

    /// Declares the pending binding for `name` at `node`. A compatible
    /// existing binding is merged. On conflict, its flags remain unchanged
    /// because the caller reports the diagnostic. Every node is still stored
    /// as a declaration for error recovery.
    pub fn declare(&mut self, tree: &Ast<'a>, name: AstWtf8, node: NodeId) -> DeclarationId {
        let target = self.pending.scope;
        let existing = {
            let text = tree.get_wtf8(name).as_bytes();
            self.own_binding(target, text).filter(|&id| {
                let flags = self.symbols[id].flags;
                !flags.intersects(SymbolFlags::INTERNAL_NAME)
            })
        };
        let declaration = {
            let mut flags = self.pending.flags;
            flags.set(SymbolFlags::EXPORTED, self.export_state.is_exported());
            flags.set(SymbolFlags::IS_DEFAULT, self.export_state.is_default());
            PendingDeclaration {
                name,
                node,
                binding: PendingBinding {
                    flags,
                    ..self.pending
                },
            }
        };
        let (symbol, declaration) = self.declare_symbol(tree, declaration, existing, true);
        if existing.is_none() {
            self.insert_scope_binding(tree, target, name, symbol);
        }
        declaration
    }

    pub(crate) fn link_parameter_vars(&mut self, scopes: &ScopeTree<'a>, body: ScopeId) {
        self.sync_scope_maps_from(scopes);
        let parameters = scopes.get(body).parent.expect("function body has a parent");
        // Inspect the completed body so a function implementation suppresses
        // parameter-to-var initialization independently of declaration order.
        for (&name, &parameter) in &self.scope_maps[parameters] {
            let parameter_flags = self.symbols[parameter].flags;
            if !parameter_flags.contains(SymbolFlags::PARAMETER) {
                continue;
            }
            let Some(var) = self.own_binding(body, name) else {
                continue;
            };
            let flags = self.symbols[var].flags;
            if flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR)
                && !flags.contains(SymbolFlags::FUNCTION_IMPLEMENTATION)
            {
                self.symbol_links.link(parameter, var);
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
        let declaration_id = self.declarations.next_idx();
        let id = if let Some(existing) = existing {
            let symbol = &mut self.symbols[existing];
            if !symbol.flags.intersects(binding.excludes) {
                symbol.flags = merge_symbol_flags(symbol.flags, declaration_flags);
            }
            let declarations = self
                .redeclarations
                .entry(existing)
                .or_insert_with(|| ArenaVec::with_capacity_in(1, self.allocator));
            declarations.push(declaration_id);
            existing
        } else {
            let new_id = self.symbols.next_idx();
            let name = {
                let text = tree.get_wtf8(name).as_bytes();
                match self.own_binding(binding.scope, text) {
                    Some(id)
                        if self.symbols[id]
                            .flags
                            .intersects(SymbolFlags::INTERNAL_NAME) =>
                    {
                        self.symbols[id].name
                    }
                    _ => name,
                }
            };
            self.symbols.push(Symbol {
                name,
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
        if record_node {
            debug_assert!(matches!(
                self.node_semantic[node],
                NodeSemanticId::None | NodeSemanticId::Symbol(_)
            ));
            self.node_semantic[node] = NodeSemanticId::Symbol(id);
        }
        (id, declaration_id)
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
            self.hoisting_variables
                .push(HoistingMap::new_in(self.allocator));
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
        current_scope: ScopeId,
        flags: ReferenceFlags,
    ) {
        let Some(local) = specifier.local(tree).as_identifier_name(tree) else {
            return;
        };
        self.add_reference(
            tree,
            local.name(tree),
            current_scope,
            local.node_id(),
            flags,
        );
    }

    pub fn add_ts_type_predicate_reference(
        &mut self,
        tree: &Ast<'a>,
        predicate: TsTypePredicate,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.sync_scope_maps_from(scope);
        // `param is T` parses `param` as an IdentifierName but it
        // references the enclosing function's parameter. Type-only signature
        // parameters are labels, so they do not create a reference here.
        let parameter_name = predicate.parameter_name(tree);
        let parameter_node = parameter_name.node_id();
        let NodeKindData::IdentifierName(parameter_name) = tree.kind_data(parameter_node) else {
            return;
        };
        let name = parameter_name.name(tree);
        let Some(parameter) = self.own_binding(current_scope, tree.get_utf8(name).as_bytes())
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
            current_scope,
            parameter_node,
            ReferenceFlags::default()
                .with_space(ReferenceSpace::Typeof)
                .no_access(),
        );
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
    pub fn in_ambient_context(&self) -> bool {
        self.ambient || self.pending.flags.contains(SymbolFlags::AMBIENT)
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id]
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
        let map = self.scope_maps.get(scope)?;
        map.get(name).copied()
    }

    /// The binding of `name` at `scope`, including a hoisting `var` passing
    /// through on its way to its hoist target.
    pub fn binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        if let Some(symbol) = self.own_binding(scope, name) {
            return Some(symbol);
        }
        let declaration = self.hoisting_declaration(scope, name)?;
        Some(self.declarations[declaration].symbol)
    }

    fn hoisting_declaration(&self, scope: ScopeId, name: &[u8]) -> Option<DeclarationId> {
        let map = self.hoisting_variables.get(scope)?;
        map.get(name).copied()
    }

    pub(crate) fn hoisting_conflict(
        &self,
        binding: PendingBinding,
        name: &[u8],
    ) -> Option<Declaration> {
        // Passing vars are conflict-only declarations, never binding candidates.
        // Only declarations that exclude var need the scope's hoisting map.
        if !binding.excludes.contains(SymbolFlags::FUNCTION_SCOPED_VAR) {
            return None;
        }
        let declaration = self.hoisting_declaration(binding.scope, name)?;
        Some(self.declarations[declaration])
    }

    /// Finalizes the tracker into a complete [`Semantic`]. Builds the
    /// declaration, use, and node indexes and resolves every reference
    /// to its declaring symbol. The result aliases tracker storage and
    /// remains valid for the source tree's arena lifetime.
    pub fn finalize(mut self, tree: &Ast<'a>, scopes: ScopeTree<'a>) -> Semantic<'a> {
        debug_assert!(self.saved_stack.is_empty());
        debug_assert_eq!(self.export_state, ExportState::None);
        self.sync_scope_maps(scopes.list.len());
        let mut use_ranges = ArenaIndexVec::with_capacity_in(self.symbols.len(), self.allocator);
        use_ranges.resize(self.symbols.len(), Range::default());
        for reference in &mut self.references {
            let name = tree.get_utf8(reference.name).as_bytes();
            reference.symbol = lookup_lexical_name(
                &scopes.list,
                &self.symbols,
                &self.scope_maps,
                &self.declarations,
                reference.scope,
                name,
                reference.flags.space,
            );
            if let Some(symbol) = reference.symbol {
                use_ranges[symbol].len += 1;
                let flags = self.symbols[symbol].flags;
                if reference.flags.space == ReferenceSpace::Any
                    && !flags.contains(SymbolFlags::IMPORT)
                    && !flags.in_value_space()
                {
                    reference.flags = reference.flags.no_access();
                }
            }
        }
        let use_ids = {
            let count = {
                let mut offset = 0;
                for range in &mut use_ranges {
                    range.start = offset;
                    offset += range.len;
                    range.len = 0;
                }
                offset as usize
            };
            let mut ids = ArenaVec::with_capacity_in(count, self.allocator);
            ids.resize(count, ReferenceId::from_usize(0));
            for (id, reference) in self.references.iter_enumerated() {
                self.node_semantic[reference.node] = NodeSemanticId::Reference(id);
                if let Some(symbol) = reference.symbol {
                    let range = &mut use_ranges[symbol];
                    ids[(range.start + range.len) as usize] = id;
                    range.len += 1;
                }
            }
            ids
        };
        Semantic {
            scopes,
            symbols: self.symbols,
            symbol_links: self.symbol_links,
            references: self.references,
            declarations: self.declarations,
            redeclarations: self.redeclarations,
            use_ids,
            use_ranges,
            scope_maps: self.scope_maps,
            hoisting_variables: self.hoisting_variables,
            node_semantic: self.node_semantic,
        }
    }
}

pub(super) fn lookup_lexical_name(
    scopes: &Scopes,
    symbols: &Symbols,
    scope_maps: &ScopeMaps<'_>,
    declarations: &Declarations,
    scope: ScopeId,
    name: &[u8],
    space: ReferenceSpace,
) -> Option<SymbolId> {
    let hash = scope_maps[ScopeId::ROOT].hasher().hash_one(name);
    let arguments_barrier =
        name == b"arguments" && matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof);
    let mut ancestor = Some(scope);
    while let Some(current) = ancestor {
        let barrier = arguments_barrier && scopes[current].flags.arguments_barrier;
        let visible = |symbol: SymbolId| {
            let flags = symbols[symbol].flags;
            flags.visible_in(space)
                && (!barrier
                    || !flags.intersects(
                        SymbolFlags::FUNCTION_EXPRESSION | SymbolFlags::ANNEX_B_IF_FUNCTION,
                    ))
        };
        let bindings = &scope_maps[current];
        let entry = bindings
            .raw_entry()
            .from_hash(hash, |candidate| *candidate == name);
        if let Some((_, &symbol)) = entry {
            if visible(symbol) {
                return Some(symbol);
            }
            if let Some(internal) = find_internal_name(symbols, declarations, current, symbol)
                && visible(internal)
            {
                return Some(internal);
            }
        }
        if barrier {
            break;
        }
        ancestor = scopes[current].parent;
    }
    None
}

fn find_internal_name(
    symbols: &Symbols,
    declarations: &Declarations,
    scope: ScopeId,
    binding: SymbolId,
) -> Option<SymbolId> {
    let name = symbols[binding].name;
    for (id, symbol) in symbols.iter_enumerated() {
        if id != binding
            && symbol.flags.intersects(SymbolFlags::INTERNAL_NAME)
            && symbol.name == name
            && declarations[symbol.declaration].scope == scope
        {
            return Some(id);
        }
    }
    None
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

fn namespace_is_instantiated(
    tree: &Ast<'_>,
    body: Option<swc_next_ecma_ast::TsModuleBody>,
) -> bool {
    let Some(body) = body else {
        return true;
    };
    let body = body.node_id();
    let NodeKindData::TsModuleBlock(block) = tree.kind_data(body) else {
        return true;
    };
    if !block.directives(tree).is_empty() {
        return true;
    }
    for statement in block.body(tree).iter() {
        let statement = tree.get_node_in_sub_range(statement).node_id();
        if is_instantiating_statement(tree, statement) {
            return true;
        }
    }
    false
}

fn is_instantiating_statement(tree: &Ast<'_>, node: NodeId) -> bool {
    match tree.kind_data(node) {
        NodeKindData::TsInterfaceDeclaration(_)
        | NodeKindData::TsTypeAliasDeclaration(_)
        | NodeKindData::TsImportEqualsDeclaration(_) => false,
        NodeKindData::TsEnumDeclaration(declaration) => !declaration.is_const(tree),
        NodeKindData::TsModuleDeclaration(module) => {
            namespace_is_instantiated(tree, module.body(tree))
        }
        NodeKindData::ExportNamedDeclaration(export) => {
            if export.export_kind(tree) == ImportOrExportKind::Type {
                return false;
            }
            if let Some(declaration) = export.declaration(tree) {
                return is_instantiating_statement(tree, declaration.node_id());
            }
            export.source(tree).is_some()
        }
        _ => true,
    }
}

fn nearest_conditional_scope(
    tree: &Ast<'_>,
    scope: &ScopeTree<'_>,
    current_scope: ScopeId,
) -> ScopeId {
    scope
        .ancestors(Some(current_scope))
        .find(|&id| tree.node_kind(scope.get(id).node) == NodeKind::TsConditionalType)
        .unwrap_or(current_scope)
}
fn merge_symbol_flags(existing: SymbolFlags, incoming: SymbolFlags) -> SymbolFlags {
    let mut merged = existing.merge(incoming);
    merged.set(
        SymbolFlags::AMBIENT,
        existing.contains(SymbolFlags::AMBIENT) && incoming.contains(SymbolFlags::AMBIENT),
    );
    merged
}
