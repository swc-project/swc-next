use napi::{Error, Result, bindgen_prelude::Buffer};
use napi_derive::napi;
use swc_next_napi::{CodegenOptions as SharedCodegenOptions, print_from_buffer};

#[napi(object)]
#[derive(Default)]
pub struct CodegenOptions {
    pub ascii_only: Option<bool>,
    pub minify: Option<bool>,
}

#[napi(object)]
pub struct CodegenOutput {
    pub code: String,
}

#[napi]
pub fn print_sync(buffer: Buffer, options: Option<CodegenOptions>) -> Result<CodegenOutput> {
    let options = options.map(|options| SharedCodegenOptions {
        ascii_only: options.ascii_only,
        minify: options.minify,
    });
    let code = print_from_buffer(&buffer, options)
        .map_err(|error| Error::from_reason(error.to_string()))?;
    Ok(CodegenOutput { code })
}
