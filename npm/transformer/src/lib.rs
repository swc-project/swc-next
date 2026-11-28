//! Internal native/Wasm transform bindings, wrapped by the public JavaScript API.
//!
//! These low-level APIs and binary protocols do not follow semantic versioning
//! and may change in any release. Use matching JavaScript wrappers and codecs.

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(target_family = "wasm")]
use swc_next::js::transformer::bridge::transform_to_buffer;
#[cfg(target_family = "wasm")]
use swc_next_bridge::bridge;
#[cfg(not(target_family = "wasm"))]
use swc_next_bridge::{napi, napi_derive::napi};

#[cfg(target_family = "wasm")]
#[bridge(ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: Uint8Array")]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
///
/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
pub unsafe fn transform_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    unsafe { transform_to_buffer(source, source_utf16_len, options) }
}

#[cfg(not(target_family = "wasm"))]
#[napi(
    js_name = "transformSync",
    ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: Uint8Array",
    ts_return_type = "string | Uint8Array"
)]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
///
/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
pub unsafe fn transform_sync(
    env: napi::Env,
    source: swc_next_bridge::NativeBytes<'_>,
    source_utf16_len: u32,
    options: swc_next_bridge::NativeBytes<'_>,
) -> napi::Result<
    napi::bindgen_prelude::Either<
        <String as swc_next_bridge::NativeAbi>::Abi,
        napi::bindgen_prelude::Buffer,
    >,
> {
    unsafe {
        swc_next::js::transformer::bridge::transform_to_node_output(
            &env,
            &source,
            source_utf16_len,
            &options,
        )
    }
}

#[cfg(target_family = "wasm")]
#[bridge(js_name = "__transformV1WithHooks")]
/// # Safety
/// Source must be valid UTF-8 and have the specified UTF-16 length.
pub unsafe fn transform_v1_with_hooks(
    source: &[u8],
    source_utf16_len: u32,
    config: &[u8],
    stages: u32,
) -> Result<Vec<u8>, String> {
    unsafe {
        swc_next::js::transformer::plugins::transform_to_wasm_buffer(
            source,
            source_utf16_len,
            config,
            stages,
        )
    }
}

#[cfg(not(target_family = "wasm"))]
use swc_next::js::transformer::plugins::{NativePluginCallback, NativePluginOutput};

#[cfg(not(target_family = "wasm"))]
#[napi(
    js_name = "__transformV1WithHooks",
    ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: Uint8Array, stages: number, callback: (stage: number, ast: Uint8Array) => Uint8Array",
    ts_return_type = "string | Uint8Array"
)]
/// # Safety
/// Source must be valid UTF-8 with the specified UTF-16 length. The callback
/// must return a matching AST transfer with the original source.
pub unsafe fn transform_v1_with_hooks(
    env: napi::Env,
    source: swc_next_bridge::NativeBytes<'_>,
    source_utf16_len: u32,
    config: swc_next_bridge::NativeBytes<'_>,
    stages: u32,
    callback: NativePluginCallback<'_>,
) -> napi::Result<NativePluginOutput> {
    unsafe {
        swc_next::js::transformer::plugins::transform_to_node_output(
            &env,
            &source,
            source_utf16_len,
            &config,
            stages,
            callback,
        )
    }
}
