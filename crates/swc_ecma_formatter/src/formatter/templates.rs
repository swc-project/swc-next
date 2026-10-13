//! Borrowed template text and physical quasi/slot delimiter geometry.

use swc_next_ecma_ast::{
    ExtraDataValue, GetSpan, Span, SubRange, TemplateElement, TemplateLiteral,
};

use super::Formatter;
use super::comments::CommentGap;
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::suppression::ContentMode;
use crate::{FormatError, document::Writer};

struct TemplateQuasi<'ast> {
    span: Span,
    raw: &'ast str,
    tail: bool,
}

impl TemplateQuasi<'_> {
    fn indentation(&self, previous: usize, tab_width: usize) -> usize {
        // Match Prettier's LF-only indentation boundaries. The parser already
        // normalizes CR/CRLF; LS/PS remain literal text.
        let Some((_, line)) = self.raw.rsplit_once('\n') else {
            return previous;
        };
        let mut width = 0;
        for byte in line.bytes() {
            match byte {
                b' ' => width += 1,
                b'\t' => width += tab_width - width % tab_width,
                _ => break,
            }
        }
        width
    }
}

/// The actual interpolation interior, independent of normalized raw byte length.
#[derive(Clone, Copy)]
pub(super) struct TemplateSlot {
    pub(super) content: Span,
    pub(super) interior: Span,
}

/// The child or its interpolation boundaries own finite-width layout.
#[derive(Clone, Copy)]
pub(super) enum TemplateSlotLayout {
    Child,
    Boundaries(ContentMode),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn template_quasi(&self, node: TemplateElement) -> TemplateQuasi<'ast> {
        TemplateQuasi {
            span: node.span(self.ast),
            raw: self.ast.get_utf8(node.raw(self.ast)),
            tail: node.tail(self.ast),
        }
    }

    fn template_token(&self, span: Span, token: &str) -> bool {
        self.ast
            .source()
            .get(span.start as usize..span.end as usize)
            == Some(token)
    }

    /// Complete child and comment scopes once, retaining their own finite-width
    /// layout when a source newline permits it.
    pub(super) fn format_template_slot(
        &mut self,
        slot: TemplateSlot,
        opening: LineConstraint,
        closing: CommentGap,
        layout: TemplateSlotLayout,
        writer: &mut Writer<'ast>,
        format: impl FnOnce(&mut Self, TemplateSlotLayout, &mut Writer<'ast>) -> Result<(), FormatError>,
    ) -> Result<(), FormatError> {
        if matches!(opening, LineConstraint::NewLine(_)) || closing.requires_line() {
            let layout = match layout {
                TemplateSlotLayout::Child => TemplateSlotLayout::Boundaries(ContentMode::Formatted),
                TemplateSlotLayout::Boundaries(_) => layout,
            };
            return Self::write_template_slot(layout, opening, closing, writer, |writer| {
                format(self, layout, writer)
            });
        }
        let source_newline = self
            .source(slot.interior)
            .contains(['\r', '\n', '\u{2028}', '\u{2029}']);
        if source_newline
            && matches!(
                layout,
                TemplateSlotLayout::Boundaries(ContentMode::Verbatim)
            )
        {
            return Self::write_template_slot(layout, opening, closing, writer, |writer| {
                format(self, layout, writer)
            });
        }
        let (content, ()) = writer.record_with_value(|writer| format(self, layout, writer))?;
        if !source_newline
            && !content.preferred_breaks()
            && let Some(content) = content.try_flat()
        {
            writer.write_flat_fragment(content);
            Ok(())
        } else {
            Self::write_template_slot(layout, opening, closing, writer, |writer| {
                writer.write_fragment(content);
                Ok(())
            })
        }
    }

    fn write_template_slot(
        layout: TemplateSlotLayout,
        opening: LineConstraint,
        closing: CommentGap,
        writer: &mut Writer<'ast>,
        format: impl FnOnce(&mut Writer<'ast>) -> Result<(), FormatError>,
    ) -> Result<(), FormatError> {
        writer.group(false, |writer| match layout {
            TemplateSlotLayout::Child => format(writer),
            TemplateSlotLayout::Boundaries(_) => {
                writer.indent(|writer| {
                    opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                    format(writer)
                })?;
                closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
                Ok(())
            }
        })
    }

    pub(super) fn format_zero_slot_template(
        &mut self,
        template: TemplateLiteral,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if !template.expressions(self.ast).is_empty() {
            return Err(self.unsupported("template literal expression slots", template));
        }
        self.format_template_without_slots(
            template.span(self.ast),
            template.quasis(self.ast),
            writer,
        )
    }

    fn format_template_without_slots(
        &self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Some(first) = quasis.first().filter(|_| quasis.len() == 1) else {
            return Err(FormatError::unsupported("template quasi count", None, span));
        };
        let quasi = self.template_quasi(self.ast.get_node_in_sub_range(first));
        if quasi.span.start > quasi.span.end
            || !quasi.tail
            || span.start >= span.end
            || quasi.span.start != span.start + 1
            || quasi.span.end != span.end - 1
            || !self.template_token(Span::new(span.start, quasi.span.start), "`")
            || !self.template_token(Span::new(quasi.span.end, span.end), "`")
        {
            return Err(FormatError::unsupported(
                "template quasi geometry",
                None,
                span,
            ));
        }
        writer.text("`");
        writer.literal_text(quasi.raw);
        writer.text("`");
        Ok(())
    }

    pub(super) fn format_template_shell<S>(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        slots: SubRange<S>,
        mut format_slot: impl FnMut(
            &mut Self,
            S,
            TemplateSlot,
            &mut Writer<'ast>,
        ) -> Result<(), FormatError>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError>
    where
        S: GetSpan + ExtraDataValue,
    {
        if slots.is_empty() {
            return self.format_template_without_slots(span, quasis, writer);
        }
        if quasis.len() != slots.len() + 1 {
            return Err(FormatError::unsupported("template quasi count", None, span));
        }
        let ast = self.ast;
        let mut quasis = quasis.iter().map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = quasis.next() else {
            return Err(FormatError::unsupported("template quasi count", None, span));
        };
        let mut current = self.template_quasi(first);
        if span.end as usize > ast.source().len()
            || span.start >= span.end
            || current.span.start != span.start + 1
            || !self.template_token(Span::new(span.start, current.span.start), "`")
        {
            return Err(FormatError::unsupported(
                "template opening geometry",
                None,
                span,
            ));
        }
        writer.text("`");
        writer.literal_text(current.raw);
        let tab_width = usize::from(self.options.indent_width.value().max(1));
        let mut indentation = 0;
        for slot in slots.iter().map(|id| ast.get_node_in_sub_range(id)) {
            let Some(next) = quasis.next() else {
                return Err(FormatError::unsupported("template quasi count", None, span));
            };
            let next = self.template_quasi(next);
            let slot_span = slot.span(ast);
            if next.span.end >= span.end
                || next.span.start > next.span.end
                || current.span.start > current.span.end
                || current.tail
                || current.span.end > slot_span.start
                || slot_span.start > slot_span.end
                || slot_span.end >= next.span.start
                || !self.template_token(Span::new(current.span.end, current.span.end + 2), "${")
                || !self.template_token(Span::new(next.span.start - 1, next.span.start), "}")
                || current.span.end + 2 > slot_span.start
            {
                return Err(FormatError::unsupported(
                    "template slot geometry",
                    None,
                    span,
                ));
            }
            indentation = current.indentation(indentation, tab_width);
            let geometry = TemplateSlot {
                content: slot_span,
                interior: Span::new(current.span.end + 2, next.span.start - 1),
            };
            let absolute = indentation > 0 || current.raw.ends_with('\n');
            writer.text("${");
            let mut format = |writer: &mut Writer<'ast>| format_slot(self, slot, geometry, writer);
            if absolute {
                writer.absolute_indent(indentation, format)?;
            } else {
                format(writer)?;
            }
            writer.text("}");
            writer.literal_text(next.raw);
            current = next;
        }
        if !current.tail
            || current.span.end != span.end - 1
            || !self.template_token(Span::new(current.span.end, span.end), "`")
        {
            return Err(FormatError::unsupported(
                "template closing geometry",
                None,
                span,
            ));
        }
        writer.text("`");
        Ok(())
    }
}
