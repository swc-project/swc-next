use napi::{Result, bindgen_prelude::Buffer};
use napi_derive::napi;
use swc_next_napi::{CodegenOptions, CodegenOutput};

#[napi]
pub fn print_sync(buffer: Buffer, options: Option<CodegenOptions>) -> Result<CodegenOutput> {
    swc_next_napi::print_from_buffer(buffer, options)
}
