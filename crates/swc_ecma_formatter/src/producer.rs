use swc_next_ecma_ast::AstCtx;
use swc_next_ecma_parser::Options as ParserOptions;
use swc_next_formatter_core::{Doc, Writer};
use swc_next_formatter_host::{DocumentProducer, FormatContext};

use crate::{FormatError, FormatOptions, SyntaxOptions, formatter::Formatter, parse_source};

/// Produces JavaScript and TypeScript documents in a generic host session.
///
/// Parser settings belong to this producer. Syntax options are supplied for
/// each document, while layout comes from the session. Comment collection and
/// parenthesis preservation are always enabled, as in [`crate::format()`].
///
/// Documents omit a leading BOM. Source-formatting convenience APIs restore it;
/// document consumers decide how it belongs in their surrounding output.
///
/// ```
/// use swc_next_ecma_formatter::{EcmaDocumentProducer, SyntaxOptions};
/// use swc_next_formatter_core::PrintOptions;
/// use swc_next_formatter_host::FormatSession;
///
/// let session = FormatSession::new(PrintOptions::default());
/// let producer = EcmaDocumentProducer::default();
/// let document = session.produce(&producer, "const answer=42", &SyntaxOptions::default())?;
/// assert_eq!(document.print(session.print_options()), "const answer = 42;\n");
/// # Ok::<(), swc_next_ecma_formatter::FormatError>(())
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct EcmaDocumentProducer {
    parser_options: ParserOptions,
}

impl EcmaDocumentProducer {
    pub fn new(parser_options: ParserOptions) -> Self {
        Self { parser_options }
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
        let ast = parse_source(context.allocator(), source, self.parser_options)?;
        let ast = context.allocator().alloc(ast);
        ast_to_doc(ast, context, *options)
    }
}

pub(crate) fn ast_to_doc<'a>(
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
