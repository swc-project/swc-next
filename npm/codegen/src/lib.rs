use napi::{Error, Result, bindgen_prelude::Buffer};
use swc_next_bridge::{CodegenOptions, CodegenOutput, bridge};

#[bridge]
pub fn print_sync(buffer: Buffer, options: Option<CodegenOptions>) -> Result<CodegenOutput> {
    swc_next_bridge::print_from_buffer(&buffer, options)
        .map_err(|error| Error::from_reason(error.to_string()))
}
