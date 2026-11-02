use std::borrow::Cow;

use swc_next_ecma_ast::Span;

use swc_next_simd::prelude::{Select, SimdPartialOrd};
use swc_next_simd::{mask8x16, u8x16};

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
        let mut renderer = Renderer::new(source);
        for operation in operations {
            renderer.apply(operation);
        }
        Cow::Owned(renderer.finish())
    }
}

/// Streams normalized operations and the unedited source between them into
/// the output.
///
/// The output has at least `source.len()` bytes of capacity, and its length
/// never exceeds the consumed source length because edits keep or shrink the
/// byte length. Copies and ASCII blanks can therefore write whole blocks that
/// extend past the logical end of the output whenever the source has as many
/// bytes left to read; the next writes overwrite the extra bytes before they
/// become part of the output.
struct Renderer<'source> {
    source: &'source str,
    output: String,
    copied_until: usize,
}

/// Block size for copying unedited source.
const COPY_BLOCK: usize = 32;
/// Copies up to this length use inline blocks instead of `memcpy`.
const INLINE_COPY_LIMIT: usize = 128;

impl<'source> Renderer<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            output: String::with_capacity(source.len()),
            copied_until: 0,
        }
    }

    #[inline(always)]
    fn apply(&mut self, operation: Operation) {
        let source = self.source;
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

        self.copy_source(start);
        match operation {
            Operation::Blank(_) => self.blank(start, end),
            Operation::Substitute { replacement, .. } => {
                push_substitution(&mut self.output, &source[start..end], replacement);
            }
        }
        self.copied_until = end;
    }

    /// Copies the unedited source from `copied_until` to `end`.
    #[inline(always)]
    fn copy_source(&mut self, end: usize) {
        let source = self.source;
        let start = self.copied_until;
        let len = end - start;
        if len <= INLINE_COPY_LIMIT && start + len.next_multiple_of(COPY_BLOCK) <= source.len() {
            // SAFETY: Each block reads within `source`. The output length is
            // at most `start`, so each block also stays within the output
            // capacity. `start` and `end` are character boundaries, so the
            // new output length ends after valid UTF-8.
            unsafe {
                let output = self.output.as_mut_vec();
                let output_len = output.len();
                let from = source.as_ptr().add(start);
                let to = output.as_mut_ptr().add(output_len);
                let mut copied = 0;
                while copied < len {
                    std::ptr::copy_nonoverlapping(from.add(copied), to.add(copied), COPY_BLOCK);
                    copied += COPY_BLOCK;
                }
                output.set_len(output_len + len);
            }
        } else {
            self.output.push_str(&source[start..end]);
        }
    }

    #[inline(always)]
    fn blank(&mut self, start: usize, end: usize) {
        let source = self.source.as_bytes();
        let mut offset = start;

        // A vector that starts before `stop` reads within `source`.
        let stop = end.min((source.len() + 1).saturating_sub(u8x16::LEN));

        // SAFETY: Each vector reads within `source`. The output length is at
        // most `offset`, so each vector also stays within the output capacity.
        // The vectors are ASCII, so `offset` stays at a character boundary and
        // the final output length ends after valid UTF-8.
        unsafe {
            let output = self.output.as_mut_vec();
            let output_start = output.len();
            let mut destination = output.as_mut_ptr().add(output_start);
            while offset < stop {
                let bytes = u8x16::from_array(
                    source
                        .as_ptr()
                        .add(offset)
                        .cast::<[u8; u8x16::LEN]>()
                        .read_unaligned(),
                );
                if bytes.simd_gt(u8x16::splat(0x7F)).any() {
                    break;
                }
                let blanked = blank_ascii_vector(bytes).to_array();
                std::ptr::copy_nonoverlapping(blanked.as_ptr(), destination, u8x16::LEN);
                offset += u8x16::LEN;
                destination = destination.add(u8x16::LEN);
            }
            if offset >= end {
                output.set_len(output_start + (end - start));
                return;
            }
            output.set_len(output_start + (offset - start));
        }

        push_blank(&mut self.output, &self.source[offset..end]);
    }

    fn finish(mut self) -> String {
        let end = self.source.len();
        self.copy_source(end);
        self.output
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

/// Blanks sixteen ASCII bytes, keeping `\t..=\r`.
#[inline]
fn blank_ascii_vector(bytes: u8x16) -> u8x16 {
    let blank: mask8x16 = (bytes - u8x16::splat(b'\t')).simd_gt(u8x16::splat(b'\r' - b'\t'));
    blank.select(u8x16::splat(b' '), bytes)
}

/// Blanks eight ASCII bytes, keeping `\t..=\r`.
#[inline]
fn blank_ascii_word(word: u64) -> u64 {
    // Per byte, bit 7 is set when the byte is outside `\t..=\r`.
    let below = !(word + 0x7777_7777_7777_7777) & HIGH_BITS;
    let above = (word + 0x7272_7272_7272_7272) & HIGH_BITS;
    let blank = ((below | above) >> 7) * 0xFF;
    (word & !blank) | (0x2020_2020_2020_2020 & blank)
}

const HIGH_BITS: u64 = 0x8080_8080_8080_8080;

fn push_blank(output: &mut String, source: &str) {
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
            let mut vectors = 0;
            let (chunks, _) = source.as_bytes()[offset..].as_chunks::<{ u8x16::LEN }>();
            for chunk in chunks {
                let bytes = u8x16::from_array(*chunk);
                if bytes.simd_gt(u8x16::splat(0x7F)).any() {
                    break;
                }

                let blanked = blank_ascii_vector(bytes).to_array();
                std::ptr::copy_nonoverlapping(
                    blanked.as_ptr(),
                    destination.add(written + vectors * u8x16::LEN),
                    u8x16::LEN,
                );
                vectors += 1;
            }
            written += vectors * u8x16::LEN;
            offset += vectors * u8x16::LEN;

            while let Some(word) = source.as_bytes().get(offset..offset + 8) {
                let word = u64::from_le_bytes(word.try_into().unwrap());
                if word & HIGH_BITS != 0 {
                    break;
                }
                let blanked = blank_ascii_word(word);
                std::ptr::copy_nonoverlapping(
                    blanked.to_le_bytes().as_ptr(),
                    destination.add(written),
                    8,
                );
                written += 8;
                offset += 8;
            }

            while offset < source.len() {
                let byte = source.as_bytes()[offset];
                if byte.is_ascii() {
                    let blanked = if is_ascii_ecmascript_whitespace(byte) {
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

/// Matches ASCII bytes kept by blanking: `\t..=\r` and space.
fn is_ascii_ecmascript_whitespace(byte: u8) -> bool {
    matches!(byte, b'\t'..=b'\r' | b' ')
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
    fn blank_preserves_ascii_whitespace_in_vector_chunks() {
        let source: String = (0u8..=127).map(char::from).collect();
        let mut output = String::with_capacity(source.len());
        push_blank(&mut output, &source);
        let expected = blank_ascii(&source);

        assert_eq!(output, expected);
    }

    #[test]
    fn blank_preserves_ascii_whitespace_in_word_tail() {
        let source: String = (0u8..=127).map(char::from).collect();
        let mut output = String::with_capacity(source.len());
        for chunk in source.as_bytes().chunks(11) {
            push_blank(&mut output, std::str::from_utf8(chunk).unwrap());
        }
        let expected = blank_ascii(&source);

        assert_eq!(output, expected);
    }

    fn blank_ascii(source: &str) -> String {
        source
            .chars()
            .map(|character| match character {
                '\t'..='\r' | ' ' => character,
                _ => ' ',
            })
            .collect()
    }

    #[test]
    fn blank_handles_every_span_length() {
        let source: String = (0u8..=127).map(char::from).chain("abc".chars()).collect();
        for length in 1..=40 {
            for start in [0, 3, 9, 17, source.len() - length] {
                let end = start + length;
                let mut edits = EditPlan::default();
                edits.blank(Span::new(start as u32, end as u32));
                let output = edits.render(&source);
                let expected = format!(
                    "{}{}{}",
                    &source[..start],
                    blank_ascii(&source[start..end]),
                    &source[end..]
                );

                assert_eq!(output, expected);
            }
        }
    }

    #[test]
    fn ascii_and_non_ascii_edits_combine() {
        let source = "a: T; b: 文; c?: number; d as X;";
        let mut edits = EditPlan::default();
        edits.blank(Span::new(1, 4));
        edits.blank(Span::new(7, 12));
        edits.blank(Span::new(15, 24));
        edits.substitute_byte(24, FixedSubstitution::LeftParenthesis);
        let output = edits.render(source);

        assert_eq!(output, "a   ; b   ; c         ( d as X;");
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
