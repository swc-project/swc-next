//! Interpretation of flat formatting instructions and bounded fit probes.

use super::{ConditionTarget, FlatSummary, FormatElement, text::Text};
use crate::{FormatOptions, IndentStyle};
use swc_next_allocator::{Allocator, vec::ArenaVec};

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

    fn continuation(&self) -> Option<ContinuationProbe>;

    /// An exact prefix advances the cursor before the continuation is measured.
    fn prepare(
        &self,
        _current: &mut Frame<'_>,
        _column: usize,
        _pending_line: &mut bool,
    ) -> Option<usize> {
        None
    }

    fn start_entry(&mut self, _column: usize, _pending_line: bool) {}

    fn end_entry(&mut self) -> bool {
        false
    }

    fn end_fill(&mut self) -> bool {
        false
    }

    fn line(&mut self, _pending_line: bool) -> bool {
        false
    }

    fn layout_dependency(&mut self) -> bool {
        false
    }
}

#[derive(Clone, Copy)]
enum ContinuationProbe {
    Group,
    Candidate,
}

impl ProbeKind for ContinuationProbe {
    const BOUNDED: bool = false;

    fn continuation(&self) -> Option<ContinuationProbe> {
        Some(*self)
    }
}

#[derive(Default)]
struct FillMeasurement {
    current_fits: bool,
    flat_pairs: usize,
}

struct FillProbe<'result> {
    result: &'result mut FillMeasurement,
    entry_depth: usize,
    completed_entries: usize,
    can_batch: bool,
    next_item_in_bounds: bool,
    line_width: usize,
}

impl FillProbe<'_> {
    fn checkpoint_only(&mut self) -> bool {
        self.can_batch = false;
        self.result.flat_pairs != 0
    }
}

impl ProbeKind for FillProbe<'_> {
    const BOUNDED: bool = true;

    fn continuation(&self) -> Option<ContinuationProbe> {
        None
    }

    fn start_entry(&mut self, column: usize, pending_line: bool) {
        self.entry_depth += 1;
        if self.entry_depth == 1 && self.completed_entries.is_multiple_of(2) {
            self.next_item_in_bounds =
                column.saturating_add(usize::from(pending_line)) <= self.line_width;
        }
    }

    fn end_entry(&mut self) -> bool {
        self.entry_depth -= 1;
        if self.entry_depth == 0 {
            self.completed_entries += 1;
            if self.completed_entries == 1 {
                self.result.current_fits = true;
            } else if self.completed_entries % 2 == 1 {
                self.result.flat_pairs += 1;
                return !self.can_batch || !self.next_item_in_bounds;
            }
        }
        false
    }

    fn end_fill(&mut self) -> bool {
        if self.entry_depth == 0 {
            if self.can_batch {
                self.result.flat_pairs += 1;
            }
            return true;
        }
        false
    }

    fn line(&mut self, pending_line: bool) -> bool {
        pending_line && self.checkpoint_only()
    }

    fn layout_dependency(&mut self) -> bool {
        self.checkpoint_only()
    }
}

#[derive(Clone, Copy)]
struct FillState {
    mode: Mode,
    separator: bool,
    separator_mode: Mode,
}

struct FillEntry<'a> {
    current: Frame<'a>,
    mode: Mode,
    has_line_suffix: bool,
}

/// Live Fill layout decisions and their batch counts share one scope owner.
/// Continuation probes only borrow `contexts`, whose representation stays small.
#[derive(Default)]
struct FillStack {
    contexts: Vec<FillState>,
    batches: Vec<usize>,
}

impl FillStack {
    fn contexts(&self) -> &[FillState] {
        &self.contexts
    }

    fn is_empty(&self) -> bool {
        self.contexts.is_empty() && self.batches.is_empty()
    }

    #[inline(never)]
    fn start(&mut self, mode: Mode) {
        self.contexts.push(FillState {
            mode,
            separator: false,
            separator_mode: Mode::Flat,
        });
        self.batches.push(0);
    }

    #[inline(never)]
    fn end(&mut self) {
        self.contexts
            .pop()
            .expect("fill scopes must close after opening");
        self.batches
            .pop()
            .expect("fill batches close with their print scopes");
    }

    // Fill's checkpoints and batch initialization must not increase register
    // pressure in the main interpreter, where ordinary text dominates.
    #[inline(never)]
    fn entry<'a>(
        &mut self,
        probe: &mut Probe<'a>,
        entry: FillEntry<'a>,
        parents: &[GroupState],
        output: &Output,
        options: FormatOptions,
        committed_groups: &[Option<Mode>],
    ) -> Mode {
        let FillEntry {
            current,
            mode: incoming_mode,
            has_line_suffix,
        } = entry;
        let fill = self
            .contexts
            .last_mut()
            .expect("entries belong to an active fill");
        if fill.separator {
            fill.separator = false;
            return fill.separator_mode;
        }
        fill.separator = true;
        let batch = self
            .batches
            .len()
            .checked_sub(1)
            .expect("fill entries retain their scope's batch");
        if self.batches[batch] != 0 {
            self.batches[batch] -= 1;
            fill.separator_mode = Mode::Flat;
            return Mode::Flat;
        }
        let mut measurement = FillMeasurement::default();
        // A bounded Fill decision does not consume a pending remeasurement
        // request from a group in the print continuation.
        let pair_fits = probe.fits(
            ProbeRequest {
                current,
                pending: &[],
                continuation: None,
                kind: FillProbe {
                    result: &mut measurement,
                    entry_depth: 1,
                    completed_entries: 0,
                    can_batch: output.fit_state != FitState::Remeasure && !has_line_suffix,
                    next_item_in_bounds: true,
                    line_width: usize::from(options.line_width),
                },
                has_line_suffix,
                fills: &[],
                entries: &[],
            },
            parents,
            output,
            options,
            incoming_mode,
            committed_groups,
        );
        if pair_fits || measurement.flat_pairs != 0 {
            self.batches[batch] = measurement.flat_pairs.saturating_sub(1);
            fill.separator_mode = Mode::Flat;
            Mode::Flat
        } else {
            fill.separator_mode = Mode::Break;
            if measurement.current_fits {
                Mode::Flat
            } else {
                Mode::Break
            }
        }
    }
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
    restore_mode: Mode,
}

/// Fill entry distances remain valid when a retained scope starts within its backing slice.
#[derive(Clone, Copy)]
struct Frame<'a> {
    remaining: &'a [FormatElement<'a>],
    flat: bool,
    restore_mode: Option<Mode>,
}

impl<'a> Frame<'a> {
    fn new(elements: &'a [FormatElement<'a>]) -> Self {
        Self {
            remaining: elements,
            flat: false,
            restore_mode: None,
        }
    }
}

struct Cursor<'a> {
    current: Frame<'a>,
    suspended: Vec<Frame<'a>>,
    suffix_scopes: Vec<SuffixScope>,
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

    fn next(
        &mut self,
        mode: &mut Mode,
        indent: &mut Indentation,
    ) -> Option<(&'a FormatElement<'a>, bool)> {
        loop {
            if let Some((element, remaining)) = self.current.remaining.split_first() {
                self.current.remaining = remaining;
                return Some((element, self.current.flat));
            }
            if let Some(restore) = self.current.restore_mode {
                *mode = restore;
            }
            if let Some(scope) = self.suffix_scopes.last()
                && scope.frame_depth == self.suspended.len()
            {
                *indent = scope.restore_indent;
                self.suffix_scopes.pop();
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
        self.frames.push(self.current);
        self.current = Frame {
            flat,
            restore_mode,
            ..Frame::new(elements)
        };
    }

    fn next(&mut self, mode: &mut Mode) -> Option<(&'a FormatElement<'a>, bool)> {
        loop {
            if let Some((element, remaining)) = self.current.remaining.split_first() {
                self.current.remaining = remaining;
                return Some((element, self.current.flat));
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

/// Suffix-only state follows its execution boundary, not every traversed frame.
struct SuffixScope {
    frame_depth: usize,
    restore_indent: Indentation,
}

struct LineSuffix<'a> {
    elements: &'a [FormatElement<'a>],
    mode: Mode,
    indent: Indentation,
}

impl ConditionTarget {
    fn mode(self, current: Mode, groups: &[Option<Mode>]) -> Option<Mode> {
        match self {
            Self::Current => Some(current),
            Self::Group(id) => groups[id.index()],
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
    fit_state: FitState,
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

// Keep the interpreter's register allocation independent of syntax construction.
#[inline(never)]
pub(super) fn print<'a>(
    elements: &'a [FormatElement<'a>],
    allocator: &'a Allocator,
    group_count: usize,
    options: FormatOptions,
) -> String {
    let mut output = Output {
        text: String::new(),
        column: 0,
        pending_indent: Indentation::default(),
        fit_state: FitState::Measured,
    };
    let mut suffixes = Vec::new();
    let mut indent = Indentation::default();
    let mut indents = ArenaVec::with_capacity_in(8, allocator);
    let mut modes = ArenaVec::with_capacity_in(8, allocator);
    let mut group_modes = ArenaVec::with_capacity_in(group_count, allocator);
    group_modes.resize(group_count, None);
    let group_modes = group_modes.into_bump_slice_mut();
    let mut mode = Mode::Break;
    let mut probe = Probe {
        modes: Vec::new(),
        frames: Vec::new(),
        fills: Vec::new(),
        entries: Vec::new(),
    };
    let mut cursor = Cursor {
        current: Frame::new(elements),
        suspended: Vec::new(),
        suffix_scopes: Vec::new(),
    };
    let mut fills = FillStack::default();
    let mut entries = Vec::new();
    loop {
        let Some((source_element, flat)) = cursor.next(&mut mode, &mut indent) else {
            if !suffixes.is_empty() {
                flush_suffixes(&mut suffixes, &mut cursor, None, &mut mode, &mut indent);
                continue;
            }
            break;
        };
        let flattened;
        let element = if flat {
            let Some(element) = source_element.flattened() else {
                continue;
            };
            flattened = element;
            &flattened
        } else {
            source_element
        };
        match *element {
            FormatElement::Fragment(fragment) => cursor.enter(fragment.elements, false, None),
            FormatElement::FlatFragment(fragment) => cursor.enter(fragment.0.elements, true, None),
            FormatElement::Alternatives(alternatives) => {
                let inherit_flat = mode == Mode::Flat && output.fit_state != FitState::Remeasure;
                let selected = if inherit_flat {
                    Some(&alternatives.preferred[0])
                } else {
                    alternatives.preferred.iter().find(|fragment| {
                        probe.fits(
                            ProbeRequest {
                                current: Frame {
                                    restore_mode: Some(mode),
                                    ..Frame::new(fragment.elements)
                                },
                                pending: &cursor.suspended,
                                continuation: Some(cursor.current),
                                kind: ContinuationProbe::Candidate,
                                has_line_suffix: !suffixes.is_empty(),
                                fills: fills.contexts(),
                                entries: &entries,
                            },
                            &modes,
                            &output,
                            options,
                            mode,
                            group_modes,
                        )
                    })
                };
                let (fragment, selected_mode) = selected
                    .map_or((alternatives.fallback, Mode::Break), |fragment| {
                        (*fragment, Mode::Flat)
                    });
                output.record_group_mode(selected_mode, inherit_flat);
                cursor.enter(fragment.elements, false, Some(mode));
                mode = selected_mode;
            }
            FormatElement::SoftLineIfBreak(id) => {
                if group_modes[id.index()] == Some(Mode::Break) && mode == Mode::Break {
                    if !suffixes.is_empty() {
                        flush_suffixes(
                            &mut suffixes,
                            &mut cursor,
                            Some(source_element),
                            &mut mode,
                            &mut indent,
                        );
                        continue;
                    }
                    output.line(indent, options, mode);
                }
            }
            FormatElement::BreakParent => {}
            FormatElement::Text(text) => output.write(text, options, mode),
            FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                if matches!(element, FormatElement::Line) {
                    output.write(Text::new(" "), options, mode);
                }
            }
            FormatElement::SoftLine | FormatElement::Line | FormatElement::HardLine => {
                if !suffixes.is_empty() {
                    flush_suffixes(
                        &mut suffixes,
                        &mut cursor,
                        Some(source_element),
                        &mut mode,
                        &mut indent,
                    );
                    continue;
                }
                output.line(indent, options, mode);
            }
            FormatElement::LiteralLine | FormatElement::LiteralLineWithIndent => {
                if !suffixes.is_empty() {
                    flush_suffixes(
                        &mut suffixes,
                        &mut cursor,
                        Some(source_element),
                        &mut mode,
                        &mut indent,
                    );
                    continue;
                }
                let next_indent = if matches!(element, FormatElement::LiteralLineWithIndent) {
                    indent
                } else {
                    Indentation::default()
                };
                output.literal_line(next_indent, options, mode);
            }
            FormatElement::IfBreak(text) | FormatElement::IfFlat(text) => {
                let broken = mode == Mode::Break;
                if broken == matches!(element, FormatElement::IfBreak(..)) {
                    output.write(text, options, mode);
                }
            }
            FormatElement::Conditional(conditional)
            | FormatElement::FlatConditional(conditional) => {
                let fragment = match conditional.target.mode(mode, group_modes) {
                    Some(Mode::Break) => conditional.broken,
                    Some(Mode::Flat) => conditional.flat,
                    None => continue,
                };
                cursor.enter(
                    fragment.elements,
                    matches!(element, FormatElement::FlatConditional(_)),
                    None,
                );
            }
            FormatElement::StartLineSuffix { end_offset } => {
                let (contents, remaining) = cursor.current.remaining.split_at(end_offset);
                suffixes.push(LineSuffix {
                    elements: &contents[..contents.len() - 1],
                    mode,
                    indent,
                });
                cursor.current.remaining = remaining;
            }
            FormatElement::EndLineSuffix => {
                unreachable!("line suffix scopes are queued as complete content")
            }
            FormatElement::LineSuffixBoundary => {
                if !suffixes.is_empty() {
                    const HARD_LINE: &[FormatElement<'_>] = &[FormatElement::HardLine];
                    cursor.enter(HARD_LINE, false, None);
                }
            }
            FormatElement::StartGroup { effect, id } => {
                let inherit_flat = mode == Mode::Flat && output.fit_state != FitState::Remeasure;
                let flat = !effect.preferred_breaks()
                    && (inherit_flat
                        || probe.fits(
                            ProbeRequest {
                                current: cursor.current,
                                pending: &cursor.suspended,
                                continuation: None,
                                kind: ContinuationProbe::Group,
                                has_line_suffix: !suffixes.is_empty(),
                                fills: fills.contexts(),
                                entries: &entries,
                            },
                            &modes,
                            &output,
                            options,
                            mode,
                            group_modes,
                        ));
                let group_mode = if flat { Mode::Flat } else { Mode::Break };
                output.record_group_mode(group_mode, inherit_flat);
                if let Some(id) = id {
                    group_modes[id.index()] = Some(group_mode);
                }
                modes.push(GroupState { restore_mode: mode });
                mode = group_mode;
            }
            FormatElement::StartGroupWithFlatWidth {
                summary,
                end_offset,
            } => {
                let inherit_flat = mode == Mode::Flat && output.fit_state != FitState::Remeasure;
                let flat = inherit_flat
                    || probe.fits(
                        ProbeRequest {
                            current: cursor.current,
                            pending: &cursor.suspended,
                            continuation: None,
                            kind: PreparedGroup {
                                summary,
                                end_offset,
                            },
                            has_line_suffix: !suffixes.is_empty(),
                            fills: fills.contexts(),
                            entries: &entries,
                        },
                        &modes,
                        &output,
                        options,
                        mode,
                        group_modes,
                    );
                let group_mode = if flat { Mode::Flat } else { Mode::Break };
                output.record_group_mode(group_mode, inherit_flat);
                modes.push(GroupState { restore_mode: mode });
                mode = group_mode;
            }
            FormatElement::EndGroup => {
                mode = modes
                    .pop()
                    .expect("group scopes must close after opening")
                    .restore_mode;
            }
            FormatElement::StartIndent => {
                indents.push(indent);
                indent = indent.indented(options);
            }
            FormatElement::StartAbsoluteIndent { width } => {
                indents.push(indent);
                indent = Indentation::absolute(width, options);
            }
            FormatElement::StartIndentIfBreak { id, end_offset } => {
                let binding = group_modes[id.index()];
                if flat || binding.is_none() {
                    let (contents, remaining) = cursor.current.remaining.split_at(end_offset);
                    cursor.current.remaining = remaining;
                    if binding.is_some() {
                        cursor.enter(&contents[..contents.len() - 1], true, None);
                    }
                } else {
                    indents.push(indent);
                    if binding == Some(Mode::Break) {
                        indent = indent.indented(options);
                    }
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
            FormatElement::StartFill => fills.start(mode),
            FormatElement::EndFill => fills.end(),
            FormatElement::StartEntry { .. } => {
                let entry_mode = fills.entry(
                    &mut probe,
                    FillEntry {
                        current: cursor.current,
                        mode,
                        has_line_suffix: !suffixes.is_empty(),
                    },
                    &modes,
                    &output,
                    options,
                    group_modes,
                );
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
    debug_assert_eq!(indent.levels, 0, "document must close every indent scope");
    debug_assert!(indents.is_empty(), "document must close every indent scope");
    debug_assert_eq!(
        indent.alignment, 0,
        "document must close every alignment scope"
    );
    debug_assert!(modes.is_empty(), "document must close every group scope");
    debug_assert!(fills.is_empty(), "document must close every fill scope");
    debug_assert!(entries.is_empty(), "document must close every fill entry");
    debug_assert!(
        cursor.suffix_scopes.is_empty(),
        "document must close every suffix scope"
    );
    output.write_indent(options);
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
    }

    #[inline]
    fn write_indent(&mut self, options: FormatOptions) {
        if self.pending_indent.levels != 0 || self.pending_indent.alignment != 0 {
            self.write_pending_indent(options);
        }
    }

    // Keep indentation materialization outside the ordinary text path.
    #[inline(never)]
    fn write_pending_indent(&mut self, options: FormatOptions) {
        let indent = std::mem::take(&mut self.pending_indent);
        match options.indent_style {
            IndentStyle::Space => self.text.extend(std::iter::repeat_n(
                ' ',
                indent.levels * usize::from(options.indent_width.value()) + indent.alignment,
            )),
            IndentStyle::Tab => {
                self.text.extend(std::iter::repeat_n('\t', indent.levels));
                self.text.extend(std::iter::repeat_n(' ', indent.alignment));
            }
        }
    }

    fn write(&mut self, text: Text<'_>, options: FormatOptions, mode: Mode) {
        self.write_indent(options);
        self.text.push_str(text.source);
        let (column, multiline) = text.last_line(self.column);
        self.column = column;
        if multiline {
            self.invalidate_line(mode);
        }
    }

    fn line(&mut self, indent: Indentation, options: FormatOptions, mode: Mode) {
        self.pending_indent = Indentation::default();
        self.text
            .truncate(self.text.trim_end_matches([' ', '\t']).len());
        self.literal_line(indent, options, mode);
    }

    fn literal_line(&mut self, indent: Indentation, options: FormatOptions, mode: Mode) {
        self.write_indent(options);
        self.invalidate_line(mode);
        self.text.push_str(options.line_ending.as_str());
        self.column = indent.width(options);
        self.pending_indent = indent;
    }
}

fn flush_suffixes<'a>(
    suffixes: &mut Vec<LineSuffix<'a>>,
    cursor: &mut Cursor<'a>,
    line: Option<&'a FormatElement<'a>>,
    mode: &mut Mode,
    indent: &mut Indentation,
) {
    debug_assert!(!suffixes.is_empty(), "flush requires pending suffixes");
    if let Some(line) = line {
        cursor.enter(std::slice::from_ref(line), false, None);
    }
    for suffix in suffixes.drain(..).rev() {
        cursor.enter(suffix.elements, false, Some(*mode));
        cursor.suffix_scopes.push(SuffixScope {
            frame_depth: cursor.suspended.len(),
            restore_indent: *indent,
        });
        *mode = suffix.mode;
        *indent = suffix.indent;
    }
}

struct ProbeRequest<'a, 'pending, K> {
    current: Frame<'a>,
    pending: &'pending [Frame<'a>],
    continuation: Option<Frame<'a>>,
    kind: K,
    has_line_suffix: bool,
    fills: &'pending [FillState],
    entries: &'pending [Mode],
}

struct PreparedGroup {
    summary: FlatSummary,
    end_offset: usize,
}

impl ProbeKind for PreparedGroup {
    const BOUNDED: bool = false;

    fn continuation(&self) -> Option<ContinuationProbe> {
        Some(ContinuationProbe::Group)
    }

    fn prepare(
        &self,
        current: &mut Frame<'_>,
        column: usize,
        pending_line: &mut bool,
    ) -> Option<usize> {
        current.remaining = &current.remaining[self.end_offset..];
        Some(self.summary.advance(column, pending_line))
    }
}

struct Probe<'a> {
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
        committed_groups: &[Option<Mode>],
    ) -> bool {
        let ProbeRequest {
            mut current,
            pending: pending_frames,
            continuation,
            mut kind,
            mut has_line_suffix,
            fills: parent_fills,
            entries: parent_entries,
        } = request;
        let Self {
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
        let mut pending_line_space = false;
        let mut column = output.column;
        modes.clear();
        frames.clear();
        if let Some(continuation) = continuation {
            frames.push(continuation);
        }
        let mut mode = Mode::Flat;
        if let Some(next_column) = kind.prepare(&mut current, column, &mut pending_line_space) {
            column = next_column;
            mode = incoming_mode;
        } else if let Some(ContinuationProbe::Group) = kind.continuation() {
            modes.push(GroupState {
                restore_mode: incoming_mode,
            });
        }
        let mut cursor = ProbeCursor {
            current,
            frames,
            parents: pending_frames,
            parent_len: pending_frames.len(),
        };
        let mut parent_len = parents.len();
        if column > usize::from(options.line_width) {
            return false;
        }
        // Active text and committed flat line spaces advance the probe column.
        while let Some((element, flat)) = cursor.next(&mut mode) {
            let flattened;
            let element = if flat {
                let Some(element) = element.flattened() else {
                    continue;
                };
                flattened = element;
                &flattened
            } else {
                element
            };
            match *element {
                FormatElement::Text(text)
                | FormatElement::IfBreak(text)
                | FormatElement::IfFlat(text) => {
                    let active = match element {
                        FormatElement::IfBreak(_) => mode == Mode::Break,
                        FormatElement::IfFlat(_) => mode == Mode::Flat,
                        _ => true,
                    };
                    if !active {
                        continue;
                    }
                    if pending_line_space {
                        // Prettier fits defers `line` spaces until nonempty text,
                        // independently of the number of spaces actually printed.
                        column += 1;
                    }
                    pending_line_space = false;
                    let (next_column, multiline) = text.first_line(column);
                    column = next_column;
                    if multiline {
                        return column <= usize::from(options.line_width);
                    }
                    if column > usize::from(options.line_width) {
                        return false;
                    }
                }
                FormatElement::StartLineSuffix { end_offset } => {
                    if kind.layout_dependency() {
                        return true;
                    }
                    has_line_suffix = true;
                    cursor.current.remaining = &cursor.current.remaining[end_offset..];
                }
                FormatElement::EndLineSuffix => {
                    unreachable!("fit probes skip complete line suffix scopes")
                }
                FormatElement::LineSuffixBoundary => {
                    if has_line_suffix {
                        return false;
                    }
                }
                FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                    if matches!(element, FormatElement::Line) {
                        if kind.line(pending_line_space) {
                            return true;
                        }
                        pending_line_space = true;
                    }
                }
                FormatElement::SoftLine
                | FormatElement::Line
                | FormatElement::HardLine
                | FormatElement::LiteralLine
                | FormatElement::LiteralLineWithIndent => return true,
                FormatElement::StartGroup { effect, id } => {
                    if id.is_some() && kind.layout_dependency() {
                        return true;
                    }
                    let expand = effect.preferred_breaks();
                    if expand && K::BOUNDED {
                        return false;
                    }
                    let group_mode = if expand { Mode::Break } else { mode };
                    modes.push(GroupState { restore_mode: mode });
                    mode = group_mode;
                }
                FormatElement::StartGroupWithFlatWidth {
                    summary,
                    end_offset,
                } => {
                    // A fit summary can coalesce line spaces that printing
                    // preserves. Do not extend a batch across this opaque scope.
                    if kind.layout_dependency() {
                        return true;
                    }
                    if mode == Mode::Flat {
                        column = summary.advance(column, &mut pending_line_space);
                        if column > usize::from(options.line_width) {
                            return false;
                        }
                        cursor.current.remaining = &cursor.current.remaining[end_offset..];
                    } else {
                        modes.push(GroupState { restore_mode: mode });
                    }
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
                FormatElement::Conditional(conditional)
                | FormatElement::FlatConditional(conditional) => {
                    if matches!(conditional.target, ConditionTarget::Group(_))
                        && kind.layout_dependency()
                    {
                        return true;
                    }
                    let fragment =
                        if conditional.target.mode(mode, committed_groups) == Some(Mode::Break) {
                            conditional.broken
                        } else {
                            conditional.flat
                        };
                    cursor.enter(
                        fragment.elements,
                        matches!(element, FormatElement::FlatConditional(_)),
                        None,
                    );
                }
                FormatElement::Fragment(fragment) => cursor.enter(fragment.elements, false, None),
                FormatElement::FlatFragment(fragment) => {
                    cursor.enter(fragment.0.elements, true, None)
                }
                FormatElement::Alternatives(alternatives) => {
                    if kind.layout_dependency() {
                        return true;
                    }
                    let fragment = if mode == Mode::Flat {
                        alternatives.first()
                    } else {
                        alternatives.fallback
                    };
                    cursor.enter(fragment.elements, false, Some(mode));
                }
                FormatElement::SoftLineIfBreak(id) => {
                    if kind.layout_dependency() {
                        return true;
                    }
                    if mode == Mode::Flat {
                        continue;
                    }
                    let named = committed_groups[id.index()];
                    if named == Some(Mode::Break) {
                        return true;
                    }
                }
                FormatElement::BreakParent => {}
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
                FormatElement::StartFill => {
                    if kind.layout_dependency() {
                        return true;
                    }
                }
                FormatElement::EndFill => {
                    if kind.end_fill() {
                        return true;
                    }
                }
                FormatElement::StartEntry { .. } => kind.start_entry(column, pending_line_space),
                FormatElement::EndEntry => {
                    if kind.end_entry() {
                        return true;
                    }
                }
                FormatElement::StartIndentIfBreak { .. } => {
                    if kind.layout_dependency() {
                        return true;
                    }
                }
                FormatElement::StartIndent
                | FormatElement::StartAbsoluteIndent { .. }
                | FormatElement::EndIndent
                | FormatElement::StartAlign { .. }
                | FormatElement::EndAlign { .. } => {}
            }
        }
        column <= usize::from(options.line_width)
    }
}
