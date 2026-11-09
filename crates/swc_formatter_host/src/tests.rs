use std::{borrow::Cow, cell::RefCell, convert::Infallible, error::Error, io};

use swc_next_formatter_core::{Doc, LineEnding, PrintOptions, Writer};

use crate::{
    DocumentProducer, EmbeddedDispatcher, EmbeddedError, EmbeddedErrorKind, EmbeddedOptions,
    EmbeddedOutcome, EmbeddedRequest, FormatContext, FormatSession, InputKind, PreserveReason,
};

struct TextProducer;

impl DocumentProducer for TextProducer {
    type Options = ();
    type Error = Infallible;

    fn produce<'a>(
        &self,
        context: FormatContext<'a, '_>,
        source: &'a str,
        (): &(),
    ) -> Result<Doc<'a>, Self::Error> {
        let mut writer = Writer::new(context.allocator());
        writer.literal_text(source);
        Ok(writer.finish())
    }
}

struct ParentProducer;

struct ParentOptions {
    language: &'static str,
    input_kind: InputKind,
}

const TEXT: ParentOptions = ParentOptions {
    language: "text",
    input_kind: InputKind::Fragment,
};

impl DocumentProducer for ParentProducer {
    type Options = ParentOptions;
    type Error = EmbeddedError;

    fn produce<'a>(
        &self,
        context: FormatContext<'a, '_>,
        source: &'a str,
        options: &ParentOptions,
    ) -> Result<Doc<'a>, Self::Error> {
        let outcome = context.embed(options.language, source, options.input_kind)?;
        let mut writer = Writer::new(context.allocator());
        writer.text("[");
        match outcome {
            EmbeddedOutcome::Formatted(document) => writer.write_doc(&document),
            EmbeddedOutcome::PreserveOriginal(_) => writer.literal_text(source),
        }
        writer.text("]");
        Ok(writer.finish())
    }
}

#[derive(Debug, PartialEq)]
struct Call {
    language: String,
    depth: u8,
    input_kind: InputKind,
    layout: PrintOptions,
}

#[derive(Default)]
struct Dispatcher {
    calls: RefCell<Vec<Call>>,
}

impl EmbeddedDispatcher for Dispatcher {
    fn dispatch<'a>(
        &self,
        context: FormatContext<'a, '_>,
        request: EmbeddedRequest<'a, '_>,
    ) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
        self.calls.borrow_mut().push(Call {
            language: request.language.to_owned(),
            depth: context.depth(),
            input_kind: context.input_kind(),
            layout: context.print_options(),
        });
        match request.language {
            "text" => {
                let document = context.produce(&TextProducer, request.source, &()).unwrap();
                Ok(EmbeddedOutcome::Formatted(document))
            }
            "empty" => {
                let document = context.produce(&TextProducer, "", &()).unwrap();
                Ok(EmbeddedOutcome::Formatted(document))
            }
            "nested" => {
                let document = context.produce(&ParentProducer, request.source, &TEXT)?;
                Ok(EmbeddedOutcome::Formatted(document))
            }
            "recursive" => context.embed("recursive", request.source, InputKind::Document),
            "nested-error" => context.embed("error", request.source, InputKind::Fragment),
            "error" => Err(EmbeddedError::new(io::Error::from(
                io::ErrorKind::BrokenPipe,
            ))),
            "boxed-error" => {
                let result: Result<Doc<'a>, Box<dyn Error + Send + Sync>> =
                    Err(Box::new(io::Error::from(io::ErrorKind::PermissionDenied)));
                Ok(EmbeddedOutcome::Formatted(result?))
            }
            _ => Ok(EmbeddedOutcome::PreserveOriginal(
                PreserveReason::Unsupported,
            )),
        }
    }
}

fn parent<'a>(
    context: FormatContext<'a, '_>,
    source: &'a str,
    options: &ParentOptions,
) -> Result<Doc<'a>, EmbeddedError> {
    context.produce(&ParentProducer, source, options)
}

fn embed<'a>(
    context: FormatContext<'a, '_>,
    language: &str,
    source: impl Into<Cow<'a, str>>,
    input_kind: InputKind,
) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
    context.embed(language, source, input_kind)
}

fn formatted(outcome: EmbeddedOutcome<'_>) -> Doc<'_> {
    match outcome {
        EmbeddedOutcome::Formatted(document) => document,
        EmbeddedOutcome::PreserveOriginal(_) => panic!("expected a formatted document"),
    }
}

fn io_source(error: &EmbeddedError) -> &io::Error {
    error.source().unwrap().downcast_ref::<io::Error>().unwrap()
}

#[test]
fn session_documents() {
    let options = PrintOptions {
        line_ending: LineEnding::CrLf,
        ..PrintOptions::default()
    };
    let session = FormatSession::new(options);
    let source = String::from("first\nλ");
    let produced = session.produce(&TextProducer, &source, &()).unwrap();
    let buffer = produced.encode();
    let decoded = session.decode(&buffer).unwrap();
    let tail = session.produce(&TextProducer, "last", &()).unwrap();
    let mut writer = Writer::new(session.allocator());
    writer.write_doc(&decoded);
    writer.hard_line();
    writer.write_doc(&tail);
    assert_eq!(
        writer.finish().print(session.print_options()),
        "first\r\nλ\r\nlast"
    );

    assert!(session.decode(b"invalid").is_err());
    let next = session.produce(&TextProducer, "next", &()).unwrap();
    assert_eq!(next.print(session.print_options()), "next");
}

#[test]
fn embedded_dispatch() {
    let session = FormatSession::default();
    let dispatcher = Dispatcher::default();
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let layout = context.print_options();
    assert_eq!(context.depth(), 0);
    assert_eq!(context.input_kind(), InputKind::Document);

    let document = parent(context, "λ", &TEXT).unwrap();
    assert_eq!(document.print(layout), "[λ]");
    assert_eq!(
        *dispatcher.calls.borrow(),
        [Call {
            language: "text".into(),
            depth: 1,
            input_kind: InputKind::Fragment,
            layout,
        }]
    );

    let document = parent(context, "\u{feff}λ", &TEXT).unwrap();
    assert_eq!(document.print(layout), "[\u{feff}λ]");

    dispatcher.calls.borrow_mut().clear();
    let nested = ParentOptions {
        language: "nested",
        input_kind: InputKind::Document,
    };
    let document = parent(context, "λ", &nested).unwrap();
    assert_eq!(document.print(layout), "[[λ]]");
    assert_eq!(
        *dispatcher.calls.borrow(),
        [
            Call {
                language: "nested".into(),
                depth: 1,
                input_kind: InputKind::Document,
                layout
            },
            Call {
                language: "text".into(),
                depth: 2,
                input_kind: InputKind::Fragment,
                layout
            },
        ]
    );
}

#[test]
fn embedded_fallbacks() {
    let session = FormatSession::default();
    let missing = embed(session.context(), "text", "raw", InputKind::Fragment).unwrap();
    assert!(matches!(
        missing,
        EmbeddedOutcome::PreserveOriginal(PreserveReason::NoDispatcher)
    ));

    let original = session.produce(&ParentProducer, "  raw  ", &TEXT).unwrap();
    assert_eq!(original.print(session.print_options()), "[  raw  ]");

    let dispatcher = Dispatcher::default();
    let disabled = EmbeddedOptions {
        enabled: false,
        max_depth: 0,
    };
    let context = session.context_with_embedded(&dispatcher, disabled);
    let result = embed(context, "text", String::from("raw"), InputKind::Fragment).unwrap();
    assert!(matches!(
        result,
        EmbeddedOutcome::PreserveOriginal(PreserveReason::Disabled)
    ));
    assert!(dispatcher.calls.borrow().is_empty());

    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let result = embed(context, "unknown", "raw", InputKind::Document).unwrap();
    assert!(matches!(
        result,
        EmbeddedOutcome::PreserveOriginal(PreserveReason::Unsupported)
    ));

    let unsupported = ParentOptions {
        language: "unknown",
        input_kind: InputKind::Fragment,
    };
    let original = parent(context, "\u{feff} raw  ", &unsupported).unwrap();
    assert_eq!(original.print(context.print_options()), "[\u{feff} raw  ]");

    let empty = ParentOptions {
        language: "empty",
        input_kind: InputKind::Fragment,
    };
    let document = parent(context, "raw", &empty).unwrap();
    assert_eq!(document.print(context.print_options()), "[]");
}

#[test]
fn embedded_errors() {
    let session = FormatSession::default();
    let dispatcher = Dispatcher::default();
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let failure = ParentOptions {
        language: "nested-error",
        input_kind: InputKind::Fragment,
    };
    let error = parent(context, "raw", &failure).err().unwrap();
    assert_eq!(error.kind(), EmbeddedErrorKind::Provider);
    assert_eq!(io_source(&error).kind(), io::ErrorKind::BrokenPipe);

    let boxed_failure = ParentOptions {
        language: "boxed-error",
        input_kind: InputKind::Fragment,
    };
    let error = parent(context, "raw", &boxed_failure).err().unwrap();
    assert_eq!(error.kind(), EmbeddedErrorKind::Provider);
    assert_eq!(io_source(&error).kind(), io::ErrorKind::PermissionDenied);

    dispatcher.calls.borrow_mut().clear();
    let limited = EmbeddedOptions {
        max_depth: 2,
        ..EmbeddedOptions::default()
    };
    let context = session.context_with_embedded(&dispatcher, limited);
    let error = embed(context, "recursive", "raw", InputKind::Document).unwrap_err();
    assert_eq!(error.kind(), EmbeddedErrorKind::DepthLimit { max_depth: 2 });
    assert!(error.source().is_none());
    assert_eq!(
        dispatcher
            .calls
            .borrow()
            .iter()
            .map(|call| call.depth)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let document = parent(context, "next", &TEXT).unwrap();
    assert_eq!(document.print(context.print_options()), "[next]");
    assert_eq!(dispatcher.calls.borrow().last().unwrap().depth, 1);

    dispatcher.calls.borrow_mut().clear();
    let disabled_budget = EmbeddedOptions {
        max_depth: 0,
        ..EmbeddedOptions::default()
    };
    let context = session.context_with_embedded(&dispatcher, disabled_budget);
    let error = embed(context, "text", "raw", InputKind::Fragment).unwrap_err();
    assert_eq!(error.kind(), EmbeddedErrorKind::DepthLimit { max_depth: 0 });
    assert!(dispatcher.calls.borrow().is_empty());
}

#[test]
fn provider_errors() {
    let source = io::Error::from(io::ErrorKind::BrokenPipe);
    let error = EmbeddedError::new(source);
    assert_eq!(error.kind(), EmbeddedErrorKind::Provider);
    assert_eq!(io_source(&error).kind(), io::ErrorKind::BrokenPipe);

    let source = Box::new(io::Error::from(io::ErrorKind::PermissionDenied));
    let source: Box<dyn Error + Send + Sync> = source;
    let error = EmbeddedError::from(source);
    assert_eq!(error.kind(), EmbeddedErrorKind::Provider);
    assert_eq!(io_source(&error).kind(), io::ErrorKind::PermissionDenied);
}

#[test]
fn debug_output() {
    let session = FormatSession::default();
    let dispatcher = Dispatcher::default();
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let source = "private_source_marker";
    let request = EmbeddedRequest {
        language: "text",
        source,
    };

    let _output = format!("{context:?}");
    assert!(dispatcher.calls.borrow().is_empty());

    let output = format!("{request:?}");
    assert!(!output.contains(source));

    let document = context.produce(&TextProducer, source, &()).unwrap();
    let outcome = EmbeddedOutcome::Formatted(document);
    let output = format!("{outcome:?}");
    assert!(!output.contains(source));
    assert_eq!(formatted(outcome).print(context.print_options()), source);
}

#[test]
fn embedded_storage() {
    let session = FormatSession::default();
    let document = {
        let dispatcher = Dispatcher::default();
        let language = String::from("text");
        let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
        let source = String::from("temporary λ");
        let result = embed(context, &language, source, InputKind::Fragment).unwrap();
        formatted(result)
    };
    assert_eq!(document.print(session.print_options()), "temporary λ");
}

#[test]
fn embedded_layout() {
    let session = FormatSession::default();
    let dispatcher = Dispatcher::default();
    let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
    let layout = PrintOptions {
        line_width: 20,
        line_ending: LineEnding::CrLf,
        ..context.print_options()
    };
    let nested = ParentOptions {
        language: "nested",
        input_kind: InputKind::Document,
    };
    let child = context.with_print_options(layout);
    let document = parent(child, "a\nb", &nested).unwrap();
    assert_eq!(document.print(layout), "[[a\r\nb]]");
    assert_eq!(dispatcher.calls.borrow()[0].layout, layout);
    assert_eq!(dispatcher.calls.borrow()[1].layout, layout);

    let document = parent(context, "a\nb", &TEXT).unwrap();
    assert_eq!(document.print(context.print_options()), "[a\nb]");
    assert_eq!(
        dispatcher.calls.borrow().last().unwrap().layout,
        session.print_options()
    );
}
