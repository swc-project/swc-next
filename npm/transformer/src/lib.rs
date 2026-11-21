//! Internal native/Wasm transform bindings, wrapped by the public JavaScript API.
//!
//! These low-level APIs and binary protocols do not follow semantic versioning
//! and may change in any release. Use matching JavaScript wrappers and codecs.

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(target_family = "wasm")]
use swc_next::js::transformer::bridge::{TransformApiOptions, transform_to_buffer};
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
    options: Option<TransformApiOptions>,
) -> Vec<u8> {
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

use swc_next_bridge::bridge as plugin_bridge;

#[plugin_bridge]
/// # Safety
/// Source must be valid UTF-8 and have the specified UTF-16 length.
pub unsafe fn parse_for_plugins_sync(
    source: &[u8],
    source_utf16_len: u32,
    config: &[u8],
) -> Result<Vec<u8>, String> {
    unsafe {
        swc_next::js::transformer::plugins::parse_for_plugins(source, source_utf16_len, config)
    }
}

#[plugin_bridge]
/// # Safety
/// Sections must come from the matching AST encoder and remain immutable.
pub unsafe fn check_plugin_ast_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    config: &[u8],
) -> Result<Vec<u8>, String> {
    unsafe {
        swc_next::js::transformer::plugins::check_edited(
            [header, spans, kinds, payloads, extra, strings],
            config,
        )
    }
}

#[cfg(target_family = "wasm")]
#[plugin_bridge]
/// # Safety
/// Sections must come from the matching AST encoder and remain immutable.
pub unsafe fn print_plugin_ast_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    config: &[u8],
) -> Result<Vec<u8>, String> {
    unsafe {
        swc_next::js::transformer::plugins::print_edited(
            [header, spans, kinds, payloads, extra, strings],
            config,
        )
    }
}

#[cfg(not(target_family = "wasm"))]
#[plugin_bridge(ts_return_type = "string")]
/// # Safety
/// Sections must come from the matching AST encoder and remain immutable.
pub unsafe fn print_plugin_ast_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    config: &[u8],
) -> Result<swc_next::js::generator::GenerateOutput, String> {
    unsafe {
        swc_next::js::transformer::plugins::print_edited_code(
            [header, spans, kinds, payloads, extra, strings],
            config,
        )
    }
}
