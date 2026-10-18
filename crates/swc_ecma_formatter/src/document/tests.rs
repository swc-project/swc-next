use swc_next_allocator::Allocator;

use super::{Doc, FormatElement, GroupRequest, Writer};
use crate::{FormatOptions, IndentStyle, LineEnding};

fn write_comment_suffix<'a>(writer: &mut Writer<'a>, text: &'a str) {
    writer.space();
    writer.text(text);
    writer.break_parent();
}

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

fn absolute_fragment_document(flat: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.absolute_indent(3, |writer| {
            writer.text("left");
            writer.line();
            writer.text("right");
        });
    });
    writer.text("{");
    writer.indent(|writer| {
        writer.hard_line();
        if flat {
            writer.write_flat_fragment(fragment.try_flat().unwrap());
        } else {
            writer.write_fragment(fragment);
        }
        writer.hard_line();
        writer.text("after");
    });
    writer.hard_line();
    writer.text("}");
    writer.finish().print(FormatOptions::prettier())
}

#[test]
fn absolute_fragment_views() {
    assert_eq!(
        absolute_fragment_document(false),
        "{\n  left\n   right\n  after\n}"
    );

    assert_eq!(
        absolute_fragment_document(true),
        "{\n  left right\n  after\n}"
    );
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

fn flat_width_document(width: u16, write: impl FnOnce(&mut Writer<'_>)) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    write(&mut writer);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn flat_width_layout(width: u16) -> String {
    flat_width_document(width, |writer| {
        writer.group(false, |writer| {
            writer.text("return");
            writer.space();
            writer.group_with_flat_width(|writer| {
                writer.if_break("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    writer.group(false, |writer| {
                        writer.text("first");
                        writer.line();
                        writer.text("second");
                    });
                });
                writer.soft_line();
                writer.if_break(")");
            });
            writer.text(";");
        });
    })
}

#[test]
fn prepared_group_layouts() {
    assert_eq!(flat_width_layout(20), "return first second;");

    assert_eq!(flat_width_layout(19), "return (\n  first second\n);");

    assert_eq!(flat_width_layout(14), "return (\n  first second\n);");

    assert_eq!(flat_width_layout(13), "return (\n  first\n  second\n);");
}

fn flat_width_gaps(reserved: bool) -> String {
    flat_width_document(5, |writer| {
        writer.text("a");
        if reserved {
            writer.reserved_space();
        } else {
            writer.space();
        }
        writer.group_with_flat_width(|writer| {
            writer.space();
            writer.reserved_space();
            writer.if_break("(");
            writer.text("b");
            writer.space();
            writer.reserved_space();
            writer.space();
            writer.text("c");
            writer.space();
            writer.if_break(")");
        });
        writer.hard_line();
        writer.text("d");
    })
}

#[test]
fn prepared_group_gaps() {
    assert_eq!(flat_width_gaps(false), "a b c\nd");

    assert_eq!(flat_width_gaps(true), "a b c\nd");

    let empty_leading = flat_width_document(4, |writer| {
        writer.text("a");
        writer.reserved_space();
        writer.group_with_flat_width(|writer| {
            writer.if_flat("");
            writer.space();
            writer.if_break("(");
            writer.text("b");
            writer.if_break(")");
        });
        writer.hard_line();
        writer.text("c");
    });
    assert_eq!(empty_leading, "a  b\nc");

    let empty_middle = flat_width_document(4, |writer| {
        writer.group_with_flat_width(|writer| {
            writer.if_break("(");
            writer.text("a");
            writer.space();
            writer.if_flat("");
            writer.space();
            writer.text("b");
            writer.if_break(")");
        });
    });
    assert_eq!(empty_middle, "a  b");

    let empty_trailing = flat_width_document(3, |writer| {
        writer.text("a");
        writer.space();
        writer.group_with_flat_width(|writer| {
            writer.if_break("(");
            writer.text("b");
            writer.space();
            writer.if_flat("");
            writer.if_break(")");
        });
        writer.hard_line();
        writer.text("c");
    });
    assert_eq!(empty_trailing, "a (b )\nc");
}

fn flat_width_text(text: &str, width: u16) -> String {
    flat_width_document(width, |writer| {
        writer.text("a");
        writer.space();
        writer.group_with_flat_width(|writer| {
            writer.if_break("(");
            writer.indent(|writer| {
                writer.soft_line();
                writer.text_copied(text);
            });
            writer.soft_line();
            writer.if_break(")");
        });
        writer.text(";");
    })
}

#[test]
fn prepared_group_columns() {
    assert_eq!(flat_width_text("世界", 7), "a 世界;");

    assert_eq!(flat_width_text("世界", 6), "a (\n  世界\n);");

    assert_eq!(flat_width_text("a\tb", 5), "a a\tb;");

    assert_eq!(flat_width_text("a\tb", 4), "a (\n  a\tb\n);");

    assert_eq!(flat_width_text("a\u{7f}b", 6), "a a\u{7f}b;");

    assert_eq!(flat_width_text("a\u{7f}b", 5), "a (\n  a\u{7f}b\n);");
}

#[test]
fn prepared_group_fallbacks() {
    let hard_line = flat_width_document(2, |writer| {
        writer.group_with_flat_width(|writer| {
            writer.text("a");
            writer.hard_line();
            writer.text("long");
        });
    });
    assert_eq!(hard_line, "a\nlong");

    let named = flat_width_document(3, |writer| {
        writer.group_with_flat_width(|writer| {
            let group = writer.named_group(false, |writer| {
                writer.text("aa");
                writer.line();
                writer.text("bb");
            });
            writer.indent_if_break(group, |writer| {
                writer.soft_line_if_break(group);
                writer.text("c");
            });
        });
    });
    assert_eq!(named, "aa\nbb\n  c");
}

#[test]
fn prepared_group_flat_views() {
    let flat = flat_width_document(4, |writer| {
        let fragment = record(writer, |writer| {
            writer.group_with_flat_width(|writer| {
                writer.if_break("(");
                writer.text("first");
                writer.line();
                writer.if_flat("second");
                writer.if_break(")");
            });
        });
        writer.text("prefix ");
        writer.write_flat_fragment(fragment.try_flat().unwrap());
    });
    assert_eq!(flat, "prefix first second");
}

fn probe_line_continuation(hard: bool) -> String {
    flat_width_document(3, |writer| {
        writer.group(false, |writer| {
            writer.text("a");
            writer.line();
            writer.text("b");
        });
        writer.text("xx");
        if hard {
            writer.hard_line();
        } else {
            writer.soft_line();
        }
        writer.text("c");
    })
}

#[test]
fn probe_overflow_boundaries() {
    assert_eq!(probe_line_continuation(true), "a\nbxx\nc");

    assert_eq!(probe_line_continuation(false), "a\nbxx\nc");

    let conditional = flat_width_document(3, |writer| {
        let group = writer.named_group(false, |writer| {
            writer.text("a");
            writer.line();
            writer.text("b");
        });
        writer.soft_line_if_break(group);
        writer.text("cc");
    });
    assert_eq!(conditional, "a\nb\ncc");
}

fn probe_gap_continuation(reserved: bool) -> String {
    flat_width_document(3, |writer| {
        writer.group(false, |writer| {
            writer.text("a");
            writer.line();
            writer.text("b");
        });
        if reserved {
            writer.reserved_space();
        } else {
            writer.space();
        }
        writer.hard_line();
        writer.text("c");
    })
}

#[test]
fn probe_pending_gaps() {
    assert_eq!(probe_gap_continuation(false), "a b\nc");

    assert_eq!(probe_gap_continuation(true), "a\nb\nc");
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
fn named_group_scopes() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = writer.named_group(true, |writer| {
        writer.text("first");
        writer.line();
        writer.text("second");
    });
    writer.hard_line();
    writer.named_group(false, |writer| writer.text("third"));
    writer.indent_if_break(first, |writer| {
        writer.hard_line();
        writer.text("last");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\nsecond\nthird\n  last"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let group = writer.flat(|writer| {
        writer.named_group(true, |writer| {
            writer.text("first");
            writer.line();
            writer.text("second");
        })
    });
    writer.indent_if_break(group, |writer| {
        writer.hard_line();
        writer.text("last");
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 8,
            ..FormatOptions::prettier()
        }),
        "first second\nlast"
    );
}

fn fill_document(parts: &[&str], width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            writer.fill(|writer| {
                let mut parts = parts.iter().peekable();
                let mut first = true;
                while let Some(part) = parts.next() {
                    if !first {
                        writer.fill_entry(|writer| writer.line());
                    }
                    writer.fill_entry(|writer| {
                        writer.text(*part);
                        if parts.peek().is_some() {
                            writer.text(",");
                        } else {
                            writer.if_break(",");
                        }
                    });
                    first = false;
                }
            });
        });
        writer.soft_line();
        writer.text("]");
    });
    writer.text(";");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn grouped_fill_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            writer.fill(|writer| {
                writer.fill_entry(|writer| {
                    writer.group(false, |writer| {
                        writer.text("[a");
                        writer.line();
                        writer.text("b]");
                    });
                    writer.text(",");
                });
                writer.fill_entry(|writer| writer.line());
                writer.fill_entry(|writer| writer.text("x,"));
                writer.fill_entry(|writer| writer.line());
                writer.fill_entry(|writer| {
                    writer.text("y");
                    writer.if_break(",");
                });
            });
        });
        writer.soft_line();
        writer.text("]");
    });
    writer.text(";");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
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
    let allocator = Allocator::new();
    let mut elements = vec![FormatElement::StartIndent; 1025];
    elements.push(FormatElement::HardLine);
    elements.push(FormatElement::Text(super::text::Text::new("value")));
    elements.extend(std::iter::repeat_n(FormatElement::EndIndent, 1025));
    let document = Doc {
        allocator: &allocator,
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

fn reserved_continuation(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("(");
        writer.indent(|writer| {
            writer.soft_line();
            writer.text("aaa");
        });
        writer.soft_line();
        writer.text(")");
    });
    writer.reserved_space();
    writer.soft_line();
    writer.text("rest");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn reserved_fill(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("aa");
    writer.reserved_space();
    writer.fill(|writer| {
        writer.fill_entry(|writer| writer.text("b"));
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("c"));
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn reserved_candidate(width: u16, canonical: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, |writer| {
        writer.group_with_request(GroupRequest::PreserveSource, |writer| {
            writer.soft_line();
            writer.text("b");
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("F"));
    writer.text("aaa");
    writer.reserved_space();
    if canonical {
        writer.write_canonical_alternatives(
            candidate,
            &[candidate.canonical_candidate()],
            fallback,
        );
    } else {
        writer.write_alternatives(candidate, &[candidate], fallback);
    }
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn reserved_gap_measurement() {
    assert_eq!(reserved_continuation(6), "(aaa)\nrest");

    assert_eq!(reserved_continuation(5), "(\n  aaa\n)\nrest");

    assert_eq!(reserved_fill(6), "aa b c");

    assert_eq!(reserved_fill(5), "aa b\nc");

    assert_eq!(reserved_candidate(5, true), "aaa\nb");

    assert_eq!(reserved_candidate(4, true), "aaa F");

    assert_eq!(reserved_candidate(4, false), "aaa\nb");

    assert_eq!(reserved_candidate(3, false), "aaa F");
}

fn reserved_retry_continuation(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = record(&mut writer, |writer| {
        writer.text("x");
        writer.reserved_space();
        writer.text("too long");
    });
    let second = record(&mut writer, |writer| {
        writer.text("b");
        writer.reserved_space();
    });
    let fallback = record(&mut writer, |writer| writer.text("F"));
    writer.text("a");
    writer.reserved_space();
    writer.write_alternatives(first, &[first, second], fallback);
    writer.group(false, |writer| {
        writer.soft_line();
        writer.if_flat("c");
        writer.if_break("C");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn reserved_gap_selection_boundaries() {
    assert_eq!(reserved_retry_continuation(5), "a b c");

    assert_eq!(reserved_retry_continuation(4), "a b\nC");

    assert_eq!(reserved_retry_continuation(3), "a F\nC");
}

#[test]
fn reserved_gap_rendering() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.space();
        writer.reserved_space();
        writer.reserved_space();
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

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("raw ");
    writer.reserved_space();
    writer.hard_line();
    writer.text("next");
    writer.reserved_space();
    writer.literal_line();
    writer.text(" original ");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "raw \nnext\n original "
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.reserved_space();
    writer.line_suffix(|writer| write_comment_suffix(writer, "// tail"));
    writer.hard_line();
    writer.text("b");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a // tail\nb"
    );
}

#[test]
fn reserved_gap_projection() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let gap = record(&mut writer, |writer| {
        writer.space();
        writer.reserved_space();
    });
    assert!(gap.try_neutral().is_some());
    writer.text("a");
    writer.write_neutral_fragment(gap.try_neutral().unwrap());
    writer.text("b");
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "a b");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let gap = record(&mut writer, |writer| {
        writer.reserved_space();
        writer.line();
    });
    assert!(gap.try_flat().is_some());
    writer.text("a");
    writer.write_flat_fragment(gap.try_flat().unwrap());
    writer.text("b");
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "a b");
}

#[test]
fn line_suffixes() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.line_suffix(|writer| write_comment_suffix(writer, "// comment"));
        writer.text(",");
        writer.line();
        writer.text("b");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a, // comment\nb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// last"));
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a // last"
    );
}

#[test]
fn line_suffix_boundaries() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix_boundary();
    writer.text("b");
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "ab");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// first"));
    writer.line_suffix_boundary();
    writer.line_suffix(|writer| write_comment_suffix(writer, "// second"));
    writer.hard_line();
    writer.text("b");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a // first\n // second\nb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// first"));
    writer.group(false, |writer| {
        writer.if_break("(");
        writer.line_suffix_boundary();
        writer.text("b");
        writer.line();
        writer.text("c");
        writer.if_break(")");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a( // first\nb\nc)"
    );
}

#[test]
fn line_suffix_documents() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| writer.text("c"));
    writer.text("b");
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "abc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.line_suffix(|writer| writer.text("// long suffix"));
        writer.line();
        writer.text("b");
    });
    let narrow = FormatOptions {
        line_width: 4,
        ..FormatOptions::prettier()
    };
    assert_eq!(writer.finish().print(narrow), "a b// long suffix");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| {
        writer.text("x");
        writer.line_suffix(|writer| writer.text("y"));
        writer.text("z");
    });
    writer.text("b");
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "abxzy");
}

#[test]
fn line_suffix_contexts() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.line_suffix(|writer| {
            writer.if_break("B");
            writer.if_flat("F");
        });
    });
    writer.group(true, |writer| {
        writer.text("b");
        writer.hard_line();
        writer.text("c");
    });
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "abF\nc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.indent(|writer| {
        writer.line_suffix(|writer| {
            writer.text("x");
            writer.hard_line();
            writer.text("y");
        });
    });
    writer.text("a");
    writer.hard_line();
    writer.text("b");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "ax\n  y\nb"
    );
}

#[test]
fn flat_line_suffix_boundaries() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// first"));
    writer.flat(|writer| {
        writer.line_suffix_boundary();
        writer.text("b");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a // first\nb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.line_suffix_boundary();
        writer.text("b");
    });
    writer.text("a");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// first"));
    writer.write_flat_fragment(fragment.try_flat().unwrap());
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a // first\nb"
    );
}

fn suffix_continuation_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let suffix = record(&mut writer, |writer| {
        writer.hard_line();
        writer.group(false, |writer| {
            writer.text("xx");
            writer.line();
            writer.text("yy");
        });
        writer.if_break("B");
        writer.if_flat("F");
    });
    writer.group(false, |writer| {
        writer.text("a");
        writer.line_suffix(|writer| writer.write_fragment(suffix));
    });
    writer.indent(|writer| {
        writer.group(false, |writer| {
            writer.line_suffix(|writer| {
                writer.if_break("BBBB");
                writer.if_flat("F");
                writer.text("p");
                writer.line();
                writer.text("q");
            });
        });
    });
    writer.text("b");
    writer.hard_line();
    writer.text("c");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn line_suffix_continuations() {
    assert_eq!(suffix_continuation_document(80), "ab\nxx yyBFp q\nc");

    assert_eq!(suffix_continuation_document(10), "ab\nxx yyBFp q\nc");

    assert_eq!(suffix_continuation_document(9), "ab\nxx\nyyBFp q\nc");
}

#[test]
fn display_columns() {
    assert_eq!(pair_document("世界", "a", 10), "[世界, a];");

    assert_eq!(pair_document("世界", "a", 9), "[\n  世界,\n  a,\n];");

    assert_eq!(pair_document("é", "a", 7), "[é, a];");

    assert_eq!(pair_document("é", "a", 6), "[\n  é,\n  a,\n];");
}

fn clean_parts_width(parts: &[&str]) -> Option<usize> {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer
        .write_with_clean_text_width(|writer| {
            for part in parts {
                writer.text(*part);
            }
            Ok::<_, ()>(())
        })
        .unwrap()
}

#[test]
fn complete_text_measurement() {
    assert_eq!(clean_parts_width(&["ASCII", "\u{7f}"]), Some(6));
    assert_eq!(clean_parts_width(&["é\u{7f}"]), Some(1));
    assert_eq!(clean_parts_width(&["a\u{7f}", "é"]), Some(2));
    assert_eq!(clean_parts_width(&["👩", "\u{200d}", "💻"]), Some(2));
    assert_eq!(clean_parts_width(&["©", "\u{fe0f}"]), Some(2));
    assert_eq!(clean_parts_width(&["🇺", "🇸"]), Some(2));
    assert_eq!(clean_parts_width(&["é", "\n"]), None);

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let middle = record(&mut writer, |writer| writer.text("\u{200d}"));
    let width = writer
        .write_with_clean_text_width(|writer| {
            writer.text("👩");
            writer.write_fragment(middle);
            writer.text("💻");
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(width, Some(2));

    let text = record(&mut writer, |writer| writer.text("👩\u{200d}💻"));
    assert_eq!(text.clean_text_width(), Some(2));

    let expanded = writer.group_fragment(true, text);
    assert_eq!(expanded.clean_text_width(), None);
}

#[test]
fn cached_text_columns() {
    assert_eq!(pair_document("لا", "a", 8), "[لا, a];");
    assert_eq!(pair_document("لا", "a", 7), "[\n  لا,\n  a,\n];");
    assert_eq!(pair_document("a\u{200d}bcd", "a", 11), "[a\u{200d}bcd, a];");
    assert_eq!(
        pair_document("a\u{200d}bcd", "a", 10),
        "[\n  a\u{200d}bcd,\n  a,\n];"
    );
    assert_eq!(pair_document("a\u{7f}b", "a", 9), "[a\u{7f}b, a];");
    assert_eq!(
        pair_document("a\u{7f}b", "a", 8),
        "[\n  a\u{7f}b,\n  a,\n];"
    );
    assert_eq!(pair_document("👩\u{200d}💻", "a", 8), "[👩\u{200d}💻, a];");

    let tab = super::text::Text::new("a\u{7f}\té");
    assert_eq!(tab.clean_width(), Some(2));
    assert_eq!(tab.first_line(0), (2, false));
    assert_eq!(tab.last_line(0), (2, false));

    let lines = super::text::Text::new("é\r\n\u{7f}b");
    assert_eq!(lines.clean_width(), None);
    assert_eq!(lines.first_line(0), (1, true));
    assert_eq!(lines.last_line(0), (1, true));
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

fn recorded_fill_document(width: u16, inherited: bool, nested: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let inner = nested.then(|| {
        let (group, ()) = writer
            .record_group_with_value(|writer| {
                writer.fill(|writer| {
                    writer.fill_entry(|writer| writer.text("aa"));
                    writer.fill_entry(Writer::line);
                    writer.fill_entry(|writer| writer.text("bb"));
                });
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();
        group.contents()
    });
    let (group, ()) = writer
        .record_group_with_value(|writer| {
            writer.fill(|writer| {
                writer.fill_entry(|writer| match inner {
                    Some(fragment) => writer.write_fragment(fragment),
                    None => writer.text("aa"),
                });
                if inner.is_none() {
                    writer.fill_entry(Writer::line);
                    writer.fill_entry(|writer| writer.text("bb"));
                }
                writer.fill_entry(Writer::line);
                writer.fill_entry(|writer| writer.text("cc"));
            });
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    let fragment = if inherited {
        group.contents()
    } else {
        group.grouped()
    };
    let relocated = record(&mut writer, |writer| {
        writer.text("p:");
        writer.hard_line();
        writer.write_fragment(fragment);
    });
    writer.group(true, |writer| writer.write_fragment(relocated));
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn fill_layout() {
    assert_eq!(
        fill_document(&["1", "2", "3", "4", "5", "6"], 20),
        "[1, 2, 3, 4, 5, 6];"
    );

    assert_eq!(
        fill_document(&["1", "2", "3", "4", "5", "6"], 10),
        "[\n  1, 2, 3,\n  4, 5, 6,\n];"
    );

    assert_eq!(
        fill_document(&["1", "2", "3", "4", "5", "6"], 8),
        "[\n  1, 2,\n  3, 4,\n  5, 6,\n];"
    );

    assert_eq!(
        fill_document(&["long_element", "2", "3"], 10),
        "[\n  long_element,\n  2, 3,\n];"
    );

    assert_eq!(grouped_fill_document(20), "[[a b], x, y];");

    assert_eq!(grouped_fill_document(10), "[\n  [a b],\n  x, y,\n];");

    assert_eq!(grouped_fill_document(7), "[\n  [a\n  b],\n  x, y,\n];");

    assert_eq!(recorded_fill_document(5, false, false), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(5, true, false), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(80, false, false), "p:\naa bb cc");

    assert_eq!(recorded_fill_document(80, true, false), "p:\naa bb cc");

    assert_eq!(recorded_fill_document(5, false, true), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(5, true, true), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(80, false, true), "p:\naa bb cc");

    assert_eq!(recorded_fill_document(80, true, true), "p:\naa bb cc");
}

#[test]
fn flat_scopes() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("before");
        writer.line();
        writer.flat(|writer| {
            writer.group(true, |writer| write_pair(writer, "first", "second"));
        });
        writer.line();
        writer.text("after");
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 5,
            ..FormatOptions::prettier()
        }),
        "before\n[first, second]\nafter"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.flat(|writer| {
        writer.group(false, |writer| {
            writer.text("first");
            writer.hard_line();
            writer.text("second");
        });
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\nsecond"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.flat(|writer| {
        writer.group(false, |writer| {
            writer.text("first");
            writer.line_suffix(|writer| write_comment_suffix(writer, "// comment"));
            writer.line();
            writer.text("second");
        });
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first // comment\nsecond"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| {
                writer.flat(|writer| write_pair(writer, "first", "second"));
            });
            writer.fill_entry(|writer| writer.line());
            writer.fill_entry(|writer| writer.text("last"));
        });
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 5,
            ..FormatOptions::prettier()
        }),
        "[first, second]\nlast"
    );
}

fn current_mode_fill(width: u16, expand_next: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(true, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.text("aa"));
            writer.fill_entry(|writer| {
                writer.if_break(",");
                writer.if_flat("unused");
                writer.if_current_break("!");
                writer.if_current_flat(" ");
                writer.soft_line();
            });
            writer.fill_entry(|writer| writer.group(expand_next, |writer| writer.text("bb")));
        });
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn current_mode_group(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.if_current_break("B");
        writer.if_current_flat("1234");
        writer.line();
        writer.text("x");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn current_mode_scopes() {
    assert_eq!(current_mode_fill(8, false), "aa, bb");

    assert_eq!(current_mode_fill(5, false), "aa,!\nbb");

    assert_eq!(current_mode_fill(8, true), "aa,!\nbb");

    assert_eq!(current_mode_group(6), "1234 x");

    assert_eq!(current_mode_group(5), "B\nx");
}

fn record<'a>(writer: &mut Writer<'a>, write: impl FnOnce(&mut Writer<'a>)) -> super::Fragment<'a> {
    writer
        .record_with_value(|writer| {
            write(writer);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap()
        .0
}

#[test]
fn fragment_line_breaks() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let atom = record(&mut writer, |writer| {
        writer.group(false, |writer| writer.text("value"))
    });
    assert!(!atom.can_break());

    let soft = record(&mut writer, Writer::soft_line);
    assert!(soft.can_break());

    let flat = record(&mut writer, |writer| {
        writer.write_flat_fragment(soft.try_flat().unwrap())
    });
    assert!(!flat.can_break());

    let nested = record(&mut writer, |writer| writer.write_fragment(soft));
    assert!(nested.can_break());

    let group = writer.named_group(false, |writer| writer.text("head"));
    let conditional = record(&mut writer, |writer| writer.soft_line_if_break(group));
    assert!(conditional.can_break());

    let hard = record(&mut writer, Writer::hard_line);
    assert!(hard.can_break());

    let literal = record(&mut writer, Writer::literal_line);
    assert!(literal.can_break());

    let current = record(&mut writer, |writer| writer.if_current_break("first\nlast"));
    assert!(current.can_break());

    let flat_current = record(&mut writer, |writer| {
        writer.write_flat_fragment(current.try_flat().unwrap())
    });
    assert!(!flat_current.can_break());

    let multiline = record(&mut writer, |writer| writer.text("first\nlast"));
    assert!(multiline.can_break());

    let fallback_only = record(&mut writer, |writer| {
        writer.write_alternatives(atom, &[atom], hard)
    });
    assert!(!fallback_only.can_break());

    let first = record(&mut writer, |writer| {
        writer.write_alternatives(soft, &[soft], atom)
    });
    assert!(first.can_break());

    let canonical = record(&mut writer, |writer| {
        writer.write_canonical_alternatives(atom, &[atom.canonical_candidate()], hard)
    });
    assert!(!canonical.can_break());
}

#[test]
fn fragment_capabilities() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let atom = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.text("宽\tvalue");
            writer.space();
            writer.text("tail");
        });
    });
    assert!(atom.try_neutral().is_some());
    assert!(atom.try_flat().is_some());

    let nested = record(&mut writer, |writer| {
        writer.write_fragment(atom);
        writer.write_flat_fragment(atom.try_flat().unwrap());
    });
    assert!(nested.try_neutral().is_some());

    let conditional = record(&mut writer, |writer| {
        writer.text("value");
        writer.if_break(",");
        writer.if_flat(".");
    });
    assert!(conditional.try_neutral().is_none());
    assert!(conditional.try_flat().is_some());

    let flat_conditional = record(&mut writer, |writer| {
        writer.write_flat_fragment(conditional.try_flat().unwrap());
    });
    assert!(flat_conditional.try_neutral().is_none());
    assert!(flat_conditional.try_flat().is_some());

    let current = record(&mut writer, |writer| {
        writer.if_current_break("broken\nbranch");
        writer.if_current_flat("flat");
    });
    assert!(current.try_neutral().is_none());
    assert!(current.try_flat().is_some());

    let projected = record(&mut writer, |writer| {
        writer.flat(|writer| writer.write_fragment(current));
    });
    assert!(projected.try_neutral().is_none());
    assert!(projected.try_flat().is_some());
    writer.write_flat_fragment(projected.try_flat().unwrap());

    let multiline_current = record(&mut writer, |writer| {
        writer.if_current_flat("first\nlast");
    });
    assert!(multiline_current.try_neutral().is_none());
    assert!(multiline_current.try_flat().is_none());

    let named = record(&mut writer, |writer| {
        writer.named_group(false, |writer| writer.text("value"));
    });
    assert!(named.try_neutral().is_none());
    assert!(named.try_flat().is_some());

    let expanded = record(&mut writer, |writer| {
        writer.group(true, |writer| writer.text("value"));
    });
    assert!(expanded.try_neutral().is_none());
    assert!(expanded.try_flat().is_some());

    let mandatory = record(&mut writer, |writer| {
        writer.text("first");
        writer.hard_line();
        writer.text("last");
    });
    assert!(mandatory.try_neutral().is_none());
    assert!(mandatory.try_flat().is_none());

    let multiline = record(&mut writer, |writer| writer.text("first\u{2028}last"));
    assert!(multiline.try_neutral().is_none());
    assert!(multiline.try_flat().is_none());

    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(atom, &[atom], expanded);
    });
    assert!(choices.try_neutral().is_none());
    assert!(choices.try_flat().is_some());

    let flattened = record(&mut writer, |writer| {
        writer.flat(|writer| {
            writer.group(true, |writer| {
                writer.text("value");
                writer.if_break(",");
                writer.if_flat(".");
                writer.if_current_break("broken\nbranch");
                writer.if_current_flat("flat");
            });
        });
    });
    assert!(flattened.try_neutral().is_some());

    let grouped = writer.group_fragment(false, atom);
    assert!(grouped.try_neutral().is_some());
    assert!(!grouped.preferred_breaks());

    let forced = writer.group_fragment(true, atom);
    assert!(forced.try_neutral().is_none());
    assert!(forced.try_flat().is_some());
    assert!(forced.preferred_breaks());

    let required = writer.group_fragment(false, mandatory);
    assert!(required.try_flat().is_none());
    assert!(required.preferred_breaks());

    let flat = writer.flat_delimited_fragment(
        "(",
        atom.try_flat().unwrap(),
        atom,
        atom.try_flat().unwrap(),
        ")",
    );
    assert!(flat.try_neutral().is_some());

    let projected = writer.flat_delimited_fragment(
        "(",
        expanded.try_flat().unwrap(),
        atom,
        current.try_flat().unwrap(),
        ")",
    );
    assert!(projected.try_neutral().is_none());
    assert!(projected.try_flat().is_some());
    assert!(!projected.preferred_breaks());
    assert!(!projected.conditional_lines);

    let multiline = writer.flat_delimited_fragment(
        "(\n",
        atom.try_flat().unwrap(),
        atom,
        atom.try_flat().unwrap(),
        ")",
    );
    assert!(multiline.try_flat().is_none());

    let fallback = writer.indented_delimited_fragment("(", atom, atom, atom, ")");
    assert!(fallback.try_neutral().is_none());
    assert!(fallback.try_flat().is_some());

    let required = writer.indented_delimited_fragment("(", atom, mandatory, atom, ")");
    assert!(required.try_flat().is_none());
    assert!(required.preferred_breaks());

    let joined = writer.join_fragments(atom, true, atom);
    assert!(joined.try_neutral().is_some());
    assert!(joined.try_flat().is_some());

    let joined = writer.join_fragments(atom, false, conditional);
    assert!(joined.try_neutral().is_none());
    assert!(joined.try_flat().is_some());

    let joined = writer.join_fragments(mandatory, true, atom);
    assert!(joined.try_flat().is_none());
    assert!(joined.preferred_breaks());

    let enclosed =
        writer.conditional_group_fragment(atom, GroupRequest::Expanded, "(", conditional, ")");
    assert!(enclosed.try_neutral().is_none());
    assert!(enclosed.try_flat().is_some());
    assert!(enclosed.preferred_breaks());

    let enclosed = writer.conditional_group_fragment(atom, GroupRequest::Auto, "(", mandatory, ")");
    assert!(enclosed.try_flat().is_none());
    assert!(enclosed.preferred_breaks());

    let enclosed = writer.conditional_group_fragment(mandatory, GroupRequest::Auto, "(", atom, ")");
    assert!(enclosed.try_flat().is_none());
    assert!(enclosed.preferred_breaks());

    let enclosed =
        writer.conditional_group_fragment(atom, GroupRequest::PreserveSource, "(", atom, ")");
    assert!(enclosed.try_neutral().is_none());
    assert!(enclosed.try_flat().is_some());
    assert!(enclosed.preferred_breaks());
    assert!(!enclosed.effect.required());
}

#[test]
fn current_mode_flat_projection() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let conditional = record(&mut writer, |writer| {
        writer.if_current_break("broken\nbranch");
        writer.if_current_flat("flat");
    });
    writer.write_flat_fragment(conditional.try_flat().unwrap());
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "flat");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.flat(|writer| {
        writer.if_current_break("broken\nbranch");
        writer.if_current_flat("flat");
    });
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "flat");
}

fn recorded_group_document(write: impl FnOnce(&mut Writer<'_>), inherited: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (group, ()) = writer
        .record_group_with_value(|writer| {
            write(writer);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    writer.group(true, |writer| {
        let fragment = if inherited {
            group.contents()
        } else {
            group.grouped()
        };
        writer.write_fragment(fragment);
    });
    writer.finish().print(FormatOptions::prettier())
}

fn completed_group_document(write: impl FnOnce(&mut Writer<'_>), expand: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let content = record(&mut writer, write);
    let group = writer.group_fragment(expand, content);
    writer.group(true, |writer| writer.write_fragment(group));
    writer.finish().print(FormatOptions::prettier())
}

fn write_group_lines(writer: &mut Writer<'_>) {
    writer.text("first");
    writer.line();
    writer.if_flat("flat");
    writer.if_break("broken");
}

#[test]
fn recorded_group_layouts() {
    assert_eq!(
        recorded_group_document(write_group_lines, false),
        "first flat"
    );

    assert_eq!(
        recorded_group_document(write_group_lines, true),
        "first\nbroken"
    );

    assert_eq!(
        completed_group_document(write_group_lines, false),
        "first flat"
    );

    assert_eq!(
        completed_group_document(write_group_lines, true),
        "first\nbroken"
    );
}

fn write_group_suffix(writer: &mut Writer<'_>) {
    writer.text("first");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// trailing"));
    writer.soft_line();
    writer.text("last");
}

#[test]
fn recorded_group_boundaries() {
    assert_eq!(
        recorded_group_document(write_group_suffix, false),
        "first // trailing\nlast"
    );

    assert_eq!(
        recorded_group_document(write_group_suffix, true),
        "first // trailing\nlast"
    );

    assert_eq!(
        completed_group_document(write_group_suffix, false),
        "first // trailing\nlast"
    );
}

fn write_group_named_scope(writer: &mut Writer<'_>) {
    let group = writer.named_group(true, |writer| {
        writer.text("first");
        writer.line();
        writer.text("last");
    });
    writer.soft_line_if_break(group);
    writer.text("tail");
}

#[test]
fn recorded_group_named_scopes() {
    assert_eq!(
        recorded_group_document(write_group_named_scope, false),
        "first\nlast\ntail"
    );

    assert_eq!(
        recorded_group_document(write_group_named_scope, true),
        "first\nlast\ntail"
    );

    assert_eq!(
        completed_group_document(write_group_named_scope, false),
        "first\nlast\ntail"
    );
}

fn completed_delimiter_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let prefix = record(&mut writer, |writer| {
        writer.soft_line();
        writer.text("source,");
        writer.line();
    });
    let content = record(&mut writer, |writer| {
        writer.text("{");
        writer.indent(|writer| {
            writer.line();
            writer.text("value");
        });
        writer.line();
        writer.text("}");
    });
    let suffix = record(&mut writer, Writer::soft_line);
    let normal = writer.group_fragment(false, content);
    let forced = writer.group_fragment(true, content);
    let fallback = writer.indented_delimited_fragment("(", prefix, normal, suffix, ")");
    let prefix = prefix.try_flat().unwrap();
    let suffix = suffix.try_flat().unwrap();
    let grouped = writer.flat_delimited_fragment("(", prefix, normal, suffix, ")");
    let expanded = writer.flat_delimited_fragment("(", prefix, forced, suffix, ")");
    writer.write_alternatives(grouped, &[grouped, expanded], fallback);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn completed_delimiter_layouts() {
    assert_eq!(completed_delimiter_document(20), "(source, { value })");

    assert_eq!(completed_delimiter_document(12), "(source, {\n  value\n})");

    assert_eq!(
        completed_delimiter_document(8),
        "(\n  source,\n  {\n    value\n  }\n)"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (prefix, ()) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(true, |writer| writer.text("source"));
            writer.soft_line_if_break(group);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    let body = record(&mut writer, |writer| writer.text("value"));
    let suffix = record(&mut writer, |writer| writer.text("tail"));
    let projected = writer.flat_delimited_fragment(
        "(",
        prefix.try_flat().unwrap(),
        body,
        suffix.try_flat().unwrap(),
        ")",
    );
    assert!(!projected.preferred_breaks());
    assert!(!projected.conditional_lines);
    assert!(projected.try_neutral().is_none());
    let unused = writer.indented_delimited_fragment("(", prefix, body, suffix, ")");
    assert!(unused.preferred_breaks());
    assert!(unused.conditional_lines);
    writer.write_fragment(projected);
    let doc = writer.finish();
    assert_eq!(doc.group_count, 1);
    assert!(!doc.conditional_lines);
    assert_eq!(doc.print(FormatOptions::prettier()), "(sourcevaluetail)");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let prefix = record(&mut writer, |writer| {
        let group = writer.named_group(true, |writer| writer.text("<T>"));
        writer.soft_line_if_break(group);
    });
    let body = record(&mut writer, |writer| writer.text("call()"));
    let enclosed =
        writer.conditional_group_fragment(prefix, GroupRequest::Expanded, "(", body, ")");
    writer.write_fragment(enclosed);
    writer.line_suffix(|writer| write_comment_suffix(writer, "// outside"));
    writer.hard_line();
    let doc = writer.finish();
    assert_eq!(doc.group_count, 1);
    assert!(doc.conditional_lines);
    assert_eq!(
        doc.print(FormatOptions::prettier()),
        "<T>\n(\n  call()\n) // outside\n"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let prefix = record(&mut writer, |writer| {
        let group = writer.named_group(true, |writer| writer.text("<T>"));
        writer.soft_line_if_break(group);
    });
    let body = record(&mut writer, write_group_named_scope);
    let enclosed =
        writer.conditional_group_fragment(prefix, GroupRequest::Expanded, "(", body, ")");
    writer.write_flat_fragment(enclosed.try_flat().unwrap());
    writer.line_suffix(|writer| write_comment_suffix(writer, "// outside"));
    writer.hard_line();
    let doc = writer.finish();
    assert_eq!(doc.group_count, 2);
    assert!(!doc.conditional_lines);
    assert_eq!(
        doc.print(FormatOptions::prettier()),
        "<T>first lasttail // outside\n"
    );
}

fn neutral_continuation(width: u16, neutral: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.group(false, |writer| writer.text("abc"));
    });
    writer.group(false, |writer| {
        if neutral {
            writer.write_neutral_fragment(fragment.try_neutral().unwrap());
        } else {
            writer.write_fragment(fragment);
        }
        writer.line();
        writer.if_flat("flat");
        writer.if_break("broken");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn neutral_named_scope(neutral: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.group(false, |writer| writer.text("宽\tz"));
    });
    let group = writer.named_group(true, |writer| {
        writer.text(">");
        if neutral {
            writer.write_neutral_fragment(fragment.try_neutral().unwrap());
        } else {
            writer.write_fragment(fragment);
        }
        writer.line();
        writer.if_flat("flat");
        writer.if_break("broken");
    });
    writer.soft_line_if_break(group);
    writer.text("after");
    writer.finish().print(FormatOptions::prettier())
}

fn neutral_fill(width: u16, neutral: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.group(false, |writer| writer.text("abc"));
    });
    writer.group(false, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| {
                if neutral {
                    writer.write_neutral_fragment(fragment.try_neutral().unwrap());
                } else {
                    writer.write_fragment(fragment);
                }
            });
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("xy"));
        });
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn neutral_replay() {
    assert_eq!(neutral_continuation(8, false), "abc flat");
    assert_eq!(
        neutral_continuation(8, true),
        neutral_continuation(8, false)
    );

    assert_eq!(neutral_continuation(7, false), "abc\nbroken");
    assert_eq!(
        neutral_continuation(7, true),
        neutral_continuation(7, false)
    );

    assert_eq!(neutral_named_scope(false), ">宽\tz\nbroken\nafter");
    assert_eq!(neutral_named_scope(true), neutral_named_scope(false));

    assert_eq!(neutral_fill(6, false), "abc xy");
    assert_eq!(neutral_fill(6, true), neutral_fill(6, false));

    assert_eq!(neutral_fill(5, false), "abc\nxy");
    assert_eq!(neutral_fill(5, true), neutral_fill(5, false));
}

fn retained_pair(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let pair = record(&mut writer, |writer| write_pair(writer, "aa", "bb"));
    writer.write_fragment(pair);
    writer.text(";");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn retained_fill(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let inner = record(&mut writer, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| write_pair(writer, "a", "b"));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("x"));
        });
    });
    let outer = record(&mut writer, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.write_fragment(inner));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("y"));
        });
    });
    writer.text("prefix:");
    writer.hard_line();
    writer.write_fragment(outer);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn retained_fill_suffix(width: u16, inherited: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (group, ()) = writer
        .record_group_with_value(|writer| {
            writer.group(true, |writer| {
                writer.fill(|writer| {
                    writer.fill_entry(|writer| writer.text("aa"));
                    writer.fill_entry(Writer::line);
                    let (entry, ()) = writer.fill_entry_with_handle(|writer| {
                        writer.group(false, |writer| writer.text("bb"));
                    });
                    writer.extend_fill_entry(entry, |writer| {
                        writer.if_break("!");
                        writer.if_flat(".");
                    });
                });
            });
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    let fragment = if inherited {
        group.contents()
    } else {
        group.grouped()
    };
    writer.text(">");
    writer.hard_line();
    writer.write_fragment(fragment);
    writer.hard_line();
    writer.write_fragment(fragment);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn retained_fill_suffixes() {
    assert_eq!(retained_fill_suffix(6, false), ">\naa bb!\naa bb!");

    assert_eq!(retained_fill_suffix(6, true), ">\naa bb!\naa bb!");

    assert_eq!(retained_fill_suffix(5, false), ">\naa\nbb!\naa\nbb!");

    assert_eq!(retained_fill_suffix(5, true), ">\naa\nbb!\naa\nbb!");
}

fn rejected_candidate(options: FormatOptions, alternatives: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (preferred, first) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(false, |writer| {
                writer.text("a");
                writer.indent(|writer| {
                    writer.align(3, |writer| {
                        writer.line_suffix(|writer| write_comment_suffix(writer, "// rejected"));
                        writer.text("too wide for the available line");
                        writer.hard_line();
                        writer.text("unselected");
                    })
                });
            });
            Ok::<_, std::convert::Infallible>(group)
        })
        .unwrap();
    let (fallback, selected) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(true, |writer| {
                writer.text("a");
                writer.indent(|writer| {
                    writer.line();
                    writer.text("b");
                });
            });
            Ok::<_, std::convert::Infallible>(group)
        })
        .unwrap();
    if alternatives {
        writer.write_alternatives(preferred, &[preferred], fallback);
    } else {
        writer.write_fragment(fallback);
    }
    writer.indent_if_break(first, |writer| {
        writer.hard_line();
        writer.text("first");
    });
    writer.indent_if_break(selected, |writer| {
        writer.hard_line();
        writer.text("selected");
    });
    writer.finish().print(options)
}

#[test]
fn retained_scopes() {
    assert_eq!(retained_pair(9), "[aa, bb];");

    assert_eq!(retained_pair(8), "[\n  aa,\n  bb,\n];");

    assert_eq!(retained_fill(20), "prefix:\n[a, b] x y");

    assert_eq!(retained_fill(6), "prefix:\n[a, b]\nx\ny");
}

#[test]
fn candidate_state() {
    let narrow = FormatOptions {
        line_width: 8,
        ..FormatOptions::prettier()
    };
    assert_eq!(
        rejected_candidate(narrow, true),
        "a\n  b\nfirst\n  selected"
    );
    assert_eq!(
        rejected_candidate(narrow, true),
        rejected_candidate(narrow, false)
    );

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(rejected_candidate(tabs, true), "a\n\tb\nfirst\n\tselected");
    assert_eq!(
        rejected_candidate(tabs, true),
        rejected_candidate(tabs, false)
    );
}

fn source_break_fill() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.text("// required");
        writer.break_parent();
        writer.soft_line();
        writer.text("next");
    });
    writer.group(false, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.write_fragment(fragment));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("tail"));
        });
    });
    writer.finish().print(FormatOptions::prettier())
}

fn source_break_candidate(expanded: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| {
        let contents = |writer: &mut Writer<'_>| {
            writer.text("// required");
            writer.break_parent();
            writer.soft_line();
            writer.text("next");
        };
        if expanded {
            writer.group(true, |writer| {
                contents(writer);
                writer.if_break(" broken");
            });
        } else {
            contents(writer);
        }
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.group(false, |writer| {
        writer.write_alternatives(preferred, &[preferred], fallback)
    });
    writer.finish().print(FormatOptions::prettier())
}

#[test]
fn retained_flat_scopes() {
    assert_eq!(source_break_fill(), "// required\nnext\ntail");

    assert_eq!(source_break_candidate(false), "fallback");

    assert_eq!(source_break_candidate(true), "// required\nnext broken");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let mandatory = record(&mut writer, |writer| {
        writer.text("a");
        writer.hard_line();
        writer.text("b");
    });
    assert!(mandatory.try_flat().is_none());
    let literal = record(&mut writer, |writer| writer.literal_text("a\nb"));
    assert!(literal.try_flat().is_none());
    let raw = record(&mut writer, |writer| writer.text("a\nb"));
    assert!(raw.try_flat().is_none());

    let carriage_return = record(&mut writer, |writer| writer.text("a\rb"));
    assert!(carriage_return.try_flat().is_none());

    let line_separator = record(&mut writer, |writer| writer.text("a\u{2028}b"));
    assert!(line_separator.try_flat().is_none());

    let paragraph_separator = record(&mut writer, |writer| writer.text("a\u{2029}b"));
    assert!(paragraph_separator.try_flat().is_none());

    let unicode = record(&mut writer, |writer| writer.text("宽文字"));
    assert!(unicode.try_flat().is_some());

    let tab = record(&mut writer, |writer| writer.text("a\tb"));
    assert!(tab.try_flat().is_some());

    let suffix = record(&mut writer, |writer| {
        writer.text("a");
        writer.line_suffix(|writer| write_comment_suffix(writer, "// kept"));
        writer.line();
        writer.text("b");
    });
    assert!(suffix.try_flat().is_none());
    let source_break = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.text("// required line");
            writer.break_parent();
            writer.soft_line();
            writer.text("next");
        })
    });
    assert!(source_break.try_flat().is_none());
    let compact = record(&mut writer, |writer| write_pair(writer, "a", "b"));
    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(compact, &[compact], mandatory)
    });
    let flat = choices.try_flat().unwrap();
    writer.group(false, |writer| {
        writer.text("before");
        writer.line();
        writer.write_flat_fragment(flat);
        writer.line();
        writer.text("after");
    });
    writer.hard_line();
    writer.write_fragment(mandatory);
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "before [a, b] after\na\nb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| write_pair(writer, "first", "second"));
    let fallback = record(&mut writer, |writer| {
        writer.text("fallback");
        writer.hard_line();
        writer.text("break");
    });
    writer.flat(|writer| writer.write_alternatives(preferred, &[preferred], fallback));
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 3,
            ..FormatOptions::prettier()
        }),
        "[first, second]"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = record(&mut writer, |writer| {
        writer.text("required");
        writer.hard_line();
        writer.text("line");
    });
    let fallback = record(&mut writer, |writer| writer.text("flat"));
    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(first, &[first], fallback)
    });
    assert!(choices.try_flat().is_none());
    writer.flat(|writer| writer.write_fragment(choices));
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "required\nline"
    );
}

fn choice_continuation(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| writer.text("abc"));
    let fallback = record(&mut writer, |writer| {
        writer.group(true, |writer| {
            writer.text("a");
            writer.line();
            writer.text("b");
        })
    });
    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(preferred, &[preferred], fallback)
    });
    writer.group(false, |writer| {
        writer.write_fragment(choices);
        writer.line();
        writer.if_flat("flat");
        writer.if_break("broken");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn candidate_continuation() {
    assert_eq!(choice_continuation(8), "abc flat");
    assert_eq!(choice_continuation(7), "abc\nbroken");
    assert_eq!(choice_continuation(2), "a\nb\nbroken");
}

fn named_line_reuse(alternatives: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (signature, group) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(false, |writer| {
                writer.text("a");
                writer.line();
                writer.text("b");
            });
            Ok::<_, std::convert::Infallible>(group)
        })
        .unwrap();
    writer.text("123456");
    writer.write_fragment(signature);
    writer.hard_line();
    let fallback = record(&mut writer, |writer| {
        writer.write_fragment(signature);
        writer.soft_line_if_break(group);
        writer.text("123456");
    });
    if alternatives {
        let preferred = record(&mut writer, |writer| writer.text("rejected candidate"));
        writer.write_alternatives(preferred, &[preferred], fallback);
    } else {
        writer.write_fragment(fallback);
    }
    writer.finish().print(FormatOptions {
        line_width: 8,
        ..FormatOptions::prettier()
    })
}

#[test]
fn named_line_scopes() {
    assert_eq!(named_line_reuse(false), "123456a\nb\na\nb\n123456");
    assert_eq!(named_line_reuse(true), named_line_reuse(false));

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (named, id) = writer
        .record_with_value(|writer| {
            let id = writer.named_group(false, |writer| writer.text("a"));
            Ok::<_, std::convert::Infallible>(id)
        })
        .unwrap();
    writer.group(true, |writer| {
        writer.write_fragment(named);
        writer.soft_line_if_break(id);
        writer.text("b");
    });
    let fragment = record(&mut writer, |writer| {
        writer.named_group(true, |writer| writer.text("c"));
        writer.soft_line_if_break(id);
    });
    writer.write_flat_fragment(fragment.try_flat().unwrap());
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "abc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let id = writer.named_group(true, |writer| writer.text("a"));
    writer.group(false, |writer| {
        writer.soft_line_if_break(id);
        writer.text("b");
    });
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "ab");
}

#[test]
fn alternative_expansion_boundaries() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| {
        writer.group(true, |writer| {
            writer.text("body");
            writer.line();
            writer.text("value");
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    let scope = record(&mut writer, |writer| {
        writer.write_alternatives(fallback, &[preferred], fallback)
    });
    assert!(preferred.preferred_breaks());
    assert!(!scope.preferred_breaks());
    writer.group(false, |writer| {
        writer.text("head ");
        writer.if_flat("F");
        writer.write_fragment(scope);
        writer.if_current_flat(" tail");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "head Fbody\nvalue tail"
    );

    let mut writer = Writer::new(&allocator);
    let normal = preferred;
    let inherited = record(&mut writer, |writer| {
        writer.write_alternatives(normal, &[normal], fallback);
    });
    assert!(inherited.preferred_breaks());
    let flat = inherited
        .try_flat()
        .expect("expansion metadata permits a flat view");
    writer.write_flat_fragment(flat);
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "body value"
    );
}

fn write_preserved_pair(writer: &mut Writer<'_>) {
    writer.group_with_request(GroupRequest::PreserveSource, |writer| {
        writer.text("a");
        writer.line();
        writer.text("bbb");
    });
}

fn canonical_group_document(request: GroupRequest, width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, |writer| {
        writer.group_with_request(request, |writer| {
            writer.group(false, write_preserved_pair);
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.write_canonical_alternatives(candidate, &[candidate.canonical_candidate()], fallback);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn canonical_group_scopes() {
    assert_eq!(canonical_group_document(GroupRequest::Auto, 4), "fallback");

    assert_eq!(canonical_group_document(GroupRequest::Auto, 5), "a\nbbb");

    assert_eq!(
        canonical_group_document(GroupRequest::Expanded, 4),
        "a\nbbb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group_with_request(GroupRequest::PreserveSource, |writer| {
        write_preserved_pair(writer);
        writer.line();
        write_preserved_pair(writer);
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a\nbbb\na\nbbb"
    );
}

fn canonical_named_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (candidate, id) = writer
        .record_with_value(|writer| {
            let id = writer.group(false, |writer| {
                let id = writer.named_group_with_request(GroupRequest::PreserveSource, |writer| {
                    writer.text("a");
                    writer.line();
                    writer.if_flat("ffff");
                    writer.if_break("b");
                });
                writer.soft_line_if_break(id);
                writer.text("cc");
                id
            });
            Ok::<_, std::convert::Infallible>(id)
        })
        .unwrap();
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.write_canonical_alternatives(candidate, &[candidate.canonical_candidate()], fallback);
    writer.indent_if_break(id, |writer| {
        writer.hard_line();
        writer.text("after");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn canonical_named_scopes() {
    assert_eq!(canonical_named_document(8), "a\nb\ncc\n  after");

    assert_eq!(canonical_named_document(7), "fallback\nafter");
}

fn canonical_nested_alternative(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preserved = record(&mut writer, write_preserved_pair);
    let unused = record(&mut writer, |writer| writer.text("unused"));
    let candidate = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.write_alternatives(preserved, &[preserved], unused);
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.write_canonical_alternatives(candidate, &[candidate.canonical_candidate()], fallback);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn canonical_continuation(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, |writer| writer.text("aaa"));
    let fallback = record(&mut writer, |writer| writer.text("k"));
    let continuation = record(&mut writer, |writer| {
        writer.if_current_break("!");
        writer.group_with_request(GroupRequest::PreserveSource, |writer| {
            writer.text("x");
            writer.line();
            writer.text("long value");
        });
    });
    writer.write_canonical_alternatives(candidate, &[candidate.canonical_candidate()], fallback);
    writer.write_fragment(continuation);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn canonical_choice_continuation() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = record(&mut writer, |writer| writer.text("aaa"));
    let first_fallback = record(&mut writer, |writer| writer.text("k"));
    let second = record(&mut writer, write_preserved_pair);
    let normal = record(&mut writer, |writer| writer.text("a"));
    let second_fallback = record(&mut writer, |writer| writer.text("z"));
    writer.group(false, |writer| {
        writer.write_canonical_alternatives(first, &[first.canonical_candidate()], first_fallback);
        writer.write_canonical_alternatives(
            normal,
            &[second.canonical_candidate()],
            second_fallback,
        );
    });
    writer.finish().print(FormatOptions {
        line_width: 4,
        ..FormatOptions::prettier()
    })
}

#[test]
fn canonical_candidate_boundaries() {
    assert_eq!(canonical_nested_alternative(4), "fallback");

    assert_eq!(canonical_nested_alternative(5), "a\nbbb");

    assert_eq!(canonical_continuation(5), "aaa!x\nlong value");

    assert_eq!(canonical_continuation(4), "k!x\nlong value");

    assert_eq!(canonical_choice_continuation(), "aaaz");
}

fn canonical_fill_document() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, write_preserved_pair);
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.fill(|writer| {
        writer.fill_entry(|writer| {
            writer.write_canonical_alternatives(
                candidate,
                &[candidate.canonical_candidate()],
                fallback,
            );
            writer.if_current_flat("F");
            writer.if_current_break("B");
        });
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("z"));
    });
    writer.finish().print(FormatOptions::prettier())
}

fn canonical_display_group() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, write_preserved_pair);
    let normal = record(&mut writer, |writer| writer.text("a"));
    let fallback = record(&mut writer, |writer| writer.text("f"));
    writer.group(false, |writer| {
        writer.if_flat("F");
        writer.if_break("B");
        writer.write_canonical_alternatives(normal, &[candidate.canonical_candidate()], fallback);
        writer.line();
        writer.text("tail");
    });
    writer.finish().print(FormatOptions {
        line_width: 8,
        ..FormatOptions::prettier()
    })
}

#[test]
fn canonical_display_probes() {
    assert_eq!(canonical_fill_document(), "a\nbbbB\nz");

    assert_eq!(canonical_display_group(), "Ff tail");
}

fn canonical_normal_effect(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let normal = record(&mut writer, write_preserved_pair);
    let preferred = record(&mut writer, |writer| writer.text("a"));
    let unused = record(&mut writer, |writer| writer.text("b"));
    let candidate = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.write_alternatives(normal, &[preferred], unused);
            writer.line();
            writer.text("bbb");
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.write_canonical_alternatives(candidate, &[candidate.canonical_candidate()], fallback);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn canonical_retained_views() {
    assert_eq!(canonical_normal_effect(4), "fallback");

    assert_eq!(canonical_normal_effect(5), "a\nbbb");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preserved = record(&mut writer, write_preserved_pair);
    let body = record(&mut writer, |writer| writer.text("body"));
    let flat = preserved.try_flat().unwrap();
    let scope = writer.flat_delimited_fragment("(", flat, body, flat, ")");
    writer.group(false, |writer| {
        writer.if_flat("F");
        writer.write_fragment(scope);
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "F(a bbbbodya bbb)"
    );

    let mut writer = Writer::new(&allocator);
    let normal = writer.indented_delimited_fragment("(", body, preserved, body, ")");
    writer.group(false, |writer| {
        writer.if_break("B");
        writer.write_fragment(normal);
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "B(bodya\n  bbbbody)"
    );
}

#[test]
fn fill_entry_boundaries() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer
        .fill_with_empty_line_policy(|writer| {
            let (entry, ()) = writer.fill_entry_with_handle(|writer| writer.text("first"));
            writer.extend_fill_entry(entry, |writer| writer.text("!"));
            writer.fill_entry(|writer| {
                writer.hard_line();
                writer.hard_line();
            });
            writer.fill_entry(|writer| writer.text("second"));
            Ok::<_, ()>(((), true))
        })
        .unwrap();
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first!\nsecond"
    );
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

fn remeasurement_document(
    write_line: impl FnOnce(&mut Writer<'_>),
    fill: bool,
    bounded: bool,
) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let line = record(&mut writer, write_line);
    let line = super::Fragment {
        effect: super::BreakEffect::default(),
        ..line
    };
    writer.group(false, |writer| {
        writer.text("head");
        writer.write_fragment(line);
        let value = |writer: &mut Writer<'_>| {
            if bounded {
                writer.group_with_flat_width(remeasured_value);
            } else {
                writer.group(false, remeasured_value);
            }
        };
        if fill {
            writer.fill(|writer| writer.fill_entry(value));
        } else {
            value(writer);
        }
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
        remeasurement_document(|writer| writer.hard_line(), false, false),
        "head\n{\n  longName\n} tail"
    );

    assert_eq!(
        remeasurement_document(|writer| writer.literal_line(), false, false),
        "head\n{\n  longName\n} tail"
    );

    assert_eq!(
        remeasurement_document(|writer| writer.text("\n"), false, false),
        "head\n{\n  longName\n} tail"
    );

    assert_eq!(
        remeasurement_document(|writer| writer.hard_line(), false, true),
        "head\n{\n  longName\n} tail"
    );
}

#[test]
fn fill_measurement_boundaries() {
    assert_eq!(
        remeasurement_document(|writer| writer.literal_line(), true, false),
        "head\n{\n  longName\n} tail"
    );
}

fn remeasured_fill_continuation(depth: u8) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let line = record(&mut writer, Writer::literal_line);
    let line = super::Fragment {
        effect: super::BreakEffect::default(),
        ..line
    };
    let mut item = record(&mut writer, |writer| writer.group(false, remeasured_value));
    for _ in 0..depth {
        item = record(&mut writer, |writer| {
            writer.fill(|writer| writer.fill_entry(|writer| writer.write_fragment(item)));
        });
    }
    writer.group(false, |writer| {
        writer.text("head");
        writer.write_fragment(line);
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.write_fragment(item));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("next"));
        });
    });
    writer.finish().print(FormatOptions {
        line_width: 12,
        ..FormatOptions::prettier()
    })
}

#[test]
fn fill_remeasurement_continuations() {
    assert_eq!(remeasured_fill_continuation(0), "head\n{longName}\nnext");

    assert_eq!(remeasured_fill_continuation(1), "head\n{longName}\nnext");

    assert_eq!(remeasured_fill_continuation(3), "head\n{longName}\nnext");
}

fn remeasured_fill_scope(request: GroupRequest) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.fill(|writer| {
        writer.fill_entry(|writer| {
            writer.text("first");
            writer.hard_line();
            writer.text("a");
        });
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| {
            writer.group_with_request(request, |writer| {
                writer.text("force");
                writer.line();
                writer.text("group");
            });
        });
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| {
            writer.group(false, |writer| {
                writer.text("longName");
                writer.line();
                writer.text("name");
            });
        });
    });
    writer.text(" tail");
    writer.finish().print(FormatOptions {
        line_width: 15,
        ..FormatOptions::prettier()
    })
}

#[test]
fn fill_remeasurement_scopes() {
    assert_eq!(
        remeasured_fill_scope(GroupRequest::Auto),
        "first\na force group\nlongName name tail"
    );

    assert_eq!(
        remeasured_fill_scope(GroupRequest::Expanded),
        "first\na force\ngroup\nlongName name tail"
    );

    assert_eq!(
        remeasured_fill_scope(GroupRequest::PreserveSource),
        "first\na force\ngroup\nlongName name tail"
    );
}

fn remeasured_fill_after_break(depth: u8) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let mut item = record(&mut writer, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| {
                writer.text("first");
                writer.literal_line();
                writer.text("a");
            });
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.group(true, |writer| writer.text("force")));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| {
                writer.text("{cccc");
                writer.line();
                writer.text("cccc}");
            });
        });
    });
    for _ in 0..depth {
        item = record(&mut writer, |writer| {
            writer.fill(|writer| writer.fill_entry(|writer| writer.write_fragment(item)));
        });
    }
    writer.write_fragment(item);
    writer.finish().print(FormatOptions {
        line_width: 10,
        ..FormatOptions::prettier()
    })
}

fn remeasured_fill_after_expansion() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.fill(|writer| {
        writer.fill_entry(|writer| {
            writer.text("first");
            writer.hard_line();
            writer.group(true, |writer| writer.text("force:"));
            writer.fill(|writer| {
                writer.fill_entry(|writer| writer.text("abcdefgh"));
                writer.fill_entry(Writer::line);
                writer.fill_entry(|writer| {
                    writer.text("bb");
                    writer.line();
                    writer.text("longName");
                });
            });
        });
    });
    writer.finish().print(FormatOptions {
        line_width: 18,
        ..FormatOptions::prettier()
    })
}

fn remeasured_fill_after_measurement() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let item = record(&mut writer, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| {
                writer.text("first");
                writer.hard_line();
                writer.text("a");
                writer.group(false, |_| {});
            });
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.group(true, |writer| writer.text("force")));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("{}"));
        });
    });
    writer.fill(|writer| writer.fill_entry(|writer| writer.write_fragment(item)));
    writer.finish().print(FormatOptions {
        line_width: 10,
        ..FormatOptions::prettier()
    })
}

#[test]
fn fill_measurement_invalidation() {
    assert_eq!(
        remeasured_fill_after_break(0),
        "first\na force\n{cccc\ncccc}"
    );

    assert_eq!(
        remeasured_fill_after_break(1),
        "first\na force\n{cccc\ncccc}"
    );

    assert_eq!(
        remeasured_fill_after_break(3),
        "first\na force\n{cccc\ncccc}"
    );

    assert_eq!(
        remeasured_fill_after_expansion(),
        "first\nforce:abcdefgh\nbb longName"
    );

    assert_eq!(remeasured_fill_after_measurement(), "first\na force\n{}");
}
