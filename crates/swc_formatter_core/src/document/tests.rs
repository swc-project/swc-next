use swc_next_allocator::Allocator;

use super::{ConditionTarget, Doc, FormatElement, Fragment, GroupRequest, Writer};
use crate::{IndentStyle, LineEnding, PrintOptions};

fn write_comment_suffix<'a>(writer: &mut Writer<'a>, text: &'a str) {
    writer.space();
    writer.text(text);
    writer.break_parent();
}

fn nested_document(options: PrintOptions) -> String {
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

fn line_document(options: PrintOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first");
    writer.hard_line();
    writer.text("second");
    writer.hard_line();
    writer.finish().print(options)
}

fn line_request_document<'a>(
    allocator: &'a Allocator,
    line: impl FnOnce(&mut Writer<'a>),
) -> Doc<'a> {
    let mut writer = Writer::new(allocator);
    writer.group(false, |writer| {
        writer.text("a");
        line(writer);
        writer.text("b");
        writer.line();
        writer.text("c");
        writer.if_break("B");
        writer.if_flat("F");
    });
    writer.finish()
}

fn indented_line_doc<'a>(allocator: &'a Allocator, line: impl FnOnce(&mut Writer<'a>)) -> Doc<'a> {
    let mut writer = Writer::new(allocator);
    writer.group(false, |writer| {
        writer.indent(|writer| {
            writer.text("a ");
            line(writer);
            writer.text("b");
        });
        writer.if_break("B");
        writer.if_flat("F");
    });
    writer.finish()
}

fn independent_document(allocator: &Allocator) -> Doc<'_> {
    let mut writer = Writer::new(allocator);
    let group = writer.named_group(false, |writer| {
        writer.text("first");
        writer.line();
        writer.text("second");
    });
    writer.indent_if_break(group, |writer| {
        writer.hard_line();
        writer.text("last");
    });
    writer.finish()
}

#[test]
fn compact_borrows() {
    use super::thin::{ThinSlice, ThinStr};

    let values = [String::from("first"), String::from("second")];
    let borrowed = ThinSlice::new(&values);
    let copied = borrowed;
    assert_eq!(borrowed.get(), values);
    assert_eq!(copied.get().as_ptr(), values.as_ptr());

    let empty: &[u64] = &[];
    let borrowed = ThinSlice::new(empty);
    assert_eq!(borrowed.get(), empty);
    assert_eq!(borrowed.get().as_ptr(), empty.as_ptr());

    let source = String::from("世界👩\u{200d}💻");
    let text = ThinStr::new(&source);
    assert_eq!(text.as_str(), source);
    assert_eq!(text.as_str().as_ptr(), source.as_ptr());

    // Packed instruction payloads can place the stored pointer off its natural
    // alignment. Reconstructing the borrow must copy it before dereferencing.
    #[repr(C, packed)]
    struct Packed<'a> {
        prefix: u8,
        values: ThinSlice<'a, u64>,
        text: ThinStr<'a>,
    }
    let values = [1, 2, 3];
    let packed = Packed {
        prefix: 0,
        values: ThinSlice::new(&values),
        text,
    };
    let borrowed = packed.values;
    let text = packed.text;
    assert_eq!(borrowed.get(), values);
    assert_eq!(text.as_str(), source);
}

#[test]
fn compact_scopes() {
    let allocator = Allocator::new();
    let source = String::from("世界");
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| writer.text(source.as_str()));
    let flat = fragment.try_flat().unwrap();
    writer.write_fragment(fragment);
    writer.line_suffix(|writer| {
        writer.text_copied(" // generated");
        writer.break_parent();
    });
    writer.hard_line();
    writer.write_flat_fragment(flat);
    let doc = writer.finish();
    // Later arena allocations must preserve the sealed instructions and text.
    let _later = allocator.alloc_str(&"later".repeat(1024));
    assert_eq!(
        doc.print(PrintOptions::default()),
        "世界 // generated\n世界"
    );
}

#[test]
fn compact_widths() {
    use super::text::Text;

    let ascii = "a".repeat(80_000);
    let text = Text::new(&ascii);
    assert_eq!(text.clean_ascii_width(), Some(80_000));
    assert_eq!(text.first_line(17), (80_017, false));

    let unicode = "界".repeat(40_000);
    let text = Text::new(&unicode);
    assert_eq!(text.clean_width(), Some(usize::from(u16::MAX)));
    assert_eq!(text.first_line(17), (usize::from(u16::MAX) + 17, false));

    let controls = "界\tend";
    let text = Text::new(controls);
    assert_eq!(text.clean_width(), Some(6));
    assert_eq!(text.first_line(17), (23, false));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn compact_bounds() {
    let too_long = [(); u32::MAX as usize + 1];
    assert!(std::panic::catch_unwind(|| super::thin::ThinSlice::new(&too_long)).is_err());
}

#[test]
fn independent_documents() {
    let allocator = Allocator::new();
    let narrow = independent_document(&allocator);
    let wide = independent_document(&allocator);
    let options = PrintOptions {
        line_width: 8,
        indent_style: IndentStyle::Tab,
        line_ending: LineEnding::CrLf,
        ..PrintOptions::default()
    };

    assert_eq!(narrow.print(options), "first\r\nsecond\r\n\tlast");
    assert_eq!(wide.print(PrintOptions::default()), "first second\nlast");
}

fn aligned_document(options: PrintOptions) -> String {
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
        aligned_document(PrintOptions::default()),
        "prefix (first second)"
    );

    let narrow = PrintOptions {
        line_width: 8,
        ..PrintOptions::default()
    };
    assert_eq!(aligned_document(narrow), "prefix\n  first\n      second");

    let tabs = PrintOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(aligned_document(tabs), "prefix\n\tfirst\n\t\t\tsecond");
}

fn absolute_document(width: usize, options: PrintOptions) -> String {
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
    let spaces = PrintOptions::default();
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

    let tabs = PrintOptions {
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

    let zero = PrintOptions {
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
    writer.finish().print(PrintOptions::default())
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn flat_width_document(width: u16, write: impl FnOnce(&mut Writer<'_>)) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    write(&mut writer);
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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

fn prepared_boundary(
    width: u16,
    before: &'static str,
    after: &'static str,
    write: impl FnOnce(&mut Writer<'_>),
) -> String {
    flat_width_document(width, |writer| {
        writer.text(before);
        writer.group_with_flat_width(write);
        writer.text(after);
    })
}

fn write_prepared_lines(writer: &mut Writer<'_>) {
    writer.line();
    writer.text("b");
    writer.line();
}

fn write_empty_prepared_lines(writer: &mut Writer<'_>) {
    writer.line();
    writer.line();
}

fn prepared_empty_continuation(width: u16) -> String {
    flat_width_document(width, |writer| {
        writer.group(false, |writer| {
            writer.text("a");
            writer.line();
            writer.group_with_flat_width(|_| {});
            writer.text("b");
        });
    })
}

fn prepared_nested_continuation(width: u16) -> String {
    flat_width_document(width, |writer| {
        writer.group_with_flat_width(|writer| {
            writer.text("a");
            writer.line();
            writer.group_with_flat_width(|writer| {
                writer.text("b");
                writer.line();
                writer.text("c");
            });
        });
        writer.text(" d");
    })
}

#[test]
fn prepared_group_continuations() {
    assert_eq!(
        prepared_boundary(5, "a", "c", write_prepared_lines),
        "a b c"
    );

    assert_eq!(
        prepared_boundary(4, "a", "c", write_prepared_lines),
        "a\nb\nc"
    );

    assert_eq!(
        prepared_boundary(3, "a", "c", write_empty_prepared_lines),
        "a  c"
    );

    assert_eq!(
        prepared_boundary(1, "a", "", write_empty_prepared_lines),
        "a  "
    );

    assert_eq!(prepared_empty_continuation(2), "a\nb");

    assert_eq!(prepared_nested_continuation(6), "a\nb c d");

    assert_eq!(prepared_nested_continuation(4), "a\nb\nc d");
}

fn flat_width_spaces(literal: bool) -> String {
    flat_width_document(5, |writer| {
        writer.text("a");
        if literal {
            writer.text(" ");
        } else {
            writer.space();
        }
        writer.group_with_flat_width(|writer| {
            writer.space();
            writer.if_break("(");
            writer.text("b");
            writer.space();
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
fn prepared_group_spaces() {
    assert_eq!(flat_width_spaces(false), "a  (b  c )\nd");

    assert_eq!(flat_width_spaces(true), "a  (b  c )\nd");

    let empty_leading = flat_width_document(4, |writer| {
        writer.text("a");
        writer.text(" ");
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

    assert_eq!(flat_width_text("a\tb", 6), "a a\tb;");

    assert_eq!(flat_width_text("a\tb", 5), "a (\n  a\tb\n);");

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

fn probe_space_continuation(literal: bool) -> String {
    flat_width_document(3, |writer| {
        writer.group(false, |writer| {
            writer.text("a");
            writer.line();
            writer.text("b");
        });
        if literal {
            writer.text(" ");
        } else {
            writer.space();
        }
        writer.hard_line();
        writer.text("c");
    })
}

#[test]
fn probe_space_continuations() {
    assert_eq!(probe_space_continuation(false), "a\nb\nc");

    assert_eq!(probe_space_continuation(true), "a\nb\nc");
}

fn dependent_document(options: PrintOptions) -> String {
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
        dependent_document(PrintOptions::default()),
        "target = [first, second];"
    );

    let medium = PrintOptions {
        line_width: 16,
        ..PrintOptions::default()
    };
    assert_eq!(
        dependent_document(medium),
        "target = [\n  first,\n  second,\n];"
    );

    let narrow = PrintOptions {
        line_width: 8,
        ..medium
    };
    assert_eq!(
        dependent_document(narrow),
        "target =\n  [\n    first,\n    second,\n  ];"
    );

    let tabs = PrintOptions {
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
        writer.finish().print(PrintOptions::default()),
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
        writer.finish().print(PrintOptions {
            line_width: 8,
            ..PrintOptions::default()
        }),
        "first\nsecond\n  last"
    );
}

fn fill_document(parts: &[&str], width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let group = writer.next_group_id();
    writer.group_with_id(GroupRequest::Auto, Some(group), |writer| {
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
                            writer.if_break_for(ConditionTarget::Group(group), ",");
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn grouped_fill_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let group = writer.next_group_id();
    writer.group_with_id(GroupRequest::Auto, Some(group), |writer| {
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
                    writer.if_break_for(ConditionTarget::Group(group), ",");
                });
            });
        });
        writer.soft_line();
        writer.text("]");
    });
    writer.text(";");
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
        writer.finish().print(PrintOptions::default()),
        "first second"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.literal_text("first\r\nsecond");
    let options = PrintOptions {
        line_ending: LineEnding::CrLf,
        ..PrintOptions::default()
    };
    assert_eq!(writer.finish().print(options), "first\r\nsecond");
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
        writer.finish().print(PrintOptions::default()),
        format!("source source {} end", "x".repeat(32 * 1024))
    );
}

#[test]
fn line_endings() {
    let lf = PrintOptions::default();
    assert_eq!(line_document(lf), "first\nsecond\n");

    let crlf = PrintOptions {
        line_ending: LineEnding::CrLf,
        ..lf
    };
    assert_eq!(line_document(crlf), "first\r\nsecond\r\n");

    let cr = PrintOptions {
        line_ending: LineEnding::Cr,
        ..lf
    };
    assert_eq!(line_document(cr), "first\rsecond\r");
}

#[test]
fn indentation() {
    let spaces = PrintOptions::default();
    assert_eq!(nested_document(spaces), "{\n  {\n    value;\n  }\n}\n");

    let tabs = PrintOptions {
        indent_style: IndentStyle::Tab,
        ..spaces
    };
    assert_eq!(nested_document(tabs), "{\n\t{\n\t\tvalue;\n\t}\n}\n");

    let zero = PrintOptions {
        indent_width: 0.try_into().unwrap(),
        ..spaces
    };
    assert_eq!(nested_document(zero), "{\n{\nvalue;\n}\n}\n");

    let crlf = PrintOptions {
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
        writer.finish().print(PrintOptions::default()),
        "first\n\n\n  last "
    );
}

#[test]
fn standard_line_spaces() {
    let adjacent = flat_width_document(3, |writer| {
        writer.group(false, |writer| {
            writer.text("a");
            writer.line();
            writer.line();
            writer.text("b");
        });
    });
    assert_eq!(adjacent, "a  b");

    let trailing = flat_width_document(3, |writer| {
        writer.group(false, |writer| {
            writer.text("a");
            writer.line();
        });
    });
    assert_eq!(trailing, "a ");

    let projected = flat_width_document(3, |writer| {
        let lines = record(writer, |writer| {
            writer.line();
            writer.line();
        });
        writer.text("a");
        writer.write_flat_fragment(lines.try_flat().unwrap());
        writer.text("b");
    });
    assert_eq!(projected, "a  b");

    let fragments = flat_width_document(3, |writer| {
        let line = record(writer, Writer::line);
        writer.group(false, |writer| {
            writer.text("a");
            writer.write_fragment(line);
            writer.write_fragment(line);
            writer.text("b");
        });
    });
    assert_eq!(fragments, "a  b");
}

#[test]
fn literal_spaces() {
    let lines = flat_width_document(3, |writer| {
        writer.group(false, |writer| {
            writer.text("a");
            writer.space();
            writer.line();
            writer.space();
            writer.line();
            writer.space();
            writer.text("b");
        });
    });
    assert_eq!(lines, "a\n\n b");

    let text = flat_width_document(3, |writer| {
        writer.text("a");
        writer.space();
        writer.text(" ");
        writer.space();
        writer.text("b");
    });
    assert_eq!(text, "a   b");

    let raw = flat_width_document(80, |writer| {
        writer.text("a");
        writer.space();
        writer.text("  ");
        writer.text("b");
    });
    assert_eq!(raw, "a   b");
}

#[test]
fn standard_line_whitespace() {
    let ordinary = flat_width_document(80, |writer| {
        writer.text("a  \t");
        writer.hard_line();
        writer.text("b");
    });
    assert_eq!(ordinary, "a\nb");

    let literal = flat_width_document(80, |writer| {
        writer.text("a  \t");
        writer.literal_line();
        writer.text("b");
    });
    assert_eq!(literal, "a  \t\nb");

    let aligned = flat_width_document(80, |writer| {
        writer.indent(|writer| {
            writer.hard_line();
            writer.text("a  ");
            writer.literal_line_with_indent();
            writer.text("b");
        });
    });
    assert_eq!(aligned, "\n  a  \n  b");

    let pending_indent = flat_width_document(80, |writer| {
        writer.text("a");
        writer.indent(Writer::hard_line);
        writer.literal_line();
        writer.text("b");
    });
    assert_eq!(pending_indent, "a\n  \nb");

    let terminal_indent = flat_width_document(80, |writer| {
        writer.text("a");
        writer.indent(Writer::hard_line);
    });
    assert_eq!(terminal_indent, "a\n  ");
}

#[test]
fn deep_indentation() {
    let allocator = Allocator::new();
    let mut elements = vec![FormatElement::StartIndent; 1025];
    elements.push(FormatElement::HardLine);
    elements.push(FormatElement::text(super::text::Text::new("value")));
    elements.extend(std::iter::repeat_n(FormatElement::EndIndent, 1025));
    let document = Doc {
        allocator: &allocator,
        elements: &elements,
        group_count: 0,
    };
    let options = PrintOptions {
        indent_width: 64.try_into().unwrap(),
        ..PrintOptions::default()
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
        writer.finish().print(PrintOptions::default()),
        "{\n  /* first \n original */\n  value;\n}"
    );
}

#[test]
fn line_break_requests() {
    let allocator = Allocator::new();
    let wide = PrintOptions::default();
    let narrow = PrintOptions {
        line_width: 0,
        ..wide
    };

    let doc = line_request_document(&allocator, Writer::hard_line_without_break_parent);
    assert_eq!(doc.print(wide), "a\nb cF");

    let doc = line_request_document(&allocator, Writer::literal_line_without_break_parent);
    assert_eq!(doc.print(wide), "a\nb cF");

    let doc = line_request_document(&allocator, Writer::hard_line_without_break_parent);
    assert_eq!(doc.print(narrow), "a\nb\ncB");

    let doc = line_request_document(&allocator, Writer::literal_line_without_break_parent);
    assert_eq!(doc.print(narrow), "a\nb\ncB");

    let doc = line_request_document(&allocator, |writer| {
        writer.hard_line_without_break_parent();
        writer.break_parent();
    });
    assert_eq!(doc.print(wide), "a\nb\ncB");

    let doc = line_request_document(&allocator, |writer| {
        writer.literal_line_without_break_parent();
        writer.break_parent();
    });
    assert_eq!(doc.print(wide), "a\nb\ncB");

    let doc = indented_line_doc(&allocator, Writer::hard_line_without_break_parent);
    assert_eq!(doc.print(wide), "a\n  bF");

    let doc = indented_line_doc(&allocator, Writer::literal_line_without_break_parent);
    assert_eq!(doc.print(wide), "a \nbF");

    let mut writer = Writer::new(&allocator);
    let hard = record(&mut writer, Writer::hard_line_without_break_parent);
    assert!(hard.can_break());
    assert!(hard.has_intrinsic_lines());
    assert!(!hard.preferred_breaks());
    assert!(hard.try_flat().is_none());

    let literal = record(&mut writer, Writer::literal_line_without_break_parent);
    assert!(literal.can_break());
    assert!(literal.has_intrinsic_lines());
    assert!(!literal.preferred_breaks());
    assert!(literal.try_flat().is_none());
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
    assert_eq!(writer.finish().print(PrintOptions::default()), "a\nb\nc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.space();
        writer.line();
        writer.text("b");
    });
    assert_eq!(
        writer.finish().print(PrintOptions {
            line_width: 3,
            ..PrintOptions::default()
        }),
        "a\nb"
    );
}

fn literal_space_continuation(width: u16) -> String {
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
    writer.text(" ");
    writer.soft_line();
    writer.text("rest");
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn literal_space_fill(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("aa");
    writer.text(" ");
    writer.fill(|writer| {
        writer.fill_entry(|writer| writer.text("b"));
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("c"));
    });
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn literal_space_candidate(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, |writer| {
        writer.group_with_request(GroupRequest::Expanded, |writer| {
            writer.soft_line();
            writer.text("b");
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("F"));
    writer.text("aaa");
    writer.text(" ");
    writer.write_alternatives(&[candidate], fallback);
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

#[test]
fn literal_space_gap_measurement() {
    assert_eq!(literal_space_continuation(6), "(aaa)\nrest");

    assert_eq!(literal_space_continuation(5), "(\n  aaa\n)\nrest");

    assert_eq!(literal_space_fill(6), "aa b c");

    assert_eq!(literal_space_fill(5), "aa b\nc");

    assert_eq!(literal_space_candidate(5), "aaa\nb");

    assert_eq!(literal_space_candidate(4), "aaa\nb");

    assert_eq!(literal_space_candidate(3), "aaa F");
}

fn literal_space_retry_continuation(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = record(&mut writer, |writer| {
        writer.text("x");
        writer.text(" ");
        writer.text("too long");
    });
    let second = record(&mut writer, |writer| {
        writer.text("b");
        writer.text(" ");
    });
    let fallback = record(&mut writer, |writer| writer.text("F"));
    writer.text("a");
    writer.text(" ");
    writer.write_alternatives(&[first, second], fallback);
    writer.group(false, |writer| {
        writer.soft_line();
        writer.if_flat("c");
        writer.if_break("C");
    });
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

#[test]
fn literal_space_gap_selection_boundaries() {
    assert_eq!(literal_space_retry_continuation(5), "a b c");

    assert_eq!(literal_space_retry_continuation(4), "a b\nC");

    assert_eq!(literal_space_retry_continuation(3), "a F\nC");
}

#[test]
fn literal_space_projection() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let gap = record(&mut writer, |writer| {
        writer.space();
        writer.text(" ");
    });
    assert!(gap.try_neutral().is_some());
    writer.text("a");
    writer.write_neutral_fragment(gap.try_neutral().unwrap());
    writer.text("b");
    assert_eq!(writer.finish().print(PrintOptions::default()), "a  b");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let gap = record(&mut writer, |writer| {
        writer.text(" ");
        writer.line();
    });
    assert!(gap.try_flat().is_some());
    writer.text("a");
    writer.write_flat_fragment(gap.try_flat().unwrap());
    writer.text("b");
    assert_eq!(writer.finish().print(PrintOptions::default()), "a  b");
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
        writer.finish().print(PrintOptions::default()),
        "a, // comment\nb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// last"));
    assert_eq!(writer.finish().print(PrintOptions::default()), "a // last");
}

#[test]
fn line_suffix_boundaries() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix_boundary();
    writer.text("b");
    assert_eq!(writer.finish().print(PrintOptions::default()), "ab");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix(|writer| write_comment_suffix(writer, "// first"));
    writer.line_suffix_boundary();
    writer.line_suffix(|writer| write_comment_suffix(writer, "// second"));
    writer.hard_line();
    writer.text("b");
    assert_eq!(
        writer.finish().print(PrintOptions::default()),
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
        writer.finish().print(PrintOptions::default()),
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
    assert_eq!(writer.finish().print(PrintOptions::default()), "abc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.line_suffix(|writer| writer.text("// long suffix"));
        writer.line();
        writer.text("b");
    });
    let narrow = PrintOptions {
        line_width: 4,
        ..PrintOptions::default()
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
    assert_eq!(writer.finish().print(PrintOptions::default()), "abxzy");
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
    assert_eq!(writer.finish().print(PrintOptions::default()), "abF\nc");

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
    assert_eq!(writer.finish().print(PrintOptions::default()), "ax\n  y\nb");
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
        writer.finish().print(PrintOptions::default()),
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
        writer.finish().print(PrintOptions::default()),
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
                writer.literal_text(*part);
            }
            Ok::<_, ()>(())
        })
        .unwrap()
}

fn clean_fragment_width<'a>(writer: &mut Writer<'a>, fragment: Fragment<'a>) -> Option<usize> {
    writer
        .write_with_clean_text_width(|writer| {
            writer.write_fragment(fragment);
            Ok::<_, ()>(())
        })
        .unwrap()
}

#[test]
fn complete_text_measurement() {
    assert_eq!(clean_parts_width(&["ASCII", "\u{7f}"]), Some(6));
    assert_eq!(clean_parts_width(&["é\u{7f}"]), Some(2));
    assert_eq!(clean_parts_width(&["a\u{7f}", "é"]), Some(3));
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
    assert_eq!(clean_fragment_width(&mut writer, text), Some(2));

    let expanded = writer.group_fragment(true, text);
    assert_eq!(clean_fragment_width(&mut writer, expanded), None);
}

#[test]
fn cached_text_columns() {
    assert_eq!(pair_document("لا", "a", 7), "[لا, a];");
    assert_eq!(pair_document("لا", "a", 6), "[\n  لا,\n  a,\n];");
    assert_eq!(pair_document("a\u{200d}bcd", "a", 10), "[a\u{200d}bcd, a];");
    assert_eq!(
        pair_document("a\u{200d}bcd", "a", 9),
        "[\n  a\u{200d}bcd,\n  a,\n];"
    );
    assert_eq!(pair_document("a\u{7f}b", "a", 9), "[a\u{7f}b, a];");
    assert_eq!(
        pair_document("a\u{7f}b", "a", 8),
        "[\n  a\u{7f}b,\n  a,\n];"
    );
    assert_eq!(pair_document("👩\u{200d}💻", "a", 8), "[👩\u{200d}💻, a];");

    let tab = super::text::Text::new("a\u{7f}\té");
    assert_eq!(tab.clean_width(), Some(4));
    assert_eq!(tab.first_line(0), (4, false));
    assert_eq!(tab.last_line(0), (4, false));
}

fn literal_text_continuation(text: &'static str) -> String {
    flat_width_document(8, |writer| {
        writer.text("pre");
        writer.literal_text(text);
        writer.group(false, |writer| {
            writer.line();
            writer.text("next");
        });
    })
}

#[test]
fn text_line_measurement() {
    let ascii = super::text::Text::new("ASCII");
    assert_eq!(ascii.last_line(3), (8, false));

    let unicode = super::text::Text::new("世界");
    assert_eq!(unicode.last_line(3), (7, false));

    let controls = super::text::Text::new("a\u{7f}\té");
    assert_eq!(controls.last_line(3), (7, false));

    let tab = super::text::Text::new("a\tb");
    assert_eq!(tab.first_line(0), (3, false));
    assert_eq!(tab.last_line(3), (6, false));

    let separators = super::text::Text::new("a\u{2028}b\u{2029}c");
    assert_eq!(separators.last_line(3), (8, false));

    assert_eq!(
        literal_text_continuation("é\r\n\u{7f}b"),
        "preé\n\u{7f}b next"
    );

    assert_eq!(
        literal_text_continuation("head\rtail"),
        "prehead\ntail\nnext"
    );

    assert_eq!(
        literal_text_continuation("head\n\t世界"),
        "prehead\n\t世界\nnext"
    );

    assert_eq!(
        literal_text_continuation("head\r\ntail\n"),
        "prehead\ntail\n next"
    );
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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

fn fill_entry_document(width: u16, write: impl FnOnce(&mut Writer<'_>)) -> String {
    flat_width_document(width, |writer| writer.fill(write))
}

#[test]
fn fill_separator_layouts() {
    let separator_lines = fill_entry_document(5, |writer| {
        writer.fill_entry(|writer| writer.text("a"));
        writer.fill_entry(|writer| {
            writer.line();
            writer.line();
        });
        writer.fill_entry(|writer| writer.text("b"));
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("c"));
    });
    assert_eq!(separator_lines, "a  b\nc");

    let trailing_line = fill_entry_document(5, |writer| {
        writer.fill_entry(|writer| {
            writer.text("a");
            writer.line();
        });
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("b"));
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("c"));
    });
    assert_eq!(trailing_line, "a  b\nc");

    let empty_item = fill_entry_document(1, |writer| {
        writer.fill_entry(|writer| writer.text("a"));
        writer.fill_entry(Writer::line);
        writer.fill_entry(|_| {});
        writer.fill_entry(Writer::soft_line);
        writer.fill_entry(|_| {});
    });
    assert_eq!(empty_item, "a\n");

    let summarized_separator = fill_entry_document(5, |writer| {
        writer.fill_entry(|writer| writer.text("a"));
        writer.fill_entry(|writer| {
            writer.group_with_flat_width(|writer| {
                writer.line();
                writer.line();
            });
        });
        writer.fill_entry(|writer| writer.text("b"));
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("c"));
    });
    assert_eq!(summarized_separator, "a  b\nc");
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
        writer.finish().print(PrintOptions {
            line_width: 5,
            ..PrintOptions::default()
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
        writer.finish().print(PrintOptions::default()),
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
        writer.finish().print(PrintOptions::default()),
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
        writer.finish().print(PrintOptions {
            line_width: 5,
            ..PrintOptions::default()
        }),
        "[first, second]\nlast"
    );
}

fn current_mode_fill(width: u16, expand_next: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let group = writer.next_group_id();
    writer.group_with_id(GroupRequest::Expanded, Some(group), |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.text("aa"));
            writer.fill_entry(|writer| {
                writer.if_break_for(ConditionTarget::Group(group), ",");
                writer.if_flat_for(ConditionTarget::Group(group), "unused");
                writer.if_break("!");
                writer.if_flat(" ");
                writer.soft_line();
            });
            writer.fill_entry(|writer| writer.group(expand_next, |writer| writer.text("bb")));
        });
    });
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn current_mode_group(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.if_break("B");
        writer.if_flat("1234");
        writer.line();
        writer.text("x");
    });
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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

fn conditional_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let broken = record(&mut writer, |writer| {
        writer.indent(|writer| {
            writer.line();
            writer.text("b");
        });
        writer.line();
        writer.text("c");
    });
    let flat = record(&mut writer, |writer| {
        writer.text(" ");
        writer.group(false, |writer| {
            writer.text("d");
            writer.line();
            writer.text("e");
        });
    });
    writer.group(false, |writer| {
        writer.text("a");
        writer.if_break_doc(ConditionTarget::Current, broken, flat);
    });
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn named_condition_document(printed: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let group = writer.next_group_id();
    let named = record(&mut writer, |writer| {
        writer.group_with_id(GroupRequest::Auto, Some(group), |writer| {
            writer.text("a");
            writer.line();
            writer.text("b");
        });
    });
    if printed {
        writer.text("xxxx");
        writer.write_fragment(named);
        writer.hard_line();
    }
    writer.group(false, |writer| {
        writer.write_fragment(named);
        writer.line();
        writer.if_break_for(ConditionTarget::Group(group), "!");
        writer.if_flat_for(ConditionTarget::Group(group), "long");
    });
    writer.finish().print(PrintOptions {
        line_width: 5,
        ..PrintOptions::default()
    })
}

#[test]
fn conditional_documents() {
    assert_eq!(conditional_document(20), "a d e");

    assert_eq!(conditional_document(4), "a\n  b\nc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let broken = record(&mut writer, |writer| writer.text("B"));
    let flat = record(&mut writer, |writer| {
        writer.hard_line();
        writer.text("flat");
    });
    writer.group(false, |writer| {
        writer.text("a");
        writer.if_break_doc(ConditionTarget::Current, broken, flat);
        writer.text("b");
    });
    assert_eq!(writer.finish().print(PrintOptions::default()), "aBb");

    let mut writer = Writer::new(&allocator);
    let empty = record(&mut writer, |_| {});
    writer.group(false, |writer| {
        writer.text("a");
        writer.if_break_doc(ConditionTarget::Current, empty, empty);
        writer.line();
        writer.text("b");
    });
    let narrow = PrintOptions {
        line_width: 3,
        ..PrintOptions::default()
    };
    assert_eq!(writer.finish().print(narrow), "a b");
}

#[test]
fn named_condition_measurement() {
    assert_eq!(named_condition_document(false), "a b\nlong");

    // A repeated group still has its previously printed mode during probing.
    // Printing the accepted flat candidate then updates that binding.
    assert_eq!(named_condition_document(true), "xxxxa\nb\na b long");
}

fn named_indent_projection(printed: bool, retained: bool) -> String {
    flat_width_document(80, |writer| {
        let group = writer.next_group_id();
        if printed {
            writer.group_with_id(GroupRequest::Expanded, Some(group), |writer| {
                writer.text("g");
            });
        }
        let write = |writer: &mut Writer<'_>| {
            writer.text("a");
            writer.indent_if_break(group, |writer| {
                writer.line();
                writer.text("F");
            });
            writer.text("b");
        };
        if retained {
            let fragment = record(writer, write);
            writer.write_flat_fragment(fragment.try_flat().unwrap());
        } else {
            writer.flat(write);
        }
        writer.hard_line();
        writer.text("z");
    })
}

fn named_indent_suffix_projection(binding: Option<bool>, retained: bool, nested: bool) -> String {
    flat_width_document(80, |writer| {
        let group = writer.next_group_id();
        if let Some(expand) = binding {
            writer.group_with_id(expand.into(), Some(group), |writer| writer.text("g"));
            writer.literal_line();
        }
        writer.text("a");
        writer.line_suffix(|writer| writer.text("S"));
        let write = |writer: &mut Writer<'_>| {
            writer.indent_if_break(group, |writer| {
                let body = |writer: &mut Writer<'_>| {
                    writer.line_suffix_boundary();
                    writer.text("x");
                };
                if nested {
                    writer.indent_if_break(group, body);
                } else {
                    body(writer);
                }
            });
        };
        if retained {
            let fragment = record(writer, write);
            writer.write_flat_fragment(fragment.try_flat().unwrap());
        } else {
            writer.flat(write);
        }
        writer.hard_line();
        writer.text("z");
    })
}

#[test]
fn named_indent_suffix_bindings() {
    for retained in [false, true] {
        assert_eq!(
            named_indent_suffix_projection(None, retained, false),
            "aS\nz"
        );

        assert_eq!(
            named_indent_suffix_projection(Some(false), retained, false),
            "g\naS\nx\nz"
        );

        assert_eq!(
            named_indent_suffix_projection(Some(true), retained, false),
            "g\naS\n  x\nz"
        );

        assert_eq!(
            named_indent_suffix_projection(Some(true), retained, true),
            "g\naS\n    x\nz"
        );
    }
}

#[test]
fn named_condition_bindings() {
    let unprinted = flat_width_document(80, |writer| {
        let group = writer.next_group_id();
        writer.text("a");
        writer.if_break_for(ConditionTarget::Group(group), "B");
        writer.if_flat_for(ConditionTarget::Group(group), "F");
        writer.text("b");
    });
    assert_eq!(unprinted, "ab");

    let unprinted_indent = flat_width_document(80, |writer| {
        let group = writer.next_group_id();
        writer.text("a");
        writer.indent_if_break(group, |writer| writer.text("F"));
        writer.text("b");
    });
    assert_eq!(unprinted_indent, "ab");

    let retained_definition = flat_width_document(80, |writer| {
        let group = writer.flat(|writer| writer.named_group(true, |writer| writer.text("a")));
        writer.if_break_for(ConditionTarget::Group(group), "B");
        writer.if_flat_for(ConditionTarget::Group(group), "F");
    });
    assert_eq!(retained_definition, "aB");

    let fill_binding = flat_width_document(80, |writer| {
        let group = writer.next_group_id();
        let definition = record(writer, |writer| {
            writer.group_with_id(GroupRequest::Expanded, Some(group), |_| {});
        });
        assert!(definition.try_flat().is_none());
        writer.write_fragment(definition);
        let projected = record(writer, |writer| {
            writer.flat(|writer| writer.write_fragment(definition));
        });
        let empty = record(writer, |_| {});
        writer.group(true, |writer| {
            writer.fill(|writer| {
                writer.fill_entry(|writer| {
                    writer.if_break_doc(ConditionTarget::Current, empty, projected);
                });
            });
        });
        writer.if_break_for(ConditionTarget::Group(group), "B");
        writer.if_flat_for(ConditionTarget::Group(group), "F");
    });
    assert_eq!(fill_binding, "B");

    assert_eq!(named_indent_projection(false, false), "ab\nz");

    assert_eq!(named_indent_projection(false, true), "ab\nz");

    assert_eq!(named_indent_projection(true, false), "ga Fb\nz");

    assert_eq!(named_indent_projection(true, true), "ga Fb\nz");
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

    let named = record(&mut writer, |writer| {
        writer.if_break_doc(ConditionTarget::Group(group), soft, soft);
    });
    assert!(named.can_break());

    let flat_named = record(&mut writer, |writer| {
        writer.write_flat_fragment(named.try_flat().unwrap());
    });
    assert!(!flat_named.can_break());

    let hard = record(&mut writer, Writer::hard_line);
    assert!(hard.can_break());

    let literal = record(&mut writer, Writer::literal_line);
    assert!(literal.can_break());

    let current = record(&mut writer, |writer| {
        writer.if_break_doc(ConditionTarget::Current, soft, atom);
    });
    assert!(current.can_break());

    let flat_current = record(&mut writer, |writer| {
        writer.write_flat_fragment(current.try_flat().unwrap())
    });
    assert!(!flat_current.can_break());

    let multiline = record(&mut writer, |writer| writer.literal_text("first\nlast"));
    assert!(multiline.can_break());

    let fallback_only = record(&mut writer, |writer| {
        writer.write_alternatives(&[atom], hard)
    });
    assert!(!fallback_only.can_break());

    let first = record(&mut writer, |writer| {
        writer.write_alternatives(&[soft], atom)
    });
    assert!(first.can_break());
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
    assert!(!atom.has_intrinsic_lines());

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
    assert!(!conditional.has_intrinsic_lines());

    let flat_conditional = record(&mut writer, |writer| {
        writer.write_flat_fragment(conditional.try_flat().unwrap());
    });
    assert!(flat_conditional.try_neutral().is_none());
    assert!(flat_conditional.try_flat().is_some());

    let current = record(&mut writer, write_current_mode_lines);
    assert!(current.try_neutral().is_none());
    assert!(current.try_flat().is_some());
    assert!(!current.has_intrinsic_lines());

    let projected = record(&mut writer, |writer| {
        writer.flat(|writer| writer.write_fragment(current));
    });
    assert!(projected.try_neutral().is_none());
    assert!(projected.try_flat().is_some());
    assert!(!projected.has_intrinsic_lines());
    writer.write_flat_fragment(projected.try_flat().unwrap());

    let empty = record(&mut writer, |_| {});
    let multiline = record(&mut writer, |writer| writer.literal_text("first\nlast"));
    let multiline_current = record(&mut writer, |writer| {
        writer.if_break_doc(ConditionTarget::Current, empty, multiline);
    });
    assert!(multiline_current.try_neutral().is_none());
    assert!(multiline_current.try_flat().is_none());

    let named = record(&mut writer, |writer| {
        writer.named_group(false, |writer| {
            writer.text("value");
            writer.soft_line();
        });
    });
    assert!(named.try_neutral().is_none());
    assert!(named.try_flat().is_none());
    assert!(!named.has_intrinsic_lines());

    let id = writer.next_group_id();
    let named_fragment = writer.group_fragment_with_id(GroupRequest::Auto, Some(id), atom);
    assert!(named_fragment.try_neutral().is_none());
    assert!(named_fragment.try_flat().is_none());

    let expanded = record(&mut writer, |writer| {
        writer.group(true, |writer| writer.text("value"));
    });
    assert!(expanded.try_neutral().is_none());
    assert!(expanded.try_flat().is_some());
    assert!(!expanded.has_intrinsic_lines());

    let propagation = record(&mut writer, Writer::break_parent);
    assert!(propagation.try_flat().is_none());
    assert!(!propagation.has_intrinsic_lines());

    let mandatory = record(&mut writer, |writer| {
        writer.text("first");
        writer.hard_line();
        writer.text("last");
    });
    assert!(mandatory.try_neutral().is_none());
    assert!(mandatory.try_flat().is_none());
    assert!(mandatory.has_intrinsic_lines());

    let suffix = record(&mut writer, |writer| {
        writer.line_suffix(|writer| writer.text("// tail"));
    });
    assert!(suffix.has_intrinsic_lines());

    let broken_lines = record(&mut writer, |writer| {
        writer.if_break_doc(ConditionTarget::Current, mandatory, atom);
    });
    assert!(broken_lines.has_intrinsic_lines());

    let flat_lines = record(&mut writer, |writer| {
        writer.if_break_doc(ConditionTarget::Current, atom, mandatory);
    });
    assert!(flat_lines.has_intrinsic_lines());

    let multiline = record(&mut writer, |writer| writer.text("first\u{2028}last"));
    assert!(multiline.try_neutral().is_none());
    assert!(multiline.try_flat().is_none());

    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(&[atom], expanded);
    });
    assert!(choices.try_neutral().is_none());
    assert!(choices.try_flat().is_some());

    let flattened = record(&mut writer, |writer| {
        writer.flat(|writer| {
            writer.group(true, |writer| {
                writer.text("value");
                writer.if_break(",");
                writer.if_flat(".");
                write_current_mode_lines(writer);
            });
        });
    });
    assert!(flattened.try_flat().is_some());
    let mut flattened_writer = Writer::new(&allocator);
    flattened_writer.write_fragment(flattened);
    let output = flattened_writer.finish().print(PrintOptions::default());
    assert_eq!(output, "value.flat");

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
}

fn write_current_mode_lines(writer: &mut Writer<'_>) {
    let broken = record(writer, |writer| {
        writer.text("broken");
        writer.line();
        writer.text("branch");
    });
    let flat = record(writer, |writer| writer.text("flat"));
    writer.if_break_doc(ConditionTarget::Current, broken, flat);
}

#[test]
fn current_mode_flat_projection() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let conditional = record(&mut writer, write_current_mode_lines);
    writer.write_flat_fragment(conditional.try_flat().unwrap());
    assert_eq!(writer.finish().print(PrintOptions::default()), "flat");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.flat(write_current_mode_lines);
    assert_eq!(writer.finish().print(PrintOptions::default()), "flat");
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
    writer.finish().print(PrintOptions::default())
}

fn completed_group_document(write: impl FnOnce(&mut Writer<'_>), expand: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let content = record(&mut writer, write);
    let group = writer.group_fragment(expand, content);
    writer.group(true, |writer| writer.write_fragment(group));
    writer.finish().print(PrintOptions::default())
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

fn record_choice<'a>(writer: &mut Writer<'a>, write: impl FnOnce(&mut Writer<'a>)) -> Fragment<'a> {
    writer
        .record_with_value(|writer| {
            write(writer);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap()
        .0
}

fn selected_choice_document(
    options: PrintOptions,
    expand: bool,
    write: impl FnOnce(&mut Writer<'_>),
) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let compact = record_choice(&mut writer, write);
    let expanded = record_choice(&mut writer, |writer| {
        writer.text("f");
        writer.if_break("!");
    });
    writer.group(expand, |writer| {
        writer.write_alternatives(&[compact], expanded);
        writer.if_break("?");
        writer.if_flat(".");
    });
    writer.finish().print(options)
}

fn write_selected_choice_lines(writer: &mut Writer<'_>) {
    writer.text("a");
    writer.line();
    writer.text("b");
    writer.if_break(",");
}

fn write_selected_choice_nested(writer: &mut Writer<'_>) {
    let nested = record(writer, |writer| {
        writer.text("long");
        let group = writer.named_group(true, |writer| {
            writer.text("(");
            writer.hard_line();
            writer.text("inner");
            writer.if_break("!");
            writer.hard_line();
            writer.text(")");
        });
        writer.indent_if_break(group, |writer| {
            writer.hard_line();
            writer.text("named");
        });
    });
    let expanded = record(writer, |writer| {
        writer.text("other");
        writer.hard_line();
        writer.text("rest");
    });
    writer.text("a");
    writer.line();
    writer.write_alternatives(&[nested], expanded);
    writer.if_flat(".");
}

fn write_selected_choice_suffix(writer: &mut Writer<'_>) {
    writer.text("a");
    writer.line_suffix(|writer| {
        writer.if_flat(" flat");
        writer.if_break(" broken");
    });
    writer.group(true, |writer| {
        writer.hard_line();
        writer.text("b");
    });
}

fn selected_suffix_document() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let compact = record_choice(&mut writer, |writer| {
        writer.text("a");
        writer.line_suffix(|writer| {
            writer.if_flat(" flat");
            writer.if_break(" broken");
        });
    });
    writer.group(true, |writer| {
        writer.write_alternatives(&[compact], compact);
        writer.hard_line();
        writer.text("b");
        writer.if_break("?");
    });
    writer.finish().print(PrintOptions::default())
}

#[test]
fn selected_layout_modes() {
    let wide = PrintOptions {
        line_width: 4,
        ..PrintOptions::default()
    };
    let narrow = PrintOptions {
        line_width: 3,
        ..wide
    };
    let nested = PrintOptions {
        line_width: 20,
        ..wide
    };

    let output = selected_choice_document(wide, false, write_selected_choice_lines);
    assert_eq!(output, "a b.");

    let output = selected_choice_document(wide, true, write_selected_choice_lines);
    assert_eq!(output, "a b?");

    let output = selected_choice_document(narrow, false, write_selected_choice_lines);
    assert_eq!(output, "f!?");

    let output = selected_choice_document(narrow, true, write_selected_choice_lines);
    assert_eq!(output, "f!?");

    let output = selected_choice_document(nested, true, write_selected_choice_nested);
    assert_eq!(output, "a long(\ninner!\n)\n  named.?");

    let output = selected_choice_document(wide, true, write_selected_choice_nested);
    assert_eq!(output, "f!?");

    let output = selected_choice_document(nested, true, write_selected_choice_suffix);
    assert_eq!(output, "a flat\nb?");

    let output = selected_suffix_document();
    assert_eq!(output, "a flat\nb?");
}

#[test]
fn selected_layout_capabilities() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let text = record_choice(&mut writer, |writer| writer.text("text"));
    assert!(text.try_neutral().is_some());

    let conditional = record_choice(&mut writer, write_selected_choice_lines);
    assert!(conditional.try_neutral().is_none());
    writer.group(true, |writer| {
        writer.write_flat_fragment(conditional.try_flat().unwrap());
    });
    assert_eq!(writer.finish().print(PrintOptions::default()), "a b");

    let mut writer = Writer::new(&allocator);
    let required = record_choice(&mut writer, Writer::hard_line);
    assert!(required.try_flat().is_none());
    assert!(required.preferred_breaks());
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
    writer.finish().print(PrintOptions::default())
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn retained_fill_suffix(width: u16, inherited: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (group, ()) = writer
        .record_group_with_value(|writer| {
            let group = writer.next_group_id();
            writer.group_with_id(GroupRequest::Expanded, Some(group), |writer| {
                writer.fill(|writer| {
                    writer.fill_entry(|writer| writer.text("aa"));
                    writer.fill_entry(Writer::line);
                    let (entry, ()) = writer.fill_entry_with_handle(|writer| {
                        writer.group(false, |writer| writer.text("bb"));
                    });
                    writer.extend_fill_entry(entry, |writer| {
                        writer.if_break_for(ConditionTarget::Group(group), "!");
                        writer.if_flat_for(ConditionTarget::Group(group), ".");
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
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

#[test]
fn retained_fill_suffixes() {
    assert_eq!(retained_fill_suffix(6, false), ">\naa bb!\naa bb!");

    assert_eq!(retained_fill_suffix(6, true), ">\naa bb!\naa bb!");

    assert_eq!(retained_fill_suffix(5, false), ">\naa\nbb!\naa\nbb!");

    assert_eq!(retained_fill_suffix(5, true), ">\naa\nbb!\naa\nbb!");
}

fn rejected_candidate(options: PrintOptions, alternatives: bool) -> String {
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
        writer.write_alternatives(&[preferred], fallback);
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
    let narrow = PrintOptions {
        line_width: 8,
        ..PrintOptions::default()
    };
    assert_eq!(rejected_candidate(narrow, true), "a\n  b\n  selected");
    assert_eq!(
        rejected_candidate(narrow, true),
        rejected_candidate(narrow, false)
    );

    let tabs = PrintOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(rejected_candidate(tabs, true), "a\n\tb\n\tselected");
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
    writer.finish().print(PrintOptions::default())
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
        writer.write_alternatives(&[preferred], fallback)
    });
    writer.finish().print(PrintOptions::default())
}

#[test]
fn retained_flat_scopes() {
    assert_eq!(source_break_fill(), "// requirednext tail");

    assert_eq!(source_break_candidate(false), "// requirednext");

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
    let carriage_return = record(&mut writer, |writer| writer.literal_text("a\rb"));
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
        writer.write_alternatives(&[compact], mandatory)
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
        writer.finish().print(PrintOptions::default()),
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
    writer.flat(|writer| writer.write_alternatives(&[preferred], fallback));
    assert_eq!(
        writer.finish().print(PrintOptions {
            line_width: 3,
            ..PrintOptions::default()
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
        writer.write_alternatives(&[first], fallback)
    });
    assert!(choices.try_flat().is_none());
    writer.flat(|writer| writer.write_fragment(choices));
    assert_eq!(
        writer.finish().print(PrintOptions::default()),
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
        writer.write_alternatives(&[preferred], fallback)
    });
    writer.group(false, |writer| {
        writer.write_fragment(choices);
        writer.line();
        writer.if_flat("flat");
        writer.if_break("broken");
    });
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
        writer.write_alternatives(&[preferred], fallback);
    } else {
        writer.write_fragment(fallback);
    }
    writer.finish().print(PrintOptions {
        line_width: 8,
        ..PrintOptions::default()
    })
}

#[test]
fn named_line_scopes() {
    // The continuation sees the previous printed binding while the reused group
    // is measured; accepting that group then changes the named condition to flat.
    assert_eq!(named_line_reuse(false), "123456a\nb\na b123456");
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
    assert!(fragment.try_flat().is_none());
    writer.flat(|writer| writer.write_fragment(fragment));
    assert_eq!(writer.finish().print(PrintOptions::default()), "abc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let id = writer.named_group(true, |writer| writer.text("a"));
    writer.group(false, |writer| {
        writer.soft_line_if_break(id);
        writer.text("b");
    });
    assert_eq!(writer.finish().print(PrintOptions::default()), "ab");
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
        writer.write_alternatives(&[preferred], fallback)
    });
    assert!(preferred.preferred_breaks());
    assert!(!scope.preferred_breaks());
    writer.group(false, |writer| {
        writer.text("head ");
        writer.if_flat("F");
        writer.write_fragment(scope);
        writer.if_flat(" tail");
    });
    assert_eq!(
        writer.finish().print(PrintOptions::default()),
        "head Fbody\nvalue tail"
    );

    let mut writer = Writer::new(&allocator);
    let normal = preferred;
    let inherited = record(&mut writer, |writer| {
        writer.write_alternatives(&[normal], fallback);
    });
    assert!(!inherited.preferred_breaks());
    let flat = inherited
        .try_flat()
        .expect("expansion metadata permits a flat view");
    writer.write_flat_fragment(flat);
    assert_eq!(writer.finish().print(PrintOptions::default()), "body value");
}

fn write_pliteral_space_pair(writer: &mut Writer<'_>) {
    writer.group_with_request(GroupRequest::Expanded, |writer| {
        writer.text("a");
        writer.line();
        writer.text("bbb");
    });
}

fn conditional_group_document(request: GroupRequest, width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, |writer| {
        writer.group_with_request(request, |writer| {
            writer.group(false, write_pliteral_space_pair);
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.write_alternatives(&[candidate], fallback);
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

#[test]
fn conditional_group_scopes() {
    assert_eq!(conditional_group_document(GroupRequest::Auto, 4), "a\nbbb");

    assert_eq!(conditional_group_document(GroupRequest::Auto, 5), "a\nbbb");

    assert_eq!(
        conditional_group_document(GroupRequest::Expanded, 4),
        "a\nbbb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group_with_request(GroupRequest::Expanded, |writer| {
        write_pliteral_space_pair(writer);
        writer.line();
        write_pliteral_space_pair(writer);
    });
    assert_eq!(
        writer.finish().print(PrintOptions::default()),
        "a\nbbb\na\nbbb"
    );
}

fn conditional_named_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (candidate, id) = writer
        .record_with_value(|writer| {
            let id = writer.group(false, |writer| {
                let id = writer.named_group_with_request(GroupRequest::Expanded, |writer| {
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
    writer.write_alternatives(&[candidate], fallback);
    writer.indent_if_break(id, |writer| {
        writer.hard_line();
        writer.text("after");
    });
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

#[test]
fn conditional_named_scopes() {
    assert_eq!(conditional_named_document(8), "a\nb\ncc\n  after");

    assert_eq!(conditional_named_document(7), "a\nb\ncc\n  after");
}

fn conditional_nested_alternative(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preserved = record(&mut writer, write_pliteral_space_pair);
    let unused = record(&mut writer, |writer| writer.text("unused"));
    let candidate = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.write_alternatives(&[preserved], unused);
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.write_alternatives(&[candidate], fallback);
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn conditional_continuation(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, |writer| writer.text("aaa"));
    let fallback = record(&mut writer, |writer| writer.text("k"));
    let continuation = record(&mut writer, |writer| {
        writer.if_break("!");
        writer.group_with_request(GroupRequest::Expanded, |writer| {
            writer.text("x");
            writer.line();
            writer.text("long value");
        });
    });
    writer.write_alternatives(&[candidate], fallback);
    writer.write_fragment(continuation);
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

fn conditional_choice_continuation() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = record(&mut writer, |writer| writer.text("aaa"));
    let first_fallback = record(&mut writer, |writer| writer.text("k"));
    let second = record(&mut writer, write_pliteral_space_pair);
    let second_fallback = record(&mut writer, |writer| writer.text("z"));
    writer.group(false, |writer| {
        writer.write_alternatives(&[first], first_fallback);
        writer.write_alternatives(&[second], second_fallback);
    });
    writer.finish().print(PrintOptions {
        line_width: 4,
        ..PrintOptions::default()
    })
}

#[test]
fn conditional_candidate_boundaries() {
    assert_eq!(conditional_nested_alternative(4), "a\nbbb");

    assert_eq!(conditional_nested_alternative(5), "a\nbbb");

    assert_eq!(conditional_continuation(5), "aaa!x\nlong value");

    assert_eq!(conditional_continuation(4), "k!x\nlong value");

    assert_eq!(conditional_choice_continuation(), "aaaa\nbbb");
}

fn conditional_fill_document() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, write_pliteral_space_pair);
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.fill(|writer| {
        writer.fill_entry(|writer| {
            writer.write_alternatives(&[candidate], fallback);
            writer.if_flat("F");
            writer.if_break("B");
        });
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("z"));
    });
    writer.finish().print(PrintOptions::default())
}

fn conditional_display_group() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let candidate = record(&mut writer, write_pliteral_space_pair);
    let fallback = record(&mut writer, |writer| writer.text("f"));
    writer.group(false, |writer| {
        writer.if_flat("F");
        writer.if_break("B");
        writer.write_alternatives(&[candidate], fallback);
        writer.line();
        writer.text("tail");
    });
    writer.finish().print(PrintOptions {
        line_width: 8,
        ..PrintOptions::default()
    })
}

#[test]
fn conditional_display_probes() {
    assert_eq!(conditional_fill_document(), "a\nbbbB\nz");

    assert_eq!(conditional_display_group(), "Fa\nbbb tail");
}

fn conditional_nested_layout(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| writer.text("a"));
    let unused = record(&mut writer, |writer| writer.text("b"));
    let candidate = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.write_alternatives(&[preferred], unused);
            writer.line();
            writer.text("bbb");
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.write_alternatives(&[candidate], fallback);
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
    })
}

#[test]
fn conditional_retained_views() {
    assert_eq!(conditional_nested_layout(4), "fallback");

    assert_eq!(conditional_nested_layout(5), "a bbb");
}

#[test]
fn fill_entry_boundaries() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.fill_with_separator_policy(|writer| {
        let (entry, ()) = writer.fill_entry_with_handle(|writer| writer.text("first"));
        writer.extend_fill_entry(entry, |writer| writer.text("!"));
        writer.fill_entry(|writer| {
            writer.hard_line();
            writer.hard_line();
        });
        writer.fill_entry(|writer| writer.text("second"));
        ((), crate::FillSeparatorPolicy::CollapseEmptyLines)
    });
    assert_eq!(
        writer.finish().print(PrintOptions::default()),
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
    writer.group(false, |writer| {
        writer.text("head");
        // A standard conditional group contains the branch's break propagation,
        // so this physical line still prints in an inherited flat mode.
        writer.write_alternatives(&[line], line);
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
    writer.finish().print(PrintOptions {
        line_width: 12,
        ..PrintOptions::default()
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
        remeasurement_document(|writer| writer.literal_text("\r\n"), false, false),
        "head\n{\n  longName\n} tail"
    );

    assert_eq!(
        remeasurement_document(|writer| writer.hard_line(), false, true),
        "head\n{\n  longName\n} tail"
    );
}

fn fill_suffix_document(write: impl FnOnce(&mut Writer<'_>)) -> String {
    flat_width_document(4, |writer| {
        let suffix = record(writer, |writer| {
            writer.line_suffix(|writer| writer.fill(write));
        });
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.write_fragment(suffix));
        });
    })
}

#[test]
fn fill_measurement_boundaries() {
    assert_eq!(
        remeasurement_document(|writer| writer.literal_line(), true, false),
        "head\n{\n  longName\n} tail"
    );

    let propagated = flat_width_document(12, |writer| {
        writer.group(false, |writer| {
            writer.text("head");
            writer.literal_line();
            writer.fill(|writer| {
                writer.fill_entry(|writer| writer.group(false, remeasured_value));
            });
            writer.text(" tail");
        });
    });
    assert_eq!(propagated, "head\n{longName} tail");

    let suffix = fill_suffix_document(|writer| {
        writer.fill_entry(|writer| writer.text("aaaaa"));
        writer.fill_entry(Writer::line);
        writer.fill_entry(|writer| writer.text("b"));
    });
    assert_eq!(suffix, "aaaaa\nb");

    let forced_suffix = fill_suffix_document(|writer| {
        writer.fill_entry(|writer| writer.group(true, |writer| writer.text("a")));
        writer.fill_entry(Writer::line);
    });
    assert_eq!(forced_suffix, "a\n");

    let nested_lines = flat_width_document(4, |writer| {
        let inner = record(writer, |writer| {
            writer.fill(|writer| {
                writer.fill_entry(|writer| writer.text(" "));
                writer.fill_entry(Writer::line);
                writer.fill_entry(Writer::literal_line);
                writer.fill_entry(Writer::soft_line);
                writer.fill_entry(|writer| writer.text(" "));
            });
        });
        writer.fill(|writer| {
            writer.fill_entry(|writer| {
                writer.text("bb");
                writer.line();
                writer.write_fragment(inner);
            });
        });
    });
    assert_eq!(nested_lines, "bb   \n\n ");
}

fn remeasured_fill_continuation(depth: u8) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let line = record(&mut writer, Writer::literal_line);
    let mut item = record(&mut writer, |writer| writer.group(false, remeasured_value));
    for _ in 0..depth {
        item = record(&mut writer, |writer| {
            writer.fill(|writer| writer.fill_entry(|writer| writer.write_fragment(item)));
        });
    }
    writer.group(false, |writer| {
        writer.text("head");
        writer.write_alternatives(&[line], line);
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.write_fragment(item));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("next"));
        });
    });
    writer.finish().print(PrintOptions {
        line_width: 12,
        ..PrintOptions::default()
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
    writer.finish().print(PrintOptions {
        line_width: 15,
        ..PrintOptions::default()
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
        remeasured_fill_scope(GroupRequest::Expanded),
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
    writer.finish().print(PrintOptions {
        line_width: 10,
        ..PrintOptions::default()
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
    writer.finish().print(PrintOptions {
        line_width: 18,
        ..PrintOptions::default()
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
    writer.finish().print(PrintOptions {
        line_width: 10,
        ..PrintOptions::default()
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

#[test]
fn group_request_codec() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.break_parent();
        writer.line();
    });
    let doc = writer.finish();
    let bytes = Doc { ..doc }.encode();
    assert_eq!(&bytes[20..], &[3, 0, 0, 0, 0, 0, 16, 2, 0, 17, 3]);
    assert_eq!(doc.print(PrintOptions::default()), "\n");

    let mut writer = Writer::new(&allocator);
    writer.group(true, Writer::line);
    let doc = writer.finish();
    let bytes = Doc { ..doc }.encode();
    assert_eq!(&bytes[20..], &[3, 1, 0, 0, 0, 0, 2, 0, 17, 3]);
    assert_eq!(doc.print(PrintOptions::default()), "\n");

    let mut writer = Writer::new(&allocator);
    let child = record(&mut writer, |writer| {
        writer.break_parent();
        writer.line();
    });
    let grouped = writer.group_fragment(false, child);
    writer.write_fragment(grouped);
    let doc = writer.finish();
    let bytes = Doc { ..doc }.encode();
    // The root references the grouped fragment; its opening scope begins in fragment 1.
    let root_len = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
    let group_start = 20 + root_len + 4;
    assert_eq!(&bytes[group_start..group_start + 6], &[3, 0, 0, 0, 0, 0]);
    assert_eq!(doc.print(PrintOptions::default()), "\n");
}
