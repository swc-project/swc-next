use swc_core::atoms::wtf8::{CodePoint, Wtf8, Wtf8Buf};
use swc_core::common::{BytePos, Span};

#[derive(Debug, Clone, Copy)]
pub enum MaybeSubUtf8 {
    Empty,
    Inline((BytePos, BytePos)),
    Alloc((u32, u32)),
}

impl MaybeSubUtf8 {
    #[inline]
    pub(crate) fn new_from_source(start: BytePos, end: BytePos) -> Self {
        Self::Inline((start, end))
    }

    #[inline]
    pub(crate) fn new_from_span(span: Span) -> Self {
        Self::Inline((span.lo, span.hi))
    }

    #[inline]
    fn new_from_allocated(start: u32, end: u32) -> Self {
        Self::Alloc((start, end))
    }

    #[inline]
    pub(crate) fn new_empty() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MaybeSubWtf8 {
    Empty,
    Inline((BytePos, BytePos)),
    Alloc((u32, u32)),
}

impl MaybeSubWtf8 {
    #[inline]
    pub(crate) fn new_from_source(start: BytePos, end: BytePos) -> Self {
        Self::Inline((start, end))
    }

    #[inline]
    fn new_from_allocated(start: u32, end: u32) -> Self {
        Self::Alloc((start, end))
    }
}

#[derive(Clone)]
pub struct StringAllocator {
    allocated_utf8: String,
    allocated_wtf8: Wtf8Buf,
}

impl StringAllocator {
    #[inline]
    pub fn new() -> Self {
        Self {
            allocated_utf8: String::new(),
            allocated_wtf8: Wtf8Buf::new(),
        }
    }

    #[inline]
    pub fn alloc_utf8(&mut self) -> Utf8Builder {
        Utf8Builder {
            start: self.allocated_utf8.len(),
        }
    }

    #[inline]
    pub fn alloc_wtf8(&mut self) -> Wtf8Builder {
        Wtf8Builder {
            start: self.allocated_wtf8.len(),
        }
    }

    #[inline]
    pub fn get_allocated_utf8(&self, start: u32, end: u32) -> &str {
        &self.allocated_utf8[start as usize..end as usize]
    }

    #[inline]
    pub fn get_allocated_wtf8(&self, start: u32, end: u32) -> &Wtf8 {
        self.allocated_wtf8.slice(start as usize, end as usize)
    }
}

pub struct Utf8Builder {
    start: usize,
}

impl Utf8Builder {
    #[inline]
    pub(crate) fn push(&mut self, alloc: &mut StringAllocator, ch: char) {
        alloc.allocated_utf8.push(ch);
    }

    #[inline]
    pub(crate) fn push_str(&mut self, alloc: &mut StringAllocator, s: &str) {
        alloc.allocated_utf8.push_str(s);
    }

    #[inline]
    pub(crate) fn finish(self, alloc: &mut StringAllocator) -> MaybeSubUtf8 {
        MaybeSubUtf8::new_from_allocated(self.start as u32, alloc.allocated_utf8.len() as u32)
    }

    #[inline]
    pub(crate) fn is_empty(&self, alloc: &StringAllocator) -> bool {
        self.start == alloc.allocated_wtf8.len()
    }
}

pub struct Wtf8Builder {
    start: usize,
}

impl Wtf8Builder {
    #[inline]
    pub(crate) fn push(&mut self, alloc: &mut StringAllocator, code_point: CodePoint) {
        alloc.allocated_wtf8.push(code_point);
    }

    #[inline]
    pub(crate) fn push_char(&mut self, alloc: &mut StringAllocator, c: char) {
        alloc.allocated_wtf8.push_char(c);
    }

    #[inline]
    pub(crate) fn push_str(&mut self, alloc: &mut StringAllocator, s: &str) {
        alloc.allocated_wtf8.push_str(s);
    }

    #[inline]
    pub(crate) fn finish(self, alloc: &mut StringAllocator) -> MaybeSubWtf8 {
        let end = alloc.allocated_wtf8.len() as u32;
        alloc.allocated_wtf8.push_char('\u{FFFD}'); // Append a separator to avoid two codepoints being merged
        MaybeSubWtf8::new_from_allocated(self.start as u32, end)
    }

    #[inline]
    pub(crate) fn is_empty(&self, alloc: &StringAllocator) -> bool {
        self.start == alloc.allocated_wtf8.len()
    }
}
