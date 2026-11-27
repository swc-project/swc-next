use std::cell::Cell;

use swc_next_formatter_core::{LineEnding, PrintOptions, Writer};
use swc_next_formatter_host::{
    EmbeddedDispatcher, EmbeddedError, EmbeddedErrorKind, EmbeddedOptions, EmbeddedOutcome,
    EmbeddedRequest, FormatContext, FormatSession, InputKind, PreserveReason,
};
use swc_next_markdown_parser::{
    BlockKind, Dialect, Document, InlineKind, ParseOptions, parse, parse_with_options,
};

use crate::{MarkdownDocumentProducer, ProseWrap, SyntaxOptions, ast_to_doc, plugin_document};

fn format(source: &str, layout: PrintOptions, syntax: SyntaxOptions) -> String {
    let session = FormatSession::new(layout);
    let produced = session
        .produce(&MarkdownDocumentProducer, source, &syntax)
        .unwrap();
    let ast = ast_to_doc(&parse(source), session.context(), syntax).unwrap();
    let result = produced.print(layout);
    assert_eq!(ast.print(layout), result);
    let repeated = session
        .produce(&MarkdownDocumentProducer, &result, &syntax)
        .unwrap();
    assert_eq!(repeated.print(layout), result);
    result
}

#[test]
fn blocks() {
    let layout = PrintOptions::default();
    let syntax = SyntaxOptions::default();

    let source = "[^n]:\n        code\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "#  heading  #\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, "# heading\n");

    let source = "#  heading ##   ###\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, "# heading ## ###\n");

    let source = "> ## # #\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "# C# #\n";
    assert_eq!(format(source, layout, syntax), "# C#\n");

    let source = "# \\# #\n";
    assert_eq!(format(source, layout, syntax), "# \\#\n");

    let source = "> * first\n>   + nested\n>\n> paragraph\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, "> - first\n>   - nested\n>\n> paragraph\n");

    let source = "2. [X]  done\n3. next\n\n4) different\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, "2. [x] done\n3. next\n\n4) different\n");

    let source = "-    item\n\n    separate code block\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, source);

    let source = "1.   first\n2.   second\n\n    separate code block\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, source);

    let source = "   -     inner code block\n\n    separate code block\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, source);

    let source = "   -\n\n    separate code block\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, source);

    let source = "$$$$ formula\n$$\nx\n$$$$\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, "$$$ formula\n$$\nx\n$$$\n");

    let source = "$$\n   \n$$\n";
    let result = format(source, layout, syntax);
    assert_eq!(result, "$$\n$$\n");
}

#[test]
fn prose() {
    fn assert_bracketed_reference(source: &str) {
        use swc_next_markdown_parser::{LinkTarget, ReferenceKind};

        let document = parse(source);
        let inlines = match &document.blocks[0].kind {
            BlockKind::Heading(heading) => heading.children.as_slice(),
            BlockKind::Paragraph(inlines) => inlines.as_slice(),
            _ => panic!("expected a heading or paragraph"),
        };
        let [open, link, close] = inlines else {
            panic!("expected a link surrounded by literal brackets");
        };
        assert!(matches!(open.kind, InlineKind::Text));
        assert!(matches!(close.kind, InlineKind::Text));
        assert_eq!(document.text(open.span), "[");
        assert_eq!(document.text(close.span), "]");
        let InlineKind::Link(link) = &link.kind else {
            panic!("expected a link");
        };
        let LinkTarget::Reference {
            kind: ReferenceKind::Shortcut,
            identifier,
            ..
        } = &link.target
        else {
            panic!("expected a shortcut reference");
        };
        let definition = document.definition(identifier).unwrap();
        assert_eq!(document.text(definition.definition.destination), "target");
    }

    let layout = PrintOptions {
        line_width: 12,
        ..PrintOptions::default()
    };
    let preserve = SyntaxOptions::default();
    let always = SyntaxOptions {
        prose_wrap: ProseWrap::Always,
        ..preserve
    };
    let never = SyntaxOptions {
        prose_wrap: ProseWrap::Never,
        ..preserve
    };

    let source = "alpha    beta\ngamma delta\n";
    assert_eq!(
        format(source, layout, preserve),
        "alpha beta\ngamma delta\n"
    );

    let source = "今天天气很好。   你好。\n另一句。\n";
    assert_eq!(
        format(source, layout, never),
        "今天天气很好。 你好。\n另一句。\n"
    );

    let source = "alpha beta gamma delta\n";
    assert_eq!(format(source, layout, always), "alpha beta\ngamma delta\n");

    let source = "alpha beta > text\n";
    assert_eq!(format(source, layout, always), "alpha beta >\ntext\n");

    let source = "alpha  \nbeta\\\ngamma\n";
    assert_eq!(format(source, layout, never), source);

    let source = "alpha\\ \nbeta\n";
    assert_eq!(format(source, layout, preserve), source);

    let source = "foo\n    # bar\n";
    for syntax in [preserve, always, never] {
        assert_eq!(format(source, layout, syntax), "foo # bar\n");
    }

    let source = "> foo  \n>     1. item\n";
    for syntax in [preserve, always, never] {
        assert_eq!(format(source, layout, syntax), source);
    }

    let source = "foo\\\n    | --- |\n";
    assert_eq!(format(source, layout, preserve), source);

    let wide = PrintOptions::default();
    for syntax in [always, never] {
        let source = "[[one\ntwo]]\n==\n";
        assert_eq!(format(source, wide, syntax), source);

        let source = "[[one\ntwo]]\n";
        assert_eq!(format(source, wide, syntax), source);

        let source = "[[one two]]\n==\n";
        assert_eq!(format(source, wide, syntax), source);

        let source = "one\ntwo\n==\n";
        assert_eq!(format(source, wide, syntax), "one two\n==\n");
    }

    for syntax in [preserve, always, never] {
        let source = "[[hello\nworld]]\n==\n\n[hello world]: target\n";
        let result = format(source, wide, syntax);
        assert_bracketed_reference(&result);
        assert_eq!(result, source);

        let source = "[[hello\nworld]]\n\n[hello world]: target\n";
        let result = format(source, wide, syntax);
        assert_bracketed_reference(&result);
        assert_eq!(result, source);

        let source = "> - [[hello\n>   world]]\n\n[hello world]: target\n";
        assert_eq!(format(source, wide, syntax), source);

        let source = "[[hello\nworld\\]]\n\n[hello world\\]]: target\n";
        assert_eq!(format(source, wide, syntax), source);

        let source = "[[hello\nworld\\]\n]]\n\n[hello world\\]]: target\n";
        assert_eq!(
            format(source, wide, syntax),
            "[[hello world\\] ]]\n\n[hello world\\]]: target\n"
        );

        let source = "\\[[hello\nworld]]\n\n[hello world]: target\n";
        assert_eq!(
            format(source, wide, syntax),
            "\\[[hello world]]\n\n[hello world]: target\n"
        );

        let source = "[ [hello\nworld]]\n\n[hello world]: target\n";
        assert_eq!(
            format(source, wide, syntax),
            "[ [hello world]]\n\n[hello world]: target\n"
        );

        let source = "\\ [[hello\nworld]]\n\n[hello world]: target\n";
        assert_eq!(format(source, wide, syntax), source);
    }
}

#[test]
fn prose_wrap_boundaries() {
    fn hard_breaks(source: &str) -> usize {
        let document = parse(source);
        let [block] = document.blocks.as_slice() else {
            panic!("expected one block");
        };
        let BlockKind::Paragraph(inlines) = &block.kind else {
            panic!("expected a paragraph");
        };
        inlines
            .iter()
            .filter(|node| matches!(node.kind, InlineKind::HardBreak))
            .count()
    }

    let layout = PrintOptions {
        line_width: 6,
        ..PrintOptions::default()
    };
    let syntax = SyntaxOptions {
        prose_wrap: ProseWrap::Always,
        ..SyntaxOptions::default()
    };
    let wrap = |source: &str| {
        let result = format(source, layout, syntax);
        assert_eq!(hard_breaks(&result), hard_breaks(source));
        result
    };

    let source = "one alpha\\ beta\n";
    assert_eq!(wrap(source), "one\nalpha\\ beta\n");

    let source = "alpha\\\\ beta\n";
    assert_eq!(wrap(source), "alpha\\\\\nbeta\n");

    let source = "hello [^n]: note\n";
    assert_eq!(wrap(source), "hello [^n]:\nnote\n");

    let source = "hello [^a\\]b]:note\n";
    assert_eq!(wrap(source), source);

    let source = "hello [^n] note\n";
    assert_eq!(wrap(source), "hello\n[^n]\nnote\n");

    let source = "hello $$ world\n";
    assert_eq!(wrap(source), "hello $$\nworld\n");

    let source = "$$$meta long $ tail\n";
    assert_eq!(wrap(source), "$$$meta long $\ntail\n");

    let source = "hi\\\n$$ long $\n";
    assert_eq!(wrap(source), source);

    let source = "hello $$x$$ world\n";
    assert_eq!(wrap(source), "hello\n$$x$$\nworld\n");

    let source = "hello $$x\nb$$ tail\n";
    assert_eq!(wrap(source), "hello $$x\nb$$\ntail\n");

    let source = "hello {% raw %}\n";
    assert_eq!(wrap(source), source);

    let source = "hello {{ value }} tail\n";
    assert_eq!(wrap(source), "hello {{ value }}\ntail\n");

    let source = "{{ value }} tail end\n";
    assert_eq!(wrap(source), "{{ value }} tail\nend\n");

    let source = "hello {% raw %}. tail\n";
    assert_eq!(wrap(source), "hello\n{% raw %}.\ntail\n");

    let source = "hello {{ multi\nline }}. tail\n";
    assert_eq!(wrap(source), "hello {{ multi\nline }}.\ntail\n");

    let source = "hello {{ a\n{{ b }} tail end\n";
    assert_eq!(wrap(source), "hello {{ a\n{{ b }} tail\nend\n");

    let source = "hello |---|\n";
    assert_eq!(wrap(source), source);

    let source = "x|y |:-- | --:|\n";
    assert_eq!(wrap(source), source);

    let source = "hello | ---|\n";
    assert_eq!(wrap(source), source);

    let source = "hello |text|\n";
    assert_eq!(wrap(source), "hello\n|text|\n");

    let source = "hello\\\n|---| next\n";
    assert_eq!(wrap(source), source);

    let source = "hello | h\\\n|---|\n";
    assert_eq!(wrap(source), "hello\n| h\\\n\\|---|\n");
}

#[test]
fn inline_links() {
    let layout = PrintOptions::default();
    let options = SyntaxOptions {
        single_quote: true,
        ..SyntaxOptions::default()
    };
    let source = "[*a*](<path> \"title\") and [a][ref].\n\n[ref]: target\n";
    assert_eq!(
        format(source, layout, options),
        "[_a_](path 'title') and [a][ref].\n\n[ref]: target\n"
    );

    let source = "[x](`ticks`) and ``a ` b``.\n";
    assert_eq!(format(source, layout, options), source);

    let source = "[x](foo&#10;bar)\n";
    assert_eq!(format(source, layout, options), source);

    let source = "[x](foo&amp;#10;bar)\n";
    assert_eq!(format(source, layout, options), source);

    let source = "[ref]: <a&#xD;b> 'a&NewLine;&NewLine;b'\n\n[ref]\n";
    assert_eq!(format(source, layout, options), source);

    let narrow = PrintOptions {
        line_width: 12,
        ..layout
    };
    let wrap = SyntaxOptions {
        prose_wrap: ProseWrap::Always,
        ..SyntaxOptions::default()
    };
    let source = "[top]: # \"long title\"\n\n[top]\n";
    let result = format(source, narrow, wrap);
    assert_eq!(result, "[top]: #\n  \"long title\"\n\n[top]\n");
    assert!(parse(&result).definition("top").is_some());

    let source = "[top]: #topic \"long title\"\n\n[top]\n";
    assert_eq!(
        format(source, narrow, wrap),
        "[top]:\n  #topic\n  \"long title\"\n\n[top]\n"
    );

    for prose_wrap in [ProseWrap::Preserve, ProseWrap::Always, ProseWrap::Never] {
        let syntax = SyntaxOptions {
            prose_wrap,
            ..options
        };

        let source = "[hello  \nworld][]\n\n[hello world]: target\n";
        assert_eq!(format(source, layout, syntax), source);

        let source = "[hello\\\nworld][]\n\n[hello\\ world]: target\n";
        assert_eq!(format(source, layout, syntax), source);

        let source = "[*hello  \nworld*]\n\n[*hello world*]: target\n";
        assert_eq!(format(source, layout, syntax), source);

        let source = "[hello\\\nworld]\n\n[hello\\ world]: target\n";
        assert_eq!(format(source, layout, syntax), source);

        let source = "[hello\nworld][]\n\n[hello world]: target\n";
        assert_eq!(
            format(source, layout, syntax),
            "[hello world][]\n\n[hello world]: target\n"
        );

        let source = "[hello  \nworld][ref]\n\n[ref]: target\n";
        assert_eq!(format(source, layout, syntax), source);
    }
}

#[test]
fn inline_markers() {
    let layout = PrintOptions::default();
    let syntax = SyntaxOptions::default();

    let source = "*a*_b_\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "**a**__b__\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "x*a*__*b*__\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "x*a**b*\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "*a__!b*\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "*a\\__!b*\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "__a***b__\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "*snake_case*\n";
    assert_eq!(format(source, layout, syntax), "_snake_case_\n");

    let source = "x*a**b**c*\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "__a**b**c__\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "A ` tick and ``code``.\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "A ` tick and [``code``](url).\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "![literal `](pic) and ``code``.\n";
    assert_eq!(format(source, layout, syntax), source);

    let source = "A \\` tick and ``code``.\n";
    assert_eq!(format(source, layout, syntax), "A \\` tick and `code`.\n");
}

#[test]
fn tables() {
    let layout = PrintOptions::default();
    let syntax = SyntaxOptions::default();
    let source = "|name|number|\n|:---|---:|\n|短|3|\n";
    let result = format(source, layout, syntax);
    assert_eq!(
        result,
        "| name | number |\n| :--- | -----: |\n| 短   |      3 |\n"
    );

    let source = "| [x](foo\\|bar) |\n| --- |\n";
    assert_eq!(
        format(source, layout, syntax),
        "| [x](foo\\|bar) |\n| ------------- |\n"
    );

    let source = "| [x](url \"a&#92;&#124;b\") |\n| --- |\n";
    assert_eq!(
        format(source, layout, syntax),
        "| [x](url \"a&#92;&#124;b\") |\n| ------------------------ |\n"
    );

    let source = "| [x](url \"a&#10;b\") |\n| --- |\n";
    assert_eq!(
        format(source, layout, syntax),
        "| [x](url \"a&#10;b\") |\n| ------------------ |\n"
    );
}

#[test]
fn verbatim() {
    let layout = PrintOptions {
        line_ending: LineEnding::CrLf,
        ..PrintOptions::default()
    };
    let source = "---\nkey:  value\n---\n\n<!-- prettier-ignore-start -->\n\n##   untouched     #  \n\n<!-- prettier-ignore-end -->\n\n##   normal #\n";
    let result = format(source, layout, SyntaxOptions::default());
    assert_eq!(
        result,
        "---\r\nkey:  value\r\n---\r\n\r\n<!-- prettier-ignore-start -->\r\n\r\n##   untouched     #  \r\n\r\n<!-- prettier-ignore-end -->\r\n\r\n## normal\r\n"
    );

    let source = "> <!-- prettier-ignore -->\n>   #### keep #  \n";
    let result = format(source, layout, SyntaxOptions::default());
    assert_eq!(
        result,
        "> <!-- prettier-ignore -->\r\n>   #### keep #  \r\n"
    );

    let source =
        "> <!-- prettier-ignore-start -->\n>\n> ##   nested #\n>\n> <!-- prettier-ignore-end -->\n";
    let result = format(source, layout, SyntaxOptions::default());
    assert_eq!(
        result,
        "> <!-- prettier-ignore-start -->\r\n>\r\n> ## nested\r\n>\r\n> <!-- prettier-ignore-end -->\r\n"
    );

    let source = "> ~~~unknown\n> a  \n> b\n> ~~~\n";
    let result = format(source, layout, SyntaxOptions::default());
    assert_eq!(result, "> ```unknown\r\n> a  \r\n> b\r\n> ```\r\n");

    let source = ">     a  \n>\n>     b\t \n";
    let result = format(source, layout, SyntaxOptions::default());
    assert_eq!(result, ">     a  \r\n>\r\n>     b\t \r\n");
}

#[test]
fn extended_dialect_and_recovery() {
    let session = FormatSession::default();
    let syntax = SyntaxOptions::default();
    let print = |document: &Document<'_>| {
        ast_to_doc(document, session.context(), syntax)
            .unwrap()
            .print(session.print_options())
    };
    let markdown = ParseOptions::default();
    let extended = ParseOptions {
        dialect: Dialect::Extended,
        ..markdown
    };

    let source = "$$\na   +   b\n$$\n";
    let parsed = parse_with_options(source, extended);
    assert!(matches!(parsed.blocks[0].kind, BlockKind::Extension(_)));
    assert_eq!(print(&parsed), source);

    for options in [markdown, extended] {
        let format = |source: &str| print(&parse_with_options(source, options));

        let source = "~~literal .~!struck?~. tail\n";
        assert_eq!(format(source), source);

        let source = "~~start .~!word!~. end~~\n";
        assert_eq!(format(source), source);
    }

    let source = "\u{feff}#  keep   \n\n";
    let parsed = parse_with_options(
        source,
        ParseOptions {
            max_input_bytes: Some(1),
            ..markdown
        },
    );
    assert_eq!(print(&parsed), "#  keep   \n\n");
}

struct TestDispatcher {
    calls: Cell<usize>,
}

impl EmbeddedDispatcher for TestDispatcher {
    fn dispatch<'a>(
        &self,
        context: FormatContext<'a, '_>,
        request: EmbeddedRequest<'a, '_>,
    ) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(context.input_kind(), InputKind::Document);
        let mut writer = Writer::new(context.allocator());
        match request.language {
            "empty" | "toml" => {}
            "yaml" => writer.text(request.source),
            "short" => writer.text("short"),
            "long" => writer.text("line\n````\nline\n"),
            "md" => {
                let doc = context.produce(
                    &MarkdownDocumentProducer,
                    request.source,
                    &SyntaxOptions::default(),
                )?;
                return Ok(EmbeddedOutcome::Formatted(doc));
            }
            "recurse" => return context.embed("recurse", request.source, InputKind::Document),
            "fail" => return Err(EmbeddedError::new(std::io::Error::other("provider error"))),
            _ => {
                return Ok(EmbeddedOutcome::PreserveOriginal(
                    PreserveReason::Unsupported,
                ));
            }
        }
        Ok(EmbeddedOutcome::Formatted(writer.finish()))
    }
}

#[test]
fn fence_dispatch() {
    let session = FormatSession::default();
    let dispatcher = TestDispatcher {
        calls: Cell::new(0),
    };
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let options = SyntaxOptions::default();
    let source = "> ~~~long\n> any source\n> ~~~\n\n~~~short\n```````\n~~~\n\n~~~empty\nbody to discard\n~~~\n";
    let result = ast_to_doc(&parse(source), context, options)
        .unwrap()
        .print(context.print_options());
    assert_eq!(
        result,
        "> `````long\n> line\n> ````\n> line\n> `````\n\n```short\nshort\n```\n\n```empty\n\n```\n"
    );
    assert_eq!(
        ast_to_doc(&parse(&result), context, options)
            .unwrap()
            .print(context.print_options()),
        result
    );

    let source = "~~~none `meta`\ncontent\n~~~\n";
    let result = ast_to_doc(&parse(source), context, options)
        .unwrap()
        .print(context.print_options());
    assert_eq!(result, source);

    let source = "```plain&#10;note\nbody\n```\n";
    assert_eq!(format(source, session.print_options(), options), source);

    let source = "```unknown note=&amp;copy;\nbody\n```\n";
    assert_eq!(format(source, session.print_options(), options), source);

    let source = "```short note=&#92;*\nbody\n```\n";
    let result = ast_to_doc(&parse(source), context, options)
        .unwrap()
        .print(context.print_options());
    assert_eq!(result, "```short note=&#92;*\nshort\n```\n");

    let source = "```sh&#111;rt flags&#13;note\nbody\n```\n";
    let result = ast_to_doc(&parse(source), context, options)
        .unwrap()
        .print(context.print_options());
    assert_eq!(result, "```sh&#111;rt flags&#13;note\nshort\n```\n");

    let source = "~~~md\n#  inner   #\n~~~\n";
    let result = ast_to_doc(&parse(source), context, options)
        .unwrap()
        .print(context.print_options());
    assert_eq!(result, "```md\n# inner\n```\n");

    let source = "[^n]:\n    > ~~~short\n    > input\n    > ~~~\n";
    let result = ast_to_doc(&parse(source), context, options)
        .unwrap()
        .print(context.print_options());
    assert_eq!(result, "[^n]:\n    > ```short\n    > short\n    > ```\n");
}

#[test]
fn front_matter_dispatch() {
    let session = FormatSession::default();
    let dispatcher = TestDispatcher {
        calls: Cell::new(0),
    };
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let source = "+++\nvalue\n+++\n";
    let doc = ast_to_doc(&parse(source), context, SyntaxOptions::default()).unwrap();
    assert_eq!(doc.print(context.print_options()), "+++\n+++\n");

    let source = "---\n  message: |+\n    value  \n\n---\n";
    let doc = ast_to_doc(&parse(source), context, SyntaxOptions::default()).unwrap();
    assert_eq!(doc.print(context.print_options()), source);
}

#[test]
fn fence_errors() {
    let session = FormatSession::default();
    let dispatcher = TestDispatcher {
        calls: Cell::new(0),
    };
    let context = session.context_with_embedded(
        &dispatcher,
        EmbeddedOptions {
            max_depth: 1,
            ..EmbeddedOptions::default()
        },
    );
    let options = SyntaxOptions::default();

    let source = "```recurse\ntext\n```\n";
    let output = ast_to_doc(&parse(source), context, options);
    assert!(
        matches!(output, Err(error) if error.kind() == EmbeddedErrorKind::DepthLimit { max_depth: 1 })
    );

    let source = "```fail\ntext\n```\n";
    let output = ast_to_doc(&parse(source), context, options);
    assert!(matches!(output, Err(error) if error.kind() == EmbeddedErrorKind::Provider));

    let disabled = session.context_with_embedded(
        &dispatcher,
        EmbeddedOptions {
            enabled: false,
            max_depth: 0,
        },
    );
    let output = ast_to_doc(&parse(source), disabled, options)
        .unwrap()
        .print(disabled.print_options());
    assert_eq!(output, source);
    assert_eq!(dispatcher.calls.get(), 2);
}

#[test]
fn fence_slots() {
    let session = FormatSession::default();
    let dispatcher = TestDispatcher {
        calls: Cell::new(0),
    };
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let source =
        "λ\n\n> ~~~short flags\n> α\n> ~~~\n\n<!-- prettier-ignore -->\n~~~short\nignored\n~~~\n";
    let (fallback, slots) =
        plugin_document(&parse(source), context, SyntaxOptions::default()).unwrap();
    assert_eq!(slots.len(), 1);
    let crate::EmbeddedBlock::Fence(slot) = &slots[0] else {
        panic!()
    };
    assert_eq!(&source[slot.start..slot.end], "~~~short flags\n> α\n> ~~~");
    assert_eq!(slot.language, "short");
    assert_eq!(slot.meta.as_deref(), Some("flags"));
    assert_eq!(slot.body, "α");
    assert_eq!(dispatcher.calls.get(), 0);
    let formatted = fallback.print(context.print_options());
    assert_eq!(
        formatted,
        "λ\n\n> ```short flags\n> α\n> ```\n\n<!-- prettier-ignore -->\n~~~short\nignored\n~~~\n"
    );
}
