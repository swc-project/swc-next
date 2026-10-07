//! Variable declarations and their real keyword, binding and initializer owners.

use std::ops::ControlFlow;

use swc_next_ecma_ast::{
    BindingPatternData, Expr, ExprData, GetSpan, Span, VariableDeclaration, VariableDeclarator,
    VariableKind,
};

use super::Formatter;
use super::bindings::{
    BindingGroup, BindingInitializerOwner, InitializerLayout, InitializerLeft, PreparedInitializer,
};
use super::comments::{CommentBinding, ExpressionTail};
use super::diagnostics::FlatSeparator;
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::expressions::assignment::RhsLayout;
use super::expressions::{ExpressionContext, ExpressionPosition, ForInitializerGrammar};
use super::statements::DeclarationKeyword;
use super::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum VariableContext {
    Statement,
    ForInitializer,
    ForIn,
    Iteration,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn variable_initializer_context(
        &self,
        expression: Expr,
        context: VariableContext,
    ) -> ExpressionContext {
        let context = match context {
            VariableContext::ForInitializer => {
                self.for_initializer_context(expression, ForInitializerGrammar::Assignment)
            }
            VariableContext::ForIn => ExpressionContext::at(ExpressionPosition::ForInInitializer),
            VariableContext::Statement => ExpressionContext::variable_initializer(),
            VariableContext::Iteration => ExpressionContext::variable_initializer(),
        };
        context.with_variable_initializer_owner()
    }

    pub(super) fn format_variable_declaration(
        &mut self,
        declaration: VariableDeclaration,
        keyword: Option<DeclarationKeyword>,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
        self.format_variable_declaration_parts(
            declaration,
            VariableContext::Statement,
            keyword,
            writer,
        )
    }

    pub(super) fn format_variable_declaration_with_context(
        &mut self,
        declaration: VariableDeclaration,
        context: VariableContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
        self.format_variable_declaration_parts(declaration, context, None, writer)
    }

    fn format_variable_declaration_parts(
        &mut self,
        declaration: VariableDeclaration,
        context: VariableContext,
        keyword: Option<DeclarationKeyword>,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
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
            let mut separator = Separator::Line;
            let mut following_mode = None;
            let mut first_item = true;
            while let Some(declarator) = declarators.next() {
                let span = declarator.span(ast);
                let following = declarators.peek().map_or(FollowingItem::End, |node| {
                    FollowingItem::Node(node.span(ast))
                });
                let mut write = |writer: &mut Writer<'ast>| {
                    if !first_item {
                        let next = self
                            .comments
                            .in_range(Span::new(previous_end, span.start))
                            .first()
                            .map_or(span.start, |comment| comment.span.start);
                        if matches!(context, VariableContext::Statement)
                            && has_value
                            && !matches!(separator, Separator::Source(_))
                        {
                            self.line_constraint(previous_end, next).write(
                                writer,
                                FlatSeparator::Space,
                                Writer::hard_line,
                            );
                        } else {
                            ArgumentLayout::Lines.separator(
                                separator,
                                self.line_constraint(previous_end, next),
                                writer,
                            );
                        }
                    }
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(previous_end, end),
                        false,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if keyword.is_some_and(|keyword| keyword.suppresses_variable(declarator)) {
                        item.set_leading_mode(ContentMode::Verbatim);
                    }
                    let tail =
                        self.format_variable_declarator(declarator, item.mode, context, writer)?;
                    if matches!(following, FollowingItem::End) {
                        Ok(ControlFlow::Break(tail))
                    } else {
                        self.finish_delimited_item(item, tail, writer)
                            .map(ControlFlow::Continue)
                    }
                };
                let boundary = if !first_item || indent_first {
                    writer.indent(write)?
                } else {
                    write(writer)?
                };
                let boundary = match boundary {
                    ControlFlow::Break(tail) => return Ok((end, tail)),
                    ControlFlow::Continue(boundary) => boundary,
                };
                previous_end = boundary.end;
                separator = boundary.separator;
                following_mode = Some(boundary.following_mode);
                first_item = false;
            }
            Ok((end, None))
        })
    }

    fn format_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        mode: ContentMode,
        context: VariableContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if mode == ContentMode::Verbatim {
            self.format_verbatim(declarator.span(ast), writer)?;
            return Ok(None);
        }
        if declarator.definite(ast) {
            return Err(self.unsupported("definite variable declarator", declarator));
        }
        if let Some(annotation) = declarator.type_annotation(ast) {
            return self.format_typed_variable_declarator(declarator, annotation, context, writer);
        }
        let left = declarator.id(ast);
        let Some(right) = declarator.init(ast) else {
            return self.format_declaration_binding(left, ContentMode::Formatted, writer);
        };
        let left_end = left.span(ast).end;
        let right_start = right.span(ast).start;
        let gap = Span::new(left_end, right_start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=") else {
            return Err(self.unsupported("variable initializer boundary", declarator));
        };
        let content = self.expression_content(right);
        let mut boundary = self.assignment_boundary(
            left_end,
            &content,
            Span::new(operator, operator + 1),
            right.span(self.ast).end,
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
        let initializer_context = self.variable_initializer_context(right, context);
        let mut rhs_layout = self.ordinary_rhs_layout(content.data, initializer_context, &boundary);
        if matches!(rhs_layout, RhsLayout::Fluid)
            && boundary.mode == ContentMode::Formatted
            && left_mode == ContentMode::Formatted
            && matches!(
                ast.binding_pattern_data(left),
                BindingPatternData::BindingIdentifier(_)
            )
            && matches!(content.data, ExprData::Class(class) if class.decorators(ast).is_empty())
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
        self.format_initializer(
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
    }
}
