use rustc_hash::FxHashMap;
use swc_experimental_allocator::atom::Atom;
use swc_experimental_ecma_ast::Span;

#[derive(Clone)]
pub struct State<'a> {
    pub labels: Vec<Atom<'a>>,
    /// Start position of an assignment expression that can become an arrow.
    ///
    /// `u32::MAX` is reserved outside real source positions, so it
    /// works as the no-potential-arrow sentinel without the extra `Option`
    /// branch on expression parser hot paths.
    pub potential_arrow_start: u32,
    /// Start position of an AST node and the span of its trailing comma.
    pub trailing_commas: FxHashMap<u32, Span>,
}

impl Default for State<'_> {
    fn default() -> Self {
        State {
            labels: Default::default(),
            potential_arrow_start: u32::MAX,
            trailing_commas: Default::default(),
        }
    }
}
