//! Logical operations over immutable instruction fragments.

use rustc_hash::FxHashMap;
use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    ConditionTarget, Doc, FlatFragment, Fragment, FragmentSummary, Group, GroupId, GroupRequest,
    IfBreak, Instruction, Text,
    facts::{Facts, GroupBoundary},
    flat_summary,
};

pub(super) type FragmentKey = (usize, usize);

pub(super) fn key(elements: &[Instruction<'_>]) -> FragmentKey {
    (elements.as_ptr() as usize, elements.len())
}

pub(super) fn remove_lines(doc: Doc<'_>) -> Doc<'_> {
    let root = rewrite(doc.allocator, doc.root, Operation::RemoveLines);
    Doc { root, ..doc }
}

pub(super) fn replace_end_of_line<'a>(doc: Doc<'a>, replacement: Option<Doc<'a>>) -> Doc<'a> {
    let allocator = doc.allocator;
    let mut group_count = doc.group_count;
    let replacement = match replacement {
        Some(replacement) => import_doc(allocator, &replacement, &mut group_count),
        None => summarize_array(allocator, [Instruction::LiteralLine]),
    };
    let original = doc.root;
    let updated = rewrite(
        allocator,
        original,
        Operation::ReplaceEndOfLine(replacement),
    );
    let mut doc = Doc {
        root: updated,
        ..doc
    };
    if key(updated.elements()) != key(original.elements()) {
        doc.group_count = group_count;
    }
    doc
}

pub(super) fn import_doc<'a>(
    allocator: &'a Allocator,
    doc: &Doc<'a>,
    group_count: &mut u32,
) -> Fragment<'a> {
    let next_count = group_count
        .checked_add(doc.group_count)
        .expect("document group count exceeds u32 capacity");
    let imported = if *group_count == 0 || doc.group_count == 0 {
        doc.root
    } else {
        rewrite(allocator, doc.root, Operation::Remap(*group_count))
    };
    *group_count = next_count;
    imported
}

#[derive(Clone, Copy)]
pub(super) enum Operation<'a> {
    RemoveLines,
    ReplaceEndOfLine(Fragment<'a>),
    Remap(u32),
}

pub(super) fn children<'a>(
    element: Instruction<'a>,
    operation: Operation<'a>,
    mut push: impl FnMut(Fragment<'a>),
) {
    match element {
        Instruction::Fragment { elements, summary }
        | Instruction::FlatFragment { elements, summary } => push(Fragment { elements, summary }),
        Instruction::IfBreak(c) | Instruction::FlatIfBreak(c) => {
            push(c.flat);
            if !matches!(operation, Operation::RemoveLines) {
                push(c.broken);
            }
        }
        Instruction::CompactGroup(group) => group.source_fragments().for_each(push),
        Instruction::SourceGroup(group) => group.source_fragments().for_each(push),
        _ => {}
    }
}

enum Work<'a> {
    Enter {
        elements: &'a [Instruction<'a>],
        summary: Option<FragmentSummary>,
    },
    Rewrite {
        elements: &'a [Instruction<'a>],
        summary: Option<FragmentSummary>,
        capacity: u32,
    },
}

fn rewrite<'a>(
    allocator: &'a Allocator,
    root: Fragment<'a>,
    operation: Operation<'a>,
) -> Fragment<'a> {
    let mut mapped = FxHashMap::default();
    let mut nodes = FxHashMap::default();
    let mut texts = FxHashMap::default();
    let mut pending = vec![Work::Enter {
        elements: root.elements(),
        summary: Some(root.summary),
    }];
    while let Some(work) = pending.pop() {
        let (elements, original_summary, capacity) = match work {
            Work::Enter { elements, summary } => (elements, summary, None),
            Work::Rewrite {
                elements,
                summary,
                capacity,
            } => (elements, summary, Some(capacity)),
        };
        let id = key(elements);
        if mapped.contains_key(&id) {
            continue;
        }
        let Some(capacity) = capacity else {
            let ready = pending.len();
            // Root fragments have checked u32 lengths, and every scheduled
            // slice is either another fragment or a subslice of one.
            debug_assert!(u32::try_from(elements.len()).is_ok());
            let capacity = elements.len() as u32;
            pending.push(Work::Rewrite {
                elements,
                summary: original_summary,
                capacity,
            });
            for scope in scopes(elements) {
                if let [element] = scope {
                    children(*element, operation, |child| {
                        pending.push(Work::Enter {
                            elements: child.elements(),
                            summary: Some(child.summary),
                        })
                    });
                } else {
                    // This inline body becomes one shared reference. Carry its
                    // emitted size instead of reserving the whole nested slice.
                    let Work::Rewrite { capacity, .. } = &mut pending[ready] else {
                        unreachable!("the parent rewrite remains scheduled while entering children")
                    };
                    *capacity -= (scope.len() - 1) as u32;
                    pending.push(Work::Enter {
                        elements: scope,
                        summary: None,
                    });
                }
            }
            continue;
        };
        let lookup = |fragment: Fragment<'a>| mapped[&key(fragment.elements())];
        let mut changed = false;
        let mut updated = Vec::with_capacity(capacity as usize);
        let mut ends = Vec::new();
        let mut source_index = 0;
        for scope in scopes(elements) {
            if let Some(&(end, start)) = ends.last()
                && end == source_index
            {
                let distance = u32::try_from(updated.len() - start)
                    .expect("document instruction distance exceeds u32 capacity");
                *end_offset(&mut updated[start]).unwrap() = distance;
                ends.pop();
            }
            if let [element] = scope {
                // These nodes carry identity across distinct fragment views.
                let node = element.group_key();
                let mut map = || map_element(allocator, *element, operation, lookup, &mut texts);
                let next = match node {
                    Some(node) => *nodes.entry(node).or_insert_with(map),
                    None => map(),
                };
                changed |= next.is_some();
                let mut next = next.unwrap_or(*element);
                if let Some(distance) = end_offset(&mut next) {
                    ends.push((source_index + *distance as usize, updated.len()));
                }
                updated.push(next);
            } else {
                let fragment = mapped[&key(scope)];
                changed |= key(scope) != key(fragment.elements());
                updated.push(Instruction::fragment(fragment));
            }
            source_index += scope.len();
        }
        // Immutable scopes retain valid facts when unchanged. Renaming group
        // identities also preserves their capabilities and cached measurements.
        let retained =
            original_summary.filter(|_| !changed || matches!(operation, Operation::Remap(_)));
        let summary = retained.unwrap_or_else(|| summarize_elements(&mut updated));
        let result = if changed {
            store(allocator, updated, summary)
        } else {
            Fragment::new(elements, summary)
        };
        mapped.insert(id, result);
    }
    mapped[&key(root.elements())]
}

fn end_offset<'e>(element: &'e mut Instruction<'_>) -> Option<&'e mut u32> {
    match element {
        Instruction::StartGroup { end_offset, .. }
        | Instruction::StartGroupWithFlatWidth { end_offset, .. }
        | Instruction::StartFillPart { end_offset }
        | Instruction::StartLineSuffix { end_offset }
        | Instruction::StartIndentIfBreak { end_offset, .. } => Some(end_offset),
        _ => None,
    }
}

// Canonicalize complete inline groups so overlapping fragment views share the
// same rewritten group. The enclosing group itself stays inline in its scope.
fn scopes<'a>(elements: &'a [Instruction<'a>]) -> impl Iterator<Item = &'a [Instruction<'a>]> {
    let mut index = 0;
    std::iter::from_fn(move || {
        let start = index;
        let element = elements.get(start)?;
        index += 1;
        if let Instruction::StartGroup { end_offset, .. }
        | Instruction::StartGroupWithFlatWidth { end_offset, .. } = element
        {
            let end = start + *end_offset as usize + 1;
            if start != 0 || end != elements.len() {
                index = end;
            }
        }
        Some(&elements[start..index])
    })
}

fn same(left: Fragment<'_>, right: Fragment<'_>) -> bool {
    key(left.elements()) == key(right.elements())
}

pub(super) fn map_element<'a>(
    allocator: &'a Allocator,
    element: Instruction<'a>,
    operation: Operation<'a>,
    lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
    texts: &mut FxHashMap<&'a str, Fragment<'a>>,
) -> Option<Instruction<'a>> {
    use Instruction as I;

    if matches!(operation, Operation::RemoveLines) {
        match element {
            I::NormalLine | I::NormalLiteralLine => return Some(I::text(Text::new(" "))),
            I::SoftLine
            | I::SoftLiteralLine
            | I::SoftLineIfGroupBreaks(_)
            | I::TextIfBreak { .. } => {
                return Some(I::text(Text::new("")));
            }
            I::TextIfFlat { source, width } => return Some(I::Text { source, width }),
            I::IfBreak(c) => return Some(I::fragment(lookup(c.flat))),
            I::FlatIfBreak(c) => return Some(I::flat_fragment(FlatFragment(lookup(c.flat)))),
            _ => {}
        }
    }

    let remap_id = |id: GroupId| match operation {
        Operation::Remap(offset) if offset != 0 => Some(GroupId(
            id.0.checked_add(offset)
                .expect("document group ID exceeds u32 capacity"),
        )),
        _ => None,
    };
    let remap_target = |target| match target {
        ConditionTarget::Current => None,
        ConditionTarget::Group(id) => remap_id(id).map(ConditionTarget::Group),
    };

    match element {
        I::Text { source, .. } | I::TextIfFlat { source, .. } | I::TextIfBreak { source, .. } => {
            let Operation::ReplaceEndOfLine(replacement) = operation else {
                return None;
            };
            let source = source.as_str();
            if !source.contains('\n') {
                return None;
            }
            let fragment = *texts.entry(source).or_insert_with(|| {
                let mut lines = source.split('\n');
                let mut elements = vec![I::text(Text::new(lines.next().unwrap()))];
                for line in lines {
                    elements.push(I::fragment(replacement));
                    elements.push(I::text(Text::new(line)));
                }
                summarize(allocator, elements)
            });
            if matches!(element, I::Text { .. }) {
                return Some(I::fragment(fragment));
            }
            let empty = summarize(allocator, Vec::new());
            let (broken, flat) = if matches!(element, I::TextIfBreak { .. }) {
                (fragment, empty)
            } else {
                (empty, fragment)
            };
            Some(I::IfBreak(allocator.alloc(IfBreak {
                target: ConditionTarget::Current,
                broken,
                flat,
            })))
        }
        I::Fragment { elements, summary } | I::FlatFragment { elements, summary } => {
            let original = Fragment { elements, summary };
            let updated = lookup(original);
            if same(original, updated) {
                None
            } else if matches!(element, I::FlatFragment { .. }) {
                Some(I::flat_fragment(FlatFragment(updated)))
            } else {
                Some(I::fragment(updated))
            }
        }
        I::IfBreak(c) | I::FlatIfBreak(c) => {
            let broken = lookup(c.broken);
            let flat = lookup(c.flat);
            let target = remap_target(c.target);
            if target.is_none() && same(c.broken, broken) && same(c.flat, flat) {
                return None;
            }
            let updated = allocator.alloc(IfBreak {
                target: target.unwrap_or(c.target),
                broken,
                flat,
            });
            Some(if matches!(element, I::FlatIfBreak(_)) {
                I::FlatIfBreak(updated)
            } else {
                I::IfBreak(updated)
            })
        }
        I::CompactGroup(group) => group.map_sources(allocator, lookup, remap_id),
        I::SourceGroup(group) => group.map_sources(allocator, lookup, remap_id),
        I::StartGroup {
            request,
            effect,
            id: Some(id),
            end_offset,
        } => remap_id(id).map(|id| I::StartGroup {
            request,
            effect,
            id: Some(id),
            end_offset,
        }),
        I::SoftLineIfGroupBreaks(id) => remap_id(id).map(I::SoftLineIfGroupBreaks),
        I::StartIndentIfBreak {
            target,
            negate,
            end_offset,
        } => remap_target(target).map(|target| I::StartIndentIfBreak {
            target,
            negate,
            end_offset,
        }),
        _ => None,
    }
}

// Recompute cached facts once for a rewritten scope.
pub(super) fn summarize<'a>(
    allocator: &'a Allocator,
    mut elements: Vec<Instruction<'a>>,
) -> Fragment<'a> {
    let summary = summarize_elements(&mut elements);
    store(allocator, elements, summary)
}

/// Construct fixed instruction sequences directly in the arena.
pub(super) fn summarize_array<'a, const N: usize>(
    allocator: &'a Allocator,
    mut elements: [Instruction<'a>; N],
) -> Fragment<'a> {
    let summary = summarize_elements(&mut elements);
    Fragment::new(allocator.alloc(elements), summary)
}

#[inline(always)]
fn store<'a>(
    allocator: &'a Allocator,
    elements: Vec<Instruction<'a>>,
    summary: FragmentSummary,
) -> Fragment<'a> {
    let mut stored = ArenaVec::with_capacity_in(elements.len(), allocator);
    stored.extend(elements);
    Fragment::new(stored.into_bump_slice(), summary)
}

fn summarize_elements(elements: &mut [Instruction<'_>]) -> FragmentSummary {
    use Instruction as I;

    let mut facts = Facts::default();
    let mut groups = Vec::new();
    for index in 0..elements.len() {
        match facts.observe(elements[index]) {
            GroupBoundary::Start => {
                groups.push((index, facts.begin_group()));
            }
            GroupBoundary::End => {
                let (start, outer) = groups.pop().expect("group scopes are balanced");
                match &mut elements[start] {
                    I::StartGroup {
                        request, effect, ..
                    } => {
                        *effect = facts.end_group(*request, outer);
                    }
                    I::StartGroupWithFlatWidth { end_offset, .. } => {
                        let effect = facts.end_group(GroupRequest::Auto, outer);
                        let end_offset = *end_offset;
                        elements[start] = match flat_summary(&elements[start + 1..index]) {
                            Some(summary) => I::StartGroupWithFlatWidth {
                                summary,
                                end_offset,
                            },
                            None => I::StartGroup {
                                request: GroupRequest::Auto,
                                effect,
                                id: None,
                                end_offset,
                            },
                        };
                    }
                    _ => unreachable!(),
                }
            }
            GroupBoundary::None => {}
        }
    }
    facts.finish(elements)
}
