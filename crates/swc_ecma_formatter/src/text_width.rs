//! Display width follows Prettier's string metric, independently of line layout.
//!
//! Only emoji sequences are composed. Other scripts retain scalar widths;
//! nonspacing/enclosing marks are zero, spacing marks and format characters are
//! not. The pinned source data and upstream licenses live beside this module.

mod data;

use std::str::Chars;

struct EmojiState {
    first: u16,
    count: u16,
    width: u8,
}

struct EmojiEdge {
    first: u32,
    last: u32,
    next: u16,
}

pub(crate) fn ascii_width(source: &str) -> Option<usize> {
    source
        .bytes()
        .all(|byte| (0x20..=0x7f).contains(&byte))
        .then_some(source.len())
}

pub(crate) fn width(source: &str) -> usize {
    if let Some(width) = ascii_width(source) {
        return width;
    }
    unicode_width(source)
}

/// The Unicode branch is retained when a whole Text containing controls is
/// measured in spans. Reselecting the ASCII shortcut for a span would count DEL.
pub(crate) fn unicode_width(source: &str) -> usize {
    if source.is_ascii() {
        return source
            .bytes()
            .filter(|byte| (0x20..0x7f).contains(byte))
            .count();
    }
    let mut characters = source.chars();
    let mut width = 0;
    while let Some(character) = characters.next() {
        if let Some(state) = emoji_next(&data::EMOJI_STATES[data::EMOJI_START], character)
            && let Some((emoji_width, remaining)) = emoji_prefix(state, characters.clone())
        {
            width += emoji_width;
            characters = remaining;
        } else {
            width += scalar_width(character);
        }
    }
    width
}

fn emoji_prefix<'a>(
    mut state: &EmojiState,
    mut characters: Chars<'a>,
) -> Option<(usize, Chars<'a>)> {
    let mut accepted = (state.width != 0).then(|| (usize::from(state.width), characters.clone()));
    while state.count != 0 {
        let Some(character) = characters.next() else {
            break;
        };
        let Some(next) = emoji_next(state, character) else {
            break;
        };
        state = next;
        if state.width != 0 {
            accepted = Some((usize::from(state.width), characters.clone()));
        }
    }
    accepted
}

fn emoji_next(state: &EmojiState, character: char) -> Option<&'static EmojiState> {
    let first = usize::from(state.first);
    let edges = &data::EMOJI_EDGES[first..first + usize::from(state.count)];
    let point = u32::from(character);
    let index = edges.partition_point(|edge| edge.last < point);
    let edge = edges.get(index)?;
    (edge.first <= point).then(|| &data::EMOJI_STATES[usize::from(edge.next)])
}

fn scalar_width(character: char) -> usize {
    let point = u32::from(character);
    if point <= 0x1f || (0x7f..=0x9f).contains(&point) || in_ranges(point, &data::ZERO_WIDTH_MARKS)
    {
        0
    } else if in_ranges(point, &data::WIDE) {
        2
    } else {
        1
    }
}

fn in_ranges(point: u32, ranges: &[(u32, u32)]) -> bool {
    let index = ranges.partition_point(|&(_, last)| last < point);
    ranges.get(index).is_some_and(|&(first, _)| first <= point)
}

#[cfg(test)]
mod tests {
    use super::width;

    #[test]
    fn ascii_and_controls() {
        assert_eq!(width("ASCII"), 5);
        assert_eq!(width("a\u{7f}b"), 3);
        assert_eq!(width("é\u{7f}b"), 2);
        assert_eq!(width("a\u{7f}\tb"), 2);
        assert_eq!(width("a\r\nb"), 2);
        assert_eq!(width("a\u{2028}b"), 3);
    }

    #[test]
    fn scalar_categories() {
        assert_eq!(width("e\u{301}"), 1);
        assert_eq!(width("a\u{488}"), 1);
        assert_eq!(width("a\u{93e}"), 2);
        assert_eq!(width("a\u{200d}b"), 3);
        assert_eq!(width("a\u{200c}b"), 3);
        assert_eq!(width("a\u{ad}b"), 3);
        assert_eq!(width("\u{e0020}"), 1);
    }

    #[test]
    fn script_and_east_asian_width() {
        assert_eq!(width("لا"), 2);
        assert_eq!(width("א\u{200d}ל"), 3);
        assert_eq!(width("\u{115f}\u{1160}"), 3);
        assert_eq!(width("\u{17d8}"), 1);
        assert_eq!(width("世界"), 4);
        assert_eq!(width("Ω"), 1);
        assert_eq!(width("Ｘ"), 2);
    }

    #[test]
    fn emoji_sequences() {
        assert_eq!(width("©"), 1);
        assert_eq!(width("©\u{fe0f}"), 2);
        assert_eq!(width("©\u{fe0e}"), 1);
        assert_eq!(width("😀\u{fe0e}"), 2);
        assert_eq!(width("👩\u{200d}💻"), 2);
        assert_eq!(width("👩\u{200d}中"), 5);
        assert_eq!(width("👍🏽"), 2);
        assert_eq!(width("🇺🇸🇺"), 3);
        assert_eq!(width("1\u{20e3}"), 2);
        assert_eq!(width("1\u{fe0f}\u{20e3}"), 2);
        assert_eq!(width("❤\u{fe0f}\u{200d}🔥"), 2);
        assert_eq!(
            width("🏴\u{e0067}\u{e0062}\u{e0073}\u{e0063}\u{e0074}\u{e007f}"),
            2
        );
    }
}
