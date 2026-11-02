use swc_next_bridge::bridge;
use swc_next_core::js::generator::{
    DeserializeError, GenerateOutput, JsGeneratorConfig as GeneratorConfig, generate_from_buffers,
    generate_from_buffers_unchecked, generate_from_parts,
};

#[bridge]
/// Generate a validated FFI2 base buffer and its edit extension.
pub fn generate_buffers_sync(
    base: &[u8],
    extension: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, swc_next_ecma_ast::ffi2::DeserializeError> {
    generate_from_buffers(base, extension, options)
}

#[bridge]
/// # Safety
/// Buffers must come from the matching parser/AST encoder and stay immutable.
pub unsafe fn generate_buffers_trusted_sync(
    base: &[u8],
    extension: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, swc_next_ecma_ast::ffi2::DeserializeError> {
    unsafe { generate_from_buffers_unchecked(base, extension, options) }
}

#[bridge]
/// # Safety
///
/// Sections must be produced by the matching JavaScript encoder and satisfy
/// the validity, alignment, and immutability contracts of
/// `swc_next_core::js::generator::generate_from_parts`.
pub unsafe fn generate_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, DeserializeError> {
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe { generate_from_parts(header, spans, kinds, payloads, extra, strings, options) }
}
