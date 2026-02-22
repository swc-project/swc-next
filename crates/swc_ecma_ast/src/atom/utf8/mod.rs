#[cfg(feature = "compat_atom")]
mod compat;
#[cfg(feature = "compat_atom")]
pub use compat::*;

#[cfg(not(feature = "compat_atom"))]
mod arena;
#[cfg(not(feature = "compat_atom"))]
pub use arena::{OptionalUtf8Ref, Utf8Allocator, Utf8Ref};
