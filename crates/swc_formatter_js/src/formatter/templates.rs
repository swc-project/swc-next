//! Borrowed template text and physical quasi/slot delimiter geometry.

use swc_next_ecma_ast::{
    AstNode, Comment, GetSpan, Span, SubRange, TemplateElement, TemplateLiteral,
};

use super::Formatter;
use super::comments::{CommentGap, CommentRunLayout, HeadGapPlacement};
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
    pub(super) source_newline: bool,
}

/// The child or its interpolation boundaries own finite-width layout.
#[derive(Clone, Copy)]
pub(super) enum TemplateSlotLayout {
    Child,
    Boundaries(ContentMode),
}

impl TemplateSlotLayout {
    fn with_boundaries(self) -> Self {
        match self {
            Self::Child => Self::Boundaries(ContentMode::Formatted),
            Self::Boundaries(_) => self,
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn template_slot_head_layout(
        &self,
        comments: &[Comment],
        next: u32,
    ) -> CommentRunLayout {
        let placement = if comments
            .first()
            .is_some_and(|comment| self.layout_position_starts_line(comment.span.start))
        {
            HeadGapPlacement::OwnLine
        } else {
            HeadGapPlacement::Inline
        };
        CommentRunLayout::SourceLeading { next, placement }
    }

    /// Template Raw Value comes from the AST. Delimiter offsets come from source spans.
    pub(super) fn template_element_raw(&self, node: TemplateElement) -> &'ast str {
        self.ast.get_utf8(node.raw(self.ast))
    }

    pub(super) fn template_raw_parts(
        &self,
        template: TemplateLiteral,
    ) -> impl Iterator<Item = &'ast str> {
        template
            .quasis(self.ast)
            .iter()
            .map(|id| self.template_element_raw(self.ast.get_node_in_sub_range(id)))
    }

    pub(super) fn single_template_raw(&self, template: TemplateLiteral) -> Option<&'ast str> {
        if !template.expressions(self.ast).is_empty() {
            return None;
        }
        self.single_template_element(template.quasis(self.ast))
            .map(|element| self.template_element_raw(element))
    }

    fn single_template_element(
        &self,
        quasis: SubRange<TemplateElement>,
    ) -> Option<TemplateElement> {
        let id = quasis.first().filter(|_| quasis.len() == 1)?;
        Some(self.ast.get_node_in_sub_range(id))
    }

    fn template_quasi(&self, node: TemplateElement) -> TemplateQuasi<'ast> {
        TemplateQuasi {
            span: node.span(self.ast),
            raw: self.template_element_raw(node),
            tail: node.tail(self.ast),
        }
    }

    fn template_token(&self, span: Span, token: &str) -> bool {
        self.ast
            .source()
            .get(span.start as usize..span.end as usize)
            == Some(token)
    }

    fn template_slot_between(
        &self,
        template: Span,
        preceding: &TemplateQuasi<'_>,
        following: &TemplateQuasi<'_>,
        content: Span,
    ) -> Result<TemplateSlot, FormatError> {
        if following.span.end >= template.end
            || following.span.start > following.span.end
            || preceding.span.start > preceding.span.end
            || preceding.tail
            || preceding.span.end > content.start
            || content.start > content.end
            || content.end >= following.span.start
            || !self.template_token(Span::new(preceding.span.end, preceding.span.end + 2), "${")
            || !self.template_token(
                Span::new(following.span.start - 1, following.span.start),
                "}",
            )
            || preceding.span.end + 2 > content.start
        {
            return Err(FormatError::unsupported(
                "template slot geometry",
                None,
                template,
            ));
        }
        let interior = Span::new(preceding.span.end + 2, following.span.start - 1);
        Ok(TemplateSlot {
            content,
            interior,
            source_newline: self.source(interior).contains(['\r', '\n']),
        })
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
        format: impl FnOnce(
            &mut Self,
            TemplateSlotLayout,
            &mut Writer<'ast>,
        ) -> Result<CommentGap, FormatError>,
    ) -> Result<(), FormatError> {
        if matches!(opening, LineConstraint::NewLine(_)) || closing.requires_line() {
            let layout = layout.with_boundaries();
            return Self::write_template_slot(layout, opening, writer, |writer| {
                format(self, layout, writer)
            });
        }
        let source_newline = slot.source_newline;
        if source_newline
            && matches!(
                layout,
                TemplateSlotLayout::Boundaries(ContentMode::Verbatim)
            )
        {
            return Self::write_template_slot(layout, opening, writer, |writer| {
                format(self, layout, writer)
            });
        }
        let (content, closing) = writer.record_with_value(|writer| format(self, layout, writer))?;
        let layout = if closing.requires_line() {
            // A child can release comments from an interior pair. The slot's
            // actual closing delimiter owns that line even without an outer run.
            layout.with_boundaries()
        } else {
            layout
        };
        if !closing.requires_line() && !source_newline && !content.preferred_breaks() {
            if let Some(content) = content.try_flat() {
                writer.write_flat_fragment(content);
                return Ok(());
            }
            if !content.has_intrinsic_lines() {
                // Keep local group bindings even when the slot is laid out inline.
                writer.write_fragment_without_lines(content);
                return Ok(());
            }
        }
        Self::write_template_slot(layout, opening, writer, |writer| {
            writer.write_fragment(content);
            Ok(closing)
        })
    }

    fn write_template_slot(
        layout: TemplateSlotLayout,
        opening: LineConstraint,
        writer: &mut Writer<'ast>,
        format: impl FnOnce(&mut Writer<'ast>) -> Result<CommentGap, FormatError>,
    ) -> Result<(), FormatError> {
        writer.group(false, |writer| match layout {
            TemplateSlotLayout::Child => format(writer).map(|_| ()),
            TemplateSlotLayout::Boundaries(_) => {
                let closing = writer.indent(|writer| {
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
        let Some(element) = self.single_template_element(quasis) else {
            return Err(FormatError::unsupported("template quasi count", None, span));
        };
        let quasi = self.template_quasi(element);
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
        S: GetSpan + AstNode,
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
            let geometry = self.template_slot_between(span, &current, &next, slot.span(ast))?;
            indentation = current.indentation(indentation, tab_width);
            let absolute = indentation > 0 || current.raw.ends_with('\n');
            writer.text("${");
            let mut format = |writer: &mut Writer<'ast>| format_slot(self, slot, geometry, writer);
            if absolute {
                writer.absolute_indent(indentation, format)?;
            } else {
                format(writer)?;
            }
            writer.line_suffix_boundary();
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
