use crate::DUMMY_SP;

pub trait GetSpan: Copy {
    fn span(self, ast: &crate::Ast<'_>) -> crate::Span;

    #[inline]
    fn span_lo(self, ast: &crate::Ast<'_>) -> u32 {
        self.span(ast).start
    }

    #[inline]
    fn span_hi(self, ast: &crate::Ast<'_>) -> u32 {
        self.span(ast).end
    }
}

pub trait SetSpan: Copy {
    fn set_span(self, ast: &mut crate::Ast<'_>, span: crate::Span);
}

impl<S> GetSpan for Option<S>
where
    S: GetSpan,
{
    #[inline]
    fn span(self, ast: &crate::Ast<'_>) -> crate::Span {
        match self {
            Some(s) => s.span(ast),
            None => DUMMY_SP,
        }
    }
}
