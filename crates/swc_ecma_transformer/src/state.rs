use crate::assumptions::Assumptions;
use swc_next_ecma_ast::TraverseMutCtx;

pub type TransformerCtx<'t, 'a> = TraverseMutCtx<'t, 'a, TransformerState>;

pub struct TransformerState {
    pub assumptions: Assumptions,
}
