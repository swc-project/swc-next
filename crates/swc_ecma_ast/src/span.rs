use swc_experimental_allocator::boxed::Box;

pub const DUMMY_SP: Span = Span { start: 0, end: 0 };

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn is_dummy(&self) -> bool {
        self.start == 0 && self.end == 0
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

pub trait GetSpan {
    fn span(&self) -> crate::Span;

    #[inline]
    fn span_lo(&self) -> u32 {
        self.span().start
    }

    #[inline]
    fn span_hi(&self) -> u32 {
        self.span().end
    }
}

pub trait SetSpan {
    fn set_span(&mut self, span: crate::Span);
}

impl<S> GetSpan for Option<S>
where
    S: GetSpan,
{
    #[inline]
    fn span(&self) -> crate::Span {
        match self {
            Some(s) => s.span(),
            None => DUMMY_SP,
        }
    }
}

impl<'a, S> GetSpan for Box<'a, S>
where
    S: GetSpan,
{
    #[inline]
    fn span(&self) -> crate::Span {
        (**self).span()
    }
}

impl<'a, S> SetSpan for Box<'a, S>
where
    S: SetSpan,
{
    #[inline]
    fn set_span(&mut self, span: crate::Span) {
        (**self).set_span(span);
    }
}
