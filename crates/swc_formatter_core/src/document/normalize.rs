//! Sequence normalization with temporary text runs. The result uses only Doc instructions.

use rustc_hash::FxHashMap;
use swc_next_allocator::Allocator;

use super::{
    BreakEffect, Doc, Fragment, Group, GroupRequest, IfBreak, Instruction as I, Text,
    transform::{FragmentKey, Operation, children, key, map_element, summarize, summarize_array},
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
    Element(I<'a>),
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
    // Keep the optional Fill memo off the ordinary sequence-folding stack.
    empty_fill_parts: Option<Box<FxHashMap<FragmentKey, bool>>>,
}

pub(super) fn clean(doc: Doc<'_>) -> Doc<'_> {
    let original = doc.root;
    let mut normalizer = Normalizer {
        allocator: doc.allocator,
        empty: Fragment::empty(),
        runs: Vec::new(),
        sequences: FxHashMap::default(),
        complete: FxHashMap::default(),
        empty_fill_parts: None,
    };
    let mut pending = vec![(original, false)];
    while let Some((fragment, ready)) = pending.pop() {
        let elements = fragment.elements();
        let id = key(elements);
        if ready {
            let sequence = normalizer.sequence(elements);
            normalizer.sequences.insert(id, sequence);
            if matches!(elements, [] | [I::Text { .. }])
                && (!sequence.is_empty() || elements.is_empty())
            {
                normalizer.complete.insert(id, fragment);
            }
        } else if !normalizer.sequences.contains_key(&id) {
            pending.push((fragment, true));
            for &element in elements {
                children(element, Operation::Clean, |child| {
                    pending.push((child, false))
                });
            }
        }
    }
    let updated = normalizer.complete(original);
    Doc {
        root: updated,
        ..doc
    }
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
            _ => pieces.push(Piece::Element(I::fragment(sequence.middle))),
        }
        if let Some(tail) = sequence.tail {
            self.append_text(pieces, tail);
        }
    }

    fn materialize_text(&self, run: RunId) -> I<'a> {
        if let Run::Text(text) = self.runs[run] {
            return I::text(text);
        }
        let mut text = String::with_capacity(self.runs[run].len() as usize);
        let mut pending = vec![run];
        while let Some(id) = pending.pop() {
            match self.runs[id] {
                Run::Text(part) => text.push_str(part.source()),
                Run::Join { left, right, .. } => pending.extend([right, left]),
            }
        }
        I::text(Text::new(self.allocator.alloc_str(&text)))
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
                elements.push(I::fragment(sequence.middle));
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
            [Piece::Element(I::Fragment { elements, summary })] => Fragment {
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

    fn sequence(&mut self, elements: &'a [I<'a>]) -> Sequence<'a> {
        let mut pieces = Vec::new();
        let mut scopes = Vec::new();
        let mut texts = FxHashMap::default();
        for &element in elements {
            match element {
                I::Text { source, width } => {
                    self.text(&mut pieces, Text::from_parts(source, width))
                }
                I::Fragment { elements, .. } => {
                    let sequence = self.sequences[&key(elements.get())];
                    self.append(&mut pieces, sequence);
                }
                I::StartGroup { .. }
                | I::StartGroupWithFlatWidth { .. }
                | I::StartIndent
                | I::StartAbsoluteIndent { .. }
                | I::StartIndentIfBreak { .. }
                | I::StartAlign { .. }
                | I::StartAlignText(_)
                | I::StartDedent
                | I::StartMarkRoot
                | I::StartDedentToRoot
                | I::StartFill
                | I::StartFillPart { .. }
                | I::StartLineSuffix { .. }
                | I::StartLabel(_) => scopes.push((element, std::mem::take(&mut pieces))),
                I::EndGroup
                | I::EndIndent
                | I::EndAlign { .. }
                | I::EndRichIndent(_)
                | I::EndFill
                | I::EndFillPart
                | I::EndLineSuffix
                | I::EndLabel => {
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
                        I::Text { source, width } => {
                            self.text(&mut pieces, Text::from_parts(source, width))
                        }
                        _ => pieces.push(Piece::Element(updated)),
                    }
                }
            }
        }
        self.finish_sequence(pieces)
    }

    fn scope(&mut self, pieces: &mut Vec<Piece<'a>>, start: I<'a>, body: Sequence<'a>, end: I<'a>) {
        if body.is_empty() && !keep_empty_scope(start) {
            return;
        }

        let start = match start {
            I::StartGroup { request, id, .. } => I::StartGroup {
                request,
                id,
                effect: BreakEffect::default(),
                end_offset: 2,
            },
            I::StartGroupWithFlatWidth { .. } => I::StartGroup {
                request: GroupRequest::Auto,
                id: None,
                effect: BreakEffect::default(),
                end_offset: 2,
            },
            I::StartFillPart { .. } => I::StartFillPart { end_offset: 2 },
            I::StartLineSuffix { .. } => I::StartLineSuffix { end_offset: 2 },
            I::StartIndentIfBreak { target, negate, .. } => I::StartIndentIfBreak {
                target,
                negate,
                end_offset: 2,
            },
            start => start,
        };

        if let I::StartGroup { request, id, .. } = start
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
        // Even a single part gets its execution mode from Fill. Only an empty
        // list of empty parts has no mode-dependent contents to preserve.
        if matches!(start, I::StartFill)
            && empty_fill_parts(
                body.elements(),
                self.empty_fill_parts.get_or_insert_with(Box::default),
            )
        {
            return;
        }
        let wrapped = summarize_array(self.allocator, [start, I::fragment(body), end]);
        pieces.push(Piece::Element(I::fragment(wrapped)));
    }
}

// Empty contents do not erase the scope's own effects. Keep this uncommon
// classification out of the ordinary sequence-folding path.
#[cold]
#[inline(never)]
fn keep_empty_scope(start: I<'_>) -> bool {
    matches!(
        start,
        I::StartGroup {
            request: GroupRequest::Expanded,
            ..
        } | I::StartGroup { id: Some(_), .. }
            | I::StartLineSuffix { .. }
            | I::StartFillPart { .. }
            | I::StartLabel(_)
    )
}

fn only_group(mut elements: &[I<'_>]) -> Option<(GroupRequest, Option<super::GroupId>)> {
    loop {
        match elements {
            [
                I::Fragment {
                    elements: child, ..
                },
            ] => elements = child.get(),
            [
                I::StartGroup {
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

// A Fill is a logical list of entry scopes, potentially through shared refs.
#[inline(never)]
fn empty_fill_parts<'a>(elements: &'a [I<'a>], memo: &mut FxHashMap<FragmentKey, bool>) -> bool {
    let mut pending = vec![(elements, false)];
    while let Some((part, ready)) = pending.pop() {
        if memo.contains_key(&key(part)) {
            continue;
        }
        if !ready {
            pending.push((part, true));
            for &element in part {
                if let I::Fragment { elements, .. } = element {
                    pending.push((elements.get(), false));
                }
            }
            continue;
        }
        let mut empty = true;
        let mut rest = part;
        while let Some((first, next)) = rest.split_first() {
            rest = next;
            let entry = match *first {
                I::Fragment { elements, .. } => memo[&key(elements.get())],
                I::StartFillPart { end_offset: 2 } => {
                    let [I::Fragment { elements, .. }, I::EndFillPart, next @ ..] = rest else {
                        empty = false;
                        break;
                    };
                    rest = next;
                    elements.get().is_empty()
                }
                _ => false,
            };
            empty &= entry;
            if !empty {
                break;
            }
        }
        memo.insert(key(part), empty);
    }
    memo[&key(elements)]
}

#[derive(Clone, Copy)]
enum PairState {
    Complete,
    NeedsLine,
}

impl PairState {
    fn step(self, element: I<'_>) -> Option<Self> {
        match (self, element) {
            (Self::Complete, I::HardLine | I::LiteralLine) => Some(Self::Complete),
            (Self::Complete, I::BreakParent) => Some(Self::NeedsLine),
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
    fn finish(self, allocator: &'a Allocator, marker: I<'a>) -> Fragment<'a> {
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
    let original = doc.root;
    let marker = summarize(doc.allocator, vec![I::BreakParent]);
    let entry = summarize(
        doc.allocator,
        vec![
            I::StartFillPart { end_offset: 2 },
            I::fragment(marker),
            I::EndFillPart,
        ],
    );
    let markers = [I::BreakParent, I::fragment(entry)];
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
    Doc {
        root: updated,
        ..doc
    }
}

fn is_line(element: I<'_>) -> bool {
    matches!(
        element,
        I::NormalLine
            | I::SoftLine
            | I::NormalLiteralLine
            | I::SoftLiteralLine
            | I::HardLineOnly
            | I::LiteralLineOnly
    )
}

fn single_leaf<'a>(mut elements: &'a [I<'a>]) -> Option<I<'a>> {
    loop {
        match elements {
            [
                I::Fragment {
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
    markers: [I<'a>; 2],
) -> Tail<'a> {
    let original = fragment.elements();
    let lookup = |child: Fragment<'a>, in_fill: bool| {
        tails[&key(child.elements())][tail_index(in_fill, PairState::Complete)]
            .finish(allocator, markers[usize::from(in_fill)])
    };
    let marker = markers[usize::from(in_fill)];

    // Clean stores each wrapper as a complete fragment with a reference to
    // its contents. An indentation wrapper is one logical part of a sequence.
    if let [start, I::Fragment { elements, summary }, end] = original
        && matches!(
            *start,
            I::StartGroup { .. }
                | I::StartIndent
                | I::StartAbsoluteIndent { .. }
                | I::StartIndentIfBreak { .. }
                | I::StartAlign { .. }
                | I::StartAlignText(_)
                | I::StartDedent
                | I::StartMarkRoot
                | I::StartDedentToRoot
                | I::StartFill
                | I::StartFillPart { .. }
                | I::StartLabel(_)
                | I::StartLineSuffix { .. }
        )
    {
        let body = Fragment {
            elements: *elements,
            summary: *summary,
        };
        if in_fill
            && matches!(*start, I::StartFillPart { .. })
            && let Some(leaf) = single_leaf(body.elements())
            && (matches!(leaf, I::BreakParent) || is_line(leaf))
            && let Some(next) = state.step(leaf)
        {
            return Tail::Continue(next);
        }
        if matches!(state, PairState::NeedsLine) {
            return Tail::Stop(summarize(allocator, vec![I::fragment(fragment), marker]));
        }
        if matches!(
            *start,
            I::StartAbsoluteIndent { .. }
                | I::StartAlign { .. }
                | I::StartAlignText(_)
                | I::StartDedent
                | I::StartMarkRoot
                | I::StartDedentToRoot
        ) {
            return Tail::Stop(fragment);
        }
        let updated = lookup(body, matches!(*start, I::StartFill));
        let fragment = if different(body, updated) {
            summarize(allocator, vec![*start, I::fragment(updated), *end])
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
        if let I::Fragment { elements, summary } = last {
            let index = tail_index(in_fill, state);
            match tails[&key(elements.get())][index] {
                Tail::Continue(next) => {
                    state = next;
                    remaining = prefix;
                    continue;
                }
                Tail::Stop(updated) => {
                    let child = Fragment { elements, summary };
                    let replacement = different(child, updated).then_some(I::fragment(updated));
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
    remaining: &[I<'a>],
    replacement: Option<I<'a>>,
    restore_marker: Option<I<'a>>,
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
    element: I<'a>,
    lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
) -> Option<I<'a>> {
    match element {
        I::Text { source, .. } | I::TextIfBreak { source, .. } | I::TextIfFlat { source, .. } => {
            let source = source.as_str();
            let stripped = source.trim_end_matches(['\r', '\n']);
            if stripped.len() == source.len() {
                return None;
            }
            let text = Text::new(stripped);
            Some(match element {
                I::Text { .. } => I::text(text),
                I::TextIfBreak { .. } => I::text_if_break(text),
                _ => I::text_if_flat(text),
            })
        }
        I::IfBreak(c) | I::FlatIfBreak(c) => {
            let broken = lookup(c.broken);
            let flat = lookup(c.flat);
            if !different(c.broken, broken) && !different(c.flat, flat) {
                return None;
            }
            let conditional = allocator.alloc(IfBreak { broken, flat, ..*c });
            Some(if matches!(element, I::IfBreak(_)) {
                I::IfBreak(conditional)
            } else {
                I::FlatIfBreak(conditional)
            })
        }
        I::CompactGroup(group) => strip_group(allocator, group, lookup),
        I::SourceGroup(group) => strip_group(allocator, group, lookup),
        _ => None,
    }
}

fn strip_group<'a>(
    allocator: &'a Allocator,
    group: impl Group<'a>,
    lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
) -> Option<I<'a>> {
    let contents = lookup(group.primary());
    different(group.primary(), contents).then(|| group.replace_primary(allocator, contents))
}
