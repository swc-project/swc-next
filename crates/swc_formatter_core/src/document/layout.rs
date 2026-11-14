//! Layout modes and candidate selection shared by physical group variants.

use super::{Fragment, GroupRequest};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Flat,
    Break,
}

// The compact representation supplies its contiguous preferred slice directly.
// Complete groups supply a primary followed by their executable alternatives.
// Static iterators retain both fast paths under one selection contract.
#[inline(always)]
pub(super) fn select<'view, 'a>(
    request: GroupRequest,
    inherit_flat: bool,
    primary: impl FnOnce() -> &'view Fragment<'a>,
    fallback: &'view Fragment<'a>,
    mut preferred: impl Iterator<Item = &'view Fragment<'a>>,
    mut fits: impl FnMut(&'view Fragment<'a>) -> bool,
) -> (Fragment<'a>, Mode) {
    let expanded = matches!(request, GroupRequest::Expanded);
    let selected = if inherit_flat {
        Some(primary())
    } else if expanded {
        None
    } else {
        preferred.find(|part| fits(part))
    };
    selected.map_or((*fallback, Mode::Break), |fragment| {
        (*fragment, if expanded { Mode::Break } else { Mode::Flat })
    })
}
