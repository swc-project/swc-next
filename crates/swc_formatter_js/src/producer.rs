use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, Severity};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_formatter_core::{Doc, Writer};
use swc_next_formatter_host::{DocumentProducer, FormatContext, FormatSession};

use crate::{FormatError, FormatOptions, SyntaxOptions, formatter::Formatter};

/// Produces JavaScript and TypeScript documents in a generic host session.
///
/// Parser settings belong to this producer. Syntax options are supplied for
/// each document, while layout comes from the session. Comment collection and
/// parenthesis preservation are always enabled.
///
/// Documents omit a leading BOM. Document consumers decide how it belongs in
/// their surrounding output.
///
/// ```
/// use swc_next_formatter_js::{EcmaDocumentProducer, SyntaxOptions};
/// use swc_next_formatter_core::PrintOptions;
/// use swc_next_formatter_host::FormatSession;
///
/// let session = FormatSession::new(PrintOptions::default());
/// let producer = EcmaDocumentProducer::default();
/// let document = session.produce(&producer, "const answer=42", &SyntaxOptions::default())?;
/// assert!(document.encode().starts_with(b"SWCD"));
/// # Ok::<(), swc_next_formatter_js::FormatError>(())
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct EcmaDocumentProducer {
    parser_options: ParserOptions,
}

impl EcmaDocumentProducer {
    pub fn new(parser_options: ParserOptions) -> Self {
        Self { parser_options }
    }

    pub(super) fn parse<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
    ) -> Result<AstCtx<'a>, FormatError> {
        let options = ParserOptions {
            comments: CommentMode::Flat,
            preserve_parens: true,
            ..self.parser_options
        };
        let parsed = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
        if parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            return Err(FormatError::Parse(parsed.diagnostics));
        }
        Ok(parsed.ast)
    }
}

impl DocumentProducer for EcmaDocumentProducer {
    type Options = SyntaxOptions;
    type Error = FormatError;

    fn produce<'a>(
        &self,
        context: FormatContext<'a, '_>,
        source: &'a str,
        options: &SyntaxOptions,
    ) -> Result<Doc<'a>, FormatError> {
        // The AST and all of its backing collections use arena storage. Keep
        // its header in the session too, so the Doc can outlive this call.
        // Reject future Drop fields rather than silently leaking their resources.
        const { assert!(!std::mem::needs_drop::<AstCtx<'_>>()) };
        let ast = self.parse(context.allocator(), source)?;
        let ast = context.allocator().alloc(ast);
        ast_to_doc(ast, context, *options)
    }
}

/// Build an unprinted document from an existing program AST.
///
/// The AST must have a program root, as produced by
/// [`swc_next_ecma_parser::Parser::parse`], and must retain its original source,
/// complete comments parsed with [`swc_next_ecma_parser::CommentMode::Flat`],
/// and parentheses parsed with `preserve_parens: true`. Missing comments or
/// parentheses can lose annotations or change which expression they apply to.
/// Use [`EcmaDocumentProducer`] to enable both parsing requirements automatically.
///
/// The document borrows the AST and context storage. It omits a leading BOM,
/// leaving printing, encoding, and BOM handling to the caller.
///
/// # Panics
///
/// Panics if the AST has no root or its root is not a program.
pub fn ast_to_doc<'a>(
    ast: &'a AstCtx<'_>,
    context: FormatContext<'a, '_>,
    syntax: SyntaxOptions,
) -> Result<Doc<'a>, FormatError> {
    // SAFETY: The formatter only reads handles obtained from this AST.
    let ast = unsafe { ast.ast() };
    let mut writer = Writer::new(context.allocator());
    let options = FormatOptions::from_parts(context.print_options(), syntax);
    Formatter::new(ast, options, context.allocator()).format_program(&mut writer)?;
    Ok(writer.finish())
}

/// Parse source and encode an unprinted document for the SWCFMT Prettier adapter.
///
/// The binary protocol is internal and must match the JavaScript decoder version.
/// All borrowed source and document storage is released before returning.
pub fn format_to_doc_buffer(
    source: &str,
    parser_options: ParserOptions,
    options: FormatOptions,
) -> Result<Vec<u8>, FormatError> {
    let session = FormatSession::new(options.into());
    let producer = EcmaDocumentProducer::new(parser_options);
    Ok(session
        .produce(&producer, source, &options.into())?
        .encode())
}
