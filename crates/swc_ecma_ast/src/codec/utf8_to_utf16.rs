/// PositionMap provides bidirectional mapping between UTF-8 byte offsets (used by Go)
/// and UTF-16 code unit offsets (used by JavaScript/TypeScript).
///
/// For ASCII-only text, the two are identical. For text containing non-ASCII characters,
/// the offsets diverge because multi-byte UTF-8 sequences map to different numbers of
/// UTF-16 code units:
///   - U+0000..U+007F:   1 byte  in UTF-8, 1 code unit  in UTF-16
///   - U+0080..U+07FF:   2 bytes in UTF-8, 1 code unit  in UTF-16
///   - U+0800..U+FFFF:   3 bytes in UTF-8, 1 code unit  in UTF-16
///   - U+10000..U+10FFFF: 4 bytes in UTF-8, 2 code units in UTF-16 (surrogate pair)
pub struct PositionMap {
    /// asciiOnly is true if the text contains only ASCII characters,
    /// meaning UTF-8 byte offsets and UTF-16 code unit offsets are identical.
    ascii_only: bool,
    /// UTF-16 code unit length of the source text.
    source_len: u32,
    // For each multi-byte character, we store:
    //   - the UTF-8 byte offset of the character
    //   - the cumulative delta (utf8Offset - utf16Offset) at that character
    // This allows O(log n) conversion in either direction.
    //
    // entries[i].utf8Pos is the byte offset of the i-th multi-byte character.
    // entries[i].delta is the total (utf8 - utf16) difference accumulated
    // through and including the i-th multi-byte character.
    entries: Vec<PositionMapEntry>,
}

struct PositionMapEntry {
    /// UTF-8 byte offset AFTER this multi-byte character
    utf8_pos: u32,
    /// cumulative (utf8 - utf16) offset difference after this character
    delta: i32,
}

impl PositionMap {
    pub fn new(source: &str) -> Self {
        let mut entries = Vec::new();
        let mut delta = 0;
        let mut source_len = 0;

        for (i, ch) in source.char_indices() {
            source_len += ch.len_utf16() as u32;
            if ch.is_ascii() {
                continue;
            }

            let utf8_size = ch.len_utf8() as i32;
            let utf16_size = ch.len_utf16() as i32;
            delta += utf8_size - utf16_size;

            entries.push(PositionMapEntry {
                utf8_pos: (i + ch.len_utf8()) as u32,
                delta,
            });
        }

        Self {
            ascii_only: entries.is_empty(),
            source_len,
            entries,
        }
    }

    /// IsAsciiOnly returns true if the text is ASCII-only,
    /// meaning UTF-8 and UTF-16 offsets are identical.
    #[cfg(test)]
    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    pub fn convert(&self, offset: u32) -> u32 {
        self.utf8_to_utf16(offset)
    }

    pub fn source_len(&self) -> u32 {
        self.source_len
    }

    /// UTF8ToUTF16 converts a UTF-8 byte offset to a UTF-16 code unit offset.
    pub fn utf8_to_utf16(&self, utf8_offset: u32) -> u32 {
        if self.ascii_only {
            return utf8_offset;
        }

        // Binary search: find the last entry where utf8Pos <= utf8Offset
        let idx = self
            .entries
            .partition_point(|entry| entry.utf8_pos <= utf8_offset);

        if idx == 0 {
            // Before any multi-byte character
            return utf8_offset;
        }

        utf8_offset - self.entries[idx - 1].delta as u32
    }

    /// UTF16ToUTF8 converts a UTF-16 code unit offset to a UTF-8 byte offset.
    #[cfg(test)]
    pub fn utf16_to_utf8(&self, utf16_offset: u32) -> u32 {
        if self.ascii_only {
            return utf16_offset;
        }

        // We need the last entry where (utf8Pos - delta) <= utf16Offset.
        // (utf8Pos - delta) is the UTF-16 offset of that entry's character.
        let idx = self.entries.partition_point(|entry| {
            let utf16_pos = entry.utf8_pos - entry.delta as u32;
            utf16_pos <= utf16_offset
        });

        if idx == 0 {
            return utf16_offset;
        }

        utf16_offset + self.entries[idx - 1].delta as u32
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn position_map_ascii() {
        let text = "const x = 1;";
        let pm = PositionMap::new(text);
        assert!(pm.is_ascii_only(), "expected ASCII-only");

        for i in 0..=text.len() as u32 {
            assert_eq!(pm.utf8_to_utf16(i), i, "UTF8ToUTF16({i})");
            assert_eq!(pm.utf16_to_utf8(i), i, "UTF16ToUTF8({i})");
        }
    }

    #[test]
    fn position_map_two_byte() {
        // "café" — é (U+00E9) is 2 bytes UTF-8, 1 code unit UTF-16
        let text = "const café = 1;\nconst x = 2;";
        let pm = PositionMap::new(text);
        assert!(!pm.is_ascii_only(), "expected non-ASCII");

        // Everything before é (byte offset 9) should be identity
        for i in 0..10 {
            assert_eq!(pm.utf8_to_utf16(i), i, "before é: UTF8ToUTF16({i})");
        }

        // é starts at UTF-8 byte 9, UTF-16 offset 9: same
        assert_eq!(pm.utf8_to_utf16(9), 9, "at é: UTF8ToUTF16(9)");

        // After é (byte 11 in UTF-8 = code unit 10 in UTF-16), delta is 1
        // ' ' after café: UTF-8 byte 11, UTF-16 offset 10
        assert_eq!(pm.utf8_to_utf16(11), 10, "after é: UTF8ToUTF16(11)");

        // 'x' on second line: UTF-8 byte 23, UTF-16 offset 22
        let x_utf8 = text.rfind('x').unwrap() as u32;
        assert_eq!(
            pm.utf8_to_utf16(x_utf8),
            x_utf8 - 1,
            "at x: UTF8ToUTF16({x_utf8})"
        );

        // Reverse: UTF-16 offset 22 should map to UTF-8 byte 23
        let x_utf16 = x_utf8 - 1;
        assert_eq!(
            pm.utf16_to_utf8(x_utf16),
            x_utf8,
            "reverse at x: UTF16ToUTF8({x_utf16})"
        );
    }

    #[test]
    fn position_map_four_byte() {
        // 🎉 (U+1F389) is 4 bytes UTF-8, 2 code units UTF-16
        let text = "const a = \"🎉\";\nconst b = 2;";
        let pm = PositionMap::new(text);
        assert!(!pm.is_ascii_only(), "expected non-ASCII");

        // 🎉 starts at byte 11 (after `const a = "`)
        // UTF-8: bytes 11-14 (4 bytes), UTF-16: units 11-12 (2 code units)
        // After 🎉: UTF-8 byte 15, UTF-16 offset 13. Delta = 2.

        // 'b' on second line
        let b_utf8 = text.rfind('b').unwrap() as u32;
        let b_utf16 = b_utf8 - 2; // delta of 2 from emoji
        assert_eq!(pm.utf8_to_utf16(b_utf8), b_utf16, "at b");
        assert_eq!(pm.utf16_to_utf8(b_utf16), b_utf8, "reverse at b");
    }

    #[test]
    fn position_map_multiple_non_ascii() {
        // Mix of 2-byte and 4-byte characters
        // "à" (U+00E0) = 2 bytes UTF-8, 1 code unit UTF-16 (delta +1)
        // "🎉" (U+1F389) = 4 bytes UTF-8, 2 code units UTF-16 (delta +2)
        let text = "à🎉x";
        let pm = PositionMap::new(text);

        // à: UTF-8 [0,2), UTF-16 [0,1)
        // 🎉: UTF-8 [2,6), UTF-16 [1,3)
        // x: UTF-8 [6,7), UTF-16 [3,4)
        let tests = [(0, 0), (2, 1), (6, 3), (7, 4)];

        for (utf8, utf16) in tests {
            assert_eq!(pm.utf8_to_utf16(utf8), utf16, "UTF8ToUTF16({utf8})");
            assert_eq!(pm.utf16_to_utf8(utf16), utf8, "UTF16ToUTF8({utf16})");
        }
    }

    #[test]
    fn position_map_roundtrip() {
        let text = "let café = \"🎉\"; // naïve";
        let pm = PositionMap::new(text);

        // Convert every valid UTF-16 position to UTF-8 and back
        let utf16_len = pm.utf8_to_utf16(text.len() as u32);
        for i in 0..=utf16_len {
            let utf8_pos = pm.utf16_to_utf8(i);
            let back = pm.utf8_to_utf16(utf8_pos);
            assert_eq!(
                back, i,
                "roundtrip UTF16->UTF8->UTF16: {i} -> {utf8_pos} -> {back}"
            );
        }
    }
}
