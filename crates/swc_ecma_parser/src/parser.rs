//! ECMAScript and TypeScript parser implementation.
//!
//! Parser based on
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).
//! Modified for SWC Next.

use std::hint::cold_path;
use std::{borrow::Cow, mem};

use bitflags::bitflags;
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use swc_next_allocator::{Allocator, vec::ArenaVec, wtf8::Wtf8Buf};
use swc_next_ecma_ast::*;
use swc_next_ecma_ast::{AstUtf8, AstWtf8};

use crate::{
    TokenFlag,
    config::{NoTokenParserConfig, ParserConfig},
    lexer::{
        HandleTokenMode, Lexer, LexerMode, LexerState, LexicalError, lexical_error_help,
        lexical_error_message,
    },
    number::{parse_binary_literal, parse_decimal_literal, parse_hex_literal, parse_octal_literal},
    token::{Token, TokenTag, flag_mask},
    util::{decode_identifier_escapes, decode_string_escapes},
};

/// Whether comments are collected in a flat, source-ordered list.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CommentMode {
    None,
    #[default]
    Flat,
}

/// The syntactic context surrounding a parsed expression or statement fragment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FragmentContext {
    /// The fragment is parsed at the top level selected by [`Options::source_type`].
    #[default]
    TopLevel,
    /// The fragment is parsed in an ordinary function body.
    Function,
    /// The fragment is parsed in an async function body.
    AsyncFunction,
    /// The fragment is parsed in a generator function body.
    Generator,
    /// The fragment is parsed in an async generator function body.
    AsyncGenerator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Options {
    /// Determines how the source is parsed and evaluated.
    ///
    /// Defaults to module semantics, which enable strict mode.
    pub source_type: SourceType,
    /// Determines which syntax features are enabled.
    ///
    /// Defaults to plain JavaScript.
    pub lang: Lang,
    /// Represents parenthesized expressions and TypeScript types as explicit
    /// parenthesized nodes when true. Otherwise only the inner node is kept.
    pub preserve_parens: bool,
    /// Determines whether comments are collected.
    pub comments: CommentMode,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            source_type: SourceType::Module,
            lang: Lang::Js,
            preserve_parens: true,
            comments: CommentMode::Flat,
        }
    }
}

bitflags! {
    /// The active values of ECMAScript grammatical parameters.
    ///
    /// See <https://tc39.es/ecma262/multipage/notational-conventions.html#sec-grammatical-parameters>.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Context: u8 {
        /// The grammar's `[In]` parameter.
        const IN = 1 << 0;
        /// The grammar's `[Yield]` parameter.
        const YIELD = 1 << 1;
        /// The grammar's `[Await]` parameter.
        const AWAIT = 1 << 2;
        /// The grammar's `[Return]` parameter.
        const RETURN = 1 << 3;
    }
}

/// Enters a derived grammatical context and returns the previous context.
///
/// The modifiers mirror ECMA-262 grammatical arguments: `+` enables a
/// parameter, `~` disables it, and `?` inherits its current value.
macro_rules! enter_grammar_context {
    (@flag In) => {
        Context::IN
    };
    (@flag Yield) => {
        Context::YIELD
    };
    (@flag Await) => {
        Context::AWAIT
    };
    (@flag Return) => {
        Context::RETURN
    };
    (@set $context:ident, + $parameter:ident) => {
        $context.insert(enter_grammar_context!(@flag $parameter));
    };
    (@set $context:ident, ~ $parameter:ident) => {
        $context.remove(enter_grammar_context!(@flag $parameter));
    };
    (@set $context:ident, ? $parameter:ident) => {};
    ($context:expr, $($modifier:tt $parameter:ident),+ $(,)?) => {{
        let context = $context;
        let previous = *context;
        $(enter_grammar_context!(@set context, $modifier $parameter);)+
        previous
    }};
}

impl Default for Context {
    fn default() -> Self {
        Self::IN
    }
}

impl From<SourceType> for Context {
    fn from(source_type: SourceType) -> Self {
        let mut context = Self::default();
        context.set(Self::AWAIT, source_type == SourceType::Module);
        context.set(Self::RETURN, source_type == SourceType::CommonJs);
        context
    }
}

impl Context {
    fn for_fragment(
        source_type: SourceType,
        fragment_context: FragmentContext,
        statement_context: StatementContext,
    ) -> Self {
        let mut context = Self::default();
        context.set(Self::YIELD, fragment_context.allows_yield());
        let allows_await = fragment_context.allows_await()
            || source_type == SourceType::Module
                && statement_context == StatementContext::ModuleItem;
        context.set(Self::AWAIT, allows_await);
        context.set(
            Self::RETURN,
            fragment_context.is_function() || source_type == SourceType::CommonJs,
        );
        context
    }
}

bitflags! {
    /// Checks and permissions active at the current syntactic boundary.
    /// `ModuleDetection` enables source-type detection at program entry.
    /// Boundaries retain or disable these checks; results accumulate separately.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    struct BoundaryContext: u8 {
        /// Feed `await` syntax here into Module/Script detection.
        const DETECT_MODULE_AWAIT = 1 << 0;
        /// Feed wrapper-level `return` and `new.target` into CommonJS detection.
        /// Arrows retain this check; functions and class initializers disable it.
        const DETECT_COMMONJS_SYNTAX = 1 << 1;
        /// The current environment permits `new.target`.
        const ALLOW_NEW_TARGET = 1 << 2;
        /// An await identifier may still become part of an async arrow's parameters.
        const ASYNC_ARROW_HEAD = 1 << 3;
    }
}

impl BoundaryContext {
    fn local() -> Self {
        Self::ALLOW_NEW_TARGET
    }

    fn arrow_head(mut self, is_async: bool) -> Self {
        self.remove(Self::ASYNC_ARROW_HEAD);
        // Non-async arrow parameters inherit the enclosing [Await].
        if is_async {
            self.remove(Self::DETECT_MODULE_AWAIT);
        }
        self
    }

    fn arrow_body(self) -> Self {
        self & (Self::DETECT_COMMONJS_SYNTAX | Self::ALLOW_NEW_TARGET)
    }

    fn ts_container(self) -> Self {
        self & Self::ALLOW_NEW_TARGET
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct TsContext: u8 {
        /// Whether parsing is inside a `declare`-prefixed declaration.
        ///
        /// Nested declarations inherit ambient-context rules such as body-less
        /// functions and initializer-less `const` declarations.
        const AMBIENT = 1 << 0;
        /// Whether `extends` may start another conditional type at this level.
        const DISALLOW_CONDITIONAL_TYPES = 1 << 1;
        /// Whether a speculatively parsed arrow with a return type can commit
        /// without an outer `:`.
        ///
        /// Restricted in conditional consequents so `(x): T => ...` leaves the
        /// ternary `:` intact; reset at independent expression boundaries.
        const ALLOW_ARROW_RETURN_TYPE = 1 << 2;
        /// Whether parsing is inside a TypeScript declaration body that is
        /// outside ECMAScript module grammar.
        const IN_TS_CONTAINER_BODY = 1 << 3;
        /// Type-only binding names ignore Await/Yield; embedded value
        /// expressions still inherit the enclosing grammatical parameters.
        const IN_TYPE_BINDING = 1 << 4;
    }
}

impl Default for TsContext {
    fn default() -> Self {
        Self::ALLOW_ARROW_RETURN_TYPE
    }
}

impl TsContext {
    fn ambient(self) -> bool {
        self.contains(Self::AMBIENT)
    }

    fn disallow_conditional_types(self) -> bool {
        self.contains(Self::DISALLOW_CONDITIONAL_TYPES)
    }

    fn allow_arrow_return_type(self) -> bool {
        self.contains(Self::ALLOW_ARROW_RETURN_TYPE)
    }

    fn in_ts_container_body(self) -> bool {
        self.contains(Self::IN_TS_CONTAINER_BODY)
    }

    fn in_type_binding(self) -> bool {
        self.contains(Self::IN_TYPE_BINDING)
    }
}

struct ParsedBody {
    directives: SubRange<Directive>,
    body: SubRange<Stmt>,
}

/// Module detection and the compatibility of the current Module AST with Script.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ModuleDetection {
    state: ModuleDetectionState,
    /// A module-sensitive `await` may have a different Script interpretation.
    ambiguous_script_different_ast: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ModuleDetectionState {
    #[default]
    Disabled,
    Ambiguous,
    Esm,
    /// Only the first wrapper-dependent use is retained before ESM is established.
    Cjs(CommonJsUse),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommonJsUse {
    Return(Span),
    NewTarget(Span),
}

impl CommonJsUse {
    #[cold]
    fn diagnostic(self) -> Diagnostic<'static> {
        match self {
            Self::Return(span) => Diagnostic {
                severity: Severity::Error,
                message: Cow::Borrowed("'return' statement is only valid inside a function"),
                span,
                help: Some(Cow::Borrowed(
                    "Remove the 'return' statement or wrap the code in a function.",
                )),
                labels: Vec::new(),
            },
            Self::NewTarget(span) => Diagnostic::error(
                "'new.target' is only valid inside functions, class field initializers, or static blocks",
                span,
            ),
        }
    }
}

impl ModuleDetection {
    fn new(source_type: SourceType) -> Self {
        Self {
            state: if source_type == SourceType::Unambiguous {
                ModuleDetectionState::Ambiguous
            } else {
                ModuleDetectionState::Disabled
            },
            ambiguous_script_different_ast: false,
        }
    }

    fn is_enabled(&self) -> bool {
        self.state != ModuleDetectionState::Disabled
    }

    fn program_scope(&self, source_type: SourceType, lang: Lang) -> BoundaryContext {
        let mut context = BoundaryContext::empty();
        context.set(
            BoundaryContext::ALLOW_NEW_TARGET,
            source_type == SourceType::CommonJs,
        );
        if self.is_enabled() {
            context.set(
                BoundaryContext::DETECT_MODULE_AWAIT,
                source_type == SourceType::Module,
            );
            context.set(BoundaryContext::DETECT_COMMONJS_SYNTAX, lang != Lang::Dts);
        }
        context
    }

    fn is_esm(&self) -> bool {
        matches!(self.state, ModuleDetectionState::Esm)
    }

    fn is_commonjs(&self) -> bool {
        matches!(self.state, ModuleDetectionState::Cjs(_))
    }

    fn pending_commonjs_use(&self) -> Option<CommonJsUse> {
        match self.state {
            ModuleDetectionState::Cjs(first) => Some(first),
            _ => None,
        }
    }

    fn parse_action(&self, diagnostics: &ParserDiagnostics) -> UnambiguousParseAction {
        if self.is_esm() {
            return UnambiguousParseAction::KeepModule;
        }
        if self.ambiguous_script_different_ast || diagnostics.has_errors() {
            return UnambiguousParseAction::RetryAsScript;
        }
        if self.is_commonjs() {
            UnambiguousParseAction::RelabelAsCommonJs
        } else {
            UnambiguousParseAction::RelabelAsScript
        }
    }

    fn record_esm(&mut self) -> Option<CommonJsUse> {
        match self.state {
            ModuleDetectionState::Disabled | ModuleDetectionState::Esm => None,
            ModuleDetectionState::Ambiguous => {
                self.state = ModuleDetectionState::Esm;
                None
            }
            ModuleDetectionState::Cjs(first) => {
                self.state = ModuleDetectionState::Esm;
                Some(first)
            }
        }
    }

    fn record_commonjs(&mut self, syntax: CommonJsUse) -> Option<CommonJsUse> {
        match self.state {
            ModuleDetectionState::Ambiguous => {
                self.state = ModuleDetectionState::Cjs(syntax);
                None
            }
            ModuleDetectionState::Cjs(_) => None,
            ModuleDetectionState::Disabled | ModuleDetectionState::Esm => Some(syntax),
        }
    }

    fn record_await(&mut self, ambiguous: bool) -> Option<CommonJsUse> {
        if ambiguous {
            self.ambiguous_script_different_ast = true;
            None
        } else {
            self.record_esm()
        }
    }

    fn speculation_key(&self) -> u8 {
        // Diagnostic positions cannot affect which grammar alternative succeeds.
        let module_detection = match self.state {
            ModuleDetectionState::Disabled => 0,
            ModuleDetectionState::Ambiguous => 1,
            ModuleDetectionState::Cjs(_) => 2,
            ModuleDetectionState::Esm => 3,
        };
        module_detection | u8::from(self.ambiguous_script_different_ast) << 2
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReportOptions {
    pub severity: Severity,
    pub help: Option<Cow<'static, str>>,
    pub labels: Vec<Label<'static>>,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            severity: Severity::Error,
            help: None,
            labels: Vec::new(),
        }
    }
}

/// Reported and deferred diagnostics share one checkpoint truncation boundary.
/// Deferred early errors do not disqualify a speculative grammar interpretation.
#[derive(Default)]
struct ParserDiagnostics {
    entries: Vec<DiagnosticEntry>,
}

enum DiagnosticEntry {
    Reported(Diagnostic<'static>),
    Deferred(Diagnostic<'static>),
}

impl ParserDiagnostics {
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn truncate(&mut self, checkpoint: usize) {
        self.entries.truncate(checkpoint);
    }

    fn push(&mut self, diagnostic: Diagnostic<'static>) {
        self.entries.push(DiagnosticEntry::Reported(diagnostic));
    }

    fn defer(&mut self, diagnostic: Diagnostic<'static>) {
        self.entries.push(DiagnosticEntry::Deferred(diagnostic));
    }

    fn is_clean_since(&self, checkpoint: usize) -> bool {
        self.entries[checkpoint..].iter().all(|entry| match entry {
            DiagnosticEntry::Reported(_) => false,
            DiagnosticEntry::Deferred(_) => true,
        })
    }

    fn has_errors(&self) -> bool {
        !self.is_clean_since(0)
    }

    fn finish(self) -> Vec<Diagnostic<'static>> {
        let mut diagnostics = Vec::new();
        for entry in self.entries {
            let diagnostic = match entry {
                DiagnosticEntry::Reported(diagnostic) | DiagnosticEntry::Deferred(diagnostic) => {
                    diagnostic
                }
            };
            diagnostics.push(diagnostic);
        }
        diagnostics.sort_by_key(|diagnostic| diagnostic.span.start);
        diagnostics
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ScratchBuffer {
    // Allocate on the first append and reuse the capacity after reset.
    items: Vec<NodeId>,
}

impl ScratchBuffer {
    fn begin(&self) -> usize {
        self.items.len()
    }

    fn append(&mut self, index: impl AstNode) {
        if self.items.len() == self.items.capacity() {
            self.items.reserve(16);
        }
        self.items.push(index.node_id());
    }

    fn reset(&mut self, checkpoint: usize) {
        self.items.truncate(checkpoint);
    }
}

#[derive(Clone, Copy)]
enum PropertyOrMethodKind {
    Init,
    Get,
    Set,
}

impl PropertyOrMethodKind {
    fn accessor_arity(self) -> Option<(usize, &'static str, &'static str)> {
        match self {
            Self::Get => Some((
                0,
                "Getter must have no parameters",
                "Remove all parameters from the getter.",
            )),
            Self::Set => Some((
                1,
                "Setter must have exactly one parameter",
                "Setters accept exactly one argument.",
            )),
            Self::Init => None,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PatternContext {
    Binding,
    Assignment,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParseFunctionOpts {
    is_async: bool,
    is_expression: bool,
    is_declare: bool,
    is_default_export: bool,
}

#[inline]
fn enter_function_grammar_context(
    context: &mut Context,
    generator: bool,
    async_function: bool,
) -> Context {
    match (generator, async_function) {
        (false, false) => enter_grammar_context![context, ~Yield, ~Await],
        (true, false) => enter_grammar_context![context, +Yield, ~Await],
        (false, true) => enter_grammar_context![context, ~Yield, +Await],
        (true, true) => enter_grammar_context![context, +Yield, +Await],
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParsedFunction {
    Function(Function),
    TsFunction(TsFunction),
}

impl ParsedFunction {
    fn into_declaration(self) -> Decl {
        match self {
            Self::Function(function) => Decl::Function(function),
            Self::TsFunction(function) => Decl::TsFunction(function),
        }
    }

    fn into_default_export_declaration(self) -> ExportDefaultDeclarationKind {
        match self {
            Self::Function(function) => ExportDefaultDeclarationKind::Function(function),
            Self::TsFunction(function) => ExportDefaultDeclarationKind::TsFunction(function),
        }
    }

    fn into_function_expression(self) -> Function {
        match self {
            Self::Function(function) => function,
            Self::TsFunction(_) => {
                unreachable!("function expressions always have a body")
            }
        }
    }
}

/// The syntactic grammar symbol accepted at a statement position.
///
/// These variants use the specification's nonterminal names.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum StatementContext {
    #[default]
    StatementListItem,
    ModuleItem,
    Statement,
    LabelledItem,
}

impl From<SourceType> for StatementContext {
    fn from(source_type: SourceType) -> Self {
        match source_type {
            SourceType::Script | SourceType::CommonJs => Self::StatementListItem,
            SourceType::Module | SourceType::Unambiguous => Self::ModuleItem,
        }
    }
}

impl StatementContext {
    const fn permits_declaration(self) -> bool {
        matches!(self, Self::StatementListItem | Self::ModuleItem)
    }
}

impl FragmentContext {
    /// Returns the broadest statement grammar which is valid for this
    /// enclosing context.
    fn statement_context(self, source_type: SourceType) -> StatementContext {
        if self.is_function() {
            StatementContext::StatementListItem
        } else {
            source_type.into()
        }
    }

    const fn is_function(self) -> bool {
        !matches!(self, Self::TopLevel)
    }

    const fn allows_yield(self) -> bool {
        matches!(self, Self::Generator | Self::AsyncGenerator)
    }

    const fn allows_await(self) -> bool {
        matches!(self, Self::AsyncFunction | Self::AsyncGenerator)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParseExpressionOpts {
    /// Permit `await`/`yield` identifiers while parsing a TypeScript signature binding.
    allow_type_binding_identifiers: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArrowHeadResult {
    parameters: FormalParameters,
    type_parameters: Option<TsTypeParameterDeclaration>,
    return_type: Option<TsTypeAnnotation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArrowBodyResult {
    body: ArrowFunctionBody,
    is_expression: bool,
}

/// Sparse evidence retained until expression syntax is resolved as a value or pattern.
/// An ordered table owns both candidate spans and their scope boundaries; a consumed
/// candidate stays in place until its scope ends. One journal restores metadata mutations.
/// Node scratch storage is reset at each conversion boundary.
#[derive(Default)]
struct CoverState {
    initialized_names: IndexMap<NodeId, Option<Span>, FxBuildHasher>,
    /// Commas which become errors only if their spread nodes become rest elements.
    trailing_commas: FxHashMap<NodeId, Span>,
    /// Targets whose parentheses were stripped during cover conversion.
    /// The same targets must be rejected if later resolved as bindings.
    parenthesized_targets: FxHashSet<NodeId>,
    changes: Vec<CoverChange>,
    scratch: ScratchBuffer,
}

#[derive(Clone, Copy)]
enum CoverChange {
    InitializedNameInserted(NodeId),
    InitializedNameConsumed(NodeId, Span),
    InitializedNamePopped(NodeId, Option<Span>),
    TrailingComma(NodeId, Option<Span>),
    ParenthesizedTarget(NodeId),
}

impl CoverState {
    fn start_expression(&self) -> usize {
        self.initialized_names.len()
    }

    #[inline]
    fn commit_expression(&mut self, start: usize, diagnostics: &mut ParserDiagnostics) {
        debug_assert!(start <= self.initialized_names.len());
        if start != self.initialized_names.len() {
            self.commit_initialized_names(start, diagnostics);
        }
    }

    fn commit_initialized_names(&mut self, start: usize, diagnostics: &mut ParserDiagnostics) {
        while self.initialized_names.len() > start {
            let (property, span) = self
                .initialized_names
                .pop()
                .expect("the cover-name scope is nonempty");
            if let Some(span) = span {
                diagnostics.defer(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Borrowed(
                        "Shorthand property cannot have a default value in object expression",
                    ),
                    span,
                    help: Some(Cow::Borrowed(
                        "Use '{ a: a = 1 }' syntax or this is only valid in destructuring patterns.",
                    )),
                    labels: Vec::new(),
                });
            }
            self.changes
                .push(CoverChange::InitializedNamePopped(property, span));
        }
    }

    /// Only the program driver retires evidence, after committing a whole statement.
    fn commit_program_statement(&mut self, diagnostics: &mut ParserDiagnostics) {
        self.commit_expression(0, diagnostics);
        self.changes.clear();
        self.trailing_commas.clear();
        self.parenthesized_targets.clear();
    }

    fn record_initialized_name(&mut self, property: NodeId, span: Span) {
        let previous = self.initialized_names.insert(property, Some(span));
        debug_assert!(
            previous.is_none(),
            "a property has only one initialized name"
        );
        self.changes
            .push(CoverChange::InitializedNameInserted(property));
    }

    fn consume_initialized_name(&mut self, property: NodeId) {
        if let Some(span) = self
            .initialized_names
            .get_mut(&property)
            .and_then(Option::take)
        {
            self.changes
                .push(CoverChange::InitializedNameConsumed(property, span));
        }
    }

    fn record_trailing_comma(&mut self, spread: NodeId, comma: Span) {
        let previous = self.trailing_commas.insert(spread, comma);
        debug_assert!(previous.is_none(), "a spread has only one trailing comma");
        self.changes
            .push(CoverChange::TrailingComma(spread, previous));
    }

    fn consume_trailing_comma(&mut self, spread: NodeId) -> Option<Span> {
        let comma = self.trailing_commas.remove(&spread)?;
        self.changes
            .push(CoverChange::TrailingComma(spread, Some(comma)));
        Some(comma)
    }

    fn record_parenthesized_target(&mut self, target: NodeId) {
        if self.parenthesized_targets.insert(target) {
            self.changes.push(CoverChange::ParenthesizedTarget(target));
        }
    }

    fn rewind(&mut self, checkpoint: usize) {
        debug_assert!(checkpoint <= self.changes.len());
        while self.changes.len() > checkpoint {
            match self.changes.pop().expect("the cover journal is nonempty") {
                CoverChange::InitializedNameInserted(property) => {
                    let removed = self.initialized_names.pop();
                    debug_assert_eq!(removed.map(|(key, _)| key), Some(property));
                }
                CoverChange::InitializedNameConsumed(property, span) => {
                    let candidate = self
                        .initialized_names
                        .get_mut(&property)
                        .expect("the consumed candidate belongs to the checkpoint");
                    debug_assert!(candidate.is_none());
                    *candidate = Some(span);
                }
                CoverChange::InitializedNamePopped(property, span) => {
                    let previous = self.initialized_names.insert(property, span);
                    debug_assert!(previous.is_none());
                }
                CoverChange::TrailingComma(spread, previous) => {
                    if let Some(comma) = previous {
                        self.trailing_commas.insert(spread, comma);
                    } else {
                        self.trailing_commas.remove(&spread);
                    }
                }
                CoverChange::ParenthesizedTarget(target) => {
                    let removed = self.parenthesized_targets.remove(&target);
                    debug_assert!(removed, "parenthesis evidence belongs to the rewound parse");
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ArrayCover {
    elements: SubRange<Option<Argument>>,
    start: u32,
    end: u32,
}

#[derive(Clone, Copy, Debug)]
struct ObjectCover {
    properties: SubRange<ObjectPropertyKind>,
    start: u32,
    end: u32,
}

#[derive(Clone, Copy, Debug)]
struct ParenthesizedCover {
    elements: SubRange<Argument>,
    start: u32,
    end: u32,
    has_trailing_comma: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeclaratorCtx {
    Normal,
    Declare,
    ForLoop,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParseClassOpts {
    is_expression: bool,
    is_default_export: bool,
    is_declare: bool,
    is_abstract: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TsDeclarationModifiers {
    declare: bool,
    r#abstract: bool,
    is_const: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClassElementModifiers {
    is_static: bool,
    is_async: bool,
    is_generator: bool,
    is_accessor: bool,
    computed: bool,
    kind: MethodDefinitionKind,
    declare: bool,
    r#abstract: bool,
    r#override: bool,
    readonly: bool,
    accessibility: Accessibility,
}

impl Default for ClassElementModifiers {
    fn default() -> Self {
        Self {
            is_static: false,
            is_async: false,
            is_generator: false,
            is_accessor: false,
            computed: false,
            kind: MethodDefinitionKind::Method,
            declare: false,
            r#abstract: false,
            r#override: false,
            readonly: false,
            accessibility: Accessibility::None,
        }
    }
}

impl ClassElementModifiers {
    fn conflicts(self, tag: TokenTag) -> bool {
        match tag {
            TokenTag::Async | TokenTag::Get | TokenTag::Set => {
                self.is_async || self.kind != MethodDefinitionKind::Method
            }
            TokenTag::Static => self.is_static,
            TokenTag::Accessor => self.is_accessor,
            TokenTag::Declare => self.declare,
            TokenTag::Abstract => self.r#abstract,
            TokenTag::Override => self.r#override,
            TokenTag::Readonly => self.readonly,
            TokenTag::Public | TokenTag::Private | TokenTag::Protected => {
                self.accessibility != Accessibility::None
            }
            _ => unreachable!(),
        }
    }

    fn set(&mut self, tag: TokenTag) {
        match tag {
            TokenTag::Static => self.is_static = true,
            TokenTag::Async => self.is_async = true,
            TokenTag::Accessor => self.is_accessor = true,
            TokenTag::Get => self.kind = MethodDefinitionKind::Get,
            TokenTag::Set => self.kind = MethodDefinitionKind::Set,
            TokenTag::Declare => self.declare = true,
            TokenTag::Abstract => self.r#abstract = true,
            TokenTag::Override => self.r#override = true,
            TokenTag::Readonly => self.readonly = true,
            TokenTag::Public => self.accessibility = Accessibility::Public,
            TokenTag::Private => self.accessibility = Accessibility::Private,
            TokenTag::Protected => self.accessibility = Accessibility::Protected,
            _ => unreachable!(),
        }
    }
}

enum ClassModifierStep {
    Consumed,
    Key(PropertyKey),
    None,
}

#[derive(Clone, Copy)]
struct SpecifierParts {
    property_name: Option<ModuleExportName>,
    name: ModuleExportName,
    name_token: Token,
    kind: ImportOrExportKind,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TypeTailResult {
    NoTypeModifier,
    ConsumedRename,
    KeepOuterRename,
}

/// The grammar role of a name, independent of its AST representation.
/// Invalid reserved words can still be retained as names after diagnosis.
#[derive(Clone, Copy, Eq, PartialEq)]
enum IdentifierRole {
    Name,
    Reference,
    Binding,
    Label,
    /// TypeScript names do not inherit JavaScript's Await/Yield parameters.
    TypeScript,
    /// Type-only bindings retain module-name checks without Await/Yield grammar.
    TypeBinding,
}

impl IdentifierRole {
    fn description(self) -> &'static str {
        match self {
            Self::Name => "an identifier name",
            Self::Reference | Self::TypeScript => "an identifier",
            Self::Binding | Self::TypeBinding => "a binding identifier",
            Self::Label => "a label",
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BinaryTypeKind {
    Union,
    Intersection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TypeParameterContext {
    Declaration,
    Signature,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AngleListKind {
    Arguments,
    Parameters(TypeParameterContext),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AngleClose {
    /// Committed types peel shifts and report missing closing angles.
    Peel,
    /// Speculative types peel shifts without building discarded diagnostics.
    SpeculativePeel,
    /// Expression arguments require a standalone closing angle.
    Exact,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    struct TypeArgumentFailures: u8 {
        const PEEL = 1 << 0;
        /// Includes exact closing and the expression's follow-token check.
        const EXPRESSION = 1 << 1;
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct SpeculationKey {
    position: u32,
    context: u8,
    ts_context: u8,
    boundary_context: BoundaryContext,
    statement_context: u8,
    module_detection: u8,
    lexer_mode: u8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TsSpeculationKey {
    Arrow {
        context: SpeculationKey,
        is_async: bool,
    },
    InferConstraint(SpeculationKey),
}

struct TypeReferenceFrame {
    start: u32,
    name: TsTypeName,
    argument_start: u32,
    scratch_start: usize,
    key: SpeculationKey,
}

enum PrimaryTypeHead {
    Parsed(TsType),
    Reference { start: u32, name: TsTypeName },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AngleList {
    Arguments(TsTypeParameterInstantiation),
    Parameters(TsTypeParameterDeclaration),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeritageKind {
    Interface,
    Class,
}

#[derive(Clone, Copy, Debug)]
enum HeritageClause {
    Interface(SubRange<TsInterfaceHeritage>),
    Class(SubRange<TsClassImplements>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeritageEntry {
    Interface(TsInterfaceHeritage),
    Class(TsClassImplements),
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum JsDocKind {
    Nullable,
    NonNullable,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum JsxElementContext {
    TopLevel,
    Child,
    Attribute,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum JsxExprContext {
    Tag,
}

#[derive(Clone, Copy, Debug)]
struct DefaultExportPart {
    declaration: ExportDefaultDeclarationKind,
    needs_semi: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParameterPropertyModifiers {
    accessibility: Accessibility,
    readonly: bool,
    r#override: bool,
    present: bool,
}

impl Default for ParameterPropertyModifiers {
    fn default() -> Self {
        Self {
            accessibility: Accessibility::None,
            readonly: false,
            r#override: false,
            present: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LhsContext {
    Expression,
    NewCallee,
    Decorator,
}

#[derive(Clone, Copy, Debug)]
struct PostfixExpression {
    expression: Expr,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[derive(Clone, Copy, Debug)]
enum PostfixStep {
    Continue(Expr),
    Finish {
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArrowHead {
    No,
    Yes,
    Maybe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnambiguousParseAction {
    KeepModule,
    RelabelAsCommonJs,
    RelabelAsScript,
    RetryAsScript,
}

impl From<PropertyKind> for PropertyOrMethodKind {
    fn from(kind: PropertyKind) -> Self {
        match kind {
            PropertyKind::Init => Self::Init,
            PropertyKind::Get => Self::Get,
            PropertyKind::Set => Self::Set,
        }
    }
}

impl From<MethodDefinitionKind> for PropertyOrMethodKind {
    fn from(kind: MethodDefinitionKind) -> Self {
        match kind {
            MethodDefinitionKind::Get => Self::Get,
            MethodDefinitionKind::Set => Self::Set,
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => Self::Init,
        }
    }
}

/// Parser state and typed AST writer.
pub struct Parser<'a, C: ParserConfig = NoTokenParserConfig> {
    ast: Ast<'a>,
    source: &'a str,
    source_type: SourceType,
    lang: Lang,
    preserve_parens: bool,
    comment_mode: CommentMode,
    boundary_context: BoundaryContext,
    lexer: Lexer<'a, C>,
    diagnostics: ParserDiagnostics,
    current_token: Token,
    scratch_statements: ScratchBuffer,
    scratch_decorators: ScratchBuffer,
    /// General-purpose scratch buffers used when two extra-data lists are
    /// assembled at the same time.
    scratch_a: ScratchBuffer,
    scratch_b: ScratchBuffer,
    scratch_string: Wtf8Buf,
    export_specifier_tags: Vec<TokenTag>,
    context: Context,
    statement_context: StatementContext,
    ts_context: TsContext,
    module_detection: ModuleDetection,
    cover: CoverState,
    /// TypeScript grammar alternatives rejected in a particular parse context.
    ///
    /// Failures survive [`Checkpoint`] rollback so enclosing speculation can
    /// skip equivalent retries, but a different context may still succeed.
    ts_rejected_speculations: FxHashSet<TsSpeculationKey>,
    /// Span of the first identifier whose validity depends on a pending async arrow head.
    /// Nested covers restore this value when they become separate functions.
    await_identifier_in_async_head: Option<Span>,
    /// Rule failures survive rollback, but are only reused in speculation so
    /// committed type parses still produce their diagnostics and recovery.
    failed_type_arguments: FxHashMap<SpeculationKey, TypeArgumentFailures>,
    type_argument_speculation_depth: u32,
    type_argument_lookahead: TypeArgumentLookahead,
}

pub struct ParseReturn<'a> {
    pub ast: Ast<'a>,
    pub tokens: ArenaVec<'a, Token>,
    pub diagnostics: Vec<Diagnostic<'static>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Checkpoint {
    // Lexer state.
    lexer_cursor: u32,
    lexer_state: LexerState,
    lexer_mode: LexerMode,
    lexer_comments_len: usize,
    lexer_processed_comments: usize,
    lexer_tokens_len: usize,
    // Parser token stream.
    current_token: Token,
    // Append-only AST storage.
    nodes_len: usize,
    extra_len: usize,
    diagnostics_len: usize,
    // Parser flags, copied by value.
    context: Context,
    statement_context: StatementContext,
    ts_context: TsContext,
    module_detection: ModuleDetection,
    cover_checkpoint: usize,
    boundary_context: BoundaryContext,
    await_identifier_in_async_head: Option<Span>,
}

struct Peek<'peek, 'a, C: ParserConfig = NoTokenParserConfig> {
    lexer: &'peek mut Lexer<'a, C>,
    state: LexerState,
    cursor: u32,
    comments_len: usize,
    module_mode_errors_len: usize,
    processed_comments: usize,
    tokens_len: usize,
    active: bool,
}

impl<'peek, 'a, C: ParserConfig> Peek<'peek, 'a, C> {
    fn new(lexer: &'peek mut Lexer<'a, C>) -> Self {
        Self {
            state: lexer.state,
            cursor: lexer.cursor,
            comments_len: lexer.comments.len(),
            module_mode_errors_len: lexer.module_mode_errors.len(),
            processed_comments: lexer.processed_comments,
            tokens_len: lexer.tokens.len(),
            active: true,
            lexer,
        }
    }

    /// Keeps the scanned tokens and their lexer side effects.
    fn commit(mut self) {
        self.active = false;
    }

    fn end(mut self) {
        self.restore();
    }

    fn restore(&mut self) {
        debug_assert!(self.active);
        debug_assert!(self.comments_len <= self.lexer.comments.len());
        self.lexer.state = self.state;
        self.lexer.cursor = self.cursor;
        self.lexer.comments.truncate(self.comments_len);
        self.lexer
            .module_mode_errors
            .truncate(self.module_mode_errors_len);
        self.lexer.processed_comments = self.processed_comments;
        self.lexer.tokens.truncate(self.tokens_len);
        self.active = false;
    }
}

impl<C: ParserConfig> Iterator for Peek<'_, '_, C> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.active);
        let mut token = Token::eof(self.lexer.cursor);
        self.lexer.next_token(&mut token).ok()?;
        Some(token)
    }
}

impl<C: ParserConfig> Drop for Peek<'_, '_, C> {
    fn drop(&mut self) {
        if self.active {
            self.restore();
        }
    }
}

#[derive(Default)]
struct TypeArgumentLookahead {
    /// Memoize lookahead whose final interpretation still belongs to grammar.
    scanned_angles: FxHashSet<u32>,
    /// Normal-goal delimiter proofs are independent of parser context.
    unclosed_angles: FxHashSet<u32>,
}

impl TypeArgumentLookahead {
    /// Classify every angle opening, including those inside balanced groups,
    /// so later speculation reuses the same scan instead of walking its suffix.
    fn can_close<C: ParserConfig>(
        &mut self,
        start: Token,
        peek: &mut Peek<'_, '_, C>,
        jsx: bool,
    ) -> bool {
        if self.cannot_close(start.start()) {
            return false;
        }
        if self.scanned_angles.contains(&start.start()) {
            return true;
        }
        let mut open_angles = vec![start.start()];
        if start.tag() == TokenTag::LeftShift {
            open_angles.push(start.start() + 1);
        }
        // Each group fences its inner angles off from enclosing closers.
        let mut groups: Vec<(Token, usize)> = Vec::new();
        for token in peek.by_ref() {
            let tag = token.tag();
            // These need expression grammar to select the lexical goal.
            if matches!(
                tag,
                TokenTag::Slash | TokenTag::SlashAssign | TokenTag::TemplateHead
            ) {
                return self.inconclusive(&open_angles);
            }
            if is_angle_open(tag) {
                open_angles.push(token.start());
                if tag == TokenTag::LeftShift {
                    open_angles.push(token.start() + 1);
                }
                if jsx && !groups.is_empty() {
                    return self.inconclusive(&open_angles);
                }
                continue;
            }
            if matches!(
                tag,
                TokenTag::LeftParen | TokenTag::LeftBracket | TokenTag::LeftBrace
            ) {
                groups.push((token, open_angles.len()));
                continue;
            }
            if matches!(
                tag,
                TokenTag::RightParen | TokenTag::RightBracket | TokenTag::RightBrace
            ) {
                let Some((open, angle_start)) = groups.pop() else {
                    return self.reject_unclosed(&open_angles);
                };
                let matches = matches!(
                    (open.tag(), tag),
                    (TokenTag::LeftParen, TokenTag::RightParen)
                        | (TokenTag::LeftBracket, TokenTag::RightBracket)
                        | (TokenTag::LeftBrace, TokenTag::RightBrace)
                );
                if !matches {
                    return self.inconclusive(&open_angles);
                }
                self.reject_unclosed(&open_angles[angle_start..]);
                open_angles.truncate(angle_start);
                continue;
            }
            if tag == TokenTag::Eof {
                return self.reject_unclosed(&open_angles);
            }
            let angle_start = groups.last().map_or(0, |(_, start)| *start);
            match tag {
                TokenTag::Semicolon => {
                    self.reject_unclosed(&open_angles[angle_start..]);
                    open_angles.truncate(angle_start);
                    if groups.is_empty() {
                        return false;
                    }
                }
                _ if is_angle_close(tag) => {
                    let closed = match tag {
                        TokenTag::RightShift | TokenTag::RightShiftAssign => 2,
                        TokenTag::UnsignedRightShift | TokenTag::UnsignedRightShiftAssign => 3,
                        _ => 1,
                    };
                    let remaining = angle_start.max(open_angles.len().saturating_sub(closed));
                    for position in open_angles.drain(remaining..) {
                        self.scanned_angles.insert(position);
                    }
                    if remaining == 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        // Lexical errors belong to the actual parser.
        self.inconclusive(&open_angles)
    }

    fn cannot_close(&self, position: u32) -> bool {
        self.unclosed_angles.contains(&position)
    }

    fn reject_unclosed(&mut self, open_angles: &[u32]) -> bool {
        self.unclosed_angles.extend(open_angles.iter().copied());
        false
    }

    fn inconclusive(&mut self, open_angles: &[u32]) -> bool {
        // Scanned positions avoid repeated lookahead; an unclosed proof always
        // takes precedence if a later scan can resolve an ambiguous region.
        self.scanned_angles.extend(open_angles.iter().copied());
        true
    }
}

fn estimate_ast_capacity(source_len: usize) -> (usize, usize) {
    let baseline_nodes = if source_len < 4_000 {
        256.max(source_len / 4)
    } else if source_len < 10_000 {
        source_len / 3
    } else if source_len < 100_000 {
        source_len / 6
    } else {
        source_len / 8
    };

    // Out-of-line AST fields are common in every grammar: lists, optional
    // metadata, and values that do not fit in the inline node payload all use
    // this table. Leave general headroom so large parses do not copy the full
    // table immediately after the initial reservation.
    let estimated_extra = baseline_nodes * 2 / 3;
    // A NodeData growth doubles the allocation and copies every initialized
    // SoA field. A smaller fixed margin absorbs source-density variation
    // without paying for the full doubled block or inspecting source syntax.
    let estimated_nodes = baseline_nodes + baseline_nodes / 2;
    (estimated_nodes, estimated_extra)
}

impl<'a, C: ParserConfig> Parser<'a, C> {
    pub fn init(allocator: &'a Allocator, source: &'a str, options: Options, config: C) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        let mut ast = Ast::new(source, allocator);
        ast.set_source_metadata(options.source_type, options.lang);
        let allow_html_like_comments = options.source_type != SourceType::Module;
        let lexer = Lexer::init(
            ast.allocator(),
            source,
            allow_html_like_comments,
            options.comments == CommentMode::Flat,
            config,
        );
        let initial_token = Token::eof(lexer.state.prev_token_end);
        Self {
            lexer,
            ast,
            source,
            source_type: options.source_type,
            lang: options.lang,
            preserve_parens: options.preserve_parens,
            comment_mode: options.comments,
            boundary_context: BoundaryContext::empty(),
            diagnostics: ParserDiagnostics::default(),
            current_token: initial_token,
            scratch_statements: ScratchBuffer::default(),
            scratch_decorators: ScratchBuffer::default(),
            scratch_a: ScratchBuffer::default(),
            scratch_b: ScratchBuffer::default(),
            scratch_string: Wtf8Buf::new(),
            export_specifier_tags: Vec::new(),
            context: Context::default(),
            statement_context: StatementContext::default(),
            ts_context: {
                let mut context = TsContext::default();
                context.set(TsContext::AMBIENT, options.lang == Lang::Dts);
                context
            },
            module_detection: ModuleDetection::new(options.source_type),
            cover: CoverState::default(),
            ts_rejected_speculations: FxHashSet::default(),
            await_identifier_in_async_head: None,
            failed_type_arguments: FxHashMap::default(),
            type_argument_speculation_depth: 0,
            type_argument_lookahead: TypeArgumentLookahead::default(),
        }
    }

    pub fn parse(mut self) -> ParseReturn<'a> {
        self.ensure_capacity();
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        match self.source_type {
            SourceType::Unambiguous => self.parse_unambiguous_program(),
            source_type => {
                self.enter_program_context(source_type);
                let body = self.parse_program_body();
                self.finish_program(body)
            }
        }
    }

    fn parse_unambiguous_program(mut self) -> ParseReturn<'a> {
        self.enter_program_context(SourceType::Module);
        let body = self.parse_program_body();
        match self.module_detection.parse_action(&self.diagnostics) {
            UnambiguousParseAction::KeepModule => {}
            UnambiguousParseAction::RelabelAsCommonJs => {
                self.set_source_type(SourceType::CommonJs);
            }
            UnambiguousParseAction::RelabelAsScript => self.set_source_type(SourceType::Script),
            UnambiguousParseAction::RetryAsScript => {
                cold_path();
                let (mut parser, body) = self.reparse_unambiguous_program_as(SourceType::Script);
                if body.is_some()
                    && !parser.diagnostics.has_errors()
                    && !parser.module_detection.is_esm()
                {
                    parser.relabel_as_script_or_commonjs();
                    return parser.finish_program(body);
                }
            }
        };
        self.finish_program(body)
    }

    /// Keep Unambiguous's lexical recovery while selecting a definite grammar.
    fn reparse_unambiguous_program_as(
        &self,
        source_type: SourceType,
    ) -> (Self, Option<ParsedBody>) {
        let mut parser = self.restart_unambiguous();
        parser.enter_program_context(source_type);
        let body = parser.parse_program_body();
        (parser, body)
    }

    /// Preserve lexical recovery for both program and expression-fragment retries.
    fn restart_unambiguous(&self) -> Self {
        let options = Options {
            source_type: SourceType::Unambiguous,
            lang: self.lang,
            preserve_parens: self.preserve_parens,
            comments: self.comment_mode,
        };
        let mut parser = Self::init(
            self.ast.allocator(),
            self.source,
            options,
            self.lexer.config.clone(),
        );
        parser.ensure_capacity();
        if parser.advance().is_none() {
            parser.current_token = Token::eof(0);
        }
        parser
    }

    fn enter_program_context(&mut self, source_type: SourceType) {
        self.set_source_type(source_type);
        self.context = Context::from(source_type);
        self.boundary_context = self.module_detection.program_scope(source_type, self.lang);
        self.statement_context = source_type.into();
    }

    fn finish_program(mut self, parsed_body: Option<ParsedBody>) -> ParseReturn<'a> {
        debug_assert!(self.cover.initialized_names.is_empty());
        self.report_module_mode_errors();
        let mut diagnostics = self.diagnostics.finish();
        if parsed_body.is_none() && diagnostics.is_empty() {
            diagnostics.push(Diagnostic::error(
                "Invalid statement",
                self.current_token.span(),
            ));
        }
        // A failed body still needs a program root for callers inspecting diagnostics.
        let ParsedBody { directives, body } = parsed_body.unwrap_or(ParsedBody {
            directives: SubRange::empty(),
            body: SubRange::empty(),
        });

        // Create the root node and set it as the AST's root.
        let end = self.current_token.end();
        let root_span = Span::new(0, end);
        let hashbang = self.lexer.hashbang.map(|hashbang| {
            let start = hashbang.start;
            let end = start + u32::from(hashbang.len);
            let value = self.ast.source_slice(start, end);
            self.ast.hashbang(Span::new(0, end), value)
        });
        let program = self.ast.program(root_span, directives, body, hashbang);
        self.ast.set_root_program(program);

        if !self.preserve_parens {
            strip_parenthesized_nodes(&mut self.ast, &mut self.lexer.comments);
        }

        if self.comment_mode == CommentMode::Flat {
            self.ast.set_attached_comments(self.lexer.comments);
        }

        // Collect tokens
        let mut tokens = self.lexer.tokens;
        if self.lexer.config.tokens()
            && tokens
                .last()
                .is_some_and(|token| token.tag() == TokenTag::Eof)
        {
            let _ = tokens.pop();
        }

        ParseReturn {
            ast: self.ast,
            tokens,
            diagnostics,
        }
    }

    /// Parses the source as exactly one expression in `fragment_context`.
    pub fn parse_expression_fragment(
        mut self,
        fragment_context: FragmentContext,
    ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
        if let Some(hashbang) = self.lexer.hashbang {
            let end = hashbang.start + u32::from(hashbang.len);
            self.report(
                Span::new(0, end),
                "Hashbang comments are only valid at the start of a program",
                ReportOptions::default(),
            );
            return Err(self.diagnostics.finish());
        }

        self.ensure_capacity();
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        match self.source_type {
            SourceType::Unambiguous => self.parse_unambiguous_expression_fragment(fragment_context),
            source_type => {
                self.enter_fragment_context(source_type, fragment_context);
                let expr = self.parse_value_expression(Precedence::Lowest);
                self.finish_expression_fragment(expr)
            }
        }
    }

    fn parse_unambiguous_expression_fragment(
        mut self,
        fragment_context: FragmentContext,
    ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
        self.enter_fragment_context(SourceType::Module, fragment_context);
        let expr = self.parse_value_expression(Precedence::Lowest);
        match self.module_detection.parse_action(&self.diagnostics) {
            UnambiguousParseAction::KeepModule => {}
            UnambiguousParseAction::RelabelAsCommonJs => {
                self.set_source_type(SourceType::CommonJs);
            }
            UnambiguousParseAction::RelabelAsScript => self.set_source_type(SourceType::Script),
            UnambiguousParseAction::RetryAsScript => {
                cold_path();
                let mut parser = self.restart_unambiguous();
                let expr = {
                    parser.enter_fragment_context(SourceType::Script, fragment_context);
                    parser.parse_value_expression(Precedence::Lowest)
                };
                if expr.is_some()
                    && parser.current_token.tag() == TokenTag::Eof
                    && !parser.diagnostics.has_errors()
                    && !parser.module_detection.is_esm()
                {
                    parser.relabel_as_script_or_commonjs();
                    return parser.finish_expression_fragment(expr);
                }
            }
        };
        self.finish_expression_fragment(expr)
    }

    fn enter_fragment_context(
        &mut self,
        source_type: SourceType,
        fragment_context: FragmentContext,
    ) -> StatementContext {
        let statement_context = fragment_context.statement_context(source_type);
        self.set_source_type(source_type);
        self.context = Context::for_fragment(source_type, fragment_context, statement_context);
        self.boundary_context = if fragment_context.is_function() {
            BoundaryContext::local()
        } else {
            self.module_detection.program_scope(source_type, self.lang)
        };
        self.statement_context = statement_context;
        statement_context
    }

    fn finish_expression_fragment(
        mut self,
        expr: Option<Expr>,
    ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
        debug_assert!(self.cover.initialized_names.is_empty());
        if self.current_token.tag() != TokenTag::Eof {
            self.report_expected(
                self.current_token.span(),
                "Expected end of fragment",
                ReportOptions::default(),
            );
        }

        self.report_module_mode_errors();
        let diagnostics = self.diagnostics.finish();
        match expr {
            Some(expr) if diagnostics.is_empty() => {
                if !self.preserve_parens {
                    strip_parenthesized_nodes(&mut self.ast, &mut self.lexer.comments);
                }
                self.ast.set_root_expression(expr);
                if self.comment_mode == CommentMode::Flat {
                    self.ast.set_attached_comments(self.lexer.comments);
                }
                Ok(self.ast)
            }
            _ => Err(diagnostics),
        }
    }

    fn report_module_mode_errors(&mut self) {
        if self.source_type != SourceType::Module {
            return;
        }
        // Both grammar attempts may fail before ESM is established. The retained
        // Module recovery still rejects the first CommonJS-only use.
        if let Some(first) = self.module_detection.pending_commonjs_use() {
            self.diagnostics.defer(first.diagnostic());
        }
        for index in 0..self.lexer.module_mode_errors.len() {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Borrowed("Legacy comments cannot be used in module code"),
                span: self.lexer.module_mode_errors[index],
                help: None,
                labels: Vec::new(),
            });
        }
    }

    fn relabel_as_script_or_commonjs(&mut self) {
        let source_type = if self.module_detection.is_commonjs() {
            SourceType::CommonJs
        } else {
            SourceType::Script
        };
        self.set_source_type(source_type);
    }

    fn set_source_type(&mut self, source_type: SourceType) {
        self.source_type = source_type;
        self.ast.set_source_metadata(source_type, self.lang);
    }

    fn record_module_syntax(&mut self) {
        if let Some(first) = self.module_detection.record_esm() {
            self.diagnostics.defer(first.diagnostic());
        }
    }

    fn check_commonjs_syntax(&mut self, syntax: CommonJsUse) {
        if self
            .boundary_context
            .contains(BoundaryContext::DETECT_COMMONJS_SYNTAX)
        {
            if let Some(syntax) = self.module_detection.record_commonjs(syntax) {
                self.diagnostics.defer(syntax.diagnostic());
            }
            return;
        }
        // `return` changes grammar success; `new.target` is an early error on
        // an otherwise complete expression, including during speculation.
        let diagnostic = syntax.diagnostic();
        match syntax {
            CommonJsUse::Return(_) => self.diagnostics.push(diagnostic),
            CommonJsUse::NewTarget(_) => self.diagnostics.defer(diagnostic),
        }
    }

    fn record_module_declaration(&mut self) {
        if self.statement_context == StatementContext::ModuleItem
            && !self.ts_context.in_ts_container_body()
        {
            self.record_module_syntax();
        }
    }

    /// Restore the scoped grammar and environment even if parsing returns early.
    fn with_scope<T>(
        &mut self,
        boundary_context: BoundaryContext,
        parse: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let context = self.context;
        let ts_context = self.ts_context;
        let previous = mem::replace(&mut self.boundary_context, boundary_context);
        let result = parse(self);
        self.context = context;
        self.ts_context = ts_context;
        self.boundary_context = previous;
        result
    }

    /// Returns the resolved name for an identifier-like token.
    ///
    /// This strips `#` from private identifiers and decodes Unicode escapes.
    #[inline]
    fn identifier_name(&mut self, token: Token) -> AstUtf8 {
        if token.tag() != TokenTag::PrivateIdentifier && !token.is_escaped() {
            return self.ast.source_slice(token.start(), token.end());
        }

        self.identifier_name_slow(token)
    }

    #[cold]
    #[inline(never)]
    fn identifier_name_slow(&mut self, token: Token) -> AstUtf8 {
        let start = token.start() + u32::from(token.tag() == TokenTag::PrivateIdentifier);
        if token.is_escaped() {
            return self.decode_escaped_identifier(start, token.end());
        }

        self.ast.source_slice(start, token.end())
    }

    /// Returns the decoded string value without the surrounding quotes.
    fn string_value(&mut self, token: Token) -> AstWtf8 {
        if token.is_escaped() {
            return self.decode_escaped_string(token.start() + 1, token.end() - 1);
        }
        self.ast
            .source_slice_wtf8(token.start() + 1, token.end() - 1)
    }

    /// Returns the decoded content of a template quasi span.
    fn template_element_raw(&mut self, token: Token, span: Span) -> AstUtf8 {
        // The lexer marks physical CR as escaped. Ordinary source-backed raw
        // values need neither a second scan nor an allocation.
        if !token.is_escaped() {
            return self.ast.source_slice(span.start, span.end);
        }
        let raw = &self.source[span.start as usize..span.end as usize];
        let Some(first_cr) = memchr::memchr(b'\r', raw.as_bytes()) else {
            return self.ast.source_slice(span.start, span.end);
        };
        // Template Raw Value normalizes CR and CRLF even when an invalid
        // escape makes the cooked value undefined in a tagged template.
        let mut normalized = String::with_capacity(raw.len());
        normalized.push_str(&raw[..first_cr]);
        for part in raw[first_cr + 1..].split('\r') {
            normalized.push('\n');
            normalized.push_str(part.strip_prefix('\n').unwrap_or(part));
        }
        self.ast.add_utf8(&normalized)
    }

    fn template_element_value(&mut self, token: Token, span: Span) -> AstWtf8 {
        if !token.is_escaped() {
            return self.ast.source_slice_wtf8(span.start, span.end);
        }
        self.decode_escaped_string(span.start, span.end)
    }

    fn decode_escaped_identifier(&mut self, start: u32, end: u32) -> AstUtf8 {
        let raw = &self.source[start as usize..end as usize];
        let mut buffer = [0u8; 256];
        if let Some(value) = decode_identifier_escapes(raw, &mut buffer) {
            // Safety: the lexer rejects lone surrogates and invalid identifier escapes.
            return self
                .ast
                .add_utf8(unsafe { std::str::from_utf8_unchecked(value) });
        }

        let mut buffer = vec![0u8; raw.len()];
        let value = decode_identifier_escapes(raw, &mut buffer)
            .expect("the lexer validated this escaped identifier");
        // Safety:
        // Source code is guaranteed to be valid UTF-8.
        // Lone surrogates are not valid in identifiers.
        // All of them are rejected by the lexer with `LexicalError::InvalidIdentifierContinue`
        let value = unsafe { std::str::from_utf8_unchecked(value) };
        self.ast.add_utf8(value)
    }

    fn decode_escaped_string(&mut self, start: u32, end: u32) -> AstWtf8 {
        self.scratch_string.clear();
        self.scratch_string
            .reserve(((end - start) as usize).max(64));
        decode_string_escapes(
            &self.source[start as usize..end as usize],
            &mut self.scratch_string,
        );
        self.ast.add_wtf8(&self.scratch_string)
    }

    fn describe_token(&self, token: Token) -> Cow<'a, str> {
        if token.tag() == TokenTag::Eof {
            return Cow::Borrowed("end of file");
        }
        if let Some(text) = token.tag().to_str() {
            Cow::Borrowed(text)
        } else {
            Cow::Borrowed(token.text(self.source))
        }
    }

    fn set_lexer_mode(&mut self, mode: LexerMode) {
        self.lexer.mode = mode;
    }

    fn span_text(&self, span: Span) -> &'a str {
        &self.source[span.start as usize..span.end as usize]
    }

    // field-specific helpers avoid allocating scratch copies while splitting mutable borrows
    fn add_extra_from_scratch_a<N>(&mut self, scratch_checkpoint: usize) -> SubRange<N>
    where
        N: AstNode,
    {
        debug_assert!(scratch_checkpoint <= self.scratch_a.items.len());
        let range = self.ast.add_sub_range(
            self.scratch_a.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(range.len(), self.scratch_a.items.len() - scratch_checkpoint);
        range
    }

    fn add_extra_from_scratch_b<N>(&mut self, scratch_checkpoint: usize) -> SubRange<N>
    where
        N: AstNode,
    {
        debug_assert!(scratch_checkpoint <= self.scratch_b.items.len());
        let range = self.ast.add_sub_range(
            self.scratch_b.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(range.len(), self.scratch_b.items.len() - scratch_checkpoint);
        range
    }

    fn add_extra_from_scratch_cover<N>(&mut self, scratch_checkpoint: usize) -> SubRange<N>
    where
        N: AstNode,
    {
        debug_assert!(scratch_checkpoint <= self.cover.scratch.items.len());
        let range = self.ast.add_sub_range(
            self.cover.scratch.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(
            range.len(),
            self.cover.scratch.items.len() - scratch_checkpoint
        );
        range
    }

    fn add_extra_from_scratch_decorators<N>(&mut self, scratch_checkpoint: usize) -> SubRange<N>
    where
        N: AstNode,
    {
        debug_assert!(scratch_checkpoint <= self.scratch_decorators.items.len());
        let range = self.ast.add_sub_range(
            self.scratch_decorators.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(
            range.len(),
            self.scratch_decorators.items.len() - scratch_checkpoint
        );
        range
    }

    fn add_extra_from_scratch_statements<N>(&mut self, scratch_checkpoint: usize) -> SubRange<N>
    where
        N: AstNode,
    {
        let scratch_end = self.scratch_statements.items.len();
        self.add_extra_from_scratch_statements_range(scratch_checkpoint, scratch_end)
    }

    fn add_extra_from_scratch_statements_range<N>(
        &mut self,
        scratch_start: usize,
        scratch_end: usize,
    ) -> SubRange<N>
    where
        N: AstNode,
    {
        debug_assert!(scratch_start <= scratch_end);
        debug_assert!(scratch_end <= self.scratch_statements.items.len());
        let range = self.ast.add_sub_range(
            self.scratch_statements.items[scratch_start..scratch_end]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(range.len(), scratch_end - scratch_start);
        range
    }

    /// Captures all state needed to commit or discard a speculative parse.
    fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            lexer_cursor: self.lexer.cursor,
            lexer_state: self.lexer.state,
            lexer_mode: self.lexer.mode,
            lexer_comments_len: self.lexer.comments.len(),
            lexer_processed_comments: self.lexer.processed_comments,
            lexer_tokens_len: self.lexer.tokens.len(),
            current_token: self.current_token,
            nodes_len: self.ast.node_count(),
            extra_len: self.ast.extra_data_count(),
            diagnostics_len: self.diagnostics.len(),
            context: self.context,
            statement_context: self.statement_context,
            ts_context: self.ts_context,
            module_detection: self.module_detection,
            cover_checkpoint: self.cover.changes.len(),
            boundary_context: self.boundary_context,
            await_identifier_in_async_head: self.await_identifier_in_async_head,
        }
    }

    /// Recovery diagnostics cannot prove an ambiguous grammar alternative.
    /// Callers with a decisive delimiter may instead commit the recovered AST.
    fn speculation_is_clean(&self, checkpoint: &Checkpoint) -> bool {
        self.diagnostics.is_clean_since(checkpoint.diagnostics_len)
    }

    fn reject_ts_speculation(&mut self, key: TsSpeculationKey) {
        let inserted = self.ts_rejected_speculations.insert(key);
        debug_assert!(inserted, "TypeScript speculation was already rejected");
    }

    /// Restores parser state captured by [`Parser::checkpoint`].
    fn rewind(&mut self, checkpoint: Checkpoint) {
        self.lexer.cursor = checkpoint.lexer_cursor;
        self.lexer.state = checkpoint.lexer_state;
        self.lexer.mode = checkpoint.lexer_mode;
        self.lexer.comments.truncate(checkpoint.lexer_comments_len);
        self.lexer
            .rewind_module_mode_errors(checkpoint.lexer_cursor);
        self.lexer.processed_comments = checkpoint.lexer_processed_comments;
        self.lexer.tokens.truncate(checkpoint.lexer_tokens_len);
        self.current_token = checkpoint.current_token;
        self.ast
            .truncate(checkpoint.nodes_len, checkpoint.extra_len);
        self.diagnostics.truncate(checkpoint.diagnostics_len);
        self.context = checkpoint.context;
        self.statement_context = checkpoint.statement_context;
        self.ts_context = checkpoint.ts_context;
        self.module_detection = checkpoint.module_detection;
        self.cover.rewind(checkpoint.cover_checkpoint);
        self.boundary_context = checkpoint.boundary_context;
        self.await_identifier_in_async_head = checkpoint.await_identifier_in_async_head;
    }

    fn begin_peek(&mut self) -> Peek<'_, 'a, C> {
        Peek::new(&mut self.lexer)
    }

    /// Peeks at the token immediately after `current_token` without advancing.
    ///
    /// For multi-token lookahead, use [`Parser::begin_peek`].
    fn peek_ahead(&mut self) -> Option<Token> {
        let mut peek = self.begin_peek();
        let token = peek.next();
        peek.end();
        token
    }

    fn next_token_may_start_with(&self, matches_ascii: impl FnOnce(u8) -> bool) -> bool {
        let start = self.current_token.end() as usize;
        let source = self.source.as_bytes();
        let Some(&byte) = source.get(start) else {
            return false;
        };
        // Keep full lookahead for trivia and non-ASCII bytes.
        // '/' may start a line or block comment, and '<' may start an HTML comment.
        byte >= 0x80 || matches!(byte, b' ' | b'\t'..=b'\r' | b'/' | b'<') || matches_ascii(byte)
    }

    fn ensure_capacity(&mut self) {
        let (estimated_nodes, estimated_extra) = estimate_ast_capacity(self.source.len());
        self.ast.reserve(estimated_nodes, estimated_extra);
    }

    // ScriptBody :
    //   StatementList[~Yield, ~Await, ~Return]
    //
    // ModuleBody :
    //   ModuleItemList
    //
    // ModuleItem :
    //   StatementListItem[~Yield, +Await, ~Return]
    //
    // FunctionBody[Yield, Await] :
    //   FunctionStatementList[?Yield, ?Await]
    //
    // FunctionStatementList[Yield, Await] :
    //   StatementList[?Yield, ?Await, +Return]?
    //
    // ExpressionStatement[Yield, Await] :
    //   [lookahead ∉ { `{`, `function`, `async` [no LineTerminator here] `function`, `class`, `let` `[` }] Expression[+In, ?Yield, ?Await] `;`
    /// The program driver owns statement commits; nested body parsing never
    /// retires cover evidence that can belong to an enclosing speculation.
    fn parse_program_body(&mut self) -> Option<ParsedBody> {
        let body = self.parse_body_with(None, self.source_type.into(), |parser, statement| {
            parser.record_statement_module_syntax(statement);
            if !parser.cover.changes.is_empty() {
                parser
                    .cover
                    .commit_program_statement(&mut parser.diagnostics);
            }
        });
        // Resolve the failed statement as well before source-type classification.
        self.cover.commit_program_statement(&mut self.diagnostics);
        body
    }

    fn parse_body(
        &mut self,
        terminator: Option<TokenTag>,
        statement_context: StatementContext,
    ) -> Option<ParsedBody> {
        self.parse_body_with(terminator, statement_context, |_, _| {})
    }

    fn parse_body_with(
        &mut self,
        terminator: Option<TokenTag>,
        statement_context: StatementContext,
        mut on_statement: impl FnMut(&mut Self, Stmt),
    ) -> Option<ParsedBody> {
        debug_assert!(statement_context.permits_declaration());
        let statement_context = mem::replace(&mut self.statement_context, statement_context);
        // `parse_body` is shared by ScriptBody, ModuleBody, FunctionBody, and
        // TypeScript module blocks, all of which preserve their initial string
        // literal ExpressionStatements as directives in the AST.
        let mut in_directive_prologue = true;
        let statements_checkpoint = self.scratch_statements.begin();
        let mut directives_end = statements_checkpoint;
        let result = (|| {
            while !self.is_at_body_end(terminator) {
                let is_directive = (|| {
                    if in_directive_prologue && self.current_token.tag() == TokenTag::StringLiteral
                    {
                        // The program or function/static block entry already establishes +In.
                        let expression =
                            self.parse_expr(Precedence::Lowest, ParseExpressionOpts::default())?;
                        if let ExprData::StringLiteral(string_literal) =
                            self.ast.expr_data(expression)
                        {
                            let directive = self
                                .parse_directive(string_literal, string_literal.span(&self.ast))?;
                            self.scratch_statements.append(directive);
                            return Some(true);
                        }
                        let start = expression.span(&self.ast).start;
                        let statement =
                            self.parse_expression_statement_with_expression(start, expression)?;
                        on_statement(self, statement);
                        self.scratch_statements.append(statement);
                        return Some(false);
                    }
                    let statement = self.parse_statement(self.statement_context)?;
                    on_statement(self, statement);
                    self.scratch_statements.append(statement);
                    Some(false)
                })()?;
                if is_directive {
                    directives_end = self.scratch_statements.items.len();
                }
                in_directive_prologue &= is_directive;
            }
            let statements_end = self.scratch_statements.items.len();
            let directives =
                self.add_extra_from_scratch_statements_range(statements_checkpoint, directives_end);
            let body = self.add_extra_from_scratch_statements_range(directives_end, statements_end);
            Some(ParsedBody { directives, body })
        })();
        self.scratch_statements.reset(statements_checkpoint);
        self.statement_context = statement_context;
        result
    }

    fn record_statement_module_syntax(&mut self, statement: Stmt) {
        if statement.is_module_indicator(&self.ast) {
            self.record_module_syntax();
        }
    }

    fn is_at_body_end(&self, terminator: Option<TokenTag>) -> bool {
        self.current_token.tag() == TokenTag::Eof
            || terminator.is_some_and(|tag| self.current_token.tag() == tag)
    }

    fn parse_statement(&mut self, statement_context: StatementContext) -> Option<Stmt> {
        let statement_context = mem::replace(&mut self.statement_context, statement_context);
        let result = self.parse_statement_inner();
        self.statement_context = statement_context;
        result
    }

    #[inline]
    fn parse_statement_inner(&mut self) -> Option<Stmt> {
        if self.lang.is_typescript() && self.is_start_of_ts_declaration()? {
            return self.parse_ts_declaration();
        }
        match self.current_token.tag() {
            TokenTag::Await => self.parse_await_using_or_expression(),
            TokenTag::Var => {
                let start = self.current_token.start();
                self.advance()?;
                self.parse_variable_declaration(VariableKind::Var, false, start)
                    .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)))
            }
            TokenTag::Const => self.parse_const_or_const_enum(),
            TokenTag::Let if !self.current_token.is_escaped() => self.parse_let(),
            TokenTag::Using => self.parse_using_or_expression(),
            TokenTag::LeftBrace => Some(Stmt::BlockStatement(self.parse_block_statement()?)),
            TokenTag::Semicolon => self.parse_empty_statement(),
            TokenTag::If => self.parse_if_statement(),
            TokenTag::While => self.parse_while_statement(),
            TokenTag::Do => self.parse_do_while_statement(),
            TokenTag::Return => self.parse_return_statement(),
            TokenTag::Throw => self.parse_throw_statement(),
            TokenTag::Break => self.parse_break_statement(),
            TokenTag::Continue => self.parse_continue_statement(),
            TokenTag::Debugger => self.parse_debugger_statement(),
            TokenTag::Function => Some(Stmt::Declaration(
                self.parse_function(ParseFunctionOpts::default(), None)?
                    .into_declaration(),
            )),
            TokenTag::Async => self.parse_async_function_or_expression(),
            TokenTag::For => self.parse_for_statement(),
            TokenTag::Class => Some(Stmt::Declaration(Decl::Class(
                self.parse_class(ParseClassOpts::default(), None)?,
            ))),
            TokenTag::At => self.parse_decorated_statement(),
            TokenTag::Switch => self.parse_switch_statement(),
            TokenTag::Try => self.parse_try_statement(),
            TokenTag::With => self.parse_with_statement(),
            TokenTag::Import => self.parse_import_declaration_or_expression(),
            TokenTag::Export => self.parse_export_declaration(),
            _ => self.parse_expression_or_labeled_statement(),
        }
    }

    fn parse_decorated_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::At);
        let (decorators_start, decorators) = self.parse_decorators()?;
        if self.current_token.tag() != TokenTag::Export {
            return Some(Stmt::Declaration(Decl::Class(self.parse_class_decorated(
                ParseClassOpts::default(),
                Some(decorators_start),
                decorators,
            )?)));
        }
        self.parse_export_decorated(decorators)
    }

    fn parse_export_decorated(&mut self, decorators: SubRange<Decorator>) -> Option<Stmt> {
        self.record_module_declaration();
        let export_start = self.current_token.start();
        self.advance()?;
        let is_default = self.eat(TokenTag::Default)?;
        self.parse_decorated_export_class(export_start, is_default, decorators)
    }

    fn parse_decorated_export_class(
        &mut self,
        export_start: u32,
        mut is_default: bool,
        mut decorators: SubRange<Decorator>,
    ) -> Option<Stmt> {
        let before_export = !decorators.is_empty();
        let mut class_start = None;
        while self.current_token.tag() == TokenTag::At {
            let (start, following) = self.parse_decorators()?;
            class_start = Some(start);
            if before_export {
                self.report(
                    Span::new(start, self.lexer.state.prev_token_end),
                    "Decorators cannot appear both before and after export",
                    ReportOptions::default(),
                );
            }
            if decorators.is_empty() {
                decorators = following;
            } else {
                let checkpoint = self.scratch_decorators.begin();
                for id in decorators.iter() {
                    self.scratch_decorators
                        .append(self.ast.get_node_in_sub_range(id));
                }
                for id in following.iter() {
                    self.scratch_decorators
                        .append(self.ast.get_node_in_sub_range(id));
                }
                decorators = self.add_extra_from_scratch_decorators(checkpoint);
                self.scratch_decorators.reset(checkpoint);
            }
            if !is_default && self.eat(TokenTag::Default)? {
                self.report(
                    Span::new(start, self.lexer.state.prev_token_end),
                    "Decorators must follow the default keyword",
                    ReportOptions::default(),
                );
                is_default = true;
                class_start = None;
            }
        }
        let class = self.parse_class_decorated(
            ParseClassOpts {
                is_default_export: is_default,
                ..ParseClassOpts::default()
            },
            class_start,
            decorators,
        )?;
        let end = class.span(&self.ast).end;
        if is_default {
            Some(self.ast.stmt_export_default_declaration(
                Span::new(export_start, end),
                ExportDefaultDeclarationKind::Class(class),
            ))
        } else {
            let specifiers = SubRange::empty();
            let attributes = SubRange::empty();
            Some(self.ast.stmt_export_named_declaration(
                Span::new(export_start, end),
                Some(Decl::Class(class)),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ))
        }
    }

    fn parse_let(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Let);
        if !self.is_let_identifier()? {
            let start = self.current_token.start();
            self.advance()?;
            return self
                .parse_variable_declaration(VariableKind::Let, false, start)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        }
        self.parse_expression_or_labeled_statement()
    }

    fn parse_using_or_expression(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Using);
        if !self.is_using_identifier()? {
            let start = self.current_token.start();
            self.advance()?;
            return self
                .parse_variable_declaration(VariableKind::Using, false, start)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        }
        self.parse_expression_or_labeled_statement()
    }

    fn parse_await_using_or_expression(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Await);
        let await_span = self.current_token.span();
        let next = self.peek_ahead()?;
        if next.tag() == TokenTag::Using && !next.has_line_terminator_before() {
            let start = await_span.start;
            self.advance()?;
            if !self.is_using_identifier()? {
                self.report_await_using_if_disallowed(await_span);
                self.advance()?;
                return self
                    .parse_variable_declaration(VariableKind::AwaitUsing, false, start)
                    .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
            }
            self.report_await_using_if_disallowed(await_span);
            return self.parse_await_expression_statement(start);
        }
        self.parse_expression_or_labeled_statement()
    }

    fn report_await_using_if_disallowed(&mut self, span: Span) {
        if self.context.contains(Context::AWAIT) {
            if self
                .boundary_context
                .contains(BoundaryContext::DETECT_MODULE_AWAIT)
            {
                self.record_module_syntax();
            }
            return;
        }
        self.report(
            span,
            "'await using' is only valid in async functions or modules",
            ReportOptions::default(),
        );
    }

    // ExpressionStatement[Yield, Await] :
    //   [lookahead ∉ { `{`, `function`, `async` [no LineTerminator here] `function`, `class`, `let` `[` }] Expression[+In, ?Yield, ?Await] `;`
    fn parse_await_expression_statement(&mut self, start: u32) -> Option<Stmt> {
        // The program or function/static block entry already establishes +In.
        let expression = self.parse_await_expression(start)?;
        let expression = self.parse_expr_from_left(expression, Precedence::Comma)?;
        self.parse_expression_statement_with_expression(start, expression)
    }

    fn parse_const_or_const_enum(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Const);
        if self.lang.is_typescript() {
            let next = self.peek_ahead()?;
            if is_const_enum_head(next) {
                return self.parse_ts_declaration();
            }
        }
        let start = self.current_token.start();
        self.advance()?;
        self.parse_variable_declaration(VariableKind::Const, false, start)
            .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)))
    }

    fn parse_async_function_or_expression(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Async);
        let next = self.peek_ahead()?;
        if next.tag() == TokenTag::Function && !next.has_line_terminator_before() {
            let start = self.current_token.start();
            self.advance()?;
            return self
                .parse_function(
                    ParseFunctionOpts {
                        is_async: true,
                        ..ParseFunctionOpts::default()
                    },
                    Some(start),
                )
                .map(|function| Stmt::Declaration(function.into_declaration()));
        }
        self.parse_expression_or_labeled_statement()
    }

    fn parse_import_declaration_or_expression(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Import);
        let import = self.current_token;
        let mut peek = self.begin_peek();
        // Keep the previous-token boundary identical to `advance`.
        peek.lexer.state.prev_token_end = import.end();
        let next = peek.next()?;
        if matches!(next.tag(), TokenTag::LeftParen | TokenTag::Dot) {
            peek.end();
            return self.parse_expression_statement();
        }
        peek.commit();
        self.check_escaped_keyword();
        self.current_token = next;
        self.parse_import_declaration_from(import.start())
    }

    fn is_let_identifier(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Let);
        let next = self.peek_ahead()?;

        // Lexical declarations cannot be single statements. In this context,
        // `let` therefore starts an ExpressionStatement unless the
        // ExpressionStatement lookahead restriction `let [` applies.
        //
        // This also lets ASI terminate `let` before a binding-looking token:
        // `if (false) let\nvalue = 1`.
        if !self.statement_context.permits_declaration() && next.tag() != TokenTag::LeftBracket {
            return Some(true);
        }

        Some(!can_start_binding_pattern(next.tag()))
    }

    fn is_using_identifier(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Using);
        let next = self.peek_ahead()?;
        if can_start_using_binding(next.tag()) && !self.can_insert_implicit_semicolon(next) {
            return Some(false);
        }
        Some(true)
    }

    fn is_await_using_declaration_ahead(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Await);
        if !self.context.contains(Context::AWAIT) {
            return Some(false);
        }

        let mut peek = self.begin_peek();
        let result = (|| {
            let using = peek.next()?;
            if using.tag() != TokenTag::Using || using.has_line_terminator_before() {
                return Some(false);
            }
            let binding = peek.next()?;
            Some(!binding.has_line_terminator_before() && can_start_using_binding(binding.tag()))
        })();
        peek.end();
        result
    }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        let start = self.current_token.start();
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start block statement",
            Some("Block statements must be enclosed in braces: { ... }"),
        ) {
            return None;
        }
        let checkpoint = self.scratch_statements.begin();
        let result = (|| {
            while !matches!(
                self.current_token.tag(),
                TokenTag::RightBrace | TokenTag::Eof
            ) {
                let statement = self.parse_statement(StatementContext::StatementListItem)?;
                self.scratch_statements.append(statement);
            }
            let end = self.current_token.end();
            if !self.expect(
                TokenTag::RightBrace,
                "Expected '}' to close block statement",
                Some(
                    "Add a closing brace '}' to complete the block statement, or check for unbalanced braces inside.",
                ),
            ) {
                return None;
            }
            let body = self.add_extra_from_scratch_statements(checkpoint);
            Some(self.ast.block_statement(Span::new(start, end), body))
        })();
        self.scratch_statements.reset(checkpoint);
        result
    }

    fn parse_empty_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Semicolon);
        let span = self.current_token.span();
        self.advance()?;
        Some(self.ast.stmt_empty_statement(span))
    }

    // IfStatement[Yield, Await, Return] :
    //   `if` `(` Expression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return] `else` Statement[?Yield, ?Await, ?Return]
    //   `if` `(` Expression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return] [lookahead != `else`]
    fn parse_if_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::If);
        let start = self.current_token.start();
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'if'", None) {
            return None;
        }
        // The program or function/static block entry already establishes +In.
        let test = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after if condition",
            None,
        ) {
            return None;
        }
        let consequent = self.parse_statement(StatementContext::Statement)?;
        let alternate = if self.eat(TokenTag::Else)? {
            Some(self.parse_statement(StatementContext::Statement)?)
        } else {
            None
        };
        let end = alternate.map_or_else(
            || consequent.span(&self.ast).end,
            |stmt| stmt.span(&self.ast).end,
        );
        Some(
            self.ast
                .stmt_if_statement(Span::new(start, end), test, consequent, alternate),
        )
    }

    // WhileStatement[Yield, Await, Return] :
    //   `while` `(` Expression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    fn parse_while_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::While);
        let start = self.current_token.start();
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'while'", None) {
            return None;
        }
        // The program or function/static block entry already establishes +In.
        let test = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after while condition",
            None,
        ) {
            return None;
        }
        let body = self.parse_statement(StatementContext::Statement)?;
        let end = body.span(&self.ast).end;
        Some(
            self.ast
                .stmt_while_statement(Span::new(start, end), test, body),
        )
    }

    // DoWhileStatement[Yield, Await, Return] :
    //   `do` Statement[?Yield, ?Await, ?Return] `while` `(` Expression[+In, ?Yield, ?Await] `)` `;`
    fn parse_do_while_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Do);
        let start = self.current_token.start();
        self.advance()?;
        let body = self.parse_statement(StatementContext::Statement)?;
        if !self.expect(
            TokenTag::While,
            "Expected 'while' after do statement body",
            None,
        ) {
            return None;
        }
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'while'", None) {
            return None;
        }
        // The program or function/static block entry already establishes +In.
        let test = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after while condition",
            None,
        ) {
            return None;
        }
        self.eat(TokenTag::Semicolon)?;
        let end = self.lexer.state.prev_token_end;
        Some(
            self.ast
                .stmt_do_while_statement(Span::new(start, end), body, test),
        )
    }

    // ReturnStatement[Yield, Await] :
    //   `return` `;`
    //   `return` [no LineTerminator here] Expression[+In, ?Yield, ?Await] `;`
    fn parse_return_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Return);
        let start = self.current_token.start();
        self.advance()?;
        if !self.context.contains(Context::RETURN) {
            let syntax = CommonJsUse::Return(Span::new(start, self.lexer.state.prev_token_end));
            self.check_commonjs_syntax(syntax);
        }
        let argument = if self.current_token.has_line_terminator_before()
            || matches!(
                self.current_token.tag(),
                TokenTag::Semicolon | TokenTag::RightBrace | TokenTag::Eof
            ) {
            None
        } else {
            // The program or function/static block entry already establishes +In.
            Some(self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?)
        };
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_return_statement(Span::new(start, end), argument),
        )
    }

    // ThrowStatement[Yield, Await] :
    //   `throw` [no LineTerminator here] Expression[+In, ?Yield, ?Await] `;`
    fn parse_throw_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Throw);
        let start = self.current_token.start();
        self.advance()?;
        if self.current_token.has_line_terminator_before() {
            self.report(
                self.current_token.span(),
                "Illegal newline after throw",
                ReportOptions::default(),
            );
        }
        // The program or function/static block entry already establishes +In.
        let argument = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_throw_statement(Span::new(start, end), argument),
        )
    }

    fn parse_break_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Break);
        let start = self.current_token.start();
        self.advance()?;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag().is_identifier_like()
        {
            None
        } else {
            Some(self.parse_label_identifier()?)
        };
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_break_statement(Span::new(start, end), label))
    }

    fn parse_continue_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Continue);
        let start = self.current_token.start();
        self.advance()?;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag().is_identifier_like()
        {
            None
        } else {
            Some(self.parse_label_identifier()?)
        };
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_continue_statement(Span::new(start, end), label),
        )
    }

    fn parse_debugger_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Debugger);
        let start = self.current_token.start();
        self.advance()?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_debugger_statement(Span::new(start, end)))
    }

    // WithStatement[Yield, Await, Return] :
    //   `with` `(` Expression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    fn parse_with_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::With);
        let start = self.current_token.start();
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'with'", None) {
            return None;
        }
        // The program or function/static block entry already establishes +In.
        let object = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after with expression",
            None,
        ) {
            return None;
        }
        let body = self.parse_statement(StatementContext::Statement)?;
        let end = body.span(&self.ast).end;
        Some(
            self.ast
                .stmt_with_statement(Span::new(start, end), object, body),
        )
    }

    // ExpressionStatement[Yield, Await] :
    //   [lookahead ∉ { `{`, `function`, `async` [no LineTerminator here] `function`, `class`, `let` `[` }] Expression[+In, ?Yield, ?Await] `;`
    fn parse_expression_or_labeled_statement(&mut self) -> Option<Stmt> {
        // The program or function/static block entry already establishes +In.
        let expression = self.parse_expr(Precedence::Lowest, ParseExpressionOpts::default())?;
        if self.ast.node_kind(expression.node_id()) == NodeKind::IdentifierReference
            && self.current_token.tag() == TokenTag::Colon
        {
            return self.parse_labeled_statement(expression);
        }
        self.parse_expression_statement_with_expression(
            expression.span(&self.ast).start,
            expression,
        )
    }

    // ExpressionStatement[Yield, Await] :
    //   [lookahead ∉ { `{`, `function`, `async` [no LineTerminator here] `function`, `class`, `let` `[` }] Expression[+In, ?Yield, ?Await] `;`
    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        let start = self.current_token.start();
        // The program or function/static block entry already establishes +In.
        let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        self.parse_expression_statement_with_expression(start, expression)
    }

    fn parse_expression_statement_with_expression(
        &mut self,
        start: u32,
        expression: Expr,
    ) -> Option<Stmt> {
        let end = self.lexer.state.prev_token_end;
        let next = self.current_token;
        let end = self.eat_semicolon(end).or_else(|| {
            // A rejected continuation can start a new statement in a statement
            // list. Keep single-statement bodies intact, including an if's else.
            (self.statement_context.permits_declaration()
                && Self::infix_precedence(next, self.lang.is_typescript())
                    > max_left_precedence(expression, &self.ast)
                && is_start_of_expression(next.tag())
                && !next.tag().is_unconditionally_reserved())
            .then_some(end)
        })?;
        Some(
            self.ast
                .stmt_expression_statement(Span::new(start, end), expression),
        )
    }

    fn parse_labeled_statement(&mut self, identifier: Expr) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Colon);
        let ExprData::IdentifierReference(identifier) = self.ast.expr_data(identifier) else {
            unreachable!("only an identifier reference can start a labeled statement")
        };
        let identifier_span = identifier.span(&self.ast);
        let label = self
            .ast
            .label_identifier(identifier_span, identifier.name(&self.ast));
        self.advance()?;
        let body = self.parse_statement(StatementContext::LabelledItem)?;
        let end = body.span(&self.ast).end;
        Some(
            self.ast
                .stmt_labeled_statement(Span::new(identifier_span.start, end), label, body),
        )
    }

    fn parse_directive(&mut self, string_literal: StringLiteral, span: Span) -> Option<Directive> {
        let raw_value = self.ast.source_slice(span.start + 1, span.end - 1);
        let end = self.eat_semicolon(span.end)?;
        Some(
            self.ast
                .directive(Span::new(span.start, end), string_literal, raw_value),
        )
    }

    // VariableStatement[Yield, Await] :
    //   `var` VariableDeclarationList[+In, ?Yield, ?Await] `;`
    //
    // Declaration[Yield, Await] :
    //   HoistableDeclaration[?Yield, ?Await, ~Default]
    //   ClassDeclaration[?Yield, ?Await, ~Default]
    //   LexicalDeclaration[+In, ?Yield, ?Await]
    fn parse_variable_declaration(
        &mut self,
        kind: VariableKind,
        is_declare: bool,
        declaration_start: u32,
    ) -> Option<VariableDeclaration> {
        let context = if is_declare || self.ts_context.ambient() {
            DeclaratorCtx::Declare
        } else {
            DeclaratorCtx::Normal
        };
        // The program or function/static block entry already establishes +In.
        let declarators = self.parse_variable_declarators(kind, context)?;
        let declaration_end = self.lexer.state.prev_token_end;
        let end = self.eat_semicolon(declaration_end)?;
        if !self.statement_context.permits_declaration()
            && matches!(
                kind,
                VariableKind::Let
                    | VariableKind::Const
                    | VariableKind::Using
                    | VariableKind::AwaitUsing
            )
        {
            self.report(
                Span::new(declaration_start, end),
                "Lexical declaration cannot appear in a single-statement context",
                ReportOptions {
                    help: Some(Cow::Borrowed("Wrap this declaration in a block statement")),
                    ..ReportOptions::default()
                },
            );
        }
        Some(self.ast.variable_declaration(
            Span::new(declaration_start, end),
            kind,
            declarators,
            is_declare,
        ))
    }

    fn parse_variable_declarators(
        &mut self,
        kind: VariableKind,
        context: DeclaratorCtx,
    ) -> Option<SubRange<VariableDeclarator>> {
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            let first = self.parse_variable_declarator(kind, context)?;
            self.scratch_a.append(first);
            while self.eat(TokenTag::Comma)? {
                if !can_start_binding_pattern(self.current_token.tag()) {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected a variable declarator after ','",
                        ReportOptions::default(),
                    );
                    break;
                }
                let declarator = self.parse_variable_declarator(kind, context)?;
                self.scratch_a.append(declarator);
            }
            Some(self.add_extra_from_scratch_a(checkpoint))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    #[inline]
    fn parse_variable_kind(&mut self) -> Option<VariableKind> {
        let tag = self.current_token.tag();
        self.advance()?;
        match tag {
            TokenTag::Var => Some(VariableKind::Var),
            TokenTag::Let => Some(VariableKind::Let),
            TokenTag::Const => Some(VariableKind::Const),
            _ => None,
        }
    }

    // VariableDeclaration[In, Yield, Await] :
    //   BindingIdentifier[?Yield, ?Await] Initializer[?In, ?Yield, ?Await]?
    //   BindingPattern[?Yield, ?Await] Initializer[?In, ?Yield, ?Await]
    //
    // LexicalBinding[In, Yield, Await, Pattern] :
    //   BindingIdentifier[?Yield, ?Await] Initializer[?In, ?Yield, ?Await]?
    //   [+Pattern] BindingPattern[?Yield, ?Await] Initializer[?In, ?Yield, ?Await]
    fn parse_variable_declarator(
        &mut self,
        kind: VariableKind,
        context: DeclaratorCtx,
    ) -> Option<VariableDeclarator> {
        let start = self.current_token.start();
        let id = self.parse_binding_pattern()?;
        let id_span = id.span(&self.ast);
        let is_using = matches!(kind, VariableKind::Using | VariableKind::AwaitUsing);
        let destructuring = self.is_destructuring_pattern(id);
        if is_using && destructuring {
            self.report(
                id_span,
                "Using declaration cannot have destructuring patterns.",
                ReportOptions::default(),
            );
        }
        let definite = self.lang.is_typescript()
            && !self.current_token.has_line_terminator_before()
            && self.eat(TokenTag::LogicalNot)?;
        let mut end = if definite {
            self.lexer.state.prev_token_end
        } else {
            id_span.end
        };
        if definite && (destructuring || context == DeclaratorCtx::ForLoop) {
            self.report(
                Span::new(end - 1, end),
                "A definite assignment assertion is not permitted in this context",
                ReportOptions::default(),
            );
        }
        let type_annotation =
            if self.lang.is_typescript() && self.current_token.tag() == TokenTag::Colon {
                let annotation = self.parse_type_annotation()?;
                end = annotation.span(&self.ast).end;
                Some(annotation)
            } else {
                None
            };
        let init = if self.eat(TokenTag::Assign)? {
            let expression =
                self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
            end = expression.span(&self.ast).end;
            Some(expression)
        } else {
            None
        };
        if init.is_none() && context == DeclaratorCtx::Normal {
            if destructuring {
                self.report(
                    id_span,
                    "Destructuring declaration must have an initializer",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Add '= value' to provide the object or array to destructure from.",
                        )),
                        ..ReportOptions::default()
                    },
                );
            } else if kind == VariableKind::Const {
                self.report(
                    id_span,
                    "'const' declarations must be initialized",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Add '= value' to initialize the constant, or use 'let' if you need to assign it later.",
                        )),
                        ..ReportOptions::default()
                    },
                );
            } else if is_using {
                self.report(
                    id_span,
                    Cow::Owned(format!("'{}' declarations must be initialized", kind.as_str())),
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Disposable resources require an initial value that implements the dispose protocol.",
                        )),
                        ..ReportOptions::default()
                    },
                );
            }
        }
        Some(self.ast.variable_declarator(
            Span::new(start, end),
            id,
            type_annotation,
            init,
            definite,
        ))
    }

    fn parse_binding_pattern(&mut self) -> Option<BindingPattern> {
        self.parse_formal_parameter_pattern().map(Into::into)
    }

    #[inline(always)]
    fn parse_formal_parameter_pattern(&mut self) -> Option<FormalParameterPattern> {
        if self.current_token.tag().is_identifier_like() {
            return self
                .parse_binding_identifier()
                .map(FormalParameterPattern::BindingIdentifier);
        }
        self.parse_destructuring_parameter_pattern()
    }

    fn parse_destructuring_parameter_pattern(&mut self) -> Option<FormalParameterPattern> {
        match self.current_token.tag() {
            TokenTag::LeftBracket => self
                .parse_array_pattern()
                .map(FormalParameterPattern::ArrayPattern),
            TokenTag::LeftBrace => self
                .parse_object_pattern()
                .map(FormalParameterPattern::ObjectPattern),
            _ => {
                let found = self.describe_token(self.current_token);
                self.report(
                    self.current_token.span(),
                    Cow::Owned(format!(
                        "Unexpected token '{}' in binding pattern",
                        found.as_ref()
                    )),
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Expected an identifier, array pattern ([a, b]), or object pattern ({a, b}).",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
        }
    }

    fn parse_array_pattern(&mut self) -> Option<ArrayPattern> {
        self.with_resolved_cover(|parser| {
            let cover = parser.parse_array_cover()?;
            parser.array_cover_to_pattern(cover, PatternContext::Binding)
        })
    }

    fn parse_object_pattern(&mut self) -> Option<ObjectPattern> {
        self.with_resolved_cover(|parser| {
            let cover = parser.parse_object_cover()?;
            parser.object_cover_to_pattern(cover, PatternContext::Binding)
        })
    }

    fn parse_function(
        &mut self,
        opts: ParseFunctionOpts,
        start_from_param: Option<u32>,
    ) -> Option<ParsedFunction> {
        debug_assert!(start_from_param.is_some() || self.current_token.tag() == TokenTag::Function);
        let start = start_from_param.unwrap_or(self.current_token.start());
        if !self.expect(TokenTag::Function, "Expected 'function' keyword", None) {
            return None;
        }
        let generator = self.eat(TokenTag::Star)?;
        let is_function_expression = opts.is_expression && !opts.is_default_export;
        let tail = self.with_scope(self.boundary_context, |parser| {
            if is_function_expression {
                parser
                    .boundary_context
                    .remove(BoundaryContext::ASYNC_ARROW_HEAD);
            }
            let context = if is_function_expression {
                enter_function_grammar_context(&mut parser.context, generator, opts.is_async)
            } else {
                enter_grammar_context![&mut parser.context, ?Yield, ?Await]
            };
            let id = if parser.current_token.tag().is_identifier_like() {
                parser.parse_binding_identifier().map(Some)
            } else {
                if !opts.is_expression && !opts.is_default_export {
                    parser.report(
                        parser.current_token.span(),
                        "Function declaration requires a name",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Add a name after 'function', e.g. 'function myFunc() {}'.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                }
                Some(None)
            };
            parser.context = context;
            let id = id?;
            parser.boundary_context = BoundaryContext::local();
            let type_parameters = if parser.lang.is_typescript()
                && parser.current_token.tag() == TokenTag::LessThan
            {
                Some(
                    parser.parse_type_parameters(
                        TypeParameterContext::Declaration,
                        AngleClose::Peel,
                    )?,
                )
            } else {
                None
            };
            enter_function_grammar_context(&mut parser.context, generator, opts.is_async);
            let (this_param, params) =
                parser.parse_formal_parameters(FormalParameterKind::FormalParameters, false)?;
            let return_type =
                if parser.lang.is_typescript() && parser.current_token.tag() == TokenTag::Colon {
                    Some(parser.parse_return_type_annotation()?)
                } else {
                    None
                };
            let params_end = params.span(&parser.ast).end;
            let return_type_end =
                return_type.map_or(params_end, |return_type| return_type.span(&parser.ast).end);
            let ambient_declaration =
                (opts.is_declare || parser.ts_context.ambient()) && !is_function_expression;
            if ambient_declaration && parser.current_token.tag() == TokenTag::LeftBrace {
                parser.report(
                    parser.current_token.span(),
                    "An implementation cannot be declared in ambient contexts",
                    ReportOptions::default(),
                );
            }
            let has_body = !parser.lang.is_typescript()
                || is_function_expression
                || parser.current_token.tag() == TokenTag::LeftBrace;
            let body = if has_body {
                Some(parser.parse_function_body()?)
            } else {
                None
            };
            let end = if let Some(body) = body {
                body.span(&parser.ast).end
            } else {
                parser.eat_semicolon(return_type_end)?
            };
            // Consume an invalid ambient body before dropping it, so its closing
            // brace cannot be mistaken for the surrounding namespace/module's.
            let body = if ambient_declaration { None } else { body };
            Some((
                id,
                this_param,
                params,
                body,
                type_parameters,
                return_type,
                params_end,
                end,
            ))
        });
        let (id, this_param, params, body, type_parameters, return_type, params_end, end) = tail?;
        // Only declarations are restricted here. A function expression is a valid
        // operand in any expression position, for example:
        //   if (a) c(function* () {});
        // even though the surrounding statement context does not permit declarations.
        if !is_function_expression && !self.statement_context.permits_declaration() {
            if opts.is_async {
                self.report(
                    self.current_token.span(),
                    "Async functions can only be declared at the top level or inside a block",
                    ReportOptions::default(),
                );
            }
            if generator {
                self.report(
                    Span::new(start, params_end),
                    "Generators can only be declared at the top level or inside a block",
                    ReportOptions::default(),
                );
            }
        }
        let span = Span::new(start, end);
        if let Some(body) = body {
            let function_type = if is_function_expression {
                FunctionType::FunctionExpression
            } else {
                FunctionType::FunctionDeclaration
            };
            let function = self.ast.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                opts.is_async,
            );
            Some(ParsedFunction::Function(function))
        } else {
            let function = self.ast.ts_function(
                span,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                generator,
                opts.is_async,
                opts.is_declare,
            );
            Some(ParsedFunction::TsFunction(function))
        }
    }

    fn parse_class(
        &mut self,
        opts: ParseClassOpts,
        start_from_param: Option<u32>,
    ) -> Option<Class> {
        let decorators = SubRange::empty();
        self.parse_class_decorated(opts, start_from_param, decorators)
    }

    fn parse_class_decorated(
        &mut self,
        opts: ParseClassOpts,
        start_from_param: Option<u32>,
        decorators: SubRange<Decorator>,
    ) -> Option<Class> {
        let start = start_from_param.unwrap_or(self.current_token.start());
        let mut is_abstract = opts.is_abstract;
        if self.current_token.tag() == TokenTag::Abstract
            && !is_abstract
            && !opts.is_expression
            && self.lang.is_typescript()
            && self.is_abstract_class_next()?
        {
            is_abstract = true;
            self.advance()?;
        }

        if !self.expect(TokenTag::Class, "Expected 'class' keyword", None) {
            return None;
        }
        let class_type = if opts.is_expression && !opts.is_default_export {
            ClassType::ClassExpression
        } else {
            ClassType::ClassDeclaration
        };
        if class_type == ClassType::ClassDeclaration
            && !self.statement_context.permits_declaration()
        {
            self.report(
                Span::new(start, self.current_token.end()),
                "Class declarations are not allowed in single-statement contexts",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Wrap the class declaration in a block: { class C {} }",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        let id = if self.can_start_class_name() {
            Some(self.parse_binding_identifier()?)
        } else if !opts.is_expression && !opts.is_default_export {
            self.report(
                self.current_token.span(),
                "Class declaration requires a name",
                ReportOptions::default(),
            );
            None
        } else {
            None
        };
        let type_parameters = if self.lang.is_typescript()
            && is_angle_open(self.current_token.tag())
        {
            Some(self.parse_type_parameters(TypeParameterContext::Declaration, AngleClose::Peel)?)
        } else {
            None
        };
        let (super_class, super_type_arguments) = if self.eat(TokenTag::Extends)? {
            let mut super_class = self.parse_left_hand_side_expression(LhsContext::Expression)?;
            let type_arguments = if self.lang.is_typescript() {
                if let ExprData::TsInstantiationExpression(instantiation) =
                    self.ast.expr_data(super_class)
                {
                    super_class = instantiation.expression(&self.ast);
                    Some(instantiation.type_arguments(&self.ast))
                } else if is_angle_open(self.current_token.tag()) {
                    Some(self.parse_type_arguments()?)
                } else {
                    None
                }
            } else {
                None
            };
            (Some(super_class), type_arguments)
        } else {
            (None, None)
        };
        if self.lang.is_typescript()
            && super_class.is_some()
            && self.current_token.tag() == TokenTag::Comma
        {
            let next = self.peek_ahead()?;
            if matches!(next.tag(), TokenTag::Implements | TokenTag::LeftBrace) {
                self.report(
                    self.current_token.span(),
                    "A class extends clause cannot have a trailing comma",
                    ReportOptions::default(),
                );
                self.advance()?;
            }
        }
        let implements = if self.lang.is_typescript() {
            self.parse_implements_clause()?
        } else {
            SubRange::empty()
        };
        let body = self.parse_class_body()?;
        let end = body.span(&self.ast).end;
        Some(self.ast.class(
            Span::new(start, end),
            class_type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            is_abstract,
            opts.is_declare,
        ))
    }

    fn can_start_class_name(&mut self) -> bool {
        let tag = self.current_token.tag();
        if !tag.is_identifier_like() || tag == TokenTag::Extends {
            return false;
        }
        if self.lang.is_typescript() && tag == TokenTag::Implements {
            let Some(next) = self.peek_ahead() else {
                return true;
            };
            return !next.tag().is_identifier_like();
        }
        true
    }

    fn parse_implements_clause(&mut self) -> Option<SubRange<TsClassImplements>> {
        match self.parse_heritage_clause(TokenTag::Implements, HeritageKind::Class)? {
            HeritageClause::Class(implements) => Some(implements),
            HeritageClause::Interface(_) => unreachable!(),
        }
    }

    fn parse_heritage_expression(&mut self) -> Option<Expr> {
        let mut expression = self.parse_identifier()?;
        while self.current_token.tag() == TokenTag::Dot {
            expression = self.parse_static_member_expression(expression, false)?;
        }
        Some(expression)
    }

    fn parse_class_body(&mut self) -> Option<ClassBody> {
        let start = self.current_token.start();
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start class body",
            Some("Class body must be enclosed in braces: class Name { ... }"),
        ) {
            return None;
        }
        let checkpoint = self.scratch_a.begin();
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            while !matches!(
                self.current_token.tag(),
                TokenTag::RightBrace | TokenTag::Eof
            ) {
                if self.eat(TokenTag::Semicolon)? {
                    continue;
                }
                let element = self.parse_class_element()?;
                self.scratch_a.append(element);
            }
            let end = self.current_token.end();
            if !self.expect(
                TokenTag::RightBrace,
                "Expected '}' to close class body",
                Some(
                    "Add a closing brace '}' to complete the class, or check for unbalanced braces inside.",
                ),
            ) {
                return None;
            }
            let elements = self.add_extra_from_scratch_a(checkpoint);
            Some(self.ast.class_body(Span::new(start, end), elements))
        })();
        self.ts_context = ts_context;
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_class_element(&mut self) -> Option<ClassElement> {
        let element_start = self.current_token.start();
        let decorators = if self.current_token.tag() == TokenTag::At {
            self.parse_decorators()?.1
        } else {
            SubRange::empty()
        };

        let mut modifiers = ClassElementModifiers::default();
        let mut key = None;
        let mut has_modifiers = false;
        loop {
            let modifier_end = self.current_token.start();
            if self.current_token.tag() == TokenTag::Static
                && let Some(block) = self.try_static_block(decorators)?
            {
                if has_modifiers {
                    self.report(
                        Span::new(element_start, modifier_end),
                        "Static blocks cannot have modifiers",
                        ReportOptions::default(),
                    );
                    let end = block.span(&self.ast).end;
                    block.set_span(&mut self.ast, Span::new(element_start, end));
                }
                return Some(block);
            }
            match self.consume_modifier(&mut modifiers)? {
                ClassModifierStep::Consumed => has_modifiers = true,
                ClassModifierStep::Key(property_key) => {
                    key = Some(property_key);
                    break;
                }
                ClassModifierStep::None => break,
            }
        }

        if key.is_none() && self.eat(TokenTag::Star)? {
            modifiers.is_generator = true;
        }

        if key.is_none()
            && self.lang.is_typescript()
            && self.current_token.tag() == TokenTag::LeftBracket
            && self.is_index_signature_start()?
        {
            return self.parse_index_signature_element(element_start, decorators, modifiers);
        }

        if key.is_none() {
            modifiers.computed = self.current_token.tag() == TokenTag::LeftBracket;
            key = Some(self.parse_class_element_key()?);
        }
        let key = key.unwrap();

        let mut optional = false;
        let mut definite = false;
        if self.lang.is_typescript() {
            match self.current_token.tag() {
                TokenTag::Question => {
                    optional = true;
                    self.advance()?;
                }
                TokenTag::LogicalNot if !self.current_token.has_line_terminator_before() => {
                    definite = true;
                    self.advance()?;
                }
                _ => {}
            }
        }

        self.validate_private_constructor(key, modifiers.computed);
        let is_method_start = self.current_token.tag() == TokenTag::LeftParen
            || (self.lang.is_typescript() && self.current_token.tag() == TokenTag::LessThan);
        if is_method_start {
            let computed = modifiers.computed;
            self.detect_constructor_kind(key, &mut modifiers, computed);
            if modifiers.is_accessor {
                return {
                    let span = self.current_token.span();
                    self.report(
                        span,
                        "Accessor properties cannot be methods",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Remove the parentheses to declare an auto-accessor field.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    None
                };
            }
            if definite {
                self.report(
                    key.span(&self.ast),
                    "Method cannot have a definite assignment assertion",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Remove the '!' or declare a property instead.",
                        )),
                        ..ReportOptions::default()
                    },
                );
            }
            return self.parse_method_definition(
                element_start,
                decorators,
                key,
                modifiers,
                optional,
            );
        }

        let (message, help) = match modifiers.kind {
            _ if modifiers.is_async || modifiers.is_generator => (
                "Expected '(' for method definition",
                "Method definitions require a parameter list. Use 'method() {}' syntax.",
            ),
            MethodDefinitionKind::Get | MethodDefinitionKind::Set => (
                "Expected '(' for getter/setter definition",
                "Getters and setters require parentheses. Use 'get prop() {}' or 'set prop(value) {}' syntax.",
            ),
            _ => {
                return self.parse_property_definition(
                    element_start,
                    decorators,
                    key,
                    modifiers,
                    optional,
                    definite,
                );
            }
        };
        self.report_expected(
            self.current_token.span(),
            message,
            ReportOptions {
                help: Some(Cow::Borrowed(help)),
                ..ReportOptions::default()
            },
        );
        None
    }

    fn parse_index_signature_element(
        &mut self,
        start: u32,
        _decorators: SubRange<Decorator>,
        modifiers: ClassElementModifiers,
    ) -> Option<ClassElement> {
        let element = self.parse_index_signature(start, modifiers.readonly, modifiers.is_static)?;
        let TsTypeElementData::TsIndexSignature(signature) = self.ast.ts_type_element_data(element)
        else {
            unreachable!("parse_index_signature always creates a TS index signature")
        };
        if self.current_token.tag() == TokenTag::Semicolon {
            self.extend_span_to(signature, self.current_token.end());
            self.advance()?;
        }
        Some(ClassElement::TsIndexSignature(signature))
    }

    fn consume_modifier(
        &mut self,
        modifiers: &mut ClassElementModifiers,
    ) -> Option<ClassModifierStep> {
        let token = self.current_token;
        if token.is_escaped() {
            return Some(ClassModifierStep::None);
        }

        let tag = token.tag();
        let is_typescript = self.lang.is_typescript();
        let is_valid_modifier = is_modifier(tag, is_typescript);
        let invalid_modifier = !is_valid_modifier
            && is_typescript
            && matches!(tag, TokenTag::Const | TokenTag::Export);
        if !is_valid_modifier && !invalid_modifier {
            return Some(ClassModifierStep::None);
        }

        let Some(next) = self.peek_ahead() else {
            return Some(ClassModifierStep::None);
        };
        let next_tag = next.tag();
        let accessor_before_star =
            next_tag == TokenTag::Star && matches!(tag, TokenTag::Get | TokenTag::Set);
        let can_start_modifier = can_start_element_key(next_tag)
            && !accessor_before_star
            && !(requires_same_line(tag) && next.has_line_terminator_before());
        let conflicting_modifier =
            can_start_modifier && !invalid_modifier && modifiers.conflicts(tag);
        let consume_as_modifier =
            can_start_modifier && !(conflicting_modifier && next.has_line_terminator_before());

        if !consume_as_modifier {
            self.advance_without_escape_check()?;
            let name = self.identifier_name(token);
            return Some(ClassModifierStep::Key(
                self.ast.property_key_identifier_name(token.span(), name),
            ));
        }

        if invalid_modifier {
            self.report(
                token.span(),
                "Invalid class member modifier",
                ReportOptions::default(),
            );
        } else {
            if conflicting_modifier {
                self.report(
                    token.span(),
                    "Duplicate or incompatible class member modifier",
                    ReportOptions::default(),
                );
            }
            modifiers.set(tag);
        }
        self.advance_without_escape_check()?;
        Some(ClassModifierStep::Consumed)
    }

    fn try_static_block(
        &mut self,
        decorators: SubRange<Decorator>,
    ) -> Option<Option<ClassElement>> {
        let static_token = self.current_token;
        if static_token.tag() != TokenTag::Static || static_token.is_escaped() {
            return Some(None);
        }
        let Some(next) = self.peek_ahead() else {
            // Normal modifier parsing will report the lexical error.
            return Some(None);
        };
        if next.tag() != TokenTag::LeftBrace {
            return Some(None);
        }
        if !decorators.is_empty() {
            self.report(
                decorators
                    .get_node(&self.ast, 0)
                    .expect("non-empty decorator range contains its first node")
                    .span(&self.ast),
                "Decorators cannot be applied to static blocks",
                ReportOptions::default(),
            );
        }
        self.advance_without_escape_check()?;
        self.parse_static_block(static_token.start()).map(Some)
    }

    // ClassStaticBlock :
    //   `static` `{` ClassStaticBlockBody `}`
    //
    // ClassStaticBlockBody :
    //   ClassStaticBlockStatementList
    //
    // ClassStaticBlockStatementList :
    //   StatementList[~Yield, +Await, ~Return]?
    fn parse_static_block(&mut self, start: u32) -> Option<ClassElement> {
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start static block",
            None,
        ) {
            return None;
        }
        let checkpoint = self.scratch_statements.begin();
        let body = self.with_scope(BoundaryContext::local(), |parser| {
            enter_grammar_context![&mut parser.context, +In, ~Yield, +Await, ~Return];
            while !matches!(
                parser.current_token.tag(),
                TokenTag::RightBrace | TokenTag::Eof
            ) {
                let statement = parser.parse_statement(StatementContext::StatementListItem)?;
                parser.scratch_statements.append(statement);
            }
            Some(parser.add_extra_from_scratch_statements(checkpoint))
        });
        self.scratch_statements.reset(checkpoint);
        let body = body?;
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close static block",
            None,
        ) {
            return None;
        }
        Some(ClassElement::StaticBlock(
            self.ast.static_block(Span::new(start, end), body),
        ))
    }

    fn validate_private_constructor(&mut self, key: PropertyKey, computed: bool) {
        if computed {
            return;
        }
        if let Some(expression) = key.as_expr(&self.ast)
            && let ExprData::PrivateIdentifier(private) = self.ast.expr_data(expression)
            && self.ast.get_utf8(private.name(&self.ast)) == "constructor"
        {
            self.report(
                key.span(&self.ast),
                "Classes can't have a private field named '#constructor'",
                ReportOptions::default(),
            );
        }
    }

    fn detect_constructor_kind(
        &mut self,
        key: PropertyKey,
        modifiers: &mut ClassElementModifiers,
        computed: bool,
    ) {
        if modifiers.is_static || computed {
            return;
        }
        let Some(span) = self.prop_name_eq(key, "constructor") else {
            return;
        };
        match modifiers.kind {
            MethodDefinitionKind::Method => {
                modifiers.kind = MethodDefinitionKind::Constructor;
            }
            MethodDefinitionKind::Get | MethodDefinitionKind::Set => self.report(
                span,
                "Constructor can't have get/set modifier",
                ReportOptions::default(),
            ),
            MethodDefinitionKind::Constructor => {}
        }
    }

    fn validate_method_modifiers(&mut self, key: PropertyKey, modifiers: ClassElementModifiers) {
        let span = key.span(&self.ast);
        if modifiers.kind == MethodDefinitionKind::Constructor {
            if modifiers.is_async {
                self.report(
                    span,
                    "Constructor cannot be async",
                    ReportOptions::default(),
                );
            }
            if modifiers.is_generator {
                self.report(
                    span,
                    "Constructor cannot be a generator",
                    ReportOptions::default(),
                );
            }
            return;
        }
        if !modifiers.is_generator {
            return;
        }
        match modifiers.kind {
            MethodDefinitionKind::Get => self.report(
                span,
                "Getter cannot be a generator",
                ReportOptions::default(),
            ),
            MethodDefinitionKind::Set => self.report(
                span,
                "Setter cannot be a generator",
                ReportOptions::default(),
            ),
            MethodDefinitionKind::Method | MethodDefinitionKind::Constructor => {}
        }
    }

    fn parse_method_definition(
        &mut self,
        start: u32,
        decorators: SubRange<Decorator>,
        key: PropertyKey,
        modifiers: ClassElementModifiers,
        optional: bool,
    ) -> Option<ClassElement> {
        self.validate_method_modifiers(key, modifiers);

        let function_start = self.current_token.start();
        let result = self.with_scope(BoundaryContext::local(), |parser| {
            let type_parameters =
                if parser.lang.is_typescript() && is_angle_open(parser.current_token.tag()) {
                    Some(parser.parse_type_parameters(
                        TypeParameterContext::Declaration,
                        AngleClose::Peel,
                    )?)
                } else {
                    None
                };
            enter_function_grammar_context(
                &mut parser.context,
                modifiers.is_generator,
                modifiers.is_async,
            );
            let (this_param, params) = parser.parse_formal_parameters(
                FormalParameterKind::UniqueFormalParameters,
                modifiers.kind == MethodDefinitionKind::Constructor,
            )?;
            parser.check_accessor_arity(modifiers.kind, params);
            let return_type =
                if parser.lang.is_typescript() && parser.current_token.tag() == TokenTag::Colon {
                    Some(parser.parse_return_type_annotation()?)
                } else {
                    None
                };
            if parser.current_token.tag() == TokenTag::LeftBrace {
                let body = parser.parse_function_body()?;
                let end = body.span(&parser.ast).end;
                Some((
                    this_param,
                    params,
                    type_parameters,
                    return_type,
                    Some(body),
                    end,
                ))
            } else if parser.lang.is_typescript() {
                let end = parser.eat_semicolon(parser.lexer.state.prev_token_end)?;
                Some((this_param, params, type_parameters, return_type, None, end))
            } else {
                parser.report_expected(
                    parser.current_token.span(),
                    "Expected '{' to start method body",
                    ReportOptions::default(),
                );
                None
            }
        });
        let (this_param, params, type_parameters, return_type, body, end) = result?;
        let function_span = Span::new(function_start, end);
        let method_span = Span::new(start, end);
        if let Some(body) = body {
            let function = self.ast.function(
                function_span,
                FunctionType::FunctionExpression,
                None,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                modifiers.is_generator,
                modifiers.is_async,
            );
            let method = self.ast.method_definition(
                method_span,
                decorators,
                key,
                function,
                modifiers.kind,
                modifiers.computed,
                modifiers.is_static,
                modifiers.r#override,
                optional,
                modifiers.r#abstract,
                modifiers.accessibility,
            );
            Some(ClassElement::MethodDefinition(method))
        } else {
            let function = self.ast.ts_function(
                function_span,
                None,
                type_parameters,
                this_param,
                params,
                return_type,
                modifiers.is_generator,
                modifiers.is_async,
                false,
            );
            let method = self.ast.ts_method_definition(
                method_span,
                decorators,
                key,
                function,
                modifiers.kind,
                modifiers.computed,
                modifiers.is_static,
                modifiers.r#override,
                optional,
                modifiers.r#abstract,
                modifiers.accessibility,
            );
            Some(ClassElement::TsMethodDefinition(method))
        }
    }

    // FieldDefinition[Yield, Await] :
    //   ClassElementName[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]?
    fn parse_property_definition(
        &mut self,
        start: u32,
        decorators: SubRange<Decorator>,
        key: PropertyKey,
        modifiers: ClassElementModifiers,
        optional: bool,
        definite: bool,
    ) -> Option<ClassElement> {
        if !modifiers.computed
            && let Some(span) = self.prop_name_eq(key, "constructor")
        {
            self.report(
                span,
                "Classes may not have a field named 'constructor'",
                ReportOptions::default(),
            );
        }

        let type_annotation =
            if self.lang.is_typescript() && self.current_token.tag() == TokenTag::Colon {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };

        let value = if self.eat(TokenTag::Assign)? {
            let value = self.with_scope(BoundaryContext::local(), |parser| {
                enter_grammar_context![&mut parser.context, +In, ~Yield, ~Await];
                parser.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())
            });
            Some(value?)
        } else {
            None
        };

        let mut end = self.lexer.state.prev_token_end;
        match self.current_token.tag() {
            TokenTag::Semicolon => {
                end = self.current_token.end();
                self.advance()?;
            }
            TokenTag::RightBrace => {}
            _ => {
                if !self.can_insert_implicit_semicolon(self.current_token) {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected ';' after class field",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Add a semicolon after the field declaration.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
            }
        }
        Some(ClassElement::PropertyDefinition(
            self.ast.property_definition(
                Span::new(start, end),
                decorators,
                key,
                type_annotation,
                value,
                modifiers.computed,
                modifiers.is_static,
                modifiers.is_accessor,
                modifiers.declare,
                modifiers.r#override,
                optional,
                definite,
                modifiers.readonly,
                modifiers.r#abstract,
                modifiers.accessibility,
            ),
        ))
    }

    fn parse_decorators(&mut self) -> Option<(u32, SubRange<Decorator>)> {
        let start = self.current_token.start();
        let checkpoint = self.scratch_decorators.begin();
        let result = (|| {
            while self.current_token.tag() == TokenTag::At {
                let decorator = self.parse_decorator()?;
                self.scratch_decorators.append(decorator);
            }
            Some((start, self.add_extra_from_scratch_decorators(checkpoint)))
        })();
        self.scratch_decorators.reset(checkpoint);
        result
    }

    fn parse_decorator(&mut self) -> Option<Decorator> {
        let start = self.current_token.start();
        if !self.expect(TokenTag::At, "Expected '@' to start a decorator", None) {
            return None;
        }
        let expression = self.parse_left_hand_side_expression(LhsContext::Decorator)?;
        Some(
            self.ast
                .decorator(Span::new(start, expression.span(&self.ast).end), expression),
        )
    }

    fn parse_left_hand_side_expression(&mut self, context: LhsContext) -> Option<Expr> {
        let expression = match self.current_token.tag() {
            TokenTag::LeftParen => self.parse_parenthesized_expression()?,
            TokenTag::New => self.parse_new_expression()?,
            TokenTag::Import => self.parse_import_expression()?,
            _ => self.parse_primary_expression(Precedence::Primary)?,
        };
        let expression = self.parse_postfix_expression(expression, context)?;
        Some(expression.expression)
    }

    fn prop_name_eq(&self, key: PropertyKey, expected: &str) -> Option<Span> {
        let matches = match self.ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(identifier) => {
                self.ast.get_utf8(identifier.name(&self.ast)) == expected
            }
            PropertyKeyData::Expr(expression) => matches!(
                self.ast.expr_data(expression),
                ExprData::StringLiteral(string)
                    if self.ast.get_wtf8(string.value(&self.ast)) == expected
            ),
        };
        if matches {
            Some(key.span(&self.ast))
        } else {
            None
        }
    }

    fn check_accessor_arity(
        &mut self,
        kind: impl Into<PropertyOrMethodKind>,
        params: FormalParameters,
    ) {
        let kind = kind.into();
        let Some((expected, msg, help)) = kind.accessor_arity() else {
            return;
        };
        let items = params.items(&self.ast);
        let count = items.len();
        let has_rest = params.rest(&self.ast).is_some();
        if count == expected && !has_rest {
            return;
        }
        self.report(
            params.span(&self.ast),
            msg,
            ReportOptions {
                help: Some(Cow::Borrowed(help)),
                ..ReportOptions::default()
            },
        );
    }

    fn parse_formal_parameters(
        &mut self,
        kind: FormalParameterKind,
        allow_parameter_properties: bool,
    ) -> Option<(Option<TsThisParameter>, FormalParameters)> {
        let start = self.current_token.start();
        if !self.expect(
            TokenTag::LeftParen,
            "Expected '(' to start parameter list",
            None,
        ) {
            return None;
        }
        let this_param = if self.lang.is_typescript() && self.current_token.tag() == TokenTag::This
        {
            if allow_parameter_properties || kind == FormalParameterKind::ArrowFormalParameters {
                self.report(
                    self.current_token.span(),
                    "A 'this' parameter is not allowed here",
                    ReportOptions::default(),
                );
            }
            // Unlike constructor functions, arrows cannot retain a receiver.
            if kind == FormalParameterKind::ArrowFormalParameters {
                return None;
            }
            let parameter = self.parse_this_parameter()?;
            if self.current_token.tag() != TokenTag::RightParen
                && !self.expect(TokenTag::Comma, "Expected ',' after this parameter", None)
            {
                return None;
            }
            Some(parameter)
        } else {
            None
        };
        let checkpoint = self.scratch_a.begin();
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            let mut rest = None;
            while self.current_token.tag() != TokenTag::RightParen
                && self.current_token.tag() != TokenTag::Eof
            {
                let parameter_start = self.current_token.start();
                let decorators =
                    if self.lang.is_typescript() && self.current_token.tag() == TokenTag::At {
                        self.parse_decorators()?.1
                    } else {
                        SubRange::empty()
                    };

                if self.current_token.tag() == TokenTag::Spread {
                    rest = Some(self.parse_formal_parameter_rest(kind, decorators)?);
                    if self.eat(TokenTag::Comma)?
                        && (!self.ts_context.ambient()
                            || self.current_token.tag() != TokenTag::RightParen)
                    {
                        self.recover_rest_parameter_end()?;
                    }
                    break;
                }

                let parameter = if self.lang.is_typescript() {
                    self.parse_formal_parameter::<true>(
                        kind,
                        allow_parameter_properties,
                        decorators,
                        parameter_start,
                    )
                } else {
                    self.parse_formal_parameter::<false>(
                        kind,
                        allow_parameter_properties,
                        decorators,
                        parameter_start,
                    )
                }?;

                self.scratch_a.append(parameter);
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            let end = self.current_token.end();
            if !self.expect(
                TokenTag::RightParen,
                "Expected ')' to close parameter list",
                None,
            ) {
                return None;
            }
            let items = self.add_extra_from_scratch_a(checkpoint);
            let parameters = self
                .ast
                .formal_parameters(Span::new(start, end), items, rest, kind);
            Some((this_param, parameters))
        })();
        self.ts_context = ts_context;
        self.scratch_a.reset(checkpoint);
        result
    }

    #[cold]
    #[inline(never)]
    fn recover_rest_parameter_end(&mut self) -> Option<()> {
        self.report(
            self.current_token.span(),
            "Rest parameter must be the last parameter",
            ReportOptions::default(),
        );
        (self.current_token.tag() == TokenTag::RightParen).then_some(())
    }

    fn parse_formal_parameter<const IS_TYPESCRIPT: bool>(
        &mut self,
        kind: FormalParameterKind,
        allow_parameter_properties: bool,
        decorators: SubRange<Decorator>,
        start: u32,
    ) -> Option<FormalParameterItem> {
        let modifiers = if IS_TYPESCRIPT && allow_parameter_properties {
            self.parse_parameter_property_modifiers()?
        } else {
            ParameterPropertyModifiers::default()
        };
        let parameter_start = self.current_token.start();
        let (pattern, cover_initializer) = if kind == FormalParameterKind::Signature {
            self.parse_signature_parameter_parts()
        } else {
            self.parse_formal_parameter_parts()
        }?;
        let optional = IS_TYPESCRIPT && self.eat(TokenTag::Question)?;
        let type_annotation = if IS_TYPESCRIPT && self.current_token.tag() == TokenTag::Colon {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        let initializer = if self.eat(TokenTag::Assign)? {
            let context = enter_grammar_context![&mut self.context, +In];
            let initializer = self.parse_value_expression(Precedence::Assignment);
            self.context = context;
            let initializer = initializer?;
            Some(initializer)
        } else {
            cover_initializer
        };
        if self.ts_context.ambient()
            && let Some(initializer) = initializer
        {
            self.report(
                initializer.span(&self.ast),
                "A parameter initializer is only allowed in a function or constructor implementation",
                ReportOptions::default(),
            );
        }
        let end = self.lexer.state.prev_token_end;
        let parameter_decorators = if modifiers.present {
            SubRange::empty()
        } else {
            decorators
        };
        let parameter_start = if modifiers.present {
            parameter_start
        } else {
            start
        };
        let parameter = self.ast.formal_parameter(
            Span::new(parameter_start, end),
            parameter_decorators,
            pattern,
            type_annotation,
            initializer,
            optional,
        );
        if modifiers.present {
            let property = self.ast.ts_parameter_property(
                Span::new(start, end),
                decorators,
                parameter,
                modifiers.r#override,
                modifiers.readonly,
                modifiers.accessibility,
            );
            Some(FormalParameterItem::TsParameterProperty(property))
        } else {
            Some(FormalParameterItem::FormalParameter(parameter))
        }
    }

    // Keep ordinary bindings direct while sharing parenthesized recovery.
    #[inline(always)]
    fn parse_formal_parameter_parts(&mut self) -> Option<(FormalParameterPattern, Option<Expr>)> {
        let tag = self.current_token.tag();
        if tag.is_identifier_like() {
            let pattern = self.parse_binding_identifier()?;
            return Some((FormalParameterPattern::BindingIdentifier(pattern), None));
        }
        if tag == TokenTag::LeftParen {
            return self.parse_parenthesized_parameter_parts();
        }
        Some((self.parse_formal_parameter_pattern()?, None))
    }

    #[inline(always)]
    fn parse_signature_parameter_parts(
        &mut self,
    ) -> Option<(FormalParameterPattern, Option<Expr>)> {
        if self.current_token.tag().is_identifier_like() {
            let pattern = self.parse_type_binding_identifier()?;
            return Some((FormalParameterPattern::BindingIdentifier(pattern), None));
        }
        self.with_type_binding_context(Self::parse_formal_parameter_parts)
    }

    fn parse_parenthesized_parameter_parts(
        &mut self,
    ) -> Option<(FormalParameterPattern, Option<Expr>)> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftParen);
        let context = enter_grammar_context![&mut self.context, +In];
        let result = self.with_resolved_cover(|parser| {
            let expression = parser.parse_expression_in_cover(Precedence::Assignment)?;
            parser.expression_to_parameter_parts(expression)
        });
        self.context = context;
        result
    }

    fn formal_parameter_pattern(
        &mut self,
        pattern: BindingPattern,
    ) -> Option<FormalParameterPattern> {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(node) => {
                Some(FormalParameterPattern::BindingIdentifier(node))
            }
            BindingPatternData::ArrayPattern(node) => {
                Some(FormalParameterPattern::ArrayPattern(node))
            }
            BindingPatternData::ObjectPattern(node) => {
                Some(FormalParameterPattern::ObjectPattern(node))
            }
            _ => {
                self.report(
                    pattern.span(&self.ast),
                    "Invalid parameter binding",
                    ReportOptions::default(),
                );
                None
            }
        }
    }

    fn parse_this_parameter(&mut self) -> Option<TsThisParameter> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::This);
        let start = self.current_token.start();
        self.advance()?;
        let type_annotation = if self.current_token.tag() == TokenTag::Colon {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        let end = type_annotation.map_or(self.lexer.state.prev_token_end, |annotation| {
            annotation.span(&self.ast).end
        });
        Some(
            self.ast
                .ts_this_parameter(Span::new(start, end), type_annotation),
        )
    }

    fn parse_formal_parameter_rest(
        &mut self,
        kind: FormalParameterKind,
        decorators: SubRange<Decorator>,
    ) -> Option<FormalParameterRest> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Spread);
        let start = self.current_token.start();
        self.advance()?;
        let argument = if kind == FormalParameterKind::Signature {
            self.with_type_binding_context(Self::parse_formal_parameter_pattern)
        } else {
            self.parse_formal_parameter_pattern()
        }?;
        let type_annotation =
            if self.lang.is_typescript() && self.current_token.tag() == TokenTag::Colon {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
        let end = type_annotation.map_or(argument.span(&self.ast).end, |annotation| {
            annotation.span(&self.ast).end
        });
        Some(self.ast.formal_parameter_rest(
            Span::new(start, end),
            decorators,
            argument,
            type_annotation,
        ))
    }

    fn parse_parameter_property_modifiers(&mut self) -> Option<ParameterPropertyModifiers> {
        let mut modifiers = ParameterPropertyModifiers::default();
        while self.is_parameter_property_modifier_start()? {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            self.report_if_escaped_keyword(token);
            match token.tag() {
                TokenTag::Public => modifiers.accessibility = Accessibility::Public,
                TokenTag::Private => modifiers.accessibility = Accessibility::Private,
                TokenTag::Protected => modifiers.accessibility = Accessibility::Protected,
                TokenTag::Readonly => modifiers.readonly = true,
                TokenTag::Override => modifiers.r#override = true,
                _ => unreachable!(),
            }
            modifiers.present = true;
        }
        Some(modifiers)
    }

    fn is_parameter_property_modifier_start(&mut self) -> Option<bool> {
        if !is_parameter_property_modifier_tag(self.current_token.tag()) {
            return Some(false);
        }
        let next = self.peek_ahead()?;
        if next.has_line_terminator_before() {
            return Some(false);
        }
        Some(can_follow_parameter_property_modifier(next.tag()))
    }

    // FunctionBody[Yield, Await] :
    //   FunctionStatementList[?Yield, ?Await]
    //
    // FunctionStatementList[Yield, Await] :
    //   StatementList[?Yield, ?Await, +Return]?
    fn parse_function_body(&mut self) -> Option<FunctionBody> {
        let start = self.current_token.start();
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start function body",
            Some("Function bodies must be enclosed in braces: function name() { ... }"),
        ) {
            return None;
        }
        let grammar_context = enter_grammar_context![&mut self.context, +In, +Return];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let body = self.parse_body(
            Some(TokenTag::RightBrace),
            StatementContext::StatementListItem,
        );
        self.ts_context = ts_context;
        self.context = grammar_context;
        let ParsedBody { directives, body } = body?;
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close function body",
            Some(
                "Add a closing brace '}' to complete the function, or check for unbalanced braces inside.",
            ),
        ) {
            return None;
        }
        Some(
            self.ast
                .function_body(Span::new(start, end), directives, body),
        )
    }

    fn parse_import_declaration(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Import);
        let start = self.current_token.start();
        self.advance()?;
        self.parse_import_declaration_from(start)
    }

    fn parse_import_declaration_from(&mut self, start: u32) -> Option<Stmt> {
        if self.current_token.tag() == TokenTag::StringLiteral {
            return self.parse_side_effect_import(start, None);
        }

        let mut phase = None;
        let mut import_kind = ImportOrExportKind::Value;
        // Ordinary imports do not need another token to classify the clause.
        let tag = self.current_token.tag();
        match tag {
            TokenTag::Type if self.lang.is_typescript() => {
                let next = self.peek_ahead()?;
                if self.is_type_import_modifier(next) {
                    import_kind = ImportOrExportKind::Type;
                    self.advance()?;
                }
            }
            TokenTag::Source => {
                let next = self.peek_ahead()?;
                if self.is_source_phase_import(next) {
                    phase = Some(ImportPhase::Source);
                    self.advance()?;
                }
            }
            TokenTag::Defer if self.peek_ahead()?.tag() == TokenTag::Star => {
                phase = Some(ImportPhase::Defer);
                self.advance()?;
            }
            _ => {}
        }

        if self.lang.is_typescript()
            && self.current_token.tag().is_identifier_like()
            && self.peek_ahead()?.tag() == TokenTag::Assign
        {
            return self.parse_import_equals_body(start, import_kind);
        }

        let specifiers = self.parse_import_clause()?;
        if self.current_token.tag() != TokenTag::From {
            self.report_expected(
                self.current_token.span(),
                "Expected 'from' after import clause",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Import statements require 'from' followed by a module specifier: import x from 'module'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.advance()?;
        let source = self.parse_module_specifier()?;
        let attributes = self.parse_with_clause();
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_import_declaration(
            Span::new(start, end),
            specifiers,
            source,
            attributes,
            phase,
            import_kind,
        ))
    }

    fn is_type_import_modifier(&mut self, after_type: Token) -> bool {
        if matches!(after_type.tag(), TokenTag::LeftBrace | TokenTag::Star) {
            return true;
        }
        if !after_type.tag().is_identifier_like() {
            return false;
        }
        if after_type.tag() == TokenTag::From {
            let mut peek = self.begin_peek();
            let after_from = (|| {
                peek.next()?;
                peek.next()
            })();
            peek.end();
            return matches!(
                after_from.map(|token| token.tag()),
                Some(TokenTag::From | TokenTag::Assign)
            );
        }
        true
    }

    fn is_source_phase_import(&mut self, after_source: Token) -> bool {
        if !after_source.tag().is_identifier_like() {
            return false;
        }
        if after_source.tag() != TokenTag::From {
            return true;
        }

        let mut peek = self.begin_peek();
        let after_from = (|| {
            peek.next()?;
            peek.next()
        })();
        peek.end();
        after_from.is_some_and(|token| token.tag() == TokenTag::From)
    }

    fn parse_side_effect_import(&mut self, start: u32, phase: Option<ImportPhase>) -> Option<Stmt> {
        let source = self.parse_module_specifier()?;
        let attributes = self.parse_with_clause();
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        let specifiers = SubRange::empty();
        Some(self.ast.stmt_import_declaration(
            Span::new(start, end),
            specifiers,
            source,
            attributes,
            phase,
            ImportOrExportKind::Value,
        ))
    }

    fn parse_import_clause(&mut self) -> Option<SubRange<ImportDeclarationSpecifier>> {
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            if self.current_token.tag() == TokenTag::Star {
                let specifier = self.parse_import_namespace_specifier()?;
                self.scratch_a
                    .append(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                        specifier,
                    ));
                return Some(self.add_extra_from_scratch_a(checkpoint));
            }
            if self.current_token.tag() == TokenTag::LeftBrace {
                return self.parse_named_imports();
            }

            let specifier = self.parse_import_default_specifier()?;
            self.scratch_a
                .append(ImportDeclarationSpecifier::ImportDefaultSpecifier(
                    specifier,
                ));
            if self.eat(TokenTag::Comma)? {
                if self.current_token.tag() == TokenTag::Star {
                    let specifier = self.parse_import_namespace_specifier()?;
                    self.scratch_a
                        .append(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                            specifier,
                        ));
                } else if self.current_token.tag() == TokenTag::LeftBrace {
                    let named = self.parse_named_imports()?;
                    for id in named.iter() {
                        self.scratch_a.append(self.ast.get_node_in_sub_range(id));
                    }
                } else {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected namespace import (* as name) or named imports ({...}) after ','",
                        ReportOptions::default(),
                    );
                    return None;
                }
            }
            Some(self.add_extra_from_scratch_a(checkpoint))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_import_default_specifier(&mut self) -> Option<ImportDefaultSpecifier> {
        let start = self.current_token.start();
        let local = self.parse_imported_binding()?;
        Some(
            self.ast
                .import_default_specifier(Span::new(start, local.span(&self.ast).end), local),
        )
    }

    fn parse_import_namespace_specifier(&mut self) -> Option<ImportNamespaceSpecifier> {
        let start = self.current_token.start();
        if !self.expect(TokenTag::Star, "Expected '*' for namespace import", None) {
            return None;
        }
        if self.current_token.tag() != TokenTag::As {
            self.report_expected(
                self.current_token.span(),
                "Expected 'as' after '*' in namespace import",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Namespace imports must use the form: * as name",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.advance()?;
        let local = self.parse_imported_binding()?;
        Some(
            self.ast
                .import_namespace_specifier(Span::new(start, local.span(&self.ast).end), local),
        )
    }

    fn parse_named_imports(&mut self) -> Option<SubRange<ImportDeclarationSpecifier>> {
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start named imports",
            None,
        ) {
            return None;
        }
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while self.current_token.tag() != TokenTag::RightBrace
                && self.current_token.tag() != TokenTag::Eof
            {
                let specifier = self.parse_import_specifier()?;
                self.scratch_a
                    .append(ImportDeclarationSpecifier::ImportSpecifier(specifier));
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            if !self.expect(
                TokenTag::RightBrace,
                "Expected '}' to close named imports",
                None,
            ) {
                return None;
            }
            Some(self.add_extra_from_scratch_a(checkpoint))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_import_specifier(&mut self) -> Option<ImportSpecifier> {
        let start = self.current_token.start();
        let parts = self.parse_specifier_parts()?;
        let imported = parts.property_name.unwrap_or(parts.name);
        let local_name = parts.name;
        let local_span = local_name.span(&self.ast);
        let Some(identifier) = local_name.as_identifier_name(&self.ast) else {
            if parts.property_name.is_none() {
                self.report(
                    local_span,
                    "String literal imports require an 'as' clause",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Use: import { \"name\" as localName } from 'module'",
                        )),
                        ..ReportOptions::default()
                    },
                );
            } else {
                self.report(
                    local_span,
                    "Import local binding must be an identifier",
                    ReportOptions::default(),
                );
            }
            return None;
        };
        let name = identifier.name(&self.ast);
        self.validate_identifier(parts.name_token, IdentifierRole::Binding);
        let local = self.ast.binding_identifier(local_span, name);
        Some(self.ast.import_specifier(
            Span::new(start, local_span.end),
            imported,
            local,
            parts.kind,
        ))
    }

    fn parse_specifier_parts(&mut self) -> Option<SpecifierParts> {
        let first_token = self.current_token;
        let first = self.parse_module_export_name()?;
        let mut parts = SpecifierParts {
            property_name: None,
            name: first,
            name_token: first_token,
            kind: ImportOrExportKind::Value,
        };
        let tail = if self.lang.is_typescript()
            && first_token.tag() == TokenTag::Type
            && first.is_identifier_name(&self.ast)
        {
            self.parse_type_specifier_tail(&mut parts, first)?
        } else {
            TypeTailResult::NoTypeModifier
        };
        if tail != TypeTailResult::ConsumedRename && self.eat(TokenTag::As)? {
            parts.property_name = Some(parts.name);
            parts.name_token = self.current_token;
            parts.name = self.parse_module_export_name()?;
        }
        Some(parts)
    }

    fn parse_type_specifier_tail(
        &mut self,
        parts: &mut SpecifierParts,
        first: ModuleExportName,
    ) -> Option<TypeTailResult> {
        if self.current_token.tag() == TokenTag::As {
            let first_as_token = self.current_token;
            let first_as = self.parse_module_export_name()?;
            if self.current_token.tag() == TokenTag::As {
                let second_as_token = self.current_token;
                let second_as = self.parse_module_export_name()?;
                if self.can_start_module_export_name() {
                    parts.kind = ImportOrExportKind::Type;
                    parts.property_name = Some(first_as);
                    parts.name_token = self.current_token;
                    parts.name = self.parse_module_export_name()?;
                } else {
                    parts.property_name = Some(first);
                    parts.name = second_as;
                    parts.name_token = second_as_token;
                }
                return Some(TypeTailResult::ConsumedRename);
            }
            if self.can_start_module_export_name() {
                parts.property_name = Some(first);
                parts.name_token = self.current_token;
                parts.name = self.parse_module_export_name()?;
                return Some(TypeTailResult::ConsumedRename);
            }
            parts.kind = ImportOrExportKind::Type;
            parts.name = first_as;
            parts.name_token = first_as_token;
            return Some(TypeTailResult::KeepOuterRename);
        }
        if self.can_start_module_export_name() {
            parts.kind = ImportOrExportKind::Type;
            parts.name_token = self.current_token;
            parts.name = self.parse_module_export_name()?;
        }
        Some(TypeTailResult::KeepOuterRename)
    }

    fn can_start_module_export_name(&self) -> bool {
        self.current_token.tag().is_identifier_like()
            || self.current_token.tag() == TokenTag::StringLiteral
    }

    fn parse_imported_binding(&mut self) -> Option<BindingIdentifier> {
        self.parse_binding_identifier()
    }

    fn parse_module_specifier(&mut self) -> Option<StringLiteral> {
        if self.current_token.tag() != TokenTag::StringLiteral {
            self.report_expected(
                self.current_token.span(),
                "Expected module specifier",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Module specifiers must be string literals, e.g., './module.js' or 'package'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.parse_string_literal()
    }

    fn parse_import_equals_body(
        &mut self,
        start: u32,
        import_kind: ImportOrExportKind,
    ) -> Option<Stmt> {
        let id = self.parse_binding_identifier()?;
        if !self.expect(
            TokenTag::Assign,
            "Expected '=' in import equals declaration",
            Some(
                "An import equals declaration is written 'import x = Foo.Bar' or 'import x = require(\"m\")'",
            ),
        ) {
            return None;
        }
        let module_reference = self.parse_module_reference()?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_decl_ts_import_equals_declaration(
            Span::new(start, end),
            id,
            module_reference,
            import_kind,
        ))
    }

    fn parse_module_reference(&mut self) -> Option<TsModuleReference> {
        if self.current_token.tag() == TokenTag::Require
            && self.peek_ahead()?.tag() == TokenTag::LeftParen
        {
            return self.parse_external_module_reference();
        }

        let head_token = self.consume_identifier(IdentifierRole::TypeScript)?;
        let name = self.identifier_name(head_token);
        let head = self.ast.identifier_reference(head_token.span(), name);
        if self.current_token.tag() != TokenTag::Dot {
            return Some(TsModuleReference::TsTypeName(
                TsTypeName::IdentifierReference(head),
            ));
        }

        let mut left = TsQualifiedNameLeft::IdentifierReference(head);
        let start = head.span(&self.ast).start;
        loop {
            self.advance()?;
            let right = self.parse_identifier_name_with_role(IdentifierRole::TypeScript)?;
            let qualified = self.ast.ts_qualified_name(
                Span::new(start, right.span(&self.ast).end),
                left,
                right,
            );
            if self.current_token.tag() != TokenTag::Dot {
                return Some(TsModuleReference::TsTypeName(TsTypeName::TsQualifiedName(
                    qualified,
                )));
            }
            left = TsQualifiedNameLeft::TsQualifiedName(qualified);
        }
    }

    fn parse_external_module_reference(&mut self) -> Option<TsModuleReference> {
        let start = self.current_token.start();
        self.advance()?;
        if !self.expect(
            TokenTag::LeftParen,
            "Expected '(' after 'require'",
            Some("External module references are written 'require(\"module\")'"),
        ) {
            return None;
        }
        let expression = self.parse_string_literal()?;
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' to close 'require'",
            None,
        ) {
            return None;
        }
        Some(
            self.ast.ts_module_reference_ts_external_module_reference(
                Span::new(start, end),
                expression,
            ),
        )
    }

    fn parse_export_declaration(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Export);
        self.record_module_declaration();
        let start = self.current_token.start();
        self.advance()?;
        let mut declaration_start = None;
        while self.current_token.tag() == TokenTag::Export {
            declaration_start.get_or_insert(self.current_token.start());
            self.report(
                self.current_token.span(),
                "Duplicate export modifier",
                ReportOptions::default(),
            );
            self.advance()?;
        }
        if self.lang.is_typescript() {
            match self.current_token.tag() {
                TokenTag::Assign => return self.parse_ts_export_assignment(start),
                TokenTag::As => return self.parse_ts_namespace_export_declaration(start),
                TokenTag::Type => {
                    let next = self.peek_ahead()?;
                    if next.tag() == TokenTag::LeftBrace {
                        self.advance()?;
                        return self
                            .parse_export_named_from_clause(start, ImportOrExportKind::Type);
                    }
                    if next.tag() == TokenTag::Star {
                        self.advance()?;
                        return self.parse_export_all_declaration(start, ImportOrExportKind::Type);
                    }
                }
                _ => {}
            }
        }
        let statement = match self.current_token.tag() {
            TokenTag::Default => self.parse_export_default_declaration(start),
            TokenTag::Star => self.parse_export_all_declaration(start, ImportOrExportKind::Value),
            TokenTag::LeftBrace => {
                self.parse_export_named_from_clause(start, ImportOrExportKind::Value)
            }
            _ => self.parse_export_with_declaration(start),
        }?;
        if let Some(declaration_start) = declaration_start {
            match self.ast.stmt_data(statement) {
                StmtData::ExportNamedDeclaration(export) => {
                    if let Some(declaration) = export.declaration(&self.ast) {
                        let span = declaration.span(&self.ast);
                        declaration.set_span(&mut self.ast, Span::new(declaration_start, span.end));
                    }
                }
                StmtData::ExportDefaultDeclaration(export) => {
                    let declaration = export.declaration(&self.ast);
                    if !declaration.is_expr(&self.ast) {
                        let span = declaration.span(&self.ast);
                        declaration.set_span(&mut self.ast, Span::new(declaration_start, span.end));
                    }
                }
                _ => {}
            }
        }
        Some(statement)
    }

    fn parse_ts_export_assignment(&mut self, start: u32) -> Option<Stmt> {
        self.advance()?;
        let expression = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_ts_export_assignment(Span::new(start, end), expression),
        )
    }

    fn parse_ts_namespace_export_declaration(&mut self, start: u32) -> Option<Stmt> {
        self.advance()?;
        if self.current_token.tag() != TokenTag::Namespace {
            self.report_expected(
                self.current_token.span(),
                "Expected 'namespace' after 'export as'",
                ReportOptions::default(),
            );
            return None;
        }
        self.advance()?;
        self.parse_namespace_export_declaration(start)
    }

    fn parse_namespace_export_declaration(&mut self, start: u32) -> Option<Stmt> {
        let id = self.parse_identifier_name_with_role(IdentifierRole::TypeScript)?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_ts_namespace_export_declaration(Span::new(start, end), id),
        )
    }

    fn parse_export_default_declaration(&mut self, start: u32) -> Option<Stmt> {
        self.advance()?;
        let part = self.parse_export_default_part()?;
        let declaration_end = part.declaration.span(&self.ast).end;
        let end = if part.needs_semi {
            self.eat_semicolon(declaration_end)?
        } else {
            declaration_end
        };
        Some(
            self.ast
                .stmt_export_default_declaration(Span::new(start, end), part.declaration),
        )
    }

    // ExportDeclaration :
    //   `export` `default` [lookahead ∉ { `function`, `async` [no LineTerminator here] `function`, `class` }] AssignmentExpression[+In, ~Yield, +Await] `;`
    fn parse_export_default_part(&mut self) -> Option<DefaultExportPart> {
        let is_ts = self.lang.is_typescript();
        let tag = self.current_token.tag();

        if tag == TokenTag::Function {
            let function = self.parse_function(
                ParseFunctionOpts {
                    is_default_export: true,
                    ..ParseFunctionOpts::default()
                },
                None,
            )?;
            return Some(DefaultExportPart {
                declaration: function.into_default_export_declaration(),
                needs_semi: false,
            });
        }

        if tag == TokenTag::Async {
            let next = self.peek_ahead()?;
            if next.tag() == TokenTag::Function && !next.has_line_terminator_before() {
                let async_start = self.current_token.start();
                self.advance()?;
                let function = self.parse_function(
                    ParseFunctionOpts {
                        is_async: true,
                        is_default_export: true,
                        ..ParseFunctionOpts::default()
                    },
                    Some(async_start),
                )?;
                return Some(DefaultExportPart {
                    declaration: function.into_default_export_declaration(),
                    needs_semi: false,
                });
            }
        }

        if tag == TokenTag::Class {
            let class = self.parse_class(
                ParseClassOpts {
                    is_default_export: true,
                    ..ParseClassOpts::default()
                },
                None,
            )?;
            return Some(DefaultExportPart {
                declaration: ExportDefaultDeclarationKind::Class(class),
                needs_semi: false,
            });
        }

        if tag == TokenTag::At {
            let (decorators_start, decorators) = self.parse_decorators()?;
            let class = self.parse_class_decorated(
                ParseClassOpts {
                    is_default_export: true,
                    ..ParseClassOpts::default()
                },
                Some(decorators_start),
                decorators,
            )?;
            return Some(DefaultExportPart {
                declaration: ExportDefaultDeclarationKind::Class(class),
                needs_semi: false,
            });
        }

        if is_ts {
            if tag == TokenTag::Abstract && self.is_abstract_class_next()? {
                let abstract_start = self.current_token.start();
                self.advance()?;
                let class = self.parse_class(
                    ParseClassOpts {
                        is_default_export: true,
                        is_abstract: true,
                        ..ParseClassOpts::default()
                    },
                    Some(abstract_start),
                )?;
                return Some(DefaultExportPart {
                    declaration: ExportDefaultDeclarationKind::Class(class),
                    needs_semi: false,
                });
            }

            if tag == TokenTag::Interface {
                let interface_start = self.current_token.start();
                let statement = self.parse_interface_declaration(interface_start, false)?;
                let NodeKindData::TsInterfaceDeclaration(interface) =
                    self.ast.kind_data(statement.node_id())
                else {
                    unreachable!()
                };
                return Some(DefaultExportPart {
                    declaration: ExportDefaultDeclarationKind::TsInterfaceDeclaration(interface),
                    needs_semi: false,
                });
            }
        }

        // The program or function/static block entry already establishes +In.
        let expression = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        Some(DefaultExportPart {
            declaration: ExportDefaultDeclarationKind::Expr(expression),
            needs_semi: true,
        })
    }

    fn is_abstract_class_next(&mut self) -> Option<bool> {
        let next = self.peek_ahead()?;
        Some(next.tag() == TokenTag::Class && !next.has_line_terminator_before())
    }

    fn is_legacy_accessibility_import(&mut self) -> Option<bool> {
        let next = self.peek_ahead()?;
        Some(next.tag() == TokenTag::Import && !next.has_line_terminator_before())
    }

    fn parse_export_all_declaration(
        &mut self,
        start: u32,
        export_kind: ImportOrExportKind,
    ) -> Option<Stmt> {
        self.advance()?;
        let exported = if self.eat(TokenTag::As)? {
            Some(self.parse_module_export_name()?)
        } else {
            None
        };
        if self.current_token.tag() != TokenTag::From {
            self.report_expected(
                self.current_token.span(),
                "Expected 'from' after export *",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Export all declarations require 'from': export * from 'module'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.advance()?;
        let source = self.parse_module_specifier()?;
        let attributes = self.parse_with_clause();
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_export_all_declaration(
            Span::new(start, end),
            exported,
            source,
            attributes,
            export_kind,
        ))
    }

    fn parse_export_with_declaration(&mut self, start: u32) -> Option<Stmt> {
        if self.lang.is_typescript() && self.is_start_of_ts_declaration()? {
            let statement = self.parse_ts_declaration()?;
            let declaration = match self.ast.stmt_data(statement) {
                StmtData::Declaration(declaration) => declaration,
                _ => {
                    self.report(
                        statement.span(&self.ast),
                        "expected a TypeScript declaration after 'export'",
                        ReportOptions::default(),
                    );
                    return None;
                }
            };
            let export_kind = self.export_kind_for_declaration(declaration);
            let end = declaration.span(&self.ast).end;
            let specifiers = SubRange::empty();
            let attributes = SubRange::empty();
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(declaration),
                specifiers,
                None,
                attributes,
                export_kind,
            ));
        }
        if self.lang.is_typescript() && self.current_token.tag() == TokenTag::Import {
            let statement = self.parse_import_declaration()?;
            let StmtData::Declaration(declaration) = self.ast.stmt_data(statement) else {
                let span = statement.span(&self.ast);
                self.report(
                    span,
                    "expected an import-equals declaration after 'export'",
                    ReportOptions::default(),
                );
                statement.set_span(&mut self.ast, Span::new(start, span.end));
                return Some(statement);
            };
            let DeclData::TsImportEqualsDeclaration(_) = self.ast.decl_data(declaration) else {
                unreachable!()
            };
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, declaration.span(&self.ast).end),
                Some(declaration),
                SubRange::empty(),
                None,
                SubRange::empty(),
                ImportOrExportKind::Value,
            ));
        }
        if self.lang.is_typescript()
            && matches!(
                self.current_token.tag(),
                TokenTag::Public | TokenTag::Private | TokenTag::Static
            )
            && self.is_legacy_accessibility_import()?
        {
            let declaration_start = self.current_token.start();
            self.advance()?;
            let statement = self.parse_import_declaration_from(declaration_start)?;
            let StmtData::Declaration(declaration) = self.ast.stmt_data(statement) else {
                unreachable!()
            };
            let DeclData::TsImportEqualsDeclaration(_) = self.ast.decl_data(declaration) else {
                unreachable!()
            };
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, declaration.span(&self.ast).end),
                Some(declaration),
                SubRange::empty(),
                None,
                SubRange::empty(),
                ImportOrExportKind::Value,
            ));
        }
        if matches!(
            self.current_token.tag(),
            TokenTag::Var | TokenTag::Let | TokenTag::Const
        ) {
            let declaration_start = self.current_token.start();
            let kind = self.parse_variable_kind()?;
            let declaration = self.parse_variable_declaration(kind, false, declaration_start)?;
            let end = declaration.span(&self.ast).end;
            let specifiers = SubRange::empty();
            let attributes = SubRange::empty();
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(Decl::VariableDeclaration(declaration)),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ));
        }
        if self.current_token.tag() == TokenTag::At {
            return self.parse_decorated_export_class(start, false, SubRange::empty());
        }
        if matches!(
            self.current_token.tag(),
            TokenTag::Function | TokenTag::Async | TokenTag::Class
        ) {
            let declaration = if self.current_token.tag() == TokenTag::Class {
                Decl::Class(self.parse_class(ParseClassOpts::default(), None)?)
            } else {
                let is_async = self.current_token.tag() == TokenTag::Async;
                let function_start = if is_async {
                    let start = self.current_token.start();
                    self.advance()?;
                    Some(start)
                } else {
                    None
                };
                self.parse_function(
                    ParseFunctionOpts {
                        is_async,
                        ..ParseFunctionOpts::default()
                    },
                    function_start,
                )?
                .into_declaration()
            };
            let end = declaration.span(&self.ast).end;
            let specifiers = SubRange::empty();
            let attributes = SubRange::empty();
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(declaration),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ));
        }
        self.report_missing_export_declaration()
    }

    #[cold]
    #[inline(never)]
    fn report_missing_export_declaration(&mut self) -> Option<Stmt> {
        self.report_expected(
            self.current_token.span(),
            "Expected declaration after 'export'",
            ReportOptions::default(),
        );
        None
    }

    #[inline]
    fn export_kind_for_declaration(&self, declaration: Decl) -> ImportOrExportKind {
        match self.ast.decl_data(declaration) {
            DeclData::TsInterfaceDeclaration(_) | DeclData::TsTypeAliasDeclaration(_) => {
                ImportOrExportKind::Type
            }
            DeclData::TsEnumDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(&self.ast))
            }
            DeclData::TsModuleDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(&self.ast))
            }
            DeclData::TsGlobalDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(&self.ast))
            }
            DeclData::VariableDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(&self.ast))
            }
            DeclData::Function(_) => ImportOrExportKind::Value,
            DeclData::TsFunction(node) => {
                import_or_export_kind_for_declare(node.declare(&self.ast))
            }
            DeclData::Class(node) => import_or_export_kind_for_declare(node.declare(&self.ast)),
            DeclData::TsImportEqualsDeclaration(_) => ImportOrExportKind::Value,
        }
    }

    fn parse_export_named_from_clause(
        &mut self,
        start: u32,
        export_kind: ImportOrExportKind,
    ) -> Option<Stmt> {
        let specifiers = self.parse_export_specifiers()?;
        let (source, attributes) = if self.eat(TokenTag::From)? {
            let source = self.parse_module_specifier()?;
            let attributes = self.parse_with_clause();
            (Some(source), attributes)
        } else {
            self.resolve_local_export_specifiers(specifiers)?;
            (None, SubRange::empty())
        };
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_export_named_declaration(
            Span::new(start, end),
            None,
            specifiers,
            source,
            attributes,
            export_kind,
        ))
    }

    fn parse_export_specifiers(&mut self) -> Option<SubRange<ExportSpecifier>> {
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start export specifiers",
            None,
        ) {
            return None;
        }
        let checkpoint = self.scratch_a.begin();
        self.export_specifier_tags.clear();
        let result = (|| {
            while self.current_token.tag() != TokenTag::RightBrace
                && self.current_token.tag() != TokenTag::Eof
            {
                self.export_specifier_tags.push(self.current_token.tag());
                let specifier = self.parse_export_specifier()?;
                self.scratch_a.append(specifier);
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            if !self.expect(
                TokenTag::RightBrace,
                "Expected '}' to close export specifiers",
                None,
            ) {
                return None;
            }
            Some(self.add_extra_from_scratch_a(checkpoint))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_export_specifier(&mut self) -> Option<ExportSpecifier> {
        let start = self.current_token.start();
        let parts = self.parse_specifier_parts()?;
        let local = parts.property_name.unwrap_or(parts.name);
        let exported = parts.name;
        Some(self.ast.export_specifier(
            Span::new(start, exported.span(&self.ast).end),
            local,
            exported,
            parts.kind,
        ))
    }

    fn resolve_local_export_specifiers(
        &mut self,
        specifiers: SubRange<ExportSpecifier>,
    ) -> Option<()> {
        for index in 0..self.export_specifier_tags.len() {
            let local_tag = self.export_specifier_tags[index];
            let specifier = specifiers
                .get_node(&self.ast, index)
                .expect("export specifier range contains its nodes");
            let local = specifier.local(&self.ast);
            if local.is_string_literal(&self.ast) {
                self.report(
                    local.span(&self.ast),
                    "A string literal cannot be used as an exported binding without 'from'",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Use: export { \"name\" } from 'some-module' or export { localName as \"name\" }",
                        )),
                        ..ReportOptions::default()
                    },
                );
                continue;
            }
            if local_tag.is_reserved() {
                self.report(
                    local.span(&self.ast),
                    "A reserved word cannot be used as an exported binding without 'from'",
                    ReportOptions::default(),
                );
            }
        }
        Some(())
    }

    fn parse_with_clause(&mut self) -> SubRange<ImportAttribute> {
        match self.current_token.tag() {
            TokenTag::With => {}
            TokenTag::Assert if !self.current_token.has_line_terminator_before() => {}
            _ => return SubRange::empty(),
        }
        if self.advance().is_none() {
            return SubRange::empty();
        }
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' after 'with' in import attributes",
            None,
        ) {
            return SubRange::empty();
        }
        let checkpoint = self.scratch_a.begin();
        while self.current_token.tag() != TokenTag::RightBrace
            && self.current_token.tag() != TokenTag::Eof
        {
            let attribute = match self.parse_import_attribute() {
                Some(attribute) => attribute,
                None => {
                    self.scratch_a.reset(checkpoint);
                    return SubRange::empty();
                }
            };
            self.scratch_a.append(attribute);
            if self.current_token.tag() == TokenTag::Comma {
                if self.advance().is_none() {
                    self.scratch_a.reset(checkpoint);
                    return SubRange::empty();
                }
            } else {
                break;
            }
        }
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close import attributes",
            None,
        ) {
            self.scratch_a.reset(checkpoint);
            return SubRange::empty();
        }
        let attributes = self.add_extra_from_scratch_a(checkpoint);
        self.scratch_a.reset(checkpoint);
        attributes
    }

    fn parse_import_attribute(&mut self) -> Option<ImportAttribute> {
        let start = self.current_token.start();
        let key = self.parse_attribute_key()?;
        if !self.expect(TokenTag::Colon, "Expected ':' in import attribute", None) {
            return None;
        }
        if self.current_token.tag() != TokenTag::StringLiteral {
            self.report(
                self.current_token.span(),
                "Import attribute value must be a string literal",
                ReportOptions::default(),
            );
            return None;
        }
        let value = self.parse_string_literal()?;
        Some(
            self.ast
                .import_attribute(Span::new(start, value.span(&self.ast).end), key, value),
        )
    }

    fn parse_attribute_key(&mut self) -> Option<ModuleExportName> {
        let token = self.current_token;
        if token.tag() == TokenTag::StringLiteral {
            return Some(ModuleExportName::StringLiteral(
                self.parse_string_literal()?,
            ));
        }
        if token.tag().is_identifier_like() {
            return Some(ModuleExportName::IdentifierName(
                self.parse_identifier_name()?,
            ));
        }
        self.report_expected(
            self.current_token.span(),
            "Expected identifier or string literal for attribute key",
            ReportOptions::default(),
        );
        None
    }

    fn parse_module_export_name(&mut self) -> Option<ModuleExportName> {
        let token = self.current_token;
        if token.tag() == TokenTag::StringLiteral {
            let name = self.parse_string_literal()?;
            // Line continuations can separate surrogate escapes without adding code units.
            if self.ast.get_wtf8(name.value(&self.ast)).as_str().is_none() {
                self.report(
                    token.span(),
                    "An export name cannot include a unicode lone surrogate",
                    ReportOptions::default(),
                );
            }
            return Some(ModuleExportName::StringLiteral(name));
        }
        if token.tag().is_identifier_like() {
            return Some(ModuleExportName::IdentifierName(
                self.parse_identifier_name()?,
            ));
        }
        self.report_expected(
            self.current_token.span(),
            "Expected identifier or string literal",
            ReportOptions::default(),
        );
        None
    }

    fn is_start_of_ts_declaration(&mut self) -> Option<bool> {
        if !self.lang.is_typescript()
            || !matches!(
                self.current_token.tag(),
                TokenTag::Declare
                    | TokenTag::Abstract
                    | TokenTag::Const
                    | TokenTag::Enum
                    | TokenTag::Type
                    | TokenTag::Interface
                    | TokenTag::Namespace
                    | TokenTag::Module
                    | TokenTag::Global
            )
        {
            return Some(false);
        }
        let mut current = self.current_token;
        let mut peek = self.begin_peek();
        let mut has_declare = false;
        let mut has_abstract = false;

        if current.tag() == TokenTag::Declare {
            current = peek.next()?;
            if current.has_line_terminator_before() {
                peek.end();
                return Some(false);
            }
            has_declare = true;
        }
        if current.tag() == TokenTag::Abstract {
            let next = peek.next()?;
            if next.has_line_terminator_before() || next.tag() != TokenTag::Class {
                peek.end();
                return Some(false);
            }
            current = next;
            has_abstract = true;
        }
        let result = match current.tag() {
            TokenTag::Const => has_declare || is_const_enum_head(peek.next()?),
            TokenTag::Enum => {
                let name = peek.next()?;
                name.tag().is_identifier_like() && !name.tag().is_unconditionally_reserved()
            }
            TokenTag::Type | TokenTag::Interface | TokenTag::Namespace => {
                is_declaration_name(peek.next()?)
            }
            TokenTag::Module => {
                let name = peek.next()?;
                !name.has_line_terminator_before()
                    && (is_declaration_name(name) || name.tag() == TokenTag::StringLiteral)
            }
            TokenTag::Global => {
                let next = peek.next()?;
                next.tag() == TokenTag::LeftBrace
            }
            TokenTag::Class => has_declare || has_abstract,
            TokenTag::Function => has_declare,
            TokenTag::Var | TokenTag::Let => has_declare,
            TokenTag::Async => {
                if !has_declare {
                    false
                } else {
                    let function = peek.next()?;
                    if function.tag() != TokenTag::Function || function.has_line_terminator_before()
                    {
                        false
                    } else {
                        is_declaration_name(peek.next()?)
                    }
                }
            }
            TokenTag::Import => {
                if !has_declare {
                    false
                } else {
                    let name = peek.next()?;
                    if !is_declaration_name(name) {
                        false
                    } else {
                        let assign = peek.next()?;
                        assign.tag() == TokenTag::Assign && !assign.has_line_terminator_before()
                    }
                }
            }
            _ => false,
        };
        peek.end();
        Some(result)
    }

    fn parse_ts_declaration(&mut self) -> Option<Stmt> {
        let start = self.current_token.start();
        let mut modifiers = TsDeclarationModifiers::default();
        if self.eat(TokenTag::Declare)? {
            modifiers.declare = true;
        }
        if self.eat(TokenTag::Abstract)? {
            modifiers.r#abstract = true;
        }
        if self.current_token.tag() == TokenTag::Const {
            let next = self.peek_ahead()?;
            if is_const_enum_head(next) {
                modifiers.is_const = true;
                self.advance()?;
            }
        }

        let ambient = self.ts_context.ambient();
        self.ts_context
            .set(TsContext::AMBIENT, ambient || modifiers.declare);
        let result = (|| match self.current_token.tag() {
            TokenTag::Type => self.parse_type_alias_declaration(start, modifiers.declare),
            TokenTag::Interface => self.parse_interface_declaration(start, modifiers.declare),
            TokenTag::Enum => {
                self.parse_enum_declaration(start, modifiers.is_const, modifiers.declare)
            }
            TokenTag::Var | TokenTag::Let | TokenTag::Const => {
                let kind = self.parse_variable_kind()?;
                let declaration =
                    self.parse_variable_declaration(kind, modifiers.declare, start)?;
                Some(Stmt::Declaration(Decl::VariableDeclaration(declaration)))
            }
            TokenTag::Function => Some(Stmt::Declaration(
                self.parse_function(
                    ParseFunctionOpts {
                        is_declare: modifiers.declare,
                        ..ParseFunctionOpts::default()
                    },
                    Some(start),
                )?
                .into_declaration(),
            )),
            TokenTag::Async => {
                self.advance()?;
                Some(Stmt::Declaration(
                    self.parse_function(
                        ParseFunctionOpts {
                            is_async: true,
                            is_declare: modifiers.declare,
                            ..ParseFunctionOpts::default()
                        },
                        Some(start),
                    )?
                    .into_declaration(),
                ))
            }
            TokenTag::Class => Some(Stmt::Declaration(Decl::Class(self.parse_class(
                ParseClassOpts {
                    is_declare: modifiers.declare,
                    is_abstract: modifiers.r#abstract,
                    ..ParseClassOpts::default()
                },
                Some(start),
            )?))),
            TokenTag::Import => self.parse_declare_import_equals(start),
            TokenTag::Namespace => self.parse_module_declaration(
                start,
                TsModuleDeclarationKind::Namespace,
                modifiers.declare,
            ),
            TokenTag::Module => self.parse_module_declaration(
                start,
                TsModuleDeclarationKind::Module,
                modifiers.declare,
            ),
            TokenTag::Global => self.parse_global_declaration(start, modifiers.declare),
            _ => None,
        })();
        self.ts_context.set(TsContext::AMBIENT, ambient);
        result
    }

    fn parse_declare_import_equals(&mut self, start: u32) -> Option<Stmt> {
        self.advance()?;
        self.parse_import_equals_body(start, ImportOrExportKind::Value)
    }

    fn parse_type_alias_declaration(&mut self, start: u32, declare: bool) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Type);
        self.advance()?;
        let id = self.parse_binding_identifier()?;
        let type_parameters = if is_angle_open(self.current_token.tag()) {
            Some(self.parse_type_parameters(TypeParameterContext::Declaration, AngleClose::Peel)?)
        } else {
            None
        };
        if !self.expect(
            TokenTag::Assign,
            "Expected '=' in type alias declaration",
            Some("A type alias is written 'type Foo = Bar'"),
        ) {
            return None;
        }
        let type_annotation = self.parse_type_alias_body()?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        let declaration = self.ast.ts_type_alias_declaration(
            Span::new(start, end),
            id,
            type_parameters,
            type_annotation,
            declare,
        );
        Some(Stmt::Declaration(Decl::TsTypeAliasDeclaration(declaration)))
    }

    fn parse_interface_declaration(&mut self, start: u32, declare: bool) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Interface);
        self.advance()?;
        let id = self.parse_binding_identifier()?;
        let type_parameters = if is_angle_open(self.current_token.tag()) {
            Some(self.parse_type_parameters(TypeParameterContext::Declaration, AngleClose::Peel)?)
        } else {
            None
        };
        let extends =
            match self.parse_heritage_clause(TokenTag::Extends, HeritageKind::Interface)? {
                HeritageClause::Interface(extends) => extends,
                HeritageClause::Class(_) => unreachable!(),
            };
        let body = self.parse_interface_body()?;
        let end = body.span(&self.ast).end;
        let declaration = self.ast.ts_interface_declaration(
            Span::new(start, end),
            id,
            type_parameters,
            extends,
            body,
            declare,
        );
        Some(Stmt::Declaration(Decl::TsInterfaceDeclaration(declaration)))
    }

    fn parse_heritage_clause(
        &mut self,
        keyword: TokenTag,
        kind: HeritageKind,
    ) -> Option<HeritageClause> {
        if !self.eat(keyword)? {
            return Some(match kind {
                HeritageKind::Interface => HeritageClause::Interface(SubRange::empty()),
                HeritageKind::Class => HeritageClause::Class(SubRange::empty()),
            });
        }
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            loop {
                match self.parse_heritage_entry(kind)? {
                    HeritageEntry::Interface(node) => self.scratch_a.append(node),
                    HeritageEntry::Class(node) => self.scratch_a.append(node),
                }
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
                if self.current_token.tag() == TokenTag::LeftBrace {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected a heritage type after ','",
                        ReportOptions::default(),
                    );
                    break;
                }
            }
            Some(match kind {
                HeritageKind::Interface => {
                    HeritageClause::Interface(self.add_extra_from_scratch_a(checkpoint))
                }
                HeritageKind::Class => {
                    HeritageClause::Class(self.add_extra_from_scratch_a(checkpoint))
                }
            })
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_heritage_entry(&mut self, kind: HeritageKind) -> Option<HeritageEntry> {
        let expression = self.parse_heritage_expression()?;
        let type_arguments = if is_angle_open(self.current_token.tag()) {
            Some(self.parse_type_arguments()?)
        } else {
            None
        };
        let end = type_arguments.map_or(expression.span(&self.ast).end, |arguments| {
            arguments.span(&self.ast).end
        });
        let span = Span::new(expression.span(&self.ast).start, end);
        Some(match kind {
            HeritageKind::Interface => HeritageEntry::Interface(self.ast.ts_interface_heritage(
                span,
                expression,
                type_arguments,
            )),
            HeritageKind::Class => HeritageEntry::Class(self.ast.ts_class_implements(
                span,
                expression,
                type_arguments,
            )),
        })
    }

    fn parse_interface_body(&mut self) -> Option<TsInterfaceBody> {
        if self.current_token.tag() != TokenTag::LeftBrace {
            self.report_expected(
                self.current_token.span(),
                "Expected '{' to start an interface body",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "An interface body is written '{ member1; member2 }'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let start = self.current_token.start();
        self.advance()?;
        let (body, end) = self.parse_object_type_members(TypeParameterContext::Declaration)?;
        Some(self.ast.ts_interface_body(Span::new(start, end), body))
    }

    fn parse_enum_declaration(
        &mut self,
        start: u32,
        is_const: bool,
        declare: bool,
    ) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Enum);
        self.advance()?;
        let id = self.parse_binding_identifier()?;
        let body = self.parse_enum_body()?;
        let end = body.span(&self.ast).end;
        let declaration =
            self.ast
                .ts_enum_declaration(Span::new(start, end), id, body, is_const, declare);
        Some(Stmt::Declaration(Decl::TsEnumDeclaration(declaration)))
    }

    fn parse_enum_body(&mut self) -> Option<TsEnumBody> {
        let start = self.current_token.start();
        if self.current_token.tag() != TokenTag::LeftBrace {
            self.report_expected(
                self.current_token.span(),
                "Expected '{' to start an enum body",
                ReportOptions {
                    help: Some(Cow::Borrowed("An enum body is written '{ A, B = 1 }'")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.advance()?;
        let checkpoint = self.scratch_a.begin();
        let result = self.with_scope(self.boundary_context.ts_container(), |parser| {
            enter_grammar_context![&mut parser.context, ~Yield, ~Await];
            parser.ts_context.insert(TsContext::IN_TS_CONTAINER_BODY);
            while parser.current_token.tag() != TokenTag::RightBrace
                && parser.current_token.tag() != TokenTag::Eof
            {
                let member = parser.parse_enum_member()?;
                parser.scratch_a.append(member);
                if !parser.eat(TokenTag::Comma)? {
                    break;
                }
            }
            let end = parser.current_token.end();
            if !parser.expect(
                TokenTag::RightBrace,
                "Expected '}' to close an enum body",
                Some("Each '{' in an enum must be matched by a '}'"),
            ) {
                return None;
            }
            let members = parser.add_extra_from_scratch_a(checkpoint);
            Some(parser.ast.ts_enum_body(Span::new(start, end), members))
        });
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_enum_member(&mut self) -> Option<TsEnumMember> {
        let (id, computed) = self.parse_enum_member_name()?;
        let start = id.span(&self.ast).start;
        let initializer = if self.eat(TokenTag::Assign)? {
            // The program or function/static block entry already establishes +In.
            Some(self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?)
        } else {
            None
        };
        let end = initializer.map_or_else(
            || {
                if computed {
                    self.lexer.state.prev_token_end
                } else {
                    id.span(&self.ast).end
                }
            },
            |expression| expression.span(&self.ast).end,
        );
        Some(
            self.ast
                .ts_enum_member(Span::new(start, end), id, initializer, computed),
        )
    }

    fn parse_enum_member_name(&mut self) -> Option<(TsEnumMemberName, bool)> {
        let computed = self.eat(TokenTag::LeftBracket)?;
        let token = self.current_token;
        let name = match token.tag() {
            tag if !computed && tag.is_identifier_like() => {
                TsEnumMemberName::IdentifierName(self.parse_identifier_name()?)
            }
            TokenTag::NoSubstitutionTemplate => {
                if !computed {
                    self.report(
                        token.span(),
                        "Template literal enum member names must be computed",
                        ReportOptions::default(),
                    );
                }
                TsEnumMemberName::TemplateLiteral(self.parse_no_substitution_template(false)?)
            }
            TokenTag::StringLiteral => {
                TsEnumMemberName::StringLiteral(self.parse_string_literal()?)
            }
            _ => {
                let found = self.describe_token(token);
                self.report(
                    token.span(),
                    Cow::Owned(format!(
                        "Unexpected token '{}' as enum member name",
                        found.as_ref()
                    )),
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Enum member names must be identifiers, string literals, or no-substitution templates.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
        };
        if computed
            && !self.expect(
                TokenTag::RightBracket,
                "Expected ']' to close a computed enum member name",
                Some("A computed enum member name is written '[\"name\"]'"),
            )
        {
            return None;
        }
        Some((name, computed))
    }

    fn parse_module_declaration(
        &mut self,
        start: u32,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> Option<Stmt> {
        self.advance()?;
        let id = if kind == TsModuleDeclarationKind::Module
            && self.current_token.tag() == TokenTag::StringLiteral
        {
            TsModuleName::StringLiteral(self.parse_string_literal()?)
        } else {
            self.parse_module_name()?
        };
        let outer_ambient = self.ts_context.ambient();
        self.ts_context
            .set(TsContext::AMBIENT, outer_ambient || declare);
        let body = self.parse_optional_module_block();
        self.ts_context.set(TsContext::AMBIENT, outer_ambient);
        let body = body?;
        let end = if let Some(body) = body {
            body.span(&self.ast).end
        } else {
            self.eat_semicolon(self.lexer.state.prev_token_end)?
        };
        let declaration =
            self.ast
                .ts_module_declaration(Span::new(start, end), id, body, kind, declare);
        Some(Stmt::Declaration(Decl::TsModuleDeclaration(declaration)))
    }

    fn parse_global_declaration(&mut self, start: u32, declare: bool) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Global);
        let token = self.current_token;
        self.advance()?;
        let name = self.identifier_name(token);
        let id = self.ast.identifier_name(token.span(), name);
        let outer_ambient = self.ts_context.ambient();
        self.ts_context
            .set(TsContext::AMBIENT, outer_ambient || declare);
        let body = self.parse_module_block();
        self.ts_context.set(TsContext::AMBIENT, outer_ambient);
        let body = body?;
        let end = body.span(&self.ast).end;
        let declaration = self
            .ast
            .ts_global_declaration(Span::new(start, end), id, body, declare);
        Some(Stmt::Declaration(Decl::TsGlobalDeclaration(declaration)))
    }

    fn parse_module_name(&mut self) -> Option<TsModuleName> {
        let head = self.parse_binding_identifier()?;
        if self.current_token.tag() != TokenTag::Dot {
            return Some(TsModuleName::BindingIdentifier(head));
        }
        let start = head.span(&self.ast).start;
        let left = TsQualifiedNameLeft::BindingIdentifier(head);
        let qualified = self.extend_qualified_name(left, start)?;
        Some(TsModuleName::TsQualifiedName(
            qualified
                .as_ts_qualified_name(&self.ast)
                .expect("a dotted module name is qualified"),
        ))
    }

    fn parse_optional_module_block(&mut self) -> Option<Option<TsModuleBody>> {
        if self.current_token.tag() != TokenTag::LeftBrace {
            return Some(None);
        }
        Some(Some(TsModuleBody::TsModuleBlock(
            self.parse_module_block()?,
        )))
    }

    fn parse_module_block(&mut self) -> Option<TsModuleBlock> {
        let start = self.current_token.start();
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start a module body",
            Some("A module body is written '{ <statements> }'"),
        ) {
            return None;
        }
        // A TypeScript module block is a declaration container, not a function
        // body, so it does not inherit an enclosing function's [Return].
        let body = self.with_scope(self.boundary_context.ts_container(), |parser| {
            enter_grammar_context![&mut parser.context, ~Await, ~Return];
            parser.ts_context.insert(TsContext::IN_TS_CONTAINER_BODY);
            parser.parse_body(Some(TokenTag::RightBrace), StatementContext::ModuleItem)
        });
        let ParsedBody { directives, body } = body?;
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close a module body",
            Some("Each '{' in a module must be matched by a '}'"),
        ) {
            return None;
        }
        Some(
            self.ast
                .ts_module_block(Span::new(start, end), directives, body),
        )
    }

    fn parse_type_alias_body(&mut self) -> Option<TsType> {
        let token = self.current_token;
        if !token.is_keyword(TokenTag::Intrinsic) || self.peek_ahead()?.tag() == TokenTag::Dot {
            return self.parse_type();
        }
        self.advance()?;
        let intrinsic = self.ast.ts_type_ts_intrinsic_keyword(token.span());
        if self.current_token.tag() == TokenTag::Semicolon
            || self.can_insert_implicit_semicolon(self.current_token)
        {
            return Some(intrinsic);
        }

        self.recover_intrinsic_type_tail(token, intrinsic)
    }

    fn recover_intrinsic_type_tail(&mut self, token: Token, intrinsic: TsType) -> Option<TsType> {
        // Intrinsic is the complete alias body. Diagnose a continuation, but
        // retain type structure when recovery can represent it. A newline
        // ends the alias above, so `intrinsic\n[]` remains two statements.
        self.report_expected(
            self.current_token.span(),
            "Expected a semicolon after an intrinsic type",
            ReportOptions::default(),
        );
        let primary = if is_angle_open(self.current_token.tag()) {
            // Only a reference can carry type arguments in the AST.
            let name = self.identifier_name(token);
            let name = self
                .ast
                .ts_type_name_identifier_reference(token.span(), name);
            self.parse_type_reference(token.start(), name)?
        } else {
            intrinsic
        };
        self.parse_type_from_primary(primary)
    }

    fn parse_type_annotation(&mut self) -> Option<TsTypeAnnotation> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Colon);
        let start = self.current_token.start();
        self.advance()?;
        let type_annotation = self.parse_type()?;
        let end = type_annotation.span(&self.ast).end;
        Some(
            self.ast
                .ts_type_annotation(Span::new(start, end), type_annotation),
        )
    }

    fn parse_return_type_annotation(&mut self) -> Option<TsTypeAnnotation> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Colon);
        let start = self.current_token.start();
        self.advance()?;
        let type_annotation = self.parse_type_or_type_predicate()?;
        let end = type_annotation.span(&self.ast).end;
        Some(
            self.ast
                .ts_type_annotation(Span::new(start, end), type_annotation),
        )
    }

    fn parse_type_or_type_predicate(&mut self) -> Option<TsType> {
        let token = self.current_token;
        // Share the reference/predicate prefix, retaining lookahead recovery
        // when the next token cannot be scanned.
        if token.tag() == TokenTag::Identifier && self.try_advance() {
            if self.current_token.is_keyword(TokenTag::Is)
                && !self.current_token.has_line_terminator_before()
            {
                let name = self.identifier_name(token);
                let parameter_name = TsTypePredicateName::IdentifierName(
                    self.ast.identifier_name(token.span(), name),
                );
                return self.finish_type_predicate(token.start(), parameter_name, false);
            }
            return self.parse_type_from_identifier(token);
        }
        if !self.is_identifier_predicate_start()? {
            return self.parse_type();
        }
        let parameter_name = TsTypePredicateName::IdentifierName(
            self.parse_identifier_name_with_role(IdentifierRole::TypeScript)?,
        );
        self.finish_type_predicate(parameter_name.span(&self.ast).start, parameter_name, false)
    }

    fn parse_this_type_or_predicate(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::This);
        let token = self.current_token;
        self.advance()?;
        if self.current_token.tag() != TokenTag::Is
            || self.current_token.is_escaped()
            || self.current_token.has_line_terminator_before()
        {
            return Some(self.ast.ts_type_ts_this_type(token.span()));
        }
        let parameter_name = self.ast.ts_type_predicate_name_ts_this_type(token.span());
        self.finish_type_predicate(token.start(), parameter_name, false)
    }

    fn parse_asserts_type_predicate(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Asserts);
        let start = self.current_token.start();
        self.advance()?;
        let parameter_name = self.parse_predicate_parameter_name()?;
        self.finish_type_predicate(start, parameter_name, true)
    }

    fn parse_predicate_parameter_name(&mut self) -> Option<TsTypePredicateName> {
        let token = self.current_token;
        if token.tag() == TokenTag::This {
            self.advance()?;
            return Some(self.ast.ts_type_predicate_name_ts_this_type(token.span()));
        }
        Some(TsTypePredicateName::IdentifierName(
            self.parse_identifier_name_with_role(IdentifierRole::TypeScript)?,
        ))
    }

    fn finish_type_predicate(
        &mut self,
        start: u32,
        parameter_name: TsTypePredicateName,
        asserts: bool,
    ) -> Option<TsType> {
        let mut end = parameter_name.span(&self.ast).end;
        let type_annotation = if self.current_token.is_keyword(TokenTag::Is) {
            self.advance()?;
            let inner = self.parse_type()?;
            end = inner.span(&self.ast).end;
            Some(self.ast.ts_type_annotation(inner.span(&self.ast), inner))
        } else {
            None
        };
        Some(TsType::TsTypePredicate(self.ast.ts_type_predicate(
            Span::new(start, end),
            parameter_name,
            type_annotation,
            asserts,
        )))
    }

    fn is_asserts_predicate_start(&mut self) -> Option<bool> {
        if !self.current_token.is_keyword(TokenTag::Asserts) {
            return Some(false);
        }
        let next = self.peek_ahead()?;
        Some(
            !next.has_line_terminator_before()
                && (next.tag() == TokenTag::This || next.tag().is_identifier_like()),
        )
    }

    fn is_identifier_predicate_start(&mut self) -> Option<bool> {
        let current = self.current_token;
        if current.tag() == TokenTag::This || !current.tag().is_identifier_like() {
            return Some(false);
        }
        if !self.next_token_may_start_with(|byte| byte == b'i') {
            return Some(false);
        }
        let next = self.peek_ahead()?;
        Some(next.is_keyword(TokenTag::Is) && !next.has_line_terminator_before())
    }

    fn parse_type(&mut self) -> Option<TsType> {
        self.parse_type_with(Self::parse_type_inner)
    }

    fn parse_type_with(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Option<TsType>,
    ) -> Option<TsType> {
        let disallow_conditional_types = self.ts_context.disallow_conditional_types();
        if disallow_conditional_types {
            self.ts_context
                .remove(TsContext::DISALLOW_CONDITIONAL_TYPES);
        }
        let result = parse(self);
        if disallow_conditional_types {
            self.ts_context
                .insert(TsContext::DISALLOW_CONDITIONAL_TYPES);
        }
        result
    }

    /// Lists and conditional branches require a type start, unlike a bare
    /// TypeReference. Keep recoverable reserved-word references in the AST.
    fn parse_type_with_start_check(&mut self) -> Option<TsType> {
        self.with_type_start_check(Self::parse_type)
    }

    fn with_type_start_check<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Option<T> {
        let token = self.current_token;
        // Non-reserved IdentifierNames already satisfy the type-start predicate.
        if token.tag().is_unconditionally_reserved() && !is_start_of_type(token.tag()) {
            self.report_expected(token.span(), "Expected a type", ReportOptions::default());
        }
        parse(self)
    }

    fn parse_type_no_conditional(&mut self) -> Option<TsType> {
        let saved = self.ts_context.disallow_conditional_types();
        self.ts_context
            .insert(TsContext::DISALLOW_CONDITIONAL_TYPES);
        let result = self.parse_type_inner();
        self.ts_context
            .set(TsContext::DISALLOW_CONDITIONAL_TYPES, saved);
        result
    }

    fn parse_type_inner(&mut self) -> Option<TsType> {
        let tag = self.current_token.tag();
        let may_start_function_or_constructor_type = matches!(
            tag,
            TokenTag::New | TokenTag::Abstract | TokenTag::LeftParen
        ) || is_angle_open(tag);
        if may_start_function_or_constructor_type
            && self.is_start_of_function_or_constructor_type()?
        {
            self.parse_function_or_constructor_type()
        } else {
            // A leading `|` or `&` has no primary yet. Other entries share the
            // same tail parser as completed generic references.
            let result = if matches!(tag, TokenTag::BitwiseOr | TokenTag::BitwiseAnd) {
                self.parse_conditional_type()
            } else {
                self.parse_primary_type()
                    .and_then(|primary| self.parse_type_from_primary(primary))
            };
            result.or_else(|| {
                self.report_expected(
                    self.current_token.span(),
                    "Expected a type",
                    ReportOptions::default(),
                );
                None
            })
        }
    }

    fn parse_conditional_type(&mut self) -> Option<TsType> {
        self.parse_conditional_type_with(Self::parse_union_type)
    }

    fn parse_conditional_type_with(
        &mut self,
        parse_check: impl FnOnce(&mut Self) -> Option<TsType>,
    ) -> Option<TsType> {
        // Specialize the first parse and fast return; share the conditional tail.
        let check_type = parse_check(self)?;
        if self.ts_context.disallow_conditional_types()
            || self.current_token.tag() != TokenTag::Extends
            || self.current_token.has_line_terminator_before()
        {
            return Some(check_type);
        }
        self.parse_conditional_type_tail(check_type)
    }

    fn parse_conditional_type_tail(&mut self, check_type: TsType) -> Option<TsType> {
        self.advance()?;
        let extends_type = self.parse_type_no_conditional()?;
        // `extends` has committed this grammar alternative. Missing separators
        // must not discard branches that can still be represented in the AST.
        self.expect_recoverable(
            TokenTag::Question,
            "Expected '?' in a conditional type",
            Some("A conditional type is written 'T extends U ? X : Y'"),
        )?;
        let true_type = self.parse_type_with_start_check()?;
        self.expect_recoverable(
            TokenTag::Colon,
            "Expected ':' in a conditional type",
            Some("A conditional type is written 'T extends U ? X : Y'"),
        )?;
        let false_type = self.parse_type_with_start_check()?;
        Some(TsType::TsConditionalType(self.ast.ts_conditional_type(
            Span::new(
                check_type.span(&self.ast).start,
                false_type.span(&self.ast).end,
            ),
            check_type,
            extends_type,
            true_type,
            false_type,
        )))
    }

    fn parse_union_type(&mut self) -> Option<TsType> {
        self.parse_binary_type_chain(
            TokenTag::BitwiseOr,
            BinaryTypeKind::Union,
            Self::parse_intersection_type,
        )
    }

    fn parse_binary_type_chain<F>(
        &mut self,
        separator: TokenTag,
        kind: BinaryTypeKind,
        parse_constituent: F,
    ) -> Option<TsType>
    where
        F: Fn(&mut Self) -> Option<TsType>,
    {
        let leading_start = if self.current_token.tag() == separator {
            let start = self.current_token.start();
            self.advance()?;
            Some(start)
        } else {
            None
        };
        let first_start = self.current_token.start();
        let first = parse_constituent(self)?;
        if leading_start.is_none() && self.current_token.tag() != separator {
            return Some(first);
        }
        self.parse_binary_type_chain_from(
            first,
            leading_start.unwrap_or(first_start),
            separator,
            kind,
            parse_constituent,
        )
    }

    fn parse_binary_type_chain_from<F>(
        &mut self,
        first: TsType,
        start: u32,
        separator: TokenTag,
        kind: BinaryTypeKind,
        parse_constituent: F,
    ) -> Option<TsType>
    where
        F: Fn(&mut Self) -> Option<TsType>,
    {
        let checkpoint = self.scratch_a.begin();
        self.scratch_a.append(first);
        let result = (|| {
            while self.eat(separator)? {
                let next = parse_constituent(self)?;
                self.scratch_a.append(next);
            }
            let span = Span::new(start, self.lexer.state.prev_token_end);
            let types = self.add_extra_from_scratch_a(checkpoint);
            Some(match kind {
                BinaryTypeKind::Union => TsType::TsUnionType(self.ast.ts_union_type(span, types)),
                BinaryTypeKind::Intersection => {
                    TsType::TsIntersectionType(self.ast.ts_intersection_type(span, types))
                }
            })
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_intersection_type(&mut self) -> Option<TsType> {
        self.parse_binary_type_chain(
            TokenTag::BitwiseAnd,
            BinaryTypeKind::Intersection,
            Self::parse_postfix_type,
        )
    }

    fn parse_postfix_type(&mut self) -> Option<TsType> {
        self.parse_postfix_type_with(Self::parse_primary_type)
    }

    fn parse_postfix_type_with(
        &mut self,
        parse_primary: impl FnOnce(&mut Self) -> Option<TsType>,
    ) -> Option<TsType> {
        let mut type_annotation = parse_primary(self)?;
        loop {
            let token = self.current_token;
            match token.tag() {
                TokenTag::LeftBracket if !token.has_line_terminator_before() => {
                    type_annotation = self.parse_array_or_indexed_access_type(type_annotation)?;
                }
                TokenTag::Question if !token.has_line_terminator_before() => {
                    if !self.is_postfix_nullable()? {
                        break;
                    }
                    type_annotation =
                        self.parse_jsdoc_postfix(type_annotation, JsDocKind::Nullable)?;
                }
                TokenTag::LogicalNot if !token.has_line_terminator_before() => {
                    type_annotation =
                        self.parse_jsdoc_postfix(type_annotation, JsDocKind::NonNullable)?;
                }
                _ => break,
            }
        }
        Some(type_annotation)
    }

    fn parse_array_or_indexed_access_type(&mut self, object_type: TsType) -> Option<TsType> {
        let start = object_type.span(&self.ast).start;
        self.advance()?;
        if self.current_token.tag() == TokenTag::RightBracket {
            let end = self.current_token.end();
            self.advance()?;
            return Some(TsType::TsArrayType(
                self.ast.ts_array_type(Span::new(start, end), object_type),
            ));
        }
        let index_type = self.parse_type_with_start_check()?;
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightBracket,
            "Expected ']' to close an indexed access type",
            Some("Each '[' in a type must be matched by a ']'"),
        ) {
            return None;
        }
        Some(TsType::TsIndexedAccessType(
            self.ast
                .ts_indexed_access_type(Span::new(start, end), object_type, index_type),
        ))
    }

    fn parse_primary_type(&mut self) -> Option<TsType> {
        self.parse_primary_type_with(std::convert::identity, Self::parse_type_reference)
    }

    /// Identifier-like heads can defer reference arguments to the work stack.
    /// Other primary forms share a fixed-result parser.
    fn parse_primary_type_with<T>(
        &mut self,
        from_type: impl FnOnce(TsType) -> T,
        from_reference: impl FnOnce(&mut Self, u32, TsTypeName) -> Option<T>,
    ) -> Option<T> {
        let token = self.current_token;
        match token.tag() {
            TokenTag::True | TokenTag::False => return self.parse_literal_type().map(from_type),
            tag @ (TokenTag::Any
            | TokenTag::Bigint
            | TokenTag::Boolean
            | TokenTag::Never
            | TokenTag::NullLiteral
            | TokenTag::Number
            | TokenTag::Object
            | TokenTag::String
            | TokenTag::Symbol
            | TokenTag::Undefined
            | TokenTag::Unknown
            | TokenTag::Void)
                if !token.is_escaped() =>
            {
                if matches!(tag, TokenTag::NullLiteral | TokenTag::Void)
                    || !self.is_qualified_type_continuation()?
                {
                    return self.parse_type_keyword().map(from_type);
                }
            }
            TokenTag::This if !token.is_escaped() => {
                return self.parse_this_type_or_predicate().map(from_type);
            }
            TokenTag::Asserts if !token.is_escaped() => {
                if self.is_asserts_predicate_start()? {
                    return self.parse_asserts_type_predicate().map(from_type);
                }
            }
            TokenTag::Keyof | TokenTag::Unique | TokenTag::Readonly if !token.is_escaped() => {
                return self.parse_type_operator().map(from_type);
            }
            TokenTag::Infer if !token.is_escaped() => {
                return self.parse_infer_type().map(from_type);
            }
            TokenTag::Typeof if !token.is_escaped() => {
                return self.parse_type_query().map(from_type);
            }
            TokenTag::Import if !token.is_escaped() => {
                return self.parse_import_type().map(from_type);
            }
            _ => {}
        }
        if token.tag().is_identifier_like() {
            let name = self.parse_entity_name()?;
            return from_reference(self, token.start(), name);
        }
        self.parse_non_identifier_primary_type().map(from_type)
    }

    fn parse_non_identifier_primary_type(&mut self) -> Option<TsType> {
        let token = self.current_token;
        match token.tag() {
            TokenTag::StringLiteral
            | TokenTag::NumericLiteral
            | TokenTag::HexLiteral
            | TokenTag::OctalLiteral
            | TokenTag::BinaryLiteral
            | TokenTag::BigIntLiteral
            | TokenTag::NoSubstitutionTemplate
            | TokenTag::Minus => self.parse_literal_type(),
            TokenTag::LeftParen => self.parse_parenthesized_type(),
            TokenTag::LeftBracket => self.parse_tuple_type(),
            TokenTag::LeftBrace => {
                self.advance()?;
                if self.is_start_of_mapped_type() {
                    self.parse_mapped_type(token.start())
                } else {
                    self.parse_type_literal(token.start())
                }
            }
            TokenTag::TemplateHead => self.parse_template_literal_type(),
            TokenTag::Question if !token.is_escaped() => {
                self.parse_jsdoc_nullable_or_unknown_type()
            }
            TokenTag::LogicalNot if !token.is_escaped() => self.parse_jsdoc_non_nullable_type(),
            _ => None,
        }
    }

    fn parse_type_keyword(&mut self) -> Option<TsType> {
        let token = self.current_token;
        let keyword = match token.tag() {
            TokenTag::Any => self.ast.ts_type_ts_any_keyword(token.span()),
            TokenTag::Unknown => self.ast.ts_type_ts_unknown_keyword(token.span()),
            TokenTag::Never => self.ast.ts_type_ts_never_keyword(token.span()),
            TokenTag::Void => self.ast.ts_type_ts_void_keyword(token.span()),
            TokenTag::NullLiteral => self.ast.ts_type_ts_null_keyword(token.span()),
            TokenTag::Undefined => self.ast.ts_type_ts_undefined_keyword(token.span()),
            TokenTag::String => self.ast.ts_type_ts_string_keyword(token.span()),
            TokenTag::Number => self.ast.ts_type_ts_number_keyword(token.span()),
            TokenTag::Bigint => self.ast.ts_type_ts_big_int_keyword(token.span()),
            TokenTag::Boolean => self.ast.ts_type_ts_boolean_keyword(token.span()),
            TokenTag::Symbol => self.ast.ts_type_ts_symbol_keyword(token.span()),
            TokenTag::Object => self.ast.ts_type_ts_object_keyword(token.span()),
            _ => unreachable!(),
        };
        self.advance()?;
        Some(keyword)
    }

    fn is_postfix_nullable(&mut self) -> Option<bool> {
        let next = self.peek_ahead()?;
        Some(!is_start_of_type(next.tag()))
    }

    fn parse_jsdoc_postfix(&mut self, inner: TsType, kind: JsDocKind) -> Option<TsType> {
        let span = Span::new(inner.span(&self.ast).start, self.current_token.end());
        self.advance()?;
        Some(self.jsdoc_node_data(inner, kind, true, span))
    }

    fn parse_parenthesized_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftParen);
        let start = self.current_token.start();
        self.advance()?;
        let type_annotation = self.parse_type()?;
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' to close a parenthesized type",
            Some("Each '(' in a type must be matched by a ')'"),
        ) {
            return None;
        }
        Some(TsType::TsParenthesizedType(self.ast.ts_parenthesized_type(
            Span::new(start, end),
            type_annotation,
        )))
    }

    fn parse_tuple_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBracket);
        let start = self.current_token.start();
        self.advance()?;
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while self.current_token.tag() != TokenTag::RightBracket
                && self.current_token.tag() != TokenTag::Eof
            {
                let element = self.parse_tuple_element()?;
                self.scratch_a.append(element);
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            let end = self.current_token.end();
            if !self.expect(
                TokenTag::RightBracket,
                "Expected ']' to close a tuple type",
                Some("Each '[' in a tuple type must be matched by a ']'"),
            ) {
                return None;
            }
            let elements = self.add_extra_from_scratch_a(checkpoint);
            Some(TsType::TsTupleType(
                self.ast.ts_tuple_type(Span::new(start, end), elements),
            ))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_tuple_element(&mut self) -> Option<TsTupleElement> {
        if self.current_token.tag() == TokenTag::Spread {
            let start = self.current_token.start();
            self.advance()?;
            let inner = self.parse_tuple_element_body(true)?;
            let span = Span::new(start, inner.span(&self.ast).end);
            return Some(TsTupleElement::TsType(TsType::TsRestType(
                self.ast.ts_rest_type(span, inner),
            )));
        }
        self.parse_tuple_element_body(false)
    }

    fn parse_tuple_element_body(&mut self, rest: bool) -> Option<TsTupleElement> {
        // Consume the common head before choosing a label or a type reference.
        if self.current_token.tag() == TokenTag::Identifier
            && let Some((token, named)) = self.try_consume_tuple_identifier()
        {
            if named {
                let name = self.identifier_name(token);
                let label = self.ast.identifier_name(token.span(), name);
                return self.parse_named_tuple_member_with_label(label);
            }
            let type_annotation = self.parse_type_with(|p| {
                let result = p
                    .parse_entity_name_from_token(token)
                    .and_then(|name| p.parse_type_reference(token.start(), name))
                    .and_then(|primary| p.parse_type_from_primary(primary));
                result.or_else(|| {
                    p.report_expected(
                        p.current_token.span(),
                        "Expected a type",
                        ReportOptions::default(),
                    );
                    None
                })
            })?;
            return Some(TsTupleElement::TsType(
                self.tuple_element_type(type_annotation),
            ));
        }
        if self.is_named_tuple_element()? {
            // Rest labels are IdentifierNames; non-rest labels must start a type.
            return if rest {
                self.parse_named_tuple_member()
            } else {
                self.with_type_start_check(Self::parse_named_tuple_member)
            };
        }
        self.parse_tuple_element_type().map(TsTupleElement::TsType)
    }

    fn try_consume_tuple_identifier(&mut self) -> Option<(Token, bool)> {
        let token = self.current_token;
        let peek = self.begin_peek();
        peek.lexer.state.prev_token_end = token.end();
        let mut next = token;
        if !peek.lexer.try_next_token(&mut next) {
            // Let the ordinary tuple parser retain lexical-failure recovery.
            peek.lexer.next_token(&mut next).ok()?;
        }
        let named = if next.tag() == TokenTag::Question {
            let mut after_question = Peek::new(peek.lexer);
            after_question.next()?.tag() == TokenTag::Colon
        } else {
            next.tag() == TokenTag::Colon
        };
        peek.commit();
        self.current_token = next;
        Some((token, named))
    }

    fn parse_tuple_element_type(&mut self) -> Option<TsType> {
        let type_annotation = self.parse_type_with_start_check()?;
        Some(self.tuple_element_type(type_annotation))
    }

    fn tuple_element_type(&mut self, type_annotation: TsType) -> TsType {
        if let Some(nullable) = type_annotation.as_ts_js_doc_nullable_type(&self.ast)
            && nullable.postfix(&self.ast)
        {
            return TsType::TsOptionalType(self.ast.ts_optional_type(
                type_annotation.span(&self.ast),
                nullable.type_annotation(&self.ast),
            ));
        }
        type_annotation
    }

    fn is_named_tuple_element(&mut self) -> Option<bool> {
        if !self.current_token.tag().is_identifier_like() {
            return Some(false);
        }
        if !self.next_token_may_start_with(|byte| matches!(byte, b':' | b'?')) {
            return Some(false);
        }
        let first = self.peek_ahead()?;
        if first.tag() == TokenTag::Colon {
            return Some(true);
        }
        if first.tag() != TokenTag::Question {
            return Some(false);
        }
        let mut peek = self.begin_peek();
        let second = (|| {
            peek.next()?;
            peek.next()
        })();
        peek.end();
        Some(second?.tag() == TokenTag::Colon)
    }

    fn parse_named_tuple_member(&mut self) -> Option<TsTupleElement> {
        let label = self.parse_identifier_name()?;
        self.parse_named_tuple_member_with_label(label)
    }

    fn parse_named_tuple_member_with_label(
        &mut self,
        label: IdentifierName,
    ) -> Option<TsTupleElement> {
        let start = label.span(&self.ast).start;
        let optional = self.eat(TokenTag::Question)?;
        if !self.expect(
            TokenTag::Colon,
            "Expected ':' after named tuple element label",
            None,
        ) {
            return None;
        }
        // Keep a misplaced rest marker inside the named member so syntax
        // checking can report its position without losing the tuple.
        let element_type = if self.current_token.tag() == TokenTag::Spread {
            self.parse_misplaced_tuple_rest_type()?
        } else {
            self.parse_tuple_element_type()?
        };
        let span = Span::new(start, element_type.span(&self.ast).end);
        Some(TsTupleElement::TsNamedTupleMember(
            self.ast
                .ts_named_tuple_member(span, label, element_type, optional),
        ))
    }

    #[cold]
    #[inline(never)]
    fn parse_misplaced_tuple_rest_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Spread);
        let start = self.current_token.start();
        self.advance()?;
        let type_annotation = self.parse_type()?;
        let span = Span::new(start, type_annotation.span(&self.ast).end);
        let inner = TsTupleElement::TsType(type_annotation);
        Some(TsType::TsRestType(self.ast.ts_rest_type(span, inner)))
    }

    fn parse_type_operator(&mut self) -> Option<TsType> {
        let token = self.current_token;
        let operator = match token.tag() {
            TokenTag::Keyof => TsTypeOperatorKind::Keyof,
            TokenTag::Unique => TsTypeOperatorKind::Unique,
            TokenTag::Readonly => TsTypeOperatorKind::Readonly,
            _ => unreachable!(),
        };
        self.advance()?;
        let type_annotation = self.parse_postfix_type()?;
        let span = Span::new(token.start(), type_annotation.span(&self.ast).end);
        Some(TsType::TsTypeOperator(self.ast.ts_type_operator(
            span,
            operator,
            type_annotation,
        )))
    }

    fn parse_infer_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Infer);
        let start = self.current_token.start();
        self.advance()?;
        let name_start = self.current_token.start();
        let name = self.parse_type_binding_identifier()?;
        let constraint = self.parse_infer_constraint()?;
        let end = constraint.map_or(name.span(&self.ast).end, |ty| ty.span(&self.ast).end);
        let parameter = self.ast.ts_type_parameter(
            Span::new(name_start, end),
            name,
            constraint,
            None,
            false,
            false,
            false,
        );
        Some(TsType::TsInferType(
            self.ast.ts_infer_type(Span::new(start, end), parameter),
        ))
    }

    fn parse_infer_constraint(&mut self) -> Option<Option<TsType>> {
        if self.current_token.tag() != TokenTag::Extends {
            return Some(None);
        }

        let key = TsSpeculationKey::InferConstraint(self.speculation_key());
        let conditional_allowed = !self.ts_context.disallow_conditional_types();
        if conditional_allowed && self.ts_rejected_speculations.contains(&key) {
            return Some(None);
        }

        let checkpoint = self.checkpoint();
        self.advance()?;
        let constraint = match self.parse_type_no_conditional() {
            Some(constraint) => constraint,
            None => {
                self.rewind(checkpoint);
                return Some(None);
            }
        };
        if conditional_allowed && self.current_token.tag() == TokenTag::Question {
            self.rewind(checkpoint);
            self.reject_ts_speculation(key);
            return Some(None);
        }
        Some(Some(constraint))
    }

    fn is_start_of_function_or_constructor_type(&mut self) -> Option<bool> {
        let tag = self.current_token.tag();
        if tag == TokenTag::New {
            return Some(true);
        }
        if tag == TokenTag::Abstract {
            return Some(self.peek_ahead()?.tag() == TokenTag::New);
        }
        if is_angle_open(tag) {
            return Some(true);
        }
        if tag != TokenTag::LeftParen {
            return Some(false);
        }
        let mut peek = self.begin_peek();
        let first = peek.next()?;
        match first.tag() {
            TokenTag::RightParen => {
                let result = peek.next()?.tag() == TokenTag::Arrow;
                peek.end();
                return Some(result);
            }
            TokenTag::Spread => {
                peek.end();
                return Some(true);
            }
            TokenTag::LeftBrace | TokenTag::LeftBracket => {
                peek.end();
                return self.is_function_type_after_pattern();
            }
            _ => {}
        }
        if !first.tag().is_identifier_candidate() && first.tag() != TokenTag::This {
            peek.end();
            return Some(false);
        }
        let second = peek.next()?;
        let result = match second.tag() {
            TokenTag::Colon | TokenTag::Question | TokenTag::Comma | TokenTag::Assign => true,
            TokenTag::RightParen => peek.next()?.tag() == TokenTag::Arrow,
            _ => false,
        };
        peek.end();
        Some(result)
    }

    fn is_function_type_after_pattern(&mut self) -> Option<bool> {
        let checkpoint = self.checkpoint();
        let result = (|| {
            self.advance()?;
            if self
                .with_type_binding_context(Self::parse_binding_pattern)
                .is_none()
                || !self.speculation_is_clean(&checkpoint)
            {
                return Some(false);
            }
            Some(match self.current_token.tag() {
                TokenTag::Colon | TokenTag::Comma | TokenTag::Question | TokenTag::Assign => true,
                TokenTag::RightParen => {
                    self.advance()?;
                    self.current_token.tag() == TokenTag::Arrow
                }
                _ => false,
            })
        })();
        self.rewind(checkpoint);
        result
    }

    fn parse_function_or_constructor_type(&mut self) -> Option<TsType> {
        let start = self.current_token.start();
        let is_abstract = self.eat(TokenTag::Abstract)?;
        let is_constructor = self.eat(TokenTag::New)?;
        let type_parameters = if is_angle_open(self.current_token.tag()) {
            Some(self.parse_type_parameters(TypeParameterContext::Signature, AngleClose::Peel)?)
        } else {
            None
        };
        let (this_param, params) =
            self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        if is_constructor && this_param.is_some() {
            self.report(
                params.span(&self.ast),
                "A 'this' parameter is not allowed here",
                ReportOptions::default(),
            );
            return None;
        }
        if self.current_token.tag() != TokenTag::Arrow {
            self.report_expected(
                self.current_token.span(),
                "Expected '=>' in function type",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "A function type is written '(params) => ReturnType'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let arrow_start = self.current_token.start();
        self.advance()?;
        let return_type_inner = self.parse_type_or_type_predicate()?;
        let end = return_type_inner.span(&self.ast).end;
        let return_type = self
            .ast
            .ts_type_annotation(Span::new(arrow_start, end), return_type_inner);
        Some(if is_constructor {
            TsType::TsConstructorType(self.ast.ts_constructor_type(
                Span::new(start, end),
                type_parameters,
                params,
                return_type,
                is_abstract,
            ))
        } else {
            TsType::TsFunctionType(self.ast.ts_function_type(
                Span::new(start, end),
                type_parameters,
                this_param,
                params,
                return_type,
            ))
        })
    }

    fn parse_type_query(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Typeof);
        let start = self.current_token.start();
        self.advance()?;
        let expr_name = if self.current_token.tag() == TokenTag::Import {
            let import_type = self.parse_import_type()?;
            let NodeKindData::TsImportType(import_type) = self.ast.kind_data(import_type.node_id())
            else {
                unreachable!()
            };
            TsTypeQueryExprName::TsImportType(import_type)
        } else if self.current_token.tag() == TokenTag::This {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            if self.current_token.tag() == TokenTag::Dot {
                let left = self
                    .ast
                    .ts_qualified_name_left_this_expression(token.span());
                TsTypeQueryExprName::TsTypeName(self.extend_qualified_name(left, token.start())?)
            } else {
                self.ast
                    .ts_type_query_expr_name_this_expression(token.span())
            }
        } else {
            TsTypeQueryExprName::TsTypeName(self.parse_entity_name()?)
        };
        let type_arguments = if self.current_token.tag() == TokenTag::LessThan
            && !self.current_token.has_line_terminator_before()
            && !expr_name.is_ts_import_type(&self.ast)
        {
            Some(self.parse_type_arguments()?)
        } else {
            None
        };
        let end = type_arguments.map_or(expr_name.span(&self.ast).end, |args| {
            args.span(&self.ast).end
        });
        Some(TsType::TsTypeQuery(self.ast.ts_type_query(
            Span::new(start, end),
            expr_name,
            type_arguments,
        )))
    }

    fn parse_import_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Import);
        let start = self.current_token.start();
        self.advance()?;
        if !self.expect(
            TokenTag::LeftParen,
            "Expected '(' after 'import' in an import type",
            Some("An import type is written 'import(\"module\").Foo'"),
        ) {
            return None;
        }
        let source = self.parse_string_literal()?;
        let options = self.parse_import_type_options()?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' to close an import type",
            Some("An import type's argument list must end with ')'"),
        ) {
            return None;
        }
        let qualifier = if self.eat(TokenTag::Dot)? {
            Some(self.parse_import_type_qualifier()?)
        } else {
            None
        };
        let type_arguments = if is_angle_open(self.current_token.tag()) {
            Some(self.parse_type_arguments()?)
        } else {
            None
        };
        let end = type_arguments.map_or_else(
            || {
                qualifier.map_or(self.lexer.state.prev_token_end, |name| {
                    name.span(&self.ast).end
                })
            },
            |args| args.span(&self.ast).end,
        );
        Some(TsType::TsImportType(self.ast.ts_import_type(
            Span::new(start, end),
            source,
            options,
            qualifier,
            type_arguments,
        )))
    }

    fn parse_import_type_options(&mut self) -> Option<Option<Expr>> {
        if !self.eat(TokenTag::Comma)? {
            return Some(None);
        }
        if self.current_token.tag() == TokenTag::RightParen {
            return Some(None);
        }
        let options = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        self.eat(TokenTag::Comma)?;
        Some(Some(options))
    }

    fn parse_import_type_qualifier(&mut self) -> Option<TsImportTypeQualifier> {
        if !self.current_token.tag().is_identifier_like() {
            self.report_expected(
                self.current_token.span(),
                "Expected an identifier after '.'",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "A qualified name must end with an identifier",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let identifier = self.parse_identifier_name()?;
        if self.current_token.tag() != TokenTag::Dot {
            return Some(TsImportTypeQualifier::IdentifierName(identifier));
        }
        let left = TsQualifiedNameLeft::IdentifierName(identifier);
        let qualified = self.extend_qualified_name(left, identifier.span(&self.ast).start)?;
        let qualified = qualified
            .as_ts_qualified_name(&self.ast)
            .expect("qualified-name parsing produces a qualified name");
        Some(TsImportTypeQualifier::TsQualifiedName(qualified))
    }

    fn extend_qualified_name(
        &mut self,
        mut left: TsQualifiedNameLeft,
        start: u32,
    ) -> Option<TsTypeName> {
        let mut result = None;
        while self.current_token.tag() == TokenTag::Dot {
            self.advance()?;
            let token = self.current_token;
            if !token.tag().is_identifier_like() {
                self.report_expected(
                    self.current_token.span(),
                    "Expected an identifier after '.'",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "A qualified name must end with an identifier",
                        )),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
            let name = self.identifier_name(token);
            let right = self.ast.identifier_name(token.span(), name);
            self.advance_without_escape_check()?;
            let qualified = self
                .ast
                .ts_qualified_name(Span::new(start, token.end()), left, right);
            result = Some(qualified);
            left = TsQualifiedNameLeft::TsQualifiedName(qualified);
        }
        Some(TsTypeName::TsQualifiedName(
            result.expect("qualified-name parsing starts on '.'"),
        ))
    }

    fn parse_jsdoc_nullable_or_unknown_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Question);
        let token = self.current_token;
        self.advance()?;
        if !can_parse_type_for_recovery(self.current_token.tag()) {
            return Some(TsType::TsJsDocUnknownType(
                self.ast.ts_js_doc_unknown_type(token.span()),
            ));
        }
        let type_annotation = self.parse_type()?;
        let span = Span::new(token.start(), type_annotation.span(&self.ast).end);
        Some(self.jsdoc_node_data(type_annotation, JsDocKind::Nullable, false, span))
    }

    fn parse_jsdoc_non_nullable_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LogicalNot);
        let start = self.current_token.start();
        self.advance()?;
        let type_annotation = self.parse_primary_type()?;
        let span = Span::new(start, type_annotation.span(&self.ast).end);
        Some(self.jsdoc_node_data(type_annotation, JsDocKind::NonNullable, false, span))
    }

    fn jsdoc_node_data(
        &mut self,
        inner: TsType,
        kind: JsDocKind,
        postfix: bool,
        span: Span,
    ) -> TsType {
        match kind {
            JsDocKind::Nullable => {
                TsType::TsJsDocNullableType(self.ast.ts_js_doc_nullable_type(span, inner, postfix))
            }
            JsDocKind::NonNullable => TsType::TsJsDocNonNullableType(
                self.ast.ts_js_doc_non_nullable_type(span, inner, postfix),
            ),
        }
    }

    fn is_qualified_type_continuation(&mut self) -> Option<bool> {
        if !self.next_token_may_start_with(|byte| byte == b'.') {
            return Some(false);
        }
        let next = self.peek_ahead()?;
        Some(next.tag() == TokenTag::Dot && !next.has_line_terminator_before())
    }

    fn parse_literal_type(&mut self) -> Option<TsType> {
        let start = self.current_token.start();
        let literal = match self.current_token.tag() {
            TokenTag::True | TokenTag::False => {
                let token = self.current_token;
                self.advance()?;
                TsLiteral::BooleanLiteral(
                    self.ast
                        .boolean_literal(token.span(), token.tag() == TokenTag::True),
                )
            }
            TokenTag::StringLiteral => TsLiteral::StringLiteral(self.parse_string_literal()?),
            TokenTag::NoSubstitutionTemplate => {
                TsLiteral::TemplateLiteral(self.parse_no_substitution_template(false)?)
            }
            TokenTag::Minus => self.parse_signed_numeric_literal_type()?,
            _ => {
                let expression = self.parse_numeric_literal()?;
                match self.ast.expr_data(expression) {
                    ExprData::NumericLiteral(literal) => TsLiteral::NumericLiteral(literal),
                    ExprData::BigIntLiteral(literal) => TsLiteral::BigIntLiteral(literal),
                    _ => unreachable!(),
                }
            }
        };
        Some(TsType::TsLiteralType(self.ast.ts_literal_type(
            Span::new(start, literal.span(&self.ast).end),
            literal,
        )))
    }

    fn parse_signed_numeric_literal_type(&mut self) -> Option<TsLiteral> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Minus);
        let sign_token = self.current_token;
        let next = self.peek_ahead()?;
        if !next.tag().is_numeric_literal() {
            return None;
        }
        self.advance()?;
        let expression = self.parse_numeric_literal()?;
        Some(TsLiteral::UnaryExpression(self.ast.unary_expression(
            Span::new(sign_token.start(), expression.span(&self.ast).end),
            expression,
            UnaryOperator::Negate,
        )))
    }

    fn parse_template_literal_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::TemplateHead);
        let start = self.current_token.start();
        let head = self.current_token;
        let quasis_checkpoint = self.scratch_a.begin();
        let types_checkpoint = self.scratch_b.begin();
        let result = (|| {
            let head = self.add_template_element(head, false, false);
            self.scratch_a.append(head);
            self.advance()?;
            let end = loop {
                let ty = self.parse_type()?;
                self.scratch_b.append(ty);
                if self.current_token.tag() != TokenTag::RightBrace {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected '}' to close template type interpolation",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Template type interpolations must be closed with '}'",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
                let right_brace = self.current_token;
                let continuation = match self
                    .lexer
                    .re_scan_template_continuation(right_brace.start())
                {
                    Ok(token) => token,
                    Err(error) => {
                        self.report(
                            right_brace.span(),
                            lexical_error_message(error),
                            ReportOptions {
                                help: Some(Cow::Borrowed(lexical_error_help(error))),
                                ..ReportOptions::default()
                            },
                        );
                        return None;
                    }
                };
                let tail = continuation.tag() == TokenTag::TemplateTail;
                let element = self.add_template_element(continuation, tail, false);
                self.scratch_a.append(element);
                let end = continuation.end();
                self.advance_with_rescanned_token(continuation)?;
                if tail {
                    break end;
                }
            };
            let quasis = self.add_extra_from_scratch_a(quasis_checkpoint);
            let types = self.add_extra_from_scratch_b(types_checkpoint);
            Some(TsType::TsTemplateLiteralType(
                self.ast
                    .ts_template_literal_type(Span::new(start, end), quasis, types),
            ))
        })();
        self.scratch_a.reset(quasis_checkpoint);
        self.scratch_b.reset(types_checkpoint);
        result
    }

    fn is_start_of_mapped_type(&mut self) -> bool {
        let mut token = self.current_token;
        if !matches!(
            token.tag(),
            TokenTag::LeftBracket | TokenTag::Readonly | TokenTag::Plus | TokenTag::Minus
        ) {
            return false;
        }
        let mut peek = self.begin_peek();
        if matches!(token.tag(), TokenTag::Plus | TokenTag::Minus) {
            let Some(next) = peek.next() else {
                return false;
            };
            token = next;
            if token.tag() != TokenTag::Readonly {
                return false;
            }
        }
        if token.tag() == TokenTag::Readonly {
            let Some(next) = peek.next() else {
                return false;
            };
            token = next;
        }
        if token.tag() != TokenTag::LeftBracket {
            return false;
        }
        let Some(name) = peek.next() else {
            return false;
        };
        if !name.tag().is_identifier_like() {
            return false;
        }
        let Some(operator) = peek.next() else {
            return false;
        };
        operator.tag() == TokenTag::In
    }

    fn parse_mapped_type(&mut self, start: u32) -> Option<TsType> {
        let readonly = self.parse_mapped_modifier(TokenTag::Readonly)?;
        if !self.expect(
            TokenTag::LeftBracket,
            "Expected '[' in a mapped type",
            Some("A mapped type is written '{ [K in T]: V }'"),
        ) {
            return None;
        }
        let key = self.parse_type_binding_identifier()?;
        if !self.expect(
            TokenTag::In,
            "Expected 'in' after the mapped type parameter name",
            Some("A mapped type iterates its key with '[K in ConstraintType]'"),
        ) {
            return None;
        }
        let constraint = self.parse_type()?;
        let name_type = if self.eat(TokenTag::As)? {
            Some(self.parse_type()?)
        } else {
            None
        };
        if !self.expect(
            TokenTag::RightBracket,
            "Expected ']' to close a mapped type parameter",
            Some("A mapped type parameter must be closed with ']'"),
        ) {
            return None;
        }
        let optional = self.parse_mapped_modifier(TokenTag::Question)?;
        let type_annotation = if self.eat(TokenTag::Colon)? {
            Some(self.parse_type()?)
        } else {
            None
        };
        if matches!(
            self.current_token.tag(),
            TokenTag::Semicolon | TokenTag::Comma
        ) {
            self.advance()?;
        }
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close a mapped type",
            Some("Each '{' in a mapped type must be matched by a '}'"),
        ) {
            return None;
        }
        Some(TsType::TsMappedType(self.ast.ts_mapped_type(
            Span::new(start, end),
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
        )))
    }

    fn parse_mapped_modifier(&mut self, terminator: TokenTag) -> Option<TsMappedTypeModifier> {
        if self.current_token.tag() == terminator {
            self.advance()?;
            return Some(TsMappedTypeModifier::True);
        }
        let modifier = match self.current_token.tag() {
            TokenTag::Plus => TsMappedTypeModifier::Plus,
            TokenTag::Minus => TsMappedTypeModifier::Minus,
            _ => return Some(TsMappedTypeModifier::None),
        };
        self.advance()?;
        let message = if terminator == TokenTag::Readonly {
            "Expected 'readonly' after '+' or '-' in a mapped type"
        } else {
            "Expected '?' after '+' or '-' in a mapped type"
        };
        if !self.expect(
            terminator,
            message,
            Some("Mapped type '+' and '-' modifiers can only decorate 'readonly' or '?'"),
        ) {
            return None;
        }
        Some(modifier)
    }

    fn parse_type_literal(&mut self, start: u32) -> Option<TsType> {
        let (members, end) = self.parse_object_type_members(TypeParameterContext::Signature)?;
        Some(TsType::TsTypeLiteral(
            self.ast.ts_type_literal(Span::new(start, end), members),
        ))
    }

    /// Parses a type body whose opening brace has already been consumed.
    fn parse_object_type_members(
        &mut self,
        context: TypeParameterContext,
    ) -> Option<(SubRange<TsTypeElement>, u32)> {
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while self.current_token.tag() != TokenTag::RightBrace
                && self.current_token.tag() != TokenTag::Eof
            {
                let member = self.parse_type_member(context)?;
                self.scratch_a.append(member);
                if !self.consume_type_member_separator(member)? {
                    return None;
                }
            }
            let end = self.current_token.end();
            if !self.expect(
                TokenTag::RightBrace,
                "Expected '}' to close an object-type body",
                Some("Each '{' in a type must be matched by a '}'"),
            ) {
                return None;
            }
            Some((self.add_extra_from_scratch_a(checkpoint), end))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn consume_type_member_separator(&mut self, member: TsTypeElement) -> Option<bool> {
        match self.current_token.tag() {
            TokenTag::Semicolon | TokenTag::Comma => {
                let end = self.current_token.end();
                self.extend_span_to(member, end);
                self.advance()?;
            }
            TokenTag::RightBrace => {}
            _ if self.current_token.has_line_terminator_before() => {}
            _ => {
                self.report_expected(
                    self.current_token.span(),
                    "Expected ';', ',', or newline between type members",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Separate type members with ';', ',', or a newline, or close the block with '}'",
                        )),
                        ..ReportOptions::default()
                    },
                );
                return Some(false);
            }
        }
        Some(true)
    }

    fn extend_span_to<N: GetSpan + SetSpan>(&mut self, node: N, end: u32) {
        let span = node.span(&self.ast);
        if end > span.end {
            node.set_span(&mut self.ast, Span::new(span.start, end));
        }
    }

    fn parse_type_member(&mut self, context: TypeParameterContext) -> Option<TsTypeElement> {
        let tag = self.current_token.tag();
        if matches!(tag, TokenTag::LeftParen | TokenTag::LessThan) {
            return self.parse_call_or_construct_signature(context, false);
        }
        if tag == TokenTag::New
            && matches!(
                self.peek_ahead()?.tag(),
                TokenTag::LeftParen | TokenTag::LessThan
            )
        {
            return self.parse_call_or_construct_signature(context, true);
        }

        let mut readonly = false;
        let start = self.current_token.start();
        if tag == TokenTag::Readonly {
            let next = self.peek_ahead()?;
            if !next.has_line_terminator_before() && can_follow_readonly_modifier(next.tag()) {
                readonly = true;
                self.advance()?;
            }
        }
        if self.current_token.tag() == TokenTag::LeftBracket && self.is_index_signature_start()? {
            return self.parse_index_signature(start, readonly, false);
        }
        self.parse_property_or_method_signature(context, start, readonly)
    }

    fn parse_call_or_construct_signature(
        &mut self,
        context: TypeParameterContext,
        is_construct: bool,
    ) -> Option<TsTypeElement> {
        let start = self.current_token.start();
        if is_construct {
            self.advance()?;
        }
        let type_parameters = if is_angle_open(self.current_token.tag()) {
            Some(self.parse_type_parameters(context, AngleClose::Peel)?)
        } else {
            None
        };
        let (this_param, params) =
            self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        if is_construct && this_param.is_some() {
            self.report(
                params.span(&self.ast),
                "A 'this' parameter is not allowed here",
                ReportOptions::default(),
            );
            return None;
        }
        let return_type = if self.current_token.tag() == TokenTag::Colon {
            Some(self.parse_return_type_annotation()?)
        } else {
            None
        };
        let end = return_type.map_or(self.lexer.state.prev_token_end, |ty| ty.span(&self.ast).end);
        Some(if is_construct {
            self.ast.ts_type_element_ts_construct_signature_declaration(
                Span::new(start, end),
                type_parameters,
                params,
                return_type,
            )
        } else {
            self.ast.ts_type_element_ts_call_signature_declaration(
                Span::new(start, end),
                type_parameters,
                this_param,
                params,
                return_type,
            )
        })
    }

    fn is_index_signature_start(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBracket);
        let mut peek = self.begin_peek();
        let Some(name) = peek.next() else {
            return Some(false);
        };
        if name.tag() == TokenTag::RightBracket {
            return Some(
                peek.next()
                    .is_some_and(|token| token.tag() == TokenTag::Colon),
            );
        }
        if !name.tag().is_identifier_like() {
            return Some(false);
        }
        let Some(second) = peek.next() else {
            return Some(false);
        };
        Some(matches!(second.tag(), TokenTag::Colon | TokenTag::Comma))
    }

    fn parse_index_signature(
        &mut self,
        start: u32,
        readonly: bool,
        r#static: bool,
    ) -> Option<TsTypeElement> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBracket);
        self.advance()?;
        let parameters = self.parse_index_signature_parameters()?;
        if !self.expect(
            TokenTag::RightBracket,
            "Expected ']' to close an index signature parameter list",
            Some("An index signature is written '[name: KeyType]: ValueType'"),
        ) {
            return None;
        }
        if self.current_token.tag() != TokenTag::Colon {
            self.report_expected(
                self.current_token.span(),
                "Expected ':' after index signature parameters",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "An index signature requires a value type: '[k: string]: number'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let type_annotation = self.parse_type_annotation()?;
        let end = type_annotation.span(&self.ast).end;
        Some(self.ast.ts_type_element_ts_index_signature(
            Span::new(start, end),
            parameters,
            type_annotation,
            readonly,
            r#static,
        ))
    }

    fn parse_index_signature_parameters(&mut self) -> Option<SubRange<TsIndexSignatureParameter>> {
        if self.current_token.tag() == TokenTag::RightBracket {
            self.report(
                self.current_token.span(),
                "An index signature must have exactly one parameter",
                ReportOptions::default(),
            );
            return Some(SubRange::empty());
        }
        let first = self.parse_index_signature_parameter()?;
        if self.current_token.tag() != TokenTag::Comma {
            return Some(self.ast.add_sub_range([first]));
        }
        self.report(
            self.current_token.span(),
            "An index signature must have exactly one parameter",
            ReportOptions::default(),
        );
        let checkpoint = self.scratch_a.begin();
        self.scratch_a.append(first);
        let result = (|| {
            while self.eat(TokenTag::Comma)? {
                if self.current_token.tag() == TokenTag::RightBracket {
                    break;
                }
                let parameter = self.parse_index_signature_parameter()?;
                self.scratch_a.append(parameter);
            }
            Some(self.add_extra_from_scratch_a(checkpoint))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_index_signature_parameter(&mut self) -> Option<TsIndexSignatureParameter> {
        let name = self.parse_type_binding_identifier()?;
        if self.current_token.tag() != TokenTag::Colon {
            self.report_expected(
                self.current_token.span(),
                "Expected ':' after index signature parameter name",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Each index signature parameter requires a type: '[k: string]'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let annotation = self.parse_type_annotation()?;
        let span = Span::new(name.span(&self.ast).start, annotation.span(&self.ast).end);
        Some(
            self.ast
                .ts_index_signature_parameter(span, name, annotation),
        )
    }

    fn parse_property_or_method_signature(
        &mut self,
        context: TypeParameterContext,
        start: u32,
        readonly: bool,
    ) -> Option<TsTypeElement> {
        let mut kind = TsMethodSignatureKind::Method;
        let head_tag = self.current_token.tag();
        if matches!(head_tag, TokenTag::Get | TokenTag::Set)
            && can_follow_accessor_keyword(self.peek_ahead()?.tag())
        {
            kind = if head_tag == TokenTag::Get {
                TsMethodSignatureKind::Get
            } else {
                TsMethodSignatureKind::Set
            };
            self.advance()?;
        }
        let computed = self.current_token.tag() == TokenTag::LeftBracket;
        let key = self.parse_property_key(false)?;
        let mut end = self.lexer.state.prev_token_end;
        let optional = if self.current_token.tag() == TokenTag::Question {
            end = self.current_token.end();
            self.advance()?;
            true
        } else {
            false
        };
        if kind != TsMethodSignatureKind::Method
            || matches!(
                self.current_token.tag(),
                TokenTag::LeftParen | TokenTag::LessThan
            )
        {
            return self.parse_method_signature_body(context, start, key, kind, computed, optional);
        }
        let type_annotation = if self.current_token.tag() == TokenTag::Colon {
            let annotation = self.parse_type_annotation()?;
            end = annotation.span(&self.ast).end;
            Some(annotation)
        } else {
            None
        };
        Some(self.ast.ts_type_element_ts_property_signature(
            Span::new(start, end),
            key,
            type_annotation,
            computed,
            optional,
            readonly,
        ))
    }

    fn parse_method_signature_body(
        &mut self,
        context: TypeParameterContext,
        start: u32,
        key: PropertyKey,
        kind: TsMethodSignatureKind,
        computed: bool,
        optional: bool,
    ) -> Option<TsTypeElement> {
        let type_parameters = if is_angle_open(self.current_token.tag()) {
            Some(self.parse_type_parameters(context, AngleClose::Peel)?)
        } else {
            None
        };
        let (this_param, params) =
            self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        let return_type = if self.current_token.tag() == TokenTag::Colon {
            let annotation = self.parse_return_type_annotation()?;
            if kind == TsMethodSignatureKind::Set {
                self.report(
                    annotation.span(&self.ast),
                    "A 'set' accessor cannot have a return type annotation",
                    ReportOptions::default(),
                );
            }
            Some(annotation)
        } else {
            None
        };
        let end = return_type.map_or(self.lexer.state.prev_token_end, |ty| ty.span(&self.ast).end);
        Some(self.ast.ts_type_element_ts_method_signature(
            Span::new(start, end),
            key,
            type_parameters,
            this_param,
            params,
            return_type,
            kind,
            computed,
            optional,
        ))
    }

    fn parse_type_reference(&mut self, start: u32, type_name: TsTypeName) -> Option<TsType> {
        let type_arguments = if is_angle_open(self.current_token.tag())
            && !self.current_token.has_line_terminator_before()
        {
            if self.type_argument_speculation_depth != 0 {
                return self.parse_type_reference_chain(start, type_name);
            }
            Some(self.parse_type_argument_list(AngleClose::Peel)?)
        } else {
            None
        };
        let end = self.lexer.state.prev_token_end;
        Some(
            self.ast
                .ts_type_ts_type_reference(Span::new(start, end), type_name, type_arguments),
        )
    }

    /// A flat comparison list can look like an arbitrarily deep chain of type
    /// references until the last argument fails. Keep that speculative chain
    /// off the native stack, carrying its failure keys through the same frames.
    /// Keep its work-stack locals out of the ordinary primary-type parser's frame.
    #[inline(never)]
    fn parse_type_reference_chain(&mut self, start: u32, name: TsTypeName) -> Option<TsType> {
        let mut references = Vec::new();
        let scratch_start = self.scratch_a.begin();
        let disallow_conditional_types = self.ts_context.disallow_conditional_types();
        let mut frame = self.begin_type_reference_arguments(start, name)?;
        let result = (|| {
            self.ts_context
                .remove(TsContext::DISALLOW_CONDITIONAL_TYPES);
            let mut completed = None;
            loop {
                let argument = if let Some(ty) = completed.take() {
                    Some(ty)
                } else if is_angle_close(self.current_token.tag())
                    || self.current_token.tag() == TokenTag::Eof
                {
                    None
                } else {
                    match self.with_type_start_check(Self::parse_type_argument_head)? {
                        PrimaryTypeHead::Parsed(ty) => Some(ty),
                        PrimaryTypeHead::Reference { start, name } => {
                            if is_angle_open(self.current_token.tag())
                                && !self.current_token.has_line_terminator_before()
                            {
                                let nested = self.begin_type_reference_arguments(start, name)?;
                                references.push(mem::replace(&mut frame, nested));
                                continue;
                            }
                            let primary = self.ast.ts_type_ts_type_reference(
                                Span::new(start, self.lexer.state.prev_token_end),
                                name,
                                None,
                            );
                            Some(self.parse_type_from_primary(primary)?)
                        }
                    }
                };
                if let Some(argument) = argument {
                    self.scratch_a.append(argument);
                    if self.eat(TokenTag::Comma)? {
                        continue;
                    }
                }
                if self.scratch_a.begin() == frame.scratch_start {
                    self.report(
                        self.current_token.span(),
                        "A type argument list cannot be empty",
                        ReportOptions::default(),
                    );
                }
                let end = self
                    .consume_angle_close(AngleListKind::Arguments, AngleClose::SpeculativePeel)?;
                let parameters = self.add_extra_from_scratch_a(frame.scratch_start);
                self.scratch_a.reset(frame.scratch_start);
                let arguments = self.ast.ts_type_parameter_instantiation(
                    Span::new(frame.argument_start, end),
                    parameters,
                );
                let ty = self.ast.ts_type_ts_type_reference(
                    Span::new(frame.start, end),
                    frame.name,
                    Some(arguments),
                );
                let Some(parent) = references.pop() else {
                    return Some(ty);
                };
                frame = parent;
                completed = Some(self.parse_type_from_primary(ty)?);
            }
        })();
        if result.is_none() {
            for parent in &references {
                self.cache_type_argument_failure(parent.key, TypeArgumentFailures::PEEL);
            }
            self.cache_type_argument_failure(frame.key, TypeArgumentFailures::PEEL);
        }
        self.scratch_a.reset(scratch_start);
        self.ts_context.set(
            TsContext::DISALLOW_CONDITIONAL_TYPES,
            disallow_conditional_types,
        );
        result
    }

    /// Parse a complete argument, deferring only reference heads whose nested
    /// arguments need the iterative work stack.
    fn parse_type_argument_head(&mut self) -> Option<PrimaryTypeHead> {
        if !self.current_token.tag().is_identifier_like()
            || matches!(self.current_token.tag(), TokenTag::New | TokenTag::Abstract)
                && self.is_start_of_function_or_constructor_type()?
        {
            return self.parse_type().map(PrimaryTypeHead::Parsed);
        }
        match self.parse_primary_type_with(PrimaryTypeHead::Parsed, |_, start, name| {
            Some(PrimaryTypeHead::Reference { start, name })
        })? {
            PrimaryTypeHead::Parsed(ty) => self
                .parse_type_from_primary(ty)
                .map(PrimaryTypeHead::Parsed),
            reference => Some(reference),
        }
    }

    fn begin_type_reference_arguments(
        &mut self,
        start: u32,
        name: TsTypeName,
    ) -> Option<TypeReferenceFrame> {
        let key = self.speculation_key();
        if self.has_failed_type_arguments(&key, TypeArgumentFailures::PEEL) {
            return None;
        }
        let argument_start = self.consume_angle_open()?;
        Some(TypeReferenceFrame {
            start,
            name,
            argument_start,
            scratch_start: self.scratch_a.begin(),
            key,
        })
    }

    fn parse_type_from_primary(&mut self, primary: TsType) -> Option<TsType> {
        let start = primary.span(&self.ast).start;
        let mut ty = self.parse_postfix_type_with(|_| Some(primary))?;
        if self.current_token.tag() == TokenTag::BitwiseAnd {
            ty = self.parse_binary_type_chain_from(
                ty,
                start,
                TokenTag::BitwiseAnd,
                BinaryTypeKind::Intersection,
                Self::parse_postfix_type,
            )?;
        }
        if self.current_token.tag() == TokenTag::BitwiseOr {
            ty = self.parse_binary_type_chain_from(
                ty,
                start,
                TokenTag::BitwiseOr,
                BinaryTypeKind::Union,
                Self::parse_intersection_type,
            )?;
        }
        self.parse_conditional_type_with(|_| Some(ty))
    }

    /// Continue a type after consuming its leading identifier.
    fn parse_type_from_identifier(&mut self, token: Token) -> Option<TsType> {
        self.parse_type_with(|parser| {
            let primary = parser
                .parse_entity_name_from_token(token)
                .and_then(|name| parser.parse_type_reference(token.start(), name));
            let result = primary.and_then(|primary| parser.parse_type_from_primary(primary));
            result.or_else(|| {
                parser.report_expected(
                    parser.current_token.span(),
                    "Expected a type",
                    ReportOptions::default(),
                );
                None
            })
        })
    }

    fn parse_entity_name(&mut self) -> Option<TsTypeName> {
        let token = self.consume_identifier(IdentifierRole::Name)?;
        self.parse_entity_name_from_token(token)
    }

    fn parse_entity_name_from_token(&mut self, token: Token) -> Option<TsTypeName> {
        let name = self.identifier_name(token);
        let identifier = self.ast.identifier_reference(token.span(), name);
        if self.current_token.tag() == TokenTag::Dot {
            self.extend_qualified_name(
                TsQualifiedNameLeft::IdentifierReference(identifier),
                token.start(),
            )
        } else {
            Some(TsTypeName::IdentifierReference(identifier))
        }
    }

    fn parse_type_parameters(
        &mut self,
        context: TypeParameterContext,
        close: AngleClose,
    ) -> Option<TsTypeParameterDeclaration> {
        match self.parse_angle_list(AngleListKind::Parameters(context), close)? {
            AngleList::Parameters(parameters) => Some(parameters),
            AngleList::Arguments(_) => unreachable!(),
        }
    }

    fn parse_type_arguments(&mut self) -> Option<TsTypeParameterInstantiation> {
        if self.type_argument_speculation_depth == 0 {
            self.parse_type_argument_list(AngleClose::Peel)
        } else {
            self.parse_speculative_type_arguments()
        }
    }

    fn parse_type_arguments_in_expression(&mut self) -> Option<TsTypeParameterInstantiation> {
        self.parse_type_argument_list(AngleClose::Exact)
    }

    fn parse_type_argument_list(
        &mut self,
        close: AngleClose,
    ) -> Option<TsTypeParameterInstantiation> {
        match self.parse_angle_list(AngleListKind::Arguments, close)? {
            AngleList::Arguments(arguments) => Some(arguments),
            AngleList::Parameters(_) => unreachable!(),
        }
    }

    fn parse_speculative_type_arguments(&mut self) -> Option<TsTypeParameterInstantiation> {
        debug_assert_ne!(self.type_argument_speculation_depth, 0);
        let key = self.speculation_key();
        if self.has_failed_type_arguments(&key, TypeArgumentFailures::PEEL) {
            return None;
        }
        // Reference chains use an explicit stack; recursive grammar entries
        // still need delimiter proofs before descending into another list.
        if !self.type_arguments_can_close() {
            self.cache_type_argument_failure(key, TypeArgumentFailures::PEEL);
            return None;
        }
        let result = self.parse_type_argument_list(AngleClose::SpeculativePeel);
        if result.is_none() {
            self.cache_type_argument_failure(key, TypeArgumentFailures::PEEL);
        }
        result
    }

    fn parse_angle_list(&mut self, kind: AngleListKind, close: AngleClose) -> Option<AngleList> {
        match kind {
            AngleListKind::Arguments => self.parse_angle_list_with(kind, close, |parser| {
                Some(parser.parse_type_with_start_check()?.node_id())
            }),
            AngleListKind::Parameters(context) => {
                self.parse_angle_list_with(kind, close, move |parser| {
                    Some(parser.parse_type_parameter(context)?.node_id())
                })
            }
        }
    }

    fn parse_angle_list_with(
        &mut self,
        kind: AngleListKind,
        close: AngleClose,
        mut parse: impl FnMut(&mut Self) -> Option<NodeId>,
    ) -> Option<AngleList> {
        let start = self.consume_angle_open()?;
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            let mut parsed_any = false;
            while !is_angle_close(self.current_token.tag())
                && self.current_token.tag() != TokenTag::Eof
            {
                let node = parse(self)?;
                self.scratch_a.items.push(node);
                parsed_any = true;
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            if !parsed_any {
                self.report(
                    self.current_token.span(),
                    match kind {
                        AngleListKind::Arguments => "A type argument list cannot be empty",
                        AngleListKind::Parameters(_) => "A type parameter list cannot be empty",
                    },
                    ReportOptions::default(),
                );
            }
            let end = self.consume_angle_close(kind, close)?;
            Some(match kind {
                AngleListKind::Arguments => {
                    let parameters = self.add_extra_from_scratch_a(checkpoint);
                    AngleList::Arguments(
                        self.ast
                            .ts_type_parameter_instantiation(Span::new(start, end), parameters),
                    )
                }
                AngleListKind::Parameters(_) => {
                    let parameters = self.add_extra_from_scratch_a(checkpoint);
                    AngleList::Parameters(
                        self.ast
                            .ts_type_parameter_declaration(Span::new(start, end), parameters),
                    )
                }
            })
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_type_parameter(&mut self, context: TypeParameterContext) -> Option<TsTypeParameter> {
        let mut is_const = false;
        let mut is_in = false;
        let mut is_out = false;
        let mut start = self.current_token.start();
        let mut start_set = false;
        loop {
            let token = self.current_token;
            let in_after_out = token.tag() == TokenTag::In && is_out;
            let flag = match token.tag() {
                TokenTag::Const => &mut is_const,
                TokenTag::In => &mut is_in,
                TokenTag::Out => &mut is_out,
                _ => break,
            };
            let Some(next) = self.peek_ahead() else {
                break;
            };
            if !next.tag().is_identifier_like() || next.has_line_terminator_before() {
                break;
            }
            if *flag {
                let text = token.tag().to_str().unwrap_or("modifier");
                self.report(
                    token.span(),
                    Cow::Owned(format!("Duplicate '{text}' modifier on type parameter")),
                    ReportOptions::default(),
                );
            } else if in_after_out {
                self.report(
                    token.span(),
                    "'in' modifier must precede 'out' modifier.",
                    ReportOptions::default(),
                );
            }
            *flag = true;
            if !start_set {
                start = token.start();
                start_set = true;
            }
            self.advance()?;
        }

        let name = match context {
            TypeParameterContext::Declaration => self.parse_binding_identifier(),
            TypeParameterContext::Signature => self.parse_type_binding_identifier(),
        }?;
        let constraint = if self.eat(TokenTag::Extends)? {
            Some(self.parse_type()?)
        } else {
            None
        };
        let default = if self.eat(TokenTag::Assign)? {
            Some(self.parse_type()?)
        } else {
            None
        };
        let end = default.map_or_else(
            || constraint.map_or(name.span(&self.ast).end, |ty| ty.span(&self.ast).end),
            |ty| ty.span(&self.ast).end,
        );
        Some(self.ast.ts_type_parameter(
            Span::new(start, end),
            name,
            constraint,
            default,
            is_in,
            is_out,
            is_const,
        ))
    }

    fn consume_angle_open(&mut self) -> Option<u32> {
        let start = self.current_token.start();
        match self.current_token.tag() {
            TokenTag::LessThan => self.advance()?,
            TokenTag::LeftShift => {
                let token = self.lexer.re_scan_less_than(start);
                self.advance_with_rescanned_token(token)?;
            }
            _ => return None,
        }
        Some(start)
    }

    fn consume_angle_close(&mut self, kind: AngleListKind, close: AngleClose) -> Option<u32> {
        debug_assert!(close != AngleClose::Exact || kind == AngleListKind::Arguments);
        match self.current_token.tag() {
            TokenTag::GreaterThan => {
                let end = self.current_token.end();
                self.advance()?;
                Some(end)
            }
            TokenTag::RightShift
            | TokenTag::UnsignedRightShift
            | TokenTag::GreaterThanEqual
            | TokenTag::RightShiftAssign
            | TokenTag::UnsignedRightShiftAssign => {
                if close == AngleClose::Exact {
                    return None;
                }
                let token = self.lexer.re_scan_greater_than(self.current_token.start());
                let end = token.end();
                self.advance_with_rescanned_token(token)?;
                Some(end)
            }
            _ => {
                if close != AngleClose::Peel {
                    return None;
                }
                self.report_expected(
                    self.current_token.span(),
                    match kind {
                        AngleListKind::Arguments => "Expected '>' to close a type argument list",
                        AngleListKind::Parameters(_) => {
                            "Expected '>' to close a type parameter list"
                        }
                    },
                    ReportOptions {
                        help: Some(Cow::Borrowed("Each '<' in a type must be matched by a '>'")),
                        ..ReportOptions::default()
                    },
                );
                None
            }
        }
    }

    fn parse_for_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::For);
        let start = self.current_token.start();
        self.advance()?;
        if self.current_token.tag() == TokenTag::Await {
            if !self.context.contains(Context::AWAIT) {
                self.report(
                    self.current_token.span(),
                    "'for await' is only valid in async functions or modules",
                    ReportOptions::default(),
                );
            }
            if self
                .boundary_context
                .contains(BoundaryContext::DETECT_MODULE_AWAIT)
            {
                self.record_module_syntax();
            }
            self.advance()?;
            if !self.expect(TokenTag::LeftParen, "Expected '(' after 'for await'", None) {
                return None;
            }
            return self.parse_for_head(start, true);
        }
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'for'", None) {
            return None;
        }
        self.parse_for_head(start, false)
    }

    fn parse_for_head(&mut self, start: u32, is_for_await: bool) -> Option<Stmt> {
        if self.current_token.tag() == TokenTag::Semicolon {
            return self.parse_for_statement_rest(start, None, is_for_await);
        }

        let declaration_start = self.current_token.start();
        match self.current_token.tag() {
            TokenTag::Var | TokenTag::Const => {
                let kind = self.parse_variable_kind()?;
                self.parse_for_with_declaration(start, is_for_await, kind, declaration_start)
            }
            TokenTag::Let if !self.current_token.is_escaped() => {
                let next = self.peek_ahead()?;
                if !can_start_binding_pattern(next.tag()) {
                    return self.parse_for_with_expression(start, is_for_await);
                }
                self.advance()?;
                self.parse_for_with_declaration(
                    start,
                    is_for_await,
                    VariableKind::Let,
                    declaration_start,
                )
            }
            TokenTag::Using => {
                let next = self.peek_ahead()?;
                if next.has_line_terminator_before() || !can_start_using_binding(next.tag()) {
                    return self.parse_for_with_expression(start, is_for_await);
                }

                if next.is_keyword(TokenTag::Of) {
                    let mut peek = self.begin_peek();
                    let after_of = (|| {
                        peek.next()?;
                        peek.next()
                    })();
                    peek.end();
                    if !matches!(
                        after_of?.tag(),
                        TokenTag::Assign | TokenTag::Semicolon | TokenTag::Colon
                    ) {
                        return self.parse_for_with_expression(start, is_for_await);
                    }
                }

                self.advance()?;
                self.parse_for_with_declaration(
                    start,
                    is_for_await,
                    VariableKind::Using,
                    declaration_start,
                )
            }
            TokenTag::Await => {
                let await_span = self.current_token.span();
                if !self.is_await_using_declaration_ahead()? {
                    return self.parse_for_with_expression(start, is_for_await);
                }
                self.report_await_using_if_disallowed(await_span);
                self.advance()?;
                self.advance()?;
                self.parse_for_with_declaration(
                    start,
                    is_for_await,
                    VariableKind::AwaitUsing,
                    declaration_start,
                )
            }
            _ => self.parse_for_with_expression(start, is_for_await),
        }
    }

    fn parse_for_with_declaration(
        &mut self,
        start: u32,
        is_for_await: bool,
        kind: VariableKind,
        declaration_start: u32,
    ) -> Option<Stmt> {
        let context = enter_grammar_context![&mut self.context, ~In];
        let declarators_result = self.parse_variable_declarators(kind, DeclaratorCtx::ForLoop);
        self.context = context;
        let declarators = declarators_result?;
        let declaration_end = self.lexer.state.prev_token_end;
        let declaration = self.ast.variable_declaration(
            Span::new(declaration_start, declaration_end),
            kind,
            declarators,
            false,
        );

        match self.current_token.tag() {
            TokenTag::In => {
                if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
                    self.report(
                        declaration.span(&self.ast),
                        Cow::Owned(format!(
                            "The left-hand side of a for...in statement cannot be an {} declaration.",
                            kind.as_str()
                        )),
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Did you mean to use a for...of statement?",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                }
                self.parse_for_in_statement_rest(
                    start,
                    ForStatementLeft::VariableDeclaration(declaration),
                    is_for_await,
                )
            }
            TokenTag::Of => self.parse_for_of_statement_rest(
                start,
                ForStatementLeft::VariableDeclaration(declaration),
                is_for_await,
            ),
            _ => {
                self.validate_regular_for_declarators(declaration);
                self.parse_for_statement_rest(
                    start,
                    Some(ForStatementInit::VariableDeclaration(declaration)),
                    is_for_await,
                )
            }
        }
    }

    // ForStatement[Yield, Await, Return] :
    //   `for` `(` [lookahead != `let` `[`] Expression[~In, ?Yield, ?Await]? `;` Expression[+In, ?Yield, ?Await]? `;` Expression[+In, ?Yield, ?Await]? `)` Statement[?Yield, ?Await, ?Return]
    fn parse_for_with_expression(&mut self, start: u32, is_for_await: bool) -> Option<Stmt> {
        self.with_resolved_cover(|parser| {
            let head = parser.current_token;
            let head_is_let = head.tag() == TokenTag::Let && !head.is_escaped();
            let context = enter_grammar_context![&mut parser.context, ~In];
            let expression = parser.parse_expression_in_cover(Precedence::Comma);
            parser.context = context;
            let expression = expression?;

            if parser.current_token.tag() == TokenTag::In {
                let left = ForStatementLeft::AssignmentTarget(
                    parser.expression_to_assignment_target(expression)?,
                );
                return parser.parse_for_in_statement_rest(start, left, is_for_await);
            }

            if parser.current_token.tag() == TokenTag::Of {
                if head_is_let {
                    parser.report(
                        head.span(),
                        "The left-hand side of a for-of statement may not start with 'let'",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Rename the variable or wrap it in parentheses: 'for ((let).x of ...)'",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                }
                if !is_for_await && parser.is_async_identifier(expression) {
                    parser.report(
                        expression.span(&parser.ast),
                        "'for (async of ...)' is not allowed, it is ambiguous with 'for await'",
                        ReportOptions::default(),
                    );
                }
                let left = ForStatementLeft::AssignmentTarget(
                    parser.expression_to_assignment_target(expression)?,
                );
                return parser.parse_for_of_statement_rest(start, left, is_for_await);
            }

            parser.parse_for_statement_rest(
                start,
                Some(ForStatementInit::Expr(expression)),
                is_for_await,
            )
        })
    }

    // ForStatement[Yield, Await, Return] :
    //   `for` `(` [lookahead != `let` `[`] Expression[~In, ?Yield, ?Await]? `;` Expression[+In, ?Yield, ?Await]? `;` Expression[+In, ?Yield, ?Await]? `)` Statement[?Yield, ?Await, ?Return]
    //   `for` `(` `var` VariableDeclarationList[~In, ?Yield, ?Await] `;` Expression[+In, ?Yield, ?Await]? `;` Expression[+In, ?Yield, ?Await]? `)` Statement[?Yield, ?Await, ?Return]
    //   `for` `(` LexicalDeclaration[~In, ?Yield, ?Await] Expression[+In, ?Yield, ?Await]? `;` Expression[+In, ?Yield, ?Await]? `)` Statement[?Yield, ?Await, ?Return]
    fn parse_for_statement_rest(
        &mut self,
        start: u32,
        init: Option<ForStatementInit>,
        is_for_await: bool,
    ) -> Option<Stmt> {
        if is_for_await {
            return {
                let span = self.current_token.span();
                self.report(
                    span,
                    "'for await' is only valid with for-of loops",
                    ReportOptions::default(),
                );
                None
            };
        }
        if !self.expect(
            TokenTag::Semicolon,
            "Expected ';' after for-loop init",
            None,
        ) {
            return None;
        }
        let test = if self.current_token.tag() == TokenTag::Semicolon {
            None
        } else {
            // The program or function/static block entry already establishes +In.
            Some(self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?)
        };
        if !self.expect(
            TokenTag::Semicolon,
            "Expected ';' after for-loop condition",
            None,
        ) {
            return None;
        }
        let update = if self.current_token.tag() == TokenTag::RightParen {
            None
        } else {
            // The program or function/static block entry already establishes +In.
            Some(self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?)
        };
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after for-loop update",
            None,
        ) {
            return None;
        }
        let body = self.parse_statement(StatementContext::Statement)?;
        let end = body.span(&self.ast).end;
        Some(
            self.ast
                .stmt_for_statement(Span::new(start, end), init, test, update, body),
        )
    }

    // ForInOfStatement[Yield, Await, Return] :
    //   `for` `(` [lookahead != `let` `[`] LeftHandSideExpression[?Yield, ?Await] `in` Expression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    //   `for` `(` `var` ForBinding[?Yield, ?Await, +Pattern] `in` Expression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    //   `for` `(` ForDeclaration[?Yield, ?Await, ~Using] `in` Expression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    fn parse_for_in_statement_rest(
        &mut self,
        start: u32,
        left: ForStatementLeft,
        is_for_await: bool,
    ) -> Option<Stmt> {
        if is_for_await {
            return {
                let span = self.current_token.span();
                self.report(
                    span,
                    "'for await' requires 'of', not 'in'",
                    ReportOptions::default(),
                );
                None
            };
        }
        self.report_if_escaped_keyword(self.current_token);
        self.advance_without_escape_check()?;
        // The program or function/static block entry already establishes +In.
        let right = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after for-in expression",
            None,
        ) {
            return None;
        }
        let body = self.parse_statement(StatementContext::Statement)?;
        let end = body.span(&self.ast).end;
        Some(
            self.ast
                .stmt_for_in_statement(Span::new(start, end), left, right, body),
        )
    }

    // ForInOfStatement[Yield, Await, Return] :
    //   `for` `(` [lookahead ∉ { `let`, `async` `of` }] LeftHandSideExpression[?Yield, ?Await] `of` AssignmentExpression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    //   `for` `(` `var` ForBinding[?Yield, ?Await, +Pattern] `of` AssignmentExpression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    //   `for` `(` [lookahead != `using` `of`] ForDeclaration[?Yield, ?Await, +Using] `of` AssignmentExpression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    //   [+Await] `for` `await` `(` [lookahead != `let`] LeftHandSideExpression[?Yield, ?Await] `of` AssignmentExpression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    //   [+Await] `for` `await` `(` `var` ForBinding[?Yield, ?Await, +Pattern] `of` AssignmentExpression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    //   [+Await] `for` `await` `(` [lookahead != `using` `of`] ForDeclaration[?Yield, ?Await, +Using] `of` AssignmentExpression[+In, ?Yield, ?Await] `)` Statement[?Yield, ?Await, ?Return]
    fn parse_for_of_statement_rest(
        &mut self,
        start: u32,
        left: ForStatementLeft,
        is_for_await: bool,
    ) -> Option<Stmt> {
        self.report_if_escaped_keyword(self.current_token);
        self.advance_without_escape_check()?;
        // The program or function/static block entry already establishes +In.
        let right = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after for-of expression",
            None,
        ) {
            return None;
        }
        let body = self.parse_statement(StatementContext::Statement)?;
        let end = body.span(&self.ast).end;
        Some(
            self.ast
                .stmt_for_of_statement(Span::new(start, end), left, right, body, is_for_await),
        )
    }

    fn is_async_identifier(&self, expression: Expr) -> bool {
        matches!(
            self.ast.expr_data(expression),
            ExprData::IdentifierReference(identifier)
                if self.ast.get_utf8(identifier.name(&self.ast)) == "async"
                    && self.span_text(identifier.span(&self.ast)) == "async"
        )
    }

    fn validate_regular_for_declarators(&mut self, declaration: VariableDeclaration) {
        let kind = declaration.kind(&self.ast);
        let declarators = declaration.declarators(&self.ast);
        for index in 0..declarators.len() {
            let declarator = declarators
                .get_node(&self.ast, index)
                .expect("variable declarator range contains its nodes");
            if declarator.init(&self.ast).is_some() {
                continue;
            }
            let id = declarator.id(&self.ast);
            if self.ast.node_kind(id.node_id()) != NodeKind::BindingIdentifier {
                self.report(
                    id.span(&self.ast),
                    "Destructuring declaration in for loop initializer must be initialized",
                    ReportOptions::default(),
                );
            } else if kind == VariableKind::Const {
                self.report(
                    id.span(&self.ast),
                    "'const' declarations in for loop initializer must be initialized",
                    ReportOptions::default(),
                );
            } else if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
                self.report(
                    id.span(&self.ast),
                    Cow::Owned(format!(
                        "'{}' declarations must be initialized",
                        kind.as_str()
                    )),
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Disposable resources require an initial value that implements the dispose protocol.",
                        )),
                        ..ReportOptions::default()
                    },
                );
            }
        }
    }

    // SwitchStatement[Yield, Await, Return] :
    //   `switch` `(` Expression[+In, ?Yield, ?Await] `)` CaseBlock[?Yield, ?Await, ?Return]
    fn parse_switch_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Switch);
        let start = self.current_token.start();
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'switch'", None) {
            return None;
        }
        // The program or function/static block entry already establishes +In.
        let discriminant = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after switch expression",
            None,
        ) {
            return None;
        }
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start switch body",
            None,
        ) {
            return None;
        }
        let cases = self.parse_switch_cases()?;
        let end = self.current_token.end();
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close switch body",
            None,
        ) {
            return None;
        }
        Some(
            self.ast
                .stmt_switch_statement(Span::new(start, end), discriminant, cases),
        )
    }

    fn parse_switch_cases(&mut self) -> Option<SubRange<SwitchCase>> {
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while matches!(self.current_token.tag(), TokenTag::Case | TokenTag::Default) {
                let case = self.parse_switch_case()?;
                self.scratch_a.append(case);
            }
            Some(self.add_extra_from_scratch_a(checkpoint))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    // CaseClause[Yield, Await, Return] :
    //   `case` Expression[+In, ?Yield, ?Await] `:` StatementList[?Yield, ?Await, ?Return]?
    fn parse_switch_case(&mut self) -> Option<SwitchCase> {
        debug_assert!(matches!(
            self.current_token.tag(),
            TokenTag::Case | TokenTag::Default
        ));
        let case_start = self.current_token.start();
        let test = if self.eat(TokenTag::Case)? {
            // The program or function/static block entry already establishes +In.
            Some(self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?)
        } else {
            self.advance()?;
            None
        };
        let colon_end = self.current_token.end();
        if !self.expect(TokenTag::Colon, "Expected ':' after case", None) {
            return None;
        }
        let consequent = self.parse_case_consequent()?;
        let case_end = if consequent.is_empty() {
            colon_end
        } else {
            consequent
                .get_node(&self.ast, consequent.len() - 1)
                .expect("non-empty case consequent contains its last statement")
                .span(&self.ast)
                .end
        };
        Some(
            self.ast
                .switch_case(Span::new(case_start, case_end), test, consequent),
        )
    }

    fn parse_case_consequent(&mut self) -> Option<SubRange<Stmt>> {
        let checkpoint = self.scratch_b.begin();
        let result = (|| {
            while !matches!(
                self.current_token.tag(),
                TokenTag::Case | TokenTag::Default | TokenTag::RightBrace | TokenTag::Eof
            ) {
                let statement = self.parse_statement(StatementContext::StatementListItem)?;
                if let StmtData::Declaration(declaration) = self.ast.stmt_data(statement)
                    && let DeclData::VariableDeclaration(declaration) =
                        self.ast.decl_data(declaration)
                    && matches!(
                        declaration.kind(&self.ast),
                        VariableKind::Using | VariableKind::AwaitUsing
                    )
                {
                    self.report(
                        statement.span(&self.ast),
                        "Using declaration cannot appear in the bare case statement.",
                        ReportOptions {
                            help: Some(Cow::Borrowed("Wrap this declaration in a block statement")),
                            ..ReportOptions::default()
                        },
                    );
                }
                self.scratch_b.append(statement);
            }
            Some(self.add_extra_from_scratch_b(checkpoint))
        })();
        self.scratch_b.reset(checkpoint);
        result
    }

    fn parse_try_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Try);
        let start = self.current_token.start();
        self.advance()?;
        let block = self.parse_block_statement()?;
        let handler = if self.current_token.tag() == TokenTag::Catch {
            self.parse_catch_clause()?.into()
        } else {
            None
        };
        let finalizer = if self.eat(TokenTag::Finally)? {
            self.parse_block_statement()?.into()
        } else {
            None
        };
        if handler.is_none() && finalizer.is_none() {
            self.report(
                self.current_token.span(),
                "Try statement requires catch or finally clause",
                ReportOptions::default(),
            );
        }
        let end = finalizer.map_or_else(
            || handler.map_or(block.span(&self.ast).end, |node| node.span(&self.ast).end),
            |node| node.span(&self.ast).end,
        );
        Some(
            self.ast
                .stmt_try_statement(Span::new(start, end), block, handler, finalizer),
        )
    }

    fn parse_catch_clause(&mut self) -> Option<CatchClause> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Catch);
        let start = self.current_token.start();
        self.advance()?;
        let mut type_annotation = None;
        let param = if self.eat(TokenTag::LeftParen)? {
            let param = self.parse_binding_pattern()?;
            if self.lang.is_typescript() && self.current_token.tag() == TokenTag::Colon {
                type_annotation = self.parse_type_annotation()?.into();
            }
            if !self.expect(
                TokenTag::RightParen,
                "Expected ')' after catch parameter",
                None,
            ) {
                return None;
            }
            Some(param)
        } else {
            None
        };
        let body = self.parse_block_statement()?;
        Some(self.ast.catch_clause(
            Span::new(start, body.span(&self.ast).end),
            param,
            type_annotation,
            body,
        ))
    }

    // Expression[In, Yield, Await] :
    //   AssignmentExpression[?In, ?Yield, ?Await]
    //   Expression[?In, ?Yield, ?Await] `,` AssignmentExpression[?In, ?Yield, ?Await]
    //
    // INLINE: This is a small and hot function.
    // Propagate the caller's precedence and TypeScript binding role into expression parsing.
    // Keep this wrapper inline when the larger Pratt loop below is forced inline.
    #[inline(always)]
    fn parse_expr(&mut self, minimum: Precedence, opts: ParseExpressionOpts) -> Option<Expr> {
        self.parse_expr_inner(minimum, opts.allow_type_binding_identifiers)
    }

    #[inline(always)]
    fn parse_expr_inner(
        &mut self,
        minimum: Precedence,
        allow_type_binding_identifiers: bool,
    ) -> Option<Expr> {
        let left = if self.current_token.tag() == TokenTag::Identifier {
            self.parse_plain_identifier()?
        } else {
            self.parse_prefix(allow_type_binding_identifiers, minimum)?
        };
        self.parse_expr_from_left(left, minimum)
    }

    /// Pair candidate lifetime with role resolution, including failed recovery.
    #[inline]
    fn with_resolved_cover<T>(&mut self, parse: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        let start = self.cover.start_expression();
        let result = parse(self);
        self.cover.commit_expression(start, &mut self.diagnostics);
        result
    }

    #[inline(always)]
    fn parse_value_expression(&mut self, minimum: Precedence) -> Option<Expr> {
        let ts_context = self.ts_context;
        self.ts_context.remove(TsContext::IN_TYPE_BINDING);
        let expression = self.with_resolved_cover(|parser| {
            parser.parse_expr(minimum, ParseExpressionOpts::default())
        });
        self.ts_context = ts_context;
        expression
    }

    // RelationalExpression[In, Yield, Await] :
    //   [+In] RelationalExpression[+In, ?Yield, ?Await] `in` ShiftExpression[?Yield, ?Await]
    //
    // Specialize the Pratt loop for the caller's minimum precedence instead of
    // passing and preserving a runtime minimum across expression-parser calls.
    //
    // TODO: this bloats binary size a lot
    #[inline(always)]
    fn parse_expr_from_left(&mut self, mut left: Expr, minimum: Precedence) -> Option<Expr> {
        let is_ts = self.lang.is_typescript();
        let tag = self.current_token.tag();
        if (tag.precedence() >= Precedence::Call
            || is_ts && (is_angle_open(tag) || tag == TokenTag::LogicalNot))
            && max_left_precedence(left, &self.ast) >= Precedence::Call
        {
            left = self
                .parse_postfix_expression(left, LhsContext::Expression)?
                .expression;
        }

        loop {
            let token = self.current_token;
            let precedence = Self::infix_precedence(token, is_ts);
            if precedence < minimum || precedence == Precedence::Lowest {
                break;
            }
            if precedence > max_left_precedence(left, &self.ast) {
                break;
            }
            left = if token.tag().is_binary_operator() {
                if token.tag() == TokenTag::In && !self.context.contains(Context::IN) {
                    break;
                }
                self.parse_binary_expression(left, token.tag())?
            } else if token.tag().is_logical_operator() {
                self.parse_logical_expression(left, token.tag())?
            } else if token.tag().is_assignment_operator() {
                self.parse_assignment_expression(left, token.tag())?
            } else {
                self.parse_infix(precedence, left)?
            };
        }
        Some(left)
    }

    // Expression[In, Yield, Await] :
    //   AssignmentExpression[?In, ?Yield, ?Await]
    //   Expression[?In, ?Yield, ?Await] `,` AssignmentExpression[?In, ?Yield, ?Await]
    // Cover callers know their grammar precedence just like ordinary expressions.
    #[inline(always)]
    fn parse_expression_in_cover(&mut self, precedence: Precedence) -> Option<Expr> {
        self.parse_expr(
            precedence,
            ParseExpressionOpts {
                allow_type_binding_identifiers: true,
            },
        )
    }

    fn parse_assignment_expression_in_cover(&mut self) -> Option<Expr> {
        self.parse_expression_in_cover(Precedence::Assignment)
    }

    #[inline]
    fn infix_precedence(token: Token, is_typescript: bool) -> Precedence {
        let precedence = token.tag().precedence();
        // Most expression terminators have no infix precedence at all.
        if precedence == Precedence::Lowest {
            return precedence;
        }
        match token.tag() {
            TokenTag::Increment | TokenTag::Decrement | TokenTag::As | TokenTag::Satisfies
                if token.has_line_terminator_before() =>
            {
                Precedence::Lowest
            }
            TokenTag::LogicalNot if !is_typescript || token.has_line_terminator_before() => {
                Precedence::Lowest
            }
            _ => precedence,
        }
    }

    fn parse_infix(&mut self, precedence: Precedence, left: Expr) -> Option<Expr> {
        let token = self.current_token;
        match token.tag() {
            TokenTag::As | TokenTag::Satisfies if self.lang.is_typescript() => {
                return self.parse_as_or_satisfies_expression(left, token.tag());
            }
            TokenTag::LogicalNot
                if self.lang.is_typescript() && !token.has_line_terminator_before() =>
            {
                return Some(
                    self.parse_postfix_expression(left, LhsContext::Expression)?
                        .expression,
                );
            }
            _ => {}
        }

        if token.tag() == TokenTag::Arrow {
            return self.parse_simple_arrow_function(left);
        }

        match token.tag() {
            TokenTag::Increment | TokenTag::Decrement => {
                self.parse_update_expression(false, Some(left))
            }
            TokenTag::Question => self.parse_conditional_expression(left),
            TokenTag::Comma => self.parse_sequence_expression(precedence, left),
            _ => {
                let found = self.describe_token(token);
                self.report(
                    token.span(),
                    Cow::Owned(format!(
                        "Unexpected token '{}' in expression",
                        found.as_ref()
                    )),
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "This token cannot be used here. Expected an operator, semicolon, or end of expression.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
        }
    }

    fn parse_prefix(
        &mut self,
        allow_type_binding_identifiers: bool,
        minimum: Precedence,
    ) -> Option<Expr> {
        let tag = self.current_token.tag();

        if matches!(tag, TokenTag::Increment | TokenTag::Decrement) {
            return self.parse_update_expression(true, None);
        }

        if tag.is_unary_operator() {
            return self.parse_unary_expression();
        }

        if tag == TokenTag::LeftParen {
            // Select the grammar mode before parsing the cover so `minimum` does not
            // need to remain live across the parse.
            if minimum <= Precedence::Assignment {
                return self.parse_parenthesized_or_arrow_function::<true>(None);
            }
            return self.parse_parenthesized_or_arrow_function::<false>(None);
        }

        if tag == TokenTag::Await && self.context.contains(Context::AWAIT) {
            if allow_type_binding_identifiers && self.ts_context.in_type_binding() {
                return self.parse_identifier();
            }
            let token = self.current_token;
            let can_classify_as_module = self
                .boundary_context
                .contains(BoundaryContext::DETECT_MODULE_AWAIT);
            self.advance_without_escape_check()?;
            self.report_if_escaped_keyword(token);
            let ambiguous_script_different_ast =
                can_classify_as_module && self.is_ambiguous_await_prefix();
            let diagnostics_len = self.diagnostics.len();
            let expression = self.parse_await_expression(token.start())?;
            if can_classify_as_module {
                // A recovered operand cannot establish that `await` is a keyword.
                let ambiguous = ambiguous_script_different_ast
                    || !self.diagnostics.is_clean_since(diagnostics_len);
                if let Some(first) = self.module_detection.record_await(ambiguous) {
                    self.diagnostics.defer(first.diagnostic());
                }
            }
            return Some(expression);
        }

        if tag == TokenTag::Yield
            && self.context.contains(Context::YIELD)
            && minimum < Precedence::Add
        {
            if allow_type_binding_identifiers && self.ts_context.in_type_binding() {
                return self.parse_identifier();
            }
            return self.parse_yield_expression();
        }

        if tag == TokenTag::New {
            return self.parse_new_expression();
        }

        if tag == TokenTag::Import {
            return self.parse_import_expression();
        }

        if tag == TokenTag::Async {
            return self.parse_async_function_or_arrow(minimum);
        }

        if tag == TokenTag::LessThan && self.lang.is_typescript() {
            let start = self.current_token.start();
            if let Some(arrow) = self.try_parse_generic_arrow(false, start)? {
                return Some(arrow);
            }
            if !self.lang.is_jsx() {
                return self.parse_type_assertion();
            }
            return self.parse_jsx_expression();
        }

        self.parse_primary_expression(minimum)
    }

    fn parse_primary_expression(&mut self, minimum: Precedence) -> Option<Expr> {
        let token = self.current_token;

        if token.tag().is_numeric_literal() {
            return self.parse_numeric_literal();
        }

        match token.tag() {
            TokenTag::PrivateIdentifier => {
                let private = self.parse_private_identifier()?;
                if self.current_token.tag() != TokenTag::In || minimum > Precedence::Compare {
                    self.report(
                        token.span(),
                        "Private names are only valid in property accesses (`obj.#field`) or `in` expressions (`#field in obj`)",
                        ReportOptions::default(),
                    );
                }
                Some(Expr::PrivateIdentifier(private))
            }
            TokenTag::This => self.parse_this_expression(),
            TokenTag::Super => self.parse_super_expression(),
            TokenTag::True | TokenTag::False => self.parse_boolean_literal(),
            TokenTag::NullLiteral => self.parse_null_literal(),
            TokenTag::StringLiteral => Some(Expr::StringLiteral(self.parse_string_literal()?)),
            TokenTag::NoSubstitutionTemplate => Some(Expr::TemplateLiteral(
                self.parse_no_substitution_template(false)?,
            )),
            TokenTag::TemplateHead => {
                Some(Expr::TemplateLiteral(self.parse_template_literal(false)?))
            }
            TokenTag::Slash | TokenTag::SlashAssign => self.parse_regexp_literal(),
            TokenTag::LeftBracket => self.parse_array_expression(),
            TokenTag::LeftBrace => self.parse_object_expression(),
            TokenTag::LessThan if self.lang.is_jsx() => {
                let expression = self.parse_jsx_expression()?;
                if self.lang.is_typescript() {
                    self.report(
                        expression.span(&self.ast),
                        "JSX expressions must be parenthesized in this position in TypeScript",
                        ReportOptions::default(),
                    );
                }
                Some(expression)
            }
            TokenTag::Async => {
                self.consume_identifier(IdentifierRole::Reference)?;
                if self.current_token.tag() == TokenTag::Function
                    && !self.current_token.has_line_terminator_before()
                {
                    return self.parse_async_function_expression(token);
                }
                let name = self.identifier_name(token);
                Some(self.ast.expr_identifier_reference(token.span(), name))
            }
            TokenTag::Function => Some(Expr::Function(
                self.parse_function(
                    ParseFunctionOpts {
                        is_expression: true,
                        ..ParseFunctionOpts::default()
                    },
                    None,
                )?
                .into_function_expression(),
            )),
            TokenTag::At => {
                let (start, decorators) = self.parse_decorators()?;
                Some(Expr::Class(self.parse_class_decorated(
                    ParseClassOpts {
                        is_expression: true,
                        ..ParseClassOpts::default()
                    },
                    Some(start),
                    decorators,
                )?))
            }
            TokenTag::Class => Some(Expr::Class(self.parse_class(
                ParseClassOpts {
                    is_expression: true,
                    ..ParseClassOpts::default()
                },
                None,
            )?)),
            // `import.meta` is a PrimaryExpression and can therefore be the
            // constructor target in `new import.meta()`. Import calls share
            // this parser entry and are rejected by `parse_new_expression`.
            TokenTag::Import => self.parse_import_expression(),
            TokenTag::Await | TokenTag::Yield => self.parse_recovered_await_or_yield(minimum),
            tag if tag.is_identifier_like() => self.parse_identifier(),
            _ => {
                let found = self.describe_token(token);
                self.report(
                    token.span(),
                    Cow::Owned(format!("Unexpected token '{}'", found.as_ref())),
                    ReportOptions {
                        help: Some(Cow::Borrowed("Expected an expression")),
                        ..ReportOptions::default()
                    },
                );
                None
            }
        }
    }

    fn parse_identifier(&mut self) -> Option<Expr> {
        let token = self.consume_identifier(IdentifierRole::Reference)?;
        let name = self.identifier_name(token);
        Some(self.ast.expr_identifier_reference(token.span(), name))
    }

    #[cold]
    fn parse_recovered_await_or_yield(&mut self, minimum: Precedence) -> Option<Expr> {
        let tag = self.current_token.tag();
        if minimum > Precedence::Prefix || minimum >= Precedence::Add && tag == TokenTag::Yield {
            return self.parse_identifier();
        }
        let can_recover = self
            .can_recover_await_or_yield_expression()
            .unwrap_or(false);
        if !can_recover {
            return self.parse_identifier();
        }

        let token = self.current_token;
        let expression = if tag == TokenTag::Yield {
            self.parse_yield_expression()?
        } else {
            self.advance_without_escape_check()?;
            self.report_if_escaped_keyword(token);
            self.parse_await_expression(token.start())?
        };
        if !self.ts_context.in_ts_container_body() {
            let message = if tag == TokenTag::Await {
                "Await expressions are only allowed within async functions and at the top level of modules"
            } else {
                "A yield expression is only allowed in a generator body"
            };
            self.report(
                expression.span(&self.ast),
                message,
                ReportOptions::default(),
            );
        }
        Some(expression)
    }

    // INLINE: This is a small and hot function.
    // Carry the current token into identifier construction without leaving a
    // wrapper call when parse_identifier_from_token is forced inline.
    #[inline(always)]
    fn parse_plain_identifier(&mut self) -> Option<Expr> {
        let token = self.current_token;
        debug_assert_eq!(token.tag(), TokenTag::Identifier);
        self.parse_identifier_from_token(token)
    }

    // INLINE: This is a small and hot function.
    // Combine token advancement and identifier construction in the caller,
    // removing this helper's call/return and register save/restore overhead.
    #[inline(always)]
    fn parse_identifier_from_token(&mut self, token: Token) -> Option<Expr> {
        self.advance_without_escape_check()?;
        let name = self.identifier_name(token);
        Some(self.ast.expr_identifier_reference(token.span(), name))
    }

    fn parse_private_identifier(&mut self) -> Option<PrivateIdentifier> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::PrivateIdentifier);
        let token = self.current_token;
        self.advance()?;
        let name = self.identifier_name(token);
        Some(self.ast.private_identifier(token.span(), name))
    }

    // INLINE: This is a small and hot function.
    // Let member parsing reuse token fields across property-name and member-node
    // construction instead of crossing a separate identifier-name helper call.
    #[inline(always)]
    fn parse_identifier_name(&mut self) -> Option<IdentifierName> {
        self.parse_identifier_name_with_role(IdentifierRole::Name)
    }

    #[inline(always)]
    fn parse_identifier_name_with_role(&mut self, role: IdentifierRole) -> Option<IdentifierName> {
        let token = self.consume_identifier(role)?;
        let name = self.identifier_name(token);
        Some(self.ast.identifier_name(token.span(), name))
    }

    fn parse_this_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::This);
        let token = self.current_token;
        self.advance()?;
        Some(self.ast.expr_this_expression(token.span()))
    }

    fn parse_super_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Super);
        let token = self.current_token;
        self.advance()?;
        if !matches!(
            self.current_token.tag(),
            TokenTag::LeftParen | TokenTag::Dot | TokenTag::LeftBracket
        ) {
            self.report(
                self.current_token.span(),
                "'super' must be followed by a call or property access",
                ReportOptions::default(),
            );
        }
        Some(self.ast.expr_super(token.span()))
    }

    fn parse_boolean_literal(&mut self) -> Option<Expr> {
        debug_assert!(matches!(
            self.current_token.tag(),
            TokenTag::True | TokenTag::False
        ));
        let token = self.current_token;
        self.advance()?;
        Some(
            self.ast
                .expr_boolean_literal(token.span(), token.tag() == TokenTag::True),
        )
    }

    fn parse_null_literal(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::NullLiteral);
        let token = self.current_token;
        self.advance()?;
        Some(self.ast.expr_null_literal(token.span()))
    }

    fn parse_numeric_literal(&mut self) -> Option<Expr> {
        debug_assert!(self.current_token.tag().is_numeric_literal());
        let token = self.current_token;
        if token.tag() == TokenTag::BigIntLiteral {
            self.advance()?;
            let raw = self.ast.source_slice(token.start(), token.end() - 1);
            return Some(self.ast.expr_big_int_literal(token.span(), raw));
        }
        self.advance()?;
        let raw = self.ast.source_slice(token.start(), token.end());
        let raw_text = self.ast.get_utf8(raw);
        let value = match token.tag() {
            TokenTag::NumericLiteral => parse_decimal_literal(raw_text),
            TokenTag::HexLiteral => parse_hex_literal(raw_text),
            TokenTag::OctalLiteral => parse_octal_literal(raw_text),
            TokenTag::BinaryLiteral => parse_binary_literal(raw_text),
            _ => unreachable!(),
        };
        Some(
            self.ast
                .expr_numeric_literal(token.span(), value, Some(raw)),
        )
    }

    fn parse_string_literal(&mut self) -> Option<StringLiteral> {
        let token = self.current_token;
        if token.tag() != TokenTag::StringLiteral {
            self.report_expected(
                self.current_token.span(),
                "Expected a string literal",
                ReportOptions::default(),
            );
            return None;
        }
        self.advance()?;
        let value = self.string_value(token);
        let raw = self.ast.source_slice(token.start(), token.end());
        Some(self.ast.string_literal(token.span(), value, raw))
    }

    fn parse_regexp_literal(&mut self) -> Option<Expr> {
        debug_assert!(matches!(
            self.current_token.tag(),
            TokenTag::Slash | TokenTag::SlashAssign
        ));
        let token = self.current_token;
        let result = match self.lexer.re_scan_as_regex(token.start()) {
            Ok(result) => result,
            Err(error) => {
                self.report(
                    token.span(),
                    lexical_error_message(error),
                    ReportOptions {
                        help: Some(Cow::Borrowed(lexical_error_help(error))),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
        };
        let regex_token =
            self.lexer
                .create_token(TokenTag::RegexLiteral, result.span.start, result.span.end);
        self.advance_with_rescanned_token(regex_token)?;
        let pattern = self
            .ast
            .source_slice(result.pattern.start, result.pattern.end);
        let flags = self.ast.source_slice(result.flags.start, result.flags.end);
        Some(self.ast.expr_reg_exp_literal(result.span, pattern, flags))
    }

    fn parse_no_substitution_template(&mut self, tagged: bool) -> Option<TemplateLiteral> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::NoSubstitutionTemplate);
        let token = self.current_token;
        let quasi = self.add_template_element(token, true, tagged);
        self.advance()?;
        let quasis = self.ast.add_sub_range([quasi]);
        let expressions = SubRange::empty();
        Some(self.ast.template_literal(token.span(), quasis, expressions))
    }

    // SubstitutionTemplate[Yield, Await] :
    //   TemplateHead Expression[+In, ?Yield, ?Await] TemplateSpans[?Yield, ?Await]
    //
    // TemplateMiddleList[Yield, Await] :
    //   TemplateMiddle Expression[+In, ?Yield, ?Await]
    //   TemplateMiddleList[?Yield, ?Await] TemplateMiddle Expression[+In, ?Yield, ?Await]
    fn parse_template_literal(&mut self, tagged: bool) -> Option<TemplateLiteral> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::TemplateHead);
        let start = self.current_token.start();
        let quasis_checkpoint = self.scratch_a.begin();
        let expressions_checkpoint = self.scratch_b.begin();
        let head = self.current_token;
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            let head = self.add_template_element(head, false, tagged);
            self.scratch_a.append(head);
            self.advance()?;
            let end = loop {
                let expression =
                    self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
                self.scratch_b.append(expression);
                if self.current_token.tag() != TokenTag::RightBrace {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected '}' to close template expression",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Template expressions must be closed with '}'",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
                let right_brace = self.current_token;
                let continuation = match self
                    .lexer
                    .re_scan_template_continuation(right_brace.start())
                {
                    Ok(token) => token,
                    Err(error) => {
                        self.report(
                            right_brace.span(),
                            lexical_error_message(error),
                            ReportOptions {
                                help: Some(Cow::Borrowed(lexical_error_help(error))),
                                ..ReportOptions::default()
                            },
                        );
                        return None;
                    }
                };
                let tail = continuation.tag() == TokenTag::TemplateTail;
                let element = self.add_template_element(continuation, tail, tagged);
                self.scratch_a.append(element);
                let end = continuation.end();
                self.advance_with_rescanned_token(continuation)?;
                if tail {
                    break end;
                }
            };
            let quasis = self.add_extra_from_scratch_a(quasis_checkpoint);
            let expressions = self.add_extra_from_scratch_b(expressions_checkpoint);
            Some(
                self.ast
                    .template_literal(Span::new(start, end), quasis, expressions),
            )
        })();
        self.ts_context = ts_context;
        self.context = grammar_context;
        self.scratch_a.reset(quasis_checkpoint);
        self.scratch_b.reset(expressions_checkpoint);
        result
    }

    fn parse_tagged_template_expression(
        &mut self,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        in_optional_chain: bool,
    ) -> Option<Expr> {
        if in_optional_chain {
            self.report(
                self.current_token.span(),
                "Tagged template expressions are not permitted in an optional chain",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Parenthesize the optional chain before the template literal.",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        let start = tag.span(&self.ast).start;
        let quasi = if self.current_token.tag() == TokenTag::NoSubstitutionTemplate {
            self.parse_no_substitution_template(true)?
        } else {
            self.parse_template_literal(true)?
        };
        Some(self.ast.expr_tagged_template_expression(
            Span::new(start, quasi.span(&self.ast).end),
            tag,
            type_arguments,
            quasi,
        ))
    }

    fn add_template_element(&mut self, token: Token, tail: bool, tagged: bool) -> TemplateElement {
        let span = self.get_template_element_span(token);
        let raw = self.template_element_raw(token, span);
        let is_cooked_undefined = token.has_invalid_escape();
        if is_cooked_undefined && !tagged {
            self.report(
                span,
                "Bad escape sequence in untagged template literal",
                ReportOptions::default(),
            );
        }
        let cooked = if is_cooked_undefined {
            AstWtf8::EMPTY
        } else {
            self.template_element_value(token, span)
        };
        self.ast
            .template_element(span, cooked, raw, tail, is_cooked_undefined)
    }

    fn get_template_element_span(&self, token: Token) -> Span {
        match token.tag() {
            TokenTag::TemplateHead | TokenTag::TemplateMiddle => {
                Span::new(token.start() + 1, token.end() - 2)
            }
            TokenTag::TemplateTail | TokenTag::NoSubstitutionTemplate => {
                Span::new(token.start() + 1, token.end() - 1)
            }
            _ => unreachable!(),
        }
    }

    fn parse_parenthesized_or_arrow_function<const CAN_BE_ARROW: bool>(
        &mut self,
        arrow_start: Option<u32>,
    ) -> Option<Expr> {
        let start = arrow_start.unwrap_or(self.current_token.start());
        if self.lang.is_typescript() && CAN_BE_ARROW {
            match self.classify_arrow_head() {
                ArrowHead::Yes => {
                    return self.parse_arrow(false, start);
                }
                ArrowHead::Maybe => {
                    if let Some(arrow) = self.try_parse_arrow(false, start)? {
                        return Some(arrow);
                    }
                }
                ArrowHead::No => {}
            }
        }
        self.with_resolved_cover(|parser| {
            let cover = parser.parse_parenthesized_cover()?;
            let is_arrow = parser.current_token.tag() == TokenTag::Arrow && CAN_BE_ARROW;
            if is_arrow {
                return parser.cover_to_arrow_function(cover, false, start);
            }
            parser.cover_to_parenthesized_expression(cover)
        })
    }

    // CoverParenthesizedExpressionAndArrowParameterList[Yield, Await] :
    //   `(` `)`
    //   `(` Expression[+In, ?Yield, ?Await] `,`? `)`
    //   `(` `...` BindingIdentifier[?Yield, ?Await] `)`
    //   `(` `...` BindingPattern[?Yield, ?Await] `)`
    //   `(` Expression[+In, ?Yield, ?Await] `,` `...` BindingIdentifier[?Yield, ?Await] `)`
    //   `(` Expression[+In, ?Yield, ?Await] `,` `...` BindingPattern[?Yield, ?Await] `)`
    fn parse_parenthesized_cover(&mut self) -> Option<ParenthesizedCover> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftParen);
        let start = self.current_token.start();
        self.advance()?;
        let checkpoint = self.cover.scratch.begin();
        let grammar_context = enter_grammar_context![&mut self.context, +In, ?Yield, ?Await];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            let mut end = start + 1;
            let mut has_trailing_comma = false;
            while self.current_token.tag() != TokenTag::RightParen
                && self.current_token.tag() != TokenTag::Eof
            {
                let element = if self.eat(TokenTag::Spread)? {
                    let spread_start = self.lexer.state.prev_token_end - 3;
                    let argument = self.parse_expr(
                        Precedence::Assignment,
                        ParseExpressionOpts {
                            allow_type_binding_identifiers: true,
                        },
                    )?;
                    Argument::SpreadElement(self.ast.spread_element(
                        Span::new(spread_start, argument.span(&self.ast).end),
                        argument,
                    ))
                } else {
                    let expression = self.parse_expr(
                        Precedence::Assignment,
                        ParseExpressionOpts {
                            allow_type_binding_identifiers: true,
                        },
                    )?;
                    Argument::Expr(expression)
                };
                end = element.span(&self.ast).end;
                self.cover.scratch.append(element);
                if self.eat(TokenTag::Comma)? {
                    has_trailing_comma = self.current_token.tag() == TokenTag::RightParen;
                } else if self.current_token.tag() != TokenTag::RightParen {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected ',' or ')' in parenthesized expression",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Add a comma between elements or close with ')'.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
            }
            if self.current_token.tag() != TokenTag::RightParen {
                self.report(
                    Span::new(start, end),
                    "Unterminated parenthesized expression",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Add a closing ')' to complete the expression.",
                        )),
                        labels: vec![self.label(Span::new(start, start + 1), "Opened here")],
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
            end = self.current_token.end();
            self.advance()?;
            let elements = self.add_extra_from_scratch_cover(checkpoint);
            Some(ParenthesizedCover {
                elements,
                start,
                end,
                has_trailing_comma,
            })
        })();
        self.ts_context = ts_context;
        self.context = grammar_context;
        self.cover.scratch.reset(checkpoint);
        result
    }

    fn cover_to_parenthesized_expression(&mut self, cover: ParenthesizedCover) -> Option<Expr> {
        if cover.elements.is_empty() {
            self.report(
                Span::new(cover.start, cover.end),
                "Empty parentheses are only valid as arrow function parameters",
                ReportOptions::default(),
            );
            return None;
        }
        if cover.has_trailing_comma {
            self.report(
                Span::new(cover.start, cover.end),
                "Trailing comma is not allowed in parenthesized expression",
                ReportOptions::default(),
            );
        }
        let checkpoint = self.cover.scratch.begin();
        let expression_result = (|| {
            for id in cover.elements.iter() {
                let argument = self.ast.get_node_in_sub_range(id);
                match self.ast.argument_data(argument) {
                    ArgumentData::Expr(expression) => {
                        self.cover.scratch.append(expression);
                    }
                    ArgumentData::SpreadElement(spread) => {
                        self.report(
                            spread.span(&self.ast),
                            "Rest element is not allowed in parenthesized expression",
                            ReportOptions::default(),
                        );
                        return None;
                    }
                }
            }
            Some(if self.cover.scratch.items.len() - checkpoint == 1 {
                unsafe { Expr::from_node_id_unchecked(self.cover.scratch.items[checkpoint]) }
            } else {
                let first =
                    unsafe { Expr::from_node_id_unchecked(self.cover.scratch.items[checkpoint]) };
                let last = unsafe {
                    Expr::from_node_id_unchecked(
                        *self
                            .cover
                            .scratch
                            .items
                            .last()
                            .expect("a sequence expression contains its last expression"),
                    )
                };
                let expressions = self.add_extra_from_scratch_cover(checkpoint);
                self.ast.expr_sequence_expression(
                    Span::new(first.span(&self.ast).start, last.span(&self.ast).end),
                    expressions,
                )
            })
        })();
        self.cover.scratch.reset(checkpoint);
        let expression = expression_result?;
        Some(
            self.ast
                .expr_parenthesized_expression(Span::new(cover.start, cover.end), expression),
        )
    }

    fn parse_parenthesized_expression(&mut self) -> Option<Expr> {
        self.with_resolved_cover(|parser| {
            let cover = parser.parse_parenthesized_cover()?;
            parser.cover_to_parenthesized_expression(cover)
        })
    }

    fn cover_to_arrow_function(
        &mut self,
        cover: ParenthesizedCover,
        is_async: bool,
        arrow_start: u32,
    ) -> Option<Expr> {
        let parameters = self.convert_to_formal_parameters(cover)?;
        self.build_arrow_function(arrow_start, parameters, is_async, None, None)
    }

    fn convert_to_formal_parameters(
        &mut self,
        cover: ParenthesizedCover,
    ) -> Option<FormalParameters> {
        let checkpoint = self.cover.scratch.begin();
        let result = (|| {
            let mut rest = None;
            for (index, id) in cover.elements.iter().enumerate() {
                let argument = self.ast.get_node_in_sub_range(id);
                match self.ast.argument_data(argument) {
                    ArgumentData::Expr(expression) => {
                        if rest.is_some() {
                            self.report(
                                expression.span(&self.ast),
                                "Rest parameter must be last formal parameter",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Move the rest parameter to the end of the parameter list",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                            return None;
                        }
                        let parameter = self.convert_to_formal_parameter(expression)?;
                        self.cover.scratch.append(parameter);
                    }
                    ArgumentData::SpreadElement(spread) => {
                        if index + 1 != cover.elements.len() {
                            self.report(
                                spread.span(&self.ast),
                                "Rest parameter must be last formal parameter",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Move the rest parameter to the end of the parameter list",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                            return None;
                        }
                        if cover.has_trailing_comma {
                            self.report(
                                spread.span(&self.ast),
                                "Rest parameter must be last formal parameter",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the trailing comma after the rest parameter",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        let argument =
                            self.expression_to_binding_pattern(spread.argument(&self.ast))?;
                        if self.ast.node_kind(argument.node_id()) == NodeKind::AssignmentPattern {
                            self.report(
                                argument.span(&self.ast),
                                "A rest element cannot have an initializer",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the '= ...' from the rest element.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        let argument = self.formal_parameter_pattern(argument)?;
                        let temporary = self.ast.formal_parameter_rest(
                            spread.span(&self.ast),
                            SubRange::empty(),
                            argument,
                            None,
                        );
                        rest = Some(temporary);
                    }
                }
            }
            let items = self.add_extra_from_scratch_cover(checkpoint);
            Some(self.ast.formal_parameters(
                Span::new(cover.start, cover.end),
                items,
                rest,
                FormalParameterKind::ArrowFormalParameters,
            ))
        })();
        self.cover.scratch.reset(checkpoint);
        result
    }

    fn convert_to_formal_parameter(&mut self, expression: Expr) -> Option<FormalParameterItem> {
        self.expression_to_parameter(expression)
    }

    fn parse_async_function_expression(&mut self, async_token: Token) -> Option<Expr> {
        self.report_if_escaped_keyword(async_token);
        let function = self.parse_function(
            ParseFunctionOpts {
                is_async: true,
                is_expression: true,
                ..ParseFunctionOpts::default()
            },
            Some(async_token.start()),
        )?;
        Some(Expr::Function(function.into_function_expression()))
    }

    fn parse_async_function_or_arrow(&mut self, minimum: Precedence) -> Option<Expr> {
        let token = self.current_token;
        let async_identifier = self.parse_identifier()?;

        if self.current_token.has_line_terminator_before() {
            return Some(async_identifier);
        }

        match self.current_token.tag() {
            TokenTag::Function => self.parse_async_function_expression(token),
            TokenTag::LeftParen => self.parse_async_arrow_function_or_call(
                token.span(),
                async_identifier,
                minimum,
                token.is_escaped(),
            ),
            TokenTag::LessThan
                if self.lang.is_typescript() && minimum <= Precedence::Assignment =>
            {
                if let Some(arrow) = self.try_parse_generic_arrow(true, token.start())? {
                    if token.is_escaped() {
                        self.report_escaped_keyword(token.span());
                    }
                    return Some(arrow);
                }
                Some(async_identifier)
            }
            tag if tag.is_identifier_like() && minimum <= Precedence::Assignment => {
                let after_identifier = self.peek_ahead()?;
                if after_identifier.tag() == TokenTag::Arrow {
                    if token.is_escaped() {
                        self.report_escaped_keyword(token.span());
                    }
                    let context = enter_grammar_context![&mut self.context, ?Yield, +Await];
                    let parameter = self.parse_identifier();
                    self.context = context;
                    let parameter = parameter?;
                    return self.identifier_to_arrow_function(parameter, true, token.start());
                }
                Some(async_identifier)
            }
            _ => Some(async_identifier),
        }
    }

    fn parse_async_arrow_function_or_call(
        &mut self,
        async_span: Span,
        async_identifier: Expr,
        minimum: Precedence,
        escaped_async: bool,
    ) -> Option<Expr> {
        if self.lang.is_typescript() && minimum <= Precedence::Assignment {
            let arrow = match self.classify_arrow_head() {
                ArrowHead::Yes => Some(self.parse_arrow(true, async_span.start)?),
                ArrowHead::Maybe => self.try_parse_arrow(true, async_span.start)?,
                ArrowHead::No => None,
            };
            if let Some(arrow) = arrow {
                if escaped_async {
                    self.report_escaped_keyword(async_span);
                }
                return Some(arrow);
            }
        }

        self.with_resolved_cover(|parser| {
            let checkpoint = parser.lang.is_typescript().then(|| parser.checkpoint());
            let outer_await = parser.await_identifier_in_async_head.take();
            let boundary_context = parser.boundary_context;
            let mut head_context = boundary_context;
            if !parser.lang.is_typescript() {
                head_context.insert(BoundaryContext::ASYNC_ARROW_HEAD);
            }
            let cover =
                parser.with_scope(head_context, |parser| parser.parse_parenthesized_cover());
            let await_identifier =
                mem::replace(&mut parser.await_identifier_in_async_head, outer_await);
            let cover = cover?;

            let is_arrow =
                parser.current_token.tag() == TokenTag::Arrow && minimum <= Precedence::Assignment;
            if is_arrow {
                if let Some(checkpoint) = checkpoint {
                    parser.rewind(checkpoint);
                    if escaped_async {
                        parser.report_escaped_keyword(async_span);
                    }
                    return parser.parse_arrow(true, async_span.start);
                }
                if escaped_async {
                    parser.report_escaped_keyword(async_span);
                }
                if let Some(span) = await_identifier {
                    parser.report_reserved_await(span, "an identifier");
                }
                // Reusing the cover parses nested defaults once per arrow level.
                return parser.cover_to_arrow_function(cover, true, async_span.start);
            }
            if boundary_context.contains(BoundaryContext::ASYNC_ARROW_HEAD) && outer_await.is_none()
            {
                parser.await_identifier_in_async_head = await_identifier;
            }
            parser.cover_to_call_expression(cover, async_identifier)
        })
    }

    fn identifier_to_arrow_function(
        &mut self,
        identifier: Expr,
        async_function: bool,
        arrow_start: u32,
    ) -> Option<Expr> {
        let parameter = self.expression_to_parameter(identifier)?;
        let parameter_span = parameter.span(&self.ast);
        let parameters = self.ast.add_sub_range([parameter]);
        let parameters = self.ast.formal_parameters(
            parameter_span,
            parameters,
            None,
            FormalParameterKind::ArrowFormalParameters,
        );
        self.build_arrow_function(arrow_start, parameters, async_function, None, None)
    }

    fn parse_simple_arrow_function(&mut self, left: Expr) -> Option<Expr> {
        if self.ast.node_kind(left.node_id()) != NodeKind::IdentifierReference {
            self.report(
                left.span(&self.ast),
                "Unexpected '=>'",
                ReportOptions::default(),
            );
            return None;
        }
        let start = left.span(&self.ast).start;
        self.identifier_to_arrow_function(left, false, start)
    }

    fn classify_arrow_head(&mut self) -> ArrowHead {
        if self.current_token.tag() != TokenTag::LeftParen {
            return ArrowHead::No;
        }
        self.classify_paren_arrow_head()
    }

    fn classify_paren_arrow_head(&mut self) -> ArrowHead {
        let mut peek = self.begin_peek();
        let result = (|| {
            let Some(second) = peek.next() else {
                return ArrowHead::No;
            };
            match second.tag() {
                TokenTag::Spread => ArrowHead::Yes,
                TokenTag::LeftBracket | TokenTag::LeftBrace => ArrowHead::Maybe,
                TokenTag::RightParen => {
                    let Some(third) = peek.next() else {
                        return ArrowHead::No;
                    };
                    if matches!(third.tag(), TokenTag::Arrow | TokenTag::Colon) {
                        ArrowHead::Yes
                    } else {
                        ArrowHead::No
                    }
                }
                TokenTag::This => {
                    let Some(third) = peek.next() else {
                        return ArrowHead::No;
                    };
                    if third.tag() == TokenTag::Colon {
                        ArrowHead::Yes
                    } else {
                        ArrowHead::No
                    }
                }
                _ if second.tag().is_identifier_like() => {
                    let Some(third) = peek.next() else {
                        return ArrowHead::No;
                    };
                    match third.tag() {
                        TokenTag::Colon => ArrowHead::Yes,
                        TokenTag::Question => {
                            let Some(fourth) = peek.next() else {
                                return ArrowHead::No;
                            };
                            if matches!(
                                fourth.tag(),
                                TokenTag::Colon
                                    | TokenTag::Comma
                                    | TokenTag::Assign
                                    | TokenTag::RightParen
                            ) {
                                ArrowHead::Yes
                            } else {
                                ArrowHead::No
                            }
                        }
                        TokenTag::Comma | TokenTag::Assign | TokenTag::RightParen => {
                            ArrowHead::Maybe
                        }
                        _ => ArrowHead::No,
                    }
                }
                _ => ArrowHead::No,
            }
        })();
        peek.end();
        result
    }

    fn try_parse_arrow(&mut self, is_async: bool, start: u32) -> Option<Option<Expr>> {
        let key = TsSpeculationKey::Arrow {
            context: self.speculation_key(),
            is_async,
        };
        if self.ts_rejected_speculations.contains(&key) {
            return Some(None);
        }

        let checkpoint = self.checkpoint();
        let arrow = (|| {
            let head = self.parse_arrow_head(is_async, false)?;
            let ambiguous_return_type =
                !self.ts_context.allow_arrow_return_type() && head.return_type.is_some();
            // Only the head resolves the ambiguity with a conditional consequent.
            // Body diagnostics still belong to the selected arrow's recovery.
            if ambiguous_return_type && !self.speculation_is_clean(&checkpoint) {
                return None;
            }
            let arrow = self.build_arrow_function(
                start,
                head.parameters,
                is_async,
                head.type_parameters,
                head.return_type,
            )?;
            if ambiguous_return_type && self.current_token.tag() != TokenTag::Colon {
                return None;
            }
            Some(arrow)
        })();
        if arrow.is_none() {
            self.rewind(checkpoint);
            self.reject_ts_speculation(key);
        }
        Some(arrow)
    }

    fn try_parse_generic_arrow(&mut self, is_async: bool, start: u32) -> Option<Option<Expr>> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LessThan);
        let next = self.peek_ahead()?;
        if !next.tag().is_identifier_like() && next.tag() != TokenTag::Const {
            return Some(None);
        }
        self.try_parse_arrow(is_async, start)
    }

    fn parse_arrow(&mut self, is_async: bool, start: u32) -> Option<Expr> {
        let head = self.parse_arrow_head(is_async, true)?;
        self.build_arrow_function(
            start,
            head.parameters,
            is_async,
            head.type_parameters,
            head.return_type,
        )
    }

    fn parse_arrow_head(
        &mut self,
        is_async: bool,
        need_diagnostics: bool,
    ) -> Option<ArrowHeadResult> {
        self.with_scope(self.boundary_context.arrow_head(is_async), |parser| {
            parser.parse_arrow_head_inner(is_async, need_diagnostics)
        })
    }

    fn parse_arrow_head_inner(
        &mut self,
        is_async: bool,
        need_diagnostics: bool,
    ) -> Option<ArrowHeadResult> {
        let type_parameters = if is_angle_open(self.current_token.tag()) {
            let close = if need_diagnostics {
                AngleClose::Peel
            } else {
                AngleClose::SpeculativePeel
            };
            Some(self.parse_type_parameters(TypeParameterContext::Declaration, close)?)
        } else {
            None
        };
        if is_async {
            // Preserve outer [Yield] so the async-arrow cover retains yield
            // expressions for the semantic Contains check.
            self.context.insert(Context::AWAIT);
        }
        if self.current_token.tag() != TokenTag::LeftParen {
            return None;
        }
        let (_, params) =
            self.parse_formal_parameters(FormalParameterKind::ArrowFormalParameters, false)?;

        let return_type =
            if self.lang.is_typescript() && self.current_token.tag() == TokenTag::Colon {
                Some(self.parse_return_type_annotation()?)
            } else {
                None
            };
        if self.current_token.tag() != TokenTag::Arrow {
            if need_diagnostics {
                // The speculative `try_parse_arrow` path skips diagnostic formatting for performance.
                self.report_expected(
                    self.current_token.span(),
                    "Expected '=>' in arrow function",
                    ReportOptions::default(),
                );
            }
            return None;
        }
        Some(ArrowHeadResult {
            parameters: params,
            type_parameters,
            return_type,
        })
    }

    fn build_arrow_function(
        &mut self,
        start: u32,
        parameters: FormalParameters,
        async_function: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Option<Expr> {
        // Keep the arrow for recovery, including when its head was parsed speculatively.
        if self.current_token.has_line_terminator_before() {
            self.report(
                self.current_token.span(),
                "A line terminator is not allowed before '=>'",
                ReportOptions::default(),
            );
        }
        self.advance()?;
        let body = self.with_scope(self.boundary_context.arrow_body(), |parser| {
            enter_function_grammar_context(&mut parser.context, false, async_function);
            parser.parse_arrow_body()
        });
        let ArrowBodyResult {
            body,
            is_expression,
        } = body?;
        let end = body.span(&self.ast).end;
        Some(self.ast.expr_arrow_function_expression(
            Span::new(start, end),
            is_expression,
            async_function,
            type_parameters,
            parameters,
            return_type,
            body,
        ))
    }

    // ConciseBody[In] :
    //   [lookahead != `{`] ExpressionBody[?In, ~Await]
    //   `{` FunctionBody[~Yield, ~Await] `}`
    //
    // AsyncConciseBody[In] :
    //   [lookahead != `{`] ExpressionBody[?In, +Await]
    //   `{` AsyncFunctionBody `}`
    //
    // ExpressionBody[In, Await] :
    //   AssignmentExpression[?In, ~Yield, ?Await]
    fn parse_arrow_body(&mut self) -> Option<ArrowBodyResult> {
        if self.current_token.tag() == TokenTag::LeftBrace {
            let body = self.parse_function_body()?;
            return Some(ArrowBodyResult {
                body: ArrowFunctionBody::FunctionBody(body),
                is_expression: false,
            });
        }
        let expression = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        Some(ArrowBodyResult {
            body: ArrowFunctionBody::Expr(expression),
            is_expression: true,
        })
    }

    fn normalize_binding_expression(&mut self, expression: Expr) -> Expr {
        if !matches!(
            self.ast.expr_data(expression),
            ExprData::ParenthesizedExpression(_)
        ) {
            return expression;
        }
        let inner = self.unwrap_parens(expression);
        self.report_parenthesized_binding(inner.span(&self.ast));
        inner
    }

    fn report_parenthesized_binding(&mut self, span: Span) {
        self.report(
            span,
            "Parentheses are not allowed in this binding pattern",
            ReportOptions {
                help: Some(Cow::Borrowed("Remove the extra parentheses. Binding patterns can only be identifiers, destructuring patterns, or assignment patterns, not parenthesized expressions.")),
                ..ReportOptions::default()
            },
        );
    }

    fn expression_to_parameter_parts(
        &mut self,
        expression: Expr,
    ) -> Option<(FormalParameterPattern, Option<Expr>)> {
        let expression = self.normalize_binding_expression(expression);
        let (binding, initializer) = if let ExprData::AssignmentExpression(assignment) =
            self.ast.expr_data(expression)
            && assignment.operator(&self.ast) == AssignmentOperator::Assign
        {
            let binding = self.assignment_target_to_pattern(
                assignment.left(&self.ast),
                PatternContext::Binding,
            )?;
            (binding, Some(assignment.right(&self.ast)))
        } else {
            (self.expression_to_binding_pattern(expression)?, None)
        };
        Some((self.formal_parameter_pattern(binding)?, initializer))
    }

    fn expression_to_parameter(&mut self, expression: Expr) -> Option<FormalParameterItem> {
        let span = expression.span(&self.ast);
        let (pattern, initializer) = self.expression_to_parameter_parts(expression)?;
        Some(FormalParameterItem::FormalParameter(
            self.ast
                .formal_parameter(span, SubRange::empty(), pattern, None, initializer, false),
        ))
    }

    fn expression_to_binding_pattern(&mut self, expression: Expr) -> Option<BindingPattern> {
        self.cover_to_pattern(expression, PatternContext::Binding)
    }

    fn expression_to_assignment_pattern(&mut self, expression: Expr) -> Option<BindingPattern> {
        self.cover_to_pattern(expression, PatternContext::Assignment)
    }

    fn expression_to_pattern(
        &mut self,
        expression: Expr,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        let expression = if context == PatternContext::Binding {
            self.normalize_binding_expression(expression)
        } else {
            expression
        };
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(identifier) if context == PatternContext::Assignment => {
                Some(BindingPattern::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::IdentifierReference(identifier),
                ))
            }
            ExprData::IdentifierReference(identifier) => {
                let binding = self
                    .ast
                    .binding_identifier(identifier.span(&self.ast), identifier.name(&self.ast));

                Some(BindingPattern::BindingIdentifier(binding))
            }
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(&self.ast) != AssignmentOperator::Assign =>
            {
                self.report(
                    expression.span(&self.ast),
                    if context == PatternContext::Binding {
                        "Invalid assignment operator in binding pattern"
                    } else {
                        "Invalid assignment operator in assignment pattern"
                    },
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Only '=' is allowed in destructuring defaults, not compound operators like '+='.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
            ExprData::MemberExpression(member) => {
                if member_expression_is_optional(member, &self.ast) {
                    self.report(
                        expression.span(&self.ast),
                        if context == PatternContext::Binding {
                            "Optional chaining is not allowed in binding pattern"
                        } else {
                            "Optional chaining is not allowed in assignment pattern"
                        },
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Optional chaining ('?.') cannot be used as an assignment target in destructuring patterns.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
                if context != PatternContext::Assignment {
                    self.report(
                        expression.span(&self.ast),
                        "Member expression is not allowed in binding pattern",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Function parameters and variable declarations can only bind to identifiers, not member expressions like 'obj.prop' or obj[key]. Use a simple identifier instead.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                }
                Some(BindingPattern::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::MemberExpression(member),
                ))
            }
            ExprData::TsAsExpression(_)
            | ExprData::TsSatisfiesExpression(_)
            | ExprData::TsTypeAssertion(_)
            | ExprData::TsNonNullExpression(_) => {
                if context != PatternContext::Assignment {
                    self.report(
                        expression.span(&self.ast),
                        "TypeScript assertion is not allowed in binding pattern",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Non-null ('!') and type ('as', 'satisfies', '<T>') assertions can only appear in assignment targets.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                }
                if !self.is_simple_assignment_target(expression) {
                    self.report(
                        expression.span(&self.ast),
                        "Invalid assignment target",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "The expression behind a non-null or type assertion must itself be a simple assignment target, like an identifier or member access, not a call or other expression.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
                Some(BindingPattern::SimpleAssignmentTarget(
                    self.expression_to_simple_assignment_target(expression)?,
                ))
            }
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(&self.ast) == AssignmentOperator::Assign =>
            {
                let left =
                    self.assignment_target_to_pattern(assignment.left(&self.ast), context)?;
                let right = assignment.right(&self.ast);
                let pattern = self
                    .ast
                    .assignment_pattern(assignment.span(&self.ast), left, right);

                Some(BindingPattern::AssignmentPattern(pattern))
            }
            ExprData::ArrayExpression(array) => self.convert_array_pattern(array, context),
            ExprData::ObjectExpression(object) => self.convert_object_pattern(object, context),
            ExprData::ParenthesizedExpression(parenthesized) => {
                let inner = parenthesized.expression(&self.ast);
                if !self.is_simple_assignment_target(inner) {
                    self.report(
                        inner.span(&self.ast),
                        "Parenthesized expression in assignment pattern must be a simple assignment target",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Only identifiers or member expressions (without optional chaining) are allowed inside parentheses in assignment patterns.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
                let pattern = self.expression_to_pattern(inner, context)?;
                self.cover.record_parenthesized_target(pattern.node_id());
                Some(pattern)
            }
            ExprData::ChainExpression(_) => {
                self.report(
                    expression.span(&self.ast),
                    if context == PatternContext::Binding {
                        "Optional chaining is not allowed in binding pattern"
                    } else {
                        "Optional chaining is not allowed in assignment pattern"
                    },
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Optional chaining ('?.') cannot be used as an assignment target in destructuring patterns.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
            _ => {
                self.report(
                    expression.span(&self.ast),
                    if context == PatternContext::Binding {
                        "Invalid element in binding pattern"
                    } else {
                        "Invalid element in assignment pattern"
                    },
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Expected an identifier, array pattern, object pattern, or assignment pattern.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
        }
    }

    fn cover_to_pattern(
        &mut self,
        expression: Expr,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        match self.ast.expr_data(expression) {
            ExprData::ArrayExpression(array) => self.convert_array_pattern(array, context),
            ExprData::ObjectExpression(object) => self.convert_object_pattern(object, context),
            _ => self.expression_to_pattern(expression, context),
        }
    }

    fn convert_array_pattern(
        &mut self,
        array: ArrayExpression,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        self.convert_array_pattern_impl(array.elements(&self.ast), array.span(&self.ast), context)
            .map(BindingPattern::ArrayPattern)
    }

    fn array_cover_to_pattern(
        &mut self,
        cover: ArrayCover,
        context: PatternContext,
    ) -> Option<ArrayPattern> {
        self.convert_array_pattern_impl(cover.elements, Span::new(cover.start, cover.end), context)
    }

    fn convert_array_pattern_impl(
        &mut self,
        elements_range: SubRange<Option<Argument>>,
        span: Span,
        context: PatternContext,
    ) -> Option<ArrayPattern> {
        let checkpoint = self.cover.scratch.begin();
        let result = (|| {
            let source_elements_len = elements_range.len();
            let none = NodeId::from_raw_unchecked(u32::MAX);
            let mut rest = None;
            for index in 0..source_elements_len {
                let argument = elements_range.get_node(&self.ast, index).unwrap();
                let Some(argument) = argument else {
                    self.cover.scratch.items.push(none);
                    continue;
                };
                match self.ast.argument_data(argument) {
                    ArgumentData::Expr(expression) => {
                        let pattern = self.expression_to_pattern(expression, context)?;
                        self.cover.scratch.append(pattern);
                    }
                    ArgumentData::SpreadElement(spread) => {
                        if let Some(comma) = self.cover.consume_trailing_comma(spread.node_id()) {
                            self.report(
                                comma,
                                "Rest element cannot have a trailing comma in array destructuring.",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the trailing comma after the rest element",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        if index + 1 != source_elements_len {
                            self.report(
                                spread.span(&self.ast),
                                "Rest element must be the last element",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "No elements can follow the rest element in a destructuring pattern.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        let argument =
                            self.expression_to_pattern(spread.argument(&self.ast), context)?;
                        if self.ast.node_kind(argument.node_id()) == NodeKind::AssignmentPattern {
                            self.report(
                                spread.span(&self.ast),
                                "A rest element cannot have an initializer",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the '= ...' from the rest element.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        let rest_element = self
                            .ast
                            .binding_rest_element(spread.span(&self.ast), argument);

                        if index + 1 == source_elements_len {
                            rest = Some(rest_element);
                        } else {
                            self.cover.scratch.append(rest_element);
                        }
                    }
                }
            }
            let elements =
                self.ast
                    .add_sub_range(self.cover.scratch.items[checkpoint..].iter().map(|node| {
                        (*node != none)
                            .then(|| unsafe { BindingPattern::from_node_id_unchecked(*node) })
                    }));
            Some(self.ast.array_pattern(span, elements, rest))
        })();
        self.cover.scratch.reset(checkpoint);
        result
    }

    fn convert_object_pattern(
        &mut self,
        object: ObjectExpression,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        self.convert_object_pattern_impl(
            object.properties(&self.ast),
            object.span(&self.ast),
            context,
        )
        .map(BindingPattern::ObjectPattern)
    }

    fn object_cover_to_pattern(
        &mut self,
        cover: ObjectCover,
        context: PatternContext,
    ) -> Option<ObjectPattern> {
        self.convert_object_pattern_impl(
            cover.properties,
            Span::new(cover.start, cover.end),
            context,
        )
    }

    fn convert_object_pattern_impl(
        &mut self,
        properties_range: SubRange<ObjectPropertyKind>,
        span: Span,
        context: PatternContext,
    ) -> Option<ObjectPattern> {
        let checkpoint = self.cover.scratch.begin();
        let result = (|| {
            let source_properties_len = properties_range.len();
            let mut rest = None;
            for index in 0..source_properties_len {
                let property = properties_range.get_node(&self.ast, index).unwrap();
                match self.ast.object_property_kind_data(property) {
                    ObjectPropertyKindData::ObjectProperty(property) => {
                        if property.method(&self.ast) {
                            self.report(
                                property.span(&self.ast),
                                "Method cannot appear in destructuring pattern",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Use a regular property instead of a method definition.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                            continue;
                        }
                        if property.kind(&self.ast) != PropertyKind::Init {
                            self.report(
                                property.span(&self.ast),
                                "Getter/setter cannot appear in destructuring pattern",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Use a regular property instead of a getter or setter.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                            continue;
                        }
                        let value =
                            self.expression_to_pattern(property.value(&self.ast), context)?;
                        self.cover.consume_initialized_name(property.node_id());
                        let property = self.ast.binding_property(
                            property.span(&self.ast),
                            property.key(&self.ast),
                            value,
                            property.shorthand(&self.ast),
                            property.computed(&self.ast),
                        );

                        self.cover.scratch.append(property);
                    }
                    ObjectPropertyKindData::SpreadElement(spread) => {
                        if let Some(comma) = self.cover.consume_trailing_comma(spread.node_id()) {
                            self.report(
                                comma,
                                "Rest element cannot have a trailing comma in object destructuring.",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the trailing comma after the rest element",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        if index + 1 != source_properties_len {
                            self.report(
                                spread.span(&self.ast),
                                "Rest element must be the last property",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "No properties can follow the rest element in a destructuring pattern.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        let argument =
                            self.expression_to_pattern(spread.argument(&self.ast), context)?;
                        if matches!(
                            self.ast.node_kind(argument.node_id()),
                            NodeKind::ArrayPattern | NodeKind::ObjectPattern
                        ) {
                            self.report(
                                argument.span(&self.ast),
                                "Object rest argument cannot be a destructuring pattern",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Collect the remaining properties into an identifier and destructure it separately.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        if self.ast.node_kind(argument.node_id()) == NodeKind::AssignmentPattern {
                            self.report(
                                spread.span(&self.ast),
                                "A rest element cannot have an initializer",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the '= ...' from the rest element.",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                        }
                        let temporary = self
                            .ast
                            .binding_rest_element(spread.span(&self.ast), argument);
                        rest = Some(temporary);
                        break;
                    }
                }
            }
            let properties = self.add_extra_from_scratch_cover(checkpoint);
            Some(self.ast.object_pattern(span, properties, rest))
        })();
        self.cover.scratch.reset(checkpoint);
        result
    }

    fn assignment_target_to_pattern(
        &mut self,
        target: AssignmentTarget,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        match self.ast.assignment_target_data(target) {
            AssignmentTargetData::SimpleAssignmentTarget(target) => {
                if context == PatternContext::Assignment {
                    return Some(BindingPattern::SimpleAssignmentTarget(target));
                }
                self.report_parenthesized_target_in_binding(target.node_id());
                match self.ast.simple_assignment_target_data(target) {
                    SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                        let binding = self.ast.binding_identifier(
                            identifier.span(&self.ast),
                            identifier.name(&self.ast),
                        );

                        Some(BindingPattern::BindingIdentifier(binding))
                    }
                    SimpleAssignmentTargetData::MemberExpression(_) => {
                        self.report(
                            target.span(&self.ast),
                            "Member expression is not allowed in binding pattern",
                            ReportOptions {
                                help: Some(Cow::Borrowed(
                                    "Function parameters and variable declarations can only bind to identifiers, not member expressions like 'obj.prop' or obj[key]. Use a simple identifier instead.",
                                )),
                                ..ReportOptions::default()
                            },
                        );
                        Some(BindingPattern::SimpleAssignmentTarget(target))
                    }
                    SimpleAssignmentTargetData::TsAsExpression(_)
                    | SimpleAssignmentTargetData::TsSatisfiesExpression(_)
                    | SimpleAssignmentTargetData::TsTypeAssertion(_)
                    | SimpleAssignmentTargetData::TsNonNullExpression(_) => {
                        self.report(
                            target.span(&self.ast),
                            "TypeScript assertion is not allowed in binding pattern",
                            ReportOptions {
                                help: Some(Cow::Borrowed(
                                    "Non-null ('!') and type ('as', 'satisfies', '<T>') assertions can only appear in assignment targets.",
                                )),
                                ..ReportOptions::default()
                            },
                        );
                        Some(BindingPattern::SimpleAssignmentTarget(target))
                    }
                }
            }
            AssignmentTargetData::ArrayAssignmentTarget(pattern) => {
                let pattern = BindingPattern::ArrayPattern(pattern);
                if context == PatternContext::Binding {
                    self.assignment_pattern_to_binding_pattern(pattern)
                } else {
                    Some(pattern)
                }
            }
            AssignmentTargetData::ObjectAssignmentTarget(pattern) => {
                let pattern = BindingPattern::ObjectPattern(pattern);
                if context == PatternContext::Binding {
                    self.assignment_pattern_to_binding_pattern(pattern)
                } else {
                    Some(pattern)
                }
            }
        }
    }

    /// Rebuild an assignment target as a binding after an arrow head chooses
    /// its role. Every changed child is carried into a newly constructed parent.
    fn assignment_pattern_to_binding_pattern(
        &mut self,
        pattern: BindingPattern,
    ) -> Option<BindingPattern> {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(_) => Some(pattern),
            BindingPatternData::SimpleAssignmentTarget(target) => self
                .assignment_target_to_pattern(
                    AssignmentTarget::SimpleAssignmentTarget(target),
                    PatternContext::Binding,
                ),
            BindingPatternData::AssignmentPattern(assignment) => {
                let left =
                    self.assignment_pattern_to_binding_pattern(assignment.left(&self.ast))?;
                Some(BindingPattern::AssignmentPattern(
                    self.ast.assignment_pattern(
                        assignment.span(&self.ast),
                        left,
                        assignment.right(&self.ast),
                    ),
                ))
            }
            BindingPatternData::BindingRestElement(rest) => {
                let argument =
                    self.assignment_pattern_to_binding_pattern(rest.argument(&self.ast))?;
                Some(BindingPattern::BindingRestElement(
                    self.ast
                        .binding_rest_element(rest.span(&self.ast), argument),
                ))
            }
            BindingPatternData::ArrayPattern(array) => {
                let checkpoint = self.cover.scratch.begin();
                let result = (|| {
                    let elements = array.elements(&self.ast);
                    let none = NodeId::from_raw_unchecked(u32::MAX);
                    for index in 0..elements.len() {
                        if let Some(element) = elements.get_node(&self.ast, index).unwrap() {
                            let element = self.assignment_pattern_to_binding_pattern(element)?;
                            self.cover.scratch.append(element);
                        } else {
                            self.cover.scratch.items.push(none);
                        }
                    }
                    let rest = if let Some(rest) = array.rest(&self.ast) {
                        let argument =
                            self.assignment_pattern_to_binding_pattern(rest.argument(&self.ast))?;
                        Some(
                            self.ast
                                .binding_rest_element(rest.span(&self.ast), argument),
                        )
                    } else {
                        None
                    };
                    let elements =
                        self.ast
                            .add_sub_range(self.cover.scratch.items[checkpoint..].iter().map(
                                |node| {
                                    (*node != none).then(|| unsafe {
                                        BindingPattern::from_node_id_unchecked(*node)
                                    })
                                },
                            ));
                    Some(BindingPattern::ArrayPattern(self.ast.array_pattern(
                        array.span(&self.ast),
                        elements,
                        rest,
                    )))
                })();
                self.cover.scratch.reset(checkpoint);
                result
            }
            BindingPatternData::ObjectPattern(object) => {
                let checkpoint = self.cover.scratch.begin();
                let result = (|| {
                    let properties = object.properties(&self.ast);
                    for index in 0..properties.len() {
                        let property = properties.get_node(&self.ast, index).unwrap();
                        let value =
                            self.assignment_pattern_to_binding_pattern(property.value(&self.ast))?;
                        let property = self.ast.binding_property(
                            property.span(&self.ast),
                            property.key(&self.ast),
                            value,
                            property.shorthand(&self.ast),
                            property.computed(&self.ast),
                        );
                        self.cover.scratch.append(property);
                    }
                    let rest = if let Some(rest) = object.rest(&self.ast) {
                        let argument =
                            self.assignment_pattern_to_binding_pattern(rest.argument(&self.ast))?;
                        Some(
                            self.ast
                                .binding_rest_element(rest.span(&self.ast), argument),
                        )
                    } else {
                        None
                    };
                    let properties = self.add_extra_from_scratch_cover(checkpoint);
                    Some(BindingPattern::ObjectPattern(self.ast.object_pattern(
                        object.span(&self.ast),
                        properties,
                        rest,
                    )))
                })();
                self.cover.scratch.reset(checkpoint);
                result
            }
        }
    }

    fn report_parenthesized_target_in_binding(&mut self, node: NodeId) {
        if self.cover.parenthesized_targets.contains(&node) {
            self.report_parenthesized_binding(self.ast.span(node));
        }
    }

    fn is_destructuring_pattern(&self, pattern: BindingPattern) -> bool {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::ArrayPattern(_) | BindingPatternData::ObjectPattern(_) => true,
            BindingPatternData::AssignmentPattern(pattern) => {
                self.is_destructuring_pattern(pattern.left(&self.ast))
            }
            _ => false,
        }
    }

    fn record_cover_trailing_comma(&mut self, spread: SpreadElement, comma: Span) {
        self.cover.record_trailing_comma(spread.node_id(), comma);
    }

    fn parse_array_expression(&mut self) -> Option<Expr> {
        let cover = self.parse_array_cover()?;
        self.array_cover_to_expression(cover)
    }

    fn parse_object_expression(&mut self) -> Option<Expr> {
        let cover = self.parse_object_cover()?;
        self.object_cover_to_expression(cover)
    }

    // ArrayLiteral[Yield, Await] :
    //   `[` Elision? `]`
    //   `[` ElementList[?Yield, ?Await] `]`
    //   `[` ElementList[?Yield, ?Await] `,` Elision? `]`
    //
    // ElementList[Yield, Await] :
    //   Elision? AssignmentExpression[+In, ?Yield, ?Await]
    //   Elision? SpreadElement[?Yield, ?Await]
    //   ElementList[?Yield, ?Await] `,` Elision? AssignmentExpression[+In, ?Yield, ?Await]
    //   ElementList[?Yield, ?Await] `,` Elision? SpreadElement[?Yield, ?Await]
    //
    // SpreadElement[Yield, Await] :
    //   `...` AssignmentExpression[+In, ?Yield, ?Await]
    //
    // BindingElement[Yield, Await] :
    //   SingleNameBinding[?Yield, ?Await]
    //   BindingPattern[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]?
    //
    // SingleNameBinding[Yield, Await] :
    //   BindingIdentifier[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]?
    fn parse_array_cover(&mut self) -> Option<ArrayCover> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBracket);
        let start = self.current_token.start();
        self.advance()?;
        let checkpoint = self.cover.scratch.begin();
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            let mut end = start + 1;
            while self.current_token.tag() != TokenTag::RightBracket
                && self.current_token.tag() != TokenTag::Eof
            {
                if self.eat(TokenTag::Comma)? {
                    self.cover
                        .scratch
                        .items
                        .push(NodeId::from_raw_unchecked(u32::MAX));
                    continue;
                }
                let argument = if self.eat(TokenTag::Spread)? {
                    let spread_start = self.lexer.state.prev_token_end - 3;
                    let expression = self.parse_assignment_expression_in_cover()?;
                    let span = Span::new(spread_start, expression.span(&self.ast).end);
                    Argument::SpreadElement(self.ast.spread_element(span, expression))
                } else {
                    Argument::Expr(self.parse_assignment_expression_in_cover()?)
                };
                end = argument.span(&self.ast).end;
                self.cover.scratch.append(argument);
                let comma = self.current_token.span();
                if self.eat(TokenTag::Comma)? {
                    if self.current_token.tag() == TokenTag::RightBracket
                        && let Some(spread) = argument.as_spread_element(&self.ast)
                    {
                        self.record_cover_trailing_comma(spread, comma);
                    }
                } else if self.current_token.tag() != TokenTag::RightBracket {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected ',' or ']' in array",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Add a comma between elements or close the array with ']'.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
            }
            if self.current_token.tag() != TokenTag::RightBracket {
                self.report(
                    Span::new(start, end),
                    "Unterminated array",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Add a closing ']' to complete the array.")),
                        labels: vec![self.label(Span::new(start, start + 1), "Opened here")],
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
            end = self.current_token.end();
            self.advance()?;
            let none = NodeId::from_raw_unchecked(u32::MAX);
            let elements =
                self.ast
                    .add_sub_range(self.cover.scratch.items[checkpoint..].iter().map(|node| {
                        (*node != none).then(|| unsafe { Argument::from_node_id_unchecked(*node) })
                    }));
            Some(ArrayCover {
                elements,
                start,
                end,
            })
        })();
        self.ts_context = ts_context;
        self.context = grammar_context;
        self.cover.scratch.reset(checkpoint);
        result
    }

    // ObjectLiteral[Yield, Await] :
    //   `{` `}`
    //   `{` PropertyDefinitionList[?Yield, ?Await] `,`? `}`
    //
    // PropertyDefinition[Yield, Await] :
    //   IdentifierReference[?Yield, ?Await]
    //   CoverInitializedName[?Yield, ?Await]
    //   PropertyName[?Yield, ?Await] `:` AssignmentExpression[+In, ?Yield, ?Await]
    //   MethodDefinition[?Yield, ?Await]
    //   `...` AssignmentExpression[+In, ?Yield, ?Await]
    //
    // CoverInitializedName[Yield, Await] :
    //   IdentifierReference[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]
    //
    // BindingElement[Yield, Await] :
    //   SingleNameBinding[?Yield, ?Await]
    //   BindingPattern[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]?
    //
    // SingleNameBinding[Yield, Await] :
    //   BindingIdentifier[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]?
    fn parse_object_cover(&mut self) -> Option<ObjectCover> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBrace);
        let start = self.current_token.start();
        self.advance()?;
        let checkpoint = self.cover.scratch.begin();
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            let mut end = start + 1;
            while self.current_token.tag() != TokenTag::RightBrace
                && self.current_token.tag() != TokenTag::Eof
            {
                let property = if self.eat(TokenTag::Spread)? {
                    let spread_start = self.lexer.state.prev_token_end - 3;
                    let expression = self.parse_expr(
                        Precedence::Assignment,
                        ParseExpressionOpts {
                            allow_type_binding_identifiers: true,
                        },
                    )?;
                    let spread = self.ast.spread_element(
                        Span::new(spread_start, expression.span(&self.ast).end),
                        expression,
                    );
                    ObjectPropertyKind::SpreadElement(spread)
                } else {
                    self.parse_cover_property()?
                };
                end = property.span(&self.ast).end;
                self.cover.scratch.append(property);
                let comma = self.current_token.span();
                if self.eat(TokenTag::Comma)? {
                    if self.current_token.tag() == TokenTag::RightBrace
                        && let Some(spread) = property.as_spread_element(&self.ast)
                    {
                        self.record_cover_trailing_comma(spread, comma);
                    }
                } else if self.current_token.tag() != TokenTag::RightBrace {
                    self.report_expected(
                        self.current_token.span(),
                        "Expected ',' or '}' in object",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Add a comma between properties or close the object with '}'.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
            }
            if self.current_token.tag() != TokenTag::RightBrace {
                self.report(
                    Span::new(start, end),
                    "Unterminated object",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Add a closing '}' to complete the object.")),
                        labels: vec![self.label(Span::new(start, start + 1), "Opened here")],
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
            end = self.current_token.end();
            self.advance()?;
            let properties = self.add_extra_from_scratch_cover(checkpoint);
            Some(ObjectCover {
                properties,
                start,
                end,
            })
        })();
        self.ts_context = ts_context;
        self.context = grammar_context;
        self.cover.scratch.reset(checkpoint);
        result
    }

    fn array_cover_to_expression(&mut self, cover: ArrayCover) -> Option<Expr> {
        Some(
            self.ast
                .expr_array_expression(Span::new(cover.start, cover.end), cover.elements),
        )
    }

    fn object_cover_to_expression(&mut self, cover: ObjectCover) -> Option<Expr> {
        Some(
            self.ast
                .expr_object_expression(Span::new(cover.start, cover.end), cover.properties),
        )
    }

    // PropertyDefinition[Yield, Await] :
    //   IdentifierReference[?Yield, ?Await]
    //   CoverInitializedName[?Yield, ?Await]
    //   PropertyName[?Yield, ?Await] `:` AssignmentExpression[+In, ?Yield, ?Await]
    //   MethodDefinition[?Yield, ?Await]
    //
    // CoverInitializedName[Yield, Await] :
    //   IdentifierReference[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]
    fn parse_cover_property(&mut self) -> Option<ObjectPropertyKind> {
        let start = self.current_token.start();
        let mut async_function = false;
        let mut generator = false;
        let mut kind = PropertyKind::Init;
        let mut key = None;
        let key_token = self.current_token;

        if self.current_token.tag() == TokenTag::Async {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            if !self.current_token.has_line_terminator_before()
                && Self::is_property_key_start(self.current_token.tag())
            {
                self.report_if_escaped_keyword(token);
                async_function = true;
            } else {
                let name = self.identifier_name(token);
                key = Some(self.ast.property_key_identifier_name(token.span(), name));
            }
        }

        if key.is_none() && self.eat(TokenTag::Star)? {
            generator = true;
        }

        if key.is_none()
            && !async_function
            && !generator
            && matches!(self.current_token.tag(), TokenTag::Get | TokenTag::Set)
        {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            if Self::is_property_key_start(self.current_token.tag()) {
                self.report_if_escaped_keyword(token);
                kind = if token.tag() == TokenTag::Get {
                    PropertyKind::Get
                } else {
                    PropertyKind::Set
                };
            } else {
                let name = self.identifier_name(token);
                key = Some(self.ast.property_key_identifier_name(token.span(), name));
            }
        }

        let computed = self.current_token.tag() == TokenTag::LeftBracket && key.is_none();
        let key = match key {
            Some(key) => key,
            None => self.parse_property_key(true)?,
        };

        if self.current_token.tag() == TokenTag::LeftParen
            || (self.lang.is_typescript() && is_angle_open(self.current_token.tag()))
        {
            return self.parse_object_method_property(
                start,
                key,
                computed,
                kind,
                async_function,
                generator,
            );
        }

        if async_function || generator || kind != PropertyKind::Init {
            self.report_expected(
                self.current_token.span(),
                "Expected '(' for method definition",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Method definitions require a parameter list. Use 'method() {}' syntax.",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }

        if self.current_token.tag() == TokenTag::Colon {
            self.advance()?;
            let value = self.parse_expr(
                Precedence::Assignment,
                ParseExpressionOpts {
                    allow_type_binding_identifiers: true,
                },
            )?;
            return Some(ObjectPropertyKind::ObjectProperty(
                self.ast.object_property(
                    Span::new(start, value.span(&self.ast).end),
                    key,
                    value,
                    PropertyKind::Init,
                    false,
                    false,
                    computed,
                ),
            ));
        }

        let reference = self.property_key_to_reference(key, key_token)?;
        let initialized_name = self.eat(TokenTag::Assign)?;
        let value = if initialized_name {
            let default_value = self.parse_value_expression(Precedence::Assignment)?;
            let target = self.expression_to_assignment_target(reference)?;
            self.ast.expr_assignment_expression(
                Span::new(
                    reference.span(&self.ast).start,
                    default_value.span(&self.ast).end,
                ),
                target,
                default_value,
                AssignmentOperator::Assign,
            )
        } else {
            reference
        };
        let property = self.ast.object_property(
            Span::new(start, value.span(&self.ast).end),
            key,
            value,
            PropertyKind::Init,
            false,
            true,
            false,
        );
        if initialized_name {
            self.cover
                .record_initialized_name(property.node_id(), property.span(&self.ast));
        }
        Some(ObjectPropertyKind::ObjectProperty(property))
    }

    /// Both shorthand forms change an IdentifierName into an IdentifierReference.
    /// Validate at that transition, before a default initializer can be attached.
    fn property_key_to_reference(&mut self, key: PropertyKey, token: Token) -> Option<Expr> {
        let PropertyKeyData::IdentifierName(identifier) = self.ast.property_key_data(key) else {
            self.report(
                key.span(&self.ast),
                "Shorthand property must be an identifier",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Property keys other than identifiers require an explicit ':' value.",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        };
        self.validate_identifier(token, IdentifierRole::Reference);
        Some(
            self.ast
                .expr_identifier_reference(identifier.span(&self.ast), identifier.name(&self.ast)),
        )
    }

    fn parse_object_method_property(
        &mut self,
        start: u32,
        key: PropertyKey,
        computed: bool,
        kind: PropertyKind,
        async_function: bool,
        generator: bool,
    ) -> Option<ObjectPropertyKind> {
        if generator {
            if kind == PropertyKind::Get {
                self.report(
                    Span::new(start, self.current_token.end()),
                    "Getter cannot be a generator",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Remove the '*' from the getter definition.")),
                        ..ReportOptions::default()
                    },
                );
                return None;
            } else if kind == PropertyKind::Set {
                self.report(
                    Span::new(start, self.current_token.end()),
                    "Setter cannot be a generator",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Remove the '*' from the setter definition.")),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
        }
        let function_start = self.current_token.start();
        let result = self.with_scope(BoundaryContext::local(), |parser| {
            let type_parameters =
                if parser.lang.is_typescript() && is_angle_open(parser.current_token.tag()) {
                    Some(parser.parse_type_parameters(
                        TypeParameterContext::Declaration,
                        AngleClose::Peel,
                    )?)
                } else {
                    None
                };
            enter_function_grammar_context(&mut parser.context, generator, async_function);
            let (this_param, params) = parser
                .parse_formal_parameters(FormalParameterKind::UniqueFormalParameters, false)?;
            parser.check_accessor_arity(kind, params);
            let return_type =
                if parser.lang.is_typescript() && parser.current_token.tag() == TokenTag::Colon {
                    Some(parser.parse_return_type_annotation()?)
                } else {
                    None
                };
            let body = parser.parse_function_body()?;
            Some((this_param, params, type_parameters, return_type, body))
        });
        let (this_param, params, type_parameters, return_type, body) = result?;
        let span = Span::new(start, body.span(&self.ast).end);
        let function = self.ast.function(
            Span::new(function_start, body.span(&self.ast).end),
            FunctionType::FunctionExpression,
            None,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            generator,
            async_function,
        );
        let value = Expr::Function(function);
        Some(ObjectPropertyKind::ObjectProperty(
            self.ast.object_property(
                span,
                key,
                value,
                kind,
                kind == PropertyKind::Init,
                false,
                computed,
            ),
        ))
    }

    fn is_property_key_start(tag: TokenTag) -> bool {
        matches!(
            tag,
            TokenTag::Star | TokenTag::LeftBracket | TokenTag::StringLiteral
        ) || tag.is_numeric_literal()
            || tag.is_identifier_like()
    }

    // ComputedPropertyName[Yield, Await] :
    //   `[` AssignmentExpression[+In, ?Yield, ?Await] `]`
    fn parse_class_element_key(&mut self) -> Option<PropertyKey> {
        let token = self.current_token;
        if token.tag() == TokenTag::LeftBracket {
            self.advance()?;
            let grammar_context = enter_grammar_context![&mut self.context, +In];
            let expression =
                self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default());
            self.context = grammar_context;
            let expression = expression?;
            if !self.expect(
                TokenTag::RightBracket,
                "Expected ']' after computed property key",
                None,
            ) {
                return None;
            }
            return Some(PropertyKey::Expr(expression));
        }

        let key = if token.tag() == TokenTag::PrivateIdentifier {
            PropertyKey::Expr(Expr::PrivateIdentifier(self.parse_private_identifier()?))
        } else if token.tag() == TokenTag::StringLiteral {
            PropertyKey::Expr(Expr::StringLiteral(self.parse_string_literal()?))
        } else if token.tag().is_numeric_literal() {
            PropertyKey::Expr(self.parse_numeric_literal()?)
        } else if token.tag().is_identifier_like() {
            PropertyKey::IdentifierName(self.parse_identifier_name()?)
        } else {
            let found = self.describe_token(token);
            self.report(
                token.span(),
                Cow::Owned(format!(
                    "Unexpected token '{}' as class element key",
                    found.as_ref()
                )),
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Class element keys must be identifiers, strings, numbers, private identifiers (#name), or computed expressions [expr].",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        };

        Some(key)
    }

    // ComputedPropertyName[Yield, Await] :
    //   `[` AssignmentExpression[+In, ?Yield, ?Await] `]`
    fn parse_property_key(&mut self, in_cover: bool) -> Option<PropertyKey> {
        let token = self.current_token;
        let token_tag = token.tag();
        if token_tag.is_identifier_like() {
            return Some(PropertyKey::IdentifierName(self.parse_identifier_name()?));
        }

        match token_tag {
            TokenTag::PrivateIdentifier => {
                let private = self.parse_private_identifier()?;
                self.report(
                    token.span(),
                    "Private property names are only allowed in class members",
                    ReportOptions::default(),
                );
                Some(PropertyKey::Expr(Expr::PrivateIdentifier(private)))
            }
            TokenTag::LeftBracket => {
                self.advance()?;
                // Computed keys stay expressions when an object cover becomes a
                // binding pattern, so they must not replace binding evidence.
                let context = enter_grammar_context![&mut self.context, +In];
                let ts_context = self.ts_context;
                self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
                self.ts_context.remove(TsContext::IN_TYPE_BINDING);
                let expression = (|| {
                    let expression = self.parse_value_expression(Precedence::Assignment)?;
                    if self.current_token.tag() == TokenTag::Comma {
                        self.report(
                            self.current_token.span(),
                            "A comma expression in a computed property key must be parenthesized",
                            ReportOptions::default(),
                        );
                        self.parse_sequence_expression(Precedence::Comma, expression)
                    } else {
                        Some(expression)
                    }
                })();
                self.ts_context = ts_context;
                self.context = context;
                let expression = expression?;
                let (message, help) = if in_cover {
                    ("Expected ']' after computed property key", None)
                } else {
                    (
                        "Expected ']' to close a computed property key",
                        Some("A computed property key is written '[expr]'"),
                    )
                };
                if !self.expect(TokenTag::RightBracket, message, help) {
                    return None;
                }
                Some(PropertyKey::Expr(expression))
            }
            TokenTag::StringLiteral => Some(PropertyKey::Expr(Expr::StringLiteral(
                self.parse_string_literal()?,
            ))),
            tag if tag.is_numeric_literal() => {
                Some(PropertyKey::Expr(self.parse_numeric_literal()?))
            }
            _ => {
                let found = self.describe_token(token);
                if in_cover {
                    self.report(
                        token.span(),
                        Cow::Owned(format!(
                            "Unexpected token '{}' as property key",
                            found.as_ref()
                        )),
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Property keys must be identifiers, strings, numbers, or computed expressions [expr].",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                } else {
                    self.report(
                        token.span(),
                        Cow::Owned(format!(
                            "Unexpected token '{}' as signature key",
                            found.as_ref()
                        )),
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Signature keys must be identifiers, strings, numbers, or computed expressions [expr].",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                }
                None
            }
        }
    }

    // MemberExpression[Yield, Await] :
    //   MemberExpression[?Yield, ?Await] `[` Expression[+In, ?Yield, ?Await] `]`
    //
    // CallExpression[Yield, Await] :
    //   CallExpression[?Yield, ?Await] `[` Expression[+In, ?Yield, ?Await] `]`
    //
    // OptionalChain[Yield, Await] :
    //   `?.` `[` Expression[+In, ?Yield, ?Await] `]`
    //   OptionalChain[?Yield, ?Await] `[` Expression[+In, ?Yield, ?Await] `]`
    fn parse_computed_member_expression(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBracket);
        let start = object.span(&self.ast).start;
        let open_bracket_span = self.current_token.span();
        self.advance()?;
        let context = enter_grammar_context![&mut self.context, +In];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let expression = self.parse_expr(Precedence::Lowest, ParseExpressionOpts::default());
        self.ts_context = ts_context;
        self.context = context;
        let expression = expression?;
        let property = PropertyKey::Expr(expression);
        let end = self.current_token.end();
        if self.current_token.tag() != TokenTag::RightBracket {
            self.report_expected(
                self.current_token.span(),
                "Expected ']' after computed property",
                ReportOptions {
                    help: Some(Cow::Borrowed("Computed member access must end with ']'.")),
                    labels: vec![self.label(open_bracket_span, "Opened here")],
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.advance()?;
        Some(self.ast.expr_member_expression(
            Span::new(start, end),
            object,
            property,
            true,
            optional,
        ))
    }

    // Expose member parsing to the caller so the hot dot path does not save
    // parser state across a separate call.
    #[inline(always)]
    fn parse_static_member_expression(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Dot);
        self.advance()?;
        self.parse_member_property(object, optional)
    }

    // Keep property and member-node construction together and expose the caller's
    // optional flag to node construction without a separate helper call.
    #[inline(always)]
    fn parse_member_property(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        let start = object.span(&self.ast).start;
        let token = self.current_token;
        if !token.tag().is_identifier_like() && token.tag() != TokenTag::PrivateIdentifier {
            self.report_expected(
                self.current_token.span(),
                "Expected property name after '.'",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Use an identifier or private identifier (#name) for member access.",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let property = if token.tag() == TokenTag::PrivateIdentifier {
            PropertyKey::Expr(Expr::PrivateIdentifier(self.parse_private_identifier()?))
        } else {
            PropertyKey::IdentifierName(self.parse_identifier_name()?)
        };
        Some(self.ast.expr_member_expression(
            Span::new(start, token.end()),
            object,
            property,
            false,
            optional,
        ))
    }

    fn parse_call_expression(
        &mut self,
        callee: Expr,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftParen);
        let start = callee.span(&self.ast).start;
        let open_paren_span = self.current_token.span();
        self.advance()?;
        let arguments = self.parse_arguments()?;
        let end = self.current_token.end();
        if self.current_token.tag() != TokenTag::RightParen {
            self.report_expected(
                self.current_token.span(),
                "Expected ')' after function arguments",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Function calls must end with ')'. Check for missing commas or unclosed parentheses.",
                    )),
                    labels: vec![self.label(open_paren_span, "Opened here")],
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.advance()?;
        Some(self.ast.expr_call_expression(
            Span::new(start, end),
            callee,
            type_arguments,
            arguments,
            optional,
        ))
    }

    fn cover_to_call_expression(
        &mut self,
        cover: ParenthesizedCover,
        callee: Expr,
    ) -> Option<Expr> {
        Some(self.ast.expr_call_expression(
            Span::new(callee.span(&self.ast).start, cover.end),
            callee,
            None,
            cover.elements,
            false,
        ))
    }

    // Ordinary suffixes only carry the expression and optional-chain state.
    // Type arguments are resolved separately when their opening token is found.
    fn parse_postfix_expression(
        &mut self,
        mut expression: Expr,
        context: LhsContext,
    ) -> Option<PostfixExpression> {
        let mut in_optional_chain = false;
        let type_arguments = loop {
            expression = match self.current_token.tag() {
                TokenTag::Dot => self.parse_static_member_expression(expression, false)?,
                TokenTag::LeftBracket if context != LhsContext::Decorator || in_optional_chain => {
                    self.parse_computed_member_expression(expression, false)?
                }
                TokenTag::LeftParen if context != LhsContext::NewCallee => {
                    self.parse_call_expression(expression, false, None)?
                }
                TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                    self.parse_tagged_template_expression(expression, None, in_optional_chain)?
                }
                TokenTag::OptionalChaining => {
                    self.check_new_optional_chain(context);
                    self.advance()?;
                    in_optional_chain = true;
                    self.parse_optional_chain_element(expression, true)?
                }
                TokenTag::LogicalNot
                    if self.lang.is_typescript()
                        && !self.current_token.has_line_terminator_before() =>
                {
                    self.parse_non_null_expression(expression)?
                }
                TokenTag::LessThan | TokenTag::LeftShift if self.lang.is_typescript() => {
                    let Some(type_arguments) = self.try_parse_type_arguments_in_expression()?
                    else {
                        break None;
                    };
                    match self.parse_type_argumented_postfix(
                        expression,
                        type_arguments,
                        context,
                        &mut in_optional_chain,
                    )? {
                        PostfixStep::Continue(expression) => expression,
                        PostfixStep::Finish {
                            expression: parsed,
                            type_arguments,
                        } => {
                            expression = parsed;
                            if context == LhsContext::NewCallee {
                                break Some(type_arguments);
                            }
                            expression =
                                self.build_instantiation_expression(expression, type_arguments);
                            break None;
                        }
                    }
                }
                _ => break None,
            };
        };
        if in_optional_chain {
            expression = self
                .ast
                .expr_chain_expression(expression.span(&self.ast), expression);
        }
        Some(PostfixExpression {
            expression,
            type_arguments,
        })
    }

    // A call or template owns the arguments immediately. A terminating suffix
    // returns them to the enclosing context; other suffixes need a bare
    // instantiation node before ordinary postfix parsing can continue.
    fn parse_type_argumented_postfix(
        &mut self,
        mut expression: Expr,
        mut type_arguments: TsTypeParameterInstantiation,
        context: LhsContext,
        in_optional_chain: &mut bool,
    ) -> Option<PostfixStep> {
        loop {
            let tag = self.current_token.tag();
            let expression = match tag {
                TokenTag::LeftParen if context != LhsContext::NewCallee => {
                    self.parse_call_expression(expression, false, Some(type_arguments))?
                }
                TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => self
                    .parse_tagged_template_expression(
                        expression,
                        Some(type_arguments),
                        *in_optional_chain,
                    )?,
                TokenTag::OptionalChaining => {
                    self.check_new_optional_chain(context);
                    self.advance()?;
                    let tag = self.current_token.tag();
                    if tag.is_identifier_like() || tag == TokenTag::PrivateIdentifier {
                        self.report_instantiation_property_access();
                    }
                    let object = self.build_instantiation_expression(expression, type_arguments);
                    *in_optional_chain = true;
                    self.parse_optional_chain_element(object, true)?
                }
                TokenTag::LessThan | TokenTag::LeftShift => {
                    if let Some(next) = self.try_parse_type_arguments_in_expression()? {
                        expression =
                            self.build_instantiation_expression(expression, type_arguments);
                        type_arguments = next;
                        continue;
                    }
                    return Some(PostfixStep::Finish {
                        expression,
                        type_arguments,
                    });
                }
                TokenTag::Dot => {
                    self.report_instantiation_property_access();
                    self.build_instantiation_expression(expression, type_arguments)
                }
                TokenTag::LeftBracket if context != LhsContext::Decorator || *in_optional_chain => {
                    self.build_instantiation_expression(expression, type_arguments)
                }
                TokenTag::LogicalNot if !self.current_token.has_line_terminator_before() => {
                    self.build_instantiation_expression(expression, type_arguments)
                }
                _ => {
                    return Some(PostfixStep::Finish {
                        expression,
                        type_arguments,
                    });
                }
            };
            return Some(PostfixStep::Continue(expression));
        }
    }

    fn check_new_optional_chain(&mut self, context: LhsContext) {
        if context == LhsContext::NewCallee {
            self.report(
                self.current_token.span(),
                "Optional chaining is not allowed in new expression",
                ReportOptions::default(),
            );
        }
    }

    fn report_instantiation_property_access(&mut self) {
        self.report(
            self.current_token.span(),
            "An instantiation expression cannot be followed by property access",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Parenthesize the instantiation expression before accessing a property.",
                )),
                ..ReportOptions::default()
            },
        );
    }

    fn build_instantiation_expression(
        &mut self,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> Expr {
        self.ast.expr_ts_instantiation_expression(
            Span::new(
                expression.span(&self.ast).start,
                type_arguments.span(&self.ast).end,
            ),
            expression,
            type_arguments,
        )
    }

    fn parse_optional_chain_element(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        let tag = self.current_token.tag();
        if tag.is_identifier_like() || tag == TokenTag::PrivateIdentifier {
            return self.parse_member_property(object, optional);
        }

        if self.lang.is_typescript() && is_angle_open(tag) {
            let checkpoint = self.checkpoint();
            let type_arguments = self.try_parse_type_arguments_in_expression()?;
            if let Some(type_arguments) = type_arguments {
                match self.current_token.tag() {
                    TokenTag::LeftParen => {
                        return self.parse_call_expression(object, optional, Some(type_arguments));
                    }
                    TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                        return self.parse_tagged_template_expression(
                            object,
                            Some(type_arguments),
                            true,
                        );
                    }
                    _ => {}
                }
            }
            self.rewind(checkpoint);
            self.lexer.rewrite_last_collected_token(self.current_token);
        }

        match tag {
            TokenTag::LeftBracket => self.parse_computed_member_expression(object, optional),
            TokenTag::LeftParen => self.parse_call_expression(object, optional, None),
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                self.parse_tagged_template_expression(object, None, true)
            }
            _ => {
                self.report_expected(
                    self.current_token.span(),
                    "Expected property name, '[', or '(' after '?.'",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Optional chaining must be followed by property access (.x), computed access ([x]), or a call (()).",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
        }
    }

    // ArgumentList[Yield, Await] :
    //   AssignmentExpression[+In, ?Yield, ?Await]
    //   `...` AssignmentExpression[+In, ?Yield, ?Await]
    //   ArgumentList[?Yield, ?Await] `,` AssignmentExpression[+In, ?Yield, ?Await]
    //   ArgumentList[?Yield, ?Await] `,` `...` AssignmentExpression[+In, ?Yield, ?Await]
    #[inline(always)]
    fn parse_arguments(&mut self) -> Option<SubRange<Argument>> {
        let checkpoint = self.scratch_a.begin();
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = self.parse_argument_list(checkpoint);
        self.ts_context = ts_context;
        self.context = grammar_context;
        self.scratch_a.reset(checkpoint);
        result
    }

    // Inline the list loop into its context-restoring wrapper.
    #[inline(always)]
    fn parse_argument_list(&mut self, checkpoint: usize) -> Option<SubRange<Argument>> {
        while self.current_token.tag() != TokenTag::RightParen
            && self.current_token.tag() != TokenTag::Eof
        {
            let argument =
                if self.eat(TokenTag::Spread)? {
                    let start = self.lexer.state.prev_token_end - 3;
                    let expression =
                        self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
                    Argument::SpreadElement(self.ast.spread_element(
                        Span::new(start, expression.span(&self.ast).end),
                        expression,
                    ))
                } else {
                    Argument::Expr(
                        self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?,
                    )
                };
            self.scratch_a.append(argument);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        Some(self.add_extra_from_scratch_a(checkpoint))
    }

    fn parse_new_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::New);
        let meta_token = self.current_token;
        let start = meta_token.start();
        self.report_if_escaped_keyword(meta_token);
        let meta_name = self.identifier_name(meta_token);
        self.advance_without_escape_check()?;
        let meta = self.ast.identifier_name(meta_token.span(), meta_name);
        if self.current_token.tag() == TokenTag::Dot {
            return self.parse_new_target(meta);
        }
        let callee_starts_with_import = self.current_token.tag() == TokenTag::Import;
        let callee = match self.current_token.tag() {
            TokenTag::LeftParen => self.parse_parenthesized_expression()?,
            TokenTag::New => self.parse_new_expression()?,
            _ => self.parse_primary_expression(Precedence::Primary)?,
        };
        let PostfixExpression {
            expression: callee,
            type_arguments,
        } = self.parse_postfix_expression(callee, LhsContext::NewCallee)?;
        if self.ast.node_kind(callee.node_id()) == NodeKind::Super {
            self.report(
                callee.span(&self.ast),
                "'super' cannot be used as the target of 'new'",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Call the parent constructor with 'super()', or construct a parent member with 'new super.prop()'.",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        if callee_starts_with_import
            && is_import_expression_or_member_access_on_import_expression(callee, &self.ast)
        {
            self.report(
                callee.span(&self.ast),
                "Dynamic import cannot be used as the target of 'new'",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Parenthesize the import call if its result should be constructed",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        let arguments = if self.current_token.tag() == TokenTag::LeftParen {
            let open_paren_span = self.current_token.span();
            self.advance()?;
            let arguments = self.parse_arguments()?;
            if self.current_token.tag() != TokenTag::RightParen {
                self.report_expected(
                    self.current_token.span(),
                    "Expected ')' after constructor arguments",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Constructor calls must end with ')'.")),
                        labels: vec![self.label(open_paren_span, "Opened here")],
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
            self.advance()?;
            arguments
        } else {
            SubRange::empty()
        };
        let end = self
            .lexer
            .state
            .prev_token_end
            .max(callee.span(&self.ast).end);
        Some(
            self.ast
                .expr_new_expression(Span::new(start, end), callee, type_arguments, arguments),
        )
    }

    fn parse_import_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Import);
        self.report_if_escaped_keyword(self.current_token);
        let import_name = self.parse_identifier_name()?;
        let start = import_name.span(&self.ast).start;
        match self.current_token.tag() {
            TokenTag::Dot => self.parse_import_meta_or_phase_import(import_name),
            TokenTag::LeftParen => self.parse_dynamic_import(start, None, false),
            _ => {
                self.report(
                    self.current_token.span(),
                    "'import' keyword is not allowed here",
                    ReportOptions::default(),
                );
                None
            }
        }
    }

    fn parse_import_meta_or_phase_import(&mut self, import_name: IdentifierName) -> Option<Expr> {
        self.advance()?;
        let property = self.current_token;
        let (phase, sync) = match property.tag() {
            TokenTag::Source => (Some(ImportPhase::Source), false),
            TokenTag::Defer => (Some(ImportPhase::Defer), false),
            TokenTag::Identifier if property.text(self.source) == "sync" => (None, true),
            TokenTag::Identifier if property.text(self.source) == "meta" => {
                self.record_module_syntax();
                if self.source_type != SourceType::Module {
                    self.diagnostics.defer(Diagnostic::error(
                        "'import.meta' is only valid in module code",
                        Span::new(import_name.span(&self.ast).start, property.end()),
                    ));
                }
                return self.parse_meta_property(import_name);
            }
            _ => {
                self.report(
                    property.span(),
                    "The only valid meta properties for 'import' are 'import.meta', 'import.source()', or 'import.defer()'",
                    ReportOptions::default(),
                );
                return self.parse_meta_property(import_name);
            }
        };
        self.advance()?;
        if self.current_token.tag() != TokenTag::LeftParen {
            self.report_expected(
                self.current_token.span(),
                "Expected '(' after import",
                ReportOptions::default(),
            );
            let name = self.identifier_name(property);
            let property = self.ast.identifier_name(property.span(), name);
            return Some(self.ast.expr_meta_property(
                Span::new(
                    import_name.span(&self.ast).start,
                    property.span(&self.ast).end,
                ),
                import_name,
                property,
            ));
        }
        self.parse_dynamic_import(import_name.span(&self.ast).start, phase, sync)
    }

    // ImportCall[Yield, Await] :
    //   `import` `(` AssignmentExpression[+In, ?Yield, ?Await] `,`? `)`
    //   `import` `(` AssignmentExpression[+In, ?Yield, ?Await] `,` AssignmentExpression[+In, ?Yield, ?Await] `,`? `)`
    fn parse_dynamic_import(
        &mut self,
        start: u32,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> Option<Expr> {
        if !self.expect(TokenTag::LeftParen, "Expected '(' after import", None) {
            return None;
        }
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            let source = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
            let options = if self.eat(TokenTag::Comma)? {
                let options = if self.current_token.tag() == TokenTag::RightParen {
                    None
                } else {
                    Some(self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?)
                };
                self.eat(TokenTag::Comma)?;
                options
            } else {
                None
            };
            let end = self.current_token.end();
            if !self.expect(
                TokenTag::RightParen,
                "Expected ')' after import()",
                Some("Dynamic import call must end with ')'"),
            ) {
                return None;
            }
            Some(self.ast.expr_import_expression(
                Span::new(start, end),
                source,
                options,
                phase,
                sync,
            ))
        })();
        self.ts_context = ts_context;
        self.context = grammar_context;
        result
    }

    fn parse_new_target(&mut self, meta: IdentifierName) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Dot);
        self.advance()?;
        let property_token = self.current_token;
        if property_token.text(self.source) != "target" {
            self.report(
                property_token.span(),
                "The only valid meta property for 'new' is 'new.target'",
                ReportOptions::default(),
            );
        } else if !self
            .boundary_context
            .contains(BoundaryContext::ALLOW_NEW_TARGET)
        {
            let syntax =
                CommonJsUse::NewTarget(Span::new(meta.span(&self.ast).start, property_token.end()));
            self.check_commonjs_syntax(syntax);
        }
        self.parse_meta_property(meta)
    }

    fn parse_meta_property(&mut self, meta: IdentifierName) -> Option<Expr> {
        if !self.current_token.tag().is_identifier_like() {
            self.report_expected(
                self.current_token.span(),
                "Expected an identifier name after '.'",
                ReportOptions::default(),
            );
            return None;
        }
        let property = self.parse_identifier_name()?;
        Some(self.ast.expr_meta_property(
            Span::new(meta.span(&self.ast).start, property.span(&self.ast).end),
            meta,
            property,
        ))
    }

    fn parse_jsx_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LessThan);
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = self.parse_jsx_element(JsxElementContext::TopLevel);
        self.ts_context = ts_context;
        result
    }

    fn enter_jsx_tag(&mut self) {
        self.set_lexer_mode(LexerMode::JsxTag);
    }

    fn exit_jsx_tag(&mut self) {
        self.set_lexer_mode(LexerMode::Normal);
    }

    fn parse_jsx_element(&mut self, context: JsxElementContext) -> Option<Expr> {
        let start = self.current_token.start();
        self.enter_jsx_tag();
        if self
            .peek_ahead()
            .is_some_and(|token| token.tag() == TokenTag::GreaterThan)
        {
            return self.parse_jsx_fragment(start, context);
        }

        let opening = self.parse_jsx_opening_element(context)?;
        if opening.self_closing(&self.ast) {
            let children = SubRange::empty();
            return Some(Expr::JsxElement(self.ast.jsx_element(
                Span::new(start, opening.span(&self.ast).end),
                opening,
                children,
                None,
            )));
        }
        let children = self.parse_jsx_children()?;
        let closing = self.parse_jsx_closing_element(opening.name(&self.ast), context)?;
        Some(Expr::JsxElement(self.ast.jsx_element(
            Span::new(start, closing.span(&self.ast).end),
            opening,
            children,
            Some(closing),
        )))
    }

    fn parse_jsx_opening_element(
        &mut self,
        context: JsxElementContext,
    ) -> Option<JsxOpeningElement> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LessThan);
        let start = self.current_token.start();
        self.advance()?;
        let name = self.parse_jsx_element_name()?;

        let type_arguments = if self.lang.is_typescript() && is_angle_open(self.current_token.tag())
        {
            self.exit_jsx_tag();
            let arguments = self.parse_type_arguments()?;
            self.enter_jsx_tag();
            self.re_scan_current()?;
            Some(arguments)
        } else {
            None
        };

        let attributes = self.parse_jsx_attributes()?;
        let self_closing = self.eat(TokenTag::Slash)?;
        if self.current_token.tag() != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span(),
                "Expected '>' to close JSX opening element",
                ReportOptions {
                    help: Some(Cow::Borrowed("Add '>' to close the JSX tag")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let end = self.current_token.end();
        if self_closing {
            if context == JsxElementContext::Attribute {
                self.enter_jsx_tag();
                self.advance()?;
            } else {
                self.exit_jsx_tag();
                if context == JsxElementContext::TopLevel {
                    self.advance()?;
                }
            }
        }
        Some(self.ast.jsx_opening_element(
            Span::new(start, end),
            name,
            type_arguments,
            attributes,
            self_closing,
        ))
    }

    fn parse_jsx_closing_element(
        &mut self,
        opening_name: JsxElementName,
        context: JsxElementContext,
    ) -> Option<JsxClosingElement> {
        if self.current_token.tag() != TokenTag::LessThan {
            self.report_expected(
                self.current_token.span(),
                "Expected '</' to close the JSX element",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add a closing tag to match the opening element",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let closing_start = self.current_token.start();
        self.enter_jsx_tag();
        self.advance()?;
        if !self.expect(
            TokenTag::Slash,
            "Expected '/' in JSX closing element",
            Some("Add '/' after '<' to close the element"),
        ) {
            return None;
        }
        let closing_name = self.parse_jsx_element_name()?;
        if self.current_token.tag() != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span(),
                "Expected '>' to close JSX closing element",
                ReportOptions {
                    help: Some(Cow::Borrowed("Add '>' to complete the closing tag")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let end = self.current_token.end();
        match context {
            JsxElementContext::Child => self.exit_jsx_tag(),
            JsxElementContext::TopLevel => {
                self.exit_jsx_tag();
                self.advance()?;
            }
            JsxElementContext::Attribute => {
                self.enter_jsx_tag();
                self.advance()?;
            }
        }
        let closing_span = closing_name.span(&self.ast);
        if !self.jsx_names_match(opening_name, closing_name) {
            let opening_span = opening_name.span(&self.ast);
            self.report(
                closing_span,
                Cow::Owned(format!(
                    "Expected closing tag for '<{}>' but found '</{}>'",
                    self.span_text(opening_span),
                    self.span_text(closing_span)
                )),
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "JSX opening and closing tags must have matching names",
                    )),
                    labels: vec![Label {
                        span: opening_span,
                        message: Cow::Borrowed("opening tag"),
                    }],
                    ..ReportOptions::default()
                },
            );
        }
        Some(
            self.ast
                .jsx_closing_element(Span::new(closing_start, end), closing_name),
        )
    }

    fn jsx_names_match(&self, a: JsxElementName, b: JsxElementName) -> bool {
        use JsxElementNameData as Name;
        use JsxMemberExpressionObjectData as Object;

        let ast = &self.ast;
        let same_name = |a: JsxIdentifier, b: JsxIdentifier| {
            ast.get_utf8(a.name(ast)) == ast.get_utf8(b.name(ast))
        };

        let a_span = a.span(ast);
        let b_span = b.span(ast);
        if a_span.end - a_span.start == b_span.end - b_span.start
            && self.span_text(a_span) == self.span_text(b_span)
        {
            return true;
        }

        let (mut a, mut b) = match (ast.jsx_element_name_data(a), ast.jsx_element_name_data(b)) {
            (Name::JsxIdentifier(a), Name::JsxIdentifier(b)) => return same_name(a, b),
            (Name::JsxNamespacedName(a), Name::JsxNamespacedName(b)) => {
                return same_name(a.namespace(ast), b.namespace(ast))
                    && same_name(a.name(ast), b.name(ast));
            }
            (Name::JsxMemberExpression(a), Name::JsxMemberExpression(b)) => (a, b),
            _ => return false,
        };

        loop {
            if !same_name(a.property(ast), b.property(ast)) {
                return false;
            }
            match (
                ast.jsx_member_expression_object_data(a.object(ast)),
                ast.jsx_member_expression_object_data(b.object(ast)),
            ) {
                (Object::JsxIdentifier(a), Object::JsxIdentifier(b)) => return same_name(a, b),
                (Object::JsxMemberExpression(next_a), Object::JsxMemberExpression(next_b)) => {
                    a = next_a;
                    b = next_b;
                }
                _ => return false,
            }
        }
    }

    fn parse_jsx_fragment(&mut self, start: u32, context: JsxElementContext) -> Option<Expr> {
        self.advance()?;
        if self.current_token.tag() != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span(),
                "Expected '>' to close JSX opening fragment",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add '>' to complete the fragment opening tag",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let opening_end = self.current_token.end();
        let opening = self.ast.jsx_opening_fragment(Span::new(start, opening_end));
        let children = self.parse_jsx_children()?;
        let closing_start = self.current_token.start();
        self.enter_jsx_tag();
        self.advance()?;
        if !self.expect(
            TokenTag::Slash,
            "Expected '/' in JSX closing fragment",
            Some("Add '/' to close the fragment"),
        ) {
            return None;
        }
        if self.current_token.tag() != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span(),
                "Expected '>' to close JSX closing element",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add '>' to complete the fragment closing tag",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let closing_end = self.current_token.end();
        match context {
            JsxElementContext::Child => self.exit_jsx_tag(),
            JsxElementContext::TopLevel => {
                self.exit_jsx_tag();
                self.advance()?;
            }
            JsxElementContext::Attribute => {
                self.enter_jsx_tag();
                self.advance()?;
            }
        }
        let closing = self
            .ast
            .jsx_closing_fragment(Span::new(closing_start, closing_end));
        Some(
            self.ast
                .expr_jsx_fragment(Span::new(start, closing_end), opening, children, closing),
        )
    }

    fn parse_jsx_children(&mut self) -> Option<SubRange<JsxChild>> {
        let checkpoint = self.scratch_b.begin();
        self.exit_jsx_tag();
        let result = (|| {
            loop {
                let token = self.lexer.scan_jsx_text();
                self.current_token = token;
                match token.tag() {
                    TokenTag::JsxText => {
                        let value = self.ast.source_slice(token.start(), token.end());
                        let jsx_text = self.ast.jsx_child_jsx_text(token.span(), value);
                        self.scratch_b.append(jsx_text);
                    }
                    TokenTag::LessThan => {
                        if self.peek_ahead()?.tag() == TokenTag::Slash {
                            break;
                        }
                        let child = self.parse_jsx_element(JsxElementContext::Child)?;
                        match self.ast.expr_data(child) {
                            ExprData::JsxElement(element) => {
                                self.scratch_b.append(JsxChild::JsxElement(element));
                            }
                            ExprData::JsxFragment(fragment) => {
                                self.scratch_b.append(JsxChild::JsxFragment(fragment));
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenTag::LeftBrace => {
                        let child = self.parse_jsx_child_from_left_brace()?;
                        self.scratch_b.append(child);
                    }
                    _ => break,
                }
            }
            Some(self.add_extra_from_scratch_b(checkpoint))
        })();
        self.scratch_b.reset(checkpoint);
        result
    }

    // JSXChild :
    //   `{` JSXChildExpression? `}`
    //
    // JSXChildExpression :
    //   AssignmentExpression
    //   `...` AssignmentExpression
    fn parse_jsx_child_from_left_brace(&mut self) -> Option<JsxChild> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBrace);
        let start = self.current_token.start();
        self.advance()?;
        if self.current_token.tag() == TokenTag::RightBrace {
            let empty_span = Span::new(start + 1, self.current_token.start());
            let empty = self.ast.jsx_empty_expression(empty_span);
            let expression = JsxExpression::JsxEmptyExpression(empty);
            let end = self.current_token.end();
            return Some(
                self.ast
                    .jsx_child_jsx_expression_container(Span::new(start, end), expression),
            );
        }
        if self.eat(TokenTag::Spread)? {
            // JSX omits grammar parameters; this parser allows in inside {...}
            // independently of an enclosing for initializer.
            let grammar_context = enter_grammar_context![&mut self.context, +In];
            let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default());
            self.context = grammar_context;
            let expression = expression?;
            let end = self.expect_jsx_child_right_brace("JSX spread")?;
            return Some(
                self.ast
                    .jsx_child_jsx_spread_child(Span::new(start, end), expression),
            );
        }
        // JSX omits grammar parameters; this parser allows in inside {...}
        // independently of an enclosing for initializer.
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default());
        self.context = grammar_context;
        let expression = expression?;
        let end = self.expect_jsx_child_right_brace("JSX expression")?;
        Some(self.ast.jsx_child_jsx_expression_container(
            Span::new(start, end),
            JsxExpression::Expr(expression),
        ))
    }

    fn expect_jsx_child_right_brace(&mut self, what: &'static str) -> Option<u32> {
        if self.current_token.tag() != TokenTag::RightBrace {
            let message = match what {
                "JSX spread" => "Expected '}' to close JSX spread",
                _ => "Expected '}' to close JSX expression",
            };
            self.report_expected(
                self.current_token.span(),
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed("Add '}' to close the expression")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        Some(self.current_token.end())
    }

    fn parse_jsx_element_name(&mut self) -> Option<JsxElementName> {
        let token = self.current_token;
        if token.tag() != TokenTag::JsxIdentifier {
            self.report_expected(
                self.current_token.span(),
                "Expected JSX element name",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "JSX element names must start with a valid identifier",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let name = self.identifier_name(token);
        self.advance()?;
        let identifier = self.ast.jsx_identifier(token.span(), name);
        let start = token.start();
        let mut element_name = JsxElementName::JsxIdentifier(identifier);
        let mut member_object = JsxMemberExpressionObject::JsxIdentifier(identifier);
        let mut is_member = false;

        while self.eat(TokenTag::Dot)? {
            if self.current_token.tag() != TokenTag::JsxIdentifier {
                self.report_expected(
                    self.current_token.span(),
                    "Expected identifier after '.' in JSX member expression",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Member expressions in JSX must have the form 'object.property'",
                        )),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
            is_member = true;
            let property_token = self.current_token;
            let property_name = self.identifier_name(property_token);
            self.advance()?;
            let property = self
                .ast
                .jsx_identifier(property_token.span(), property_name);
            let member = self.ast.jsx_member_expression(
                Span::new(start, property_token.end()),
                member_object,
                property,
            );
            member_object = JsxMemberExpressionObject::JsxMemberExpression(member);
            element_name = JsxElementName::JsxMemberExpression(member);
        }

        if !is_member && self.eat(TokenTag::Colon)? {
            if self.current_token.tag() != TokenTag::JsxIdentifier {
                self.report_expected(
                    self.current_token.span(),
                    "Expected identifier after ':' in namespaced element name",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Namespaced element names must have the form 'namespace:name'",
                        )),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
            let local_token = self.current_token;
            let local_name = self.identifier_name(local_token);
            self.advance()?;
            let local = self.ast.jsx_identifier(local_token.span(), local_name);
            element_name = JsxElementName::JsxNamespacedName(self.ast.jsx_namespaced_name(
                Span::new(start, local_token.end()),
                identifier,
                local,
            ));
        }
        Some(element_name)
    }

    fn parse_jsx_attributes(&mut self) -> Option<SubRange<JsxAttributeItem>> {
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while matches!(
                self.current_token.tag(),
                TokenTag::JsxIdentifier | TokenTag::LeftBrace
            ) {
                let attribute = self.parse_jsx_attribute()?;
                self.scratch_a.append(attribute);
            }
            Some(self.add_extra_from_scratch_a(checkpoint))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_jsx_attribute(&mut self) -> Option<JsxAttributeItem> {
        if self.current_token.tag() == TokenTag::LeftBrace {
            return self.parse_jsx_spread_attribute();
        }
        let name = self.parse_jsx_attribute_name()?;
        let start = name.span(&self.ast).start;
        let value = if self.eat(TokenTag::Assign)? {
            Some(self.parse_jsx_attribute_value()?)
        } else {
            None
        };
        let end = value.map_or(name.span(&self.ast).end, |value| value.span(&self.ast).end);
        Some(JsxAttributeItem::JsxAttribute(self.ast.jsx_attribute(
            Span::new(start, end),
            name,
            value,
        )))
    }

    fn parse_jsx_attribute_name(&mut self) -> Option<JsxAttributeName> {
        let token = self.current_token;
        let name = self.identifier_name(token);
        self.advance()?;
        let namespace = self.ast.jsx_identifier(token.span(), name);
        if !self.eat(TokenTag::Colon)? {
            return Some(JsxAttributeName::JsxIdentifier(namespace));
        }
        if self.current_token.tag() != TokenTag::JsxIdentifier {
            self.report_expected(
                self.current_token.span(),
                "Expected identifier after ':' in namespaced attribute",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Namespaced attributes must have the form 'namespace:name'",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let local_token = self.current_token;
        let local_name = self.identifier_name(local_token);
        self.advance()?;
        let local = self.ast.jsx_identifier(local_token.span(), local_name);
        Some(JsxAttributeName::JsxNamespacedName(
            self.ast.jsx_namespaced_name(
                Span::new(token.start(), local_token.end()),
                namespace,
                local,
            ),
        ))
    }

    fn parse_jsx_attribute_value(&mut self) -> Option<JsxAttributeValue> {
        match self.current_token.tag() {
            TokenTag::StringLiteral => Some(JsxAttributeValue::StringLiteral(
                self.parse_string_literal()?,
            )),
            TokenTag::LeftBrace => {
                let container = self.parse_jsx_expression_container(JsxExprContext::Tag)?;
                if container
                    .expression(&self.ast)
                    .is_jsx_empty_expression(&self.ast)
                {
                    self.report(
                        container.span(&self.ast),
                        "JSX attribute value cannot be an empty expression",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Replace {} with a valid expression or remove the braces to use a string literal",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                }
                Some(JsxAttributeValue::JsxExpressionContainer(container))
            }
            TokenTag::LessThan => {
                let expression = self.parse_jsx_element(JsxElementContext::Attribute)?;
                Some(match self.ast.expr_data(expression) {
                    ExprData::JsxElement(element) => JsxAttributeValue::JsxElement(element),
                    ExprData::JsxFragment(fragment) => JsxAttributeValue::JsxFragment(fragment),
                    _ => unreachable!(),
                })
            }
            _ => {
                self.report_expected(
                    self.current_token.span(),
                    "Expected string literal or JSX expression for attribute value",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "JSX attribute values must be either a string literal (e.g. \"value\") or an expression in braces (e.g. {expression})",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
        }
    }

    // JSXAttributeValue :
    //   `{` AssignmentExpression `}`
    //
    // JSXChild :
    //   `{` JSXChildExpression? `}`
    //
    // JSXChildExpression :
    //   AssignmentExpression
    //   `...` AssignmentExpression
    fn parse_jsx_expression_container(
        &mut self,
        context: JsxExprContext,
    ) -> Option<JsxExpressionContainer> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBrace);
        let start = self.current_token.start();
        if context == JsxExprContext::Tag {
            self.exit_jsx_tag();
        }
        self.advance()?;
        if self.current_token.tag() == TokenTag::RightBrace {
            let end = self.current_token.end();
            if context == JsxExprContext::Tag {
                self.enter_jsx_tag();
            }
            self.advance()?;
            let empty = self.ast.jsx_empty_expression(Span::new(start + 1, end - 1));
            return Some(self.ast.jsx_expression_container(
                Span::new(start, end),
                JsxExpression::JsxEmptyExpression(empty),
            ));
        }
        // JSX omits grammar parameters; this parser allows in inside {...}
        // independently of an enclosing for initializer.
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default());
        self.context = grammar_context;
        let expression = expression?;
        let end = self.current_token.end();
        if context == JsxExprContext::Tag {
            self.enter_jsx_tag();
        }
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close JSX expression",
            Some("Add '}' to close the expression"),
        ) {
            return None;
        }
        Some(
            self.ast
                .jsx_expression_container(Span::new(start, end), JsxExpression::Expr(expression)),
        )
    }

    // JSXSpreadAttribute :
    //   `{` `...` AssignmentExpression `}`
    fn parse_jsx_spread_attribute(&mut self) -> Option<JsxAttributeItem> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LeftBrace);
        let start = self.current_token.start();
        self.exit_jsx_tag();
        self.advance()?;
        if !self.expect(
            TokenTag::Spread,
            "Expected '...' after '{' in JSX spread",
            Some("Add '...' to spread the expression"),
        ) {
            return None;
        }
        // JSX omits grammar parameters; this parser allows in inside {...}
        // independently of an enclosing for initializer.
        let grammar_context = enter_grammar_context![&mut self.context, +In];
        let argument = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default());
        self.context = grammar_context;
        let argument = argument?;
        let end = self.current_token.end();
        self.enter_jsx_tag();
        if !self.expect(
            TokenTag::RightBrace,
            "Expected '}' to close JSX spread",
            Some("Add '}' to close the spread expression"),
        ) {
            return None;
        }
        Some(JsxAttributeItem::JsxSpreadAttribute(
            self.ast
                .jsx_spread_attribute(Span::new(start, end), argument),
        ))
    }

    fn parse_await_expression(&mut self, start: u32) -> Option<Expr> {
        let argument = self.parse_expr(Precedence::Prefix, ParseExpressionOpts::default())?;
        if self.current_token.tag() == TokenTag::Exponent {
            self.report(
                Span::new(start, argument.span(&self.ast).end),
                "The left-hand side of '**' cannot be an unparenthesized await expression",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add parentheses to make precedence explicit, for example `(await x) ** y`.",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        Some(
            self.ast
                .expr_await_expression(Span::new(start, argument.span(&self.ast).end), argument),
        )
    }

    /// Preserve an await/yield expression when an identifier interpretation is
    /// impossible. TypeScript declaration containers defer context checks to
    /// semantic analysis; other recovery paths report a parser error.
    #[cold]
    #[inline(never)]
    fn can_recover_await_or_yield_expression(&mut self) -> Option<bool> {
        debug_assert!(matches!(
            self.current_token.tag(),
            TokenTag::Await | TokenTag::Yield
        ));
        let next = self.peek_ahead()?;
        let can_follow_identifier =
            is_binary_operator_like(next.tag()) || next.tag() == TokenTag::Of;
        Some(
            !next.has_line_terminator_before()
                && !can_follow_identifier
                && (next.tag().is_identifier_like()
                    || next.tag().is_numeric_literal()
                    || next.tag() == TokenTag::StringLiteral),
        )
    }

    fn is_ambiguous_await_prefix(&self) -> bool {
        if self.current_token.has_line_terminator_before() {
            return true;
        }
        match self.current_token.tag() {
            TokenTag::Plus
            | TokenTag::Minus
            | TokenTag::LeftParen
            | TokenTag::LeftBracket
            | TokenTag::NoSubstitutionTemplate
            | TokenTag::TemplateHead
            | TokenTag::Slash
            | TokenTag::SlashAssign
            | TokenTag::LogicalNot
            | TokenTag::LessThan => true,
            TokenTag::Of | TokenTag::Instanceof if !self.current_token.is_escaped() => true,
            TokenTag::In if !self.current_token.is_escaped() => self.context.contains(Context::IN),
            TokenTag::As | TokenTag::Satisfies if !self.current_token.is_escaped() => {
                self.lang.is_typescript()
            }
            TokenTag::RegexLiteral => {
                unreachable!("regular expressions are initially scanned as '/' or '/='")
            }
            _ => false,
        }
    }

    fn try_parse_type_arguments_in_expression(
        &mut self,
    ) -> Option<Option<TsTypeParameterInstantiation>> {
        if !is_angle_open(self.current_token.tag()) {
            return Some(None);
        }
        // Exact lists are only tried by this rule. A Peel failure also rules
        // them out, so one lookup covers both the grammar and follow check.
        if !self.failed_type_arguments.is_empty()
            && self.has_failed_type_arguments(
                &self.speculation_key(),
                TypeArgumentFailures::EXPRESSION | TypeArgumentFailures::PEEL,
            )
            || self.lexer.mode == LexerMode::Normal
                && self
                    .type_argument_lookahead
                    .cannot_close(self.current_token.start())
        {
            return Some(None);
        }
        let after = self.peek_ahead()?;
        // Reserved names are recoverable type arguments. The closing angle
        // and follow-token checks still decide whether this is a type list.
        if !can_parse_type_for_recovery(after.tag()) && after.tag() != TokenTag::GreaterThan {
            return Some(None);
        }

        if self.type_argument_speculation_depth != 0 && !self.type_arguments_can_close() {
            self.cache_type_argument_failure(
                self.speculation_key(),
                TypeArgumentFailures::EXPRESSION,
            );
            return Some(None);
        }

        let checkpoint = self.checkpoint();
        self.type_argument_speculation_depth += 1;
        let parsed = self.parse_type_arguments_in_expression();
        self.type_argument_speculation_depth -= 1;
        let type_arguments = match parsed {
            Some(type_arguments) if can_follow_type_arguments_in_expression(self.current_token) => {
                type_arguments
            }
            Some(_) | None => {
                self.rewind(checkpoint);
                self.lexer.rewrite_last_collected_token(self.current_token);
                // Rewinding restores the opening token and its cache context.
                self.cache_type_argument_failure(
                    self.speculation_key(),
                    TypeArgumentFailures::EXPRESSION,
                );
                return Some(None);
            }
        };
        Some(Some(type_arguments))
    }

    fn has_failed_type_arguments(&self, key: &SpeculationKey, rules: TypeArgumentFailures) -> bool {
        self.failed_type_arguments
            .get(key)
            .is_some_and(|failed| failed.intersects(rules))
    }

    fn cache_type_argument_failure(&mut self, key: SpeculationKey, rule: TypeArgumentFailures) {
        *self.failed_type_arguments.entry(key).or_default() |= rule;
    }

    fn speculation_key(&self) -> SpeculationKey {
        SpeculationKey {
            position: self.current_token.start(),
            context: self.context.bits(),
            ts_context: self.ts_context.bits(),
            boundary_context: self.boundary_context,
            statement_context: self.statement_context as u8,
            module_detection: self.module_detection.speculation_key(),
            lexer_mode: self.lexer.mode as u8,
        }
    }

    fn type_arguments_can_close(&mut self) -> bool {
        if self.lexer.mode != LexerMode::Normal {
            return true;
        }
        let mut lookahead = mem::take(&mut self.type_argument_lookahead);
        let token = self.current_token;
        let jsx = self.lang.is_jsx();
        let mut peek = self.begin_peek();
        let can_close = lookahead.can_close(token, &mut peek, jsx);
        peek.end();
        self.type_argument_lookahead = lookahead;
        can_close
    }

    fn parse_type_assertion(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LessThan);
        let start = self.current_token.start();
        self.advance()?;
        let type_annotation = self.parse_type()?;
        if !self.expect(
            TokenTag::GreaterThan,
            "Expected '>' to close a type assertion",
            Some("A type assertion is written '<Type>expression'"),
        ) {
            return None;
        }
        let expression = self.parse_expr(Precedence::Prefix, ParseExpressionOpts::default())?;
        Some(self.ast.expr_ts_type_assertion(
            Span::new(start, expression.span(&self.ast).end),
            type_annotation,
            expression,
        ))
    }

    fn parse_as_or_satisfies_expression(&mut self, left: Expr, tag: TokenTag) -> Option<Expr> {
        debug_assert!(matches!(tag, TokenTag::As | TokenTag::Satisfies));
        self.advance()?;
        let type_annotation = self.parse_type()?;
        if assertion_erasure_changes_binary_grouping(left, self.current_token.tag(), &self.ast) {
            self.report(
                self.current_token.span(),
                "An unparenthesized `as` or `satisfies` expression cannot be followed by an operator that would bind differently after type erasure",
                ReportOptions {
                    help: Some(Cow::Borrowed("Wrap the expression in parentheses.")),
                    ..ReportOptions::default()
                },
            );
        }
        let span = Span::new(
            left.span(&self.ast).start,
            type_annotation.span(&self.ast).end,
        );
        Some(if tag == TokenTag::As {
            self.ast.expr_ts_as_expression(span, left, type_annotation)
        } else {
            self.ast
                .expr_ts_satisfies_expression(span, left, type_annotation)
        })
    }

    fn parse_non_null_expression(&mut self, left: Expr) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::LogicalNot);
        let end = self.current_token.end();
        self.advance()?;
        Some(
            self.ast
                .expr_ts_non_null_expression(Span::new(left.span(&self.ast).start, end), left),
        )
    }

    // YieldExpression[In, Await] :
    //   `yield`
    //   `yield` [no LineTerminator here] AssignmentExpression[?In, +Yield, ?Await]
    //   `yield` [no LineTerminator here] `*` AssignmentExpression[?In, +Yield, ?Await]
    fn parse_yield_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag(), TokenTag::Yield);
        let start = self.current_token.start();
        self.advance()?;
        let delegate = if self.current_token.tag() == TokenTag::Star
            && !self.current_token.has_line_terminator_before()
        {
            self.advance()?;
            true
        } else {
            false
        };
        let argument = if (!delegate && self.current_token.has_line_terminator_before())
            || matches!(
                self.current_token.tag(),
                TokenTag::Semicolon
                    | TokenTag::RightBrace
                    | TokenTag::RightParen
                    | TokenTag::RightBracket
                    | TokenTag::Colon
                    | TokenTag::Comma
                    | TokenTag::Eof
            ) {
            None
        } else {
            Some(self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?)
        };
        if delegate && argument.is_none() {
            self.report_expected(
                self.current_token.span(),
                "Expected expression after 'yield*'",
                ReportOptions::default(),
            );
        }
        let end = argument.map_or(self.lexer.state.prev_token_end, |expr| {
            expr.span(&self.ast).end
        });
        Some(
            self.ast
                .expr_yield_expression(Span::new(start, end), argument, delegate),
        )
    }

    fn parse_unary_expression(&mut self) -> Option<Expr> {
        debug_assert!(self.current_token.tag().is_unary_operator());
        let token = self.current_token;
        self.advance()?;
        let argument = self.parse_expr(Precedence::Prefix, ParseExpressionOpts::default())?;
        if self.current_token.tag() == TokenTag::Exponent {
            self.report(
                Span::new(token.start(), argument.span(&self.ast).end),
                "The left-hand side of '**' cannot be an unparenthesized unary expression",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add parentheses to make precedence explicit, for example `(-x) ** y` or `-(x ** y)`.",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        let operator = match token.tag() {
            TokenTag::Minus => UnaryOperator::Negate,
            TokenTag::Plus => UnaryOperator::Positive,
            TokenTag::LogicalNot => UnaryOperator::LogicalNot,
            TokenTag::BitwiseNot => UnaryOperator::BitwiseNot,
            TokenTag::Typeof => UnaryOperator::Typeof,
            TokenTag::Void => UnaryOperator::Void,
            TokenTag::Delete => UnaryOperator::Delete,
            _ => unreachable!(),
        };
        Some(self.ast.expr_unary_expression(
            Span::new(token.start(), argument.span(&self.ast).end),
            argument,
            operator,
        ))
    }

    fn parse_update_expression(&mut self, prefix: bool, left: Option<Expr>) -> Option<Expr> {
        let token = self.current_token;
        self.advance()?;
        let argument = if prefix {
            self.parse_expr(Precedence::Prefix, ParseExpressionOpts::default())?
        } else {
            left.expect("a postfix update expression has a left operand")
        };
        let start = if prefix {
            token.start()
        } else {
            argument.span(&self.ast).start
        };
        let unwrapped = self.unwrap_parens(argument);
        if !self.is_simple_assignment_target(unwrapped) {
            self.report(
                argument.span(&self.ast),
                "Invalid operand for increment/decrement operator",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "The '++' and '--' operators require a variable or property reference, not a literal or complex expression.",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let target = self.expression_to_simple_assignment_target(unwrapped)?;
        let operator = if token.tag() == TokenTag::Increment {
            UpdateOperator::Increment
        } else {
            UpdateOperator::Decrement
        };
        let end = if prefix {
            argument.span(&self.ast).end
        } else {
            self.lexer.state.prev_token_end
        };
        Some(
            self.ast
                .expr_update_expression(Span::new(start, end), target, operator, prefix),
        )
    }

    fn parse_binary_expression(&mut self, left: Expr, tag: TokenTag) -> Option<Expr> {
        let start = left.span(&self.ast).start;
        let precedence = tag.precedence();
        self.advance()?;
        let right = self.parse_expr(
            if tag == TokenTag::Exponent {
                precedence
            } else {
                precedence.next()
            },
            ParseExpressionOpts::default(),
        )?;
        let operator = binary_operator(tag);
        Some(self.ast.expr_binary_expression(
            Span::new(start, right.span(&self.ast).end),
            left,
            right,
            operator,
        ))
    }

    fn parse_logical_expression(&mut self, left: Expr, tag: TokenTag) -> Option<Expr> {
        let start = left.span(&self.ast).start;
        let precedence = tag.precedence();
        self.advance()?;
        let right = self.parse_expr(precedence.next(), ParseExpressionOpts::default())?;
        let operator = match tag {
            TokenTag::LogicalAnd => LogicalOperator::And,
            TokenTag::LogicalOr => LogicalOperator::Or,
            TokenTag::NullishCoalescing => LogicalOperator::NullishCoalescing,
            _ => unreachable!(),
        };
        let mixes_coalesce = [left, right].into_iter().any(|expression| {
            let ExprData::LogicalExpression(expression) = self.ast.expr_data(expression) else {
                return false;
            };
            (operator == LogicalOperator::NullishCoalescing)
                != (expression.operator(&self.ast) == LogicalOperator::NullishCoalescing)
        });
        if mixes_coalesce {
            self.report(
                Span::new(start, right.span(&self.ast).end),
                "Logical expressions and nullish coalescing cannot be mixed",
                ReportOptions::default(),
            );
        }
        Some(self.ast.expr_logical_expression(
            Span::new(start, right.span(&self.ast).end),
            left,
            right,
            operator,
        ))
    }

    // AssignmentExpression[In, Yield, Await] :
    //   LeftHandSideExpression[?Yield, ?Await] `=` AssignmentExpression[?In, ?Yield, ?Await]
    //   LeftHandSideExpression[?Yield, ?Await] AssignmentOperator AssignmentExpression[?In, ?Yield, ?Await]
    //   LeftHandSideExpression[?Yield, ?Await] `&&=` AssignmentExpression[?In, ?Yield, ?Await]
    //   LeftHandSideExpression[?Yield, ?Await] `||=` AssignmentExpression[?In, ?Yield, ?Await]
    //   LeftHandSideExpression[?Yield, ?Await] `??=` AssignmentExpression[?In, ?Yield, ?Await]
    fn parse_assignment_expression(&mut self, left: Expr, tag: TokenTag) -> Option<Expr> {
        let start = left.span(&self.ast).start;
        let target = if tag == TokenTag::Assign {
            self.expression_to_assignment_target(left)?
        } else if self.is_simple_assignment_target(left) {
            self.expression_to_assignment_target(self.unwrap_parens(left))?
        } else {
            self.report(
                left.span(&self.ast),
                "Invalid assignment target",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "The left-hand side of a compound assignment must be a variable or property access",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        };
        self.advance()?;
        let right = self.parse_value_expression(Precedence::Assignment)?;
        Some(self.ast.expr_assignment_expression(
            Span::new(start, right.span(&self.ast).end),
            target,
            right,
            assignment_operator(tag),
        ))
    }

    // ConditionalExpression[In, Yield, Await] :
    //   ShortCircuitExpression[?In, ?Yield, ?Await]
    //   ShortCircuitExpression[?In, ?Yield, ?Await] `?` AssignmentExpression[+In, ?Yield, ?Await] `:` AssignmentExpression[?In, ?Yield, ?Await]
    fn parse_conditional_expression(&mut self, test: Expr) -> Option<Expr> {
        let start = test.span(&self.ast).start;
        debug_assert_eq!(self.current_token.tag(), TokenTag::Question);
        self.advance()?;
        let context = enter_grammar_context![&mut self.context, +In];
        let ts_context = self.ts_context;
        self.ts_context.remove(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let consequent = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default());
        self.context = context;
        self.ts_context = ts_context;
        let consequent = consequent?;
        if !self.expect(
            TokenTag::Colon,
            "Expected ':' after conditional expression consequent",
            Some(
                "The ternary operator requires a colon (:) to separate the consequent and alternate expressions.",
            ),
        ) {
            return None;
        }
        let alternate = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        Some(self.ast.expr_conditional_expression(
            Span::new(start, alternate.span(&self.ast).end),
            test,
            consequent,
            alternate,
        ))
    }

    fn parse_sequence_expression(&mut self, precedence: Precedence, left: Expr) -> Option<Expr> {
        let start = left.span(&self.ast).start;
        let checkpoint = self.scratch_a.begin();
        self.scratch_a.append(left);
        let result = (|| {
            let mut end = left.span(&self.ast).end;
            while self.eat(TokenTag::Comma)? {
                let expression =
                    self.parse_expr(precedence.next(), ParseExpressionOpts::default())?;
                end = expression.span(&self.ast).end;
                self.scratch_a.append(expression);
            }
            let expressions = self.add_extra_from_scratch_a(checkpoint);
            Some(
                self.ast
                    .expr_sequence_expression(Span::new(start, end), expressions),
            )
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn expression_to_simple_assignment_target(
        &mut self,
        expression: Expr,
    ) -> Option<SimpleAssignmentTarget> {
        if !self.is_simple_assignment_target(expression) {
            self.report(
                expression.span(&self.ast),
                "Invalid assignment target",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "The left-hand side of a compound assignment must be a variable or property access",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(node) => {
                Some(SimpleAssignmentTarget::IdentifierReference(node))
            }
            ExprData::MemberExpression(node) if !member_expression_is_optional(node, &self.ast) => {
                Some(SimpleAssignmentTarget::MemberExpression(node))
            }
            ExprData::TsAsExpression(node) => {
                self.expression_to_simple_assignment_target(node.expression(&self.ast))?;
                Some(SimpleAssignmentTarget::TsAsExpression(node))
            }
            ExprData::TsSatisfiesExpression(node) => {
                self.expression_to_simple_assignment_target(node.expression(&self.ast))?;
                Some(SimpleAssignmentTarget::TsSatisfiesExpression(node))
            }
            ExprData::TsTypeAssertion(node) => {
                self.expression_to_simple_assignment_target(node.expression(&self.ast))?;
                Some(SimpleAssignmentTarget::TsTypeAssertion(node))
            }
            ExprData::TsNonNullExpression(node) => {
                self.expression_to_simple_assignment_target(node.expression(&self.ast))?;
                Some(SimpleAssignmentTarget::TsNonNullExpression(node))
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.expression_to_simple_assignment_target(parenthesized.expression(&self.ast))
            }
            _ => unreachable!("simple assignment targets were validated above"),
        }
    }

    fn is_simple_assignment_target(&self, expression: Expr) -> bool {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(_) => true,
            ExprData::MemberExpression(member) => !member_expression_is_optional(member, &self.ast),
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.is_simple_assignment_target(parenthesized.expression(&self.ast))
            }
            ExprData::TsNonNullExpression(node) => {
                self.is_simple_assignment_target(node.expression(&self.ast))
            }
            ExprData::TsAsExpression(node) => {
                self.is_simple_assignment_target(node.expression(&self.ast))
            }
            ExprData::TsSatisfiesExpression(node) => {
                self.is_simple_assignment_target(node.expression(&self.ast))
            }
            ExprData::TsTypeAssertion(node) => {
                self.is_simple_assignment_target(node.expression(&self.ast))
            }
            _ => false,
        }
    }

    fn unwrap_parens(&self, mut expression: Expr) -> Expr {
        while let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(expression)
        {
            expression = parenthesized.expression(&self.ast);
        }
        expression
    }

    fn expression_to_assignment_target(&mut self, expression: Expr) -> Option<AssignmentTarget> {
        match self.ast.expr_data(expression) {
            ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_) => {
                let pattern = self.expression_to_assignment_pattern(expression)?;
                match self.ast.binding_pattern_data(pattern) {
                    BindingPatternData::ArrayPattern(pattern) => {
                        Some(AssignmentTarget::ArrayAssignmentTarget(pattern))
                    }
                    BindingPatternData::ObjectPattern(pattern) => {
                        Some(AssignmentTarget::ObjectAssignmentTarget(pattern))
                    }
                    _ => unreachable!("array and object expressions convert to matching patterns"),
                }
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                let inner = self.unwrap_parens(parenthesized.expression(&self.ast));
                if matches!(
                    self.ast.node_kind(inner.node_id()),
                    NodeKind::ArrayExpression | NodeKind::ObjectExpression
                ) {
                    self.report(
                        inner.span(&self.ast),
                        "Parenthesized expression in assignment pattern must be a simple assignment target",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Only identifiers or member expressions (without optional chaining) are allowed inside parentheses in assignment patterns.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    return None;
                }
                let target = self.expression_to_assignment_target(inner)?;
                self.cover.record_parenthesized_target(target.node_id());
                Some(target)
            }
            _ => Some(AssignmentTarget::SimpleAssignmentTarget(
                self.expression_to_simple_assignment_target(expression)?,
            )),
        }
    }

    /// Relax binding-name checks without changing embedded expression grammar.
    fn with_type_binding_context<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Option<T> {
        let ts_context = self.ts_context;
        self.ts_context.insert(TsContext::IN_TYPE_BINDING);
        let result = parse(self);
        self.ts_context = ts_context;
        result
    }

    // INLINE: This is a small and hot function.
    // Combine validation, token advancement and binding construction in the
    // caller, avoiding a helper call and another load of current_token.
    #[inline(always)]
    fn parse_binding_identifier(&mut self) -> Option<BindingIdentifier> {
        self.parse_binding_identifier_with_role(IdentifierRole::Binding)
    }

    #[inline(always)]
    fn parse_type_binding_identifier(&mut self) -> Option<BindingIdentifier> {
        self.parse_binding_identifier_with_role(IdentifierRole::TypeBinding)
    }

    #[inline(always)]
    fn parse_binding_identifier_with_role(
        &mut self,
        role: IdentifierRole,
    ) -> Option<BindingIdentifier> {
        let token = self.consume_identifier(role)?;
        let name = self.identifier_name(token);
        Some(self.ast.binding_identifier(token.span(), name))
    }

    fn parse_label_identifier(&mut self) -> Option<LabelIdentifier> {
        let token = self.consume_identifier(IdentifierRole::Label)?;
        let name = self.identifier_name(token);
        Some(self.ast.label_identifier(token.span(), name))
    }

    /// Diagnose the grammar role, but only stop when a token cannot be a name.
    /// Always consume identifiers without the keyword escape check.
    #[inline(always)]
    fn consume_identifier(&mut self, role: IdentifierRole) -> Option<Token> {
        let token = self.current_token;
        self.validate_identifier(token, role);
        if !token.tag().is_identifier_like() {
            return None;
        }
        self.advance_without_escape_check()?;
        Some(token)
    }

    #[inline]
    fn validate_identifier(&mut self, token: Token, role: IdentifierRole) {
        if token.tag() == TokenTag::Identifier
            || role == IdentifierRole::Name && token.tag().is_identifier_like()
        {
            return;
        }

        self.validate_identifier_slow(token, role);
    }

    #[cold]
    #[inline(never)]
    fn validate_identifier_slow(&mut self, token: Token, role: IdentifierRole) {
        let as_what = role.description();
        if !token.tag().is_identifier_like() {
            self.report_expected(
                token.span(),
                "Expected an identifier",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Identifiers must start with a letter, underscore (_), or dollar sign ($)",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        if role == IdentifierRole::Name {
            return;
        }
        if token.tag().is_unconditionally_reserved() {
            let found = self.describe_token(token);
            self.report(
                token.span(),
                Cow::Owned(format!(
                    "'{found}' is reserved and cannot be used as {as_what}"
                )),
                ReportOptions::default(),
            );
        }
        if role == IdentifierRole::TypeScript {
            return;
        }
        let in_type_binding =
            role == IdentifierRole::TypeBinding || self.ts_context.in_type_binding();
        if token.tag() == TokenTag::Yield
            && !in_type_binding
            && self.context.contains(Context::YIELD)
        {
            self.report(
                token.span(),
                Cow::Owned(format!(
                    "'yield' is reserved in a generator context and cannot be used as {as_what}"
                )),
                ReportOptions::default(),
            );
        }
        if token.tag() == TokenTag::Await {
            let await_is_reserved = !in_type_binding && self.context.contains(Context::AWAIT)
                || self.source_type == SourceType::Module
                    && !self.ts_context.in_ts_container_body();
            if !in_type_binding
                && !await_is_reserved
                && self
                    .boundary_context
                    .contains(BoundaryContext::ASYNC_ARROW_HEAD)
            {
                self.await_identifier_in_async_head
                    .get_or_insert(token.span());
            }
            if !self.ts_context.ambient() && await_is_reserved {
                self.report_reserved_await(token.span(), as_what);
            }
        }
    }

    #[cold]
    fn report_reserved_await(&mut self, span: Span, as_what: &str) {
        self.report(
            span,
            Cow::Owned(format!(
                "'await' is reserved in an async/module context and cannot be used as {as_what}"
            )),
            ReportOptions::default(),
        );
    }

    #[inline]
    fn next_token(&mut self) -> Option<()> {
        match self.lexer.next_token(&mut self.current_token) {
            Ok(()) => Some(()),
            Err(error) => self.next_token_error(error),
        }
    }

    #[cold]
    #[inline(never)]
    fn next_token_error(&mut self, error: LexicalError) -> Option<()> {
        let current_token_end = self.current_token.end();
        let lexer_cursor = self.lexer.cursor;
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            message: Cow::Borrowed(lexical_error_message(error)),
            span: Span::new(
                current_token_end.min(lexer_cursor),
                current_token_end.max(lexer_cursor),
            ),
            help: Some(Cow::Borrowed(lexical_error_help(error))),
            labels: Vec::new(),
        });
        None
    }

    /// Advances to the next token.
    ///
    /// Reports an error when an escaped keyword is consumed in a keyword
    /// position.
    #[inline]
    fn advance(&mut self) -> Option<()> {
        self.check_escaped_keyword();
        self.lexer.state.prev_token_end = self.current_token.end();
        if self.lexer.try_next_token(&mut self.current_token) {
            Some(())
        } else {
            self.next_token()
        }
    }

    /// Advance without reporting lexical errors, restoring lexer state on failure.
    fn try_advance(&mut self) -> bool {
        let previous_end = self.current_token.end();
        let peek = self.begin_peek();
        peek.lexer.state.prev_token_end = previous_end;
        let mut next = Token::eof(peek.lexer.cursor);
        if !peek.lexer.try_next_token(&mut next) && peek.lexer.next_token(&mut next).is_err() {
            return false;
        }
        peek.commit();
        self.check_escaped_keyword();
        self.current_token = next;
        true
    }

    #[inline]
    fn eat(&mut self, tag: TokenTag) -> Option<bool> {
        let matched = self.current_token.tag() == tag;
        if matched {
            self.advance()?;
        }
        Some(matched)
    }

    /// Advances without checking whether the current token is an escaped
    /// keyword.
    #[inline]
    fn advance_without_escape_check(&mut self) -> Option<()> {
        self.lexer.state.prev_token_end = self.current_token.end();
        if self.lexer.try_next_token(&mut self.current_token) {
            Some(())
        } else {
            self.next_token()
        }
    }

    #[inline]
    fn check_escaped_keyword(&mut self) {
        let current_token = self.current_token;
        if !current_token.is_escaped() {
            return;
        }
        if current_token.tag().is_keyword() {
            self.report_escaped_keyword(current_token.span());
        }
    }

    #[cold]
    #[inline(never)]
    fn report_escaped_keyword(&mut self, span: Span) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            message: Cow::Borrowed("Keywords cannot contain escape characters"),
            span,
            help: Some(Cow::Borrowed("Remove the escape characters")),
            labels: Vec::new(),
        });
    }

    /// Reports an escaped-keyword error if `token` has the escaped flag.
    ///
    /// Use this for deferred checks where the token was consumed before its
    /// syntactic role was known.
    fn report_if_escaped_keyword(&mut self, token: Token) {
        if token.is_escaped() {
            self.report_escaped_keyword(token.span());
        }
    }

    /// Sets the current token to a rescanned token and advances.
    ///
    /// Use this when retokenizing the current source range, such as for
    /// regular expressions and template continuations.
    #[inline]
    fn advance_with_rescanned_token(&mut self, mut token: Token) -> Option<()> {
        if self.lexer.config.tokens() {
            token.set_flags(
                token.flags()
                    | (self.current_token.flags() & flag_mask(TokenFlag::LineTerminatorBefore)),
            );
        }
        self.lexer.handle_token(token, HandleTokenMode::Replace);
        self.current_token = token;
        self.advance()
    }

    /// Retokenizes the current token in the current lexer mode.
    fn re_scan_current(&mut self) -> Option<()> {
        let line_terminator_before =
            self.current_token.flags() & flag_mask(TokenFlag::LineTerminatorBefore);
        self.lexer.rewind_to(self.current_token.start());
        if self.lexer.config.tokens() {
            self.lexer.tokens.pop();
        }
        self.next_token()?;
        if self.lexer.config.tokens() {
            self.current_token
                .set_flags(self.current_token.flags() | line_terminator_before);
            self.lexer.rewrite_last_collected_token(self.current_token);
        }
        Some(())
    }

    fn expect(&mut self, tag: TokenTag, message: &'static str, help: Option<&'static str>) -> bool {
        if self.current_token.tag() == tag {
            return self.advance().is_some();
        }

        self.report_expected(
            self.current_token.span(),
            message,
            ReportOptions {
                help: help.map(Cow::Borrowed),
                ..ReportOptions::default()
            },
        );
        false
    }

    /// Continue a committed grammar alternative after a missing token, while
    /// still propagating lexical failures when consuming a present token.
    fn expect_recoverable(
        &mut self,
        tag: TokenTag,
        message: &'static str,
        help: Option<&'static str>,
    ) -> Option<()> {
        if !self.eat(tag)? {
            self.report_expected(
                self.current_token.span(),
                message,
                ReportOptions {
                    help: help.map(Cow::Borrowed),
                    ..ReportOptions::default()
                },
            );
        }
        Some(())
    }

    /// Implements the ECMAScript
    /// [automatic semicolon insertion rules](https://tc39.es/ecma262/#sec-rules-of-automatic-semicolon-insertion).
    fn can_insert_implicit_semicolon(&self, token: Token) -> bool {
        token.tag() == TokenTag::Eof
            || token.has_line_terminator_before()
            || token.tag() == TokenTag::RightBrace
    }

    fn eat_semicolon(&mut self, end: u32) -> Option<u32> {
        if self.current_token.tag() == TokenTag::Semicolon {
            let semicolon_end = self.current_token.end();
            self.advance()?;
            return Some(semicolon_end);
        }
        if !self.can_insert_implicit_semicolon(self.current_token) {
            self.report_expected(
                self.current_token.span(),
                "Expected a semicolon or an implicit semicolon after a statement",
                ReportOptions {
                    help: Some(Cow::Borrowed("Try inserting a semicolon here")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        Some(end)
    }

    #[cold]
    #[inline(never)]
    fn report(
        &mut self,
        span: Span,
        message: impl Into<Cow<'static, str>>,
        options: ReportOptions,
    ) {
        self.diagnostics.push(Diagnostic {
            severity: options.severity,
            message: message.into(),
            span,
            help: options.help,
            labels: options.labels,
        });
    }

    #[cold]
    #[inline(never)]
    fn report_expected(&mut self, span: Span, message: &str, options: ReportOptions) {
        let found = self.describe_token(self.current_token);
        self.report(
            span,
            Cow::Owned(format!("{message}, but found '{found}'")),
            options,
        );
    }

    fn label(&self, span: Span, message: &'static str) -> Label<'static> {
        Label {
            span,
            message: Cow::Borrowed(message),
        }
    }
}

fn member_expression_is_optional(member: MemberExpression, ast: &Ast<'_>) -> bool {
    if member.optional(ast) {
        return true;
    }
    match ast.expr_data(member.object(ast)) {
        ExprData::MemberExpression(object) => member_expression_is_optional(object, ast),
        ExprData::CallExpression(call) => call.optional(ast),
        ExprData::ChainExpression(_) => true,
        _ => false,
    }
}

fn is_import_expression_or_member_access_on_import_expression(
    mut expression: Expr,
    ast: &Ast<'_>,
) -> bool {
    loop {
        expression = match ast.expr_data(expression) {
            ExprData::ImportExpression(_) => return true,
            ExprData::MemberExpression(member) => member.object(ast),
            ExprData::TaggedTemplateExpression(tagged) => tagged.tag(ast),
            ExprData::TsNonNullExpression(non_null) => non_null.expression(ast),
            _ => return false,
        };
    }
}

#[inline]
fn max_left_precedence(expression: Expr, ast: &Ast<'_>) -> Precedence {
    match ast.node_kind(expression.node_id()) {
        // Their right-hand sides already consumed every tighter continuation.
        NodeKind::ArrowFunctionExpression
        | NodeKind::YieldExpression
        | NodeKind::ConditionalExpression
        | NodeKind::AssignmentExpression
        | NodeKind::SequenceExpression => Precedence::Comma,
        NodeKind::UpdateExpression
        | NodeKind::UnaryExpression
        | NodeKind::AwaitExpression
        | NodeKind::BinaryExpression
        | NodeKind::LogicalExpression
        | NodeKind::TsAsExpression
        | NodeKind::TsSatisfiesExpression => Precedence::Prefix,
        _ => Precedence::Primary,
    }
}

#[inline]
fn assertion_erasure_changes_binary_grouping(
    mut expression: Expr,
    next_operator: TokenTag,
    ast: &Ast<'_>,
) -> bool {
    // Only the assertion before an operator needs to inspect the chain.
    if !next_operator.is_binary_operator() && !next_operator.is_logical_operator() {
        return false;
    }

    loop {
        expression = match ast.expr_data(expression) {
            ExprData::TsAsExpression(assertion) => assertion.expression(ast),
            ExprData::TsSatisfiesExpression(assertion) => assertion.expression(ast),
            _ => break,
        };
    }

    let current_precedence = match ast.expr_data(expression) {
        ExprData::BinaryExpression(binary) => binary.operator(ast).precedence(),
        ExprData::LogicalExpression(logical) => logical.operator(ast).precedence(),
        _ => return false,
    };
    let next_precedence = next_operator.precedence();
    next_precedence > current_precedence
        || (next_precedence == current_precedence && next_operator == TokenTag::Exponent)
}

#[inline]
fn is_angle_open(tag: TokenTag) -> bool {
    matches!(tag, TokenTag::LessThan | TokenTag::LeftShift)
}

#[inline]
fn is_angle_close(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::GreaterThan
            | TokenTag::RightShift
            | TokenTag::UnsignedRightShift
            | TokenTag::GreaterThanEqual
            | TokenTag::RightShiftAssign
            | TokenTag::UnsignedRightShiftAssign
    )
}

#[inline]
fn is_start_of_type(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::Function
            | TokenTag::Any
            | TokenTag::Bigint
            | TokenTag::Boolean
            | TokenTag::Never
            | TokenTag::NullLiteral
            | TokenTag::Number
            | TokenTag::Object
            | TokenTag::String
            | TokenTag::Symbol
            | TokenTag::This
            | TokenTag::Undefined
            | TokenTag::Unknown
            | TokenTag::Void
            | TokenTag::True
            | TokenTag::False
            | TokenTag::StringLiteral
            | TokenTag::NumericLiteral
            | TokenTag::HexLiteral
            | TokenTag::OctalLiteral
            | TokenTag::BinaryLiteral
            | TokenTag::BigIntLiteral
            | TokenTag::NoSubstitutionTemplate
            | TokenTag::TemplateHead
            | TokenTag::Minus
            | TokenTag::LeftParen
            | TokenTag::LeftBracket
            | TokenTag::LeftBrace
            | TokenTag::LessThan
            | TokenTag::LeftShift
            | TokenTag::Keyof
            | TokenTag::Unique
            | TokenTag::Readonly
            | TokenTag::Infer
            | TokenTag::Typeof
            | TokenTag::Import
            | TokenTag::New
            | TokenTag::Abstract
            | TokenTag::Asserts
            | TokenTag::Question
            | TokenTag::LogicalNot
            | TokenTag::Spread
            | TokenTag::BitwiseAnd
            | TokenTag::BitwiseOr
    ) || tag.is_identifier_candidate()
}

/// A candidate for recovery, never proof that a grammar alternative is valid.
/// Committed entries still diagnose reserved names through their start check.
#[inline]
fn can_parse_type_for_recovery(tag: TokenTag) -> bool {
    is_start_of_type(tag) || tag.is_identifier_like()
}

#[inline]
fn can_follow_accessor_keyword(tag: TokenTag) -> bool {
    tag == TokenTag::LeftBracket
        || tag.is_identifier_like()
        || tag == TokenTag::StringLiteral
        || tag.is_numeric_literal()
}

#[inline]
fn can_follow_readonly_modifier(tag: TokenTag) -> bool {
    can_follow_accessor_keyword(tag)
        || matches!(tag, TokenTag::LeftBrace | TokenTag::Star | TokenTag::Spread)
}

#[inline]
fn can_start_binding_identifier(tag: TokenTag) -> bool {
    tag.is_identifier_candidate()
}

#[inline]
fn can_start_binding_pattern(tag: TokenTag) -> bool {
    matches!(tag, TokenTag::LeftBracket | TokenTag::LeftBrace) || tag.is_identifier_candidate()
}

#[inline]
fn can_start_using_binding(tag: TokenTag) -> bool {
    // Object patterns can be recovered as declarations. '[' must still allow
    // a member expression with the identifier 'using' as its object.
    tag == TokenTag::LeftBrace || can_start_binding_identifier(tag)
}

#[inline]
fn is_parameter_property_modifier_tag(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::Public
            | TokenTag::Private
            | TokenTag::Protected
            | TokenTag::Readonly
            | TokenTag::Override
    )
}

#[inline]
fn is_modifier(tag: TokenTag, is_ts: bool) -> bool {
    match tag {
        TokenTag::Static | TokenTag::Async | TokenTag::Get | TokenTag::Set | TokenTag::Accessor => {
            true
        }
        TokenTag::Declare
        | TokenTag::Public
        | TokenTag::Private
        | TokenTag::Protected
        | TokenTag::Override
        | TokenTag::Readonly
        | TokenTag::Abstract => is_ts,
        _ => false,
    }
}

#[inline]
fn can_start_element_key(tag: TokenTag) -> bool {
    tag.is_identifier_like()
        || tag.is_numeric_literal()
        || matches!(
            tag,
            TokenTag::StringLiteral
                | TokenTag::PrivateIdentifier
                | TokenTag::LeftBracket
                | TokenTag::Star
        )
}

#[inline]
fn requires_same_line(tag: TokenTag) -> bool {
    !matches!(tag, TokenTag::Static | TokenTag::Get | TokenTag::Set)
}

#[inline]
fn is_const_enum_head(after_const: Token) -> bool {
    after_const.tag() == TokenTag::Enum
}

#[inline]
fn can_follow_parameter_property_modifier(tag: TokenTag) -> bool {
    tag.is_identifier_like()
        || matches!(
            tag,
            TokenTag::LeftBracket | TokenTag::LeftBrace | TokenTag::Spread
        )
}

#[inline]
fn is_declaration_name(token: Token) -> bool {
    token.tag().is_identifier_like()
        && !token.tag().is_unconditionally_reserved()
        && !token.has_line_terminator_before()
}

#[inline]
fn import_or_export_kind_for_declare(declare: bool) -> ImportOrExportKind {
    if declare {
        ImportOrExportKind::Type
    } else {
        ImportOrExportKind::Value
    }
}

#[inline]
fn can_follow_type_arguments_in_expression(token: Token) -> bool {
    match token.tag() {
        TokenTag::LeftParen | TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => true,
        TokenTag::LessThan | TokenTag::GreaterThan | TokenTag::Plus | TokenTag::Minus => false,
        _ => {
            token.has_line_terminator_before()
                || is_binary_operator_like(token.tag())
                || !is_start_of_expression(token.tag())
        }
    }
}

#[inline]
fn is_binary_operator_like(tag: TokenTag) -> bool {
    tag.is_binary_operator()
        || tag.is_logical_operator()
        || tag.is_assignment_operator()
        || matches!(
            tag,
            TokenTag::Comma | TokenTag::Question | TokenTag::As | TokenTag::Satisfies
        )
}

#[inline]
fn is_start_of_expression(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::This
            | TokenTag::Super
            | TokenTag::True
            | TokenTag::False
            | TokenTag::NullLiteral
            | TokenTag::Function
            | TokenTag::Class
            | TokenTag::New
            | TokenTag::Import
            | TokenTag::Identifier
            | TokenTag::PrivateIdentifier
            | TokenTag::StringLiteral
            | TokenTag::NoSubstitutionTemplate
            | TokenTag::TemplateHead
            | TokenTag::RegexLiteral
            | TokenTag::LeftParen
            | TokenTag::LeftBracket
            | TokenTag::LeftBrace
            | TokenTag::Plus
            | TokenTag::Minus
            | TokenTag::LogicalNot
            | TokenTag::BitwiseNot
            | TokenTag::Increment
            | TokenTag::Decrement
            | TokenTag::Spread
            | TokenTag::At
            | TokenTag::Slash
            | TokenTag::SlashAssign
    ) || tag.is_identifier_candidate()
        || tag.is_unary_operator()
        || tag.is_numeric_literal()
}

#[inline]
fn binary_operator(tag: TokenTag) -> BinaryOperator {
    match tag {
        TokenTag::Equal => BinaryOperator::Equal,
        TokenTag::NotEqual => BinaryOperator::NotEqual,
        TokenTag::StrictEqual => BinaryOperator::StrictEqual,
        TokenTag::StrictNotEqual => BinaryOperator::StrictNotEqual,
        TokenTag::LessThan => BinaryOperator::LessThan,
        TokenTag::LessThanEqual => BinaryOperator::LessThanOrEqual,
        TokenTag::GreaterThan => BinaryOperator::GreaterThan,
        TokenTag::GreaterThanEqual => BinaryOperator::GreaterThanOrEqual,
        TokenTag::Plus => BinaryOperator::Add,
        TokenTag::Minus => BinaryOperator::Subtract,
        TokenTag::Star => BinaryOperator::Multiply,
        TokenTag::Slash => BinaryOperator::Divide,
        TokenTag::Percent => BinaryOperator::Modulo,
        TokenTag::Exponent => BinaryOperator::Exponent,
        TokenTag::BitwiseOr => BinaryOperator::BitwiseOr,
        TokenTag::BitwiseXor => BinaryOperator::BitwiseXor,
        TokenTag::BitwiseAnd => BinaryOperator::BitwiseAnd,
        TokenTag::LeftShift => BinaryOperator::LeftShift,
        TokenTag::RightShift => BinaryOperator::RightShift,
        TokenTag::UnsignedRightShift => BinaryOperator::UnsignedRightShift,
        TokenTag::In => BinaryOperator::In,
        TokenTag::Instanceof => BinaryOperator::Instanceof,
        _ => unreachable!(),
    }
}

#[inline]
fn assignment_operator(tag: TokenTag) -> AssignmentOperator {
    match tag {
        TokenTag::Assign => AssignmentOperator::Assign,
        TokenTag::PlusAssign => AssignmentOperator::AddAssign,
        TokenTag::MinusAssign => AssignmentOperator::SubtractAssign,
        TokenTag::StarAssign => AssignmentOperator::MultiplyAssign,
        TokenTag::SlashAssign => AssignmentOperator::DivideAssign,
        TokenTag::PercentAssign => AssignmentOperator::ModuloAssign,
        TokenTag::ExponentAssign => AssignmentOperator::ExponentAssign,
        TokenTag::LeftShiftAssign => AssignmentOperator::LeftShiftAssign,
        TokenTag::RightShiftAssign => AssignmentOperator::RightShiftAssign,
        TokenTag::UnsignedRightShiftAssign => AssignmentOperator::UnsignedRightShiftAssign,
        TokenTag::BitwiseOrAssign => AssignmentOperator::BitwiseOrAssign,
        TokenTag::BitwiseXorAssign => AssignmentOperator::BitwiseXorAssign,
        TokenTag::BitwiseAndAssign => AssignmentOperator::BitwiseAndAssign,
        TokenTag::LogicalOrAssign => AssignmentOperator::LogicalOrAssign,
        TokenTag::LogicalAndAssign => AssignmentOperator::LogicalAndAssign,
        TokenTag::NullishAssign => AssignmentOperator::NullishAssign,
        _ => unreachable!(),
    }
}

fn strip_parenthesized_nodes(ast: &mut Ast<'_>, comments: &mut [Comment]) {
    let mut trailing_comment_index = 0;
    for index in 0..ast.node_count() {
        let node = NodeId::from_raw_unchecked(index as u32);
        let replacement = match ast.kind_data(node) {
            NodeKindData::ParenthesizedExpression(parenthesized) => {
                let mut inner = parenthesized.expression(ast);
                // Resolve nested wrappers to the innermost non-parenthesized node.
                while let ExprData::ParenthesizedExpression(parenthesized) = ast.expr_data(inner) {
                    inner = parenthesized.expression(ast);
                }
                inner.node_id()
            }
            NodeKindData::TsParenthesizedType(parenthesized) => {
                let mut inner = parenthesized.type_annotation(ast);
                // Resolve nested wrappers to the innermost non-parenthesized node.
                while let TsTypeData::TsParenthesizedType(parenthesized) = ast.ts_type_data(inner) {
                    inner = parenthesized.type_annotation(ast);
                }
                inner.node_id()
            }
            _ => continue,
        };
        let parenthesized_span = ast.span(node);
        let replacement_span = ast.span(replacement);

        let leading_from = parenthesized_span.start;
        let leading_comment_index =
            comments.partition_point(|comment| comment.attached_to <= leading_from);
        for comment in comments[..leading_comment_index].iter_mut().rev() {
            if comment.position != CommentPosition::Leading || comment.attached_to != leading_from {
                break;
            }
            comment.attached_to = replacement_span.start;
        }

        let trailing_from = parenthesized_span.end;
        trailing_comment_index += comments[trailing_comment_index..]
            .partition_point(|comment| comment.attached_to < trailing_from);
        for comment in &mut comments[trailing_comment_index..] {
            if comment.position != CommentPosition::Trailing || comment.attached_to != trailing_from
            {
                break;
            }
            comment.attached_to = replacement_span.end;
        }

        // SAFETY: parsing has not exposed any concrete handles yet. This pass
        // retires the parenthesized node, and all expression or type edges that
        // refer to it accept the corresponding replacement kind.
        unsafe { ast.set_data(node, replacement) };
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use super::*;
    use crate::TokenParserConfig;

    fn parse_tokens<'a>(allocator: &'a Allocator, source: &'a str) -> ParseReturn<'a> {
        let options = Options {
            lang: Lang::Ts,
            source_type: SourceType::Module,
            ..Options::default()
        };
        Parser::init(allocator, source, options, TokenParserConfig).parse()
    }

    fn token_texts<'a>(source: &'a str, tokens: &[Token]) -> Vec<&'a str> {
        tokens
            .iter()
            .map(|token| &source[token.start() as usize..token.end() as usize])
            .collect()
    }

    fn error_spans(source: &str, options: Options) -> Vec<&str> {
        let allocator = Allocator::new();
        let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        parsed
            .diagnostics
            .iter()
            .map(|error| &source[error.span.start as usize..error.span.end as usize])
            .collect()
    }

    #[test]
    fn token_lookahead() {
        let allocator = Allocator::new();
        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let parser = |source| {
            let mut parser = Parser::init(&allocator, source, options, TokenParserConfig);
            parser.advance().unwrap();
            parser
        };

        let mut parsed = parser("head /* note */ next");
        assert!(parsed.try_advance());
        assert_eq!(
            token_texts(parsed.source, &parsed.lexer.tokens),
            ["head", "next"]
        );
        assert_eq!(parsed.lexer.comments.len(), 1);
        assert_eq!(parsed.lexer.comments[0].position, CommentPosition::Trailing);
        assert_eq!(parsed.lexer.comments[0].attached_to, 4);

        let mut parsed = parser("head /*");
        assert!(!parsed.try_advance());
        assert_eq!(parsed.current_token.span(), Span::new(0, 4));
        assert_eq!(token_texts(parsed.source, &parsed.lexer.tokens), ["head"]);
        assert!(parsed.lexer.comments.is_empty());
        assert!(parsed.diagnostics.entries.is_empty());
        assert!(parsed.advance().is_none());
        assert!(!parsed.diagnostics.entries.is_empty());
    }

    #[test]
    fn type_entry_recovery() {
        fn alias(ast: &Ast<'_>) -> TsType {
            ast.root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap()
                .as_ts_type_alias_declaration(ast)
                .unwrap()
                .type_annotation(ast)
        }

        fn reference_name<'a>(ast: &'a Ast<'_>, ty: TsType) -> &'a str {
            let reference = ty.as_ts_type_reference(ast).unwrap();
            let name = reference
                .type_name(ast)
                .as_identifier_reference(ast)
                .unwrap();
            ast.get_utf8(name.name(ast))
        }

        fn nullable(ast: &Ast<'_>, ty: TsType) -> TsType {
            ty.as_ts_js_doc_nullable_type(ast)
                .unwrap()
                .type_annotation(ast)
        }

        fn return_type(ast: &Ast<'_>, ty: TsType) -> TsType {
            ty.as_ts_function_type(ast)
                .unwrap()
                .return_type(ast)
                .type_annotation(ast)
        }

        fn argument(ast: &Ast<'_>) -> TsType {
            alias(ast)
                .as_ts_type_reference(ast)
                .unwrap()
                .type_arguments(ast)
                .unwrap()
                .params(ast)
                .get_node(ast, 0)
                .unwrap()
        }

        fn member(ast: &Ast<'_>) -> TsNamedTupleMember {
            alias(ast)
                .as_ts_tuple_type(ast)
                .unwrap()
                .element_types(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_ts_named_tuple_member(ast)
                .unwrap()
        }

        let allocator = Allocator::new();
        let parse = |source| {
            let parsed = parse_tokens(&allocator, source);
            (parsed.ast, !parsed.diagnostics.is_empty())
        };
        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let parse_type = |source| {
            let mut parser = Parser::init(&allocator, source, options, NoTokenParserConfig);
            parser.advance().unwrap();
            let ty = parser.parse_type().unwrap();
            let kind = parser.ast.node_kind(ty.node_id());
            (kind, parser.current_token.tag())
        };

        let (ast, errors) = parse("type T = A<if>; after;");
        assert!(errors);
        assert_eq!(ast.root_program().body(&ast).len(), 2);
        assert_eq!(reference_name(&ast, argument(&ast)), "if");

        let (ast, errors) = parse("type T = 1 extends 2 ? break : for;");
        assert!(errors);
        let conditional = alias(&ast).as_ts_conditional_type(&ast).unwrap();
        assert_eq!(reference_name(&ast, conditional.true_type(&ast)), "break");
        assert_eq!(reference_name(&ast, conditional.false_type(&ast)), "for");

        let (ast, errors) = parse("type T = A extends () => B ? break : C;");
        assert!(errors);
        let conditional = alias(&ast).as_ts_conditional_type(&ast).unwrap();
        let returned = return_type(&ast, conditional.extends_type(&ast));
        assert_eq!(reference_name(&ast, nullable(&ast, returned)), "B");
        assert_eq!(reference_name(&ast, conditional.true_type(&ast)), "break");
        assert_eq!(reference_name(&ast, conditional.false_type(&ast)), "C");

        let (ast, errors) = parse("type T = A extends infer U extends B ? break : C;");
        assert!(errors);
        let conditional = alias(&ast).as_ts_conditional_type(&ast).unwrap();
        let inferred = conditional
            .extends_type(&ast)
            .as_ts_infer_type(&ast)
            .unwrap();
        let constraint = inferred.type_parameter(&ast).constraint(&ast).unwrap();
        assert_eq!(reference_name(&ast, nullable(&ast, constraint)), "B");
        assert_eq!(reference_name(&ast, conditional.true_type(&ast)), "break");
        assert_eq!(reference_name(&ast, conditional.false_type(&ast)), "C");

        let (ast, errors) = parse("type T = A extends () => B? extends C ? D : E ? F : G;");
        assert!(!errors);
        let conditional = alias(&ast).as_ts_conditional_type(&ast).unwrap();
        let returned = return_type(&ast, conditional.extends_type(&ast));
        let nested = returned.as_ts_conditional_type(&ast).unwrap();
        assert_eq!(
            reference_name(&ast, nullable(&ast, nested.check_type(&ast))),
            "B"
        );
        assert_eq!(reference_name(&ast, nested.extends_type(&ast)), "C");
        assert_eq!(reference_name(&ast, nested.true_type(&ast)), "D");
        assert_eq!(reference_name(&ast, nested.false_type(&ast)), "E");
        assert_eq!(reference_name(&ast, conditional.true_type(&ast)), "F");
        assert_eq!(reference_name(&ast, conditional.false_type(&ast)), "G");

        let (ast, errors) = parse("type T = A extends Box<B extends C ? D : E> ? break : F;");
        assert!(errors);
        let conditional = alias(&ast).as_ts_conditional_type(&ast).unwrap();
        let reference = nullable(&ast, conditional.extends_type(&ast))
            .as_ts_type_reference(&ast)
            .unwrap();
        let argument = reference
            .type_arguments(&ast)
            .unwrap()
            .params(&ast)
            .get_node(&ast, 0)
            .unwrap();
        assert!(argument.is_ts_conditional_type(&ast));
        assert_eq!(reference_name(&ast, conditional.true_type(&ast)), "break");
        assert_eq!(reference_name(&ast, conditional.false_type(&ast)), "F");

        let (ast, errors) = parse("type T = A extends B C : D; after;");
        assert!(errors);
        assert_eq!(ast.root_program().body(&ast).len(), 2);
        let conditional = alias(&ast).as_ts_conditional_type(&ast).unwrap();
        assert_eq!(reference_name(&ast, conditional.extends_type(&ast)), "B");
        assert_eq!(reference_name(&ast, conditional.true_type(&ast)), "C");
        assert_eq!(reference_name(&ast, conditional.false_type(&ast)), "D");

        let (ast, errors) = parse("type T = A extends B ? C D; after;");
        assert!(errors);
        assert_eq!(ast.root_program().body(&ast).len(), 2);
        let conditional = alias(&ast).as_ts_conditional_type(&ast).unwrap();
        assert_eq!(reference_name(&ast, conditional.true_type(&ast)), "C");
        assert_eq!(reference_name(&ast, conditional.false_type(&ast)), "D");

        let (ast, errors) = parse("type T = [br\\u0065ak?: number];");
        assert!(errors);
        let member = member(&ast);
        assert_eq!(ast.get_utf8(member.label(&ast).name(&ast)), "break");
        assert!(member.optional(&ast));
        assert!(member.element_type(&ast).is_ts_number_keyword(&ast));

        assert!(has_errors("type T = [for];", options));

        assert!(has_errors("type T = A[while];", options));

        assert!(!has_errors("type T = [...break: number[]];", options));

        assert!(!has_errors("type T = [null: number];", options));

        assert!(!has_errors("type T = A<function>;", options));

        assert!(!has_errors("type T = for;", options));

        let (kind, next) = parse_type("?for");
        assert_eq!(kind, NodeKind::TsJsDocNullableType);
        assert_eq!(next, TokenTag::Eof);
    }

    #[test]
    fn typescript_identifier_recovery() {
        fn namespace_name(source: &str) -> (bool, String) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let declaration = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_ts_namespace_export_declaration(ast)
                .unwrap();
            (
                !parsed.diagnostics.is_empty(),
                ast.get_utf8(declaration.id(ast).name(ast)).to_owned(),
            )
        }

        fn import_name(source: &str) -> (bool, String) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let declaration = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap()
                .as_ts_import_equals_declaration(ast)
                .unwrap();
            let name = declaration
                .module_reference(ast)
                .as_ts_type_name(ast)
                .unwrap()
                .as_ts_qualified_name(ast)
                .unwrap();
            (
                !parsed.diagnostics.is_empty(),
                ast.get_utf8(name.right(ast).name(ast)).to_owned(),
            )
        }

        let (errors, name) = namespace_name("export as namespace if;");
        assert!(errors);
        assert_eq!(name, "if");

        let (errors, name) = namespace_name("export as namespace a\\u0077ait;");
        assert!(!errors);
        assert_eq!(name, "await");

        let (errors, name) = import_name("import X = A.if;");
        assert!(errors);
        assert_eq!(name, "if");

        let (errors, name) = import_name("import X = A.await;");
        assert!(!errors);
        assert_eq!(name, "await");

        let (errors, name) = import_name("import X = if.A;");
        assert!(errors);
        assert_eq!(name, "A");

        let allocator = Allocator::new();
        let parsed = parse_tokens(&allocator, "export as namespace 1;");
        assert!(!parsed.diagnostics.is_empty());
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());

        let parsed = parse_tokens(&allocator, "type T = A.if;");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn type_binding_contexts() {
        let script = Options {
            lang: Lang::Ts,
            source_type: SourceType::Script,
            ..Options::default()
        };
        let module = Options {
            source_type: SourceType::Module,
            ..script
        };
        let rejects = |source| has_errors(source, script);

        let errors = rejects("async function f() { type T = 0 extends infer await ? 0 : 0; }");
        assert!(!errors);

        let errors = rejects("function* f() { type T = 0 extends infer yield ? 0 : 0; }");
        assert!(!errors);

        let errors = rejects("async function f() { type T = (await: number) => void; }");
        assert!(!errors);

        let errors = rejects("async function f() { type T = ({[await x]: await}: any) => void; }");
        assert!(!errors);

        let errors = rejects("async function f() { type T = { [await in K]: 0 }; }");
        assert!(!errors);

        let errors = rejects("async function f() { type T = { [await: string]: number }; }");
        assert!(!errors);

        let errors = rejects("async function f() { type T = { m<await>(): void }; }");
        assert!(!errors);

        let errors = rejects("async function f() { interface I { m<await>(): void } }");
        assert!(errors);

        let errors = rejects("async function f() { type T = { [await x]: number }; }");
        assert!(!errors);

        let errors = rejects("async function f() { type T<await> = 0; }");
        assert!(errors);

        let errors = has_errors("type T = 0 extends infer await ? 0 : 0;", module);
        assert!(errors);
    }

    #[test]
    fn type_body_tokens() {
        fn parse(source: &str) -> (&str, String, Vec<&str>) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            assert!(parsed.diagnostics.is_empty());
            let ast = &parsed.ast;
            let declaration = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap();
            let span = if let Some(alias) = declaration.as_ts_type_alias_declaration(ast) {
                alias.type_annotation(ast).span(ast)
            } else {
                declaration
                    .as_ts_interface_declaration(ast)
                    .unwrap()
                    .body(ast)
                    .span(ast)
            };
            let comments = ast
                .comments()
                .iter()
                .map(|comment| &source[comment.span.start as usize..comment.span.end as usize])
                .collect();
            (
                &source[span.start as usize..span.end as usize],
                token_texts(source, &parsed.tokens).join(" "),
                comments,
            )
        }

        let (body, tokens, comments) = parse("type T = { /* member */ a: {} };");
        assert_eq!(body, "{ /* member */ a: {} }");
        assert_eq!(tokens, "type T = { a : { } } ;");
        assert_eq!(comments, ["/* member */"]);

        let (body, tokens, comments) = parse("type T = { /* key */ readonly [K in T]?: U };");
        assert_eq!(body, "{ /* key */ readonly [K in T]?: U }");
        assert_eq!(tokens, "type T = { readonly [ K in T ] ? : U } ;");
        assert_eq!(comments, ["/* key */"]);

        let (body, tokens, comments) = parse("interface I { /* member */ [key: string]: T }");
        assert_eq!(body, "{ /* member */ [key: string]: T }");
        assert_eq!(tokens, "interface I { [ key : string ] : T }");
        assert_eq!(comments, ["/* member */"]);
    }

    #[test]
    fn function_head_contexts() {
        let script = Options {
            lang: Lang::Ts,
            source_type: SourceType::Script,
            ..Options::default()
        };
        let rejects = |source| has_errors(source, script);

        let errors = rejects("async function f<await>() {}");
        assert!(!errors);

        let errors = rejects("function* f<yield>() {}");
        assert!(!errors);

        let errors = rejects("class C { async m<await>() {} }");
        assert!(!errors);

        let errors = rejects("({ async m<await>() {} });");
        assert!(!errors);

        let errors = rejects("async <await>() => {};");
        assert!(!errors);

        let errors = rejects("async function f(await) {}");
        assert!(errors);

        let errors = rejects("async function f() { function g<await>() {} }");
        assert!(errors);
    }

    #[test]
    fn shorthand_identifier_recovery() {
        fn parse(source: &str) -> (bool, String, f64, bool) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let declaration = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap()
                .as_variable_declaration(ast)
                .unwrap();
            let pattern = declaration
                .declarators(ast)
                .get_node(ast, 0)
                .unwrap()
                .id(ast)
                .as_object_pattern(ast)
                .unwrap();
            let property = pattern.properties(ast).get_node(ast, 0).unwrap();
            let value = property.value(ast).as_assignment_pattern(ast).unwrap();
            let identifier = value.left(ast).as_binding_identifier(ast).unwrap();
            (
                !parsed.diagnostics.is_empty(),
                ast.get_utf8(identifier.name(ast)).to_owned(),
                value.right(ast).as_numeric_literal(ast).unwrap().value(ast),
                property.shorthand(ast),
            )
        }

        let (errors, name, default, shorthand) = parse("let {if = 1} = x;");
        assert!(errors);
        assert_eq!(name, "if");
        assert_eq!(default, 1.0);
        assert!(shorthand);

        let (errors, name, default, shorthand) = parse("let {if: value = 1} = x;");
        assert!(!errors);
        assert_eq!(name, "value");
        assert_eq!(default, 1.0);
        assert!(!shorthand);

        let options = Options::default();
        assert!(has_errors("({if = 1} = x);", options));

        assert!(has_errors("({\\u0069f = 1}) => 0;", options));
    }

    #[test]
    fn contextual_labels() {
        fn labels(source: &str) -> (String, Option<String>) {
            let allocator = Allocator::new();
            let options = Options {
                source_type: SourceType::Script,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            assert!(parsed.diagnostics.is_empty());
            let ast = &parsed.ast;
            let label = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_labeled_statement(ast)
                .unwrap();
            let target = label.body(ast).as_while_statement(ast).map(|statement| {
                let body = statement
                    .body(ast)
                    .as_block_statement(ast)
                    .unwrap()
                    .body(ast)
                    .get_node(ast, 0)
                    .unwrap();
                let identifier = if let Some(statement) = body.as_break_statement(ast) {
                    statement.label(ast).unwrap()
                } else {
                    body.as_continue_statement(ast).unwrap().label(ast).unwrap()
                };
                ast.get_utf8(identifier.name(ast)).to_owned()
            });
            (ast.get_utf8(label.label(ast).name(ast)).to_owned(), target)
        }

        let (label, target) = labels("async: while (1) { break as\\u0079nc; }");
        assert_eq!(label, "async");
        assert_eq!(target.as_deref(), Some("async"));

        let (label, target) = labels("let: while (1) { continue l\\u0065t; }");
        assert_eq!(label, "let");
        assert_eq!(target.as_deref(), Some("let"));

        let (label, target) = labels("using: 1;");
        assert_eq!(label, "using");
        assert_eq!(target, None);
    }

    #[test]
    fn using_binding_keywords() {
        fn binding(source: &str) -> (VariableKind, String, bool) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            assert!(parsed.diagnostics.is_empty());
            let ast = &parsed.ast;
            let statement = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_for_of_statement(ast)
                .unwrap();
            let declaration = statement.left(ast).as_variable_declaration(ast).unwrap();
            let identifier = declaration
                .declarators(ast)
                .get_node(ast, 0)
                .unwrap()
                .id(ast)
                .as_binding_identifier(ast)
                .unwrap();
            (
                declaration.kind(ast),
                ast.get_utf8(identifier.name(ast)).to_owned(),
                statement.r#await(ast),
            )
        }

        let (kind, name, is_await) = binding("for (using o\\u0066 of [x]);");
        assert_eq!(kind, VariableKind::Using);
        assert_eq!(name, "of");
        assert!(!is_await);

        let (kind, name, is_await) = binding("for await (using o\\u0066 of of);");
        assert_eq!(kind, VariableKind::Using);
        assert_eq!(name, "of");
        assert!(is_await);

        let options = Options::default();
        assert!(has_errors("for (a o\\u0066 x);", options));
        assert!(!has_errors("for (using of of);", options));
    }

    #[test]
    fn intrinsic_type_recovery() {
        fn parse(source: &str) -> (bool, NodeKind, usize) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let body = ast.root_program().body(ast);
            let alias = body
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap()
                .as_ts_type_alias_declaration(ast)
                .unwrap();
            (
                !parsed.diagnostics.is_empty(),
                ast.node_kind(alias.type_annotation(ast).node_id()),
                body.len(),
            )
        }

        let (errors, kind, statements) = parse("type T = intrinsic | string; after;");
        assert!(errors);
        assert_eq!(kind, NodeKind::TsUnionType);
        assert_eq!(statements, 2);

        let (errors, kind, statements) = parse("type T = intrinsic<A>;");
        assert!(errors);
        assert_eq!(kind, NodeKind::TsTypeReference);
        assert_eq!(statements, 1);

        let (errors, kind, statements) = parse("type T = intrinsic\n[];");
        assert!(!errors);
        assert_eq!(kind, NodeKind::TsIntrinsicKeyword);
        assert_eq!(statements, 2);

        let (errors, kind, statements) = parse("type T = intrinsic.A[];");
        assert!(!errors);
        assert_eq!(kind, NodeKind::TsArrayType);
        assert_eq!(statements, 1);

        let (errors, kind, statements) = parse("type T = intr\\u0069nsic[];");
        assert!(!errors);
        assert_eq!(kind, NodeKind::TsArrayType);
        assert_eq!(statements, 1);
    }

    #[test]
    fn decorator_instantiations() {
        fn decorator(source: &str) -> NodeKind {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            assert!(parsed.diagnostics.is_empty());
            let ast = &parsed.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let class = if let Some(export) = statement.as_export_named_declaration(ast) {
                export.declaration(ast).unwrap().as_class(ast).unwrap()
            } else {
                statement
                    .as_export_default_declaration(ast)
                    .unwrap()
                    .declaration(ast)
                    .as_class(ast)
                    .unwrap()
            };
            assert_eq!(class.decorators(ast).len(), 1);
            let expression = class
                .decorators(ast)
                .get_node(ast, 0)
                .unwrap()
                .expression(ast);
            ast.node_kind(expression.node_id())
        }

        let kind = decorator("@dec<T> export class A {}");
        assert_eq!(kind, NodeKind::TsInstantiationExpression);

        let kind = decorator("@dec<T> export default class A {}");
        assert_eq!(kind, NodeKind::TsInstantiationExpression);

        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        assert!(has_errors("@dec<T> class A {}", options));
        assert!(has_errors("export @dec<T> class A {}", options));
    }

    #[test]
    fn expression_boundaries() {
        fn expression_kinds(ast: &Ast<'_>, body: SubRange<Stmt>) -> Vec<NodeKind> {
            body.iter()
                .map(|id| {
                    let statement = ast.get_node_in_sub_range(id);
                    let expression = statement
                        .as_expression_statement(ast)
                        .unwrap()
                        .expression(ast);
                    ast.node_kind(expression.node_id())
                })
                .collect()
        }

        fn parse(source: &str, options: Options) -> (Vec<Span>, Vec<NodeKind>) {
            let allocator = Allocator::new();
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            let ast = &parsed.ast;
            let errors = parsed.diagnostics.iter().map(|error| error.span).collect();
            let expressions = expression_kinds(ast, ast.root_program().body(ast));
            (errors, expressions)
        }

        for preserve_parens in [false, true] {
            let options = Options {
                preserve_parens,
                ..Options::default()
            };

            let (errors, expressions) = parse("x = () => {}\n+ 1;", options);
            assert!(errors.is_empty());
            assert_eq!(
                expressions,
                [NodeKind::AssignmentExpression, NodeKind::UnaryExpression]
            );

            let (errors, expressions) = parse("x = () => {} + 1; after;", options);
            assert_eq!(errors, [Span::new(13, 14)]);
            assert_eq!(
                expressions,
                [
                    NodeKind::AssignmentExpression,
                    NodeKind::UnaryExpression,
                    NodeKind::IdentifierReference
                ]
            );
        }

        let (errors, expressions) = parse("x = () => {}\n/foo/.test(s);", Options::default());
        assert!(errors.is_empty());
        assert_eq!(
            expressions,
            [NodeKind::AssignmentExpression, NodeKind::CallExpression]
        );

        let (errors, expressions) = parse("a ? b : () => {}\n/foo/.test(s);", Options::default());
        assert!(errors.is_empty());
        assert_eq!(
            expressions,
            [NodeKind::ConditionalExpression, NodeKind::CallExpression]
        );

        let (errors, expressions) = parse("a, () => {}\n/foo/.test(s);", Options::default());
        assert!(errors.is_empty());
        assert_eq!(
            expressions,
            [NodeKind::SequenceExpression, NodeKind::CallExpression]
        );

        let errors = has_errors("a ? b : () => {} || 1;", Options::default());
        assert!(errors);

        let errors = has_errors("a, () => {} + 1;", Options::default());
        assert!(errors);

        let allocator = Allocator::new();
        let source = "function* g() {\n  x = yield\n  /foo/.test(s);\n}";
        let parsed =
            Parser::init(&allocator, source, Options::default(), NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty());
        let ast = &parsed.ast;
        let function = ast
            .root_program()
            .body(ast)
            .get_node(ast, 0)
            .unwrap()
            .as_declaration(ast)
            .unwrap()
            .as_function(ast)
            .unwrap();
        let expressions = expression_kinds(ast, function.body(ast).body(ast));
        assert_eq!(
            expressions,
            [NodeKind::AssignmentExpression, NodeKind::CallExpression]
        );
    }

    #[test]
    fn definite_assignment_recovery() {
        fn parse(source: &str) -> (Vec<&str>, &str, Option<&str>, Option<&str>) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let body = ast.root_program().body(ast);
            assert_eq!(body.len(), 1);
            let statement = body.get_node(ast, 0).unwrap();
            let declaration = if let Some(declaration) = statement.as_declaration(ast) {
                declaration.as_variable_declaration(ast).unwrap()
            } else if let Some(statement) = statement.as_for_statement(ast) {
                statement
                    .init(ast)
                    .unwrap()
                    .as_variable_declaration(ast)
                    .unwrap()
            } else if let Some(statement) = statement.as_for_in_statement(ast) {
                statement.left(ast).as_variable_declaration(ast).unwrap()
            } else {
                statement
                    .as_for_of_statement(ast)
                    .unwrap()
                    .left(ast)
                    .as_variable_declaration(ast)
                    .unwrap()
            };
            let declarator = declaration.declarators(ast).get_node(ast, 0).unwrap();
            assert!(declarator.definite(ast));
            let text = |span: Span| &source[span.start as usize..span.end as usize];
            let errors = parsed
                .diagnostics
                .iter()
                .map(|error| text(error.span))
                .collect();
            (
                errors,
                text(declarator.id(ast).span(ast)),
                declarator
                    .type_annotation(ast)
                    .map(|annotation| text(annotation.span(ast))),
                declarator.init(ast).map(|init| text(init.span(ast))),
            )
        }

        let result = parse("var {}! = {};");
        assert_eq!(result, (vec!["!"], "{}", None, Some("{}")));

        let result = parse("declare var {}! = {};");
        assert_eq!(result, (vec!["!"], "{}", None, Some("{}")));

        let result = parse("let [a]!: T = [1];");
        assert_eq!(result, (vec!["!"], "[a]", Some(": T"), Some("[1]")));

        let result = parse("for (let x!: number;;);");
        assert_eq!(result, (vec!["!"], "x", Some(": number"), None));

        let result = parse("for (var x! in y);");
        assert_eq!(result, (vec!["!"], "x", None, None));

        let result = parse("for (let x! of y);");
        assert_eq!(result, (vec!["!"], "x", None, None));

        let result = parse("let x!: number;");
        assert_eq!(result, (vec![], "x", Some(": number"), None));
    }

    #[test]
    fn type_argument_speculation() {
        fn parse<T>(
            source: &str,
            options: Options,
            inspect: fn(&Ast<'_>) -> T,
        ) -> (Vec<Diagnostic<'static>>, T) {
            let allocator = Allocator::new();
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            let result = inspect(&parsed.ast);
            (parsed.diagnostics, result)
        }

        fn expression(ast: &Ast<'_>) -> Expr {
            ast.root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast)
        }

        fn array_length(ast: &Ast<'_>) -> usize {
            expression(ast)
                .as_array_expression(ast)
                .unwrap()
                .elements(ast)
                .len()
        }

        fn expression_kind(ast: &Ast<'_>) -> NodeKind {
            ast.node_kind(expression(ast).node_id())
        }

        fn arguments(ast: &Ast<'_>) -> (usize, NodeKind) {
            let call = expression(ast).as_call_expression(ast).unwrap();
            let arguments = call.arguments(ast);
            let last = arguments.get_node(ast, arguments.len() - 1).unwrap();
            (arguments.len(), ast.node_kind(last.node_id()))
        }

        fn type_argument_depth(ast: &Ast<'_>) -> usize {
            let call = expression(ast).as_call_expression(ast).unwrap();
            let mut arguments = call.type_arguments(ast);
            let mut depth = 0;
            while let Some(list) = arguments {
                depth += 1;
                let ty = list.params(ast).get_node(ast, 0).unwrap();
                arguments = ty.as_ts_type_reference(ast).unwrap().type_arguments(ast);
            }
            depth
        }

        let typescript = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let tsx = Options {
            lang: Lang::Tsx,
            ..typescript
        };
        for options in [typescript, tsx] {
            let (errors, length) = parse("[a < b, a < b];", options, array_length);
            assert!(errors.is_empty());
            assert_eq!(length, 2);

            let (errors, args) = parse("f(i < n, j < m, 1);", options, arguments);
            assert!(errors.is_empty());
            assert_eq!(args, (3, NodeKind::NumericLiteral));

            let (errors, args) = parse("f(i < n, g<A<B>>(c));", options, arguments);
            assert!(errors.is_empty());
            assert_eq!(args, (2, NodeKind::CallExpression));

            let (errors, args) = parse("f(a < b, g<{[/>/]: T}>(c));", options, arguments);
            assert!(errors.is_empty());
            assert_eq!(args, (2, NodeKind::CallExpression));

            let (errors, depth) = parse("f<A<B<C>>>();", options, type_argument_depth);
            assert!(errors.is_empty());
            assert_eq!(depth, 3);

            let (errors, depth) = parse("f<A<B<if>>>();", options, type_argument_depth);
            assert!(!errors.is_empty());
            assert_eq!(depth, 3);

            let (errors, kind) = parse("a < {[a < {[x]: 1}]: 1};", options, expression_kind);
            assert!(errors.is_empty());
            assert_eq!(kind, NodeKind::BinaryExpression);
        }
    }

    #[test]
    fn primary_types() {
        let allocator = Allocator::new();

        let parsed = parse_tokens(&allocator, "type A = +1;");
        assert!(!parsed.diagnostics.is_empty());

        let parsed = parse_tokens(&allocator, "type A = +1n;");
        assert!(!parsed.diagnostics.is_empty());

        let parsed = parse_tokens(&allocator, "type A = null.B;");
        assert!(!parsed.diagnostics.is_empty());

        let parsed = parse_tokens(&allocator, "type A = -1;");
        assert!(parsed.diagnostics.is_empty());

        let parsed = parse_tokens(&allocator, "type A = -1n;");
        assert!(parsed.diagnostics.is_empty());

        let parsed = parse_tokens(&allocator, "type A = null;");
        assert!(parsed.diagnostics.is_empty());

        let parsed = parse_tokens(&allocator, "type A = A.B;");
        assert!(parsed.diagnostics.is_empty());

        let parsed = parse_tokens(&allocator, "type A = typeof a.b;");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn array_rest_recovery() {
        fn text(source: &str, span: Span) -> &str {
            &source[span.start as usize..span.end as usize]
        }

        fn parse(source: &str) -> (Vec<&str>, Vec<&str>, Option<&str>) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let variable = statement
                .as_declaration(ast)
                .unwrap()
                .as_variable_declaration(ast)
                .unwrap();
            let declarator = variable.declarators(ast).get_node(ast, 0).unwrap();
            let pattern = declarator.id(ast).as_array_pattern(ast).unwrap();
            let errors = parsed
                .diagnostics
                .iter()
                .map(|diagnostic| text(source, diagnostic.span));
            let elements = pattern.elements(ast);
            let elements = elements
                .iter()
                .map(|id| text(source, ast.get_node_in_sub_range(id).unwrap().span(ast)));
            let rest = pattern.rest(ast).map(|rest| text(source, rest.span(ast)));
            (errors.collect(), elements.collect(), rest)
        }

        fn is_discarded(source: &str) -> bool {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            parsed.ast.root_program().body(&parsed.ast).is_empty()
        }

        macro_rules! check {
            ($source:expr, errors: [$($error:expr),*], elements: [$($element:expr),*], rest: $rest:expr) => {
                assert_eq!(parse($source), (vec![$($error),*], vec![$($element),*], $rest))
            };
        }

        check!("const [a, ...b, c, ...d] = v;",
            errors: ["...b"], elements: ["a", "...b", "c"], rest: Some("...d"));

        check!("const [...a, b,] = v;",
            errors: ["...a"], elements: ["...a", "b"], rest: None);

        assert!(is_discarded("const [...a, 0] = v;"));
    }

    #[test]
    fn cover_trailing_commas() {
        struct Rests<'ast, 'a> {
            ast: &'ast Ast<'a>,
            spans: Vec<Span>,
        }

        impl<'a> Visit<'a> for Rests<'_, 'a> {
            fn ast(&self) -> &Ast<'a> {
                self.ast
            }

            fn visit_binding_rest_element(&mut self, rest: BindingRestElement) {
                self.spans.push(rest.span(self.ast));
                rest.visit_children_with(self);
            }
        }

        fn parse(source: &str) -> (Vec<std::ops::Range<u32>>, Vec<&str>, String) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let mut rests = Rests {
                ast: &parsed.ast,
                spans: Vec::new(),
            };
            parsed.ast.root_program().visit_with(&mut rests);
            let errors = parsed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.span.start..diagnostic.span.end)
                .collect();
            let rests = rests
                .spans
                .iter()
                .map(|span| &source[span.start as usize..span.end as usize])
                .collect();
            (errors, rests, token_texts(source, &parsed.tokens).join(" "))
        }

        macro_rules! check {
            ($source:expr, errors: [$($error:expr),*], rests: [$($rest:expr),*], tokens: $tokens:expr) => {
                assert_eq!(parse($source), (vec![$($error),*], vec![$($rest),*], $tokens.into()))
            };
        }

        check!("const [[...a,], [...b,],] = v;", errors: [12..13, 21..22], rests: ["...a", "...b"],
            tokens: "const [ [ ... a , ] , [ ... b , ] , ] = v ;");

        check!("const {a: [...b,], ...c,} = v;", errors: [15..16, 23..24], rests: ["...b", "...c"],
            tokens: "const { a : [ ... b , ] , ... c , } = v ;");

        check!("({a: {...b,}, ...c,} = v);", errors: [10..11, 18..19], rests: ["...b", "...c"],
            tokens: "( { a : { ... b , } , ... c , } = v ) ;");

        check!("const [{...a,}, x = async([...b,])] = v;", errors: [12..13], rests: ["...a"],
            tokens: "const [ { ... a , } , x = async ( [ ... b , ] ) ] = v ;");

        check!("const a = [[...b,], {...c,},];", errors: [], rests: [],
            tokens: "const a = [ [ ... b , ] , { ... c , } , ] ;");
    }

    #[test]
    fn cover_initialized_names() {
        for preserve_parens in [false, true] {
            let javascript = Options {
                preserve_parens,
                ..Options::default()
            };
            let typescript = Options {
                lang: Lang::Ts,
                ..javascript
            };
            for options in [javascript, typescript] {
                let errors = error_spans("(({a = 0}));", options);
                assert_eq!(errors, ["a = 0"]);

                let errors = error_spans("(({a: ({b = 0}), c = 0}));", options);
                assert_eq!(errors, ["b = 0", "c = 0"]);

                let errors = error_spans("[{a = 1}, x = {}];", options);
                assert_eq!(errors, ["a = 1"]);

                let errors = error_spans("[{a = 1}, 1 + {}];", options);
                assert_eq!(errors, ["a = 1"]);

                let errors = error_spans("[{a = 1}, x = []];", options);
                assert_eq!(errors, ["a = 1"]);

                let errors = error_spans("[{a = 1}, f({})];", options);
                assert_eq!(errors, ["a = 1"]);

                let errors = error_spans("[{a = 1}, x = {b = 2}];", options);
                assert_eq!(errors, ["a = 1", "b = 2"]);

                let errors = error_spans("[{a: 1}, x = {}];", options);
                assert!(errors.is_empty());

                let errors = error_spans("[{a = 1}] in x;", options);
                assert_eq!(errors, ["a = 1"]);

                let errors = error_spans("({[{x = 1}]: b} = c);", options);
                assert_eq!(errors, ["x = 1"]);

                let errors = error_spans("({a: {b = 1}}.c);", options);
                assert_eq!(errors, ["b = 1"]);

                let errors = error_spans("[{a = 1}.b] = c;", options);
                assert_eq!(errors, ["a = 1"]);

                let errors = error_spans("[{a = 1}, x = {}] = xs;", options);
                assert!(errors.is_empty());

                let errors = error_spans("[{a = 1}]; x = 1 +;", options);
                assert_eq!(errors, ["a = 1", ";"]);

                let errors = error_spans("function f() { [{a = 1}]; x = 1 +; }", options);
                assert_eq!(errors, ["a = 1", ";"]);

                let errors = error_spans("[{a = 1}, x = () => { return; }] = xs;", options);
                assert!(errors.is_empty());

                let errors = error_spans("[{a = 1}, x = () => { [{b = 2}]; }] = xs;", options);
                assert_eq!(errors, ["b = 2"]);
            }

            let errors = error_spans("[{a = 1}, <T>(x = {})];", typescript);
            assert_eq!(errors, ["a = 1"]);
        }
    }

    #[test]
    fn syntax_errors() {
        fn diagnostics(source: &str, options: Options) -> Vec<Diagnostic<'static>> {
            let allocator = Allocator::new();
            Parser::init(&allocator, source, options, NoTokenParserConfig)
                .parse()
                .diagnostics
        }

        let script = Options {
            source_type: SourceType::Script,
            ..Options::default()
        };
        let module = Options {
            source_type: SourceType::Module,
            ..script
        };
        let commonjs = Options {
            source_type: SourceType::CommonJs,
            ..script
        };
        let unambiguous = Options {
            source_type: SourceType::Unambiguous,
            ..script
        };
        let typescript = Options {
            lang: Lang::Ts,
            ..module
        };

        let errors = diagnostics("function f(,) {\n  return;\n}", script);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].span, Span::new(11, 12));

        let errors = diagnostics("function f(,) {\n  return;\n}", module);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].span, Span::new(11, 12));

        let errors = diagnostics("function f(,) {\n  return;\n}", commonjs);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].span, Span::new(11, 12));

        let errors = diagnostics("function f(,) {\n  return;\n}", unambiguous);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].span, Span::new(11, 12));

        let errors = diagnostics("{ function f(,) {\n  return;\n} }", script);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("function outer() { function f(,) {\n  return;\n} }", script);
        assert_eq!(errors.len(), 1);

        let source = "class C { static { function f(,) {\n  return;\n} } }";
        let errors = diagnostics(source, script);
        assert_eq!(errors.len(), 1);

        let source = "switch (0) { case 0: function f(,) {\n  return;\n} }";
        let errors = diagnostics(source, script);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("switch (0) { case ,:\n  return;\n}", script);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("namespace N { function f(,) {\n  return;\n} }", typescript);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("function f() { let # name }", script);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("function f() { let # name }", unambiguous);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("class A { constructor; }", script);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("class A { static constructor; }", script);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("class A { \"constructor\" = 1; }", typescript);
        assert_eq!(errors.len(), 1);

        let errors = diagnostics("class A { static \"constructor\"; }", typescript);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn error_recovery() {
        fn throw_argument(ast: &Ast<'_>) -> IdentifierReference {
            ast.root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_throw_statement(ast)
                .unwrap()
                .argument(ast)
                .as_identifier_reference(ast)
                .unwrap()
        }

        let allocator = Allocator::new();
        let source = "function f(,) {\n  return;\n}";
        let parsed = parse_tokens(&allocator, source);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            token_texts(source, &parsed.tokens),
            ["function", "f", "(", ","]
        );

        let allocator = Allocator::new();
        let source = "return 1;\nconst after = 1;";
        let parsed = parse_tokens(&allocator, source);
        assert!(!parsed.diagnostics.is_empty());
        assert_eq!(
            token_texts(source, &parsed.tokens),
            ["return", "1", ";", "const", "after", "=", "1", ";"]
        );

        let allocator = Allocator::new();
        let source = "const before = 1;\nconst text = 'unterminated";
        let parsed = parse_tokens(&allocator, source);
        assert!(!parsed.diagnostics.is_empty());
        assert_eq!(
            token_texts(source, &parsed.tokens),
            ["const", "before", "=", "1", ";", "const", "text", "="]
        );

        let allocator = Allocator::new();
        let parsed = parse_tokens(&allocator, "'unterminated");
        assert!(!parsed.diagnostics.is_empty());
        assert!(parsed.tokens.is_empty());

        let allocator = Allocator::new();
        let parsed = parse_tokens(&allocator, "");
        assert!(parsed.diagnostics.is_empty());
        assert!(parsed.tokens.is_empty());

        let allocator = Allocator::new();
        let source = "throw\nvalue;\nafter;";
        let parsed = parse_tokens(&allocator, source);
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].span, throw_argument(ast).span(ast));
        assert_eq!(ast.get_utf8(throw_argument(ast).name(ast)), "value");
        let body = ast.root_program().body(ast);
        assert_eq!(body.len(), 2);
        let after = body
            .get_node(ast, 1)
            .unwrap()
            .as_expression_statement(ast)
            .unwrap()
            .expression(ast)
            .as_identifier_reference(ast)
            .unwrap();
        assert_eq!(ast.get_utf8(after.name(ast)), "after");
        assert_eq!(
            token_texts(source, &parsed.tokens),
            ["throw", "value", ";", "after", ";"]
        );

        let parsed = parse_tokens(&allocator, "throw /*\n*/ value;");
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(ast.get_utf8(throw_argument(ast).name(ast)), "value");

        let parsed = parse_tokens(&allocator, "throw // comment\nvalue;");
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(ast.get_utf8(throw_argument(ast).name(ast)), "value");

        let parsed = parse_tokens(&allocator, "throw\n;");
        assert!(!parsed.diagnostics.is_empty());
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());

        let parsed = parse_tokens(&allocator, "throw\n");
        assert!(!parsed.diagnostics.is_empty());
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());
    }

    #[test]
    fn try_recovery() {
        fn first_try(ast: &Ast<'_>) -> TryStatement {
            ast.root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_try_statement(ast)
                .unwrap()
        }

        fn expression_name<'a>(ast: &Ast<'a>, statement: Stmt) -> &'a str {
            let identifier = statement
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast)
                .as_identifier_reference(ast)
                .unwrap();
            ast.get_utf8(identifier.name(ast))
        }

        let allocator = Allocator::new();
        let source = "try { inside; }\nafter;";
        let parsed = parse_tokens(&allocator, source);
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].span, Span::new(16, 21));
        let statement = first_try(ast);
        assert!(statement.handler(ast).is_none());
        assert!(statement.finalizer(ast).is_none());
        let block = statement.block(ast).body(ast);
        assert_eq!(block.len(), 1);
        assert_eq!(
            expression_name(ast, block.get_node(ast, 0).unwrap()),
            "inside"
        );
        let body = ast.root_program().body(ast);
        assert_eq!(body.len(), 2);
        assert_eq!(
            expression_name(ast, body.get_node(ast, 1).unwrap()),
            "after"
        );
        assert_eq!(
            token_texts(source, &parsed.tokens),
            ["try", "{", "inside", ";", "}", "after", ";"]
        );

        let parsed = parse_tokens(&allocator, "try {}");
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].span, Span::new(6, 6));
        assert_eq!(ast.root_program().body(ast).len(), 1);
        let statement = first_try(ast);
        assert!(statement.block(ast).body(ast).is_empty());
        assert!(statement.handler(ast).is_none());
        assert!(statement.finalizer(ast).is_none());

        let parsed = parse_tokens(&allocator, "try { inside;");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());

        let parsed = parse_tokens(&allocator, "try {} catch (error)");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());

        let parsed = parse_tokens(&allocator, "try {} finally");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());
    }

    fn parse_typescript_declaration<'a>(
        allocator: &'a Allocator,
        source: &'a str,
    ) -> (Ast<'a>, Decl) {
        let options = Options {
            lang: Lang::Ts,
            source_type: SourceType::Module,
            ..Options::default()
        };
        let parsed = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty());
        let ast = parsed.ast;
        let body = ast.root_program().body(&ast);
        assert_eq!(body.len(), 1);
        let statement = body.get_node(&ast, 0).unwrap();
        let declaration = statement.as_declaration(&ast).unwrap_or_else(|| {
            statement
                .as_export_named_declaration(&ast)
                .unwrap()
                .declaration(&ast)
                .unwrap()
        });
        (ast, declaration)
    }

    #[test]
    fn ambient_variables() {
        fn parse_variable(source: &str) -> (VariableKind, String) {
            let allocator = Allocator::new();
            let (ast, declaration) = parse_typescript_declaration(&allocator, source);
            let variable = declaration.as_variable_declaration(&ast).unwrap();
            assert!(variable.declare(&ast));
            assert_eq!(variable.declarators(&ast).len(), 1);
            let declarator = variable.declarators(&ast).get_node(&ast, 0).unwrap();
            assert!(declarator.type_annotation(&ast).is_some());
            assert!(declarator.init(&ast).is_none());
            let binding = declarator.id(&ast);
            let name = if let Some(identifier) = binding.as_binding_identifier(&ast) {
                ast.get_utf8(identifier.name(&ast)).to_owned()
            } else if binding.as_array_pattern(&ast).is_some() {
                "array".to_owned()
            } else {
                assert!(binding.as_object_pattern(&ast).is_some());
                "object".to_owned()
            };
            (variable.kind(&ast), name)
        }

        let variable = parse_variable("declare var /* marker\n continued */ foo: string;");
        assert_eq!(variable, (VariableKind::Var, "foo".into()));

        let variable = parse_variable("declare let // marker\nfoo: string;");
        assert_eq!(variable, (VariableKind::Let, "foo".into()));

        let variable = parse_variable("declare const\nfoo: string;");
        assert_eq!(variable, (VariableKind::Const, "foo".into()));

        let variable = parse_variable("export declare const\nfoo: string;");
        assert_eq!(variable, (VariableKind::Const, "foo".into()));

        let variable = parse_variable("export declare const // marker\nfoo: string;");
        assert_eq!(variable, (VariableKind::Const, "foo".into()));

        let variable = parse_variable("export declare const /* marker\n continued */ foo: string;");
        assert_eq!(variable, (VariableKind::Const, "foo".into()));

        let variable = parse_variable("declare const // marker\nglobal: string;");
        assert_eq!(variable, (VariableKind::Const, "global".into()));

        let variable = parse_variable("declare const\n[foo]: [string];");
        assert_eq!(variable, (VariableKind::Const, "array".into()));

        let variable = parse_variable("declare let // marker\n{ foo }: { foo: string };");
        assert_eq!(variable, (VariableKind::Let, "object".into()));
    }

    #[test]
    fn const_enums() {
        fn is_ambient(source: &str) -> bool {
            let allocator = Allocator::new();
            let (ast, declaration) = parse_typescript_declaration(&allocator, source);
            let enumeration = declaration.as_ts_enum_declaration(&ast).unwrap();
            assert!(enumeration.is_const(&ast));
            assert_eq!(ast.get_utf8(enumeration.id(&ast).name(&ast)), "E");
            assert_eq!(enumeration.body(&ast).members(&ast).len(), 1);
            enumeration.declare(&ast)
        }

        let ambient = is_ambient("const\nenum E { A = 1 }");
        assert!(!ambient);

        let ambient = is_ambient("const // marker\nenum E { A = 1 }");
        assert!(!ambient);

        let ambient = is_ambient("const /* marker\n continued */ enum E { A = 1 }");
        assert!(!ambient);

        let ambient = is_ambient("declare const // marker\nenum E { A = 1 }");
        assert!(ambient);

        let ambient = is_ambient("export const /* marker\n continued */ enum E { A = 1 }");
        assert!(!ambient);
    }

    #[test]
    fn enum_member_names() {
        fn texts(source: &str, spans: impl Iterator<Item = Span>) -> Vec<&str> {
            spans
                .map(|span| &source[span.start as usize..span.end as usize])
                .collect()
        }

        fn parse(source: &str) -> (Vec<&str>, Vec<&str>) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let declaration = statement.as_declaration(ast).unwrap();
            let enumeration = declaration.as_ts_enum_declaration(ast).unwrap();
            let errors = parsed.diagnostics.iter().map(|diagnostic| diagnostic.span);
            let members = enumeration.body(ast).members(ast);
            let members = members
                .iter()
                .map(|id| ast.get_node_in_sub_range(id).id(ast).span(ast));
            (texts(source, errors), texts(source, members))
        }

        macro_rules! check {
            ($source:expr, errors: [$($error:expr),*], members: [$($member:expr),*]) => {
                assert_eq!(parse($source), (vec![$($error),*], vec![$($member),*]))
            };
        }

        check!("enum E { `a`, B }", errors: ["`a`"], members: ["`a`", "B"]);

        check!("enum E { [`a`] }", errors: [], members: ["`a`"]);
    }

    #[test]
    fn global_augmentations() {
        fn interface_name(source: &str) -> String {
            let allocator = Allocator::new();
            let (ast, declaration) = parse_typescript_declaration(&allocator, source);
            let global = declaration.as_ts_global_declaration(&ast).unwrap();
            assert!(global.declare(&ast));
            assert_eq!(ast.get_utf8(global.id(&ast).name(&ast)), "global");
            let body = global.body(&ast).body(&ast);
            assert_eq!(body.len(), 1);
            let interface = body
                .get_node(&ast, 0)
                .unwrap()
                .as_declaration(&ast)
                .unwrap()
                .as_ts_interface_declaration(&ast)
                .unwrap();
            ast.get_utf8(interface.id(&ast).name(&ast)).to_owned()
        }

        let name = interface_name("declare global\n{ interface Foo {} }");
        assert_eq!(name, "Foo");

        let name = interface_name("declare global // marker\n{ interface Foo {} }");
        assert_eq!(name, "Foo");

        let name = interface_name("declare global /* marker\n continued */ { interface Foo {} }");
        assert_eq!(name, "Foo");
    }

    #[test]
    fn named_declarations() {
        fn class_flags(source: &str) -> (bool, bool, usize) {
            let allocator = Allocator::new();
            let (ast, declaration) = parse_typescript_declaration(&allocator, source);
            let class = declaration.as_class(&ast).unwrap();
            assert_eq!(class.class_type(&ast), ClassType::ClassDeclaration);
            assert_eq!(ast.get_utf8(class.id(&ast).unwrap().name(&ast)), "Foo");
            (
                class.declare(&ast),
                class.r#abstract(&ast),
                class.body(&ast).body(&ast).len(),
            )
        }

        let flags = class_flags("declare class // marker\nFoo {}");
        assert_eq!(flags, (true, false, 0));

        let flags = class_flags("export declare class /* marker\n continued */ Foo {}");
        assert_eq!(flags, (true, false, 0));

        let flags = class_flags(
            "abstract class\nFoo { protected abstract readonly foo: number; public static method() {} }",
        );
        assert_eq!(flags, (false, true, 2));

        let flags = class_flags(
            "abstract class // marker\nFoo { protected abstract readonly foo: number; public static method() {} }",
        );
        assert_eq!(flags, (false, true, 2));

        let flags = class_flags(
            "abstract class /* marker\n continued */ Foo { protected abstract readonly foo: number; public static method() {} }",
        );
        assert_eq!(flags, (false, true, 2));

        let flags = class_flags("abstract class\nFoo { abstract method<T>(x: T): T; }");
        assert_eq!(flags, (false, true, 1));

        let flags = class_flags("abstract class // marker\nFoo { abstract method<T>(x: T): T; }");
        assert_eq!(flags, (false, true, 1));

        let flags = class_flags(
            "abstract class /* marker\n continued */ Foo { abstract method<T>(x: T): T; }",
        );
        assert_eq!(flags, (false, true, 1));

        let allocator = Allocator::new();
        let (ast, declaration) =
            parse_typescript_declaration(&allocator, "declare function\nfoo(): void;");
        let function = declaration.as_ts_function(&ast).unwrap();
        assert!(function.declare(&ast));
        assert_eq!(ast.get_utf8(function.id(&ast).unwrap().name(&ast)), "foo");
        assert!(function.params(&ast).items(&ast).is_empty());
        assert!(function.return_type(&ast).is_some());
    }

    #[test]
    fn ambient_function_recovery() {
        fn function_name(ast: &Ast<'_>, statement: Stmt) -> String {
            let function = statement
                .as_declaration(ast)
                .unwrap()
                .as_ts_function(ast)
                .unwrap();
            ast.get_utf8(function.id(ast).unwrap().name(ast)).to_owned()
        }

        fn recovered_names(source: &str, lang: Lang) -> Vec<String> {
            let allocator = Allocator::new();
            let options = Options {
                lang,
                source_type: SourceType::Module,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            assert_eq!(parsed.diagnostics.len(), 1);
            let ast = &parsed.ast;
            let body = ast.root_program().body(ast);
            assert_eq!(body.len(), 2);
            let declaration = body.get_node(ast, 0).unwrap().as_declaration(ast).unwrap();
            let functions = if let Some(module) = declaration.as_ts_module_declaration(ast) {
                assert_eq!(function_name(ast, body.get_node(ast, 1).unwrap()), "after");
                module
                    .body(ast)
                    .unwrap()
                    .as_ts_module_block(ast)
                    .unwrap()
                    .body(ast)
            } else {
                assert!(declaration.as_ts_function(ast).unwrap().declare(ast));
                body
            };
            functions
                .iter()
                .map(|id| function_name(ast, ast.get_node_in_sub_range(id)))
                .collect()
        }

        let source = "declare namespace N { function bad() {} function good(): void; } declare function after(): void;";
        let names = recovered_names(source, Lang::Ts);
        assert_eq!(names, ["bad", "good"]);

        let source = "declare module \"pkg\" { function bad() {} function good(): void; } declare function after(): void;";
        let names = recovered_names(source, Lang::Ts);
        assert_eq!(names, ["bad", "good"]);

        let source = "declare function bad() {} declare function good(): void;";
        let names = recovered_names(source, Lang::Dts);
        assert_eq!(names, ["bad", "good"]);
    }

    #[test]
    fn template_raw_values() {
        fn values(source: &str) -> (String, Option<String>) {
            let allocator = Allocator::new();
            let parsed =
                Parser::init(&allocator, source, Options::default(), NoTokenParserConfig).parse();
            assert!(parsed.diagnostics.is_empty());
            let ast = &parsed.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let expression = statement
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast);
            let template = match ast.expr_data(expression) {
                ExprData::TemplateLiteral(template) => template,
                ExprData::TaggedTemplateExpression(tagged) => tagged.quasi(ast),
                _ => panic!("expected a template expression"),
            };
            let quasi = template.quasis(ast).get_node(ast, 0).unwrap();
            let raw = ast.get_utf8(quasi.raw(ast)).to_owned();
            let cooked = (!quasi.is_cooked_undefined(ast)).then(|| {
                ast.get_wtf8(quasi.cooked(ast))
                    .to_string_lossy()
                    .into_owned()
            });
            (raw, cooked)
        }

        let (raw, cooked) = values("`a\rb\r\nc\r\r\n`");
        assert_eq!(raw, "a\nb\nc\n\n");
        assert_eq!(cooked.as_deref(), Some("a\nb\nc\n\n"));

        let (raw, cooked) = values("tag`\\unicode\r\ntext`");
        assert_eq!(raw, "\\unicode\ntext");
        assert!(cooked.is_none());

        let (raw, cooked) = values("`a\u{2028}b\u{2029}c`");
        assert_eq!(raw, "a\u{2028}b\u{2029}c");
        assert_eq!(cooked.as_deref(), Some("a\u{2028}b\u{2029}c"));

        let (raw, cooked) = values("`a\\rb`");
        assert_eq!(raw, "a\\rb");
        assert_eq!(cooked.as_deref(), Some("a\rb"));
    }

    #[test]
    fn type_parameter_modifiers() {
        fn parse_parameter<'a>(
            allocator: &'a Allocator,
            source: &'a str,
        ) -> (Ast<'a>, TsTypeParameter) {
            let (ast, declaration) = parse_typescript_declaration(allocator, source);
            let alias = declaration.as_ts_type_alias_declaration(&ast).unwrap();
            let parameters = if let Some(parameters) = alias.type_parameters(&ast) {
                parameters
            } else {
                let function = alias
                    .type_annotation(&ast)
                    .as_ts_function_type(&ast)
                    .unwrap();
                function.type_parameters(&ast).unwrap()
            };
            let parameter = parameters.params(&ast).get_node(&ast, 0).unwrap();
            (ast, parameter)
        }

        fn has_errors(source: &str) -> bool {
            let allocator = Allocator::new();
            let options = Options {
                lang: Lang::Ts,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            !parsed.diagnostics.is_empty()
        }

        let allocator = Allocator::new();

        let (ast, parameter) = parse_parameter(&allocator, "type F = <const T>(x:T) => T;");
        assert!(parameter.r#const(&ast));
        assert_eq!(ast.get_utf8(parameter.name(&ast).name(&ast)), "T");

        let (ast, parameter) =
            parse_parameter(&allocator, "type F = <const /* note */ T>(x:T) => T;");
        assert!(parameter.r#const(&ast));
        assert_eq!(ast.get_utf8(parameter.name(&ast).name(&ast)), "T");

        let (ast, parameter) = parse_parameter(&allocator, r"type F = <const \u0054>(x:T) => T;");
        assert!(parameter.r#const(&ast));
        assert_eq!(ast.get_utf8(parameter.name(&ast).name(&ast)), "T");

        let (ast, parameter) = parse_parameter(&allocator, "type A<const T> = T;");
        assert!(parameter.r#const(&ast));
        assert_eq!(ast.get_utf8(parameter.name(&ast).name(&ast)), "T");

        let (ast, parameter) = parse_parameter(&allocator, "type A<out> = string;");
        assert_eq!(ast.get_utf8(parameter.name(&ast).name(&ast)), "out");
        assert!(!parameter.out(&ast));

        let (ast, parameter) = parse_parameter(&allocator, "type A<in T> = (x:T) => void;");
        assert!(parameter.r#in(&ast));
        assert!(!parameter.out(&ast));
        assert_eq!(ast.get_utf8(parameter.name(&ast).name(&ast)), "T");

        let (ast, parameter) = parse_parameter(&allocator, "type A<out T> = () => T;");
        assert!(parameter.out(&ast));
        assert!(!parameter.r#in(&ast));
        assert_eq!(ast.get_utf8(parameter.name(&ast).name(&ast)), "T");

        let (ast, parameter) = parse_parameter(&allocator, "type A<in out T> = (x:T) => T;");
        assert!(parameter.r#in(&ast));
        assert!(parameter.out(&ast));

        assert!(has_errors("type F = <const /* note\n */ T>(x:T) => T;"));

        assert!(has_errors("type F = <const // note\n T>(x:T) => T;"));

        assert!(has_errors("type F = <const>() => const;"));

        assert!(has_errors("type A<in> = string;"));

        assert!(has_errors("type F = <const\nT>() => T;"));

        assert!(has_errors("type A<in\nT> = (x:T) => void;"));

        assert!(has_errors("type A<out\nT> = () => T;"));

        assert!(has_errors("type A<in /*\n*/ out T> = (x:T) => T;"));
    }

    #[test]
    fn infer_constraints() {
        fn constraint(ast: &Ast<'_>) -> TsType {
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let declaration = statement.as_declaration(ast).unwrap();
            let alias = declaration.as_ts_type_alias_declaration(ast).unwrap();
            let conditional = alias
                .type_annotation(ast)
                .as_ts_conditional_type(ast)
                .unwrap();
            let infer = conditional.extends_type(ast).as_ts_infer_type(ast).unwrap();
            infer.type_parameter(ast).constraint(ast).unwrap()
        }

        for lang in [Lang::Ts, Lang::Tsx] {
            let allocator = Allocator::new();
            let options = Options {
                lang,
                ..Options::default()
            };
            let parse = |source| {
                let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
                assert!(result.diagnostics.is_empty());
                result.ast
            };

            let ast = parse("type T<A> = A extends infer U extends () => void ? U : never;");
            let function = constraint(&ast).as_ts_function_type(&ast).unwrap();
            assert!(function.type_parameters(&ast).is_none());

            let ast = parse("type T<A> = A extends infer U extends <V>() => V ? U : never;");
            let function = constraint(&ast).as_ts_function_type(&ast).unwrap();
            assert_eq!(
                function.type_parameters(&ast).unwrap().params(&ast).len(),
                1
            );

            let ast = parse("type T<A> = A extends infer U extends new () => object ? U : never;");
            let constructor = constraint(&ast).as_ts_constructor_type(&ast).unwrap();
            assert!(!constructor.r#abstract(&ast));

            let ast = parse(
                "type T<A> = A extends infer U extends abstract new () => object ? U : never;",
            );
            let constructor = constraint(&ast).as_ts_constructor_type(&ast).unwrap();
            assert!(constructor.r#abstract(&ast));
        }
    }

    #[test]
    fn predicate_parameter_names() {
        fn parsed_name(source: &str) -> (bool, String) {
            let allocator = Allocator::new();
            let options = Options {
                lang: Lang::Ts,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            let ast = &parsed.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let alias = statement
                .as_declaration(ast)
                .unwrap()
                .as_ts_type_alias_declaration(ast)
                .unwrap();
            let function = alias.type_annotation(ast).as_ts_function_type(ast).unwrap();
            let predicate = function
                .return_type(ast)
                .type_annotation(ast)
                .as_ts_type_predicate(ast)
                .unwrap();
            let name = predicate
                .parameter_name(ast)
                .as_identifier_name(ast)
                .unwrap();
            (
                !parsed.diagnostics.is_empty(),
                ast.get_utf8(name.name(ast)).to_owned(),
            )
        }

        fn name(source: &str) -> String {
            let (errors, name) = parsed_name(source);
            assert!(!errors);
            name
        }

        fn has_errors(source: &str) -> bool {
            let allocator = Allocator::new();
            let options = Options {
                lang: Lang::Ts,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            !parsed.diagnostics.is_empty()
        }

        assert_eq!(name(r"type F = (x: unknown) => \u0078 is string;"), "x");

        assert_eq!(name("type F = (x: unknown) => x\x0Bis string;"), "x");

        assert_eq!(name("type F = (x: unknown) => x\u{00A0}is string;"), "x");

        assert_eq!(
            name(r"type F = (x: unknown) => asserts \u0078 is string;"),
            "x"
        );

        assert!(has_errors(r"type F = (x: unknown) => x \u0069s string;"));

        assert!(has_errors("type F = (x: unknown) => x\nis string;"));

        assert!(has_errors("type F = (x: unknown) => \\u0078\nis string;"));
        let (errors, name) = parsed_name("type F = (x: unknown) => break is string;");
        assert!(errors);
        assert_eq!(name, "break");

        let (errors, name) = parsed_name("type F = (x: unknown) => asserts br\\u0065ak is string;");
        assert!(errors);
        assert_eq!(name, "break");

        let (errors, name) = parsed_name("type F = (x: unknown) => await is string;");
        assert!(!errors);
        assert_eq!(name, "await");
    }

    fn serialize_ast(ast: &Ast<'_>) -> Vec<u8> {
        use swc_next_ecma_ast::ffi::ast::{buffer_size, serialize_into};
        let mut buffer = vec![0; buffer_size(ast, &[])];
        serialize_into(ast, &[], &mut buffer, ast.source().encode_utf16().count());
        buffer
    }

    #[test]
    fn default_export_declarations() {
        fn declaration(ast: &Ast<'_>) -> ExportDefaultDeclarationKindData {
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let export = statement.as_export_default_declaration(ast).unwrap();
            ast.export_default_declaration_kind_data(export.declaration(ast))
        }

        fn exported_function(ast: &Ast<'_>) -> Function {
            let ExportDefaultDeclarationKindData::Function(function) = declaration(ast) else {
                panic!("expected a default-exported function declaration");
            };
            function
        }

        for lang in [Lang::Js, Lang::Ts, Lang::Tsx] {
            let allocator = Allocator::new();
            let options = Options {
                lang,
                source_type: SourceType::Module,
                ..Options::default()
            };
            let parse = |source| {
                let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
                assert!(result.diagnostics.is_empty());
                result.ast
            };

            let ast = parse("export default\nasync function f() {}");
            let function = exported_function(&ast);
            assert_eq!(
                function.function_type(&ast),
                FunctionType::FunctionDeclaration
            );
            assert!(function.r#async(&ast));
            assert!(!function.generator(&ast));
            assert_eq!(ast.get_utf8(function.id(&ast).unwrap().name(&ast)), "f");

            let ast = parse("export default /*\n*/ async function* () {}");
            let function = exported_function(&ast);
            assert_eq!(
                function.function_type(&ast),
                FunctionType::FunctionDeclaration
            );
            assert!(function.r#async(&ast));
            assert!(function.generator(&ast));
            assert!(function.id(&ast).is_none());

            let ast = parse(
                "export default /* marker\n continued */ async function* f() {}\nexport { f };",
            );
            let function = exported_function(&ast);
            assert_eq!(
                function.function_type(&ast),
                FunctionType::FunctionDeclaration
            );
            assert!(function.r#async(&ast));
            assert!(function.generator(&ast));
            assert_eq!(ast.get_utf8(function.id(&ast).unwrap().name(&ast)), "f");
            assert_eq!(ast.root_program().body(&ast).len(), 2);
            let export = ast.root_program().body(&ast).get_node(&ast, 1).unwrap();
            assert!(export.as_export_named_declaration(&ast).is_some());

            let ast = parse("export default async\nfunction f() {}");
            let ExportDefaultDeclarationKindData::Expr(expression) = declaration(&ast) else {
                panic!("expected a default-exported expression");
            };
            assert!(expression.as_identifier_reference(&ast).is_some());
            assert_eq!(ast.root_program().body(&ast).len(), 2);

            for preserve_parens in [false, true] {
                let options = Options {
                    preserve_parens,
                    ..options
                };
                let source = "export default (async function f() {});";
                let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
                assert!(result.diagnostics.is_empty());
                let ast = result.ast;
                let ExportDefaultDeclarationKindData::Expr(expression) = declaration(&ast) else {
                    panic!("expected a default-exported expression");
                };
                let expression = if preserve_parens {
                    expression
                        .as_parenthesized_expression(&ast)
                        .unwrap()
                        .expression(&ast)
                } else {
                    expression
                };
                let function = expression.as_function(&ast).unwrap();
                assert_eq!(
                    function.function_type(&ast),
                    FunctionType::FunctionExpression
                );
                assert!(function.r#async(&ast));
            }
        }
    }

    #[test]
    fn parenthesized_bindings() {
        fn recovery(
            source: &str,
            options: Options,
        ) -> (Vec<Span>, usize, usize, String, String, String) {
            let allocator = Allocator::new();
            let parsed = Parser::init(&allocator, source, options, TokenParserConfig).parse();
            let errors = parsed.diagnostics.iter().map(|error| error.span).collect();
            let ast = &parsed.ast;
            let body = ast.root_program().body(ast);
            let arrow = body
                .get_node(ast, 0)
                .unwrap()
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast)
                .as_arrow_function_expression(ast)
                .unwrap();
            let parameters = arrow.params(ast).items(ast);
            let first = parameters
                .get_node(ast, 0)
                .unwrap()
                .as_formal_parameter(ast)
                .unwrap();
            let binding = first.pattern(ast).as_binding_identifier(ast).unwrap();
            let name = ast.get_utf8(binding.name(ast)).to_owned();
            let last = body.get_node(ast, 1).unwrap().span(ast);
            let after = source[last.start as usize..last.end as usize].to_owned();
            let tokens = token_texts(source, &parsed.tokens).join(" ");
            (errors, body.len(), parameters.len(), name, after, tokens)
        }
        for preserve_parens in [false, true] {
            let javascript = Options {
                preserve_parens,
                ..Options::default()
            };
            let typescript = Options {
                lang: Lang::Ts,
                ..javascript
            };
            for options in [javascript, typescript] {
                let errors = error_spans("([(a)] = [], {b = (c) = 1}) => {};", options);
                assert_eq!(errors, ["a"]);

                let errors = error_spans("([(a), (b)] = []) => {};", options);
                assert_eq!(errors, ["a", "b"]);

                let errors = error_spans("([(a)] = [], [(b)] = []) => {};", options);
                assert_eq!(errors, ["a", "b"]);

                let errors = error_spans("([a] = [], {b = (c) = 1}) => {};", options);
                assert!(errors.is_empty());

                let errors = error_spans("[(a)] = [];", options);
                assert!(errors.is_empty());

                let errors = error_spans("([(a)] = () => { return; }) => {};", options);
                assert_eq!(errors, ["a"]);

                let source = "((a) = 1, {b = (c) = 1}) => {};\nafter;";
                let (errors, statements, parameters, name, after, tokens) =
                    recovery(source, options);
                assert_eq!(errors, [Span::new(2, 3)]);
                assert_eq!(statements, 2);
                assert_eq!(parameters, 2);
                assert_eq!(name, "a");
                assert_eq!(after, "after;");
                assert_eq!(tokens, "( ( a ) = 1 , { b = ( c ) = 1 } ) => { } ; after ;");
            }

            // A rejected generic arrow must roll back its cover candidates too.
            let errors = error_spans("<T>(x, a = (b) = ([(c)] = []) => c);", typescript);
            assert_eq!(errors, ["c"]);
        }
    }

    #[test]
    fn formal_parameters() {
        fn class_members(ast: &Ast<'_>) -> SubRange<ClassElement> {
            let class = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap()
                .as_class(ast)
                .unwrap();
            class.body(ast).body(ast)
        }

        fn parameter_kinds(source: &str) -> Vec<FormalParameterKind> {
            struct Parameters<'ast, 'a> {
                ast: &'ast Ast<'a>,
                kinds: Vec<FormalParameterKind>,
            }

            impl<'a> Visit<'a> for Parameters<'_, 'a> {
                fn ast(&self) -> &Ast<'a> {
                    self.ast
                }

                fn visit_formal_parameters(&mut self, parameters: FormalParameters) {
                    self.kinds.push(parameters.kind(self.ast));
                    parameters.visit_children_with(self);
                }
            }

            let allocator = Allocator::new();
            let options = Options {
                source_type: SourceType::Script,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            assert!(parsed.diagnostics.is_empty());
            let mut visitor = Parameters {
                ast: &parsed.ast,
                kinds: Vec::new(),
            };
            parsed.ast.root_program().visit_with(&mut visitor);
            visitor.kinds
        }

        let kinds = parameter_kinds("function f(a,a){}");
        assert_eq!(kinds, [FormalParameterKind::FormalParameters]);

        let kinds = parameter_kinds("function* f(a,a){}");
        assert_eq!(kinds, [FormalParameterKind::FormalParameters]);

        let kinds = parameter_kinds("async function f(a,a){}");
        assert_eq!(kinds, [FormalParameterKind::FormalParameters]);

        let kinds = parameter_kinds("async function* f(a,a){}");
        assert_eq!(kinds, [FormalParameterKind::FormalParameters]);

        let kinds = parameter_kinds("(function* (a,a){});");
        assert_eq!(kinds, [FormalParameterKind::FormalParameters]);

        let kinds = parameter_kinds("(async function(a,a){});");
        assert_eq!(kinds, [FormalParameterKind::FormalParameters]);

        let kinds = parameter_kinds("(async function*(a,a){});");
        assert_eq!(kinds, [FormalParameterKind::FormalParameters]);

        let kinds = parameter_kinds("({*f(a,b){}});");
        assert_eq!(kinds, [FormalParameterKind::UniqueFormalParameters]);

        let kinds = parameter_kinds("({async f(a,b){}});");
        assert_eq!(kinds, [FormalParameterKind::UniqueFormalParameters]);

        let kinds = parameter_kinds("({async *f(a,b){}});");
        assert_eq!(kinds, [FormalParameterKind::UniqueFormalParameters]);

        let kinds = parameter_kinds("(a,b)=>{};");
        assert_eq!(kinds, [FormalParameterKind::ArrowFormalParameters]);

        let kinds = parameter_kinds("async (a,b)=>{};");
        assert_eq!(kinds, [FormalParameterKind::ArrowFormalParameters]);

        let allocator = Allocator::new();
        let source = "class C { constructor(this: C, value) { return; } }\nafter;";
        let parsed = parse_tokens(&allocator, source);
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].span, Span::new(22, 26));
        let body = ast.root_program().body(ast);
        assert_eq!(body.len(), 2);
        let after = body
            .get_node(ast, 1)
            .unwrap()
            .as_expression_statement(ast)
            .unwrap()
            .expression(ast)
            .as_identifier_reference(ast)
            .unwrap();
        assert_eq!(ast.get_utf8(after.name(ast)), "after");
        let members = class_members(ast);
        assert_eq!(members.len(), 1);
        let method = members
            .get_node(ast, 0)
            .unwrap()
            .as_method_definition(ast)
            .unwrap();
        assert_eq!(method.kind(ast), MethodDefinitionKind::Constructor);
        let function = method.value(ast);
        let receiver = function.this_param(ast).unwrap();
        assert_eq!(receiver.span(ast), Span::new(22, 29));
        assert!(receiver.type_annotation(ast).is_some());
        let params = function.params(ast);
        assert_eq!(params.items(ast).len(), 1);
        assert!(params.rest(ast).is_none());
        let parameter = params
            .items(ast)
            .get_node(ast, 0)
            .unwrap()
            .as_formal_parameter(ast)
            .unwrap();
        let binding = parameter.pattern(ast).as_binding_identifier(ast).unwrap();
        assert_eq!(ast.get_utf8(binding.name(ast)), "value");
        let statements = function.body(ast).body(ast);
        assert_eq!(statements.len(), 1);
        assert!(
            statements
                .get_node(ast, 0)
                .unwrap()
                .as_return_statement(ast)
                .is_some()
        );

        let parsed = parse_tokens(&allocator, "class C { constructor(this: C, ...args); }");
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(ast.root_program().body(ast).len(), 1);
        let members = class_members(ast);
        assert_eq!(members.len(), 1);
        let method = members
            .get_node(ast, 0)
            .unwrap()
            .as_ts_method_definition(ast)
            .unwrap();
        assert_eq!(method.kind(ast), MethodDefinitionKind::Constructor);
        let function = method.value(ast);
        let receiver = function.this_param(ast).unwrap();
        assert!(receiver.type_annotation(ast).is_some());
        let params = function.params(ast);
        assert!(params.items(ast).is_empty());
        let rest = params.rest(ast).unwrap();
        let binding = rest.argument(ast).as_binding_identifier(ast).unwrap();
        assert_eq!(ast.get_utf8(binding.name(ast)), "args");

        let source = "(this: C) => {};";
        let parsed = parse_tokens(&allocator, source);
        assert!(!parsed.diagnostics.is_empty());
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());

        let source = "type C = new (this: C) => C;";
        let parsed = parse_tokens(&allocator, source);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());

        let source = "interface C { new (this: C): C; }";
        let parsed = parse_tokens(&allocator, source);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());
    }

    #[test]
    fn jsx_primary_expressions() {
        fn parse(source: &str, options: Options) -> (bool, usize) {
            let allocator = Allocator::new();
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            let statements = parsed.ast.root_program().body(&parsed.ast).len();
            (!parsed.diagnostics.is_empty(), statements)
        }

        let jsx = Options {
            lang: Lang::Jsx,
            ..Options::default()
        };
        let tsx = Options {
            lang: Lang::Tsx,
            ..jsx
        };

        assert!(!has_errors("new <a/>();", jsx));

        assert!(!has_errors("class A extends <a/> {}", jsx));

        let (errors, statements) = parse("new <a/>();", tsx);
        assert!(errors);
        assert_eq!(statements, 1);

        let (errors, statements) = parse("class A extends <a/> {}", tsx);
        assert!(errors);
        assert_eq!(statements, 1);
    }

    #[test]
    fn jsx_text() {
        fn parse_text(source: &str, lang: Lang) -> Vec<(String, String)> {
            let allocator = Allocator::new();
            let options = Options {
                lang,
                source_type: SourceType::Module,
                ..Options::default()
            };
            let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            assert!(result.diagnostics.is_empty());
            let ast = &result.ast;
            assert_eq!(ast.root_program().body(ast).len(), 1);
            let declaration = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap()
                .as_variable_declaration(ast)
                .unwrap();
            let expression = declaration
                .declarators(ast)
                .get_node(ast, 0)
                .unwrap()
                .init(ast)
                .unwrap();
            let children = if let Some(element) = expression.as_jsx_element(ast) {
                element.children(ast)
            } else {
                expression.as_jsx_fragment(ast).unwrap().children(ast)
            };
            children
                .iter()
                .filter_map(|id| {
                    let text = ast.get_node_in_sub_range(id).as_jsx_text(ast)?;
                    let span = text.span(ast);
                    Some((
                        ast.get_utf8(text.value(ast)).to_owned(),
                        source[span.start as usize..span.end as usize].to_owned(),
                    ))
                })
                .collect()
        }

        let text = parse_text("const node = <span>=</span>;", Lang::Tsx);
        assert_eq!(text, [("=".into(), "=".into())]);

        let text = parse_text("const node = <span>=</span>;", Lang::Jsx);
        assert_eq!(text, [("=".into(), "=".into())]);

        let text = parse_text("const node = <>=</>;", Lang::Tsx);
        assert_eq!(text, [("=".into(), "=".into())]);

        let text = parse_text("const node = <span><br/>=</span>;", Lang::Tsx);
        assert_eq!(text, [("=".into(), "=".into())]);

        let text = parse_text("const node = <Component<T>>=</Component>;", Lang::Tsx);
        assert_eq!(text, [("=".into(), "=".into())]);
    }

    #[test]
    fn jsx_attributes() {
        fn parse_tag_ends(source: &str, lang: Lang) -> (u32, u32) {
            let allocator = Allocator::new();
            let options = Options {
                lang,
                source_type: SourceType::Module,
                ..Options::default()
            };
            let ast = Parser::init(&allocator, source, options, NoTokenParserConfig)
                .parse_expression_fragment(FragmentContext::TopLevel)
                .unwrap();
            let element = ast.root_expression().as_jsx_element(&ast).unwrap();
            let opening = element.opening_element(&ast);
            let attributes = opening.attributes(&ast);
            assert_eq!(attributes.len(), 1);
            let attribute = attributes
                .get_node(&ast, 0)
                .unwrap()
                .as_jsx_attribute(&ast)
                .unwrap();
            let value = attribute.value(&ast).unwrap().as_jsx_element(&ast).unwrap();
            let name = value
                .opening_element(&ast)
                .name(&ast)
                .as_jsx_identifier(&ast)
                .unwrap();
            assert_eq!(ast.get_utf8(name.name(&ast)), "e");
            assert!(value.children(&ast).is_empty());
            assert!(element.children(&ast).is_empty());
            (
                value.closing_element(&ast).unwrap().span(&ast).end,
                opening.span(&ast).end,
            )
        }

        let ends = parse_tag_ends("<div attr=<e></e>></div>", Lang::Jsx);
        assert_eq!(ends, (17, 18));

        let ends = parse_tag_ends("<div attr=<e></e>></div>", Lang::Tsx);
        assert_eq!(ends, (17, 18));
    }

    #[test]
    fn super_recovery() {
        fn expression(ast: &Ast<'_>, index: usize) -> Expr {
            ast.root_program()
                .body(ast)
                .get_node(ast, index)
                .unwrap()
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast)
        }

        let allocator = Allocator::new();
        let source = "super;\nafter;";
        let parsed = parse_tokens(&allocator, source);
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].span, Span::new(5, 6));
        assert_eq!(ast.root_program().body(ast).len(), 2);
        assert!(expression(ast, 0).is_super(ast));
        let after = expression(ast, 1).as_identifier_reference(ast).unwrap();
        assert_eq!(ast.get_utf8(after.name(ast)), "after");
        assert_eq!(
            token_texts(source, &parsed.tokens),
            ["super", ";", "after", ";"]
        );

        let parsed = parse_tokens(&allocator, "super\nafter;");
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(ast.root_program().body(ast).len(), 2);
        assert!(expression(ast, 0).is_super(ast));
        let after = expression(ast, 1).as_identifier_reference(ast).unwrap();
        assert_eq!(ast.get_utf8(after.name(ast)), "after");

        let parsed = parse_tokens(&allocator, "super");
        let ast = &parsed.ast;
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].span, Span::new(5, 5));
        assert_eq!(ast.root_program().body(ast).len(), 1);
        assert!(expression(ast, 0).is_super(ast));

        let parsed = parse_tokens(&allocator, "super value;\nafter;");
        assert!(!parsed.diagnostics.is_empty());
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());

        let parsed = parse_tokens(&allocator, "super.;\nafter;");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.ast.root_program().body(&parsed.ast).is_empty());
    }

    #[test]
    fn new_expression() {
        fn parse<'a>(allocator: &'a Allocator, source: &'a str, options: Options) -> Ast<'a> {
            Parser::init(allocator, source, options, NoTokenParserConfig)
                .parse_expression_fragment(FragmentContext::TopLevel)
                .unwrap()
        }

        let allocator = Allocator::new();
        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let ast = parse(&allocator, "new parent.components!.KeepAlive()", options);
        let expression = ast.root_expression().as_new_expression(&ast).unwrap();
        let callee = expression.callee(&ast).as_member_expression(&ast).unwrap();
        let property = callee.property(&ast).as_identifier_name(&ast).unwrap();
        assert_eq!(ast.get_utf8(property.name(&ast)), "KeepAlive");
        let non_null = callee.object(&ast).as_ts_non_null_expression(&ast).unwrap();
        let member = non_null
            .expression(&ast)
            .as_member_expression(&ast)
            .unwrap();
        let property = member.property(&ast).as_identifier_name(&ast).unwrap();
        assert_eq!(ast.get_utf8(property.name(&ast)), "components");
        let object = member.object(&ast).as_identifier_reference(&ast).unwrap();
        assert_eq!(ast.get_utf8(object.name(&ast)), "parent");
        assert!(expression.arguments(&ast).is_empty());

        let ast = parse(&allocator, "new async(1)", options);
        let expression = ast.root_expression().as_new_expression(&ast).unwrap();
        assert!(
            expression
                .callee(&ast)
                .as_identifier_reference(&ast)
                .is_some()
        );
        assert_eq!(expression.arguments(&ast).len(), 1);

        let source = "new async function*(){}.constructor(1)";
        let ast = parse(&allocator, source, options);
        let expression = ast.root_expression().as_new_expression(&ast).unwrap();
        let member = expression.callee(&ast).as_member_expression(&ast).unwrap();
        let function = member.object(&ast).as_function(&ast).unwrap();
        assert!(function.r#async(&ast));
        assert!(function.generator(&ast));
        assert_eq!(function.span(&ast), Span::new(4, 23));
        assert_eq!(expression.arguments(&ast).len(), 1);
    }

    #[test]
    fn meta_property_contexts() {
        for preserve_parens in [false, true] {
            for lang in [Lang::Js, Lang::Ts] {
                let module = Options {
                    lang,
                    preserve_parens,
                    ..Options::default()
                };
                let script = Options {
                    source_type: SourceType::Script,
                    ..module
                };
                let commonjs = Options {
                    source_type: SourceType::CommonJs,
                    ..module
                };

                let errors = error_spans("import.meta;", module);
                assert!(errors.is_empty());

                let errors = error_spans("import.meta;", script);
                assert_eq!(errors, ["import.meta"]);

                let errors = error_spans("import.meta;", commonjs);
                assert_eq!(errors, ["import.meta"]);

                let errors = error_spans("function f() { return import.meta; }", script);
                assert_eq!(errors, ["import.meta"]);

                let errors = error_spans("() => import.meta;", module);
                assert!(errors.is_empty());

                let errors = error_spans("new.target;", module);
                assert_eq!(errors, ["new.target"]);

                let errors = error_spans("new.target;", script);
                assert_eq!(errors, ["new.target"]);

                let errors = error_spans("new.target;", commonjs);
                assert!(errors.is_empty());

                let errors = error_spans("() => new.target;", module);
                assert_eq!(errors, ["new.target"]);

                let errors = error_spans("(value = new.target) => value;", module);
                assert_eq!(errors, ["new.target"]);

                let errors = error_spans("function f(value = new.target) {}", module);
                assert!(errors.is_empty());

                let errors = error_spans("function f() { return () => new.target; }", module);
                assert!(errors.is_empty());

                let errors = error_spans("({ method(value = new.target) {} });", module);
                assert!(errors.is_empty());

                let errors = error_spans("class C { field = () => new.target; }", module);
                assert!(errors.is_empty());

                let errors = error_spans("class C { static { new.target; } }", module);
                assert!(errors.is_empty());

                let errors = error_spans("class C { [new.target]; }", module);
                assert_eq!(errors, ["new.target"]);

                let errors = error_spans("class C extends new.target {}", module);
                assert_eq!(errors, ["new.target"]);
            }

            for lang in [Lang::Ts, Lang::Dts] {
                let commonjs = Options {
                    source_type: SourceType::CommonJs,
                    lang,
                    preserve_parens,
                    ..Options::default()
                };
                let unambiguous = Options {
                    source_type: SourceType::Unambiguous,
                    ..commonjs
                };

                let errors = error_spans("namespace N { new.target; }", commonjs);
                assert!(errors.is_empty());

                let errors = error_spans("namespace N { new.target; }", unambiguous);
                assert_eq!(errors, ["new.target"]);

                let errors = error_spans("namespace N { return; }", commonjs);
                assert_eq!(errors, ["return"]);
            }
        }
    }

    #[test]
    fn generic_postfix() {
        fn parse<'a>(
            allocator: &'a Allocator,
            source: &'a str,
            options: Options,
        ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
            Parser::init(allocator, source, options, NoTokenParserConfig)
                .parse_expression_fragment(FragmentContext::TopLevel)
        }

        let allocator = Allocator::new();
        let ts_options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let tsx_options = Options {
            lang: Lang::Tsx,
            ..ts_options
        };
        for options in [ts_options, tsx_options] {
            for preserve_parens in [false, true] {
                let options = Options {
                    preserve_parens,
                    ..options
                };

                let ast = parse(&allocator, "new f<T>`x`", options).unwrap();
                let new = ast.root_expression().as_new_expression(&ast).unwrap();
                let tag = new
                    .callee(&ast)
                    .as_tagged_template_expression(&ast)
                    .unwrap();
                assert!(new.type_arguments(&ast).is_none());
                assert!(new.arguments(&ast).is_empty());
                assert!(tag.tag(&ast).is_identifier_reference(&ast));
                assert!(tag.type_arguments(&ast).is_some());

                let ast = parse(&allocator, "new f<T>`x`(y)", options).unwrap();
                let new = ast.root_expression().as_new_expression(&ast).unwrap();
                assert!(new.callee(&ast).is_tagged_template_expression(&ast));
                assert_eq!(new.arguments(&ast).len(), 1);

                let ast = parse(&allocator, "new f<T>`x`.y", options).unwrap();
                let new = ast.root_expression().as_new_expression(&ast).unwrap();
                let member = new.callee(&ast).as_member_expression(&ast).unwrap();
                assert!(member.object(&ast).is_tagged_template_expression(&ast));

                let ast = parse(&allocator, "new f<T>`x`<U>(y)", options).unwrap();
                let new = ast.root_expression().as_new_expression(&ast).unwrap();
                let tag = new
                    .callee(&ast)
                    .as_tagged_template_expression(&ast)
                    .unwrap();
                assert!(new.type_arguments(&ast).is_some());
                assert!(tag.type_arguments(&ast).is_some());
                assert_eq!(new.arguments(&ast).len(), 1);

                let ast = parse(&allocator, "new f<T>[0]", options).unwrap();
                let comparison = ast.root_expression().as_binary_expression(&ast).unwrap();
                let comparison = comparison.left(&ast).as_binary_expression(&ast).unwrap();
                assert!(comparison.left(&ast).is_new_expression(&ast));

                let parsed = parse(&allocator, "new f<T>?.x", options);
                assert!(parsed.is_err());

                let parsed = parse(&allocator, "new f<T>?.()", options);
                assert!(parsed.is_err());

                let parsed = parse(&allocator, "new f<T>?.[x]", options);
                assert!(parsed.is_err());

                let parsed = parse(&allocator, "new f<T>.x", options);
                assert!(parsed.is_err());

                let parsed = parse(&allocator, "new f<T>()?.x", options);
                assert!(parsed.is_ok());

                let parsed = parse(&allocator, "new (f<T>).x()", options);
                assert!(parsed.is_ok());

                let parsed = parse(&allocator, "f?.x<T>`y`", options);
                assert!(parsed.is_err());

                let parsed = parse(&allocator, "f?.()<T>`y`", options);
                assert!(parsed.is_err());

                let parsed = parse(&allocator, "f?.x!<T>`y`", options);
                assert!(parsed.is_err());

                let parsed = parse(&allocator, "(f?.x)<T>`y`", options);
                assert!(parsed.is_ok());

                let ast = parse(&allocator, "f<T>?.()", options).unwrap();
                let chain = ast.root_expression().as_chain_expression(&ast).unwrap();
                let call = chain.expression(&ast).as_call_expression(&ast).unwrap();
                assert!(call.optional(&ast));
                assert!(call.type_arguments(&ast).is_none());
                assert!(call.callee(&ast).is_ts_instantiation_expression(&ast));
            }
        }
    }

    #[test]
    fn class_heritage() {
        fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> (Ast<'a>, Expr) {
            let result =
                Parser::init(allocator, source, Options::default(), NoTokenParserConfig).parse();
            assert!(result.diagnostics.is_empty());
            let ast = result.ast;
            let statement = ast.root_program().body(&ast).get_node(&ast, 0).unwrap();
            let StmtData::Declaration(declaration) = ast.stmt_data(statement) else {
                panic!("expected a class declaration");
            };
            let DeclData::Class(class) = ast.decl_data(declaration) else {
                panic!("expected a class declaration");
            };
            let superclass = class
                .super_class(&ast)
                .expect("class must have a heritage expression");
            (ast, superclass)
        }

        let allocator = Allocator::new();
        let (ast, superclass) = parse(&allocator, "class C extends Base {}");
        assert!(superclass.as_identifier_reference(&ast).is_some());

        let (ast, superclass) = parse(&allocator, "class C extends async() {}");
        let call = superclass.as_call_expression(&ast).unwrap();
        assert!(call.callee(&ast).as_identifier_reference(&ast).is_some());

        let source = "class C extends async function(){}.constructor {}";
        let (ast, superclass) = parse(&allocator, source);
        let member = superclass.as_member_expression(&ast).unwrap();
        let property = member.property(&ast).as_identifier_name(&ast).unwrap();
        assert_eq!(ast.get_utf8(property.name(&ast)), "constructor");
        let function = member.object(&ast).as_function(&ast).unwrap();
        assert!(function.r#async(&ast));
        assert!(!function.generator(&ast));
    }

    #[test]
    fn decorated_class_expression() {
        let allocator = Allocator::new();
        let source = "new @dec class {}(value)";
        let ast = Parser::init(&allocator, source, Options::default(), NoTokenParserConfig)
            .parse_expression_fragment(FragmentContext::TopLevel)
            .unwrap();
        let expression = ast.root_expression().as_new_expression(&ast).unwrap();
        let class = expression.callee(&ast).as_class(&ast).unwrap();
        assert_eq!(class.class_type(&ast), ClassType::ClassExpression);
        assert_eq!(class.decorators(&ast).len(), 1);
        assert_eq!(expression.arguments(&ast).len(), 1);
    }

    #[test]
    fn class_modifiers() {
        fn parse_fields(source: &str, lang: Lang) -> Vec<(String, bool)> {
            let allocator = Allocator::new();
            let options = Options {
                lang,
                ..Options::default()
            };
            let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            assert!(result.diagnostics.is_empty());
            let ast = &result.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let declaration = statement.as_declaration(ast).unwrap();
            let class = declaration.as_class(ast).unwrap();
            let body = class.body(ast).body(ast);
            body.iter()
                .map(|id| {
                    let element = ast.get_node_in_sub_range(id);
                    let field = element.as_property_definition(ast).unwrap();
                    let key = field.key(ast).as_identifier_name(ast).unwrap();
                    let name = ast.get_utf8(key.name(ast)).to_owned();
                    (name, field.r#static(ast))
                })
                .collect()
        }

        let fields = parse_fields("class A { st\\u0061tic\nfoo }", Lang::Js);
        assert_eq!(fields, [("static".into(), false), ("foo".into(), false)]);

        let fields = parse_fields("class A { st\\u0061tic\nfoo }", Lang::Ts);
        assert_eq!(fields, [("static".into(), false), ("foo".into(), false)]);

        let fields = parse_fields("class A { static\nfoo }", Lang::Js);
        assert_eq!(fields, [("foo".into(), true)]);

        let fields = parse_fields("class A { static st\\u0061tic\nfoo }", Lang::Js);
        assert_eq!(fields, [("static".into(), true), ("foo".into(), false)]);

        let fields = parse_fields("class A { g\\u0065t\nfoo }", Lang::Js);
        assert_eq!(fields, [("get".into(), false), ("foo".into(), false)]);

        let fields = parse_fields("class A { s\\u0065t\nfoo }", Lang::Js);
        assert_eq!(fields, [("set".into(), false), ("foo".into(), false)]);

        let fields = parse_fields("class A { a\\u0073ync\nfoo }", Lang::Js);
        assert_eq!(fields, [("async".into(), false), ("foo".into(), false)]);
    }

    #[test]
    fn module_names() {
        fn texts(source: &str, spans: impl Iterator<Item = Span>) -> Vec<&str> {
            spans
                .map(|span| &source[span.start as usize..span.end as usize])
                .collect()
        }

        fn parse(source: &str) -> (Vec<&str>, Vec<NodeKind>) {
            let allocator = Allocator::new();
            let parsed = parse_tokens(&allocator, source);
            let ast = &parsed.ast;
            let errors = parsed.diagnostics.iter().map(|diagnostic| diagnostic.span);
            let body = ast.root_program().body(ast);
            let statements = body
                .iter()
                .map(|id| ast.node_kind(ast.get_node_in_sub_range(id).node_id()));
            (texts(source, errors), statements.collect())
        }

        macro_rules! check {
            ($source:expr, errors: [$($error:expr),*], statements: [$($statement:expr),*]) => {
                assert_eq!(parse($source), (vec![$($error),*], vec![$($statement),*]))
            };
        }

        use NodeKind::{ExportAllDeclaration, ExportNamedDeclaration, ImportDeclaration};

        check!(r#"import { "\uD800" as x } from "m";"#,
            errors: [r#""\uD800""#], statements: [ImportDeclaration]);

        check!(r#"export { x as "\uDC00" };"#,
            errors: [r#""\uDC00""#], statements: [ExportNamedDeclaration]);

        check!(r#"export * as "\uD800" from "m";"#,
            errors: [r#""\uD800""#], statements: [ExportAllDeclaration]);
    }

    #[test]
    fn binding_identifier_recovery() {
        let module = Options {
            lang: Lang::Js,
            source_type: SourceType::Module,
            ..Options::default()
        };
        let typescript = Options {
            lang: Lang::Ts,
            ..module
        };

        assert!(has_errors(r#"import * as "\uD800" ns from "x";"#, module));

        assert!(has_errors(r#"import * as "\u{D800}" from "x";"#, module));

        assert!(has_errors(r#"import `\uDC00` from "x";"#, module));

        assert!(has_errors(r#"type T = infer "\uD800";"#, typescript));

        assert!(has_errors(r#"function f<"\uD800">() {}"#, typescript));

        assert!(has_errors(r#"import x = "\uD800";"#, typescript));

        assert!(!has_errors(r#"import a from "x";"#, module));

        assert!(!has_errors(r#"import * as ns from "x";"#, module));

        assert!(!has_errors("type T = infer U;", typescript));
    }

    #[test]
    fn import_assertions() {
        let allocator = Allocator::new();
        let options = Options {
            source_type: SourceType::Module,
            lang: Lang::Js,
            ..Options::default()
        };
        let source = "import 'x'\nassert ({});";
        let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.ast.root_program().body(&result.ast).len(), 2);
    }

    #[test]
    fn in_grammar_context() {
        let allocator = Allocator::new();
        let source = "for (true || async(...a in b); console.log('parsed'), false;);";
        let options = Options {
            source_type: SourceType::Script,
            ..Options::default()
        };
        let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    }

    #[test]
    fn expression_fragment() {
        fn parse<'a>(allocator: &'a Allocator, source: &'a str, options: Options) -> Ast<'a> {
            Parser::init(allocator, source, options, NoTokenParserConfig)
                .parse_expression_fragment(FragmentContext::TopLevel)
                .unwrap()
        }

        let allocator = Allocator::default();
        let unambiguous = Options {
            source_type: SourceType::Unambiguous,
            ..Options::default()
        };

        let ast = parse(&allocator, "1 + 1", Options::default());
        let root_expression = ast.expr_data(ast.root_expression());
        assert!(matches!(root_expression, ExprData::BinaryExpression(_)));

        let ast = parse(&allocator, "await value", unambiguous);
        let root_expression = ast.expr_data(ast.root_expression());
        assert_eq!(ast.source_type(), SourceType::Module);
        assert!(matches!(root_expression, ExprData::AwaitExpression(_)));

        let ast = parse(&allocator, "value", unambiguous);
        assert_eq!(ast.source_type(), SourceType::Script);
    }

    #[test]
    fn unambiguous_commonjs_expression_fragments() {
        fn parse<'a>(
            allocator: &'a Allocator,
            source: &'a str,
            lang: Lang,
            source_type: SourceType,
            context: FragmentContext,
        ) -> Ast<'a> {
            Parser::init(
                allocator,
                source,
                Options {
                    lang,
                    source_type,
                    ..Options::default()
                },
                NoTokenParserConfig,
            )
            .parse_expression_fragment(context)
            .unwrap()
        }

        fn assert_resolves_as(
            source: &str,
            lang: Lang,
            context: FragmentContext,
            expected: SourceType,
        ) {
            let unambiguous_allocator = Allocator::new();
            let unambiguous = parse(
                &unambiguous_allocator,
                source,
                lang,
                SourceType::Unambiguous,
                context,
            );
            assert_eq!(unambiguous.source_type(), expected, "{source}");

            let explicit_allocator = Allocator::new();
            let explicit = parse(&explicit_allocator, source, lang, expected, context);
            assert_eq!(
                serialize_ast(&unambiguous),
                serialize_ast(&explicit),
                "{source}"
            );
        }

        assert_resolves_as(
            "new.target",
            Lang::Js,
            FragmentContext::TopLevel,
            SourceType::CommonJs,
        );

        assert_resolves_as(
            "new.target + await + 0",
            Lang::Js,
            FragmentContext::TopLevel,
            SourceType::CommonJs,
        );

        assert_resolves_as(
            "new.target",
            Lang::Js,
            FragmentContext::Function,
            SourceType::Script,
        );

        let allocator = Allocator::new();
        let options = Options {
            lang: Lang::Dts,
            source_type: SourceType::Unambiguous,
            ..Options::default()
        };
        let result = Parser::init(&allocator, "new.target", options, NoTokenParserConfig)
            .parse_expression_fragment(FragmentContext::TopLevel);
        assert!(result.is_err());
    }

    #[test]
    fn disallow_unerasable_assertions() {
        fn parse_typescript_diagnostics(source: &str) -> Vec<Diagnostic<'static>> {
            let allocator = Allocator::new();
            Parser::init(
                &allocator,
                source,
                Options {
                    lang: Lang::Ts,
                    ..Options::default()
                },
                NoTokenParserConfig,
            )
            .parse()
            .diagnostics
        }

        let diagnostics = parse_typescript_diagnostics("1 + 1 as number * 2");
        assert!(!diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("1 + 1 satisfies number * 2");
        assert!(!diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("1 + 1 as unknown as number * 2");
        assert!(!diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("1 >> 1 as number + 2");
        assert!(!diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("2 * 3 as number * 2");
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("2 * 3 satisfies number * 2");
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("2 ** 3 as number ** 2");
        assert!(!diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("2 ** 3 satisfies number ** 2");
        assert!(!diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("2 ** 3 as unknown as number ** 2");
        assert!(!diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("1 * 1 as number + 2");
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse_typescript_diagnostics("1 as number * 2");
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn await_using_context() {
        fn parse(source: &str, source_type: SourceType) -> Vec<Diagnostic<'static>> {
            let allocator = Allocator::new();
            Parser::init(
                &allocator,
                source,
                Options {
                    source_type,
                    ..Options::default()
                },
                NoTokenParserConfig,
            )
            .parse()
            .diagnostics
        }

        let script = SourceType::Script;

        let diagnostics = parse("await using in object;", script);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");

        let diagnostics = parse("function f() { await using instanceof C; }", script);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");

        let diagnostics = parse("async function f() { await using in object; }", script);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");

        let diagnostics = parse("await using in object;", SourceType::Module);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn private_name_early_errors() {
        let allocator = Allocator::new();
        let result = Parser::init(
            &allocator,
            "class C { #x; m() { a + #x in object; } }",
            Options {
                source_type: SourceType::Script,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();

        assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn unambiguous() {
        fn assert_unambiguous_source_type_with_lang(
            source: &str,
            lang: Lang,
            expected: SourceType,
        ) {
            let unambiguous_allocator = Allocator::new();
            let unambiguous_options = Options {
                source_type: SourceType::Unambiguous,
                lang,
                ..Options::default()
            };
            let unambiguous_result = Parser::init(
                &unambiguous_allocator,
                source,
                unambiguous_options,
                NoTokenParserConfig,
            )
            .parse();
            assert!(unambiguous_result.diagnostics.is_empty(),);
            assert_eq!(unambiguous_result.ast.source_type(), expected, "{source}");

            let explicit_allocator = Allocator::new();
            let explicit_options = Options {
                source_type: expected,
                lang,
                ..Options::default()
            };
            let explicit_result = Parser::init(
                &explicit_allocator,
                source,
                explicit_options,
                NoTokenParserConfig,
            )
            .parse();
            assert!(explicit_result.diagnostics.is_empty(),);

            let unambiguous_ast = serialize_ast(&unambiguous_result.ast);
            let explicit_ast = serialize_ast(&explicit_result.ast);
            assert!(unambiguous_ast == explicit_ast);
        }

        fn assert_unambiguous_source_type_js(source: &str, expected: SourceType) {
            assert_unambiguous_source_type_with_lang(source, Lang::Js, expected);
        }

        fn assert_unambiguous_source_type(source: &str, expected: SourceType) {
            assert_unambiguous_source_type_with_lang(source, Lang::Ts, expected);
        }

        assert_unambiguous_source_type_js(
            "const f = (x = await + value) => x;",
            SourceType::Script,
        );
        assert_unambiguous_source_type_js("(x = await value);", SourceType::Module);
        assert_unambiguous_source_type_js("(a = (b = (c = await value)));", SourceType::Module);

        assert_unambiguous_source_type("const value = 1;", SourceType::Script);
        assert_unambiguous_source_type("const f = (x = await + value) => x;", SourceType::Script);
        assert_unambiguous_source_type("import value from 'value';", SourceType::Module);
        assert_unambiguous_source_type("export const value = 1;", SourceType::Module);
        assert_unambiguous_source_type("const value = import.meta.url;", SourceType::Module);
        assert_unambiguous_source_type("import('value');", SourceType::Script);
        assert_unambiguous_source_type("import value = require('value');", SourceType::Module);
        assert_unambiguous_source_type("import Value = Namespace.Value;", SourceType::Script);
        assert_unambiguous_source_type("export as namespace Foo", SourceType::Module);
        assert_unambiguous_source_type("await;", SourceType::Script);
        assert_unambiguous_source_type("await value;", SourceType::Module);
        assert_unambiguous_source_type("await\nvalue;", SourceType::Script);
        assert_unambiguous_source_type("for (await of values) {}", SourceType::Script);
        assert_unambiguous_source_type("await+0;", SourceType::Script);
        assert_unambiguous_source_type("await-0;", SourceType::Script);
        assert_unambiguous_source_type("await+=0;", SourceType::Script);
        assert_unambiguous_source_type("await-=0;", SourceType::Script);
        assert_unambiguous_source_type("await*=0;", SourceType::Script);
        assert_unambiguous_source_type("await/=0;", SourceType::Script);
        assert_unambiguous_source_type("await**0;", SourceType::Script);
        assert_unambiguous_source_type("await&0;", SourceType::Script);
        assert_unambiguous_source_type("await|0;", SourceType::Script);
        assert_unambiguous_source_type("await&&0;", SourceType::Script);
        assert_unambiguous_source_type("await||0;", SourceType::Script);
        assert_unambiguous_source_type("await<0;", SourceType::Script);
        assert_unambiguous_source_type("await>0;", SourceType::Script);
        assert_unambiguous_source_type("await(0);", SourceType::Script);
        assert_unambiguous_source_type("await[0];", SourceType::Script);
        assert_unambiguous_source_type("await`0`;", SourceType::Script);
        assert_unambiguous_source_type("await`0${0}`;", SourceType::Script);
        assert_unambiguous_source_type("await /1/u; var await = 1, u = 1;", SourceType::Script);
        assert_unambiguous_source_type("await! + value;", SourceType::Script);
        assert_unambiguous_source_type("await<T>(value);", SourceType::Script);
        assert_unambiguous_source_type("await++a;", SourceType::Module);
        assert_unambiguous_source_type("await--a;", SourceType::Module);
        assert_unambiguous_source_type("await? 0 : 1;", SourceType::Script);
        assert_unambiguous_source_type("await?.();", SourceType::Script);
        assert_unambiguous_source_type("await => value;", SourceType::Script);
        assert_unambiguous_source_type("await as T;", SourceType::Script);
        assert_unambiguous_source_type("await satisfies T;", SourceType::Script);
        assert_unambiguous_source_type("await in T;", SourceType::Script);
        assert_unambiguous_source_type("await instanceof Object;", SourceType::Script);
        assert_unambiguous_source_type("class C { field = await + value; }", SourceType::Script);
        assert_unambiguous_source_type(
            "namespace N { const value = await input; }",
            SourceType::Script,
        );
        assert_unambiguous_source_type("enum E { value = await input }", SourceType::Script);
        assert_unambiguous_source_type("{ await value; }", SourceType::Module);
        assert_unambiguous_source_type("for await (const value of values) {}", SourceType::Module);
        assert_unambiguous_source_type(
            "{ for await (const value of values) {} }",
            SourceType::Module,
        );
        assert_unambiguous_source_type("await using value = resource;", SourceType::Module);
        assert_unambiguous_source_type("{ await using value = resource; }", SourceType::Module);
        assert_unambiguous_source_type("using value = resource;", SourceType::Script);
        assert_unambiguous_source_type("async function f() { await value; }", SourceType::Script);
        assert_unambiguous_source_type(
            "async function f() { for await (const value of values) {} }",
            SourceType::Script,
        );
        assert_unambiguous_source_type(
            "async function f() { await using value = resource; }",
            SourceType::Script,
        );
        assert_unambiguous_source_type(
            "async function f() { await value; } await other;",
            SourceType::Module,
        );
        assert_unambiguous_source_type("{ using value = resource; }", SourceType::Script);
        assert_unambiguous_source_type("export {}; await\nvalue;", SourceType::Module);
        assert_unambiguous_source_type("await\nvalue; export {};", SourceType::Module);
        assert_unambiguous_source_type("await + 0; export {};", SourceType::Module);
        assert_unambiguous_source_type("foo(await 0);", SourceType::Module);

        fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> ParseReturn<'a> {
            let parser = Parser::init(
                allocator,
                source,
                Options {
                    source_type: SourceType::Unambiguous,
                    lang: Lang::Js,
                    ..Options::default()
                },
                NoTokenParserConfig,
            );
            parser.parse()
        }

        fn first_function_body(ast: &Ast<'_>) -> FunctionBody {
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let declaration = statement.as_declaration(ast).unwrap();
            let function = declaration.as_function(ast).unwrap();
            function.body(ast)
        }

        let allocator = Allocator::new();
        let result = parse(&allocator, "function f() { await value; }");
        assert_eq!(result.ast.source_type(), SourceType::Module);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(first_function_body(&result.ast).body(&result.ast).len(), 1);

        let result = parse(&allocator, "function f() { yield value; }");
        assert_eq!(result.ast.source_type(), SourceType::Module);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(first_function_body(&result.ast).body(&result.ast).len(), 1);

        let result = parse(&allocator, "return 1; function f() { await value; }");
        assert_eq!(result.ast.source_type(), SourceType::Module);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.ast.root_program().body(&result.ast).len(), 2);

        let allocator = Allocator::new();
        let result = parse(&allocator, "value + await \"unterminated");
        assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);

        let allocator = Allocator::new();
        let result = parse(&allocator, "await\nimport value from 'value';");
        assert_eq!(result.ast.source_type(), SourceType::Module);
        assert!(!result.diagnostics.is_empty());

        let allocator = Allocator::new();
        let result = parse(&allocator, "value\n<!-- comment");
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
        assert_eq!(result.ast.source_type(), SourceType::Script);
        assert_eq!(result.ast.root_program().body(&result.ast).len(), 1);

        let allocator = Allocator::new();
        let result = parse(&allocator, "value\n<!-- comment\nexport {};");
        assert_eq!(result.ast.source_type(), SourceType::Module);
        assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);
    }

    #[test]
    fn unambiguous_recovery() {
        fn parse(source: &str, options: Options) -> (Vec<Span>, SourceType, Vec<NodeKind>) {
            let allocator = Allocator::new();
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            let ast = &parsed.ast;
            let nodes = ast
                .root_program()
                .body(ast)
                .iter()
                .map(|id| {
                    let statement = ast.get_node_in_sub_range(id);
                    let node = match statement.as_expression_statement(ast) {
                        Some(statement) => statement.expression(ast).node_id(),
                        None => statement.node_id(),
                    };
                    ast.node_kind(node)
                })
                .collect();
            let errors = parsed.diagnostics.iter().map(|error| error.span).collect();
            (errors, ast.source_type(), nodes)
        }

        let unambiguous = Options {
            source_type: SourceType::Unambiguous,
            ..Options::default()
        };
        let module = Options {
            source_type: SourceType::Module,
            ..unambiguous
        };

        let (errors, source_type, nodes) = parse("for (await in x);", unambiguous);
        assert!(errors.is_empty());
        assert_eq!(source_type, SourceType::Script);
        assert_eq!(nodes, [NodeKind::ForInStatement]);

        let (errors, source_type, _) = parse("for (await in x);", module);
        assert!(!errors.is_empty());
        assert_eq!(source_type, SourceType::Module);

        let (errors, source_type, _) = parse("for (await in x); export {};", unambiguous);
        assert!(!errors.is_empty());
        assert_eq!(source_type, SourceType::Module);

        let (errors, source_type, nodes) = parse("await enum;", unambiguous);
        assert!(!errors.is_empty());
        assert_eq!(source_type, SourceType::Module);
        assert_eq!(nodes, [NodeKind::AwaitExpression]);

        for preserve_parens in [false, true] {
            let javascript = Options {
                preserve_parens,
                ..unambiguous
            };
            let typescript = Options {
                lang: Lang::Ts,
                ..javascript
            };
            for options in [javascript, typescript] {
                let (errors, source_type, _) = parse("await; ({a = 1});", options);
                assert_eq!(errors, [Span::new(9, 14)]);
                assert_eq!(source_type, SourceType::Script);

                let (errors, source_type, _) = parse("await; [{a = 1}];", options);
                assert_eq!(errors, [Span::new(9, 14)]);
                assert_eq!(source_type, SourceType::Script);

                let (errors, source_type, nodes) = parse("return; [{a = 1}];", options);
                assert_eq!(errors, [Span::new(10, 15)]);
                assert_eq!(source_type, SourceType::CommonJs);
                assert_eq!(
                    nodes,
                    [NodeKind::ReturnStatement, NodeKind::ArrayExpression]
                );

                let (errors, source_type, _) = parse("return; ({a = 1});", options);
                assert_eq!(errors, [Span::new(10, 15)]);
                assert_eq!(source_type, SourceType::CommonJs);

                let (errors, source_type, _) = parse("[{a = 1}];", options);
                assert_eq!(errors, [Span::new(2, 7)]);
                assert_eq!(source_type, SourceType::Script);

                let (errors, source_type, _) = parse("return; [{a = 1}]; export {};", options);
                assert_eq!(errors, [Span::new(0, 6), Span::new(10, 15)]);
                assert_eq!(source_type, SourceType::Module);

                let (errors, source_type, _) = parse("return; [{a = 1}]; x = 1 +;", options);
                assert_eq!(
                    errors,
                    [Span::new(0, 6), Span::new(10, 15), Span::new(26, 27)]
                );
                assert_eq!(source_type, SourceType::Module);
            }
        }
    }

    #[test]
    fn unambiguous_commonjs() {
        use crate::TokenParserConfig;

        fn parse<'a>(
            allocator: &'a Allocator,
            source: &'a str,
            lang: Lang,
            source_type: SourceType,
        ) -> ParseReturn<'a> {
            Parser::init(
                allocator,
                source,
                Options {
                    source_type,
                    lang,
                    ..Options::default()
                },
                NoTokenParserConfig,
            )
            .parse()
        }

        fn assert_resolves_as(source: &str, lang: Lang, expected: SourceType) {
            let unambiguous_allocator = Allocator::new();
            let unambiguous = parse(
                &unambiguous_allocator,
                source,
                lang,
                SourceType::Unambiguous,
            );
            assert!(
                unambiguous.diagnostics.is_empty(),
                "{:#?}",
                unambiguous.diagnostics
            );
            assert_eq!(unambiguous.ast.source_type(), expected, "{source}");

            let explicit_allocator = Allocator::new();
            let explicit = parse(&explicit_allocator, source, lang, expected);
            assert!(
                explicit.diagnostics.is_empty(),
                "{:#?}",
                explicit.diagnostics
            );

            let unambiguous_ast = serialize_ast(&unambiguous.ast);
            let explicit_ast = serialize_ast(&explicit.ast);
            assert_eq!(unambiguous_ast, explicit_ast, "{source}");
        }

        fn assert_rejects_as(source: &str, lang: Lang, expected: SourceType) {
            let allocator = Allocator::new();
            let result = parse(&allocator, source, lang, SourceType::Unambiguous);
            assert_eq!(result.ast.source_type(), expected, "{source}");
            assert!(!result.diagnostics.is_empty(), "{source}");
        }

        fn assert_tokens_and_comments_match(source: &str) {
            let unambiguous_allocator = Allocator::new();
            let unambiguous = Parser::init(
                &unambiguous_allocator,
                source,
                Options {
                    source_type: SourceType::Unambiguous,
                    ..Options::default()
                },
                TokenParserConfig,
            )
            .parse();
            assert!(
                unambiguous.diagnostics.is_empty(),
                "{:#?}",
                unambiguous.diagnostics
            );
            assert_eq!(unambiguous.ast.source_type(), SourceType::CommonJs);

            let explicit_allocator = Allocator::new();
            let explicit = Parser::init(
                &explicit_allocator,
                source,
                Options {
                    source_type: SourceType::CommonJs,
                    ..Options::default()
                },
                TokenParserConfig,
            )
            .parse();
            assert!(
                explicit.diagnostics.is_empty(),
                "{:#?}",
                explicit.diagnostics
            );
            assert!(unambiguous.tokens.iter().eq(explicit.tokens.iter()));
            assert_eq!(unambiguous.ast.comments(), explicit.ast.comments());
        }

        assert_resolves_as("return;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("{ return; }", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("if (condition) return;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("'use strict'; return;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("return;\nreturn;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("return; import('value');", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("with (object) return;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("return;\n<!-- comment", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("return; await + 0;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("return await + 0;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("new.target;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("{ new.target; }", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("() => new.target;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as("async () => new.target;", Lang::Js, SourceType::CommonJs);

        assert_resolves_as(
            "const f = (value = new.target) => value;",
            Lang::Js,
            SourceType::CommonJs,
        );

        assert_resolves_as("new.target; await + 0;", Lang::Js, SourceType::CommonJs);

        assert_tokens_and_comments_match("/* leading */ return; await + 0; // trailing");

        assert_tokens_and_comments_match("/* leading */ new.target; await + 0; // trailing");

        assert_resolves_as("function f() { return; }", Lang::Js, SourceType::Script);

        assert_resolves_as("() => { return; }", Lang::Js, SourceType::Script);

        assert_resolves_as("function f() { new.target; }", Lang::Js, SourceType::Script);

        assert_resolves_as(
            "function f() { return () => new.target; }",
            Lang::Js,
            SourceType::Script,
        );

        assert_resolves_as(
            "class C { field = new.target; }",
            Lang::Js,
            SourceType::Script,
        );

        assert_resolves_as(
            "class C { static { new.target; } }",
            Lang::Js,
            SourceType::Script,
        );

        assert_rejects_as("namespace N { new.target; }", Lang::Ts, SourceType::Script);

        assert_rejects_as("return;", Lang::Dts, SourceType::Module);

        assert_rejects_as("new.target;", Lang::Dts, SourceType::Script);

        assert_rejects_as("export {}; return;", Lang::Js, SourceType::Module);

        assert_rejects_as("return; export {};", Lang::Js, SourceType::Module);

        assert_rejects_as("return; import.meta;", Lang::Js, SourceType::Module);

        assert_rejects_as("return await value;", Lang::Js, SourceType::Module);

        let allocator = Allocator::new();
        let result = parse(
            &allocator,
            "return; export {}; return;",
            Lang::Js,
            SourceType::Unambiguous,
        );
        assert_eq!(result.ast.source_type(), SourceType::Module);
        assert_eq!(result.diagnostics.len(), 2);

        assert_rejects_as("export {}; new.target;", Lang::Js, SourceType::Module);

        assert_rejects_as("new.target; export {};", Lang::Js, SourceType::Module);

        assert_rejects_as("new.target; import.meta;", Lang::Js, SourceType::Module);

        assert_rejects_as("new.target; await value;", Lang::Js, SourceType::Module);

        assert_rejects_as(
            "class C { static { return; } }",
            Lang::Js,
            SourceType::Module,
        );

        assert_rejects_as("namespace N { return; }", Lang::Ts, SourceType::Module);
    }

    #[test]
    fn unambiguous_module_detection() {
        fn parse(source: &str, options: Options) -> (SourceType, Vec<Span>) {
            let allocator = Allocator::new();
            let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            (
                result.ast.source_type(),
                result.diagnostics.iter().map(|error| error.span).collect(),
            )
        }

        for preserve_parens in [false, true] {
            let javascript = Options {
                source_type: SourceType::Unambiguous,
                preserve_parens,
                ..Options::default()
            };
            let typescript = Options {
                lang: Lang::Ts,
                ..javascript
            };
            for options in [javascript, typescript] {
                let (kind, errors) = parse("return; new.target; export {}; new.target;", options);
                assert_eq!(kind, SourceType::Module);
                assert_eq!(errors, [Span::new(0, 6), Span::new(31, 41)]);

                let (kind, errors) = parse("new.target; return; import.meta; return;", options);
                assert_eq!(kind, SourceType::Module);
                assert_eq!(errors, [Span::new(0, 10), Span::new(33, 39)]);

                let (kind, errors) = parse("export {}; return; new.target; return;", options);
                assert_eq!(kind, SourceType::Module);
                assert_eq!(
                    errors,
                    [Span::new(11, 17), Span::new(19, 29), Span::new(31, 37)]
                );

                let (kind, errors) = parse("return; return; export {}; return;", options);
                assert_eq!(kind, SourceType::Module);
                assert_eq!(errors, [Span::new(0, 6), Span::new(27, 33)]);

                let (kind, errors) = parse("return; new.target; +;", options);
                assert_eq!(kind, SourceType::Module);
                assert_eq!(errors, [Span::new(0, 6), Span::new(21, 22)]);

                let (kind, errors) = parse("new.target; return; +;", options);
                assert_eq!(kind, SourceType::Module);
                assert_eq!(errors, [Span::new(0, 10), Span::new(21, 22)]);

                let (kind, errors) = parse("return; new.target; await + 0;", options);
                assert_eq!(kind, SourceType::CommonJs);
                assert!(errors.is_empty());

                let (kind, errors) = parse("return; new.target; await + 0; export {};", options);
                assert_eq!(kind, SourceType::Module);
                assert_eq!(errors, [Span::new(0, 6)]);
            }
        }
    }

    #[test]
    fn unambiguous_lexical_recovery() {
        fn parse<'a>(
            allocator: &'a Allocator,
            source: &'a str,
            options: Options,
        ) -> ParseReturn<'a> {
            Parser::init(allocator, source, options, TokenParserConfig).parse()
        }

        for preserve_parens in [false, true] {
            let flat_comments = Options {
                source_type: SourceType::Unambiguous,
                preserve_parens,
                comments: CommentMode::Flat,
                ..Options::default()
            };
            let no_comments = Options {
                comments: CommentMode::None,
                ..flat_comments
            };
            for options in [flat_comments, no_comments] {
                let allocator = Allocator::new();
                let source = "return;\n<!-- comment\nexport {};";
                let parsed = parse(&allocator, source, options);
                assert_eq!(parsed.ast.source_type(), SourceType::Module);
                assert_eq!(
                    parsed
                        .diagnostics
                        .iter()
                        .map(|error| error.span)
                        .collect::<Vec<_>>(),
                    [Span::new(0, 6), Span::new(8, 20)]
                );
                assert_eq!(parsed.ast.root_program().body(&parsed.ast).len(), 2);
                assert_eq!(
                    token_texts(source, &parsed.tokens),
                    ["return", ";", "export", "{", "}", ";"]
                );
                if options.comments == CommentMode::Flat {
                    let comments = parsed.ast.comments();
                    assert_eq!(comments.len(), 1);
                    assert_eq!(comments[0].span, Span::new(8, 20));
                    assert_eq!(comments[0].position, CommentPosition::Leading);
                    assert_eq!(comments[0].attached_to, 21);
                } else {
                    assert!(parsed.ast.comments().is_empty());
                }

                let source = "#!/bin/node\nreturn; export {};";
                let parsed = parse(&allocator, source, options);
                assert_eq!(parsed.ast.source_type(), SourceType::Module);
                assert_eq!(parsed.diagnostics.len(), 1);
                assert_eq!(parsed.diagnostics[0].span, Span::new(12, 18));
                assert!(parsed.ast.root_program().hashbang(&parsed.ast).is_some());
                assert_eq!(
                    token_texts(source, &parsed.tokens),
                    ["return", ";", "export", "{", "}", ";"]
                );
            }
        }
    }

    #[test]
    fn fragment_context() {
        fn parse_expr<'a>(
            allocator: &'a Allocator,
            source: &'a str,
            options: Options,
            context: FragmentContext,
        ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
            let parser = Parser::init(allocator, source, options, NoTokenParserConfig);
            parser.parse_expression_fragment(context)
        }

        use FragmentContext::{AsyncFunction, Generator, TopLevel};

        let allocator = Allocator::new();
        let script = Options {
            source_type: SourceType::Script,
            ..Options::default()
        };
        let auto = Options {
            source_type: SourceType::Unambiguous,
            ..Options::default()
        };

        let ast = parse_expr(&allocator, "await value", script, AsyncFunction).unwrap();
        let root_expression = ast.expr_data(ast.root_expression());
        assert!(matches!(root_expression, ExprData::AwaitExpression(_)));

        let ast = parse_expr(&allocator, "yield value", script, Generator).unwrap();
        let root_expression = ast.expr_data(ast.root_expression());
        assert!(matches!(root_expression, ExprData::YieldExpression(_)));

        let result = parse_expr(&allocator, "await\n? import.meta : x", auto, TopLevel);
        assert!(result.is_err_and(|diagnostics| !diagnostics.is_empty()));

        let ast = parse_expr(&allocator, "await\n{}", auto, TopLevel).unwrap();
        assert_eq!(ast.source_type(), SourceType::Module);
        assert!(matches!(
            ast.expr_data(ast.root_expression()),
            ExprData::AwaitExpression(_)
        ));
    }

    #[test]
    fn estimate_capacity() {
        let (nodes, extra) = estimate_ast_capacity(0);
        assert_eq!(nodes, 384);
        assert!(extra <= nodes);

        let (nodes, extra) = estimate_ast_capacity(8_000_000);
        assert_eq!(nodes, 1_500_000);
        assert_eq!(extra, 666_666);
    }

    #[test]
    fn comments() {
        macro_rules! check {
            ($comment:expr, $start:literal, $end:literal, $position:ident, $attached_to:literal) => {
                assert_eq!(
                    ($comment.span, $comment.position, $comment.attached_to),
                    (
                        Span::new($start, $end),
                        CommentPosition::$position,
                        $attached_to
                    )
                );
            };
        }

        fn parse_comments_with_source_type(
            source: &str,
            lang: Lang,
            source_type: SourceType,
            preserve_parens: bool,
        ) -> Vec<Comment> {
            let allocator = Allocator::new();
            let options = Options {
                lang,
                source_type,
                preserve_parens,
                ..Options::default()
            };
            let result = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();

            assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
            result.ast.comments().to_vec()
        }

        fn parse_comments(source: &str, lang: Lang) -> Vec<Comment> {
            parse_comments_with_source_type(source, lang, SourceType::Module, true)
        }

        fn parse_return_comments(source: &str, lang: Lang) -> Vec<Comment> {
            parse_comments_with_source_type(source, lang, SourceType::CommonJs, true)
        }

        fn parse_common_js_comments(source: &str, lang: Lang) -> Vec<Comment> {
            parse_comments_with_source_type(source, lang, SourceType::CommonJs, true)
        }

        // The expected values are generated by swc_core. Its spans and comment map keys are 1-based,
        // so those values are converted to 0-based offsets here.
        let comments = parse_comments(
            "/* Leading 1 */\n// Leading 2\n/* Leading 3 */ token /* Trailing 1 */ // Trailing 2\n// Leading of EOF token",
            Lang::Js,
        );
        assert_eq!(comments.len(), 6);
        check!(comments[0], 0, 15, Leading, 45);
        check!(comments[1], 16, 28, Leading, 45);
        check!(comments[2], 29, 44, Leading, 45);
        check!(comments[3], 51, 67, Trailing, 50);
        check!(comments[4], 68, 81, Trailing, 50);
        check!(comments[5], 82, 105, Trailing, 50);

        let comments = parse_comments(
            "#!/usr/bin/env node\n/* Leading 1 */\ntoken /* Trailing 1 */",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 20, 35, Leading, 36);
        check!(comments[1], 42, 58, Trailing, 41);

        let comments = parse_comments("/*\n * A\n **/\n/*\n * B\n **/\ntoken", Lang::Js);
        assert_eq!(comments.len(), 2);
        check!(comments[0], 0, 12, Leading, 26);
        check!(comments[1], 13, 25, Leading, 26);

        let comments = parse_comments(
            "foo();/**\n * @license MIT\n **/\nfunction bar() {}",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 6, 30, Trailing, 6);

        let comments = parse_comments("foo();//! @license MIT\nfunction bar() {}", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 6, 22, Trailing, 6);

        let comments = parse_comments(
            "function foo() {}/* #__NO_SIDE_EFFECTS__ */\nfunction bar() {}",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 17, 43, Trailing, 17);

        let comments = parse_comments("foo();// @__NO_SIDE_EFFECTS__\nfunction bar() {}", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 6, 29, Trailing, 6);

        let comments = parse_comments("foo();/* @__PURE__ */new Bar()", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 6, 21, Trailing, 6);

        let comments = parse_comments(
            "const v1 = // Leading comment 1\nfoo();\nfunction foo(param =// Leading comment 2\nnew Foo()\n) {}",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 11, 31, Trailing, 10);
        check!(comments[1], 59, 79, Trailing, 59);

        let comments = parse_comments(
            "call(// Leading comment 1\narguments)\n(// Leading comment 2\narguments)",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 5, 25, Trailing, 5);
        check!(comments[1], 38, 58, Trailing, 38);

        let comments = parse_comments("v = cond ? a : // Leading comment\nb;", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 15, 33, Trailing, 14);

        let comments = parse_comments("// test", Lang::Ts);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 7, Leading, 0);

        let comments = parse_comments("#!/foo/bar\n// test", Lang::Ts);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 11, 18, Trailing, 10);

        for lang in [Lang::Tsx, Lang::Jsx] {
            let comments = parse_comments("const t = <Switch>\n// 1\n/* 2 */\n</Switch>", lang);
            assert!(comments.is_empty());
        }

        let comments = parse_comments(
            "/** block comment */\nimport h from 'h';\n<div></div>",
            Lang::Tsx,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 20, Leading, 21);

        let comments = parse_comments("import // c\nx from 'x';", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 11, Trailing, 6);

        let comments = parse_comments("import /* c */ x from 'x';", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 14, Leading, 15);

        let comments = parse_comments("import // c\n('x');", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 11, Trailing, 6);

        let comments = parse_comments(
            "const foo = <h1>/* no */{/* 1 */ bar /* 2 */}/* no */</h1>;",
            Lang::Tsx,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 25, 32, Trailing, 25);
        check!(comments[1], 37, 44, Trailing, 36);

        let comments = parse_comments("const t = <T,>() => {\n// 1\n/* 2 */\ntest;\n};", Lang::Tsx);
        assert_eq!(comments.len(), 2);
        check!(comments[0], 22, 26, Leading, 35);
        check!(comments[1], 27, 34, Leading, 35);

        let comments = parse_comments("re = // ...\n/w+/", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 5, 11, Trailing, 4);

        let comments = parse_comments("re = /* ... *//w+/", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 5, 14, Leading, 14);

        let comments = parse_comments("re =\n/*\n ...\n */\n/w+/", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 5, 16, Leading, 17);

        // From js_parser_test.rs.

        let comments = parse_return_comments("return //\n x", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 9, Trailing, 6);

        let comments = parse_return_comments("return /**/\n x", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 11, Leading, 13);

        let comments = parse_common_js_comments("return <!--\n x", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 11, Trailing, 6);

        let comments = parse_common_js_comments("-->\nx", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 3, Leading, 4);

        let comments = parse_common_js_comments("x\n-->\ny", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 2, 5, Leading, 6);

        let comments = parse_common_js_comments("x\n/**/ /**/-->\ny", Lang::Js);
        assert_eq!(comments.len(), 3);
        check!(comments[0], 2, 6, Leading, 15);
        check!(comments[1], 7, 11, Leading, 15);
        check!(comments[2], 11, 14, Leading, 15);

        let comments = parse_comments(
            "function f() { //! @legal comment\n 'use strict' }",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 15, 33, Trailing, 14);

        let comments = parse_comments("foo() //! test", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 6, 14, Trailing, 5);

        let comments = parse_comments("//! test\nfoo()", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 8, Leading, 9);

        let comments = parse_comments("if (1) {//! test\nfoo()}", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 8, 16, Trailing, 8);

        let comments = parse_comments("if (1) {foo() //! test\n}", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 14, 22, Trailing, 13);

        let comments = parse_comments("    /*!\r\n     * Re-indent test\r\n     */", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 4, 39, Leading, 0);

        let comments = parse_comments(
            "x\u{2028}    /*!\u{2028}     * Re-indent test\u{2028}     */",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 8, 45, Trailing, 1);

        let comments = parse_comments("//�\n123", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 5, Leading, 6);

        let comments = parse_comments("/*�*/123", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 7, Leading, 7);

        let comments = parse_comments("< /**/ a/>", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 2, 6, Leading, 7);

        let comments = parse_comments("<a /**/ />", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 3, 7, Trailing, 2);

        let comments = parse_comments("<a/ /**/ >", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 4, 8, Leading, 9);

        let comments = parse_comments("<a>b</ /**/ a>", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 11, Leading, 12);

        let comments = parse_comments("<a>b</ //\n a>", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 7, 9, Trailing, 6);

        let comments = parse_comments("<a>b</a /**/ >", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 8, 12, Trailing, 7);

        let comments = parse_comments("// @jsx h\n<a/>", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 9, Leading, 10);

        let comments = parse_comments("<a/>\n// @jsx h", Lang::Jsx);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 5, 14, Trailing, 4);

        // From js_printer_test.rs.

        let comments = parse_comments(
            "new Worker(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 11, 27, Trailing, 11);
        check!(comments[1], 29, 45, Leading, 47);

        let comments = parse_comments(
            "new Worker(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 11, 30, Leading, 51);
        check!(comments[1], 31, 50, Leading, 51);

        let comments = parse_comments(
            "new Worker(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 16, 58, Leading, 59);

        let comments = parse_comments(
            "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 11, 30, Leading, 31);
        check!(comments[1], 38, 56, Trailing, 37);

        let comments = parse_comments(
            "new Worker(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 11, 30, Leading, 31);
        check!(comments[1], 39, 57, Leading, 58);

        let comments = parse_comments(
            "new Worker(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 27, 66, Leading, 67);

        let comments = parse_comments(
            "switch (x) { /* 1 */ case 1: /* 2 */ case 2: /* default */ default: break }",
            Lang::Js,
        );
        assert_eq!(comments.len(), 3);
        check!(comments[0], 13, 20, Trailing, 12);
        check!(comments[1], 29, 36, Leading, 37);
        check!(comments[2], 45, 58, Leading, 59);

        let comments = parse_comments("(/* foo */ { x() { foo() } }.x());", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 1, 10, Leading, 11);

        let comments = parse_comments("(/* @__PURE__ */ (() => foo())());", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 1, 16, Leading, 17);

        let comments = parse_comments("export default (/* foo */ function f() {});", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 16, 25, Leading, 26);

        let comments = parse_comments("x = () => (/* foo */ {});", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 11, 20, Leading, 21);

        let comments = parse_comments("function *x() { yield (/* foo */ y) }", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 23, 32, Leading, 33);

        let comments = parse_comments("/*@__PURE__*/(function() { foo() })()", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 13, Leading, 13);

        let comments = parse_comments(
            "export default /*@__PURE__*/(function() { foo() })()",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 15, 28, Leading, 28);

        let comments = parse_comments(
            "import(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 7, 23, Trailing, 7);
        check!(comments[1], 25, 41, Leading, 43);

        let comments = parse_comments(
            "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 7, 26, Leading, 47);
        check!(comments[1], 27, 46, Leading, 47);

        let comments = parse_comments(
            "import(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 12, 54, Leading, 55);

        let comments = parse_comments(
            "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 7, 26, Leading, 27);
        check!(comments[1], 34, 52, Trailing, 33);

        let comments = parse_comments(
            "import(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 7, 26, Leading, 27);
        check!(comments[1], 35, 53, Leading, 54);

        let comments = parse_comments(
            "import(/* webpackFoo: 1 */ 'path', { type: 'module' } /* webpackBar:2 */ );",
            Lang::Js,
        );
        assert_eq!(comments.len(), 2);
        check!(comments[0], 7, 26, Leading, 27);
        check!(comments[1], 54, 72, Trailing, 53);

        let comments = parse_comments(
            "import(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
            Lang::Js,
        );
        assert_eq!(comments.len(), 1);
        check!(comments[0], 23, 62, Leading, 63);

        let comments = parse_comments("//!single\nthrow 1 + 2", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 9, Leading, 10);

        let comments = parse_comments("/*!multi-\nline*/\nthrow 1 + 2", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 16, Leading, 17);

        let comments = parse_comments("//! </script\n//! >/script\n//! /script", Lang::Js);
        assert_eq!(comments.len(), 3);
        check!(comments[0], 0, 12, Leading, 0);
        check!(comments[1], 13, 25, Leading, 0);
        check!(comments[2], 26, 37, Leading, 0);

        let comments = parse_comments("/*! </script \n </script */", Lang::Js);
        assert_eq!(comments.len(), 1);
        check!(comments[0], 0, 26, Leading, 0);

        let comments = parse_comments_with_source_type(
            "/*a*/((//c\nx))//b",
            Lang::Js,
            SourceType::Module,
            false,
        );
        assert_eq!(comments.len(), 3);
        check!(comments[0], 0, 5, Leading, 11);
        check!(comments[1], 7, 10, Trailing, 7);
        check!(comments[2], 14, 17, Trailing, 12);
    }

    #[test]
    fn single_statement_function_expressions() {
        fn parse_js_diagnostics(source: &str) -> Vec<Diagnostic<'static>> {
            let allocator = Allocator::new();
            Parser::init(&allocator, source, Options::default(), NoTokenParserConfig)
                .parse()
                .diagnostics
        }

        let diagnostics = parse_js_diagnostics("if (a) c(function* () {});");
        assert!(diagnostics.is_empty());

        let diagnostics = parse_js_diagnostics("if (a) c(async function () {});");
        assert!(diagnostics.is_empty());

        let diagnostics = parse_js_diagnostics("if (a) function* f() {}");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn function_type_lookahead() {
        fn alias(source: &str, options: Options) -> (Vec<&str>, NodeKind) {
            let allocator = Allocator::new();
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            let ast = &parsed.ast;
            let alias = ast
                .root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_declaration(ast)
                .unwrap()
                .as_ts_type_alias_declaration(ast)
                .unwrap();
            let errors = parsed
                .diagnostics
                .iter()
                .map(|error| &source[error.span.start as usize..error.span.end as usize])
                .collect();
            (errors, ast.node_kind(alias.type_annotation(ast).node_id()))
        }

        fn return_type(source: &str) -> NodeKind {
            let allocator = Allocator::new();
            let options = Options {
                lang: Lang::Ts,
                preserve_parens: false,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            assert!(parsed.diagnostics.is_empty());
            let ast = &parsed.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let expression = statement
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast);
            let arrow = expression.as_arrow_function_expression(ast).unwrap();
            let ty = arrow.return_type(ast).unwrap().type_annotation(ast);
            ast.node_kind(ty.node_id())
        }

        let kind = return_type("(a): ([A.B]) => a;");
        assert_eq!(kind, NodeKind::TsTupleType);

        let kind = return_type("(a): ({ a: A.B }) => a;");
        assert_eq!(kind, NodeKind::TsTypeLiteral);
        let kind = return_type("(a): (void) => a;");
        assert_eq!(kind, NodeKind::TsVoidKeyword);

        let kind = return_type("(a): (null) => a;");
        assert_eq!(kind, NodeKind::TsNullKeyword);

        let kind = return_type("(a): (true) => a;");
        assert_eq!(kind, NodeKind::TsLiteralType);

        let allocator = Allocator::new();
        let (ast, declaration) =
            parse_typescript_declaration(&allocator, "type F = (this) => void;");
        let function = declaration
            .as_ts_type_alias_declaration(&ast)
            .unwrap()
            .type_annotation(&ast)
            .as_ts_function_type(&ast)
            .unwrap();
        assert!(function.this_param(&ast).is_some());
        assert!(function.params(&ast).items(&ast).is_empty());

        for preserve_parens in [false, true] {
            let typescript = Options {
                lang: Lang::Ts,
                preserve_parens,
                ..Options::default()
            };
            let tsx = Options {
                lang: Lang::Tsx,
                ..typescript
            };
            for options in [typescript, tsx] {
                let (errors, kind) = alias("type T = ({a = {b = 1}}) => void;", options);
                assert_eq!(errors, ["b = 1"]);
                assert_eq!(kind, NodeKind::TsFunctionType);

                let (errors, kind) = alias("type T = ([a = {b = 1}]) => void;", options);
                assert_eq!(errors, ["b = 1"]);
                assert_eq!(kind, NodeKind::TsFunctionType);
            }
        }
    }

    #[test]
    fn conditional_arrow_recovery() {
        fn parse(source: &str) -> (bool, Vec<NodeKind>) {
            let allocator = Allocator::new();
            let options = Options {
                lang: Lang::Ts,
                preserve_parens: false,
                ..Options::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            let ast = &parsed.ast;
            let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
            let mut expression = statement
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast);
            let mut consequents = Vec::new();
            loop {
                consequents.push(ast.node_kind(expression.node_id()));
                let Some(conditional) = expression.as_conditional_expression(ast) else {
                    break;
                };
                expression = conditional.consequent(ast);
            }
            (!parsed.diagnostics.is_empty(), consequents)
        }

        use NodeKind::{ArrowFunctionExpression, ConditionalExpression, ObjectExpression};

        let (errors, consequents) = parse("a ? b ? ({m(){}}) : c => d : e;");
        assert!(!errors);
        assert_eq!(
            consequents,
            [
                ConditionalExpression,
                ConditionalExpression,
                ObjectExpression
            ]
        );

        let (errors, consequents) = parse("a ? (x): T => ({m(){}} = x) : e;");
        assert!(errors);
        assert_eq!(
            consequents,
            [ConditionalExpression, ArrowFunctionExpression]
        );

        let (errors, consequents) = parse("({m(){}}) => 0;");
        assert!(errors);
        assert_eq!(consequents, [ArrowFunctionExpression]);
    }

    #[test]
    fn typed_arrow_context() {
        fn parse<'a>(allocator: &'a Allocator, source: &'a str, options: Options) -> Ast<'a> {
            let result = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
            assert!(result.diagnostics.is_empty());
            assert_eq!(result.ast.root_program().body(&result.ast).len(), 1);
            result.ast
        }

        fn conditional(ast: &Ast<'_>) -> ConditionalExpression {
            ast.root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast)
                .as_conditional_expression(ast)
                .unwrap()
        }

        fn assert_identifier(ast: &Ast<'_>, expression: Expr, expected: &str) {
            let identifier = expression.as_identifier_reference(ast).unwrap();
            assert_eq!(ast.get_utf8(identifier.name(ast)), expected);
        }

        fn assert_parenthesized_a(ast: &Ast<'_>, expression: Expr) {
            let parenthesized = expression.as_parenthesized_expression(ast).unwrap();
            assert_identifier(ast, parenthesized.expression(ast), "a");
        }

        fn assert_alternate(ast: &Ast<'_>, expression: ConditionalExpression) {
            let arrow = expression
                .alternate(ast)
                .as_arrow_function_expression(ast)
                .unwrap();
            let params = arrow.params(ast).items(ast);
            assert_eq!(params.len(), 1);
            let parameter = params
                .get_node(ast, 0)
                .unwrap()
                .as_formal_parameter(ast)
                .unwrap();
            let binding = parameter.pattern(ast);
            let identifier = binding.as_binding_identifier(ast).unwrap();
            assert_eq!(ast.get_utf8(identifier.name(ast)), "b");
            assert!(arrow.return_type(ast).is_none());
        }

        fn return_types(ast: &Ast<'_>, expression: Expr) -> Vec<String> {
            struct TypedArrows<'ast, 'a> {
                ast: &'ast Ast<'a>,
                return_types: Vec<String>,
            }

            impl<'a> Visit<'a> for TypedArrows<'_, 'a> {
                fn ast(&self) -> &Ast<'a> {
                    self.ast
                }

                fn visit_arrow_function_expression(&mut self, arrow: ArrowFunctionExpression) {
                    if let Some(annotation) = arrow.return_type(self.ast) {
                        let ty = annotation
                            .type_annotation(self.ast)
                            .as_ts_type_reference(self.ast)
                            .unwrap();
                        let name = ty
                            .type_name(self.ast)
                            .as_identifier_reference(self.ast)
                            .unwrap();
                        self.return_types
                            .push(self.ast.get_utf8(name.name(self.ast)).to_owned());
                    }
                    arrow.visit_children_with(self);
                }
            }

            let mut visitor = TypedArrows {
                ast,
                return_types: Vec::new(),
            };
            expression.visit_with(&mut visitor);
            visitor.return_types
        }

        fn parse_boundary(source: &str, options: Options) -> Vec<String> {
            let allocator = Allocator::new();
            let plain_source = format!("cond ? {source} : undefined;");
            let ast = parse(&allocator, &plain_source, options);
            let expression = conditional(&ast);
            assert_identifier(&ast, expression.alternate(&ast), "undefined");
            let plain_types = return_types(&ast, expression.consequent(&ast));

            let nested_source = format!("cond ? inner ? {source} : (a) : b => b;");
            let ast = parse(&allocator, &nested_source, options);
            let expression = conditional(&ast);
            assert_alternate(&ast, expression);
            let inner = expression
                .consequent(&ast)
                .as_conditional_expression(&ast)
                .unwrap();
            assert_parenthesized_a(&ast, inner.alternate(&ast));
            let nested_types = return_types(&ast, inner.consequent(&ast));
            assert_eq!(nested_types, plain_types);
            plain_types
        }

        let ts = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let tsx = Options {
            lang: Lang::Tsx,
            ..ts
        };

        let types = parse_boundary("items.map((item): Item => item)", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("items.map(([a, b]): Item => ({ a, b }))", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("items.map?.((item): Item => item)", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("new Factory((item): Item => item)", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("[(item): Item => item]", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("{ f: (item): Item => item }", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("items[(item): Item => item]", ts);
        assert_eq!(types, ["Item"]);

        let errors = has_errors("c ? f < { [(x): T => x]: 0 } > y : z;", ts);
        assert!(!errors);

        let errors = has_errors("c ? f<{ [(x): T => x]: 0 }>() : z;", ts);
        assert!(!errors);

        let types = parse_boundary("consume(...(item): Item => item)", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("function f(x = (item): Item => item) {}", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("function f() { return (item): Item => item; }", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("() => { return (item): Item => item; }", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("class { f = (item): Item => item; }", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("`${(item): Item => item}`", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("import((item): Item => item)", ts);
        assert_eq!(types, ["Item"]);

        let types = parse_boundary("<Component render={(item): Item => item} />", tsx);
        assert_eq!(types, ["Item"]);

        let allocator = Allocator::new();

        let ast = parse(&allocator, "cond ? f((x): T => x) + (a) : b => b;", ts);
        let expression = conditional(&ast);
        assert_alternate(&ast, expression);
        let binary = expression
            .consequent(&ast)
            .as_binary_expression(&ast)
            .unwrap();
        assert_parenthesized_a(&ast, binary.right(&ast));

        let ast = parse(&allocator, "cond ? x = (a) : b => b;", ts);
        let expression = conditional(&ast);
        assert_alternate(&ast, expression);
        let assignment = expression
            .consequent(&ast)
            .as_assignment_expression(&ast)
            .unwrap();
        assert_parenthesized_a(&ast, assignment.right(&ast));

        let ast = parse(&allocator, "cond ? inner ? yes : (a) : b => b;", ts);
        let expression = conditional(&ast);
        assert_alternate(&ast, expression);
        let inner = expression
            .consequent(&ast)
            .as_conditional_expression(&ast)
            .unwrap();
        assert_parenthesized_a(&ast, inner.alternate(&ast));

        let ast = parse(&allocator, "cond ? x => (a) : b => b;", ts);
        let expression = conditional(&ast);
        assert_alternate(&ast, expression);
        let arrow = expression
            .consequent(&ast)
            .as_arrow_function_expression(&ast)
            .unwrap();
        assert_parenthesized_a(&ast, arrow.body(&ast).as_expr(&ast).unwrap());

        let ast = parse(&allocator, "function* f(){return c ? yield(a) : b=>b}", ts);
        let declaration = ast
            .root_program()
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_declaration(&ast)
            .unwrap();
        let function = declaration.as_function(&ast).unwrap();
        let statement = function
            .body(&ast)
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_return_statement(&ast)
            .unwrap();
        let expression = statement
            .argument(&ast)
            .unwrap()
            .as_conditional_expression(&ast)
            .unwrap();
        assert_alternate(&ast, expression);
        let yielded = expression
            .consequent(&ast)
            .as_yield_expression(&ast)
            .unwrap();
        assert_parenthesized_a(&ast, yielded.argument(&ast).unwrap());
    }

    #[track_caller]
    fn has_errors(source: &str, options: Options) -> bool {
        let allocator = Allocator::new();
        let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        !parsed.diagnostics.is_empty()
    }

    #[test]
    fn async_primary() {
        fn constructor(ast: &Ast<'_>) -> Expr {
            ast.root_program()
                .body(ast)
                .get_node(ast, 0)
                .unwrap()
                .as_expression_statement(ast)
                .unwrap()
                .expression(ast)
                .as_new_expression(ast)
                .unwrap()
                .callee(ast)
        }

        let options = Options::default();
        let errors = |source| has_errors(source, options);

        let has_errors = errors("class C extends async x => x {}");
        assert!(has_errors);

        let has_errors = errors("class C extends async () => {} {}");
        assert!(has_errors);

        let has_errors = errors("class C extends async() {}");
        assert!(!has_errors);

        let has_errors = errors("class C extends as\\u0079nc function(){} {}");
        assert!(has_errors);

        let has_errors = errors("class C extends async function(){}.constructor {}");
        assert!(!has_errors);

        let has_errors = errors("class C extends async function(){} {}");
        assert!(!has_errors);

        let has_errors = errors("class C extends async function*(){}.constructor {}");
        assert!(!has_errors);

        let has_errors = errors("class C extends async function*(){} {}");
        assert!(!has_errors);

        let has_errors = errors("class C extends async\nfunction(){} {}");
        assert!(has_errors);

        let has_errors = errors("class C extends (async function(){}) {}");
        assert!(!has_errors);

        let has_errors = errors("new async x => x;");
        assert!(has_errors);

        let has_errors = errors("new async () => {};");
        assert!(has_errors);

        let has_errors = errors("new as\\u0079nc function(){};");
        assert!(has_errors);

        let has_errors = errors("new async function(){}.constructor(\"return 1\");");
        assert!(!has_errors);

        let has_errors = errors("new async function(){};");
        assert!(!has_errors);

        let has_errors = errors("new async function*(){}.constructor();");
        assert!(!has_errors);

        let has_errors = errors("new async function*(){};");
        assert!(!has_errors);

        let has_errors = errors("new async();");
        assert!(!has_errors);

        let has_errors = errors("new async\nfunction f(){}");
        assert!(!has_errors);

        let has_errors = errors("new function(){};");
        assert!(!has_errors);

        let has_errors = errors("new (async function(){})();");
        assert!(!has_errors);

        let allocator = Allocator::new();
        let script = Options {
            source_type: SourceType::Script,
            ..options
        };
        let parse = |source| {
            let result = Parser::init(&allocator, source, script, NoTokenParserConfig).parse();
            assert!(result.diagnostics.is_empty());
            result.ast
        };

        let ast = parse("new async function(){};");
        let function = constructor(&ast).as_function(&ast).unwrap();
        assert_eq!(
            function.function_type(&ast),
            FunctionType::FunctionExpression
        );
        assert!(function.r#async(&ast));
        assert!(!function.generator(&ast));

        let ast = parse("new async function*(){};");
        let function = constructor(&ast).as_function(&ast).unwrap();
        assert_eq!(
            function.function_type(&ast),
            FunctionType::FunctionExpression
        );
        assert!(function.r#async(&ast));
        assert!(function.generator(&ast));

        let ast = parse("new async\nfunction f(){}");
        let identifier = constructor(&ast).as_identifier_reference(&ast).unwrap();
        assert_eq!(ast.get_utf8(identifier.name(&ast)), "async");
        assert_eq!(ast.root_program().body(&ast).len(), 2);
        let declaration = ast.root_program().body(&ast).get_node(&ast, 1).unwrap();
        let function = declaration
            .as_declaration(&ast)
            .unwrap()
            .as_function(&ast)
            .unwrap();
        assert!(!function.r#async(&ast));
    }

    #[test]
    fn dynamic_import() {
        let options = Options::default();
        let errors = |source| has_errors(source, options);

        let has_errors = errors("import.defer('m', {with: {type: 'json'}},);");
        assert!(!has_errors);

        let has_errors = errors("import.defer('m',);");
        assert!(!has_errors);

        let has_errors = errors("import.source('m', {}, {});");
        assert!(has_errors);

        let has_errors = errors("import.source('m', {with: {type: 'json'}},);");
        assert!(!has_errors);

        let has_errors = errors("import.source('m',);");
        assert!(!has_errors);
    }

    #[test]
    fn html_comments() {
        let script = Options {
            source_type: SourceType::Script,
            ..Options::default()
        };

        let script_errors = has_errors("<!-- --> \"\nlet x = 1;", script);
        assert!(!script_errors);

        let module_errors = has_errors("<!-- --> \"\nlet x = 1;", Options::default());
        assert!(module_errors);
    }

    #[test]
    fn import_type_qualifiers() {
        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let errors = |source| has_errors(source, options);

        let has_errors = errors("let x: import(\"x\").");
        assert!(has_errors);

        let has_errors = errors("type A = import(\"x\").;");
        assert!(has_errors);

        let has_errors = errors("type A = import(\"x\").#a;");
        assert!(has_errors);

        let has_errors = errors("type A = import(\"x\").A.B;");
        assert!(!has_errors);

        let has_errors = errors("type A = import(\"x\").`a`;");
        assert!(has_errors);

        let has_errors = errors("type A = import(\"x\").A.;");
        assert!(has_errors);

        let has_errors = errors("type A = typeof import(\"x\").");
        assert!(has_errors);

        let has_errors = errors("type A = typeof import(\"x\").A;");
        assert!(!has_errors);
    }

    #[test]
    fn module_export_names() {
        let options = Options::default();
        let errors = |source| has_errors(source, options);

        let has_errors =
            errors("export { \"\\uD800\\\n\\u0041\" as \"\\uD800\\\n\\u0041\" } from \"mod\";");
        assert!(has_errors);

        let has_errors =
            errors("export { \"\\uD800\\\r\\uDC00\" as \"\\uD800\\\r\\uDC00\" } from \"mod\";");
        assert!(!has_errors);

        let has_errors =
            errors("export { \"\\uD800\\\r\n\\uDC00\" as \"\\uD800\\\r\n\\uDC00\" } from \"mod\";");
        assert!(!has_errors);

        let has_errors =
            errors("export { \"\\uD800\\\n\\uDC00\" as \"\\uD800\\\n\\uDC00\" } from \"mod\";");
        assert!(!has_errors);

        let has_errors = errors(
            "export { \"\\uD800\\\u{2028}\\uDC00\" as \"\\uD800\\\u{2028}\\uDC00\" } from \"mod\";",
        );
        assert!(!has_errors);

        let has_errors = errors(
            "export { \"\\uD800\\\u{2029}\\uDC00\" as \"\\uD800\\\u{2029}\\uDC00\" } from \"mod\";",
        );
        assert!(!has_errors);

        let has_errors = errors("export { \"\\uD800\" as \"\\uD800\" } from \"mod\";");
        assert!(has_errors);

        let has_errors = errors("export { \"\\uDC00\" as \"\\uDC00\" } from \"mod\";");
        assert!(has_errors);

        let has_errors = errors("export { x as \"\\uD800\\\n\\u0041\" };");
        assert!(has_errors);

        let has_errors = errors("export { x as \"\\uD800\\\r\\uDC00\" };");
        assert!(!has_errors);

        let has_errors = errors("export { x as \"\\uD800\\\r\n\\uDC00\" };");
        assert!(!has_errors);

        let has_errors = errors("export { x as \"\\uD800\\\n\\uDC00\" };");
        assert!(!has_errors);

        let has_errors = errors("export { x as \"\\uD800\\\u{2028}\\uDC00\" };");
        assert!(!has_errors);

        let has_errors = errors("export { x as \"\\uD800\\\u{2029}\\uDC00\" };");
        assert!(!has_errors);

        let has_errors = errors("export { x as \"\\uD800\" };");
        assert!(has_errors);

        let has_errors = errors("export { x as \"\\uDC00\" };");
        assert!(has_errors);

        let has_errors = errors("import { \"\\uD800\\\n\\u0041\" as x } from \"mod\";");
        assert!(has_errors);

        let has_errors = errors("import { \"\\uD800\\\r\\uDC00\" as x } from \"mod\";");
        assert!(!has_errors);

        let has_errors = errors("import { \"\\uD800\\\r\n\\uDC00\" as x } from \"mod\";");
        assert!(!has_errors);

        let has_errors = errors("import { \"\\uD800\\\n\\uDC00\" as x } from \"mod\";");
        assert!(!has_errors);

        let has_errors = errors("import { \"\\uD800\\\u{2028}\\uDC00\" as x } from \"mod\";");
        assert!(!has_errors);

        let has_errors = errors("import { \"\\uD800\\\u{2029}\\uDC00\" as x } from \"mod\";");
        assert!(!has_errors);

        let has_errors = errors("import { \"\\uD800\" as x } from \"mod\";");
        assert!(has_errors);

        let has_errors = errors("import { \"\\uDC00\" as x } from \"mod\";");
        assert!(has_errors);
    }

    #[test]
    fn parenthesized_optional_chain() {
        for preserve_parens in [false, true] {
            let options = Options {
                preserve_parens,
                ..Options::default()
            };
            let errors = |source| has_errors(source, options);

            let has_errors = errors("(a?.b)[k] = v;");
            assert!(!has_errors);

            let has_errors = errors("[(a?.b)[k]] = values;");
            assert!(!has_errors);
        }
    }

    #[test]
    fn template_types() {
        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let errors = |source| has_errors(source, options);

        let has_errors = errors("type T = `\\unicode${string}`;");
        assert!(has_errors);

        let has_errors = errors("type T = `\\n${string}\\8`;");
        assert!(has_errors);

        let has_errors = errors("namespace N { type T = `\\n${string | number}\\u0061`; }");
        assert!(!has_errors);

        let has_errors =
            errors("type T = `\r\n${`\r\n${string | number}\r\n` | \"fallback\"}\r\n`;");
        assert!(!has_errors);
    }

    #[test]
    fn type_queries() {
        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let errors = |source| has_errors(source, options);

        let has_errors = errors("type Query = typeof f<<T>(value: T) => T>;");
        assert!(has_errors);

        let has_errors = errors("one < two as unknown as typeof f << other;");
        assert!(has_errors);

        let has_errors = errors("one < two satisfies A | typeof f << other;");
        assert!(has_errors);

        let has_errors = errors("one < two as typeof f << other;");
        assert!(has_errors);
    }

    #[test]
    fn tsx_expressions() {
        let allocator = Allocator::new();
        let options = Options {
            lang: Lang::Tsx,
            preserve_parens: false,
            ..Options::default()
        };
        let parse = |source| {
            Parser::init(&allocator, source, options, NoTokenParserConfig)
                .parse_expression_fragment(FragmentContext::TopLevel)
                .unwrap()
        };

        let ast = parse("(f<T>) + value");
        let expression = ast.root_expression().as_binary_expression(&ast).unwrap();
        assert_eq!(expression.operator(&ast), BinaryOperator::Add);
        assert!(
            expression
                .left(&ast)
                .as_ts_instantiation_expression(&ast)
                .is_some()
        );

        let ast = parse("(f<T>) < value");
        let expression = ast.root_expression().as_binary_expression(&ast).unwrap();
        assert_eq!(expression.operator(&ast), BinaryOperator::LessThan);
        assert!(
            expression
                .left(&ast)
                .as_ts_instantiation_expression(&ast)
                .is_some()
        );

        let ast = parse("f<T>(<Component />)");
        let expression = ast.root_expression().as_call_expression(&ast).unwrap();
        assert_eq!(
            expression.type_arguments(&ast).unwrap().params(&ast).len(),
            1
        );
        assert!(
            expression
                .arguments(&ast)
                .get_node(&ast, 0)
                .unwrap()
                .as_expr(&ast)
                .unwrap()
                .as_jsx_element(&ast)
                .is_some()
        );

        let ast = parse("(value as T) ? <Yes /> : <No />");
        let expression = ast
            .root_expression()
            .as_conditional_expression(&ast)
            .unwrap();
        assert!(expression.test(&ast).as_ts_as_expression(&ast).is_some());
        assert!(expression.consequent(&ast).as_jsx_element(&ast).is_some());
        assert!(expression.alternate(&ast).as_jsx_element(&ast).is_some());

        let ast = parse("(value as number)++");
        let expression = ast.root_expression().as_update_expression(&ast).unwrap();
        assert_eq!(expression.operator(&ast), UpdateOperator::Increment);
        assert!(!expression.prefix(&ast));
        assert!(
            expression
                .argument(&ast)
                .as_ts_as_expression(&ast)
                .is_some()
        );

        let ast = parse("value as typeof f << other");
        let expression = ast.root_expression().as_binary_expression(&ast).unwrap();
        assert_eq!(expression.operator(&ast), BinaryOperator::LeftShift);
        let assertion = expression.left(&ast).as_ts_as_expression(&ast).unwrap();
        assert!(
            assertion
                .type_annotation(&ast)
                .as_ts_type_query(&ast)
                .is_some()
        );
    }

    #[test]
    fn namespace_await_using() {
        let options = Options {
            lang: Lang::Ts,
            ..Options::default()
        };
        let errors = |source| has_errors(source, options);

        let has_errors = errors("namespace N { await using x = null; }");
        assert!(has_errors);
    }
}
