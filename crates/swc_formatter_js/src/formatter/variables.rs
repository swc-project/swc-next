//! Variable declarations and their real keyword, binding and initializer owners.

use std::ops::ControlFlow;

use swc_next_ecma_ast::{
    Ast, BindingPatternData, Expr, ExprData, GetSpan, Span, VariableDeclaration,
    VariableDeclarator, VariableKind,
};

use super::Formatter;
use super::bindings::{
    BindingGroup, BindingInitializerOwner, InitializerLayout, InitializerLeft, PreparedInitializer,
};
use super::comments::{CommentBinding, ExpressionTail};
use super::diagnostics::FlatSeparator;
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::expressions::assignment::RhsLayout;
use super::expressions::{
    ExpressionContext, ExpressionOutput, ExpressionPosition, ExpressionRightEdge,
    ForInitializerGrammar, TailBoundary,
};
use super::statements::DeclarationKeyword;
use super::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum VariableContext {
    Statement(TailBoundary),
    ForInitializer,
    ForIn,
    Iteration,
}

pub(super) struct RawFenceRequest {
    pub(super) span: Span,
    pub(super) terminal: super::types::RawTypeTerminal,
}

pub(super) struct VariableDeclarationOutput<'ast> {
    pub(super) end: u32,
    pub(super) tail: Option<ExpressionTail<'ast>>,
    pub(super) fence_request: Option<RawFenceRequest>,
}

pub(super) struct VariableDeclaratorOutput<'ast> {
    tail: Option<ExpressionTail<'ast>>,
    fence_request: Option<RawFenceRequest>,
    expression_edge: Option<ExpressionRightEdge>,
}

impl<'ast> VariableDeclaratorOutput<'ast> {
    pub(super) fn formatted(tail: Option<ExpressionTail<'ast>>) -> Self {
        Self {
            tail,
            fence_request: None,
            expression_edge: None,
        }
    }

    pub(super) fn initializer(output: ExpressionOutput<'ast>) -> Self {
        Self {
            tail: output.pending,
            fence_request: None,
            expression_edge: Some(output.right_edge),
        }
    }
}

fn copied_declarator_terminal(
    declarator: VariableDeclarator,
    ast: &Ast<'_>,
) -> super::types::RawTypeTerminal {
    if let Some(initializer) = declarator.init(ast) {
        return super::expressions::raw_tail::raw_expression_terminal(initializer, ast);
    }
    declarator
        .type_annotation(ast)
        .map_or(super::types::RawTypeTerminal::Closed, |annotation| {
            super::types::raw_type_terminal(annotation.type_annotation(ast), ast)
        })
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn variable_initializer_context(
        &self,
        expression: Expr,
        context: VariableContext,
    ) -> ExpressionContext<'ast> {
        let context = match context {
            VariableContext::ForInitializer => {
                self.for_initializer_context(expression, ForInitializerGrammar::Assignment)
            }
            VariableContext::ForIn => ExpressionContext::at(ExpressionPosition::ForInInitializer),
            VariableContext::Statement(boundary) => {
                ExpressionContext::variable_initializer().with_tail_boundary(boundary)
            }
            VariableContext::Iteration => ExpressionContext::variable_initializer(),
        };
        context.with_variable_initializer_owner()
    }

    pub(super) fn format_variable_declaration(
        &mut self,
        declaration: VariableDeclaration,
        keyword: Option<DeclarationKeyword>,
        tail_boundary: TailBoundary,
        source_candidate: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<VariableDeclarationOutput<'ast>, FormatError> {
        self.format_variable_declaration_parts(
            declaration,
            VariableContext::Statement(tail_boundary),
            keyword,
            source_candidate,
            writer,
        )
    }

    pub(super) fn format_variable_declaration_with_context(
        &mut self,
        declaration: VariableDeclaration,
        context: VariableContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
        self.format_variable_declaration_parts(declaration, context, None, false, writer)
            .map(|output| (output.end, output.tail))
    }

    fn format_variable_declaration_parts(
        &mut self,
        declaration: VariableDeclaration,
        context: VariableContext,
        keyword: Option<DeclarationKeyword>,
        source_candidate: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<VariableDeclarationOutput<'ast>, FormatError> {
        let ast = self.ast;
        if declaration.declare(ast) && keyword.is_none() {
            return Err(self.unsupported("ambient variable declaration", declaration));
        }
        let mut nodes = declaration.declarators(ast);
        let Some(first) = nodes.first().map(|id| ast.get_node_in_sub_range(id)) else {
            return Err(self.unsupported("empty variable declaration", declaration));
        };
        let remaining = nodes.split_off(1);
        let (end, has_value) = remaining.iter().fold(
            (first.span(ast).end, first.init(ast).is_some()),
            |(_, has_value), id| {
                let declarator = ast.get_node_in_sub_range(id);
                (
                    declarator.span(ast).end,
                    has_value || declarator.init(ast).is_some(),
                )
            },
        );
        let mut declarators = std::iter::once(first)
            .chain(remaining.iter().map(|id| ast.get_node_in_sub_range(id)))
            .peekable();
        let start = keyword.map_or(declaration.span(ast).start, |keyword| keyword.span.start);
        let kind = declaration.kind(ast);
        // The actual declarator resolves this proof using its prepared RHS.
        let definition_owner =
            (kind == VariableKind::Const && remaining.is_empty()).then_some(first);
        let trivia_start = if kind == VariableKind::AwaitUsing {
            let gap = Span::new(start + 5, first.span(ast).start);
            let comments = self.comments.in_range(gap);
            let using = self
                .find_token(gap, comments, "using")
                .ok_or_else(|| self.unsupported("await using keyword boundary", declaration))?;
            let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
                let semantics = self.comment_semantics(comment);
                if semantics.diagnostic || semantics.file_directive {
                    CommentBinding::SourceBound
                } else if semantics.suppression {
                    CommentBinding::BoundTo(first.span(ast))
                } else {
                    // Neither keyword trivia nor a binding head supplies a cast
                    // or optimizer expression target.
                    CommentBinding::Unbound
                }
            });
            if !self.diagnostic_range_is_sensitive(gap)
                && binding.allows_relocation(Some(first.span(ast)))
            {
                writer.text(kind.as_str());
                // The first declarator owns both keyword-adjacent packets.
                // Keep their actual source start, not the printed prefix width.
                gap.start
            } else {
                writer.text("await");
                self.format_source_token_gap(start + 5, using, true, writer)?;
                writer.text("using");
                using + 5
            }
        } else {
            writer.text(kind.as_str());
            start + kind.as_str().len() as u32
        };
        let trivia_start = keyword.map_or(trivia_start, DeclarationKeyword::name_prefix_start);
        let comments = self
            .comments
            .in_range(Span::new(trivia_start, first.span(ast).start));
        let first_start = comments
            .first()
            .map_or(first.span(ast).start, |comment| comment.span.start);
        let indent_first = !remaining.is_empty() || !comments.is_empty();
        self.line_constraint(trivia_start, first_start).write(
            writer,
            FlatSeparator::Space,
            Writer::space,
        );
        writer.group(false, |writer| {
            let mut previous_end = trivia_start;
            let mut transition = None;
            let mut separator = Separator::Line;
            let mut following_mode = None;
            let mut first_item = true;
            while let Some(declarator) = declarators.next() {
                let span = declarator.span(ast);
                let following = declarators.peek().map_or(FollowingItem::End, |node| {
                    FollowingItem::Node(node.span(ast))
                });
                let context = match (context, following) {
                    (VariableContext::Statement(_), FollowingItem::Node(_)) => {
                        VariableContext::Statement(TailBoundary::Adjacent)
                    }
                    _ => context,
                };
                let mut write = |writer: &mut Writer<'ast>| {
                    if !first_item {
                        let next = self
                            .comments
                            .in_range(Span::new(previous_end, span.start))
                            .first()
                            .map_or(span.start, |comment| comment.span.start);
                        if matches!(context, VariableContext::Statement(_))
                            && has_value
                            && !matches!(separator, Separator::Source(_))
                        {
                            self.source_separator_constraint(
                                previous_end,
                                next,
                                transition.as_ref(),
                            )
                            .write(
                                writer,
                                FlatSeparator::Space,
                                Writer::hard_line,
                            );
                        } else {
                            ArgumentLayout::Lines.separator(
                                separator,
                                self.source_separator_constraint(
                                    previous_end,
                                    next,
                                    transition.as_ref(),
                                ),
                                writer,
                            );
                        }
                    }
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(previous_end, end),
                        false,
                        transition,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if keyword.is_some_and(|keyword| keyword.suppresses_variable(declarator)) {
                        item.set_leading_mode(ContentMode::Verbatim);
                    }
                    let output = self.format_variable_declarator(
                        declarator,
                        item.mode,
                        context,
                        definition_owner,
                        source_candidate && matches!(following, FollowingItem::End),
                        writer,
                    )?;
                    if matches!(following, FollowingItem::End) {
                        Ok(ControlFlow::Break(output))
                    } else {
                        self.finish_delimited_item(item, output.tail, writer)
                            .map(ControlFlow::Continue)
                    }
                };
                let boundary = if !first_item || indent_first {
                    writer.indent(write)?
                } else {
                    write(writer)?
                };
                let boundary = match boundary {
                    ControlFlow::Break(output) => {
                        let fence_request = output.fence_request.or_else(|| {
                            if !source_candidate {
                                return None;
                            }
                            output.expression_edge.and_then(|edge| {
                                let ExpressionRightEdge::SourceFenceCandidate(raw) = edge else {
                                    return None;
                                };
                                let raw_span = raw.span(ast);
                                if span.start > raw_span.start || raw_span.end > span.end {
                                    return None;
                                }
                                let terminal = raw.terminal(ast);
                                (terminal != super::types::RawTypeTerminal::Closed).then_some(
                                    RawFenceRequest {
                                        span: raw_span,
                                        terminal,
                                    },
                                )
                            })
                        });
                        return Ok(VariableDeclarationOutput {
                            end,
                            tail: output.tail,
                            fence_request,
                        });
                    }
                    ControlFlow::Continue(boundary) => boundary,
                };
                previous_end = boundary.end;
                separator = boundary.separator;
                transition = boundary.transition;
                following_mode = Some(boundary.following_mode);
                first_item = false;
            }
            Ok(VariableDeclarationOutput {
                end,
                tail: None,
                fence_request: None,
            })
        })
    }

    fn format_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        mode: ContentMode,
        context: VariableContext,
        definition_owner: Option<VariableDeclarator>,
        source_candidate: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<VariableDeclaratorOutput<'ast>, FormatError> {
        let ast = self.ast;
        if mode == ContentMode::Verbatim {
            let span = declarator.span(ast);
            self.format_verbatim(span, writer)?;
            let fence_request = if source_candidate {
                let terminal = copied_declarator_terminal(declarator, ast);
                (terminal != super::types::RawTypeTerminal::Closed)
                    .then_some(RawFenceRequest { span, terminal })
            } else {
                None
            };
            return Ok(VariableDeclaratorOutput {
                tail: None,
                fence_request,
                expression_edge: None,
            });
        }
        let annotation = declarator.type_annotation(ast);
        if annotation.is_some() || declarator.definite(ast) {
            return self.format_typed_variable_declarator(
                declarator,
                annotation,
                context,
                definition_owner,
                writer,
            );
        }
        let left = declarator.id(ast);
        let Some(right) = declarator.init(ast) else {
            return self
                .format_declaration_binding(left, ContentMode::Formatted, writer)
                .map(VariableDeclaratorOutput::formatted);
        };
        let left_end = left.span(ast).end;
        let right_start = right.span(ast).start;
        let gap = Span::new(left_end, right_start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=") else {
            return Err(self.unsupported("variable initializer boundary", declarator));
        };
        let content = self.expression_content(right);
        let decorated_function = if definition_owner == Some(declarator) {
            self.decorated_function_call_content(&content)
        } else {
            None
        };
        let initializer_context = self
            .variable_initializer_context(right, context)
            .with_decorated_function_call(decorated_function);
        let rhs = self.assignment_rhs(&content);
        let mut boundary = self.assignment_boundary(
            left_end,
            &rhs,
            Span::new(operator, operator + 1),
            right.span(self.ast).end,
            initializer_context,
        )?;
        let mut left_mode = ContentMode::Formatted;
        if self.has_suppression {
            let before =
                &comments[..comments.partition_point(|comment| comment.span.end <= operator)];
            for position in self.suppression_positions(before, operator) {
                match position {
                    SuppressionPosition::Inline => left_mode = ContentMode::Verbatim,
                    SuppressionPosition::OwnLine | SuppressionPosition::EndOfLine => {
                        boundary.mode = ContentMode::Verbatim;
                        if let Some(comment) = boundary.before.last() {
                            boundary.before_operator =
                                self.source_line_constraint(comment.span.end, operator);
                        }
                    }
                }
            }
        }
        let mut rhs_layout = self.ordinary_rhs_layout(&rhs, initializer_context, &boundary);
        let fixed_left_policy = matches!(rhs_layout, RhsLayout::Fluid)
            && boundary.mode == ContentMode::Formatted
            && matches!(
                content.data,
                ExprData::TemplateLiteral(_)
                    | ExprData::TaggedTemplateExpression(_)
                    | ExprData::Class(_)
            );
        if fixed_left_policy
            && left_mode == ContentMode::Formatted
            && matches!(
                ast.binding_pattern_data(left),
                BindingPatternData::BindingIdentifier(_)
            )
            && !matches!(content.data, ExprData::Class(class) if !class.decorators(ast).is_empty())
        {
            rhs_layout = RhsLayout::NeverBreakAfterOperator;
        }
        let (group, rhs_layout) = if left_mode == ContentMode::Formatted {
            self.binding_initializer_layout(
                left,
                BindingInitializerOwner::Variable { annotation: None },
                content.data,
                &boundary,
                rhs_layout,
            )
        } else {
            (BindingGroup::Auto, rhs_layout)
        };
        self.format_initializer_with_raw_end(
            PreparedInitializer {
                left: InitializerLeft::Binding {
                    pattern: left,
                    group,
                },
                right,
                layout: InitializerLayout::Formatted {
                    boundary,
                    rhs_layout,
                },
                left_mode,
                context: initializer_context,
            },
            writer,
        )
        .map(VariableDeclaratorOutput::initializer)
    }
}
