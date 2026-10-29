//! Propagation facts for documents with independently selected group states.
use rustc_hash::{FxHashMap, FxHashSet};

use super::{FormatElement, GroupRequest};

type Key = (usize, usize, bool);

enum Event<'a> {
    Slice(&'a [FormatElement<'a>], bool),
    EndSlice(Key),
    Scan(&'a [FormatElement<'a>], usize, bool),
    EndGroup(usize),
    EndBarrier(bool),
}

pub(super) fn analyze<'a>(root: &'a [FormatElement<'a>]) -> FxHashMap<usize, bool> {
    let mut groups = FxHashMap::default();
    let mut slices = FxHashMap::default();
    let mut barriers = FxHashSet::default();
    let mut effects = vec![false];
    let mut events = vec![Event::Slice(root, false)];
    while let Some(event) = events.pop() {
        match event {
            Event::Slice(elements, flat) => {
                let key = (elements.as_ptr() as usize, elements.len(), flat);
                if let Some(&effect) = slices.get(&key) {
                    *effects.last_mut().unwrap() |= effect;
                } else {
                    effects.push(false);
                    events.push(Event::EndSlice(key));
                    events.push(Event::Scan(elements, 0, flat));
                }
            }
            Event::EndSlice(key) => {
                let effect = effects.pop().unwrap();
                slices.insert(key, effect);
                *effects.last_mut().unwrap() |= effect;
            }
            Event::EndGroup(key) => {
                let effect = effects.pop().unwrap();
                groups.insert(key, effect);
                *effects.last_mut().unwrap() |= effect;
            }
            Event::EndBarrier(effect) => {
                effects.pop();
                *effects.last_mut().unwrap() |= effect;
            }
            Event::Scan(elements, index, flat) => {
                let Some(source) = elements.get(index) else {
                    continue;
                };
                let Some(element) = (if flat {
                    source.flattened()
                } else {
                    Some(*source)
                }) else {
                    events.push(Event::Scan(elements, index + 1, flat));
                    continue;
                };
                if let FormatElement::StartGroup {
                    request,
                    end_offset,
                    ..
                } = element
                {
                    let end = index + end_offset as usize;
                    events.push(Event::Scan(elements, end + 1, flat));
                    let key = source as *const _ as usize;
                    if let Some(&effect) = groups.get(&key) {
                        *effects.last_mut().unwrap() |= effect;
                    } else {
                        effects.push(matches!(request, GroupRequest::Expanded));
                        events.push(Event::EndGroup(key));
                        events.push(Event::Slice(&elements[index + 1..end], false));
                    }
                    continue;
                }
                events.push(Event::Scan(elements, index + 1, flat));
                match element {
                    FormatElement::BreakParent
                    | FormatElement::HardLine
                    | FormatElement::LiteralLine
                    | FormatElement::LiteralLineWithIndent => *effects.last_mut().unwrap() = true,
                    FormatElement::Fragment { elements, .. } => {
                        events.push(Event::Slice(elements.get(), false))
                    }
                    FormatElement::FlatFragment { elements, .. } => {
                        events.push(Event::Slice(elements.get(), true))
                    }
                    FormatElement::Conditional(c) | FormatElement::FlatConditional(c) => {
                        let flat = matches!(element, FormatElement::FlatConditional(_));
                        events.push(Event::Slice(c.flat.elements(), flat));
                        events.push(Event::Slice(c.broken.elements(), flat));
                    }
                    FormatElement::Alternatives(a) => {
                        if !barriers.insert((a as *const _ as usize, false)) {
                            continue;
                        }
                        effects.push(false);
                        events.push(Event::EndBarrier(false));
                        events.push(Event::Slice(a.fallback.elements(), false));
                        for state in a.preferred.iter().rev() {
                            events.push(Event::Slice(state.elements(), false));
                        }
                    }
                    FormatElement::GroupWithStates(group) => {
                        if !barriers.insert((group as *const _ as usize, true)) {
                            *effects.last_mut().unwrap() |=
                                matches!(group.request, GroupRequest::Expanded);
                            continue;
                        }
                        effects.push(false);
                        events.push(Event::EndBarrier(matches!(
                            group.request,
                            GroupRequest::Expanded
                        )));
                        for state in group.states.iter().rev() {
                            events.push(Event::Slice(state.elements(), false));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    groups
}
