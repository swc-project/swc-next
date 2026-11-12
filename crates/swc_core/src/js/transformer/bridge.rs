use swc_next_ecma_ast::ffi::ast::{diagnostics_buffer_size, serialize_diagnostics_into};

use super::transform_js;
use crate::js::JsTransformApiOptions;

/// Transform source and encode the result for the JavaScript bindings.
///
/// The buffer contains three little-endian `u32` values (code byte length, source
/// map byte length, diagnostic count), then UTF-8 code, an optional UTF-8 source
/// map, and packed diagnostics with UTF-16 source offsets.
///
/// # Safety
///
/// `source` must contain valid UTF-8, and `source_utf16_len` must match its
/// UTF-16 code unit length.
pub unsafe fn transform_source_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: &str,
) -> Result<Vec<u8>, String> {
    let options: JsTransformApiOptions = serde_json::from_str(options)
        .map_err(|error| format!("Invalid transform options: {error}"))?;
    // SAFETY: The caller guarantees valid UTF-8.
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let result = transform_js(source, options);
    let code = result.code.as_bytes();
    let map = result.map.as_deref().unwrap_or_default().as_bytes();
    let code_len = u32::try_from(code.len()).expect("generated code length exceeds u32");
    let map_len = u32::try_from(map.len()).expect("source map length exceeds u32");
    let diagnostic_count =
        u32::try_from(result.diagnostics.len()).expect("diagnostic count exceeds u32");
    let diagnostics_offset = 12 + code.len() + map.len();
    let mut output = vec![0; diagnostics_offset + diagnostics_buffer_size(&result.diagnostics)];
    output[..4].copy_from_slice(&code_len.to_le_bytes());
    output[4..8].copy_from_slice(&map_len.to_le_bytes());
    output[8..12].copy_from_slice(&diagnostic_count.to_le_bytes());
    output[12..12 + code.len()].copy_from_slice(code);
    output[12 + code.len()..diagnostics_offset].copy_from_slice(map);
    serialize_diagnostics_into(
        &result.diagnostics,
        source,
        &mut output[diagnostics_offset..],
        source_utf16_len as usize,
    );
    Ok(output)
}
