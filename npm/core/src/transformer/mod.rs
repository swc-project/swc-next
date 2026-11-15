#[cfg(target_family = "wasm")]
use swc_next_bridge::bridge;
#[cfg(not(target_family = "wasm"))]
use swc_next_bridge::{napi, napi_derive::napi};
#[cfg(target_family = "wasm")]
use swc_next_core::js::transformer::bridge::{TransformApiOptions, transform_to_buffer};

#[cfg(target_family = "wasm")]
#[bridge]
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
    ts_args_type = "source: Buffer, sourceUtf16Len: number, options?: TransformApiOptions | undefined | null",
    ts_return_type = "string | Buffer"
)]
/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
pub unsafe fn transform_sync(
    env: napi::Env,
    source: napi::bindgen_prelude::Buffer,
    source_utf16_len: u32,
    options: napi::bindgen_prelude::Buffer,
) -> napi::Result<
    napi::bindgen_prelude::Either<
        <String as swc_next_bridge::NativeAbi>::Abi,
        napi::bindgen_prelude::Buffer,
    >,
> {
    unsafe {
        swc_next_core::js::transformer::bridge::transform_to_node_output(
            &env,
            &source,
            source_utf16_len,
            &options,
        )
    }
}
