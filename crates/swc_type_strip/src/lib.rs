//! Fixed-width type stripping for the TypeScript `erasableSyntaxOnly` subset.

#![deny(clippy::all)]
#![deny(unused)]

use std::borrow::Cow;

use edit::{EditPlan, FixedSubstitution};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Accessibility, ArrayPattern, ArrowFunctionExpression, AssignmentPattern, Ast, AstNode,
    BindingIdentifier, BindingPattern, BindingRestElement, Class, Decl, Decorator,
    ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, Expr, FormalParameters,
    GetSpan, ImportDeclaration, ImportOrExportKind, MethodDefinition, NodeId, NodeKind,
    NodeKindData, ObjectPattern, PropertyDefinition, Stmt, Traverse, TraverseAction, TraverseCtx,
    TraverseWith, TsAsExpression, TsEnumDeclaration, TsExportAssignment, TsFunction,
    TsGlobalDeclaration, TsImportEqualsDeclaration, TsIndexSignature, TsInterfaceDeclaration,
    TsMethodDefinition, TsModuleDeclaration, TsModuleDeclarationKind, TsNamespaceExportDeclaration,
    TsNonNullExpression, TsParameterProperty, TsSatisfiesExpression, TsTypeAliasDeclaration,
    TsTypeAnnotation, TsTypeAssertion, TsTypeParameterDeclaration, TsTypeParameterInstantiation,
    TypedSubRange, VariableDeclaration, VariableDeclarator,
};
pub use swc_next_ecma_ast::{Diagnostic, Label, Lang, Severity, Span};
pub use swc_next_ecma_parser::Options;
use swc_next_ecma_parser::{CommentMode, ParseReturn, Parser, TokenParserConfig, TokenTag};

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
/// do not prevent recovery output. Comments are preserved from the original source,
/// so [`Options::comments`] does not affect the result.
pub fn strip_types(source: &str, options: Options) -> StripResult<'_> {
    if !matches!(options.lang, Lang::Ts | Lang::Tsx) {
        return StripResult {
            code: Cow::Borrowed(source),
            diagnostics: vec![Diagnostic::error(UNSUPPORTED_LANG_MESSAGE, Span::new(0, 0))],
        };
    }

    let mut parser_options = options;
    parser_options.preserve_parens = true;
    parser_options.comments = CommentMode::None;
    let allocator = Allocator::default();
    let parse = Parser::init(&allocator, source, parser_options, TokenParserConfig).parse();
    let ParseReturn {
        mut ast,
        tokens,
        mut diagnostics,
    } = parse;

    let program = ast.root_program();
    let node_count = ast.node_count();
    let mut context = TraverseCtx::new(&mut ast, EditPlan::default());
    let mut eraser = TypeEraser {
        tokens: TokenCursor::new(source, tokens.as_slice()),
        diagnostic: None,
        previous_statement: vec![None; node_count],
        last_statement_by_parent: vec![None; node_count],
        erased_statements: vec![false; node_count],
    };
    let _ = program.traverse_with(&mut eraser, &mut context);
    let edits = std::mem::take(&mut context.state);
    let diagnostic = eraser.diagnostic.take();
    drop(context);
    drop(eraser);
    diagnostics.extend(diagnostic);
    diagnostics.sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.span.end));

    StripResult {
        code: edits.render(source),
        diagnostics,
    }
}

struct TypeEraser<'source, 'tokens> {
    tokens: TokenCursor<'source, 'tokens>,
    diagnostic: Option<Diagnostic<'static>>,
    previous_statement: Vec<Option<NodeId>>,
    last_statement_by_parent: Vec<Option<NodeId>>,
    erased_statements: Vec<bool>,
}

impl TypeEraser<'_, '_> {
    fn report_erasable_syntax_only(&mut self, span: Span) {
        if self.diagnostic.is_none() {
            self.diagnostic = Some(Diagnostic::error(ERASABLE_SYNTAX_ONLY_MESSAGE, span));
        }
    }

    fn erase_token(
        &self,
        start: u32,
        end: u32,
        tag: TokenTag,
        edits: &mut EditPlan,
    ) -> Option<u32> {
        let token = self.tokens.find_forward(start, end, tag)?;
        edits.blank(token.span);
        Some(token.span.start)
    }

    fn erase_optional_pattern<'ast>(
        &self,
        optional: bool,
        type_annotation: Option<TsTypeAnnotation>,
        span: Span,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) {
        if !optional {
            return;
        }

        let marker_end =
            type_annotation.map_or(span.end, |annotation| annotation.span(context.ast).start);
        if let Some(marker) = self
            .tokens
            .find_backward(span.start, marker_end, TokenTag::Question)
        {
            context.state.blank(marker.span);
        }
    }

    fn erase_list_item<'ast>(
        &self,
        item: Span,
        next: Option<Span>,
        container: Span,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) {
        let gap_end = next.map_or_else(
            || {
                self.tokens
                    .find_forward(item.end, container.end, TokenTag::RightBrace)
                    .map_or(container.end, |closing| closing.span.start)
            },
            |next| next.start,
        );
        if let Some(comma) = self.tokens.find_forward(item.end, gap_end, TokenTag::Comma) {
            context.state.blank(Span::new(item.start, comma.span.end));
        } else {
            context.state.blank(item);
        }
    }

    fn erase_statement<'ast>(&mut self, span: Span, context: &mut TraverseCtx<'_, 'ast, EditPlan>) {
        context.state.blank(span);

        let Some(current) = context.current_node() else {
            return;
        };
        self.erased_statements[current.index()] = true;
        let terminates_directive_prologue = self.is_first_non_directive_statement(current, context);
        let prevents_asi = self.needs_semicolon_before_erasure(current, context);
        if !terminates_directive_prologue && !prevents_asi {
            return;
        }
        let Some(first) = self.tokens.first_in_range(span.start, span.end) else {
            return;
        };
        let Some(replacement_span) = self.tokens.leading_character_span(first.span.start) else {
            return;
        };
        context
            .state
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn is_first_non_directive_statement<'ast>(
        &self,
        current: NodeId,
        context: &TraverseCtx<'_, 'ast, EditPlan>,
    ) -> bool {
        let Some(parent) = context.parent() else {
            return false;
        };
        let first = match context.ast.kind_data(parent) {
            NodeKindData::Program(program) => program.body(context.ast).get_node(context.ast, 0),
            NodeKindData::FunctionBody(body) => body.body(context.ast).get_node(context.ast, 0),
            NodeKindData::TsModuleBlock(block) => block.body(context.ast).get_node(context.ast, 0),
            _ => None,
        };
        first.is_some_and(|statement| statement.node_id() == current)
    }

    fn needs_semicolon_before_erasure<'ast>(
        &self,
        current: NodeId,
        context: &TraverseCtx<'_, 'ast, EditPlan>,
    ) -> bool {
        let Some(parent) = context.parent() else {
            return false;
        };
        if is_required_statement_parent(context.node_kind(parent)) {
            return true;
        }

        let mut previous = self.previous_statement[current.index()];
        while let Some(node_id) = previous {
            if self.erased_statements[node_id.index()] {
                previous = self.previous_statement[node_id.index()];
                continue;
            }

            let span = context.ast.span(node_id);
            let last = self.tokens.last_in_range(span.start, span.end);
            return !last.is_some_and(|token| token.tag == TokenTag::Semicolon);
        }
        false
    }

    fn erase_suffix_expression<'ast>(
        &mut self,
        expression: Expr,
        wrapper_span: Span,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) {
        let expression_span = expression.span(context.ast);
        context
            .state
            .blank(Span::new(expression_span.end, wrapper_span.end));
    }

    fn erase_postfix_type_assertion<'ast>(
        &mut self,
        expression: Expr,
        wrapper_span: Span,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) {
        let expression_span = expression.span(context.ast);
        self.erase_suffix_expression(expression, wrapper_span, context);

        let Some(next) = self.tokens.at_or_after(wrapper_span.end) else {
            return;
        };
        if !next.has_line_terminator_before()
            || !matches!(
                next.tag,
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
        context
            .state
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn move_opening_parenthesis_across_multiline_type_parameters<'ast>(
        &self,
        arrow: ArrowFunctionExpression,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) {
        let Some(type_parameters) = arrow.type_parameters(context.ast) else {
            return;
        };
        if !self.has_line_sensitive_arrow_prefix(arrow, context) {
            return;
        }

        let type_parameters = type_parameters.span(context.ast);
        let parameters = arrow.params(context.ast).span(context.ast);
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

        context
            .state
            .substitute(replacement_span, FixedSubstitution::LeftParenthesis);
        context.state.blank(opening.span);
    }

    fn has_line_sensitive_arrow_prefix<'ast>(
        &self,
        arrow: ArrowFunctionExpression,
        context: &TraverseCtx<'_, 'ast, EditPlan>,
    ) -> bool {
        if arrow.r#async(context.ast) {
            return true;
        }

        let Some(parent) = context.parent() else {
            return false;
        };
        let keyword_length = match context.node_kind(parent) {
            NodeKind::ReturnStatement => 6,
            NodeKind::ThrowStatement | NodeKind::YieldExpression => 5,
            _ => return false,
        };
        let Some(type_parameters) = arrow.type_parameters(context.ast) else {
            return false;
        };
        let keyword_end = context.ast.span(parent).start + keyword_length;
        !self
            .tokens
            .has_line_terminator(keyword_end, type_parameters.span(context.ast).start)
    }

    fn separate_hazardous_class_element<'ast>(
        &self,
        member_span: Span,
        first_erased_modifier: Option<u32>,
        hazardous_key: bool,
        protected_by_javascript_syntax: bool,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) {
        if !hazardous_key || protected_by_javascript_syntax {
            return;
        }
        if first_erased_modifier != Some(member_span.start) {
            return;
        }
        let Some(current) = context.current_node() else {
            return;
        };
        if !self.needs_semicolon_before_erasure(current, context) {
            return;
        }
        let Some(replacement_span) = self.tokens.leading_character_span(member_span.start) else {
            return;
        };
        context
            .state
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }

    fn preserve_keyword_named_class_field<'ast>(
        &self,
        key_span: Span,
        separator_start: Option<u32>,
        has_value: bool,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
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
        context
            .state
            .substitute(replacement_span, FixedSubstitution::Semicolon);
    }
}

impl<'ast> Traverse<'ast, EditPlan> for TypeEraser<'_, '_> {
    fn enter_node<T: AstNode>(
        &mut self,
        node_id: NodeId,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<T> {
        let Some(parent) = context.parent() else {
            return TraverseAction::Proceed;
        };
        let parent_kind = context.node_kind(parent);
        let node_kind = context.node_kind(node_id);
        if !is_statement_list_item(node_kind, parent_kind) {
            return TraverseAction::Proceed;
        }

        self.previous_statement[node_id.index()] = self.last_statement_by_parent[parent.index()];
        self.last_statement_by_parent[parent.index()] = Some(node_id);
        TraverseAction::Proceed
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        self.move_opening_parenthesis_across_multiline_type_parameters(node, context);

        let Some(return_type) = node.return_type(context.ast) else {
            return TraverseAction::Proceed;
        };
        let return_span = return_type.span(context.ast);
        let Some(arrow) =
            self.tokens
                .find_forward(return_span.end, node.span(context.ast).end, TokenTag::Arrow)
        else {
            return TraverseAction::Proceed;
        };
        let parameters = node.params(context.ast).span(context.ast);
        if !self
            .tokens
            .has_line_terminator(parameters.end, arrow.span.start)
        {
            return TraverseAction::Proceed;
        }
        let Some(closing_parenthesis) =
            self.tokens
                .find_backward(parameters.start, parameters.end, TokenTag::RightParen)
        else {
            return TraverseAction::Proceed;
        };
        let Some(replacement_span) = self.tokens.trailing_character_span(return_span.end) else {
            return TraverseAction::Proceed;
        };

        context
            .state
            .blank(Span::new(closing_parenthesis.span.start, return_span.end));
        context
            .state
            .substitute(replacement_span, FixedSubstitution::RightParenthesis);
        TraverseAction::Proceed
    }

    fn enter_formal_parameters(
        &mut self,
        node: FormalParameters,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<FormalParameters> {
        let items = node.items(context.ast);
        let Some(first_item) = items.get_node(context.ast, 0) else {
            return TraverseAction::Proceed;
        };
        let Some(first_parameter) = first_item.as_formal_parameter(context.ast) else {
            return TraverseAction::Proceed;
        };
        if !first_parameter
            .pattern(context.ast)
            .is_ts_this_parameter(context.ast)
        {
            return TraverseAction::Proceed;
        }

        let first_span = first_item.span(context.ast);
        let search_end = items
            .get_node(context.ast, 1)
            .map(|item| item.span(context.ast).start)
            .or_else(|| {
                node.rest(context.ast)
                    .map(|rest| rest.span(context.ast).start)
            })
            .unwrap_or_else(|| node.span(context.ast).end);
        let erase_end = self
            .tokens
            .find_forward(first_span.end, search_end, TokenTag::Comma)
            .map_or(first_span.end, |comma| comma.span.end);
        context.state.blank(Span::new(first_span.start, erase_end));
        TraverseAction::Proceed
    }

    fn enter_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<VariableDeclaration> {
        if node.declare(context.ast) {
            self.erase_statement(node.span(context.ast), context);
            TraverseAction::Skip
        } else {
            TraverseAction::Proceed
        }
    }

    fn enter_class(
        &mut self,
        node: Class,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<Class> {
        let span = node.span(context.ast);
        if node.declare(context.ast) {
            self.erase_statement(span, context);
            return TraverseAction::Skip;
        }

        let identifier_start = node.id(context.ast).map_or_else(
            || node.body(context.ast).span(context.ast).start,
            |id| id.span(context.ast).start,
        );
        if node.r#abstract(context.ast) {
            let modifiers_start =
                modifiers_start(span.start, node.decorators(context.ast), context.ast);
            self.erase_token(
                modifiers_start,
                identifier_start,
                TokenTag::Abstract,
                &mut context.state,
            );
        }

        let implements = node.implements(context.ast);
        if let (Some(first), Some(last)) = (
            implements.get_node(context.ast, 0),
            implements.get_node(context.ast, implements.len().saturating_sub(1)),
        ) {
            let first_span = first.span(context.ast);
            let last_span = last.span(context.ast);
            if let Some(keyword) =
                self.tokens
                    .find_backward(span.start, first_span.start, TokenTag::Implements)
            {
                context
                    .state
                    .blank(Span::new(keyword.span.start, last_span.end));
            }
        }

        TraverseAction::Proceed
    }

    fn enter_property_definition(
        &mut self,
        node: PropertyDefinition,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<PropertyDefinition> {
        let span = node.span(context.ast);
        if node.declare(context.ast) || node.r#abstract(context.ast) {
            self.erase_statement(span, context);
            return TraverseAction::Skip;
        }

        let key_span = node.key(context.ast).span(context.ast);
        let key_token = self.tokens.first_in_range(key_span.start, key_span.end);
        let modifiers_start =
            modifiers_start(span.start, node.decorators(context.ast), context.ast);
        let mut first_erased_modifier = None;
        if node.readonly(context.ast) {
            let start = self.erase_token(
                modifiers_start,
                key_span.start,
                TokenTag::Readonly,
                &mut context.state,
            );
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        if node.r#override(context.ast) {
            let start = self.erase_token(
                modifiers_start,
                key_span.start,
                TokenTag::Override,
                &mut context.state,
            );
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        if let Some(tag) = accessibility_token(node.accessibility(context.ast)) {
            let start = self.erase_token(modifiers_start, key_span.start, tag, &mut context.state);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        let hazardous_key = node.computed(context.ast)
            || key_token
                .is_some_and(|token| matches!(token.tag, TokenTag::In | TokenTag::Instanceof));
        let protected_by_javascript_syntax = node.r#static(context.ast)
            || node.accessor(context.ast)
            || !node.decorators(context.ast).is_empty();
        self.separate_hazardous_class_element(
            span,
            first_erased_modifier,
            hazardous_key,
            protected_by_javascript_syntax,
            context,
        );

        let mut marker_end = span.end;
        if let Some(annotation) = node.type_annotation(context.ast) {
            marker_end = marker_end.min(annotation.span(context.ast).start);
        }
        if let Some(value) = node.value(context.ast) {
            marker_end = marker_end.min(value.span(context.ast).start);
        }
        let optional_start = if node.optional(context.ast) {
            self.erase_token(
                key_span.end,
                marker_end,
                TokenTag::Question,
                &mut context.state,
            )
        } else {
            None
        };
        let definite_start = if node.definite(context.ast) {
            self.erase_token(
                key_span.end,
                marker_end,
                TokenTag::LogicalNot,
                &mut context.state,
            )
        } else {
            None
        };
        let separator_start = node
            .type_annotation(context.ast)
            .map(|annotation| annotation.span(context.ast).start)
            .or(optional_start)
            .or(definite_start);
        self.preserve_keyword_named_class_field(
            key_span,
            separator_start,
            node.value(context.ast).is_some(),
            context,
        );

        TraverseAction::Proceed
    }

    fn enter_method_definition(
        &mut self,
        node: MethodDefinition,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<MethodDefinition> {
        let span = node.span(context.ast);
        let key_span = node.key(context.ast).span(context.ast);
        let key_token = self.tokens.first_in_range(key_span.start, key_span.end);
        let modifiers_start =
            modifiers_start(span.start, node.decorators(context.ast), context.ast);
        let mut first_erased_modifier = None;
        if node.r#override(context.ast) {
            let start = self.erase_token(
                modifiers_start,
                key_span.start,
                TokenTag::Override,
                &mut context.state,
            );
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        if let Some(tag) = accessibility_token(node.accessibility(context.ast)) {
            let start = self.erase_token(modifiers_start, key_span.start, tag, &mut context.state);
            first_erased_modifier = earliest_position(first_erased_modifier, start);
        }
        let hazardous_key = node.computed(context.ast)
            || node.value(context.ast).generator(context.ast)
            || key_token
                .is_some_and(|token| matches!(token.tag, TokenTag::In | TokenTag::Instanceof));
        let protected_by_javascript_syntax =
            node.r#static(context.ast) || !node.decorators(context.ast).is_empty();
        self.separate_hazardous_class_element(
            span,
            first_erased_modifier,
            hazardous_key,
            protected_by_javascript_syntax,
            context,
        );
        if node.optional(context.ast) {
            let function_start = node.value(context.ast).span(context.ast).start;
            self.erase_token(
                key_span.end,
                function_start,
                TokenTag::Question,
                &mut context.state,
            );
        }
        TraverseAction::Proceed
    }

    fn enter_ts_function(
        &mut self,
        node: TsFunction,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsFunction> {
        self.erase_statement(node.span(context.ast), context);
        TraverseAction::Skip
    }

    fn enter_ts_method_definition(
        &mut self,
        node: TsMethodDefinition,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsMethodDefinition> {
        self.erase_statement(node.span(context.ast), context);
        TraverseAction::Skip
    }

    fn enter_ts_index_signature(
        &mut self,
        node: TsIndexSignature,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsIndexSignature> {
        self.erase_statement(node.span(context.ast), context);
        TraverseAction::Skip
    }

    fn enter_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsEnumDeclaration> {
        let span = node.span(context.ast);
        if node.declare(context.ast) {
            self.erase_statement(span, context);
            TraverseAction::Skip
        } else {
            self.report_erasable_syntax_only(span);
            TraverseAction::Proceed
        }
    }

    fn enter_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsModuleDeclaration> {
        let span = node.span(context.ast);
        let is_type_only = is_type_only_module(node, context.ast);
        if !is_type_only {
            self.report_erasable_syntax_only(span);
        }
        self.erase_statement(span, context);
        if is_type_only {
            TraverseAction::Skip
        } else {
            TraverseAction::Proceed
        }
    }

    fn enter_ts_global_declaration(
        &mut self,
        node: TsGlobalDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsGlobalDeclaration> {
        self.erase_statement(node.span(context.ast), context);
        TraverseAction::Skip
    }

    fn enter_ts_parameter_property(
        &mut self,
        node: TsParameterProperty,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsParameterProperty> {
        let span = node.span(context.ast);
        let decorators = node.decorators(context.ast);
        let modifiers_start = modifiers_start(span.start, decorators, context.ast);
        let parameter_start = node.parameter(context.ast).span(context.ast).start;
        let diagnostic_span = self
            .tokens
            .first_in_range(modifiers_start, parameter_start)
            .map_or(span, |modifier| modifier.span);

        self.report_erasable_syntax_only(diagnostic_span);
        context
            .state
            .blank(Span::new(modifiers_start, parameter_start));
        TraverseAction::Proceed
    }

    fn enter_ts_export_assignment(
        &mut self,
        node: TsExportAssignment,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsExportAssignment> {
        self.report_erasable_syntax_only(node.span(context.ast));
        TraverseAction::Proceed
    }

    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsImportEqualsDeclaration> {
        let span = node.span(context.ast);
        if node.import_kind(context.ast) == ImportOrExportKind::Type {
            self.erase_statement(span, context);
            TraverseAction::Skip
        } else {
            self.report_erasable_syntax_only(span);
            TraverseAction::Proceed
        }
    }

    fn enter_import_declaration(
        &mut self,
        node: ImportDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<ImportDeclaration> {
        let container = node.span(context.ast);
        if node.import_kind(context.ast) == ImportOrExportKind::Type {
            self.erase_statement(container, context);
            return TraverseAction::Skip;
        }

        let specifiers = node.specifiers(context.ast);
        for position in 0..specifiers.len() {
            let Some(specifier) = specifiers.get_node(context.ast, position) else {
                continue;
            };
            let Some(specifier) = specifier.as_import_specifier(context.ast) else {
                continue;
            };
            if specifier.import_kind(context.ast) != ImportOrExportKind::Type {
                continue;
            }

            let item = specifier.span(context.ast);
            let next = specifiers
                .get_node(context.ast, position + 1)
                .map(|specifier| specifier.span(context.ast));
            self.erase_list_item(item, next, container, context);
        }
        TraverseAction::Proceed
    }

    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<ExportNamedDeclaration> {
        let container = node.span(context.ast);
        if node.export_kind(context.ast) == ImportOrExportKind::Type
            || node
                .declaration(context.ast)
                .is_some_and(|declaration| is_whole_type_declaration(declaration, context.ast))
        {
            self.erase_statement(container, context);
            return TraverseAction::Skip;
        }

        let specifiers = node.specifiers(context.ast);
        for position in 0..specifiers.len() {
            let Some(specifier) = specifiers.get_node(context.ast, position) else {
                continue;
            };
            if specifier.export_kind(context.ast) != ImportOrExportKind::Type {
                continue;
            }

            let item = specifier.span(context.ast);
            let next = specifiers
                .get_node(context.ast, position + 1)
                .map(|specifier| specifier.span(context.ast));
            self.erase_list_item(item, next, container, context);
        }
        TraverseAction::Proceed
    }

    fn enter_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<ExportAllDeclaration> {
        if node.export_kind(context.ast) == ImportOrExportKind::Type {
            self.erase_statement(node.span(context.ast), context);
            TraverseAction::Skip
        } else {
            TraverseAction::Proceed
        }
    }

    fn enter_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<ExportDefaultDeclaration> {
        let declaration = node.declaration(context.ast);
        if declaration.is_ts_function(context.ast)
            || declaration.is_ts_interface_declaration(context.ast)
        {
            self.erase_statement(node.span(context.ast), context);
            TraverseAction::Skip
        } else {
            TraverseAction::Proceed
        }
    }

    fn enter_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsTypeParameterDeclaration> {
        context.state.blank(node.span(context.ast));
        TraverseAction::Skip
    }

    fn enter_ts_type_parameter_instantiation(
        &mut self,
        node: TsTypeParameterInstantiation,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsTypeParameterInstantiation> {
        context.state.blank(node.span(context.ast));
        TraverseAction::Skip
    }

    fn enter_ts_type_annotation(
        &mut self,
        node: TsTypeAnnotation,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsTypeAnnotation> {
        context.state.blank(node.span(context.ast));
        TraverseAction::Skip
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsTypeAliasDeclaration> {
        self.erase_statement(node.span(context.ast), context);
        TraverseAction::Skip
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsInterfaceDeclaration> {
        self.erase_statement(node.span(context.ast), context);
        TraverseAction::Skip
    }

    fn enter_ts_namespace_export_declaration(
        &mut self,
        node: TsNamespaceExportDeclaration,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsNamespaceExportDeclaration> {
        self.erase_statement(node.span(context.ast), context);
        TraverseAction::Skip
    }

    fn enter_ts_as_expression(
        &mut self,
        node: TsAsExpression,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsAsExpression> {
        self.erase_postfix_type_assertion(
            node.expression(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
    }

    fn enter_ts_satisfies_expression(
        &mut self,
        node: TsSatisfiesExpression,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsSatisfiesExpression> {
        self.erase_postfix_type_assertion(
            node.expression(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
    }

    fn enter_ts_non_null_expression(
        &mut self,
        node: TsNonNullExpression,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsNonNullExpression> {
        self.erase_suffix_expression(
            node.expression(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
    }

    fn enter_ts_type_assertion(
        &mut self,
        node: TsTypeAssertion,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<TsTypeAssertion> {
        let wrapper_start = node.span(context.ast).start;
        let expression_start = node.expression(context.ast).span(context.ast).start;
        self.report_erasable_syntax_only(Span::new(wrapper_start, expression_start));
        context
            .state
            .blank(Span::new(wrapper_start, expression_start));
        TraverseAction::Proceed
    }

    fn enter_variable_declarator(
        &mut self,
        node: VariableDeclarator,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<VariableDeclarator> {
        if node.definite(context.ast) {
            let pattern = node.id(context.ast);
            let span = pattern.span(context.ast);
            let marker_end = pattern_type_annotation(pattern, context.ast)
                .map_or(span.end, |annotation| annotation.span(context.ast).start);
            if let Some(marker) =
                self.tokens
                    .find_backward(span.start, marker_end, TokenTag::LogicalNot)
            {
                context.state.blank(marker.span);
            }
        }
        TraverseAction::Proceed
    }

    fn enter_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<BindingIdentifier> {
        self.erase_optional_pattern(
            node.optional(context.ast),
            node.type_annotation(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
    }

    fn enter_assignment_pattern(
        &mut self,
        node: AssignmentPattern,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<AssignmentPattern> {
        self.erase_optional_pattern(
            node.optional(context.ast),
            node.type_annotation(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
    }

    fn enter_binding_rest_element(
        &mut self,
        node: BindingRestElement,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<BindingRestElement> {
        self.erase_optional_pattern(
            node.optional(context.ast),
            node.type_annotation(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
    }

    fn enter_array_pattern(
        &mut self,
        node: ArrayPattern,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<ArrayPattern> {
        self.erase_optional_pattern(
            node.optional(context.ast),
            node.type_annotation(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
    }

    fn enter_object_pattern(
        &mut self,
        node: ObjectPattern,
        context: &mut TraverseCtx<'_, 'ast, EditPlan>,
    ) -> TraverseAction<ObjectPattern> {
        self.erase_optional_pattern(
            node.optional(context.ast),
            node.type_annotation(context.ast),
            node.span(context.ast),
            context,
        );
        TraverseAction::Proceed
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

fn is_statement_list_item(node: NodeKind, parent: NodeKind) -> bool {
    if parent == NodeKind::ClassBody {
        return matches!(
            node,
            NodeKind::MethodDefinition
                | NodeKind::PropertyDefinition
                | NodeKind::StaticBlock
                | NodeKind::TsMethodDefinition
                | NodeKind::TsIndexSignature
        );
    }
    if !matches!(
        parent,
        NodeKind::Program
            | NodeKind::FunctionBody
            | NodeKind::BlockStatement
            | NodeKind::TsModuleBlock
            | NodeKind::StaticBlock
            | NodeKind::SwitchCase
    ) {
        return false;
    }

    matches!(
        node,
        NodeKind::Directive
            | NodeKind::BlockStatement
            | NodeKind::BreakStatement
            | NodeKind::ContinueStatement
            | NodeKind::DebuggerStatement
            | NodeKind::DoWhileStatement
            | NodeKind::EmptyStatement
            | NodeKind::ExpressionStatement
            | NodeKind::ForInStatement
            | NodeKind::ForOfStatement
            | NodeKind::ForStatement
            | NodeKind::IfStatement
            | NodeKind::LabeledStatement
            | NodeKind::ReturnStatement
            | NodeKind::SwitchStatement
            | NodeKind::ThrowStatement
            | NodeKind::TryStatement
            | NodeKind::WhileStatement
            | NodeKind::WithStatement
            | NodeKind::VariableDeclaration
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::TsFunction
            | NodeKind::TsTypeAliasDeclaration
            | NodeKind::TsInterfaceDeclaration
            | NodeKind::TsEnumDeclaration
            | NodeKind::TsModuleDeclaration
            | NodeKind::TsGlobalDeclaration
            | NodeKind::TsImportEqualsDeclaration
            | NodeKind::ImportDeclaration
            | NodeKind::ExportNamedDeclaration
            | NodeKind::ExportDefaultDeclaration
            | NodeKind::ExportAllDeclaration
            | NodeKind::TsExportAssignment
            | NodeKind::TsNamespaceExportDeclaration
    )
}

fn is_required_statement_parent(parent: NodeKind) -> bool {
    matches!(
        parent,
        NodeKind::IfStatement
            | NodeKind::LabeledStatement
            | NodeKind::WhileStatement
            | NodeKind::DoWhileStatement
            | NodeKind::ForStatement
            | NodeKind::ForInStatement
            | NodeKind::ForOfStatement
            | NodeKind::WithStatement
    )
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
    if let Some(import_equals) = declaration.as_ts_import_equals_declaration(ast) {
        return import_equals.import_kind(ast) == ImportOrExportKind::Type;
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
