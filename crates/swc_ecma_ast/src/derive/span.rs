use crate::{DUMMY_SP, NodeIdTrait};

pub trait GetSpan {
    fn span(&self, ast: &crate::Ast) -> crate::Span;

    #[inline]
    fn span_lo(&self, ast: &crate::Ast) -> u32 {
        self.span(ast).start
    }

    #[inline]
    fn span_hi(&self, ast: &crate::Ast) -> u32 {
        self.span(ast).end
    }
}

pub trait SetSpan {
    fn set_span(&mut self, ast: &mut crate::Ast, span: crate::Span);
}

impl<T: NodeIdTrait> GetSpan for T {
    #[inline]
    fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}

impl<T: NodeIdTrait> SetSpan for T {
    #[inline]
    fn set_span(&mut self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}

impl<S> GetSpan for Option<S>
where
    S: GetSpan,
{
    #[inline]
    fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Some(s) => s.span(ast),
            None => DUMMY_SP,
        }
    }
}
