#[cfg(feature = "compat_atom")]
mod compat;
#[cfg(feature = "compat_atom")]
pub use compat::*;

#[cfg(not(feature = "compat_atom"))]
mod intern;
#[cfg(not(feature = "compat_atom"))]
pub use intern::{OptionalUtf8Ref, Utf8Allocator, Utf8Ref};
