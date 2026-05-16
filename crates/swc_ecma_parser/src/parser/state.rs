use rustc_hash::FxHashMap;
use swc_core::common::{BytePos, Span};
use swc_experimental_ecma_ast::Utf8Ref;

#[derive(Clone)]
pub struct State {
    pub labels: Vec<Utf8Ref>,
    /// Start position of an assignment expression that can become an arrow.
    ///
    /// `BytePos::SYNTHESIZED` is reserved outside real source positions, so it
    /// works as the no-potential-arrow sentinel without the extra `Option`
    /// branch on expression parser hot paths.
    pub potential_arrow_start: BytePos,
    /// Start position of an AST node and the span of its trailing comma.
    pub trailing_commas: FxHashMap<BytePos, Span>,
}

impl Default for State {
    fn default() -> Self {
        State {
            labels: Default::default(),
            potential_arrow_start: BytePos::SYNTHESIZED,
            trailing_commas: Default::default(),
        }
    }
}
