use swc_next::js::generator::{
    DeserializeError, GenerateOutput, JsGeneratorConfig as GeneratorConfig, generate_from_buffers,
    generate_from_buffers_unchecked,
};
use swc_next_bridge::bridge;

#[bridge(
    ts_args_type = "base: Uint8Array, extension: Uint8Array, options: number",
    ts_return_type = "string"
)]
/// Generate from a validated base buffer and its edit extension.
pub fn generate_sync(
    base: &[u8],
    extension: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, DeserializeError> {
    generate_from_buffers(base, extension, options)
}

#[bridge(
    ts_args_type = "base: Uint8Array, extension: Uint8Array, options: number",
    ts_return_type = "string"
)]
/// # Safety
/// Buffers must come from the matching parser/AST encoder and stay immutable.
pub unsafe fn generate_sync_unchecked(
    base: &[u8],
    extension: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, DeserializeError> {
    unsafe { generate_from_buffers_unchecked(base, extension, options) }
}
