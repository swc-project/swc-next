//! Fixed-width type stripping for the TypeScript `erasableSyntaxOnly` subset.

#![deny(clippy::all)]
#![deny(unused)]

use std::borrow::Cow;

use edit::{EditPlan, FixedSubstitution};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Accessibility, ArrayPattern, ArrowFunctionExpression, AssignmentPattern, Ast,
    BindingIdentifier, BindingPattern, BindingRestElement, Class, ClassElement, Decl, Decorator,
    DoWhileStatement, ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, Expr,
    ForInStatement, ForOfStatement, ForStatement, FormalParameters, FunctionBody, GetSpan,
    IfStatement, ImportDeclaration, ImportOrExportKind, LabeledStatement, MethodDefinition,
    ObjectPattern, Program, PropertyDefinition, ReturnStatement, Stmt, ThrowStatement,
    TsAsExpression, TsEnumDeclaration, TsExportAssignment, TsFunction, TsGlobalDeclaration,
    TsImportEqualsDeclaration, TsIndexSignature, TsInterfaceDeclaration, TsMethodDefinition,
    TsModuleBlock, TsModuleDeclaration, TsModuleDeclarationKind, TsNamespaceExportDeclaration,
    TsNonNullExpression, TsParameterProperty, TsSatisfiesExpression, TsTypeAliasDeclaration,
    TsTypeAnnotation, TsTypeAssertion, TsTypeParameterDeclaration, TsTypeParameterInstantiation,
    TypedSubRange, VariableDeclaration, VariableDeclarator, Visit, VisitWith, WhileStatement,
    WithStatement, YieldExpression,
};
pub use swc_next_ecma_ast::{Diagnostic, Label, Lang, Severity, Span};
pub use swc_next_ecma_parser::Options;
use swc_next_ecma_parser::{ParseReturn, Parser, TokenParserConfig, TokenTag};

use crate::token_cursor::TokenCursor;

mod edit;
mod simd;
mod token_cursor;

const UNSUPPORTED_LANG_MESSAGE: &str = "TypeError: Unsupported lang variant";
const ERASABLE_SYNTAX_ONLY_MESSAGE: &str =
    "SyntaxError: This syntax is not allowed when `erasableSyntaxOnly` is enabled";

/// The stripped source and all diagnostics produced while processing it.
#[derive(Debug, Eq, PartialEq)]
pub struct StripResult<'source> {
    /// Best-effort output with the same UTF-16 length and line layout as the source.
    ///
    /// The original source is borrowed when the output is unchanged.
    pub code: Cow<'source, str>,
    /// Diagnostics in source order.
    ///
    /// Parser diagnostics are retained. At most one type-stripping diagnostic is
    /// added, for the first unsupported construct encountered.
    ///
    /// When this is non-empty, `code` is recovery output and is not guaranteed
    /// to preserve the input program's semantics.
    pub diagnostics: Vec<Diagnostic<'static>>,
}

/// Strips TypeScript syntax while preserving source positions.
///
/// Only [`Lang::Ts`] and [`Lang::Tsx`] are supported in [`Options::lang`].
/// Other language variants return the unchanged source with an error diagnostic.
/// Parentheses are preserved regardless of [`Options::preserve_parens`]. Diagnostics
/// do not prevent recovery output.
pub fn strip_types(source: &str, options: Options) -> StripResult<'_> {
    if !matches!(options.lang, Lang::Ts | Lang::Tsx) {
        return StripResult {
            code: Cow::Borrowed(source),
            diagnostics: vec![Diagnostic::error(UNSUPPORTED_LANG_MESSAGE, Span::new(0, 0))],
        };
    }

    let mut parser_options = options;
    parser_options.preserve_parens = true;
    let allocator = Allocator::default();
    let parse = Parser::init(&allocator, source, parser_options, TokenParserConfig).parse();
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
        list_item: ListItemContext::default(),
        containing_statement_or_property_end: None,
        pending_arrow_prefix_end: None,
        current_arrow_prefix_end: None,
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

#[derive(Clone, Copy, Default)]
struct ListItemContext {
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
    list_item: ListItemContext,
    containing_statement_or_property_end: Option<u32>,
    pending_arrow_prefix_end: Option<u32>,
    current_arrow_prefix_end: Option<u32>,
}

impl TypeEraser<'_, '_, '_> {
    fn report_erasable_syntax_only(&mut self, span: Span) {
        if self.diagnostic.is_none() {
            self.diagnostic = Some(Diagnostic::error(ERASABLE_SYNTAX_ONLY_MESSAGE, span));
        }
    }

    fn erase_token(&mut self, start: u32, end: u32, tag: TokenTag) -> Option<u32> {
        let token = self.tokens.find_forward(start, end, tag)?;
        self.edits.blank(token.span);
        Some(token.span.start)
    }

    fn erase_optional_pattern(
        &mut self,
        optional: bool,
        type_annotation: Option<TsTypeAnnotation>,
        span: Span,
    ) {
        if !optional {
            return;
        }

        let marker_end =
            type_annotation.map_or(span.end, |annotation| annotation.span(self.ast).start);
        if let Some(marker) = self
            .tokens
            .find_backward(span.start, marker_end, TokenTag::Question)
        {
            self.edits.blank(marker.span);
        }
    }

    fn erase_list_item(&mut self, item: Span, next: Option<Span>, container: Span) {
        let gap_end = next.map_or_else(
            || {
                self.tokens
                    .find_forward(item.end, container.end, TokenTag::RightBrace)
                    .map_or(container.end, |closing| closing.span.start)
            },
            |next| next.start,
        );
        if let Some(comma) = self.tokens.find_forward(item.end, gap_end, TokenTag::Comma) {
            self.edits.blank(Span::new(item.start, comma.span.end));
        } else {
            self.edits.blank(item);
        }
    }

    fn erase_statement(&mut self, span: Span) {
        self.edits.blank(span);
        self.list_item.erased = true;

        let terminates_directive_prologue = self.list_item.terminates_directive_prologue;
        let prevents_asi = self.needs_semicolon_before_erasure();
        if !terminates_directive_prologue && !prevents_asi {
            return;
        }
        let Some(first) = self.tokens.first_in_range(span.start, span.end) else {
            return;
        };
        let Some(replacement_span) = self.tokens.leading_character_span(first.span.start) else {
            return;
        };
        self.edits
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn needs_semicolon_before_erasure(&self) -> bool {
        if self.list_item.required_statement {
            return true;
        }

        let Some(previous) = self.list_item.previous_retained else {
            return false;
        };
        let last = self.tokens.last_in_range(previous.start, previous.end);
        !last.is_some_and(|token| token.tag == TokenTag::Semicolon)
    }

    fn erase_suffix_expression(&mut self, expression: Expr, wrapper_span: Span) {
        let expression_span = expression.span(self.ast);
        self.edits
            .blank(Span::new(expression_span.end, wrapper_span.end));

        if !self.ends_containing_statement(wrapper_span) {
            return;
        }
        let Some(replacement_span) = self.tokens.leading_character_span(expression_span.end) else {
            return;
        };
        self.edits
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn ends_containing_statement(&self, wrapper_span: Span) -> bool {
        if self.tokens.byte_at(wrapper_span.end) == Some(b';') {
            return false;
        }

        self.containing_statement_or_property_end == Some(wrapper_span.end)
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
        self.edits.blank(opening.span);
    }

    fn has_line_sensitive_arrow_prefix(&self, arrow: ArrowFunctionExpression) -> bool {
        if arrow.r#async(self.ast) {
            return true;
        }

        let Some(keyword_end) = self.current_arrow_prefix_end else {
            return false;
        };
        let Some(type_parameters) = arrow.type_parameters(self.ast) else {
            return false;
        };
        !self
            .tokens
            .has_line_terminator(keyword_end, type_parameters.span(self.ast).start)
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
        key_span: Span,
        separator_start: Option<u32>,
        has_value: bool,
    ) {
        if has_value {
            return;
        }
        let Some(separator_start) = separator_start else {
            return;
        };
        let Some(key) = self.tokens.first_in_range(key_span.start, key_span.end) else {
            return;
        };
        if !matches!(key.tag, TokenTag::Get | TokenTag::Set | TokenTag::Static) {
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
        statements: TypedSubRange<Stmt>,
        terminates_directive_prologue: bool,
    ) {
        let saved_context = self.list_item;
        let mut previous_retained = None;
        for position in 0..statements.len() {
            let Some(statement) = statements.get_node(self.ast, position) else {
                continue;
            };
            self.list_item = ListItemContext {
                previous_retained,
                terminates_directive_prologue: terminates_directive_prologue && position == 0,
                required_statement: false,
                erased: false,
            };
            statement.visit_with(self);
            if !self.list_item.erased {
                previous_retained = Some(statement.span(self.ast));
            }
        }
        self.list_item = saved_context;
    }

    fn visit_class_element_list(&mut self, elements: TypedSubRange<ClassElement>) {
        let saved_context = self.list_item;
        let mut previous_retained = None;
        for position in 0..elements.len() {
            let Some(element) = elements.get_node(self.ast, position) else {
                continue;
            };
            self.list_item = ListItemContext {
                previous_retained,
                ..ListItemContext::default()
            };
            element.visit_with(self);
            if !self.list_item.erased {
                previous_retained = Some(element.span(self.ast));
            }
        }
        self.list_item = saved_context;
    }

    fn visit_required_statement(&mut self, statement: Stmt) {
        let saved_context = self.list_item;
        self.list_item = ListItemContext {
            required_statement: true,
            ..ListItemContext::default()
        };
        statement.visit_with(self);
        self.list_item = saved_context;
    }

    fn visit_expr_with_arrow_prefix(&mut self, expression: Expr, keyword_end: u32) {
        let saved_prefix = self.pending_arrow_prefix_end.replace(keyword_end);
        expression.visit_with(self);
        self.pending_arrow_prefix_end = saved_prefix;
    }
}

impl<'ast> Visit<'ast> for TypeEraser<'ast, '_, '_> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_program(&mut self, node: Program) {
        node.directives(self.ast).visit_with(self);
        self.visit_statement_list(node.body(self.ast), true);
        node.hashbang(self.ast).visit_with(self);
    }

    fn visit_function_body(&mut self, node: FunctionBody) {
        node.directives(self.ast).visit_with(self);
        self.visit_statement_list(node.body(self.ast), true);
    }

    fn visit_ts_module_block(&mut self, node: TsModuleBlock) {
        node.directives(self.ast).visit_with(self);
        self.visit_statement_list(node.body(self.ast), true);
    }

    fn visit_stmts(&mut self, node: TypedSubRange<Stmt>) {
        self.visit_statement_list(node, false);
    }

    fn visit_class_elements(&mut self, node: TypedSubRange<ClassElement>) {
        self.visit_class_element_list(node);
    }

    fn visit_stmt(&mut self, node: Stmt) {
        let saved_end = self
            .containing_statement_or_property_end
            .replace(node.span(self.ast).end);
        node.visit_children_with(self);
        self.containing_statement_or_property_end = saved_end;
    }

    fn visit_expr(&mut self, node: Expr) {
        let prefix_end = self.pending_arrow_prefix_end.take();
        let saved_prefix = self.current_arrow_prefix_end;
        self.current_arrow_prefix_end = prefix_end;
        node.visit_children_with(self);
        self.current_arrow_prefix_end = saved_prefix;
    }

    fn visit_if_statement(&mut self, node: IfStatement) {
        node.test(self.ast).visit_with(self);
        self.visit_required_statement(node.consequent(self.ast));
        if let Some(alternate) = node.alternate(self.ast) {
            self.visit_required_statement(alternate);
        }
    }

    fn visit_labeled_statement(&mut self, node: LabeledStatement) {
        node.label(self.ast).visit_with(self);
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

    fn visit_return_statement(&mut self, node: ReturnStatement) {
        if let Some(argument) = node.argument(self.ast) {
            self.visit_expr_with_arrow_prefix(argument, node.span(self.ast).start + 6);
        }
    }

    fn visit_throw_statement(&mut self, node: ThrowStatement) {
        self.visit_expr_with_arrow_prefix(node.argument(self.ast), node.span(self.ast).start + 5);
    }

    fn visit_yield_expression(&mut self, node: YieldExpression) {
        if let Some(argument) = node.argument(self.ast) {
            self.visit_expr_with_arrow_prefix(argument, node.span(self.ast).start + 5);
        }
    }

    fn visit_arrow_function_expression(&mut self, node: ArrowFunctionExpression) {
        self.move_opening_parenthesis_across_multiline_type_parameters(node);

        let Some(return_type) = node.return_type(self.ast) else {
            node.visit_children_with(self);
            return;
        };
        let return_span = return_type.span(self.ast);
        let Some(arrow) =
            self.tokens
                .find_forward(return_span.end, node.span(self.ast).end, TokenTag::Arrow)
        else {
            node.visit_children_with(self);
            return;
        };
        let parameters = node.params(self.ast).span(self.ast);
        if !self
            .tokens
            .has_line_terminator(parameters.end, arrow.span.start)
        {
            node.visit_children_with(self);
            return;
        }
        let Some(closing_parenthesis) =
            self.tokens
                .find_backward(parameters.start, parameters.end, TokenTag::RightParen)
        else {
            node.visit_children_with(self);
            return;
        };
        let Some(replacement_span) = self.tokens.trailing_character_span(return_span.end) else {
            node.visit_children_with(self);
            return;
        };

        self.edits
            .blank(Span::new(closing_parenthesis.span.start, return_span.end));
        self.edits
            .substitute(replacement_span, FixedSubstitution::RightParenthesis);
        node.visit_children_with(self);
    }

    fn visit_formal_parameters(&mut self, node: FormalParameters) {
        let items = node.items(self.ast);
        let Some(first_item) = items.get_node(self.ast, 0) else {
            node.visit_children_with(self);
            return;
        };
        let Some(first_parameter) = first_item.as_formal_parameter(self.ast) else {
            node.visit_children_with(self);
            return;
        };
        if !first_parameter
            .pattern(self.ast)
            .is_ts_this_parameter(self.ast)
        {
            node.visit_children_with(self);
            return;
        }

        let first_span = first_item.span(self.ast);
        let search_end = items
            .get_node(self.ast, 1)
            .map(|item| item.span(self.ast).start)
            .or_else(|| node.rest(self.ast).map(|rest| rest.span(self.ast).start))
            .unwrap_or_else(|| node.span(self.ast).end);
        let erase_end = self
            .tokens
            .find_forward(first_span.end, search_end, TokenTag::Comma)
            .map_or(first_span.end, |comma| comma.span.end);
        self.edits.blank(Span::new(first_span.start, erase_end));
        node.visit_children_with(self);
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
        if let (Some(first), Some(last)) = (
            implements.get_node(self.ast, 0),
            implements.get_node(self.ast, implements.len().saturating_sub(1)),
        ) {
            let first_span = first.span(self.ast);
            let last_span = last.span(self.ast);
            if let Some(keyword) =
                self.tokens
                    .find_backward(span.start, first_span.start, TokenTag::Implements)
            {
                self.edits
                    .blank(Span::new(keyword.span.start, last_span.end));
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
                .is_some_and(|token| matches!(token.tag, TokenTag::In | TokenTag::Instanceof));
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
            key_span,
            separator_start,
            node.value(self.ast).is_some(),
        );

        let saved_end = self.containing_statement_or_property_end.replace(span.end);
        node.visit_children_with(self);
        self.containing_statement_or_property_end = saved_end;
    }

    fn visit_method_definition(&mut self, node: MethodDefinition) {
        let span = node.span(self.ast);
        let key_span = node.key(self.ast).span(self.ast);
        let key_token = self.tokens.first_in_range(key_span.start, key_span.end);
        let modifiers_start = modifiers_start(span.start, node.decorators(self.ast), self.ast);
        let mut first_erased_modifier = None;
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
                .is_some_and(|token| matches!(token.tag, TokenTag::In | TokenTag::Instanceof));
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
        let span = node.span(self.ast);
        if node.declare(self.ast) {
            self.erase_statement(span);
            return;
        }
        self.report_erasable_syntax_only(span);
        node.visit_children_with(self);
    }

    fn visit_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        let span = node.span(self.ast);
        let is_type_only = is_type_only_module(node, self.ast);
        if !is_type_only {
            self.report_erasable_syntax_only(span);
        }
        self.erase_statement(span);
        if !is_type_only {
            node.visit_children_with(self);
        }
    }

    fn visit_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        self.erase_statement(node.span(self.ast));
    }

    fn visit_ts_parameter_property(&mut self, node: TsParameterProperty) {
        let span = node.span(self.ast);
        let decorators = node.decorators(self.ast);
        let modifiers_start = modifiers_start(span.start, decorators, self.ast);
        let parameter_start = node.parameter(self.ast).span(self.ast).start;
        let diagnostic_span = self
            .tokens
            .first_in_range(modifiers_start, parameter_start)
            .map_or(span, |modifier| modifier.span);

        self.report_erasable_syntax_only(diagnostic_span);
        self.edits
            .blank(Span::new(modifiers_start, parameter_start));
        node.visit_children_with(self);
    }

    fn visit_ts_export_assignment(&mut self, node: TsExportAssignment) {
        self.report_erasable_syntax_only(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        let span = node.span(self.ast);
        if node.import_kind(self.ast) == ImportOrExportKind::Type {
            self.erase_statement(span);
            return;
        }
        self.report_erasable_syntax_only(span);
        node.visit_children_with(self);
    }

    fn visit_import_declaration(&mut self, node: ImportDeclaration) {
        let container = node.span(self.ast);
        if node.import_kind(self.ast) == ImportOrExportKind::Type {
            self.erase_statement(container);
            return;
        }

        let specifiers = node.specifiers(self.ast);
        for position in 0..specifiers.len() {
            let Some(specifier) = specifiers.get_node(self.ast, position) else {
                continue;
            };
            let Some(specifier) = specifier.as_import_specifier(self.ast) else {
                continue;
            };
            if specifier.import_kind(self.ast) != ImportOrExportKind::Type {
                continue;
            }

            let item = specifier.span(self.ast);
            let next = specifiers
                .get_node(self.ast, position + 1)
                .map(|specifier| specifier.span(self.ast));
            self.erase_list_item(item, next, container);
        }
        node.visit_children_with(self);
    }

    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        let container = node.span(self.ast);
        if node.export_kind(self.ast) == ImportOrExportKind::Type
            || node
                .declaration(self.ast)
                .is_some_and(|declaration| is_whole_type_declaration(declaration, self.ast))
        {
            self.erase_statement(container);
            return;
        }

        let specifiers = node.specifiers(self.ast);
        for position in 0..specifiers.len() {
            let Some(specifier) = specifiers.get_node(self.ast, position) else {
                continue;
            };
            if specifier.export_kind(self.ast) != ImportOrExportKind::Type {
                continue;
            }

            let item = specifier.span(self.ast);
            let next = specifiers
                .get_node(self.ast, position + 1)
                .map(|specifier| specifier.span(self.ast));
            self.erase_list_item(item, next, container);
        }
        node.visit_children_with(self);
    }

    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        if node.export_kind(self.ast) == ImportOrExportKind::Type {
            self.erase_statement(node.span(self.ast));
            return;
        }
        node.visit_children_with(self);
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
        self.erase_suffix_expression(node.expression(self.ast), node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_ts_satisfies_expression(&mut self, node: TsSatisfiesExpression) {
        self.erase_suffix_expression(node.expression(self.ast), node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_ts_non_null_expression(&mut self, node: TsNonNullExpression) {
        self.erase_suffix_expression(node.expression(self.ast), node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_ts_type_assertion(&mut self, node: TsTypeAssertion) {
        let wrapper_start = node.span(self.ast).start;
        let expression_start = node.expression(self.ast).span(self.ast).start;
        self.report_erasable_syntax_only(Span::new(wrapper_start, expression_start));
        self.edits.blank(Span::new(wrapper_start, expression_start));
        node.visit_children_with(self);
    }

    fn visit_variable_declarator(&mut self, node: VariableDeclarator) {
        if node.definite(self.ast) {
            let pattern = node.id(self.ast);
            let span = pattern.span(self.ast);
            let marker_end = pattern_type_annotation(pattern, self.ast)
                .map_or(span.end, |annotation| annotation.span(self.ast).start);
            if let Some(marker) =
                self.tokens
                    .find_backward(span.start, marker_end, TokenTag::LogicalNot)
            {
                self.edits.blank(marker.span);
            }
        }
        node.visit_children_with(self);
    }

    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        self.erase_optional_pattern(
            node.optional(self.ast),
            node.type_annotation(self.ast),
            node.span(self.ast),
        );
        node.visit_children_with(self);
    }

    fn visit_assignment_pattern(&mut self, node: AssignmentPattern) {
        self.erase_optional_pattern(
            node.optional(self.ast),
            node.type_annotation(self.ast),
            node.span(self.ast),
        );
        node.visit_children_with(self);
    }

    fn visit_binding_rest_element(&mut self, node: BindingRestElement) {
        self.erase_optional_pattern(
            node.optional(self.ast),
            node.type_annotation(self.ast),
            node.span(self.ast),
        );
        node.visit_children_with(self);
    }

    fn visit_array_pattern(&mut self, node: ArrayPattern) {
        self.erase_optional_pattern(
            node.optional(self.ast),
            node.type_annotation(self.ast),
            node.span(self.ast),
        );
        node.visit_children_with(self);
    }

    fn visit_object_pattern(&mut self, node: ObjectPattern) {
        self.erase_optional_pattern(
            node.optional(self.ast),
            node.type_annotation(self.ast),
            node.span(self.ast),
        );
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

fn modifiers_start<'ast>(
    member_start: u32,
    decorators: TypedSubRange<Decorator>,
    ast: &Ast<'ast>,
) -> u32 {
    decorators
        .get_node(ast, decorators.len().saturating_sub(1))
        .map_or(member_start, |decorator| decorator.span(ast).end)
}

fn pattern_type_annotation(
    pattern: BindingPattern,
    ast: &swc_next_ecma_ast::Ast<'_>,
) -> Option<TsTypeAnnotation> {
    if let Some(pattern) = pattern.as_binding_identifier(ast) {
        return pattern.type_annotation(ast);
    }
    if let Some(pattern) = pattern.as_assignment_pattern(ast) {
        return pattern.type_annotation(ast);
    }
    if let Some(pattern) = pattern.as_binding_rest_element(ast) {
        return pattern.type_annotation(ast);
    }
    if let Some(pattern) = pattern.as_array_pattern(ast) {
        return pattern.type_annotation(ast);
    }
    pattern
        .as_object_pattern(ast)
        .and_then(|pattern| pattern.type_annotation(ast))
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
        || declaration.is_ts_global_declaration(ast)
    {
        return true;
    }
    if let Some(class) = declaration.as_class(ast) {
        return class.declare(ast);
    }
    if let Some(variable) = declaration.as_variable_declaration(ast) {
        return variable.declare(ast);
    }
    if let Some(enumeration) = declaration.as_ts_enum_declaration(ast) {
        return enumeration.declare(ast);
    }
    if let Some(module) = declaration.as_ts_module_declaration(ast) {
        return is_type_only_module(module, ast);
    }
    false
}

fn is_type_only_module(module: TsModuleDeclaration, ast: &Ast<'_>) -> bool {
    if module.kind(ast) == TsModuleDeclarationKind::Module {
        return module.id(ast).is_string_literal(ast);
    }
    if module.declare(ast) {
        return true;
    }

    let Some(body) = module.body(ast) else {
        return false;
    };
    if let Some(nested) = body.as_ts_module_declaration(ast) {
        return is_type_only_module(nested, ast);
    }
    let Some(block) = body.as_ts_module_block(ast) else {
        return false;
    };
    let statements = block.body(ast);
    for position in 0..statements.len() {
        let Some(statement) = statements.get_node(ast, position) else {
            return false;
        };
        if statement_has_runtime_effect(statement, ast) {
            return false;
        }
    }
    true
}

fn statement_has_runtime_effect(statement: Stmt, ast: &Ast<'_>) -> bool {
    if let Some(declaration) = statement.as_declaration(ast) {
        return declaration_has_runtime_effect(declaration, ast);
    }
    if statement.is_import_declaration(ast) {
        return false;
    }
    if let Some(export) = statement.as_export_named_declaration(ast) {
        if let Some(declaration) = export.declaration(ast) {
            if declaration.is_ts_import_equals_declaration(ast) {
                return true;
            }
            return declaration_has_runtime_effect(declaration, ast);
        }
        return export.export_kind(ast) != ImportOrExportKind::Type;
    }
    !statement.is_ts_namespace_export_declaration(ast)
}

fn declaration_has_runtime_effect(declaration: Decl, ast: &Ast<'_>) -> bool {
    if declaration.is_ts_type_alias_declaration(ast) {
        return false;
    }
    if declaration.is_ts_interface_declaration(ast) {
        return false;
    }
    if let Some(import_equals) = declaration.as_ts_import_equals_declaration(ast) {
        return import_equals.import_kind(ast) != ImportOrExportKind::Type;
    }
    if let Some(module) = declaration.as_ts_module_declaration(ast) {
        return !is_type_only_module(module, ast);
    }
    true
}

#[cfg(test)]
mod tests;
