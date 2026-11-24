use swc_next_ecma_ast::*;

use super::{object, property, string, undefined};

pub(crate) struct JsxSrc {
    lines: Vec<usize>,
}

impl JsxSrc {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_common/src/syntax_pos/analyze_source_file.rs#L19>.
    pub(crate) fn new(source: &str) -> Self {
        let mut lines = vec![0];
        let mut previous_cr = false;
        for (offset, character) in source.char_indices() {
            if character == '\n' && previous_cr {
                *lines.last_mut().unwrap() = offset + 1;
            } else if matches!(character, '\r' | '\n') {
                lines.push(offset + character.len_utf8());
            }
            previous_cr = character == '\r';
        }
        Self { lines }
    }

    /// SWC source object: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_src.rs#L18>.
    /// SWC fragment source argument: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L499>.
    /// SWC line lookup: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_common/src/source_map.rs#L318>.
    /// SWC UTF-16 columns: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_common/src/source_map.rs#L1018>.
    pub(crate) fn jsx_dev_source_arg(&self, ast: &mut Ast<'_>, span: Span, filename: &str) -> Expr {
        if span == DUMMY_SP {
            return undefined(ast);
        }
        let offset = span.start as usize;
        let line = self.lines.partition_point(|&start| start <= offset);
        // SWC SourceMap reports UTF-16 columns.
        let column = ast.source()[self.lines[line - 1]..offset]
            .encode_utf16()
            .count()
            + 1;
        let filename = string(ast, DUMMY_SP, filename);
        let line = ast.expr_numeric_literal(DUMMY_SP, line as f64, None);
        let column = ast.expr_numeric_literal(DUMMY_SP, column as f64, None);
        let properties = vec![
            property(ast, "fileName", filename),
            property(ast, "lineNumber", line),
            property(ast, "columnNumber", column),
        ];
        object(ast, properties)
    }
}
