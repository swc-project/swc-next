//! Interpretation of flat formatting instructions and bounded fit probes.

use std::{fmt, ops::Range};

use super::{
    AlignText, ConditionTarget, Doc, FlatSummary, Group, GroupRequest, Instruction, indent,
    layout::Mode,
    text::{Text, Width},
};
use crate::{IndentStyle, LineEnding, PrintOptions};
use swc_next_allocator::vec::ArenaVec;

#[derive(Debug, PartialEq, Eq)]
/// Printed text and the range delimited by its emitted cursor markers.
pub struct PrintResult {
    /// Text with the requested output line endings.
    pub text: String,
    /// UTF-8 byte offsets into the final text.
    pub cursor_range: Option<Range<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// More than two cursor markers were emitted.
pub struct CursorError;

impl fmt::Display for CursorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("document printed more than two cursor markers")
    }
}

impl std::error::Error for CursorError {}

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

struct FillEntry<'a, 'pending> {
    current: Frame<'a>,
    pending: &'pending [Frame<'a>],
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
    fn entry<'a, const SOURCE: bool, const EXTENDED: bool>(
        &mut self,
        probe: &mut Probe<'a, SOURCE, EXTENDED>,
        entry: FillEntry<'a, '_>,
        parents: &[GroupState],
        output: &OutputView,
        options: PrintOptions,
        committed_groups: &[Option<Mode>],
    ) -> Mode {
        let FillEntry {
            current,
            pending,
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
                pending,
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
    remaining: &'a [Instruction<'a>],
    flat: bool,
    restore_mode: Option<Mode>,
}

impl<'a> Frame<'a> {
    fn new(elements: &'a [Instruction<'a>]) -> Self {
        Self {
            remaining: elements,
            flat: false,
            restore_mode: None,
        }
    }
}

struct Cursor<'a, S> {
    current: Frame<'a>,
    suspended: Vec<Frame<'a>>,
    suffix_scopes: Vec<SuffixScope<S>>,
}

impl<'a, S: Copy> Cursor<'a, S> {
    fn enter(&mut self, elements: &'a [Instruction<'a>], flat: bool, restore_mode: Option<Mode>) {
        self.suspended.push(self.current);
        self.current = Frame {
            flat,
            restore_mode,
            ..Frame::new(elements)
        };
    }

    fn next(&mut self, mode: &mut Mode, indent: &mut S) -> Option<(&'a Instruction<'a>, bool)> {
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
    fn enter(&mut self, elements: &'a [Instruction<'a>], flat: bool, restore_mode: Option<Mode>) {
        self.frames.push(self.current);
        self.current = Frame {
            flat,
            restore_mode,
            ..Frame::new(elements)
        };
    }

    fn next(&mut self, mode: &mut Mode) -> Option<(&'a Instruction<'a>, bool)> {
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
struct SuffixScope<S> {
    frame_depth: usize,
    restore_indent: S,
}

struct LineSuffix<'a, S> {
    elements: &'a [Instruction<'a>],
    mode: Mode,
    indent: S,
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

struct OutputView {
    column: usize,
    fit_state: FitState,
}

struct Output<'a, M: IndentModel<'a>, const EXTENDED: bool> {
    text: String,
    cursors: [Option<usize>; 2],
    view: OutputView,
    pending_indent: M::State,
    model: M,
    marker: std::marker::PhantomData<&'a ()>,
}

#[derive(Clone, Copy, Default)]
struct Indentation {
    levels: usize,
    alignment: usize,
    alignments: usize,
}

impl Indentation {
    fn absolute(width: usize, options: PrintOptions) -> Self {
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

    fn width(self, options: PrintOptions) -> usize {
        self.levels * usize::from(options.indent_width.value()) + self.alignment
    }

    fn indented(mut self, options: PrintOptions) -> Self {
        if options.indent_style == IndentStyle::Tab {
            self.levels += self.alignments;
            self.alignment = 0;
            self.alignments = 0;
        }
        self.levels += 1;
        self
    }
}

trait IndentModel<'a>: Default {
    type State: Copy + Default;
    const RICH: bool;
    fn indent(&mut self, state: Self::State, options: PrintOptions) -> Self::State;
    fn align(&mut self, state: Self::State, width: usize) -> Self::State;
    fn unalign(&mut self, state: Self::State, width: usize) -> Self::State;
    fn absolute(&mut self, state: Self::State, width: usize, options: PrintOptions) -> Self::State;
    fn align_text(
        &mut self,
        state: Self::State,
        text: &'a AlignText<'a>,
        options: PrintOptions,
    ) -> Self::State;
    fn dedent(&mut self, state: Self::State, options: PrintOptions) -> Self::State;
    fn mark_root(&mut self, state: Self::State) -> Self::State;
    fn root(&self, state: Self::State) -> Self::State;
    fn width(&self, state: Self::State, options: PrintOptions) -> usize;
    fn is_empty(&self, state: Self::State) -> bool;
    fn non_whitespace(&self, state: Self::State) -> bool;
    fn emit(&mut self, state: Self::State, output: &mut String, options: PrintOptions);
}

#[derive(Default)]
struct SimpleIndent;
impl<'a> IndentModel<'a> for SimpleIndent {
    type State = Indentation;
    const RICH: bool = false;
    fn indent(&mut self, s: Indentation, o: PrintOptions) -> Indentation {
        s.indented(o)
    }
    fn align(&mut self, mut s: Indentation, width: usize) -> Indentation {
        s.alignment += width;
        s.alignments += 1;
        s
    }
    fn unalign(&mut self, mut s: Indentation, width: usize) -> Indentation {
        s.alignment = s
            .alignment
            .checked_sub(width)
            .expect("alignment scopes must close after opening");
        s.alignments = s
            .alignments
            .checked_sub(1)
            .expect("alignment scopes must close after opening");
        s
    }
    fn absolute(&mut self, _: Indentation, width: usize, o: PrintOptions) -> Indentation {
        Indentation::absolute(width, o)
    }
    fn align_text(&mut self, _: Indentation, _: &'a AlignText<'a>, _: PrintOptions) -> Indentation {
        unreachable!("rich indent flag is required")
    }
    fn dedent(&mut self, _: Indentation, _: PrintOptions) -> Indentation {
        unreachable!("rich indent flag is required")
    }
    fn mark_root(&mut self, _: Indentation) -> Indentation {
        unreachable!("rich indent flag is required")
    }
    fn root(&self, _: Indentation) -> Indentation {
        Indentation::default()
    }
    fn width(&self, s: Indentation, o: PrintOptions) -> usize {
        s.width(o)
    }
    fn is_empty(&self, s: Indentation) -> bool {
        s.levels == 0 && s.alignment == 0
    }
    fn non_whitespace(&self, _: Indentation) -> bool {
        false
    }
    fn emit(&mut self, s: Indentation, text: &mut String, o: PrintOptions) {
        match o.indent_style {
            IndentStyle::Space => text.extend(std::iter::repeat_n(
                ' ',
                s.levels * usize::from(o.indent_width.value()) + s.alignment,
            )),
            IndentStyle::Tab => {
                text.extend(std::iter::repeat_n('\t', s.levels));
                text.extend(std::iter::repeat_n(' ', s.alignment));
            }
        }
    }
}

impl<'a> IndentModel<'a> for indent::Rich<'a> {
    type State = indent::State;
    const RICH: bool = true;
    fn indent(&mut self, s: Self::State, o: PrintOptions) -> Self::State {
        self.indent(s, o)
    }
    fn align(&mut self, s: Self::State, width: usize) -> Self::State {
        self.align(s, width)
    }
    fn unalign(&mut self, _: Self::State, _: usize) -> Self::State {
        unreachable!("rich align scopes restore state")
    }
    fn absolute(&mut self, s: Self::State, width: usize, o: PrintOptions) -> Self::State {
        self.absolute(s, width, o)
    }
    fn align_text(&mut self, s: Self::State, t: &'a AlignText<'a>, o: PrintOptions) -> Self::State {
        self.text(s, t, o)
    }
    fn dedent(&mut self, s: Self::State, o: PrintOptions) -> Self::State {
        indent::Rich::dedent(self, s, o)
    }
    fn mark_root(&mut self, s: Self::State) -> Self::State {
        self.mark_root(s)
    }
    fn root(&self, s: Self::State) -> Self::State {
        self.root(s)
    }
    fn width(&self, s: Self::State, _: PrintOptions) -> usize {
        self.width(s)
    }
    fn is_empty(&self, s: Self::State) -> bool {
        indent::Rich::is_empty(self, s)
    }
    fn non_whitespace(&self, s: Self::State) -> bool {
        self.non_whitespace(s)
    }
    fn emit(&mut self, s: Self::State, text: &mut String, _: PrintOptions) {
        self.emit(s, text)
    }
}

pub(super) fn print<'a>(
    doc: Doc<'a>,
    options: PrintOptions,
    track_cursor: bool,
) -> Result<PrintResult, CursorError> {
    if doc.root.extended_print() || track_cursor {
        print_for::<true>(doc, options, track_cursor)
    } else {
        print_for::<false>(doc, options, false)
    }
}

fn print_for<'a, const EXTENDED: bool>(
    doc: Doc<'a>,
    options: PrintOptions,
    track_cursor: bool,
) -> Result<PrintResult, CursorError> {
    match (doc.root.has_source_groups(), doc.root.rich_indent()) {
        (false, false) => print_impl::<false, EXTENDED, SimpleIndent>(doc, options, track_cursor),
        (true, false) => print_impl::<true, EXTENDED, SimpleIndent>(doc, options, track_cursor),
        (false, true) => {
            print_impl::<false, EXTENDED, indent::Rich<'a>>(doc, options, track_cursor)
        }
        (true, true) => print_impl::<true, EXTENDED, indent::Rich<'a>>(doc, options, track_cursor),
    }
}

// Keep the interpreter's register allocation independent of syntax construction.
#[inline(never)]
fn print_impl<'a, const SOURCE: bool, const EXTENDED: bool, M: IndentModel<'a>>(
    doc: Doc<'a>,
    options: PrintOptions,
    track_cursor: bool,
) -> Result<PrintResult, CursorError> {
    let Doc {
        root,
        allocator,
        group_count,
        ..
    } = doc;
    let elements = root.elements();
    let mut output = Output::<M, EXTENDED> {
        text: String::new(),
        cursors: [None; 2],
        view: OutputView {
            column: 0,
            fit_state: FitState::Measured,
        },
        pending_indent: M::State::default(),
        model: M::default(),
        marker: std::marker::PhantomData,
    };
    let mut suffixes = Vec::new();
    let mut indent = M::State::default();
    let mut indents = ArenaVec::with_capacity_in(8, allocator);
    let mut modes = ArenaVec::with_capacity_in(8, allocator);
    let group_count = group_count as usize;
    let mut group_modes = ArenaVec::with_capacity_in(group_count, allocator);
    group_modes.resize(group_count, None);
    let group_modes = group_modes.into_bump_slice_mut();
    let mut mode = Mode::Break;
    let mut probe = Probe::<SOURCE, EXTENDED> {
        modes: Vec::new(),
        frames: Vec::new(),
        fills: Vec::new(),
        entries: Vec::new(),
        group_effects: (SOURCE && root.requires_group_analysis())
            .then(|| super::breaks::analyze(elements)),
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
            Instruction::Fragment { elements, .. } => cursor.enter(elements.get(), false, None),
            Instruction::FlatFragment { elements, .. } => cursor.enter(elements.get(), true, None),
            Instruction::CompactGroup(group) => {
                let inherit_flat =
                    mode == Mode::Flat && output.view.fit_state != FitState::Remeasure;
                let (selected, selected_mode) = group.select(inherit_flat, |fragment| {
                    probe.fits(
                        ProbeRequest {
                            current: Frame {
                                restore_mode: Some(mode),
                                ..Frame::new(fragment.elements())
                            },
                            pending: &cursor.suspended,
                            continuation: Some(cursor.current),
                            kind: ContinuationProbe::Candidate,
                            has_line_suffix: !suffixes.is_empty(),
                            fills: fills.contexts(),
                            entries: &entries,
                        },
                        &modes,
                        &output.view,
                        options,
                        mode,
                        group_modes,
                    )
                });
                output.record_group_mode(selected_mode, inherit_flat);
                cursor.enter(selected.elements(), false, Some(mode));
                mode = selected_mode;
            }
            Instruction::SourceGroup(group) => {
                if !SOURCE {
                    unreachable!("source groups require their printer specialization");
                }
                let inherit_flat =
                    mode == Mode::Flat && output.view.fit_state != FitState::Remeasure;
                let (selected, selected_mode) = group.select(inherit_flat, |fragment| {
                    probe.fits(
                        ProbeRequest {
                            current: Frame {
                                restore_mode: Some(mode),
                                ..Frame::new(fragment.elements())
                            },
                            pending: &cursor.suspended,
                            continuation: Some(cursor.current),
                            kind: ContinuationProbe::Candidate,
                            has_line_suffix: !suffixes.is_empty(),
                            fills: fills.contexts(),
                            entries: &entries,
                        },
                        &modes,
                        &output.view,
                        options,
                        mode,
                        group_modes,
                    )
                });
                output.record_group_mode(selected_mode, inherit_flat);
                if let Some(id) = group.id() {
                    group_modes[id.index()] = Some(selected_mode);
                }
                cursor.enter(selected.elements(), false, Some(mode));
                mode = selected_mode;
            }
            Instruction::SoftLineIfGroupBreaks(id) => {
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
            Instruction::BreakParent | Instruction::StartLabel(_) | Instruction::EndLabel => {}
            Instruction::Cursor => {
                if EXTENDED && track_cursor {
                    output.write_indent(options);
                    let marker = output
                        .cursors
                        .iter_mut()
                        .find(|marker| marker.is_none())
                        .ok_or(CursorError)?;
                    *marker = Some(output.text.len());
                }
            }
            Instruction::Trim => {
                if EXTENDED {
                    output.write_indent(options);
                    let trimmed = output.trim();
                    output.view.column = output.view.column.saturating_sub(trimmed);
                }
            }
            Instruction::Text { source, width } => {
                output.write(Text::from_parts(source, width), options)
            }
            Instruction::SoftLine
            | Instruction::NormalLine
            | Instruction::SoftLiteralLine
            | Instruction::NormalLiteralLine
                if mode == Mode::Flat =>
            {
                if matches!(
                    element,
                    Instruction::NormalLine | Instruction::NormalLiteralLine
                ) {
                    output.write(Text::new(" "), options);
                }
            }
            Instruction::SoftLine
            | Instruction::NormalLine
            | Instruction::HardLine
            | Instruction::HardLineOnly => {
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
            Instruction::LiteralLine
            | Instruction::SoftLiteralLine
            | Instruction::NormalLiteralLine
            | Instruction::LiteralLineOnly
            | Instruction::LiteralLineAtCurrentIndent => {
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
                let next_indent = if matches!(element, Instruction::LiteralLineAtCurrentIndent) {
                    indent
                } else {
                    output.model.root(indent)
                };
                output.literal_line(next_indent, options, mode);
            }
            Instruction::TextIfBreak { source, width }
            | Instruction::TextIfFlat { source, width } => {
                let broken = mode == Mode::Break;
                if broken == matches!(element, Instruction::TextIfBreak { .. }) {
                    output.write(Text::from_parts(source, width), options);
                }
            }
            Instruction::IfBreak(conditional) | Instruction::FlatIfBreak(conditional) => {
                let fragment = match conditional.target.mode(mode, group_modes) {
                    Some(Mode::Break) => conditional.broken,
                    Some(Mode::Flat) => conditional.flat,
                    None => continue,
                };
                cursor.enter(
                    fragment.elements(),
                    matches!(element, Instruction::FlatIfBreak(_)),
                    None,
                );
            }
            Instruction::StartLineSuffix { end_offset } => {
                let (contents, remaining) = cursor.current.remaining.split_at(end_offset as usize);
                suffixes.push(LineSuffix {
                    elements: &contents[..contents.len() - 1],
                    mode,
                    indent,
                });
                cursor.current.remaining = remaining;
            }
            Instruction::EndLineSuffix => {
                unreachable!("line suffix scopes are queued as complete content")
            }
            Instruction::LineSuffixBoundary => {
                if !suffixes.is_empty() {
                    // Layout is already selected; flushing only needs a newline.
                    const LINE_BREAK: &[Instruction<'_>] = &[Instruction::HardLineOnly];
                    cursor.enter(LINE_BREAK, false, None);
                }
            }
            Instruction::StartGroup {
                request,
                effect,
                id,
                ..
            } => {
                let expand =
                    group_expands::<SOURCE>(&probe.group_effects, request, effect, source_element);
                let inherit_flat =
                    mode == Mode::Flat && output.view.fit_state != FitState::Remeasure;
                let flat = !expand
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
                            &output.view,
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
            Instruction::StartGroupWithFlatWidth {
                summary,
                end_offset,
            } => {
                let inherit_flat =
                    mode == Mode::Flat && output.view.fit_state != FitState::Remeasure;
                let request = ProbeRequest {
                    current: cursor.current,
                    pending: &cursor.suspended,
                    continuation: None,
                    kind: ContinuationProbe::Group,
                    has_line_suffix: !suffixes.is_empty(),
                    fills: fills.contexts(),
                    entries: &entries,
                };
                let flat = inherit_flat
                    || if EXTENDED {
                        probe.fits(request, &modes, &output.view, options, mode, group_modes)
                    } else {
                        probe.fits(
                            ProbeRequest {
                                current: request.current,
                                pending: request.pending,
                                continuation: request.continuation,
                                kind: PreparedGroup {
                                    summary,
                                    end_offset: end_offset as usize,
                                },
                                has_line_suffix: request.has_line_suffix,
                                fills: request.fills,
                                entries: request.entries,
                            },
                            &modes,
                            &output.view,
                            options,
                            mode,
                            group_modes,
                        )
                    };
                let group_mode = if flat { Mode::Flat } else { Mode::Break };
                output.record_group_mode(group_mode, inherit_flat);
                modes.push(GroupState { restore_mode: mode });
                mode = group_mode;
            }
            Instruction::EndGroup => {
                mode = modes
                    .pop()
                    .expect("group scopes must close after opening")
                    .restore_mode;
            }
            Instruction::StartIndent => {
                indents.push(indent);
                indent = output.model.indent(indent, options);
            }
            Instruction::StartAbsoluteIndent { width } => {
                indents.push(indent);
                indent = output.model.absolute(indent, width as usize, options);
            }
            Instruction::StartIndentIfBreak {
                target,
                negate,
                end_offset,
            } => {
                let binding = target.mode(mode, group_modes);
                if flat || binding.is_none() {
                    let (contents, remaining) =
                        cursor.current.remaining.split_at(end_offset as usize);
                    cursor.current.remaining = remaining;
                    if binding.is_some() {
                        let (body, end) = contents.split_at(contents.len() - 1);
                        if binding == Some(if negate { Mode::Flat } else { Mode::Break }) {
                            indents.push(indent);
                            indent = output.model.indent(indent, options);
                            // Restore indentation after the projected body completes.
                            cursor.enter(end, false, None);
                        }
                        cursor.enter(body, true, None);
                    }
                } else {
                    indents.push(indent);
                    if binding == Some(if negate { Mode::Flat } else { Mode::Break }) {
                        indent = output.model.indent(indent, options);
                    }
                }
            }
            Instruction::EndIndent => {
                indent = indents
                    .pop()
                    .expect("indent scopes must close after opening");
            }
            Instruction::StartAlign { width } => {
                if M::RICH {
                    indents.push(indent);
                }
                indent = output.model.align(indent, width as usize);
            }
            Instruction::EndAlign { width } => {
                if M::RICH {
                    indent = indents
                        .pop()
                        .expect("alignment scopes must close after opening");
                } else {
                    indent = output.model.unalign(indent, width as usize);
                }
            }
            Instruction::StartAlignText(text) => {
                indents.push(indent);
                indent = output.model.align_text(indent, text, options);
            }
            Instruction::StartDedent => {
                indents.push(indent);
                indent = output.model.dedent(indent, options);
            }
            Instruction::StartMarkRoot => {
                indents.push(indent);
                indent = output.model.mark_root(indent);
            }
            Instruction::StartDedentToRoot => {
                indents.push(indent);
                indent = output.model.root(indent);
            }
            Instruction::EndRichIndent(_) => {
                indent = indents
                    .pop()
                    .expect("indent scopes must close after opening");
            }
            Instruction::StartFill => fills.start(mode),
            Instruction::EndFill => fills.end(),
            Instruction::StartFillPart { .. } => {
                let entry_mode = fills.entry(
                    &mut probe,
                    FillEntry {
                        current: cursor.current,
                        pending: &cursor.suspended,
                        mode,
                        has_line_suffix: !suffixes.is_empty(),
                    },
                    &modes,
                    &output.view,
                    options,
                    group_modes,
                );
                entries.push(mode);
                mode = entry_mode;
            }
            Instruction::EndFillPart => {
                mode = entries
                    .pop()
                    .expect("fill entries must close after opening");
            }
        }
    }
    debug_assert!(
        output.model.is_empty(indent),
        "document must close every indent scope"
    );
    debug_assert!(indents.is_empty(), "document must close every indent scope");
    debug_assert!(modes.is_empty(), "document must close every group scope");
    debug_assert!(fills.is_empty(), "document must close every fill scope");
    debug_assert!(entries.is_empty(), "document must close every fill entry");
    debug_assert!(
        cursor.suffix_scopes.is_empty(),
        "document must close every suffix scope"
    );
    output.write_indent(options);
    let [start, end] = output.cursors;
    Ok(PrintResult {
        text: output.text,
        cursor_range: start.zip(end).map(|(start, end)| start..end),
    })
}

impl<'a, M: IndentModel<'a>, const EXTENDED: bool> Output<'a, M, EXTENDED> {
    // An inherited mode consumes the request without refreshing its measurement.
    fn record_group_mode(&mut self, mode: Mode, inherited: bool) {
        if mode == Mode::Break {
            self.view.fit_state = FitState::Unmeasured;
        } else if !inherited {
            self.view.fit_state = FitState::Measured;
        }
    }

    fn invalidate_line(&mut self, mode: Mode) {
        if mode == Mode::Flat {
            self.view.fit_state = FitState::Remeasure;
        } else if self.view.fit_state == FitState::Measured {
            self.view.fit_state = FitState::Unmeasured;
        }
    }

    #[inline]
    fn write_indent(&mut self, options: PrintOptions) {
        if !self.model.is_empty(self.pending_indent) {
            self.write_pending_indent(options);
        }
    }

    #[inline(never)]
    fn write_pending_indent(&mut self, options: PrintOptions) {
        let indent = std::mem::take(&mut self.pending_indent);
        self.model.emit(indent, &mut self.text, options);
    }

    #[inline]
    fn write(&mut self, text: Text<'_>, options: PrintOptions) {
        self.write_parts(text.source(), text.width(), options);
    }

    // Keep common text emission inline in both interpreter specializations.
    #[inline(always)]
    fn write_parts(&mut self, source: &str, width: Width, options: PrintOptions) {
        if EXTENDED && source.is_empty() {
            return;
        }
        self.write_indent(options);
        if options.line_ending != LineEnding::Lf && source.contains('\n') {
            self.write_text_lines(source, options.line_ending);
        } else {
            self.text.push_str(source);
        }
        self.view.column = Text::from_source_parts(source, width).advance_column(self.view.column);
    }

    fn write_text_lines(&mut self, source: &str, ending: LineEnding) {
        let mut lines = source.split('\n');
        self.text.push_str(lines.next().unwrap_or_default());
        for line in lines {
            self.text.push_str(ending.as_str());
            self.text.push_str(line);
        }
    }

    #[inline]
    fn trim(&mut self) -> usize {
        let len = self.text.trim_end_matches([' ', '\t']).len();
        let trimmed = self.text.len() - len;
        self.text.truncate(len);
        if EXTENDED && trimmed != 0 {
            for position in self.cursors.iter_mut().flatten() {
                *position = (*position).min(len);
            }
        }
        trimmed
    }

    // The specialized interpreters emit ordinary lines on their hot paths.
    #[inline(always)]
    fn line(&mut self, indent: M::State, options: PrintOptions, mode: Mode) {
        if self.model.non_whitespace(self.pending_indent) {
            self.write_indent(options);
        }
        self.pending_indent = M::State::default();
        self.trim();
        self.literal_line(indent, options, mode);
    }

    fn literal_line(&mut self, indent: M::State, options: PrintOptions, mode: Mode) {
        self.write_indent(options);
        self.invalidate_line(mode);
        self.text.push_str(options.line_ending.as_str());
        self.view.column = self.model.width(indent, options);
        self.pending_indent = indent;
    }
}

// Source-free printers use cached effects directly. A source-bearing document
// consults graph facts only when qualification required that analysis.
#[inline(always)]
fn group_expands<const SOURCE: bool>(
    effects: &Option<rustc_hash::FxHashMap<usize, bool>>,
    request: GroupRequest,
    cached: super::BreakEffect,
    source: &Instruction<'_>,
) -> bool {
    if SOURCE && let Some(effects) = effects {
        matches!(request, GroupRequest::Expanded)
            || effects
                .get(&(source as *const _ as usize))
                .copied()
                .unwrap_or(false)
    } else {
        cached.preferred_breaks()
    }
}

fn flush_suffixes<'a, S: Copy>(
    suffixes: &mut Vec<LineSuffix<'a, S>>,
    cursor: &mut Cursor<'a, S>,
    line: Option<&'a Instruction<'a>>,
    mode: &mut Mode,
    indent: &mut S,
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

struct Probe<'a, const SOURCE: bool, const EXTENDED: bool> {
    modes: Vec<GroupState>,
    frames: Vec<Frame<'a>>,
    fills: Vec<FillContext>,
    entries: Vec<Mode>,
    group_effects: Option<rustc_hash::FxHashMap<usize, bool>>,
}

impl<'a, const SOURCE: bool, const EXTENDED: bool> Probe<'a, SOURCE, EXTENDED> {
    fn fits<K: ProbeKind>(
        &mut self,
        request: ProbeRequest<'a, '_, K>,
        parents: &[GroupState],
        output: &OutputView,
        options: PrintOptions,
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
            group_effects,
        } = self;
        let mut fills = BorrowedStack::new(parent_fills, fills);
        let mut entries = BorrowedStack::new(parent_entries, entries);
        // Newly entered Fills have no committed separator modes. In a valid,
        // balanced Fill, each entry inherits the Fill mode; its groups restore
        // that mode before EndFillPart, so part push/pop pairs are identities.
        // Count these Fills instead of storing their scopes. Their entry tags
        // must not consume the borrowed contexts of Fills already being printed.
        let mut uncommitted_fills = 0usize;
        let mut pending_line_space = false;
        let mut trailing_space = 0;
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
        while let Some((source_element, flat)) = cursor.next(&mut mode) {
            let element = source_element;
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
                Instruction::Text { source, width }
                | Instruction::TextIfBreak { source, width }
                | Instruction::TextIfFlat { source, width } => {
                    let active = match element {
                        Instruction::TextIfBreak { .. } => mode == Mode::Break,
                        Instruction::TextIfFlat { .. } => mode == Mode::Flat,
                        _ => true,
                    };
                    if !active {
                        continue;
                    }
                    let source = source.as_str();
                    if EXTENDED && source.is_empty() {
                        continue;
                    }
                    if pending_line_space {
                        // Prettier fits defers `line` spaces until nonempty text,
                        // independently of the number of spaces actually printed.
                        column += 1;
                        if EXTENDED {
                            trailing_space += 1;
                        }
                    }
                    pending_line_space = false;
                    if EXTENDED {
                        let trimmed = source.trim_end_matches([' ', '\t']).len();
                        trailing_space = if trimmed == 0 {
                            trailing_space + source.len()
                        } else {
                            source.len() - trimmed
                        };
                    }
                    column = Text::from_source_parts(source, width).advance_column(column);
                    if column > usize::from(options.line_width) {
                        return false;
                    }
                }
                Instruction::StartLineSuffix { end_offset } => {
                    if kind.layout_dependency() {
                        return true;
                    }
                    has_line_suffix = true;
                    cursor.current.remaining = &cursor.current.remaining[end_offset as usize..];
                }
                Instruction::EndLineSuffix => {
                    unreachable!("fit probes skip complete line suffix scopes")
                }
                Instruction::LineSuffixBoundary => {
                    if has_line_suffix {
                        return false;
                    }
                }
                Instruction::SoftLine
                | Instruction::NormalLine
                | Instruction::SoftLiteralLine
                | Instruction::NormalLiteralLine
                    if mode == Mode::Flat =>
                {
                    if matches!(
                        element,
                        Instruction::NormalLine | Instruction::NormalLiteralLine
                    ) {
                        if kind.line(pending_line_space) {
                            return true;
                        }
                        pending_line_space = true;
                    }
                }
                Instruction::SoftLine
                | Instruction::NormalLine
                | Instruction::HardLine
                | Instruction::HardLineOnly
                | Instruction::SoftLiteralLine
                | Instruction::NormalLiteralLine
                | Instruction::LiteralLine
                | Instruction::LiteralLineOnly
                | Instruction::LiteralLineAtCurrentIndent => return true,
                Instruction::StartGroup {
                    request,
                    effect,
                    id,
                    ..
                } => {
                    if id.is_some() && kind.layout_dependency() {
                        return true;
                    }
                    let expand =
                        group_expands::<SOURCE>(group_effects, request, effect, source_element);
                    if expand && K::BOUNDED {
                        return false;
                    }
                    let group_mode = if expand { Mode::Break } else { mode };
                    modes.push(GroupState { restore_mode: mode });
                    mode = group_mode;
                }
                Instruction::StartGroupWithFlatWidth {
                    summary,
                    end_offset,
                } => {
                    // A fit summary can coalesce line spaces that printing
                    // preserves. Do not extend a batch across this opaque scope.
                    if kind.layout_dependency() {
                        return true;
                    }
                    if !EXTENDED && mode == Mode::Flat {
                        column = summary.advance(column, &mut pending_line_space);
                        if column > usize::from(options.line_width) {
                            return false;
                        }
                        cursor.current.remaining = &cursor.current.remaining[end_offset as usize..];
                    } else {
                        modes.push(GroupState { restore_mode: mode });
                    }
                }
                Instruction::EndGroup => {
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
                Instruction::IfBreak(conditional) | Instruction::FlatIfBreak(conditional) => {
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
                        fragment.elements(),
                        matches!(element, Instruction::FlatIfBreak(_)),
                        None,
                    );
                }
                Instruction::Fragment { elements, .. } => cursor.enter(elements.get(), false, None),
                Instruction::FlatFragment { elements, .. } => {
                    cursor.enter(elements.get(), true, None)
                }
                Instruction::SourceGroup(group) => {
                    if !SOURCE {
                        unreachable!("source groups require their probe specialization");
                    }
                    if kind.layout_dependency() {
                        return true;
                    }
                    let expanded = matches!(group.request(), GroupRequest::Expanded);
                    if expanded && K::BOUNDED {
                        return false;
                    }
                    let group_mode = if expanded { Mode::Break } else { mode };
                    let fragment = if group_mode == Mode::Break {
                        group.fallback_ref()
                    } else {
                        group.primary_ref()
                    };
                    cursor.enter(fragment.elements(), false, Some(mode));
                    mode = group_mode;
                }
                Instruction::CompactGroup(group) => {
                    if kind.layout_dependency() {
                        return true;
                    }
                    let fragment = if mode == Mode::Flat {
                        group.primary_ref()
                    } else {
                        group.fallback_ref()
                    };
                    cursor.enter(fragment.elements(), false, Some(mode));
                }
                Instruction::SoftLineIfGroupBreaks(id) => {
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
                Instruction::BreakParent
                | Instruction::Cursor
                | Instruction::StartLabel(_)
                | Instruction::EndLabel => {}
                Instruction::Trim => {
                    if EXTENDED {
                        if kind.layout_dependency() {
                            return true;
                        }
                        column = column.saturating_sub(trailing_space);
                        trailing_space = 0;
                    }
                }
                Instruction::StartFill if !K::BOUNDED => {
                    uncommitted_fills += 1;
                }
                Instruction::EndFill if !K::BOUNDED => {
                    if uncommitted_fills != 0 {
                        uncommitted_fills -= 1;
                    } else {
                        fills.pop().expect("fit probe closes an active fill scope");
                    }
                }
                Instruction::StartFillPart { .. } if !K::BOUNDED && uncommitted_fills == 0 => {
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
                Instruction::EndFillPart if !K::BOUNDED && uncommitted_fills == 0 => {
                    mode = entries
                        .pop()
                        .expect("fit probe closes an active entry scope");
                }
                Instruction::StartFill => {
                    if kind.layout_dependency() {
                        return true;
                    }
                }
                Instruction::EndFill => {
                    if kind.end_fill() {
                        return true;
                    }
                }
                Instruction::StartFillPart { .. } => kind.start_entry(column, pending_line_space),
                Instruction::EndFillPart => {
                    if kind.end_entry() {
                        return true;
                    }
                }
                Instruction::StartIndentIfBreak { .. } => {
                    if kind.layout_dependency() {
                        return true;
                    }
                }
                Instruction::StartIndent
                | Instruction::StartAbsoluteIndent { .. }
                | Instruction::EndIndent
                | Instruction::StartAlign { .. }
                | Instruction::EndAlign { .. }
                | Instruction::StartAlignText(_)
                | Instruction::StartDedent
                | Instruction::StartMarkRoot
                | Instruction::StartDedentToRoot
                | Instruction::EndRichIndent(_) => {}
            }
        }
        column <= usize::from(options.line_width)
    }
}
