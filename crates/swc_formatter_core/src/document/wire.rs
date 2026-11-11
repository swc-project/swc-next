//! Version-0 identifiers belong to the transport, not to native scope kinds.

use super::RichIndentKind;

pub(super) fn rich_indent_tag(kind: RichIndentKind) -> u8 {
    match kind {
        RichIndentKind::AlignText => 23,
        RichIndentKind::Dedent => 24,
        RichIndentKind::MarkRoot => 25,
        RichIndentKind::DedentToRoot => 26,
    }
}

pub(super) fn rich_indent_kind(tag: u8) -> Option<RichIndentKind> {
    match tag {
        23 => Some(RichIndentKind::AlignText),
        24 => Some(RichIndentKind::Dedent),
        25 => Some(RichIndentKind::MarkRoot),
        26 => Some(RichIndentKind::DedentToRoot),
        _ => None,
    }
}
