use swc_next::js::generator::{GenerateOutput, JsGeneratorConfig as GeneratorConfig};
use swc_next_allocator::Allocator;
use swc_next_bridge::bridge;
use swc_next_ecma_ast::ffi2::{DeserializeError, deserialize, deserialize_unchecked};
use swc_next_ecma_codegen::generate;

#[bridge(
    ts_args_type = "base: Uint8Array, extension: Uint8Array, options: number",
    ts_return_type = "string"
)]
pub fn generate_sync2(
    base: &[u8],
    extension: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, DeserializeError> {
    let allocator = Allocator::default();
    let ast = deserialize(base, extension, &allocator)?;
    let result = generate(&ast, options.unwrap_or_default().into());
    Ok(GenerateOutput { code: result.code })
}

#[bridge(
    ts_args_type = "base: Uint8Array, extension: Uint8Array, options: number",
    ts_return_type = "string"
)]
/// # Safety
/// Buffers must satisfy `deserialize_unchecked`'s contract and stay immutable.
pub unsafe fn generate_sync2_unchecked(
    base: &[u8],
    extension: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, DeserializeError> {
    let allocator = Allocator::default();
    let ast = unsafe { deserialize_unchecked(base, extension, &allocator) }?;
    let result = generate(&ast, options.unwrap_or_default().into());
    Ok(GenerateOutput { code: result.code })
}
