use crate::context::Container;
use crate::used_names::UsedNameState;
use crate::{assumptions::Assumptions, context::StatementInjector, helpers::Helpers};
use swc_next_ecma_ast::NodeId;
use swc_next_ecma_semantic::ScopeModel;

/// Configuration, caches and pending work for a single transform.
/// Passes access state through the context's mutation APIs.
pub(crate) struct TransformerState<'s, 'a> {
    pub(crate) path: Vec<NodeId>,
    pub(crate) semantic: &'s mut ScopeModel<'a>,
    pub(crate) assumptions: Assumptions,
    pub(crate) helpers: Helpers,
    pub(crate) statement_injector: StatementInjector,
    pub(crate) container: Container,
    pub(crate) used_names: Option<UsedNameState<'a>>,
}

impl<'s, 'a> TransformerState<'s, 'a> {
    pub(crate) fn new(semantic: &'s mut ScopeModel<'a>, assumptions: Assumptions) -> Self {
        Self {
            path: Vec::with_capacity(64),
            semantic,
            assumptions,
            helpers: Helpers::default(),
            statement_injector: Default::default(),
            container: Default::default(),
            used_names: None,
        }
    }
}
