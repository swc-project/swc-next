use super::TransformPassOptions;
use crate::{BridgeDecode, bridge};

const OPTIONS_PRESENT: u32 = 1 << 0;
const TYPESCRIPT_SHIFT: u32 = 1;
const TYPESCRIPT_MASK: u32 = 0b11 << TYPESCRIPT_SHIFT;
const STATIC_BLOCKS_SHIFT: u32 = 3;
const STATIC_BLOCKS_MASK: u32 = 0b11 << STATIC_BLOCKS_SHIFT;
const VERBATIM_MODULE_SYNTAX_SHIFT: u32 = 5;
const VERBATIM_MODULE_SYNTAX_MASK: u32 = 0b11 << VERBATIM_MODULE_SYNTAX_SHIFT;
const PRESERVE_CONST_ENUMS_SHIFT: u32 = 7;
const PRESERVE_CONST_ENUMS_MASK: u32 = 0b11 << PRESERVE_CONST_ENUMS_SHIFT;
const KNOWN_MASK: u32 = OPTIONS_PRESENT
    | TYPESCRIPT_MASK
    | STATIC_BLOCKS_MASK
    | VERBATIM_MODULE_SYNTAX_MASK
    | PRESERVE_CONST_ENUMS_MASK;

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
            verbatim_module_syntax: boolean_field(
                value,
                VERBATIM_MODULE_SYNTAX_MASK,
                VERBATIM_MODULE_SYNTAX_SHIFT,
            )?,
            preserve_const_enums: boolean_field(
                value,
                PRESERVE_CONST_ENUMS_MASK,
                PRESERVE_CONST_ENUMS_SHIFT,
            )?,
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
