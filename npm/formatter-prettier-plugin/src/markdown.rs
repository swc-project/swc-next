//! Native and Wasm Markdown formatting document production.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.

use swc_next_bridge::bridge;
use swc_next_formatter_core::{IndentStyle, IndentWidth, LineEnding, PrintOptions};
use swc_next_formatter_host::FormatSession;
use swc_next_formatter_markdown::{
    EmbeddedBlock, MarkdownDocumentProducer, ProseWrap, SyntaxOptions, plugin_document,
};

#[bridge(
    ts_args_type = "source: string, proseWrap: number, singleQuote: boolean, lineWidth: number, indentWidth: number, useTabs: boolean, lineEnding: number, includeEmbeds: boolean",
    ts_return_type = "Uint8Array"
)]
/// Produces a SWCD document with optional records for deferred Markdown blocks.
#[allow(
    clippy::too_many_arguments,
    reason = "The binding passes scalar options across the native and Wasm ABIs."
)]
pub fn format_markdown_doc_buffer_sync(
    source: String,
    prose_wrap: u32,
    single_quote: bool,
    line_width: u32,
    indent_width: u32,
    use_tabs: bool,
    line_ending: u32,
    include_embeds: bool,
) -> Result<Vec<u8>, String> {
    let prose_wrap = match prose_wrap {
        0 => ProseWrap::Preserve,
        1 => ProseWrap::Always,
        2 => ProseWrap::Never,
        _ => return Err("invalid Markdown prose wrap option".into()),
    };
    let options = SyntaxOptions {
        prose_wrap,
        single_quote,
    };
    let line_ending = match line_ending {
        0 => LineEnding::Lf,
        1 => LineEnding::CrLf,
        2 => LineEnding::Cr,
        _ => return Err("invalid Markdown line ending".into()),
    };
    let indent_width = u8::try_from(indent_width).map_err(|_| "invalid Markdown indent width")?;
    let session = FormatSession::new(PrintOptions {
        line_width: u16::try_from(line_width).map_err(|_| "invalid Markdown line width")?,
        indent_width: IndentWidth::try_from(indent_width).map_err(|error| error.to_string())?,
        indent_style: if use_tabs {
            IndentStyle::Tab
        } else {
            IndentStyle::Space
        },
        line_ending,
    });
    let context = session.context();
    if !include_embeds {
        let doc = context
            .produce(&MarkdownDocumentProducer, &source, &options)
            .map_err(|error| error.to_string())?;
        return Ok(doc.encode_for_prettier());
    }
    let (doc, blocks) =
        plugin_document(&swc_next_markdown_parser::parse(&source), context, options)
            .map_err(|error| error.to_string())?;
    let records = blocks
        .into_iter()
        .map(encode_block)
        .collect::<Result<Vec<_>, _>>()?;
    doc.encode_for_prettier_with_label_data(&records)
        .map_err(|error| error.to_string())
}

fn encode_block(block: EmbeddedBlock) -> Result<Vec<u8>, String> {
    match block {
        EmbeddedBlock::Fence(fence) => {
            let mut record = block_record(
                0,
                fence.start,
                fence.end,
                &[
                    &fence.language,
                    &fence.body,
                    fence.meta.as_deref().unwrap_or(""),
                ],
            )?;
            if let Some(info) = &fence.source_info {
                write_text(&mut record, info)?;
            }
            Ok(record)
        }
        EmbeddedBlock::FrontMatter {
            start,
            end,
            language,
            opening,
            closing,
            body,
        } => block_record(1, start, end, &[&language, &body, &opening, &closing]),
    }
}

fn block_record(kind: u32, start: usize, end: usize, fields: &[&str]) -> Result<Vec<u8>, String> {
    let mut record = Vec::new();
    record.extend(kind.to_le_bytes());
    write_word(&mut record, start)?;
    write_word(&mut record, end)?;
    for field in fields {
        write_text(&mut record, field)?;
    }
    Ok(record)
}

fn write_text(output: &mut Vec<u8>, text: &str) -> Result<(), String> {
    write_word(output, text.len())?;
    output.extend_from_slice(text.as_bytes());
    Ok(())
}

fn write_word(output: &mut Vec<u8>, value: usize) -> Result<(), String> {
    let value = u32::try_from(value).map_err(|_| "Markdown document exceeds bridge size limit")?;
    output.extend(value.to_le_bytes());
    Ok(())
}
