use crate::{
    assumptions::Assumptions,
    context::{GeneratedBinding, StatementInjector},
    helpers::Helpers,
};
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{AstUtf8, NodeId};
use swc_next_ecma_semantic::Semantic;

/// Configuration, caches and pending work for a single transform.
/// Passes access state through the context's mutation APIs.
pub(crate) struct TransformerState<'s, 'a> {
    pub(crate) path: Vec<NodeId>,
    pub(crate) semantic: &'s mut Semantic<'a>,
    pub(crate) assumptions: Assumptions,
    pub(crate) helpers: Helpers,
    pub(crate) statement_injector: StatementInjector,
    pub(crate) generated_bindings: FxHashMap<AstUtf8, GeneratedBinding>,
}

impl<'s, 'a> TransformerState<'s, 'a> {
    pub(crate) fn new(semantic: &'s mut Semantic<'a>, assumptions: Assumptions) -> Self {
        Self {
            path: Vec::with_capacity(64),
            semantic,
            assumptions,
            helpers: Helpers::default(),
            statement_injector: Default::default(),
            generated_bindings: Default::default(),
        }
    }
}
