use super::CodegenOptions;
use crate::{BridgeDecode, bridge};

const OPTIONS_PRESENT: u32 = 1 << 0;

const ASCII_ONLY_SHIFT: u32 = 1;
const ASCII_ONLY_MASK: u32 = 0b11 << ASCII_ONLY_SHIFT;

const MINIFY_SHIFT: u32 = 3;
const MINIFY_MASK: u32 = 0b11 << MINIFY_SHIFT;

const KNOWN_MASK: u32 = OPTIONS_PRESENT | ASCII_ONLY_MASK | MINIFY_MASK;

#[bridge]
impl BridgeDecode for Option<CodegenOptions> {
    type Wire = u32;

    fn decode(value: Self::Wire) -> Option<Self> {
        if value == 0 {
            return Some(None);
        }
        if value & OPTIONS_PRESENT == 0 || value & !KNOWN_MASK != 0 {
            return None;
        }

        let ascii_only = boolean_field(value, ASCII_ONLY_MASK, ASCII_ONLY_SHIFT)?;
        let minify = boolean_field(value, MINIFY_MASK, MINIFY_SHIFT)?;

        Some(Some(CodegenOptions { ascii_only, minify }))
    }
}

fn boolean_field(value: u32, mask: u32, shift: u32) -> Option<Option<bool>> {
    match (value & mask) >> shift {
        0 => Some(None),
        1 => Some(Some(false)),
        2 => Some(Some(true)),
        _ => None,
    }
}
