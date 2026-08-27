use std::borrow::Cow;

use swc_next_ecma_ast::Span;

use crate::simd::prelude::SimdPartialOrd;
use crate::simd::{AVAILABLE as SIMD_AVAILABLE, mask8x16, u8x16};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FixedSubstitution {
    #[allow(dead_code)]
    Semicolon,
    #[allow(dead_code)]
    LeftParenthesis,
    #[allow(dead_code)]
    RightParenthesis,
}

impl FixedSubstitution {
    fn character(self) -> char {
        match self {
            Self::Semicolon => ';',
            Self::LeftParenthesis => '(',
            Self::RightParenthesis => ')',
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operation {
    Blank(Span),
    Substitute {
        span: Span,
        replacement: FixedSubstitution,
    },
}

impl Operation {
    fn span(self) -> Span {
        match self {
            Self::Blank(span) | Self::Substitute { span, .. } => span,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct EditPlan {
    operations: Vec<Operation>,
}

impl EditPlan {
    pub(crate) fn blank(&mut self, span: Span) {
        assert!(span.start <= span.end, "invalid edit span: {span:?}");
        if span.start != span.end {
            self.operations.push(Operation::Blank(span));
        }
    }

    pub(crate) fn substitute(&mut self, span: Span, replacement: FixedSubstitution) {
        assert!(span.start < span.end, "invalid substitution span: {span:?}");
        self.operations
            .push(Operation::Substitute { span, replacement });
    }

    #[allow(dead_code)]
    pub(crate) fn substitute_byte(&mut self, offset: u32, replacement: FixedSubstitution) {
        self.substitute(Span::new(offset, offset + 1), replacement);
    }

    pub(crate) fn render<'source>(mut self, source: &'source str) -> Cow<'source, str> {
        if self.operations.is_empty() {
            return Cow::Borrowed(source);
        }

        let operations = normalize_operations(&mut self.operations);
        let mut output = String::with_capacity(source.len());
        let mut copied_until = 0;

        for operation in operations {
            let span = operation.span();
            let start = span.start as usize;
            let end = span.end as usize;
            assert!(
                end <= source.len(),
                "edit span is outside the source: {span:?}"
            );
            assert!(
                source.is_char_boundary(start) && source.is_char_boundary(end),
                "edit span does not follow UTF-8 character boundaries: {span:?}"
            );

            output.push_str(&source[copied_until..start]);
            match operation {
                Operation::Blank(_) => push_blank(&mut output, &source[start..end]),
                Operation::Substitute { replacement, .. } => {
                    push_substitution(&mut output, &source[start..end], replacement);
                }
            }
            copied_until = end;
        }

        output.push_str(&source[copied_until..]);
        Cow::Owned(output)
    }
}

fn normalize_operations(operations: &mut [Operation]) -> Vec<Operation> {
    operations.sort_unstable_by_key(|operation| {
        let span = operation.span();
        let is_substitution = matches!(operation, Operation::Substitute { .. });
        (span.start, is_substitution, span.end)
    });

    let mut normalized = Vec::with_capacity(operations.len());
    let mut active_blank: Option<Span> = None;
    let mut previous_substitution: Option<(Span, FixedSubstitution)> = None;

    for operation in operations.iter().copied() {
        match operation {
            Operation::Blank(mut span) => {
                if let Some((substitution, _)) = previous_substitution
                    && span.start < substitution.end
                    && span.end > substitution.start
                {
                    if span.end <= substitution.end {
                        continue;
                    }
                    span.start = substitution.end;
                }

                if let Some(active) = &mut active_blank {
                    if span.start <= active.end {
                        active.end = active.end.max(span.end);
                    } else {
                        normalized.push(Operation::Blank(*active));
                        *active = span;
                    }
                } else {
                    active_blank = Some(span);
                }
            }
            Operation::Substitute { span, replacement } => {
                if let Some((previous_span, previous_replacement)) = previous_substitution {
                    if previous_span.start == span.start {
                        assert_eq!(previous_span, span, "conflicting substitution spans");
                        assert_eq!(
                            previous_replacement, replacement,
                            "conflicting substitutions"
                        );
                        continue;
                    }
                    assert!(
                        previous_span.end <= span.start,
                        "overlapping substitutions are not supported"
                    );
                }

                if let Some(blank) = active_blank {
                    if blank.end <= span.start {
                        normalized.push(Operation::Blank(blank));
                        active_blank = None;
                    } else {
                        if blank.start < span.start {
                            normalized.push(Operation::Blank(Span::new(blank.start, span.start)));
                        }
                        active_blank = (blank.end > span.end)
                            .then(|| Span::new(blank.start.max(span.end), blank.end));
                    }
                }

                normalized.push(operation);
                previous_substitution = Some((span, replacement));
            }
        }
    }

    if let Some(blank) = active_blank {
        normalized.push(Operation::Blank(blank));
    }

    normalized
}

fn push_blank(output: &mut String, source: &str) {
    if SIMD_AVAILABLE {
        push_blank_simd(output, source);
    } else {
        push_blank_scalar(output, source);
    }
}

fn push_blank_scalar(output: &mut String, source: &str) {
    debug_assert!(output.capacity() - output.len() >= source.len());
    let output_start = output.len();
    let mut written = 0;
    let mut offset = 0;

    // SAFETY: `output` has enough spare capacity for `source`. Each write stays
    // within that capacity and produces valid UTF-8.
    unsafe {
        let output = output.as_mut_vec();
        let destination = output.as_mut_ptr().add(output_start);

        while offset < source.len() {
            let byte = source.as_bytes()[offset];
            if byte.is_ascii() {
                let blanked = if byte.is_ascii_whitespace() {
                    byte
                } else {
                    b' '
                };
                destination.add(written).write(blanked);
                offset += 1;
                written += 1;
                continue;
            }

            // SAFETY: `offset` is at a non-ASCII character boundary and the
            // destination has `source.len() - offset` bytes available.
            let (consumed, produced) = blank_non_ascii(destination.add(written), &source[offset..]);
            offset += consumed;
            written += produced;
        }

        output.set_len(output_start + written);
    }
}

fn push_blank_simd(output: &mut String, source: &str) {
    debug_assert!(output.capacity() - output.len() >= source.len());
    let output_start = output.len();
    let mut written = 0;
    let mut offset = 0;

    // SAFETY: `output` has enough spare capacity for `source`. Each write stays
    // within that capacity and produces valid UTF-8.
    unsafe {
        let output = output.as_mut_vec();
        let destination = output.as_mut_ptr().add(output_start);

        while offset < source.len() {
            while offset + u8x16::LEN <= source.len() {
                let bytes = u8x16::from_slice(&source.as_bytes()[offset..]);
                let non_ascii = bytes.simd_ge(u8x16::splat(0x80));
                if non_ascii.any() {
                    break;
                }

                let control_whitespace: mask8x16 =
                    bytes.simd_ge(u8x16::splat(b'\t')) & bytes.simd_le(u8x16::splat(b'\r'));
                let blanked = control_whitespace
                    .select(bytes, u8x16::splat(b' '))
                    .to_array();
                std::ptr::copy_nonoverlapping(
                    blanked.as_ptr(),
                    destination.add(written),
                    u8x16::LEN,
                );
                written += u8x16::LEN;
                offset += u8x16::LEN;
            }

            while offset < source.len() {
                let byte = source.as_bytes()[offset];
                if byte.is_ascii() {
                    let blanked = if byte.is_ascii_whitespace() {
                        byte
                    } else {
                        b' '
                    };
                    destination.add(written).write(blanked);
                    offset += 1;
                    written += 1;
                    continue;
                }

                // SAFETY: `offset` is at a non-ASCII character boundary and
                // the destination has `source.len() - offset` bytes available.
                let (consumed, produced) =
                    blank_non_ascii(destination.add(written), &source[offset..]);
                offset += consumed;
                written += produced;
                break;
            }
        }

        output.set_len(output_start + written);
    }
}

#[cold]
#[inline(never)]
unsafe fn blank_non_ascii(destination: *mut u8, source: &str) -> (usize, usize) {
    let character = source
        .chars()
        .next()
        .expect("blanking source must start with a character");
    let consumed = character.len_utf8();

    if is_ecmascript_whitespace_or_line_terminator(character) {
        // SAFETY: The caller provides `source.len()` bytes of writable space.
        unsafe {
            std::ptr::copy_nonoverlapping(source.as_ptr(), destination, consumed);
        }
        (consumed, consumed)
    } else {
        let produced = character.len_utf16();
        // SAFETY: UTF-16 width never exceeds the character's UTF-8 width.
        unsafe {
            destination.write_bytes(b' ', produced);
        }
        (consumed, produced)
    }
}

fn push_substitution(output: &mut String, source: &str, replacement: FixedSubstitution) {
    let replaced_width = source.encode_utf16().count();
    assert!(replaced_width > 0, "cannot substitute an empty span");
    output.push(replacement.character());
    output.extend(std::iter::repeat_n(' ', replaced_width - 1));
}

fn is_ecmascript_whitespace_or_line_terminator(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_edits_returns_borrowed_source() {
        let source = "const value = 1;";
        let output = EditPlan::default().render(source);

        assert!(matches!(output, Cow::Borrowed(value) if value == source));
    }

    #[test]
    fn blank_replaces_code_with_spaces() {
        let source = "let value: number = 1;";
        let mut edits = EditPlan::default();
        edits.blank(Span::new(9, 17));
        let output = edits.render(source);

        assert_eq!(output, "let value         = 1;");
    }

    #[test]
    fn blank_preserves_ecmascript_whitespace_and_line_terminators() {
        let source = ":\tA\r\nB\u{2028}C\u{2029}D\u{00A0}";
        let mut edits = EditPlan::default();
        edits.blank(Span::new(0, source.len() as u32));
        let output = edits.render(source);

        assert_eq!(output, " \t \r\n \u{2028} \u{2029} \u{00A0}");
    }

    #[test]
    fn blank_preserves_utf16_width_for_bmp_and_astral_code_points() {
        let source = ": 文𐀀";
        let mut edits = EditPlan::default();
        edits.blank(Span::new(0, source.len() as u32));
        let output = edits.render(source);

        assert_eq!(output, "     ");
        assert_eq!(output.encode_utf16().count(), source.encode_utf16().count());
    }

    #[test]
    fn overlapping_blanks_are_merged() {
        let source = "abcdef";
        let mut edits = EditPlan::default();
        edits.blank(Span::new(1, 4));
        edits.blank(Span::new(3, 5));
        let output = edits.render(source);

        assert_eq!(output, "a    f");
    }

    #[test]
    fn substitution_wins_over_overlapping_blanks() {
        let source = "abstract [key]: Type";
        let mut edits = EditPlan::default();
        edits.blank(Span::new(0, 8));
        edits.blank(Span::new(4, 15));
        edits.substitute_byte(0, FixedSubstitution::Semicolon);
        let output = edits.render(source);

        assert_eq!(output, ";               Type");
    }

    #[test]
    fn substitution_preserves_utf16_width() {
        let source = "value as 类型𝒳";
        let start = source.len() - "𝒳".len();
        let mut edits = EditPlan::default();
        edits.blank(Span::new(5, source.len() as u32));
        edits.substitute(
            Span::new(start as u32, source.len() as u32),
            FixedSubstitution::RightParenthesis,
        );
        let output = edits.render(source);

        assert_eq!(output, "value      ) ");
        assert_eq!(output.encode_utf16().count(), source.encode_utf16().count());
    }
}
