use super::{CommentMode, Lang, ParserOptions, SourceType};
use crate::{BridgeDecode, bridge};

const OPTIONS_PRESENT: u32 = 1 << 0;

const SOURCE_TYPE_SHIFT: u32 = 1;
const SOURCE_TYPE_MASK: u32 = 0b111 << SOURCE_TYPE_SHIFT;

const LANG_SHIFT: u32 = 4;
const LANG_MASK: u32 = 0b111 << LANG_SHIFT;

const PRESERVE_PARENS_SHIFT: u32 = 7;
const PRESERVE_PARENS_MASK: u32 = 0b11 << PRESERVE_PARENS_SHIFT;

const COMMENTS_SHIFT: u32 = 9;
const COMMENTS_MASK: u32 = 0b11 << COMMENTS_SHIFT;

const KNOWN_MASK: u32 =
    OPTIONS_PRESENT | SOURCE_TYPE_MASK | LANG_MASK | PRESERVE_PARENS_MASK | COMMENTS_MASK;

#[bridge]
impl BridgeDecode for Option<ParserOptions> {
    type Wire = u32;

    fn decode(value: Self::Wire) -> Option<Self> {
        if value == 0 {
            return Some(None);
        }
        if value & OPTIONS_PRESENT == 0 || value & !KNOWN_MASK != 0 {
            return None;
        }

        let source_type = match field(value, SOURCE_TYPE_MASK, SOURCE_TYPE_SHIFT) {
            0 => None,
            1 => Some(SourceType::Module),
            2 => Some(SourceType::Script),
            3 => Some(SourceType::CommonJs),
            4 => Some(SourceType::Unambiguous),
            _ => return None,
        };
        let lang = match field(value, LANG_MASK, LANG_SHIFT) {
            0 => None,
            1 => Some(Lang::Js),
            2 => Some(Lang::Ts),
            3 => Some(Lang::Jsx),
            4 => Some(Lang::Tsx),
            5 => Some(Lang::Dts),
            _ => return None,
        };
        let preserve_parens = match field(value, PRESERVE_PARENS_MASK, PRESERVE_PARENS_SHIFT) {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            _ => return None,
        };
        let comments = match field(value, COMMENTS_MASK, COMMENTS_SHIFT) {
            0 => None,
            1 => Some(CommentMode::None),
            2 => Some(CommentMode::Flat),
            _ => return None,
        };

        Some(Some(ParserOptions {
            source_type,
            lang,
            preserve_parens,
            comments,
        }))
    }
}

fn field(value: u32, mask: u32, shift: u32) -> u32 {
    (value & mask) >> shift
}
