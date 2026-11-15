//! Internal native/Wasm generator bindings, wrapped by the public JavaScript API.
//!
//! These low-level APIs and binary protocols do not follow semantic versioning
//! and may change in any release. Use a matching JavaScript wrapper and encoder.

use swc_next_bridge::bridge;
use swc_next_core::js::generator::{
    DeserializeError, GenerateOutput, JsGeneratorConfig as GeneratorConfig, generate_from_parts,
};

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
