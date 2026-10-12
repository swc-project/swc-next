use swc_next_ecma_ast::Ast;

#[derive(Default)]
pub(super) struct SourceLocations {
    checkpoints: Option<Vec<SourcePosition>>,
}

struct SourcePosition {
    offset: u32,
    line: u32,
    column: u32,
}

impl SourceLocations {
    pub(super) fn line_column(&mut self, ast: &Ast<'_>, offset: u32) -> (u32, u32) {
        let checkpoints = self
            .checkpoints
            .get_or_insert_with(|| source_positions(ast.source()));
        let index = checkpoints.partition_point(|position| position.offset <= offset);
        // The initial checkpoint covers offset zero and every following offset.
        let position = &checkpoints[index - 1];
        (position.line, position.column + offset - position.offset)
    }
}

// Between checkpoints, each byte advances the UTF-16 column by one. Record
// only line breaks and non-ASCII characters, so even a single long line can
// be queried without rescanning its prefix for every JSX element.
fn source_positions(source: &str) -> Vec<SourcePosition> {
    let mut positions = vec![SourcePosition {
        offset: 0,
        line: 1,
        column: 1,
    }];
    let mut line = 1;
    let mut column = 1;
    let bytes = source.as_bytes();
    let mut start = 0;
    for end in memchr::memchr2_iter(b'\r', b'\n', bytes).chain(std::iter::once(bytes.len())) {
        let text = &source[start..end];
        if text.is_ascii() {
            column += text.len() as u32;
        } else {
            for (offset, character) in text.char_indices() {
                if matches!(character, '\u{2028}' | '\u{2029}') {
                    line += 1;
                    column = 1;
                } else {
                    column += character.len_utf16() as u32;
                }
                if !character.is_ascii() {
                    positions.push(SourcePosition {
                        offset: (start + offset + character.len_utf8()) as u32,
                        line,
                        column,
                    });
                }
            }
        }
        if let Some(&byte) = bytes.get(end) {
            let follows_cr = end > 0 && bytes[end - 1] == b'\r';
            line += u32::from(byte == b'\r' || !follows_cr);
            column = 1;
            start = end + 1;
            positions.push(SourcePosition {
                offset: start as u32,
                line,
                column,
            });
        }
    }
    positions
}
