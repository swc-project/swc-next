//! Width-independent logical queries over the primary contents of a flat DAG.

use rustc_hash::FxHashSet;

use super::{Fragment, FragmentCapabilities, Group, GroupRequest, Instruction as I, Text};

fn is_line(element: I<'_>) -> bool {
    matches!(
        element,
        I::SoftLine
            | I::SoftLineIfGroupBreaks(_)
            | I::NormalLine
            | I::HardLine
            | I::HardLineOnly
            | I::LiteralLine
            | I::LiteralLineOnly
            | I::LiteralLineAtCurrentIndent
            | I::SoftLiteralLine
            | I::NormalLiteralLine
    )
}

pub(super) fn can_break(root: Fragment<'_>) -> bool {
    any_in_primary_contents(root, is_line)
}

pub(super) fn has_line_breaks(root: Fragment<'_>) -> bool {
    any_in_primary_contents(root, |element| {
        is_line(element)
            || match element {
                I::Text { source, width }
                | I::TextIfBreak { source, width }
                | I::TextIfFlat { source, width } => {
                    !Text::from_parts(source, width).is_single_line()
                }
                _ => false,
            }
    })
}

pub(super) fn will_break(root: Fragment<'_>) -> bool {
    any_in_primary_contents(root, |element| match element {
        I::BreakParent
        | I::HardLine
        | I::HardLineOnly
        | I::LiteralLine
        | I::LiteralLineOnly
        | I::LiteralLineAtCurrentIndent
        | I::StartGroup {
            request: GroupRequest::Expanded,
            ..
        } => true,
        I::SourceGroup(group) => matches!(group.request(), GroupRequest::Expanded),
        _ => false,
    })
}

enum Scan<'a> {
    Found,
    Descend {
        remaining: &'a [I<'a>],
        element: I<'a>,
    },
    End,
}

// Keep graph bookkeeping out of the flat scan's register allocation.
#[inline(never)]
fn scan<'a, const FLAT: bool>(
    elements: &'a [I<'a>],
    predicate: &impl Fn(I<'a>) -> bool,
) -> Scan<'a> {
    let mut elements = elements.iter();
    while let Some(&source) = elements.next() {
        let element = if FLAT {
            let Some(element) = source.flattened() else {
                continue;
            };
            element
        } else {
            source
        };
        if predicate(element) {
            return Scan::Found;
        }
        if matches!(
            element,
            I::Fragment { .. }
                | I::FlatFragment { .. }
                | I::IfBreak(_)
                | I::FlatIfBreak(_)
                | I::CompactGroup(_)
                | I::SourceGroup(_)
        ) {
            return Scan::Descend {
                remaining: elements.as_slice(),
                element,
            };
        }
    }
    Scan::End
}

fn any_in_primary_contents<'a>(root: Fragment<'a>, predicate: impl Fn(I<'a>) -> bool) -> bool {
    if matches!(root.capabilities(), FragmentCapabilities::Neutral) {
        return false;
    }
    let mut pending = Vec::new();
    let mut visited = FxHashSet::default();
    let mut current = (root.elements(), false);
    loop {
        let (elements, flat) = current;
        let scanned = if flat {
            scan::<true>(elements, &predicate)
        } else {
            scan::<false>(elements, &predicate)
        };
        let resume = match scanned {
            Scan::Found => return true,
            Scan::End => None,
            Scan::Descend { remaining, element } => {
                let mut child = None;
                let mut enter = |fragment: Fragment<'a>, child_flat| {
                    if matches!(fragment.capabilities(), FragmentCapabilities::Neutral) {
                        return;
                    }
                    let next = fragment.elements();
                    if !visited.insert((next.as_ptr() as usize, next.len(), child_flat)) {
                        return;
                    }
                    if let Some(previous) = child.replace((next, child_flat)) {
                        pending.push(previous);
                    } else if !remaining.is_empty() {
                        pending.push((remaining, flat));
                    }
                };
                match element {
                    I::Fragment { elements, summary } => {
                        enter(Fragment { elements, summary }, false)
                    }
                    I::FlatFragment { elements, summary } => {
                        enter(Fragment { elements, summary }, true)
                    }
                    I::IfBreak(c) | I::FlatIfBreak(c) => {
                        let flat = matches!(element, I::FlatIfBreak(_));
                        enter(c.broken, flat);
                        enter(c.flat, flat);
                    }
                    I::CompactGroup(group) => enter(group.primary(), false),
                    I::SourceGroup(group) => enter(group.primary(), false),
                    _ => unreachable!("the scan only yields child-bearing instructions"),
                }
                child.or_else(|| (!remaining.is_empty()).then_some((remaining, flat)))
            }
        };
        let Some(next) = resume.or_else(|| pending.pop()) else {
            return false;
        };
        current = next;
    }
}
