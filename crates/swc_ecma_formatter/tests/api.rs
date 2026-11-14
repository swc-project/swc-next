use std::borrow::Cow;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_formatter::{
    EcmaDocumentProducer, FormatError, FormatOptions, IndentWidth, QuoteStyle, Semicolons,
    SyntaxOptions, format, format_ast,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_formatter_core::{LineEnding, PrintOptions, Writer};
use swc_next_formatter_host::{
    EmbeddedDispatcher, EmbeddedError, EmbeddedOptions, EmbeddedOutcome, EmbeddedRequest,
    FormatContext, FormatSession, InputKind, PreserveReason,
};

#[test]
fn indentation_width_bounds() {
    assert_eq!(IndentWidth::try_from(0).unwrap().value(), 0);
    assert_eq!(IndentWidth::try_from(64).unwrap().value(), 64);
    assert!(IndentWidth::try_from(65).is_err());
    assert!(IndentWidth::try_from(255).is_err());
}

#[test]
fn parser_options() {
    let options = FormatOptions::prettier();
    let invalid = format("const = ;", ParserOptions::default(), options);
    assert!(matches!(invalid, Err(FormatError::Parse(_))));

    let module = ParserOptions {
        source_type: SourceType::Module,
        ..ParserOptions::default()
    };
    let reserved = format("await;", module, options);
    assert!(matches!(reserved, Err(FormatError::Parse(_))));

    let script = ParserOptions {
        source_type: SourceType::Script,
        ..ParserOptions::default()
    };
    let identifier = format("await;", script, options);
    assert!(matches!(
        identifier,
        Ok(_) | Err(FormatError::Unsupported { .. })
    ));
}

#[test]
fn host_documents() {
    let layout = PrintOptions {
        indent_width: IndentWidth::try_from(4).unwrap(),
        line_ending: LineEnding::CrLf,
        ..PrintOptions::default()
    };
    let session = FormatSession::new(layout);
    let source = String::from("function f():string{return 'λ'}");
    let document = {
        let parser_options = ParserOptions {
            lang: Lang::Ts,
            ..ParserOptions::default()
        };
        let producer = EcmaDocumentProducer::new(parser_options);
        let syntax = SyntaxOptions {
            semicolons: Semicolons::AsNeeded,
            quote_style: QuoteStyle::Single,
            ..SyntaxOptions::default()
        };
        session.produce(&producer, &source, &syntax).unwrap()
    };
    assert_eq!(
        document.print(layout),
        "function f(): string {\r\n    return 'λ'\r\n}\r\n"
    );

    let producer = EcmaDocumentProducer::default();
    let syntax = SyntaxOptions::default();
    let invalid = session.produce(&producer, "const = ;", &syntax);
    assert!(matches!(invalid, Err(FormatError::Parse(_))));

    let next = session
        .produce(&producer, "const value=1", &syntax)
        .unwrap();
    assert_eq!(next.print(layout), "const value = 1;\r\n");
}

struct EcmaDispatcher {
    producer: EcmaDocumentProducer,
    syntax: SyntaxOptions,
}

impl EmbeddedDispatcher for EcmaDispatcher {
    fn dispatch<'a>(
        &self,
        context: FormatContext<'a, '_>,
        request: EmbeddedRequest<'a, '_>,
    ) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
        if request.language != "typescript" || context.input_kind() != InputKind::Document {
            return Ok(EmbeddedOutcome::PreserveOriginal(
                PreserveReason::Unsupported,
            ));
        }
        match context.produce(&self.producer, request.source, &self.syntax) {
            Ok(document) => Ok(EmbeddedOutcome::Formatted(document)),
            Err(FormatError::Parse(_) | FormatError::Unsupported { .. }) => Ok(
                EmbeddedOutcome::PreserveOriginal(PreserveReason::Unsupported),
            ),
        }
    }
}

fn embed_ecma<'a>(
    context: FormatContext<'a, '_>,
    source: impl Into<Cow<'a, str>>,
    input_kind: InputKind,
) -> EmbeddedOutcome<'a> {
    context.embed("typescript", source, input_kind).unwrap()
}

#[test]
fn host_embedding() {
    let layout = PrintOptions {
        line_ending: LineEnding::CrLf,
        ..PrintOptions::default()
    };
    let session = FormatSession::new(layout);
    let dispatcher = EcmaDispatcher {
        producer: EcmaDocumentProducer::new(ParserOptions {
            lang: Lang::Ts,
            ..ParserOptions::default()
        }),
        syntax: SyntaxOptions {
            semicolons: Semicolons::AsNeeded,
            ..SyntaxOptions::default()
        },
    };
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let source = String::from("const value:number=1");
    let result = embed_ecma(context, source, InputKind::Document);
    let EmbeddedOutcome::Formatted(document) = result else {
        panic!("expected a formatted document")
    };
    let mut writer = Writer::new(context.allocator());
    writer.text("```ts");
    writer.hard_line();
    writer.write_doc(&document);
    writer.text("```");
    assert_eq!(
        writer.finish().print(layout),
        "```ts\r\nconst value: number = 1\r\n```"
    );

    let invalid = embed_ecma(context, "const = ;", InputKind::Document);
    assert!(matches!(
        invalid,
        EmbeddedOutcome::PreserveOriginal(PreserveReason::Unsupported)
    ));

    let fragment = embed_ecma(context, "value: number", InputKind::Fragment);
    assert!(matches!(
        fragment,
        EmbeddedOutcome::PreserveOriginal(PreserveReason::Unsupported)
    ));
}

fn assert_stable(input: &str, options: FormatOptions) {
    let first = format(input, ParserOptions::default(), options).unwrap();
    let second = format(&first, ParserOptions::default(), options).unwrap();
    assert_eq!(first, second);
}

#[test]
fn comment_layout_stability() {
    let options = FormatOptions {
        line_width: 20,
        semicolons: Semicolons::AsNeeded,
        ..FormatOptions::prettier()
    };

    assert_stable(
        "(/** @type {number} */ first || second || third) + fourth;",
        options,
    );

    assert_stable(
        "fourth + (/** @type {number} */ first || second || third);",
        options,
    );

    assert_stable(
        "(/** @satisfies {number} */ [first, second, third]) + fourth;",
        options,
    );

    assert_stable(
        "result = (/** @type {number} */ left ? middle : right)(arg);",
        options,
    );

    assert_stable("call() /* #__PURE__ */;\nnext();", options);

    assert_stable(
        "call() /* #__PURE__ */;\nnext();",
        FormatOptions::prettier(),
    );

    assert_stable("call() /** @type {number} */;\n(value);", options);

    assert_stable(
        "call() /** @type {number} */;\n(value);",
        FormatOptions::prettier(),
    );
}

#[test]
fn ambient_module_attribute_recovery() {
    let allocator = Allocator::new();
    let parser_options = ParserOptions {
        lang: Lang::Dts,
        source_type: SourceType::Unambiguous,
        ..ParserOptions::default()
    };
    let source = "declare module '*.data' /* before */ with /* after */ { readonly type:'json'|'json5'; 'resolution-mode':'import' };\n";
    let parsed = Parser::init(&allocator, source, parser_options, NoTokenParserConfig).parse();
    assert_eq!(parsed.diagnostics.len(), 1);
    let result = format(source, parser_options, FormatOptions::prettier());
    assert!(matches!(result, Err(FormatError::Parse(errors)) if errors.len() == 1));

    for (line_width, semicolons, expected) in [
        (
            20,
            Semicolons::Always,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type:\n    | \"json\"\n    | \"json5\";\n  \"resolution-mode\": \"import\";\n};\n",
        ),
        (
            80,
            Semicolons::Always,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type: \"json\" | \"json5\";\n  \"resolution-mode\": \"import\";\n};\n",
        ),
        (
            20,
            Semicolons::AsNeeded,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type:\n    \"json\" | \"json5\"\n  \"resolution-mode\": \"import\"\n}\n",
        ),
        (
            80,
            Semicolons::AsNeeded,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type: \"json\" | \"json5\"\n  \"resolution-mode\": \"import\"\n}\n",
        ),
    ] {
        let options = FormatOptions {
            line_width,
            semicolons,
            ..FormatOptions::prettier()
        };
        let output = format_ast(&parsed.ast, options).unwrap();
        assert_eq!(output, expected);
        let reparsed =
            Parser::init(&allocator, &output, parser_options, NoTokenParserConfig).parse();
        assert_eq!(reparsed.diagnostics.len(), 1);
        assert_eq!(reparsed.ast.comments().len(), parsed.ast.comments().len());
        assert_eq!(format_ast(&reparsed.ast, options).unwrap(), output);
    }
}
