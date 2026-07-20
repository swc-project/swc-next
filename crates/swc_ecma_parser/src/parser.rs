use std::{borrow::Cow, str};

use swc_experimental_allocator::wtf8::Wtf8;
use swc_experimental_ecma_ast::*;

use crate::{
    error::{Diagnostic, Error, Label, Severity},
    lexer::{
        Lexer, LexerMode, LexerState, LexicalError, lexical_error_help, lexical_error_message,
    },
    token::{Precedence, Token, TokenTag},
    util::{decode_identifier_escapes, decode_string_escapes},
};

pub type ParseResult<T> = Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum CommentMode {
    None,
    #[default]
    Flat,
    Attached,
    Both,
}

impl CommentMode {
    pub fn collects(self) -> bool {
        self != Self::None
    }

    pub fn attaches(self) -> bool {
        matches!(self, Self::Attached | Self::Both)
    }

    pub fn is_flat(self) -> bool {
        matches!(self, Self::Flat | Self::Both)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Options {
    pub source_type: SourceType,
    pub lang: Lang,
    pub preserve_parens: bool,
    pub allow_return_outside_function: bool,
    pub semantic_errors: bool,
    pub comments: CommentMode,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            source_type: SourceType::Module,
            lang: Lang::Js,
            preserve_parens: true,
            allow_return_outside_function: false,
            semantic_errors: false,
            comments: CommentMode::Flat,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Context {
    pub in_: bool,
    pub yield_: bool,
    pub await_: bool,
    pub return_: bool,
    pub single_statement: bool,
    pub directive_prologue: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TsContext {
    pub ambient: bool,
    pub disallow_conditional_types: bool,
    pub allow_arrow_return_type: bool,
}

impl Default for TsContext {
    fn default() -> Self {
        Self {
            ambient: false,
            disallow_conditional_types: false,
            allow_arrow_return_type: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyKind {
    Program,
    Function,
    ModuleBlock,
    Other,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ParserState {
    pub cover_has_trailing_comma: Option<u32>,
    pub cover_has_init_name: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReportOptions {
    pub severity: Severity,
    pub help: Option<Cow<'static, str>>,
    pub labels: Vec<Label>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScratchBuffer {
    pub items: Vec<NodeId>,
}

impl ScratchBuffer {
    pub fn begin(&self) -> usize {
        self.items.len()
    }

    pub fn append(&mut self, index: impl AstNode) {
        self.items.push(index.node_id());
    }

    pub fn reset(&mut self, checkpoint: usize) {
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
    fn accessor_arity(self) -> Option<(usize, &'static str)> {
        match self {
            Self::Get => Some((0, "a getter must have no parameters")),
            Self::Set => Some((1, "a setter must have exactly one parameter")),
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParseStatementOpts {
    can_be_single_statement_context: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParseExpressionOpts {
    in_cover: bool,
    respect_allow_in: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArrowBodyResult {
    body: ArrowFunctionBody,
    is_expression: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ArrayCover {
    pub elements: TypedSubRange<Option<Argument>>,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ObjectCover {
    pub properties: TypedSubRange<ObjectPropertyKind>,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ParenthesizedCover {
    pub elements: TypedSubRange<Argument>,
    pub start: u32,
    pub end: u32,
    pub has_trailing_comma: bool,
}

#[derive(Clone, Copy, Debug)]
enum Cover {
    Array(ArrayCover),
    Object(ObjectCover),
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

impl Default for Context {
    fn default() -> Self {
        Self {
            in_: true,
            yield_: false,
            await_: false,
            return_: false,
            single_statement: false,
            directive_prologue: false,
        }
    }
}

/// Parser state and typed AST writer.
pub struct Parser<'parser, 'allocator, 'source> {
    pub ast: &'parser mut Ast<'allocator>,
    pub source: &'source [u8],
    pub source_type: SourceType,
    pub lang: Lang,
    pub preserve_parens: bool,
    pub allow_return_outside_function: bool,
    pub comment_mode: CommentMode,
    pub lexer: Lexer<'source>,
    pub diagnostics: Vec<Diagnostic>,
    pub current_token: Token,
    pub prev_token_end: u32,
    pub scratch_statements: ScratchBuffer,
    pub scratch_cover: ScratchBuffer,
    pub scratch_decorators: ScratchBuffer,
    pub scratch_a: ScratchBuffer,
    pub scratch_b: ScratchBuffer,
    pub context: Context,
    pub ts_context: TsContext,
    pub state: ParserState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub lexer_cursor: u32,
    pub lexer_state: LexerState,
    pub lexer_mode: LexerMode,
    pub lexer_comments_len: usize,
    pub current_token: Token,
    pub prev_token_end: u32,
    pub nodes_len: usize,
    pub extra_len: usize,
    pub diagnostics_len: usize,
    pub scratch_a_len: usize,
    pub scratch_b_len: usize,
    pub scratch_cover_len: usize,
    pub scratch_decorators_len: usize,
    pub scratch_statements_len: usize,
    pub context: Context,
    pub ts_context: TsContext,
    pub state: ParserState,
}

pub struct Peek<'peek, 'parser, 'allocator, 'source> {
    parser: &'peek mut Parser<'parser, 'allocator, 'source>,
    state: LexerState,
    cursor: u32,
    comments_len: usize,
}

impl<'parser, 'allocator, 'source> Peek<'_, 'parser, 'allocator, 'source> {
    pub fn next(&mut self) -> ParseResult<Token> {
        let mut token = Token::eof(self.parser.lexer.cursor);
        self.parser
            .lexer
            .next_token(&mut token)
            .map_err(|error| self.parser.next_token_error(error))?;
        Ok(token)
    }

    pub fn end(self) {
        self.parser.lexer.state = self.state;
        self.parser.lexer.cursor = self.cursor;
        self.parser.lexer.comments.truncate(self.comments_len);
    }
}

impl<'parser, 'allocator, 'source> Parser<'parser, 'allocator, 'source> {
    pub fn init(
        ast: &'parser mut Ast<'allocator>,
        source: &'source [u8],
        options: Options,
    ) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        ast.set_source_metadata(options.source_type, options.lang);
        build_line_starts(ast, source);
        let lexer = Lexer::init(source, options.source_type, options.comments.collects());
        let mut context = Context::default();
        context.await_ = options.source_type == SourceType::Module;
        context.return_ = options.allow_return_outside_function;
        Self {
            ast,
            source,
            source_type: options.source_type,
            lang: options.lang,
            preserve_parens: options.preserve_parens,
            allow_return_outside_function: options.allow_return_outside_function,
            comment_mode: options.comments,
            lexer,
            diagnostics: Vec::new(),
            current_token: Token::eof(0),
            prev_token_end: 0,
            scratch_statements: ScratchBuffer::default(),
            scratch_cover: ScratchBuffer::default(),
            scratch_decorators: ScratchBuffer::default(),
            scratch_a: ScratchBuffer::default(),
            scratch_b: ScratchBuffer::default(),
            context,
            ts_context: TsContext::default(),
            state: ParserState::default(),
        }
    }

    pub fn new(ast: &'parser mut Ast<'allocator>, source: &'source [u8], options: Options) -> Self {
        Self::init(ast, source, options)
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        self.ensure_capacity();
        self.advance()?;
        let body = self.parse_body(None, BodyKind::Program)?;
        let end = self.current_token.span.end;
        let hashbang = self.lexer.hashbang.map(|hashbang| {
            let start = hashbang.start;
            let end = start + u32::from(hashbang.len);
            let value = self.atom_from_span(Span::new(start, end));
            self.ast.hashbang(Span::new(0, end), value)
        });
        Ok(self
            .ast
            .program(Span::new(0, end), self.source_type, body, hashbang))
    }

    pub fn comments(&self) -> Vec<Comment<'allocator>> {
        self.lexer
            .comments
            .iter()
            .map(|comment| {
                let bytes = &self.source[comment.value.start as usize..comment.value.end as usize];
                let value = String::from_utf8_lossy(bytes);
                Comment {
                    comment_type: comment.comment_type,
                    value: self.ast.add_atom(&value),
                    span: comment.span,
                }
            })
            .collect()
    }

    pub fn identifier_name(&self, token: Token) -> ParseResult<Atom<'allocator>> {
        let is_private = token.tag == TokenTag::PrivateIdentifier;
        let start = token.span.start + u32::from(is_private);
        if token.is_escaped() {
            return self.decode_escaped_identifier(start, token.span.end);
        }
        Ok(self.atom_from_span(Span::new(start, token.span.end)))
    }

    pub fn string_value(&self, token: Token) -> ParseResult<Wtf8Atom<'allocator>> {
        let raw = &self.source[token.span.start as usize + 1..token.span.end as usize - 1];
        if token.is_escaped() {
            let decoded = self.decode_escaped_string(token.span.start + 1, token.span.end - 1);
            let value = Wtf8::from_bytes(&decoded)
                .map_err(|_| Error::new(token.span, "string is not valid WTF-8"))?;
            return Ok(self.ast.add_wtf8_atom(value));
        }
        let value = Wtf8::from_bytes(raw)
            .map_err(|_| Error::new(token.span, "string is not valid WTF-8"))?;
        Ok(self.ast.add_wtf8_atom(value))
    }

    pub fn template_element_value(
        &self,
        token: Token,
        span: Span,
    ) -> ParseResult<Atom<'allocator>> {
        if !token.is_escaped() {
            return Ok(self.atom_from_span(span));
        }
        let decoded = self.decode_escaped_string(span.start, span.end);
        let value = str::from_utf8(&decoded)
            .map_err(|_| Error::new(span, "template value is not valid UTF-8"))?;
        Ok(self.ast.add_atom(value))
    }

    fn decode_escaped_identifier(&self, start: u32, end: u32) -> ParseResult<Atom<'allocator>> {
        let mut buffer = [0u8; 256];
        let value =
            decode_identifier_escapes(&self.source[start as usize..end as usize], &mut buffer);
        let value = str::from_utf8(value)
            .map_err(|_| Error::new(Span::new(start, end), "identifier is not valid UTF-8"))?;
        Ok(self.ast.add_atom(value))
    }

    fn decode_escaped_string(&self, start: u32, end: u32) -> Vec<u8> {
        let mut decoded = Vec::new();
        decode_string_escapes(&self.source[start as usize..end as usize], &mut decoded);
        decoded
    }

    pub fn describe_token(&self, token: Token) -> Cow<'source, [u8]> {
        if token.tag == TokenTag::Eof {
            return Cow::Borrowed(b"end of file");
        }
        if let Some(text) = token.tag.to_str() {
            Cow::Borrowed(text.as_bytes())
        } else {
            Cow::Borrowed(token.text(self.source))
        }
    }

    pub fn set_lexer_mode(&mut self, mode: LexerMode) {
        self.lexer.mode = mode;
    }

    pub fn span_text(&self, span: Span) -> &'source [u8] {
        &self.source[span.start as usize..span.end as usize]
    }

    pub fn add_extra_from_scratch<N>(
        &mut self,
        scratch: &ScratchBuffer,
        scratch_checkpoint: usize,
    ) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'allocator>,
    {
        assert!(scratch_checkpoint <= scratch.items.len());
        self.ast.add_typed_sub_range(
            scratch.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        )
    }

    pub fn add_extra_from_scratch_a<N>(&mut self, scratch_checkpoint: usize) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'allocator>,
    {
        assert!(scratch_checkpoint <= self.scratch_a.items.len());
        self.ast.add_typed_sub_range(
            self.scratch_a.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        )
    }

    pub fn add_extra_from_scratch_b<N>(&mut self, scratch_checkpoint: usize) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'allocator>,
    {
        assert!(scratch_checkpoint <= self.scratch_b.items.len());
        self.ast.add_typed_sub_range(
            self.scratch_b.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        )
    }

    pub fn add_extra_from_scratch_cover<N>(&mut self, scratch_checkpoint: usize) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'allocator>,
    {
        assert!(scratch_checkpoint <= self.scratch_cover.items.len());
        self.ast.add_typed_sub_range(
            self.scratch_cover.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        )
    }

    pub fn add_extra_from_scratch_decorators<N>(
        &mut self,
        scratch_checkpoint: usize,
    ) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'allocator>,
    {
        assert!(scratch_checkpoint <= self.scratch_decorators.items.len());
        self.ast.add_typed_sub_range(
            self.scratch_decorators.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        )
    }

    pub fn add_extra_from_scratch_statements<N>(
        &mut self,
        scratch_checkpoint: usize,
    ) -> TypedSubRange<N>
    where
        N: AstNode + ExtraDataCompact<'allocator>,
    {
        assert!(scratch_checkpoint <= self.scratch_statements.items.len());
        self.ast.add_typed_sub_range(
            self.scratch_statements.items[scratch_checkpoint..]
                .iter()
                .map(|node| unsafe { N::from_node_id_unchecked(*node) }),
        )
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            lexer_cursor: self.lexer.cursor,
            lexer_state: self.lexer.state,
            lexer_mode: self.lexer.mode,
            lexer_comments_len: self.lexer.comments.len(),
            current_token: self.current_token,
            prev_token_end: self.prev_token_end,
            nodes_len: self.ast.node_count(),
            extra_len: self.ast.extra_data_count(),
            diagnostics_len: self.diagnostics.len(),
            scratch_a_len: self.scratch_a.items.len(),
            scratch_b_len: self.scratch_b.items.len(),
            scratch_cover_len: self.scratch_cover.items.len(),
            scratch_decorators_len: self.scratch_decorators.items.len(),
            scratch_statements_len: self.scratch_statements.items.len(),
            context: self.context,
            ts_context: self.ts_context,
            state: self.state,
        }
    }

    pub fn rewind(&mut self, checkpoint: Checkpoint) {
        self.lexer.cursor = checkpoint.lexer_cursor;
        self.lexer.state = checkpoint.lexer_state;
        self.lexer.mode = checkpoint.lexer_mode;
        self.lexer.comments.truncate(checkpoint.lexer_comments_len);
        self.current_token = checkpoint.current_token;
        self.prev_token_end = checkpoint.prev_token_end;
        self.ast
            .truncate(checkpoint.nodes_len, checkpoint.extra_len);
        self.diagnostics.truncate(checkpoint.diagnostics_len);
        self.scratch_a.reset(checkpoint.scratch_a_len);
        self.scratch_b.reset(checkpoint.scratch_b_len);
        self.scratch_cover.reset(checkpoint.scratch_cover_len);
        self.scratch_decorators
            .reset(checkpoint.scratch_decorators_len);
        self.scratch_statements
            .reset(checkpoint.scratch_statements_len);
        self.context = checkpoint.context;
        self.ts_context = checkpoint.ts_context;
        self.state = checkpoint.state;
    }

    pub fn begin_peek(&mut self) -> Peek<'_, 'parser, 'allocator, 'source> {
        Peek {
            state: self.lexer.state,
            cursor: self.lexer.cursor,
            comments_len: self.lexer.comments.len(),
            parser: self,
        }
    }

    pub fn peek_ahead(&mut self) -> ParseResult<Token> {
        let mut peek = self.begin_peek();
        let token = peek.next();
        peek.end();
        token
    }

    pub fn ensure_capacity(&mut self) {
        let source_len = self.source.len();
        let estimated_nodes = if source_len < 4_000 {
            256.max(source_len / 4)
        } else if source_len < 10_000 {
            source_len / 3
        } else if source_len < 100_000 {
            source_len / 6
        } else {
            source_len / 8
        };
        let estimated_extra = if source_len < 5_000_000 {
            estimated_nodes / 4
        } else {
            estimated_nodes / 3
        };
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
        kind: BodyKind,
    ) -> ParseResult<TypedSubRange<StatementListItem>> {
        self.context.directive_prologue = matches!(
            kind,
            BodyKind::Program | BodyKind::Function | BodyKind::ModuleBlock
        );
        let mut body = Vec::new();
        while !self.is_at_body_end(terminator) {
            let statement = self.parse_statement(ParseStatementOpts::default())?;
            let item = if self.context.directive_prologue
                && let StmtData::ExpressionStatement(expression_statement) =
                    self.ast.stmt_data(statement)
                && let ExprData::StringLiteral(string_literal) = self
                    .ast
                    .expr_data(expression_statement.expression(self.ast))
            {
                let span = string_literal.span(self.ast);
                self.parse_directive(statement, string_literal, span)?
            } else {
                StatementListItem::Stmt(statement)
            };
            self.context.directive_prologue =
                self.context.directive_prologue && item.is_directive(self.ast);
            body.push(item);
        }
        self.context.directive_prologue = false;
        Ok(self.ast.add_typed_sub_range(body))
    }

    pub fn is_at_body_end(&self, terminator: Option<TokenTag>) -> bool {
        self.current_token.tag == TokenTag::Eof
            || terminator.is_some_and(|tag| self.current_token.tag == tag)
    }

    fn parse_statement(&mut self, opts: ParseStatementOpts) -> ParseResult<Stmt> {
        self.context.single_statement = opts.can_be_single_statement_context;
        if self.is_start_of_ts_declaration()? {
            let result = self.parse_ts_declaration();
            self.context.single_statement = false;
            return result;
        }
        let result = match self.current_token.tag {
            TokenTag::Await => self.parse_await_using_or_expression(),
            TokenTag::Var => self
                .parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration))),
            TokenTag::Const => self.parse_const_or_const_enum(),
            TokenTag::Let => self.parse_let(),
            TokenTag::Using => self.parse_using_or_expression(),
            TokenTag::LeftBrace => Ok(Stmt::BlockStatement(self.parse_block_statement()?)),
            TokenTag::Semicolon => self.parse_empty_statement(),
            TokenTag::If => self.parse_if_statement(),
            TokenTag::While => self.parse_while_statement(),
            TokenTag::Do => self.parse_do_while_statement(),
            TokenTag::Return => self.parse_return_statement(),
            TokenTag::Throw => self.parse_throw_statement(),
            TokenTag::Break => self.parse_break_statement(),
            TokenTag::Continue => self.parse_continue_statement(),
            TokenTag::Debugger => self.parse_debugger_statement(),
            TokenTag::Function => Ok(Stmt::Declaration(Decl::Function(
                self.parse_function(ParseFunctionOpts::default(), None)?,
            ))),
            TokenTag::Async => self.parse_async_function_or_expression(),
            TokenTag::For => self.parse_for_statement(),
            TokenTag::Class => Ok(Stmt::Declaration(Decl::Class(
                self.parse_class(ParseClassOpts::default(), None)?,
            ))),
            TokenTag::At => self.parse_decorated_statement(),
            TokenTag::Switch => self.parse_switch_statement(),
            TokenTag::Try => self.parse_try_statement(),
            TokenTag::With => self.parse_with_statement(),
            TokenTag::Import => self.parse_import_declaration_or_expression(),
            TokenTag::Export => self.parse_export_declaration(),
            _ => self.parse_expression_or_labeled_statement_or_directive(),
        };
        self.context.single_statement = false;
        result
    }

    fn parse_decorated_statement(&mut self) -> ParseResult<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::At);
        let (decorators_start, decorators) = self.parse_decorators()?;
        if self.current_token.tag != TokenTag::Export {
            return Ok(Stmt::Declaration(Decl::Class(self.parse_class_decorated(
                ParseClassOpts::default(),
                Some(decorators_start),
                decorators,
            )?)));
        }
        self.parse_export_decorated(decorators_start, decorators)
    }

    fn parse_export_decorated(
        &mut self,
        decorators_start: u32,
        decorators: TypedSubRange<Decorator>,
    ) -> ParseResult<Stmt> {
        let export_start = self.current_token.span.start;
        self.advance()?;
        let is_default = self.eat(TokenTag::Default)?;
        let class = self.parse_class_decorated(
            ParseClassOpts {
                is_default_export: is_default,
                ..ParseClassOpts::default()
            },
            Some(decorators_start),
            decorators,
        )?;
        let end = class.span(self.ast).end;
        if is_default {
            Ok(self.ast.stmt_export_default_declaration(
                Span::new(export_start, end),
                ExportDefaultDeclarationKind::Class(class),
            ))
        } else {
            let specifiers = self.empty_range();
            let attributes = self.empty_range();
            Ok(self.ast.stmt_export_named_declaration(
                Span::new(export_start, end),
                Some(Decl::Class(class)),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ))
        }
    }

    fn parse_let(&mut self) -> ParseResult<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Let);
        if !self.is_let_identifier()? {
            return self
                .parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        }
        self.parse_expression_statement()
    }

    fn parse_using_or_expression(&mut self) -> ParseResult<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Using);
        if !self.is_using_identifier()? {
            return self
                .parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)
                .map(|declaration| Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        }
        self.parse_expression_statement()
    }

    fn parse_await_using_or_expression(&mut self) -> ParseResult<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Await);
        let next = self.peek_ahead()?;
        if next.tag == TokenTag::Using {
            let start = self.current_token.span.start;
            self.advance()?;
            if !self.is_using_identifier()? {
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
            return self.parse_await_expression_statement(start);
        }
        self.parse_expression_or_labeled_statement_or_directive()
    }

    fn parse_await_expression_statement(&mut self, start: u32) -> ParseResult<Stmt> {
        let expression = self.parse_await_expression(start)?;
        self.parse_expression_statement_with_expression(start, expression)
    }

    fn parse_const_or_const_enum(&mut self) -> ParseResult<Stmt> {
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

    fn parse_async_function_or_expression(&mut self) -> ParseResult<Stmt> {
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
                .map(|function| Stmt::Declaration(Decl::Function(function)));
        }
        self.parse_expression_or_labeled_statement_or_directive()
    }

    fn parse_import_declaration_or_expression(&mut self) -> ParseResult<Stmt> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Import);
        let next = self.peek_ahead()?;
        if matches!(next.tag, TokenTag::LeftParen | TokenTag::Dot) {
            return self.parse_expression_statement();
        }
        self.parse_import_declaration()
    }

    fn is_let_identifier(&mut self) -> ParseResult<bool> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Let);
        let next = self.peek_ahead()?;
        if next.tag == TokenTag::Semicolon {
            return Ok(true);
        }
        if self.context.single_statement && !can_start_binding(next.tag) {
            return Ok(true);
        }
        Ok(false)
    }

    fn is_using_identifier(&mut self) -> ParseResult<bool> {
        debug_assert_eq!(self.current_token.tag, TokenTag::Using);
        let next = self.peek_ahead()?;
        if next.tag.is_identifier_like() && !self.can_insert_implicit_semicolon(next) {
            return Ok(false);
        }
        Ok(true)
    }

    fn parse_block_statement(&mut self) -> ParseResult<BlockStatement> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut statements = Vec::new();
        while !matches!(self.current_token.tag, TokenTag::RightBrace | TokenTag::Eof) {
            statements.push(self.parse_statement(ParseStatementOpts::default())?);
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let body = self.ast.add_typed_sub_range(statements);
        Ok(self.ast.block_statement(Span::new(start, end), body))
    }

    fn parse_empty_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.expect(TokenTag::Semicolon)?.span;
        Ok(self.ast.stmt_empty_statement(span))
    }

    fn parse_if_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::If)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let test = self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        self.expect(TokenTag::RightParen)?;
        let consequent = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        let alternate = if self.eat(TokenTag::Else)? {
            Some(self.parse_statement(ParseStatementOpts {
                can_be_single_statement_context: true,
            })?)
        } else {
            None
        };
        let end = alternate.map_or_else(
            || consequent.span(self.ast).end,
            |stmt| stmt.span(self.ast).end,
        );
        Ok(self
            .ast
            .stmt_if_statement(Span::new(start, end), test, consequent, alternate))
    }

    fn parse_while_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::While)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let test = self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_while_statement(Span::new(start, end), test, body))
    }

    fn parse_do_while_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Do)?.span.start;
        let body = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        self.expect(TokenTag::While)?;
        self.expect(TokenTag::LeftParen)?;
        let test = self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        self.expect(TokenTag::RightParen)?;
        self.eat(TokenTag::Semicolon)?;
        let end = self.prev_token_end;
        Ok(self
            .ast
            .stmt_do_while_statement(Span::new(start, end), body, test))
    }

    fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Return)?.span.start;
        if !self.context.return_ {
            return self.error_at(
                Span::new(start, self.prev_token_end),
                "return is not allowed outside a function",
            );
        }
        let argument = if self.current_token.has_line_terminator_before()
            || matches!(
                self.current_token.tag,
                TokenTag::Semicolon | TokenTag::RightBrace | TokenTag::Eof
            ) {
            None
        } else {
            Some(self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?)
        };
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self
            .ast
            .stmt_return_statement(Span::new(start, end), argument))
    }

    fn parse_throw_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Throw)?.span.start;
        if self.current_token.has_line_terminator_before() {
            return self.error("line terminator is not allowed after throw");
        }
        let argument = self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self
            .ast
            .stmt_throw_statement(Span::new(start, end), argument))
    }

    fn parse_break_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Break)?.span.start;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag.is_identifier_like()
        {
            None
        } else {
            Some(self.parse_label_identifier()?)
        };
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self.ast.stmt_break_statement(Span::new(start, end), label))
    }

    fn parse_continue_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Continue)?.span.start;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag.is_identifier_like()
        {
            None
        } else {
            Some(self.parse_label_identifier()?)
        };
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self
            .ast
            .stmt_continue_statement(Span::new(start, end), label))
    }

    fn parse_debugger_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Debugger)?.span.start;
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self.ast.stmt_debugger_statement(Span::new(start, end)))
    }

    fn parse_with_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::With)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let object = self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_with_statement(Span::new(start, end), object, body))
    }

    fn parse_expression_or_labeled_statement_or_directive(&mut self) -> ParseResult<Stmt> {
        if self.current_token.tag.is_identifier_like() && self.peek_ahead()?.tag == TokenTag::Colon
        {
            return self.parse_labeled_statement();
        }
        self.parse_expression_statement()
    }

    fn parse_expression_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.current_token.span.start;
        let expression =
            self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        self.parse_expression_statement_with_expression(start, expression)
    }

    fn parse_expression_statement_with_expression(
        &mut self,
        start: u32,
        expression: Expr,
    ) -> ParseResult<Stmt> {
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self
            .ast
            .stmt_expression_statement(Span::new(start, end), expression))
    }

    fn parse_labeled_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.current_token.span.start;
        let label = self.parse_label_identifier()?;
        self.expect(TokenTag::Colon)?;
        let body = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_labeled_statement(Span::new(start, end), label, body))
    }

    fn parse_directive(
        &mut self,
        statement: Stmt,
        string_literal: StringLiteral,
        span: Span,
    ) -> ParseResult<StatementListItem> {
        let raw_value =
            Wtf8::from_bytes(&self.source[span.start as usize + 1..span.end as usize - 1])
                .map_err(|_| Error::new(span, "directive is not valid WTF-8"))?;
        Ok(StatementListItem::Directive(self.ast.directive(
            statement.span(self.ast),
            string_literal,
            self.ast.add_wtf8_atom(raw_value),
        )))
    }

    fn parse_variable_declaration(
        &mut self,
        opts: ParseVariableDeclarationOpts,
        start_from_param: Option<u32>,
    ) -> ParseResult<VariableDeclaration> {
        let declaration_start = start_from_param.unwrap_or(self.current_token.span.start);
        if self.current_token.tag == TokenTag::Await {
            self.advance()?;
            if self.current_token.has_line_terminator_before() {
                return self.error("'await using' may not contain a line terminator");
            }
        }
        let kind = self.parse_variable_kind(opts.await_using)?;
        let context = if opts.is_declare || self.ts_context.ambient {
            DeclaratorCtx::Declare
        } else {
            DeclaratorCtx::Normal
        };
        let mut declarators = vec![self.parse_variable_declarator(kind, context)?];
        while self.eat(TokenTag::Comma)? {
            declarators.push(self.parse_variable_declarator(kind, context)?);
        }
        let end = declarators
            .last()
            .expect("a variable declaration has at least one declarator")
            .span(self.ast)
            .end;
        let end = self.eat_semicolon(end)?;
        if self.context.single_statement
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
        let declarators = self.ast.add_typed_sub_range(declarators);
        Ok(self.ast.variable_declaration(
            Span::new(declaration_start, end),
            kind,
            declarators,
            opts.is_declare,
        ))
    }

    fn parse_variable_kind(&mut self, await_using: bool) -> ParseResult<VariableKind> {
        let tag = self.current_token.tag;
        self.advance()?;
        Ok(match tag {
            TokenTag::Var => VariableKind::Var,
            TokenTag::Let => VariableKind::Let,
            TokenTag::Const => VariableKind::Const,
            TokenTag::Using => {
                if await_using {
                    VariableKind::AwaitUsing
                } else {
                    VariableKind::Using
                }
            }
            _ => return self.error("expected variable declaration"),
        })
    }

    fn parse_variable_declarator(
        &mut self,
        kind: VariableKind,
        context: DeclaratorCtx,
    ) -> ParseResult<VariableDeclarator> {
        let start = self.current_token.span.start;
        let id = self.parse_binding_pattern()?;
        let id_span = id.span(self.ast);
        let is_using = matches!(kind, VariableKind::Using | VariableKind::AwaitUsing);
        let destructuring = self.is_destructuring_pattern(id);
        if is_using && destructuring {
            return self.error_at(
                id_span,
                "using declarations cannot have destructuring patterns",
            );
        }
        let definite = self.lang.is_typescript()
            && !self.current_token.has_line_terminator_before()
            && self.eat(TokenTag::LogicalNot)?;
        let mut end = if definite {
            self.prev_token_end
        } else {
            id_span.end
        };
        if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
            let annotation = self.parse_type_annotation()?;
            end = annotation.span(self.ast).end;
            self.apply_type_annotation_to_pattern(id, annotation);
        }
        let init = if self.eat(TokenTag::Assign)? {
            let expression = self.parse_expression(
                Precedence::ASSIGNMENT,
                ParseExpressionOpts {
                    respect_allow_in: context == DeclaratorCtx::ForLoop,
                    ..ParseExpressionOpts::default()
                },
            )?;
            end = expression.span(self.ast).end;
            Some(expression)
        } else {
            None
        };
        if init.is_none() && context == DeclaratorCtx::Normal {
            if destructuring {
                return self.error_at(id_span, "destructuring declarations require an initializer");
            }
            if matches!(
                kind,
                VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing
            ) {
                return self.error_at(
                    id_span,
                    "const and using declarations require an initializer",
                );
            }
        }
        Ok(self
            .ast
            .variable_declarator(Span::new(start, end), id, init, definite))
    }

    fn parse_binding_pattern(&mut self) -> ParseResult<BindingPattern> {
        match self.current_token.tag {
            tag if tag.is_identifier_like() && !tag.is_unconditionally_reserved() => {
                let token = self.current_token;
                let name = self.identifier_name(token)?;
                self.validate_identifier(token, name.as_str(), "a binding identifier")?;
                self.advance()?;
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_binding_identifier(
                    token.span, name, decorators, None, false, None,
                ))
            }
            TokenTag::LeftBracket => self.parse_array_pattern(),
            TokenTag::LeftBrace => self.parse_object_pattern(),
            _ => self.error("expected a binding pattern"),
        }
    }

    fn parse_array_pattern(&mut self) -> ParseResult<BindingPattern> {
        let expression = self.parse_array_expression(true)?;
        self.expression_to_binding_pattern(expression)
    }

    fn parse_object_pattern(&mut self) -> ParseResult<BindingPattern> {
        let expression = self.parse_object_expression(true)?;
        self.expression_to_binding_pattern(expression)
    }

    fn parse_assignment_pattern(&mut self, left: BindingPattern) -> ParseResult<BindingPattern> {
        if !self.eat(TokenTag::Assign)? {
            return Ok(left);
        }
        let right =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        let span = Span::new(left.span(self.ast).start, right.span(self.ast).end);
        let decorators = self.empty_range();
        Ok(self
            .ast
            .binding_pattern_assignment_pattern(span, left, right, decorators, None, false))
    }

    fn mark_pattern_optional(&mut self, pattern: BindingPattern, end: u32) {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(node) => node.set_optional(self.ast, true),
            BindingPatternData::ArrayPattern(node) => node.set_optional(self.ast, true),
            BindingPatternData::ObjectPattern(node) => node.set_optional(self.ast, true),
            _ => unreachable!("only binding patterns can be marked optional"),
        }
        let span = pattern.span(self.ast);
        pattern.set_span(self.ast, Span::new(span.start, end));
    }

    fn apply_type_annotation_to_pattern(
        &mut self,
        pattern: BindingPattern,
        annotation: TsTypeAnnotation,
    ) {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(node) => {
                node.set_type_annotation(self.ast, Some(annotation));
            }
            BindingPatternData::ArrayPattern(node) => {
                node.set_type_annotation(self.ast, Some(annotation));
            }
            BindingPatternData::ObjectPattern(node) => {
                node.set_type_annotation(self.ast, Some(annotation));
            }
            _ => unreachable!("only binding patterns can have type annotations"),
        }
        let span = pattern.span(self.ast);
        pattern.set_span(
            self.ast,
            Span::new(span.start, annotation.span(self.ast).end),
        );
    }

    fn parse_function(
        &mut self,
        opts: ParseFunctionOpts,
        start_from_param: Option<u32>,
    ) -> ParseResult<Function> {
        let start = start_from_param.unwrap_or(self.current_token.span.start);
        self.expect(TokenTag::Function)?;
        let generator = self.eat(TokenTag::Star)?;
        let old_context = self.context;
        let is_function_expression = opts.is_expression && !opts.is_default_export;
        if is_function_expression {
            self.context.yield_ = generator;
            self.context.await_ = opts.is_async;
        }
        let id = if self.current_token.tag.is_identifier_like() {
            Some(self.parse_binding_identifier()?)
        } else if !opts.is_expression && !opts.is_default_export {
            return self.error("function declarations require a name");
        } else {
            None
        };
        self.context = old_context;
        let old_context = self.context;
        self.context.return_ = true;
        self.context.yield_ = generator;
        self.context.await_ = opts.is_async;
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
            let body = if self.current_token.tag == TokenTag::LeftBrace {
                Some(self.parse_function_body()?)
            } else if self.lang.is_typescript() {
                None
            } else {
                return self.error("expected '{' to start function body");
            };
            let end = if let Some(body) = body {
                body.span(self.ast).end
            } else {
                self.eat_semicolon(self.prev_token_end)?
            };
            Ok((params, body, type_parameters, return_type, end))
        })();
        self.context = old_context;
        let (params, body, type_parameters, return_type, end) = tail?;
        let function_type = if is_function_expression {
            FunctionType::FunctionExpression
        } else if body.is_none() {
            FunctionType::TsDeclareFunction
        } else {
            FunctionType::FunctionDeclaration
        };
        if self.context.single_statement && (opts.is_async || generator) {
            return self.error_at(
                Span::new(start, end),
                "async and generator functions are not allowed in a single-statement context",
            );
        }
        Ok(self.ast.function(
            Span::new(start, end),
            function_type,
            id,
            generator,
            opts.is_async,
            opts.is_declare,
            params,
            body,
            type_parameters,
            return_type,
        ))
    }

    fn parse_class(
        &mut self,
        opts: ParseClassOpts,
        start_from_param: Option<u32>,
    ) -> ParseResult<Class> {
        let decorators = self.empty_range();
        self.parse_class_decorated(opts, start_from_param, decorators)
    }

    fn parse_class_decorated(
        &mut self,
        opts: ParseClassOpts,
        start_from_param: Option<u32>,
        decorators: TypedSubRange<Decorator>,
    ) -> ParseResult<Class> {
        let start = start_from_param.unwrap_or(self.current_token.span.start);
        self.expect(TokenTag::Class)?;
        let class_type = if opts.is_expression && !opts.is_default_export {
            ClassType::ClassExpression
        } else {
            ClassType::ClassDeclaration
        };
        if class_type == ClassType::ClassDeclaration && self.context.single_statement {
            return self.error_at(
                Span::new(start, self.current_token.span.end),
                "a class declaration is not allowed in a single-statement context",
            );
        }
        let id = if self.can_start_class_name()? {
            Some(self.parse_binding_identifier()?)
        } else if !opts.is_expression && !opts.is_default_export {
            return self.error("class declarations require a name");
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
                    super_class = instantiation.expression(self.ast);
                    Some(instantiation.type_arguments(self.ast))
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
            self.empty_range()
        };
        let body = self.parse_class_body()?;
        let end = body.span(self.ast).end;
        Ok(self.ast.class(
            Span::new(start, end),
            class_type,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_arguments,
            implements,
            opts.is_abstract,
            opts.is_declare,
        ))
    }

    fn can_start_class_name(&mut self) -> ParseResult<bool> {
        let tag = self.current_token.tag;
        if !tag.is_identifier_like() || tag == TokenTag::Extends {
            return Ok(false);
        }
        if self.lang.is_typescript() && tag == TokenTag::Implements {
            return Ok(!self.peek_ahead()?.tag.is_identifier_like());
        }
        Ok(true)
    }

    fn parse_implements_clause(&mut self) -> ParseResult<TypedSubRange<TsClassImplements>> {
        match self.parse_heritage_clause(TokenTag::Implements, HeritageKind::Class)? {
            HeritageClause::Class(implements) => Ok(implements),
            HeritageClause::Interface(_) => unreachable!(),
        }
    }

    fn parse_heritage_expression(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            return self.error("expected an identifier in a heritage clause");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        let mut expression = self.ast.expr_identifier_reference(token.span, name, None);
        while self.current_token.tag == TokenTag::Dot {
            expression = self.parse_static_member_expression(expression, false)?;
        }
        Ok(expression)
    }

    fn parse_class_body(&mut self) -> ParseResult<ClassBody> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut elements = Vec::new();
        while !matches!(self.current_token.tag, TokenTag::RightBrace | TokenTag::Eof) {
            if self.eat(TokenTag::Semicolon)? {
                continue;
            }
            elements.push(self.parse_class_element()?);
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let elements = self.ast.add_typed_sub_range(elements);
        Ok(self.ast.class_body(Span::new(start, end), elements))
    }

    fn parse_class_element(&mut self) -> ParseResult<ClassElement> {
        let element_start = self.current_token.span.start;
        let decorators = if self.current_token.tag == TokenTag::At {
            self.parse_decorators()?.1
        } else {
            self.empty_range()
        };

        if let Some(block) = self.try_static_block(decorators)? {
            return Ok(block);
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
                return self.error("accessor properties cannot be methods");
            }
            if definite {
                return self.error_at(
                    key.span(self.ast),
                    "a method cannot have a definite assignment assertion",
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
            return self.error("expected '(' for method definition");
        }
        if modifiers.kind != MethodDefinitionKind::Method {
            return self.error("expected '(' for getter or setter definition");
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

    fn consume_modifier(
        &mut self,
        modifiers: &mut ClassElementModifiers,
    ) -> ParseResult<ClassModifierStep> {
        let token = self.current_token;
        let is_typescript = self.lang.is_typescript();
        if !is_modifier(token.tag, is_typescript) {
            return Ok(ClassModifierStep::None);
        }

        let next = self.peek_ahead()?;
        let consume_as_modifier = can_start_element_key(next.tag)
            && !(requires_same_line(token.tag) && next.has_line_terminator_before())
            && !modifiers.conflicts(token.tag);

        if !consume_as_modifier {
            let name = self.identifier_name(token)?;
            self.advance()?;
            return Ok(ClassModifierStep::Key(
                self.ast.property_key_identifier_name(token.span, name),
            ));
        }

        if token.is_escaped() {
            return self.error_at(token.span, "keywords may not contain escape sequences");
        }
        self.advance()?;
        modifiers.set(token.tag);
        Ok(ClassModifierStep::Consumed)
    }

    fn try_static_block(
        &mut self,
        decorators: TypedSubRange<Decorator>,
    ) -> ParseResult<Option<ClassElement>> {
        if self.current_token.tag != TokenTag::Static
            || self.peek_ahead()?.tag != TokenTag::LeftBrace
        {
            return Ok(None);
        }
        let static_token = self.current_token;
        if !decorators.is_empty() {
            self.report(
                decorators
                    .get_node(self.ast, 0)
                    .expect("non-empty decorator range contains its first node")
                    .span(self.ast),
                "Decorators cannot be applied to static blocks",
                ReportOptions::default(),
            );
        }
        self.report_if_escaped_keyword(static_token);
        self.advance_without_escape_check()?;
        self.parse_static_block(static_token.span.start).map(Some)
    }

    fn parse_static_block(&mut self, start: u32) -> ParseResult<ClassElement> {
        let old_context = self.context;
        self.context.return_ = false;
        self.context.yield_ = false;
        self.context.await_ = true;
        let block = self.parse_block_statement();
        self.context = old_context;
        let block = block?;
        let body = block.body(self.ast);
        Ok(ClassElement::StaticBlock(self.ast.static_block(
            Span::new(start, block.span(self.ast).end),
            body,
        )))
    }

    fn validate_private_constructor(&mut self, key: PropertyKey, computed: bool) {
        if computed {
            return;
        }
        if matches!(
            self.ast.property_key_data(key),
            PropertyKeyData::PrivateIdentifier(private)
                if private.name(self.ast).as_str() == "constructor"
        ) {
            self.report(
                key.span(self.ast),
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
        let span = key.span(self.ast);
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
        if let Some(span) = self.prop_name_eq(key, "prototype") {
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

    fn restore_method_context(&mut self, context: Context) {
        self.context = context;
    }

    fn parse_method_definition(
        &mut self,
        start: u32,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        modifiers: ClassElementModifiers,
        optional: bool,
    ) -> ParseResult<ClassElement> {
        self.validate_method_modifiers(key, modifiers);
        if modifiers.is_static && !modifiers.computed {
            self.validate_static_prototype_or_constructor(key, StaticMemberKind::Method);
        }

        let function_start = self.current_token.span.start;
        let old_context = self.context;
        self.context.return_ = true;
        self.context.await_ = modifiers.is_async;
        self.context.yield_ = modifiers.is_generator;
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
            self.check_accessor_arity(modifiers.kind, params)?;
            let return_type =
                if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_return_type_annotation()?)
                } else {
                    None
                };
            if self.current_token.tag == TokenTag::LeftBrace {
                let body = self.parse_function_body()?;
                let end = body.span(self.ast).end;
                Ok((
                    params,
                    type_parameters,
                    return_type,
                    Some(body),
                    FunctionType::FunctionExpression,
                    end,
                ))
            } else if self.lang.is_typescript() {
                let end = self.eat_semicolon(self.prev_token_end)?;
                Ok((
                    params,
                    type_parameters,
                    return_type,
                    None,
                    FunctionType::TsEmptyBodyFunctionExpression,
                    end,
                ))
            } else {
                self.error("expected '{' to start method body")
            }
        })();
        self.restore_method_context(old_context);
        let (params, type_parameters, return_type, body, function_type, end) = result?;
        let function = self.ast.function(
            Span::new(function_start, end),
            function_type,
            None,
            modifiers.is_generator,
            modifiers.is_async,
            false,
            params,
            body,
            type_parameters,
            return_type,
        );
        let method = self.ast.method_definition(
            Span::new(start, end),
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
        Ok(ClassElement::MethodDefinition(method))
    }

    fn parse_property_definition(
        &mut self,
        start: u32,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        modifiers: ClassElementModifiers,
        optional: bool,
        definite: bool,
    ) -> ParseResult<ClassElement> {
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

        let old_context = self.context;
        self.context.await_ = false;
        self.context.yield_ = false;
        let value = if self.eat(TokenTag::Assign)? {
            Some(self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?)
        } else {
            None
        };
        self.context = old_context;

        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(ClassElement::PropertyDefinition(
            self.ast.property_definition(
                Span::new(start, end),
                decorators,
                key,
                value,
                modifiers.computed,
                modifiers.is_static,
                modifiers.is_accessor,
                type_annotation,
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

    fn parse_decorators(&mut self) -> ParseResult<(u32, TypedSubRange<Decorator>)> {
        let start = self.current_token.span.start;
        let mut decorators = Vec::new();
        while self.current_token.tag == TokenTag::At {
            decorators.push(self.parse_decorator()?);
        }
        Ok((start, self.ast.add_typed_sub_range(decorators)))
    }

    fn parse_decorator(&mut self) -> ParseResult<Decorator> {
        let start = self.expect(TokenTag::At)?.span.start;
        let expression = self.parse_left_hand_side_expression(LhsContext::Decorator)?;
        Ok(self
            .ast
            .decorator(Span::new(start, expression.span(self.ast).end), expression))
    }

    fn parse_left_hand_side_expression(&mut self, context: LhsContext) -> ParseResult<Expr> {
        let mut expression = match self.current_token.tag {
            TokenTag::LeftParen => {
                self.parse_parenthesized_or_arrow_function(None, false, Precedence::GROUPING)?
            }
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

        Ok(expression)
    }

    fn prop_name_eq(&self, key: PropertyKey, expected: &str) -> Option<Span> {
        let matches = match self.ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(identifier) => {
                identifier.name(self.ast).as_str() == expected
            }
            PropertyKeyData::StringLiteral(string) => {
                string.value(self.ast).as_wtf8().as_str() == Some(expected)
            }
            _ => false,
        };
        if matches {
            Some(key.span(self.ast))
        } else {
            None
        }
    }

    fn check_accessor_arity(
        &self,
        kind: impl Into<PropertyOrMethodKind>,
        params: FormalParameters,
    ) -> ParseResult<()> {
        let kind = kind.into();
        let count = params.items(self.ast).len();
        let has_rest = params.rest(self.ast).is_some();
        if let Some((expected, message)) = kind.accessor_arity()
            && (count != expected || has_rest)
        {
            return self.error_at(params.span(self.ast), message);
        }
        Ok(())
    }

    fn parse_formal_parameters(
        &mut self,
        kind: FormalParameterKind,
        allow_parameter_properties: bool,
    ) -> ParseResult<FormalParameters> {
        let start = self.expect(TokenTag::LeftParen)?.span.start;
        let mut items = Vec::new();
        let mut rest = None;
        while self.current_token.tag != TokenTag::RightParen {
            let parameter_start = self.current_token.span.start;
            let decorators = if self.lang.is_typescript() && self.current_token.tag == TokenTag::At
            {
                self.parse_decorators()?.1
            } else {
                self.empty_range()
            };

            if self.current_token.tag == TokenTag::Spread {
                rest = Some(self.parse_binding_rest_element(decorators)?);
                if self.eat(TokenTag::Comma)? {
                    return self.error("a rest parameter must be last");
                }
                break;
            }

            items.push(self.parse_formal_parameter(
                allow_parameter_properties,
                decorators,
                parameter_start,
            )?);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let end = self.expect(TokenTag::RightParen)?.span.end;
        let items = self.ast.add_typed_sub_range(items);
        Ok(self
            .ast
            .formal_parameters(Span::new(start, end), items, rest, kind))
    }

    fn parse_formal_parameter(
        &mut self,
        allow_parameter_properties: bool,
        decorators: TypedSubRange<Decorator>,
        start: u32,
    ) -> ParseResult<FormalParameterItem> {
        if self.lang.is_typescript() && self.current_token.tag == TokenTag::This {
            let this_parameter = self.parse_this_parameter()?;
            let span = this_parameter.span(self.ast);
            let parameter = self.ast.formal_parameter(
                span,
                FormalParameterPattern::TsThisParameter(this_parameter),
            );
            return Ok(FormalParameterItem::FormalParameter(parameter));
        }

        let parameter_property = if self.lang.is_typescript() && allow_parameter_properties {
            self.parse_parameter_property_modifiers()?
        } else {
            ParameterPropertyModifiers::default()
        };
        let mut pattern = self.parse_binding_pattern()?;
        if self.lang.is_typescript() && self.eat(TokenTag::Question)? {
            self.mark_pattern_optional(pattern, self.prev_token_end);
        }
        if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
            let annotation = self.parse_type_annotation()?;
            self.apply_type_annotation_to_pattern(pattern, annotation);
        }
        pattern = self.parse_assignment_pattern(pattern)?;
        let span = pattern.span(self.ast);
        if parameter_property.present {
            let parameter_property = self.ast.ts_parameter_property(
                Span::new(start, span.end),
                decorators,
                pattern,
                parameter_property.r#override,
                parameter_property.readonly,
                parameter_property.accessibility,
            );
            return Ok(FormalParameterItem::TsParameterProperty(parameter_property));
        }

        self.apply_decorators_to_pattern(pattern, decorators);
        let parameter_pattern = FormalParameterPattern::BindingPattern(pattern);
        let parameter = self.ast.formal_parameter(span, parameter_pattern);
        Ok(FormalParameterItem::FormalParameter(parameter))
    }

    fn parse_this_parameter(&mut self) -> ParseResult<TsThisParameter> {
        let start = self.expect(TokenTag::This)?.span.start;
        let type_annotation = if self.current_token.tag == TokenTag::Colon {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        let end = type_annotation.map_or(self.prev_token_end, |annotation| {
            annotation.span(self.ast).end
        });
        Ok(self
            .ast
            .ts_this_parameter(Span::new(start, end), type_annotation))
    }

    fn parse_binding_rest_element(
        &mut self,
        decorators: TypedSubRange<Decorator>,
    ) -> ParseResult<BindingRestElement> {
        let start = self.expect(TokenTag::Spread)?.span.start;
        let argument = self.parse_binding_pattern()?;
        let type_annotation =
            if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
        let end = type_annotation.map_or(argument.span(self.ast).end, |annotation| {
            annotation.span(self.ast).end
        });
        Ok(self.ast.binding_rest_element(
            Span::new(start, end),
            argument,
            decorators,
            type_annotation,
            false,
        ))
    }

    fn parse_parameter_property_modifiers(&mut self) -> ParseResult<ParameterPropertyModifiers> {
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
        Ok(modifiers)
    }

    fn is_parameter_property_modifier_start(&mut self) -> ParseResult<bool> {
        if !is_parameter_property_modifier_tag(self.current_token.tag) {
            return Ok(false);
        }
        let next = self.peek_ahead()?;
        if next.has_line_terminator_before() {
            return Ok(false);
        }
        Ok(can_follow_parameter_property_modifier(next.tag))
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
                node.set_decorators(self.ast, decorators)
            }
            BindingPatternData::AssignmentPattern(node) => {
                node.set_decorators(self.ast, decorators)
            }
            BindingPatternData::BindingRestElement(node) => {
                node.set_decorators(self.ast, decorators)
            }
            BindingPatternData::ArrayPattern(node) => node.set_decorators(self.ast, decorators),
            BindingPatternData::ObjectPattern(node) => node.set_decorators(self.ast, decorators),
            BindingPatternData::SimpleAssignmentTarget(_) => {}
        }
    }

    fn parse_function_body(&mut self) -> ParseResult<FunctionBody> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let body = self.parse_body(Some(TokenTag::RightBrace), BodyKind::Function)?;
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        Ok(self.ast.function_body(Span::new(start, end), body))
    }

    fn parse_import_declaration(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Import)?.span.start;
        self.parse_import_declaration_from(start)
    }

    fn parse_import_declaration_from(&mut self, start: u32) -> ParseResult<Stmt> {
        if self.current_token.tag == TokenTag::StringLiteral {
            return self.parse_side_effect_import(start, None);
        }

        let mut phase = None;
        let mut import_kind = ImportOrExportKind::Value;
        let next = self.peek_ahead()?;
        if self.lang.is_typescript()
            && self.current_token.tag == TokenTag::Type
            && self.is_type_import_modifier(next)?
        {
            import_kind = ImportOrExportKind::Type;
            self.advance()?;
        } else if self.current_token.tag == TokenTag::Source
            && next.tag.is_identifier_like()
            && next.tag != TokenTag::From
        {
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
        self.expect(TokenTag::From)?;
        let source = self.parse_module_specifier()?;
        let attributes = self.parse_with_clause()?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self.ast.stmt_import_declaration(
            Span::new(start, end),
            specifiers,
            source,
            attributes,
            phase,
            import_kind,
        ))
    }

    fn is_type_import_modifier(&mut self, after_type: Token) -> ParseResult<bool> {
        if matches!(after_type.tag, TokenTag::LeftBrace | TokenTag::Star) {
            return Ok(true);
        }
        if !after_type.tag.is_identifier_like() {
            return Ok(false);
        }
        if after_type.tag == TokenTag::From {
            let mut peek = self.begin_peek();
            let after_from = (|| {
                peek.next()?;
                peek.next()
            })();
            peek.end();
            let after_from = after_from?;
            return Ok(matches!(after_from.tag, TokenTag::From | TokenTag::Assign));
        }
        Ok(true)
    }

    fn parse_side_effect_import(
        &mut self,
        start: u32,
        phase: Option<ImportPhase>,
    ) -> ParseResult<Stmt> {
        let source = self.parse_module_specifier()?;
        let attributes = self.parse_with_clause()?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        let specifiers = self.empty_range();
        Ok(self.ast.stmt_import_declaration(
            Span::new(start, end),
            specifiers,
            source,
            attributes,
            phase,
            ImportOrExportKind::Value,
        ))
    }

    fn parse_import_clause(&mut self) -> ParseResult<TypedSubRange<ImportDeclarationSpecifier>> {
        let mut specifiers = Vec::new();
        if self.current_token.tag == TokenTag::Star {
            let specifier = self.parse_import_namespace_specifier()?;
            specifiers.push(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                specifier,
            ));
            return Ok(self.ast.add_typed_sub_range(specifiers));
        }
        if self.current_token.tag == TokenTag::LeftBrace {
            return self.parse_named_imports();
        }

        let specifier = self.parse_import_default_specifier()?;
        specifiers.push(ImportDeclarationSpecifier::ImportDefaultSpecifier(
            specifier,
        ));
        if self.eat(TokenTag::Comma)? {
            if self.current_token.tag == TokenTag::Star {
                let specifier = self.parse_import_namespace_specifier()?;
                specifiers.push(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                    specifier,
                ));
            } else if self.current_token.tag == TokenTag::LeftBrace {
                let named = self.parse_named_imports()?;
                specifiers.extend(named.iter().map(|id| self.ast.get_node_in_sub_range(id)));
            } else {
                return self.error("expected namespace or named imports after ','");
            }
        }
        Ok(self.ast.add_typed_sub_range(specifiers))
    }

    fn parse_import_default_specifier(&mut self) -> ParseResult<ImportDefaultSpecifier> {
        let start = self.current_token.span.start;
        let local = self.parse_imported_binding()?;
        Ok(self
            .ast
            .import_default_specifier(Span::new(start, local.span(self.ast).end), local))
    }

    fn parse_import_namespace_specifier(&mut self) -> ParseResult<ImportNamespaceSpecifier> {
        let start = self.expect(TokenTag::Star)?.span.start;
        self.expect(TokenTag::As)?;
        let local = self.parse_imported_binding()?;
        Ok(self
            .ast
            .import_namespace_specifier(Span::new(start, local.span(self.ast).end), local))
    }

    fn parse_named_imports(&mut self) -> ParseResult<TypedSubRange<ImportDeclarationSpecifier>> {
        self.expect(TokenTag::LeftBrace)?;
        let mut specifiers = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace
            && self.current_token.tag != TokenTag::Eof
        {
            specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(
                self.parse_import_specifier()?,
            ));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        self.expect(TokenTag::RightBrace)?;
        Ok(self.ast.add_typed_sub_range(specifiers))
    }

    fn parse_import_specifier(&mut self) -> ParseResult<ImportSpecifier> {
        let start = self.current_token.span.start;
        let parts = self.parse_specifier_parts()?;
        let imported = parts.property_name.unwrap_or(parts.name);
        let local_name = parts.name;
        let local_span = local_name.span(self.ast);
        let Some(identifier) = local_name.as_identifier_name(self.ast) else {
            return self.error_at(local_span, "import local binding must be an identifier");
        };
        let name = identifier.name(self.ast);
        self.validate_identifier(parts.name_token, name.as_str(), "an imported binding")?;
        let decorators = self.empty_range();
        let local = self
            .ast
            .binding_identifier(local_span, name, decorators, None, false, None);
        Ok(self.ast.import_specifier(
            Span::new(start, local_span.end),
            imported,
            local,
            parts.kind,
        ))
    }

    fn parse_specifier_parts(&mut self) -> ParseResult<SpecifierParts> {
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
            && first.is_identifier_name(self.ast)
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
        Ok(parts)
    }

    fn parse_type_specifier_tail(
        &mut self,
        parts: &mut SpecifierParts,
        first: ModuleExportName,
    ) -> ParseResult<TypeTailResult> {
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
                return Ok(TypeTailResult::ConsumedRename);
            }
            if self.can_start_module_export_name() {
                parts.property_name = Some(first);
                parts.name_token = self.current_token;
                parts.name = self.parse_module_export_name()?;
                return Ok(TypeTailResult::ConsumedRename);
            }
            parts.kind = ImportOrExportKind::Type;
            parts.name = first_as;
            parts.name_token = first_as_token;
            return Ok(TypeTailResult::KeepOuterRename);
        }
        if self.can_start_module_export_name() {
            parts.kind = ImportOrExportKind::Type;
            parts.name_token = self.current_token;
            parts.name = self.parse_module_export_name()?;
        }
        Ok(TypeTailResult::KeepOuterRename)
    }

    fn can_start_module_export_name(&self) -> bool {
        self.current_token.tag.is_identifier_like()
            || self.current_token.tag == TokenTag::StringLiteral
    }

    fn parse_imported_binding(&mut self) -> ParseResult<BindingIdentifier> {
        self.parse_binding_identifier()
    }

    fn parse_module_specifier(&mut self) -> ParseResult<StringLiteral> {
        if self.current_token.tag != TokenTag::StringLiteral {
            return self.error("expected a module specifier");
        }
        self.parse_string_literal()
    }

    fn parse_import_equals_body(
        &mut self,
        start: u32,
        import_kind: ImportOrExportKind,
    ) -> ParseResult<Stmt> {
        let id = self.parse_binding_identifier()?;
        self.expect(TokenTag::Assign)?;
        let module_reference = self.parse_module_reference()?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self.ast.stmt_ts_import_equals_declaration(
            Span::new(start, end),
            id,
            module_reference,
            import_kind,
        ))
    }

    fn parse_module_reference(&mut self) -> ParseResult<TsModuleReference> {
        if self.current_token.tag == TokenTag::Require
            && self.peek_ahead()?.tag == TokenTag::LeftParen
        {
            return self.parse_external_module_reference();
        }

        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            return self.error("expected a module reference");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        if self.current_token.tag != TokenTag::Dot {
            return Ok(self
                .ast
                .ts_module_reference_ts_type_name_identifier_reference(token.span, name, None));
        }

        let mut left = self
            .ast
            .ts_qualified_name_left_identifier_reference(token.span, name, None);
        let start = token.span.start;
        loop {
            self.expect(TokenTag::Dot)?;
            let right_token = self.current_token;
            if !right_token.tag.is_identifier_like() {
                return self.error("expected an identifier after '.'");
            }
            let right_name = self.identifier_name(right_token)?;
            self.advance()?;
            let right = self.ast.identifier_name(right_token.span, right_name);
            let qualified =
                self.ast
                    .ts_qualified_name(Span::new(start, right_token.span.end), left, right);
            if self.current_token.tag != TokenTag::Dot {
                return Ok(TsModuleReference::TsTypeName(TsTypeName::TsQualifiedName(
                    qualified,
                )));
            }
            left = TsQualifiedNameLeft::TsQualifiedName(qualified);
        }
    }

    fn parse_external_module_reference(&mut self) -> ParseResult<TsModuleReference> {
        let start = self.expect(TokenTag::Require)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let expression = self.parse_string_literal()?;
        let end = self.expect(TokenTag::RightParen)?.span.end;
        Ok(self
            .ast
            .ts_module_reference_ts_external_module_reference(Span::new(start, end), expression))
    }

    fn parse_export_declaration(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Export)?.span.start;
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

    fn parse_ts_export_assignment(&mut self, start: u32) -> ParseResult<Stmt> {
        self.expect(TokenTag::Assign)?;
        let expression =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self
            .ast
            .stmt_ts_export_assignment(Span::new(start, end), expression))
    }

    fn parse_ts_namespace_export_declaration(&mut self, start: u32) -> ParseResult<Stmt> {
        self.expect(TokenTag::As)?;
        self.expect(TokenTag::Namespace)?;
        self.parse_namespace_export_declaration(start)
    }

    fn parse_namespace_export_declaration(&mut self, start: u32) -> ParseResult<Stmt> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            return self.error("expected an identifier after 'export as namespace'");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        let id = self.ast.identifier_name(token.span, name);
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self
            .ast
            .stmt_ts_namespace_export_declaration(Span::new(start, end), id))
    }

    fn parse_export_default_declaration(&mut self, start: u32) -> ParseResult<Stmt> {
        self.expect(TokenTag::Default)?;
        let part = self.parse_export_default_part()?;
        let declaration_end = part.declaration.span(self.ast).end;
        let end = if part.needs_semi {
            self.eat_semicolon(declaration_end)?
        } else {
            declaration_end
        };
        Ok(self
            .ast
            .stmt_export_default_declaration(Span::new(start, end), part.declaration))
    }

    fn parse_export_default_part(&mut self) -> ParseResult<DefaultExportPart> {
        let (declaration, needs_semicolon) = if self.current_token.tag == TokenTag::Function {
            let function = self.parse_function(
                ParseFunctionOpts {
                    is_default_export: true,
                    ..ParseFunctionOpts::default()
                },
                None,
            )?;
            (ExportDefaultDeclarationKind::Function(function), false)
        } else if self.current_token.tag == TokenTag::Async
            && !self.current_token.has_line_terminator_before()
        {
            let function_start = self.current_token.span.start;
            self.expect(TokenTag::Async)?;
            if self.current_token.tag == TokenTag::Function {
                let function = self.parse_function(
                    ParseFunctionOpts {
                        is_async: true,
                        is_default_export: true,
                        ..ParseFunctionOpts::default()
                    },
                    Some(function_start),
                )?;
                (ExportDefaultDeclarationKind::Function(function), false)
            } else {
                let span = Span::new(function_start, function_start + 5);
                let name = self.atom_from_span(span);
                let expression = self.ast.expr_identifier_reference(span, name, None);
                (ExportDefaultDeclarationKind::Expr(expression), true)
            }
        } else if self.current_token.tag == TokenTag::Class {
            let class = self.parse_class(
                ParseClassOpts {
                    is_default_export: true,
                    ..ParseClassOpts::default()
                },
                None,
            )?;
            (ExportDefaultDeclarationKind::Class(class), false)
        } else if self.current_token.tag == TokenTag::At {
            let (decorators_start, decorators) = self.parse_decorators()?;
            let class = self.parse_class_decorated(
                ParseClassOpts {
                    is_default_export: true,
                    ..ParseClassOpts::default()
                },
                Some(decorators_start),
                decorators,
            )?;
            (ExportDefaultDeclarationKind::Class(class), false)
        } else if self.lang.is_typescript()
            && self.current_token.tag == TokenTag::Abstract
            && self.is_abstract_class_next()?
        {
            let class_start = self.current_token.span.start;
            self.advance()?;
            let class = self.parse_class(
                ParseClassOpts {
                    is_default_export: true,
                    is_abstract: true,
                    ..ParseClassOpts::default()
                },
                Some(class_start),
            )?;
            (ExportDefaultDeclarationKind::Class(class), false)
        } else if self.lang.is_typescript() && self.current_token.tag == TokenTag::Interface {
            let interface_start = self.current_token.span.start;
            let statement = self.parse_interface_declaration(interface_start, false)?;
            let StmtData::Declaration(declaration) = self.ast.stmt_data(statement) else {
                unreachable!()
            };
            let DeclData::TsInterfaceDeclaration(interface) = self.ast.decl_data(declaration)
            else {
                unreachable!()
            };
            (
                ExportDefaultDeclarationKind::TsInterfaceDeclaration(interface),
                false,
            )
        } else {
            (
                ExportDefaultDeclarationKind::Expr(
                    self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?,
                ),
                true,
            )
        };
        Ok(DefaultExportPart {
            declaration,
            needs_semi: needs_semicolon,
        })
    }

    fn is_abstract_class_next(&mut self) -> ParseResult<bool> {
        let next = self.peek_ahead()?;
        Ok(next.tag == TokenTag::Class && !next.has_line_terminator_before())
    }

    fn is_legacy_accessibility_import(&mut self) -> ParseResult<bool> {
        let next = self.peek_ahead()?;
        Ok(next.tag == TokenTag::Import && !next.has_line_terminator_before())
    }

    fn parse_export_all_declaration(
        &mut self,
        start: u32,
        export_kind: ImportOrExportKind,
    ) -> ParseResult<Stmt> {
        self.expect(TokenTag::Star)?;
        let exported = if self.eat(TokenTag::As)? {
            Some(self.parse_module_export_name()?)
        } else {
            None
        };
        self.expect(TokenTag::From)?;
        let source = self.parse_module_specifier()?;
        let attributes = self.parse_with_clause()?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self.ast.stmt_export_all_declaration(
            Span::new(start, end),
            exported,
            source,
            attributes,
            export_kind,
        ))
    }

    fn parse_export_with_declaration(&mut self, start: u32) -> ParseResult<Stmt> {
        if self.lang.is_typescript()
            && matches!(
                self.current_token.tag,
                TokenTag::Public | TokenTag::Private | TokenTag::Static
            )
            && self.is_legacy_accessibility_import()?
        {
            return self.error(
                "legacy accessibility import exports cannot be represented by the current AST",
            );
        }
        if self.lang.is_typescript() && self.is_start_of_ts_declaration()? {
            let statement = self.parse_ts_declaration()?;
            let StmtData::Declaration(declaration) = self.ast.stmt_data(statement) else {
                return self.error_at(
                    statement.span(self.ast),
                    "this TypeScript declaration cannot be represented inside an export declaration",
                );
            };
            let export_kind = self.export_kind_for_declaration(declaration);
            let end = declaration.span(self.ast).end;
            let specifiers = self.empty_range();
            let attributes = self.empty_range();
            return Ok(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(declaration),
                specifiers,
                None,
                attributes,
                export_kind,
            ));
        }
        if matches!(
            self.current_token.tag,
            TokenTag::Var | TokenTag::Let | TokenTag::Const
        ) {
            let declaration =
                self.parse_variable_declaration(ParseVariableDeclarationOpts::default(), None)?;
            let end = declaration.span(self.ast).end;
            let specifiers = self.empty_range();
            let attributes = self.empty_range();
            return Ok(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(Decl::VariableDeclaration(declaration)),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ));
        }
        if self.current_token.tag == TokenTag::Function
            || (self.current_token.tag == TokenTag::Async && {
                let next = self.peek_ahead()?;
                next.tag == TokenTag::Function && !next.has_line_terminator_before()
            })
            || matches!(self.current_token.tag, TokenTag::Class | TokenTag::At)
        {
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
                    self.expect(TokenTag::Async)?;
                    Some(start)
                } else {
                    None
                };
                Decl::Function(self.parse_function(
                    ParseFunctionOpts {
                        is_async,
                        ..ParseFunctionOpts::default()
                    },
                    function_start,
                )?)
            };
            let end = declaration.span(self.ast).end;
            let specifiers = self.empty_range();
            let attributes = self.empty_range();
            return Ok(self.ast.stmt_export_named_declaration(
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

    fn report_missing_export_declaration(&self) -> ParseResult<Stmt> {
        self.error("expected a declaration after 'export'")
    }

    fn export_kind_for_declaration(&self, declaration: Decl) -> ImportOrExportKind {
        match self.ast.decl_data(declaration) {
            DeclData::TsInterfaceDeclaration(_) | DeclData::TsTypeAliasDeclaration(_) => {
                ImportOrExportKind::Type
            }
            DeclData::TsEnumDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(self.ast))
            }
            DeclData::TsModuleDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(self.ast))
            }
            DeclData::TsGlobalDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(self.ast))
            }
            DeclData::VariableDeclaration(node) => {
                import_or_export_kind_for_declare(node.declare(self.ast))
            }
            DeclData::Function(node) => import_or_export_kind_for_declare(node.declare(self.ast)),
            DeclData::Class(node) => import_or_export_kind_for_declare(node.declare(self.ast)),
        }
    }

    fn parse_export_named_from_clause(
        &mut self,
        start: u32,
        export_kind: ImportOrExportKind,
    ) -> ParseResult<Stmt> {
        let result = self.parse_export_specifiers()?;
        let source = if self.eat(TokenTag::From)? {
            Some(self.parse_module_specifier()?)
        } else {
            self.resolve_local_export_specifiers(&result)?;
            None
        };
        let attributes = self.parse_with_clause()?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        Ok(self.ast.stmt_export_named_declaration(
            Span::new(start, end),
            None,
            result.specifiers,
            source,
            attributes,
            export_kind,
        ))
    }

    fn parse_export_specifiers(&mut self) -> ParseResult<ExportSpecifiersResult> {
        self.expect(TokenTag::LeftBrace)?;
        let mut specifiers = Vec::new();
        let mut local_tags = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace
            && self.current_token.tag != TokenTag::Eof
        {
            local_tags.push(self.current_token.tag);
            specifiers.push(self.parse_export_specifier()?);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        self.expect(TokenTag::RightBrace)?;
        Ok(ExportSpecifiersResult {
            specifiers: self.ast.add_typed_sub_range(specifiers),
            local_tags,
        })
    }

    fn parse_export_specifier(&mut self) -> ParseResult<ExportSpecifier> {
        let start = self.current_token.span.start;
        let parts = self.parse_specifier_parts()?;
        let local = parts.property_name.unwrap_or(parts.name);
        let exported = parts.name;
        Ok(self.ast.export_specifier(
            Span::new(start, exported.span(self.ast).end),
            local,
            exported,
            parts.kind,
        ))
    }

    fn resolve_local_export_specifiers(&self, result: &ExportSpecifiersResult) -> ParseResult<()> {
        for (index, local_tag) in result.local_tags.iter().copied().enumerate() {
            let specifier = result
                .specifiers
                .get_node(self.ast, index)
                .expect("export specifier range contains its nodes");
            let local = specifier.local(self.ast);
            if local.is_string_literal(self.ast) {
                return self.error_at(
                    local.span(self.ast),
                    "a string export binding requires a source module",
                );
            }
            if local_tag.is_reserved() {
                return self.error_at(
                    local.span(self.ast),
                    "a reserved word cannot be a local export binding",
                );
            }
        }
        Ok(())
    }

    fn parse_with_clause(&mut self) -> ParseResult<TypedSubRange<ImportAttribute>> {
        if !matches!(self.current_token.tag, TokenTag::With | TokenTag::Assert) {
            return Ok(self.empty_range());
        }
        self.advance()?;
        self.expect(TokenTag::LeftBrace)?;
        let mut attributes = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            attributes.push(self.parse_import_attribute()?);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        self.expect(TokenTag::RightBrace)?;
        Ok(self.ast.add_typed_sub_range(attributes))
    }

    fn parse_import_attribute(&mut self) -> ParseResult<ImportAttribute> {
        let start = self.current_token.span.start;
        let key = self.parse_attribute_key()?;
        self.expect(TokenTag::Colon)?;
        let value = self.parse_string_literal()?;
        Ok(self
            .ast
            .import_attribute(Span::new(start, value.span(self.ast).end), key, value))
    }

    fn parse_attribute_key(&mut self) -> ParseResult<ModuleExportName> {
        self.parse_module_export_name()
    }

    fn parse_module_export_name(&mut self) -> ParseResult<ModuleExportName> {
        let token = self.current_token;
        if token.tag == TokenTag::StringLiteral {
            if token.has_lone_surrogates() {
                return self.error_at(
                    token.span,
                    "an import or export name cannot contain a lone surrogate",
                );
            }
            let raw = self.atom_from_span(token.span);
            let value = self.string_value(token)?;
            self.advance()?;
            return Ok(self
                .ast
                .module_export_name_string_literal(token.span, value, raw));
        }
        if token.tag.is_identifier_like() {
            let name = self.identifier_name(token)?;
            self.advance()?;
            return Ok(self
                .ast
                .module_export_name_identifier_name(token.span, name));
        }
        self.error("expected an import or export name")
    }

    fn is_start_of_ts_declaration(&mut self) -> ParseResult<bool> {
        if !self.lang.is_typescript() {
            return Ok(false);
        }
        let mut peek = self.begin_peek();
        let tokens = (|| {
            let mut tokens = Vec::with_capacity(4);
            for _ in 0..4 {
                tokens.push(peek.next()?);
            }
            Ok(tokens)
        })();
        peek.end();
        let tokens = tokens?;
        let mut current = self.current_token;
        let mut index = 0usize;
        let mut has_declare = false;
        let mut has_abstract = false;

        if current.tag == TokenTag::Declare {
            current = tokens[index];
            index += 1;
            if current.has_line_terminator_before() {
                return Ok(false);
            }
            has_declare = true;
        }
        if current.tag == TokenTag::Abstract {
            let next = tokens[index];
            index += 1;
            if next.has_line_terminator_before() || next.tag != TokenTag::Class {
                return Ok(false);
            }
            current = next;
            has_abstract = true;
        }
        if current.tag == TokenTag::Const {
            let next = tokens[index];
            if is_const_enum_head(next) {
                current = next;
                index += 1;
            } else if has_declare {
                return Ok(!next.has_line_terminator_before() && can_start_binding(next.tag));
            } else {
                return Ok(false);
            }
        }

        let result = match current.tag {
            TokenTag::Type | TokenTag::Interface | TokenTag::Enum | TokenTag::Namespace => {
                is_declaration_name(tokens[index])
            }
            TokenTag::Module => {
                let name = tokens[index];
                !name.has_line_terminator_before()
                    && (is_declaration_name(name) || name.tag == TokenTag::StringLiteral)
            }
            TokenTag::Global => {
                let next = tokens[index];
                next.tag == TokenTag::LeftBrace && !next.has_line_terminator_before()
            }
            TokenTag::Var | TokenTag::Let | TokenTag::Function | TokenTag::Class => {
                if !has_declare && !has_abstract {
                    false
                } else {
                    let name = tokens[index];
                    !name.has_line_terminator_before()
                        && match current.tag {
                            TokenTag::Var | TokenTag::Let => can_start_binding(name.tag),
                            _ => is_declaration_name(name),
                        }
                }
            }
            TokenTag::Async if has_declare => {
                let function = tokens[index];
                let name = tokens[index + 1];
                function.tag == TokenTag::Function
                    && !function.has_line_terminator_before()
                    && is_declaration_name(name)
            }
            TokenTag::Import if has_declare => {
                let name = tokens[index];
                let assign = tokens[index + 1];
                is_declaration_name(name)
                    && assign.tag == TokenTag::Assign
                    && !assign.has_line_terminator_before()
            }
            _ => false,
        };
        Ok(result)
    }

    fn parse_ts_declaration(&mut self) -> ParseResult<Stmt> {
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

        match self.current_token.tag {
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
                Ok(Stmt::Declaration(Decl::VariableDeclaration(declaration)))
            }
            TokenTag::Function => Ok(Stmt::Declaration(Decl::Function(self.parse_function(
                ParseFunctionOpts {
                    is_declare: modifiers.declare,
                    ..ParseFunctionOpts::default()
                },
                Some(start),
            )?))),
            TokenTag::Async => {
                self.advance()?;
                Ok(Stmt::Declaration(Decl::Function(self.parse_function(
                    ParseFunctionOpts {
                        is_async: true,
                        is_declare: modifiers.declare,
                        ..ParseFunctionOpts::default()
                    },
                    Some(start),
                )?)))
            }
            TokenTag::Class => Ok(Stmt::Declaration(Decl::Class(self.parse_class(
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
            _ => self.error("expected a TypeScript declaration"),
        }
    }

    fn parse_declare_import_equals(&mut self, start: u32) -> ParseResult<Stmt> {
        self.advance()?;
        self.parse_import_equals_body(start, ImportOrExportKind::Value)
    }

    fn parse_type_alias_declaration(&mut self, start: u32, declare: bool) -> ParseResult<Stmt> {
        self.expect(TokenTag::Type)?;
        let id = self.parse_binding_identifier()?;
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        self.expect(TokenTag::Assign)?;
        let type_annotation = self.parse_type_alias_body()?;
        let end = self.eat_semicolon(self.prev_token_end)?;
        let declaration = self.ast.ts_type_alias_declaration(
            Span::new(start, end),
            id,
            type_parameters,
            type_annotation,
            declare,
        );
        Ok(Stmt::Declaration(Decl::TsTypeAliasDeclaration(declaration)))
    }

    fn parse_interface_declaration(&mut self, start: u32, declare: bool) -> ParseResult<Stmt> {
        self.expect(TokenTag::Interface)?;
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
        let end = body.span(self.ast).end;
        let declaration = self.ast.ts_interface_declaration(
            Span::new(start, end),
            id,
            type_parameters,
            extends,
            body,
            declare,
        );
        Ok(Stmt::Declaration(Decl::TsInterfaceDeclaration(declaration)))
    }

    fn parse_heritage_clause(
        &mut self,
        keyword: TokenTag,
        kind: HeritageKind,
    ) -> ParseResult<HeritageClause> {
        if !self.eat(keyword)? {
            return Ok(match kind {
                HeritageKind::Interface => HeritageClause::Interface(self.empty_range()),
                HeritageKind::Class => HeritageClause::Class(self.empty_range()),
            });
        }
        let mut entries = Vec::new();
        loop {
            let node = match self.parse_heritage_entry(kind)? {
                HeritageEntry::Interface(node) => node.node_id(),
                HeritageEntry::Class(node) => node.node_id(),
            };
            entries.push(node);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        Ok(match kind {
            HeritageKind::Interface => HeritageClause::Interface(
                self.ast.add_typed_sub_range(
                    entries
                        .into_iter()
                        .map(|node| unsafe { TsInterfaceHeritage::from_node_id_unchecked(node) }),
                ),
            ),
            HeritageKind::Class => HeritageClause::Class(
                self.ast.add_typed_sub_range(
                    entries
                        .into_iter()
                        .map(|node| unsafe { TsClassImplements::from_node_id_unchecked(node) }),
                ),
            ),
        })
    }

    fn parse_heritage_entry(&mut self, kind: HeritageKind) -> ParseResult<HeritageEntry> {
        let expression = self.parse_heritage_expression()?;
        let type_arguments = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_arguments()?)
        } else {
            None
        };
        let end = type_arguments.map_or(expression.span(self.ast).end, |arguments| {
            arguments.span(self.ast).end
        });
        let span = Span::new(expression.span(self.ast).start, end);
        Ok(match kind {
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

    fn parse_interface_body(&mut self) -> ParseResult<TsInterfaceBody> {
        let start = self.current_token.span.start;
        let (body, end) = self.parse_object_type_members()?;
        Ok(self.ast.ts_interface_body(Span::new(start, end), body))
    }

    fn parse_enum_declaration(
        &mut self,
        start: u32,
        is_const: bool,
        declare: bool,
    ) -> ParseResult<Stmt> {
        self.expect(TokenTag::Enum)?;
        let id = self.parse_binding_identifier()?;
        let body = self.parse_enum_body()?;
        let end = body.span(self.ast).end;
        let declaration =
            self.ast
                .ts_enum_declaration(Span::new(start, end), id, body, is_const, declare);
        Ok(Stmt::Declaration(Decl::TsEnumDeclaration(declaration)))
    }

    fn parse_enum_body(&mut self) -> ParseResult<TsEnumBody> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut members = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace
            && self.current_token.tag != TokenTag::Eof
        {
            members.push(self.parse_enum_member()?);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let members = self.ast.add_typed_sub_range(members);
        Ok(self.ast.ts_enum_body(Span::new(start, end), members))
    }

    fn parse_enum_member(&mut self) -> ParseResult<TsEnumMember> {
        let (id, computed) = self.parse_enum_member_name()?;
        let start = id.span(self.ast).start;
        let initializer = if self.eat(TokenTag::Assign)? {
            Some(self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?)
        } else {
            None
        };
        let end = initializer.map_or_else(
            || {
                if computed {
                    self.prev_token_end
                } else {
                    id.span(self.ast).end
                }
            },
            |expression| expression.span(self.ast).end,
        );
        Ok(self
            .ast
            .ts_enum_member(Span::new(start, end), id, initializer, computed))
    }

    fn parse_enum_member_name(&mut self) -> ParseResult<(TsEnumMemberName, bool)> {
        let token = self.current_token;
        if self.eat(TokenTag::LeftBracket)? {
            let expression =
                self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
            self.expect(TokenTag::RightBracket)?;
            return Ok((TsEnumMemberName::Expr(expression), true));
        }
        if token.tag.is_identifier_like() {
            let name = self.identifier_name(token)?;
            self.advance()?;
            return Ok((
                self.ast
                    .ts_enum_member_name_identifier_name(token.span, name),
                false,
            ));
        }
        if token.tag == TokenTag::StringLiteral {
            let literal = self.parse_string_literal()?;
            return Ok((TsEnumMemberName::StringLiteral(literal), false));
        }
        if token.tag == TokenTag::NoSubstitutionTemplate {
            let literal = self.parse_no_substitution_template(false)?;
            return Ok((TsEnumMemberName::TemplateLiteral(literal), false));
        }
        self.error("expected an enum member name")
    }

    fn parse_module_declaration(
        &mut self,
        start: u32,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> ParseResult<Stmt> {
        self.advance()?;
        let id = if kind == TsModuleDeclarationKind::Module
            && self.current_token.tag == TokenTag::StringLiteral
        {
            TsModuleName::StringLiteral(self.parse_string_literal()?)
        } else {
            self.parse_module_name()?
        };
        let body = self.parse_optional_module_block()?;
        let end = if let Some(body) = body {
            body.span(self.ast).end
        } else {
            self.eat_semicolon(self.prev_token_end)?
        };
        let declaration =
            self.ast
                .ts_module_declaration(Span::new(start, end), id, body, kind, declare);
        Ok(Stmt::Declaration(Decl::TsModuleDeclaration(declaration)))
    }

    fn parse_global_declaration(&mut self, start: u32, declare: bool) -> ParseResult<Stmt> {
        let token = self.expect(TokenTag::Global)?;
        let name = self.identifier_name(token)?;
        let id = self.ast.identifier_name(token.span, name);
        let body = self.parse_module_block()?;
        let end = body.span(self.ast).end;
        let declaration = self
            .ast
            .ts_global_declaration(Span::new(start, end), id, body, declare);
        Ok(Stmt::Declaration(Decl::TsGlobalDeclaration(declaration)))
    }

    fn parse_module_name(&mut self) -> ParseResult<TsModuleName> {
        let head = self.parse_binding_identifier()?;
        if self.current_token.tag != TokenTag::Dot {
            return Ok(TsModuleName::BindingIdentifier(head));
        }
        let start = head.span(self.ast).start;
        let left = TsQualifiedNameLeft::BindingIdentifier(head);
        let qualified = self.extend_qualified_name(left, start)?;
        Ok(TsModuleName::TsQualifiedName(
            qualified
                .as_ts_qualified_name(self.ast)
                .expect("a dotted module name is qualified"),
        ))
    }

    fn parse_optional_module_block(&mut self) -> ParseResult<Option<TsModuleBody>> {
        if self.current_token.tag != TokenTag::LeftBrace {
            return Ok(None);
        }
        Ok(Some(TsModuleBody::TsModuleBlock(
            self.parse_module_block()?,
        )))
    }

    fn parse_module_block(&mut self) -> ParseResult<TsModuleBlock> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let body = self.parse_body(Some(TokenTag::RightBrace), BodyKind::ModuleBlock)?;
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        Ok(self.ast.ts_module_block(Span::new(start, end), body))
    }

    fn parse_type_alias_body(&mut self) -> ParseResult<TsType> {
        if self.current_token.tag == TokenTag::Intrinsic && !self.current_token.is_escaped() {
            let next = self.peek_ahead()?;
            if !continues_type(next.tag) {
                let span = self.current_token.span;
                self.advance()?;
                return Ok(self.ast.ts_type_ts_intrinsic_keyword(span));
            }
        }
        self.parse_type()
    }

    fn parse_type_annotation(&mut self) -> ParseResult<TsTypeAnnotation> {
        let start = self.expect(TokenTag::Colon)?.span.start;
        let type_annotation = self.parse_type()?;
        let end = type_annotation.span(self.ast).end;
        Ok(self
            .ast
            .ts_type_annotation(Span::new(start, end), type_annotation))
    }

    fn parse_return_type_annotation(&mut self) -> ParseResult<TsTypeAnnotation> {
        let start = self.expect(TokenTag::Colon)?.span.start;
        let type_annotation = self.parse_type_or_type_predicate()?;
        let end = type_annotation.span(self.ast).end;
        Ok(self
            .ast
            .ts_type_annotation(Span::new(start, end), type_annotation))
    }

    fn parse_type_or_type_predicate(&mut self) -> ParseResult<TsType> {
        if !self.is_identifier_predicate_start()? {
            return self.parse_type();
        }
        let token = self.current_token;
        let name = self.identifier_name(token)?;
        self.advance()?;
        let parameter_name = self
            .ast
            .ts_type_predicate_name_identifier_name(token.span, name);
        self.finish_type_predicate(token.span.start, parameter_name, false)
    }

    fn parse_this_type_or_predicate(&mut self) -> ParseResult<TsType> {
        let token = self.expect(TokenTag::This)?;
        if self.current_token.tag != TokenTag::Is
            || self.current_token.is_escaped()
            || self.current_token.has_line_terminator_before()
        {
            return Ok(self.ast.ts_type_ts_this_type(token.span));
        }
        let parameter_name = self.ast.ts_type_predicate_name_ts_this_type(token.span);
        self.finish_type_predicate(token.span.start, parameter_name, false)
    }

    fn parse_asserts_type_predicate(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::Asserts)?.span.start;
        let parameter_name = self.parse_predicate_parameter_name()?;
        self.finish_type_predicate(start, parameter_name, true)
    }

    fn parse_predicate_parameter_name(&mut self) -> ParseResult<TsTypePredicateName> {
        let token = self.current_token;
        if token.tag == TokenTag::This {
            self.advance()?;
            return Ok(self.ast.ts_type_predicate_name_ts_this_type(token.span));
        }
        if !token.tag.is_identifier_like() {
            return self.error("expected a type predicate parameter name");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        Ok(self
            .ast
            .ts_type_predicate_name_identifier_name(token.span, name))
    }

    fn finish_type_predicate(
        &mut self,
        start: u32,
        parameter_name: TsTypePredicateName,
        asserts: bool,
    ) -> ParseResult<TsType> {
        let mut end = parameter_name.span(self.ast).end;
        let type_annotation =
            if self.current_token.tag == TokenTag::Is && !self.current_token.is_escaped() {
                self.advance()?;
                let inner = self.parse_type()?;
                end = inner.span(self.ast).end;
                Some(self.ast.ts_type_annotation(inner.span(self.ast), inner))
            } else {
                None
            };
        Ok(TsType::TsTypePredicate(self.ast.ts_type_predicate(
            Span::new(start, end),
            parameter_name,
            type_annotation,
            asserts,
        )))
    }

    fn is_asserts_predicate_start(&mut self) -> ParseResult<bool> {
        if self.current_token.tag != TokenTag::Asserts || self.current_token.is_escaped() {
            return Ok(false);
        }
        let next = self.peek_ahead()?;
        Ok(!next.has_line_terminator_before()
            && (next.tag == TokenTag::This || next.tag.is_identifier_like()))
    }

    fn is_identifier_predicate_start(&mut self) -> ParseResult<bool> {
        let current = self.current_token;
        if current.is_escaped()
            || current.tag == TokenTag::This
            || !current.tag.is_identifier_like()
        {
            return Ok(false);
        }
        let next = self.peek_ahead()?;
        Ok(next.tag == TokenTag::Is && !next.is_escaped() && !next.has_line_terminator_before())
    }

    fn parse_type(&mut self) -> ParseResult<TsType> {
        let saved = self.ts_context.disallow_conditional_types;
        self.ts_context.disallow_conditional_types = false;
        let result = (|| {
            if self.is_start_of_function_or_constructor_type()? {
                self.parse_function_or_constructor_type()
            } else {
                self.parse_conditional_type()
            }
        })();
        self.ts_context.disallow_conditional_types = saved;
        result
    }

    fn parse_type_no_conditional(&mut self) -> ParseResult<TsType> {
        if self.is_start_of_function_or_constructor_type()? {
            self.parse_function_or_constructor_type()
        } else {
            self.parse_union_type()
        }
    }

    fn parse_conditional_type(&mut self) -> ParseResult<TsType> {
        let check_type = self.parse_union_type()?;
        if self.ts_context.disallow_conditional_types
            || self.current_token.tag != TokenTag::Extends
            || self.current_token.has_line_terminator_before()
        {
            return Ok(check_type);
        }
        self.advance()?;
        let saved = self.ts_context.disallow_conditional_types;
        self.ts_context.disallow_conditional_types = true;
        let extends_type = self.parse_type_no_conditional();
        self.ts_context.disallow_conditional_types = saved;
        let extends_type = extends_type?;
        self.expect(TokenTag::Question)?;
        let true_type = self.parse_type()?;
        self.expect(TokenTag::Colon)?;
        let false_type = self.parse_type()?;
        Ok(TsType::TsConditionalType(self.ast.ts_conditional_type(
            Span::new(
                check_type.span(self.ast).start,
                false_type.span(self.ast).end,
            ),
            check_type,
            extends_type,
            true_type,
            false_type,
        )))
    }

    fn parse_union_type(&mut self) -> ParseResult<TsType> {
        self.parse_binary_type_chain(TokenTag::BitwiseOr, BinaryTypeKind::Union)
    }

    fn parse_binary_type_chain(
        &mut self,
        separator: TokenTag,
        kind: BinaryTypeKind,
    ) -> ParseResult<TsType> {
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
            return Ok(first);
        }
        let mut types = vec![first];
        while self.eat(separator)? {
            types.push(match kind {
                BinaryTypeKind::Union => self.parse_intersection_type()?,
                BinaryTypeKind::Intersection => self.parse_postfix_type()?,
            });
        }
        let span = Span::new(
            leading_start.unwrap_or(first.span(self.ast).start),
            types.last().unwrap().span(self.ast).end,
        );
        let types = self.ast.add_typed_sub_range(types);
        Ok(match kind {
            BinaryTypeKind::Union => TsType::TsUnionType(self.ast.ts_union_type(span, types)),
            BinaryTypeKind::Intersection => {
                TsType::TsIntersectionType(self.ast.ts_intersection_type(span, types))
            }
        })
    }

    fn parse_intersection_type(&mut self) -> ParseResult<TsType> {
        self.parse_binary_type_chain(TokenTag::BitwiseAnd, BinaryTypeKind::Intersection)
    }

    fn parse_postfix_type(&mut self) -> ParseResult<TsType> {
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
        Ok(type_annotation)
    }

    fn parse_array_or_indexed_access_type(&mut self, object_type: TsType) -> ParseResult<TsType> {
        let start = object_type.span(self.ast).start;
        self.expect(TokenTag::LeftBracket)?;
        if self.current_token.tag == TokenTag::RightBracket {
            let end = self.expect(TokenTag::RightBracket)?.span.end;
            return Ok(TsType::TsArrayType(
                self.ast.ts_array_type(Span::new(start, end), object_type),
            ));
        }
        let index_type = self.parse_type()?;
        let end = self.expect(TokenTag::RightBracket)?.span.end;
        Ok(TsType::TsIndexedAccessType(
            self.ast
                .ts_indexed_access_type(Span::new(start, end), object_type, index_type),
        ))
    }

    fn parse_primary_type(&mut self) -> ParseResult<TsType> {
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
            if is_type_keyword(token.tag) && !self.is_qualified_type_continuation()? {
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
                    if self.is_start_of_mapped_type()? {
                        return self.parse_mapped_type();
                    }
                    return self.parse_type_literal();
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
        if (token.tag.is_identifier_like() && !token.tag.is_unconditionally_reserved())
            || token.tag == TokenTag::Const
        {
            return self.parse_type_reference();
        }
        self.error("expected a TypeScript type")
    }

    fn parse_type_keyword(&mut self) -> ParseResult<TsType> {
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
        Ok(keyword)
    }

    fn is_postfix_nullable(&mut self) -> ParseResult<bool> {
        let next = self.peek_ahead()?;
        Ok(!is_start_of_type(next.tag))
    }

    fn parse_jsdoc_postfix(&mut self, inner: TsType, kind: JsDocKind) -> ParseResult<TsType> {
        let span = Span::new(inner.span(self.ast).start, self.current_token.span.end);
        self.advance()?;
        Ok(self.jsdoc_node_data(inner, kind, true, span))
    }

    fn parse_parenthesized_type(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::LeftParen)?.span.start;
        let type_annotation = self.parse_type()?;
        let end = self.expect(TokenTag::RightParen)?.span.end;
        Ok(TsType::TsParenthesizedType(self.ast.ts_parenthesized_type(
            Span::new(start, end),
            type_annotation,
        )))
    }

    fn parse_tuple_type(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::LeftBracket)?.span.start;
        let mut elements = Vec::new();
        while self.current_token.tag != TokenTag::RightBracket
            && self.current_token.tag != TokenTag::Eof
        {
            elements.push(self.parse_tuple_element()?);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let end = self.expect(TokenTag::RightBracket)?.span.end;
        let elements = self.ast.add_typed_sub_range(elements);
        Ok(TsType::TsTupleType(
            self.ast.ts_tuple_type(Span::new(start, end), elements),
        ))
    }

    fn parse_tuple_element(&mut self) -> ParseResult<TsTupleElement> {
        if self.current_token.tag == TokenTag::Spread {
            let start = self.current_token.span.start;
            self.advance()?;
            let inner = self.parse_tuple_element_body()?;
            if let TsTupleElementData::TsNamedTupleMember(named) =
                self.ast.ts_tuple_element_data(inner)
            {
                let element_type = named.element_type(self.ast);
                let rest = self.ast.ts_rest_type(
                    Span::new(start, element_type.span(self.ast).end),
                    element_type,
                );
                let named = self.ast.ts_named_tuple_member(
                    Span::new(start, named.span(self.ast).end),
                    named.label(self.ast),
                    TsType::TsRestType(rest),
                    named.optional(self.ast),
                );
                return Ok(TsTupleElement::TsNamedTupleMember(named));
            }
            let inner_type = tuple_element_as_type(inner, self.ast);
            let span = Span::new(start, inner_type.span(self.ast).end);
            return Ok(TsTupleElement::TsRestType(
                self.ast.ts_rest_type(span, inner_type),
            ));
        }
        self.parse_tuple_element_body()
    }

    fn parse_tuple_element_body(&mut self) -> ParseResult<TsTupleElement> {
        if self.is_named_tuple_element()? {
            return self.parse_named_tuple_member();
        }
        let type_annotation = self.parse_type()?;
        if let TsTypeData::TsJsDocNullableType(nullable) = self.ast.ts_type_data(type_annotation)
            && nullable.postfix(self.ast)
        {
            return Ok(TsTupleElement::TsOptionalType(self.ast.ts_optional_type(
                type_annotation.span(self.ast),
                nullable.type_annotation(self.ast),
            )));
        }
        Ok(TsTupleElement::TsType(type_annotation))
    }

    fn is_named_tuple_element(&mut self) -> ParseResult<bool> {
        if !self.current_token.tag.is_identifier_like() {
            return Ok(false);
        }
        let first = self.peek_ahead()?;
        if first.tag == TokenTag::Colon {
            return Ok(true);
        }
        if first.tag != TokenTag::Question {
            return Ok(false);
        }
        let mut peek = self.begin_peek();
        let second = (|| {
            peek.next()?;
            peek.next()
        })();
        peek.end();
        Ok(second?.tag == TokenTag::Colon)
    }

    fn parse_named_tuple_member(&mut self) -> ParseResult<TsTupleElement> {
        let token = self.current_token;
        let name = self.identifier_name(token)?;
        self.advance()?;
        let label = self.ast.identifier_name(token.span, name);
        let optional = self.eat(TokenTag::Question)?;
        self.expect(TokenTag::Colon)?;
        let element_type = self.parse_type()?;
        let span = Span::new(token.span.start, element_type.span(self.ast).end);
        Ok(TsTupleElement::TsNamedTupleMember(
            self.ast
                .ts_named_tuple_member(span, label, element_type, optional),
        ))
    }

    fn parse_type_operator(&mut self) -> ParseResult<TsType> {
        let token = self.current_token;
        let operator = match token.tag {
            TokenTag::Keyof => TsTypeOperatorKind::Keyof,
            TokenTag::Unique => TsTypeOperatorKind::Unique,
            TokenTag::Readonly => TsTypeOperatorKind::Readonly,
            _ => unreachable!(),
        };
        self.advance()?;
        let type_annotation = self.parse_postfix_type()?;
        let span = Span::new(token.span.start, type_annotation.span(self.ast).end);
        Ok(TsType::TsTypeOperator(self.ast.ts_type_operator(
            span,
            operator,
            type_annotation,
        )))
    }

    fn parse_infer_type(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::Infer)?.span.start;
        let name_start = self.current_token.span.start;
        let name = self.parse_binding_identifier()?;
        let constraint = if self.eat(TokenTag::Extends)? {
            Some(self.parse_union_type()?)
        } else {
            None
        };
        let end = constraint.map_or(name.span(self.ast).end, |ty| ty.span(self.ast).end);
        let parameter = self.ast.ts_type_parameter(
            Span::new(name_start, end),
            name,
            constraint,
            None,
            false,
            false,
            false,
        );
        Ok(TsType::TsInferType(
            self.ast.ts_infer_type(Span::new(start, end), parameter),
        ))
    }

    fn is_start_of_function_or_constructor_type(&mut self) -> ParseResult<bool> {
        let tag = self.current_token.tag;
        if tag == TokenTag::New {
            return Ok(true);
        }
        if tag == TokenTag::Abstract {
            return Ok(self.peek_ahead()?.tag == TokenTag::New);
        }
        if is_angle_open(tag) {
            return Ok(true);
        }
        if tag != TokenTag::LeftParen {
            return Ok(false);
        }
        let mut peek = self.begin_peek();
        let first = peek.next()?;
        match first.tag {
            TokenTag::RightParen => {
                let result = peek.next()?.tag == TokenTag::Arrow;
                peek.end();
                return Ok(result);
            }
            TokenTag::Spread => {
                peek.end();
                return Ok(true);
            }
            TokenTag::This => {
                let result = peek.next()?.tag == TokenTag::Colon;
                peek.end();
                return Ok(result);
            }
            TokenTag::LeftBrace | TokenTag::LeftBracket => {
                peek.end();
                return self.is_function_type_after_pattern();
            }
            _ => {}
        }
        if !first.tag.is_identifier_like() {
            peek.end();
            return Ok(false);
        }
        let second = peek.next()?;
        let result = match second.tag {
            TokenTag::Colon | TokenTag::Question | TokenTag::Comma | TokenTag::Assign => true,
            TokenTag::RightParen => peek.next()?.tag == TokenTag::Arrow,
            _ => false,
        };
        peek.end();
        Ok(result)
    }

    fn is_function_type_after_pattern(&mut self) -> ParseResult<bool> {
        let checkpoint = self.checkpoint();
        let result = (|| {
            self.advance()?;
            if self.parse_binding_pattern().is_err() {
                return Ok(false);
            }
            Ok(match self.current_token.tag {
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

    fn parse_function_or_constructor_type(&mut self) -> ParseResult<TsType> {
        let start = self.current_token.span.start;
        let is_abstract = self.eat(TokenTag::Abstract)?;
        let is_constructor = self.eat(TokenTag::New)?;
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        let params = self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        let arrow_start = self.expect(TokenTag::Arrow)?.span.start;
        let return_type_inner = self.parse_type_or_type_predicate()?;
        let end = return_type_inner.span(self.ast).end;
        let return_type = self
            .ast
            .ts_type_annotation(Span::new(arrow_start, end), return_type_inner);
        Ok(if is_constructor {
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

    fn parse_type_query(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::Typeof)?.span.start;
        let expr_name = if self.current_token.tag == TokenTag::Import {
            let import_type = self.parse_import_type()?;
            let TsTypeData::TsImportType(import_type) = self.ast.ts_type_data(import_type) else {
                unreachable!()
            };
            TsTypeQueryExprName::TsImportType(import_type)
        } else if self.current_token.tag == TokenTag::This {
            let token = self.current_token;
            self.advance()?;
            if self.current_token.tag == TokenTag::Dot {
                let left = self.ast.ts_qualified_name_left_this_expression(token.span);
                TsTypeQueryExprName::TsTypeName(self.extend_qualified_name(left, token.span.start)?)
            } else {
                self.ast.ts_type_query_expr_name_this_expression(token.span)
            }
        } else {
            TsTypeQueryExprName::TsTypeName(self.parse_entity_name()?)
        };
        let type_arguments = if expr_name.is_ts_import_type(self.ast) {
            None
        } else {
            self.parse_type_arguments_after_entity_name()?
        };
        let end =
            type_arguments.map_or(expr_name.span(self.ast).end, |args| args.span(self.ast).end);
        Ok(TsType::TsTypeQuery(self.ast.ts_type_query(
            Span::new(start, end),
            expr_name,
            type_arguments,
        )))
    }

    fn parse_import_type(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::Import)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let source = self.parse_string_literal()?;
        let options = self.parse_import_type_options()?;
        self.expect(TokenTag::RightParen)?;
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
            || qualifier.map_or(self.prev_token_end, |name| name.span(self.ast).end),
            |args| args.span(self.ast).end,
        );
        Ok(TsType::TsImportType(self.ast.ts_import_type(
            Span::new(start, end),
            source,
            options,
            qualifier,
            type_arguments,
        )))
    }

    fn parse_import_type_options(&mut self) -> ParseResult<Option<Expr>> {
        if !self.eat(TokenTag::Comma)? {
            return Ok(None);
        }
        if self.current_token.tag == TokenTag::RightParen {
            return Ok(None);
        }
        let options =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        self.eat(TokenTag::Comma)?;
        Ok(Some(options))
    }

    fn parse_import_type_qualifier(&mut self) -> ParseResult<TsImportTypeQualifier> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            return self.error("expected an identifier");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        let identifier = self.ast.identifier_name(token.span, name);
        if self.current_token.tag != TokenTag::Dot {
            return Ok(TsImportTypeQualifier::IdentifierName(identifier));
        }
        let left = TsQualifiedNameLeft::IdentifierName(identifier);
        let qualified = self.extend_qualified_name(left, token.span.start)?;
        let qualified = qualified
            .as_ts_qualified_name(self.ast)
            .expect("qualified-name parsing produces a qualified name");
        Ok(TsImportTypeQualifier::TsQualifiedName(qualified))
    }

    fn extend_qualified_name(
        &mut self,
        mut left: TsQualifiedNameLeft,
        start: u32,
    ) -> ParseResult<TsTypeName> {
        let mut result = None;
        while self.current_token.tag == TokenTag::Dot
            && !self.current_token.has_line_terminator_before()
        {
            self.advance()?;
            let token = self.current_token;
            if !token.tag.is_identifier_like() {
                return self.error("expected an identifier after '.'");
            }
            let name = self.identifier_name(token)?;
            self.advance()?;
            let right = self.ast.identifier_name(token.span, name);
            let qualified =
                self.ast
                    .ts_qualified_name(Span::new(start, token.span.end), left, right);
            result = Some(qualified);
            left = TsQualifiedNameLeft::TsQualifiedName(qualified);
        }
        Ok(TsTypeName::TsQualifiedName(
            result.expect("qualified-name parsing starts on '.'"),
        ))
    }

    fn parse_jsdoc_nullable_or_unknown_type(&mut self) -> ParseResult<TsType> {
        let token = self.expect(TokenTag::Question)?;
        if !is_start_of_type(self.current_token.tag) {
            return Ok(TsType::TsJsDocUnknownType(
                self.ast.ts_js_doc_unknown_type(token.span),
            ));
        }
        let type_annotation = self.parse_type()?;
        let span = Span::new(token.span.start, type_annotation.span(self.ast).end);
        Ok(self.jsdoc_node_data(type_annotation, JsDocKind::Nullable, false, span))
    }

    fn parse_jsdoc_non_nullable_type(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::LogicalNot)?.span.start;
        let type_annotation = self.parse_primary_type()?;
        let span = Span::new(start, type_annotation.span(self.ast).end);
        Ok(self.jsdoc_node_data(type_annotation, JsDocKind::NonNullable, false, span))
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

    fn is_qualified_type_continuation(&mut self) -> ParseResult<bool> {
        let next = self.peek_ahead()?;
        Ok(next.tag == TokenTag::Dot && !next.has_line_terminator_before())
    }

    fn parse_literal_type(&mut self) -> ParseResult<TsType> {
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
        Ok(TsType::TsLiteralType(self.ast.ts_literal_type(
            Span::new(start, literal.span(self.ast).end),
            literal,
        )))
    }

    fn parse_signed_numeric_literal_type(&mut self) -> ParseResult<TsLiteral> {
        let sign_token = self.current_token;
        let next = self.peek_ahead()?;
        if !next.tag.is_numeric_literal() {
            return self.error("expected a numeric literal after a sign");
        }
        self.advance()?;
        let expression = self.parse_numeric_literal()?;
        let operator = if sign_token.tag == TokenTag::Minus {
            UnaryOperator::Negate
        } else {
            UnaryOperator::Positive
        };
        Ok(TsLiteral::UnaryExpression(self.ast.unary_expression(
            Span::new(sign_token.span.start, expression.span(self.ast).end),
            expression,
            operator,
        )))
    }

    fn parse_template_literal_type(&mut self) -> ParseResult<TsType> {
        let start = self.current_token.span.start;
        let head = self.current_token;
        let mut quasis = vec![self.add_template_element(head, false, false)?];
        let mut types = Vec::new();
        self.advance()?;
        let end = loop {
            types.push(self.parse_type()?);
            if self.current_token.tag != TokenTag::RightBrace {
                return self.error("expected '}' to close template type interpolation");
            }
            let continuation = self
                .lexer
                .re_scan_template_continuation(self.current_token.span.start)
                .map_err(|error| self.next_token_error(error))?;
            let tail = continuation.tag == TokenTag::TemplateTail;
            quasis.push(self.add_template_element(continuation, tail, false)?);
            let end = continuation.span.end;
            self.current_token = continuation;
            self.advance()?;
            if tail {
                break end;
            }
        };
        let quasis = self.ast.add_typed_sub_range(quasis);
        let types = self.ast.add_typed_sub_range(types);
        Ok(TsType::TsTemplateLiteralType(
            self.ast
                .ts_template_literal_type(Span::new(start, end), quasis, types),
        ))
    }

    fn is_start_of_mapped_type(&mut self) -> ParseResult<bool> {
        let cursor = self.lexer.cursor;
        let state = self.lexer.state;
        let mode = self.lexer.mode;
        let comments_len = self.lexer.comments.len();
        let result = (|| {
            let mut token = Token::eof(cursor);
            self.lexer.next_token(&mut token)?;
            if matches!(token.tag, TokenTag::Plus | TokenTag::Minus) {
                self.lexer.next_token(&mut token)?;
                if token.tag != TokenTag::Readonly {
                    return Ok(false);
                }
                self.lexer.next_token(&mut token)?;
            } else if token.tag == TokenTag::Readonly {
                self.lexer.next_token(&mut token)?;
            }
            if token.tag != TokenTag::LeftBracket {
                return Ok(false);
            }
            self.lexer.next_token(&mut token)?;
            if !token.tag.is_identifier_like() {
                return Ok(false);
            }
            self.lexer.next_token(&mut token)?;
            Ok(token.tag == TokenTag::In)
        })();
        self.lexer.cursor = cursor;
        self.lexer.state = state;
        self.lexer.mode = mode;
        self.lexer.comments.truncate(comments_len);
        result.map_err(|error| self.next_token_error(error))
    }

    fn parse_mapped_type(&mut self) -> ParseResult<TsType> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let readonly = self.parse_mapped_modifier(TokenTag::Readonly)?;
        self.expect(TokenTag::LeftBracket)?;
        let key = self.parse_binding_identifier()?;
        self.expect(TokenTag::In)?;
        let constraint = self.parse_type()?;
        let name_type = if self.eat(TokenTag::As)? {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(TokenTag::RightBracket)?;
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
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        Ok(TsType::TsMappedType(self.ast.ts_mapped_type(
            Span::new(start, end),
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
        )))
    }

    fn parse_mapped_modifier(&mut self, terminator: TokenTag) -> ParseResult<TsMappedTypeModifier> {
        if self.current_token.tag == terminator {
            self.advance()?;
            return Ok(TsMappedTypeModifier::True);
        }
        let modifier = match self.current_token.tag {
            TokenTag::Plus => TsMappedTypeModifier::Plus,
            TokenTag::Minus => TsMappedTypeModifier::Minus,
            _ => return Ok(TsMappedTypeModifier::None),
        };
        self.advance()?;
        self.expect(terminator)?;
        Ok(modifier)
    }

    fn parse_type_literal(&mut self) -> ParseResult<TsType> {
        let start = self.current_token.span.start;
        let (members, end) = self.parse_object_type_members()?;
        Ok(TsType::TsTypeLiteral(
            self.ast.ts_type_literal(Span::new(start, end), members),
        ))
    }

    fn parse_object_type_members(&mut self) -> ParseResult<(TypedSubRange<TsTypeElement>, u32)> {
        self.expect(TokenTag::LeftBrace)?;
        let mut members = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace
            && self.current_token.tag != TokenTag::Eof
        {
            let member = self.parse_type_member()?;
            self.consume_type_member_separator(member)?;
            members.push(member);
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        Ok((self.ast.add_typed_sub_range(members), end))
    }

    fn consume_type_member_separator(&mut self, member: TsTypeElement) -> ParseResult<bool> {
        match self.current_token.tag {
            TokenTag::Semicolon | TokenTag::Comma => {
                let end = self.current_token.span.end;
                self.extend_span_to(member, end);
                self.advance()?;
            }
            TokenTag::RightBrace => {}
            _ if self.current_token.has_line_terminator_before() => {}
            _ => return self.error("expected ';', ',', or newline between type members"),
        }
        Ok(true)
    }

    fn extend_span_to<N: GetSpan + SetSpan>(&mut self, node: N, end: u32) {
        let span = node.span(self.ast);
        if end > span.end {
            node.set_span(self.ast, Span::new(span.start, end));
        }
    }

    fn parse_type_member(&mut self) -> ParseResult<TsTypeElement> {
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

    fn parse_call_or_construct_signature(
        &mut self,
        is_construct: bool,
    ) -> ParseResult<TsTypeElement> {
        let start = self.current_token.span.start;
        if is_construct {
            self.expect(TokenTag::New)?;
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
        let end = return_type.map_or(self.prev_token_end, |ty| ty.span(self.ast).end);
        Ok(if is_construct {
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

    fn is_index_signature_start(&mut self) -> ParseResult<bool> {
        let name = self.peek_ahead()?;
        if !name.tag.is_identifier_like() {
            return Ok(false);
        }
        let mut peek = self.begin_peek();
        let second = (|| {
            peek.next()?;
            peek.next()
        })();
        peek.end();
        Ok(matches!(second?.tag, TokenTag::Colon | TokenTag::Comma))
    }

    fn parse_index_signature(
        &mut self,
        start: u32,
        readonly: bool,
        r#static: bool,
    ) -> ParseResult<TsTypeElement> {
        self.expect(TokenTag::LeftBracket)?;
        let parameter = self.parse_index_signature_parameter()?;
        if self.current_token.tag == TokenTag::Comma {
            return self.error("an index signature must have exactly one parameter");
        }
        let parameters = self.ast.add_typed_sub_range([parameter]);
        self.expect(TokenTag::RightBracket)?;
        if self.current_token.tag != TokenTag::Colon {
            return self.error("expected ':' after index signature parameters");
        }
        let type_annotation = self.parse_type_annotation()?;
        let end = type_annotation.span(self.ast).end;
        Ok(self.ast.ts_type_element_ts_index_signature(
            Span::new(start, end),
            parameters,
            type_annotation,
            readonly,
            r#static,
        ))
    }

    fn parse_index_signature_parameter(&mut self) -> ParseResult<BindingIdentifier> {
        let name = self.parse_binding_identifier()?;
        if self.current_token.tag != TokenTag::Colon {
            return self.error("expected ':' after index signature parameter name");
        }
        let annotation = self.parse_type_annotation()?;
        name.set_type_annotation(self.ast, Some(annotation));
        name.set_span(
            self.ast,
            Span::new(name.span(self.ast).start, annotation.span(self.ast).end),
        );
        Ok(name)
    }

    fn parse_property_or_method_signature(
        &mut self,
        start: u32,
        readonly: bool,
    ) -> ParseResult<TsTypeElement> {
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
        let key = self.parse_property_key()?;
        let mut end = key.span(self.ast).end;
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
            end = annotation.span(self.ast).end;
            Some(annotation)
        } else {
            None
        };
        Ok(self.ast.ts_type_element_ts_property_signature(
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
    ) -> ParseResult<TsTypeElement> {
        let type_parameters = if is_angle_open(self.current_token.tag) {
            Some(self.parse_type_parameters()?)
        } else {
            None
        };
        let params = self.parse_formal_parameters(FormalParameterKind::Signature, false)?;
        let return_type = if self.current_token.tag == TokenTag::Colon {
            let annotation = self.parse_return_type_annotation()?;
            if kind == TsMethodSignatureKind::Set {
                return self.error_at(
                    annotation.span(self.ast),
                    "a 'set' accessor cannot have a return type annotation",
                );
            }
            Some(annotation)
        } else {
            None
        };
        let end = return_type.map_or(self.prev_token_end, |ty| ty.span(self.ast).end);
        Ok(self.ast.ts_type_element_ts_method_signature(
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

    fn parse_type_reference(&mut self) -> ParseResult<TsType> {
        let type_name = self.parse_entity_name()?;
        let start = type_name.span(self.ast).start;
        let type_arguments = self.parse_type_arguments_after_entity_name()?;
        let end =
            type_arguments.map_or(type_name.span(self.ast).end, |args| args.span(self.ast).end);
        Ok(self
            .ast
            .ts_type_ts_type_reference(Span::new(start, end), type_name, type_arguments))
    }

    fn parse_entity_name(&mut self) -> ParseResult<TsTypeName> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            return self.error("expected a type name");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        let mut type_name = self
            .ast
            .ts_type_name_identifier_reference(token.span, name, None);
        while self.current_token.tag == TokenTag::Dot
            && !self.current_token.has_line_terminator_before()
        {
            type_name = self.parse_qualified_segment(type_name)?;
        }
        Ok(type_name)
    }

    fn parse_qualified_segment(&mut self, left: TsTypeName) -> ParseResult<TsTypeName> {
        let start = left.span(self.ast).start;
        self.expect(TokenTag::Dot)?;
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            return self.error("expected an identifier after '.'");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        let right = self.ast.identifier_name(token.span, name);
        let left = match self.ast.ts_type_name_data(left) {
            TsTypeNameData::IdentifierReference(identifier) => {
                TsQualifiedNameLeft::IdentifierReference(identifier)
            }
            TsTypeNameData::TsQualifiedName(qualified) => {
                TsQualifiedNameLeft::TsQualifiedName(qualified)
            }
        };
        Ok(TsTypeName::TsQualifiedName(self.ast.ts_qualified_name(
            Span::new(start, token.span.end),
            left,
            right,
        )))
    }

    fn parse_type_arguments_after_entity_name(
        &mut self,
    ) -> ParseResult<Option<TsTypeParameterInstantiation>> {
        if !is_angle_open(self.current_token.tag) || self.current_token.has_line_terminator_before()
        {
            return Ok(None);
        }
        self.parse_type_arguments().map(Some)
    }

    fn parse_type_parameters(&mut self) -> ParseResult<TsTypeParameterDeclaration> {
        match self.parse_angle_list(AngleListKind::Parameters)? {
            AngleList::Parameters(parameters) => Ok(parameters),
            AngleList::Arguments(_) => unreachable!(),
        }
    }

    fn parse_type_arguments(&mut self) -> ParseResult<TsTypeParameterInstantiation> {
        match self.parse_angle_list(AngleListKind::Arguments)? {
            AngleList::Arguments(arguments) => Ok(arguments),
            AngleList::Parameters(_) => unreachable!(),
        }
    }

    fn parse_angle_list(&mut self, kind: AngleListKind) -> ParseResult<AngleList> {
        let start = self.consume_angle_open()?;
        let mut nodes = Vec::new();
        while !is_angle_close(self.current_token.tag) && self.current_token.tag != TokenTag::Eof {
            let node = match kind {
                AngleListKind::Arguments => self.parse_type()?.node_id(),
                AngleListKind::Parameters => self.parse_type_parameter()?.node_id(),
            };
            nodes.push(node);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        if kind == AngleListKind::Arguments && nodes.is_empty() {
            return self.error("a type argument list cannot be empty");
        }
        let end = self.consume_angle_close()?;
        Ok(match kind {
            AngleListKind::Arguments => {
                let parameters = self.ast.add_typed_sub_range(
                    nodes
                        .into_iter()
                        .map(|node| unsafe { TsType::from_node_id_unchecked(node) }),
                );
                AngleList::Arguments(
                    self.ast
                        .ts_type_parameter_instantiation(Span::new(start, end), parameters),
                )
            }
            AngleListKind::Parameters => {
                let parameters = self.ast.add_typed_sub_range(
                    nodes
                        .into_iter()
                        .map(|node| unsafe { TsTypeParameter::from_node_id_unchecked(node) }),
                );
                AngleList::Parameters(
                    self.ast
                        .ts_type_parameter_declaration(Span::new(start, end), parameters),
                )
            }
        })
    }

    fn parse_type_parameter(&mut self) -> ParseResult<TsTypeParameter> {
        let mut is_const = false;
        let mut is_in = false;
        let mut is_out = false;
        let mut modifier_step = 0u8;
        let start = self.current_token.span.start;
        loop {
            let token = self.current_token;
            let (step, flag) = match token.tag {
                TokenTag::Const => (1, &mut is_const),
                TokenTag::In => (2, &mut is_in),
                TokenTag::Out => (3, &mut is_out),
                _ => break,
            };
            if !self.peek_ahead()?.tag.is_identifier_like() {
                break;
            }
            if *flag {
                return self.error_at(token.span, "duplicate modifier on type parameter");
            }
            if step < modifier_step {
                return self.error_at(
                    token.span,
                    "type parameter modifiers must appear in the order 'const in out'",
                );
            }
            *flag = true;
            modifier_step = modifier_step.max(step);
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
            || constraint.map_or(name.span(self.ast).end, |ty| ty.span(self.ast).end),
            |ty| ty.span(self.ast).end,
        );
        Ok(self.ast.ts_type_parameter(
            Span::new(start, end),
            name,
            constraint,
            default,
            is_in,
            is_out,
            is_const,
        ))
    }

    fn consume_angle_open(&mut self) -> ParseResult<u32> {
        let start = self.current_token.span.start;
        match self.current_token.tag {
            TokenTag::LessThan => self.advance()?,
            TokenTag::LeftShift => {
                let token = self.lexer.re_scan_less_than(start);
                self.advance_with_rescanned_token(token)?;
            }
            _ => return self.error("expected '<' to start a type argument list"),
        }
        Ok(start)
    }

    fn consume_angle_close(&mut self) -> ParseResult<u32> {
        match self.current_token.tag {
            TokenTag::GreaterThan => Ok(self.expect(TokenTag::GreaterThan)?.span.end),
            TokenTag::RightShift
            | TokenTag::UnsignedRightShift
            | TokenTag::GreaterThanEqual
            | TokenTag::RightShiftAssign
            | TokenTag::UnsignedRightShiftAssign => {
                let token = self
                    .lexer
                    .re_scan_greater_than(self.current_token.span.start);
                let end = token.span.end;
                self.advance_with_rescanned_token(token)?;
                Ok(end)
            }
            _ => self.error("expected '>' to close a type argument list"),
        }
    }

    fn parse_for_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::For)?.span.start;
        if self.current_token.tag == TokenTag::Await {
            if !self.context.await_ {
                self.report(
                    self.current_token.span,
                    "'for await' is only valid in async functions or modules",
                    ReportOptions::default(),
                );
            }
            self.advance()?;
            self.expect(TokenTag::LeftParen)?;
            return self.parse_for_head(start, true);
        }
        self.expect(TokenTag::LeftParen)?;
        self.parse_for_head(start, false)
    }

    fn parse_for_head(&mut self, start: u32, is_for_await: bool) -> ParseResult<Stmt> {
        if self.current_token.tag == TokenTag::Semicolon {
            return self.parse_for_statement_rest(start, None, is_for_await);
        }

        let declaration_start = self.current_token.span.start;
        match self.current_token.tag {
            TokenTag::Var | TokenTag::Let | TokenTag::Const => {
                let kind = self.parse_variable_kind(false)?;
                self.parse_for_with_declaration(start, is_for_await, kind, declaration_start)
            }
            TokenTag::Using => {
                let next = self.peek_ahead()?;
                if next.has_line_terminator_before()
                    || matches!(
                        next.tag,
                        TokenTag::In
                            | TokenTag::Dot
                            | TokenTag::LeftParen
                            | TokenTag::LeftBracket
                            | TokenTag::LeftBrace
                    )
                {
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
                let next = self.peek_ahead()?;
                if next.tag != TokenTag::Using || next.has_line_terminator_before() {
                    return self.parse_for_with_expression(start, is_for_await);
                }
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
    ) -> ParseResult<Stmt> {
        let old_allow_in = self.context.in_;
        self.context.in_ = false;
        let mut declarators = vec![self.parse_variable_declarator(kind, DeclaratorCtx::ForLoop)?];
        while self.eat(TokenTag::Comma)? {
            declarators.push(self.parse_variable_declarator(kind, DeclaratorCtx::ForLoop)?);
        }
        self.context.in_ = old_allow_in;

        let declaration_end = declarators
            .last()
            .expect("a variable declaration has at least one declarator")
            .span(self.ast)
            .end;
        let declarators = self.ast.add_typed_sub_range(declarators);
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
                        declaration.span(self.ast),
                        format!(
                            "The left-hand side of a for...in statement cannot be an {} declaration.",
                            kind.as_str()
                        ),
                        ReportOptions::default(),
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

    fn parse_for_with_expression(&mut self, start: u32, is_for_await: bool) -> ParseResult<Stmt> {
        let old_allow_in = self.context.in_;
        self.context.in_ = false;
        let expression = self.parse_expression(
            Precedence::COMMA,
            ParseExpressionOpts {
                respect_allow_in: true,
                ..ParseExpressionOpts::default()
            },
        );
        self.context.in_ = old_allow_in;
        let expression = expression?;

        if self.current_token.tag == TokenTag::In {
            let left = ForStatementLeft::AssignmentTarget(
                self.expression_to_assignment_target(expression)?,
            );
            return self.parse_for_in_statement_rest(start, left, is_for_await);
        }

        if self.current_token.tag == TokenTag::Of {
            if !is_for_await && self.is_async_identifier(expression) {
                self.report(
                    expression.span(self.ast),
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
    ) -> ParseResult<Stmt> {
        if is_for_await {
            return self.error("'for await' is only valid with for-of loops");
        }
        self.expect(TokenTag::Semicolon)?;
        let test = if self.current_token.tag == TokenTag::Semicolon {
            None
        } else {
            Some(self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?)
        };
        self.expect(TokenTag::Semicolon)?;
        let update = if self.current_token.tag == TokenTag::RightParen {
            None
        } else {
            Some(self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?)
        };
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_for_statement(Span::new(start, end), init, test, update, body))
    }

    fn parse_for_in_statement_rest(
        &mut self,
        start: u32,
        left: ForStatementLeft,
        is_for_await: bool,
    ) -> ParseResult<Stmt> {
        if is_for_await {
            return self.error("'for await' requires 'of', not 'in'");
        }
        self.report_if_escaped_keyword(self.current_token);
        self.advance_without_escape_check()?;
        let right = self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_for_in_statement(Span::new(start, end), left, right, body))
    }

    fn parse_for_of_statement_rest(
        &mut self,
        start: u32,
        left: ForStatementLeft,
        is_for_await: bool,
    ) -> ParseResult<Stmt> {
        self.report_if_escaped_keyword(self.current_token);
        self.advance_without_escape_check()?;
        let right =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement(ParseStatementOpts {
            can_be_single_statement_context: true,
        })?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_for_of_statement(Span::new(start, end), left, right, body, is_for_await))
    }

    fn is_async_identifier(&self, expression: Expr) -> bool {
        matches!(
            self.ast.expr_data(expression),
            ExprData::IdentifierReference(identifier)
                if identifier.name(self.ast) == "async"
                    && self.span_text(identifier.span(self.ast)) == b"async"
        )
    }

    fn validate_regular_for_declarators(
        &self,
        declaration: VariableDeclaration,
    ) -> ParseResult<()> {
        let kind = declaration.kind(self.ast);
        for index in 0..declaration.declarators(self.ast).len() {
            let declarator = declaration
                .declarators(self.ast)
                .get_node(self.ast, index)
                .expect("variable declarator range contains its nodes");
            if declarator.init(self.ast).is_some() {
                continue;
            }
            let id = declarator.id(self.ast);
            if !matches!(
                self.ast.binding_pattern_data(id),
                BindingPatternData::BindingIdentifier(_)
            ) {
                return self.error_at(
                    id.span(self.ast),
                    "destructuring declaration in a for-loop initializer requires an initializer",
                );
            }
            if kind == VariableKind::Const {
                return self.error_at(
                    id.span(self.ast),
                    "const declaration in a for-loop initializer requires an initializer",
                );
            }
        }
        Ok(())
    }

    fn parse_switch_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Switch)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let discriminant =
            self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        self.expect(TokenTag::RightParen)?;
        self.expect(TokenTag::LeftBrace)?;
        let cases = self.parse_switch_cases()?;
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        Ok(self
            .ast
            .stmt_switch_statement(Span::new(start, end), discriminant, cases))
    }

    fn parse_switch_cases(&mut self) -> ParseResult<TypedSubRange<SwitchCase>> {
        let mut cases = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            cases.push(self.parse_switch_case()?);
        }
        Ok(self.ast.add_typed_sub_range(cases))
    }

    fn parse_switch_case(&mut self) -> ParseResult<SwitchCase> {
        let case_start = self.current_token.span.start;
        let test = if self.eat(TokenTag::Case)? {
            Some(self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?)
        } else {
            self.expect(TokenTag::Default)?;
            None
        };
        let colon_end = self.expect(TokenTag::Colon)?.span.end;
        let consequent = self.parse_case_consequent()?;
        let case_end = if consequent.is_empty() {
            colon_end
        } else {
            consequent
                .get_node(self.ast, consequent.len() - 1)
                .expect("non-empty case consequent contains its last statement")
                .span(self.ast)
                .end
        };
        Ok(self
            .ast
            .switch_case(Span::new(case_start, case_end), test, consequent))
    }

    fn parse_case_consequent(&mut self) -> ParseResult<TypedSubRange<Stmt>> {
        let mut consequent = Vec::new();
        while !matches!(
            self.current_token.tag,
            TokenTag::Case | TokenTag::Default | TokenTag::RightBrace | TokenTag::Eof
        ) {
            let statement = self.parse_statement(ParseStatementOpts::default())?;
            if let StmtData::Declaration(declaration) = self.ast.stmt_data(statement)
                && let DeclData::VariableDeclaration(declaration) = self.ast.decl_data(declaration)
                && matches!(
                    declaration.kind(self.ast),
                    VariableKind::Using | VariableKind::AwaitUsing
                )
            {
                self.report(
                    statement.span(self.ast),
                    "Using declaration cannot appear in the bare case statement.",
                    ReportOptions {
                        help: Some(Cow::Borrowed("Wrap this declaration in a block statement")),
                        ..ReportOptions::default()
                    },
                );
            }
            consequent.push(statement);
        }
        Ok(self.ast.add_typed_sub_range(consequent))
    }

    fn parse_try_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Try)?.span.start;
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
            return self.error("try requires catch or finally");
        }
        let end = finalizer.map_or_else(
            || handler.map_or(block.span(self.ast).end, |node| node.span(self.ast).end),
            |node| node.span(self.ast).end,
        );
        Ok(self
            .ast
            .stmt_try_statement(Span::new(start, end), block, handler, finalizer))
    }

    fn parse_catch_clause(&mut self) -> ParseResult<CatchClause> {
        let start = self.expect(TokenTag::Catch)?.span.start;
        let param = if self.eat(TokenTag::LeftParen)? {
            let param = self.parse_binding_pattern()?;
            if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                let annotation = self.parse_type_annotation()?;
                self.apply_type_annotation_to_pattern(param, annotation);
            }
            self.expect(TokenTag::RightParen)?;
            Some(param)
        } else {
            None
        };
        let body = self.parse_block_statement()?;
        Ok(self
            .ast
            .catch_clause(Span::new(start, body.span(self.ast).end), param, body))
    }

    fn parse_expression(&mut self, minimum: u8, opts: ParseExpressionOpts) -> ParseResult<Expr> {
        let mut left = self.parse_prefix(opts.in_cover, minimum)?;
        loop {
            let token = self.current_token;
            if self.lang.is_typescript()
                && is_angle_open(token.tag)
                && max_left_precedence(left, self.ast) >= Precedence::CALL
                && let Some(expression) = self.parse_type_argumented_call_or_instantiation(left)?
            {
                left = expression;
                continue;
            }
            if opts.respect_allow_in && token.tag == TokenTag::In && !self.context.in_ {
                break;
            }
            let precedence = self.infix_precedence(token);
            if precedence < minimum || precedence == Precedence::LOWEST {
                break;
            }
            if precedence > max_left_precedence(left, self.ast) {
                break;
            }
            left = self.parse_infix(precedence, left)?;
        }
        Ok(left)
    }

    fn parse_expression_in_cover(&mut self, precedence: u8) -> ParseResult<Expr> {
        self.parse_expression(
            precedence,
            ParseExpressionOpts {
                in_cover: true,
                ..ParseExpressionOpts::default()
            },
        )
    }

    fn parse_assignment_expression_in_cover(&mut self) -> ParseResult<Expr> {
        self.parse_expression_in_cover(Precedence::ASSIGNMENT)
    }

    fn infix_precedence(&self, token: Token) -> u8 {
        match token.tag {
            TokenTag::Increment | TokenTag::Decrement | TokenTag::As | TokenTag::Satisfies
                if token.has_line_terminator_before() =>
            {
                Precedence::LOWEST
            }
            TokenTag::LogicalNot
                if !self.lang.is_typescript() || token.has_line_terminator_before() =>
            {
                Precedence::LOWEST
            }
            _ => token.tag.precedence(),
        }
    }

    fn parse_infix(&mut self, _precedence: u8, left: Expr) -> ParseResult<Expr> {
        let token = self.current_token;
        match token.tag {
            TokenTag::Dot => self.parse_static_member_expression(left, false),
            TokenTag::LeftBracket => self.parse_computed_member_expression(left, false),
            TokenTag::OptionalChaining => self.parse_optional_chain(left),
            TokenTag::LeftParen => self.parse_call_expression(left, false, None),
            TokenTag::Increment | TokenTag::Decrement => {
                self.parse_update_expression(false, Some(left))
            }
            TokenTag::Question => self.parse_conditional_expression(left),
            TokenTag::As | TokenTag::Satisfies if self.lang.is_typescript() => {
                self.parse_as_or_satisfies_expression(left, token.tag)
            }
            TokenTag::LogicalNot if self.lang.is_typescript() => {
                self.parse_non_null_expression(left)
            }
            tag if tag.is_assignment_operator() => self.parse_assignment_expression(left, tag),
            TokenTag::Arrow => self.parse_simple_arrow_function(left),
            tag if tag.is_binary_operator() => self.parse_binary_expression(left, tag),
            tag if tag.is_logical_operator() => self.parse_logical_expression(left, tag),
            TokenTag::Comma => self.parse_sequence_expression(left),
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                if expression_contains_optional_chain(left, self.ast) {
                    return self.error_at(
                        left.span(self.ast),
                        "An optional chain cannot be used as a tagged template tag",
                    );
                }
                self.parse_tagged_template_expression(left, None)
            }
            _ => self.error("unexpected token in expression"),
        }
    }

    fn parse_prefix(&mut self, in_cover: bool, minimum: u8) -> ParseResult<Expr> {
        let token = self.current_token;
        match token.tag {
            TokenTag::Async => self.parse_async_function_or_arrow(minimum),
            TokenTag::Await if self.context.await_ => {
                let start = self.expect(TokenTag::Await)?.span.start;
                self.parse_await_expression(start)
            }
            TokenTag::Yield if self.context.yield_ && minimum < Precedence::ADDITIVE => {
                self.parse_yield_expression()
            }
            TokenTag::LeftParen => self.parse_parenthesized_or_arrow_function(None, false, minimum),
            TokenTag::New => self.parse_new_expression(),
            TokenTag::At => {
                let (start, decorators) = self.parse_decorators()?;
                Ok(Expr::Class(self.parse_class_decorated(
                    ParseClassOpts {
                        is_expression: true,
                        ..ParseClassOpts::default()
                    },
                    Some(start),
                    decorators,
                )?))
            }
            TokenTag::LessThan if self.lang.is_typescript() => {
                if let Some(arrow) = self.try_parse_generic_arrow(false, token.span.start)? {
                    Ok(arrow)
                } else if !self.lang.is_jsx() {
                    self.parse_type_assertion()
                } else {
                    self.parse_jsx_expression()
                }
            }
            TokenTag::LessThan if self.lang.is_jsx() => self.parse_jsx_expression(),
            TokenTag::Import => self.parse_import_expression(),
            TokenTag::Increment | TokenTag::Decrement => self.parse_update_expression(true, None),
            tag if tag.is_unary_operator() => self.parse_unary_expression(),
            _ => self.parse_primary_expression(in_cover),
        }
    }

    fn parse_primary_expression(&mut self, in_cover: bool) -> ParseResult<Expr> {
        let token = self.current_token;
        match token.tag {
            TokenTag::PrivateIdentifier => {
                let private = self.parse_private_identifier()?;
                if self.current_token.tag != TokenTag::In {
                    return self.error_at(
                        token.span,
                        "Private names are only valid in property accesses or 'in' expressions",
                    );
                }
                Ok(Expr::PrivateIdentifier(private))
            }
            tag if tag.is_identifier_like() && !tag.is_unconditionally_reserved() => {
                self.parse_identifier()
            }
            TokenTag::This => self.parse_this_expression(),
            TokenTag::Super => self.parse_super_expression(),
            TokenTag::True | TokenTag::False => self.parse_boolean_literal(),
            TokenTag::NullLiteral => self.parse_null_literal(),
            tag if tag.is_numeric_literal() => self.parse_numeric_literal(),
            TokenTag::StringLiteral => Ok(Expr::StringLiteral(self.parse_string_literal()?)),
            TokenTag::NoSubstitutionTemplate => Ok(Expr::TemplateLiteral(
                self.parse_no_substitution_template(false)?,
            )),
            TokenTag::TemplateHead => {
                Ok(Expr::TemplateLiteral(self.parse_template_literal(false)?))
            }
            TokenTag::Slash | TokenTag::SlashAssign => self.parse_regexp_literal(),
            TokenTag::LeftBracket => self.parse_array_expression(in_cover),
            TokenTag::LeftBrace => self.parse_object_expression(in_cover),
            TokenTag::Function => Ok(Expr::Function(self.parse_function(
                ParseFunctionOpts {
                    is_expression: true,
                    ..ParseFunctionOpts::default()
                },
                None,
            )?)),
            TokenTag::Class => Ok(Expr::Class(self.parse_class(
                ParseClassOpts {
                    is_expression: true,
                    ..ParseClassOpts::default()
                },
                None,
            )?)),
            _ => self.error("expected an expression"),
        }
    }

    fn parse_identifier(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        let name = self.identifier_name(token)?;
        self.validate_identifier(token, name.as_str(), "an identifier")?;
        self.advance_without_escape_check()?;
        Ok(self.ast.expr_identifier_reference(token.span, name, None))
    }

    fn parse_private_identifier(&mut self) -> ParseResult<PrivateIdentifier> {
        let token = self.current_token;
        let name = self.identifier_name(token)?;
        let private = self.ast.private_identifier(token.span, name);
        self.advance()?;
        Ok(private)
    }

    fn parse_identifier_name(&mut self) -> ParseResult<IdentifierName> {
        let token = self.current_token;
        let name = self.identifier_name(token)?;
        self.advance_without_escape_check()?;
        Ok(self.ast.identifier_name(token.span, name))
    }

    fn parse_this_expression(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        Ok(self.ast.expr_this_expression(token.span))
    }

    fn parse_super_expression(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        if !matches!(
            self.current_token.tag,
            TokenTag::LeftParen | TokenTag::Dot | TokenTag::LeftBracket
        ) {
            return self.error("'super' must be followed by a call or property access");
        }
        Ok(self.ast.expr_super(token.span))
    }

    fn parse_boolean_literal(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        Ok(self
            .ast
            .expr_boolean_literal(token.span, token.tag == TokenTag::True))
    }

    fn parse_null_literal(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        Ok(self.ast.expr_null_literal(token.span))
    }

    fn parse_numeric_literal(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        let kind = match token.tag {
            TokenTag::NumericLiteral => NumericLiteralKind::Decimal,
            TokenTag::HexLiteral => NumericLiteralKind::Hex,
            TokenTag::OctalLiteral => NumericLiteralKind::Octal,
            TokenTag::BinaryLiteral => NumericLiteralKind::Binary,
            TokenTag::BigIntLiteral => {
                let raw = self.atom_from_span(Span::new(token.span.start, token.span.end - 1));
                self.advance()?;
                return Ok(self.ast.expr_big_int_literal(token.span, raw));
            }
            _ => unreachable!(),
        };
        let raw = self.atom_from_span(token.span);
        self.advance()?;
        Ok(self.ast.expr_numeric_literal(token.span, kind, raw))
    }

    fn parse_string_literal(&mut self) -> ParseResult<StringLiteral> {
        let token = self.current_token;
        if token.tag != TokenTag::StringLiteral {
            return self.error("expected a string literal");
        }
        let raw = self.atom_from_span(token.span);
        let value = self.string_value(token)?;
        self.advance()?;
        Ok(self.ast.string_literal(token.span, value, raw))
    }

    fn parse_regexp_literal(&mut self) -> ParseResult<Expr> {
        let result = self
            .lexer
            .re_scan_as_regex(self.current_token.span.start)
            .map_err(|error| self.next_token_error(error))?;
        let pattern = self.atom_from_span(result.pattern);
        let flags = self.atom_from_span(result.flags);
        let expression = self.ast.expr_reg_exp_literal(result.span, pattern, flags);
        self.current_token = Token::eof(result.span.end);
        self.advance()?;
        Ok(expression)
    }

    fn parse_no_substitution_template(&mut self, tagged: bool) -> ParseResult<TemplateLiteral> {
        let token = self.current_token;
        let quasi = self.add_template_element(token, true, tagged)?;
        self.advance()?;
        let quasis = self.ast.add_typed_sub_range([quasi]);
        let expressions = self.ast.add_typed_sub_range(std::iter::empty());
        Ok(self.ast.template_literal(token.span, quasis, expressions))
    }

    fn parse_template_literal(&mut self, tagged: bool) -> ParseResult<TemplateLiteral> {
        let start = self.current_token.span.start;
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();
        let head = self.current_token;
        quasis.push(self.add_template_element(head, false, tagged)?);
        self.advance()?;
        let end = loop {
            expressions
                .push(self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?);
            if self.current_token.tag != TokenTag::RightBrace {
                return self.error("expected '}' in template literal");
            }
            let continuation = self
                .lexer
                .re_scan_template_continuation(self.current_token.span.start)
                .map_err(|error| self.next_token_error(error))?;
            let tail = continuation.tag == TokenTag::TemplateTail;
            quasis.push(self.add_template_element(continuation, tail, tagged)?);
            let end = continuation.span.end;
            self.current_token = continuation;
            self.advance()?;
            if tail {
                break end;
            }
        };
        let quasis = self.ast.add_typed_sub_range(quasis);
        let expressions = self.ast.add_typed_sub_range(expressions);
        Ok(self
            .ast
            .template_literal(Span::new(start, end), quasis, expressions))
    }

    fn parse_tagged_template_expression(
        &mut self,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ParseResult<Expr> {
        let start = tag.span(self.ast).start;
        let quasi = if self.current_token.tag == TokenTag::NoSubstitutionTemplate {
            self.parse_no_substitution_template(true)?
        } else {
            self.parse_template_literal(true)?
        };
        Ok(self.ast.expr_tagged_template_expression(
            Span::new(start, quasi.span(self.ast).end),
            tag,
            quasi,
            type_arguments,
        ))
    }

    fn add_template_element(
        &mut self,
        token: Token,
        tail: bool,
        tagged: bool,
    ) -> ParseResult<TemplateElement> {
        let span = self.get_template_element_span(token);
        let raw = self.atom_from_span(span);
        let is_cooked_undefined = token.has_invalid_escape();
        if is_cooked_undefined && !tagged {
            return self.error_at(span, "invalid escape sequence in template literal");
        }
        let cooked = if is_cooked_undefined {
            self.ast.empty_atom()
        } else {
            self.template_element_value(token, span)?
        };
        Ok(self
            .ast
            .template_element(span, cooked, raw, tail, is_cooked_undefined))
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

    fn parse_parenthesized_or_arrow_function(
        &mut self,
        arrow_start: Option<u32>,
        async_function: bool,
        minimum: u8,
    ) -> ParseResult<Expr> {
        if minimum > Precedence::ASSIGNMENT {
            return self.parse_parenthesized_expression();
        }
        let arrow_start_value = arrow_start.unwrap_or(self.current_token.span.start);
        if self.lang.is_typescript() && minimum <= Precedence::ASSIGNMENT {
            match self.classify_arrow_head()? {
                ArrowHead::Yes => {
                    return self.parse_arrow_inner(async_function, arrow_start_value);
                }
                ArrowHead::Maybe => {
                    if let Some(arrow) = self.try_parse_arrow(async_function, arrow_start_value)? {
                        return Ok(arrow);
                    }
                }
                ArrowHead::No => {}
            }
        }
        let old_context = self.context;
        if async_function {
            self.context.await_ = true;
        }
        let cover = self.parse_parenthesized_cover()?;
        let is_arrow = self.current_token.tag == TokenTag::Arrow
            && !self.current_token.has_line_terminator_before()
            && minimum <= Precedence::ASSIGNMENT;
        self.context = old_context;
        if is_arrow {
            return self.cover_to_arrow_function(
                cover,
                async_function,
                arrow_start.unwrap_or(cover.start),
            );
        }
        self.cover_to_parenthesized_expression(cover)
    }

    fn parse_parenthesized_cover(&mut self) -> ParseResult<ParenthesizedCover> {
        let start = self.expect(TokenTag::LeftParen)?.span.start;
        let mut elements = Vec::new();
        let mut has_trailing_comma = false;
        while self.current_token.tag != TokenTag::RightParen
            && self.current_token.tag != TokenTag::Eof
        {
            if self.eat(TokenTag::Spread)? {
                let spread_start = self.prev_token_end - 3;
                let argument = self.parse_expression(
                    Precedence::ASSIGNMENT,
                    ParseExpressionOpts {
                        in_cover: true,
                        ..ParseExpressionOpts::default()
                    },
                )?;
                elements.push(Argument::SpreadElement(self.ast.spread_element(
                    Span::new(spread_start, argument.span(self.ast).end),
                    argument,
                )));
            } else {
                elements.push(Argument::Expr(self.parse_expression(
                    Precedence::ASSIGNMENT,
                    ParseExpressionOpts {
                        in_cover: true,
                        ..ParseExpressionOpts::default()
                    },
                )?));
            }
            if !self.eat(TokenTag::Comma)? {
                break;
            }
            if self.current_token.tag == TokenTag::RightParen {
                has_trailing_comma = true;
                break;
            }
        }
        let end = self.expect(TokenTag::RightParen)?.span.end;
        let elements = self.ast.add_typed_sub_range(elements);
        Ok(ParenthesizedCover {
            elements,
            start,
            end,
            has_trailing_comma,
        })
    }

    fn cover_to_parenthesized_expression(
        &mut self,
        cover: ParenthesizedCover,
    ) -> ParseResult<Expr> {
        if cover.elements.is_empty() {
            return self.error_at(
                Span::new(cover.start, cover.end),
                "empty parentheses are only valid as arrow function parameters",
            );
        }
        if cover.has_trailing_comma {
            return self.error_at(
                Span::new(cover.start, cover.end),
                "a parenthesized expression cannot have a trailing comma",
            );
        }
        let arguments = (0..cover.elements.len())
            .map(|index| {
                cover
                    .elements
                    .get_node(self.ast, index)
                    .expect("parenthesized cover contains its arguments")
            })
            .collect::<Vec<_>>();
        let mut expressions = Vec::with_capacity(arguments.len());
        for argument in arguments {
            match self.ast.argument_data(argument) {
                ArgumentData::Expr(expression) => {
                    self.validate_no_cover_initialized_syntax(expression)?;
                    expressions.push(expression);
                }
                ArgumentData::SpreadElement(spread) => {
                    return self.error_at(
                        spread.span(self.ast),
                        "spread is not allowed in a parenthesized expression",
                    );
                }
            }
        }
        let expression = if expressions.len() == 1 {
            expressions[0]
        } else {
            let expressions = self.ast.add_typed_sub_range(expressions);
            self.ast
                .expr_sequence_expression(Span::new(cover.start + 1, cover.end - 1), expressions)
        };
        if self.state.cover_has_init_name {
            self.state.cover_has_init_name = false;
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        if self.preserve_parens {
            Ok(self
                .ast
                .expr_parenthesized_expression(Span::new(cover.start, cover.end), expression))
        } else {
            Ok(expression)
        }
    }

    fn parse_parenthesized_expression(&mut self) -> ParseResult<Expr> {
        let cover = self.parse_parenthesized_cover()?;
        self.cover_to_parenthesized_expression(cover)
    }

    fn cover_to_arrow_function(
        &mut self,
        cover: ParenthesizedCover,
        is_async: bool,
        arrow_start: u32,
    ) -> ParseResult<Expr> {
        let (parameters, rest) = self.convert_to_formal_parameters(cover)?;
        self.expect(TokenTag::Arrow)?;
        self.state.cover_has_init_name = false;
        self.parse_arrow(arrow_start, parameters, rest, is_async)
    }

    fn convert_to_formal_parameters(
        &mut self,
        cover: ParenthesizedCover,
    ) -> ParseResult<(Vec<FormalParameterItem>, Option<BindingRestElement>)> {
        let arguments = (0..cover.elements.len())
            .map(|index| {
                cover
                    .elements
                    .get_node(self.ast, index)
                    .expect("parenthesized cover contains its arguments")
            })
            .collect::<Vec<_>>();
        let mut parameters = Vec::new();
        let mut rest = None;
        for (index, argument) in arguments.iter().copied().enumerate() {
            match self.ast.argument_data(argument) {
                ArgumentData::Expr(expression) => {
                    if rest.is_some() {
                        return self
                            .error_at(expression.span(self.ast), "a rest parameter must be last");
                    }
                    parameters.push(self.convert_to_formal_parameter(expression)?);
                }
                ArgumentData::SpreadElement(spread) => {
                    if index + 1 != arguments.len() {
                        return self
                            .error_at(spread.span(self.ast), "a rest parameter must be last");
                    }
                    if cover.has_trailing_comma {
                        return self.error_at(
                            spread.span(self.ast),
                            "a rest parameter cannot have a trailing comma",
                        );
                    }
                    let argument = self.expression_to_binding_pattern(spread.argument(self.ast))?;
                    if matches!(
                        self.ast.binding_pattern_data(argument),
                        BindingPatternData::AssignmentPattern(_)
                    ) {
                        return self.error_at(
                            argument.span(self.ast),
                            "a rest parameter may not have an initializer",
                        );
                    }
                    let decorators = self.empty_range();
                    rest = Some(self.ast.binding_rest_element(
                        spread.span(self.ast),
                        argument,
                        decorators,
                        None,
                        false,
                    ));
                }
            }
        }
        Ok((parameters, rest))
    }

    fn convert_to_formal_parameter(
        &mut self,
        expression: Expr,
    ) -> ParseResult<FormalParameterItem> {
        self.expression_to_parameter(expression)
    }

    fn parse_async_function_or_arrow(&mut self, minimum: u8) -> ParseResult<Expr> {
        let token = self.current_token;
        let is_arrow_head = {
            let mut peek = self.begin_peek();
            let result = (|| {
                let mut next = peek.next()?;
                if next.has_line_terminator_before() {
                    return Ok(false);
                }
                if next.tag.is_identifier_like() {
                    return Ok(peek.next()?.tag == TokenTag::Arrow);
                }
                if next.tag != TokenTag::LeftParen {
                    return Ok(false);
                }
                let mut depth = 1u32;
                while depth > 0 {
                    next = peek.next()?;
                    match next.tag {
                        TokenTag::LeftParen => depth += 1,
                        TokenTag::RightParen => depth -= 1,
                        TokenTag::Eof => return Ok(false),
                        _ => {}
                    }
                }
                Ok(peek.next()?.tag == TokenTag::Arrow)
            })();
            peek.end();
            result?
        };
        let name = self.identifier_name(token)?;
        self.advance()?;
        let async_identifier = self.ast.expr_identifier_reference(token.span, name, None);

        if self.current_token.has_line_terminator_before() {
            return Ok(async_identifier);
        }

        match self.current_token.tag {
            TokenTag::Function => {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                Ok(Expr::Function(self.parse_function(
                    ParseFunctionOpts {
                        is_async: true,
                        is_expression: true,
                        ..ParseFunctionOpts::default()
                    },
                    Some(token.span.start),
                )?))
            }
            TokenTag::LeftParen => self.parse_async_arrow_function_or_call(
                token.span,
                async_identifier,
                minimum,
                token.is_escaped(),
                is_arrow_head,
            ),
            TokenTag::LessThan if self.lang.is_typescript() => {
                if let Some(arrow) = self.try_parse_generic_arrow(true, token.span.start)? {
                    if token.is_escaped() {
                        return self
                            .error_at(token.span, "keywords may not contain escape sequences");
                    }
                    return Ok(arrow);
                }
                Ok(async_identifier)
            }
            tag if tag.is_identifier_like() && is_arrow_head => {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                let parameter = self.parse_primary_expression(false)?;
                self.identifier_to_arrow_function(parameter, true, Some(token.span.start))
            }
            _ => Ok(async_identifier),
        }
    }

    fn parse_async_arrow_function_or_call(
        &mut self,
        async_span: Span,
        async_identifier: Expr,
        minimum: u8,
        escaped_async: bool,
        is_arrow_head: bool,
    ) -> ParseResult<Expr> {
        if self.lang.is_typescript() && minimum <= Precedence::ASSIGNMENT {
            let arrow = match self.classify_arrow_head()? {
                ArrowHead::Yes => Some(self.parse_arrow_inner(true, async_span.start)?),
                ArrowHead::Maybe => self.try_parse_arrow(true, async_span.start)?,
                ArrowHead::No => None,
            };
            if let Some(arrow) = arrow {
                if escaped_async {
                    return self.error_at(async_span, "keywords may not contain escape sequences");
                }
                return Ok(arrow);
            }
        }
        if is_arrow_head && minimum <= Precedence::ASSIGNMENT {
            if escaped_async {
                return self.error_at(async_span, "keywords may not contain escape sequences");
            }
            return self.parse_parenthesized_or_arrow_function(
                Some(async_span.start),
                true,
                minimum,
            );
        }

        let old_context = self.context;
        self.context.await_ = true;
        let expression = self.parse_call_expression(async_identifier, false, None);
        self.context = old_context;
        expression
    }

    fn identifier_to_arrow_function(
        &mut self,
        left: Expr,
        async_function: bool,
        arrow_start: Option<u32>,
    ) -> ParseResult<Expr> {
        let ExprData::IdentifierReference(identifier) = self.ast.expr_data(left) else {
            return self.error_at(
                left.span(self.ast),
                "'=>' is only valid after a single identifier or a parenthesized parameter list",
            );
        };
        let span = identifier.span(self.ast);
        let name = identifier.name(self.ast);
        if async_function && name == "await" {
            return self.error_at(
                span,
                "'await' is reserved in an async context and cannot be used as a binding identifier",
            );
        }
        self.advance()?;
        let decorators = self.empty_range();
        let pattern = self
            .ast
            .binding_pattern_binding_identifier(span, name, decorators, None, false, None);
        let parameter = self
            .ast
            .formal_parameter(span, FormalParameterPattern::BindingPattern(pattern));
        self.parse_arrow(
            arrow_start.unwrap_or(span.start),
            vec![FormalParameterItem::FormalParameter(parameter)],
            None,
            async_function,
        )
    }

    fn parse_simple_arrow_function(&mut self, left: Expr) -> ParseResult<Expr> {
        self.identifier_to_arrow_function(left, false, None)
    }

    fn classify_arrow_head(&mut self) -> ParseResult<ArrowHead> {
        if self.current_token.tag != TokenTag::LeftParen {
            return Ok(ArrowHead::No);
        }
        self.classify_paren_arrow_head()
    }

    fn classify_paren_arrow_head(&mut self) -> ParseResult<ArrowHead> {
        let mut peek = self.begin_peek();
        let result = (|| {
            let second = peek.next()?;
            Ok(match second.tag {
                TokenTag::Spread => ArrowHead::Yes,
                TokenTag::LeftBracket | TokenTag::LeftBrace => ArrowHead::Maybe,
                TokenTag::RightParen => {
                    let third = peek.next()?;
                    if matches!(third.tag, TokenTag::Arrow | TokenTag::Colon) {
                        ArrowHead::Yes
                    } else {
                        ArrowHead::No
                    }
                }
                TokenTag::This => {
                    let third = peek.next()?;
                    if third.tag == TokenTag::Colon {
                        ArrowHead::Yes
                    } else {
                        ArrowHead::No
                    }
                }
                _ if second.tag.is_identifier_like() => {
                    let third = peek.next()?;
                    match third.tag {
                        TokenTag::Colon => ArrowHead::Yes,
                        TokenTag::Question => {
                            let fourth = peek.next()?;
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
            })
        })();
        peek.end();
        result
    }

    fn try_parse_arrow(&mut self, is_async: bool, start: u32) -> ParseResult<Option<Expr>> {
        let checkpoint = self.checkpoint();
        let arrow = match self.parse_arrow_inner(is_async, start) {
            Ok(arrow) => arrow,
            Err(_) => {
                self.rewind(checkpoint);
                return Ok(None);
            }
        };
        let return_type = arrow
            .as_arrow_function_expression(self.ast)
            .expect("arrow speculation produces an arrow function")
            .return_type(self.ast);
        if !self.ts_context.allow_arrow_return_type
            && return_type.is_some()
            && self.current_token.tag != TokenTag::Colon
        {
            self.rewind(checkpoint);
            return Ok(None);
        }
        Ok(Some(arrow))
    }

    fn try_parse_generic_arrow(&mut self, is_async: bool, start: u32) -> ParseResult<Option<Expr>> {
        if self.current_token.tag != TokenTag::LessThan {
            return Ok(None);
        }
        let next = self.peek_ahead()?;
        if !next.tag.is_identifier_like() && next.tag != TokenTag::Const {
            return Ok(None);
        }
        self.try_parse_arrow(is_async, start)
    }

    fn parse_arrow_inner(&mut self, is_async: bool, start: u32) -> ParseResult<Expr> {
        let old_context = self.context;
        if is_async {
            self.context.await_ = true;
        }
        let result = (|| {
            let type_parameters = if is_angle_open(self.current_token.tag) {
                Some(self.parse_type_parameters()?)
            } else {
                None
            };
            if self.current_token.tag != TokenTag::LeftParen {
                return self.error("expected '(' in arrow function");
            }
            let params =
                self.parse_formal_parameters(FormalParameterKind::ArrowFormalParameters, false)?;
            let return_type = if self.current_token.tag == TokenTag::Colon {
                Some(self.parse_return_type_annotation()?)
            } else {
                None
            };
            if self.current_token.tag != TokenTag::Arrow
                || self.current_token.has_line_terminator_before()
            {
                return self.error("expected '=>' in arrow function");
            }
            self.advance()?;
            self.build_arrow_function(start, params, is_async, type_parameters, return_type)
        })();
        self.context = old_context;
        result
    }

    fn parse_arrow(
        &mut self,
        start: u32,
        parameters: Vec<FormalParameterItem>,
        rest: Option<BindingRestElement>,
        async_function: bool,
    ) -> ParseResult<Expr> {
        let parameter_end = self.prev_token_end;
        let parameters = self.ast.add_typed_sub_range(parameters);
        let parameters = self.ast.formal_parameters(
            Span::new(start, parameter_end),
            parameters,
            rest,
            FormalParameterKind::ArrowFormalParameters,
        );
        self.build_arrow_function(start, parameters, async_function, None, None)
    }

    fn build_arrow_function(
        &mut self,
        start: u32,
        parameters: FormalParameters,
        async_function: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ParseResult<Expr> {
        let old_context = self.context;
        self.context.return_ = true;
        self.context.await_ = async_function;
        self.context.yield_ = false;
        let ArrowBodyResult {
            body,
            is_expression,
        } = self.parse_arrow_body()?;
        let end = body.span(self.ast).end;
        self.context = old_context;
        Ok(self.ast.expr_arrow_function_expression(
            Span::new(start, end),
            is_expression,
            async_function,
            parameters,
            body,
            type_parameters,
            return_type,
        ))
    }

    fn parse_arrow_body(&mut self) -> ParseResult<ArrowBodyResult> {
        if self.current_token.tag == TokenTag::LeftBrace {
            let body = self.parse_function_body()?;
            return Ok(ArrowBodyResult {
                body: ArrowFunctionBody::FunctionBody(body),
                is_expression: false,
            });
        }
        let expression =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        Ok(ArrowBodyResult {
            body: ArrowFunctionBody::Expr(expression),
            is_expression: true,
        })
    }

    fn expression_to_parameter(&mut self, expression: Expr) -> ParseResult<FormalParameterItem> {
        let pattern = self.expression_to_binding_pattern(expression)?;
        let span = pattern.span(self.ast);
        let pattern = FormalParameterPattern::BindingPattern(pattern);
        let parameter = self.ast.formal_parameter(span, pattern);
        Ok(FormalParameterItem::FormalParameter(parameter))
    }

    fn expression_to_binding_pattern(&mut self, expression: Expr) -> ParseResult<BindingPattern> {
        self.cover_to_pattern(expression, PatternContext::Binding)
    }

    fn expression_to_assignment_pattern(
        &mut self,
        expression: Expr,
    ) -> ParseResult<BindingPattern> {
        self.cover_to_pattern(expression, PatternContext::Assignment)
    }

    fn expression_to_pattern(
        &mut self,
        expression: Expr,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(identifier) if context == PatternContext::Assignment => {
                Ok(BindingPattern::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::IdentifierReference(identifier),
                ))
            }
            ExprData::IdentifierReference(identifier) => {
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_binding_identifier(
                    identifier.span(self.ast),
                    identifier.name(self.ast),
                    decorators,
                    None,
                    false,
                    None,
                ))
            }
            ExprData::MemberExpression(member)
                if context == PatternContext::Assignment
                    && !member_expression_is_optional(member, self.ast) =>
            {
                Ok(BindingPattern::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::MemberExpression(member),
                ))
            }
            ExprData::TsAsExpression(_)
            | ExprData::TsSatisfiesExpression(_)
            | ExprData::TsTypeAssertion(_)
            | ExprData::TsNonNullExpression(_)
                if context == PatternContext::Assignment =>
            {
                Ok(BindingPattern::SimpleAssignmentTarget(
                    self.expression_to_simple_assignment_target(expression)?,
                ))
            }
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(self.ast) == AssignmentOperator::Assign =>
            {
                let left = self.assignment_target_to_pattern(assignment.left(self.ast), context)?;
                let right = assignment.right(self.ast);
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_assignment_pattern(
                    assignment.span(self.ast),
                    left,
                    right,
                    decorators,
                    None,
                    false,
                ))
            }
            ExprData::ArrayExpression(array) => self.to_array_pattern(array, context),
            ExprData::ObjectExpression(object) => self.to_object_pattern(object, context),
            ExprData::ParenthesizedExpression(_) if context == PatternContext::Binding => self
                .error_at(
                    expression.span(self.ast),
                    "parentheses are not allowed in a binding pattern",
                ),
            ExprData::ParenthesizedExpression(parenthesized) => {
                let inner = parenthesized.expression(self.ast);
                if !matches!(
                    self.ast.expr_data(inner),
                    ExprData::IdentifierReference(_) | ExprData::MemberExpression(_)
                ) {
                    return self.error_at(
                        inner.span(self.ast),
                        "a parenthesized assignment target must be an identifier or member expression",
                    );
                }
                self.expression_to_pattern(inner, context)
            }
            _ => self.error_at(expression.span(self.ast), "invalid arrow parameter"),
        }
    }

    fn cover_to_pattern(
        &mut self,
        expression: Expr,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        match self.ast.expr_data(expression) {
            ExprData::ArrayExpression(array) => self.to_array_pattern_impl(array, context),
            ExprData::ObjectExpression(object) => self.to_object_pattern_impl(object, context),
            _ => self.expression_to_pattern(expression, context),
        }
    }

    fn to_array_pattern(
        &mut self,
        array: ArrayExpression,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        self.to_array_pattern_impl(array, context)
    }

    fn to_array_pattern_impl(
        &mut self,
        array: ArrayExpression,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        let source_elements = array.elements(self.ast);
        let source_elements = (0..source_elements.len())
            .map(|index| source_elements.get_node(self.ast, index).unwrap())
            .collect::<Vec<_>>();
        let source_elements_len = source_elements.len();
        let mut elements = Vec::new();
        let mut rest = None;
        for (index, argument) in source_elements.into_iter().enumerate() {
            let Some(argument) = argument else {
                elements.push(None);
                continue;
            };
            match self.ast.argument_data(argument) {
                ArgumentData::Expr(expression) => {
                    elements.push(Some(self.expression_to_pattern(expression, context)?));
                }
                ArgumentData::SpreadElement(spread) => {
                    if index + 1 != source_elements_len {
                        return self.error_at(spread.span(self.ast), "a rest element must be last");
                    }
                    if self.state.cover_has_trailing_comma == Some(array.span(self.ast).start) {
                        self.state.cover_has_trailing_comma = None;
                        return self.error_at(
                            spread.span(self.ast),
                            "a rest element may not have a trailing comma",
                        );
                    }
                    let argument =
                        self.expression_to_pattern(spread.argument(self.ast), context)?;
                    if matches!(
                        self.ast.binding_pattern_data(argument),
                        BindingPatternData::AssignmentPattern(_)
                    ) {
                        return self.error_at(
                            argument.span(self.ast),
                            "a rest element may not have an initializer",
                        );
                    }
                    let decorators = self.empty_range();
                    rest = Some(self.ast.binding_rest_element(
                        spread.span(self.ast),
                        argument,
                        decorators,
                        None,
                        false,
                    ));
                }
            }
        }
        let elements = self.ast.add_typed_sub_range(elements);
        let decorators = self.empty_range();
        Ok(self.ast.binding_pattern_array_pattern(
            array.span(self.ast),
            elements,
            rest,
            decorators,
            None,
            false,
        ))
    }

    fn to_object_pattern(
        &mut self,
        object: ObjectExpression,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        self.to_object_pattern_impl(object, context)
    }

    fn to_object_pattern_impl(
        &mut self,
        object: ObjectExpression,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        let source_properties = object.properties(self.ast);
        let source_properties = (0..source_properties.len())
            .map(|index| source_properties.get_node(self.ast, index).unwrap())
            .collect::<Vec<_>>();
        let source_properties_len = source_properties.len();
        let mut properties = Vec::new();
        let mut rest = None;
        for (index, property) in source_properties.into_iter().enumerate() {
            match self.ast.object_property_kind_data(property) {
                ObjectPropertyKindData::ObjectProperty(property) => {
                    if property.method(self.ast) || property.kind(self.ast) != PropertyKind::Init {
                        return self.error_at(
                            property.span(self.ast),
                            "methods are not allowed in patterns",
                        );
                    }
                    let value = self.expression_to_pattern(property.value(self.ast), context)?;
                    properties.push(self.ast.binding_property(
                        property.span(self.ast),
                        property.key(self.ast),
                        value,
                        property.shorthand(self.ast),
                        property.computed(self.ast),
                    ));
                }
                ObjectPropertyKindData::SpreadElement(spread) => {
                    if index + 1 != source_properties_len {
                        return self
                            .error_at(spread.span(self.ast), "a rest property must be last");
                    }
                    if self.state.cover_has_trailing_comma == Some(object.span(self.ast).start) {
                        self.state.cover_has_trailing_comma = None;
                        return self.error_at(
                            spread.span(self.ast),
                            "a rest property may not have a trailing comma",
                        );
                    }
                    let argument =
                        self.expression_to_pattern(spread.argument(self.ast), context)?;
                    if matches!(
                        self.ast.binding_pattern_data(argument),
                        BindingPatternData::AssignmentPattern(_)
                    ) {
                        return self.error_at(
                            argument.span(self.ast),
                            "a rest property may not have an initializer",
                        );
                    }
                    let decorators = self.empty_range();
                    rest = Some(self.ast.binding_rest_element(
                        spread.span(self.ast),
                        argument,
                        decorators,
                        None,
                        false,
                    ));
                }
            }
        }
        let properties = self.ast.add_typed_sub_range(properties);
        let decorators = self.empty_range();
        Ok(self.ast.binding_pattern_object_pattern(
            object.span(self.ast),
            properties,
            rest,
            decorators,
            None,
            false,
        ))
    }

    fn assignment_target_to_pattern(
        &mut self,
        target: AssignmentTarget,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        match self.ast.assignment_target_data(target) {
            AssignmentTargetData::SimpleAssignmentTarget(target) => {
                match self.ast.simple_assignment_target_data(target) {
                    SimpleAssignmentTargetData::IdentifierReference(_)
                        if context == PatternContext::Assignment =>
                    {
                        Ok(BindingPattern::SimpleAssignmentTarget(target))
                    }
                    SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                        let decorators = self.empty_range();
                        Ok(self.ast.binding_pattern_binding_identifier(
                            identifier.span(self.ast),
                            identifier.name(self.ast),
                            decorators,
                            None,
                            false,
                            None,
                        ))
                    }
                    SimpleAssignmentTargetData::MemberExpression(_)
                        if context == PatternContext::Assignment =>
                    {
                        Ok(BindingPattern::SimpleAssignmentTarget(target))
                    }
                    _ => self.error_at(target.span(self.ast), "invalid binding target"),
                }
            }
            AssignmentTargetData::ArrayAssignmentTarget(pattern) => {
                Ok(BindingPattern::ArrayPattern(pattern))
            }
            AssignmentTargetData::ObjectAssignmentTarget(pattern) => {
                Ok(BindingPattern::ObjectPattern(pattern))
            }
        }
    }

    fn is_destructuring_pattern(&self, pattern: BindingPattern) -> bool {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::ArrayPattern(_) | BindingPatternData::ObjectPattern(_) => true,
            BindingPatternData::AssignmentPattern(pattern) => {
                self.is_destructuring_pattern(pattern.left(self.ast))
            }
            _ => false,
        }
    }

    fn finish_cover_expression(&mut self, expression: Expr, in_cover: bool) -> ParseResult<Expr> {
        let needs_validation =
            !in_cover && self.state.cover_has_init_name && !self.is_part_of_pattern();
        if !in_cover {
            self.state.cover_has_init_name = false;
        }
        if needs_validation {
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        Ok(expression)
    }

    fn is_part_of_pattern(&self) -> bool {
        matches!(
            self.current_token.tag,
            TokenTag::Assign | TokenTag::In | TokenTag::Of
        )
    }

    fn validate_no_cover_initialized_syntax(&mut self, expression: Expr) -> ParseResult<()> {
        match self.ast.expr_data(expression) {
            ExprData::ObjectExpression(object) => {
                for id in object.properties(self.ast).iter() {
                    let property = self.ast.get_node_in_sub_range(id);
                    match self.ast.object_property_kind_data(property) {
                        ObjectPropertyKindData::ObjectProperty(property) => {
                            let value = property.value(self.ast);
                            if property.shorthand(self.ast) && self.is_cover_initialized_name(value)
                            {
                                self.report_cover_initialized_name_error(property.span(self.ast));
                            }
                            self.validate_no_cover_initialized_syntax(value)?;
                        }
                        ObjectPropertyKindData::SpreadElement(spread) => {
                            self.validate_no_cover_initialized_syntax(spread.argument(self.ast))?;
                        }
                    }
                }
            }
            ExprData::ArrayExpression(array) => {
                for index in 0..array.elements(self.ast).len() {
                    let Some(argument) =
                        array.elements(self.ast).get_node(self.ast, index).unwrap()
                    else {
                        continue;
                    };
                    let expression = match self.ast.argument_data(argument) {
                        ArgumentData::Expr(expression) => expression,
                        ArgumentData::SpreadElement(spread) => spread.argument(self.ast),
                    };
                    self.validate_no_cover_initialized_syntax(expression)?;
                }
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.validate_no_cover_initialized_syntax(parenthesized.expression(self.ast))?;
            }
            ExprData::SequenceExpression(sequence) => {
                for index in 0..sequence.expressions(self.ast).len() {
                    let expression = sequence
                        .expressions(self.ast)
                        .get_node(self.ast, index)
                        .unwrap();
                    self.validate_no_cover_initialized_syntax(expression)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn is_cover_initialized_name(&self, expression: Expr) -> bool {
        matches!(
            self.ast.expr_data(expression),
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(self.ast) == AssignmentOperator::Assign
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

    fn parse_array_expression(&mut self, in_cover: bool) -> ParseResult<Expr> {
        let cover = self.parse_cover()?;
        debug_assert!(matches!(cover, Cover::Array(_)));
        self.cover_to_expression(cover, in_cover)
    }

    fn parse_object_expression(&mut self, in_cover: bool) -> ParseResult<Expr> {
        let cover = self.parse_cover()?;
        debug_assert!(matches!(cover, Cover::Object(_)));
        self.cover_to_expression(cover, in_cover)
    }

    fn parse_cover(&mut self) -> ParseResult<Cover> {
        match self.current_token.tag {
            TokenTag::LeftBracket => self.parse_array_cover().map(Cover::Array),
            TokenTag::LeftBrace => self.parse_object_cover().map(Cover::Object),
            _ => self.error("expected array or object cover"),
        }
    }

    fn parse_array_cover(&mut self) -> ParseResult<ArrayCover> {
        let start = self.expect(TokenTag::LeftBracket)?.span.start;
        let mut elements = Vec::new();
        while self.current_token.tag != TokenTag::RightBracket {
            if self.eat(TokenTag::Comma)? {
                elements.push(None);
                continue;
            }
            let argument = if self.eat(TokenTag::Spread)? {
                let spread_start = self.prev_token_end - 3;
                let expression = self.parse_assignment_expression_in_cover()?;
                let span = Span::new(spread_start, expression.span(self.ast).end);
                Argument::SpreadElement(self.ast.spread_element(span, expression))
            } else {
                Argument::Expr(self.parse_assignment_expression_in_cover()?)
            };
            elements.push(Some(argument));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
            if self.current_token.tag == TokenTag::RightBracket {
                self.state.cover_has_trailing_comma = Some(start);
            }
        }
        let end = self.expect(TokenTag::RightBracket)?.span.end;
        let elements = self.ast.add_typed_sub_range(elements);
        Ok(ArrayCover {
            elements,
            start,
            end,
        })
    }

    fn parse_object_cover(&mut self) -> ParseResult<ObjectCover> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut properties = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            if self.eat(TokenTag::Spread)? {
                let spread_start = self.prev_token_end - 3;
                let expression = self.parse_expression(
                    Precedence::ASSIGNMENT,
                    ParseExpressionOpts {
                        in_cover: true,
                        ..ParseExpressionOpts::default()
                    },
                )?;
                let spread = self.ast.spread_element(
                    Span::new(spread_start, expression.span(self.ast).end),
                    expression,
                );
                properties.push(ObjectPropertyKind::SpreadElement(spread));
            } else {
                properties.push(self.parse_cover_property()?);
            }
            if !self.eat(TokenTag::Comma)? {
                break;
            }
            if self.current_token.tag == TokenTag::RightBrace {
                self.state.cover_has_trailing_comma = Some(start);
            }
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let properties = self.ast.add_typed_sub_range(properties);
        Ok(ObjectCover {
            properties,
            start,
            end,
        })
    }

    fn cover_to_expression(&mut self, cover: Cover, in_cover: bool) -> ParseResult<Expr> {
        let expression = match cover {
            Cover::Array(cover) => self
                .ast
                .expr_array_expression(Span::new(cover.start, cover.end), cover.elements),
            Cover::Object(cover) => self
                .ast
                .expr_object_expression(Span::new(cover.start, cover.end), cover.properties),
        };
        self.finish_cover_expression(expression, in_cover)
    }

    fn parse_cover_property(&mut self) -> ParseResult<ObjectPropertyKind> {
        let start = self.current_token.span.start;
        let mut async_function = false;
        let mut generator = false;
        let mut kind = PropertyKind::Init;
        let mut key = None;

        if self.current_token.tag == TokenTag::Async {
            let token = self.current_token;
            self.advance()?;
            if !self.current_token.has_line_terminator_before()
                && Self::is_property_key_start(self.current_token.tag)
            {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                async_function = true;
            } else {
                let name = self.identifier_name(token)?;
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
            self.advance()?;
            if Self::is_property_key_start(self.current_token.tag) {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                kind = if token.tag == TokenTag::Get {
                    PropertyKind::Get
                } else {
                    PropertyKind::Set
                };
            } else {
                let name = self.identifier_name(token)?;
                key = Some(self.ast.property_key_identifier_name(token.span, name));
            }
        }

        let computed = self.current_token.tag == TokenTag::LeftBracket && key.is_none();
        let key = match key {
            Some(key) => key,
            None => self.parse_property_key()?,
        };
        if matches!(
            self.ast.property_key_data(key),
            PropertyKeyData::PrivateIdentifier(_)
        ) {
            return self.error_at(
                key.span(self.ast),
                "private names are not allowed in object literals",
            );
        }

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
            return self.error("expected '(' after method name");
        }

        let (value, shorthand) = if self.eat(TokenTag::Colon)? {
            (
                self.parse_expression(
                    Precedence::ASSIGNMENT,
                    ParseExpressionOpts {
                        in_cover: true,
                        ..ParseExpressionOpts::default()
                    },
                )?,
                false,
            )
        } else {
            let PropertyKeyData::IdentifierName(identifier) = self.ast.property_key_data(key)
            else {
                return self.error_at(key.span(self.ast), "expected ':' after property name");
            };
            if is_unconditionally_reserved_word(identifier.name(self.ast).as_str()) {
                return self.error_at(
                    identifier.span(self.ast),
                    "reserved words may not be used as shorthand properties",
                );
            }
            self.validate_identifier_name(
                identifier.span(self.ast),
                identifier.name(self.ast).as_str(),
                "an identifier",
            )?;
            let identifier = self.ast.expr_identifier_reference(
                identifier.span(self.ast),
                identifier.name(self.ast),
                None,
            );
            let value = if self.eat(TokenTag::Assign)? {
                self.state.cover_has_init_name = true;
                let right = self.parse_expression(
                    Precedence::ASSIGNMENT,
                    ParseExpressionOpts {
                        in_cover: true,
                        ..ParseExpressionOpts::default()
                    },
                )?;
                let target = self.expression_to_assignment_target(identifier)?;
                self.ast.expr_assignment_expression(
                    Span::new(identifier.span(self.ast).start, right.span(self.ast).end),
                    target,
                    right,
                    AssignmentOperator::Assign,
                )
            } else {
                identifier
            };
            (value, true)
        };
        let span = Span::new(start, value.span(self.ast).end);
        let property = self.ast.object_property(
            span,
            key,
            value,
            PropertyKind::Init,
            false,
            shorthand,
            computed,
        );
        Ok(ObjectPropertyKind::ObjectProperty(property))
    }

    fn parse_object_method_property(
        &mut self,
        start: u32,
        key: PropertyKey,
        computed: bool,
        kind: PropertyKind,
        async_function: bool,
        generator: bool,
    ) -> ParseResult<ObjectPropertyKind> {
        if generator && matches!(kind, PropertyKind::Get | PropertyKind::Set) {
            return self.error_at(
                Span::new(start, self.current_token.span.end),
                "an accessor may not be a generator",
            );
        }
        let function_start = self.current_token.span.start;
        let old_context = self.context;
        self.context.return_ = true;
        self.context.await_ = async_function;
        self.context.yield_ = generator;
        let result = (|| {
            let type_parameters =
                if self.lang.is_typescript() && is_angle_open(self.current_token.tag) {
                    Some(self.parse_type_parameters()?)
                } else {
                    None
                };
            let params =
                self.parse_formal_parameters(FormalParameterKind::UniqueFormalParameters, false)?;
            self.check_accessor_arity(kind, params)?;
            let return_type =
                if self.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_return_type_annotation()?)
                } else {
                    None
                };
            let body = self.parse_function_body()?;
            Ok((params, type_parameters, return_type, body))
        })();
        self.context = old_context;
        let (params, type_parameters, return_type, body) = result?;
        let span = Span::new(start, body.span(self.ast).end);
        let function = self.ast.function(
            Span::new(function_start, body.span(self.ast).end),
            FunctionType::FunctionExpression,
            None,
            generator,
            async_function,
            false,
            params,
            Some(body),
            type_parameters,
            return_type,
        );
        let value = Expr::Function(function);
        Ok(ObjectPropertyKind::ObjectProperty(
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
            TokenTag::Star
                | TokenTag::LeftBracket
                | TokenTag::StringLiteral
                | TokenTag::PrivateIdentifier
        ) || tag.is_numeric_literal()
            || tag.is_identifier_like()
    }

    fn parse_class_element_key(&mut self) -> ParseResult<PropertyKey> {
        self.parse_property_key()
    }

    fn parse_property_key(&mut self) -> ParseResult<PropertyKey> {
        let token = self.current_token;
        match token.tag {
            TokenTag::LeftBracket => {
                self.advance()?;
                let expression =
                    self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
                self.expect(TokenTag::RightBracket)?;
                Ok(PropertyKey::Expr(expression))
            }
            TokenTag::PrivateIdentifier => Ok(PropertyKey::PrivateIdentifier(
                self.parse_private_identifier()?,
            )),
            TokenTag::StringLiteral => {
                let raw = self.atom_from_span(token.span);
                let value = self.string_value(token)?;
                self.advance()?;
                Ok(self.ast.property_key_string_literal(token.span, value, raw))
            }
            tag if tag.is_numeric_literal() => {
                let kind = match tag {
                    TokenTag::HexLiteral => NumericLiteralKind::Hex,
                    TokenTag::OctalLiteral => NumericLiteralKind::Octal,
                    TokenTag::BinaryLiteral => NumericLiteralKind::Binary,
                    _ => NumericLiteralKind::Decimal,
                };
                let raw = self.atom_from_span(token.span);
                self.advance()?;
                Ok(self.ast.property_key_numeric_literal(token.span, kind, raw))
            }
            tag if tag.is_identifier_like() => {
                Ok(PropertyKey::IdentifierName(self.parse_identifier_name()?))
            }
            _ => self.error("expected a property name"),
        }
    }

    fn parse_computed_member_expression(
        &mut self,
        object: Expr,
        optional: bool,
    ) -> ParseResult<Expr> {
        let start = object.span(self.ast).start;
        self.expect(TokenTag::LeftBracket)?;
        let expression =
            self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        let property = PropertyKey::Expr(expression);
        let end = self.expect(TokenTag::RightBracket)?.span.end;
        Ok(self
            .ast
            .expr_member_expression(Span::new(start, end), object, property, true, optional))
    }

    fn parse_static_member_expression(
        &mut self,
        object: Expr,
        optional: bool,
    ) -> ParseResult<Expr> {
        self.expect(TokenTag::Dot)?;
        self.parse_member_property(object, optional)
    }

    fn parse_member_property(&mut self, object: Expr, optional: bool) -> ParseResult<Expr> {
        let start = object.span(self.ast).start;
        let token = self.current_token;
        if !token.tag.is_identifier_like() && token.tag != TokenTag::PrivateIdentifier {
            return self.error("expected an identifier name after '.'");
        }
        let property = if token.tag == TokenTag::PrivateIdentifier {
            PropertyKey::PrivateIdentifier(self.parse_private_identifier()?)
        } else {
            PropertyKey::IdentifierName(self.parse_identifier_name()?)
        };
        Ok(self.ast.expr_member_expression(
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
    ) -> ParseResult<Expr> {
        let cover = self.parse_parenthesized_cover()?;
        self.cover_to_call_expression(cover, callee, optional, type_arguments)
    }

    fn cover_to_call_expression(
        &mut self,
        cover: ParenthesizedCover,
        callee: Expr,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ParseResult<Expr> {
        let arguments = (0..cover.elements.len())
            .map(|index| {
                cover
                    .elements
                    .get_node(self.ast, index)
                    .expect("call cover contains its arguments")
            })
            .collect::<Vec<_>>();
        for argument in arguments {
            let expression = match self.ast.argument_data(argument) {
                ArgumentData::Expr(expression) => expression,
                ArgumentData::SpreadElement(spread) => spread.argument(self.ast),
            };
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        Ok(self.ast.expr_call_expression(
            Span::new(callee.span(self.ast).start, cover.end),
            callee,
            cover.elements,
            optional,
            type_arguments,
        ))
    }

    fn parse_optional_chain(&mut self, left: Expr) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
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
                    return self.error(
                        "tagged template expressions are not permitted in an optional chain",
                    );
                }
                _ => break,
            };
        }
        Ok(self
            .ast
            .expr_chain_expression(Span::new(start, expression.span(self.ast).end), expression))
    }

    fn parse_optional_chain_element(&mut self, object: Expr, optional: bool) -> ParseResult<Expr> {
        match self.current_token.tag {
            tag if tag.is_identifier_like() || tag == TokenTag::PrivateIdentifier => {
                self.parse_member_property(object, optional)
            }
            TokenTag::LeftBracket => self.parse_computed_member_expression(object, optional),
            TokenTag::LeftParen => self.parse_call_expression(object, optional, None),
            TokenTag::LessThan | TokenTag::LeftShift if self.lang.is_typescript() => {
                let Some(type_arguments) = self.try_parse_type_arguments_in_expression()? else {
                    return self.error("expected type arguments after '?.'");
                };
                self.parse_call_expression(object, optional, Some(type_arguments))
            }
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                self.error("tagged template expressions are not permitted in an optional chain")
            }
            _ => self.error("expected a property name, '[' or '(' after '?.'"),
        }
    }

    fn parse_arguments(&mut self) -> ParseResult<TypedSubRange<Argument>> {
        self.expect(TokenTag::LeftParen)?;
        let mut arguments = Vec::new();
        while self.current_token.tag != TokenTag::RightParen {
            let argument = if self.eat(TokenTag::Spread)? {
                let start = self.prev_token_end - 3;
                let expression =
                    self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
                Argument::SpreadElement(
                    self.ast.spread_element(
                        Span::new(start, expression.span(self.ast).end),
                        expression,
                    ),
                )
            } else {
                Argument::Expr(
                    self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?,
                )
            };
            arguments.push(argument);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        self.expect(TokenTag::RightParen)?;
        Ok(self.ast.add_typed_sub_range(arguments))
    }

    fn parse_new_expression(&mut self) -> ParseResult<Expr> {
        if self.peek_ahead()?.tag == TokenTag::Dot {
            return self.parse_new_target();
        }
        let start = self.expect(TokenTag::New)?.span.start;
        let mut callee = match self.current_token.tag {
            TokenTag::LeftParen => {
                self.parse_parenthesized_or_arrow_function(None, false, Precedence::GROUPING)?
            }
            TokenTag::New => self.parse_new_expression()?,
            TokenTag::Import => self.parse_import_expression()?,
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
                    return self.error("optional chaining is not allowed in a new expression");
                }
                _ => break,
            };
        }
        if matches!(self.ast.expr_data(callee), ExprData::ImportExpression(_)) {
            return self.error_at(callee.span(self.ast), "import() cannot be used with new");
        }
        let type_arguments = if self.lang.is_typescript() {
            self.try_parse_type_arguments_in_expression()?
        } else {
            None
        };
        let arguments = if self.current_token.tag == TokenTag::LeftParen {
            self.parse_arguments()?
        } else {
            self.empty_range()
        };
        let end = self.prev_token_end.max(callee.span(self.ast).end);
        Ok(self
            .ast
            .expr_new_expression(Span::new(start, end), callee, arguments, type_arguments))
    }

    fn parse_import_expression(&mut self) -> ParseResult<Expr> {
        let import_token = self.expect(TokenTag::Import)?;
        let start = import_token.span.start;
        if self.current_token.tag == TokenTag::Dot {
            return self.parse_import_meta_or_phase_import(import_token);
        }
        self.parse_dynamic_import(start, None)
    }

    fn parse_import_meta_or_phase_import(&mut self, import_token: Token) -> ParseResult<Expr> {
        self.expect(TokenTag::Dot)?;
        let property = self.current_token;
        let phase = match property.tag {
            TokenTag::Source => Some(ImportPhase::Source),
            TokenTag::Defer => Some(ImportPhase::Defer),
            TokenTag::Identifier if property.text(self.source) == b"meta" => {
                let meta_name = self.identifier_name(import_token)?;
                let property_name = self.identifier_name(property)?;
                self.advance()?;
                let meta = self.ast.identifier_name(import_token.span, meta_name);
                let property = self.ast.identifier_name(property.span, property_name);
                return Ok(self.ast.expr_meta_property(
                    Span::new(import_token.span.start, property.span(self.ast).end),
                    meta,
                    property,
                ));
            }
            _ => return self.error("expected 'meta', 'source', or 'defer' after import."),
        };
        self.advance()?;
        self.parse_dynamic_import(import_token.span.start, phase)
    }

    fn parse_dynamic_import(
        &mut self,
        start: u32,
        phase: Option<ImportPhase>,
    ) -> ParseResult<Expr> {
        self.expect(TokenTag::LeftParen)?;
        let source =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        let options = if phase.is_none() && self.eat(TokenTag::Comma)? {
            let options = if self.current_token.tag == TokenTag::RightParen {
                None
            } else {
                Some(self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?)
            };
            self.eat(TokenTag::Comma)?;
            options
        } else {
            None
        };
        let end = self.expect(TokenTag::RightParen)?.span.end;
        Ok(self
            .ast
            .expr_import_expression(Span::new(start, end), source, options, phase))
    }

    fn parse_new_target(&mut self) -> ParseResult<Expr> {
        let meta_token = self.current_token;
        if meta_token.is_escaped() {
            return self.error_at(
                meta_token.span,
                "meta-property keywords may not contain escape sequences",
            );
        }
        let meta_name = self.identifier_name(meta_token)?;
        self.advance()?;
        self.expect(TokenTag::Dot)?;
        let property_token = self.current_token;
        if !property_token.tag.is_identifier_like()
            || self
                .source
                .get(property_token.span.start as usize..property_token.span.end as usize)
                != Some(b"target")
        {
            return self.error("the only valid meta-property for 'new' is 'new.target'");
        }
        if property_token.is_escaped() {
            return self.error_at(
                property_token.span,
                "meta-property names may not contain escape sequences",
            );
        }
        let property_name = self.identifier_name(property_token)?;
        self.advance()?;
        let meta = self.ast.identifier_name(meta_token.span, meta_name);
        let property = self.ast.identifier_name(property_token.span, property_name);
        Ok(self.ast.expr_meta_property(
            Span::new(meta_token.span.start, property_token.span.end),
            meta,
            property,
        ))
    }

    fn parse_jsx_expression(&mut self) -> ParseResult<Expr> {
        self.parse_jsx_element(JsxElementContext::TopLevel)
    }

    fn enter_jsx_tag(&mut self) {
        self.set_lexer_mode(LexerMode::JsxTag);
    }

    fn exit_jsx_tag(&mut self) {
        self.set_lexer_mode(LexerMode::Normal);
    }

    fn parse_jsx_element(&mut self, context: JsxElementContext) -> ParseResult<Expr> {
        let start = self.current_token.span.start;
        if self.peek_ahead()?.tag == TokenTag::GreaterThan {
            return self.parse_jsx_fragment(start);
        }

        let opening = self.parse_jsx_opening_element(context)?;
        if opening.self_closing(self.ast) {
            let children = self.empty_range();
            return Ok(Expr::JsxElement(self.ast.jsx_element(
                Span::new(start, opening.span(self.ast).end),
                opening,
                children,
                None,
            )));
        }
        let children = self.parse_jsx_children(opening.span(self.ast).end)?;
        let closing = self.parse_jsx_closing_element(opening.name(self.ast), context)?;
        Ok(Expr::JsxElement(self.ast.jsx_element(
            Span::new(start, closing.span(self.ast).end),
            opening,
            children,
            Some(closing),
        )))
    }

    fn parse_jsx_opening_element(
        &mut self,
        context: JsxElementContext,
    ) -> ParseResult<JsxOpeningElement> {
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
            return self.error("expected '>' to close JSX opening element");
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
        Ok(self.ast.jsx_opening_element(
            Span::new(start, end),
            name,
            attributes,
            self_closing,
            type_arguments,
        ))
    }

    fn parse_jsx_closing_element(
        &mut self,
        opening_name: JsxElementName,
        context: JsxElementContext,
    ) -> ParseResult<JsxClosingElement> {
        if self.current_token.tag != TokenTag::LessThan {
            return self.error("expected '</' to close JSX element");
        }
        let closing_start = self.current_token.span.start;
        self.enter_jsx_tag();
        self.advance()?;
        self.expect(TokenTag::Slash)?;
        let closing_name = self.parse_jsx_element_name()?;
        if self.current_token.tag != TokenTag::GreaterThan {
            return self.error("expected '>' to close JSX closing element");
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
        let closing_span = closing_name.span(self.ast);
        if !self.jsx_names_match(opening_name, closing_name) {
            return self.error_at(closing_span, "JSX closing tag does not match");
        }
        Ok(self
            .ast
            .jsx_closing_element(Span::new(closing_start, end), closing_name))
    }

    fn jsx_names_match(&self, a: JsxElementName, b: JsxElementName) -> bool {
        let a = a.span(self.ast);
        let b = b.span(self.ast);
        a.end - a.start == b.end - b.start && self.span_text(a) == self.span_text(b)
    }

    fn parse_jsx_fragment(&mut self, start: u32) -> ParseResult<Expr> {
        self.advance()?;
        if self.current_token.tag != TokenTag::GreaterThan {
            return self.error("expected '>' to close JSX opening fragment");
        }
        let opening_end = self.current_token.span.end;
        let opening = self.ast.jsx_opening_fragment(Span::new(start, opening_end));
        let children = self.parse_jsx_children(opening_end)?;
        let closing_start = self.current_token.span.start;
        self.enter_jsx_tag();
        self.advance()?;
        self.expect(TokenTag::Slash)?;
        if self.current_token.tag != TokenTag::GreaterThan {
            return self.error("expected '>' to close JSX fragment");
        }
        let closing_end = self.current_token.span.end;
        let closing = self
            .ast
            .jsx_closing_fragment(Span::new(closing_start, closing_end));
        self.exit_jsx_tag();
        self.advance()?;
        Ok(self
            .ast
            .expr_jsx_fragment(Span::new(start, closing_end), opening, children, closing))
    }

    fn parse_jsx_children(&mut self, mut scan_from: u32) -> ParseResult<TypedSubRange<JsxChild>> {
        let mut children = Vec::new();
        self.exit_jsx_tag();
        loop {
            let text = self.lexer.re_scan_jsx_text(scan_from);
            if text.span.start < text.span.end {
                let bytes = &self.source[text.span.start as usize..text.span.end as usize];
                let value = Wtf8::from_bytes(bytes)
                    .map_err(|_| Error::new(text.span, "JSX text is not valid WTF-8"))?;
                children.push(
                    self.ast
                        .jsx_child_jsx_text(text.span, self.ast.add_wtf8_atom(value)),
                );
            }
            self.current_token = text;
            self.advance()?;
            match self.current_token.tag {
                TokenTag::LessThan => {
                    if self.peek_ahead()?.tag == TokenTag::Slash {
                        break;
                    }
                    let child = self.parse_jsx_element(JsxElementContext::Child)?;
                    scan_from = child.span(self.ast).end;
                    match self.ast.expr_data(child) {
                        ExprData::JsxElement(element) => {
                            children.push(JsxChild::JsxElement(element));
                        }
                        ExprData::JsxFragment(fragment) => {
                            children.push(JsxChild::JsxFragment(fragment));
                        }
                        _ => unreachable!(),
                    }
                }
                TokenTag::LeftBrace => {
                    let child = self.parse_jsx_child_from_left_brace()?;
                    scan_from = child.span(self.ast).end;
                    children.push(child);
                }
                _ => break,
            }
        }
        Ok(self.ast.add_typed_sub_range(children))
    }

    fn parse_jsx_child_from_left_brace(&mut self) -> ParseResult<JsxChild> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        if self.current_token.tag == TokenTag::RightBrace {
            let empty_span = Span::new(start + 1, self.current_token.span.start);
            let empty = self.ast.jsx_empty_expression(empty_span);
            let expression = JsxExpression::JsxEmptyExpression(empty);
            let end = self.expect_jsx_child_right_brace()?;
            return Ok(self
                .ast
                .jsx_child_jsx_expression_container(Span::new(start, end), expression));
        }
        if self.eat(TokenTag::Spread)? {
            let expression =
                self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
            let end = self.expect_jsx_child_right_brace()?;
            return Ok(self
                .ast
                .jsx_child_jsx_spread_child(Span::new(start, end), expression));
        }
        let expression =
            self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        let end = self.expect_jsx_child_right_brace()?;
        Ok(self.ast.jsx_child_jsx_expression_container(
            Span::new(start, end),
            JsxExpression::Expr(expression),
        ))
    }

    fn expect_jsx_child_right_brace(&self) -> ParseResult<u32> {
        if self.current_token.tag != TokenTag::RightBrace {
            return self.error("expected '}' to close JSX expression");
        }
        Ok(self.current_token.span.end)
    }

    fn parse_jsx_element_name(&mut self) -> ParseResult<JsxElementName> {
        let token = self.current_token;
        if token.tag != TokenTag::JsxIdentifier {
            return self.error("expected a JSX element name");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        let identifier = self.ast.jsx_identifier(token.span, name);
        let start = token.span.start;
        let mut element_name = JsxElementName::JsxIdentifier(identifier);
        let mut member_object = JsxMemberExpressionObject::JsxIdentifier(identifier);
        let mut is_member = false;

        while self.eat(TokenTag::Dot)? {
            if self.current_token.tag != TokenTag::JsxIdentifier {
                return self.error("expected an identifier after '.'");
            }
            is_member = true;
            let property_token = self.current_token;
            let property_name = self.identifier_name(property_token)?;
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
                return self.error("expected an identifier after ':'");
            }
            let local_token = self.current_token;
            let local_name = self.identifier_name(local_token)?;
            self.advance()?;
            let local = self.ast.jsx_identifier(local_token.span, local_name);
            element_name = JsxElementName::JsxNamespacedName(self.ast.jsx_namespaced_name(
                Span::new(start, local_token.span.end),
                identifier,
                local,
            ));
        }
        Ok(element_name)
    }

    fn parse_jsx_attributes(&mut self) -> ParseResult<TypedSubRange<JsxAttributeItem>> {
        let mut attributes = Vec::new();
        while matches!(
            self.current_token.tag,
            TokenTag::JsxIdentifier | TokenTag::LeftBrace
        ) {
            attributes.push(self.parse_jsx_attribute()?);
        }
        Ok(self.ast.add_typed_sub_range(attributes))
    }

    fn parse_jsx_attribute(&mut self) -> ParseResult<JsxAttributeItem> {
        if self.current_token.tag == TokenTag::LeftBrace {
            return self.parse_jsx_spread_attribute();
        }
        let name = self.parse_jsx_attribute_name()?;
        let start = name.span(self.ast).start;
        let value = if self.eat(TokenTag::Assign)? {
            Some(self.parse_jsx_attribute_value()?)
        } else {
            None
        };
        let end = value.map_or(name.span(self.ast).end, |value| value.span(self.ast).end);
        Ok(JsxAttributeItem::JsxAttribute(self.ast.jsx_attribute(
            Span::new(start, end),
            name,
            value,
        )))
    }

    fn parse_jsx_attribute_name(&mut self) -> ParseResult<JsxAttributeName> {
        let token = self.current_token;
        if token.tag != TokenTag::JsxIdentifier {
            return self.error("expected a JSX attribute name");
        }
        let name = self.identifier_name(token)?;
        self.advance()?;
        let namespace = self.ast.jsx_identifier(token.span, name);
        if !self.eat(TokenTag::Colon)? {
            return Ok(JsxAttributeName::JsxIdentifier(namespace));
        }
        if self.current_token.tag != TokenTag::JsxIdentifier {
            return self.error("expected an identifier after ':'");
        }
        let local_token = self.current_token;
        let local_name = self.identifier_name(local_token)?;
        self.advance()?;
        let local = self.ast.jsx_identifier(local_token.span, local_name);
        Ok(JsxAttributeName::JsxNamespacedName(
            self.ast.jsx_namespaced_name(
                Span::new(token.span.start, local_token.span.end),
                namespace,
                local,
            ),
        ))
    }

    fn parse_jsx_attribute_value(&mut self) -> ParseResult<JsxAttributeValue> {
        match self.current_token.tag {
            TokenTag::StringLiteral => Ok(JsxAttributeValue::StringLiteral(
                self.parse_string_literal()?,
            )),
            TokenTag::LeftBrace => {
                let container = self.parse_jsx_expression_container(JsxExprContext::Tag)?;
                if container
                    .expression(self.ast)
                    .is_jsx_empty_expression(self.ast)
                {
                    return self.error_at(
                        container.span(self.ast),
                        "JSX attribute value cannot be an empty expression",
                    );
                }
                Ok(JsxAttributeValue::JsxExpressionContainer(container))
            }
            TokenTag::LessThan => {
                let expression = self.parse_jsx_element(JsxElementContext::Attribute)?;
                Ok(match self.ast.expr_data(expression) {
                    ExprData::JsxElement(element) => JsxAttributeValue::JsxElement(element),
                    ExprData::JsxFragment(fragment) => JsxAttributeValue::JsxFragment(fragment),
                    _ => unreachable!(),
                })
            }
            _ => self.error("expected a JSX attribute value"),
        }
    }

    fn parse_jsx_expression_container(
        &mut self,
        context: JsxExprContext,
    ) -> ParseResult<JsxExpressionContainer> {
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
            return Ok(self.ast.jsx_expression_container(
                Span::new(start, end),
                JsxExpression::JsxEmptyExpression(empty),
            ));
        }
        let expression =
            self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        let end = self.current_token.span.end;
        if context == JsxExprContext::Tag {
            self.enter_jsx_tag();
        }
        self.expect(TokenTag::RightBrace)?;
        Ok(self
            .ast
            .jsx_expression_container(Span::new(start, end), JsxExpression::Expr(expression)))
    }

    fn parse_jsx_spread_attribute(&mut self) -> ParseResult<JsxAttributeItem> {
        let start = self.current_token.span.start;
        self.exit_jsx_tag();
        self.advance()?;
        self.expect(TokenTag::Spread)?;
        let argument = self.parse_expression(Precedence::COMMA, ParseExpressionOpts::default())?;
        let end = self.current_token.span.end;
        self.enter_jsx_tag();
        self.expect(TokenTag::RightBrace)?;
        Ok(JsxAttributeItem::JsxSpreadAttribute(
            self.ast
                .jsx_spread_attribute(Span::new(start, end), argument),
        ))
    }

    fn parse_await_expression(&mut self, start: u32) -> ParseResult<Expr> {
        let argument = self.parse_expression(Precedence::UNARY, ParseExpressionOpts::default())?;
        if self.current_token.tag == TokenTag::Exponent {
            return self.error_at(
                Span::new(start, argument.span(self.ast).end),
                "The left-hand side of '**' cannot be an unparenthesized await expression",
            );
        }
        Ok(self
            .ast
            .expr_await_expression(Span::new(start, argument.span(self.ast).end), argument))
    }

    fn parse_type_argumented_call_or_instantiation(
        &mut self,
        callee: Expr,
    ) -> ParseResult<Option<Expr>> {
        let Some(type_arguments) = self.try_parse_type_arguments_in_expression()? else {
            return Ok(None);
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
                    callee.span(self.ast).start,
                    type_arguments.span(self.ast).end,
                ),
                callee,
                type_arguments,
            ),
        };
        Ok(Some(expression))
    }

    fn try_parse_type_arguments_in_expression(
        &mut self,
    ) -> ParseResult<Option<TsTypeParameterInstantiation>> {
        if !is_angle_open(self.current_token.tag) {
            return Ok(None);
        }
        let after = self.peek_ahead()?;
        if !is_start_of_type(after.tag) && after.tag != TokenTag::GreaterThan {
            return Ok(None);
        }

        let checkpoint = self.checkpoint();
        let type_arguments = match self.parse_type_arguments() {
            Ok(type_arguments) if can_follow_type_arguments_in_expression(self.current_token) => {
                type_arguments
            }
            Ok(_) | Err(_) => {
                self.rewind(checkpoint);
                return Ok(None);
            }
        };
        Ok(Some(type_arguments))
    }

    fn parse_type_assertion(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::LessThan)?.span.start;
        let type_annotation = self.parse_type()?;
        self.expect(TokenTag::GreaterThan)?;
        let expression =
            self.parse_expression(Precedence::UNARY, ParseExpressionOpts::default())?;
        Ok(self.ast.expr_ts_type_assertion(
            Span::new(start, expression.span(self.ast).end),
            type_annotation,
            expression,
        ))
    }

    fn parse_as_or_satisfies_expression(&mut self, left: Expr, tag: TokenTag) -> ParseResult<Expr> {
        self.advance()?;
        let type_annotation = self.parse_type()?;
        let span = Span::new(
            left.span(self.ast).start,
            type_annotation.span(self.ast).end,
        );
        Ok(if tag == TokenTag::As {
            self.ast.expr_ts_as_expression(span, left, type_annotation)
        } else {
            self.ast
                .expr_ts_satisfies_expression(span, left, type_annotation)
        })
    }

    fn parse_non_null_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        let end = self.expect(TokenTag::LogicalNot)?.span.end;
        Ok(self
            .ast
            .expr_ts_non_null_expression(Span::new(left.span(self.ast).start, end), left))
    }

    fn parse_yield_expression(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::Yield)?.span.start;
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
            Some(self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?)
        };
        if delegate && argument.is_none() {
            return self.error("expected an expression after 'yield*'");
        }
        let end = argument.map_or(self.prev_token_end, |expr| expr.span(self.ast).end);
        Ok(self
            .ast
            .expr_yield_expression(Span::new(start, end), argument, delegate))
    }

    fn parse_unary_expression(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        let argument = self.parse_expression(Precedence::UNARY, ParseExpressionOpts::default())?;
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
        Ok(self.ast.expr_unary_expression(
            Span::new(token.span.start, argument.span(self.ast).end),
            argument,
            operator,
        ))
    }

    fn parse_update_expression(&mut self, prefix: bool, left: Option<Expr>) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        let argument = if prefix {
            self.parse_expression(Precedence::UNARY, ParseExpressionOpts::default())?
        } else {
            left.expect("a postfix update expression has a left operand")
        };
        let start = if prefix {
            token.span.start
        } else {
            argument.span(self.ast).start
        };
        let target = self.expression_to_simple_assignment_target(argument)?;
        let operator = if token.tag == TokenTag::Increment {
            UpdateOperator::Increment
        } else {
            UpdateOperator::Decrement
        };
        let end = if prefix {
            argument.span(self.ast).end
        } else {
            self.prev_token_end
        };
        Ok(self
            .ast
            .expr_update_expression(Span::new(start, end), target, operator, prefix))
    }

    fn parse_binary_expression(&mut self, left: Expr, tag: TokenTag) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let precedence = tag.precedence();
        if tag == TokenTag::Exponent
            && matches!(
                self.ast.expr_data(left),
                ExprData::UnaryExpression(_) | ExprData::AwaitExpression(_)
            )
        {
            return self.error_at(
                left.span(self.ast),
                "A unary expression cannot be the left operand of exponentiation",
            );
        }
        self.advance()?;
        let right = self.parse_expression(
            if tag == TokenTag::Exponent {
                precedence
            } else {
                precedence + 1
            },
            ParseExpressionOpts::default(),
        )?;
        if matches!(self.ast.expr_data(right), ExprData::PrivateIdentifier(_)) {
            return self.error_at(
                right.span(self.ast),
                "Private names are only valid on the left of an 'in' expression",
            );
        }
        let operator = binary_operator(tag);
        Ok(self.ast.expr_binary_expression(
            Span::new(start, right.span(self.ast).end),
            left,
            right,
            operator,
        ))
    }

    fn parse_logical_expression(&mut self, left: Expr, tag: TokenTag) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let precedence = tag.precedence();
        self.advance()?;
        let right = self.parse_expression(precedence + 1, ParseExpressionOpts::default())?;
        let operator = match tag {
            TokenTag::LogicalAnd => LogicalOperator::And,
            TokenTag::LogicalOr => LogicalOperator::Or,
            TokenTag::NullishCoalescing => LogicalOperator::NullishCoalescing,
            _ => unreachable!(),
        };
        if logical_expression_mixes_coalesce(left, operator, self.ast)
            || logical_expression_mixes_coalesce(right, operator, self.ast)
        {
            return self.error_at(
                Span::new(start, right.span(self.ast).end),
                "Nullish coalescing cannot be mixed with logical AND or OR without parentheses",
            );
        }
        Ok(self.ast.expr_logical_expression(
            Span::new(start, right.span(self.ast).end),
            left,
            right,
            operator,
        ))
    }

    fn parse_assignment_expression(&mut self, left: Expr, tag: TokenTag) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let target = self.expression_to_assignment_target(left)?;
        if tag != TokenTag::Assign
            && !matches!(
                self.ast.assignment_target_data(target),
                AssignmentTargetData::SimpleAssignmentTarget(_)
            )
        {
            return self.error_at(
                target.span(self.ast),
                "A compound assignment requires a simple assignment target",
            );
        }
        self.advance()?;
        let right =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        Ok(self.ast.expr_assignment_expression(
            Span::new(start, right.span(self.ast).end),
            target,
            right,
            assignment_operator(tag),
        ))
    }

    fn parse_conditional_expression(&mut self, test: Expr) -> ParseResult<Expr> {
        let start = test.span(self.ast).start;
        self.expect(TokenTag::Question)?;
        let consequent =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        self.expect(TokenTag::Colon)?;
        let alternate =
            self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?;
        Ok(self.ast.expr_conditional_expression(
            Span::new(start, alternate.span(self.ast).end),
            test,
            consequent,
            alternate,
        ))
    }

    fn parse_sequence_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let mut expressions = vec![left];
        while self.eat(TokenTag::Comma)? {
            expressions.push(
                self.parse_expression(Precedence::ASSIGNMENT, ParseExpressionOpts::default())?,
            );
        }
        let end = expressions.last().unwrap().span(self.ast).end;
        let expressions = self.ast.add_typed_sub_range(expressions);
        Ok(self
            .ast
            .expr_sequence_expression(Span::new(start, end), expressions))
    }

    fn expression_to_simple_assignment_target(
        &self,
        expression: Expr,
    ) -> ParseResult<SimpleAssignmentTarget> {
        if !self.is_simple_assignment_target(expression) {
            return Err(Error::new(
                expression.span(self.ast),
                "invalid assignment target",
            ));
        }
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(node) => {
                Ok(SimpleAssignmentTarget::IdentifierReference(node))
            }
            ExprData::MemberExpression(node) if !member_expression_is_optional(node, self.ast) => {
                Ok(SimpleAssignmentTarget::MemberExpression(node))
            }
            ExprData::TsAsExpression(node) => {
                self.expression_to_simple_assignment_target(node.expression(self.ast))?;
                Ok(SimpleAssignmentTarget::TsAsExpression(node))
            }
            ExprData::TsSatisfiesExpression(node) => {
                self.expression_to_simple_assignment_target(node.expression(self.ast))?;
                Ok(SimpleAssignmentTarget::TsSatisfiesExpression(node))
            }
            ExprData::TsTypeAssertion(node) => {
                self.expression_to_simple_assignment_target(node.expression(self.ast))?;
                Ok(SimpleAssignmentTarget::TsTypeAssertion(node))
            }
            ExprData::TsNonNullExpression(node) => {
                self.expression_to_simple_assignment_target(node.expression(self.ast))?;
                Ok(SimpleAssignmentTarget::TsNonNullExpression(node))
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.expression_to_simple_assignment_target(parenthesized.expression(self.ast))
            }
            _ => unreachable!("simple assignment targets were validated above"),
        }
    }

    fn is_simple_assignment_target(&self, expression: Expr) -> bool {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(_) => true,
            ExprData::MemberExpression(member) => !member_expression_is_optional(member, self.ast),
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.is_simple_assignment_target(parenthesized.expression(self.ast))
            }
            ExprData::TsNonNullExpression(node) => {
                self.is_simple_assignment_target(node.expression(self.ast))
            }
            ExprData::TsAsExpression(node) => {
                self.is_simple_assignment_target(node.expression(self.ast))
            }
            ExprData::TsSatisfiesExpression(node) => {
                self.is_simple_assignment_target(node.expression(self.ast))
            }
            ExprData::TsTypeAssertion(node) => {
                self.is_simple_assignment_target(node.expression(self.ast))
            }
            _ => false,
        }
    }

    fn unwrap_parens(&self, expression: Expr) -> Expr {
        match self.ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.unwrap_parens(parenthesized.expression(self.ast))
            }
            _ => expression,
        }
    }

    fn expression_to_assignment_target(
        &mut self,
        expression: Expr,
    ) -> ParseResult<AssignmentTarget> {
        match self.ast.expr_data(expression) {
            ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_) => {
                let pattern = self.expression_to_assignment_pattern(expression)?;
                match self.ast.binding_pattern_data(pattern) {
                    BindingPatternData::ArrayPattern(pattern) => {
                        Ok(AssignmentTarget::ArrayAssignmentTarget(pattern))
                    }
                    BindingPatternData::ObjectPattern(pattern) => {
                        Ok(AssignmentTarget::ObjectAssignmentTarget(pattern))
                    }
                    _ => unreachable!("array and object expressions convert to matching patterns"),
                }
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                let inner = self.unwrap_parens(parenthesized.expression(self.ast));
                if matches!(
                    self.ast.expr_data(inner),
                    ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_)
                ) {
                    return self.error_at(
                        expression.span(self.ast),
                        "A parenthesized destructuring pattern is not an assignment target",
                    );
                }
                self.expression_to_assignment_target(inner)
            }
            _ => Ok(AssignmentTarget::SimpleAssignmentTarget(
                self.expression_to_simple_assignment_target(expression)?,
            )),
        }
    }

    fn parse_binding_identifier(&mut self) -> ParseResult<BindingIdentifier> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() || token.tag.is_unconditionally_reserved() {
            return self.error("expected an identifier");
        }
        let name = self.identifier_name(token)?;
        self.validate_identifier(token, name.as_str(), "a binding identifier")?;
        self.advance()?;
        let decorators = self.empty_range();
        Ok(self
            .ast
            .binding_identifier(token.span, name, decorators, None, false, None))
    }

    fn parse_label_identifier(&mut self) -> ParseResult<LabelIdentifier> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() || token.tag.is_unconditionally_reserved() {
            return self.error("expected a label");
        }
        let name = self.identifier_name(token)?;
        self.validate_identifier(token, name.as_str(), "a label")?;
        self.advance()?;
        Ok(self.ast.label_identifier(token.span, name))
    }

    fn validate_identifier(&self, token: Token, name: &str, as_what: &str) -> ParseResult<()> {
        self.validate_identifier_name(token.span, name, as_what)
    }

    fn validate_identifier_name(&self, span: Span, name: &str, as_what: &str) -> ParseResult<()> {
        if name == "yield" && self.context.yield_ {
            return Err(Error::new(
                span,
                format!(
                    "'yield' is reserved in a generator context and cannot be used as {as_what}"
                ),
            ));
        }
        if name == "await" && (self.context.await_ || self.source_type == SourceType::Module) {
            return Err(Error::new(
                span,
                format!(
                    "'await' is reserved in an async/module context and cannot be used as {as_what}"
                ),
            ));
        }
        Ok(())
    }

    fn atom_from_span(&self, span: Span) -> Atom<'allocator> {
        let bytes = &self.source[span.start as usize..span.end as usize];
        let value = str::from_utf8(bytes).expect("the lexer only accepts valid UTF-8 text");
        self.ast.add_atom(value)
    }

    fn empty_range<T: ExtraDataCompact<'allocator>>(&mut self) -> TypedSubRange<T> {
        self.ast.add_typed_sub_range(std::iter::empty())
    }

    fn next_token(&mut self) -> ParseResult<()> {
        self.lexer
            .next_token(&mut self.current_token)
            .map_err(|error| self.next_token_error(error))
    }

    #[cold]
    #[inline(never)]
    fn next_token_error(&self, error: LexicalError) -> Error {
        Error::new(
            Span::new(
                self.prev_token_end.min(self.lexer.cursor),
                self.prev_token_end.max(self.lexer.cursor),
            ),
            format!(
                "{}. {}",
                lexical_error_message(error),
                lexical_error_help(error)
            ),
        )
    }

    pub fn advance(&mut self) -> ParseResult<()> {
        self.check_escaped_keyword();
        self.prev_token_end = self.current_token.span.end;
        self.next_token()
    }

    pub fn advance_without_escape_check(&mut self) -> ParseResult<()> {
        self.prev_token_end = self.current_token.span.end;
        self.next_token()
    }

    pub fn check_escaped_keyword(&mut self) {
        let current_token = self.current_token;
        if current_token.is_escaped() && current_token.tag.is_keyword() {
            self.report_escaped_keyword(current_token.span);
        }
    }

    pub fn report_escaped_keyword(&mut self, span: Span) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            message: "Keywords cannot contain escape characters".into(),
            span,
            help: Some("Remove the escape characters".into()),
            labels: Vec::new(),
        });
    }

    pub fn report_if_escaped_keyword(&mut self, token: Token) {
        if token.is_escaped() {
            self.report_escaped_keyword(token.span);
        }
    }

    pub fn advance_with_rescanned_token(&mut self, token: Token) -> ParseResult<()> {
        self.current_token = token;
        self.advance()
    }

    pub fn re_scan_current(&mut self) -> ParseResult<()> {
        self.lexer.rewind_to(self.current_token.span.start);
        self.next_token()
    }

    fn expect(&mut self, tag: TokenTag) -> ParseResult<Token> {
        if self.current_token.tag != tag {
            let found = self.describe_token(self.current_token);
            return self.error(format!(
                "expected '{}', found '{}'",
                tag.to_str().unwrap_or("token"),
                String::from_utf8_lossy(found.as_ref())
            ));
        }
        let token = self.current_token;
        if tag.is_keyword() && token.is_escaped() {
            return self.error_at(token.span, "keywords may not contain escape sequences");
        }
        self.advance()?;
        Ok(token)
    }

    fn eat(&mut self, tag: TokenTag) -> ParseResult<bool> {
        if self.current_token.tag != tag {
            return Ok(false);
        }
        if tag.is_keyword() && self.current_token.is_escaped() {
            return self.error_at(
                self.current_token.span,
                "keywords may not contain escape sequences",
            );
        }
        self.advance()?;
        Ok(true)
    }

    pub fn can_insert_implicit_semicolon(&self, token: Token) -> bool {
        token.tag == TokenTag::Eof
            || token.has_line_terminator_before()
            || token.tag == TokenTag::RightBrace
    }

    pub fn eat_semicolon(&mut self, end: u32) -> ParseResult<u32> {
        if self.current_token.tag == TokenTag::Semicolon {
            return Ok(self.expect(TokenTag::Semicolon)?.span.end);
        }
        if self.can_insert_implicit_semicolon(self.current_token) {
            return Ok(end);
        }
        self.error("expected ';'")
    }

    pub fn eat_semicolon_lenient(&mut self, end: u32) -> ParseResult<u32> {
        if self.current_token.tag == TokenTag::Semicolon {
            return Ok(self.expect(TokenTag::Semicolon)?.span.end);
        }
        Ok(end)
    }

    pub fn report(
        &mut self,
        span: Span,
        message: impl Into<Cow<'static, str>>,
        options: ReportOptions,
    ) {
        self.diagnostics.push(Diagnostic {
            severity: options.severity,
            message: message.into().into_owned(),
            span,
            help: options.help.map(Cow::into_owned),
            labels: options.labels,
        });
    }

    pub fn report_expected(&mut self, span: Span, message: &str, options: ReportOptions) {
        let found = self.describe_token(self.current_token);
        self.report(
            span,
            Cow::Owned(format!(
                "{message}, but found '{}'",
                String::from_utf8_lossy(found.as_ref())
            )),
            options,
        );
    }

    pub fn label(&self, span: Span, message: impl Into<String>) -> Label {
        Label {
            span,
            message: message.into(),
        }
    }

    pub fn labels(&self, items: &[Label]) -> Vec<Label> {
        items.to_vec()
    }

    pub fn recover(&mut self, terminator: Option<TokenTag>) {
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

    fn error<T>(&self, message: impl Into<String>) -> ParseResult<T> {
        Err(Error::new(self.current_token.span, message))
    }

    fn error_at<T>(&self, span: Span, message: impl Into<String>) -> ParseResult<T> {
        Err(Error::new(span, message))
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
            matches!(
                ast.expr_data(parenthesized.expression(ast)),
                ExprData::ChainExpression(_)
            )
        }
        _ => false,
    }
}

fn expression_contains_optional_chain(expression: Expr, ast: &Ast<'_>) -> bool {
    match ast.expr_data(expression) {
        ExprData::MemberExpression(member) => member_expression_is_optional(member, ast),
        ExprData::CallExpression(call) => {
            call.optional(ast) || expression_contains_optional_chain(call.callee(ast), ast)
        }
        ExprData::ChainExpression(_) => true,
        ExprData::ParenthesizedExpression(_) => false,
        _ => false,
    }
}

fn max_left_precedence(expression: Expr, ast: &Ast<'_>) -> u8 {
    match ast.expr_data(expression) {
        ExprData::ArrowFunctionExpression(_) | ExprData::YieldExpression(_) => Precedence::COMMA,
        ExprData::UpdateExpression(_)
        | ExprData::UnaryExpression(_)
        | ExprData::AwaitExpression(_)
        | ExprData::BinaryExpression(_)
        | ExprData::LogicalExpression(_)
        | ExprData::ConditionalExpression(_)
        | ExprData::AssignmentExpression(_)
        | ExprData::SequenceExpression(_)
        | ExprData::TsAsExpression(_)
        | ExprData::TsSatisfiesExpression(_) => Precedence::UNARY,
        _ => u8::MAX,
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
        TsTupleElementData::TsOptionalType(ty) => TsType::TsOptionalType(ty),
        TsTupleElementData::TsRestType(ty) => TsType::TsRestType(ty),
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

pub fn build_line_starts(ast: &mut Ast<'_>, source: &[u8]) {
    debug_assert!(source.len() <= u32::MAX as usize);
    let mut starts = Vec::with_capacity(std::cmp::max(8, source.len() / 24));
    starts.push(0);

    const BYTE_LOW_BITS: u64 = 0x0101_0101_0101_0101;
    const BYTE_HIGH_BITS: u64 = 0x8080_8080_8080_8080;
    const LINE_FEEDS: u64 = u64::from_le_bytes([b'\n'; 8]);

    let mut chunks = source.chunks_exact(8);
    for (chunk_index, chunk) in chunks.by_ref().enumerate() {
        let bytes: [u8; 8] = chunk.try_into().expect("chunk size is fixed");
        let mut matches = u64::from_le_bytes(bytes) ^ LINE_FEEDS;
        for _ in 0..8 {
            let zero_high_bits = matches.wrapping_sub(BYTE_LOW_BITS) & !matches & BYTE_HIGH_BITS;
            if zero_high_bits == 0 {
                break;
            }
            let byte_index = zero_high_bits.trailing_zeros() as usize / 8;
            let source_index = chunk_index * 8 + byte_index;
            if source[source_index] == b'\n' {
                starts.push((source_index + 1) as u32);
            }
            matches |= 0xff << (byte_index * 8);
        }
    }

    let remainder_start = source.len() - chunks.remainder().len();
    for (remainder_index, byte) in chunks.remainder().iter().enumerate() {
        if *byte == b'\n' {
            starts.push((remainder_start + remainder_index + 1) as u32);
        }
    }
    ast.set_line_starts(starts);
}

fn can_start_binding(tag: TokenTag) -> bool {
    matches!(tag, TokenTag::LeftBracket | TokenTag::LeftBrace)
        || (tag.is_identifier_like() && !tag.is_unconditionally_reserved())
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
    !matches!(
        tag,
        TokenTag::Static | TokenTag::Async | TokenTag::Get | TokenTag::Set
    )
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

fn logical_expression_mixes_coalesce(
    expression: Expr,
    parent_operator: LogicalOperator,
    ast: &Ast<'_>,
) -> bool {
    let ExprData::LogicalExpression(logical) = ast.expr_data(expression) else {
        return false;
    };
    let child_operator = logical.operator(ast);
    (parent_operator == LogicalOperator::NullishCoalescing)
        != (child_operator == LogicalOperator::NullishCoalescing)
}

fn is_unconditionally_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
    )
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
