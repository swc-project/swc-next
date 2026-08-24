mod options;

use napi::{Error, Result, bindgen_prelude::Buffer};
use napi_derive::napi;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::deserialize_from;
use swc_next_ecma_codegen::print;

pub use self::options::CodegenOptions;

#[napi(object)]
pub struct CodegenOutput {
    pub code: String,
}

pub fn print_from_buffer(buffer: Buffer, options: Option<CodegenOptions>) -> Result<CodegenOutput> {
    let allocator = Allocator::default();
    let tree = deserialize_from(&buffer, &allocator)
        .map_err(|error| Error::from_reason(error.to_string()))?;
    let result = print(&allocator, &tree, options.unwrap_or_default().into());
    Ok(CodegenOutput {
        code: result.code.into(),
    })
}
