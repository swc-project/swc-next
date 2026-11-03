use swc_next_bridge::{ParserOptions, bridge};

#[cfg(all(feature = "test-native", not(target_family = "wasm")))]
pub mod test;

#[bridge]
pub fn format_doc_buffer_sync(
    source: String,
    parser: Option<ParserOptions>,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    swc_next_bridge::format_doc(source, parser, options)
}

#[bridge]
pub fn print_doc_buffer_sync(buffer: &[u8], options: &[u8]) -> Result<String, String> {
    swc_next_bridge::print_doc(buffer, options)
}

#[bridge]
pub fn print_doc_buffer_with_cursor_sync(buffer: &[u8], options: &[u8]) -> Result<Vec<u8>, String> {
    let result = swc_next_bridge::print_doc_with_cursor(buffer, options)?;
    let mut output = Vec::with_capacity(result.text.len() + 9);
    if let Some(range) = result.cursor_range {
        let start = u32::try_from(range.start).map_err(|error| error.to_string())?;
        let end = u32::try_from(range.end).map_err(|error| error.to_string())?;
        output.push(1);
        output.extend(start.to_le_bytes());
        output.extend(end.to_le_bytes());
    } else {
        output.push(0);
    }
    output.extend(result.text.into_bytes());
    Ok(output)
}
