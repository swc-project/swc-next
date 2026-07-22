use crate::assumptions::Assumptions;
use swc_experimental_ecma_ast::TraverseCtx;

pub type TransformerCtx<'t, 'a> = TraverseCtx<'t, 'a, TransformerState>;

pub struct TransformerState {
    pub assumptions: Assumptions,
}
