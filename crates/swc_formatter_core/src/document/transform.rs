//! Logical operations over immutable instruction fragments.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    Alternatives, BreakEffect, ConditionTarget, Conditional, Doc, FlatFragment, FormatElement,
    Fragment, FragmentCapabilities, GroupId, GroupRequest, GroupWithStates, Text, flat_summary,
};

pub(super) type FragmentKey = (usize, usize);

pub(super) fn key(elements: &[FormatElement<'_>]) -> FragmentKey {
    (elements.as_ptr() as usize, elements.len())
}

pub(super) fn will_break<'a>(root: &'a [FormatElement<'a>]) -> bool {
    let mut visited = FxHashSet::default();
    let mut pending = vec![(root, false)];
    while let Some((elements, flat)) = pending.pop() {
        if !visited.insert((key(elements), flat)) {
            continue;
        }
        for &element in elements {
            let element = if flat {
                let Some(element) = element.flattened() else {
                    continue;
                };
                element
            } else {
                element
            };
            match element {
                FormatElement::BreakParent
                | FormatElement::HardLine
                | FormatElement::HardLineWithoutBreakParent
                | FormatElement::LiteralLine
                | FormatElement::LiteralLineWithoutBreakParent
                | FormatElement::LiteralLineWithIndent
                | FormatElement::StartGroup {
                    request: GroupRequest::Expanded,
                    ..
                } => return true,
                FormatElement::Fragment { elements, .. } => pending.push((elements.get(), false)),
                FormatElement::FlatFragment { elements, .. } => {
                    pending.push((elements.get(), true))
                }
                FormatElement::Conditional(c) | FormatElement::FlatConditional(c) => {
                    let flat = matches!(element, FormatElement::FlatConditional(_));
                    pending.push((c.broken.elements(), flat));
                    pending.push((c.flat.elements(), flat));
                }
                FormatElement::Alternatives(a) => pending.push((a.first().elements(), false)),
                FormatElement::GroupWithStates(group) => {
                    if matches!(group.request, GroupRequest::Expanded) {
                        return true;
                    }
                    pending.push((group.contents.elements(), false));
                }
                _ => {}
            }
        }
    }
    false
}

pub(super) fn remove_lines(doc: Doc<'_>) -> Doc<'_> {
    let root = rewrite(doc.allocator, root(&doc), Operation::RemoveLines);
    with_root(doc, root)
}

pub(super) fn replace_end_of_line<'a>(doc: Doc<'a>, replacement: Option<Doc<'a>>) -> Doc<'a> {
    let allocator = doc.allocator;
    let (replacement, additional_groups) = match replacement {
        Some(replacement) => {
            let original = root(&replacement);
            let updated = rewrite(allocator, original, Operation::Remap(doc.group_count));
            let imported = if same(original, updated) {
                summarize(allocator, updated.elements().to_vec())
            } else {
                updated
            };
            (imported, replacement.group_count)
        }
        None => (summarize(allocator, vec![FormatElement::LiteralLine]), 0),
    };
    let original = root(&doc);
    let updated = rewrite(
        allocator,
        original,
        Operation::ReplaceEndOfLine(replacement),
    );
    let mut doc = with_root(doc, updated);
    if key(updated.elements()) != key(original.elements()) {
        doc.group_count = doc
            .group_count
            .checked_add(additional_groups)
            .expect("document group count exceeds u32 capacity");
    }
    doc
}

pub(super) fn root<'a>(doc: &Doc<'a>) -> Fragment<'a> {
    // Only the root's printer flags are needed if the root remains unchanged.
    Fragment::new(
        doc.elements,
        BreakEffect::default(),
        FragmentCapabilities::General,
        false,
        false,
        doc.group_states,
        doc.rich_indent,
    )
}

pub(super) fn with_root<'a>(doc: Doc<'a>, fragment: Fragment<'a>) -> Doc<'a> {
    Doc {
        elements: fragment.elements(),
        group_states: fragment.group_states(),
        rich_indent: fragment.rich_indent(),
        extended_print: true,
        ..doc
    }
}

#[derive(Clone, Copy)]
pub(super) enum Operation<'a> {
    RemoveLines,
    ReplaceEndOfLine(Fragment<'a>),
    Remap(u32),
    Clean,
}

pub(super) fn children<'a>(
    element: FormatElement<'a>,
    operation: Operation<'a>,
    mut push: impl FnMut(Fragment<'a>),
) {
    match element {
        FormatElement::Fragment { elements, summary }
        | FormatElement::FlatFragment { elements, summary } => push(Fragment { elements, summary }),
        FormatElement::Conditional(c) | FormatElement::FlatConditional(c) => {
            push(c.flat);
            if !matches!(operation, Operation::RemoveLines) {
                push(c.broken);
            }
        }
        FormatElement::Alternatives(a) => {
            push(a.fallback);
            a.preferred.iter().copied().for_each(push);
        }
        FormatElement::GroupWithStates(group) => {
            push(group.contents);
            group.states.iter().copied().for_each(push);
        }
        _ => {}
    }
}

fn rewrite<'a>(
    allocator: &'a Allocator,
    root: Fragment<'a>,
    operation: Operation<'a>,
) -> Fragment<'a> {
    let mut mapped = FxHashMap::default();
    let mut texts = FxHashMap::default();
    let mut pending = vec![(root, false)];
    while let Some((fragment, ready)) = pending.pop() {
        let elements = fragment.elements();
        let id = key(elements);
        if mapped.contains_key(&id) {
            continue;
        }
        if !ready {
            pending.push((fragment, true));
            for &element in elements {
                children(element, operation, |child| pending.push((child, false)));
            }
            continue;
        }
        let lookup = |fragment: Fragment<'a>| mapped[&key(fragment.elements())];
        let mut changed = None::<Vec<_>>;
        for (index, &element) in elements.iter().enumerate() {
            let next = map_element(allocator, element, operation, lookup, &mut texts);
            if let Some(next) = next {
                changed
                    .get_or_insert_with(|| elements[..index].to_vec())
                    .push(next);
            } else if let Some(changed) = &mut changed {
                changed.push(element);
            }
        }
        let result = changed.map_or(fragment, |changed| summarize(allocator, changed));
        mapped.insert(id, result);
    }
    mapped[&key(root.elements())]
}

fn same(left: Fragment<'_>, right: Fragment<'_>) -> bool {
    key(left.elements()) == key(right.elements())
}

pub(super) fn map_element<'a>(
    allocator: &'a Allocator,
    element: FormatElement<'a>,
    operation: Operation<'a>,
    lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
    texts: &mut FxHashMap<&'a str, Fragment<'a>>,
) -> Option<FormatElement<'a>> {
    use FormatElement as E;

    if matches!(operation, Operation::RemoveLines) {
        match element {
            E::Line | E::NormalLiteralLine => return Some(E::text(Text::new(" "))),
            E::SoftLine | E::SoftLiteralLine | E::SoftLineIfBreak(_) | E::IfBreak { .. } => {
                return Some(E::text(Text::new("")));
            }
            E::IfFlat { source, width } => return Some(E::Text { source, width }),
            E::Conditional(c) => return Some(E::fragment(lookup(c.flat))),
            E::FlatConditional(c) => return Some(E::flat_fragment(FlatFragment(lookup(c.flat)))),
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
        E::Text { source, .. } | E::IfFlat { source, .. } | E::IfBreak { source, .. } => {
            let Operation::ReplaceEndOfLine(replacement) = operation else {
                return None;
            };
            let source = source.as_str();
            if !source.contains('\n') {
                return None;
            }
            let fragment = *texts.entry(source).or_insert_with(|| {
                let mut lines = source.split('\n');
                let mut elements = vec![E::text(Text::new(lines.next().unwrap()))];
                for line in lines {
                    elements.push(E::fragment(replacement));
                    elements.push(E::text(Text::new(line)));
                }
                summarize(allocator, elements)
            });
            if matches!(element, E::Text { .. }) {
                return Some(E::fragment(fragment));
            }
            let empty = summarize(allocator, Vec::new());
            let (broken, flat) = if matches!(element, E::IfBreak { .. }) {
                (fragment, empty)
            } else {
                (empty, fragment)
            };
            Some(E::Conditional(allocator.alloc(Conditional {
                target: ConditionTarget::Current,
                broken,
                flat,
            })))
        }
        E::Fragment { elements, summary } | E::FlatFragment { elements, summary } => {
            let original = Fragment { elements, summary };
            let updated = lookup(original);
            if same(original, updated) {
                None
            } else if matches!(element, E::FlatFragment { .. }) {
                Some(E::flat_fragment(FlatFragment(updated)))
            } else {
                Some(E::fragment(updated))
            }
        }
        E::Conditional(c) | E::FlatConditional(c) => {
            let broken = lookup(c.broken);
            let flat = lookup(c.flat);
            if matches!(operation, Operation::Clean)
                && broken.elements().is_empty()
                && flat.elements().is_empty()
            {
                return Some(E::text(Text::new("")));
            }
            let target = remap_target(c.target);
            if target.is_none() && same(c.broken, broken) && same(c.flat, flat) {
                return None;
            }
            let updated = allocator.alloc(Conditional {
                target: target.unwrap_or(c.target),
                broken,
                flat,
            });
            Some(if matches!(element, E::FlatConditional(_)) {
                E::FlatConditional(updated)
            } else {
                E::Conditional(updated)
            })
        }
        E::GroupWithStates(group) => {
            let contents = lookup(group.contents);
            let id = group.id.and_then(remap_id);
            if id.is_none()
                && same(group.contents, contents)
                && group
                    .states
                    .iter()
                    .copied()
                    .all(|state| same(state, lookup(state)))
            {
                return None;
            }
            Some(E::GroupWithStates(allocator.alloc(GroupWithStates {
                contents,
                states: map_slice(allocator, group.states, lookup),
                request: group.request,
                id: id.or(group.id),
            })))
        }
        E::Alternatives(a) => {
            let fallback = lookup(a.fallback);
            if same(a.fallback, fallback)
                && a.preferred
                    .iter()
                    .copied()
                    .all(|state| same(state, lookup(state)))
            {
                return None;
            }
            Some(E::Alternatives(allocator.alloc(Alternatives {
                preferred: map_slice(allocator, a.preferred, lookup),
                fallback,
            })))
        }
        E::StartGroup {
            request,
            effect,
            id: Some(id),
            end_offset,
        } => remap_id(id).map(|id| E::StartGroup {
            request,
            effect,
            id: Some(id),
            end_offset,
        }),
        E::SoftLineIfBreak(id) => remap_id(id).map(E::SoftLineIfBreak),
        E::StartIndentIfBreak {
            target,
            negate,
            end_offset,
        } => remap_target(target).map(|target| E::StartIndentIfBreak {
            target,
            negate,
            end_offset,
        }),
        _ => None,
    }
}

fn map_slice<'a>(
    allocator: &'a Allocator,
    states: &'a [Fragment<'a>],
    lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
) -> &'a [Fragment<'a>] {
    let mut mapped = ArenaVec::with_capacity_in(states.len(), allocator);
    mapped.extend(states.iter().copied().map(lookup));
    mapped.into_bump_slice()
}

#[derive(Default)]
struct Facts {
    effect: BreakEffect,
    conditional: bool,
    intrinsic: bool,
    states: bool,
    rich_indent: bool,
}

impl Facts {
    fn include(&mut self, fragment: Fragment<'_>) {
        self.conditional |= fragment.conditional_lines();
        self.states |= fragment.group_states();
        self.rich_indent |= fragment.rich_indent();
    }
}

// Rewrite preserves each instruction's position within its scope. Recompute
// cached facts once for a changed slice, including nested inline groups.
pub(super) fn summarize<'a>(
    allocator: &'a Allocator,
    mut elements: Vec<FormatElement<'a>>,
) -> Fragment<'a> {
    use FormatElement as E;

    let mut facts = Facts::default();
    let mut groups = Vec::new();
    for index in 0..elements.len() {
        match elements[index] {
            E::StartGroup { .. } | E::StartGroupWithFlatWidth { .. } => {
                groups.push((index, std::mem::take(&mut facts.effect)));
            }
            E::EndGroup => {
                let (start, outer) = groups.pop().expect("group scopes are balanced");
                match &mut elements[start] {
                    E::StartGroup {
                        request, effect, ..
                    } => {
                        *effect = BreakEffect::from_request(*request).join(facts.effect);
                        facts.effect = *effect;
                    }
                    E::StartGroupWithFlatWidth { end_offset, .. } => {
                        let end_offset = *end_offset;
                        elements[start] = match flat_summary(&elements[start + 1..index]) {
                            Some(summary) => E::StartGroupWithFlatWidth {
                                summary,
                                end_offset,
                            },
                            None => E::StartGroup {
                                request: GroupRequest::Auto,
                                effect: facts.effect,
                                id: None,
                                end_offset,
                            },
                        };
                    }
                    _ => unreachable!(),
                }
                facts.effect = outer.join(facts.effect);
            }
            E::BreakParent => facts.effect = BreakEffect::REQUIRED,
            E::HardLine | E::LiteralLine | E::LiteralLineWithIndent => {
                facts.effect = BreakEffect::REQUIRED;
                facts.intrinsic = true;
            }
            E::HardLineWithoutBreakParent
            | E::LiteralLineWithoutBreakParent
            | E::StartLineSuffix { .. } => facts.intrinsic = true,
            E::Text { source, width }
            | E::IfBreak { source, width }
            | E::IfFlat { source, width } => {
                facts.intrinsic |= !Text::from_parts(source, width).is_single_line();
            }
            E::SoftLineIfBreak(_) => facts.conditional = true,
            E::StartAlignText(_) | E::StartDedent | E::StartMarkRoot | E::StartDedentToRoot => {
                facts.rich_indent = true;
            }
            E::Fragment { elements, summary } => {
                let fragment = Fragment { elements, summary };
                facts.include(fragment);
                facts.effect = facts.effect.join(fragment.effect());
                facts.intrinsic |= fragment.has_intrinsic_lines();
            }
            E::FlatFragment { elements, summary } => facts.include(Fragment { elements, summary }),
            E::Conditional(c) | E::FlatConditional(c) => {
                facts.include(c.broken);
                facts.include(c.flat);
                facts.conditional |= matches!(c.target, ConditionTarget::Group(_));
                facts.effect = facts.effect.join(c.broken.effect()).join(c.flat.effect());
                facts.intrinsic |= c.broken.has_intrinsic_lines() || c.flat.has_intrinsic_lines();
            }
            E::Alternatives(a) => {
                for &state in a.preferred.iter().chain(std::iter::once(&a.fallback)) {
                    facts.include(state);
                }
                facts.intrinsic |= a.first().has_intrinsic_lines();
            }
            E::GroupWithStates(group) => {
                facts.states = true;
                facts.effect = facts.effect.join(BreakEffect::from_request(group.request));
                facts.intrinsic |= group.contents.has_intrinsic_lines();
                facts.include(group.contents);
                for &state in group.states {
                    facts.include(state);
                }
            }
            _ => {}
        }
    }
    let capabilities = if facts.effect.preferred_breaks() || facts.conditional {
        FragmentCapabilities::from_flat(elements.iter().all(|element| element.permits_flat()))
    } else {
        FragmentCapabilities::classify(&elements)
    };
    let mut stored = ArenaVec::with_capacity_in(elements.len(), allocator);
    stored.extend(elements);
    Fragment::new(
        stored.into_bump_slice(),
        facts.effect,
        capabilities,
        facts.conditional,
        facts.intrinsic,
        facts.states,
        facts.rich_indent,
    )
}
