use crate::{DUMMY_SP, NodeIdTrait};

pub trait GetSpan {
    fn span(&self, ast: &crate::Ast<'_>) -> crate::Span;

    #[inline]
    fn span_lo(&self, ast: &crate::Ast<'_>) -> u32 {
        self.span(ast).start
    }

    #[inline]
    fn span_hi(&self, ast: &crate::Ast<'_>) -> u32 {
        self.span(ast).end
    }
}

pub trait SetSpan {
    fn set_span(&mut self, ast: &mut crate::Ast<'_>, span: crate::Span);
}

impl<T: NodeIdTrait> GetSpan for T {
    #[inline]
    fn span(&self, ast: &crate::Ast<'_>) -> crate::Span {
        // SAFETY: `NodeIdTrait` guarantees that this ID belongs to the AST.
        unsafe { ast.nodes.get_unchecked(self.node_id().index()).span }
    }
}

impl<T: NodeIdTrait> SetSpan for T {
    #[inline]
    fn set_span(&mut self, ast: &mut crate::Ast<'_>, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.node_id().index()).span = span;
        }
    }
}

impl<S> GetSpan for Option<S>
where
    S: GetSpan,
{
    #[inline]
    fn span(&self, ast: &crate::Ast<'_>) -> crate::Span {
        match self {
            Some(s) => s.span(ast),
            None => DUMMY_SP,
        }
    }
}
