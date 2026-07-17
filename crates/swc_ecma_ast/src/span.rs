pub const DUMMY_SP: Span = Span { start: 0, end: 0 };

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct OptionalSpan {
    start: u32,
    end: u32,
}

impl From<Option<Span>> for OptionalSpan {
    #[inline]
    fn from(value: Option<Span>) -> Self {
        value.map_or(
            Self {
                start: u32::MAX,
                end: u32::MAX,
            },
            |span| Self {
                start: span.start,
                end: span.end,
            },
        )
    }
}

impl From<OptionalSpan> for Option<Span> {
    #[inline]
    fn from(value: OptionalSpan) -> Self {
        (value.start != u32::MAX).then_some(Span {
            start: value.start,
            end: value.end,
        })
    }
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn is_dummy(self) -> bool {
        self.start == 0 && self.end == 0
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
