mod parser;

pub use parser::{
    CommentMode, FragmentContext, FunctionFragmentContext, Lang, ParserOptions, SourceType,
    parse_expression_fragment_to_buffer, parse_statement_fragment_to_buffer, parse_to_buffer,
};
