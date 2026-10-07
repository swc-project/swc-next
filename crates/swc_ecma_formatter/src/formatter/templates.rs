//! Borrowed template text and physical delimiter geometry for zero-slot enum names.
use super::Formatter;
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{GetSpan, Span, SubRange, TemplateElement, TemplateLiteral};
struct TemplateQuasi<'ast> {
    span: Span,
    raw: &'ast str,
    tail: bool,
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
}
