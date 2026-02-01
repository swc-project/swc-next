use std::rc::Rc;

use swc_core::{
    atoms::wtf8::{CodePoint, Wtf8Buf},
    common::BytePos,
};
use swc_experimental_ecma_ast::{Span, StringAllocator, Utf8Ref, Wtf8Ref};

use crate::Lexer;

#[derive(Debug, Clone, Copy)]
pub enum MaybeSubUtf8 {
    Inline((BytePos, BytePos)),
    Alloc(Utf8Ref),
}

impl MaybeSubUtf8 {
    #[inline]
    pub(crate) fn new_from_span(span: Span) -> Self {
        Self::Inline((span.lo, span.hi))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MaybeSubWtf8 {
    Inline((BytePos, BytePos)),
    Alloc(Wtf8Ref),
}

impl MaybeSubWtf8 {
    #[inline]
    pub(crate) fn new_from_source(start: BytePos, end: BytePos) -> Self {
        Self::Inline((start, end))
    }
}

#[derive(Clone)]
pub(crate) struct StringBuilder {
    string_allocator: Rc<StringAllocator>,
    utf8_buf: String,
    wtf8_buf: Wtf8Buf,
}

impl StringBuilder {
    pub(crate) fn new(string_allocator: Rc<StringAllocator>) -> Self {
        Self {
            string_allocator,
            wtf8_buf: Wtf8Buf::default(),
            utf8_buf: String::new(),
        }
    }

    pub(crate) fn alloc_utf8(&mut self) -> Utf8Builder {
        Utf8Builder {
            start: self.utf8_buf.len(),
        }
    }

    #[inline]
    pub fn alloc_wtf8(&mut self) -> Wtf8Builder {
        Wtf8Builder {
            start: self.wtf8_buf.len(),
        }
    }
}

pub struct Utf8Builder {
    start: usize,
}

impl Utf8Builder {
    #[inline]
    pub(crate) fn push(&mut self, sb: &mut StringBuilder, ch: char) {
        sb.utf8_buf.push(ch);
    }

    #[inline]
    pub(crate) fn push_str(&mut self, sb: &mut StringBuilder, s: &str) {
        sb.utf8_buf.push_str(s);
    }

    #[inline]
    pub(crate) fn finish(self, sb: &mut StringBuilder) -> MaybeSubUtf8 {
        MaybeSubUtf8::Alloc(
            sb.string_allocator
                .add_utf8(sb.utf8_buf.drain(self.start..).as_str()),
        )
    }

    #[inline]
    pub(crate) fn is_empty(&self, sb: &StringBuilder) -> bool {
        self.start == sb.utf8_buf.len()
    }
}

pub struct Wtf8Builder {
    start: usize,
}

impl Wtf8Builder {
    #[inline]
    pub(crate) fn push(&mut self, sb: &mut StringBuilder, code_point: CodePoint) {
        sb.wtf8_buf.push(code_point);
    }

    #[inline]
    pub(crate) fn push_char(&mut self, sb: &mut StringBuilder, c: char) {
        sb.wtf8_buf.push_char(c);
    }

    #[inline]
    pub(crate) fn push_str(&mut self, sb: &mut StringBuilder, s: &str) {
        sb.wtf8_buf.push_str(s);
    }

    #[inline]
    pub(crate) fn finish(self, sb: &mut StringBuilder) -> MaybeSubWtf8 {
        let id = sb
            .string_allocator
            .add_wtf8(sb.wtf8_buf.slice(self.start, sb.wtf8_buf.len()));
        sb.wtf8_buf.truncate(self.start);
        MaybeSubWtf8::Alloc(id)
    }

    #[inline]
    pub(crate) fn is_empty(&self, sb: &StringBuilder) -> bool {
        self.start == sb.wtf8_buf.len()
    }
}

impl<'a> Lexer<'a> {
    pub(crate) fn _string_allocator(&self) -> Rc<StringAllocator> {
        self.sb.string_allocator.clone()
    }

    pub(crate) fn get_utf8(&self, maybe: MaybeSubUtf8) -> &str {
        match maybe {
            MaybeSubUtf8::Inline((lo, hi)) => unsafe { self.input.slice(lo, hi) },
            MaybeSubUtf8::Alloc(id) => self.sb.string_allocator.get_utf8(id),
        }
    }

    pub(crate) fn empty_utf8_ref(&self) -> MaybeSubUtf8 {
        MaybeSubUtf8::Alloc(self.sb.string_allocator.empty_utf8_ref())
    }
}
