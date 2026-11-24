use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::Options as ParserOptions;
use swc_next_formatter_host::{
    EmbeddedDispatcher, EmbeddedError, EmbeddedOutcome, EmbeddedRequest, FormatContext, InputKind,
    PreserveReason,
};
use swc_next_formatter_js::{EcmaDocumentProducer, SyntaxOptions as JsSyntaxOptions};
use swc_next_formatter_markdown::{
    MarkdownDocumentProducer, SyntaxOptions as MarkdownSyntaxOptions,
};

/// Routes embedded documents to the native formatters and supplies their syntax options.
#[derive(Clone, Copy, Debug, Default)]
pub struct BuiltinFormatters {
    /// Options used when the selected producer is JavaScript or TypeScript.
    pub js: JsSyntaxOptions,
    /// Options used when the selected producer is Markdown.
    pub markdown: MarkdownSyntaxOptions,
}

impl EmbeddedDispatcher for BuiltinFormatters {
    fn dispatch<'a>(
        &self,
        context: FormatContext<'a, '_>,
        request: EmbeddedRequest<'a, '_>,
    ) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
        let preserve = EmbeddedOutcome::PreserveOriginal(PreserveReason::Unsupported);
        if context.input_kind() != InputKind::Document {
            return Ok(preserve);
        }
        let (lang, source_type) = match request.language.to_ascii_lowercase().as_str() {
            "md" | "markdown" => {
                return context
                    .produce(&MarkdownDocumentProducer, request.source, &self.markdown)
                    .map(EmbeddedOutcome::Formatted);
            }
            "js" | "javascript" | "jsx" => (Lang::Jsx, SourceType::Unambiguous),
            "mjs" => (Lang::Jsx, SourceType::Module),
            "cjs" => (Lang::Jsx, SourceType::CommonJs),
            "ts" | "typescript" | "angular-ts" => (Lang::Ts, SourceType::Unambiguous),
            "mts" => (Lang::Ts, SourceType::Module),
            "cts" => (Lang::Ts, SourceType::CommonJs),
            "tsx" => (Lang::Tsx, SourceType::Unambiguous),
            _ => return Ok(preserve),
        };
        let producer = EcmaDocumentProducer::new(ParserOptions {
            lang,
            source_type,
            ..Default::default()
        });
        Ok(match context.produce(&producer, request.source, &self.js) {
            Ok(doc) => EmbeddedOutcome::Formatted(doc),
            Err(_) => preserve,
        })
    }
}
