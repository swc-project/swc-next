use swc_next_allocator::Allocator;
use swc_next_formatter_core::PrintOptions;

use super::{ConditionTarget, Fragment, GroupRequest, Writer, WriterExt};

fn record<'a>(writer: &mut Writer<'a>, write: impl FnOnce(&mut Writer<'a>)) -> Fragment<'a> {
    writer
        .record_with_value(|writer| {
            write(writer);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap()
        .0
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

fn write_comment_suffix<'a>(writer: &mut Writer<'a>, text: &'a str) {
    writer.space();
    writer.text(text);
    writer.break_parent();
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

fn write_pliteral_space_pair(writer: &mut Writer<'_>) {
    writer.group_with_request(GroupRequest::Expanded, |writer| {
        writer.text("a");
        writer.line();
        writer.text("bbb");
    });
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
    let conditional = record(&mut writer, |writer| {
        writer.text("value");
        writer.if_break(",");
        writer.if_flat(".");
    });
    let current = record(&mut writer, write_current_mode_lines);
    let expanded = record(&mut writer, |writer| {
        writer.group(true, |writer| writer.text("value"))
    });
    let mandatory = record(&mut writer, |writer| {
        writer.text("first");
        writer.hard_line();
        writer.text("last");
    });
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

    let multiline = writer.flat_delimited_fragment(
        "(",
        atom.try_flat().unwrap(),
        mandatory,
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

    let enclosed = writer.conditional_group_fragment(atom, GroupRequest::Expanded, "(", atom, ")");
    assert!(enclosed.try_neutral().is_none());
    assert!(enclosed.try_flat().is_some());
    assert!(enclosed.preferred_breaks());
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
    writer.write_alternatives(&[grouped, expanded], fallback);
    writer.finish().print(PrintOptions {
        line_width: width,
        ..PrintOptions::default()
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
    assert!(prefix.try_flat().is_none());
    let fallback = writer.indented_delimited_fragment("(", prefix, body, suffix, ")");
    assert!(fallback.preferred_breaks());
    assert!(fallback.try_flat().is_none());
    writer.flat(|writer| writer.write_fragment(fallback));
    let doc = writer.finish();
    assert_eq!(doc.print(PrintOptions::default()), "(source\n  valuetail)");

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
    assert_eq!(
        doc.print(PrintOptions::default()),
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
    assert!(enclosed.try_flat().is_none());
    writer.flat(|writer| writer.write_fragment(enclosed));
    writer.line_suffix(|writer| write_comment_suffix(writer, "// outside"));
    writer.hard_line();
    let doc = writer.finish();
    assert_eq!(
        doc.print(PrintOptions::default()),
        "<T>\n(\n  first\n  last\n  tail\n) // outside\n"
    );
}

#[test]
fn conditional_retained_views() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preserved = record(&mut writer, write_pliteral_space_pair);
    let body = record(&mut writer, |writer| writer.text("body"));
    let flat = preserved.try_flat().unwrap();
    let scope = writer.flat_delimited_fragment("(", flat, body, flat, ")");
    writer.group(false, |writer| {
        writer.if_flat("F");
        writer.write_fragment(scope);
    });
    assert_eq!(
        writer.finish().print(PrintOptions::default()),
        "F(a bbbbodya bbb)"
    );

    let mut writer = Writer::new(&allocator);
    let normal = writer.indented_delimited_fragment("(", body, preserved, body, ")");
    writer.group(false, |writer| {
        writer.if_break("B");
        writer.write_fragment(normal);
    });
    assert_eq!(
        writer.finish().print(PrintOptions::default()),
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
        writer.finish().print(PrintOptions::default()),
        "first!\nsecond"
    );
}
