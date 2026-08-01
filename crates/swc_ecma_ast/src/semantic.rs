use std::num::NonZeroU32;

use oxc_index::Idx;

oxc_index::define_nonmax_u32_index_type! {
    pub struct SymbolId;
}

oxc_index::define_nonmax_u32_index_type! {
    /// Identifier for a semantic reference.
    pub struct ReferenceId;
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
    /// The global scope at the root of every semantic scope tree.
    pub const ROOT: Self = Self(NonZeroU32::MIN);

    /// The module wrapper scope, present for module source types.
    pub const MODULE: Self = Self(NonZeroU32::new(2).unwrap());

    pub fn raw(self) -> u32 {
        self.0.get()
    }
}
