use swc_experimental_allocator::{
    Allocator, CloneIn, atom::Atom, hash_map::HashMap as ArenaHashMap, vec::Vec,
};
use swc_experimental_ecma_ast::Span;

pub struct State<'a> {
    pub labels: Vec<'a, Atom<'a>>,
    /// Start position of an assignment expression that can become an arrow.
    ///
    /// `u32::MAX` is reserved outside real source positions, so it
    /// works as the no-potential-arrow sentinel without the extra `Option`
    /// branch on expression parser hot paths.
    pub potential_arrow_start: u32,
    /// Start position of an AST node and the span of its trailing comma.
    pub trailing_commas: ArenaHashMap<'a, u32, Span>,
}

impl<'a> CloneIn<'a> for State<'_> {
    type Cloned = State<'a>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        let mut trailing_commas =
            ArenaHashMap::with_capacity_in(self.trailing_commas.len(), allocator);
        for (&pos, &span) in &self.trailing_commas {
            trailing_commas.insert(pos, span);
        }

        State {
            labels: self.labels.clone_in(allocator),
            potential_arrow_start: self.potential_arrow_start,
            trailing_commas,
        }
    }
}

impl<'a> State<'a> {
    pub fn new_in(allocator: &'a Allocator) -> Self {
        State {
            labels: Vec::new_in(allocator),
            potential_arrow_start: u32::MAX,
            trailing_commas: ArenaHashMap::new_in(allocator),
        }
    }
}
