//! Runtime class prefixes and source-owned field separators.
use crate::formatter::Formatter;
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{GetSpan, PropertyKey, Span};

pub(in crate::formatter) struct ClassMemberPrefix {
    span: Span,
    is_static: bool,
}
impl ClassMemberPrefix {
    pub(in crate::formatter) fn ordinary(start: u32, is_static: bool) -> Self {
        Self {
            span: Span::new(start, start + if is_static { 6 } else { 0 }),
            is_static,
        }
    }
    pub(in crate::formatter) fn end(&self) -> u32 {
        self.span.end
    }
    pub(in crate::formatter) fn has_modifiers(&self) -> bool {
        self.is_static
    }
}
impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_class_member_prefix(
        &mut self,
        prefix: &ClassMemberPrefix,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if prefix.is_static {
            writer.text("static");
        }
        Ok(())
    }
    pub(in crate::formatter) fn source_class_member_needs_field_separator(
        &self,
        span: Span,
        key: PropertyKey,
    ) -> bool {
        let source = self.source(span);
        source.starts_with(['[', '*'])
            || span.start == key.span(self.ast).start
                && matches!(
                    self.property_key_identifier(
                        key,
                        crate::formatter::expressions::objects::PropertyKeyContext::Ordinary,
                        crate::formatter::suppression::ContentMode::Verbatim,
                        crate::formatter::expressions::objects::PropertyKeyQuoting::Preserve
                    ),
                    Some("in" | "instanceof")
                )
    }
}
