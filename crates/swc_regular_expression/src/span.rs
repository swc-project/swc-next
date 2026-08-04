#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub const fn empty(offset: u32) -> Self {
        Self::new(offset, offset)
    }

    pub const fn sized(start: u32, size: u32) -> Self {
        Self::new(start, start + size)
    }

    pub fn merge(self, other: Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }

    pub fn source_text(self, source_text: &str) -> &str {
        &source_text[self.start as usize..self.end as usize]
    }
}
