#[cfg(feature = "codegen")]
mod codegen;
#[cfg(feature = "parser")]
mod parser;

#[cfg(feature = "codegen")]
pub use codegen::{CodegenOptions, CodegenOutput, print_from_buffer};
#[cfg(feature = "parser")]
pub use parser::{CommentMode, Lang, ParserOptions, SourceType, parse_to_buffer};
