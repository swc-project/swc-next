//! Propagation facts for documents with independently selected group states.
use rustc_hash::{FxHashMap, FxHashSet};

use super::{Group, GroupRequest, Instruction};

type Key = (usize, usize, bool);

enum Event<'a> {
    Slice(&'a [Instruction<'a>], bool),
    EndSlice(Key),
    Scan(&'a [Instruction<'a>], usize, bool),
    EndGroup(usize),
    EndBarrier(bool),
}

pub(super) fn analyze<'a>(root: &'a [Instruction<'a>]) -> FxHashMap<usize, bool> {
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
                if let Instruction::StartGroup {
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
                    Instruction::BreakParent
                    | Instruction::HardLine
                    | Instruction::LiteralLine
                    | Instruction::LiteralLineAtCurrentIndent => {
                        *effects.last_mut().unwrap() = true
                    }
                    Instruction::Fragment { elements, .. } => {
                        events.push(Event::Slice(elements.get(), false))
                    }
                    Instruction::FlatFragment { elements, .. } => {
                        events.push(Event::Slice(elements.get(), true))
                    }
                    Instruction::IfBreak(c) | Instruction::FlatIfBreak(c) => {
                        let flat = matches!(element, Instruction::FlatIfBreak(_));
                        events.push(Event::Slice(c.flat.elements(), flat));
                        events.push(Event::Slice(c.broken.elements(), flat));
                    }
                    Instruction::CompactGroup(group) => {
                        enter_group(group, &mut barriers, &mut effects, &mut events)
                    }
                    Instruction::SourceGroup(group) => {
                        enter_group(group, &mut barriers, &mut effects, &mut events)
                    }
                    _ => {}
                }
            }
        }
    }
    groups
}

fn enter_group<'a>(
    group: impl Group<'a>,
    barriers: &mut FxHashSet<usize>,
    effects: &mut Vec<bool>,
    events: &mut Vec<Event<'a>>,
) {
    let expanded = matches!(group.request(), GroupRequest::Expanded);
    if !barriers.insert(group.key()) {
        *effects.last_mut().unwrap() |= expanded;
        return;
    }
    effects.push(false);
    events.push(Event::EndBarrier(expanded));
    for state in group.layouts().all().rev() {
        events.push(Event::Slice(state.elements(), false));
    }
}
