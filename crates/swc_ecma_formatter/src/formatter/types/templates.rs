//! TypeScript template slots retain their actual prepared type owners.

use swc_next_ecma_ast::{GetSpan, Span, TsTemplateLiteralType, TsType};

use super::{PreparedType, TypeContext, TypeLayout};
use crate::formatter::Formatter;
use crate::formatter::comments::CommentRunLayout;
use crate::formatter::suppression::ContentMode;
use crate::formatter::templates::{TemplateSlot, TemplateSlotLayout};
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_template_literal_type(
        &mut self,
        template: TsTemplateLiteralType,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        self.format_template_shell(
            template.span(ast),
            template.quasis(ast),
            template.types(ast),
            Self::format_template_type_slot,
            writer,
        )
    }

    fn format_template_type_slot(
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
            for (_, position) in self.suppressed_comment_runs(leading, span.start) {
                self.select_type_head(&mut node, position);
            }
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
        self.format_template_slot(
            slot,
            opening,
            after.gap(),
            layout,
            writer,
            |formatter, layout, writer| {
                formatter.comments.claim(leading)?;
                formatter.format_comment_run(
                    leading,
                    CommentRunLayout::Leading(span.start),
                    writer,
                )?;
                let tail = formatter.format_prepared_type_with_layout(
                    node,
                    TypeContext::Root,
                    match layout {
                        TemplateSlotLayout::Child => TypeLayout::Root,
                        TemplateSlotLayout::Boundaries(_) => TypeLayout::CallerIndented,
                    },
                    writer,
                )?;
                formatter.format_expression_tail(tail, writer)?;
                formatter.format_comment_boundary(&after, writer)
            },
        )
    }
}
