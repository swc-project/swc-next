//! Shared import source/options slots retain their real expressions and delimiters.

use swc_next_ecma_ast::{
    Expr, ExprData, GetSpan, ImportExpression, ImportPhase, ObjectExpression, Span,
};

use super::arguments::{ArgumentLayout, FollowingItem, ItemBoundary};
use super::arrows::ArrowLayout;
use super::objects::RecordedObject;
use super::parentheses::ExpressionContent;
use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError,
    document::{FlatFragment, Fragment, Writer},
};

pub(in crate::formatter) struct ImportArguments {
    pub(in crate::formatter) source: Expr,
    pub(in crate::formatter) options: Option<Expr>,
    pub(in crate::formatter) content: Span,
    pub(in crate::formatter) opening: u32,
    pub(in crate::formatter) last_mode: ContentMode,
}

struct ImportSlot {
    expression: Expr,
    span: Span,
    content: ExpressionContent,
    content_start: u32,
}

enum RecordedImportOptions<'a> {
    Ordinary(Fragment<'a>),
    Selected(RecordedObject<'a>),
}

fn flat_import_fragment(fragment: Fragment<'_>) -> Option<FlatFragment<'_>> {
    if fragment.preferred_breaks() {
        return None;
    }
    fragment.try_flat()
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_import_expression(
        &mut self,
        node: ImportExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let keyword_end = span.start + 6;
        let source = node.source(ast);
        let source_span = source.span(ast);
        let options = node.options(ast);
        let opening_gap = Span::new(keyword_end, source_span.start);
        let comments = self.comments.in_range(opening_gap);
        let opening = self
            .find_token(opening_gap, comments, "(")
            .ok_or_else(|| self.unsupported("import opening boundary", node))?;
        let prefix_count = comments.partition_point(|comment| comment.span.end <= opening);
        let prefix = &comments[..prefix_count];
        let prefix_span = Span::new(keyword_end, opening);
        let target = self.has_suppression.then_some(source_span);
        let binding = self.comment_binding(prefix, Span::new(keyword_end, opening + 1), target);
        let move_prefix = binding.allows_relocation(target);
        let modifier = if node.sync(ast) {
            Some("sync")
        } else {
            match node.phase(ast) {
                Some(ImportPhase::Source) => Some("source"),
                Some(ImportPhase::Defer) => Some("defer"),
                None => None,
            }
        };
        let modifier_tokens = if !move_prefix {
            modifier
                .map(|name| {
                    let dot = self
                        .find_token(prefix_span, prefix, ".")
                        .ok_or_else(|| self.unsupported("import phase dot boundary", node))?;
                    let start = self
                        .find_token(Span::new(dot + 1, opening), prefix, name)
                        .ok_or_else(|| self.unsupported("import phase keyword boundary", node))?;
                    Ok::<_, FormatError>((dot, Span::new(start, start + name.len() as u32)))
                })
                .transpose()?
        } else {
            None
        };
        writer.text("import");
        if let Some(name) = modifier {
            if let Some((dot, token)) = modifier_tokens {
                self.format_source_token_gap(keyword_end, dot, false, writer)?;
                writer.text(".");
                self.format_source_token_gap(dot + 1, token.start, false, writer)?;
                writer.text(name);
                self.format_source_token_gap(token.end, opening, false, writer)?;
            } else {
                writer.text(".");
                writer.text(name);
            }
        } else if !move_prefix {
            self.format_source_token_gap(keyword_end, opening, false, writer)?;
        }
        let content = Span::new(
            if move_prefix && !prefix.is_empty() {
                keyword_end
            } else {
                opening + 1
            },
            span.end - 1,
        );
        self.format_import_arguments(
            ImportArguments {
                source,
                options,
                content,
                opening,
                last_mode: ContentMode::Formatted,
            },
            writer,
        )?;
        Ok(None)
    }

    pub(in crate::formatter) fn format_import_arguments(
        &mut self,
        arguments: ImportArguments,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ImportArguments {
            source,
            options,
            content,
            opening,
            last_mode,
        } = arguments;
        let source = self.prepare_import_slot(source);
        let options = options.map(|expression| self.prepare_import_slot(expression));
        let source_span = source.span;
        let expand = self.next_line_is_empty(opening + 1, source_span.start)
            || options
                .as_ref()
                .is_some_and(|option| self.next_line_is_empty(source_span.end, option.span.start));
        let selected = options.as_ref().and_then(|option| {
            let ExprData::ObjectExpression(object) = option.content.data else {
                return None;
            };
            if option.expression != option.content.expression
                || matches!(source.content.data, ExprData::ObjectExpression(_))
                || expand
                || last_mode == ContentMode::Verbatim
                || self.diagnostic_range_is_sensitive(content)
            {
                return None;
            }
            if object.properties(self.ast).is_empty()
                && self
                    .comments
                    .in_range(Span::new(option.span.start + 1, option.span.end - 1))
                    .is_empty()
            {
                return None;
            }
            Some(object)
        });
        let options = match (selected, options) {
            (Some(object), Some(option)) => {
                return self
                    .format_selected_import_arguments(source, option, object, content, writer);
            }
            (_, options) => options,
        };
        let hugged = options.is_none()
            && matches!(source.content.data, ExprData::StringLiteral(_))
            && self.comments.in_range(content).is_empty()
            && !self.diagnostic_range_is_sensitive(content);
        writer.group(!hugged && expand, |writer| {
            writer.text("(");
            let write_items = |writer: &mut Writer<'ast>| {
                if !hugged {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut expressions = [Some(source), options].into_iter().flatten().peekable();
                let mut previous: Option<ItemBoundary> = None;
                while let Some(expression) = expressions.next() {
                    let item_span = expression.span;
                    let end = previous.map_or(content.start, |boundary| boundary.end);
                    if let Some(boundary) = previous {
                        let next = self
                            .comments
                            .between(Span::new(end, item_span.start))?
                            .first()
                            .map_or(item_span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            boundary.separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let following = expressions
                        .peek()
                        .map_or(FollowingItem::End, |next| FollowingItem::Node(next.span));
                    let mut item = self.prepare_call_argument_item(
                        item_span,
                        following,
                        Span::new(end, content.end),
                        false,
                        writer,
                    )?;
                    if let Some(boundary) = previous {
                        item.set_leading_mode(boundary.following_mode);
                    }
                    if expressions.peek().is_none() && last_mode == ContentMode::Verbatim {
                        item.set_leading_mode(last_mode);
                    }
                    let tail = self.format_import_slot(expression, item.mode, writer)?;
                    previous = Some(self.finish_delimited_item(item, tail, writer)?);
                }
                let end = previous.map_or(content.start, |boundary| boundary.end);
                self.format_argument_dangling(Span::new(end, content.end), writer)
            };
            let end = if hugged {
                write_items(writer)?
            } else {
                writer.indent(write_items)?
            };
            if !hugged {
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
            }
            writer.text(")");
            Ok(())
        })
    }

    fn prepare_import_slot(&self, expression: Expr) -> ImportSlot {
        let content = self.expression_content(expression);
        let content_start = content.expression.span(self.ast).start;
        ImportSlot {
            expression,
            span: expression.span(self.ast),
            content,
            content_start,
        }
    }

    fn format_import_slot(
        &mut self,
        slot: ImportSlot,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_prepared_expression_with_leading_comments(
            slot.expression,
            slot.content,
            NodeHead {
                comments: &[],
                content_start: slot.content_start,
                layout: CommentRunLayout::Source,
                gap_end: None,
            },
            ExpressionContext::at(ExpressionPosition::Nested)
                .with_content_mode(mode)
                .with_arrow_layout(ArrowLayout::Argument),
            writer,
        )
    }

    fn format_selected_import_arguments(
        &mut self,
        source: ImportSlot,
        options: ImportSlot,
        object: ObjectExpression,
        content: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (prefix, item) = writer.record_with_value(|writer| {
            self.opening_line_constraint(content).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            let item = self.prepare_call_argument_item(
                source.span,
                FollowingItem::Node(options.span),
                content,
                false,
                writer,
            )?;
            let tail = self.format_import_slot(source, item.mode, writer)?;
            let boundary = self.finish_delimited_item(item, tail, writer)?;
            let next = self
                .comments
                .between(Span::new(boundary.end, options.span.start))?
                .first()
                .map_or(options.span.start, |comment| comment.span.start);
            ArgumentLayout::Lines.separator(
                boundary.separator,
                self.line_constraint(boundary.end, next),
                writer,
            );
            let mut item = self.prepare_call_argument_item(
                options.span,
                FollowingItem::End,
                Span::new(boundary.end, content.end),
                false,
                writer,
            )?;
            item.set_leading_mode(boundary.following_mode);
            Ok::<_, FormatError>(item)
        })?;
        let (value, tail) = if item.mode == ContentMode::Formatted
            && !self.argument_has_boundary_comments(&item)
        {
            (
                RecordedImportOptions::Selected(self.record_object_argument(
                    options.expression,
                    object,
                    writer,
                )?),
                None,
            )
        } else {
            let (value, tail) = writer
                .record_with_value(|writer| self.format_import_slot(options, item.mode, writer))?;
            (RecordedImportOptions::Ordinary(value), tail)
        };
        let (suffix, ()) = writer.record_with_value(|writer| {
            let boundary =
                writer.indent(|writer| self.finish_delimited_item(item, tail, writer))?;
            let end =
                self.format_argument_dangling(Span::new(boundary.end, content.end), writer)?;
            self.line_constraint(end, content.end).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            Ok::<_, FormatError>(())
        })?;
        let normal = match value {
            RecordedImportOptions::Ordinary(value) => value,
            RecordedImportOptions::Selected(object) => object.normal,
        };
        let fallback = writer.indented_delimited_fragment("(", prefix, normal, suffix, ")");
        if let RecordedImportOptions::Selected(object) = value
            && let Some(prefix) = flat_import_fragment(prefix)
            && let Some(suffix) = flat_import_fragment(suffix)
        {
            let grouped = writer.flat_delimited_fragment("(", prefix, object.normal, suffix, ")");
            if let Some(forced) = object.forced {
                let expanded = writer.flat_delimited_fragment("(", prefix, forced, suffix, ")");
                writer.write_alternatives(fallback, &[grouped, expanded], fallback);
            } else {
                writer.write_alternatives(fallback, &[grouped], fallback);
            }
        } else {
            writer.write_fragment(fallback);
        }
        Ok(())
    }
}
