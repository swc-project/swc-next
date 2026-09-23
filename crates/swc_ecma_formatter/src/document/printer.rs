//! Interpretation of flat text, line breaks and indentation scopes.
use super::{FormatElement, text::Text};
use crate::{FormatOptions, IndentStyle};

struct Output {
    text: String,
    column: usize,
    pending_indent: Indentation,
    pending_space: bool,
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
    };
    let mut indent = Indentation::default();
    let mut indents = Vec::new();
    for &element in elements {
        match element {
            FormatElement::Text(text) => output.write(text, options),
            FormatElement::Space => output.pending_space = true,
            FormatElement::HardLine => output.line(indent, options),
            FormatElement::LiteralLine => output.line(Indentation::default(), options),
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
    fn write(&mut self, text: Text<'_>, options: FormatOptions) {
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
        self.column = text.last_line(self.column).0;
        self.pending_indent = Indentation::default();
        self.pending_space = false;
    }

    fn line(&mut self, indent: Indentation, options: FormatOptions) {
        self.text.push_str(options.line_ending.as_str());
        self.column = indent.width(options);
        self.pending_indent = indent;
        self.pending_space = false;
    }
}
