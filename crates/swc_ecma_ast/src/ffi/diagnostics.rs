//! Packed diagnostic records used by the transform/minify bridge.
//!
//! Records keep the shipped wire format: a severity byte, UTF-16 spans, and
//! length-prefixed UTF-8 messages, help text and labels. The AST transfer buffer
//! carries its own diagnostics in the self-contained AST format, so this module
//! only serves callers that emit diagnostics separately from an AST.
//!
//! All integers are little-endian. Use a matching SWC Next consumer.

use std::mem::size_of;

use crate::Diagnostic;

const SPAN_SIZE: usize = 8;
// Skip ASCII runs in 64-byte chunks instead of decoding each byte as a character.
const ASCII_SCAN_CHUNK_SIZE: usize = 64;

/// Return the size of packed diagnostic records in the JavaScript transfer format.
pub fn diagnostics_buffer_size(diagnostics: &[Diagnostic]) -> usize {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let mut size = 0;

            size += size_of::<u8>(); // severity
            size += SPAN_SIZE; // start, end
            size += size_of::<u32>() + diagnostic.message.len(); // message_len, message
            size += size_of::<u8>(); // has_help
            if let Some(help) = &diagnostic.help {
                size += size_of::<u32>() + help.len(); // help_len, help
            }
            size += size_of::<u32>(); // label_count
            for label in &diagnostic.labels {
                size += SPAN_SIZE; // start, end
                size += size_of::<u32>() + label.message.len(); // message_len, message
            }

            size
        })
        .sum::<usize>()
}

/// Serialize packed diagnostics with UTF-16 spans into a caller-owned buffer.
/// This internal API and format do not follow semver; use a matching consumer.
///
/// Returns the required size without writing when the buffer is too small.
/// `source_utf16_len` must match the source's UTF-16 length.
pub fn serialize_diagnostics(
    source: &str,
    diagnostics: &[Diagnostic],
    buf: &mut [u8],
    source_utf16_len: usize,
) -> usize {
    let required = diagnostics_buffer_size(diagnostics);
    if buf.len() < required || diagnostics.is_empty() {
        return required;
    }
    let positions = build_pos_map(source, source_utf16_len);
    let mut cache = PositionCache::new(&positions, source.len() as u32);
    let mut pos = 0;
    write_diagnostics(diagnostics, buf, &mut pos, &mut cache);
    debug_assert_eq!(pos, required);
    required
}

fn write_diagnostics(
    diagnostics: &[Diagnostic],
    buf: &mut [u8],
    pos: &mut usize,
    cache: &mut PositionCache<'_>,
) {
    for diagnostic in diagnostics {
        write(buf, pos, &[diagnostic.severity as u8]);
        let start = cache.convert(diagnostic.span.start);
        let end = cache.convert(diagnostic.span.end);
        write_span(buf, pos, start, end);
        write_bytes_with_length(buf, pos, diagnostic.message.as_bytes());
        write(buf, pos, &[u8::from(diagnostic.help.is_some())]);
        if let Some(help) = &diagnostic.help {
            write_bytes_with_length(buf, pos, help.as_bytes());
        }
        let label_count =
            u32::try_from(diagnostic.labels.len()).expect("diagnostic label count exceeds u32");
        write(buf, pos, &label_count.to_le_bytes());
        for label in &diagnostic.labels {
            let start = cache.convert(label.span.start);
            let end = cache.convert(label.span.end);
            write_span(buf, pos, start, end);
            write_bytes_with_length(buf, pos, label.message.as_bytes());
        }
    }
}

// Each entry records the byte offset where the UTF-8 to UTF-16 difference changes.
type PositionMap = Vec<(u32, u32)>;

fn build_pos_map(source: &str, source_utf16_len: usize) -> PositionMap {
    if source.len() == source_utf16_len {
        return Vec::new();
    }

    let bytes = source.as_bytes();
    let mut positions = Vec::new();
    let mut byte = 0;
    let mut delta = 0;
    while byte < bytes.len() {
        let chunk_end = (byte + ASCII_SCAN_CHUNK_SIZE).min(bytes.len());
        if bytes[byte..chunk_end].is_ascii() {
            byte = chunk_end;
            continue;
        }
        while byte < chunk_end {
            let len = match bytes[byte] {
                0..=0x7f => 1,
                0xc0..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => unreachable!("source is valid UTF-8"),
            };
            if len > 1 {
                // Match the existing mapping even for offsets inside a UTF-8 scalar.
                for offset in 1..len {
                    if len == 4 && offset == 3 {
                        continue;
                    }
                    delta += 1;
                    positions.push(((byte + offset) as u32, delta));
                }
            }
            byte += len;
        }
    }
    positions
}

/// Cache an interval over which the UTF-8 to UTF-16 difference is constant.
struct PositionCache<'a> {
    positions: &'a PositionMap,
    source_len: u32,
    next: usize,
    range_start: u32,
    range_end: u64,
    delta: u32,
}

impl<'a> PositionCache<'a> {
    fn new(positions: &'a PositionMap, source_len: u32) -> Self {
        Self {
            positions,
            source_len,
            next: 0,
            range_start: 0,
            range_end: positions
                .first()
                .map_or(u64::from(source_len) + 1, |entry| u64::from(entry.0)),
            delta: 0,
        }
    }

    #[inline]
    fn convert(&mut self, byte: u32) -> u32 {
        if byte > self.source_len {
            return byte;
        }
        if byte >= self.range_start && u64::from(byte) < self.range_end {
            return byte - self.delta;
        }
        self.convert_slow(byte)
    }

    #[cold]
    fn convert_slow(&mut self, byte: u32) -> u32 {
        let positions = self.positions;
        let next = if byte < self.range_start {
            let mut next = self.next;
            let limit = next.saturating_sub(8);
            while next > limit && positions[next - 1].0 > byte {
                next -= 1;
            }
            if next == limit && next > 0 && positions[next - 1].0 > byte {
                next = positions[..next].partition_point(|&(offset, _)| offset <= byte);
            }
            next
        } else {
            let mut next = self.next;
            let limit = (next + 8).min(positions.len());
            while next < limit && positions[next].0 <= byte {
                next += 1;
            }
            if next == limit {
                next += positions[next..].partition_point(|&(offset, _)| offset <= byte);
            }
            next
        };
        self.next = next;
        if next == 0 {
            self.range_start = 0;
            self.delta = 0;
        } else {
            (self.range_start, self.delta) = positions[next - 1];
        }
        self.range_end = positions
            .get(next)
            .map_or(u64::from(self.source_len) + 1, |entry| u64::from(entry.0));
        byte - self.delta
    }
}

#[inline]
fn write(buf: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    buf[*pos..*pos + bytes.len()].copy_from_slice(bytes);
    *pos += bytes.len();
}

#[inline]
fn write_span(buf: &mut [u8], pos: &mut usize, start: u32, end: u32) {
    let output = &mut buf[*pos..*pos + SPAN_SIZE];
    output[..4].copy_from_slice(&start.to_le_bytes());
    output[4..].copy_from_slice(&end.to_le_bytes());
    *pos += SPAN_SIZE;
}

#[inline]
fn write_bytes_with_length(buf: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).expect("byte sequence length exceeds u32");
    let size = size_of::<u32>() + bytes.len();
    let output = &mut buf[*pos..*pos + size];
    output[..4].copy_from_slice(&len.to_le_bytes());
    output[4..].copy_from_slice(bytes);
    *pos += size;
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::{Label, Severity, Span};

    fn u32_at(buffer: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap())
    }

    #[test]
    fn packed_diagnostic_format() {
        let diagnostics = [Diagnostic {
            severity: Severity::Warning,
            message: Cow::Borrowed("é"),
            span: Span::new(0, 2),
            help: Some(Cow::Borrowed("h")),
            labels: vec![Label {
                span: Span::new(2, 3),
                message: Cow::Borrowed("abc"),
            }],
        }];
        let required = diagnostics_buffer_size(&diagnostics);
        assert_eq!(required, 40);

        let mut too_small = vec![0xaa; required - 1];
        assert_eq!(
            serialize_diagnostics("éx", &diagnostics, &mut too_small, 2),
            required
        );
        assert!(too_small.iter().all(|&byte| byte == 0xaa));

        let mut buffer = vec![0; required];
        assert_eq!(
            serialize_diagnostics("éx", &diagnostics, &mut buffer, 2),
            required
        );
        assert_eq!(buffer[0], Severity::Warning as u8);
        assert_eq!(u32_at(&buffer, 1), 0); // start
        assert_eq!(u32_at(&buffer, 5), 1); // end
        assert_eq!(u32_at(&buffer, 9), 2); // message length
        assert_eq!(&buffer[13..15], "é".as_bytes());
        assert_eq!(buffer[15], 1); // has help
        assert_eq!(u32_at(&buffer, 16), 1); // help length
        assert_eq!(buffer[20], b'h');
        assert_eq!(u32_at(&buffer, 21), 1); // label count
        assert_eq!(u32_at(&buffer, 25), 1); // label start
        assert_eq!(u32_at(&buffer, 29), 2); // label end
        assert_eq!(u32_at(&buffer, 33), 3); // label message length
        assert_eq!(&buffer[37..40], b"abc");
    }
}
