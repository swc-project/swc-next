//! Internal native/Wasm generator bindings, wrapped by the public JavaScript API.
//!
//! These low-level APIs and binary protocols do not follow semantic versioning
//! and may change in any release. Use a matching JavaScript wrapper and encoder.

use swc_next::js::generator::{
    DeserializeError, GenerateOutput, JsGeneratorConfig as GeneratorConfig, generate_from_buffers,
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
