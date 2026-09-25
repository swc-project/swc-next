//! Literal token emission and normalization.

use swc_next_ecma_ast::RegExpLiteral;

use super::Formatter;
use crate::document::Writer;

impl<'ast, 'source> Formatter<'ast, 'source> {
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
