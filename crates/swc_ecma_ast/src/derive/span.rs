use crate::NodeIdTrait;

pub trait Spanned {
    fn span(&self, ast: &crate::Ast) -> crate::Span;

    fn set_span(&self, ast: &mut crate::Ast, span: crate::Span);

    #[inline]
    fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }

    #[inline]
    fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
}

impl<T: NodeIdTrait> Spanned for T {
    #[inline]
    fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.get_node_unchecked(self.node_id()).span }
    }

    #[inline]
    fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.get_node_unchecked_mut(self.node_id()).span = span;
        }
    }
}
