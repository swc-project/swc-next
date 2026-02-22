#[cfg(feature = "compat_atom")]
mod compat;
#[cfg(feature = "compat_atom")]
pub use compat::*;

#[cfg(not(feature = "compat_atom"))]
mod arena;
#[cfg(not(feature = "compat_atom"))]
pub use arena::{OptionalWtf8Ref, Wtf8Allocator, Wtf8Ref};
