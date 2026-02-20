use rustc_hash::FxHashMap;
use swc_core::common::{BytePos, Span};
use swc_experimental_ecma_ast::Utf8Ref;

#[derive(Clone, Default)]
pub struct State {
    pub labels: Vec<Utf8Ref>,
    /// Start position of an assignment expression.
    pub potential_arrow_start: Option<BytePos>,
    /// Start position of an AST node and the span of its trailing comma.
    pub trailing_commas: FxHashMap<BytePos, Span>,
}
