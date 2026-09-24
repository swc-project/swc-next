//! Interpretation of flat text, line breaks and indentation scopes.
use super::{FormatElement, text::Text};
use crate::{FormatOptions, IndentStyle};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flat,
    Break,
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

pub(super) fn print(elements: &[FormatElement<'_>], options: FormatOptions) -> String {
    let mut output = Output {
        text: String::new(),
        column: 0,
        pending_indent: Indentation::default(),
        pending_space: false,
        fit_state: FitState::Measured,
    };
    let mut indent = Indentation::default();
    let mut indents = Vec::new();
    let mut modes = Vec::new();
    let mut mode = Mode::Break;
    let mut probe_modes = Vec::new();
    for (index, &element) in elements.iter().enumerate() {
        match element {
            FormatElement::StartGroup { effect } => {
                let inherited = mode == Mode::Flat && output.fit_state != FitState::Remeasure;
                let flat = !effect.preferred_breaks()
                    && (inherited
                        || fits(
                            &elements[index + 1..],
                            &modes,
                            &output,
                            options,
                            mode,
                            &mut probe_modes,
                        ));
                modes.push(mode);
                mode = if flat { Mode::Flat } else { Mode::Break };
                output.record_group_mode(mode, inherited);
            }
            FormatElement::EndGroup => {
                mode = modes.pop().expect("group scopes must close after opening")
            }
            FormatElement::BreakParent => {}
            FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                if matches!(element, FormatElement::Line) {
                    output.pending_space = true;
                }
            }
            FormatElement::SoftLine | FormatElement::Line => output.line(indent, options, mode),
            FormatElement::Text(text) => output.write(text, options, mode),
            FormatElement::Space => output.pending_space = true,
            FormatElement::HardLine => output.line(indent, options, mode),
            FormatElement::LiteralLine => output.line(Indentation::default(), options, mode),
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

fn fits(
    elements: &[FormatElement<'_>],
    parents: &[Mode],
    output: &Output,
    options: FormatOptions,
    incoming: Mode,
    modes: &mut Vec<Mode>,
) -> bool {
    let mut column = output.column;
    let mut pending_space = output.pending_space;
    let limit = usize::from(options.line_width);
    let mut parent_len = parents.len();
    let mut mode = Mode::Flat;
    modes.clear();
    modes.push(incoming);
    if column > limit {
        return false;
    }
    for &element in elements {
        match element {
            FormatElement::Text(text) => {
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
            FormatElement::StartGroup { effect } => {
                modes.push(mode);
                if effect.preferred_breaks() {
                    mode = Mode::Break;
                }
            }
            FormatElement::EndGroup => {
                mode = if let Some(restore) = modes.pop() {
                    restore
                } else {
                    parent_len = parent_len
                        .checked_sub(1)
                        .expect("fit probe closes an active group scope");
                    parents[parent_len]
                };
            }
            FormatElement::BreakParent => {
                if mode == Mode::Flat {
                    return false;
                }
            }
            FormatElement::StartIndent
            | FormatElement::StartAbsoluteIndent { .. }
            | FormatElement::EndIndent
            | FormatElement::StartAlign { .. }
            | FormatElement::EndAlign { .. } => {}
        }
    }
    column <= limit
}
