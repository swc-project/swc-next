use oxc_index::Idx;
use swc_next_allocator::{Allocator, CloneIn, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    Ast, Class, ClassType, ExportNamedDeclaration, Function, FunctionType, ImportDeclaration,
    ImportOrExportKind, ImportSpecifier, JsxElementName, JsxElementNameData, JsxIdentifier,
    JsxMemberExpressionObjectData, NodeId, NodeKind, NodeKindData, ReferenceId, ScopeId,
    SourceType, String as AstString, SymbolId, TsEnumDeclaration, TsInterfaceDeclaration,
    TsModuleDeclaration, TsTypeAliasDeclaration, VariableDeclaration, VariableKind,
};

use crate::{
    ScopeMap, Semantic,
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
/// with `interface` merging. [`Semantic::decls`] lists every declaration
/// site.
///
/// `flags` describes the declaration kinds, the modifiers, and the spaces
/// the symbol occupies.
#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    pub name: AstString,
    pub flags: SymbolFlags,
    /// The scope the symbol is declared in. For a hoisting `var`,
    /// this is the hoist target it lands in, not the block it is
    /// written in.
    pub scope: ScopeId,
    // window into `Semantic.decl_nodes`, expanded by `Semantic.decls`
    pub decls: Range,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SymbolFlags {
    pub function_scoped_var: bool,
    pub block_scoped_var: bool,
    pub function: bool,
    pub class: bool,
    pub regular_enum: bool,
    pub const_enum: bool,
    pub value_module: bool,
    pub interface: bool,
    pub type_alias: bool,
    pub type_parameter: bool,
    pub namespace_module: bool,
    pub import: bool,
    pub type_import: bool,
    pub const_var: bool,
    pub ambient: bool,
    pub parameter: bool,
    pub catch_var: bool,
    pub exported: bool,
    pub is_default: bool,
    pub enum_member: bool,
    pub _reserved: u16,
}

impl SymbolFlags {
    /// `var` / `let` / `const`, parameters and catch bindings included.
    pub const VARIABLE: Self = Self {
        function_scoped_var: true,
        block_scoped_var: true,
        ..Self::NONE
    };

    /// Any import binding, value (`import x`) or type-only (`import type x`).
    pub const ANY_IMPORT: Self = Self {
        import: true,
        type_import: true,
        ..Self::NONE
    };

    /// Declarations that live in JS value space (visible at runtime).
    pub const VALUE_SPACE: Self = Self {
        function_scoped_var: true,
        block_scoped_var: true,
        function: true,
        class: true,
        regular_enum: true,
        const_enum: true,
        value_module: true,
        enum_member: true,
        ..Self::NONE
    };

    /// Declarations that live in TS type space.
    pub const TYPE_SPACE: Self = Self {
        class: true,
        regular_enum: true,
        const_enum: true,
        interface: true,
        type_alias: true,
        type_parameter: true,
        enum_member: true,
        ..Self::NONE
    };

    /// Declarations a dotted type name can start from.
    pub const NAMESPACE_SPACE: Self = Self {
        value_module: true,
        namespace_module: true,
        regular_enum: true,
        const_enum: true,
        ..Self::NONE
    };

    pub const NONE: Self = Self {
        function_scoped_var: false,
        block_scoped_var: false,
        function: false,
        class: false,
        regular_enum: false,
        const_enum: false,
        value_module: false,
        interface: false,
        type_alias: false,
        type_parameter: false,
        namespace_module: false,
        import: false,
        type_import: false,
        const_var: false,
        ambient: false,
        parameter: false,
        catch_var: false,
        exported: false,
        is_default: false,
        enum_member: false,
        _reserved: 0,
    };

    const BLOCK_SCOPED_LIKE: Self = Self {
        block_scoped_var: true,
        class: true,
        function: true,
        ..Self::NONE
    };

    /// True when `a` and `b` have at least one flag in common.
    pub fn intersects(self, other: Self) -> bool {
        (self.function_scoped_var && other.function_scoped_var)
            || (self.block_scoped_var && other.block_scoped_var)
            || (self.function && other.function)
            || (self.class && other.class)
            || (self.regular_enum && other.regular_enum)
            || (self.const_enum && other.const_enum)
            || (self.value_module && other.value_module)
            || (self.interface && other.interface)
            || (self.type_alias && other.type_alias)
            || (self.type_parameter && other.type_parameter)
            || (self.namespace_module && other.namespace_module)
            || (self.import && other.import)
            || (self.type_import && other.type_import)
            || (self.const_var && other.const_var)
            || (self.ambient && other.ambient)
            || (self.parameter && other.parameter)
            || (self.catch_var && other.catch_var)
            || (self.exported && other.exported)
            || (self.is_default && other.is_default)
            || (self.enum_member && other.enum_member)
    }

    /// The union of two flag sets. Used when merging two compatible
    /// declarations into a single symbol.
    pub fn merge(self, other: Self) -> Self {
        Self {
            function_scoped_var: self.function_scoped_var || other.function_scoped_var,
            block_scoped_var: self.block_scoped_var || other.block_scoped_var,
            function: self.function || other.function,
            class: self.class || other.class,
            regular_enum: self.regular_enum || other.regular_enum,
            const_enum: self.const_enum || other.const_enum,
            value_module: self.value_module || other.value_module,
            interface: self.interface || other.interface,
            type_alias: self.type_alias || other.type_alias,
            type_parameter: self.type_parameter || other.type_parameter,
            namespace_module: self.namespace_module || other.namespace_module,
            import: self.import || other.import,
            type_import: self.type_import || other.type_import,
            const_var: self.const_var || other.const_var,
            ambient: self.ambient || other.ambient,
            parameter: self.parameter || other.parameter,
            catch_var: self.catch_var || other.catch_var,
            exported: self.exported || other.exported,
            is_default: self.is_default || other.is_default,
            enum_member: self.enum_member || other.enum_member,
            _reserved: 0,
        }
    }

    /// True for a `var` that hoists past intermediate blocks.
    /// Parameters and catch variables are function-scoped but do
    /// not hoist, so they return false here.
    pub fn is_hoisting_var(self) -> bool {
        self.function_scoped_var && !self.parameter && !self.catch_var
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
    /// Import bindings alias symbols of unknowable space and are
    /// visible in every space.
    pub fn visible_in(self, space: ReferenceSpace) -> bool {
        if self.intersects(Self::ANY_IMPORT) {
            return true;
        }
        match space {
            ReferenceSpace::Value | ReferenceSpace::Typeof => self.in_value_space(),
            ReferenceSpace::Type => self.in_type_space(),
            ReferenceSpace::Namespace => self.in_namespace_space(),
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
        if self.function {
            "function"
        } else if self.class {
            "class"
        } else if self.regular_enum || self.const_enum {
            "enum"
        } else if self.value_module || self.namespace_module {
            "namespace"
        } else if self.interface {
            "interface"
        } else if self.type_alias {
            "type alias"
        } else if self.type_import {
            "type import"
        } else if self.import {
            "import"
        } else if self.parameter {
            "parameter"
        } else if self.catch_var {
            "catch parameter"
        } else if self.type_parameter {
            "type parameter"
        } else if self.enum_member {
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
    pub const FUNCTION_SCOPED_VAR: SymbolFlags = SymbolFlags {
        function_scoped_var: false,
        function: false,
        ..SymbolFlags::VALUE_SPACE
    };
    /// Function in a hoist scope (function/global/static block). TS
    /// allows function overloads and sloppy JS permits merging with
    /// `var`. Lexical scopes use the block-scoped excludes instead.
    pub const FUNCTION: SymbolFlags = SymbolFlags {
        function_scoped_var: false,
        function: false,
        value_module: false,
        class: false,
        ..SymbolFlags::VALUE_SPACE
    };
    pub const CLASS: SymbolFlags = SymbolFlags {
        function_scoped_var: true,
        block_scoped_var: true,
        function: true,
        class: true,
        regular_enum: true,
        const_enum: true,
        value_module: false,
        interface: false,
        type_alias: true,
        type_parameter: true,
        enum_member: true,
        ..SymbolFlags::NONE
    };
    pub const INTERFACE: SymbolFlags = SymbolFlags {
        class: false,
        regular_enum: true,
        const_enum: true,
        interface: false,
        type_alias: true,
        type_parameter: true,
        enum_member: true,
        ..SymbolFlags::NONE
    };
    pub const TYPE_ALIAS: SymbolFlags = SymbolFlags::TYPE_SPACE;
    pub const REGULAR_ENUM: SymbolFlags = SymbolFlags {
        function_scoped_var: true,
        block_scoped_var: true,
        function: true,
        class: true,
        regular_enum: false,
        const_enum: true,
        value_module: false,
        interface: true,
        type_alias: true,
        type_parameter: true,
        enum_member: true,
        ..SymbolFlags::NONE
    };
    pub const CONST_ENUM: SymbolFlags = SymbolFlags {
        function_scoped_var: true,
        block_scoped_var: true,
        function: true,
        class: true,
        regular_enum: true,
        const_enum: false,
        value_module: true,
        interface: true,
        type_alias: true,
        type_parameter: true,
        enum_member: true,
        ..SymbolFlags::NONE
    };
    pub const VALUE_MODULE: SymbolFlags = SymbolFlags {
        function_scoped_var: true,
        block_scoped_var: true,
        function: false,
        class: false,
        regular_enum: false,
        const_enum: true,
        value_module: false,
        enum_member: true,
        ..SymbolFlags::NONE
    };
    pub const NAMESPACE_MODULE: SymbolFlags = SymbolFlags::NONE;
    pub const IMPORT_BINDING: SymbolFlags = SymbolFlags::ANY_IMPORT;
    pub const PARAMETER: SymbolFlags = SymbolFlags {
        function_scoped_var: false,
        ..SymbolFlags::VALUE_SPACE
    };
    pub const CATCH_VAR: SymbolFlags = SymbolFlags::VALUE_SPACE;
    // Type parameters merge (multiple `infer T` in one conditional
    // unify). Duplicate explicit `<T, T>` is caught structurally.
    pub const TYPE_PARAMETER: SymbolFlags = SymbolFlags {
        type_parameter: false,
        ..SymbolFlags::TYPE_SPACE
    };
    pub const ENUM_MEMBER: SymbolFlags = SymbolFlags {
        function_scoped_var: true,
        block_scoped_var: true,
        function: true,
        class: true,
        regular_enum: true,
        const_enum: true,
        value_module: true,
        interface: true,
        type_alias: true,
        type_parameter: true,
        enum_member: true,
        ..SymbolFlags::NONE
    };
}

/// A use of a name. One is recorded for every `identifier_reference`
/// in the source, for JSX component tag names (`<Foo>`), and for TS
/// type-predicate parameters (`x is T`). Declaration sites live on
/// the symbol itself.
///
/// ## Example
/// ```ts
/// let a = b; a = 2;
/// //      ^ a read of `b`
/// //         ^ a write of `a`, flags.write
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Reference {
    pub name: AstString,
    /// The scope the reference appears in.
    pub scope: ScopeId,
    /// The referencing node.
    pub node: NodeId,
    /// The declaring symbol this reference resolves to. `None` for
    /// names with no binding visible in the reference's space.
    pub symbol: Option<SymbolId>,
    pub flags: ReferenceFlags,
}

#[derive(Debug, Clone, Copy)]
pub struct ReferenceFlags {
    /// True when this reference (re)assigns its binding: the target of
    /// an assignment, the operand of `++`/`--`, the iteration variable
    /// of for-in/for-of, or a destructuring assignment leaf.
    /// Initializers in declarations are not references, so an initialized
    /// but never reassigned binding has no write.
    pub write: bool,
    /// The declaration space this position resolves in.
    pub space: ReferenceSpace,
    pub _reserved: u8,
}

impl Default for ReferenceFlags {
    fn default() -> Self {
        Self {
            write: false,
            space: ReferenceSpace::Value,
            _reserved: 0,
        }
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
    /// an alias position that accepts every space: `export { x }`,
    /// `export default x`, `export = x`, and `import a = x`
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
    decl_pairs: ArenaVec<'a, DeclPair>,
    // first declarator of each symbol, parallel to `symbols`
    first_decls: ArenaVec<'a, NodeId>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, ScopeMap<'a>>,

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
    saved_stack: ArenaVec<'a, SavedContext>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ExportState {
    #[default]
    None,
    Named,
    Default,
}

/// The declaration context for the next `binding_identifier`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingBinding {
    pub flags: SymbolFlags,
    pub excludes: SymbolFlags,
    pub scope: ScopeId,
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
struct DeclPair {
    symbol_id: SymbolId,
    node: NodeId,
}

#[derive(Clone, Copy)]
struct SavedContext {
    pending: PendingBinding,
    export_state: ExportState,
    ambient: bool,
}

/// Per-node facts computed by the walker for
/// [`SymbolTracker::declare_bindings`].
#[derive(Clone, Copy)]
pub(crate) struct RefContext {
    /// the node is an identifier in assignment-target position
    pub is_write: bool,
    /// the declaration space the identifier resolves in
    pub space: ReferenceSpace,
}

impl Default for RefContext {
    fn default() -> Self {
        Self {
            is_write: false,
            space: ReferenceSpace::Value,
        }
    }
}

impl<'a> SymbolTracker<'a> {
    pub fn init(tree: &Ast<'a>) -> Self {
        assert!(tree.node_count() > 0);
        let allocator = tree.allocator();
        let nodes = tree.node_count();
        Self {
            allocator,
            symbols: ArenaVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            references: ArenaVec::with_capacity_in(16usize.max(nodes / 4), allocator),
            decl_pairs: ArenaVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            first_decls: ArenaVec::with_capacity_in(16usize.max(nodes / 12), allocator),
            scope_maps: ArenaVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            hoisting_variables: ArenaVec::with_capacity_in(8usize.max(nodes / 16), allocator),
            pending: PendingBinding::default(),
            export_state: ExportState::None,
            // A declaration file is ambient in its entirety.
            ambient: tree.lang() == swc_next_ecma_ast::Lang::Dts,
            saved_stack: ArenaVec::with_capacity_in(32, allocator),
        }
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
            self.export_state = ExportState::Named;
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
    ) {
        self.push_saved_context();
        self.pending = match declaration.kind(tree) {
            VariableKind::Var => {
                let target = scope.hoist_target();
                let mut excludes = Excludes::FUNCTION_SCOPED_VAR;
                // Module-level function declarations are lexical;
                // everywhere else (and in TS) `var` merges with them.
                if !tree.is_ts() && scope.get(target).kind == ScopeKind::Module {
                    excludes.function = true;
                }
                PendingBinding {
                    flags: SymbolFlags {
                        function_scoped_var: true,
                        ambient: declaration.declare(tree) || self.ambient,
                        ..SymbolFlags::NONE
                    },
                    excludes,
                    scope: target,
                }
            }
            VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing => {
                PendingBinding {
                    flags: SymbolFlags {
                        block_scoped_var: true,
                        const_var: true,
                        ambient: declaration.declare(tree) || self.ambient,
                        ..SymbolFlags::NONE
                    },
                    excludes: Excludes::BLOCK_SCOPED_VAR,
                    scope: scope.current,
                }
            }
            VariableKind::Let => PendingBinding {
                flags: SymbolFlags {
                    block_scoped_var: true,
                    ambient: declaration.declare(tree) || self.ambient,
                    ..SymbolFlags::NONE
                },
                excludes: Excludes::BLOCK_SCOPED_VAR,
                scope: scope.current,
            },
        };
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
        let target_kind = scope.get(target).kind;
        let allow_overload = tree.is_ts()
            || matches!(
                target_kind,
                ScopeKind::Function
                    | ScopeKind::FunctionBody
                    | ScopeKind::Global
                    | ScopeKind::StaticBlock
            );
        // TS overloads, sloppy JS Annex B.3.2, and global merge with
        // var. Lexical scopes (block/module) do not.
        self.pending = PendingBinding {
            flags: SymbolFlags {
                function: true,
                ambient: self.ambient,
                ..SymbolFlags::NONE
            },
            excludes: if allow_overload {
                Excludes::FUNCTION
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
            flags: SymbolFlags {
                function: true,
                ambient: true,
                ..SymbolFlags::NONE
            },
            excludes: Excludes::FUNCTION,
            scope: target,
        };
    }

    pub fn enter_class(&mut self, tree: &Ast, class: Class, scope: &ScopeTracker<'a>) {
        self.push_saved_context();
        let is_declaration = class.class_type(tree) == ClassType::ClassDeclaration;
        self.pending = PendingBinding {
            flags: SymbolFlags {
                class: true,
                ambient: class.declare(tree) || self.ambient,
                ..SymbolFlags::NONE
            },
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
            flags: SymbolFlags {
                function_scoped_var: true,
                parameter: true,
                ambient: self.pending.flags.ambient,
                ..SymbolFlags::NONE
            },
            excludes: Excludes::PARAMETER,
            scope: scope.current,
        };
        self.export_state = ExportState::None;
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
            self.pending.flags = SymbolFlags {
                type_import: true,
                ..SymbolFlags::NONE
            };
            self.pending.excludes = Excludes::IMPORT_BINDING;
        }
    }

    pub fn enter_catch_clause(&mut self, scope: &ScopeTracker<'a>) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags {
                function_scoped_var: true,
                catch_var: true,
                ..SymbolFlags::NONE
            },
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
            flags: SymbolFlags {
                interface: true,
                ambient: declaration.declare(tree) || self.ambient,
                ..SymbolFlags::NONE
            },
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
            flags: SymbolFlags {
                type_alias: true,
                ambient: declaration.declare(tree) || self.ambient,
                ..SymbolFlags::NONE
            },
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
                SymbolFlags {
                    const_enum: true,
                    ambient: declaration.declare(tree) || self.ambient,
                    ..SymbolFlags::NONE
                }
            } else {
                SymbolFlags {
                    regular_enum: true,
                    ambient: declaration.declare(tree) || self.ambient,
                    ..SymbolFlags::NONE
                }
            },
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
            flags: SymbolFlags {
                value_module: instantiated,
                namespace_module: true,
                ambient: declaration.declare(tree) || self.ambient,
                ..SymbolFlags::NONE
            },
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
            flags: SymbolFlags {
                namespace_module: true,
                ..SymbolFlags::NONE
            },
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
            flags: SymbolFlags {
                type_parameter: true,
                ..SymbolFlags::NONE
            },
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
            flags: SymbolFlags {
                type_parameter: true,
                ..SymbolFlags::NONE
            },
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

    /// Materializes the pending binding context into a symbol (for a
    /// `binding_identifier`) or records a reference (for an
    /// `identifier_reference`). Called by the concrete enter hooks it handles,
    /// before their children.
    pub fn declare_bindings(
        &mut self,
        tree: &Ast,
        data: NodeKindData,
        scope: &ScopeTracker<'a>,
        reference_context: RefContext,
    ) {
        if self.scope_maps.len() < scope.scopes.len() {
            self.sync_scope_maps(scope.scopes.len());
        }
        match data {
            NodeKindData::BindingIdentifier(identifier) => {
                let node = identifier.node_id();
                // Type-position identifiers are parameter labels. Only type
                // parameters are real declarations there.
                if reference_context.space.in_type_position() && !self.pending.flags.type_parameter
                {
                    return;
                }
                let symbol_id = self.declare(tree, identifier.name(tree), node);

                // Register a hoisting var in every block it passes through so
                // block-scoped redeclarations see it.
                if self.pending.flags.is_hoisting_var() {
                    for scope_id in scope.ancestors(Some(scope.current)) {
                        if scope_id == self.pending.scope {
                            break;
                        }
                        let name = tree.string(identifier.name(tree)).clone_in(self.allocator);
                        self.hoisting_variables[scope_id.index()]
                            .entry(name)
                            .or_insert(symbol_id);
                    }
                }
            }
            NodeKindData::IdentifierReference(identifier) => {
                let node = identifier.node_id();
                self.add_reference(
                    tree,
                    identifier.name(tree),
                    scope.current,
                    node,
                    ReferenceFlags {
                        write: reference_context.is_write,
                        space: reference_context.space,
                        _reserved: 0,
                    },
                );
            }
            NodeKindData::TsTypePredicate(predicate) => {
                // `param is T` parses `param` as an IdentifierName but it
                // references the enclosing function's parameter. Synthesize
                // a reference so renamers update both together. In type-only
                // signatures the parameter is only a label, so no reference
                // is recorded.
                let parameter_name = predicate.parameter_name(tree);
                let parameter_node = parameter_name.node_id();
                let NodeKindData::IdentifierName(parameter_name) = tree.kind_data(parameter_node)
                else {
                    return;
                };
                let name = parameter_name.name(tree);
                let Some(parameter) = self.own_binding(scope.current, tree.string(name)) else {
                    return;
                };
                if !self.symbol(parameter).flags.parameter {
                    return;
                }
                self.add_reference(
                    tree,
                    name,
                    scope.current,
                    parameter_node,
                    ReferenceFlags {
                        space: ReferenceSpace::Typeof,
                        ..ReferenceFlags::default()
                    },
                );
            }
            NodeKindData::TsEnumMember(member) => {
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
                let member_id = member.id(tree).node_id();
                let name = match tree.kind_data(member_id) {
                    NodeKindData::IdentifierName(identifier) => identifier.name(tree),
                    NodeKindData::StringLiteral(literal) => literal.value(tree),
                    _ => return,
                };
                let saved = self.pending;
                self.pending = PendingBinding {
                    flags: SymbolFlags {
                        enum_member: true,
                        ambient: self.ambient,
                        ..SymbolFlags::NONE
                    },
                    excludes: Excludes::ENUM_MEMBER,
                    scope: scope.current,
                };
                self.declare(tree, name, member_id);
                self.pending = saved;
            }
            NodeKindData::JsxOpeningElement(element) => {
                // The leftmost capitalized JSX identifier is a JS binding
                // reference. Lowercase tags are intrinsic element names; the
                // rest of a member chain is property syntax.
                self.add_jsx_reference(tree, element.name(tree), scope.current);
            }
            NodeKindData::JsxClosingElement(element) => {
                self.add_jsx_reference(tree, element.name(tree), scope.current);
            }
            _ => {}
        }
    }

    fn add_jsx_reference(&mut self, tree: &Ast, name: JsxElementName, scope: ScopeId) {
        let Some(identifier) = jsx_tag_root(tree, tree.jsx_element_name_data(name)) else {
            return;
        };
        let text = tree.string(identifier.name(tree));
        if text.first().is_some_and(u8::is_ascii_uppercase) {
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
    pub fn declare(&mut self, tree: &Ast, name: AstString, node: NodeId) -> SymbolId {
        let target = self.pending.scope;
        assert!(target.index() < self.scope_maps.len());
        assert!(node.index() < tree.node_count());

        let name_text = tree.string(name);
        let id = if let Some(existing) = self.binding(target, name_text) {
            let symbol = &mut self.symbols[existing.index()];
            if !symbol.flags.intersects(self.pending.excludes) {
                let mut merged = symbol.flags.merge(self.pending.flags);
                merged.exported = merged.exported || self.export_state != ExportState::None;
                merged.is_default = merged.is_default || self.export_state == ExportState::Default;
                // An overload implementation emits at runtime, so the merged
                // symbol is non-ambient.
                merged.ambient = symbol.flags.ambient && self.pending.flags.ambient;
                symbol.flags = merged;
            }
            existing
        } else {
            assert!(self.symbols.len() < u32::MAX as usize);
            let new_id = symbol_id(self.symbols.len());
            let mut flags = self.pending.flags;
            flags.exported = self.export_state != ExportState::None;
            flags.is_default = self.export_state == ExportState::Default;
            self.symbols.push(Symbol {
                name,
                flags,
                scope: target,
                decls: Range::default(),
            });
            self.first_decls.push(node);
            let key = tree.string(name).clone_in(self.allocator);
            self.scope_maps[target.index()].insert(key, new_id);
            new_id
        };

        self.decl_pairs.push(DeclPair {
            symbol_id: id,
            node,
        });
        id
    }

    /// Records an identifier reference at `node` in `scope`.
    pub fn add_reference(
        &mut self,
        tree: &Ast,
        name: AstString,
        scope: ScopeId,
        node: NodeId,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        assert!(node.index() < tree.node_count());
        assert!(self.references.len() < u32::MAX as usize);
        let id = reference_id(self.references.len());
        self.references.push(Reference {
            name,
            scope,
            node,
            symbol: None,
            flags,
        });
        id
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        assert!(id.index() < self.symbols.len());
        self.symbols[id.index()]
    }

    /// The `binding_identifier` node of the earliest declaration of `id`.
    /// Later declarations merged into the symbol do not change it.
    pub fn first_decl_of(&self, id: SymbolId) -> NodeId {
        assert!(id.index() < self.first_decls.len());
        self.first_decls[id.index()]
    }

    /// The binding of `name` declared directly in `scope`, excluding
    /// hoisting `var`s passing through.
    pub fn own_binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps
            .get(scope.index())
            .and_then(|map| map.get(name).copied())
    }

    /// The binding of `name` at `scope`, including a hoisting `var` passing
    /// through on its way to its hoist target.
    pub fn binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.own_binding(scope, name).or_else(|| {
            self.hoisting_variables
                .get(scope.index())
                .and_then(|map| map.get(name).copied())
        })
    }

    fn sync_scope_maps(&mut self, count: usize) {
        self.scope_maps.reserve(count - self.scope_maps.len());
        while self.scope_maps.len() < count {
            self.scope_maps.push(ScopeMap::new_in(self.allocator));
        }
        self.hoisting_variables
            .reserve(count - self.hoisting_variables.len());
        while self.hoisting_variables.len() < count {
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
        node_scopes: ArenaVec<'a, ScopeId>,
        node_parents: ArenaVec<'a, Option<NodeId>>,
    ) -> Semantic<'a> {
        // The walk must have ended balanced.
        assert!(self.saved_stack.is_empty());
        assert_eq!(self.export_state, ExportState::None);
        self.sync_scope_maps(scopes.list.len());

        // Declaration index: count, prefix-sum, fill. `decls.len`
        // doubles as the write cursor during the fill.
        let symbol_count = self.symbols.len();
        let mut decl_nodes = ArenaVec::with_capacity_in(self.decl_pairs.len(), self.allocator);
        for _ in 0..self.decl_pairs.len() {
            decl_nodes.push(NodeId::from_usize_unchecked(0));
        }
        for symbol in &mut self.symbols {
            symbol.decls = Range::default();
        }
        for pair in &self.decl_pairs {
            self.symbols[pair.symbol_id.index()].decls.len += 1;
        }
        let mut declaration_offset = 0u32;
        for symbol in &mut self.symbols {
            let count = symbol.decls.len;
            symbol.decls.start = declaration_offset;
            symbol.decls.len = 0;
            declaration_offset += count;
        }
        assert_eq!(declaration_offset as usize, self.decl_pairs.len());
        for pair in &self.decl_pairs {
            let symbol = &mut self.symbols[pair.symbol_id.index()];
            let index = (symbol.decls.start + symbol.decls.len) as usize;
            decl_nodes[index] = pair.node;
            symbol.decls.len += 1;
        }

        let node_count = tree.node_count();
        let mut node_symbols = ArenaVec::with_capacity_in(node_count, self.allocator);
        let mut node_references = ArenaVec::with_capacity_in(node_count, self.allocator);
        for _ in 0..node_count {
            node_symbols.push(None);
            node_references.push(None);
        }
        for pair in &self.decl_pairs {
            node_symbols[pair.node.index()] = Some(pair.symbol_id);
        }
        for (index, reference) in self.references.iter().enumerate() {
            node_references[reference.node.index()] = Some(reference_id(index));
        }

        for reference_index in 0..self.references.len() {
            let reference = self.references[reference_index];
            let name = tree.string(reference.name);
            // The implicit arguments object (10.2.11
            // argumentsObjectNeeded) shadows outer bindings; an own
            // parameter or var still wins.
            let arguments_barrier = matches!(
                reference.flags.space,
                ReferenceSpace::Value | ReferenceSpace::Typeof
            ) && name == b"arguments";
            let mut resolved = None;
            for ancestor in scopes.ancestors(Some(reference.scope)) {
                if let Some(&id) = self.scope_maps[ancestor.index()].get(name) {
                    let symbol = self.symbols[id.index()];
                    // A binding outside the reference's space does not
                    // shadow; keep walking.
                    if symbol.flags.visible_in(reference.flags.space)
                        && type_parameter_visible(
                            tree,
                            symbol,
                            reference.node,
                            &scopes,
                            &node_parents,
                        )
                    {
                        resolved = Some(id);
                        break;
                    }
                }
                if arguments_barrier && is_arguments_barrier(tree, scopes.get(ancestor)) {
                    break;
                }
            }
            self.references[reference_index].symbol = resolved;
        }

        // Use index: count, prefix-sum, fill. `len` doubles as the write
        // cursor during the fill.
        let mut use_ranges = ArenaVec::with_capacity_in(symbol_count, self.allocator);
        for _ in 0..symbol_count {
            use_ranges.push(Range::default());
        }
        for reference in &self.references {
            if let Some(symbol) = reference.symbol {
                use_ranges[symbol.index()].len += 1;
            }
        }
        let mut use_offset = 0u32;
        for range in &mut use_ranges {
            range.start = use_offset;
            use_offset += range.len;
            range.len = 0;
        }
        let mut use_ids = ArenaVec::with_capacity_in(use_offset as usize, self.allocator);
        for _ in 0..use_offset {
            use_ids.push(reference_id(0));
        }
        for (index, reference) in self.references.iter().enumerate() {
            let Some(symbol) = reference.symbol else {
                continue;
            };
            let range = &mut use_ranges[symbol.index()];
            use_ids[(range.start + range.len) as usize] = reference_id(index);
            range.len += 1;
        }

        Semantic {
            scopes,
            symbols: self.symbols,
            references: self.references,
            decl_nodes,
            use_ids,
            use_ranges,
            scope_maps: self.scope_maps,
            hoisting_variables: self.hoisting_variables,
            node_scopes,
            node_parents,
            node_symbols,
            node_references,
        }
    }
}

fn import_pending(kind: ImportOrExportKind, scope: ScopeId) -> PendingBinding {
    PendingBinding {
        flags: if kind == ImportOrExportKind::Type {
            SymbolFlags {
                type_import: true,
                ..SymbolFlags::NONE
            }
        } else {
            SymbolFlags {
                import: true,
                ..SymbolFlags::NONE
            }
        },
        excludes: Excludes::IMPORT_BINDING,
        scope,
    }
}

// TypeScript restricts type parameters beyond lexical scoping: infer
// variables exist only in their conditional's true branch, and class
// type parameters are hidden in static members (TS2302) and computed
// member keys (TS2467).
fn type_parameter_visible(
    tree: &Ast<'_>,
    symbol: Symbol,
    reference_node: NodeId,
    scopes: &ScopeTree<'_>,
    node_parents: &[Option<NodeId>],
) -> bool {
    if !symbol.flags.type_parameter {
        return true;
    }
    let scope_node = scopes.get(symbol.scope).node;
    match tree.kind_data(scope_node) {
        NodeKindData::TsConditionalType(conditional) => in_subtree(
            node_parents,
            reference_node,
            scope_node,
            conditional.true_type(tree).node_id(),
        ),
        NodeKindData::Class(_) => {
            !class_type_parameter_hidden(tree, node_parents, reference_node, scope_node)
        }
        _ => true,
    }
}

// Whether walking up from `node` reaches `root` through `subtree`.
fn in_subtree(
    node_parents: &[Option<NodeId>],
    node: NodeId,
    root: NodeId,
    subtree: NodeId,
) -> bool {
    let mut child = node;
    let mut parent = node_parents[child.index()];
    while let Some(current_parent) = parent {
        if current_parent == root {
            return child == subtree;
        }
        child = current_parent;
        parent = node_parents[current_parent.index()];
    }
    false
}

fn class_type_parameter_hidden(
    tree: &Ast<'_>,
    node_parents: &[Option<NodeId>],
    reference_node: NodeId,
    class_node: NodeId,
) -> bool {
    let mut child = reference_node;
    let mut parent = node_parents[child.index()];
    while let Some(current_parent) = parent {
        match tree.kind_data(current_parent) {
            NodeKindData::MethodDefinition(method) => {
                if method.computed(tree)
                    && method.key(tree).node_id() == child
                    && member_owner(node_parents, current_parent) == Some(class_node)
                {
                    return true;
                }
            }
            NodeKindData::TsMethodDefinition(method) => {
                if method.computed(tree)
                    && method.key(tree).node_id() == child
                    && member_owner(node_parents, current_parent) == Some(class_node)
                {
                    return true;
                }
            }
            NodeKindData::PropertyDefinition(property) => {
                if property.computed(tree)
                    && property.key(tree).node_id() == child
                    && member_owner(node_parents, current_parent) == Some(class_node)
                {
                    return true;
                }
            }
            NodeKindData::ClassBody(_) => {
                if node_parents[current_parent.index()] != Some(class_node) {
                    child = current_parent;
                    parent = node_parents[current_parent.index()];
                    continue;
                }
                return match tree.kind_data(child) {
                    NodeKindData::MethodDefinition(method) => method.r#static(tree),
                    NodeKindData::TsMethodDefinition(method) => method.r#static(tree),
                    NodeKindData::PropertyDefinition(property) => property.r#static(tree),
                    NodeKindData::TsIndexSignature(signature) => signature.r#static(tree),
                    NodeKindData::StaticBlock(_) => true,
                    _ => false,
                };
            }
            _ => {}
        }
        child = current_parent;
        parent = node_parents[current_parent.index()];
    }
    false
}

fn member_owner(node_parents: &[Option<NodeId>], member: NodeId) -> Option<NodeId> {
    node_parents[member.index()].and_then(|body| node_parents[body.index()])
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

#[inline]
fn symbol_id(index: usize) -> SymbolId {
    assert!(index < SymbolId::MAX);
    SymbolId::from_usize(index)
}

#[inline]
fn reference_id(index: usize) -> ReferenceId {
    assert!(index < ReferenceId::MAX);
    ReferenceId::from_usize(index)
}
