use swc_next_allocator::Allocator;

use super::{Doc, FormatElement, GroupRequest, Writer};
use crate::{FormatOptions, IndentStyle, LineEnding};

fn nested_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("{");
    writer.indent(|writer| {
        writer.hard_line();
        writer.text("{");
        writer.indent(|writer| {
            writer.hard_line();
            writer.text("value;");
        });
        writer.hard_line();
        writer.text("}");
    });
    writer.hard_line();
    writer.text("}");
    writer.hard_line();
    writer.finish().print(options)
}

fn line_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first");
    writer.hard_line();
    writer.text("second");
    writer.hard_line();
    writer.finish().print(options)
}

fn absolute_document(width: usize, options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("{");
    writer.indent(|writer| {
        writer.align(3, |writer| {
            writer.hard_line();
            writer.text("before");
            writer.absolute_indent(width, |writer| {
                writer.hard_line();
                writer.text("root");
                writer.indent(|writer| {
                    writer.hard_line();
                    writer.text("child");
                });
            });
            writer.hard_line();
            writer.text("after");
        });
    });
    writer.hard_line();
    writer.text("}");
    writer.finish().print(options)
}

#[test]
fn absolute_indentation() {
    let spaces = FormatOptions::prettier();
    assert_eq!(
        absolute_document(0, spaces),
        "{\n     before\nroot\n  child\n     after\n}"
    );

    assert_eq!(
        absolute_document(3, spaces),
        "{\n     before\n   root\n     child\n     after\n}"
    );

    assert_eq!(
        absolute_document(6, spaces),
        "{\n     before\n      root\n        child\n     after\n}"
    );

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..spaces
    };
    assert_eq!(
        absolute_document(3, tabs),
        "{\n\t   before\n\t\troot\n\t\t\tchild\n\t   after\n}"
    );

    assert_eq!(
        absolute_document(1, tabs),
        "{\n\t   before\n root\n\t\tchild\n\t   after\n}"
    );

    let zero = FormatOptions {
        indent_width: 0.try_into().unwrap(),
        ..spaces
    };
    assert_eq!(
        absolute_document(3, zero),
        "{\n   before\n   root\n   child\n   after\n}"
    );
}

#[test]
fn text_fragments() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first");
    writer.text("");
    writer.text(" second");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first second"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first\r\nsecond");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\r\nsecond"
    );
}

#[test]
fn generated_text() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let source = String::from("source");
    writer.text(source.as_str());
    writer.text(format!(" {source}"));
    writer.text(" ");
    writer.text("x".repeat(32 * 1024));
    writer.text(String::from(" end"));
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        format!("source source {} end", "x".repeat(32 * 1024))
    );
}

#[test]
fn line_endings() {
    let lf = FormatOptions::prettier();
    assert_eq!(line_document(lf), "first\nsecond\n");

    let crlf = FormatOptions {
        line_ending: LineEnding::CrLf,
        ..lf
    };
    assert_eq!(line_document(crlf), "first\r\nsecond\r\n");

    let cr = FormatOptions {
        line_ending: LineEnding::Cr,
        ..lf
    };
    assert_eq!(line_document(cr), "first\rsecond\r");
}

#[test]
fn indentation() {
    let spaces = FormatOptions::prettier();
    assert_eq!(nested_document(spaces), "{\n  {\n    value;\n  }\n}\n");

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..spaces
    };
    assert_eq!(nested_document(tabs), "{\n\t{\n\t\tvalue;\n\t}\n}\n");

    let zero = FormatOptions {
        indent_width: 0.try_into().unwrap(),
        ..spaces
    };
    assert_eq!(nested_document(zero), "{\n{\nvalue;\n}\n}\n");

    let crlf = FormatOptions {
        line_ending: LineEnding::CrLf,
        ..spaces
    };
    assert_eq!(
        nested_document(crlf),
        "{\r\n  {\r\n    value;\r\n  }\r\n}\r\n"
    );
}

#[test]
fn pending_whitespace() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first");
    writer.space();
    writer.hard_line();
    writer.indent(|writer| {
        writer.hard_line();
        writer.space();
        writer.hard_line();
        writer.text("last");
        writer.space();
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\n\n\n  last"
    );
}

#[test]
fn deep_indentation() {
    let mut elements = vec![FormatElement::StartIndent; 1025];
    elements.push(FormatElement::HardLine);
    elements.push(FormatElement::Text(super::text::Text::new("value")));
    elements.extend(std::iter::repeat_n(FormatElement::EndIndent, 1025));
    let document = Doc {
        elements: &elements,
        group_count: 0,
        conditional_lines: false,
    };
    let options = FormatOptions {
        indent_width: 64.try_into().unwrap(),
        ..FormatOptions::prettier()
    };
    let output = document.print(options);
    assert_eq!(output.len(), 1 + 1025 * 64 + "value".len());
    assert!(output.starts_with('\n'));
    assert!(output.ends_with("value"));
    assert!(
        output[1..output.len() - "value".len()]
            .bytes()
            .all(|byte| byte == b' ')
    );
}

#[test]
fn literal_lines() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("{");
    writer.indent(|writer| {
        writer.hard_line();
        writer.text("/* first");
        writer.space();
        writer.literal_line();
        writer.text(" original */");
        writer.hard_line();
        writer.text("value;");
    });
    writer.hard_line();
    writer.text("}");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "{\n  /* first\n original */\n  value;\n}"
    );
}

#[test]
fn literal_text() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.literal_text("first\r\nsecond\rthird");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\nsecond\nthird"
    );
}

#[test]
fn group_breaks() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.line();
        writer.group(true, |writer| {
            writer.text("b");
            writer.line();
            writer.text("c");
        });
    });
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "a\nb\nc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.space();
        writer.line();
        writer.text("b");
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 3,
            ..FormatOptions::prettier()
        }),
        "a b"
    );
}

#[test]
fn group_requests() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group_with_request(GroupRequest::PreserveSource, |writer| {
        writer.text("left");
        writer.soft_line();
        writer.text("right");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "left\nright"
    );

    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("left");
        writer.line();
        writer.text("right");
        writer.break_parent();
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "left\nright"
    );
}

fn aligned_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("prefix");
        writer.indent(|writer| {
            writer.line();
            writer.if_flat("(");
            writer.align(2, |writer| {
                writer.text("first");
                writer.indent(|writer| {
                    writer.line();
                    writer.text("second");
                });
            });
            writer.if_flat(")");
        });
    });
    writer.finish().print(options)
}

#[test]
fn aligned_groups() {
    assert_eq!(
        aligned_document(FormatOptions::prettier()),
        "prefix (first second)"
    );

    let narrow = FormatOptions {
        line_width: 8,
        ..FormatOptions::prettier()
    };
    assert_eq!(aligned_document(narrow), "prefix\n  first\n      second");

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(aligned_document(tabs), "prefix\n\tfirst\n\t\t\tsecond");
}

fn write_pair<'a>(writer: &mut Writer<'a>, first: &'a str, last: &'a str) {
    writer.group(false, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            writer.text(first);
            writer.text(",");
            writer.line();
            writer.text(last);
            writer.if_break(",");
        });
        writer.soft_line();
        writer.text("]");
    });
}

fn pair_document(first: &str, last: &str, width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    write_pair(&mut writer, first, last);
    writer.text(";");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn nested_group_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(true, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            write_pair(writer, "aa", "bb");
            writer.text(",");
            writer.line();
            writer.text("tail");
            writer.if_break(",");
        });
        writer.soft_line();
        writer.text("]");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn dependent_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("target =");
        let separator = writer.named_group(false, |writer| {
            writer.indent(Writer::line);
        });
        writer.indent_if_break(separator, |writer| write_pair(writer, "first", "second"));
    });
    writer.text(";");
    writer.finish().print(options)
}

#[test]
fn dependent_indentation() {
    assert_eq!(
        dependent_document(FormatOptions::prettier()),
        "target = [first, second];"
    );

    let medium = FormatOptions {
        line_width: 16,
        ..FormatOptions::prettier()
    };
    assert_eq!(
        dependent_document(medium),
        "target = [\n  first,\n  second,\n];"
    );

    let narrow = FormatOptions {
        line_width: 8,
        ..medium
    };
    assert_eq!(
        dependent_document(narrow),
        "target =\n  [\n    first,\n    second,\n  ];"
    );

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(
        dependent_document(tabs),
        "target =\n\t[\n\t\tfirst,\n\t\tsecond,\n\t];"
    );
}

#[test]
fn group_continuation() {
    assert_eq!(pair_document("aa", "bb", 9), "[aa, bb];");

    assert_eq!(pair_document("aa", "bb", 8), "[\n  aa,\n  bb,\n];");

    assert_eq!(nested_group_document(11), "[\n  [aa, bb],\n  tail,\n]");

    assert_eq!(
        nested_group_document(10),
        "[\n  [\n    aa,\n    bb,\n  ],\n  tail,\n]"
    );
}

#[test]
fn display_columns() {
    assert_eq!(pair_document("世界", "a", 10), "[世界, a];");

    assert_eq!(pair_document("世界", "a", 9), "[\n  世界,\n  a,\n];");

    assert_eq!(pair_document("é", "a", 7), "[é, a];");

    assert_eq!(pair_document("é", "a", 6), "[\n  é,\n  a,\n];");
}

#[test]
fn named_lines() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let group = writer.named_group(true, |writer| writer.text("first"));
    writer.soft_line_if_break(group);
    writer.text("second");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\nsecond"
    );
}

#[test]
fn text_line_measurement() {
    let ascii = super::text::Text::new("ASCII");
    assert_eq!(ascii.last_line(3), (8, false));

    let unicode = super::text::Text::new("世界");
    assert_eq!(unicode.last_line(3), (7, false));

    let controls = super::text::Text::new("a\u{7f}\té");
    assert_eq!(controls.last_line(3), (5, false));

    let tab = super::text::Text::new("a\tb");
    assert_eq!(tab.first_line(0), (2, false));
    assert_eq!(tab.last_line(3), (5, false));

    let separators = super::text::Text::new("a\u{2028}b\u{2029}c");
    assert_eq!(separators.last_line(3), (8, false));

    let cr = super::text::Text::new("head\rtail");
    assert_eq!(cr.last_line(3), (4, true));

    let lf = super::text::Text::new("head\n\t世界");
    assert_eq!(lf.first_line(3), (7, true));
    assert_eq!(lf.last_line(3), (4, true));

    let crlf = super::text::Text::new("head\r\ntail\n");
    assert_eq!(crlf.last_line(3), (0, true));
}

fn remeasured_value(writer: &mut Writer<'_>) {
    writer.text("{");
    writer.indent(|writer| {
        writer.soft_line();
        writer.text("longName");
    });
    writer.soft_line();
    writer.text("}");
}

fn remeasurement_document(write_line: impl FnOnce(&mut Writer<'_>)) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("head");
        write_line(writer);
        writer.group(false, remeasured_value);
        writer.text(" tail");
    });
    writer.finish().print(FormatOptions {
        line_width: 12,
        ..FormatOptions::prettier()
    })
}

#[test]
fn forced_line_remeasurement() {
    assert_eq!(
        remeasurement_document(|writer| writer.text("\n")),
        "head\n{\n  longName\n} tail"
    );

    assert_eq!(
        remeasurement_document(|writer| writer.hard_line()),
        "head\n{\n  longName\n} tail"
    );

    assert_eq!(
        remeasurement_document(|writer| writer.literal_line()),
        "head\n{\n  longName\n} tail"
    );
}
