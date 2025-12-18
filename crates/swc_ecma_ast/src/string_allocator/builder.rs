use swc_core::atoms::wtf8::CodePoint;

use crate::{StringAllocator, Utf8Ref, Wtf8Ref};

pub struct Utf8Builder {
    pub(crate) start: usize,
}

impl Utf8Builder {
    #[inline]
    pub fn push(&mut self, alloc: &mut StringAllocator, ch: char) {
        alloc.allocated_utf8.push(ch);
    }

    #[inline]
    pub fn push_str(&mut self, alloc: &mut StringAllocator, s: &str) {
        alloc.allocated_utf8.push_str(s);
    }

    #[inline]
    pub fn finish(self, alloc: &mut StringAllocator) -> Utf8Ref {
        Utf8Ref {
            lo: self.start as u32,
            hi: alloc.allocated_utf8.len() as u32,
        }
    }
}

pub struct Wtf8Builder {
    pub(crate) start: usize,
}

impl Wtf8Builder {
    #[inline]
    pub fn push(&mut self, alloc: &mut StringAllocator, code_point: CodePoint) {
        alloc.allocated_wtf8.push(code_point);
    }

    #[inline]
    pub fn push_char(&mut self, alloc: &mut StringAllocator, c: char) {
        alloc.allocated_wtf8.push_char(c);
    }

    #[inline]
    pub fn push_str(&mut self, alloc: &mut StringAllocator, s: &str) {
        alloc.allocated_wtf8.push_str(s);
    }

    #[inline]
    pub fn finish(self, alloc: &mut StringAllocator) -> Wtf8Ref {
        let end = alloc.allocated_wtf8.len() as u32;
        alloc.allocated_wtf8.push_char('\u{FFFD}'); // Append a separator to avoid two codepoints being merged
        Wtf8Ref::new_ref(self.start as u32, end)
    }

    #[inline]
    pub fn is_empty(&self, alloc: &StringAllocator) -> bool {
        self.start == alloc.allocated_wtf8.len()
    }
}
