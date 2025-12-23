use swc_core::common::BytePos;

/// Optimized [swc_core::common::input::StringInput]
#[derive(Clone)]
pub struct StringSource<'a> {
    source: &'a [u8],
    /// One-based index that should always be on a UTF-8 character boundary, and never after end of source
    cur: BytePos,
}

impl<'a> StringSource<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            cur: 0usize.byte_pos(),
        }
    }
}

impl<'a> StringSource<'a> {
    #[inline]
    /// Note that `start` and `end` is one-based index.
    /// # Safety
    /// 1. `start` is before or equal to `end`.
    /// 2. `start` and `end` must be on UTF-8 character boundary, or come from [Self::cur_pos].
    pub(crate) unsafe fn slice(&self, start: BytePos, end: BytePos) -> &'a str {
        debug_assert!(start <= end);
        debug_assert!(
            end.index() <= self.source.len(),
            "end({}) index out of bounds({})",
            end.index(),
            self.source.len()
        );
        debug_assert!(
            start.index() == self.source.len() || !is_utf8_cont_byte(self.source[start.index()])
        );
        debug_assert!(
            end.index() == self.source.len() || !is_utf8_cont_byte(self.source[end.index()])
        );

        unsafe { str::from_utf8_unchecked(&self.source[start.index()..end.index()]) }
    }

    #[inline]
    /// # Safety
    /// 1. `start` is before or equal to `cur`.
    /// 2. `start` must be on UTF-8 character boundary, or come from [Self::cur_pos].
    pub(crate) unsafe fn slice_to_cur(&self, start: BytePos) -> &'a str {
        unsafe { self.slice(start, self.cur) }
    }

    #[inline]
    /// This is a safe function because [StringSource] always hold a valid `cur`
    pub(crate) fn as_str(&self) -> &str {
        debug_assert!(self.cur.index() <= self.source.len());
        debug_assert!(
            self.cur.index() == self.source.len()
                || !is_utf8_cont_byte(self.source[self.cur.index()])
        );

        unsafe { str::from_utf8_unchecked(&self.source[self.cur.index()..]) }
    }

    #[inline]
    pub(crate) fn cur_pos(&self) -> BytePos {
        self.cur
    }

    #[inline]
    pub(crate) fn end_pos(&self) -> BytePos {
        self.source.len().byte_pos()
    }

    #[inline]
    pub(crate) fn is_start(&self) -> bool {
        self.cur.index() == 0
    }

    #[inline]
    pub(crate) fn peek_char(&self) -> Option<char> {
        let byte = self.peek()?;
        if byte.is_ascii() {
            Some(byte as char)
        } else {
            let mut iter = self.as_str().chars();
            iter.next()
        }
    }

    #[inline]
    pub(crate) fn peek(&self) -> Option<u8> {
        self.source.get(self.cur.index()).copied()
    }

    #[inline]
    pub(crate) fn peek_2(&self) -> Option<u8> {
        self.source.get(self.cur.index() + 1).copied()
    }

    #[inline]
    pub(crate) fn peek_3(&self) -> Option<u8> {
        self.source.get(self.cur.index() + 2).copied()
    }

    #[inline]
    pub(crate) fn peek_ascii(&self) -> Option<u8> {
        let byte = self.source.get(self.cur.index()).copied()?;
        if byte.is_ascii() {
            return Some(byte);
        }
        None
    }

    #[inline]
    pub(crate) fn eat_byte(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            unsafe { self.bump_bytes(1) };
            true
        } else {
            false
        }
    }

    #[inline]
    pub(crate) unsafe fn bump_bytes(&mut self, n: usize) {
        unsafe {
            // Safety: We only proceed, not go back.
            self.reset_to(self.cur + BytePos(n as u32));
        }
    }

    #[inline]
    /// # Safety
    /// 1. `to` must be less than `self.source.len()` on UTF-8 charater boundary.
    pub(crate) unsafe fn reset_to(&mut self, to: BytePos) {
        debug_assert!(to.index() <= self.source.len());
        debug_assert!(
            to.index() == self.source.len() || !is_utf8_cont_byte(self.source[to.index()]),
        );
        self.cur = to;
    }
}

/// Return if byte is a UTF-8 continuation byte.
#[inline]
const fn is_utf8_cont_byte(byte: u8) -> bool {
    // 0x80 - 0xBF are continuation bytes i.e. not 1st byte of a UTF-8 character sequence
    byte >= 0x80 && byte < 0xC0
}

trait BytePosExt {
    fn index(self) -> usize;
}

impl BytePosExt for BytePos {
    #[inline]
    fn index(self) -> usize {
        (self.0 as usize).saturating_sub(1)
    }
}

trait IndexExt {
    fn byte_pos(self) -> BytePos;
}

impl IndexExt for usize {
    fn byte_pos(self) -> BytePos {
        BytePos(self as u32 + 1)
    }
}
