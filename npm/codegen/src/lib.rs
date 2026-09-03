use swc_next_bridge::{CodegenOptions, CodegenOutput, DeserializeError, bridge};

#[bridge]
pub fn print_sync(
    buffer: &[u8],
    options: Option<CodegenOptions>,
) -> Result<CodegenOutput, DeserializeError> {
    swc_next_bridge::print_from_buffer(buffer, options)
}
