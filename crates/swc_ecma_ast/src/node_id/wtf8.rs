/// A reference to a wtf8 string in the string allocator.
#[derive(Debug, Clone, Copy)]
pub struct Wtf8Ref {
    /// The start pos of the wtf8 string in the string allocator.
    lo: u32,
    /// The end pos of the wtf8 string in the string allocator.
    hi: u32,
}

impl Wtf8Ref {
    pub(crate) const fn new_ref(lo: u32, hi: u32) -> Self {
        Self { lo, hi }
    }

    pub const fn lo(&self) -> u32 {
        self.lo
    }

    pub const fn hi(&self) -> u32 {
        self.hi
    }
}

/// An optimized version of `Option<Wtf8Ref>`
///
/// We regard it as `None` if `OptionalWtf8Ref::hi` is `u32::MAX`.
#[derive(Debug, Clone, Copy, Hash)]
pub struct OptionalWtf8Ref {
    lo: u32,
    hi: u32,
}
impl OptionalWtf8Ref {
    pub(crate) const fn new_ref(lo: u32, hi: u32) -> Self {
        Self { lo, hi }
    }

    pub const fn new_none() -> Self {
        Self {
            lo: 0,
            hi: u32::MAX,
        }
    }

    pub const fn to_option(self) -> Option<Wtf8Ref> {
        if self.hi == u32::MAX {
            return None;
        }

        Some(Wtf8Ref {
            lo: self.lo,
            hi: self.hi,
        })
    }
}

impl From<Wtf8Ref> for OptionalWtf8Ref {
    fn from(value: Wtf8Ref) -> Self {
        Self::new_ref(value.lo, value.hi)
    }
}
