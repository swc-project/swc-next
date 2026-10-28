bitflags::bitflags! {
    /// Scope-dependent syntax checks to run during semantic analysis.
    ///
    /// The default is [`SyntaxChecks::empty`]. Use [`SyntaxChecks::all`] to
    /// enable every check, `|` to combine checks, or `all() - flag` to exclude
    /// individual checks. These flags only control diagnostics; scope, binding,
    /// reference, and alias resolution are independent of the selected checks.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct SyntaxChecks: u64 {
        /// Octal string escapes in strict mode.
        const OCTAL_ESCAPES = 1 << 0;
        /// Legacy octal and leading-zero decimal literals in strict mode.
        const LEGACY_NUMERIC_LITERALS = 1 << 1;
        /// Regular expression patterns and flags.
        const REGEXP_LITERALS = 1 << 2;
        /// Strict-mode reserved words used as references, bindings, or labels.
        const STRICT_RESERVED_WORDS = 1 << 3;
        /// References to `arguments` in class field initializers and static blocks.
        const ARGUMENTS = 1 << 4;
        /// Bindings named `eval` or `arguments` in strict mode.
        const STRICT_BINDING_NAMES = 1 << 5;
        /// Lexical declarations with a binding named `let`.
        const LEXICAL_BINDING_NAMES = 1 << 6;
        /// Conflicting declarations, including duplicate parameters and TypeScript merges.
        const REDECLARATIONS = 1 << 7;
        /// References to private names outside their declaring class.
        const PRIVATE_NAME_REFERENCES = 1 << 8;
        /// Deleting an identifier in strict mode.
        const DELETE_IDENTIFIERS = 1 << 9;
        /// Deleting a private field.
        const DELETE_PRIVATE_FIELDS = 1 << 10;
        /// Assignments and updates to `eval` or `arguments` in strict mode, including loop targets.
        const STRICT_ASSIGNMENT_TARGETS = 1 << 11;
        /// Duplicate `__proto__` properties in object literals.
        const DUPLICATE_PROTO = 1 << 12;
        /// Invalid `super` property access, including private properties.
        const SUPER_PROPERTIES = 1 << 13;
        /// Calls to `super()` outside derived constructors.
        const SUPER_CALLS = 1 << 14;
        /// Await expressions in invalid contexts.
        const AWAIT_EXPRESSIONS = 1 << 15;
        /// Yield expressions in invalid contexts.
        const YIELD_EXPRESSIONS = 1 << 16;
        /// Resource declarations in invalid script, static-block, or ambient contexts.
        const USING_DECLARATIONS = 1 << 19;
        /// Multiple default clauses in a switch statement.
        const DUPLICATE_SWITCH_DEFAULTS = 1 << 20;
        /// Invalid declaration initializers in for-in and for-of loops.
        const FOR_IN_OF_INITIALIZERS = 1 << 21;
        /// For-await loops in class static blocks.
        const FOR_AWAIT = 1 << 22;
        /// Break statements without a valid target.
        const BREAK_STATEMENTS = 1 << 23;
        /// Continue statements without a valid iteration target.
        const CONTINUE_STATEMENTS = 1 << 24;
        /// Duplicate labels.
        const LABELS = 1 << 25;
        /// With statements in strict mode.
        const WITH_STATEMENTS = 1 << 26;
        /// Function declarations in invalid statement positions.
        const FUNCTION_DECLARATIONS = 1 << 27;
        /// Use-strict directives with non-simple parameter lists.
        const USE_STRICT_DIRECTIVES = 1 << 28;
        /// Duplicate JavaScript class constructors.
        const DUPLICATE_CONSTRUCTORS = 1 << 29;
        /// Conflicting private class elements, allowing matching getter/setter pairs.
        const DUPLICATE_PRIVATE_NAMES = 1 << 30;
        /// Local exports without a binding, including TypeScript export aliases.
        const UNRESOLVED_EXPORTS = 1 << 31;
        /// Import and export declarations outside valid module positions.
        const IMPORT_EXPORT_POSITION = 1 << 32;
        /// Duplicate import attribute keys.
        const DUPLICATE_IMPORT_ATTRIBUTES = 1 << 33;
        /// Duplicate JavaScript exported names.
        const DUPLICATE_EXPORTS = 1 << 34;
        /// Abstract methods with implementations.
        const TS_ABSTRACT_METHODS = 1 << 35;
        /// Invalid rest tuple element syntax.
        const TS_TUPLE_REST = 1 << 36;
        /// Invalid optional labeled tuple element syntax.
        const TS_TUPLE_OPTIONAL = 1 << 37;
        /// TypeScript namespace and ambient module declarations in invalid contexts.
        const TS_MODULE_DECLARATIONS = 1 << 38;
        /// Global augmentations with invalid modifiers or containers.
        const TS_GLOBAL_DECLARATIONS = 1 << 39;
        /// TypeScript export aliases referencing non-local declarations.
        const TS_NON_LOCAL_EXPORTS = 1 << 40;
        /// Duplicate TypeScript namespace export aliases.
        const TS_DUPLICATE_EXPORT_ALIASES = 1 << 41;
        /// TypeScript namespace export aliases conflicting with exported declarations.
        const TS_EXPORT_ALIAS_CONFLICTS = 1 << 42;
        /// TypeScript import aliases conflicting with declarations in their resolved spaces.
        const TS_IMPORT_ALIAS_CONFLICTS = 1 << 43;
        /// TypeScript import aliases referencing type-only exports.
        const TS_TYPE_ONLY_IMPORT_ALIASES = 1 << 44;
        /// Non-entity-name export assignments in ambient contexts.
        const TS_AMBIENT_EXPORT_ASSIGNMENTS = 1 << 45;
        /// TypeScript identifiers named `await` in top-level module contexts.
        const TS_CONTEXTUAL_AWAIT = 1 << 46;
        /// Invalid parameter property bindings and names.
        const TS_PARAMETER_PROPERTIES = 1 << 47;
        /// Non-computed static class members named `prototype`.
        const CLASS_MEMBER_NAMES = 1 << 48;

        /// Checks that inspect identifier references.
        const IDENTIFIER_REFERENCES = Self::STRICT_RESERVED_WORDS.bits()
            | Self::ARGUMENTS.bits()
            | Self::TS_CONTEXTUAL_AWAIT.bits();

        /// Checks that inspect binding identifiers.
        const BINDING_IDENTIFIERS = Self::STRICT_RESERVED_WORDS.bits()
            | Self::STRICT_BINDING_NAMES.bits()
            | Self::LEXICAL_BINDING_NAMES.bits()
            | Self::REDECLARATIONS.bits()
            | Self::DUPLICATE_EXPORTS.bits()
            | Self::TS_CONTEXTUAL_AWAIT.bits();

        /// Checks that inspect class bodies for invalid or conflicting elements.
        const CLASS_BODY = Self::DUPLICATE_CONSTRUCTORS.bits()
            | Self::DUPLICATE_PRIVATE_NAMES.bits()
            | Self::CLASS_MEMBER_NAMES.bits();

        /// Checks that validate TypeScript namespace export aliases.
        const TS_EXPORT_ALIASES = Self::UNRESOLVED_EXPORTS.bits()
            | Self::TS_NON_LOCAL_EXPORTS.bits()
            | Self::TS_DUPLICATE_EXPORT_ALIASES.bits()
            | Self::TS_EXPORT_ALIAS_CONFLICTS.bits();

        /// Checks that validate TypeScript import aliases.
        const TS_IMPORT_ALIASES = Self::TS_IMPORT_ALIAS_CONFLICTS.bits()
            | Self::TS_TYPE_ONLY_IMPORT_ALIASES.bits();

        /// Checks that validate TypeScript import or export aliases.
        const TS_ALIASES = Self::TS_EXPORT_ALIASES.bits()
            | Self::TS_IMPORT_ALIASES.bits();
    }
}

impl SyntaxChecks {
    /// Whether [`Self::OCTAL_ESCAPES`] is enabled.
    #[inline]
    pub const fn octal_escapes(&self) -> bool {
        self.contains(Self::OCTAL_ESCAPES)
    }

    /// Whether [`Self::LEGACY_NUMERIC_LITERALS`] is enabled.
    #[inline]
    pub const fn legacy_numeric_literals(&self) -> bool {
        self.contains(Self::LEGACY_NUMERIC_LITERALS)
    }

    /// Whether [`Self::REGEXP_LITERALS`] is enabled.
    #[inline]
    pub const fn regexp_literals(&self) -> bool {
        self.contains(Self::REGEXP_LITERALS)
    }

    /// Whether [`Self::STRICT_RESERVED_WORDS`] is enabled.
    #[inline]
    pub const fn strict_reserved_words(&self) -> bool {
        self.contains(Self::STRICT_RESERVED_WORDS)
    }

    /// Whether [`Self::ARGUMENTS`] is enabled.
    #[inline]
    pub const fn arguments(&self) -> bool {
        self.contains(Self::ARGUMENTS)
    }

    /// Whether [`Self::STRICT_BINDING_NAMES`] is enabled.
    #[inline]
    pub const fn strict_binding_names(&self) -> bool {
        self.contains(Self::STRICT_BINDING_NAMES)
    }

    /// Whether [`Self::LEXICAL_BINDING_NAMES`] is enabled.
    #[inline]
    pub const fn lexical_binding_names(&self) -> bool {
        self.contains(Self::LEXICAL_BINDING_NAMES)
    }

    /// Whether [`Self::REDECLARATIONS`] is enabled.
    #[inline]
    pub const fn redeclarations(&self) -> bool {
        self.contains(Self::REDECLARATIONS)
    }

    /// Whether [`Self::PRIVATE_NAME_REFERENCES`] is enabled.
    #[inline]
    pub const fn private_name_references(&self) -> bool {
        self.contains(Self::PRIVATE_NAME_REFERENCES)
    }

    /// Whether [`Self::DELETE_IDENTIFIERS`] is enabled.
    #[inline]
    pub const fn delete_identifiers(&self) -> bool {
        self.contains(Self::DELETE_IDENTIFIERS)
    }

    /// Whether [`Self::DELETE_PRIVATE_FIELDS`] is enabled.
    #[inline]
    pub const fn delete_private_fields(&self) -> bool {
        self.contains(Self::DELETE_PRIVATE_FIELDS)
    }

    /// Whether [`Self::STRICT_ASSIGNMENT_TARGETS`] is enabled.
    #[inline]
    pub const fn strict_assignment_targets(&self) -> bool {
        self.contains(Self::STRICT_ASSIGNMENT_TARGETS)
    }

    /// Whether [`Self::DUPLICATE_PROTO`] is enabled.
    #[inline]
    pub const fn duplicate_proto(&self) -> bool {
        self.contains(Self::DUPLICATE_PROTO)
    }

    /// Whether [`Self::SUPER_PROPERTIES`] is enabled.
    #[inline]
    pub const fn super_properties(&self) -> bool {
        self.contains(Self::SUPER_PROPERTIES)
    }

    /// Whether [`Self::SUPER_CALLS`] is enabled.
    #[inline]
    pub const fn super_calls(&self) -> bool {
        self.contains(Self::SUPER_CALLS)
    }

    /// Whether [`Self::AWAIT_EXPRESSIONS`] is enabled.
    #[inline]
    pub const fn await_expressions(&self) -> bool {
        self.contains(Self::AWAIT_EXPRESSIONS)
    }

    /// Whether [`Self::YIELD_EXPRESSIONS`] is enabled.
    #[inline]
    pub const fn yield_expressions(&self) -> bool {
        self.contains(Self::YIELD_EXPRESSIONS)
    }

    /// Whether [`Self::USING_DECLARATIONS`] is enabled.
    #[inline]
    pub const fn using_declarations(&self) -> bool {
        self.contains(Self::USING_DECLARATIONS)
    }

    /// Whether [`Self::DUPLICATE_SWITCH_DEFAULTS`] is enabled.
    #[inline]
    pub const fn duplicate_switch_defaults(&self) -> bool {
        self.contains(Self::DUPLICATE_SWITCH_DEFAULTS)
    }

    /// Whether [`Self::FOR_IN_OF_INITIALIZERS`] is enabled.
    #[inline]
    pub const fn for_in_of_initializers(&self) -> bool {
        self.contains(Self::FOR_IN_OF_INITIALIZERS)
    }

    /// Whether [`Self::FOR_AWAIT`] is enabled.
    #[inline]
    pub const fn for_await(&self) -> bool {
        self.contains(Self::FOR_AWAIT)
    }

    /// Whether [`Self::BREAK_STATEMENTS`] is enabled.
    #[inline]
    pub const fn break_statements(&self) -> bool {
        self.contains(Self::BREAK_STATEMENTS)
    }

    /// Whether [`Self::CONTINUE_STATEMENTS`] is enabled.
    #[inline]
    pub const fn continue_statements(&self) -> bool {
        self.contains(Self::CONTINUE_STATEMENTS)
    }

    /// Whether [`Self::LABELS`] is enabled.
    #[inline]
    pub const fn labels(&self) -> bool {
        self.contains(Self::LABELS)
    }

    /// Whether [`Self::WITH_STATEMENTS`] is enabled.
    #[inline]
    pub const fn with_statements(&self) -> bool {
        self.contains(Self::WITH_STATEMENTS)
    }

    /// Whether [`Self::FUNCTION_DECLARATIONS`] is enabled.
    #[inline]
    pub const fn function_declarations(&self) -> bool {
        self.contains(Self::FUNCTION_DECLARATIONS)
    }

    /// Whether [`Self::USE_STRICT_DIRECTIVES`] is enabled.
    #[inline]
    pub const fn use_strict_directives(&self) -> bool {
        self.contains(Self::USE_STRICT_DIRECTIVES)
    }

    /// Whether [`Self::DUPLICATE_CONSTRUCTORS`] is enabled.
    #[inline]
    pub const fn duplicate_constructors(&self) -> bool {
        self.contains(Self::DUPLICATE_CONSTRUCTORS)
    }

    /// Whether [`Self::DUPLICATE_PRIVATE_NAMES`] is enabled.
    #[inline]
    pub const fn duplicate_private_names(&self) -> bool {
        self.contains(Self::DUPLICATE_PRIVATE_NAMES)
    }

    /// Whether [`Self::UNRESOLVED_EXPORTS`] is enabled.
    #[inline]
    pub const fn unresolved_exports(&self) -> bool {
        self.contains(Self::UNRESOLVED_EXPORTS)
    }

    /// Whether [`Self::IMPORT_EXPORT_POSITION`] is enabled.
    #[inline]
    pub const fn import_export_position(&self) -> bool {
        self.contains(Self::IMPORT_EXPORT_POSITION)
    }

    /// Whether [`Self::DUPLICATE_IMPORT_ATTRIBUTES`] is enabled.
    #[inline]
    pub const fn duplicate_import_attributes(&self) -> bool {
        self.contains(Self::DUPLICATE_IMPORT_ATTRIBUTES)
    }

    /// Whether [`Self::DUPLICATE_EXPORTS`] is enabled.
    #[inline]
    pub const fn duplicate_exports(&self) -> bool {
        self.contains(Self::DUPLICATE_EXPORTS)
    }

    /// Whether [`Self::TS_ABSTRACT_METHODS`] is enabled.
    #[inline]
    pub const fn ts_abstract_methods(&self) -> bool {
        self.contains(Self::TS_ABSTRACT_METHODS)
    }

    /// Whether [`Self::TS_TUPLE_REST`] is enabled.
    #[inline]
    pub const fn ts_tuple_rest(&self) -> bool {
        self.contains(Self::TS_TUPLE_REST)
    }

    /// Whether [`Self::TS_TUPLE_OPTIONAL`] is enabled.
    #[inline]
    pub const fn ts_tuple_optional(&self) -> bool {
        self.contains(Self::TS_TUPLE_OPTIONAL)
    }

    /// Whether [`Self::TS_MODULE_DECLARATIONS`] is enabled.
    #[inline]
    pub const fn ts_module_declarations(&self) -> bool {
        self.contains(Self::TS_MODULE_DECLARATIONS)
    }

    /// Whether [`Self::TS_GLOBAL_DECLARATIONS`] is enabled.
    #[inline]
    pub const fn ts_global_declarations(&self) -> bool {
        self.contains(Self::TS_GLOBAL_DECLARATIONS)
    }

    /// Whether [`Self::TS_NON_LOCAL_EXPORTS`] is enabled.
    #[inline]
    pub const fn ts_non_local_exports(&self) -> bool {
        self.contains(Self::TS_NON_LOCAL_EXPORTS)
    }

    /// Whether [`Self::TS_DUPLICATE_EXPORT_ALIASES`] is enabled.
    #[inline]
    pub const fn ts_duplicate_export_aliases(&self) -> bool {
        self.contains(Self::TS_DUPLICATE_EXPORT_ALIASES)
    }

    /// Whether [`Self::TS_EXPORT_ALIAS_CONFLICTS`] is enabled.
    #[inline]
    pub const fn ts_export_alias_conflicts(&self) -> bool {
        self.contains(Self::TS_EXPORT_ALIAS_CONFLICTS)
    }

    /// Whether [`Self::TS_IMPORT_ALIAS_CONFLICTS`] is enabled.
    #[inline]
    pub const fn ts_import_alias_conflicts(&self) -> bool {
        self.contains(Self::TS_IMPORT_ALIAS_CONFLICTS)
    }

    /// Whether [`Self::TS_TYPE_ONLY_IMPORT_ALIASES`] is enabled.
    #[inline]
    pub const fn ts_type_only_import_aliases(&self) -> bool {
        self.contains(Self::TS_TYPE_ONLY_IMPORT_ALIASES)
    }

    /// Whether [`Self::TS_AMBIENT_EXPORT_ASSIGNMENTS`] is enabled.
    #[inline]
    pub const fn ts_ambient_export_assignments(&self) -> bool {
        self.contains(Self::TS_AMBIENT_EXPORT_ASSIGNMENTS)
    }

    /// Whether [`Self::TS_CONTEXTUAL_AWAIT`] is enabled.
    #[inline]
    pub const fn ts_contextual_await(&self) -> bool {
        self.contains(Self::TS_CONTEXTUAL_AWAIT)
    }

    /// Whether [`Self::TS_PARAMETER_PROPERTIES`] is enabled.
    #[inline]
    pub const fn ts_parameter_properties(&self) -> bool {
        self.contains(Self::TS_PARAMETER_PROPERTIES)
    }

    /// Whether [`Self::CLASS_MEMBER_NAMES`] is enabled.
    #[inline]
    pub const fn class_member_names(&self) -> bool {
        self.contains(Self::CLASS_MEMBER_NAMES)
    }

    /// Whether any of the [`Self::IDENTIFIER_REFERENCES`] checks are enabled.
    #[inline]
    pub const fn identifier_references(&self) -> bool {
        self.intersects(Self::IDENTIFIER_REFERENCES)
    }

    /// Whether any of the [`Self::BINDING_IDENTIFIERS`] checks are enabled.
    #[inline]
    pub const fn binding_identifiers(&self) -> bool {
        self.intersects(Self::BINDING_IDENTIFIERS)
    }

    /// Whether any of the [`Self::CLASS_BODY`] checks are enabled.
    #[inline]
    pub const fn class_body(&self) -> bool {
        self.intersects(Self::CLASS_BODY)
    }

    /// Whether any of the [`Self::TS_EXPORT_ALIASES`] checks are enabled.
    #[inline]
    pub const fn ts_export_aliases(&self) -> bool {
        self.intersects(Self::TS_EXPORT_ALIASES)
    }

    /// Whether any of the [`Self::TS_IMPORT_ALIASES`] checks are enabled.
    #[inline]
    pub const fn ts_import_aliases(&self) -> bool {
        self.intersects(Self::TS_IMPORT_ALIASES)
    }

    /// Whether any of the [`Self::TS_ALIASES`] checks are enabled.
    #[inline]
    pub const fn ts_aliases(&self) -> bool {
        self.intersects(Self::TS_ALIASES)
    }
}
