mod options;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{DeserializeError, deserialize_from};
use swc_next_ecma_codegen::print;

pub use self::options::CodegenOptions;

pub fn print_from_buffer(
    buffer: &[u8],
    options: Option<CodegenOptions>,
) -> Result<String, DeserializeError> {
    let allocator = Allocator::default();
    let tree = deserialize_from(buffer, &allocator)?;
    let result = print(&allocator, &tree, options.unwrap_or_default().into());
    Ok(result.code.into())
}
