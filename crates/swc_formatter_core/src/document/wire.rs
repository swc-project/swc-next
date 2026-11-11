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
