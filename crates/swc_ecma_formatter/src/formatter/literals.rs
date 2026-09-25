//! Literal token emission and normalization.

mod number;

use swc_next_ecma_ast::{BigIntLiteral, NumericLiteral, RegExpLiteral};

use super::Formatter;
use crate::document::Writer;

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_numeric_literal(
        &self,
        literal: NumericLiteral,
        writer: &mut Writer<'ast>,
    ) {
        match literal.raw(self.ast) {
            Some(raw) => writer.text(number::format_number(self.ast.get_utf8(raw))),
            None => {
                let mut buffer = zmij_ecma::Buffer::new();
                writer.text_copied(buffer.format(literal.value(self.ast)));
            }
        }
    }

    pub(super) fn format_bigint_literal(&self, literal: BigIntLiteral, writer: &mut Writer<'ast>) {
        writer.text(number::lowercase(self.ast.get_utf8(literal.raw(self.ast))));
        writer.text("n");
    }

    pub(super) fn format_regexp_literal(&self, literal: RegExpLiteral, writer: &mut Writer<'ast>) {
        writer.text("/");
        writer.text(self.ast.get_utf8(literal.pattern(self.ast)));
        writer.text("/");

        let flags = self.ast.get_utf8(literal.flags(self.ast));
        if flags.as_bytes().windows(2).all(|pair| pair[0] <= pair[1]) {
            writer.text(flags);
        } else {
            let mut sorted: Vec<_> = flags.chars().collect();
            sorted.sort_unstable();
            let sorted: String = sorted.into_iter().collect();
            writer.text(sorted);
        }
    }
}
