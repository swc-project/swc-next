use swc_core::common::{BytePos, DUMMY_SP};

use crate::NodeIdTrait;

pub trait GetSpan {
    fn span(&self, ast: &crate::Ast) -> crate::Span;

    #[inline]
    fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }

    #[inline]
    fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
}

pub trait SetSpan {
    fn set_span(&mut self, ast: &mut crate::Ast, span: crate::Span);
}

impl<T: NodeIdTrait> GetSpan for T {
    #[inline]
    fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.get_node_unchecked(self.node_id()).span }
    }
}

impl<T: NodeIdTrait> SetSpan for T {
    #[inline]
    fn set_span(&mut self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.get_node_unchecked_mut(self.node_id()).span = span;
        }
    }
}

impl GetSpan for BytePos {
    /// Creates a new single-byte span.
    #[inline(always)]
    fn span(&self, _ast: &crate::Ast) -> crate::Span {
        crate::Span::new(*self, *self)
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
