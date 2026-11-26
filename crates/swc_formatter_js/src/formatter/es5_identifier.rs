//! ES5.1 identifier restrictions for removing property key quotes.

use swc_next_ecma_ast::{
    is_identifier_continue_es5_and_esnext, is_identifier_start_es5_and_esnext,
};

#[path = "es5_identifier_tables.rs"]
mod tables;

use tables::{EXCLUDED_CONTINUE, EXCLUDED_START_ONLY};

pub(super) fn is_es5_identifier_name(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(|ch| {
        // The continuation exclusions also apply to identifier starts.
        is_identifier_start_es5_and_esnext(ch)
            && (ch.is_ascii()
                || (!contains(EXCLUDED_CONTINUE, ch) && !contains(EXCLUDED_START_ONLY, ch)))
    }) && chars.all(|ch| {
        is_identifier_continue_es5_and_esnext(ch)
            && (ch.is_ascii() || !contains(EXCLUDED_CONTINUE, ch))
    })
}

fn contains(boundaries: &[u16], ch: char) -> bool {
    // The lexer check rejects supplementary characters before this lookup.
    let point = ch as u16;
    boundaries.partition_point(|&boundary| boundary <= point) % 2 == 1
}
