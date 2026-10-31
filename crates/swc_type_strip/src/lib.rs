//! Fixed-width type stripping for the TypeScript `erasableSyntaxOnly` subset.

#![deny(clippy::all)]
#![deny(unused)]

use std::borrow::Cow;

use edit::{EditPlan, FixedSubstitution};
use swc_next_allocator::{Allocator, vec::ArenaVec};
use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionExpression, Ast, BinaryExpression, CallExpression, ChainExpression,
    Class, ClassElement, Decl, DeclData, Decorator, DoWhileStatement, ExportAllDeclaration,
    ExportDefaultDeclaration, ExportNamedDeclaration, Expr, ExprData, ForInStatement,
    ForOfStatement, ForStatement, FormalParameter, FunctionBody, GetSpan, IfStatement,
    ImportDeclaration, ImportOrExportKind, LabeledStatement, LogicalExpression, MemberExpression,
    MethodDefinition, Program, PropertyDefinition, PropertyKey, Stmt, SubRange,
    TaggedTemplateExpression, TsAsExpression, TsClassImplements, TsEnumDeclaration,
    TsExportAssignment, TsFunction, TsGlobalDeclaration, TsImportEqualsDeclaration,
    TsIndexSignature, TsInstantiationExpression, TsInterfaceDeclaration, TsMethodDefinition,
    TsModuleDeclaration, TsNamespaceExportDeclaration, TsNonNullExpression, TsParameterProperty,
    TsSatisfiesExpression, TsThisParameter, TsTypeAliasDeclaration, TsTypeAnnotation,
    TsTypeAssertion, TsTypeParameterDeclaration, TsTypeParameterInstantiation, VariableDeclaration,
    VariableDeclarator, Visit, VisitWith, WhileStatement, WithStatement,
};
pub use swc_next_ecma_ast::{Diagnostic, Label, Lang, Severity, SourceType, Span};
use swc_next_ecma_parser::{
    CommentMode, Options as ParserOptions, ParseReturn, Parser, Token, TokenParserConfig, TokenTag,
};

use crate::token_cursor::TokenCursor;

mod edit;
mod module;
mod token_cursor;

const UNSUPPORTED_LANG_MESSAGE: &str = "TypeError: Unsupported lang variant";
const ERASABLE_SYNTAX_ONLY_MESSAGE: &str =
    "SyntaxError: This syntax is not allowed when `erasableSyntaxOnly` is enabled";

/// Options for [`strip_types`].
///
/// Converting from [`ParserOptions`] retains only [`Self::source_type`] and
/// [`Self::lang`]. Converting back enables parenthesized nodes and disables
/// comment collection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Options {
    /// How the source is parsed and evaluated. Defaults to [`SourceType::Module`].
    pub source_type: SourceType,
    /// Language variant of the source.
    ///
    /// Must be [`Lang::Ts`] or [`Lang::Tsx`] (defaults to [`Lang::Js`]).
    /// [`Lang::Tsx`] preserves JSX syntax in the output without lowering it.
    pub lang: Lang,
}

impl From<ParserOptions> for Options {
    fn from(options: ParserOptions) -> Self {
        Self {
            source_type: options.source_type,
            lang: options.lang,
        }
    }
}

impl From<Options> for ParserOptions {
    fn from(options: Options) -> Self {
        Self {
            source_type: options.source_type,
            lang: options.lang,
            preserve_parens: true,
            comments: CommentMode::None,
        }
    }
}

/// Stripped output and diagnostics returned by [`strip_types`].
#[derive(Debug, Eq, PartialEq)]
pub struct StripResult<'source> {
    /// Stripped source with the same UTF-16 length and line layout as the input.
    ///
    /// Borrows the input when no edits are made, including on unsupported [`Lang`]
    /// variants or unrecoverable parse errors.
    pub code: Cow<'source, str>,
    /// Diagnostics in source order.
    ///
    /// Contains parser diagnostics and at most one `erasableSyntaxOnly` error for
    /// the first unsupported TypeScript construct. When non-empty, [`Self::code`]
    /// is recovery output and may not preserve program semantics.
    pub diagnostics: Vec<Diagnostic<'static>>,
}

/// Strips erasable TypeScript syntax from `source` while preserving UTF-16 offsets
/// and line layout.
///
/// Erased spans are replaced with spaces, with minimal `;`, `(`, and `)` adjustments
/// where required to keep the remaining JavaScript syntax valid.
///
/// Only the TypeScript `erasableSyntaxOnly` subset is supported:
/// - Non-erasable constructs (`enum`, instantiated `namespace`/`module`/`global`,
///   parameter properties, `import =`, `export =`, and `<T>` type assertions) are
///   not lowered. Declarations are erased as whole statements, parameter property
///   modifiers and `<T>` prefixes are blanked in place, and a diagnostic is
///   reported.
/// - Stripping is purely syntactic: names are not resolved and `const enum` values
///   are not inlined, so type-only imports and exports must use explicit `type`
///   modifiers.
///
/// [`Options::lang`] must be [`Lang::Ts`] or [`Lang::Tsx`]; [`Lang::Tsx`] preserves
/// JSX syntax without lowering it, and other [`Lang`] variants return `source`
/// unchanged with an error diagnostic.
#[cfg_attr(feature = "benchmarking", inline(never))]
pub fn strip_types(source: &str, options: Options) -> StripResult<'_> {
    if !matches!(options.lang, Lang::Ts | Lang::Tsx) {
        return StripResult {
            code: Cow::Borrowed(source),
            diagnostics: vec![Diagnostic::error(UNSUPPORTED_LANG_MESSAGE, Span::new(0, 0))],
        };
    }

    let allocator = Allocator::default();
    let parse = Parser::init(&allocator, source, options.into(), TokenParserConfig).parse();
    #[cfg(feature = "benchmarking")]
    let parse = benchmark_parse_boundary(parse);
    let ParseReturn {
        ast,
        tokens,
        mut diagnostics,
    } = parse;

    let program = ast.root_program();
    let mut eraser = TypeEraser {
        ast: &ast,
        tokens: TokenCursor::new(source, tokens.as_slice()),
        edits: EditPlan::default(),
        diagnostic: None,
        item_context: ItemContext::default(),
        expr_frames: ArenaVec::new_in(&allocator),
    };
    program.visit_with(&mut eraser);
    let TypeEraser {
        edits, diagnostic, ..
    } = eraser;
    diagnostics.extend(diagnostic);
    diagnostics.sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.span.end));

    StripResult {
        code: edits.render(source),
        diagnostics,
    }
}

#[cfg(feature = "benchmarking")]
#[inline(never)]
fn benchmark_parse_boundary(parse: ParseReturn<'_>) -> ParseReturn<'_> {
    std::hint::black_box(parse)
}

#[derive(Clone, Copy, Default)]
struct ItemContext {
    previous_retained: Option<Span>,
    terminates_directive_prologue: bool,
    required_statement: bool,
    erased: bool,
}

struct TypeEraser<'ast, 'source, 'tokens> {
    ast: &'ast Ast<'ast>,
    tokens: TokenCursor<'source, 'tokens>,
    edits: EditPlan,
    diagnostic: Option<Diagnostic<'static>>,
    item_context: ItemContext,
    expr_frames: ArenaVec<'ast, ExprFrame>,
}

enum ExprFrame {
    Expression(Expr),
    Property(PropertyKey),
    Call(CallExpression),
    TaggedTemplate(TaggedTemplateExpression),
    TypeArguments(TsTypeParameterInstantiation),
    As(TsAsExpression),
    Satisfies(TsSatisfiesExpression),
    NonNull(TsNonNullExpression),
}

impl ExprFrame {
    fn visit_remaining(self, eraser: &mut TypeEraser<'_, '_, '_>) {
        let ast = eraser.ast;
        match self {
            Self::Expression(expression) => expression.visit_with(eraser),
            Self::Property(property) => property.visit_with(eraser),
            Self::Call(call) => {
                call.type_arguments(ast).visit_with(eraser);
                call.arguments(ast).visit_with(eraser);
            }
            Self::TaggedTemplate(tagged) => {
                tagged.type_arguments(ast).visit_with(eraser);
                tagged.quasi(ast).visit_with(eraser);
            }
            Self::TypeArguments(arguments) => arguments.visit_with(eraser),
            Self::As(assertion) => {
                eraser.erase_postfix_type_assertion(assertion.expression(ast), assertion.span(ast));
            }
            Self::Satisfies(assertion) => {
                eraser.erase_postfix_type_assertion(assertion.expression(ast), assertion.span(ast));
            }
            Self::NonNull(assertion) => {
                eraser.erase_suffix_expression(assertion.expression(ast), assertion.span(ast));
            }
        }
    }
}

impl TypeEraser<'_, '_, '_> {
    fn visit_chain_operand(&mut self, mut node: Expr) {
        loop {
            match self.ast.expr_data(node) {
                ExprData::IdentifierReference(_) => return,
                // Static property names cannot contain types or side expressions.
                ExprData::MemberExpression(member) if !member.computed(self.ast) => {
                    node = member.object(self.ast);
                }
                ExprData::ChainExpression(chain) => node = chain.expression(self.ast),
                ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::MemberExpression(_)
                | ExprData::CallExpression(_)
                | ExprData::TaggedTemplateExpression(_)
                | ExprData::TsAsExpression(_)
                | ExprData::TsSatisfiesExpression(_)
                | ExprData::TsNonNullExpression(_)
                | ExprData::TsInstantiationExpression(_) => {
                    self.visit_expr_chain(node);
                    return;
                }
                _ => {
                    node.visit_with(self);
                    return;
                }
            }
        }
    }

    fn visit_expr_chain(&mut self, mut node: Expr) {
        // The parser builds long left-associative and postfix chains iteratively.
        // Only nested chain operands need an explicit stack; ordinary expressions
        // retain the generated visitor's direct dispatch.
        let stack_bottom = self.expr_frames.len();
        let mut outer_frame = None;
        loop {
            let (next, frame) = match self.ast.expr_data(node) {
                ExprData::IdentifierReference(_) => break,
                ExprData::BinaryExpression(binary) => (
                    binary.left(self.ast),
                    ExprFrame::Expression(binary.right(self.ast)),
                ),
                ExprData::LogicalExpression(logical) => (
                    logical.left(self.ast),
                    ExprFrame::Expression(logical.right(self.ast)),
                ),
                ExprData::MemberExpression(member) => {
                    if !member.computed(self.ast) {
                        node = member.object(self.ast);
                        continue;
                    }
                    (
                        member.object(self.ast),
                        ExprFrame::Property(member.property(self.ast)),
                    )
                }
                ExprData::CallExpression(call) => (call.callee(self.ast), ExprFrame::Call(call)),
                ExprData::TaggedTemplateExpression(tagged) => {
                    (tagged.tag(self.ast), ExprFrame::TaggedTemplate(tagged))
                }
                ExprData::ChainExpression(chain) => {
                    node = chain.expression(self.ast);
                    continue;
                }
                ExprData::TsAsExpression(assertion) => {
                    (assertion.expression(self.ast), ExprFrame::As(assertion))
                }
                ExprData::TsSatisfiesExpression(assertion) => (
                    assertion.expression(self.ast),
                    ExprFrame::Satisfies(assertion),
                ),
                ExprData::TsNonNullExpression(assertion) => (
                    assertion.expression(self.ast),
                    ExprFrame::NonNull(assertion),
                ),
                ExprData::TsInstantiationExpression(instantiation) => (
                    instantiation.expression(self.ast),
                    ExprFrame::TypeArguments(instantiation.type_arguments(self.ast)),
                ),
                _ => {
                    node.visit_with(self);
                    break;
                }
            };
            // A single link needs no heap allocation.
            if outer_frame.is_none() {
                outer_frame = Some(frame);
            } else {
                self.expr_frames.push(frame);
            }
            node = next;
        }

        // Side expressions may start another chain with outer frames pending.
        while self.expr_frames.len() > stack_bottom
            && let Some(frame) = self.expr_frames.pop()
        {
            frame.visit_remaining(self);
        }
        if let Some(frame) = outer_frame {
            frame.visit_remaining(self);
        }
    }

    fn report_erasable_syntax_only(&mut self, span: Span) {
        if self.diagnostic.is_none() {
            self.diagnostic = Some(Diagnostic::error(ERASABLE_SYNTAX_ONLY_MESSAGE, span));
        }
    }

    fn erase_token(&mut self, start: u32, end: u32, tag: TokenTag) -> Option<u32> {
        let token = self.tokens.find_forward(start, end, tag)?;
        self.edits.blank(token.span());
        Some(token.start())
    }

    fn erase_optional_parameter(&mut self, type_annotation: Option<TsTypeAnnotation>, span: Span) {
        let marker_end =
            type_annotation.map_or(span.end, |annotation| annotation.span(self.ast).start);
        if let Some(marker) = self
            .tokens
            .find_backward(span.start, marker_end, TokenTag::Question)
        {
            self.edits.blank(marker.span());
        }
    }

    fn erase_list_item(&mut self, item: Span, next: Option<Span>, container: Span) {
        let gap_end = next.map_or_else(
            || {
                self.tokens
                    .find_forward(item.end, container.end, TokenTag::RightBrace)
                    .map_or(container.end, |closing| closing.start())
            },
            |next| next.start,
        );
        if let Some(comma) = self.tokens.find_forward(item.end, gap_end, TokenTag::Comma) {
            self.edits.blank(Span::new(item.start, comma.end()));
        } else {
            self.edits.blank(item);
        }
    }

    fn erase_statement(&mut self, span: Span) {
        self.edits.blank(span);
        self.item_context.erased = true;

        let terminates_directive_prologue = self.item_context.terminates_directive_prologue;
        let prevents_asi = self.needs_semicolon_before_erasure();
        if !terminates_directive_prologue && !prevents_asi {
            return;
        }
        let Some(first) = self.tokens.first_in_range(span.start, span.end) else {
            return;
        };
        let Some(replacement_span) = self.tokens.leading_character_span(first.start()) else {
            return;
        };
        self.edits
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn needs_semicolon_before_erasure(&self) -> bool {
        if self.item_context.required_statement {
            return true;
        }

        let Some(previous) = self.item_context.previous_retained else {
            return false;
        };
        let last = self.tokens.last_in_range(previous.start, previous.end);
        !last.is_some_and(|token| token.tag() == TokenTag::Semicolon)
    }

    fn erase_suffix_expression(&mut self, expression: Expr, wrapper_span: Span) {
        let expression_span = expression.span(self.ast);
        self.edits
            .blank(Span::new(expression_span.end, wrapper_span.end));
    }

    fn erase_postfix_type_assertion(&mut self, expression: Expr, wrapper_span: Span) {
        let expression_span = expression.span(self.ast);
        self.erase_suffix_expression(expression, wrapper_span);

        let Some(next) = self.tokens.at_or_after(wrapper_span.end) else {
            return;
        };
        if !next.has_line_terminator_before()
            || !matches!(
                next.tag(),
                TokenTag::LeftParen
                    | TokenTag::LeftBracket
                    | TokenTag::NoSubstitutionTemplate
                    | TokenTag::TemplateHead
            )
        {
            return;
        }
        let Some(replacement_span) = self.tokens.leading_character_span(expression_span.end) else {
            return;
        };
        self.edits
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn move_opening_parenthesis_across_multiline_type_parameters(
        &mut self,
        arrow: ArrowFunctionExpression,
    ) {
        let Some(type_parameters) = arrow.type_parameters(self.ast) else {
            return;
        };
        if !self.has_line_sensitive_arrow_prefix(arrow) {
            return;
        }

        let type_parameters = type_parameters.span(self.ast);
        let parameters = arrow.params(self.ast).span(self.ast);
        if !self
            .tokens
            .has_line_terminator(type_parameters.start, parameters.start)
        {
            return;
        }
        let Some(opening) =
            self.tokens
                .find_forward(parameters.start, parameters.end, TokenTag::LeftParen)
        else {
            return;
        };
        let Some(replacement_span) = self.tokens.leading_character_span(type_parameters.start)
        else {
            return;
        };

        self.edits
            .substitute(replacement_span, FixedSubstitution::LeftParenthesis);
        self.edits.blank(opening.span());
    }

    fn move_closing_parenthesis_across_multiline_return_type(
        &mut self,
        arrow: ArrowFunctionExpression,
    ) {
        let Some(return_type) = arrow.return_type(self.ast) else {
            return;
        };
        let return_span = return_type.span(self.ast);
        let Some(arrow_token) =
            self.tokens
                .find_forward(return_span.end, arrow.span(self.ast).end, TokenTag::Arrow)
        else {
            return;
        };
        let parameters = arrow.params(self.ast).span(self.ast);
        if !self
            .tokens
            .has_line_terminator(parameters.end, arrow_token.start())
        {
            return;
        }
        let Some(closing_parenthesis) =
            self.tokens
                .find_backward(parameters.start, parameters.end, TokenTag::RightParen)
        else {
            return;
        };
        let Some(replacement_span) = self.tokens.trailing_character_span(return_span.end) else {
            return;
        };

        self.edits
            .blank(Span::new(closing_parenthesis.start(), return_span.end));
        self.edits
            .substitute(replacement_span, FixedSubstitution::RightParenthesis);
    }

    fn has_line_sensitive_arrow_prefix(&self, arrow: ArrowFunctionExpression) -> bool {
        if arrow.r#async(self.ast) {
            return true;
        }

        let Some(type_parameters) = arrow.type_parameters(self.ast) else {
            return false;
        };
        let type_parameters = type_parameters.span(self.ast);
        let Some(opening) = self
            .tokens
            .first_in_range(type_parameters.start, type_parameters.end)
        else {
            return false;
        };
        if opening.has_line_terminator_before() {
            return false;
        }
        self.tokens
            .previous_before(opening.start())
            .is_some_and(|token| {
                matches!(
                    token.tag(),
                    TokenTag::Return | TokenTag::Throw | TokenTag::Yield
                )
            })
    }

    fn separate_hazardous_class_element(
        &mut self,
        member_span: Span,
        first_erased_modifier: Option<u32>,
        hazardous_key: bool,
        protected_by_javascript_syntax: bool,
    ) {
        if !hazardous_key || protected_by_javascript_syntax {
            return;
        }
        if first_erased_modifier != Some(member_span.start) {
            return;
        }
        if !self.needs_semicolon_before_erasure() {
            return;
        }
        let Some(replacement_span) = self.tokens.leading_character_span(member_span.start) else {
            return;
        };
        self.edits
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn preserve_keyword_named_class_field(
        &mut self,
        key_token: Option<Token>,
        separator_start: Option<u32>,
        has_value: bool,
    ) {
        if has_value {
            return;
        }
        let Some(separator_start) = separator_start else {
            return;
        };
        let Some(key) = key_token else {
            return;
        };
        if !matches!(key.tag(), TokenTag::Get | TokenTag::Set | TokenTag::Static) {
            return;
        }
        let Some(replacement_span) = self.tokens.leading_character_span(separator_start) else {
            return;
        };
        self.edits
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn visit_statement_list(
        &mut self,
        statements: SubRange<Stmt>,
        terminates_directive_prologue: bool,
    ) {
        let saved_context = self.item_context;
        let mut previous_retained = None;
        for (position, statement) in statements.iter().enumerate() {
            let statement = self.ast.get_node_in_sub_range(statement);
            self.item_context = ItemContext {
                previous_retained,
                terminates_directive_prologue: terminates_directive_prologue && position == 0,
                required_statement: false,
                erased: false,
            };
            statement.visit_with(self);
            if !self.item_context.erased {
                previous_retained = Some(statement.span(self.ast));
            }
        }
        self.item_context = saved_context;
    }

    fn visit_class_element_list(&mut self, elements: SubRange<ClassElement>) {
        let saved_context = self.item_context;
        let mut previous_retained = None;
        for element in elements.iter() {
            let element = self.ast.get_node_in_sub_range(element);
            self.item_context = ItemContext {
                previous_retained,
                ..ItemContext::default()
            };
            element.visit_with(self);
            if !self.item_context.erased {
                previous_retained = Some(element.span(self.ast));
            }
        }
        self.item_context = saved_context;
    }

    fn visit_required_statement(&mut self, statement: Stmt) {
        let saved_context = self.item_context;
        self.item_context = ItemContext {
            required_statement: true,
            ..ItemContext::default()
        };
        statement.visit_with(self);
        self.item_context = saved_context;
    }

    fn erase_enum(&mut self, node: TsEnumDeclaration, statement_span: Span) {
        self.erase_statement(statement_span);
        if !node.declare(self.ast) {
            self.report_erasable_syntax_only(node.span(self.ast));
        }
    }

    fn erase_module(&mut self, node: TsModuleDeclaration, statement_span: Span) {
        self.erase_statement(statement_span);
        if self.diagnostic.is_none()
            && let Some(span) = module::check(node, self.ast)
        {
            self.report_erasable_syntax_only(span);
        }
    }

    fn erase_global(&mut self, node: TsGlobalDeclaration, statement_span: Span) {
        self.erase_statement(statement_span);
        if self.diagnostic.is_none()
            && let Some(span) = module::check_global(node, self.ast)
        {
            self.report_erasable_syntax_only(span);
        }
    }

    fn erase_import_equals(&mut self, node: TsImportEqualsDeclaration, statement_span: Span) {
        self.erase_statement(statement_span);
        self.report_erasable_syntax_only(node.span(self.ast));
    }
}

impl<'ast> Visit<'ast> for TypeEraser<'ast, '_, '_> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_expr(&mut self, node: Expr) {
        if !node.is_identifier_reference(self.ast) {
            node.visit_children_with(self);
        }
    }

    fn visit_binary_expression(&mut self, node: BinaryExpression) {
        self.visit_chain_operand(node.left(self.ast));
        node.right(self.ast).visit_with(self);
    }

    fn visit_logical_expression(&mut self, node: LogicalExpression) {
        self.visit_chain_operand(node.left(self.ast));
        node.right(self.ast).visit_with(self);
    }

    fn visit_member_expression(&mut self, node: MemberExpression) {
        self.visit_chain_operand(node.object(self.ast));
        node.property(self.ast).visit_with(self);
    }

    fn visit_call_expression(&mut self, node: CallExpression) {
        self.visit_chain_operand(node.callee(self.ast));
        node.type_arguments(self.ast).visit_with(self);
        node.arguments(self.ast).visit_with(self);
    }

    fn visit_chain_expression(&mut self, node: ChainExpression) {
        self.visit_chain_operand(node.expression(self.ast));
    }

    fn visit_tagged_template_expression(&mut self, node: TaggedTemplateExpression) {
        self.visit_chain_operand(node.tag(self.ast));
        node.type_arguments(self.ast).visit_with(self);
        node.quasi(self.ast).visit_with(self);
    }

    fn visit_program(&mut self, node: Program) {
        self.visit_statement_list(node.body(self.ast), true);
    }

    fn visit_function_body(&mut self, node: FunctionBody) {
        self.visit_statement_list(node.body(self.ast), true);
    }

    fn visit_stmts(&mut self, node: SubRange<Stmt>) {
        self.visit_statement_list(node, false);
    }

    fn visit_class_elements(&mut self, node: SubRange<ClassElement>) {
        self.visit_class_element_list(node);
    }

    fn visit_if_statement(&mut self, node: IfStatement) {
        node.test(self.ast).visit_with(self);
        self.visit_required_statement(node.consequent(self.ast));
        if let Some(alternate) = node.alternate(self.ast) {
            self.visit_required_statement(alternate);
        }
    }

    fn visit_labeled_statement(&mut self, node: LabeledStatement) {
        self.visit_required_statement(node.body(self.ast));
    }

    fn visit_while_statement(&mut self, node: WhileStatement) {
        node.test(self.ast).visit_with(self);
        self.visit_required_statement(node.body(self.ast));
    }

    fn visit_do_while_statement(&mut self, node: DoWhileStatement) {
        self.visit_required_statement(node.body(self.ast));
        node.test(self.ast).visit_with(self);
    }

    fn visit_for_statement(&mut self, node: ForStatement) {
        node.init(self.ast).visit_with(self);
        node.test(self.ast).visit_with(self);
        node.update(self.ast).visit_with(self);
        self.visit_required_statement(node.body(self.ast));
    }

    fn visit_for_in_statement(&mut self, node: ForInStatement) {
        node.left(self.ast).visit_with(self);
        node.right(self.ast).visit_with(self);
        self.visit_required_statement(node.body(self.ast));
    }

    fn visit_for_of_statement(&mut self, node: ForOfStatement) {
        node.left(self.ast).visit_with(self);
        node.right(self.ast).visit_with(self);
        self.visit_required_statement(node.body(self.ast));
    }

    fn visit_with_statement(&mut self, node: WithStatement) {
        node.object(self.ast).visit_with(self);
        self.visit_required_statement(node.body(self.ast));
    }

    fn visit_arrow_function_expression(&mut self, node: ArrowFunctionExpression) {
        self.move_opening_parenthesis_across_multiline_type_parameters(node);
        self.move_closing_parenthesis_across_multiline_return_type(node);
        node.visit_children_with(self);
    }

    fn visit_ts_this_parameter(&mut self, node: TsThisParameter) {
        let span = node.span(self.ast);
        let next = self
            .tokens
            .first_in_range(span.end, self.ast.source().len() as u32);
        let end = next
            .filter(|token| token.tag() == TokenTag::Comma)
            .map_or(span.end, |token| token.end());
        self.edits.blank(Span::new(span.start, end));
    }

    fn visit_variable_declaration(&mut self, node: VariableDeclaration) {
        if node.declare(self.ast) {
            self.erase_statement(node.span(self.ast));
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_class(&mut self, node: Class) {
        let span = node.span(self.ast);
        if node.declare(self.ast) {
            self.erase_statement(span);
            return;
        }

        let identifier_start = node.id(self.ast).map_or_else(
            || node.body(self.ast).span(self.ast).start,
            |id| id.span(self.ast).start,
        );
        if node.r#abstract(self.ast) {
            let modifiers_start = modifiers_start(span.start, node.decorators(self.ast), self.ast);
            self.erase_token(modifiers_start, identifier_start, TokenTag::Abstract);
        }

        let implements = node.implements(self.ast);
        if let (Some(first), Some(last)) = (implements.first(), implements.last()) {
            let first = self.ast.get_node_in_sub_range(first);
            let last = self.ast.get_node_in_sub_range(last);
            let first_span = first.span(self.ast);
            let last_span = last.span(self.ast);
            if let Some(keyword) =
                self.tokens
                    .find_backward(span.start, first_span.start, TokenTag::Implements)
            {
                self.edits.blank(Span::new(keyword.start(), last_span.end));
            }
        }

        node.visit_children_with(self);
    }

    fn visit_property_definition(&mut self, node: PropertyDefinition) {
        let span = node.span(self.ast);
        if node.declare(self.ast) || node.r#abstract(self.ast) {
            self.erase_statement(span);
            return;
        }

        let key_span = node.key(self.ast).span(self.ast);
        let key_token = self.tokens.first_in_range(key_span.start, key_span.end);
        let modifiers_start = modifiers_start(span.start, node.decorators(self.ast), self.ast);
        let mut first_erased_modifier = None;
        if node.readonly(self.ast) {
            let start = self.erase_token(modifiers_start, key_span.start, TokenTag::Readonly);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        if node.r#override(self.ast) {
            let start = self.erase_token(modifiers_start, key_span.start, TokenTag::Override);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        if let Some(tag) = accessibility_token(node.accessibility(self.ast)) {
            let start = self.erase_token(modifiers_start, key_span.start, tag);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        let hazardous_key = node.computed(self.ast)
            || key_token
                .is_some_and(|token| matches!(token.tag(), TokenTag::In | TokenTag::Instanceof));
        let protected_by_javascript_syntax = node.r#static(self.ast)
            || node.accessor(self.ast)
            || !node.decorators(self.ast).is_empty();
        self.separate_hazardous_class_element(
            span,
            first_erased_modifier,
            hazardous_key,
            protected_by_javascript_syntax,
        );

        let mut marker_end = span.end;
        if let Some(annotation) = node.type_annotation(self.ast) {
            marker_end = marker_end.min(annotation.span(self.ast).start);
        }
        if let Some(value) = node.value(self.ast) {
            marker_end = marker_end.min(value.span(self.ast).start);
        }
        let optional_start = if node.optional(self.ast) {
            self.erase_token(key_span.end, marker_end, TokenTag::Question)
        } else {
            None
        };
        let definite_start = if node.definite(self.ast) {
            self.erase_token(key_span.end, marker_end, TokenTag::LogicalNot)
        } else {
            None
        };
        let separator_start = node
            .type_annotation(self.ast)
            .map(|annotation| annotation.span(self.ast).start)
            .or(optional_start)
            .or(definite_start);
        self.preserve_keyword_named_class_field(
            key_token,
            separator_start,
            node.value(self.ast).is_some(),
        );

        node.visit_children_with(self);
    }

    fn visit_method_definition(&mut self, node: MethodDefinition) {
        let span = node.span(self.ast);
        let key_span = node.key(self.ast).span(self.ast);
        let key_token = self.tokens.first_in_range(key_span.start, key_span.end);
        let modifiers_start = modifiers_start(span.start, node.decorators(self.ast), self.ast);
        let mut first_erased_modifier = None;
        if node.r#abstract(self.ast) {
            let start = self.erase_token(modifiers_start, key_span.start, TokenTag::Abstract);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        if node.r#override(self.ast) {
            let start = self.erase_token(modifiers_start, key_span.start, TokenTag::Override);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        if let Some(tag) = accessibility_token(node.accessibility(self.ast)) {
            let start = self.erase_token(modifiers_start, key_span.start, tag);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        let hazardous_key = node.computed(self.ast)
            || node.value(self.ast).generator(self.ast)
            || key_token
                .is_some_and(|token| matches!(token.tag(), TokenTag::In | TokenTag::Instanceof));
        let protected_by_javascript_syntax =
            node.r#static(self.ast) || !node.decorators(self.ast).is_empty();
        self.separate_hazardous_class_element(
            span,
            first_erased_modifier,
            hazardous_key,
            protected_by_javascript_syntax,
        );
        if node.optional(self.ast) {
            let function_start = node.value(self.ast).span(self.ast).start;
            self.erase_token(key_span.end, function_start, TokenTag::Question);
        }
        node.visit_children_with(self);
    }

    fn visit_ts_function(&mut self, node: TsFunction) {
        self.erase_statement(node.span(self.ast));
    }

    fn visit_ts_method_definition(&mut self, node: TsMethodDefinition) {
        self.erase_statement(node.span(self.ast));
    }

    fn visit_ts_index_signature(&mut self, node: TsIndexSignature) {
        self.erase_statement(node.span(self.ast));
    }

    fn visit_ts_enum_declaration(&mut self, node: TsEnumDeclaration) {
        self.erase_enum(node, node.span(self.ast));
    }

    fn visit_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        self.erase_module(node, node.span(self.ast));
    }

    fn visit_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        self.erase_global(node, node.span(self.ast));
    }

    fn visit_ts_parameter_property(&mut self, node: TsParameterProperty) {
        let span = node.span(self.ast);
        let decorators = node.decorators(self.ast);
        let modifiers_start = modifiers_start(span.start, decorators, self.ast);
        let parameter_start = node.parameter(self.ast).span(self.ast).start;
        if self.diagnostic.is_none() {
            let diagnostic_span = self
                .tokens
                .first_in_range(modifiers_start, parameter_start)
                .map_or(span, |modifier| modifier.span());
            self.report_erasable_syntax_only(diagnostic_span);
        }
        self.edits
            .blank(Span::new(modifiers_start, parameter_start));
        node.visit_children_with(self);
    }

    fn visit_ts_export_assignment(&mut self, node: TsExportAssignment) {
        self.erase_statement(node.span(self.ast));
        self.report_erasable_syntax_only(node.span(self.ast));
    }

    fn visit_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        self.erase_import_equals(node, node.span(self.ast));
    }

    fn visit_import_declaration(&mut self, node: ImportDeclaration) {
        let container = node.span(self.ast);
        if node.import_kind(self.ast) == ImportOrExportKind::Type {
            self.erase_statement(container);
            return;
        }

        let specifiers = node.specifiers(self.ast);
        let mut specifier_ids = specifiers.iter().peekable();
        while let Some(specifier) = specifier_ids.next() {
            let specifier = self.ast.get_node_in_sub_range(specifier);
            let Some(specifier) = specifier.as_import_specifier(self.ast) else {
                continue;
            };
            if specifier.import_kind(self.ast) != ImportOrExportKind::Type {
                continue;
            }

            let item = specifier.span(self.ast);
            let next = specifier_ids
                .peek()
                .copied()
                .map(|specifier| self.ast.get_node_in_sub_range(specifier).span(self.ast));
            self.erase_list_item(item, next, container);
        }
    }

    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        let container = node.span(self.ast);
        let declaration = node.declaration(self.ast);
        if let Some(declaration) = declaration {
            match self.ast.decl_data(declaration) {
                DeclData::TsEnumDeclaration(enumeration) => {
                    self.erase_enum(enumeration, container);
                    return;
                }
                DeclData::TsModuleDeclaration(module) => {
                    self.erase_module(module, container);
                    return;
                }
                DeclData::TsGlobalDeclaration(global) => {
                    self.erase_global(global, container);
                    return;
                }
                DeclData::TsImportEqualsDeclaration(import) => {
                    self.erase_import_equals(import, container);
                    return;
                }
                _ => {}
            }
        }
        if node.export_kind(self.ast) == ImportOrExportKind::Type {
            self.erase_statement(container);
            return;
        }

        if declaration.is_some_and(|declaration| is_whole_type_declaration(declaration, self.ast)) {
            self.erase_statement(container);
            return;
        }

        let specifiers = node.specifiers(self.ast);
        let mut specifier_ids = specifiers.iter().peekable();
        while let Some(specifier) = specifier_ids.next() {
            let specifier = self.ast.get_node_in_sub_range(specifier);
            if specifier.export_kind(self.ast) != ImportOrExportKind::Type {
                continue;
            }

            let item = specifier.span(self.ast);
            let next = specifier_ids
                .peek()
                .copied()
                .map(|specifier| self.ast.get_node_in_sub_range(specifier).span(self.ast));
            self.erase_list_item(item, next, container);
        }
        declaration.visit_with(self);
    }

    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        if node.export_kind(self.ast) == ImportOrExportKind::Type {
            self.erase_statement(node.span(self.ast));
        }
    }

    fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        let declaration = node.declaration(self.ast);
        if declaration.is_ts_function(self.ast) || declaration.is_ts_interface_declaration(self.ast)
        {
            self.erase_statement(node.span(self.ast));
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_ts_type_parameter_declaration(&mut self, node: TsTypeParameterDeclaration) {
        self.edits.blank(node.span(self.ast));
    }

    fn visit_ts_type_parameter_instantiation(&mut self, node: TsTypeParameterInstantiation) {
        self.edits.blank(node.span(self.ast));
    }

    fn visit_ts_class_implements(&mut self, _node: TsClassImplements) {}

    fn visit_ts_type_annotation(&mut self, node: TsTypeAnnotation) {
        self.edits.blank(node.span(self.ast));
    }

    fn visit_ts_type_alias_declaration(&mut self, node: TsTypeAliasDeclaration) {
        self.erase_statement(node.span(self.ast));
    }

    fn visit_ts_interface_declaration(&mut self, node: TsInterfaceDeclaration) {
        self.erase_statement(node.span(self.ast));
    }

    fn visit_ts_namespace_export_declaration(&mut self, node: TsNamespaceExportDeclaration) {
        self.erase_statement(node.span(self.ast));
    }

    fn visit_ts_as_expression(&mut self, node: TsAsExpression) {
        let expression = node.expression(self.ast);
        self.visit_chain_operand(expression);
        self.erase_postfix_type_assertion(expression, node.span(self.ast));
    }

    fn visit_ts_satisfies_expression(&mut self, node: TsSatisfiesExpression) {
        let expression = node.expression(self.ast);
        self.visit_chain_operand(expression);
        self.erase_postfix_type_assertion(expression, node.span(self.ast));
    }

    fn visit_ts_non_null_expression(&mut self, node: TsNonNullExpression) {
        self.visit_chain_operand(node.expression(self.ast));
        self.erase_suffix_expression(node.expression(self.ast), node.span(self.ast));
    }

    fn visit_ts_instantiation_expression(&mut self, node: TsInstantiationExpression) {
        self.visit_chain_operand(node.expression(self.ast));
        node.type_arguments(self.ast).visit_with(self);
    }

    fn visit_ts_type_assertion(&mut self, node: TsTypeAssertion) {
        let wrapper_start = node.span(self.ast).start;
        let expression_start = node.expression(self.ast).span(self.ast).start;
        self.report_erasable_syntax_only(Span::new(wrapper_start, expression_start));
        self.edits.blank(Span::new(wrapper_start, expression_start));
        node.expression(self.ast).visit_with(self);
    }

    fn visit_variable_declarator(&mut self, node: VariableDeclarator) {
        if node.definite(self.ast) {
            let pattern = node.id(self.ast);
            let span = pattern.span(self.ast);
            let marker_end = node
                .type_annotation(self.ast)
                .map_or(node.span(self.ast).end, |annotation| {
                    annotation.span(self.ast).start
                });
            if let Some(marker) =
                self.tokens
                    .find_backward(span.start, marker_end, TokenTag::LogicalNot)
            {
                self.edits.blank(marker.span());
            }
        }
        node.visit_children_with(self);
    }

    fn visit_formal_parameter(&mut self, node: FormalParameter) {
        if node.optional(self.ast) {
            self.erase_optional_parameter(node.type_annotation(self.ast), node.span(self.ast));
        }
        node.visit_children_with(self);
    }
}

fn earliest_position(current: Option<u32>, candidate: Option<u32>) -> Option<u32> {
    match (current, candidate) {
        (Some(current), Some(candidate)) => Some(current.min(candidate)),
        (Some(current), None) => Some(current),
        (None, candidate) => candidate,
    }
}

fn modifiers_start(member_start: u32, decorators: SubRange<Decorator>, ast: &Ast<'_>) -> u32 {
    let Some(decorator) = decorators.last() else {
        return member_start;
    };
    ast.get_node_in_sub_range(decorator).span(ast).end
}

fn accessibility_token(accessibility: Accessibility) -> Option<TokenTag> {
    match accessibility {
        Accessibility::None => None,
        Accessibility::Public => Some(TokenTag::Public),
        Accessibility::Private => Some(TokenTag::Private),
        Accessibility::Protected => Some(TokenTag::Protected),
    }
}

fn is_whole_type_declaration(declaration: Decl, ast: &Ast<'_>) -> bool {
    if declaration.is_ts_function(ast)
        || declaration.is_ts_type_alias_declaration(ast)
        || declaration.is_ts_interface_declaration(ast)
    {
        return true;
    }
    if let Some(class) = declaration.as_class(ast) {
        return class.declare(ast);
    }
    if let Some(variable) = declaration.as_variable_declaration(ast) {
        return variable.declare(ast);
    }
    false
}
