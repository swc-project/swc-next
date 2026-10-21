use super::TransformPassOptions;
use crate::{BridgeDecode, bridge};

const OPTIONS_PRESENT: u32 = 1 << 0;
const TYPESCRIPT_SHIFT: u32 = 1;
const TYPESCRIPT_MASK: u32 = 0b11 << TYPESCRIPT_SHIFT;
const STATIC_BLOCKS_SHIFT: u32 = 3;
const STATIC_BLOCKS_MASK: u32 = 0b11 << STATIC_BLOCKS_SHIFT;
const KNOWN_MASK: u32 = OPTIONS_PRESENT | TYPESCRIPT_MASK | STATIC_BLOCKS_MASK;

#[bridge]
impl BridgeDecode for Option<TransformPassOptions> {
    type Wire = u32;

    fn decode(value: Self::Wire) -> Option<Self> {
        if value == 0 {
            return Some(None);
        }
        if value & OPTIONS_PRESENT == 0 || value & !KNOWN_MASK != 0 {
            return None;
        }
        Some(Some(TransformPassOptions {
            typescript: boolean_field(value, TYPESCRIPT_MASK, TYPESCRIPT_SHIFT)?,
            static_blocks: boolean_field(value, STATIC_BLOCKS_MASK, STATIC_BLOCKS_SHIFT)?,
        }))
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
