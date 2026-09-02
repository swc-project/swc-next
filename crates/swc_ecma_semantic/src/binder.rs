use oxc_index::Idx;
use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    Ast, AstInternedUtf8, AstUtf8, AstWtf8, BindingIdentifier, Class, ClassType,
    ExportNamedDeclaration, ExportSpecifier, Function, FunctionType, IdentifierReference,
    ImportDeclaration, ImportOrExportKind, ImportSpecifier, JsxClosingElement, JsxElementName,
    JsxElementNameData, JsxIdentifier, JsxMemberExpressionObjectData, JsxOpeningElement, NodeId,
    NodeKind, NodeKindData, ReferenceId, ScopeId, SourceType, SymbolId, TsEnumDeclaration,
    TsEnumMember, TsInterfaceDeclaration, TsModuleDeclaration, TsTypeAliasDeclaration,
    TsTypePredicate, VariableDeclaration, VariableKind,
};

use crate::{
    DeclarationId, Redeclarations, ScopeMap, ScopeName, Semantic, Wtf8ScopeMap,
    module_record::ModuleRecordBuilder,
    scope::{self, ScopeKind, ScopeTracker, ScopeTree},
};

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
    pub name: crate::SymbolName,
    pub flags: SymbolFlags,
    /// The scope the symbol is declared in. For a hoisting `var`,
    /// this is the hoist target it lands in, not the block it is
    /// written in.
    pub scope: ScopeId,
    /// The first declaration contributing to this symbol.
    pub declaration: DeclarationId,
}

/// One declaration contributing to a [`Symbol`].
///
/// Unlike the aggregate flags on a symbol, `flags` describes only this
/// declaration. This preserves the distinct type, value, and namespace
/// meanings of declarations that merge into one symbol.
#[derive(Debug, Clone, Copy)]
pub struct Declaration {
    pub node: NodeId,
    pub symbol: SymbolId,
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
        /// An ordinary (non-async, non-generator) function declaration.
        /// Used internally to identify Annex B.3.3.4's duplicate-function
        /// exception.
        const ANNEX_B_FUNCTION = 1 << 20;

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

        const BLOCK_SCOPED_LIKE = Self::BLOCK_SCOPED_VAR.bits()
            | Self::CLASS.bits()
            | Self::FUNCTION.bits();
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

    /// True when a symbol with these flags is visible in `space`, the
    /// acceptance rule of name resolution.
    ///
    /// ## Example
    /// ```ts
    /// type T = string;
    /// function f() {
    ///   const T = 1;
    ///   let x: T;
    ///   //     ^ the const is not visible in type space, so
    ///   //       resolution walks on to the outer alias
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
    /// through: block-scoped bindings (`let`, `const`), classes, and
    /// functions.
    pub fn is_block_scoped_like(self) -> bool {
        self.intersects(Self::BLOCK_SCOPED_LIKE)
    }

    /// Human-readable category for diagnostics.
    pub fn to_string(self) -> &'static str {
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
    pub const FUNCTION_SCOPED_VAR: SymbolFlags = SymbolFlags::VALUE_SPACE
        .difference(SymbolFlags::FUNCTION_SCOPED_VAR.union(SymbolFlags::FUNCTION));
    /// Function in a hoist scope (function/global/static block). TS
    /// allows function overloads and sloppy JS permits merging with
    /// `var`. Lexical scopes use the block-scoped excludes instead.
    pub const FUNCTION: SymbolFlags = SymbolFlags::VALUE_SPACE.difference(
        SymbolFlags::FUNCTION_SCOPED_VAR
            .union(SymbolFlags::FUNCTION)
            .union(SymbolFlags::VALUE_MODULE)
            .union(SymbolFlags::CLASS),
    );
    /// Annex B.3.3.4 permits duplicate ordinary function declarations in a
    /// sloppy block. The marker check in `allows_annex_b_function_duplicate`
    /// narrows this otherwise-all-value-space exclusion to that exact case.
    pub const ANNEX_B_BLOCK_FUNCTION: SymbolFlags = SymbolFlags::from_bits_retain(
        SymbolFlags::VALUE_SPACE.bits() | SymbolFlags::ANNEX_B_FUNCTION.bits(),
    );
    pub const CLASS: SymbolFlags = SymbolFlags::from_bits_retain(
        SymbolFlags::FUNCTION_SCOPED_VAR.bits()
            | SymbolFlags::BLOCK_SCOPED_VAR.bits()
            | SymbolFlags::FUNCTION.bits()
            | SymbolFlags::CLASS.bits()
            | SymbolFlags::REGULAR_ENUM.bits()
            | SymbolFlags::CONST_ENUM.bits()
            | SymbolFlags::TYPE_ALIAS.bits()
            | SymbolFlags::TYPE_PARAMETER.bits()
            | SymbolFlags::ENUM_MEMBER.bits(),
    );
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
/// in the source, for a `var` binding whose runtime write crosses a `with`
/// object environment, for the local name in an `export { name }` specifier,
/// for JSX component tag names (`<Foo>`), and for TS type-predicate parameters
/// (`x is T`). Re-export names in `export { name } from "module"` are external
/// names, not references. A dynamic `var` write is both a declaration and a
/// reference at the same node.
///
/// ## Example
/// ```ts
/// let a = b; a = 2;
/// //      ^ a read of `b`
/// //         ^ a write of `a`, flags.is_write()
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Reference {
    pub name: AstInternedUtf8,
    /// The scope the reference appears in.
    pub scope: ScopeId,
    /// The referencing node.
    pub node: NodeId,
    /// The lexical fallback symbol this reference resolves to. `None` for
    /// names with no binding visible in the reference's space.
    ///
    /// When [`ReferenceFlags::is_dynamic`] is true, a `with` object may
    /// intercept the reference at runtime before this fallback is reached.
    pub symbol: Option<SymbolId>,
    pub flags: ReferenceFlags,
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct ReferenceProperties: u8 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const DYNAMIC = 1 << 2;
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

    /// True when a `with` object environment can intercept this value
    /// reference before its lexical fallback binding is reached.
    ///
    /// [`Reference::symbol`] remains the lexical fallback when one exists;
    /// a dynamic reference with no fallback is not definitely global.
    pub fn is_dynamic(self) -> bool {
        self.properties.contains(ReferenceProperties::DYNAMIC)
    }

    pub(crate) fn with_space(mut self, space: ReferenceSpace) -> Self {
        self.space = space;
        if space.in_type_position() {
            self.properties = ReferenceProperties::empty();
        }
        self
    }

    pub(crate) fn write_only(mut self) -> Self {
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

    fn with_dynamic(mut self) -> Self {
        self.properties.insert(ReferenceProperties::DYNAMIC);
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
    /// the qualifier of a dotted type name, `ns` in `ns.T`
    Namespace,
    /// a value use inside a type, the entity of a `typeof` query or a
    /// type-predicate parameter
    Typeof,
    /// an alias position that accepts every space: `export { x }`, a direct
    /// identifier in `export default` or `export =`, and `import a = x`
    Any,
}

impl ReferenceSpace {
    /// True for positions inside a type-only subtree.
    pub fn in_type_position(self) -> bool {
        matches!(self, Self::Type | Self::Namespace | Self::Typeof)
    }
}

/// Collects symbols and references during the AST walk and finalizes
/// them into a [`Semantic`].
pub(crate) struct SymbolTracker<'a> {
    allocator: &'a Allocator,
    symbols: ArenaVec<'a, Symbol>,
    references: ArenaVec<'a, Reference>,
    declarations: ArenaVec<'a, Declaration>,
    redeclarations: Redeclarations<'a>,
    node_symbols: ArenaVec<'a, Option<SymbolId>>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, ScopeMap<'a>>,
    wtf8_bindings: Wtf8ScopeMap<'a>,

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

impl PendingBinding {
    pub(crate) fn allows_annex_b_function_duplicate(self, existing: SymbolFlags) -> bool {
        self.excludes == Excludes::ANNEX_B_BLOCK_FUNCTION
            && self.flags.contains(SymbolFlags::ANNEX_B_FUNCTION)
            && existing.contains(SymbolFlags::ANNEX_B_FUNCTION)
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
        let mut node_symbols = ArenaVec::with_capacity_in(nodes, allocator);
        node_symbols.resize(nodes, None);
        Self {
            allocator,
            symbols: ArenaVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            references: ArenaVec::with_capacity_in(16usize.max(nodes / 4), allocator),
            declarations: ArenaVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            redeclarations: Redeclarations::new_in(allocator),
            node_symbols,
            scope_maps: ArenaVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            hoisting_variables: ArenaVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            wtf8_bindings: Wtf8ScopeMap::new_in(allocator),
            pending: PendingBinding::default(),
            export_state: ExportState::None,
            // A declaration file is ambient in its entirety.
            ambient: tree.lang() == swc_next_ecma_ast::Lang::Dts,
            saved_stack: Vec::with_capacity(32),
        }
    }

    #[inline]
    pub fn in_ambient_context(&self) -> bool {
        self.ambient || self.pending.flags.contains(SymbolFlags::AMBIENT)
    }

    pub fn enter_export_named_declaration(
        &mut self,
        tree: &Ast,
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

    pub fn enter_variable_declaration(
        &mut self,
        tree: &Ast,
        declaration: VariableDeclaration,
        scope: &ScopeTracker<'a>,
    ) -> bool {
        self.push_saved_context();
        let kind = declaration.kind(tree);
        self.pending = match kind {
            VariableKind::Var => {
                let target = scope.hoist_target();
                let mut excludes = Excludes::FUNCTION_SCOPED_VAR;
                // Module-level function declarations are lexical;
                // everywhere else (and in TS) `var` merges with them.
                if !tree.is_ts() && scope.get(target).kind == ScopeKind::Module {
                    excludes.insert(SymbolFlags::FUNCTION);
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
        kind == VariableKind::Var && scope.var_write_crosses_with()
    }

    pub fn enter_function(&mut self, tree: &Ast, function: Function, scope: &ScopeTracker<'a>) {
        self.push_saved_context();
        let is_declaration = function.function_type(tree) == FunctionType::FunctionDeclaration;
        let target = if is_declaration {
            scope
                .current_scope()
                .parent
                .expect("a function declaration scope has a parent")
        } else {
            expression_name_scope(scope)
        };

        let target_scope = scope.get(target);
        let target_kind = target_scope.kind;
        // TS permits var/function merges in ordinary blocks, but not in
        // catch scopes.
        let ts_catch_scope =
            tree.is_ts() && tree.node_kind(target_scope.node) == NodeKind::CatchClause;
        let annex_b_block_function = !tree.is_ts()
            && is_declaration
            && target_kind == ScopeKind::Block
            && !target_scope.flags.strict
            && !function.generator(tree)
            && !function.r#async(tree);
        let allow_overload = (tree.is_ts() && !ts_catch_scope)
            || matches!(
                target_kind,
                ScopeKind::Function
                    | ScopeKind::FunctionBody
                    | ScopeKind::Global
                    | ScopeKind::StaticBlock
            );
        // TS overloads, sloppy JS Annex B.3.2, and global merge with
        // var. Strict lexical scopes (block/module) do not.
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION
                .with(
                    SymbolFlags::ANNEX_B_FUNCTION,
                    is_declaration && !function.generator(tree) && !function.r#async(tree),
                )
                .with(SymbolFlags::AMBIENT, self.ambient),
            excludes: if annex_b_block_function {
                Excludes::ANNEX_B_BLOCK_FUNCTION
            } else if allow_overload {
                if tree.is_ts() {
                    Excludes::FUNCTION
                } else {
                    Excludes::FUNCTION.union(SymbolFlags::CLASS)
                }
            } else {
                Excludes::BLOCK_SCOPED_VAR
            },
            scope: target,
        };
        // Expression names are local.
        if !is_declaration {
            self.export_state = ExportState::None;
        }
    }

    pub fn enter_ts_function(&mut self, scope: &ScopeTracker<'a>) {
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
    }

    pub fn enter_class(&mut self, tree: &Ast, class: Class, scope: &ScopeTracker<'a>) {
        self.push_saved_context();
        let is_declaration = class.class_type(tree) == ClassType::ClassDeclaration;
        self.pending = PendingBinding {
            flags: SymbolFlags::CLASS
                .with(SymbolFlags::AMBIENT, class.declare(tree) || self.ambient),
            excludes: Excludes::CLASS,
            scope: if is_declaration {
                scope
                    .current_scope()
                    .parent
                    .expect("a class declaration scope has a parent")
            } else {
                expression_name_scope(scope)
            },
        };
        // Expression names are local.
        if !is_declaration {
            self.export_state = ExportState::None;
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
        tree: &Ast,
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
        tree: &Ast,
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
        tree: &Ast,
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
        tree: &Ast,
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
        tree: &Ast,
        declaration: TsModuleDeclaration,
        scope: &ScopeTracker<'a>,
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
            scope: scope.current,
        };
        // `declare` makes the whole body ambient.
        if declaration.declare(tree) {
            self.ambient = true;
        }
    }

    pub fn enter_ts_global_declaration(&mut self) {
        self.push_saved_context();
        self.ambient = true;
    }

    pub fn enter_ts_namespace_export_declaration(&mut self, scope: &ScopeTracker<'a>) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::NAMESPACE_MODULE,
            excludes: Excludes::NAMESPACE_MODULE,
            scope: scope.current,
        };
    }

    pub fn enter_ts_type_parameter(
        &mut self,
        tree: &Ast,
        parent: Option<NodeId>,
        scope: &ScopeTracker<'a>,
    ) {
        // `TsTypeParameter` wraps `<T>` and `infer U`.
        self.push_saved_context();
        // `infer` declares in the enclosing conditional scope so the
        // true branch sees it, even from inside a signature type.
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            nearest_conditional_scope(tree, scope)
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

    pub fn enter_ts_mapped_type(
        &mut self,
        tree: &Ast,
        parent: Option<NodeId>,
        scope: &ScopeTracker<'a>,
    ) {
        // Mapped type keys `[K in T]` are bare binding identifiers, so the
        // same pending context as a type parameter applies.
        self.push_saved_context();
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            nearest_conditional_scope(tree, scope)
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
        // Signature-type parameter names are labels, not declarations.
        // Clear an enclosing type-parameter context so nesting never
        // declares them, as in `<T extends { [s: string]: number }> `.
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::NONE,
            excludes: SymbolFlags::NONE,
            scope: scope.current,
        };
    }

    /// Restores the context to what it was before the matching enter.
    pub fn exit_formal_parameters(&mut self) {
        if let Some(saved) = self.saved_stack.pop() {
            self.pending = saved.pending;
            self.ambient = saved.ambient;
        }
    }

    pub fn exit_saved_context(&mut self) {
        if let Some(saved) = self.saved_stack.pop() {
            self.pending = saved.pending;
            self.export_state = saved.export_state;
            self.ambient = saved.ambient;
        }
    }

    pub fn exit_export_declaration(&mut self) {
        self.export_state = ExportState::None;
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
        tree: &mut Ast,
        identifier: BindingIdentifier,
        scope: &ScopeTracker<'a>,
        space: ReferenceSpace,
    ) -> Option<SymbolId> {
        self.sync_scope_maps_from(scope);
        let node = identifier.node_id();
        // Type-position identifiers are parameter labels. Only type
        // parameters are real declarations there.
        if space.in_type_position() && !self.pending.flags.contains(SymbolFlags::TYPE_PARAMETER) {
            return None;
        }
        let name = self.canonicalize_name(tree, identifier.name(tree));
        let symbol_id = self.declare(tree, name, node);

        // Register a hoisting var in every block it passes through so
        // block-scoped redeclarations see it.
        if self.pending.flags.is_hoisting_var() && scope.current != self.pending.scope {
            for scope_id in scope.ancestors(Some(scope.current)) {
                if scope_id == self.pending.scope {
                    break;
                }
                self.hoisting_variables[scope_id.index()]
                    .entry(name)
                    .or_insert(symbol_id);
            }
        }
        Some(symbol_id)
    }

    pub fn add_identifier_reference(
        &mut self,
        tree: &mut Ast,
        identifier: IdentifierReference,
        scope: &ScopeTracker<'a>,
        flags: ReferenceFlags,
    ) {
        self.sync_scope_maps_from(scope);
        self.add_reference(
            tree,
            identifier.name(tree),
            scope.current,
            identifier.node_id(),
            flags,
        );
    }

    pub fn add_var_write_reference(
        &mut self,
        tree: &mut Ast,
        identifier: BindingIdentifier,
        scope: &ScopeTracker<'a>,
        flags: ReferenceFlags,
    ) {
        debug_assert!(flags.is_write_only());
        debug_assert!(scope.var_write_crosses_with());
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
        tree: &mut Ast,
        specifier: ExportSpecifier,
        scope: &ScopeTracker<'a>,
        flags: ReferenceFlags,
    ) {
        let Some(local) = specifier.local(tree).as_identifier_name(tree) else {
            return;
        };
        self.sync_scope_maps_from(scope);
        self.add_reference(
            tree,
            local.name(tree),
            scope.current,
            local.node_id(),
            flags,
        );
    }

    pub fn add_ts_type_predicate_reference(
        &mut self,
        tree: &mut Ast,
        predicate: TsTypePredicate,
        scope: &ScopeTracker<'a>,
    ) {
        self.sync_scope_maps_from(scope);
        // `param is T` parses `param` as an IdentifierName but it
        // references the enclosing function's parameter. Synthesize
        // a reference so renamers update both together. In type-only
        // signatures the parameter is only a label, so no reference
        // is recorded.
        let parameter_name = predicate.parameter_name(tree);
        let parameter_node = parameter_name.node_id();
        let NodeKindData::IdentifierName(parameter_name) = tree.kind_data(parameter_node) else {
            return;
        };
        let name = parameter_name.name(tree);
        let canonical_name = self.canonicalize_name(tree, name);
        let Some(parameter) = self.own_binding(scope.current, canonical_name) else {
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
            ReferenceFlags::default().with_space(ReferenceSpace::Typeof),
        );
    }

    pub fn declare_ts_enum_member(
        &mut self,
        tree: &mut Ast,
        member: TsEnumMember,
        scope: &ScopeTracker<'a>,
    ) {
        self.sync_scope_maps_from(scope);
        // Members are lexically visible inside the enum body:
        //
        //   enum E { a, b = a }
        //   //              ^ resolves to the member
        //
        // Member names are IdentifierName or StringLiteral nodes,
        // not BindingIdentifiers, so they declare here. Computed and
        // template names never declare.
        if member.computed(tree) {
            return;
        }
        let member_name = member.id(tree).node_id();
        let name = match tree.kind_data(member_name) {
            NodeKindData::IdentifierName(identifier) => identifier.name(tree).into(),
            NodeKindData::StringLiteral(literal) => literal.value(tree),
            _ => return,
        };
        let saved = self.pending;
        self.pending = PendingBinding {
            flags: SymbolFlags::ENUM_MEMBER.with(SymbolFlags::AMBIENT, self.ambient),
            excludes: Excludes::ENUM_MEMBER,
            scope: scope.current,
        };
        self.declare_wtf8(tree, name, member.node_id());
        self.pending = saved;
    }

    pub fn add_jsx_opening_element_reference(
        &mut self,
        tree: &mut Ast,
        element: JsxOpeningElement,
        scope: &ScopeTracker<'a>,
    ) {
        self.sync_scope_maps_from(scope);
        // The leftmost capitalized JSX identifier is a JS binding
        // reference. Lowercase tags are intrinsic element names; the
        // rest of a member chain is property syntax.
        self.add_jsx_reference(tree, element.name(tree), scope.current);
    }

    pub fn add_jsx_closing_element_reference(
        &mut self,
        tree: &mut Ast,
        element: JsxClosingElement,
        scope: &ScopeTracker<'a>,
    ) {
        self.sync_scope_maps_from(scope);
        self.add_jsx_reference(tree, element.name(tree), scope.current);
    }

    fn add_jsx_reference(&mut self, tree: &mut Ast, name: JsxElementName, scope: ScopeId) {
        let Some(identifier) = jsx_tag_root(tree, tree.jsx_element_name_data(name)) else {
            return;
        };
        let text = tree.get_utf8(identifier.name(tree));
        if text.as_bytes().first().is_some_and(u8::is_ascii_uppercase) {
            self.add_reference(
                tree,
                identifier.name(tree),
                scope,
                identifier.node_id(),
                ReferenceFlags::default(),
            );
        }
    }

    /// Declares the pending binding for `name` at `node`. A compatible
    /// existing binding is merged. On conflict, its flags remain unchanged
    /// because the caller reports the diagnostic. Every node is still stored
    /// as a declaration for error recovery.
    pub fn declare(&mut self, tree: &Ast, name: AstInternedUtf8, node: NodeId) -> SymbolId {
        debug_assert!(node.index() < tree.node_count());
        self.declare_name(ScopeName::Utf8(name), node)
    }

    fn declare_wtf8(&mut self, tree: &mut Ast, name: AstWtf8, node: NodeId) -> SymbolId {
        debug_assert!(node.index() < tree.node_count());
        let name = ScopeName::intern_wtf8(tree, name);
        self.declare_name(name, node)
    }

    fn declare_name(&mut self, scope_name: ScopeName, node: NodeId) -> SymbolId {
        let target = self.pending.scope;

        let mut declaration_flags = self.pending.flags;
        declaration_flags.set(SymbolFlags::EXPORTED, self.export_state.is_exported());
        declaration_flags.set(SymbolFlags::IS_DEFAULT, self.export_state.is_default());

        let declaration_id = DeclarationId::from_usize(self.declarations.len());
        let id = if let Some(existing) = self.binding_scope_name(target, scope_name) {
            let symbol = &mut self.symbols[existing.index()];
            if !symbol.flags.intersects(self.pending.excludes)
                || self.pending.allows_annex_b_function_duplicate(symbol.flags)
            {
                let mut merged = symbol.flags.merge(declaration_flags);
                // An overload implementation emits at runtime, so the merged
                // symbol is non-ambient.
                merged.set(
                    SymbolFlags::AMBIENT,
                    symbol.flags.contains(SymbolFlags::AMBIENT)
                        && self.pending.flags.contains(SymbolFlags::AMBIENT),
                );
                symbol.flags = merged;
            }
            self.redeclarations
                .entry(existing)
                .or_insert_with(|| ArenaVec::with_capacity_in(1, self.allocator))
                .push(declaration_id);
            existing
        } else {
            let new_id = SymbolId::from_usize(self.symbols.len());
            self.symbols.push(Symbol {
                name: scope_name,
                flags: declaration_flags,
                scope: target,
                declaration: declaration_id,
            });
            match scope_name {
                ScopeName::Utf8(name) => {
                    self.scope_maps[target.index()].insert(name, new_id);
                }
                ScopeName::Wtf8(name) => {
                    self.wtf8_bindings.insert((target, name), new_id);
                }
            }
            new_id
        };

        self.declarations.push(Declaration {
            node,
            symbol: id,
            flags: declaration_flags,
        });
        self.node_symbols[node.index()] = Some(id);
        id
    }

    /// Records an identifier reference at `node` in `scope`.
    pub fn add_reference(
        &mut self,
        tree: &mut Ast,
        name: AstUtf8,
        scope: ScopeId,
        node: NodeId,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        debug_assert!(node.index() < tree.node_count());
        let name = self.canonicalize_name(tree, name);
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

    pub(crate) fn canonicalize_name(&mut self, tree: &mut Ast, name: AstUtf8) -> AstInternedUtf8 {
        tree.intern_utf8(name)
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id.index()]
    }

    /// The `binding_identifier` node of the earliest declaration of `id`.
    /// Later declarations merged into the symbol do not change it.
    pub fn first_decl_of(&self, id: SymbolId) -> NodeId {
        let declaration_id = self.symbols[id.index()].declaration;
        self.declarations[declaration_id.index()].node
    }

    /// The binding of `name` declared directly in `scope`, excluding
    /// hoisting `var`s passing through.
    pub fn own_binding(&self, scope: ScopeId, name: AstInternedUtf8) -> Option<SymbolId> {
        self.scope_maps
            .get(scope.index())
            .and_then(|map| map.get(&name).copied())
    }

    /// The binding of `name` at `scope`, including a hoisting `var` passing
    /// through on its way to its hoist target.
    pub fn binding(&self, scope: ScopeId, name: AstInternedUtf8) -> Option<SymbolId> {
        self.own_binding(scope, name).or_else(|| {
            self.hoisting_variables
                .get(scope.index())
                .and_then(|map| map.get(&name).copied())
        })
    }

    fn binding_scope_name(&self, scope: ScopeId, name: ScopeName) -> Option<SymbolId> {
        match name {
            ScopeName::Utf8(name) => self
                .scope_maps
                .get(scope.index())
                .and_then(|map| map.get(&name).copied())
                .or_else(|| {
                    self.hoisting_variables
                        .get(scope.index())
                        .and_then(|map| map.get(&name).copied())
                }),
            ScopeName::Wtf8(name) => self.wtf8_bindings.get(&(scope, name)).copied(),
        }
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
            self.hoisting_variables
                .push(ScopeMap::new_in(self.allocator));
        }
    }

    /// Finalizes the tracker into a complete [`Semantic`]. Builds the
    /// declaration, use, and node indexes and resolves every reference
    /// to its declaring symbol. The result aliases tracker storage and
    /// remains valid for the source tree's arena lifetime.
    pub fn finalize(
        mut self,
        tree: &Ast,
        scopes: ScopeTree<'a>,
        has_with_statement: bool,
        module_record: ModuleRecordBuilder<'a>,
    ) -> Semantic<'a> {
        // The walk must have ended balanced.
        debug_assert!(self.saved_stack.is_empty());
        debug_assert_eq!(self.export_state, ExportState::None);
        self.sync_scope_maps(scopes.list.len());

        let symbol_count = self.symbols.len();
        let declarations = self.declarations;
        let redeclarations = self.redeclarations;
        let node_symbols = self.node_symbols;

        let node_count = tree.node_count();
        let mut node_references = ArenaVec::with_capacity_in(node_count, self.allocator);
        node_references.resize(node_count, None);

        let mut use_ranges = ArenaVec::with_capacity_in(symbol_count, self.allocator);
        use_ranges.resize(symbol_count, Range::default());

        for reference_index in 0..self.references.len() {
            let reference = self.references[reference_index];
            node_references[reference.node.index()] =
                Some(ReferenceId::from_usize(reference_index));
            let name = reference.name;
            // The implicit arguments object (10.2.11
            // argumentsObjectNeeded) shadows outer bindings; an own
            // parameter or var still wins.
            let arguments_barrier = matches!(
                reference.flags.space,
                ReferenceSpace::Value | ReferenceSpace::Typeof
            ) && tree.get_utf8(reference.name.into()).as_bytes()
                == b"arguments";
            let mut resolved = None;
            for ancestor in scopes.ancestors(Some(reference.scope)) {
                if let Some(&id) = self.scope_maps[ancestor.index()].get(&name) {
                    let symbol = self.symbols[id.index()];
                    // A binding outside the reference's space does not
                    // shadow; keep walking.
                    if symbol.flags.visible_in(reference.flags.space) {
                        resolved = Some(id);
                        break;
                    }
                }
                if arguments_barrier && is_arguments_barrier(tree, scopes.get(ancestor)) {
                    break;
                }
            }
            self.references[reference_index].symbol = resolved;
            if let Some(symbol) = resolved {
                use_ranges[symbol.index()].len += 1;
            }
            if reference.flags.space == ReferenceSpace::Any
                && resolved.is_some_and(|symbol| {
                    let flags = self.symbols[symbol.index()].flags;
                    !flags.contains(SymbolFlags::IMPORT) && !flags.in_value_space()
                })
            {
                self.references[reference_index].flags = reference.flags.no_access();
            }
        }

        if has_with_statement {
            let with_ancestry = with_ancestry(tree, &scopes);
            for reference_index in 0..self.references.len() {
                let reference = self.references[reference_index];
                if reference.flags.space != ReferenceSpace::Value {
                    continue;
                }
                let reference_ancestors = with_ancestry[reference.scope.index()];
                let Some(reference_with) = reference_ancestors.lexical else {
                    continue;
                };
                let target_with = if let Some(symbol) = reference.symbol {
                    let scope = self.symbols[symbol.index()].scope;
                    with_ancestry[scope.index()].lexical
                } else if tree.get_utf8(reference.name.into()).as_bytes() == b"arguments" {
                    reference_ancestors.arguments
                } else {
                    None
                };
                if target_with != Some(reference_with) {
                    self.references[reference_index].flags = reference.flags.with_dynamic();
                }
            }
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
        for (index, reference) in self.references.iter().enumerate() {
            let Some(symbol) = reference.symbol else {
                continue;
            };
            let range = &mut use_ranges[symbol.index()];
            use_ids[(range.start + range.len) as usize] = ReferenceId::from_usize(index);
            range.len += 1;
        }

        let module_record =
            module_record.finalize(&node_symbols, &self.references, &node_references);

        Semantic {
            scopes,
            symbols: self.symbols,
            references: self.references,
            declarations,
            redeclarations,
            module_record,
            use_ids,
            use_ranges,
            scope_maps: self.scope_maps,
            hoisting_variables: self.hoisting_variables,
            wtf8_bindings: self.wtf8_bindings,
            node_symbols,
            node_references,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct WithAncestry {
    lexical: Option<ScopeId>,
    arguments: Option<ScopeId>,
}

fn with_ancestry(tree: &Ast<'_>, scopes: &ScopeTree<'_>) -> Vec<WithAncestry> {
    // Scope ids are allocated parent-first, so both nearest-with chains are
    // computed in one linear pass and stored in one temporary allocation.
    let mut ancestry: Vec<WithAncestry> = Vec::with_capacity(scopes.list.len());
    for (index, scope) in scopes.list.iter().enumerate() {
        let id = ScopeId::from_usize(index);
        let parent = match scope.parent {
            Some(parent) => ancestry[parent.index()],
            None => WithAncestry::default(),
        };
        let lexical = if scope.kind == ScopeKind::With {
            Some(id)
        } else {
            parent.lexical
        };
        let arguments = if is_arguments_barrier(tree, *scope) {
            lexical
        } else {
            parent.arguments
        };
        ancestry.push(WithAncestry { lexical, arguments });
    }
    ancestry
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

fn nearest_conditional_scope(tree: &Ast<'_>, scope: &ScopeTracker<'_>) -> ScopeId {
    for id in scope.ancestors(Some(scope.current)) {
        if matches!(
            tree.node_kind(scope.get(id).node),
            NodeKind::TsConditionalType
        ) {
            return id;
        }
    }
    scope.current
}

// Non-arrow functions hold the implicit arguments object. Static blocks
// have no arguments at all; referencing it there is an early error.
fn is_arguments_barrier(tree: &Ast<'_>, scope: scope::Scope) -> bool {
    match scope.kind {
        ScopeKind::Function => match tree.node_kind(scope.node) {
            NodeKind::Function => true,
            NodeKind::Program => tree.source_type() == SourceType::CommonJs,
            _ => false,
        },
        ScopeKind::StaticBlock => true,
        _ => false,
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

// The expression-name scope sits between the outer and function/class
// scopes, so it is the parent of the current scope.
fn expression_name_scope(scope: &ScopeTracker<'_>) -> ScopeId {
    let current = scope.current_scope();
    if matches!(current.kind, ScopeKind::Function | ScopeKind::Class) {
        current
            .parent
            .expect("function and class scopes have a parent")
    } else {
        scope.current
    }
}

// A namespace occupies value space when its body has a value-producing
// statement. Body-less ambient modules are instantiated by specification.
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
