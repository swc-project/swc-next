pub const DUMMY_SP: Span = Span { start: 0, end: 0 };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
