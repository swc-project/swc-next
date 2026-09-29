# swc_next_simd

This crate makes the subset of `std::simd` used by this workspace available on
stable Rust. It follows the standard library's API and semantics while portable
SIMD remains unstable.

It uses native SIMD instructions where supported and scalar operations elsewhere,
providing a consistent interface across targets.

Once portable SIMD is stabilized, `std::simd` will be a drop-in replacement for
this compatibility layer. The crate can then re-export the standard library's
types and traits without requiring changes to callers.
