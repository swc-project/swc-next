use std::num::NonZeroU32;

use oxc_index::Idx;

oxc_index::define_nonmax_u32_index_type! {
    pub struct SymbolId;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopeId(pub(crate) NonZeroU32);

impl Idx for ScopeId {
    const MAX: usize = (u32::MAX - 1) as usize;

    unsafe fn from_usize_unchecked(idx: usize) -> Self {
        unsafe { Self(NonZeroU32::new_unchecked(idx as u32 + 1)) }
    }

    fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}

impl ScopeId {
    pub fn raw(self) -> u32 {
        self.0.get()
    }
}
