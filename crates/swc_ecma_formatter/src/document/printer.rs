//! Interpretation of flat text, line breaks and indentation scopes.
use super::{FormatElement, GroupId, text::Text};
use crate::{FormatOptions, IndentStyle};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flat,
    Break,
}

#[derive(Clone, Copy)]
struct GroupState {
    mode: Mode,
    restore_mode: Mode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FitState {
    Measured,
    // Groups may inherit Flat without a valid measurement for the current line.
    Unmeasured,
    // A forced break under Flat requires a new group decision.
    Remeasure,
}

struct Output {
    text: String,
    column: usize,
    pending_indent: Indentation,
    pending_space: bool,
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
        pending_space: false,
        fit_state: FitState::Measured,
    };
    let mut suffixes = Vec::new();
    let mut indent = Indentation::default();
    let mut indents = Vec::new();
    let mut modes = Vec::new();
    let mut mode = Mode::Break;
    let mut probe = Probe {
        track_groups: conditional_lines,
        ..Probe::default()
    };
    let mut group_modes = vec![Mode::Flat; group_count];
    for (index, &element) in elements.iter().enumerate() {
        match element {
            FormatElement::StartGroup { effect, id } => {
                let inherited = mode == Mode::Flat && output.fit_state != FitState::Remeasure;
                let flat = !effect.preferred_breaks()
                    && (inherited
                        || probe.fits(
                            ProbeRequest {
                                elements: &elements[index + 1..],
                                id,
                            },
                            &modes,
                            &output,
                            options,
                            mode,
                            &group_modes,
                        ));
                let group_mode = if flat { Mode::Flat } else { Mode::Break };
                output.record_group_mode(group_mode, inherited);
                if let Some(id) = id {
                    group_modes[id.0] = group_mode;
                }
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
                    .restore_mode
            }
            FormatElement::SoftLineIfBreak(id) => {
                if group_modes[id.0] == Mode::Break && mode == Mode::Break {
                    flush_suffixes(&mut suffixes, &mut output, options, mode);
                    output.line(indent, options, mode);
                }
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
            FormatElement::StartIndentIfBreak(id) => {
                indents.push(indent);
                if group_modes[id.0] == Mode::Break {
                    indent = indent.indented(options);
                }
            }
            FormatElement::BreakParent => {}
            FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                if matches!(element, FormatElement::Line) {
                    output.pending_space = true;
                }
            }
            FormatElement::SoftLine | FormatElement::Line => {
                flush_suffixes(&mut suffixes, &mut output, options, mode);
                output.line(indent, options, mode);
            }
            FormatElement::LineSuffix(text) => suffixes.push(text),
            FormatElement::Text(text) => output.write(text, options, mode),
            FormatElement::Space => output.pending_space = true,
            FormatElement::HardLine => {
                flush_suffixes(&mut suffixes, &mut output, options, mode);
                output.line(indent, options, mode);
            }
            FormatElement::LiteralLine => {
                flush_suffixes(&mut suffixes, &mut output, options, mode);
                output.line(Indentation::default(), options, mode);
            }
            FormatElement::StartIndent => {
                indents.push(indent);
                indent = indent.indented(options);
            }
            FormatElement::StartAbsoluteIndent { width } => {
                indents.push(indent);
                indent = Indentation::absolute(width, options);
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
        }
    }
    flush_suffixes(&mut suffixes, &mut output, options, mode);
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
        if self.pending_space {
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
        self.pending_space = false;
    }

    fn line(&mut self, indent: Indentation, options: FormatOptions, mode: Mode) {
        self.invalidate_line(mode);
        self.text.push_str(options.line_ending.as_str());
        self.column = indent.width(options);
        self.pending_indent = indent;
        self.pending_space = false;
    }
}

struct ProbeRequest<'a> {
    elements: &'a [FormatElement<'a>],
    id: Option<GroupId>,
}

#[derive(Default)]
struct Probe {
    track_groups: bool,
    group_writes: Vec<(GroupId, Mode)>,
    modes: Vec<GroupState>,
}

impl Probe {
    fn fits(
        &mut self,
        request: ProbeRequest<'_>,
        parents: &[GroupState],
        output: &Output,
        options: FormatOptions,
        incoming: Mode,
        committed_groups: &[Mode],
    ) -> bool {
        let Self {
            track_groups,
            group_writes,
            modes,
        } = self;
        group_writes.clear();
        if *track_groups && let Some(id) = request.id {
            group_writes.push((id, Mode::Flat));
        }
        let mut column = output.column;
        let mut pending_space = output.pending_space;
        let limit = usize::from(options.line_width);
        let mut parent_len = parents.len();
        let mut mode = Mode::Flat;
        modes.clear();
        modes.push(GroupState {
            mode: Mode::Flat,
            restore_mode: incoming,
        });
        if column > limit {
            return false;
        }
        for &element in request.elements {
            match element {
                FormatElement::LineSuffix(_) => {}
                FormatElement::Text(text)
                | FormatElement::IfBreak(text)
                | FormatElement::IfFlat(text) => {
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
                    column += usize::from(pending_space);
                    pending_space = false;
                    let (next, multiline) = text.first_line(column);
                    column = next;
                    if multiline {
                        return column <= limit;
                    }
                    if column > limit {
                        return false;
                    }
                }
                FormatElement::Space => pending_space = true,
                FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                    if matches!(element, FormatElement::Line) {
                        pending_space = true;
                    }
                }
                FormatElement::SoftLine
                | FormatElement::Line
                | FormatElement::HardLine
                | FormatElement::LiteralLine => return true,
                FormatElement::StartGroup { effect, id } => {
                    let group_mode = if effect.preferred_breaks() {
                        Mode::Break
                    } else {
                        mode
                    };
                    modes.push(GroupState {
                        mode: group_mode,
                        restore_mode: mode,
                    });
                    mode = group_mode;
                    if *track_groups && let Some(id) = id {
                        group_writes.push((id, mode));
                    }
                }
                FormatElement::EndGroup => {
                    mode = if let Some(group) = modes.pop() {
                        group.restore_mode
                    } else {
                        parent_len = parent_len
                            .checked_sub(1)
                            .expect("fit probe closes an active group scope");
                        parents[parent_len].restore_mode
                    };
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
                FormatElement::BreakParent => {
                    if mode == Mode::Flat {
                        return false;
                    }
                }
                FormatElement::StartIndent
                | FormatElement::StartAbsoluteIndent { .. }
                | FormatElement::StartIndentIfBreak(_)
                | FormatElement::EndIndent
                | FormatElement::StartAlign { .. }
                | FormatElement::EndAlign { .. } => {}
            }
        }
        column <= limit
    }
}

fn flush_suffixes(
    suffixes: &mut Vec<Text<'_>>,
    output: &mut Output,
    options: FormatOptions,
    mode: Mode,
) {
    for text in suffixes.drain(..) {
        output.pending_space = true;
        output.write(text, options, mode);
    }
}
