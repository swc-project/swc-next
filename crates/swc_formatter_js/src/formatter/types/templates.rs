//! TypeScript template slots retain their actual prepared type owners.

use swc_next_ecma_ast::{GetSpan, Span, TsTemplateLiteralType, TsType};

use super::{PreparedType, TypeContext, TypeLayout};
use crate::formatter::Formatter;
use crate::formatter::suppression::ContentMode;
use crate::formatter::templates::{TemplateSlot, TemplateSlotLayout};
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_template_literal_type_body(
        &mut self,
        template: TsTemplateLiteralType,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        self.format_template_shell(
            template.span(ast),
            template.quasis(ast),
            template.types(ast),
            |formatter, annotation, slot, writer| {
                formatter.format_template_type_slot_body(annotation, slot, writer)
            },
            writer,
        )
    }

    fn format_template_type_slot_body(
        &mut self,
        annotation: TsType,
        slot: TemplateSlot,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let span = slot.content;
        let leading = self
            .comments
            .in_range(Span::new(slot.interior.start, span.start));
        let trailing = self
            .comments
            .in_range(Span::new(span.end, slot.interior.end));
        let leading_mode = self.content_mode(leading);
        let trailing_mode = self.content_mode(trailing);
        let mut node: PreparedType = self.prepare_type(annotation, trailing_mode);
        if leading_mode == ContentMode::Verbatim {
            self.select_type_head_comments(&mut node, leading, span.start);
        }
        let opening = if leading_mode == ContentMode::Verbatim {
            let start = leading
                .first()
                .map_or(span.start, |comment| comment.span.start);
            self.source_line_constraint(slot.interior.start, start)
        } else {
            self.line_constraint(slot.interior.start, span.start)
        };
        let after = self.prepare_comment_boundary(trailing, span.end, slot.interior.end);
        let layout = if node.source_owner.is_some() {
            TemplateSlotLayout::Boundaries(ContentMode::Verbatim)
        } else if !leading.is_empty() || !trailing.is_empty() {
            TemplateSlotLayout::Boundaries(ContentMode::Formatted)
        } else {
            TemplateSlotLayout::Child
        };
        let head_layout = self.template_slot_head_layout(leading, span.start);
        self.format_template_slot(
            slot,
            opening,
            after.gap(),
            layout,
            writer,
            |formatter, layout, writer| {
                formatter.comments.claim(leading)?;
                formatter.format_comment_run(leading, head_layout, writer)?;
                let layout = match layout {
                    TemplateSlotLayout::Child => TypeLayout::Root,
                    TemplateSlotLayout::Boundaries(_) => TypeLayout::CallerIndented,
                };
                let output = if trailing.is_empty() {
                    let layout = formatter.continuation_type_layout(span, layout);
                    formatter.format_type_body_before_continuation(
                        span,
                        writer,
                        |formatter, writer| {
                            formatter.format_prepared_type_body(
                                node,
                                TypeContext::Root,
                                layout,
                                writer,
                            )
                        },
                    )?
                } else {
                    formatter.format_prepared_type_body(node, TypeContext::Root, layout, writer)?
                };
                formatter.format_expression_tail(output.pending, writer)?;
                formatter.format_comment_boundary(&after, writer)
            },
        )
    }
}
