//! Sequence normalization with temporary text runs. The result uses only Doc instructions.

use rustc_hash::FxHashMap;
use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    BreakEffect, Conditional, Doc, FormatElement as E, Fragment, FragmentCapabilities,
    GroupRequest, GroupWithStates, Text,
    transform::{FragmentKey, Operation, children, key, map_element, root, summarize, with_root},
};

type RunId = usize;

enum Run<'a> {
    Text(Text<'a>),
    Join { left: RunId, right: RunId, len: u32 },
}

impl Run<'_> {
    fn len(&self) -> u32 {
        match self {
            Self::Text(text) => {
                u32::try_from(text.source().len()).expect("text already fits in u32")
            }
            Self::Join { len, .. } => *len,
        }
    }
}

#[derive(Clone, Copy)]
enum Piece<'a> {
    Text(RunId),
    Element(E<'a>),
}

#[derive(Clone, Copy)]
struct Sequence<'a> {
    head: Option<RunId>,
    middle: Fragment<'a>,
    tail: Option<RunId>,
}

impl Sequence<'_> {
    fn is_empty(self) -> bool {
        self.head.is_none() && self.middle.elements().is_empty()
    }
}

struct Normalizer<'a> {
    allocator: &'a Allocator,
    empty: Fragment<'a>,
    runs: Vec<Run<'a>>,
    sequences: FxHashMap<FragmentKey, Sequence<'a>>,
    complete: FxHashMap<FragmentKey, Fragment<'a>>,
    fill_parts: FxHashMap<FragmentKey, Option<FillParts<'a>>>,
}

pub(super) fn clean(doc: Doc<'_>) -> Doc<'_> {
    let original = root(&doc);
    let mut normalizer = Normalizer {
        allocator: doc.allocator,
        empty: Fragment::new(
            &[],
            BreakEffect::default(),
            FragmentCapabilities::Neutral,
            false,
            false,
            false,
            false,
        ),
        runs: Vec::new(),
        sequences: FxHashMap::default(),
        complete: FxHashMap::default(),
        fill_parts: FxHashMap::default(),
    };
    let mut pending = vec![(original, false)];
    while let Some((fragment, ready)) = pending.pop() {
        let elements = fragment.elements();
        let id = key(elements);
        if normalizer.sequences.contains_key(&id) {
            continue;
        }
        if ready {
            let sequence = normalizer.sequence(elements);
            normalizer.sequences.insert(id, sequence);
            if matches!(elements, [] | [E::Text { .. }])
                && (!sequence.is_empty() || elements.is_empty())
            {
                normalizer.complete.insert(id, fragment);
            }
        } else {
            pending.push((fragment, true));
            for &element in elements {
                children(element, Operation::Clean, |child| {
                    pending.push((child, false))
                });
            }
        }
    }
    let updated = normalizer.complete(original);
    with_root(doc, updated)
}

impl<'a> Normalizer<'a> {
    fn append_text(&mut self, pieces: &mut Vec<Piece<'a>>, right: RunId) {
        if let Some(Piece::Text(previous)) = pieces.last_mut() {
            let left = *previous;
            let len = self.runs[left]
                .len()
                .checked_add(self.runs[right].len())
                .expect("cleaned document text exceeds u32 capacity");
            *previous = self.runs.len();
            self.runs.push(Run::Join { left, right, len });
        } else {
            pieces.push(Piece::Text(right));
        }
    }

    fn text(&mut self, pieces: &mut Vec<Piece<'a>>, text: Text<'a>) {
        if !text.source().is_empty() {
            let id = self.runs.len();
            self.runs.push(Run::Text(text));
            self.append_text(pieces, id);
        }
    }

    fn append(&mut self, pieces: &mut Vec<Piece<'a>>, sequence: Sequence<'a>) {
        if let Some(head) = sequence.head {
            self.append_text(pieces, head);
        }
        match sequence.middle.elements() {
            [] => {}
            [element] => pieces.push(Piece::Element(*element)),
            _ => pieces.push(Piece::Element(E::fragment(sequence.middle))),
        }
        if let Some(tail) = sequence.tail {
            self.append_text(pieces, tail);
        }
    }

    fn materialize_text(&self, run: RunId) -> E<'a> {
        if let Run::Text(text) = self.runs[run] {
            return E::text(text);
        }
        let mut text = String::with_capacity(self.runs[run].len() as usize);
        let mut pending = vec![run];
        while let Some(id) = pending.pop() {
            match self.runs[id] {
                Run::Text(part) => text.push_str(part.source()),
                Run::Join { left, right, .. } => pending.extend([right, left]),
            }
        }
        E::text(Text::new(self.allocator.alloc_str(&text)))
    }

    fn seal(&mut self, sequence: Sequence<'a>) -> Fragment<'a> {
        let fragment = if sequence.head.is_none() && sequence.tail.is_none() {
            sequence.middle
        } else {
            let mut elements = Vec::with_capacity(3);
            if let Some(head) = sequence.head {
                elements.push(self.materialize_text(head));
            }
            if !sequence.middle.elements().is_empty() {
                elements.push(E::fragment(sequence.middle));
            }
            if let Some(tail) = sequence.tail {
                elements.push(self.materialize_text(tail));
            }
            summarize(self.allocator, elements)
        };
        self.sequences
            .entry(key(fragment.elements()))
            .or_insert(sequence);
        fragment
    }

    fn complete(&mut self, original: Fragment<'a>) -> Fragment<'a> {
        let id = key(original.elements());
        if let Some(&complete) = self.complete.get(&id) {
            return complete;
        }
        let complete = self.seal(self.sequences[&id]);
        self.complete.insert(id, complete);
        complete
    }

    fn finish_sequence(&self, mut pieces: Vec<Piece<'a>>) -> Sequence<'a> {
        let tail = if pieces.len() > 1
            && let Some(Piece::Text(tail)) = pieces.last().copied()
        {
            pieces.pop();
            Some(tail)
        } else {
            None
        };
        let (head, middle) = if let Some(Piece::Text(head)) = pieces.first().copied() {
            (Some(head), &pieces[1..])
        } else {
            (None, pieces.as_slice())
        };
        let middle = match middle {
            [] => self.empty,
            [Piece::Element(E::Fragment { elements, summary })] => Fragment {
                elements: *elements,
                summary: *summary,
            },
            middle => {
                let elements = middle
                    .iter()
                    .map(|piece| match *piece {
                        Piece::Text(run) => self.materialize_text(run),
                        Piece::Element(element) => element,
                    })
                    .collect();
                summarize(self.allocator, elements)
            }
        };
        Sequence { head, middle, tail }
    }

    fn sequence(&mut self, elements: &'a [E<'a>]) -> Sequence<'a> {
        let mut pieces = Vec::new();
        let mut scopes = Vec::new();
        let mut texts = FxHashMap::default();
        for &element in elements {
            match element {
                E::Text { source, width } => {
                    self.text(&mut pieces, Text::from_parts(source, width))
                }
                E::Fragment { elements, .. } => {
                    let sequence = self.sequences[&key(elements.get())];
                    self.append(&mut pieces, sequence);
                }
                E::StartGroup { .. }
                | E::StartGroupWithFlatWidth { .. }
                | E::StartIndent
                | E::StartAbsoluteIndent { .. }
                | E::StartIndentIfBreak { .. }
                | E::StartAlign { .. }
                | E::StartAlignText(_)
                | E::StartDedent
                | E::StartMarkRoot
                | E::StartDedentToRoot
                | E::StartFill
                | E::StartEntry { .. }
                | E::StartLineSuffix { .. }
                | E::StartLabel(_) => scopes.push((element, std::mem::take(&mut pieces))),
                E::EndGroup
                | E::EndIndent
                | E::EndAlign { .. }
                | E::EndRichIndent(_)
                | E::EndFill
                | E::EndEntry
                | E::EndLineSuffix
                | E::EndLabel => {
                    let (start, parent) = scopes.pop().expect("Doc scopes are balanced");
                    let body = self.finish_sequence(std::mem::replace(&mut pieces, parent));
                    self.scope(&mut pieces, start, body, element);
                }
                _ => {
                    children(element, Operation::Clean, |child| {
                        self.complete(child);
                    });
                    let updated = map_element(
                        self.allocator,
                        element,
                        Operation::Clean,
                        |child| self.complete[&key(child.elements())],
                        &mut texts,
                    )
                    .unwrap_or(element);
                    match updated {
                        E::Text { source, width } => {
                            self.text(&mut pieces, Text::from_parts(source, width))
                        }
                        _ => pieces.push(Piece::Element(updated)),
                    }
                }
            }
        }
        self.finish_sequence(pieces)
    }

    fn scope(&mut self, pieces: &mut Vec<Piece<'a>>, start: E<'a>, body: Sequence<'a>, end: E<'a>) {
        let start = match start {
            E::StartGroup { request, id, .. } => E::StartGroup {
                request,
                id,
                effect: BreakEffect::default(),
                end_offset: 2,
            },
            E::StartGroupWithFlatWidth { .. } => E::StartGroup {
                request: GroupRequest::Auto,
                id: None,
                effect: BreakEffect::default(),
                end_offset: 2,
            },
            E::StartEntry { .. } => E::StartEntry { end_offset: 2 },
            E::StartLineSuffix { .. } => E::StartLineSuffix { end_offset: 2 },
            E::StartIndentIfBreak { target, negate, .. } => E::StartIndentIfBreak {
                target,
                negate,
                end_offset: 2,
            },
            start => start,
        };
        let keep_empty = matches!(
            start,
            E::StartGroup {
                request: GroupRequest::Expanded,
                ..
            } | E::StartGroup { id: Some(_), .. }
                | E::StartEntry { .. }
                | E::StartLabel(_)
        );
        if body.is_empty() && !keep_empty {
            return;
        }

        if let E::StartGroup { request, id, .. } = start
            && body.head.is_none()
            && body.tail.is_none()
            && let Some((child_request, child_id)) = only_group(body.middle.elements())
            && matches!(request, GroupRequest::Expanded)
                == matches!(child_request, GroupRequest::Expanded)
            && id == child_id
        {
            self.append(pieces, body);
            return;
        }

        let body = self.seal(body);
        if matches!(start, E::StartFill)
            && let Some(parts) = fill_parts(body.elements(), &mut self.fill_parts)
        {
            if parts.all_empty {
                return;
            }
            if let Some(part) = parts.single {
                self.append(pieces, self.sequences[&key(part.elements())]);
                return;
            }
        }
        let wrapped = summarize(self.allocator, vec![start, E::fragment(body), end]);
        pieces.push(Piece::Element(E::fragment(wrapped)));
    }
}

fn only_group(mut elements: &[E<'_>]) -> Option<(GroupRequest, Option<super::GroupId>)> {
    loop {
        match elements {
            [
                E::Fragment {
                    elements: child, ..
                },
            ] => elements = child.get(),
            [
                E::StartGroup {
                    request,
                    id,
                    end_offset,
                    ..
                },
                ..,
            ] if *end_offset as usize + 1 == elements.len() => {
                return Some((*request, *id));
            }
            _ => return None,
        }
    }
}

#[derive(Clone, Copy)]
struct FillParts<'a> {
    count: u8,
    all_empty: bool,
    single: Option<Fragment<'a>>,
}

impl<'a> FillParts<'a> {
    const EMPTY: Self = Self {
        count: 0,
        all_empty: true,
        single: None,
    };

    fn join(self, other: Self) -> Self {
        let count = (self.count + other.count).min(2);
        Self {
            count,
            all_empty: self.all_empty && other.all_empty,
            single: if count == 1 {
                self.single.or(other.single)
            } else {
                None
            },
        }
    }
}

// A Fill is a logical list of entry scopes, potentially through shared refs.
fn fill_parts<'a>(
    elements: &'a [E<'a>],
    memo: &mut FxHashMap<FragmentKey, Option<FillParts<'a>>>,
) -> Option<FillParts<'a>> {
    let mut pending = vec![(elements, false)];
    while let Some((part, ready)) = pending.pop() {
        if memo.contains_key(&key(part)) {
            continue;
        }
        if !ready {
            pending.push((part, true));
            for &element in part {
                if let E::Fragment { elements, .. } = element {
                    pending.push((elements.get(), false));
                }
            }
            continue;
        }
        let mut result = Some(FillParts::EMPTY);
        let mut rest = part;
        while let Some((first, next)) = rest.split_first() {
            rest = next;
            let entry = match *first {
                E::Fragment { elements, .. } => memo[&key(elements.get())],
                E::StartEntry { end_offset: 2 } => {
                    let [E::Fragment { elements, summary }, E::EndEntry, next @ ..] = rest else {
                        result = None;
                        break;
                    };
                    rest = next;
                    Some(FillParts {
                        count: 1,
                        all_empty: elements.get().is_empty(),
                        single: Some(Fragment {
                            elements: *elements,
                            summary: *summary,
                        }),
                    })
                }
                _ => None,
            };
            result = result.zip(entry).map(|(parts, entry)| parts.join(entry));
            if result.is_none() {
                break;
            }
        }
        memo.insert(key(part), result);
    }
    memo[&key(elements)]
}

#[derive(Clone, Copy)]
enum PairState {
    Complete,
    NeedsLine,
}

impl PairState {
    fn step(self, element: E<'_>) -> Option<Self> {
        match (self, element) {
            (Self::Complete, E::HardLine | E::LiteralLine) => Some(Self::Complete),
            (Self::Complete, E::BreakParent) => Some(Self::NeedsLine),
            (Self::NeedsLine, element) if is_line(element) => Some(Self::Complete),
            _ => None,
        }
    }
}

// Match pairs from right to left. The second state carries a BreakParent
// across a shared sequence boundary until we see its preceding instruction.
#[derive(Clone, Copy)]
enum Tail<'a> {
    Continue(PairState),
    Stop(Fragment<'a>),
}

impl<'a> Tail<'a> {
    fn finish(self, allocator: &'a Allocator, marker: E<'a>) -> Fragment<'a> {
        match self {
            Self::Continue(PairState::Complete) => summarize(allocator, Vec::new()),
            Self::Continue(PairState::NeedsLine) => summarize(allocator, vec![marker]),
            Self::Stop(fragment) => fragment,
        }
    }
}

type Tails<'a> = FxHashMap<FragmentKey, [Tail<'a>; 4]>;

fn tail_index(in_fill: bool, state: PairState) -> usize {
    usize::from(in_fill) * 2 + usize::from(matches!(state, PairState::NeedsLine))
}

pub(super) fn strip_trailing_hardline(doc: Doc<'_>) -> Doc<'_> {
    let doc = clean(doc);
    let original = root(&doc);
    let marker = summarize(doc.allocator, vec![E::BreakParent]);
    let entry = summarize(
        doc.allocator,
        vec![
            E::StartEntry { end_offset: 2 },
            E::fragment(marker),
            E::EndEntry,
        ],
    );
    let markers = [E::BreakParent, E::fragment(entry)];
    let mut tails = Tails::default();
    let mut pending = vec![(original, false)];
    while let Some((fragment, ready)) = pending.pop() {
        let elements = fragment.elements();
        let id = key(elements);
        if tails.contains_key(&id) {
            continue;
        }
        if ready {
            let results = std::array::from_fn(|index| {
                let in_fill = index >= 2;
                let state = if index % 2 == 0 {
                    PairState::Complete
                } else {
                    PairState::NeedsLine
                };
                strip_tail(doc.allocator, fragment, &tails, state, in_fill, markers)
            });
            tails.insert(id, results);
        } else {
            pending.push((fragment, true));
            for &element in elements {
                children(element, Operation::Clean, |child| {
                    pending.push((child, false))
                });
            }
        }
    }
    let updated = tails[&key(original.elements())][0].finish(doc.allocator, markers[0]);
    with_root(doc, updated)
}

fn is_line(element: E<'_>) -> bool {
    matches!(
        element,
        E::Line
            | E::SoftLine
            | E::NormalLiteralLine
            | E::SoftLiteralLine
            | E::HardLineWithoutBreakParent
            | E::LiteralLineWithoutBreakParent
    )
}

fn single_leaf<'a>(mut elements: &'a [E<'a>]) -> Option<E<'a>> {
    loop {
        match elements {
            [
                E::Fragment {
                    elements: child, ..
                },
            ] => elements = child.get(),
            [element] => return Some(*element),
            _ => return None,
        }
    }
}

fn strip_tail<'a>(
    allocator: &'a Allocator,
    fragment: Fragment<'a>,
    tails: &Tails<'a>,
    mut state: PairState,
    in_fill: bool,
    markers: [E<'a>; 2],
) -> Tail<'a> {
    let original = fragment.elements();
    let lookup = |child: Fragment<'a>, in_fill: bool| {
        tails[&key(child.elements())][tail_index(in_fill, PairState::Complete)]
            .finish(allocator, markers[usize::from(in_fill)])
    };
    let marker = markers[usize::from(in_fill)];

    // Clean stores each wrapper as a complete fragment with a reference to
    // its contents. An indentation wrapper is one logical part of a sequence.
    if let [start, E::Fragment { elements, summary }, end] = original
        && matches!(
            *start,
            E::StartGroup { .. }
                | E::StartIndent
                | E::StartAbsoluteIndent { .. }
                | E::StartIndentIfBreak { .. }
                | E::StartAlign { .. }
                | E::StartAlignText(_)
                | E::StartDedent
                | E::StartMarkRoot
                | E::StartDedentToRoot
                | E::StartFill
                | E::StartEntry { .. }
                | E::StartLabel(_)
                | E::StartLineSuffix { .. }
        )
    {
        let body = Fragment {
            elements: *elements,
            summary: *summary,
        };
        if in_fill
            && matches!(*start, E::StartEntry { .. })
            && let Some(leaf) = single_leaf(body.elements())
            && (matches!(leaf, E::BreakParent) || is_line(leaf))
            && let Some(next) = state.step(leaf)
        {
            return Tail::Continue(next);
        }
        if matches!(state, PairState::NeedsLine) {
            return Tail::Stop(summarize(allocator, vec![E::fragment(fragment), marker]));
        }
        if matches!(
            *start,
            E::StartAbsoluteIndent { .. }
                | E::StartAlign { .. }
                | E::StartAlignText(_)
                | E::StartDedent
                | E::StartMarkRoot
                | E::StartDedentToRoot
        ) {
            return Tail::Stop(fragment);
        }
        let updated = lookup(body, matches!(*start, E::StartFill));
        let fragment = if different(body, updated) {
            summarize(allocator, vec![*start, E::fragment(updated), *end])
        } else {
            fragment
        };
        return Tail::Stop(fragment);
    }

    let mut remaining = original;
    while let Some((&last, prefix)) = remaining.split_last() {
        if let Some(next) = state.step(last) {
            state = next;
            remaining = prefix;
            continue;
        }
        if let E::Fragment { elements, summary } = last {
            let index = tail_index(in_fill, state);
            match tails[&key(elements.get())][index] {
                Tail::Continue(next) => {
                    state = next;
                    remaining = prefix;
                    continue;
                }
                Tail::Stop(updated) => {
                    let child = Fragment { elements, summary };
                    let replacement = different(child, updated).then_some(E::fragment(updated));
                    return finish_tail(allocator, fragment, remaining, replacement, None);
                }
            }
        }
        if matches!(state, PairState::NeedsLine) {
            return finish_tail(allocator, fragment, remaining, None, Some(marker));
        }
        let replacement = strip_node(allocator, last, |child| lookup(child, false));
        return finish_tail(allocator, fragment, remaining, replacement, None);
    }
    Tail::Continue(state)
}

fn finish_tail<'a>(
    allocator: &'a Allocator,
    original: Fragment<'a>,
    remaining: &[E<'a>],
    replacement: Option<E<'a>>,
    restore_marker: Option<E<'a>>,
) -> Tail<'a> {
    let fragment = if let Some(replacement) = replacement {
        let mut elements = remaining[..remaining.len() - 1].to_vec();
        elements.push(replacement);
        summarize(allocator, elements)
    } else if let Some(marker) = restore_marker {
        let mut elements = remaining.to_vec();
        elements.push(marker);
        summarize(allocator, elements)
    } else if remaining.len() != original.elements().len() {
        summarize(allocator, remaining.to_vec())
    } else {
        original
    };
    Tail::Stop(fragment)
}

fn different(left: Fragment<'_>, right: Fragment<'_>) -> bool {
    key(left.elements()) != key(right.elements())
}

fn strip_node<'a>(
    allocator: &'a Allocator,
    element: E<'a>,
    lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
) -> Option<E<'a>> {
    match element {
        E::Text { source, .. } | E::IfBreak { source, .. } | E::IfFlat { source, .. } => {
            let source = source.as_str();
            let stripped = source.trim_end_matches(['\r', '\n']);
            if stripped.len() == source.len() {
                return None;
            }
            let text = Text::new(stripped);
            Some(match element {
                E::Text { .. } => E::text(text),
                E::IfBreak { .. } => E::if_break(text),
                _ => E::if_flat(text),
            })
        }
        E::Conditional(c) | E::FlatConditional(c) => {
            let broken = lookup(c.broken);
            let flat = lookup(c.flat);
            if !different(c.broken, broken) && !different(c.flat, flat) {
                return None;
            }
            let conditional = allocator.alloc(Conditional { broken, flat, ..*c });
            Some(if matches!(element, E::Conditional(_)) {
                E::Conditional(conditional)
            } else {
                E::FlatConditional(conditional)
            })
        }
        E::GroupWithStates(group) => {
            let contents = lookup(group.contents);
            different(group.contents, contents).then(|| {
                E::GroupWithStates(allocator.alloc(GroupWithStates { contents, ..*group }))
            })
        }
        E::Alternatives(a) => {
            let contents = lookup(a.first());
            if !different(a.first(), contents) {
                return None;
            }
            let mut states = ArenaVec::with_capacity_in(a.preferred.len() + 1, allocator);
            states.extend_from_slice(a.preferred);
            states.push(a.fallback);
            Some(E::GroupWithStates(allocator.alloc(GroupWithStates {
                contents,
                states: states.into_bump_slice(),
                request: GroupRequest::Auto,
                id: None,
            })))
        }
        _ => None,
    }
}
