use swc_next_bridge::bridge;

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
