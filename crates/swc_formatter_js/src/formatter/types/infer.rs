//! Infer consumes its real parameter independently of generic-list punctuation.

use swc_next_ecma_ast::{GetSpan, Span, TsInferType};

use super::generics::TypeParameterContext;
use super::{Formatter, TypeOutput};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_infer_type_body(
        &mut self,
        node: TsInferType,
        continuation: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<TypeOutput<'ast>, FormatError> {
        let parameter = node.type_parameter(self.ast);
        let prefix = self.prepare_type_prefix(Span::new(
            node.span(self.ast).start + 5,
            parameter.span(self.ast).start,
        ));
        let prepared = if prefix.mode == ContentMode::Verbatim {
            None
        } else {
            Some(self.prepare_type_parameter(parameter, TypeParameterContext::Infer)?)
        };
        writer.text("infer");
        self.format_type_prefix(prefix, FlatSeparator::Space, writer)?;
        match prepared {
            None => {
                self.format_verbatim(parameter.span(self.ast), writer)?;
                Ok(TypeOutput::closed())
            }
            Some(parameter) => {
                self.format_prepared_type_parameter_body(parameter, continuation, writer)
            }
        }
    }
}
