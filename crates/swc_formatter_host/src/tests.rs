use crate::{DocumentProducer, FormatContext, FormatSession};
use std::convert::Infallible;
use swc_next_formatter_core::{Doc, LineEnding, PrintOptions, Writer};

struct TextProducer;

impl DocumentProducer for TextProducer {
    type Options = ();
    type Error = Infallible;

    fn produce<'a>(
        &self,
        context: FormatContext<'a>,
        source: &'a str,
        (): &(),
    ) -> Result<Doc<'a>, Self::Error> {
        let mut writer = Writer::new(context.allocator());
        writer.literal_text(source);
        Ok(writer.finish())
    }
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
