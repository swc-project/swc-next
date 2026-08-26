//! ECMAScript and TypeScript parser implementation.
//!
//! This module is a Rust port of the parser design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

use std::{borrow::Cow, hint::cold_path, mem};

use bitflags::bitflags;
use rustc_hash::FxHashSet;
use swc_next_allocator::{Allocator, vec::Vec as ArenaVec, wtf8::Wtf8Buf};
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
    /// The active values of ECMAScript expression grammatical parameters.
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
        context
    }
}

bitflags! {
    /// Parser state inherited across syntactic boundaries.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    struct BoundaryContext: u8 {
        /// Whether parsing is inside a function.
        const IN_FUNCTION = 1 << 0;
        /// Whether module-only syntax at the current position can classify an
        /// unambiguous Program as a Module.
        const CAN_BE_MODULE = 1 << 1;
    }
}

impl BoundaryContext {
    fn in_function(self) -> bool {
        self.contains(Self::IN_FUNCTION)
    }

    fn can_be_module(self) -> bool {
        self.contains(Self::CAN_BE_MODULE)
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
        /// Whether a trailing `?` after a type is forbidden from starting a new
        /// conditional type.
        const DISALLOW_CONDITIONAL_TYPES = 1 << 1;
        /// Whether a speculatively parsed arrow with a return type can commit
        /// without an outer `:`.
        ///
        /// This is false only while reparsing the consequent of a conditional
        /// expression after a `(x): T => ...` arrow consumed the ternary `:`.
        const ALLOW_ARROW_RETURN_TYPE = 1 << 2;
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
}

struct ParsedBody {
    directives: TypedSubRange<Directive>,
    body: TypedSubRange<Stmt>,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    struct ParserState: u8 {
        /// Whether the current cover grammar parsed a
        /// `CoverInitializedName` (`{ a = 1 }`).
        const COVER_HAS_INIT_NAME = 1 << 0;
        /// Whether the source contains syntax which is only valid in a module.
        const HAS_MODULE_SYNTAX = 1 << 1;
        /// Whether a top-level `await` has a different valid Script
        /// interpretation.
        const AMBIGUOUS_SCRIPT_DIFFERENT_AST = 1 << 2;
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ScratchBuffer {
    items: Vec<NodeId>,
}

impl ScratchBuffer {
    fn begin(&self) -> usize {
        self.items.len()
    }

    fn append(&mut self, index: impl AstNode) {
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
    in_cover: bool,
    respect_in_parameter: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArrowBodyResult {
    body: ArrowFunctionBody,
    is_expression: bool,
}

#[derive(Clone, Copy, Debug)]
struct ArrayCover {
    elements: TypedSubRange<Option<Argument>>,
    start: u32,
    end: u32,
}

#[derive(Clone, Copy, Debug)]
struct ObjectCover {
    properties: TypedSubRange<ObjectPropertyKind>,
    start: u32,
    end: u32,
}

#[derive(Clone, Copy, Debug)]
struct ParenthesizedCover {
    elements: TypedSubRange<Argument>,
    start: u32,
    end: u32,
    has_trailing_comma: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParseVariableDeclarationOpts {
    await_using: bool,
    is_declare: bool,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticMemberKind {
    Method,
    Field,
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum BinaryTypeKind {
    Union,
    Intersection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AngleListKind {
    Arguments,
    Parameters,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AngleClose {
    Peel,
    Exact,
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
    Interface(TypedSubRange<TsInterfaceHeritage>),
    Class(TypedSubRange<TsClassImplements>),
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

struct ExportSpecifiersResult {
    specifiers: TypedSubRange<ExportSpecifier>,
    local_tags: Vec<TokenTag>,
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
    ExtendsClause,
    Decorator,
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
    pub(super) diagnostics: Vec<Diagnostic<'static>>,
    current_token: Token,
    scratch_statements: ScratchBuffer,
    scratch_cover: ScratchBuffer,
    scratch_decorators: ScratchBuffer,
    /// General-purpose scratch buffers used when two extra-data lists are
    /// assembled at the same time.
    scratch_a: ScratchBuffer,
    scratch_b: ScratchBuffer,
    context: Context,
    statement_context: StatementContext,
    ts_context: TsContext,
    state: ParserState,
    /// Start offset of an array or object cover grammar with a trailing comma.
    cover_has_trailing_comma: Option<u32>,
    /// Target node whose parentheses were stripped while converting an
    /// assignment target. The same target must be rejected if cover grammar
    /// later establishes that it is a binding pattern.
    stripped_paren: Option<NodeId>,
    /// Start offsets which failed speculative parenthesized-arrow parsing.
    ///
    /// This cache is intentionally not part of [`Checkpoint`]: a failed
    /// speculation remains impossible after rewinding and must not be retried
    /// by an enclosing speculative parse.
    not_parenthesized_arrow: FxHashSet<u32>,
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
    state: ParserState,
    cover_has_trailing_comma: Option<u32>,
    stripped_paren: Option<NodeId>,
    boundary_context: BoundaryContext,
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

impl<C: ParserConfig> Peek<'_, '_, C> {
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
            ast,
            source,
            source_type: options.source_type,
            lang: options.lang,
            preserve_parens: options.preserve_parens,
            comment_mode: options.comments,
            boundary_context: BoundaryContext::default(),
            lexer,
            diagnostics: Vec::new(),
            current_token: initial_token,
            scratch_statements: ScratchBuffer::default(),
            scratch_cover: ScratchBuffer::default(),
            scratch_decorators: ScratchBuffer::default(),
            scratch_a: ScratchBuffer::default(),
            scratch_b: ScratchBuffer::default(),
            context: Context::default(),
            statement_context: StatementContext::default(),
            ts_context: {
                let mut context = TsContext::default();
                context.set(TsContext::AMBIENT, options.lang == Lang::Dts);
                context
            },
            state: ParserState::default(),
            cover_has_trailing_comma: None,
            stripped_paren: None,
            not_parenthesized_arrow: FxHashSet::default(),
        }
    }

    pub fn parse(mut self) -> ParseReturn<'a> {
        let is_unambiguous = self.source_type == SourceType::Unambiguous;
        let initial_source_type = if is_unambiguous {
            SourceType::Module
        } else {
            self.source_type
        };
        self.enter_program_context(initial_source_type, is_unambiguous);
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        self.ensure_capacity();
        let parsed_body = self.parse_body(None, self.source_type.into());
        if is_unambiguous {
            match self.unambiguous_parse_action() {
                UnambiguousParseAction::KeepModule => {}
                UnambiguousParseAction::RelabelAsScript => {
                    self.set_source_type(SourceType::Script);
                }
                UnambiguousParseAction::RetryAsScript => {
                    cold_path();
                    let allocator = self.ast.allocator();
                    let source = self.source;
                    let options = self.script_options();
                    let (module_result, config) = self.finish_program_with_config(parsed_body);
                    return Parser::init(allocator, source, options, config)
                        .parse_program_as_script()
                        .unwrap_or(module_result);
                }
            }
        }
        self.finish_program(parsed_body)
    }

    fn finish_program(self, parsed_body: ParsedBody) -> ParseReturn<'a> {
        self.finish_program_with_config(parsed_body).0
    }

    fn finish_program_with_config(mut self, parsed_body: ParsedBody) -> (ParseReturn<'a>, C) {
        let ParsedBody { directives, body } = parsed_body;
        let end = self.current_token.span.end;
        let hashbang = self.lexer.hashbang.map(|hashbang| {
            let start = hashbang.start;
            let end = start + u32::from(hashbang.len);
            let value = self.ast.source_slice(start, end);
            self.ast.hashbang(Span::new(0, end), value)
        });
        self.report_module_mode_errors();
        let program = self
            .ast
            .program(Span::new(0, end), directives, body, hashbang);
        if !self.preserve_parens {
            strip_parenthesized_nodes(&mut self.ast, &mut self.lexer.comments);
        }

        if self.comment_mode == CommentMode::Flat {
            self.ast.set_attached_comments(self.lexer.comments);
        }
        self.ast.set_root_program(program);

        let diagnostics = self.diagnostics;
        let mut tokens = self.lexer.tokens;
        if self.lexer.config.tokens() && !diagnostics.iter().any(|d| d.severity == Severity::Error)
        {
            let last_token = tokens.pop();
            debug_assert!(last_token.is_some_and(|token| token.tag == TokenTag::Eof));
        } else {
            tokens.clear();
        }

        let config = self.lexer.config;
        (
            ParseReturn {
                ast: self.ast,
                tokens,
                diagnostics,
            },
            config,
        )
    }

    /// Parses the source as exactly one expression in `fragment_context`.
    pub fn parse_expression_fragment(
        mut self,
        fragment_context: FragmentContext,
    ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
        if !self.validate_fragment_root() {
            return Err(self.diagnostics);
        }
        let is_unambiguous = self.source_type == SourceType::Unambiguous;
        let initial_source_type = if is_unambiguous {
            SourceType::Module
        } else {
            self.source_type
        };
        self.enter_fragment_context(initial_source_type, fragment_context, is_unambiguous);
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        self.ensure_capacity();
        let expr = self.parse_expression_fragment_once();
        if is_unambiguous {
            match self.unambiguous_parse_action() {
                UnambiguousParseAction::KeepModule => {}
                UnambiguousParseAction::RelabelAsScript => {
                    self.set_source_type(SourceType::Script);
                }
                UnambiguousParseAction::RetryAsScript => {
                    cold_path();
                    let allocator = self.ast.allocator();
                    let source = self.source;
                    let options = self.script_options();
                    let (module_result, config) = self.finish_expression_fragment_with_config(expr);
                    return Parser::init(allocator, source, options, config)
                        .parse_expression_fragment_as_script(fragment_context)
                        .unwrap_or(module_result);
                }
            }
        }
        self.finish_expression_fragment(expr)
    }

    fn finish_expression_fragment(
        self,
        expr: Option<Expr>,
    ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
        self.finish_expression_fragment_with_config(expr).0
    }

    fn finish_expression_fragment_with_config(
        mut self,
        expr: Option<Expr>,
    ) -> (Result<Ast<'a>, Vec<Diagnostic<'a>>>, C) {
        self.report_module_mode_errors();
        let result = match expr {
            Some(expr) if self.diagnostics.is_empty() => {
                self.finish_fragment();
                self.ast.set_root_expression(expr);
                if self.comment_mode == CommentMode::Flat {
                    self.ast.set_attached_comments(self.lexer.comments);
                }
                Ok(self.ast)
            }
            _ => Err(self.diagnostics),
        };
        let config = self.lexer.config;
        (result, config)
    }

    /// Parses the source as exactly one statement in `fragment_context`.
    pub fn parse_statement_fragment(
        mut self,
        fragment_context: FragmentContext,
    ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
        if !self.validate_fragment_root() {
            return Err(self.diagnostics);
        }
        let is_unambiguous = self.source_type == SourceType::Unambiguous;
        let initial_source_type = if is_unambiguous {
            SourceType::Module
        } else {
            self.source_type
        };
        let statement_context =
            self.enter_fragment_context(initial_source_type, fragment_context, is_unambiguous);
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        self.ensure_capacity();
        let statement = self.parse_statement_fragment_once(statement_context, fragment_context);
        if is_unambiguous {
            match self.unambiguous_parse_action() {
                UnambiguousParseAction::KeepModule => {}
                UnambiguousParseAction::RelabelAsScript => {
                    self.set_source_type(SourceType::Script);
                }
                UnambiguousParseAction::RetryAsScript => {
                    cold_path();
                    let allocator = self.ast.allocator();
                    let source = self.source;
                    let options = self.script_options();
                    let (module_result, config) =
                        self.finish_statement_fragment_with_config(statement);
                    return Parser::init(allocator, source, options, config)
                        .parse_statement_fragment_as_script(fragment_context)
                        .unwrap_or(module_result);
                }
            }
        }
        self.finish_statement_fragment(statement)
    }

    fn finish_statement_fragment(
        self,
        statement: Option<Stmt>,
    ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
        self.finish_statement_fragment_with_config(statement).0
    }

    fn finish_statement_fragment_with_config(
        mut self,
        statement: Option<Stmt>,
    ) -> (Result<Ast<'a>, Vec<Diagnostic<'a>>>, C) {
        self.report_module_mode_errors();
        let result = match statement {
            Some(statement) if self.diagnostics.is_empty() => {
                self.finish_fragment();
                self.ast.set_root_statement(statement);
                if self.comment_mode == CommentMode::Flat {
                    self.ast.set_attached_comments(self.lexer.comments);
                }
                Ok(self.ast)
            }
            _ => Err(self.diagnostics),
        };
        let config = self.lexer.config;
        (result, config)
    }

    fn enter_program_context(&mut self, source_type: SourceType, can_be_module: bool) {
        self.set_source_type(source_type);
        self.context = Context::from(source_type);
        self.boundary_context.set(
            BoundaryContext::IN_FUNCTION,
            source_type == SourceType::CommonJs,
        );
        self.boundary_context
            .set(BoundaryContext::CAN_BE_MODULE, can_be_module);
        self.statement_context = source_type.into();
    }

    fn enter_fragment_context(
        &mut self,
        source_type: SourceType,
        fragment_context: FragmentContext,
        can_be_module: bool,
    ) -> StatementContext {
        let statement_context = fragment_context.statement_context(source_type);
        self.set_source_type(source_type);
        self.context = Context::for_fragment(source_type, fragment_context, statement_context);
        self.boundary_context.set(
            BoundaryContext::IN_FUNCTION,
            fragment_context.is_function() || source_type == SourceType::CommonJs,
        );
        self.boundary_context.set(
            BoundaryContext::CAN_BE_MODULE,
            can_be_module && statement_context == StatementContext::ModuleItem,
        );
        self.statement_context = statement_context;
        statement_context
    }

    fn validate_fragment_root(&mut self) -> bool {
        let Some(hashbang) = self.lexer.hashbang else {
            return true;
        };
        let end = hashbang.start + u32::from(hashbang.len);
        self.report(
            Span::new(0, end),
            "Hashbang comments are only valid at the start of a program",
            ReportOptions::default(),
        );
        false
    }

    fn script_options(&self) -> Options {
        Options {
            source_type: SourceType::Script,
            lang: self.lang,
            preserve_parens: self.preserve_parens,
            comments: self.comment_mode,
        }
    }

    fn parse_program_as_script(mut self) -> Option<ParseReturn<'a>> {
        self.enter_program_context(SourceType::Script, false);
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        self.ensure_capacity();
        let parsed_body = self.parse_body(None, self.source_type.into());
        if self.script_retry_failed() {
            None
        } else {
            Some(self.finish_program(parsed_body))
        }
    }

    fn parse_expression_fragment_as_script(
        mut self,
        fragment_context: FragmentContext,
    ) -> Option<Result<Ast<'a>, Vec<Diagnostic<'a>>>> {
        self.enter_fragment_context(SourceType::Script, fragment_context, false);
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        self.ensure_capacity();
        let expr = self.parse_expression_fragment_once();
        if self.script_retry_failed() {
            None
        } else {
            Some(self.finish_expression_fragment(expr))
        }
    }

    fn parse_statement_fragment_as_script(
        mut self,
        fragment_context: FragmentContext,
    ) -> Option<Result<Ast<'a>, Vec<Diagnostic<'a>>>> {
        let statement_context =
            self.enter_fragment_context(SourceType::Script, fragment_context, false);
        if self.advance().is_none() {
            self.current_token = Token::eof(0);
        }
        self.ensure_capacity();
        let statement = self.parse_statement_fragment_once(statement_context, fragment_context);
        if self.script_retry_failed() {
            None
        } else {
            Some(self.finish_statement_fragment(statement))
        }
    }

    fn parse_expression_fragment_once(&mut self) -> Option<Expr> {
        let expression = self.parse_expr(Precedence::Lowest, ParseExpressionOpts::default());
        self.report_fragment_trailing_tokens();
        expression
    }

    fn parse_statement_fragment_once(
        &mut self,
        statement_context: StatementContext,
        fragment_context: FragmentContext,
    ) -> Option<Stmt> {
        let statement = self.parse_statement(statement_context);
        if let Some(statement) = statement {
            if statement_context == StatementContext::ModuleItem {
                self.record_statement_module_syntax(statement);
            } else if self.statement_requires_module_item_context(statement, fragment_context) {
                self.mark_module_syntax();
                self.report(
                    statement.span(&self.ast),
                    "Import and export declarations may only appear at the top level of a module",
                    ReportOptions::default(),
                );
            }
        }
        self.report_fragment_trailing_tokens();
        statement
    }

    fn report_fragment_trailing_tokens(&mut self) {
        if self.current_token.tag != TokenTag::Eof {
            self.report_expected(
                self.current_token.span,
                "Expected end of fragment",
                ReportOptions::default(),
            );
        }
    }

    fn finish_fragment(&mut self) {
        if !self.preserve_parens {
            strip_parenthesized_nodes(&mut self.ast, &mut self.lexer.comments);
        }
    }

    fn report_module_mode_errors(&mut self) {
        if self.source_type != SourceType::Module {
            return;
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

    fn unambiguous_parse_action(&self) -> UnambiguousParseAction {
        if self.state.contains(ParserState::HAS_MODULE_SYNTAX) {
            UnambiguousParseAction::KeepModule
        } else if self.has_errors()
            || self
                .state
                .contains(ParserState::AMBIGUOUS_SCRIPT_DIFFERENT_AST)
        {
            UnambiguousParseAction::RetryAsScript
        } else {
            UnambiguousParseAction::RelabelAsScript
        }
    }

    fn script_retry_failed(&self) -> bool {
        self.has_errors() || self.state.contains(ParserState::HAS_MODULE_SYNTAX)
    }

    fn set_source_type(&mut self, source_type: SourceType) {
        self.source_type = source_type;
        self.ast.set_source_metadata(source_type, self.lang);
    }

    fn mark_module_syntax(&mut self) {
        self.state.insert(ParserState::HAS_MODULE_SYNTAX);
    }

    fn enter_module_context(&mut self) {
        if self.statement_context == StatementContext::ModuleItem
            && self.boundary_context.can_be_module()
        {
            self.mark_module_syntax();
        }
    }

    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    /// Returns the resolved name for an identifier-like token.
    ///
    /// This strips `#` from private identifiers and decodes Unicode escapes.
    #[inline]
    fn identifier_name(&mut self, token: Token) -> AstUtf8 {
        if token.tag != TokenTag::PrivateIdentifier && !token.is_escaped() {
            return self.ast.source_slice(token.span.start, token.span.end);
        }

        self.identifier_name_slow(token)
    }

    #[cold]
    #[inline(never)]
    fn identifier_name_slow(&mut self, token: Token) -> AstUtf8 {
        let start = token.span.start + u32::from(token.tag == TokenTag::PrivateIdentifier);
        if token.is_escaped() {
            return self.decode_escaped_identifier(start, token.span.end);
        }

        self.ast.source_slice(start, token.span.end)
    }

    /// Returns the decoded string value without the surrounding quotes.
    fn string_value(&mut self, token: Token) -> AstWtf8 {
        if token.is_escaped() {
            return self.decode_escaped_string(token.span.start + 1, token.span.end - 1);
        }
        self.ast
            .source_slice_wtf8(token.span.start + 1, token.span.end - 1)
    }

    /// Returns the decoded content of a template quasi span.
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
        let mut decoded = Wtf8Buf::with_capacity((end - start) as usize);
        decode_string_escapes(&self.source[start as usize..end as usize], &mut decoded);
        self.ast.add_wtf8(&decoded)
    }

    fn describe_token(&self, token: Token) -> Cow<'a, str> {
        if token.tag == TokenTag::Eof {
            return Cow::Borrowed("end of file");
        }
        if let Some(text) = token.tag.to_str() {
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
    fn add_extra_from_scratch_a<N>(&mut self, scratch_checkpoint: usize) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'a>,
    {
        debug_assert!(scratch_checkpoint <= self.scratch_a.items.len());
        let range = self.ast.add_typed_sub_range(
            self.scratch_a.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(range.len(), self.scratch_a.items.len() - scratch_checkpoint);
        range
    }

    fn add_extra_from_scratch_b<N>(&mut self, scratch_checkpoint: usize) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'a>,
    {
        debug_assert!(scratch_checkpoint <= self.scratch_b.items.len());
        let range = self.ast.add_typed_sub_range(
            self.scratch_b.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(range.len(), self.scratch_b.items.len() - scratch_checkpoint);
        range
    }

    fn add_extra_from_scratch_cover<N>(&mut self, scratch_checkpoint: usize) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'a>,
    {
        debug_assert!(scratch_checkpoint <= self.scratch_cover.items.len());
        let range = self.ast.add_typed_sub_range(
            self.scratch_cover.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        );
        debug_assert_eq!(
            range.len(),
            self.scratch_cover.items.len() - scratch_checkpoint
        );
        range
    }

    fn add_extra_from_scratch_decorators<N>(
        &mut self,
        scratch_checkpoint: usize,
    ) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'a>,
    {
        debug_assert!(scratch_checkpoint <= self.scratch_decorators.items.len());
        let range = self.ast.add_typed_sub_range(
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

    fn add_extra_from_scratch_statements<N>(
        &mut self,
        scratch_checkpoint: usize,
    ) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'a>,
    {
        let scratch_end = self.scratch_statements.items.len();
        self.add_extra_from_scratch_statements_range(scratch_checkpoint, scratch_end)
    }

    fn add_extra_from_scratch_statements_range<N>(
        &mut self,
        scratch_start: usize,
        scratch_end: usize,
    ) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'a>,
    {
        debug_assert!(scratch_start <= scratch_end);
        debug_assert!(scratch_end <= self.scratch_statements.items.len());
        let range = self.ast.add_typed_sub_range(
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
            state: self.state,
            cover_has_trailing_comma: self.cover_has_trailing_comma,
            stripped_paren: self.stripped_paren,
            boundary_context: self.boundary_context,
        }
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
        self.state = checkpoint.state;
        self.cover_has_trailing_comma = checkpoint.cover_has_trailing_comma;
        self.stripped_paren = checkpoint.stripped_paren;
        self.boundary_context = checkpoint.boundary_context;
    }

    fn begin_peek(&mut self) -> Peek<'_, 'a, C> {
        Peek {
            state: self.lexer.state,
            cursor: self.lexer.cursor,
            comments_len: self.lexer.comments.len(),
            module_mode_errors_len: self.lexer.module_mode_errors.len(),
            processed_comments: self.lexer.processed_comments,
            tokens_len: self.lexer.tokens.len(),
            active: true,
            lexer: &mut self.lexer,
        }
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

    fn ensure_capacity(&mut self) {
        let (estimated_nodes, estimated_extra) = estimate_ast_capacity(self.source.len());
        self.ast.reserve(estimated_nodes, estimated_extra);
        self.scratch_cover.items.reserve(256);
        self.scratch_statements.items.reserve(256);
        self.scratch_a.items.reserve(256);
        self.scratch_b.items.reserve(256);
        self.scratch_decorators.items.reserve(128);
    }

    fn parse_body(
        &mut self,
        terminator: Option<TokenTag>,
        statement_context: StatementContext,
    ) -> ParsedBody {
        debug_assert!(statement_context.permits_declaration());
        let statement_context = mem::replace(&mut self.statement_context, statement_context);
        // `parse_body` is shared by ScriptBody, ModuleBody, FunctionBody, and
        // TypeScript module blocks, all of which preserve their initial string
        // literal ExpressionStatements as directives in the AST.
        let mut in_directive_prologue = true;
        let statements_checkpoint = self.scratch_statements.begin();
        let mut directives_end = statements_checkpoint;
        while !self.is_at_body_end(terminator) {
            let is_directive = (|| {
                if in_directive_prologue && self.current_token.tag == TokenTag::StringLiteral {
                    let expression =
                        self.parse_expr(Precedence::Lowest, ParseExpressionOpts::default())?;
                    if let ExprData::StringLiteral(string_literal) = self.ast.expr_data(expression)
                    {
                        let directive =
                            self.parse_directive(string_literal, string_literal.span(&self.ast))?;
                        self.scratch_statements.append(directive);
                        return Some(true);
                    }
                    let start = expression.span(&self.ast).start;
                    let statement =
                        self.parse_expression_statement_with_expression(start, expression)?;
                    self.scratch_statements.append(statement);
                    return Some(false);
                }
                let statement = self.parse_statement(self.statement_context)?;
                if terminator.is_none() {
                    self.record_statement_module_syntax(statement);
                }
                self.scratch_statements.append(statement);
                Some(false)
            })();
            if let Some(is_directive) = is_directive {
                if is_directive {
                    directives_end = self.scratch_statements.items.len();
                }
                in_directive_prologue &= is_directive;
            } else {
                self.recover(terminator);
            }
        }
        let statements_end = self.scratch_statements.items.len();
        let directives =
            self.add_extra_from_scratch_statements_range(statements_checkpoint, directives_end);
        let body = self.add_extra_from_scratch_statements_range(directives_end, statements_end);
        self.scratch_statements.reset(statements_checkpoint);
        self.statement_context = statement_context;
        ParsedBody { directives, body }
    }

    fn statement_has_module_syntax(&self, statement: Stmt) -> bool {
        match self.ast.stmt_data(statement) {
            StmtData::ImportDeclaration(_) => true,
            StmtData::ExportNamedDeclaration(_) => true,
            StmtData::ExportDefaultDeclaration(_) => true,
            StmtData::ExportAllDeclaration(_) => true,
            StmtData::TsExportAssignment(_) => true,
            StmtData::Declaration(declaration) => {
                let DeclData::TsImportEqualsDeclaration(declaration) =
                    self.ast.decl_data(declaration)
                else {
                    return false;
                };
                declaration
                    .module_reference(&self.ast)
                    .is_ts_external_module_reference(&self.ast)
            }
            _ => false,
        }
    }

    fn record_statement_module_syntax(&mut self, statement: Stmt) {
        if self.statement_has_module_syntax(statement) {
            self.mark_module_syntax();
        }
    }

    fn statement_requires_module_item_context(
        &self,
        statement: Stmt,
        fragment_context: FragmentContext,
    ) -> bool {
        match self.ast.stmt_data(statement) {
            StmtData::ImportDeclaration(_)
            | StmtData::ExportNamedDeclaration(_)
            | StmtData::ExportDefaultDeclaration(_)
            | StmtData::ExportAllDeclaration(_)
            | StmtData::TsExportAssignment(_)
            | StmtData::TsNamespaceExportDeclaration(_) => true,
            StmtData::Declaration(declaration) => {
                let DeclData::TsImportEqualsDeclaration(declaration) =
                    self.ast.decl_data(declaration)
                else {
                    return false;
                };
                fragment_context.is_function()
                    || declaration
                        .module_reference(&self.ast)
                        .is_ts_external_module_reference(&self.ast)
            }
            _ => false,
        }
    }

    fn is_at_body_end(&self, terminator: Option<TokenTag>) -> bool {
        self.current_token.tag == TokenTag::Eof
            || terminator.is_some_and(|tag| self.current_token.tag == tag)
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
        match self.current_token.tag {
            TokenTag::Await => self.parse_await_using_or_expression(),
            TokenTag::Var => self
                .parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration))),
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
        debug_assert_eq!(self.current_token.tag, TokenTag::At);
        let (decorators_start, decorators) = self.parse_decorators()?;
        if self.current_token.tag != TokenTag::Export {
            return Some(Stmt::Declaration(Decl::Class(self.parse_class_decorated(
                ParseClassOpts::default(),
                Some(decorators_start),
                decorators,
            )?)));
        }
        self.parse_export_decorated(decorators)
    }

    fn parse_export_decorated(&mut self, decorators: TypedSubRange<Decorator>) -> Option<Stmt> {
        self.enter_module_context();
        let export_start = self.current_token.span.start;
        self.advance()?;
        let is_default = self.eat(TokenTag::Default)?;
        let class = self.parse_class_decorated(
            ParseClassOpts {
                is_default_export: is_default,
                ..ParseClassOpts::default()
            },
            None,
            decorators,
        )?;
        let end = class.span(&self.ast).end;
        if is_default {
            Some(self.ast.stmt_export_default_declaration(
                Span::new(export_start, end),
                ExportDefaultDeclarationKind::Class(class),
            ))
        } else {
            let specifiers = TypedSubRange::empty();
            let attributes = TypedSubRange::empty();
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Let);
        if !self.is_let_identifier()? {
            return self
                .parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        }
        self.parse_expression_statement()
    }

    fn parse_using_or_expression(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Using);
        if !self.is_using_identifier()? {
            return self
                .parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        }
        self.parse_expression_statement()
    }

    fn parse_await_using_or_expression(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Await);
        let await_span = self.current_token.span;
        let next = self.peek_ahead()?;
        if next.tag == TokenTag::Using && !next.has_line_terminator_before() {
            let start = await_span.start;
            self.advance()?;
            if !self.is_using_identifier()? {
                self.report_await_using_if_disallowed(await_span);
                return self
                    .parse_variable_declaration(
                        ParseVariableDeclarationOpts {
                            await_using: true,
                            ..ParseVariableDeclarationOpts::default()
                        },
                        Some(start),
                    )
                    .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
            }
            self.report_await_using_if_disallowed(await_span);
            return self.parse_await_expression_statement(start);
        }
        self.parse_expression_or_labeled_statement()
    }

    fn report_await_using_if_disallowed(&mut self, span: Span) {
        if self.boundary_context.can_be_module() {
            self.mark_module_syntax();
            return;
        }
        if self.context.contains(Context::AWAIT) {
            return;
        }
        self.report(
            span,
            "'await using' is only valid in async functions or modules",
            ReportOptions::default(),
        );
    }

    fn parse_await_expression_statement(&mut self, start: u32) -> Option<Stmt> {
        let expression = self.parse_await_expression(start)?;
        let expression = self.parse_expr_from_left(
            expression,
            Precedence::Comma,
            ParseExpressionOpts::default(),
        )?;
        self.parse_expression_statement_with_expression(start, expression)
    }

    fn parse_const_or_const_enum(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Const);
        if self.lang.is_typescript() {
            let next = self.peek_ahead()?;
            if is_const_enum_head(next) {
                return self.parse_ts_declaration();
            }
        }
        self.parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)
            .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)))
    }

    fn parse_async_function_or_expression(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Async);
        let next = self.peek_ahead()?;
        if next.tag == TokenTag::Function && !next.has_line_terminator_before() {
            let start = self.current_token.span.start;
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Import);
        let next = self.peek_ahead()?;
        if matches!(next.tag, TokenTag::LeftParen | TokenTag::Dot) {
            return self.parse_expression_statement();
        }
        self.parse_import_declaration()
    }

    fn is_let_identifier(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Let);
        let next = self.peek_ahead()?;

        // Lexical declarations cannot be single statements. In this context,
        // `let` therefore starts an ExpressionStatement unless the
        // ExpressionStatement lookahead restriction `let [` applies.
        //
        // This also lets ASI terminate `let` before a binding-looking token:
        // `if (false) let\nvalue = 1`.
        if !self.statement_context.permits_declaration() && next.tag != TokenTag::LeftBracket {
            return Some(true);
        }

        Some(!can_start_let_binding(next.tag))
    }

    fn is_using_identifier(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Using);
        let next = self.peek_ahead()?;
        if can_start_binding_identifier(next.tag) && !self.can_insert_implicit_semicolon(next) {
            return Some(false);
        }
        Some(true)
    }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        let start = self.current_token.span.start;
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start block statement",
            Some("Block statements must be enclosed in braces: { ... }"),
        ) {
            return None;
        }
        let checkpoint = self.scratch_statements.begin();
        let result = (|| {
            while !matches!(self.current_token.tag, TokenTag::RightBrace | TokenTag::Eof) {
                match self.parse_statement(StatementContext::StatementListItem) {
                    Some(statement) => self.scratch_statements.append(statement),
                    None => self.recover(Some(TokenTag::RightBrace)),
                }
            }
            let end = self.current_token.span.end;
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Semicolon);
        let span = self.current_token.span;
        self.advance()?;
        Some(self.ast.stmt_empty_statement(span))
    }

    fn parse_if_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::If);
        let start = self.current_token.span.start;
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'if'", None) {
            return None;
        }
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

    fn parse_while_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::While);
        let start = self.current_token.span.start;
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'while'", None) {
            return None;
        }
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

    fn parse_do_while_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Do);
        let start = self.current_token.span.start;
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

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Return);
        let start = self.current_token.span.start;
        self.advance()?;
        if !self.boundary_context.in_function() {
            self.report(
                Span::new(start, self.lexer.state.prev_token_end),
                "'return' statement is only valid inside a function",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Remove the 'return' statement or wrap the code in a function.",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        let argument = if self.current_token.has_line_terminator_before()
            || matches!(
                self.current_token.tag,
                TokenTag::Semicolon | TokenTag::RightBrace | TokenTag::Eof
            ) {
            None
        } else {
            Some(self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?)
        };
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_return_statement(Span::new(start, end), argument),
        )
    }

    fn parse_throw_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Throw);
        let start = self.current_token.span.start;
        self.advance()?;
        if self.current_token.has_line_terminator_before() {
            return {
                let span = self.current_token.span;
                self.report(
                    span,
                    "Illegal newline after throw",
                    ReportOptions::default(),
                );
                None
            };
        }
        let argument = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_throw_statement(Span::new(start, end), argument),
        )
    }

    fn parse_break_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Break);
        let start = self.current_token.span.start;
        self.advance()?;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag.is_identifier_like()
        {
            None
        } else {
            Some(self.parse_label_identifier()?)
        };
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_break_statement(Span::new(start, end), label))
    }

    fn parse_continue_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Continue);
        let start = self.current_token.span.start;
        self.advance()?;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag.is_identifier_like()
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Debugger);
        let start = self.current_token.span.start;
        self.advance()?;
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_debugger_statement(Span::new(start, end)))
    }

    fn parse_with_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::With);
        let start = self.current_token.span.start;
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'with'", None) {
            return None;
        }
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

    fn parse_expression_or_labeled_statement(&mut self) -> Option<Stmt> {
        let expression = self.parse_expr(Precedence::Lowest, ParseExpressionOpts::default())?;
        if self.ast.node_kind(expression.node_id()) == NodeKind::IdentifierReference
            && self.current_token.tag == TokenTag::Colon
        {
            return self.parse_labeled_statement(expression);
        }
        self.parse_expression_statement_with_expression(
            expression.span(&self.ast).start,
            expression,
        )
    }

    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        let start = self.current_token.span.start;
        let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        self.parse_expression_statement_with_expression(start, expression)
    }

    fn parse_expression_statement_with_expression(
        &mut self,
        start: u32,
        expression: Expr,
    ) -> Option<Stmt> {
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(
            self.ast
                .stmt_expression_statement(Span::new(start, end), expression),
        )
    }

    fn parse_labeled_statement(&mut self, identifier: Expr) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Colon);
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

    fn parse_variable_declaration(
        &mut self,
        opts: ParseVariableDeclarationOpts,
        start_from_param: Option<u32>,
    ) -> Option<VariableDeclaration> {
        debug_assert!(
            start_from_param.is_some()
                || matches!(
                    self.current_token.tag,
                    TokenTag::Var | TokenTag::Let | TokenTag::Const | TokenTag::Using
                )
        );
        let declaration_start = start_from_param.unwrap_or(self.current_token.span.start);
        let kind = self.parse_variable_kind(opts.await_using)?;
        let context = if opts.is_declare || self.ts_context.ambient() {
            DeclaratorCtx::Declare
        } else {
            DeclaratorCtx::Normal
        };
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            let first = self.parse_variable_declarator(kind, context)?;
            let mut declaration_end = first.span(&self.ast).end;
            self.scratch_a.append(first);
            while self.eat(TokenTag::Comma)? {
                let declarator = self.parse_variable_declarator(kind, context)?;
                declaration_end = declarator.span(&self.ast).end;
                self.scratch_a.append(declarator);
            }
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
            let declarators = self.add_extra_from_scratch_a(checkpoint);
            Some(self.ast.variable_declaration(
                Span::new(declaration_start, end),
                kind,
                declarators,
                opts.is_declare,
            ))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_variable_kind(&mut self, await_using: bool) -> Option<VariableKind> {
        let tag = self.current_token.tag;
        self.advance()?;
        match tag {
            TokenTag::Var => Some(VariableKind::Var),
            TokenTag::Let => Some(VariableKind::Let),
            TokenTag::Const => Some(VariableKind::Const),
            TokenTag::Using => {
                if await_using {
                    Some(VariableKind::AwaitUsing)
                } else {
                    Some(VariableKind::Using)
                }
            }
            _ => None,
        }
    }

    fn parse_variable_declarator(
        &mut self,
        kind: VariableKind,
        context: DeclaratorCtx,
    ) -> Option<VariableDeclarator> {
        let start = self.current_token.span.start;
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
        if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
            let annotation = self.parse_type_annotation()?;
            end = annotation.span(&self.ast).end;
            self.apply_type_annotation_to_pattern(id, annotation);
        }
        let init = if self.eat(TokenTag::Assign)? {
            let expression = self.parse_expr(
                Precedence::Assignment,
                ParseExpressionOpts {
                    respect_in_parameter: true,
                    ..ParseExpressionOpts::default()
                },
            )?;
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
        Some(
            self.ast
                .variable_declarator(Span::new(start, end), id, init, definite),
        )
    }

    fn parse_binding_pattern(&mut self) -> Option<BindingPattern> {
        match self.current_token.tag {
            tag if tag.is_identifier_like() => Some(BindingPattern::BindingIdentifier(
                self.parse_binding_identifier()?,
            )),
            TokenTag::LeftBracket => self.parse_array_pattern(),
            TokenTag::LeftBrace => self.parse_object_pattern(),
            _ => {
                let found = self.describe_token(self.current_token);
                self.report(
                    self.current_token.span,
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

    fn parse_array_pattern(&mut self) -> Option<BindingPattern> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBracket);
        let cover = self.parse_array_cover()?;
        self.array_cover_to_pattern(cover, PatternContext::Binding)
    }

    fn parse_object_pattern(&mut self) -> Option<BindingPattern> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let cover = self.parse_object_cover()?;
        self.object_cover_to_pattern(cover, PatternContext::Binding)
    }

    fn parse_assignment_pattern(&mut self, left: BindingPattern) -> Option<BindingPattern> {
        if !self.eat(TokenTag::Assign)? {
            return Some(left);
        }
        let right = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        let span = Span::new(left.span(&self.ast).start, right.span(&self.ast).end);
        if self.ts_context.ambient() {
            self.report(
                span,
                "A parameter initializer is only allowed in a function or constructor implementation",
                ReportOptions::default(),
            );
        }
        let decorators = TypedSubRange::empty();
        Some(
            self.ast
                .binding_pattern_assignment_pattern(span, decorators, left, None, right, false),
        )
    }

    fn mark_pattern_optional(&mut self, pattern: BindingPattern, end: u32) {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(node) => node.set_optional(&mut self.ast, true),
            BindingPatternData::ArrayPattern(node) => node.set_optional(&mut self.ast, true),
            BindingPatternData::ObjectPattern(node) => node.set_optional(&mut self.ast, true),
            _ => unreachable!("only binding patterns can be marked optional"),
        }
        let span = pattern.span(&self.ast);
        pattern.set_span(&mut self.ast, Span::new(span.start, end));
    }

    fn apply_type_annotation_to_pattern(
        &mut self,
        pattern: BindingPattern,
        annotation: TsTypeAnnotation,
    ) {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(node) => {
                node.set_type_annotation(&mut self.ast, Some(annotation));
            }
            BindingPatternData::ArrayPattern(node) => {
                node.set_type_annotation(&mut self.ast, Some(annotation));
            }
            BindingPatternData::ObjectPattern(node) => {
                node.set_type_annotation(&mut self.ast, Some(annotation));
            }
            _ => unreachable!("only binding patterns can have type annotations"),
        }
        let span = pattern.span(&self.ast);
        let end = annotation.span(&self.ast).end;
        pattern.set_span(&mut self.ast, Span::new(span.start, end));
    }

    fn parse_function(
        &mut self,
        opts: ParseFunctionOpts,
        start_from_param: Option<u32>,
    ) -> Option<ParsedFunction> {
        debug_assert!(start_from_param.is_some() || self.current_token.tag == TokenTag::Function);
        let start = start_from_param.unwrap_or(self.current_token.span.start);
        if !self.expect(TokenTag::Function, "Expected 'function' keyword", None) {
            return None;
        }
        let generator = self.eat(TokenTag::Star)?;
        let is_function_expression = opts.is_expression && !opts.is_default_export;
        let context = if is_function_expression {
            enter_function_grammar_context(&mut self.context, generator, opts.is_async)
        } else {
            enter_grammar_context![&mut self.context, ?Yield, ?Await]
        };
        let id = if self.current_token.tag.is_identifier_like() {
            self.parse_binding_identifier().map(Some)
        } else {
            if !opts.is_expression && !opts.is_default_export {
                self.report(
                    self.current_token.span,
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
        self.context = context;
        let id = id?;
        let context = enter_function_grammar_context(&mut self.context, generator, opts.is_async);
        let boundary_context =
            mem::replace(&mut self.boundary_context, BoundaryContext::IN_FUNCTION);
        let tail = (|| {
            let type_parameters =
                if self.lang.is_typescript() && self.current_token.tag == TokenTag::LessThan {
                    Some(self.parse_type_parameters()?)
                } else {
                    None
                };
            let params = self.parse_formal_parameters(
                if generator || opts.is_async {
                    FormalParameterKind::UniqueFormalParameters
                } else {
                    FormalParameterKind::FormalParameters
                },
                false,
            )?;
            let return_type =
                if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_return_type_annotation()?)
                } else {
                    None
                };
            let params_end = params.span(&self.ast).end;
            let return_type_end =
                return_type.map_or(params_end, |return_type| return_type.span(&self.ast).end);
            let explicit_ambient_declaration = opts.is_declare && !is_function_expression;
            if explicit_ambient_declaration && self.current_token.tag == TokenTag::LeftBrace {
                self.report(
                    self.current_token.span,
                    "An implementation cannot be declared in ambient contexts",
                    ReportOptions::default(),
                );
                return None;
            }
            let has_body = !explicit_ambient_declaration
                && (!self.lang.is_typescript()
                    || is_function_expression
                    || self.current_token.tag == TokenTag::LeftBrace);
            let body = if has_body {
                Some(self.parse_function_body()?)
            } else {
                None
            };
            let end = if let Some(body) = body {
                body.span(&self.ast).end
            } else {
                self.eat_semicolon(return_type_end)?
            };
            Some((params, body, type_parameters, return_type, params_end, end))
        })();
        self.context = context;
        self.boundary_context = boundary_context;
        let (params, body, type_parameters, return_type, params_end, end) = tail?;
        if !self.statement_context.permits_declaration() {
            if opts.is_async {
                self.report(
                    self.current_token.span,
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
        let decorators = TypedSubRange::empty();
        self.parse_class_decorated(opts, start_from_param, decorators)
    }

    fn parse_class_decorated(
        &mut self,
        opts: ParseClassOpts,
        start_from_param: Option<u32>,
        decorators: TypedSubRange<Decorator>,
    ) -> Option<Class> {
        let start = start_from_param.unwrap_or(self.current_token.span.start);
        let mut is_abstract = opts.is_abstract;
        if self.current_token.tag == TokenTag::Abstract
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
                Span::new(start, self.current_token.span.end),
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
            return {
                let span = self.current_token.span;
                self.report(
                    span,
                    "Class declaration requires a name",
                    ReportOptions::default(),
                );
                None
            };
        } else {
            None
        };
        let type_parameters = if self.lang.is_typescript() && is_angle_open(self.current_token.tag)
        {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        let (super_class, super_type_arguments) = if self.eat(TokenTag::Extends)? {
            let mut super_class =
                self.parse_left_hand_side_expression(LhsContext::ExtendsClause)?;
            let type_arguments = if self.lang.is_typescript() {
                if let ExprData::TsInstantiationExpression(instantiation) =
                    self.ast.expr_data(super_class)
                {
                    super_class = instantiation.expression(&self.ast);
                    Some(instantiation.type_arguments(&self.ast))
                } else if is_angle_open(self.current_token.tag) {
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
        let implements = if self.lang.is_typescript() {
            self.parse_implements_clause()?
        } else {
            TypedSubRange::empty()
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
        let tag = self.current_token.tag;
        if !tag.is_identifier_like() || tag == TokenTag::Extends {
            return false;
        }
        if self.lang.is_typescript() && tag == TokenTag::Implements {
            let Some(next) = self.peek_ahead() else {
                return true;
            };
            return !next.tag.is_identifier_like();
        }
        true
    }

    fn parse_implements_clause(&mut self) -> Option<TypedSubRange<TsClassImplements>> {
        match self.parse_heritage_clause(TokenTag::Implements, HeritageKind::Class)? {
            HeritageClause::Class(implements) => Some(implements),
            HeritageClause::Interface(_) => unreachable!(),
        }
    }

    fn parse_heritage_expression(&mut self) -> Option<Expr> {
        let mut expression = self.parse_identifier()?;
        while self.current_token.tag == TokenTag::Dot {
            expression = self.parse_static_member_expression(expression, false)?;
        }
        Some(expression)
    }

    fn parse_class_body(&mut self) -> Option<ClassBody> {
        let start = self.current_token.span.start;
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start class body",
            Some("Class body must be enclosed in braces: class Name { ... }"),
        ) {
            return None;
        }
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while !matches!(self.current_token.tag, TokenTag::RightBrace | TokenTag::Eof) {
                if self.eat(TokenTag::Semicolon)? {
                    continue;
                }
                let element = self.parse_class_element()?;
                self.scratch_a.append(element);
            }
            let end = self.current_token.span.end;
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
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_class_element(&mut self) -> Option<ClassElement> {
        let element_start = self.current_token.span.start;
        let decorators = if self.current_token.tag == TokenTag::At {
            self.parse_decorators()?.1
        } else {
            TypedSubRange::empty()
        };

        if let Some(block) = self.try_static_block(decorators) {
            return Some(block);
        }

        let mut modifiers = ClassElementModifiers::default();
        let mut key = None;
        loop {
            match self.consume_modifier(&mut modifiers)? {
                ClassModifierStep::Consumed => {}
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
            && self.current_token.tag == TokenTag::LeftBracket
            && self.is_index_signature_start()?
        {
            return self.parse_index_signature_element(element_start, decorators, modifiers);
        }

        if key.is_none() {
            modifiers.computed = self.current_token.tag == TokenTag::LeftBracket;
            key = Some(self.parse_class_element_key()?);
        }
        let key = key.unwrap();

        let mut optional = false;
        let mut definite = false;
        if self.lang.is_typescript() {
            match self.current_token.tag {
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
        let computed = modifiers.computed;
        self.detect_constructor_kind(key, &mut modifiers, computed);

        let is_method_start = self.current_token.tag == TokenTag::LeftParen
            || (self.lang.is_typescript() && self.current_token.tag == TokenTag::LessThan);
        if is_method_start {
            if modifiers.is_accessor {
                return {
                    let span = self.current_token.span;
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

        if modifiers.is_async || modifiers.is_generator {
            self.report_expected(
                self.current_token.span,
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
        if modifiers.kind != MethodDefinitionKind::Method {
            self.report_expected(
                self.current_token.span,
                "Expected '(' for getter/setter definition",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Getters and setters require parentheses. Use 'get prop() {}' or 'set prop(value) {}' syntax.",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        self.parse_property_definition(
            element_start,
            decorators,
            key,
            modifiers,
            optional,
            definite,
        )
    }

    fn parse_index_signature_element(
        &mut self,
        start: u32,
        _decorators: TypedSubRange<Decorator>,
        modifiers: ClassElementModifiers,
    ) -> Option<ClassElement> {
        let element = self.parse_index_signature(start, modifiers.readonly, modifiers.is_static)?;
        let TsTypeElementData::TsIndexSignature(signature) = self.ast.ts_type_element_data(element)
        else {
            unreachable!("parse_index_signature always creates a TS index signature")
        };
        if self.current_token.tag == TokenTag::Semicolon {
            self.extend_span_to(signature, self.current_token.span.end);
            self.advance()?;
        }
        Some(ClassElement::TsIndexSignature(signature))
    }

    fn consume_modifier(
        &mut self,
        modifiers: &mut ClassElementModifiers,
    ) -> Option<ClassModifierStep> {
        let token = self.current_token;
        let is_typescript = self.lang.is_typescript();
        if !is_modifier(token.tag, is_typescript) {
            return Some(ClassModifierStep::None);
        }

        let Some(next) = self.peek_ahead() else {
            return Some(ClassModifierStep::None);
        };
        let accessor_before_star =
            next.tag == TokenTag::Star && matches!(token.tag, TokenTag::Get | TokenTag::Set);
        let consume_as_modifier = can_start_element_key(next.tag)
            && !accessor_before_star
            && !(requires_same_line(token.tag) && next.has_line_terminator_before())
            && !modifiers.conflicts(token.tag);

        if !consume_as_modifier {
            self.advance_without_escape_check()?;
            let name = self.identifier_name(token);
            return Some(ClassModifierStep::Key(
                self.ast.property_key_identifier_name(token.span, name),
            ));
        }

        self.report_if_escaped_keyword(token);
        self.advance_without_escape_check()?;
        modifiers.set(token.tag);
        Some(ClassModifierStep::Consumed)
    }

    fn try_static_block(&mut self, decorators: TypedSubRange<Decorator>) -> Option<ClassElement> {
        if self.current_token.tag != TokenTag::Static {
            return None;
        }
        let next = self.peek_ahead()?;
        if next.tag != TokenTag::LeftBrace {
            return None;
        }
        let static_token = self.current_token;
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
        self.report_if_escaped_keyword(static_token);
        self.advance_without_escape_check()?;
        self.parse_static_block(static_token.span.start)
    }

    fn parse_static_block(&mut self, start: u32) -> Option<ClassElement> {
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start static block",
            None,
        ) {
            return None;
        }
        let context = enter_grammar_context![&mut self.context, ~Yield, +Await];
        let boundary_context = mem::take(&mut self.boundary_context);
        let checkpoint = self.scratch_statements.begin();
        let body = {
            while !matches!(self.current_token.tag, TokenTag::RightBrace | TokenTag::Eof) {
                match self.parse_statement(StatementContext::StatementListItem) {
                    Some(statement) => self.scratch_statements.append(statement),
                    None => self.recover(Some(TokenTag::RightBrace)),
                }
            }
            self.add_extra_from_scratch_statements(checkpoint)
        };
        self.context = context;
        self.boundary_context = boundary_context;
        self.scratch_statements.reset(checkpoint);
        let end = self.current_token.span.end;
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
        if matches!(
            self.ast.property_key_data(key),
            PropertyKeyData::PrivateIdentifier(private)
                if self.ast.get_utf8(private.name(&self.ast)) == "constructor"
        ) {
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

    fn validate_static_prototype_or_constructor(
        &mut self,
        key: PropertyKey,
        kind: StaticMemberKind,
    ) {
        if !self.ts_context.ambient()
            && let Some(span) = self.prop_name_eq(key, "prototype")
        {
            self.report(
                span,
                "Classes may not have a static property named 'prototype'",
                ReportOptions::default(),
            );
        } else if kind == StaticMemberKind::Field
            && let Some(span) = self.prop_name_eq(key, "constructor")
        {
            self.report(
                span,
                "Classes may not have a static field named 'constructor'",
                ReportOptions::default(),
            );
        }
    }

    fn validate_field_constructor(&mut self, key: PropertyKey) {
        if let Some(span) = self.prop_name_eq(key, "constructor") {
            self.report(
                span,
                "Classes may not have a non-static field named 'constructor'",
                ReportOptions::default(),
            );
        }
    }

    fn parse_method_definition(
        &mut self,
        start: u32,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        modifiers: ClassElementModifiers,
        optional: bool,
    ) -> Option<ClassElement> {
        self.validate_method_modifiers(key, modifiers);
        if modifiers.is_static && !modifiers.computed {
            self.validate_static_prototype_or_constructor(key, StaticMemberKind::Method);
        }

        let function_start = self.current_token.span.start;
        let context = enter_function_grammar_context(
            &mut self.context,
            modifiers.is_generator,
            modifiers.is_async,
        );
        let boundary_context =
            mem::replace(&mut self.boundary_context, BoundaryContext::IN_FUNCTION);
        let result = (|| {
            let type_parameters =
                if self.lang.is_typescript() && is_angle_open(self.current_token.tag) {
                    Some(self.parse_type_parameters()?)
                } else {
                    None
                };
            let params = self.parse_formal_parameters(
                FormalParameterKind::UniqueFormalParameters,
                modifiers.kind == MethodDefinitionKind::Constructor,
            )?;
            let _ = self.check_accessor_arity(modifiers.kind, params);
            let return_type =
                if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_return_type_annotation()?)
                } else {
                    None
                };
            if self.current_token.tag == TokenTag::LeftBrace {
                let body = self.parse_function_body()?;
                let end = body.span(&self.ast).end;
                Some((params, type_parameters, return_type, Some(body), end))
            } else if self.lang.is_typescript() {
                let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
                Some((params, type_parameters, return_type, None, end))
            } else {
                self.report_expected(
                    self.current_token.span,
                    "Expected '{' to start method body",
                    ReportOptions::default(),
                );
                None
            }
        })();
        self.context = context;
        self.boundary_context = boundary_context;
        let (params, type_parameters, return_type, body, end) = result?;
        let function_span = Span::new(function_start, end);
        let method_span = Span::new(start, end);
        if let Some(body) = body {
            let function = self.ast.function(
                function_span,
                FunctionType::FunctionExpression,
                None,
                type_parameters,
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

    fn parse_property_definition(
        &mut self,
        start: u32,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        modifiers: ClassElementModifiers,
        optional: bool,
        definite: bool,
    ) -> Option<ClassElement> {
        if !modifiers.computed {
            if modifiers.is_static {
                self.validate_static_prototype_or_constructor(key, StaticMemberKind::Field);
            } else {
                self.validate_field_constructor(key);
            }
        }

        let type_annotation =
            if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };

        let value = if self.eat(TokenTag::Assign)? {
            let context = enter_grammar_context![&mut self.context, ~Yield, ~Await];
            let boundary_context = self.boundary_context;
            self.boundary_context.remove(BoundaryContext::CAN_BE_MODULE);
            let value = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default());
            self.context = context;
            self.boundary_context = boundary_context;
            Some(value?)
        } else {
            None
        };

        let mut end = self.lexer.state.prev_token_end;
        match self.current_token.tag {
            TokenTag::Semicolon => {
                end = self.current_token.span.end;
                self.advance()?;
            }
            TokenTag::RightBrace => {}
            _ => {
                if !self.can_insert_implicit_semicolon(self.current_token) {
                    self.report_expected(
                        self.current_token.span,
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

    fn parse_decorators(&mut self) -> Option<(u32, TypedSubRange<Decorator>)> {
        let start = self.current_token.span.start;
        let checkpoint = self.scratch_decorators.begin();
        let result = (|| {
            while self.current_token.tag == TokenTag::At {
                let decorator = self.parse_decorator()?;
                self.scratch_decorators.append(decorator);
            }
            Some((start, self.add_extra_from_scratch_decorators(checkpoint)))
        })();
        self.scratch_decorators.reset(checkpoint);
        result
    }

    fn parse_decorator(&mut self) -> Option<Decorator> {
        let start = self.current_token.span.start;
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
        let mut expression = match self.current_token.tag {
            TokenTag::LeftParen => self.parse_parenthesized_expression()?,
            TokenTag::New => self.parse_new_expression()?,
            TokenTag::Import => self.parse_import_expression()?,
            _ => self.parse_primary_expression(false)?,
        };

        loop {
            expression = match self.current_token.tag {
                TokenTag::Dot => self.parse_static_member_expression(expression, false)?,
                TokenTag::LeftBracket if context == LhsContext::ExtendsClause => {
                    self.parse_computed_member_expression(expression, false)?
                }
                TokenTag::LeftParen => self.parse_call_expression(expression, false, None)?,
                TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                    self.parse_tagged_template_expression(expression, None)?
                }
                TokenTag::OptionalChaining => self.parse_optional_chain(expression)?,
                TokenTag::LogicalNot
                    if self.lang.is_typescript()
                        && !self.current_token.has_line_terminator_before() =>
                {
                    self.parse_non_null_expression(expression)?
                }
                TokenTag::LessThan | TokenTag::LeftShift if self.lang.is_typescript() => {
                    let Some(next) =
                        self.parse_type_argumented_call_or_instantiation(expression)?
                    else {
                        break;
                    };
                    next
                }
                _ => break,
            };
        }

        Some(expression)
    }

    fn prop_name_eq(&self, key: PropertyKey, expected: &str) -> Option<Span> {
        let matches = match self.ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(identifier) => {
                self.ast.get_utf8(identifier.name(&self.ast)) == expected
            }
            PropertyKeyData::StringLiteral(string) => {
                self.ast.get_wtf8(string.value(&self.ast)) == expected
            }
            _ => false,
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
    ) -> bool {
        let kind = kind.into();
        let Some((expected, msg, help)) = kind.accessor_arity() else {
            return true;
        };
        let items = params.items(&self.ast);
        let has_this = items.get_node(&self.ast, 0).is_some_and(|item| {
            let FormalParameterItemData::FormalParameter(parameter) =
                self.ast.formal_parameter_item_data(item)
            else {
                return false;
            };
            matches!(
                self.ast
                    .formal_parameter_pattern_data(parameter.pattern(&self.ast)),
                FormalParameterPatternData::TsThisParameter(_)
            )
        });
        let count = items.len() - usize::from(has_this);
        let has_rest = params.rest(&self.ast).is_some();
        if count == expected && !has_rest {
            return true;
        }
        self.report(
            params.span(&self.ast),
            msg,
            ReportOptions {
                help: Some(Cow::Borrowed(help)),
                ..ReportOptions::default()
            },
        );
        false
    }

    fn parse_formal_parameters(
        &mut self,
        kind: FormalParameterKind,
        allow_parameter_properties: bool,
    ) -> Option<FormalParameters> {
        let start = self.current_token.span.start;
        if !self.expect(
            TokenTag::LeftParen,
            "Expected '(' to start parameter list",
            None,
        ) {
            return None;
        }
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            let mut rest = None;
            while self.current_token.tag != TokenTag::RightParen
                && self.current_token.tag != TokenTag::Eof
            {
                let parameter_start = self.current_token.span.start;
                let decorators =
                    if self.lang.is_typescript() && self.current_token.tag == TokenTag::At {
                        self.parse_decorators()?.1
                    } else {
                        TypedSubRange::empty()
                    };

                if self.current_token.tag == TokenTag::Spread {
                    rest = Some(self.parse_binding_rest_element(decorators)?);
                    if self.eat(TokenTag::Comma)?
                        && (!self.ts_context.ambient()
                            || self.current_token.tag != TokenTag::RightParen)
                    {
                        return {
                            let span = self.current_token.span;
                            self.report(
                                span,
                                "Rest parameter must be the last parameter",
                                ReportOptions::default(),
                            );
                            None
                        };
                    }
                    break;
                }

                let parameter = self.parse_formal_parameter(
                    allow_parameter_properties,
                    decorators,
                    parameter_start,
                )?;
                self.scratch_a.append(parameter);
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            let end = self.current_token.span.end;
            if !self.expect(
                TokenTag::RightParen,
                "Expected ')' to close parameter list",
                None,
            ) {
                return None;
            }
            let items = self.add_extra_from_scratch_a(checkpoint);
            Some(
                self.ast
                    .formal_parameters(Span::new(start, end), items, rest, kind),
            )
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_formal_parameter(
        &mut self,
        allow_parameter_properties: bool,
        decorators: TypedSubRange<Decorator>,
        start: u32,
    ) -> Option<FormalParameterItem> {
        if self.lang.is_typescript() && self.current_token.tag == TokenTag::This {
            let this_parameter = self.parse_this_parameter()?;
            let span = this_parameter.span(&self.ast);
            let parameter = self.ast.formal_parameter(
                span,
                FormalParameterPattern::TsThisParameter(this_parameter),
            );
            return Some(FormalParameterItem::FormalParameter(parameter));
        }

        let parameter_property = if self.lang.is_typescript() && allow_parameter_properties {
            self.parse_parameter_property_modifiers()?
        } else {
            ParameterPropertyModifiers::default()
        };
        let mut pattern = self.parse_binding_pattern()?;
        if self.lang.is_typescript() && self.eat(TokenTag::Question)? {
            self.mark_pattern_optional(pattern, self.lexer.state.prev_token_end);
        }
        if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
            let annotation = self.parse_type_annotation()?;
            self.apply_type_annotation_to_pattern(pattern, annotation);
        }
        pattern = self.parse_assignment_pattern(pattern)?;
        let span = pattern.span(&self.ast);
        if parameter_property.present {
            let parameter_property = self.ast.ts_parameter_property(
                Span::new(start, span.end),
                decorators,
                pattern,
                parameter_property.r#override,
                parameter_property.readonly,
                parameter_property.accessibility,
            );
            return Some(FormalParameterItem::TsParameterProperty(parameter_property));
        }

        self.apply_decorators_to_pattern(pattern, decorators);
        let parameter_pattern = FormalParameterPattern::BindingPattern(pattern);
        let parameter = self.ast.formal_parameter(span, parameter_pattern);
        Some(FormalParameterItem::FormalParameter(parameter))
    }

    fn parse_this_parameter(&mut self) -> Option<TsThisParameter> {
        debug_assert_eq!(self.current_token.tag, TokenTag::This);
        let start = self.current_token.span.start;
        self.advance()?;
        let type_annotation = if self.current_token.tag == TokenTag::Colon {
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

    fn parse_binding_rest_element(
        &mut self,
        decorators: TypedSubRange<Decorator>,
    ) -> Option<BindingRestElement> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Spread);
        let start = self.current_token.span.start;
        self.advance()?;
        let argument = self.parse_binding_pattern()?;
        let type_annotation =
            if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
        let end = type_annotation.map_or(argument.span(&self.ast).end, |annotation| {
            annotation.span(&self.ast).end
        });
        Some(self.ast.binding_rest_element(
            Span::new(start, end),
            decorators,
            argument,
            type_annotation,
            false,
        ))
    }

    fn parse_parameter_property_modifiers(&mut self) -> Option<ParameterPropertyModifiers> {
        let mut modifiers = ParameterPropertyModifiers::default();
        while self.is_parameter_property_modifier_start()? {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            self.report_if_escaped_keyword(token);
            match token.tag {
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
        if !is_parameter_property_modifier_tag(self.current_token.tag) {
            return Some(false);
        }
        let next = self.peek_ahead()?;
        if next.has_line_terminator_before() {
            return Some(false);
        }
        Some(can_follow_parameter_property_modifier(next.tag))
    }

    fn apply_decorators_to_pattern(
        &mut self,
        pattern: BindingPattern,
        decorators: TypedSubRange<Decorator>,
    ) {
        if decorators.is_empty() {
            return;
        }
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(node) => {
                node.set_decorators(&mut self.ast, decorators)
            }
            BindingPatternData::AssignmentPattern(node) => {
                node.set_decorators(&mut self.ast, decorators)
            }
            BindingPatternData::BindingRestElement(node) => {
                node.set_decorators(&mut self.ast, decorators)
            }
            BindingPatternData::ArrayPattern(node) => {
                node.set_decorators(&mut self.ast, decorators)
            }
            BindingPatternData::ObjectPattern(node) => {
                node.set_decorators(&mut self.ast, decorators)
            }
            BindingPatternData::SimpleAssignmentTarget(_) => {}
        }
    }

    fn parse_function_body(&mut self) -> Option<FunctionBody> {
        let start = self.current_token.span.start;
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start function body",
            Some("Function bodies must be enclosed in braces: function name() { ... }"),
        ) {
            return None;
        }
        let boundary_context = self.boundary_context;
        self.boundary_context.insert(BoundaryContext::IN_FUNCTION);
        let ParsedBody { directives, body } = self.parse_body(
            Some(TokenTag::RightBrace),
            StatementContext::StatementListItem,
        );
        self.boundary_context = boundary_context;
        let end = self.current_token.span.end;
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Import);
        let start = self.current_token.span.start;
        self.advance()?;
        self.parse_import_declaration_from(start)
    }

    fn parse_import_declaration_from(&mut self, start: u32) -> Option<Stmt> {
        if self.current_token.tag == TokenTag::StringLiteral {
            return self.parse_side_effect_import(start, None);
        }

        let mut phase = None;
        let mut import_kind = ImportOrExportKind::Value;
        let next = self.peek_ahead()?;
        if self.lang.is_typescript()
            && self.current_token.tag == TokenTag::Type
            && self.is_type_import_modifier(next)
        {
            import_kind = ImportOrExportKind::Type;
            self.advance()?;
        } else if self.current_token.tag == TokenTag::Source && self.is_source_phase_import(next) {
            phase = Some(ImportPhase::Source);
            self.advance()?;
        } else if self.current_token.tag == TokenTag::Defer && next.tag == TokenTag::Star {
            phase = Some(ImportPhase::Defer);
            self.advance()?;
        }

        if self.lang.is_typescript()
            && self.current_token.tag.is_identifier_like()
            && self.peek_ahead()?.tag == TokenTag::Assign
        {
            return self.parse_import_equals_body(start, import_kind);
        }

        let specifiers = self.parse_import_clause()?;
        if self.current_token.tag != TokenTag::From {
            self.report_expected(
                self.current_token.span,
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
        if matches!(after_type.tag, TokenTag::LeftBrace | TokenTag::Star) {
            return true;
        }
        if !after_type.tag.is_identifier_like() {
            return false;
        }
        if after_type.tag == TokenTag::From {
            let mut peek = self.begin_peek();
            let after_from = (|| {
                peek.next()?;
                peek.next()
            })();
            peek.end();
            return matches!(
                after_from.map(|token| token.tag),
                Some(TokenTag::From | TokenTag::Assign)
            );
        }
        true
    }

    fn is_source_phase_import(&mut self, after_source: Token) -> bool {
        if !after_source.tag.is_identifier_like() {
            return false;
        }
        if after_source.tag != TokenTag::From {
            return true;
        }

        let mut peek = self.begin_peek();
        let after_from = (|| {
            peek.next()?;
            peek.next()
        })();
        peek.end();
        after_from.is_some_and(|token| token.tag == TokenTag::From)
    }

    fn parse_side_effect_import(&mut self, start: u32, phase: Option<ImportPhase>) -> Option<Stmt> {
        let source = self.parse_module_specifier()?;
        let attributes = self.parse_with_clause();
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        let specifiers = TypedSubRange::empty();
        Some(self.ast.stmt_import_declaration(
            Span::new(start, end),
            specifiers,
            source,
            attributes,
            phase,
            ImportOrExportKind::Value,
        ))
    }

    fn parse_import_clause(&mut self) -> Option<TypedSubRange<ImportDeclarationSpecifier>> {
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            if self.current_token.tag == TokenTag::Star {
                let specifier = self.parse_import_namespace_specifier()?;
                self.scratch_a
                    .append(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                        specifier,
                    ));
                return Some(self.add_extra_from_scratch_a(checkpoint));
            }
            if self.current_token.tag == TokenTag::LeftBrace {
                return self.parse_named_imports();
            }

            let specifier = self.parse_import_default_specifier()?;
            self.scratch_a
                .append(ImportDeclarationSpecifier::ImportDefaultSpecifier(
                    specifier,
                ));
            if self.eat(TokenTag::Comma)? {
                if self.current_token.tag == TokenTag::Star {
                    let specifier = self.parse_import_namespace_specifier()?;
                    self.scratch_a
                        .append(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                            specifier,
                        ));
                } else if self.current_token.tag == TokenTag::LeftBrace {
                    let named = self.parse_named_imports()?;
                    for id in named.iter() {
                        self.scratch_a.append(self.ast.get_node_in_sub_range(id));
                    }
                } else {
                    self.report_expected(
                        self.current_token.span,
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
        let start = self.current_token.span.start;
        let local = self.parse_imported_binding()?;
        Some(
            self.ast
                .import_default_specifier(Span::new(start, local.span(&self.ast).end), local),
        )
    }

    fn parse_import_namespace_specifier(&mut self) -> Option<ImportNamespaceSpecifier> {
        let start = self.current_token.span.start;
        if !self.expect(TokenTag::Star, "Expected '*' for namespace import", None) {
            return None;
        }
        if self.current_token.tag != TokenTag::As {
            self.report_expected(
                self.current_token.span,
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

    fn parse_named_imports(&mut self) -> Option<TypedSubRange<ImportDeclarationSpecifier>> {
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start named imports",
            None,
        ) {
            return None;
        }
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while self.current_token.tag != TokenTag::RightBrace
                && self.current_token.tag != TokenTag::Eof
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
        let start = self.current_token.span.start;
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
        self.validate_identifier(parts.name_token, "an imported binding");
        let decorators = TypedSubRange::empty();
        let local = self
            .ast
            .binding_identifier(local_span, name, decorators, None, false);
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
            && first_token.tag == TokenTag::Type
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
        if self.current_token.tag == TokenTag::As {
            let first_as_token = self.current_token;
            let first_as = self.parse_module_export_name()?;
            if self.current_token.tag == TokenTag::As {
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
        self.current_token.tag.is_identifier_like()
            || self.current_token.tag == TokenTag::StringLiteral
    }

    fn parse_imported_binding(&mut self) -> Option<BindingIdentifier> {
        self.parse_binding_identifier()
    }

    fn parse_module_specifier(&mut self) -> Option<StringLiteral> {
        if self.current_token.tag != TokenTag::StringLiteral {
            self.report_expected(
                self.current_token.span,
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
        if self.current_token.tag == TokenTag::Require
            && self.peek_ahead()?.tag == TokenTag::LeftParen
        {
            return self.parse_external_module_reference();
        }

        let head = self.parse_identifier()?;
        let NodeKindData::IdentifierReference(head) = self.ast.kind_data(head.node_id()) else {
            unreachable!()
        };
        if self.current_token.tag != TokenTag::Dot {
            return Some(TsModuleReference::TsTypeName(
                TsTypeName::IdentifierReference(head),
            ));
        }

        let mut left = TsQualifiedNameLeft::IdentifierReference(head);
        let start = head.span(&self.ast).start;
        loop {
            self.advance()?;
            let right_token = self.current_token;
            if !right_token.tag.is_identifier_like() {
                self.report_expected(
                    self.current_token.span,
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
            let right_name = self.identifier_name(right_token);
            self.advance_without_escape_check()?;
            let right = self.ast.identifier_name(right_token.span, right_name);
            let qualified =
                self.ast
                    .ts_qualified_name(Span::new(start, right_token.span.end), left, right);
            if self.current_token.tag != TokenTag::Dot {
                return Some(TsModuleReference::TsTypeName(TsTypeName::TsQualifiedName(
                    qualified,
                )));
            }
            left = TsQualifiedNameLeft::TsQualifiedName(qualified);
        }
    }

    fn parse_external_module_reference(&mut self) -> Option<TsModuleReference> {
        let start = self.current_token.span.start;
        self.advance()?;
        if !self.expect(
            TokenTag::LeftParen,
            "Expected '(' after 'require'",
            Some("External module references are written 'require(\"module\")'"),
        ) {
            return None;
        }
        let expression = self.parse_string_literal()?;
        let end = self.current_token.span.end;
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Export);
        self.enter_module_context();
        let start = self.current_token.span.start;
        self.advance()?;
        if self.lang.is_typescript() {
            match self.current_token.tag {
                TokenTag::Assign => return self.parse_ts_export_assignment(start),
                TokenTag::As => return self.parse_ts_namespace_export_declaration(start),
                TokenTag::Type => {
                    let next = self.peek_ahead()?;
                    if next.tag == TokenTag::LeftBrace {
                        self.advance()?;
                        return self
                            .parse_export_named_from_clause(start, ImportOrExportKind::Type);
                    }
                    if next.tag == TokenTag::Star {
                        self.advance()?;
                        return self.parse_export_all_declaration(start, ImportOrExportKind::Type);
                    }
                }
                _ => {}
            }
        }
        match self.current_token.tag {
            TokenTag::Default => self.parse_export_default_declaration(start),
            TokenTag::Star => self.parse_export_all_declaration(start, ImportOrExportKind::Value),
            TokenTag::LeftBrace => {
                self.parse_export_named_from_clause(start, ImportOrExportKind::Value)
            }
            _ => self.parse_export_with_declaration(start),
        }
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
        if self.current_token.tag != TokenTag::Namespace {
            self.report_expected(
                self.current_token.span,
                "Expected 'namespace' after 'export as'",
                ReportOptions::default(),
            );
            return None;
        }
        self.advance()?;
        self.parse_namespace_export_declaration(start)
    }

    fn parse_namespace_export_declaration(&mut self, start: u32) -> Option<Stmt> {
        let id = self.parse_identifier_name()?;
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

    fn parse_export_default_part(&mut self) -> Option<DefaultExportPart> {
        let is_ts = self.lang.is_typescript();
        let tag = self.current_token.tag;

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

        if tag == TokenTag::Async && !self.current_token.has_line_terminator_before() {
            let next = self.peek_ahead()?;
            if next.tag == TokenTag::Function && !next.has_line_terminator_before() {
                let async_start = self.current_token.span.start;
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
                let abstract_start = self.current_token.span.start;
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
                let interface_start = self.current_token.span.start;
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

        let expression = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        Some(DefaultExportPart {
            declaration: ExportDefaultDeclarationKind::Expr(expression),
            needs_semi: true,
        })
    }

    fn is_abstract_class_next(&mut self) -> Option<bool> {
        let next = self.peek_ahead()?;
        Some(next.tag == TokenTag::Class && !next.has_line_terminator_before())
    }

    fn is_legacy_accessibility_import(&mut self) -> Option<bool> {
        let next = self.peek_ahead()?;
        Some(next.tag == TokenTag::Import && !next.has_line_terminator_before())
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
        if self.current_token.tag != TokenTag::From {
            self.report_expected(
                self.current_token.span,
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
            let specifiers = TypedSubRange::empty();
            let attributes = TypedSubRange::empty();
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(declaration),
                specifiers,
                None,
                attributes,
                export_kind,
            ));
        }
        if self.lang.is_typescript() && self.current_token.tag == TokenTag::Import {
            let statement = self.parse_import_declaration()?;
            let StmtData::Declaration(declaration) = self.ast.stmt_data(statement) else {
                self.report(
                    statement.span(&self.ast),
                    "expected an import-equals declaration after 'export'",
                    ReportOptions::default(),
                );
                return None;
            };
            let DeclData::TsImportEqualsDeclaration(_) = self.ast.decl_data(declaration) else {
                unreachable!()
            };
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, declaration.span(&self.ast).end),
                Some(declaration),
                TypedSubRange::empty(),
                None,
                TypedSubRange::empty(),
                ImportOrExportKind::Value,
            ));
        }
        if self.lang.is_typescript()
            && matches!(
                self.current_token.tag,
                TokenTag::Public | TokenTag::Private | TokenTag::Static
            )
            && self.is_legacy_accessibility_import()?
        {
            let declaration_start = self.current_token.span.start;
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
                TypedSubRange::empty(),
                None,
                TypedSubRange::empty(),
                ImportOrExportKind::Value,
            ));
        }
        if matches!(
            self.current_token.tag,
            TokenTag::Var | TokenTag::Let | TokenTag::Const
        ) {
            let declaration =
                self.parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)?;
            let end = declaration.span(&self.ast).end;
            let specifiers = TypedSubRange::empty();
            let attributes = TypedSubRange::empty();
            return Some(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(Decl::VariableDeclaration(declaration)),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ));
        }
        if matches!(
            self.current_token.tag,
            TokenTag::Function | TokenTag::Async | TokenTag::Class | TokenTag::At
        ) {
            let declaration = if self.current_token.tag == TokenTag::At {
                let (decorators_start, decorators) = self.parse_decorators()?;
                Decl::Class(self.parse_class_decorated(
                    ParseClassOpts::default(),
                    Some(decorators_start),
                    decorators,
                )?)
            } else if self.current_token.tag == TokenTag::Class {
                Decl::Class(self.parse_class(ParseClassOpts::default(), None)?)
            } else {
                let is_async = self.current_token.tag == TokenTag::Async;
                let function_start = if is_async {
                    let start = self.current_token.span.start;
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
            let specifiers = TypedSubRange::empty();
            let attributes = TypedSubRange::empty();
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

    fn report_missing_export_declaration(&mut self) -> Option<Stmt> {
        self.report_expected(
            self.current_token.span,
            "Expected declaration after 'export'",
            ReportOptions::default(),
        );
        None
    }

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
        let result = self.parse_export_specifiers()?;
        let (source, attributes) = if self.eat(TokenTag::From)? {
            let source = self.parse_module_specifier()?;
            let attributes = self.parse_with_clause();
            (Some(source), attributes)
        } else {
            self.resolve_local_export_specifiers(&result)?;
            (None, TypedSubRange::empty())
        };
        let end = self.eat_semicolon(self.lexer.state.prev_token_end)?;
        Some(self.ast.stmt_export_named_declaration(
            Span::new(start, end),
            None,
            result.specifiers,
            source,
            attributes,
            export_kind,
        ))
    }

    fn parse_export_specifiers(&mut self) -> Option<ExportSpecifiersResult> {
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start export specifiers",
            None,
        ) {
            return None;
        }
        let checkpoint = self.scratch_a.begin();
        let mut local_tags = Vec::new();
        let result = (|| {
            while self.current_token.tag != TokenTag::RightBrace
                && self.current_token.tag != TokenTag::Eof
            {
                local_tags.push(self.current_token.tag);
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
            Some(ExportSpecifiersResult {
                specifiers: self.add_extra_from_scratch_a(checkpoint),
                local_tags,
            })
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_export_specifier(&mut self) -> Option<ExportSpecifier> {
        let start = self.current_token.span.start;
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

    fn resolve_local_export_specifiers(&mut self, result: &ExportSpecifiersResult) -> Option<()> {
        for (index, local_tag) in result.local_tags.iter().copied().enumerate() {
            let specifier = result
                .specifiers
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

    fn parse_with_clause(&mut self) -> TypedSubRange<ImportAttribute> {
        if !matches!(self.current_token.tag, TokenTag::With | TokenTag::Assert) {
            return TypedSubRange::empty();
        }
        if self.advance().is_none() {
            return TypedSubRange::empty();
        }
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' after 'with' in import attributes",
            None,
        ) {
            return TypedSubRange::empty();
        }
        let checkpoint = self.scratch_a.begin();
        while self.current_token.tag != TokenTag::RightBrace
            && self.current_token.tag != TokenTag::Eof
        {
            let attribute = match self.parse_import_attribute() {
                Some(attribute) => attribute,
                None => {
                    self.scratch_a.reset(checkpoint);
                    return TypedSubRange::empty();
                }
            };
            self.scratch_a.append(attribute);
            if self.current_token.tag == TokenTag::Comma {
                if self.advance().is_none() {
                    self.scratch_a.reset(checkpoint);
                    return TypedSubRange::empty();
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
            return TypedSubRange::empty();
        }
        let attributes = self.add_extra_from_scratch_a(checkpoint);
        self.scratch_a.reset(checkpoint);
        attributes
    }

    fn parse_import_attribute(&mut self) -> Option<ImportAttribute> {
        let start = self.current_token.span.start;
        let key = self.parse_attribute_key()?;
        if !self.expect(TokenTag::Colon, "Expected ':' in import attribute", None) {
            return None;
        }
        if self.current_token.tag != TokenTag::StringLiteral {
            self.report(
                self.current_token.span,
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
        if token.tag == TokenTag::StringLiteral {
            return Some(ModuleExportName::StringLiteral(
                self.parse_string_literal()?,
            ));
        }
        if token.tag.is_identifier_like() {
            return Some(ModuleExportName::IdentifierName(
                self.parse_identifier_name()?,
            ));
        }
        self.report_expected(
            self.current_token.span,
            "Expected identifier or string literal for attribute key",
            ReportOptions::default(),
        );
        None
    }

    fn parse_module_export_name(&mut self) -> Option<ModuleExportName> {
        let token = self.current_token;
        if token.tag == TokenTag::StringLiteral {
            if token.has_lone_surrogates() {
                self.report(
                    token.span,
                    "An export name cannot include a unicode lone surrogate",
                    ReportOptions::default(),
                );
                return None;
            }
            return Some(ModuleExportName::StringLiteral(
                self.parse_string_literal()?,
            ));
        }
        if token.tag.is_identifier_like() {
            return Some(ModuleExportName::IdentifierName(
                self.parse_identifier_name()?,
            ));
        }
        self.report_expected(
            self.current_token.span,
            "Expected identifier or string literal",
            ReportOptions::default(),
        );
        None
    }

    fn is_start_of_ts_declaration(&mut self) -> Option<bool> {
        if !self.lang.is_typescript() {
            return Some(false);
        }
        let mut current = self.current_token;
        let mut peek = self.begin_peek();
        let mut has_declare = false;
        let mut has_abstract = false;

        if current.tag == TokenTag::Declare {
            current = peek.next()?;
            if current.has_line_terminator_before() {
                peek.end();
                return Some(false);
            }
            has_declare = true;
        }
        if current.tag == TokenTag::Abstract {
            let next = peek.next()?;
            if next.has_line_terminator_before() || next.tag != TokenTag::Class {
                peek.end();
                return Some(false);
            }
            current = next;
            has_abstract = true;
        }
        if current.tag == TokenTag::Const {
            let next = peek.next()?;
            if is_const_enum_head(next) {
                current = next;
            } else if has_declare {
                let result = !next.has_line_terminator_before() && can_start_binding(next.tag);
                peek.end();
                return Some(result);
            } else {
                peek.end();
                return Some(false);
            }
        }

        let result = match current.tag {
            TokenTag::Enum => {
                let name = peek.next()?;
                name.tag.is_identifier_like() && !name.tag.is_unconditionally_reserved()
            }
            TokenTag::Type | TokenTag::Interface | TokenTag::Namespace => {
                is_declaration_name(peek.next()?)
            }
            TokenTag::Module => {
                let name = peek.next()?;
                !name.has_line_terminator_before()
                    && (is_declaration_name(name) || name.tag == TokenTag::StringLiteral)
            }
            TokenTag::Global => {
                let next = peek.next()?;
                next.tag == TokenTag::LeftBrace && !next.has_line_terminator_before()
            }
            TokenTag::Var | TokenTag::Let | TokenTag::Function | TokenTag::Class => {
                if !has_declare && !has_abstract {
                    false
                } else {
                    let name = peek.next()?;
                    !name.has_line_terminator_before()
                        && match current.tag {
                            TokenTag::Var | TokenTag::Let => can_start_binding(name.tag),
                            _ => is_declaration_name(name),
                        }
                }
            }
            TokenTag::Async => {
                if !has_declare {
                    false
                } else {
                    let function = peek.next()?;
                    if function.tag != TokenTag::Function || function.has_line_terminator_before() {
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
                        assign.tag == TokenTag::Assign && !assign.has_line_terminator_before()
                    }
                }
            }
            _ => false,
        };
        peek.end();
        Some(result)
    }

    fn parse_ts_declaration(&mut self) -> Option<Stmt> {
        let start = self.current_token.span.start;
        let mut modifiers = TsDeclarationModifiers::default();
        if self.eat(TokenTag::Declare)? {
            modifiers.declare = true;
        }
        if self.eat(TokenTag::Abstract)? {
            modifiers.r#abstract = true;
        }
        if self.current_token.tag == TokenTag::Const {
            let next = self.peek_ahead()?;
            if is_const_enum_head(next) {
                modifiers.is_const = true;
                self.advance()?;
            }
        }

        let ambient = self.ts_context.ambient();
        self.ts_context
            .set(TsContext::AMBIENT, ambient || modifiers.declare);
        let result = (|| match self.current_token.tag {
            TokenTag::Type => self.parse_type_alias_declaration(start, modifiers.declare),
            TokenTag::Interface => self.parse_interface_declaration(start, modifiers.declare),
            TokenTag::Enum => {
                self.parse_enum_declaration(start, modifiers.is_const, modifiers.declare)
            }
            TokenTag::Var | TokenTag::Let | TokenTag::Const => {
                let declaration = self.parse_variable_declaration(
                    ParseVariableDeclarationOpts {
                        is_declare: modifiers.declare,
                        ..ParseVariableDeclarationOpts::default()
                    },
                    Some(start),
                )?;
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Type);
        self.advance()?;
        let id = self.parse_binding_identifier()?;
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Interface);
        self.advance()?;
        let id = self.parse_binding_identifier()?;
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
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
                HeritageKind::Interface => HeritageClause::Interface(TypedSubRange::empty()),
                HeritageKind::Class => HeritageClause::Class(TypedSubRange::empty()),
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
        let type_arguments = if is_angle_open(self.current_token.tag) {
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
        if self.current_token.tag != TokenTag::LeftBrace {
            self.report_expected(
                self.current_token.span,
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
        let start = self.current_token.span.start;
        let (body, end) = self.parse_object_type_members()?;
        Some(self.ast.ts_interface_body(Span::new(start, end), body))
    }

    fn parse_enum_declaration(
        &mut self,
        start: u32,
        is_const: bool,
        declare: bool,
    ) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Enum);
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
        let start = self.current_token.span.start;
        if self.current_token.tag != TokenTag::LeftBrace {
            self.report_expected(
                self.current_token.span,
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
        let result = (|| {
            while self.current_token.tag != TokenTag::RightBrace
                && self.current_token.tag != TokenTag::Eof
            {
                let member = self.parse_enum_member()?;
                self.scratch_a.append(member);
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            let end = self.current_token.span.end;
            if !self.expect(
                TokenTag::RightBrace,
                "Expected '}' to close an enum body",
                Some("Each '{' in an enum must be matched by a '}'"),
            ) {
                return None;
            }
            let members = self.add_extra_from_scratch_a(checkpoint);
            Some(self.ast.ts_enum_body(Span::new(start, end), members))
        })();
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_enum_member(&mut self) -> Option<TsEnumMember> {
        let (id, computed) = self.parse_enum_member_name()?;
        let start = id.span(&self.ast).start;
        let initializer = if self.eat(TokenTag::Assign)? {
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
        let token = self.current_token;
        if self.eat(TokenTag::LeftBracket)? {
            let expression =
                self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
            if !self.expect(
                TokenTag::RightBracket,
                "Expected ']' to close a computed enum member name",
                Some("A computed enum member name is written '[\"name\"]'"),
            ) {
                return None;
            }
            return Some((TsEnumMemberName::Expr(expression), true));
        }
        if token.tag.is_identifier_like() {
            return Some((
                TsEnumMemberName::IdentifierName(self.parse_identifier_name()?),
                false,
            ));
        }
        if token.tag == TokenTag::StringLiteral {
            let literal = self.parse_string_literal()?;
            return Some((TsEnumMemberName::StringLiteral(literal), false));
        }
        if token.tag == TokenTag::NoSubstitutionTemplate {
            let literal = self.parse_no_substitution_template(false)?;
            return Some((TsEnumMemberName::TemplateLiteral(literal), false));
        }
        let found = self.describe_token(self.current_token);
        self.report(
            self.current_token.span,
            Cow::Owned(format!(
                "Unexpected token '{}' as enum member name",
                found.as_ref()
            )),
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Enum member names must be identifiers, string literals, or computed expressions '[name]'.",
                )),
                ..ReportOptions::default()
            },
        );
        None
    }

    fn parse_module_declaration(
        &mut self,
        start: u32,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> Option<Stmt> {
        self.advance()?;
        let id = if kind == TsModuleDeclarationKind::Module
            && self.current_token.tag == TokenTag::StringLiteral
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Global);
        let token = self.current_token;
        self.advance()?;
        let name = self.identifier_name(token);
        let id = self.ast.identifier_name(token.span, name);
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
        if self.current_token.tag != TokenTag::Dot {
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
        if self.current_token.tag != TokenTag::LeftBrace {
            return Some(None);
        }
        Some(Some(TsModuleBody::TsModuleBlock(
            self.parse_module_block()?,
        )))
    }

    fn parse_module_block(&mut self) -> Option<TsModuleBlock> {
        let start = self.current_token.span.start;
        if !self.expect(
            TokenTag::LeftBrace,
            "Expected '{' to start a module body",
            Some("A module body is written '{ <statements> }'"),
        ) {
            return None;
        }
        let boundary_context = mem::take(&mut self.boundary_context);
        let ParsedBody { directives, body } =
            self.parse_body(Some(TokenTag::RightBrace), StatementContext::ModuleItem);
        self.boundary_context = boundary_context;
        let end = self.current_token.span.end;
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
        if self.current_token.tag == TokenTag::Intrinsic && !self.current_token.is_escaped() {
            let next = self.peek_ahead()?;
            if !continues_type(next.tag) {
                let span = self.current_token.span;
                self.advance()?;
                return Some(self.ast.ts_type_ts_intrinsic_keyword(span));
            }
        }
        self.parse_type()
    }

    fn parse_type_annotation(&mut self) -> Option<TsTypeAnnotation> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Colon);
        let start = self.current_token.span.start;
        self.advance()?;
        let type_annotation = self.parse_type()?;
        let end = type_annotation.span(&self.ast).end;
        Some(
            self.ast
                .ts_type_annotation(Span::new(start, end), type_annotation),
        )
    }

    fn parse_return_type_annotation(&mut self) -> Option<TsTypeAnnotation> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Colon);
        let start = self.current_token.span.start;
        self.advance()?;
        let type_annotation = self.parse_type_or_type_predicate()?;
        let end = type_annotation.span(&self.ast).end;
        Some(
            self.ast
                .ts_type_annotation(Span::new(start, end), type_annotation),
        )
    }

    fn parse_type_or_type_predicate(&mut self) -> Option<TsType> {
        if !self.is_identifier_predicate_start()? {
            return self.parse_type();
        }
        let parameter_name = TsTypePredicateName::IdentifierName(self.parse_identifier_name()?);
        self.finish_type_predicate(parameter_name.span(&self.ast).start, parameter_name, false)
    }

    fn parse_this_type_or_predicate(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::This);
        let token = self.current_token;
        self.advance()?;
        if self.current_token.tag != TokenTag::Is
            || self.current_token.is_escaped()
            || self.current_token.has_line_terminator_before()
        {
            return Some(self.ast.ts_type_ts_this_type(token.span));
        }
        let parameter_name = self.ast.ts_type_predicate_name_ts_this_type(token.span);
        self.finish_type_predicate(token.span.start, parameter_name, false)
    }

    fn parse_asserts_type_predicate(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Asserts);
        let start = self.current_token.span.start;
        self.advance()?;
        let parameter_name = self.parse_predicate_parameter_name()?;
        self.finish_type_predicate(start, parameter_name, true)
    }

    fn parse_predicate_parameter_name(&mut self) -> Option<TsTypePredicateName> {
        let token = self.current_token;
        if token.tag == TokenTag::This {
            self.advance()?;
            return Some(self.ast.ts_type_predicate_name_ts_this_type(token.span));
        }
        Some(TsTypePredicateName::IdentifierName(
            self.parse_identifier_name()?,
        ))
    }

    fn finish_type_predicate(
        &mut self,
        start: u32,
        parameter_name: TsTypePredicateName,
        asserts: bool,
    ) -> Option<TsType> {
        let mut end = parameter_name.span(&self.ast).end;
        let type_annotation =
            if self.current_token.tag == TokenTag::Is && !self.current_token.is_escaped() {
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
        if self.current_token.tag != TokenTag::Asserts || self.current_token.is_escaped() {
            return Some(false);
        }
        let next = self.peek_ahead()?;
        Some(
            !next.has_line_terminator_before()
                && (next.tag == TokenTag::This || next.tag.is_identifier_like()),
        )
    }

    fn is_identifier_predicate_start(&mut self) -> Option<bool> {
        let current = self.current_token;
        if current.is_escaped()
            || current.tag == TokenTag::This
            || !current.tag.is_identifier_like()
        {
            return Some(false);
        }
        let next = self.peek_ahead()?;
        Some(next.tag == TokenTag::Is && !next.is_escaped() && !next.has_line_terminator_before())
    }

    fn parse_type(&mut self) -> Option<TsType> {
        let saved = self.ts_context.disallow_conditional_types();
        self.ts_context
            .remove(TsContext::DISALLOW_CONDITIONAL_TYPES);
        let result = (|| {
            if self.is_start_of_function_or_constructor_type()? {
                self.parse_function_or_constructor_type()
            } else {
                self.parse_conditional_type().or_else(|| {
                    self.report_expected(
                        self.current_token.span,
                        "Expected a type",
                        ReportOptions::default(),
                    );
                    None
                })
            }
        })();
        self.ts_context
            .set(TsContext::DISALLOW_CONDITIONAL_TYPES, saved);
        result
    }

    fn parse_type_no_conditional(&mut self) -> Option<TsType> {
        if self.is_start_of_function_or_constructor_type()? {
            self.parse_function_or_constructor_type()
        } else {
            self.parse_union_type().or_else(|| {
                self.report_expected(
                    self.current_token.span,
                    "Expected a type",
                    ReportOptions::default(),
                );
                None
            })
        }
    }

    fn parse_conditional_type(&mut self) -> Option<TsType> {
        let check_type = self.parse_union_type()?;
        if self.ts_context.disallow_conditional_types()
            || self.current_token.tag != TokenTag::Extends
            || self.current_token.has_line_terminator_before()
        {
            return Some(check_type);
        }
        self.advance()?;
        let saved = self.ts_context.disallow_conditional_types();
        self.ts_context
            .insert(TsContext::DISALLOW_CONDITIONAL_TYPES);
        let extends_type = self.parse_type_no_conditional();
        self.ts_context
            .set(TsContext::DISALLOW_CONDITIONAL_TYPES, saved);
        let extends_type = extends_type?;
        if !self.expect(
            TokenTag::Question,
            "Expected '?' in a conditional type",
            Some("A conditional type is written 'T extends U ? X : Y'"),
        ) {
            return None;
        }
        let true_type = self.parse_type()?;
        if !self.expect(
            TokenTag::Colon,
            "Expected ':' in a conditional type",
            Some("A conditional type is written 'T extends U ? X : Y'"),
        ) {
            return None;
        }
        let false_type = self.parse_type()?;
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
        self.parse_binary_type_chain(TokenTag::BitwiseOr, BinaryTypeKind::Union)
    }

    fn parse_binary_type_chain(
        &mut self,
        separator: TokenTag,
        kind: BinaryTypeKind,
    ) -> Option<TsType> {
        let leading_start = if self.current_token.tag == separator {
            let start = self.current_token.span.start;
            self.advance()?;
            Some(start)
        } else {
            None
        };
        let first = match kind {
            BinaryTypeKind::Union => self.parse_intersection_type()?,
            BinaryTypeKind::Intersection => self.parse_postfix_type()?,
        };
        if leading_start.is_none() && self.current_token.tag != separator {
            return Some(first);
        }
        let checkpoint = self.scratch_a.begin();
        self.scratch_a.append(first);
        let result = (|| {
            let mut last = first;
            while self.eat(separator)? {
                last = match kind {
                    BinaryTypeKind::Union => self.parse_intersection_type()?,
                    BinaryTypeKind::Intersection => self.parse_postfix_type()?,
                };
                self.scratch_a.append(last);
            }
            let span = Span::new(
                leading_start.unwrap_or(first.span(&self.ast).start),
                last.span(&self.ast).end,
            );
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
        self.parse_binary_type_chain(TokenTag::BitwiseAnd, BinaryTypeKind::Intersection)
    }

    fn parse_postfix_type(&mut self) -> Option<TsType> {
        let mut type_annotation = self.parse_primary_type()?;
        while !self.current_token.has_line_terminator_before() {
            match self.current_token.tag {
                TokenTag::LeftBracket => {
                    type_annotation = self.parse_array_or_indexed_access_type(type_annotation)?;
                }
                TokenTag::Question => {
                    if !self.is_postfix_nullable()? {
                        break;
                    }
                    type_annotation =
                        self.parse_jsdoc_postfix(type_annotation, JsDocKind::Nullable)?;
                }
                TokenTag::LogicalNot => {
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
        if self.current_token.tag == TokenTag::RightBracket {
            let end = self.current_token.span.end;
            self.advance()?;
            return Some(TsType::TsArrayType(
                self.ast.ts_array_type(Span::new(start, end), object_type),
            ));
        }
        let index_type = self.parse_type()?;
        let end = self.current_token.span.end;
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
        let token = self.current_token;
        if matches!(
            token.tag,
            TokenTag::True
                | TokenTag::False
                | TokenTag::StringLiteral
                | TokenTag::NumericLiteral
                | TokenTag::HexLiteral
                | TokenTag::OctalLiteral
                | TokenTag::BinaryLiteral
                | TokenTag::BigIntLiteral
                | TokenTag::NoSubstitutionTemplate
                | TokenTag::Minus
                | TokenTag::Plus
        ) {
            return self.parse_literal_type();
        }
        if !token.is_escaped() {
            if is_type_keyword(token.tag)
                && (token.tag == TokenTag::Void || !self.is_qualified_type_continuation()?)
            {
                return self.parse_type_keyword();
            }
            if token.tag == TokenTag::This {
                return self.parse_this_type_or_predicate();
            }
            if token.tag == TokenTag::Asserts && self.is_asserts_predicate_start()? {
                return self.parse_asserts_type_predicate();
            }
            match token.tag {
                TokenTag::LeftParen => return self.parse_parenthesized_type(),
                TokenTag::LeftBracket => return self.parse_tuple_type(),
                TokenTag::LeftBrace => {
                    return if self.is_start_of_mapped_type()? {
                        self.parse_mapped_type()
                    } else {
                        self.parse_type_literal()
                    };
                }
                TokenTag::Keyof | TokenTag::Unique | TokenTag::Readonly => {
                    return self.parse_type_operator();
                }
                TokenTag::Infer => return self.parse_infer_type(),
                TokenTag::TemplateHead => return self.parse_template_literal_type(),
                TokenTag::Typeof => return self.parse_type_query(),
                TokenTag::Import => return self.parse_import_type(),
                TokenTag::Question => return self.parse_jsdoc_nullable_or_unknown_type(),
                TokenTag::LogicalNot => return self.parse_jsdoc_non_nullable_type(),
                _ => {}
            }
        }
        if token.tag.is_identifier_like() {
            return self.parse_type_reference();
        }
        None
    }

    fn parse_type_keyword(&mut self) -> Option<TsType> {
        let token = self.current_token;
        let keyword = match token.tag {
            TokenTag::Any => self.ast.ts_type_ts_any_keyword(token.span),
            TokenTag::Unknown => self.ast.ts_type_ts_unknown_keyword(token.span),
            TokenTag::Never => self.ast.ts_type_ts_never_keyword(token.span),
            TokenTag::Void => self.ast.ts_type_ts_void_keyword(token.span),
            TokenTag::NullLiteral => self.ast.ts_type_ts_null_keyword(token.span),
            TokenTag::Undefined => self.ast.ts_type_ts_undefined_keyword(token.span),
            TokenTag::String => self.ast.ts_type_ts_string_keyword(token.span),
            TokenTag::Number => self.ast.ts_type_ts_number_keyword(token.span),
            TokenTag::Bigint => self.ast.ts_type_ts_big_int_keyword(token.span),
            TokenTag::Boolean => self.ast.ts_type_ts_boolean_keyword(token.span),
            TokenTag::Symbol => self.ast.ts_type_ts_symbol_keyword(token.span),
            TokenTag::Object => self.ast.ts_type_ts_object_keyword(token.span),
            _ => unreachable!(),
        };
        self.advance()?;
        Some(keyword)
    }

    fn is_postfix_nullable(&mut self) -> Option<bool> {
        let next = self.peek_ahead()?;
        Some(!is_start_of_type(next.tag))
    }

    fn parse_jsdoc_postfix(&mut self, inner: TsType, kind: JsDocKind) -> Option<TsType> {
        let span = Span::new(inner.span(&self.ast).start, self.current_token.span.end);
        self.advance()?;
        Some(self.jsdoc_node_data(inner, kind, true, span))
    }

    fn parse_parenthesized_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftParen);
        let start = self.current_token.span.start;
        self.advance()?;
        let type_annotation = self.parse_type()?;
        let end = self.current_token.span.end;
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
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBracket);
        let start = self.current_token.span.start;
        self.advance()?;
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while self.current_token.tag != TokenTag::RightBracket
                && self.current_token.tag != TokenTag::Eof
            {
                let element = self.parse_tuple_element()?;
                self.scratch_a.append(element);
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            let end = self.current_token.span.end;
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
        if self.current_token.tag == TokenTag::Spread {
            let start = self.current_token.span.start;
            self.advance()?;
            let inner = self.parse_tuple_element_body()?;
            if let TsTupleElementData::TsNamedTupleMember(named) =
                self.ast.ts_tuple_element_data(inner)
            {
                let element_type = named.element_type(&self.ast);
                let rest = self.ast.ts_rest_type(
                    Span::new(start, element_type.span(&self.ast).end),
                    element_type,
                );
                let named = self.ast.ts_named_tuple_member(
                    Span::new(start, named.span(&self.ast).end),
                    named.label(&self.ast),
                    TsType::TsRestType(rest),
                    named.optional(&self.ast),
                );
                return Some(TsTupleElement::TsNamedTupleMember(named));
            }
            let inner_type = tuple_element_as_type(inner, &self.ast);
            let span = Span::new(start, inner_type.span(&self.ast).end);
            return Some(TsTupleElement::TsType(TsType::TsRestType(
                self.ast.ts_rest_type(span, inner_type),
            )));
        }
        self.parse_tuple_element_body()
    }

    fn parse_tuple_element_body(&mut self) -> Option<TsTupleElement> {
        if self.is_named_tuple_element()? {
            return self.parse_named_tuple_member();
        }
        let type_annotation = self.parse_type()?;
        if let TsTypeData::TsJsDocNullableType(nullable) = self.ast.ts_type_data(type_annotation)
            && nullable.postfix(&self.ast)
        {
            return Some(TsTupleElement::TsType(TsType::TsOptionalType(
                self.ast.ts_optional_type(
                    type_annotation.span(&self.ast),
                    nullable.type_annotation(&self.ast),
                ),
            )));
        }
        Some(TsTupleElement::TsType(type_annotation))
    }

    fn is_named_tuple_element(&mut self) -> Option<bool> {
        if !self.current_token.tag.is_identifier_like() {
            return Some(false);
        }
        let first = self.peek_ahead()?;
        if first.tag == TokenTag::Colon {
            return Some(true);
        }
        if first.tag != TokenTag::Question {
            return Some(false);
        }
        let mut peek = self.begin_peek();
        let second = (|| {
            peek.next()?;
            peek.next()
        })();
        peek.end();
        Some(second?.tag == TokenTag::Colon)
    }

    fn parse_named_tuple_member(&mut self) -> Option<TsTupleElement> {
        let start = self.current_token.span.start;
        let label = self.parse_identifier_name()?;
        let optional = self.eat(TokenTag::Question)?;
        if !self.expect(
            TokenTag::Colon,
            "Expected ':' after named tuple element label",
            None,
        ) {
            return None;
        }
        let element_type = self.parse_type()?;
        let span = Span::new(start, element_type.span(&self.ast).end);
        Some(TsTupleElement::TsNamedTupleMember(
            self.ast
                .ts_named_tuple_member(span, label, element_type, optional),
        ))
    }

    fn parse_type_operator(&mut self) -> Option<TsType> {
        let token = self.current_token;
        let operator = match token.tag {
            TokenTag::Keyof => TsTypeOperatorKind::Keyof,
            TokenTag::Unique => TsTypeOperatorKind::Unique,
            TokenTag::Readonly => TsTypeOperatorKind::Readonly,
            _ => unreachable!(),
        };
        self.advance()?;
        let type_annotation = self.parse_postfix_type()?;
        let span = Span::new(token.span.start, type_annotation.span(&self.ast).end);
        Some(TsType::TsTypeOperator(self.ast.ts_type_operator(
            span,
            operator,
            type_annotation,
        )))
    }

    fn parse_infer_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Infer);
        let start = self.current_token.span.start;
        self.advance()?;
        let name_start = self.current_token.span.start;
        let name = self.parse_binding_identifier()?;
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
        if self.current_token.tag != TokenTag::Extends {
            return Some(None);
        }

        let checkpoint = self.checkpoint();
        self.advance()?;
        self.ts_context
            .insert(TsContext::DISALLOW_CONDITIONAL_TYPES);
        let constraint = match self.parse_union_type() {
            Some(constraint) => constraint,
            None => {
                self.rewind(checkpoint);
                return Some(None);
            }
        };
        self.ts_context.set(
            TsContext::DISALLOW_CONDITIONAL_TYPES,
            checkpoint.ts_context.disallow_conditional_types(),
        );

        if !checkpoint.ts_context.disallow_conditional_types()
            && self.current_token.tag == TokenTag::Question
        {
            self.rewind(checkpoint);
            return Some(None);
        }
        Some(Some(constraint))
    }

    fn is_start_of_function_or_constructor_type(&mut self) -> Option<bool> {
        let tag = self.current_token.tag;
        if tag == TokenTag::New {
            return Some(true);
        }
        if tag == TokenTag::Abstract {
            return Some(self.peek_ahead()?.tag == TokenTag::New);
        }
        if is_angle_open(tag) {
            return Some(true);
        }
        if tag != TokenTag::LeftParen {
            return Some(false);
        }
        let mut peek = self.begin_peek();
        let first = peek.next()?;
        match first.tag {
            TokenTag::RightParen => {
                let result = peek.next()?.tag == TokenTag::Arrow;
                peek.end();
                return Some(result);
            }
            TokenTag::Spread => {
                peek.end();
                return Some(true);
            }
            TokenTag::This => {
                let result = peek.next()?.tag == TokenTag::Colon;
                peek.end();
                return Some(result);
            }
            TokenTag::LeftBrace | TokenTag::LeftBracket => {
                peek.end();
                return self.is_function_type_after_pattern();
            }
            _ => {}
        }
        if !first.tag.is_identifier_like() {
            peek.end();
            return Some(false);
        }
        let second = peek.next()?;
        let result = match second.tag {
            TokenTag::Colon | TokenTag::Question | TokenTag::Comma | TokenTag::Assign => true,
            TokenTag::RightParen => peek.next()?.tag == TokenTag::Arrow,
            _ => false,
        };
        peek.end();
        Some(result)
    }

    fn is_function_type_after_pattern(&mut self) -> Option<bool> {
        let checkpoint = self.checkpoint();
        let result = (|| {
            self.advance()?;
            if self.parse_binding_pattern().is_none() {
                return Some(false);
            }
            Some(match self.current_token.tag {
                TokenTag::Colon | TokenTag::Comma | TokenTag::Question | TokenTag::Assign => true,
                TokenTag::RightParen => {
                    self.advance()?;
                    self.current_token.tag == TokenTag::Arrow
                }
                _ => false,
            })
        })();
        self.rewind(checkpoint);
        result
    }

    fn parse_function_or_constructor_type(&mut self) -> Option<TsType> {
        let start = self.current_token.span.start;
        let is_abstract = self.eat(TokenTag::Abstract)?;
        let is_constructor = self.eat(TokenTag::New)?;
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        let params = self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        if self.current_token.tag != TokenTag::Arrow {
            self.report_expected(
                self.current_token.span,
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
        let arrow_start = self.current_token.span.start;
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
                params,
                return_type,
            ))
        })
    }

    fn parse_type_query(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Typeof);
        let start = self.current_token.span.start;
        self.advance()?;
        let expr_name = if self.current_token.tag == TokenTag::Import {
            let import_type = self.parse_import_type()?;
            let NodeKindData::TsImportType(import_type) = self.ast.kind_data(import_type.node_id())
            else {
                unreachable!()
            };
            TsTypeQueryExprName::TsImportType(import_type)
        } else if self.current_token.tag == TokenTag::This {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            if self.current_token.tag == TokenTag::Dot {
                let left = self.ast.ts_qualified_name_left_this_expression(token.span);
                TsTypeQueryExprName::TsTypeName(self.extend_qualified_name(left, token.span.start)?)
            } else {
                self.ast.ts_type_query_expr_name_this_expression(token.span)
            }
        } else {
            TsTypeQueryExprName::TsTypeName(self.parse_entity_name()?)
        };
        let type_arguments = if expr_name.is_ts_import_type(&self.ast) {
            None
        } else {
            self.parse_type_arguments_after_entity_name()?
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Import);
        let start = self.current_token.span.start;
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
        let type_arguments = if is_angle_open(self.current_token.tag) {
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
        if self.current_token.tag == TokenTag::RightParen {
            return Some(None);
        }
        let options = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        self.eat(TokenTag::Comma)?;
        Some(Some(options))
    }

    fn parse_import_type_qualifier(&mut self) -> Option<TsImportTypeQualifier> {
        let identifier = self.parse_identifier_name()?;
        if self.current_token.tag != TokenTag::Dot {
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
        while self.current_token.tag == TokenTag::Dot {
            self.advance()?;
            let token = self.current_token;
            if !token.tag.is_identifier_like() {
                self.report_expected(
                    self.current_token.span,
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
            let right = self.ast.identifier_name(token.span, name);
            self.advance_without_escape_check()?;
            let qualified =
                self.ast
                    .ts_qualified_name(Span::new(start, token.span.end), left, right);
            result = Some(qualified);
            left = TsQualifiedNameLeft::TsQualifiedName(qualified);
        }
        Some(TsTypeName::TsQualifiedName(
            result.expect("qualified-name parsing starts on '.'"),
        ))
    }

    fn parse_jsdoc_nullable_or_unknown_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Question);
        let token = self.current_token;
        self.advance()?;
        if !is_start_of_type(self.current_token.tag) {
            return Some(TsType::TsJsDocUnknownType(
                self.ast.ts_js_doc_unknown_type(token.span),
            ));
        }
        let type_annotation = self.parse_type()?;
        let span = Span::new(token.span.start, type_annotation.span(&self.ast).end);
        Some(self.jsdoc_node_data(type_annotation, JsDocKind::Nullable, false, span))
    }

    fn parse_jsdoc_non_nullable_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LogicalNot);
        let start = self.current_token.span.start;
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
        let next = self.peek_ahead()?;
        Some(next.tag == TokenTag::Dot && !next.has_line_terminator_before())
    }

    fn parse_literal_type(&mut self) -> Option<TsType> {
        let start = self.current_token.span.start;
        let literal = match self.current_token.tag {
            TokenTag::True | TokenTag::False => {
                let token = self.current_token;
                self.advance()?;
                TsLiteral::BooleanLiteral(
                    self.ast
                        .boolean_literal(token.span, token.tag == TokenTag::True),
                )
            }
            TokenTag::StringLiteral => TsLiteral::StringLiteral(self.parse_string_literal()?),
            TokenTag::NoSubstitutionTemplate => {
                TsLiteral::TemplateLiteral(self.parse_no_substitution_template(false)?)
            }
            TokenTag::Minus | TokenTag::Plus => self.parse_signed_numeric_literal_type()?,
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
        let sign_token = self.current_token;
        let next = self.peek_ahead()?;
        if !next.tag.is_numeric_literal() {
            return None;
        }
        self.advance()?;
        let expression = self.parse_numeric_literal()?;
        let operator = if sign_token.tag == TokenTag::Minus {
            UnaryOperator::Negate
        } else {
            UnaryOperator::Positive
        };
        Some(TsLiteral::UnaryExpression(self.ast.unary_expression(
            Span::new(sign_token.span.start, expression.span(&self.ast).end),
            expression,
            operator,
        )))
    }

    fn parse_template_literal_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::TemplateHead);
        let start = self.current_token.span.start;
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
                if self.current_token.tag != TokenTag::RightBrace {
                    self.report_expected(
                        self.current_token.span,
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
                    .re_scan_template_continuation(right_brace.span.start)
                {
                    Ok(token) => token,
                    Err(error) => {
                        self.report(
                            right_brace.span,
                            lexical_error_message(error),
                            ReportOptions {
                                help: Some(Cow::Borrowed(lexical_error_help(error))),
                                ..ReportOptions::default()
                            },
                        );
                        return None;
                    }
                };
                let tail = continuation.tag == TokenTag::TemplateTail;
                let element = self.add_template_element(continuation, tail, false);
                self.scratch_a.append(element);
                let end = continuation.span.end;
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

    fn is_start_of_mapped_type(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let mut peek = self.begin_peek();
        let Some(mut token) = peek.next() else {
            return Some(false);
        };
        if matches!(token.tag, TokenTag::Plus | TokenTag::Minus) {
            let Some(next) = peek.next() else {
                return Some(false);
            };
            token = next;
            if token.tag != TokenTag::Readonly {
                return Some(false);
            }
            let Some(next) = peek.next() else {
                return Some(false);
            };
            token = next;
        } else if token.tag == TokenTag::Readonly {
            let Some(next) = peek.next() else {
                return Some(false);
            };
            token = next;
        }
        if token.tag != TokenTag::LeftBracket {
            return Some(false);
        }
        let Some(name) = peek.next() else {
            return Some(false);
        };
        if !name.tag.is_identifier_like() {
            return Some(false);
        }
        let Some(operator) = peek.next() else {
            return Some(false);
        };
        Some(operator.tag == TokenTag::In)
    }

    fn parse_mapped_type(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let start = self.current_token.span.start;
        self.advance()?;
        let readonly = self.parse_mapped_modifier(TokenTag::Readonly)?;
        if !self.expect(
            TokenTag::LeftBracket,
            "Expected '[' in a mapped type",
            Some("A mapped type is written '{ [K in T]: V }'"),
        ) {
            return None;
        }
        let key = self.parse_binding_identifier()?;
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
            self.current_token.tag,
            TokenTag::Semicolon | TokenTag::Comma
        ) {
            self.advance()?;
        }
        let end = self.current_token.span.end;
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
        if self.current_token.tag == terminator {
            self.advance()?;
            return Some(TsMappedTypeModifier::True);
        }
        let modifier = match self.current_token.tag {
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

    fn parse_type_literal(&mut self) -> Option<TsType> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let start = self.current_token.span.start;
        let (members, end) = self.parse_object_type_members()?;
        Some(TsType::TsTypeLiteral(
            self.ast.ts_type_literal(Span::new(start, end), members),
        ))
    }

    fn parse_object_type_members(&mut self) -> Option<(TypedSubRange<TsTypeElement>, u32)> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        self.advance()?;
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while self.current_token.tag != TokenTag::RightBrace
                && self.current_token.tag != TokenTag::Eof
            {
                let member = self.parse_type_member()?;
                self.scratch_a.append(member);
                if !self.consume_type_member_separator(member)? {
                    return None;
                }
            }
            let end = self.current_token.span.end;
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
        match self.current_token.tag {
            TokenTag::Semicolon | TokenTag::Comma => {
                let end = self.current_token.span.end;
                self.extend_span_to(member, end);
                self.advance()?;
            }
            TokenTag::RightBrace => {}
            _ if self.current_token.has_line_terminator_before() => {}
            _ => {
                self.report_expected(
                    self.current_token.span,
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

    fn parse_type_member(&mut self) -> Option<TsTypeElement> {
        let tag = self.current_token.tag;
        if matches!(tag, TokenTag::LeftParen | TokenTag::LessThan) {
            return self.parse_call_or_construct_signature(false);
        }
        if tag == TokenTag::New
            && matches!(
                self.peek_ahead()?.tag,
                TokenTag::LeftParen | TokenTag::LessThan
            )
        {
            return self.parse_call_or_construct_signature(true);
        }

        let mut readonly = false;
        let start = self.current_token.span.start;
        if tag == TokenTag::Readonly {
            let next = self.peek_ahead()?;
            if !next.has_line_terminator_before() && can_follow_readonly_modifier(next.tag) {
                readonly = true;
                self.advance()?;
            }
        }
        if self.current_token.tag == TokenTag::LeftBracket && self.is_index_signature_start()? {
            return self.parse_index_signature(start, readonly, false);
        }
        self.parse_property_or_method_signature(start, readonly)
    }

    fn parse_call_or_construct_signature(&mut self, is_construct: bool) -> Option<TsTypeElement> {
        let start = self.current_token.span.start;
        if is_construct {
            self.advance()?;
        }
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        let params = self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        let return_type = if self.current_token.tag == TokenTag::Colon {
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
                params,
                return_type,
            )
        })
    }

    fn is_index_signature_start(&mut self) -> Option<bool> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBracket);
        let mut peek = self.begin_peek();
        let Some(name) = peek.next() else {
            return Some(false);
        };
        if !name.tag.is_identifier_like() {
            return Some(false);
        }
        let Some(second) = peek.next() else {
            return Some(false);
        };
        Some(matches!(second.tag, TokenTag::Colon | TokenTag::Comma))
    }

    fn parse_index_signature(
        &mut self,
        start: u32,
        readonly: bool,
        r#static: bool,
    ) -> Option<TsTypeElement> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBracket);
        self.advance()?;
        let parameter = self.parse_index_signature_parameter()?;
        if self.current_token.tag == TokenTag::Comma {
            return {
                let span = self.current_token.span;
                self.report(
                    span,
                    "An index signature must have exactly one parameter",
                    ReportOptions::default(),
                );
                None
            };
        }
        let parameters = self.ast.add_typed_sub_range([parameter]);
        if !self.expect(
            TokenTag::RightBracket,
            "Expected ']' to close an index signature parameter list",
            Some("An index signature is written '[name: KeyType]: ValueType'"),
        ) {
            return None;
        }
        if self.current_token.tag != TokenTag::Colon {
            self.report_expected(
                self.current_token.span,
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

    fn parse_index_signature_parameter(&mut self) -> Option<BindingIdentifier> {
        let name = self.parse_binding_identifier()?;
        if self.current_token.tag != TokenTag::Colon {
            self.report_expected(
                self.current_token.span,
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
        name.set_type_annotation(&mut self.ast, Some(annotation));
        let span = Span::new(name.span(&self.ast).start, annotation.span(&self.ast).end);
        name.set_span(&mut self.ast, span);
        Some(name)
    }

    fn parse_property_or_method_signature(
        &mut self,
        start: u32,
        readonly: bool,
    ) -> Option<TsTypeElement> {
        let mut kind = TsMethodSignatureKind::Method;
        let head_tag = self.current_token.tag;
        if matches!(head_tag, TokenTag::Get | TokenTag::Set)
            && can_follow_accessor_keyword(self.peek_ahead()?.tag)
        {
            kind = if head_tag == TokenTag::Get {
                TsMethodSignatureKind::Get
            } else {
                TsMethodSignatureKind::Set
            };
            self.advance()?;
        }
        let computed = self.current_token.tag == TokenTag::LeftBracket;
        let key = self.parse_property_key(false)?;
        let mut end = key.span(&self.ast).end;
        let optional = if self.current_token.tag == TokenTag::Question {
            end = self.current_token.span.end;
            self.advance()?;
            true
        } else {
            false
        };
        if kind != TsMethodSignatureKind::Method
            || matches!(
                self.current_token.tag,
                TokenTag::LeftParen | TokenTag::LessThan
            )
        {
            return self.parse_method_signature_body(start, key, kind, computed, optional);
        }
        let type_annotation = if self.current_token.tag == TokenTag::Colon {
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
        start: u32,
        key: PropertyKey,
        kind: TsMethodSignatureKind,
        computed: bool,
        optional: bool,
    ) -> Option<TsTypeElement> {
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        let params = self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        let return_type = if self.current_token.tag == TokenTag::Colon {
            let annotation = self.parse_return_type_annotation()?;
            if kind == TsMethodSignatureKind::Set {
                self.report(
                    annotation.span(&self.ast),
                    "A 'set' accessor cannot have a return type annotation",
                    ReportOptions::default(),
                );
                return None;
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
            params,
            return_type,
            kind,
            computed,
            optional,
        ))
    }

    fn parse_type_reference(&mut self) -> Option<TsType> {
        let type_name = self.parse_entity_name()?;
        let start = type_name.span(&self.ast).start;
        let type_arguments = self.parse_type_arguments_after_entity_name()?;
        let end = type_arguments.map_or(type_name.span(&self.ast).end, |args| {
            args.span(&self.ast).end
        });
        Some(
            self.ast
                .ts_type_ts_type_reference(Span::new(start, end), type_name, type_arguments),
        )
    }

    fn parse_entity_name(&mut self) -> Option<TsTypeName> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            self.report_expected(
                token.span,
                "Expected an identifier",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "An entity name must start with an identifier",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let name = self.identifier_name(token);
        let mut type_name = self.ast.ts_type_name_identifier_reference(token.span, name);
        self.advance_without_escape_check()?;
        while self.current_token.tag == TokenTag::Dot {
            type_name = self.parse_qualified_segment(type_name)?;
        }
        Some(type_name)
    }

    fn parse_qualified_segment(&mut self, left: TsTypeName) -> Option<TsTypeName> {
        let start = left.span(&self.ast).start;
        self.advance()?;
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            self.report_expected(
                self.current_token.span,
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
        let right = self.ast.identifier_name(token.span, name);
        self.advance_without_escape_check()?;
        let left = match self.ast.ts_type_name_data(left) {
            TsTypeNameData::IdentifierReference(identifier) => {
                TsQualifiedNameLeft::IdentifierReference(identifier)
            }
            TsTypeNameData::TsQualifiedName(qualified) => {
                TsQualifiedNameLeft::TsQualifiedName(qualified)
            }
        };
        Some(TsTypeName::TsQualifiedName(self.ast.ts_qualified_name(
            Span::new(start, token.span.end),
            left,
            right,
        )))
    }

    fn parse_type_arguments_after_entity_name(
        &mut self,
    ) -> Option<Option<TsTypeParameterInstantiation>> {
        if !is_angle_open(self.current_token.tag) || self.current_token.has_line_terminator_before()
        {
            return Some(None);
        }
        self.parse_type_arguments().map(Some)
    }

    fn parse_type_parameters(&mut self) -> Option<TsTypeParameterDeclaration> {
        match self.parse_angle_list(AngleListKind::Parameters, AngleClose::Peel)? {
            AngleList::Parameters(parameters) => Some(parameters),
            AngleList::Arguments(_) => unreachable!(),
        }
    }

    fn parse_type_arguments(&mut self) -> Option<TsTypeParameterInstantiation> {
        match self.parse_angle_list(AngleListKind::Arguments, AngleClose::Peel)? {
            AngleList::Arguments(arguments) => Some(arguments),
            AngleList::Parameters(_) => unreachable!(),
        }
    }

    fn parse_type_arguments_in_expression(&mut self) -> Option<TsTypeParameterInstantiation> {
        match self.parse_angle_list(AngleListKind::Arguments, AngleClose::Exact)? {
            AngleList::Arguments(arguments) => Some(arguments),
            AngleList::Parameters(_) => unreachable!(),
        }
    }

    fn parse_angle_list(&mut self, kind: AngleListKind, close: AngleClose) -> Option<AngleList> {
        let start = self.consume_angle_open()?;
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            let mut parsed_any = false;
            while !is_angle_close(self.current_token.tag) && self.current_token.tag != TokenTag::Eof
            {
                let node = match kind {
                    AngleListKind::Arguments => self.parse_type()?.node_id(),
                    AngleListKind::Parameters => self.parse_type_parameter()?.node_id(),
                };
                self.scratch_a.items.push(node);
                parsed_any = true;
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            if !parsed_any {
                self.report(
                    self.current_token.span,
                    match kind {
                        AngleListKind::Arguments => "A type argument list cannot be empty",
                        AngleListKind::Parameters => "A type parameter list cannot be empty",
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
                AngleListKind::Parameters => {
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

    fn parse_type_parameter(&mut self) -> Option<TsTypeParameter> {
        let mut is_const = false;
        let mut is_in = false;
        let mut is_out = false;
        let mut start = self.current_token.span.start;
        let mut start_set = false;
        loop {
            let token = self.current_token;
            let in_after_out = token.tag == TokenTag::In && is_out;
            let flag = match token.tag {
                TokenTag::Const => &mut is_const,
                TokenTag::In => &mut is_in,
                TokenTag::Out => &mut is_out,
                _ => break,
            };
            let Some(next) = self.peek_ahead() else {
                break;
            };
            if !next.tag.is_identifier_like() {
                break;
            }
            if *flag {
                let text = token.tag.to_str().unwrap_or("modifier");
                self.report(
                    token.span,
                    Cow::Owned(format!("Duplicate '{text}' modifier on type parameter")),
                    ReportOptions::default(),
                );
            } else if in_after_out {
                self.report(
                    token.span,
                    "'in' modifier must precede 'out' modifier.",
                    ReportOptions::default(),
                );
            }
            *flag = true;
            if !start_set {
                start = token.span.start;
                start_set = true;
            }
            self.advance()?;
        }

        let name = self.parse_binding_identifier()?;
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
        let start = self.current_token.span.start;
        match self.current_token.tag {
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
        match self.current_token.tag {
            TokenTag::GreaterThan => {
                let end = self.current_token.span.end;
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
                let token = self
                    .lexer
                    .re_scan_greater_than(self.current_token.span.start);
                let end = token.span.end;
                self.advance_with_rescanned_token(token)?;
                Some(end)
            }
            _ => {
                self.report_expected(
                    self.current_token.span,
                    match kind {
                        AngleListKind::Arguments => "Expected '>' to close a type argument list",
                        AngleListKind::Parameters => "Expected '>' to close a type parameter list",
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
        debug_assert_eq!(self.current_token.tag, TokenTag::For);
        let start = self.current_token.span.start;
        self.advance()?;
        if self.current_token.tag == TokenTag::Await {
            if !self.context.contains(Context::AWAIT) && !self.boundary_context.can_be_module() {
                self.report(
                    self.current_token.span,
                    "'for await' is only valid in async functions or modules",
                    ReportOptions::default(),
                );
            }
            if self.boundary_context.can_be_module() {
                self.mark_module_syntax();
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
        if self.current_token.tag == TokenTag::Semicolon {
            return self.parse_for_statement_rest(start, None, is_for_await);
        }

        let declaration_start = self.current_token.span.start;
        match self.current_token.tag {
            TokenTag::Var | TokenTag::Const => {
                let kind = self.parse_variable_kind(false)?;
                self.parse_for_with_declaration(start, is_for_await, kind, declaration_start)
            }
            TokenTag::Let if !self.current_token.is_escaped() => {
                let next = self.peek_ahead()?;
                if !can_start_let_binding(next.tag) {
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
                if next.has_line_terminator_before() || !can_start_binding_identifier(next.tag) {
                    return self.parse_for_with_expression(start, is_for_await);
                }

                if next.tag == TokenTag::Of {
                    let mut peek = self.begin_peek();
                    let after_of = (|| {
                        peek.next()?;
                        peek.next()
                    })();
                    peek.end();
                    if !matches!(
                        after_of?.tag,
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
                let await_span = self.current_token.span;
                let next = self.peek_ahead()?;
                if next.tag != TokenTag::Using || next.has_line_terminator_before() {
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
        let checkpoint = self.scratch_a.begin();
        let declarators_result = (|| {
            let first = self.parse_variable_declarator(kind, DeclaratorCtx::ForLoop)?;
            let mut declaration_end = first.span(&self.ast).end;
            self.scratch_a.append(first);
            while self.eat(TokenTag::Comma)? {
                let declarator = self.parse_variable_declarator(kind, DeclaratorCtx::ForLoop)?;
                declaration_end = declarator.span(&self.ast).end;
                self.scratch_a.append(declarator);
            }
            let declarators = self.add_extra_from_scratch_a(checkpoint);
            Some((declarators, declaration_end))
        })();
        self.context = context;
        self.scratch_a.reset(checkpoint);
        let (declarators, declaration_end) = declarators_result?;
        let declaration = self.ast.variable_declaration(
            Span::new(declaration_start, declaration_end),
            kind,
            declarators,
            false,
        );

        match self.current_token.tag {
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
                self.validate_regular_for_declarators(declaration)?;
                self.parse_for_statement_rest(
                    start,
                    Some(ForStatementInit::VariableDeclaration(declaration)),
                    is_for_await,
                )
            }
        }
    }

    fn parse_for_with_expression(&mut self, start: u32, is_for_await: bool) -> Option<Stmt> {
        let head = self.current_token;
        let head_is_let = head.tag == TokenTag::Let && !head.is_escaped();
        let context = enter_grammar_context![&mut self.context, ~In];
        let expression = self.parse_expr(
            Precedence::Comma,
            ParseExpressionOpts {
                respect_in_parameter: true,
                ..ParseExpressionOpts::default()
            },
        );
        self.context = context;
        let expression = expression?;

        if self.current_token.tag == TokenTag::In {
            let left = ForStatementLeft::AssignmentTarget(
                self.expression_to_assignment_target(expression)?,
            );
            return self.parse_for_in_statement_rest(start, left, is_for_await);
        }

        if self.current_token.tag == TokenTag::Of {
            if head_is_let {
                self.report(
                    head.span,
                    "The left-hand side of a for-of statement may not start with 'let'",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Rename the variable or wrap it in parentheses: 'for ((let).x of ...)'",
                        )),
                        ..ReportOptions::default()
                    },
                );
            }
            if !is_for_await && self.is_async_identifier(expression) {
                self.report(
                    expression.span(&self.ast),
                    "'for (async of ...)' is not allowed, it is ambiguous with 'for await'",
                    ReportOptions::default(),
                );
            }
            let left = ForStatementLeft::AssignmentTarget(
                self.expression_to_assignment_target(expression)?,
            );
            return self.parse_for_of_statement_rest(start, left, is_for_await);
        }

        self.parse_for_statement_rest(
            start,
            Some(ForStatementInit::Expr(expression)),
            is_for_await,
        )
    }

    fn parse_for_statement_rest(
        &mut self,
        start: u32,
        init: Option<ForStatementInit>,
        is_for_await: bool,
    ) -> Option<Stmt> {
        if is_for_await {
            return {
                let span = self.current_token.span;
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
        let test = if self.current_token.tag == TokenTag::Semicolon {
            None
        } else {
            Some(self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?)
        };
        if !self.expect(
            TokenTag::Semicolon,
            "Expected ';' after for-loop condition",
            None,
        ) {
            return None;
        }
        let update = if self.current_token.tag == TokenTag::RightParen {
            None
        } else {
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

    fn parse_for_in_statement_rest(
        &mut self,
        start: u32,
        left: ForStatementLeft,
        is_for_await: bool,
    ) -> Option<Stmt> {
        if is_for_await {
            return {
                let span = self.current_token.span;
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

    fn parse_for_of_statement_rest(
        &mut self,
        start: u32,
        left: ForStatementLeft,
        is_for_await: bool,
    ) -> Option<Stmt> {
        self.report_if_escaped_keyword(self.current_token);
        self.advance_without_escape_check()?;
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

    fn validate_regular_for_declarators(&mut self, declaration: VariableDeclaration) -> Option<()> {
        let kind = declaration.kind(&self.ast);
        for index in 0..declaration.declarators(&self.ast).len() {
            let declarator = declaration
                .declarators(&self.ast)
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
                return None;
            }
            if kind == VariableKind::Const {
                self.report(
                    id.span(&self.ast),
                    "'const' declarations in for loop initializer must be initialized",
                    ReportOptions::default(),
                );
                return None;
            }
        }
        Some(())
    }

    fn parse_switch_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Switch);
        let start = self.current_token.span.start;
        self.advance()?;
        if !self.expect(TokenTag::LeftParen, "Expected '(' after 'switch'", None) {
            return None;
        }
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
        let cases = self.parse_switch_cases();
        let end = self.current_token.span.end;
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

    fn parse_switch_cases(&mut self) -> TypedSubRange<SwitchCase> {
        let checkpoint = self.scratch_a.begin();
        while matches!(self.current_token.tag, TokenTag::Case | TokenTag::Default) {
            match self.parse_switch_case() {
                Some(case) => self.scratch_a.append(case),
                None => self.recover(Some(TokenTag::RightBrace)),
            }
        }
        let cases = self.add_extra_from_scratch_a(checkpoint);
        self.scratch_a.reset(checkpoint);
        cases
    }

    fn parse_switch_case(&mut self) -> Option<SwitchCase> {
        debug_assert!(matches!(
            self.current_token.tag,
            TokenTag::Case | TokenTag::Default
        ));
        let case_start = self.current_token.span.start;
        let test = if self.eat(TokenTag::Case)? {
            Some(self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?)
        } else {
            self.advance()?;
            None
        };
        let colon_end = self.current_token.span.end;
        if !self.expect(TokenTag::Colon, "Expected ':' after case", None) {
            return None;
        }
        let consequent = self.parse_case_consequent();
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

    fn parse_case_consequent(&mut self) -> TypedSubRange<Stmt> {
        let checkpoint = self.scratch_b.begin();
        while !matches!(
            self.current_token.tag,
            TokenTag::Case | TokenTag::Default | TokenTag::RightBrace | TokenTag::Eof
        ) {
            let Some(statement) = self.parse_statement(StatementContext::StatementListItem) else {
                break;
            };
            if let StmtData::Declaration(declaration) = self.ast.stmt_data(statement)
                && let DeclData::VariableDeclaration(declaration) = self.ast.decl_data(declaration)
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
        let result = self.add_extra_from_scratch_b(checkpoint);
        self.scratch_b.reset(checkpoint);
        result
    }

    fn parse_try_statement(&mut self) -> Option<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Try);
        let start = self.current_token.span.start;
        self.advance()?;
        let block = self.parse_block_statement()?;
        let handler = if self.current_token.tag == TokenTag::Catch {
            Some(self.parse_catch_clause()?)
        } else {
            None
        };
        let finalizer = if self.eat(TokenTag::Finally)? {
            Some(self.parse_block_statement()?)
        } else {
            None
        };
        if handler.is_none() && finalizer.is_none() {
            return {
                let span = self.current_token.span;
                self.report(
                    span,
                    "Try statement requires catch or finally clause",
                    ReportOptions::default(),
                );
                None
            };
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Catch);
        let start = self.current_token.span.start;
        self.advance()?;
        let param = if self.eat(TokenTag::LeftParen)? {
            let param = self.parse_binding_pattern()?;
            if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                let annotation = self.parse_type_annotation()?;
                self.apply_type_annotation_to_pattern(param, annotation);
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
        Some(
            self.ast
                .catch_clause(Span::new(start, body.span(&self.ast).end), param, body),
        )
    }

    #[inline(always)]
    fn parse_expr(&mut self, minimum: Precedence, opts: ParseExpressionOpts) -> Option<Expr> {
        let left = if self.current_token.tag == TokenTag::Identifier {
            self.parse_plain_identifier()?
        } else {
            self.parse_prefix(opts.in_cover, minimum)?
        };
        self.parse_expr_from_left(left, minimum, opts)
    }

    #[inline(always)]
    fn parse_expr_preserving_stripped_paren(
        &mut self,
        minimum: Precedence,
        opts: ParseExpressionOpts,
    ) -> Option<Expr> {
        let stripped_paren = self.stripped_paren;
        let expression = self.parse_expr(minimum, opts);
        self.stripped_paren = stripped_paren;
        expression
    }

    #[inline(always)]
    fn parse_expr_from_left(
        &mut self,
        mut left: Expr,
        minimum: Precedence,
        opts: ParseExpressionOpts,
    ) -> Option<Expr> {
        let is_ts = self.lang.is_typescript();

        loop {
            let token = self.current_token;
            if is_ts
                && is_angle_open(token.tag)
                && max_left_precedence(left, &self.ast) >= Precedence::Call
                && let Some(expression) = self.parse_type_argumented_call_or_instantiation(left)?
            {
                left = expression;
                continue;
            }
            let precedence = self.infix_precedence(token);
            if precedence < minimum || precedence == Precedence::Lowest {
                break;
            }
            if precedence > max_left_precedence(left, &self.ast) {
                break;
            }
            if opts.respect_in_parameter
                && token.tag == TokenTag::In
                && !self.context.contains(Context::IN)
            {
                break;
            }
            left = if token.tag.is_binary_operator() {
                self.parse_binary_expression(left, token.tag)?
            } else if token.tag.is_logical_operator() {
                self.parse_logical_expression(left, token.tag)?
            } else if token.tag.is_assignment_operator() {
                self.parse_assignment_expression(
                    precedence,
                    left,
                    token.tag,
                    opts.respect_in_parameter,
                )?
            } else {
                self.parse_infix(precedence, left)?
            };
        }
        Some(left)
    }

    fn parse_expression_in_cover(&mut self, precedence: Precedence) -> Option<Expr> {
        self.parse_expr(
            precedence,
            ParseExpressionOpts {
                in_cover: true,
                ..ParseExpressionOpts::default()
            },
        )
    }

    fn parse_assignment_expression_in_cover(&mut self) -> Option<Expr> {
        self.parse_expression_in_cover(Precedence::Assignment)
    }

    fn infix_precedence(&self, token: Token) -> Precedence {
        match token.tag {
            TokenTag::Increment | TokenTag::Decrement | TokenTag::As | TokenTag::Satisfies
                if token.has_line_terminator_before() =>
            {
                Precedence::Lowest
            }
            TokenTag::LogicalNot
                if !self.lang.is_typescript() || token.has_line_terminator_before() =>
            {
                Precedence::Lowest
            }
            _ => token.tag.precedence(),
        }
    }

    fn parse_infix(&mut self, precedence: Precedence, left: Expr) -> Option<Expr> {
        let token = self.current_token;

        match token.tag {
            TokenTag::As | TokenTag::Satisfies if self.lang.is_typescript() => {
                return self.parse_as_or_satisfies_expression(left, token.tag);
            }
            TokenTag::LogicalNot
                if self.lang.is_typescript() && !token.has_line_terminator_before() =>
            {
                return self.parse_non_null_expression(left);
            }
            _ => {}
        }

        if token.tag == TokenTag::Arrow && !token.has_line_terminator_before() {
            return self.parse_simple_arrow_function(left);
        }

        match token.tag {
            TokenTag::Increment | TokenTag::Decrement => {
                self.parse_update_expression(false, Some(left))
            }
            TokenTag::Question => self.parse_conditional_expression(left),
            TokenTag::Comma => self.parse_sequence_expression(precedence, left),
            TokenTag::Dot => self.parse_static_member_expression(left, false),
            TokenTag::LeftBracket => self.parse_computed_member_expression(left, false),
            TokenTag::LeftParen => self.parse_call_expression(left, false, None),
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                self.parse_tagged_template_expression(left, None)
            }
            TokenTag::OptionalChaining => self.parse_optional_chain(left),
            _ => {
                let found = self.describe_token(token);
                self.report(
                    token.span,
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

    fn parse_prefix(&mut self, in_cover: bool, minimum: Precedence) -> Option<Expr> {
        let tag = self.current_token.tag;

        if matches!(tag, TokenTag::Increment | TokenTag::Decrement) {
            return self.parse_update_expression(true, None);
        }

        if tag == TokenTag::At {
            let (start, decorators) = self.parse_decorators()?;
            return Some(Expr::Class(self.parse_class_decorated(
                ParseClassOpts {
                    is_expression: true,
                    ..ParseClassOpts::default()
                },
                Some(start),
                decorators,
            )?));
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

        if tag == TokenTag::Await
            && (self.context.contains(Context::AWAIT) || self.boundary_context.can_be_module())
        {
            let token = self.current_token;
            let can_classify_as_module = self.boundary_context.can_be_module();
            self.advance_without_escape_check()?;
            let parse_as_await = self.context.contains(Context::AWAIT)
                || !matches!(
                    self.current_token.tag,
                    TokenTag::Semicolon
                        | TokenTag::RightBrace
                        | TokenTag::RightParen
                        | TokenTag::RightBracket
                        | TokenTag::Comma
                        | TokenTag::Colon
                        | TokenTag::Eof
                );
            if parse_as_await {
                self.report_if_escaped_keyword(token);
                let ambiguous_script_different_ast =
                    can_classify_as_module && self.is_ambiguous_await_prefix();
                let expression = self.parse_await_expression(token.span.start)?;
                if ambiguous_script_different_ast {
                    self.state
                        .insert(ParserState::AMBIGUOUS_SCRIPT_DIFFERENT_AST);
                } else if can_classify_as_module {
                    self.mark_module_syntax();
                }
                return Some(expression);
            }

            self.validate_identifier(token, "an identifier");
            let name = self.identifier_name(token);
            return Some(self.ast.expr_identifier_reference(token.span, name));
        }

        if tag == TokenTag::Yield
            && self.context.contains(Context::YIELD)
            && minimum < Precedence::Add
        {
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

        if tag == TokenTag::LessThan {
            if self.lang.is_typescript() {
                let start = self.current_token.span.start;
                if let Some(arrow) = self.try_parse_generic_arrow(false, start)? {
                    return Some(arrow);
                }
                if !self.lang.is_jsx() {
                    return self.parse_type_assertion();
                }
            }
            if self.lang.is_jsx() {
                return self.parse_jsx_expression();
            }
        }

        self.parse_primary_expression(in_cover)
    }

    fn parse_primary_expression(&mut self, in_cover: bool) -> Option<Expr> {
        let token = self.current_token;

        if token.tag.is_numeric_literal() {
            return self.parse_numeric_literal();
        }

        match token.tag {
            TokenTag::PrivateIdentifier => {
                let private = self.parse_private_identifier()?;
                if self.current_token.tag != TokenTag::In {
                    self.report(
                        token.span,
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
            TokenTag::LeftBracket => self.parse_array_expression(in_cover),
            TokenTag::LeftBrace => self.parse_object_expression(in_cover),
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
            tag if tag.is_identifier_like() => self.parse_identifier(),
            _ => {
                let found = self.describe_token(token);
                self.report(
                    token.span,
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
        let token = self.current_token;
        self.validate_identifier(token, "an identifier");
        self.parse_identifier_from_token(token)
    }

    #[inline(always)]
    fn parse_plain_identifier(&mut self) -> Option<Expr> {
        let token = self.current_token;
        debug_assert_eq!(token.tag, TokenTag::Identifier);
        self.parse_identifier_from_token(token)
    }

    #[inline(always)]
    fn parse_identifier_from_token(&mut self, token: Token) -> Option<Expr> {
        self.advance_without_escape_check()?;
        let name = self.identifier_name(token);
        Some(self.ast.expr_identifier_reference(token.span, name))
    }

    fn parse_private_identifier(&mut self) -> Option<PrivateIdentifier> {
        debug_assert_eq!(self.current_token.tag, TokenTag::PrivateIdentifier);
        let token = self.current_token;
        self.advance()?;
        let name = self.identifier_name(token);
        Some(self.ast.private_identifier(token.span, name))
    }

    #[inline(always)]
    fn parse_identifier_name(&mut self) -> Option<IdentifierName> {
        debug_assert!(self.current_token.tag.is_identifier_like());
        let token = self.current_token;
        self.advance_without_escape_check()?;
        let name = self.identifier_name(token);
        Some(self.ast.identifier_name(token.span, name))
    }

    fn parse_this_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::This);
        let token = self.current_token;
        self.advance()?;
        Some(self.ast.expr_this_expression(token.span))
    }

    fn parse_super_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Super);
        let token = self.current_token;
        self.advance()?;
        if !matches!(
            self.current_token.tag,
            TokenTag::LeftParen | TokenTag::Dot | TokenTag::LeftBracket
        ) {
            return {
                let span = self.current_token.span;
                self.report(
                    span,
                    "'super' must be followed by a call or property access",
                    ReportOptions::default(),
                );
                None
            };
        }
        Some(self.ast.expr_super(token.span))
    }

    fn parse_boolean_literal(&mut self) -> Option<Expr> {
        debug_assert!(matches!(
            self.current_token.tag,
            TokenTag::True | TokenTag::False
        ));
        let token = self.current_token;
        self.advance()?;
        Some(
            self.ast
                .expr_boolean_literal(token.span, token.tag == TokenTag::True),
        )
    }

    fn parse_null_literal(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::NullLiteral);
        let token = self.current_token;
        self.advance()?;
        Some(self.ast.expr_null_literal(token.span))
    }

    fn parse_numeric_literal(&mut self) -> Option<Expr> {
        debug_assert!(self.current_token.tag.is_numeric_literal());
        let token = self.current_token;
        if token.tag == TokenTag::BigIntLiteral {
            self.advance()?;
            let raw = self.ast.source_slice(token.span.start, token.span.end - 1);
            return Some(self.ast.expr_big_int_literal(token.span, raw));
        }
        self.advance()?;
        let raw = self.ast.source_slice(token.span.start, token.span.end);
        let raw_text = self.ast.get_utf8(raw);
        let value = match token.tag {
            TokenTag::NumericLiteral => parse_decimal_literal(raw_text),
            TokenTag::HexLiteral => parse_hex_literal(raw_text),
            TokenTag::OctalLiteral => parse_octal_literal(raw_text),
            TokenTag::BinaryLiteral => parse_binary_literal(raw_text),
            _ => unreachable!(),
        };
        Some(self.ast.expr_numeric_literal(token.span, value, Some(raw)))
    }

    fn parse_string_literal(&mut self) -> Option<StringLiteral> {
        let token = self.current_token;
        if token.tag != TokenTag::StringLiteral {
            self.report_expected(
                self.current_token.span,
                "Expected a string literal",
                ReportOptions::default(),
            );
            return None;
        }
        self.advance()?;
        let value = self.string_value(token);
        let raw = self.ast.source_slice(token.span.start, token.span.end);
        Some(self.ast.string_literal(token.span, value, raw))
    }

    fn parse_regexp_literal(&mut self) -> Option<Expr> {
        debug_assert!(matches!(
            self.current_token.tag,
            TokenTag::Slash | TokenTag::SlashAssign
        ));
        let token = self.current_token;
        let result = match self.lexer.re_scan_as_regex(token.span.start) {
            Ok(result) => result,
            Err(error) => {
                self.report(
                    token.span,
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
        debug_assert_eq!(self.current_token.tag, TokenTag::NoSubstitutionTemplate);
        let token = self.current_token;
        let quasi = self.add_template_element(token, true, tagged);
        self.advance()?;
        let quasis = self.ast.add_typed_sub_range([quasi]);
        let expressions = TypedSubRange::empty();
        Some(self.ast.template_literal(token.span, quasis, expressions))
    }

    fn parse_template_literal(&mut self, tagged: bool) -> Option<TemplateLiteral> {
        debug_assert_eq!(self.current_token.tag, TokenTag::TemplateHead);
        let start = self.current_token.span.start;
        let quasis_checkpoint = self.scratch_a.begin();
        let expressions_checkpoint = self.scratch_b.begin();
        let head = self.current_token;
        let result = (|| {
            let head = self.add_template_element(head, false, tagged);
            self.scratch_a.append(head);
            self.advance()?;
            let end = loop {
                let expression =
                    self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
                self.scratch_b.append(expression);
                if self.current_token.tag != TokenTag::RightBrace {
                    self.report_expected(
                        self.current_token.span,
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
                    .re_scan_template_continuation(right_brace.span.start)
                {
                    Ok(token) => token,
                    Err(error) => {
                        self.report(
                            right_brace.span,
                            lexical_error_message(error),
                            ReportOptions {
                                help: Some(Cow::Borrowed(lexical_error_help(error))),
                                ..ReportOptions::default()
                            },
                        );
                        return None;
                    }
                };
                let tail = continuation.tag == TokenTag::TemplateTail;
                let element = self.add_template_element(continuation, tail, tagged);
                self.scratch_a.append(element);
                let end = continuation.span.end;
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
        self.scratch_a.reset(quasis_checkpoint);
        self.scratch_b.reset(expressions_checkpoint);
        result
    }

    fn parse_tagged_template_expression(
        &mut self,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Option<Expr> {
        let start = tag.span(&self.ast).start;
        let quasi = if self.current_token.tag == TokenTag::NoSubstitutionTemplate {
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
        let raw = self.ast.source_slice(span.start, span.end);
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
        match token.tag {
            TokenTag::TemplateHead | TokenTag::TemplateMiddle => {
                Span::new(token.span.start + 1, token.span.end - 2)
            }
            TokenTag::TemplateTail | TokenTag::NoSubstitutionTemplate => {
                Span::new(token.span.start + 1, token.span.end - 1)
            }
            _ => unreachable!(),
        }
    }

    fn parse_parenthesized_or_arrow_function<const CAN_BE_ARROW: bool>(
        &mut self,
        arrow_start: Option<u32>,
    ) -> Option<Expr> {
        let start = arrow_start.unwrap_or(self.current_token.span.start);
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
        let cover = self.parse_parenthesized_cover()?;
        let is_arrow = self.current_token.tag == TokenTag::Arrow
            && !self.current_token.has_line_terminator_before()
            && CAN_BE_ARROW;
        if is_arrow {
            return self.cover_to_arrow_function(cover, false, start);
        }
        self.cover_to_parenthesized_expression(cover)
    }

    fn parse_parenthesized_cover(&mut self) -> Option<ParenthesizedCover> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftParen);
        let start = self.current_token.span.start;
        self.advance()?;
        let checkpoint = self.scratch_cover.begin();
        let context = enter_grammar_context![&mut self.context, +In];
        let allow_arrow_return_type = self.ts_context.allow_arrow_return_type();
        self.ts_context.insert(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let result = (|| {
            let mut end = start + 1;
            let mut has_trailing_comma = false;
            while self.current_token.tag != TokenTag::RightParen
                && self.current_token.tag != TokenTag::Eof
            {
                let element = if self.eat(TokenTag::Spread)? {
                    let spread_start = self.lexer.state.prev_token_end - 3;
                    let argument = self.parse_expr(
                        Precedence::Assignment,
                        ParseExpressionOpts {
                            in_cover: true,
                            ..ParseExpressionOpts::default()
                        },
                    )?;
                    Argument::SpreadElement(self.ast.spread_element(
                        Span::new(spread_start, argument.span(&self.ast).end),
                        argument,
                    ))
                } else {
                    Argument::Expr(self.parse_expr(
                        Precedence::Assignment,
                        ParseExpressionOpts {
                            in_cover: true,
                            ..ParseExpressionOpts::default()
                        },
                    )?)
                };
                end = element.span(&self.ast).end;
                self.scratch_cover.append(element);
                if self.eat(TokenTag::Comma)? {
                    has_trailing_comma = self.current_token.tag == TokenTag::RightParen;
                } else if self.current_token.tag != TokenTag::RightParen {
                    self.report_expected(
                        self.current_token.span,
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
            if self.current_token.tag != TokenTag::RightParen {
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
            end = self.current_token.span.end;
            self.advance()?;
            let elements = self.add_extra_from_scratch_cover(checkpoint);
            Some(ParenthesizedCover {
                elements,
                start,
                end,
                has_trailing_comma,
            })
        })();
        self.context = context;
        self.ts_context
            .set(TsContext::ALLOW_ARROW_RETURN_TYPE, allow_arrow_return_type);
        self.scratch_cover.reset(checkpoint);
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
            return None;
        }
        let arguments = (0..cover.elements.len())
            .map(|index| {
                cover
                    .elements
                    .get_node(&self.ast, index)
                    .expect("parenthesized cover contains its arguments")
            })
            .collect::<Vec<_>>();
        let checkpoint = self.scratch_cover.begin();
        let expression_result = (|| {
            for argument in arguments {
                match self.ast.argument_data(argument) {
                    ArgumentData::Expr(expression) => {
                        self.validate_no_cover_initialized_syntax(expression)?;
                        self.scratch_cover.append(expression);
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
            Some(if self.scratch_cover.items.len() - checkpoint == 1 {
                unsafe { Expr::from_node_id_unchecked(self.scratch_cover.items[checkpoint]) }
            } else {
                let first =
                    unsafe { Expr::from_node_id_unchecked(self.scratch_cover.items[checkpoint]) };
                let last = unsafe {
                    Expr::from_node_id_unchecked(
                        *self
                            .scratch_cover
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
        self.scratch_cover.reset(checkpoint);
        let expression = expression_result?;
        Some(
            self.ast
                .expr_parenthesized_expression(Span::new(cover.start, cover.end), expression),
        )
    }

    fn parse_parenthesized_expression(&mut self) -> Option<Expr> {
        let cover = self.parse_parenthesized_cover()?;
        self.cover_to_parenthesized_expression(cover)
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
        let arguments = (0..cover.elements.len())
            .map(|index| {
                cover
                    .elements
                    .get_node(&self.ast, index)
                    .expect("parenthesized cover contains its arguments")
            })
            .collect::<Vec<_>>();
        let checkpoint = self.scratch_cover.begin();
        let result = (|| {
            let mut rest = None;
            for (index, argument) in arguments.iter().copied().enumerate() {
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
                        self.scratch_cover.append(parameter);
                    }
                    ArgumentData::SpreadElement(spread) => {
                        if index + 1 != arguments.len() {
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
                            return None;
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
                        let temporary = self.ast.binding_rest_element(
                            spread.span(&self.ast),
                            TypedSubRange::empty(),
                            argument,
                            None,
                            false,
                        );
                        // SAFETY: `temporary` is the unattached last node, and
                        // `spread` is retired in favor of the returned handle.
                        rest = Some(unsafe { self.ast.replace_data(spread, temporary) });
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
        self.scratch_cover.reset(checkpoint);
        result
    }

    fn convert_to_formal_parameter(&mut self, expression: Expr) -> Option<FormalParameterItem> {
        self.expression_to_parameter(expression)
    }

    fn parse_async_function_or_arrow(&mut self, minimum: Precedence) -> Option<Expr> {
        let token = self.current_token;
        let async_identifier = self.parse_identifier()?;

        if self.current_token.has_line_terminator_before() {
            return Some(async_identifier);
        }

        match self.current_token.tag {
            TokenTag::Function => {
                if token.is_escaped() {
                    self.report_escaped_keyword(token.span);
                }
                Some(Expr::Function(
                    self.parse_function(
                        ParseFunctionOpts {
                            is_async: true,
                            is_expression: true,
                            ..ParseFunctionOpts::default()
                        },
                        Some(token.span.start),
                    )?
                    .into_function_expression(),
                ))
            }
            TokenTag::LeftParen => self.parse_async_arrow_function_or_call(
                token.span,
                async_identifier,
                minimum,
                token.is_escaped(),
            ),
            TokenTag::LessThan
                if self.lang.is_typescript() && minimum <= Precedence::Assignment =>
            {
                if let Some(arrow) = self.try_parse_generic_arrow(true, token.span.start)? {
                    if token.is_escaped() {
                        self.report_escaped_keyword(token.span);
                    }
                    return Some(arrow);
                }
                Some(async_identifier)
            }
            tag if tag.is_identifier_like() && minimum <= Precedence::Assignment => {
                let after_identifier = self.peek_ahead()?;
                if after_identifier.tag == TokenTag::Arrow
                    && !after_identifier.has_line_terminator_before()
                {
                    if token.is_escaped() {
                        self.report_escaped_keyword(token.span);
                    }
                    let context = enter_grammar_context![&mut self.context, ?Yield, +Await];
                    let parameter = self.parse_identifier();
                    self.context = context;
                    let parameter = parameter?;
                    return self.identifier_to_arrow_function(parameter, true, token.span.start);
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

        let checkpoint = self.checkpoint();
        let cover = self.parse_parenthesized_cover()?;

        let is_arrow = self.current_token.tag == TokenTag::Arrow
            && !self.current_token.has_line_terminator_before()
            && minimum <= Precedence::Assignment;
        if is_arrow {
            self.rewind(checkpoint);
            if escaped_async {
                self.report_escaped_keyword(async_span);
            }
            return self.parse_arrow(true, async_span.start);
        }
        self.cover_to_call_expression(cover, async_identifier)
    }

    fn identifier_to_arrow_function(
        &mut self,
        identifier: Expr,
        async_function: bool,
        arrow_start: u32,
    ) -> Option<Expr> {
        let parameter = self.expression_to_parameter(identifier)?;
        let parameter_span = parameter.span(&self.ast);
        let parameters = self.ast.add_typed_sub_range([parameter]);
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
        if self.current_token.tag != TokenTag::LeftParen {
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
            match second.tag {
                TokenTag::Spread => ArrowHead::Yes,
                TokenTag::LeftBracket | TokenTag::LeftBrace => ArrowHead::Maybe,
                TokenTag::RightParen => {
                    let Some(third) = peek.next() else {
                        return ArrowHead::No;
                    };
                    if matches!(third.tag, TokenTag::Arrow | TokenTag::Colon) {
                        ArrowHead::Yes
                    } else {
                        ArrowHead::No
                    }
                }
                TokenTag::This => {
                    let Some(third) = peek.next() else {
                        return ArrowHead::No;
                    };
                    if third.tag == TokenTag::Colon {
                        ArrowHead::Yes
                    } else {
                        ArrowHead::No
                    }
                }
                _ if second.tag.is_identifier_like() => {
                    let Some(third) = peek.next() else {
                        return ArrowHead::No;
                    };
                    match third.tag {
                        TokenTag::Colon => ArrowHead::Yes,
                        TokenTag::Question => {
                            let Some(fourth) = peek.next() else {
                                return ArrowHead::No;
                            };
                            if matches!(
                                fourth.tag,
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
        let position = self.current_token.span.start;
        if self.not_parenthesized_arrow.contains(&position) {
            return Some(None);
        }

        let checkpoint = self.checkpoint();
        let arrow = match self.parse_arrow(is_async, start) {
            Some(arrow) => arrow,
            None => {
                self.rewind(checkpoint);
                self.not_parenthesized_arrow.insert(position);
                return Some(None);
            }
        };
        let return_type = arrow
            .as_arrow_function_expression(&self.ast)
            .expect("arrow speculation produces an arrow function")
            .return_type(&self.ast);
        if !self.ts_context.allow_arrow_return_type()
            && return_type.is_some()
            && self.current_token.tag != TokenTag::Colon
        {
            self.rewind(checkpoint);
            self.not_parenthesized_arrow.insert(position);
            return Some(None);
        }
        Some(Some(arrow))
    }

    fn try_parse_generic_arrow(&mut self, is_async: bool, start: u32) -> Option<Option<Expr>> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LessThan);
        let next = self.peek_ahead()?;
        if !next.tag.is_identifier_like() && next.tag != TokenTag::Const {
            return Some(None);
        }
        self.try_parse_arrow(is_async, start)
    }

    fn parse_arrow(&mut self, is_async: bool, start: u32) -> Option<Expr> {
        let context = if is_async {
            enter_grammar_context![&mut self.context, ?Yield, +Await]
        } else {
            enter_grammar_context![&mut self.context, ?Yield, ?Await]
        };
        // Non-async arrow parameters inherit [Await] from the surrounding grammar.
        let module_sensitive_await = !is_async && self.boundary_context.can_be_module();
        let mut arrow_boundary_context = BoundaryContext::IN_FUNCTION;
        arrow_boundary_context.set(BoundaryContext::CAN_BE_MODULE, module_sensitive_await);
        let boundary_context = mem::replace(&mut self.boundary_context, arrow_boundary_context);
        let result = self.parse_arrow_inner(is_async, start);
        self.context = context;
        self.boundary_context = boundary_context;
        result
    }

    fn parse_arrow_inner(&mut self, is_async: bool, start: u32) -> Option<Expr> {
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        if self.current_token.tag != TokenTag::LeftParen {
            return None;
        }
        let params =
            self.parse_formal_parameters(FormalParameterKind::ArrowFormalParameters, false)?;
        let return_type = if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon
        {
            Some(self.parse_return_type_annotation()?)
        } else {
            None
        };
        if self.current_token.tag != TokenTag::Arrow
            || self.current_token.has_line_terminator_before()
        {
            return None;
        }
        self.build_arrow_function(start, params, is_async, type_parameters, return_type)
    }

    fn build_arrow_function(
        &mut self,
        start: u32,
        parameters: FormalParameters,
        async_function: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Option<Expr> {
        self.advance()?;
        let context = enter_function_grammar_context(&mut self.context, false, async_function);
        let boundary_context =
            mem::replace(&mut self.boundary_context, BoundaryContext::IN_FUNCTION);
        let body = self.parse_arrow_body();
        self.context = context;
        self.boundary_context = boundary_context;
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

    fn parse_arrow_body(&mut self) -> Option<ArrowBodyResult> {
        if self.current_token.tag == TokenTag::LeftBrace {
            let body = self.parse_function_body()?;
            return Some(ArrowBodyResult {
                body: ArrowFunctionBody::FunctionBody(body),
                is_expression: false,
            });
        }
        let expression = self.parse_expr(
            Precedence::Assignment,
            ParseExpressionOpts {
                respect_in_parameter: true,
                ..ParseExpressionOpts::default()
            },
        )?;
        Some(ArrowBodyResult {
            body: ArrowFunctionBody::Expr(expression),
            is_expression: true,
        })
    }

    fn expression_to_parameter(&mut self, expression: Expr) -> Option<FormalParameterItem> {
        let pattern = self.expression_to_binding_pattern(expression)?;
        let span = pattern.span(&self.ast);
        let pattern = FormalParameterPattern::BindingPattern(pattern);
        let parameter = self.ast.formal_parameter(span, pattern);
        Some(FormalParameterItem::FormalParameter(parameter))
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
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(identifier) if context == PatternContext::Assignment => {
                Some(BindingPattern::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::IdentifierReference(identifier),
                ))
            }
            ExprData::IdentifierReference(identifier) => {
                let decorators = TypedSubRange::empty();
                let binding = self.ast.binding_identifier(
                    identifier.span(&self.ast),
                    identifier.name(&self.ast),
                    decorators,
                    None,
                    false,
                );
                // SAFETY: `binding` is the unattached last node, and
                // `identifier` is retired in favor of the returned handle.
                let binding = unsafe { self.ast.replace_data(identifier, binding) };
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
                let decorators = TypedSubRange::empty();
                let pattern = self.ast.assignment_pattern(
                    assignment.span(&self.ast),
                    decorators,
                    left,
                    None,
                    right,
                    false,
                );
                // SAFETY: `pattern` is the unattached last node, and
                // `assignment` is retired in favor of the returned handle.
                let pattern = unsafe { self.ast.replace_data(assignment, pattern) };
                Some(BindingPattern::AssignmentPattern(pattern))
            }
            ExprData::ArrayExpression(array) => self.convert_array_pattern(array, context),
            ExprData::ObjectExpression(object) => self.convert_object_pattern(object, context),
            ExprData::ParenthesizedExpression(_) if context == PatternContext::Binding => {
                self.report(
                    expression.span(&self.ast),
                    "Parentheses are not allowed in this binding pattern",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Remove the extra parentheses. Binding patterns can only be identifiers, destructuring patterns, or assignment patterns, not parenthesized expressions.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
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
                self.stripped_paren = Some(pattern.node_id());
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
        self.convert_array_pattern_impl(
            Some(array),
            array.elements(&self.ast),
            array.span(&self.ast),
            context,
        )
    }

    fn array_cover_to_pattern(
        &mut self,
        cover: ArrayCover,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        self.convert_array_pattern_impl(
            None,
            cover.elements,
            Span::new(cover.start, cover.end),
            context,
        )
    }

    fn convert_array_pattern_impl(
        &mut self,
        mutate_node: Option<ArrayExpression>,
        elements_range: TypedSubRange<Option<Argument>>,
        span: Span,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        let checkpoint = self.scratch_cover.begin();
        let result = (|| {
            let source_elements_len = elements_range.len();
            let none = NodeId::from_raw_unchecked(u32::MAX);
            let mut rest = None;
            for index in 0..source_elements_len {
                let argument = elements_range.get_node(&self.ast, index).unwrap();
                let Some(argument) = argument else {
                    self.scratch_cover.items.push(none);
                    continue;
                };
                match self.ast.argument_data(argument) {
                    ArgumentData::Expr(expression) => {
                        let pattern = self.expression_to_pattern(expression, context)?;
                        self.scratch_cover.append(pattern);
                    }
                    ArgumentData::SpreadElement(spread) => {
                        if self.cover_has_trailing_comma == Some(span.start) {
                            self.report(
                                span,
                                "Rest element cannot have a trailing comma in array destructuring.",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the trailing comma after the rest element",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                            self.cover_has_trailing_comma = None;
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
                        let temporary = self.ast.binding_rest_element(
                            spread.span(&self.ast),
                            TypedSubRange::empty(),
                            argument,
                            None,
                            false,
                        );
                        // SAFETY: `temporary` is the unattached last node, and
                        // `spread` is retired in favor of the returned handle.
                        rest = Some(unsafe { self.ast.replace_data(spread, temporary) });
                        break;
                    }
                }
            }
            let elements =
                self.ast
                    .add_typed_sub_range(self.scratch_cover.items[checkpoint..].iter().map(
                        |node| {
                            (*node != none)
                                .then(|| unsafe { BindingPattern::from_node_id_unchecked(*node) })
                        },
                    ));
            let temporary =
                self.ast
                    .array_pattern(span, TypedSubRange::empty(), elements, rest, None, false);
            let pattern = if let Some(array) = mutate_node {
                // SAFETY: `temporary` is the unattached last node, and `array`
                // is retired in favor of the returned handle.
                unsafe { self.ast.replace_data(array, temporary) }
            } else {
                temporary
            };
            Some(BindingPattern::ArrayPattern(pattern))
        })();
        self.scratch_cover.reset(checkpoint);
        result
    }

    fn convert_object_pattern(
        &mut self,
        object: ObjectExpression,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        self.convert_object_pattern_impl(
            Some(object),
            object.properties(&self.ast),
            object.span(&self.ast),
            context,
        )
    }

    fn object_cover_to_pattern(
        &mut self,
        cover: ObjectCover,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        self.convert_object_pattern_impl(
            None,
            cover.properties,
            Span::new(cover.start, cover.end),
            context,
        )
    }

    fn convert_object_pattern_impl(
        &mut self,
        mutate_node: Option<ObjectExpression>,
        properties_range: TypedSubRange<ObjectPropertyKind>,
        span: Span,
        context: PatternContext,
    ) -> Option<BindingPattern> {
        let checkpoint = self.scratch_cover.begin();
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
                        let temporary = self.ast.binding_property(
                            property.span(&self.ast),
                            property.key(&self.ast),
                            value,
                            property.shorthand(&self.ast),
                            property.computed(&self.ast),
                        );
                        // SAFETY: `temporary` is the unattached last node, and
                        // `property` is retired in favor of the returned handle.
                        let property = unsafe { self.ast.replace_data(property, temporary) };
                        self.scratch_cover.append(property);
                    }
                    ObjectPropertyKindData::SpreadElement(spread) => {
                        if self.cover_has_trailing_comma == Some(span.start) {
                            self.report(
                                span,
                                "Rest element cannot have a trailing comma in object destructuring.",
                                ReportOptions {
                                    help: Some(Cow::Borrowed(
                                        "Remove the trailing comma after the rest element",
                                    )),
                                    ..ReportOptions::default()
                                },
                            );
                            self.cover_has_trailing_comma = None;
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
                        let temporary = self.ast.binding_rest_element(
                            spread.span(&self.ast),
                            TypedSubRange::empty(),
                            argument,
                            None,
                            false,
                        );
                        // SAFETY: `temporary` is the unattached last node, and
                        // `spread` is retired in favor of the returned handle.
                        rest = Some(unsafe { self.ast.replace_data(spread, temporary) });
                        break;
                    }
                }
            }
            let properties = self.add_extra_from_scratch_cover(checkpoint);
            let temporary = self.ast.object_pattern(
                span,
                TypedSubRange::empty(),
                properties,
                rest,
                None,
                false,
            );
            let pattern = if let Some(object) = mutate_node {
                // SAFETY: `temporary` is the unattached last node, and `object`
                // is retired in favor of the returned handle.
                unsafe { self.ast.replace_data(object, temporary) }
            } else {
                temporary
            };
            Some(BindingPattern::ObjectPattern(pattern))
        })();
        self.scratch_cover.reset(checkpoint);
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
                if self.report_stripped_paren_in_binding(target.node_id()) {
                    return Some(BindingPattern::SimpleAssignmentTarget(target));
                }
                match self.ast.simple_assignment_target_data(target) {
                    SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                        let decorators = TypedSubRange::empty();
                        let binding = self.ast.binding_identifier(
                            identifier.span(&self.ast),
                            identifier.name(&self.ast),
                            decorators,
                            None,
                            false,
                        );
                        // SAFETY: `binding` is the unattached last node, and
                        // `identifier` is retired in favor of the returned handle.
                        let binding = unsafe { self.ast.replace_data(identifier, binding) };
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

    /// Revisit a pattern that was already converted under assignment-target
    /// rules before cover grammar established that it is a binding pattern.
    fn assignment_pattern_to_binding_pattern(
        &mut self,
        pattern: BindingPattern,
    ) -> Option<BindingPattern> {
        if self.report_stripped_paren_in_binding(pattern.node_id()) {
            return Some(pattern);
        }
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(_) => {}
            BindingPatternData::SimpleAssignmentTarget(target) => {
                return self.assignment_target_to_pattern(
                    AssignmentTarget::SimpleAssignmentTarget(target),
                    PatternContext::Binding,
                );
            }
            BindingPatternData::AssignmentPattern(assignment) => {
                self.assignment_pattern_to_binding_pattern(assignment.left(&self.ast))?;
            }
            BindingPatternData::BindingRestElement(rest) => {
                self.assignment_pattern_to_binding_pattern(rest.argument(&self.ast))?;
            }
            BindingPatternData::ArrayPattern(array) => {
                let elements = array.elements(&self.ast);
                for index in 0..elements.len() {
                    if let Some(element) = elements.get_node(&self.ast, index).unwrap() {
                        self.assignment_pattern_to_binding_pattern(element)?;
                    }
                }
                if let Some(rest) = array.rest(&self.ast) {
                    self.assignment_pattern_to_binding_pattern(rest.argument(&self.ast))?;
                }
            }
            BindingPatternData::ObjectPattern(object) => {
                let properties = object.properties(&self.ast);
                for index in 0..properties.len() {
                    let property = properties.get_node(&self.ast, index).unwrap();
                    self.assignment_pattern_to_binding_pattern(property.value(&self.ast))?;
                }
                if let Some(rest) = object.rest(&self.ast) {
                    self.assignment_pattern_to_binding_pattern(rest.argument(&self.ast))?;
                }
            }
        }
        Some(pattern)
    }

    fn report_stripped_paren_in_binding(&mut self, node: NodeId) -> bool {
        if self.stripped_paren != Some(node) {
            return false;
        }
        self.report(
            self.ast.span(node),
            "Parentheses are not allowed in this binding pattern",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Remove the extra parentheses. Binding patterns can only be identifiers, destructuring patterns, or assignment patterns, not parenthesized expressions.",
                )),
                ..ReportOptions::default()
            },
        );
        true
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

    fn is_part_of_pattern(&self) -> bool {
        matches!(
            self.current_token.tag,
            TokenTag::Assign | TokenTag::In | TokenTag::Of
        )
    }

    fn validate_no_cover_initialized_syntax(&mut self, expression: Expr) -> Option<()> {
        match self.ast.expr_data(expression) {
            ExprData::ObjectExpression(object) => {
                for id in object.properties(&self.ast).iter() {
                    let property = self.ast.get_node_in_sub_range(id);
                    match self.ast.object_property_kind_data(property) {
                        ObjectPropertyKindData::ObjectProperty(property) => {
                            let value = property.value(&self.ast);
                            if property.shorthand(&self.ast)
                                && self.is_cover_initialized_name(value)
                            {
                                self.report_cover_initialized_name_error(property.span(&self.ast));
                            }
                            self.validate_no_cover_initialized_syntax(value)?;
                        }
                        ObjectPropertyKindData::SpreadElement(spread) => {
                            self.validate_no_cover_initialized_syntax(spread.argument(&self.ast))?;
                        }
                    }
                }
            }
            ExprData::ArrayExpression(array) => {
                for index in 0..array.elements(&self.ast).len() {
                    let Some(argument) = array
                        .elements(&self.ast)
                        .get_node(&self.ast, index)
                        .unwrap()
                    else {
                        continue;
                    };
                    let expression = match self.ast.argument_data(argument) {
                        ArgumentData::Expr(expression) => expression,
                        ArgumentData::SpreadElement(spread) => spread.argument(&self.ast),
                    };
                    self.validate_no_cover_initialized_syntax(expression)?;
                }
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.validate_no_cover_initialized_syntax(parenthesized.expression(&self.ast))?;
            }
            ExprData::SequenceExpression(sequence) => {
                for index in 0..sequence.expressions(&self.ast).len() {
                    let expression = sequence
                        .expressions(&self.ast)
                        .get_node(&self.ast, index)
                        .unwrap();
                    self.validate_no_cover_initialized_syntax(expression)?;
                }
            }
            _ => {}
        }
        Some(())
    }

    fn is_cover_initialized_name(&self, expression: Expr) -> bool {
        matches!(
            self.ast.expr_data(expression),
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(&self.ast) == AssignmentOperator::Assign
        )
    }

    fn report_cover_initialized_name_error(&mut self, span: Span) {
        self.report(
            span,
            "Shorthand property cannot have a default value in object expression",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Use '{ a: a = 1 }' syntax or this is only valid in destructuring patterns.",
                )),
                ..ReportOptions::default()
            },
        );
    }

    fn parse_array_expression(&mut self, in_cover: bool) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBracket);
        let cover = self.parse_array_cover()?;
        let needs_validation = !in_cover
            && self.state.contains(ParserState::COVER_HAS_INIT_NAME)
            && !self.is_part_of_pattern();
        if !in_cover {
            self.state.remove(ParserState::COVER_HAS_INIT_NAME);
        }
        self.array_cover_to_expression(cover, needs_validation)
    }

    fn parse_object_expression(&mut self, in_cover: bool) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let cover = self.parse_object_cover()?;
        let needs_validation = !in_cover
            && self.state.contains(ParserState::COVER_HAS_INIT_NAME)
            && !self.is_part_of_pattern();
        if !in_cover {
            self.state.remove(ParserState::COVER_HAS_INIT_NAME);
        }
        self.object_cover_to_expression(cover, needs_validation)
    }

    fn parse_array_cover(&mut self) -> Option<ArrayCover> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBracket);
        let start = self.current_token.span.start;
        self.advance()?;
        let checkpoint = self.scratch_cover.begin();
        let result =
            (|| {
                let mut end = start + 1;
                while self.current_token.tag != TokenTag::RightBracket
                    && self.current_token.tag != TokenTag::Eof
                {
                    if self.eat(TokenTag::Comma)? {
                        self.scratch_cover
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
                    self.scratch_cover.append(argument);
                    if self.eat(TokenTag::Comma)? {
                        if self.current_token.tag == TokenTag::RightBracket {
                            self.cover_has_trailing_comma = Some(start);
                        }
                    } else if self.current_token.tag != TokenTag::RightBracket {
                        self.report_expected(
                            self.current_token.span,
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
                if self.current_token.tag != TokenTag::RightBracket {
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
                end = self.current_token.span.end;
                self.advance()?;
                let none = NodeId::from_raw_unchecked(u32::MAX);
                let elements = self.ast.add_typed_sub_range(
                    self.scratch_cover.items[checkpoint..].iter().map(|node| {
                        (*node != none).then(|| unsafe { Argument::from_node_id_unchecked(*node) })
                    }),
                );
                Some(ArrayCover {
                    elements,
                    start,
                    end,
                })
            })();
        self.scratch_cover.reset(checkpoint);
        result
    }

    fn parse_object_cover(&mut self) -> Option<ObjectCover> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let start = self.current_token.span.start;
        self.advance()?;
        let checkpoint = self.scratch_cover.begin();
        let result = (|| {
            let mut end = start + 1;
            while self.current_token.tag != TokenTag::RightBrace
                && self.current_token.tag != TokenTag::Eof
            {
                let property = if self.eat(TokenTag::Spread)? {
                    let spread_start = self.lexer.state.prev_token_end - 3;
                    let expression = self.parse_expr(
                        Precedence::Assignment,
                        ParseExpressionOpts {
                            in_cover: true,
                            ..ParseExpressionOpts::default()
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
                self.scratch_cover.append(property);
                if self.eat(TokenTag::Comma)? {
                    if self.current_token.tag == TokenTag::RightBrace {
                        self.cover_has_trailing_comma = Some(start);
                    }
                } else if self.current_token.tag != TokenTag::RightBrace {
                    self.report_expected(
                        self.current_token.span,
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
            if self.current_token.tag != TokenTag::RightBrace {
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
            end = self.current_token.span.end;
            self.advance()?;
            let properties = self.add_extra_from_scratch_cover(checkpoint);
            Some(ObjectCover {
                properties,
                start,
                end,
            })
        })();
        self.scratch_cover.reset(checkpoint);
        result
    }

    fn array_cover_to_expression(&mut self, cover: ArrayCover, validate: bool) -> Option<Expr> {
        let expression = self
            .ast
            .expr_array_expression(Span::new(cover.start, cover.end), cover.elements);
        if validate {
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        Some(expression)
    }

    fn object_cover_to_expression(&mut self, cover: ObjectCover, validate: bool) -> Option<Expr> {
        let expression = self
            .ast
            .expr_object_expression(Span::new(cover.start, cover.end), cover.properties);
        if validate {
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        Some(expression)
    }

    fn parse_cover_property(&mut self) -> Option<ObjectPropertyKind> {
        let start = self.current_token.span.start;
        let mut async_function = false;
        let mut generator = false;
        let mut kind = PropertyKind::Init;
        let mut key = None;
        let mut key_identifier_token = None;

        if self.current_token.tag == TokenTag::Async {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            if !self.current_token.has_line_terminator_before()
                && Self::is_property_key_start(self.current_token.tag)
            {
                self.report_if_escaped_keyword(token);
                async_function = true;
            } else {
                let name = self.identifier_name(token);
                key = Some(self.ast.property_key_identifier_name(token.span, name));
            }
        }

        if key.is_none() && self.eat(TokenTag::Star)? {
            generator = true;
        }

        if key.is_none()
            && !async_function
            && !generator
            && matches!(self.current_token.tag, TokenTag::Get | TokenTag::Set)
        {
            let token = self.current_token;
            self.advance_without_escape_check()?;
            if Self::is_property_key_start(self.current_token.tag) {
                self.report_if_escaped_keyword(token);
                kind = if token.tag == TokenTag::Get {
                    PropertyKind::Get
                } else {
                    PropertyKind::Set
                };
            } else {
                let name = self.identifier_name(token);
                key = Some(self.ast.property_key_identifier_name(token.span, name));
            }
        }

        let computed = self.current_token.tag == TokenTag::LeftBracket && key.is_none();
        let key = match key {
            Some(key) => key,
            None => {
                if self.current_token.tag.is_identifier_like() {
                    key_identifier_token = Some(self.current_token);
                }
                self.parse_property_key(true)?
            }
        };
        let key_span = key.span(&self.ast);

        if self.current_token.tag == TokenTag::LeftParen
            || (self.lang.is_typescript() && is_angle_open(self.current_token.tag))
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
                self.current_token.span,
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

        if self.current_token.tag == TokenTag::Colon {
            self.advance()?;
            let value = self.parse_expr(
                Precedence::Assignment,
                ParseExpressionOpts {
                    in_cover: true,
                    ..ParseExpressionOpts::default()
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

        if self.current_token.tag == TokenTag::Assign {
            if computed {
                self.report(
                    key_span,
                    "Computed property cannot have a default value without ':'",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Use '[key]: value = default' syntax instead.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }

            let PropertyKeyData::IdentifierName(identifier) = self.ast.property_key_data(key)
            else {
                self.report(
                    key_span,
                    "Invalid shorthand property initializer",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Only identifier keys can have default values. Use 'key: value = default' syntax.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                return None;
            };

            self.advance()?;
            let default_value = self.parse_expr(
                Precedence::Assignment,
                ParseExpressionOpts {
                    in_cover: true,
                    ..ParseExpressionOpts::default()
                },
            )?;
            let identifier = self
                .ast
                .expr_identifier_reference(identifier.span(&self.ast), identifier.name(&self.ast));
            let target = self.expression_to_assignment_target(identifier)?;
            let assignment = self.ast.expr_assignment_expression(
                Span::new(
                    identifier.span(&self.ast).start,
                    default_value.span(&self.ast).end,
                ),
                target,
                default_value,
                AssignmentOperator::Assign,
            );
            self.state.insert(ParserState::COVER_HAS_INIT_NAME);
            return Some(ObjectPropertyKind::ObjectProperty(
                self.ast.object_property(
                    Span::new(start, default_value.span(&self.ast).end),
                    key,
                    assignment,
                    PropertyKind::Init,
                    false,
                    true,
                    false,
                ),
            ));
        }

        if computed {
            self.report(
                key_span,
                "Computed property must have a value",
                ReportOptions {
                    help: Some(Cow::Borrowed("Add ': value' after the computed key.")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }

        if let Some(key_token) = key_identifier_token {
            self.validate_identifier(key_token, "an identifier");
        }

        let PropertyKeyData::IdentifierName(identifier) = self.ast.property_key_data(key) else {
            self.report(
                key_span,
                "Shorthand property must be an identifier",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "String and numeric keys require explicit ': value' syntax.",
                    )),
                    ..ReportOptions::default()
                },
            );
            return None;
        };
        let value = self
            .ast
            .expr_identifier_reference(identifier.span(&self.ast), identifier.name(&self.ast));
        Some(ObjectPropertyKind::ObjectProperty(
            self.ast.object_property(
                Span::new(start, value.span(&self.ast).end),
                key,
                value,
                PropertyKind::Init,
                false,
                true,
                false,
            ),
        ))
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
                    Span::new(start, self.current_token.span.end),
                    "Getter cannot be a generator",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Remove the '*' from the getter definition.")),
                        ..ReportOptions::default()
                    },
                );
                return None;
            } else if kind == PropertyKind::Set {
                self.report(
                    Span::new(start, self.current_token.span.end),
                    "Setter cannot be a generator",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Remove the '*' from the setter definition.")),
                        ..ReportOptions::default()
                    },
                );
                return None;
            }
        }
        let function_start = self.current_token.span.start;
        let context = enter_function_grammar_context(&mut self.context, generator, async_function);
        let boundary_context =
            mem::replace(&mut self.boundary_context, BoundaryContext::IN_FUNCTION);
        let result = (|| {
            let type_parameters =
                if self.lang.is_typescript() && is_angle_open(self.current_token.tag) {
                    Some(self.parse_type_parameters()?)
                } else {
                    None
                };
            let params =
                self.parse_formal_parameters(FormalParameterKind::UniqueFormalParameters, false)?;
            if !self.check_accessor_arity(kind, params) {
                return None;
            }
            let return_type =
                if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_return_type_annotation()?)
                } else {
                    None
                };
            let body = self.parse_function_body()?;
            Some((params, type_parameters, return_type, body))
        })();
        self.context = context;
        self.boundary_context = boundary_context;
        let (params, type_parameters, return_type, body) = result?;
        let span = Span::new(start, body.span(&self.ast).end);
        let function = self.ast.function(
            Span::new(function_start, body.span(&self.ast).end),
            FunctionType::FunctionExpression,
            None,
            type_parameters,
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

    fn parse_class_element_key(&mut self) -> Option<PropertyKey> {
        let token = self.current_token;
        if token.tag == TokenTag::LeftBracket {
            self.advance()?;
            let expression =
                self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
            if !self.expect(
                TokenTag::RightBracket,
                "Expected ']' after computed property key",
                None,
            ) {
                return None;
            }
            return Some(PropertyKey::Expr(expression));
        }

        let key = if token.tag == TokenTag::PrivateIdentifier {
            PropertyKey::PrivateIdentifier(self.parse_private_identifier()?)
        } else if token.tag == TokenTag::StringLiteral {
            PropertyKey::StringLiteral(self.parse_string_literal()?)
        } else if token.tag.is_numeric_literal() {
            let expression = self.parse_numeric_literal()?;
            match self.ast.expr_data(expression) {
                ExprData::NumericLiteral(literal) => PropertyKey::NumericLiteral(literal),
                ExprData::BigIntLiteral(literal) => PropertyKey::BigIntLiteral(literal),
                _ => unreachable!(),
            }
        } else if token.tag.is_identifier_like() {
            self.advance_without_escape_check()?;
            let name = self.identifier_name(token);
            PropertyKey::IdentifierName(self.ast.identifier_name(token.span, name))
        } else {
            let found = self.describe_token(token);
            self.report(
                token.span,
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

    fn parse_property_key(&mut self, in_cover: bool) -> Option<PropertyKey> {
        let token = self.current_token;
        match token.tag {
            TokenTag::LeftBracket => {
                self.advance()?;
                // Computed keys stay expressions when an object cover becomes a
                // binding pattern, so they must not replace binding evidence.
                let expression = self.parse_expr_preserving_stripped_paren(
                    Precedence::Assignment,
                    ParseExpressionOpts {
                        in_cover,
                        ..ParseExpressionOpts::default()
                    },
                )?;
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
            TokenTag::StringLiteral => {
                Some(PropertyKey::StringLiteral(self.parse_string_literal()?))
            }
            tag if tag.is_numeric_literal() => {
                let expression = self.parse_numeric_literal()?;
                match self.ast.expr_data(expression) {
                    ExprData::NumericLiteral(literal) => Some(PropertyKey::NumericLiteral(literal)),
                    ExprData::BigIntLiteral(literal) => Some(PropertyKey::BigIntLiteral(literal)),
                    _ => unreachable!(),
                }
            }
            tag if tag.is_identifier_like() => {
                Some(PropertyKey::IdentifierName(self.parse_identifier_name()?))
            }
            _ => {
                let found = self.describe_token(token);
                if in_cover {
                    self.report(
                        token.span,
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
                        token.span,
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

    fn parse_computed_member_expression(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBracket);
        let start = object.span(&self.ast).start;
        let open_bracket_span = self.current_token.span;
        self.advance()?;
        let context = enter_grammar_context![&mut self.context, +In];
        let expression = self.parse_expr(Precedence::Lowest, ParseExpressionOpts::default());
        self.context = context;
        let expression = expression?;
        let property = PropertyKey::Expr(expression);
        let end = self.current_token.span.end;
        if self.current_token.tag != TokenTag::RightBracket {
            self.report_expected(
                self.current_token.span,
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

    fn parse_static_member_expression(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Dot);
        self.advance()?;
        self.parse_member_property(object, optional)
    }

    #[inline(always)]
    fn parse_member_property(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        let start = object.span(&self.ast).start;
        let token = self.current_token;
        if !token.tag.is_identifier_like() && token.tag != TokenTag::PrivateIdentifier {
            self.report_expected(
                self.current_token.span,
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
        let property = if token.tag == TokenTag::PrivateIdentifier {
            PropertyKey::PrivateIdentifier(self.parse_private_identifier()?)
        } else {
            PropertyKey::IdentifierName(self.parse_identifier_name()?)
        };
        Some(self.ast.expr_member_expression(
            Span::new(start, token.span.end),
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
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftParen);
        let start = callee.span(&self.ast).start;
        let open_paren_span = self.current_token.span;
        self.advance()?;
        let arguments = self.parse_arguments()?;
        let end = self.current_token.span.end;
        if self.current_token.tag != TokenTag::RightParen {
            self.report_expected(
                self.current_token.span,
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
        let arguments = (0..cover.elements.len())
            .map(|index| {
                cover
                    .elements
                    .get_node(&self.ast, index)
                    .expect("call cover contains its arguments")
            })
            .collect::<Vec<_>>();
        for argument in arguments {
            let expression = match self.ast.argument_data(argument) {
                ArgumentData::Expr(expression) => expression,
                ArgumentData::SpreadElement(spread) => spread.argument(&self.ast),
            };
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        Some(self.ast.expr_call_expression(
            Span::new(callee.span(&self.ast).start, cover.end),
            callee,
            None,
            cover.elements,
            false,
        ))
    }

    fn parse_optional_chain(&mut self, left: Expr) -> Option<Expr> {
        let start = left.span(&self.ast).start;
        self.advance()?;
        let mut expression = self.parse_optional_chain_element(left, true)?;
        loop {
            expression = match self.current_token.tag {
                TokenTag::Dot => self.parse_static_member_expression(expression, false)?,
                TokenTag::LeftBracket => {
                    self.parse_computed_member_expression(expression, false)?
                }
                TokenTag::LeftParen => self.parse_call_expression(expression, false, None)?,
                TokenTag::OptionalChaining => {
                    self.advance()?;
                    self.parse_optional_chain_element(expression, true)?
                }
                TokenTag::LogicalNot
                    if self.lang.is_typescript()
                        && !self.current_token.has_line_terminator_before() =>
                {
                    self.parse_non_null_expression(expression)?
                }
                TokenTag::LessThan | TokenTag::LeftShift if self.lang.is_typescript() => {
                    let Some(next) =
                        self.parse_type_argumented_call_or_instantiation(expression)?
                    else {
                        break;
                    };
                    next
                }
                TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                    self.report(
                        self.current_token.span,
                        "Tagged template expressions are not permitted in an optional chain",
                        ReportOptions {
                            help: Some(Cow::Borrowed(
                                "Remove the optional chaining operator '?.' before the template literal or add parentheses.",
                            )),
                            ..ReportOptions::default()
                        },
                    );
                    break;
                }
                _ => break,
            };
        }
        Some(
            self.ast.expr_chain_expression(
                Span::new(start, expression.span(&self.ast).end),
                expression,
            ),
        )
    }

    fn parse_optional_chain_element(&mut self, object: Expr, optional: bool) -> Option<Expr> {
        let tag = self.current_token.tag;
        if tag.is_identifier_like() || tag == TokenTag::PrivateIdentifier {
            return self.parse_member_property(object, optional);
        }

        if self.lang.is_typescript() && is_angle_open(tag) {
            let checkpoint = self.checkpoint();
            let type_arguments = self.try_parse_type_arguments_in_expression()?;
            if let Some(type_arguments) = type_arguments
                && self.current_token.tag == TokenTag::LeftParen
            {
                return self.parse_call_expression(object, optional, Some(type_arguments));
            }
            self.rewind(checkpoint);
            self.lexer.rewrite_last_collected_token(self.current_token);
        }

        match tag {
            TokenTag::LeftBracket => self.parse_computed_member_expression(object, optional),
            TokenTag::LeftParen => self.parse_call_expression(object, optional, None),
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                self.report(
                    self.current_token.span,
                    "Tagged template expressions are not permitted in an optional chain",
                    ReportOptions {
                        help: Some(Cow::Borrowed(
                            "Remove the optional chaining operator '?.' before the template literal.",
                        )),
                        ..ReportOptions::default()
                    },
                );
                None
            }
            _ => {
                self.report_expected(
                    self.current_token.span,
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

    fn parse_arguments(&mut self) -> Option<TypedSubRange<Argument>> {
        let checkpoint = self.scratch_a.begin();
        let context = enter_grammar_context![&mut self.context, +In];
        let result = (|| {
            while self.current_token.tag != TokenTag::RightParen
                && self.current_token.tag != TokenTag::Eof
            {
                let argument = if self.eat(TokenTag::Spread)? {
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
        })();
        self.context = context;
        self.scratch_a.reset(checkpoint);
        result
    }

    fn parse_new_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::New);
        let meta_token = self.current_token;
        let start = meta_token.span.start;
        self.report_if_escaped_keyword(meta_token);
        let meta_name = self.identifier_name(meta_token);
        self.advance_without_escape_check()?;
        let meta = self.ast.identifier_name(meta_token.span, meta_name);
        if self.current_token.tag == TokenTag::Dot {
            return self.parse_new_target(meta);
        }
        let callee_starts_with_import = self.current_token.tag == TokenTag::Import;
        let mut callee = match self.current_token.tag {
            TokenTag::LeftParen => self.parse_parenthesized_expression()?,
            TokenTag::New => self.parse_new_expression()?,
            _ => self.parse_primary_expression(false)?,
        };
        loop {
            callee = match self.current_token.tag {
                TokenTag::Dot => self.parse_static_member_expression(callee, false)?,
                TokenTag::LeftBracket => self.parse_computed_member_expression(callee, false)?,
                TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                    self.parse_tagged_template_expression(callee, None)?
                }
                TokenTag::OptionalChaining => {
                    return {
                        let span = self.current_token.span;
                        self.report(
                            span,
                            "Optional chaining is not allowed in new expression",
                            ReportOptions::default(),
                        );
                        None
                    };
                }
                _ => break,
            };
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
        let type_arguments = if self.lang.is_typescript() {
            self.try_parse_type_arguments_in_expression()?
        } else {
            None
        };
        let arguments = if self.current_token.tag == TokenTag::LeftParen {
            let open_paren_span = self.current_token.span;
            self.advance()?;
            let arguments = self.parse_arguments()?;
            if self.current_token.tag != TokenTag::RightParen {
                self.report_expected(
                    self.current_token.span,
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
            TypedSubRange::empty()
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
        debug_assert_eq!(self.current_token.tag, TokenTag::Import);
        self.report_if_escaped_keyword(self.current_token);
        let import_name = self.parse_identifier_name()?;
        let start = import_name.span(&self.ast).start;
        match self.current_token.tag {
            TokenTag::Dot => self.parse_import_meta_or_phase_import(import_name),
            TokenTag::LeftParen => self.parse_dynamic_import(start, None, false),
            _ => {
                self.report(
                    self.current_token.span,
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
        let (phase, sync) = match property.tag {
            TokenTag::Source => (Some(ImportPhase::Source), false),
            TokenTag::Defer => (Some(ImportPhase::Defer), false),
            TokenTag::Identifier if property.text(self.source) == "sync" => (None, true),
            TokenTag::Identifier if property.text(self.source) == "meta" => {
                let property = self.parse_identifier_name()?;
                self.mark_module_syntax();
                return Some(self.ast.expr_meta_property(
                    Span::new(
                        import_name.span(&self.ast).start,
                        property.span(&self.ast).end,
                    ),
                    import_name,
                    property,
                ));
            }
            _ => {
                return {
                    let span = self.current_token.span;
                    self.report(
                        span,
                        "The only valid meta properties for 'import' are 'import.meta', 'import.source()', or 'import.defer()'",
                        ReportOptions::default(),
                    );
                    None
                };
            }
        };
        self.advance()?;
        self.parse_dynamic_import(import_name.span(&self.ast).start, phase, sync)
    }

    fn parse_dynamic_import(
        &mut self,
        start: u32,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> Option<Expr> {
        if !self.expect(TokenTag::LeftParen, "Expected '(' after import", None) {
            return None;
        }
        let source = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?;
        let options = if phase.is_none() && self.eat(TokenTag::Comma)? {
            let options = if self.current_token.tag == TokenTag::RightParen {
                None
            } else {
                Some(self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default())?)
            };
            self.eat(TokenTag::Comma)?;
            options
        } else {
            None
        };
        let end = self.current_token.span.end;
        if !self.expect(
            TokenTag::RightParen,
            "Expected ')' after import()",
            Some("Dynamic import call must end with ')'"),
        ) {
            return None;
        }
        Some(
            self.ast
                .expr_import_expression(Span::new(start, end), source, options, phase, sync),
        )
    }

    fn parse_new_target(&mut self, meta: IdentifierName) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Dot);
        self.advance()?;
        let property_token = self.current_token;
        if property_token.text(self.source) != "target" {
            return {
                let span = self.current_token.span;
                self.report(
                    span,
                    "The only valid meta property for 'new' is 'new.target'",
                    ReportOptions::default(),
                );
                None
            };
        }
        let property = self.parse_identifier_name()?;
        Some(self.ast.expr_meta_property(
            Span::new(meta.span(&self.ast).start, property.span(&self.ast).end),
            meta,
            property,
        ))
    }

    fn parse_jsx_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LessThan);
        self.parse_jsx_element(JsxElementContext::TopLevel)
    }

    fn enter_jsx_tag(&mut self) {
        self.set_lexer_mode(LexerMode::JsxTag);
    }

    fn exit_jsx_tag(&mut self) {
        self.set_lexer_mode(LexerMode::Normal);
    }

    fn parse_jsx_element(&mut self, context: JsxElementContext) -> Option<Expr> {
        let start = self.current_token.span.start;
        if self.peek_ahead()?.tag == TokenTag::GreaterThan {
            return self.parse_jsx_fragment(start, context);
        }

        let opening = self.parse_jsx_opening_element(context)?;
        if opening.self_closing(&self.ast) {
            let children = TypedSubRange::empty();
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
        debug_assert_eq!(self.current_token.tag, TokenTag::LessThan);
        let start = self.current_token.span.start;
        self.enter_jsx_tag();
        self.advance()?;
        let name = self.parse_jsx_element_name()?;

        let type_arguments = if self.lang.is_typescript() && is_angle_open(self.current_token.tag) {
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
        if self.current_token.tag != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span,
                "Expected '>' to close JSX opening element",
                ReportOptions {
                    help: Some(Cow::Borrowed("Add '>' to close the JSX tag")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let end = self.current_token.span.end;
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
        if self.current_token.tag != TokenTag::LessThan {
            self.report_expected(
                self.current_token.span,
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
        let closing_start = self.current_token.span.start;
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
        if self.current_token.tag != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span,
                "Expected '>' to close JSX closing element",
                ReportOptions {
                    help: Some(Cow::Borrowed("Add '>' to complete the closing tag")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        let end = self.current_token.span.end;
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
            return None;
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
        if self.current_token.tag != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span,
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
        let opening_end = self.current_token.span.end;
        let opening = self.ast.jsx_opening_fragment(Span::new(start, opening_end));
        let children = self.parse_jsx_children()?;
        let closing_start = self.current_token.span.start;
        self.enter_jsx_tag();
        self.advance()?;
        if !self.expect(
            TokenTag::Slash,
            "Expected '/' in JSX closing fragment",
            Some("Add '/' to close the fragment"),
        ) {
            return None;
        }
        if self.current_token.tag != TokenTag::GreaterThan {
            self.report_expected(
                self.current_token.span,
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
        let closing_end = self.current_token.span.end;
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

    fn parse_jsx_children(&mut self) -> Option<TypedSubRange<JsxChild>> {
        let checkpoint = self.scratch_b.begin();
        self.exit_jsx_tag();
        let result = (|| {
            loop {
                let token = self.lexer.scan_jsx_text();
                self.current_token = token;
                match token.tag {
                    TokenTag::JsxText => {
                        let value = self.ast.source_slice(token.span.start, token.span.end);
                        let jsx_text = self.ast.jsx_child_jsx_text(token.span, value);
                        self.scratch_b.append(jsx_text);
                    }
                    TokenTag::LessThan => {
                        if self.peek_ahead()?.tag == TokenTag::Slash {
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

    fn parse_jsx_child_from_left_brace(&mut self) -> Option<JsxChild> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let start = self.current_token.span.start;
        self.advance()?;
        if self.current_token.tag == TokenTag::RightBrace {
            let empty_span = Span::new(start + 1, self.current_token.span.start);
            let empty = self.ast.jsx_empty_expression(empty_span);
            let expression = JsxExpression::JsxEmptyExpression(empty);
            let end = self.current_token.span.end;
            return Some(
                self.ast
                    .jsx_child_jsx_expression_container(Span::new(start, end), expression),
            );
        }
        if self.eat(TokenTag::Spread)? {
            let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
            let end = self.expect_jsx_child_right_brace("JSX spread")?;
            return Some(
                self.ast
                    .jsx_child_jsx_spread_child(Span::new(start, end), expression),
            );
        }
        let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        let end = self.expect_jsx_child_right_brace("JSX expression")?;
        Some(self.ast.jsx_child_jsx_expression_container(
            Span::new(start, end),
            JsxExpression::Expr(expression),
        ))
    }

    fn expect_jsx_child_right_brace(&mut self, what: &'static str) -> Option<u32> {
        if self.current_token.tag != TokenTag::RightBrace {
            let message = match what {
                "JSX spread" => "Expected '}' to close JSX spread",
                _ => "Expected '}' to close JSX expression",
            };
            self.report_expected(
                self.current_token.span,
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed("Add '}' to close the expression")),
                    ..ReportOptions::default()
                },
            );
            return None;
        }
        Some(self.current_token.span.end)
    }

    fn parse_jsx_element_name(&mut self) -> Option<JsxElementName> {
        let token = self.current_token;
        if token.tag != TokenTag::JsxIdentifier {
            self.report_expected(
                self.current_token.span,
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
        let identifier = self.ast.jsx_identifier(token.span, name);
        let start = token.span.start;
        let mut element_name = JsxElementName::JsxIdentifier(identifier);
        let mut member_object = JsxMemberExpressionObject::JsxIdentifier(identifier);
        let mut is_member = false;

        while self.eat(TokenTag::Dot)? {
            if self.current_token.tag != TokenTag::JsxIdentifier {
                self.report_expected(
                    self.current_token.span,
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
            let property = self.ast.jsx_identifier(property_token.span, property_name);
            let member = self.ast.jsx_member_expression(
                Span::new(start, property_token.span.end),
                member_object,
                property,
            );
            member_object = JsxMemberExpressionObject::JsxMemberExpression(member);
            element_name = JsxElementName::JsxMemberExpression(member);
        }

        if !is_member && self.eat(TokenTag::Colon)? {
            if self.current_token.tag != TokenTag::JsxIdentifier {
                self.report_expected(
                    self.current_token.span,
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
            let local = self.ast.jsx_identifier(local_token.span, local_name);
            element_name = JsxElementName::JsxNamespacedName(self.ast.jsx_namespaced_name(
                Span::new(start, local_token.span.end),
                identifier,
                local,
            ));
        }
        Some(element_name)
    }

    fn parse_jsx_attributes(&mut self) -> Option<TypedSubRange<JsxAttributeItem>> {
        let checkpoint = self.scratch_a.begin();
        let result = (|| {
            while matches!(
                self.current_token.tag,
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
        if self.current_token.tag == TokenTag::LeftBrace {
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
        let namespace = self.ast.jsx_identifier(token.span, name);
        if !self.eat(TokenTag::Colon)? {
            return Some(JsxAttributeName::JsxIdentifier(namespace));
        }
        if self.current_token.tag != TokenTag::JsxIdentifier {
            self.report_expected(
                self.current_token.span,
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
        let local = self.ast.jsx_identifier(local_token.span, local_name);
        Some(JsxAttributeName::JsxNamespacedName(
            self.ast.jsx_namespaced_name(
                Span::new(token.span.start, local_token.span.end),
                namespace,
                local,
            ),
        ))
    }

    fn parse_jsx_attribute_value(&mut self) -> Option<JsxAttributeValue> {
        match self.current_token.tag {
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
                    return None;
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
                    self.current_token.span,
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

    fn parse_jsx_expression_container(
        &mut self,
        context: JsxExprContext,
    ) -> Option<JsxExpressionContainer> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let start = self.current_token.span.start;
        if context == JsxExprContext::Tag {
            self.exit_jsx_tag();
        }
        self.advance()?;
        if self.current_token.tag == TokenTag::RightBrace {
            let end = self.current_token.span.end;
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
        let expression = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        let end = self.current_token.span.end;
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

    fn parse_jsx_spread_attribute(&mut self) -> Option<JsxAttributeItem> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LeftBrace);
        let start = self.current_token.span.start;
        self.exit_jsx_tag();
        self.advance()?;
        if !self.expect(
            TokenTag::Spread,
            "Expected '...' after '{' in JSX spread",
            Some("Add '...' to spread the expression"),
        ) {
            return None;
        }
        let argument = self.parse_expr(Precedence::Comma, ParseExpressionOpts::default())?;
        let end = self.current_token.span.end;
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
        if self.current_token.tag == TokenTag::Exponent {
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

    fn is_ambiguous_await_prefix(&self) -> bool {
        if self.current_token.has_line_terminator_before() {
            return true;
        }
        match self.current_token.tag {
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

    fn parse_type_argumented_call_or_instantiation(
        &mut self,
        callee: Expr,
    ) -> Option<Option<Expr>> {
        let Some(type_arguments) = self.try_parse_type_arguments_in_expression()? else {
            return Some(None);
        };
        let expression = match self.current_token.tag {
            TokenTag::LeftParen => {
                self.parse_call_expression(callee, false, Some(type_arguments))?
            }
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                self.parse_tagged_template_expression(callee, Some(type_arguments))?
            }
            _ => self.ast.expr_ts_instantiation_expression(
                Span::new(
                    callee.span(&self.ast).start,
                    type_arguments.span(&self.ast).end,
                ),
                callee,
                type_arguments,
            ),
        };
        Some(Some(expression))
    }

    fn try_parse_type_arguments_in_expression(
        &mut self,
    ) -> Option<Option<TsTypeParameterInstantiation>> {
        if !is_angle_open(self.current_token.tag) {
            return Some(None);
        }
        let after = self.peek_ahead()?;
        if !is_start_of_type(after.tag) && after.tag != TokenTag::GreaterThan {
            return Some(None);
        }

        let checkpoint = self.checkpoint();
        let type_arguments = match self.parse_type_arguments_in_expression() {
            Some(type_arguments) if can_follow_type_arguments_in_expression(self.current_token) => {
                type_arguments
            }
            Some(_) | None => {
                self.rewind(checkpoint);
                self.lexer.rewrite_last_collected_token(self.current_token);
                return Some(None);
            }
        };
        Some(Some(type_arguments))
    }

    fn parse_type_assertion(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::LessThan);
        let start = self.current_token.span.start;
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
        debug_assert_eq!(self.current_token.tag, TokenTag::LogicalNot);
        let end = self.current_token.span.end;
        self.advance()?;
        Some(
            self.ast
                .expr_ts_non_null_expression(Span::new(left.span(&self.ast).start, end), left),
        )
    }

    fn parse_yield_expression(&mut self) -> Option<Expr> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Yield);
        let start = self.current_token.span.start;
        self.advance()?;
        let delegate = if self.current_token.tag == TokenTag::Star
            && !self.current_token.has_line_terminator_before()
        {
            self.advance()?;
            true
        } else {
            false
        };
        let argument = if (!delegate && self.current_token.has_line_terminator_before())
            || matches!(
                self.current_token.tag,
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
            Some(self.parse_expr(
                Precedence::Assignment,
                ParseExpressionOpts {
                    respect_in_parameter: true,
                    ..ParseExpressionOpts::default()
                },
            )?)
        };
        if delegate && argument.is_none() {
            self.report_expected(
                self.current_token.span,
                "Expected expression after 'yield*'",
                ReportOptions::default(),
            );
            return None;
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
        debug_assert!(self.current_token.tag.is_unary_operator());
        let token = self.current_token;
        self.advance()?;
        let argument = self.parse_expr(Precedence::Prefix, ParseExpressionOpts::default())?;
        if self.current_token.tag == TokenTag::Exponent {
            self.report(
                Span::new(token.span.start, argument.span(&self.ast).end),
                "The left-hand side of '**' cannot be an unparenthesized unary expression",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add parentheses to make precedence explicit, for example `(-x) ** y` or `-(x ** y)`.",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        let operator = match token.tag {
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
            Span::new(token.span.start, argument.span(&self.ast).end),
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
            token.span.start
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
        let operator = if token.tag == TokenTag::Increment {
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
        if self.ast.node_kind(right.node_id()) == NodeKind::PrivateIdentifier {
            self.report(
                right.span(&self.ast),
                "Private names are only allowed in property accesses (`obj.#field`) or in `in` expressions (`#field in obj`)",
                ReportOptions::default(),
            );
        }
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
        let operator_to_check = match self.ast.expr_data(left) {
            ExprData::LogicalExpression(expression) => Some(expression.operator(&self.ast)),
            _ => match self.ast.expr_data(right) {
                ExprData::LogicalExpression(expression) => Some(expression.operator(&self.ast)),
                _ => None,
            },
        };
        if let Some(operator_to_check) = operator_to_check {
            let mixes_coalesce = (operator == LogicalOperator::NullishCoalescing)
                != (operator_to_check == LogicalOperator::NullishCoalescing);
            if mixes_coalesce {
                self.report(
                    Span::new(start, right.span(&self.ast).end),
                    "Logical expressions and nullish coalescing cannot be mixed",
                    ReportOptions::default(),
                );
                return None;
            }
        }
        Some(self.ast.expr_logical_expression(
            Span::new(start, right.span(&self.ast).end),
            left,
            right,
            operator,
        ))
    }

    fn parse_assignment_expression(
        &mut self,
        precedence: Precedence,
        left: Expr,
        tag: TokenTag,
        respect_in_parameter: bool,
    ) -> Option<Expr> {
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
        // A stripped parenthesis in the right-hand side is in expression
        // position and must not overwrite evidence from the arrow-head target.
        let right = self.parse_expr_preserving_stripped_paren(
            precedence,
            ParseExpressionOpts {
                respect_in_parameter,
                ..ParseExpressionOpts::default()
            },
        )?;
        Some(self.ast.expr_assignment_expression(
            Span::new(start, right.span(&self.ast).end),
            target,
            right,
            assignment_operator(tag),
        ))
    }

    fn parse_conditional_expression(&mut self, test: Expr) -> Option<Expr> {
        let start = test.span(&self.ast).start;
        debug_assert_eq!(self.current_token.tag, TokenTag::Question);
        self.advance()?;
        let context = enter_grammar_context![&mut self.context, +In];
        let old_allow_arrow_return_type = self.ts_context.allow_arrow_return_type();
        self.ts_context.remove(TsContext::ALLOW_ARROW_RETURN_TYPE);
        let consequent = self.parse_expr(Precedence::Assignment, ParseExpressionOpts::default());
        self.context = context;
        self.ts_context.set(
            TsContext::ALLOW_ARROW_RETURN_TYPE,
            old_allow_arrow_return_type,
        );
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
        let alternate = self.parse_expr(
            Precedence::Assignment,
            ParseExpressionOpts {
                respect_in_parameter: true,
                ..ParseExpressionOpts::default()
            },
        )?;
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

    fn unwrap_parens(&self, expression: Expr) -> Expr {
        match self.ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.unwrap_parens(parenthesized.expression(&self.ast))
            }
            _ => expression,
        }
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
                self.stripped_paren = Some(target.node_id());
                Some(target)
            }
            _ => Some(AssignmentTarget::SimpleAssignmentTarget(
                self.expression_to_simple_assignment_target(expression)?,
            )),
        }
    }

    #[inline(always)]
    fn parse_binding_identifier(&mut self) -> Option<BindingIdentifier> {
        let token = self.current_token;
        self.validate_identifier(token, "a binding identifier");
        self.advance_without_escape_check()?;
        let name = self.identifier_name(token);
        let decorators = TypedSubRange::empty();
        Some(
            self.ast
                .binding_identifier(token.span, name, decorators, None, false),
        )
    }

    fn parse_label_identifier(&mut self) -> Option<LabelIdentifier> {
        let token = self.current_token;
        self.validate_identifier(token, "a label");
        self.advance()?;
        let name = self.identifier_name(token);
        Some(self.ast.label_identifier(token.span, name))
    }

    #[inline]
    fn validate_identifier(&mut self, token: Token, as_what: &str) {
        if token.tag == TokenTag::Identifier {
            return;
        }

        self.validate_identifier_slow(token, as_what);
    }

    #[cold]
    #[inline(never)]
    fn validate_identifier_slow(&mut self, token: Token, as_what: &str) {
        if !token.tag.is_identifier_like() {
            self.report_expected(
                token.span,
                "Expected an identifier",
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Identifiers must start with a letter, underscore (_), or dollar sign ($)",
                    )),
                    ..ReportOptions::default()
                },
            );
        }
        if token.tag.is_unconditionally_reserved() {
            let found = self.describe_token(token);
            self.report(
                token.span,
                Cow::Owned(format!(
                    "'{found}' is reserved and cannot be used as {as_what}"
                )),
                ReportOptions::default(),
            );
        }
        if token.tag == TokenTag::Yield && self.context.contains(Context::YIELD) {
            self.report(
                token.span,
                Cow::Owned(format!(
                    "'yield' is reserved in a generator context and cannot be used as {as_what}"
                )),
                ReportOptions::default(),
            );
        }
        if token.tag == TokenTag::Await
            && !self.ts_context.ambient()
            && (self.context.contains(Context::AWAIT) || self.source_type == SourceType::Module)
        {
            self.report(
                token.span,
                Cow::Owned(format!(
                    "'await' is reserved in an async/module context and cannot be used as {as_what}"
                )),
                ReportOptions::default(),
            );
        }
    }

    fn next_token(&mut self) -> Option<()> {
        match self.lexer.next_token(&mut self.current_token) {
            Ok(()) => Some(()),
            Err(error) => self.next_token_error(error),
        }
    }

    #[cold]
    #[inline(never)]
    fn next_token_error(&mut self, error: LexicalError) -> Option<()> {
        let current_token_end = self.current_token.span.end;
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
    #[inline(always)]
    fn advance(&mut self) -> Option<()> {
        self.check_escaped_keyword();
        self.lexer.state.prev_token_end = self.current_token.span.end;
        self.next_token()
    }

    #[inline]
    fn eat(&mut self, tag: TokenTag) -> Option<bool> {
        let matched = self.current_token.tag == tag;
        if matched {
            self.advance()?;
        }
        Some(matched)
    }

    /// Advances without checking whether the current token is an escaped
    /// keyword.
    fn advance_without_escape_check(&mut self) -> Option<()> {
        self.lexer.state.prev_token_end = self.current_token.span.end;
        self.next_token()
    }

    fn check_escaped_keyword(&mut self) {
        let current_token = self.current_token;
        if !current_token.is_escaped() {
            return;
        }
        if current_token.tag.is_keyword() {
            self.report_escaped_keyword(current_token.span);
        }
    }

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
            self.report_escaped_keyword(token.span);
        }
    }

    /// Sets the current token to a rescanned token and advances.
    ///
    /// Use this when retokenizing the current source range, such as for
    /// regular expressions and template continuations.
    #[inline]
    fn advance_with_rescanned_token(&mut self, mut token: Token) -> Option<()> {
        if self.lexer.config.tokens() {
            token.flags |= self.current_token.flags & flag_mask(TokenFlag::LineTerminatorBefore);
        }
        self.lexer.handle_token(token, HandleTokenMode::Replace);
        self.current_token = token;
        self.advance()
    }

    /// Retokenizes the current token in the current lexer mode.
    fn re_scan_current(&mut self) -> Option<()> {
        let line_terminator_before =
            self.current_token.flags & flag_mask(TokenFlag::LineTerminatorBefore);
        self.lexer.rewind_to(self.current_token.span.start);
        if self.lexer.config.tokens() {
            self.lexer.tokens.pop();
        }
        self.next_token()?;
        if self.lexer.config.tokens() {
            self.current_token.flags |= line_terminator_before;
            self.lexer.rewrite_last_collected_token(self.current_token);
        }
        Some(())
    }

    fn expect(&mut self, tag: TokenTag, message: &'static str, help: Option<&'static str>) -> bool {
        if self.current_token.tag == tag {
            return self.advance().is_some();
        }

        self.report_expected(
            self.current_token.span,
            message,
            ReportOptions {
                help: help.map(Cow::Borrowed),
                ..ReportOptions::default()
            },
        );
        false
    }

    /// Implements the ECMAScript
    /// [automatic semicolon insertion rules](https://tc39.es/ecma262/#sec-rules-of-automatic-semicolon-insertion).
    fn can_insert_implicit_semicolon(&self, token: Token) -> bool {
        token.tag == TokenTag::Eof
            || token.has_line_terminator_before()
            || token.tag == TokenTag::RightBrace
    }

    fn eat_semicolon(&mut self, end: u32) -> Option<u32> {
        if self.current_token.tag == TokenTag::Semicolon {
            let semicolon_end = self.current_token.span.end;
            self.advance()?;
            return Some(semicolon_end);
        }
        if !self.can_insert_implicit_semicolon(self.current_token) {
            self.report_expected(
                self.current_token.span,
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

    fn recover(&mut self, terminator: Option<TokenTag>) {
        while self.current_token.tag != TokenTag::Eof {
            self.current_token = self.recover_next_token();
            if self.current_token.tag == TokenTag::Eof {
                break;
            }
            if terminator.is_some_and(|tag| self.current_token.tag == tag) {
                break;
            }
            if self.current_token.has_line_terminator_before()
                && self.current_token.tag.is_keyword()
            {
                break;
            }
        }
    }

    /// Advances during error recovery, skipping bytes that cause lexical
    /// errors.
    fn recover_next_token(&mut self) -> Token {
        loop {
            let mut token = Token::eof(self.lexer.cursor);
            match self.lexer.next_token(&mut token) {
                Ok(()) => return token,
                Err(_) if (self.lexer.cursor as usize) < self.source.len() => {
                    self.lexer.cursor += 1;
                }
                Err(_) => return Token::eof(self.source.len() as u32),
            }
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
        ExprData::ParenthesizedExpression(parenthesized) => {
            ast.node_kind(parenthesized.expression(ast).node_id()) == NodeKind::ChainExpression
        }
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
            _ => return false,
        };
    }
}

fn max_left_precedence(expression: Expr, ast: &Ast<'_>) -> Precedence {
    match ast.node_kind(expression.node_id()) {
        NodeKind::ArrowFunctionExpression | NodeKind::YieldExpression => Precedence::Comma,
        NodeKind::UpdateExpression
        | NodeKind::UnaryExpression
        | NodeKind::AwaitExpression
        | NodeKind::BinaryExpression
        | NodeKind::LogicalExpression
        | NodeKind::ConditionalExpression
        | NodeKind::AssignmentExpression
        | NodeKind::SequenceExpression
        | NodeKind::TsAsExpression
        | NodeKind::TsSatisfiesExpression => Precedence::Prefix,
        _ => Precedence::Primary,
    }
}

fn is_angle_open(tag: TokenTag) -> bool {
    matches!(tag, TokenTag::LessThan | TokenTag::LeftShift)
}

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

fn continues_type(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::Dot
            | TokenTag::LessThan
            | TokenTag::LeftShift
            | TokenTag::LeftBracket
            | TokenTag::BitwiseOr
            | TokenTag::BitwiseAnd
            | TokenTag::Extends
            | TokenTag::Question
            | TokenTag::LogicalNot
    )
}

fn is_start_of_type(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::Any
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
            | TokenTag::Plus
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
    ) || tag.is_identifier_like()
}

fn tuple_element_as_type(element: TsTupleElement, ast: &Ast<'_>) -> TsType {
    match ast.ts_tuple_element_data(element) {
        TsTupleElementData::TsType(ty) => ty,
        TsTupleElementData::TsNamedTupleMember(_) => {
            unreachable!("a named tuple member is handled before conversion")
        }
    }
}

fn can_follow_accessor_keyword(tag: TokenTag) -> bool {
    tag == TokenTag::LeftBracket
        || tag.is_identifier_like()
        || tag == TokenTag::StringLiteral
        || tag.is_numeric_literal()
}

fn can_follow_readonly_modifier(tag: TokenTag) -> bool {
    can_follow_accessor_keyword(tag)
        || matches!(tag, TokenTag::LeftBrace | TokenTag::Star | TokenTag::Spread)
}

fn can_start_binding(tag: TokenTag) -> bool {
    tag.is_identifier_like() || matches!(tag, TokenTag::LeftBracket | TokenTag::LeftBrace)
}

fn can_start_binding_identifier(tag: TokenTag) -> bool {
    tag.is_identifier_like() && !tag.is_unconditionally_reserved()
}

fn can_start_let_binding(tag: TokenTag) -> bool {
    matches!(tag, TokenTag::LeftBracket | TokenTag::LeftBrace) || can_start_binding_identifier(tag)
}

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

fn requires_same_line(tag: TokenTag) -> bool {
    !matches!(tag, TokenTag::Static | TokenTag::Get | TokenTag::Set)
}

fn is_const_enum_head(after_const: Token) -> bool {
    after_const.tag == TokenTag::Enum && !after_const.has_line_terminator_before()
}

fn is_type_keyword(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::Any
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
            | TokenTag::Void
    )
}

fn can_follow_parameter_property_modifier(tag: TokenTag) -> bool {
    tag.is_identifier_like()
        || matches!(
            tag,
            TokenTag::LeftBracket | TokenTag::LeftBrace | TokenTag::Spread
        )
}

fn is_declaration_name(token: Token) -> bool {
    token.tag.is_identifier_like()
        && !token.tag.is_unconditionally_reserved()
        && !token.has_line_terminator_before()
}

fn import_or_export_kind_for_declare(declare: bool) -> ImportOrExportKind {
    if declare {
        ImportOrExportKind::Type
    } else {
        ImportOrExportKind::Value
    }
}

fn can_follow_type_arguments_in_expression(token: Token) -> bool {
    match token.tag {
        TokenTag::LeftParen | TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => true,
        TokenTag::LessThan | TokenTag::GreaterThan | TokenTag::Plus | TokenTag::Minus => false,
        _ => {
            token.has_line_terminator_before()
                || is_binary_operator_like(token.tag)
                || !is_start_of_expression(token.tag)
        }
    }
}

fn is_binary_operator_like(tag: TokenTag) -> bool {
    tag.is_binary_operator()
        || tag.is_logical_operator()
        || tag.is_assignment_operator()
        || matches!(
            tag,
            TokenTag::Comma | TokenTag::Question | TokenTag::As | TokenTag::Satisfies
        )
}

fn is_start_of_expression(tag: TokenTag) -> bool {
    matches!(
        tag,
        TokenTag::Identifier
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
    ) || tag.is_identifier_like()
        || tag.is_numeric_literal()
}

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

    #[test]
    fn parse_expression_fragment() {
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
    fn unambiguous() {
        fn assert_unambiguous_source_type_with_lang(
            source: &str,
            lang: Lang,
            expected: SourceType,
        ) {
            fn serialize_ast(ast: &Ast<'_>) -> Vec<u8> {
                use swc_next_ecma_ast::ffi::ast::{buffer_size, serialize_into};
                let mut buffer = vec![0; buffer_size(ast)];
                serialize_into(ast, &mut buffer);
                buffer
            }

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

        let allocator = Allocator::new();
        let result = parse(&allocator, "value + await \"unterminated");
        assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);

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

        fn parse_stmt<'a>(
            allocator: &'a Allocator,
            source: &'a str,
            options: Options,
            context: FragmentContext,
        ) -> Result<Ast<'a>, Vec<Diagnostic<'a>>> {
            let parser = Parser::init(allocator, source, options, NoTokenParserConfig);
            parser.parse_statement_fragment(context)
        }

        use FragmentContext::{AsyncFunction, Function, Generator, TopLevel};

        let allocator = Allocator::new();
        let script = Options {
            source_type: SourceType::Script,
            ..Options::default()
        };
        let auto = Options {
            source_type: SourceType::Unambiguous,
            ..Options::default()
        };

        let ast = parse_stmt(&allocator, "return value;", script, Function).unwrap();
        let root_statement = ast.stmt_data(ast.root_statement());
        assert!(matches!(root_statement, StmtData::ReturnStatement(_)));

        let result = parse_stmt(&allocator, "return value;", script, TopLevel);
        assert!(result.is_err_and(|result| !result.is_empty()));

        let ast = parse_expr(&allocator, "await value", script, AsyncFunction).unwrap();
        let root_expression = ast.expr_data(ast.root_expression());
        assert!(matches!(root_expression, ExprData::AwaitExpression(_)));

        let ast = parse_expr(&allocator, "yield value", script, Generator).unwrap();
        let root_expression = ast.expr_data(ast.root_expression());
        assert!(matches!(root_expression, ExprData::YieldExpression(_)));

        let ast = parse_stmt(&allocator, "import value from 'value';", auto, TopLevel).unwrap();
        let root_statement = ast.stmt_data(ast.root_statement());
        assert!(matches!(root_statement, StmtData::ImportDeclaration(_)));
        assert_eq!(ast.source_type(), SourceType::Module);

        let ast = parse_stmt(&allocator, "await value;", auto, AsyncFunction).unwrap();
        assert_eq!(ast.source_type(), SourceType::Script);

        let result = parse_stmt(&allocator, "value; other;", script, TopLevel);
        assert!(result.is_err_and(|result| !result.is_empty()));
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
}
