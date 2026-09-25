//! Interpretation of flat formatting instructions and bounded fit probes.

use super::{FormatElement, Measurement, SpaceKind, text::Text};
use crate::{FormatOptions, IndentStyle};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flat,
    Break,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FitState {
    Measured,
    // Groups can inherit Flat, while Fill still needs its own measurement.
    Unmeasured,
    // A forced break under Flat requires a new group decision.
    Remeasure,
}

/// Selects the probe boundary and initial scope together, with static dispatch.
trait ProbeKind {
    const BOUNDED: bool;

    fn continuation(self) -> Option<ContinuationProbe>;
}

enum ContinuationProbe {
    Group(Option<super::GroupId>),
    Candidate,
}

impl ProbeKind for ContinuationProbe {
    const BOUNDED: bool = false;

    fn continuation(self) -> Option<ContinuationProbe> {
        Some(self)
    }
}

struct FillProbe;

impl ProbeKind for FillProbe {
    const BOUNDED: bool = true;

    fn continuation(self) -> Option<ContinuationProbe> {
        None
    }
}

#[derive(Clone, Copy)]
struct FillState {
    mode: Mode,
    flat: bool,
    // Flat inheritance belongs to this Fill and expires on any emitted line.
    line_generation: usize,
    separator: bool,
    separator_mode: Mode,
}

#[derive(Clone, Copy)]
struct FillContext {
    mode: Mode,
    separator: bool,
    separator_mode: Mode,
}

impl From<FillState> for FillContext {
    fn from(fill: FillState) -> Self {
        Self {
            mode: fill.mode,
            separator: fill.separator,
            separator_mode: fill.separator_mode,
        }
    }
}

#[derive(Clone, Copy)]
struct GroupState {
    mode: Mode,
    restore_mode: Mode,
}

/// Fill entry distances remain valid when a retained scope starts within its backing slice.
#[derive(Clone, Copy)]
struct Frame<'a> {
    remaining: &'a [FormatElement<'a>],
    flat: bool,
    restore_mode: Option<Mode>,
    measurement: Measurement,
}

impl<'a> Frame<'a> {
    fn new(elements: &'a [FormatElement<'a>]) -> Self {
        Self {
            remaining: elements,
            flat: false,
            restore_mode: None,
            measurement: Measurement::Display,
        }
    }

    fn next(&mut self) -> Option<FormatElement<'a>> {
        while let Some((&element, remaining)) = self.remaining.split_first() {
            self.remaining = remaining;
            if self.flat {
                if let Some(element) = element.flattened() {
                    return Some(element);
                }
            } else {
                return Some(element);
            }
        }
        None
    }
}

struct Cursor<'a> {
    current: Frame<'a>,
    suspended: Vec<Frame<'a>>,
}

impl<'a> Cursor<'a> {
    fn enter(&mut self, elements: &'a [FormatElement<'a>], flat: bool, restore_mode: Option<Mode>) {
        self.suspended.push(self.current);
        self.current = Frame {
            flat,
            restore_mode,
            ..Frame::new(elements)
        };
    }

    fn next(&mut self, mode: &mut Mode) -> Option<FormatElement<'a>> {
        loop {
            if let Some(element) = self.current.next() {
                return Some(element);
            }
            if let Some(restore) = self.current.restore_mode {
                *mode = restore;
            }
            self.current = self.suspended.pop()?;
        }
    }
}

/// A probe borrows the untouched print continuation. Only new frames go in the
/// scratch stack; crossing a retained scope never clones the pending queue.
struct ProbeCursor<'a, 'scratch> {
    current: Frame<'a>,
    frames: &'scratch mut Vec<Frame<'a>>,
    parents: &'scratch [Frame<'a>],
    parent_len: usize,
}

impl<'a> ProbeCursor<'a, '_> {
    fn enter(&mut self, elements: &'a [FormatElement<'a>], flat: bool, restore_mode: Option<Mode>) {
        let measurement = self.current.measurement;
        self.frames.push(self.current);
        self.current = Frame {
            flat,
            restore_mode,
            measurement,
            ..Frame::new(elements)
        };
    }

    fn next(&mut self, mode: &mut Mode) -> Option<FormatElement<'a>> {
        loop {
            if let Some(element) = self.current.next() {
                return Some(element);
            }
            if let Some(restore) = self.current.restore_mode {
                *mode = restore;
            }
            if let Some(frame) = self.frames.pop() {
                self.current = frame;
            } else {
                let index = self.parent_len.checked_sub(1)?;
                self.parent_len = index;
                self.current = self.parents[index];
            }
        }
    }
}

/// Reuses print scopes without copying unrelated ancestors into each fit probe.
struct BorrowedStack<'parents, 'scratch, P, T> {
    parents: &'parents [P],
    parent_len: usize,
    scratch: &'scratch mut Vec<T>,
}

impl<'parents, 'scratch, P: Copy + Into<T>, T: Copy> BorrowedStack<'parents, 'scratch, P, T> {
    fn new(parents: &'parents [P], scratch: &'scratch mut Vec<T>) -> Self {
        scratch.clear();
        Self {
            parents,
            parent_len: parents.len(),
            scratch,
        }
    }

    fn push(&mut self, value: T) {
        self.scratch.push(value);
    }

    fn pop(&mut self) -> Option<T> {
        self.scratch.pop().or_else(|| {
            self.parent_len = self.parent_len.checked_sub(1)?;
            Some(self.parents[self.parent_len].into())
        })
    }

    fn last_mut(&mut self) -> Option<&mut T> {
        if self.scratch.is_empty() {
            self.parent_len = self.parent_len.checked_sub(1)?;
            self.scratch.push(self.parents[self.parent_len].into());
        }
        self.scratch.last_mut()
    }
}

struct Output {
    text: String,
    column: usize,
    pending_indent: Indentation,
    pending_gap: PendingGap,
    fit_state: FitState,
    line_generation: usize,
}

#[derive(Clone, Copy, Default)]
enum PendingGap {
    #[default]
    None,
    Deferred,
    Reserved,
}

impl PendingGap {
    fn defer(&mut self) {
        if matches!(self, Self::None) {
            *self = Self::Deferred;
        }
    }

    fn reserve(&mut self) -> bool {
        let added = !matches!(self, Self::Reserved);
        *self = Self::Reserved;
        added
    }

    fn reserved_width(self) -> usize {
        usize::from(matches!(self, Self::Reserved))
    }

    fn deferred_width(self) -> usize {
        usize::from(matches!(self, Self::Deferred))
    }

    fn has_space(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Default)]
struct Indentation {
    levels: usize,
    alignment: usize,
    alignments: usize,
}

impl Indentation {
    fn absolute(width: usize, options: FormatOptions) -> Self {
        let unit = usize::from(options.indent_width.value());
        let (levels, alignment) = match width.checked_div(unit) {
            Some(levels) => (levels, width % unit),
            None => (0, width),
        };
        let mut indent = Self {
            alignment,
            alignments: usize::from(alignment != 0),
            ..Self::default()
        };
        if levels != 0 {
            // Align before indent so tabs retain the existing alignment flush.
            indent = indent.indented(options);
            indent.levels += levels - 1;
        }
        indent
    }

    fn width(self, options: FormatOptions) -> usize {
        self.levels * usize::from(options.indent_width.value()) + self.alignment
    }

    fn indented(mut self, options: FormatOptions) -> Self {
        if options.indent_style == IndentStyle::Tab {
            self.levels += self.alignments;
            self.alignment = 0;
            self.alignments = 0;
        }
        self.levels += 1;
        self
    }
}

pub(super) fn print(
    elements: &[FormatElement<'_>],
    group_count: usize,
    conditional_lines: bool,
    options: FormatOptions,
) -> String {
    let mut output = Output {
        text: String::new(),
        column: 0,
        pending_indent: Indentation::default(),
        pending_gap: PendingGap::None,
        fit_state: FitState::Measured,
        line_generation: 0,
    };
    let mut suffixes = Vec::new();
    let mut indent = Indentation::default();
    let mut indents = Vec::new();
    let mut modes = Vec::new();
    let mut group_modes = vec![Mode::Flat; group_count];
    let mut mode = Mode::Break;
    let mut probe = Probe {
        track_groups: conditional_lines,
        ..Probe::default()
    };
    let mut cursor = Cursor {
        current: Frame::new(elements),
        suspended: Vec::new(),
    };
    let mut fills = Vec::new();
    let mut entries = Vec::new();
    while let Some(element) = cursor.next(&mut mode) {
        match element {
            FormatElement::Fragment(fragment) => cursor.enter(fragment.elements, false, None),
            FormatElement::FlatFragment(fragment) => cursor.enter(fragment.0.elements, true, None),
            FormatElement::Alternatives(alternatives)
            | FormatElement::CanonicalAlternatives(alternatives) => {
                let measurement = match element {
                    FormatElement::CanonicalAlternatives(_) => Measurement::Canonical,
                    _ => Measurement::Display,
                };
                let selected = alternatives.preferred.iter().find(|fragment| {
                    probe.fits(
                        ProbeRequest {
                            current: Frame {
                                restore_mode: Some(mode),
                                measurement,
                                ..Frame::new(fragment.elements)
                            },
                            pending: &cursor.suspended,
                            continuation: Some(cursor.current),
                            kind: ContinuationProbe::Candidate,
                            fills: &fills,
                            entries: &entries,
                        },
                        &modes,
                        &output,
                        options,
                        mode,
                        &group_modes,
                    )
                });
                let (fragment, selected_mode) = selected
                    .map_or((alternatives.fallback, Mode::Break), |fragment| {
                        (*fragment, Mode::Flat)
                    });
                output.record_group_mode(selected_mode, false);
                cursor.enter(fragment.elements, false, Some(mode));
                mode = selected_mode;
            }
            FormatElement::SoftLineIfBreak(id) => {
                if group_modes[id.0] == Mode::Break && mode == Mode::Break {
                    flush_suffixes(&mut suffixes, &mut output, options, mode);
                    output.line(indent, options, mode);
                }
            }
            FormatElement::BreakParent => {}
            FormatElement::Text(text) => output.write(text, options, mode),
            FormatElement::Space(kind) => match kind {
                SpaceKind::Deferred => output.pending_gap.defer(),
                SpaceKind::Reserved => {
                    output.pending_gap.reserve();
                }
            },
            FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                if matches!(element, FormatElement::Line) {
                    output.pending_gap.defer();
                }
            }
            FormatElement::SoftLine | FormatElement::Line | FormatElement::HardLine => {
                flush_suffixes(&mut suffixes, &mut output, options, mode);
                output.line(indent, options, mode);
            }
            FormatElement::LiteralLine => {
                flush_suffixes(&mut suffixes, &mut output, options, mode);
                output.line(Indentation::default(), options, mode);
            }
            FormatElement::IfBreak(text) | FormatElement::IfFlat(text) => {
                let group_mode = modes
                    .last()
                    .map_or(Mode::Break, |group: &GroupState| group.mode);
                let active = match element {
                    FormatElement::IfBreak(_) => group_mode == Mode::Break,
                    _ => group_mode == Mode::Flat,
                };
                if active {
                    output.write(text, options, mode);
                }
            }
            FormatElement::IfCurrentBreak(text) | FormatElement::IfCurrentFlat(text) => {
                let active = match element {
                    FormatElement::IfCurrentBreak(_) => mode == Mode::Break,
                    _ => mode == Mode::Flat,
                };
                if active {
                    output.write(text, options, mode);
                }
            }
            FormatElement::LineSuffix(text) => suffixes.push(text),
            FormatElement::StartGroup { effect, id } => {
                let inherit_flat = mode == Mode::Flat && output.fit_state != FitState::Remeasure;
                let flat = !effect.preferred_breaks()
                    && (inherit_flat
                        || probe.fits(
                            ProbeRequest {
                                current: cursor.current,
                                pending: &cursor.suspended,
                                continuation: None,
                                kind: ContinuationProbe::Group(id),
                                fills: &fills,
                                entries: &entries,
                            },
                            &modes,
                            &output,
                            options,
                            mode,
                            &group_modes,
                        ));
                let group_mode = if flat { Mode::Flat } else { Mode::Break };
                output.record_group_mode(group_mode, inherit_flat);
                if let Some(id) = id {
                    group_modes[id.0] = group_mode;
                }
                modes.push(GroupState {
                    mode: group_mode,
                    restore_mode: mode,
                });
                mode = group_mode;
            }
            FormatElement::StartGroupWithFlatWidth { width } => {
                let inherit_flat = mode == Mode::Flat && output.fit_state != FitState::Remeasure;
                let column = output.column + output.pending_gap.reserved_width();
                let flat = inherit_flat
                    || column.saturating_add(width) <= usize::from(options.line_width)
                        && probe.fits(
                            ProbeRequest {
                                current: cursor.current,
                                pending: &cursor.suspended,
                                continuation: None,
                                kind: ContinuationProbe::Group(None),
                                fills: &fills,
                                entries: &entries,
                            },
                            &modes,
                            &output,
                            options,
                            mode,
                            &group_modes,
                        );
                let group_mode = if flat { Mode::Flat } else { Mode::Break };
                output.record_group_mode(group_mode, inherit_flat);
                modes.push(GroupState {
                    mode: group_mode,
                    restore_mode: mode,
                });
                mode = group_mode;
            }
            FormatElement::EndGroup => {
                mode = modes
                    .pop()
                    .expect("group scopes must close after opening")
                    .restore_mode;
            }
            FormatElement::FlatGroup(id) => group_modes[id.0] = Mode::Flat,
            FormatElement::StartIndent => {
                indents.push(indent);
                indent = indent.indented(options);
            }
            FormatElement::StartAbsoluteIndent { width } => {
                indents.push(indent);
                indent = Indentation::absolute(width, options);
            }
            FormatElement::StartIndentIfBreak(id) => {
                indents.push(indent);
                if group_modes[id.0] == Mode::Break {
                    indent = indent.indented(options);
                }
            }
            FormatElement::EndIndent => {
                indent = indents
                    .pop()
                    .expect("indent scopes must close after opening");
            }
            FormatElement::StartAlign { width } => {
                indent.alignment += width;
                indent.alignments += 1;
            }
            FormatElement::EndAlign { width } => {
                indent.alignment = indent
                    .alignment
                    .checked_sub(width)
                    .expect("alignment scopes must close after opening");
                indent.alignments = indent
                    .alignments
                    .checked_sub(1)
                    .expect("alignment scopes must close after opening");
            }
            FormatElement::StartFill => fills.push(FillState {
                mode,
                flat: mode == Mode::Flat && output.fit_state == FitState::Measured,
                line_generation: output.line_generation,
                separator: false,
                separator_mode: Mode::Flat,
            }),
            FormatElement::EndFill => {
                fills.pop().expect("fill scopes must close after opening");
            }
            FormatElement::StartEntry { end_offset } => {
                let end = end_offset - 1;
                let fill_index = fills
                    .len()
                    .checked_sub(1)
                    .expect("entries belong to an active fill");
                let fill = fills[fill_index];
                let entry_mode = if fill.separator {
                    fills[fill_index].separator = false;
                    fill.separator_mode
                } else {
                    fills[fill_index].separator = true;
                    if fill.flat
                        && fill.line_generation == output.line_generation
                        && output.fit_state == FitState::Measured
                    {
                        Mode::Flat
                    } else {
                        let separator_start = end + 1;
                        let next_end = match cursor.current.remaining.get(separator_start) {
                            Some(FormatElement::StartEntry { end_offset }) => {
                                let separator_end = separator_start + end_offset;
                                let content_start = separator_end + 1;
                                match cursor.current.remaining.get(content_start) {
                                    Some(FormatElement::StartEntry { end_offset }) => {
                                        content_start + end_offset
                                    }
                                    _ => separator_end,
                                }
                            }
                            _ => end,
                        };
                        // This bounded decision does not cover the print continuation;
                        // a group remeasurement request must remain pending.
                        let pair_fits = probe.fits(
                            ProbeRequest {
                                current: Frame {
                                    remaining: &cursor.current.remaining[..next_end],
                                    restore_mode: None,
                                    ..cursor.current
                                },
                                pending: &[],
                                continuation: None,
                                kind: FillProbe,
                                fills: &[],
                                entries: &[],
                            },
                            &modes,
                            &output,
                            options,
                            mode,
                            &group_modes,
                        );
                        if pair_fits {
                            fills[fill_index].separator_mode = Mode::Flat;
                            Mode::Flat
                        } else {
                            fills[fill_index].separator_mode = Mode::Break;
                            if next_end != end
                                && probe.fits(
                                    ProbeRequest {
                                        current: Frame {
                                            remaining: &cursor.current.remaining[..end],
                                            restore_mode: None,
                                            ..cursor.current
                                        },
                                        pending: &[],
                                        continuation: None,
                                        kind: FillProbe,
                                        fills: &[],
                                        entries: &[],
                                    },
                                    &modes,
                                    &output,
                                    options,
                                    mode,
                                    &group_modes,
                                )
                            {
                                Mode::Flat
                            } else {
                                Mode::Break
                            }
                        }
                    }
                };
                entries.push(mode);
                mode = entry_mode;
            }
            FormatElement::EndEntry => {
                mode = entries
                    .pop()
                    .expect("fill entries must close after opening");
            }
        }
    }
    flush_suffixes(&mut suffixes, &mut output, options, mode);
    debug_assert_eq!(indent.levels, 0, "document must close every indent scope");
    debug_assert!(indents.is_empty(), "document must close every indent scope");
    debug_assert_eq!(
        indent.alignment, 0,
        "document must close every alignment scope"
    );
    debug_assert!(modes.is_empty(), "document must close every group scope");
    debug_assert!(fills.is_empty(), "document must close every fill scope");
    debug_assert!(entries.is_empty(), "document must close every fill entry");
    output.text
}

impl Output {
    // An inherited mode consumes the request without refreshing its measurement.
    fn record_group_mode(&mut self, mode: Mode, inherited: bool) {
        if mode == Mode::Break {
            self.fit_state = FitState::Unmeasured;
        } else if !inherited {
            self.fit_state = FitState::Measured;
        }
    }

    fn invalidate_line(&mut self, mode: Mode) {
        if mode == Mode::Flat {
            self.fit_state = FitState::Remeasure;
        } else if self.fit_state == FitState::Measured {
            self.fit_state = FitState::Unmeasured;
        }
        self.line_generation += 1;
    }

    fn write(&mut self, text: Text<'_>, options: FormatOptions, mode: Mode) {
        let (character, count) = match options.indent_style {
            IndentStyle::Tab => ('\t', self.pending_indent.levels),
            IndentStyle::Space => (
                ' ',
                self.pending_indent.levels * usize::from(options.indent_width.value()),
            ),
        };
        self.text.extend(std::iter::repeat_n(character, count));
        self.text
            .extend(std::iter::repeat_n(' ', self.pending_indent.alignment));
        if self.pending_gap.has_space() {
            self.text.push(' ');
            self.column += 1;
        }
        self.text.push_str(text.source);
        let (column, multiline) = text.last_line(self.column);
        self.column = column;
        if multiline {
            self.invalidate_line(mode);
        }
        self.pending_indent = Indentation::default();
        self.pending_gap = PendingGap::None;
    }

    fn line(&mut self, indent: Indentation, options: FormatOptions, mode: Mode) {
        self.invalidate_line(mode);
        self.text.push_str(options.line_ending.as_str());
        self.column = indent.width(options);
        self.pending_indent = indent;
        self.pending_gap = PendingGap::None;
    }
}

fn flush_suffixes(
    suffixes: &mut Vec<Text<'_>>,
    output: &mut Output,
    options: FormatOptions,
    mode: Mode,
) {
    for text in suffixes.drain(..) {
        output.pending_gap.defer();
        output.write(text, options, mode);
    }
}

struct ProbeRequest<'a, 'pending, K> {
    current: Frame<'a>,
    pending: &'pending [Frame<'a>],
    continuation: Option<Frame<'a>>,
    kind: K,
    fills: &'pending [FillState],
    entries: &'pending [Mode],
}

#[derive(Default)]
struct Probe<'a> {
    track_groups: bool,
    group_writes: Vec<(super::GroupId, Mode)>,
    modes: Vec<GroupState>,
    frames: Vec<Frame<'a>>,
    fills: Vec<FillContext>,
    entries: Vec<Mode>,
}

impl<'a> Probe<'a> {
    fn fits<K: ProbeKind>(
        &mut self,
        request: ProbeRequest<'a, '_, K>,
        parents: &[GroupState],
        output: &Output,
        options: FormatOptions,
        incoming_mode: Mode,
        committed_groups: &[Mode],
    ) -> bool {
        let ProbeRequest {
            current,
            pending: pending_frames,
            continuation,
            kind,
            fills: parent_fills,
            entries: parent_entries,
        } = request;
        let Self {
            track_groups,
            group_writes,
            modes,
            frames,
            fills,
            entries,
        } = self;
        let mut fills = BorrowedStack::new(parent_fills, fills);
        let mut entries = BorrowedStack::new(parent_entries, entries);
        // Newly entered Fills have no committed separator modes. In a valid,
        // balanced Fill, each entry inherits the Fill mode; its groups restore
        // that mode before EndEntry, so entry push/pop pairs are identities.
        // Count these Fills instead of storing their scopes. Their entry tags
        // must not consume the borrowed contexts of Fills already being printed.
        let mut uncommitted_fills = 0usize;
        group_writes.clear();
        let mut pending_gap = output.pending_gap;
        let mut column = output.column + pending_gap.reserved_width();
        modes.clear();
        frames.clear();
        if let Some(continuation) = continuation {
            frames.push(continuation);
        }
        let mut cursor = ProbeCursor {
            current,
            frames,
            parents: pending_frames,
            parent_len: pending_frames.len(),
        };
        if let Some(ContinuationProbe::Group(id)) = kind.continuation() {
            if *track_groups && let Some(id) = id {
                group_writes.push((id, Mode::Flat));
            }
            modes.push(GroupState {
                mode: Mode::Flat,
                restore_mode: incoming_mode,
            });
        }
        let mut parent_len = parents.len();
        let mut mode = Mode::Flat;
        if column > usize::from(options.line_width) {
            return false;
        }
        // Only active text and reserved spaces advance the probe column.
        while let Some(element) = cursor.next(&mut mode) {
            match element {
                FormatElement::Text(text)
                | FormatElement::IfBreak(text)
                | FormatElement::IfFlat(text)
                | FormatElement::IfCurrentBreak(text)
                | FormatElement::IfCurrentFlat(text) => {
                    if matches!(
                        element,
                        FormatElement::IfBreak(_) | FormatElement::IfFlat(_)
                    ) {
                        let group_mode = modes
                            .last()
                            .map(|group| group.mode)
                            .or_else(|| parents[..parent_len].last().map(|group| group.mode))
                            .unwrap_or(Mode::Break);
                        let active = match element {
                            FormatElement::IfBreak(_) => group_mode == Mode::Break,
                            _ => group_mode == Mode::Flat,
                        };
                        if !active {
                            continue;
                        }
                    }
                    let inactive_current = match element {
                        FormatElement::IfCurrentBreak(_) => mode != Mode::Break,
                        FormatElement::IfCurrentFlat(_) => mode != Mode::Flat,
                        _ => false,
                    };
                    if inactive_current {
                        continue;
                    }
                    column += pending_gap.deferred_width();
                    pending_gap = PendingGap::None;
                    let (next_column, multiline) = text.first_line(column);
                    column = next_column;
                    if multiline {
                        return column <= usize::from(options.line_width);
                    }
                    if column > usize::from(options.line_width) {
                        return false;
                    }
                }
                FormatElement::LineSuffix(_) => {}
                FormatElement::Space(kind) => match kind {
                    SpaceKind::Deferred => pending_gap.defer(),
                    SpaceKind::Reserved => {
                        column += usize::from(pending_gap.reserve());
                        if column > usize::from(options.line_width) {
                            return false;
                        }
                    }
                },
                FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                    if matches!(element, FormatElement::Line) {
                        pending_gap.defer();
                    }
                }
                FormatElement::SoftLine
                | FormatElement::Line
                | FormatElement::HardLine
                | FormatElement::LiteralLine => return true,
                FormatElement::StartGroup { effect, id } => {
                    let expand = match cursor.current.measurement {
                        Measurement::Display => effect.preferred_breaks(),
                        Measurement::Canonical => effect.required(),
                    };
                    if expand && K::BOUNDED {
                        return false;
                    }
                    let group_mode = if expand { Mode::Break } else { mode };
                    modes.push(GroupState {
                        mode: group_mode,
                        restore_mode: mode,
                    });
                    mode = group_mode;
                    if *track_groups && let Some(id) = id {
                        group_writes.push((id, group_mode));
                    }
                }
                FormatElement::StartGroupWithFlatWidth { width } => {
                    if mode == Mode::Flat
                        && column.saturating_add(width) > usize::from(options.line_width)
                    {
                        return false;
                    }
                    modes.push(GroupState {
                        mode,
                        restore_mode: mode,
                    });
                }
                FormatElement::EndGroup => {
                    if let Some(group) = modes.pop() {
                        mode = group.restore_mode;
                    } else {
                        parent_len = parent_len
                            .checked_sub(1)
                            .expect("fit probe closes an active group scope");
                        mode = parents[parent_len].restore_mode;
                    }
                    if K::BOUNDED && modes.is_empty() {
                        mode = Mode::Flat;
                    }
                }
                FormatElement::Fragment(fragment) => cursor.enter(fragment.elements, false, None),
                FormatElement::FlatFragment(fragment) => {
                    cursor.enter(fragment.0.elements, true, None)
                }
                FormatElement::Alternatives(alternatives)
                | FormatElement::CanonicalAlternatives(alternatives) => {
                    let fragment = if mode == Mode::Flat {
                        alternatives.first()
                    } else {
                        alternatives.fallback
                    };
                    cursor.enter(fragment.elements, false, Some(mode));
                }
                FormatElement::SoftLineIfBreak(id) => {
                    if mode == Mode::Flat {
                        continue;
                    }
                    let named = group_writes
                        .iter()
                        .rev()
                        .find(|(written, _)| written.0 == id.0)
                        .map_or(committed_groups[id.0], |(_, mode)| *mode);
                    if named == Mode::Break {
                        return true;
                    }
                }
                FormatElement::FlatGroup(id) => {
                    if *track_groups {
                        group_writes.push((id, Mode::Flat));
                    }
                }
                FormatElement::BreakParent => {
                    if mode == Mode::Flat {
                        return false;
                    }
                }
                FormatElement::StartFill if !K::BOUNDED => {
                    uncommitted_fills += 1;
                }
                FormatElement::EndFill if !K::BOUNDED => {
                    if uncommitted_fills != 0 {
                        uncommitted_fills -= 1;
                    } else {
                        fills.pop().expect("fit probe closes an active fill scope");
                    }
                }
                FormatElement::StartEntry { .. } if !K::BOUNDED && uncommitted_fills == 0 => {
                    entries.push(mode);
                    let fill = fills.last_mut().expect("fit entry belongs to a fill");
                    // Live scopes retain their committed next separator. The
                    // remaining entries then use the Fill's original mode.
                    mode = if fill.separator {
                        fill.separator = false;
                        fill.separator_mode
                    } else {
                        fill.separator = true;
                        fill.separator_mode = fill.mode;
                        fill.mode
                    };
                }
                FormatElement::EndEntry if !K::BOUNDED && uncommitted_fills == 0 => {
                    mode = entries
                        .pop()
                        .expect("fit probe closes an active entry scope");
                }
                FormatElement::StartIndent
                | FormatElement::StartAbsoluteIndent { .. }
                | FormatElement::StartIndentIfBreak(_)
                | FormatElement::EndIndent
                | FormatElement::StartAlign { .. }
                | FormatElement::EndAlign { .. }
                | FormatElement::StartFill
                | FormatElement::EndFill
                | FormatElement::StartEntry { .. }
                | FormatElement::EndEntry => {}
            }
        }
        column <= usize::from(options.line_width)
    }
}
