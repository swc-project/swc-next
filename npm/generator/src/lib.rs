//! Internal native/Wasm generator bindings, wrapped by the public JavaScript API.
//!
//! These low-level APIs and binary protocols do not follow semantic versioning
//! and may change in any release. Use a matching JavaScript wrapper and encoder.

use swc_next::js::generator::generate_from_buffers;
use swc_next::js::generator::{
    DeserializeError, GenerateOutput, JsGeneratorConfig as GeneratorConfig, generate_from_parts,
};
use swc_next_bridge::bridge;

#[bridge(
    ts_args_type = "header: Uint8Array, spans: Uint8Array, kinds: Uint8Array, payloads: Uint8Array, extra: Uint8Array, strings: Uint8Array, options: number",
    ts_return_type = "string"
)]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
///
/// # Safety
///
/// Sections must be produced by the matching JavaScript encoder and satisfy
/// the validity, alignment, and immutability contracts of
/// `swc_next::js::generator::generate_from_parts`.
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

#[bridge(
    ts_args_type = "base: Uint8Array, extension: Uint8Array, options: number",
    ts_return_type = "string"
)]
/// Generate from a validated base buffer and its edit extension.
pub fn generate_sync2(
    base: &[u8],
    extension: &[u8],
    options: Option<GeneratorConfig>,
) -> Result<GenerateOutput, swc_next::js::generator::DeserializeError2> {
    generate_from_buffers(base, extension, options)
}
