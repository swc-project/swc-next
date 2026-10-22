use std::num::NonZeroU32;

use oxc_index::Idx;

use crate::SourceType;

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
    /// Shares its numeric ID with [`Self::COMMONJS`].
    pub const MODULE: Self = Self(NonZeroU32::new(2).unwrap());

    /// The function wrapper scope, present for CommonJS source types.
    /// Shares its numeric ID with [`Self::MODULE`].
    pub const COMMONJS: Self = Self::MODULE;

    /// The program's top-level scope for the resolved source type.
    /// Returns [`Self::ROOT`] for scripts, [`Self::MODULE`] for ES modules,
    /// or [`Self::COMMONJS`] for CommonJS.
    ///
    /// # Panics
    /// Panics for [`SourceType::Unambiguous`]; use the parsed AST's source type.
    pub const fn program(source_type: SourceType) -> Self {
        match source_type {
            SourceType::Script => Self::ROOT,
            SourceType::Module => Self::MODULE,
            SourceType::CommonJs => Self::COMMONJS,
            SourceType::Unambiguous => {
                panic!("program scope requires a resolved source type")
            }
        }
    }

    pub fn raw(self) -> u32 {
        self.0.get()
    }
}
