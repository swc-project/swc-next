//! Temporary portable-SIMD compatibility layer for `swc_next_type_strip`.
//!
//! This crate strips erasable TypeScript syntax while preserving UTF-16 source
//! positions. The types and traits in this module provide the `std::simd`-shaped
//! subset used to accelerate that blanking on supported targets without exposing
//! architecture-specific intrinsics to the rest of the crate.
//!
//! Once portable SIMD is stable in the Rust standard library, this module is
//! intended to be removed and its imports replaced with `std::simd`, leaving the
//! blanking algorithm unchanged.

use std::marker::PhantomData;
use std::ops::{BitAnd, BitOr};

pub(crate) const AVAILABLE: bool = cfg!(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    all(target_arch = "wasm32", target_feature = "simd128")
));

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Simd<T, const N: usize>([T; N]);

impl<T: Copy, const N: usize> Simd<T, N> {
    pub(crate) const LEN: usize = N;

    #[inline]
    pub(crate) const fn splat(value: T) -> Self {
        Self([value; N])
    }

    #[inline]
    pub(crate) const fn from_array(array: [T; N]) -> Self {
        Self(array)
    }

    #[inline]
    pub(crate) fn from_slice(slice: &[T]) -> Self {
        let array = slice
            .first_chunk::<N>()
            .expect("source slice is shorter than the SIMD vector");
        Self(*array)
    }

    #[inline]
    pub(crate) const fn to_array(self) -> [T; N] {
        self.0
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Mask<T, const N: usize> {
    lanes: [u8; N],
    element: PhantomData<T>,
}

impl<T, const N: usize> Mask<T, N> {
    #[inline]
    fn from_array(lanes: [u8; N]) -> Self {
        Self {
            lanes,
            element: PhantomData,
        }
    }
}

impl Mask<i8, 16> {
    #[inline]
    pub(crate) fn select(
        self,
        true_values: Simd<u8, 16>,
        false_values: Simd<u8, 16>,
    ) -> Simd<u8, 16> {
        Simd::from_array(backend::select(self.lanes, true_values.0, false_values.0))
    }
}

impl<T> Mask<T, 16> {
    #[inline]
    pub(crate) fn any(self) -> bool {
        backend::any(self.lanes)
    }
}

impl<T> BitAnd for Mask<T, 16> {
    type Output = Self;

    #[inline]
    fn bitand(self, other: Self) -> Self::Output {
        Self::from_array(backend::and(self.lanes, other.lanes))
    }
}

impl<T> BitOr for Mask<T, 16> {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self::Output {
        Self::from_array(backend::or(self.lanes, other.lanes))
    }
}

#[allow(dead_code)]
pub(crate) trait SimdPartialEq {
    type Mask;

    fn simd_eq(self, other: Self) -> Self::Mask;
    fn simd_ne(self, other: Self) -> Self::Mask;
}

#[allow(dead_code)]
pub(crate) trait SimdPartialOrd: SimdPartialEq {
    fn simd_lt(self, other: Self) -> Self::Mask;
    fn simd_le(self, other: Self) -> Self::Mask;
    fn simd_gt(self, other: Self) -> Self::Mask;
    fn simd_ge(self, other: Self) -> Self::Mask;
}

impl SimdPartialEq for Simd<u8, 16> {
    type Mask = Mask<i8, 16>;

    #[inline]
    fn simd_eq(self, other: Self) -> Self::Mask {
        Mask::from_array(backend::eq(self.0, other.0))
    }

    #[inline]
    fn simd_ne(self, other: Self) -> Self::Mask {
        Mask::from_array(backend::ne(self.0, other.0))
    }
}

impl SimdPartialOrd for Simd<u8, 16> {
    #[inline]
    fn simd_lt(self, other: Self) -> Self::Mask {
        Mask::from_array(backend::lt(self.0, other.0))
    }

    #[inline]
    fn simd_le(self, other: Self) -> Self::Mask {
        Mask::from_array(backend::le(self.0, other.0))
    }

    #[inline]
    fn simd_gt(self, other: Self) -> Self::Mask {
        Mask::from_array(backend::gt(self.0, other.0))
    }

    #[inline]
    fn simd_ge(self, other: Self) -> Self::Mask {
        Mask::from_array(backend::ge(self.0, other.0))
    }
}

#[allow(non_camel_case_types)]
pub(crate) type u8x16 = Simd<u8, 16>;

#[allow(non_camel_case_types)]
pub(crate) type mask8x16 = Mask<i8, 16>;

pub(crate) mod prelude {
    #[allow(unused_imports)]
    pub(crate) use super::{SimdPartialEq, SimdPartialOrd};
}

#[cfg(target_arch = "x86_64")]
mod backend {
    use std::arch::x86_64::{
        __m128i, _mm_and_si128, _mm_andnot_si128, _mm_cmpeq_epi8, _mm_cmpgt_epi8,
        _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8, _mm_xor_si128,
    };

    macro_rules! sse2 {
        ($expression:expr) => {{
            // SAFETY: SSE2 is mandatory on x86-64.
            unsafe { $expression }
        }};
    }

    #[inline(always)]
    fn vector(value: [u8; 16]) -> __m128i {
        // SAFETY: `__m128i` and `[u8; 16]` have identical sizes.
        unsafe { std::mem::transmute(value) }
    }

    #[inline(always)]
    fn array(value: __m128i) -> [u8; 16] {
        // SAFETY: `__m128i` and `[u8; 16]` have identical sizes.
        unsafe { std::mem::transmute(value) }
    }

    #[inline(always)]
    fn unsigned(value: __m128i) -> __m128i {
        sse2!(_mm_xor_si128(value, _mm_set1_epi8(i8::MIN)))
    }

    #[inline(always)]
    fn not(value: __m128i) -> __m128i {
        sse2!(_mm_xor_si128(value, _mm_set1_epi8(-1)))
    }

    #[inline(always)]
    pub(super) fn eq(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(sse2!(_mm_cmpeq_epi8(vector(left), vector(right))))
    }

    #[inline(always)]
    pub(super) fn ne(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(not(sse2!(_mm_cmpeq_epi8(vector(left), vector(right)))))
    }

    #[inline(always)]
    pub(super) fn lt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(sse2!(_mm_cmpgt_epi8(
            unsigned(vector(right)),
            unsigned(vector(left)),
        )))
    }

    #[inline(always)]
    pub(super) fn le(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(not(sse2!(_mm_cmpgt_epi8(
            unsigned(vector(left)),
            unsigned(vector(right)),
        ))))
    }

    #[inline(always)]
    pub(super) fn gt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(sse2!(_mm_cmpgt_epi8(
            unsigned(vector(left)),
            unsigned(vector(right)),
        )))
    }

    #[inline(always)]
    pub(super) fn ge(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(not(sse2!(_mm_cmpgt_epi8(
            unsigned(vector(right)),
            unsigned(vector(left)),
        ))))
    }

    #[inline(always)]
    pub(super) fn any(mask: [u8; 16]) -> bool {
        sse2!(_mm_movemask_epi8(vector(mask))) != 0
    }

    #[inline(always)]
    pub(super) fn and(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(sse2!(_mm_and_si128(vector(left), vector(right))))
    }

    #[inline(always)]
    pub(super) fn or(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(sse2!(_mm_or_si128(vector(left), vector(right))))
    }

    #[inline(always)]
    pub(super) fn select(
        mask: [u8; 16],
        true_values: [u8; 16],
        false_values: [u8; 16],
    ) -> [u8; 16] {
        let mask = vector(mask);
        let selected = sse2!({
            let selected_true = _mm_and_si128(mask, vector(true_values));
            let selected_false = _mm_andnot_si128(mask, vector(false_values));
            _mm_or_si128(selected_true, selected_false)
        });
        array(selected)
    }
}

#[cfg(target_arch = "aarch64")]
mod backend {
    use std::arch::aarch64::{
        uint8x16_t, vandq_u8, vbslq_u8, vceqq_u8, vcgeq_u8, vcgtq_u8, vcleq_u8, vcltq_u8,
        vmaxvq_u8, vmvnq_u8, vorrq_u8,
    };

    #[inline(always)]
    fn vector(value: [u8; 16]) -> uint8x16_t {
        // SAFETY: `uint8x16_t` and `[u8; 16]` have identical sizes.
        unsafe { std::mem::transmute(value) }
    }

    #[inline(always)]
    fn array(value: uint8x16_t) -> [u8; 16] {
        // SAFETY: `uint8x16_t` and `[u8; 16]` have identical sizes.
        unsafe { std::mem::transmute(value) }
    }

    #[inline(always)]
    pub(super) fn eq(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vceqq_u8(vector(left), vector(right)) })
    }

    #[inline(always)]
    pub(super) fn ne(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vmvnq_u8(vceqq_u8(vector(left), vector(right))) })
    }

    #[inline(always)]
    pub(super) fn lt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vcltq_u8(vector(left), vector(right)) })
    }

    #[inline(always)]
    pub(super) fn le(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vcleq_u8(vector(left), vector(right)) })
    }

    #[inline(always)]
    pub(super) fn gt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vcgtq_u8(vector(left), vector(right)) })
    }

    #[inline(always)]
    pub(super) fn ge(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vcgeq_u8(vector(left), vector(right)) })
    }

    #[inline(always)]
    pub(super) fn any(mask: [u8; 16]) -> bool {
        // SAFETY: NEON is mandatory on AArch64.
        unsafe { vmaxvq_u8(vector(mask)) != 0 }
    }

    #[inline(always)]
    pub(super) fn and(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vandq_u8(vector(left), vector(right)) })
    }

    #[inline(always)]
    pub(super) fn or(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vorrq_u8(vector(left), vector(right)) })
    }

    #[inline(always)]
    pub(super) fn select(
        mask: [u8; 16],
        true_values: [u8; 16],
        false_values: [u8; 16],
    ) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { vbslq_u8(vector(mask), vector(true_values), vector(false_values)) })
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
mod backend {
    use std::arch::wasm32::{
        u8x16_eq, u8x16_ge, u8x16_gt, u8x16_le, u8x16_lt, v128, v128_and, v128_any_true,
        v128_bitselect, v128_not, v128_or,
    };

    #[inline(always)]
    fn vector(value: [u8; 16]) -> v128 {
        // SAFETY: `v128` and `[u8; 16]` have identical sizes.
        unsafe { std::mem::transmute(value) }
    }

    #[inline(always)]
    fn array(value: v128) -> [u8; 16] {
        // SAFETY: `v128` and `[u8; 16]` have identical sizes.
        unsafe { std::mem::transmute(value) }
    }

    #[inline(always)]
    pub(super) fn eq(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(u8x16_eq(vector(left), vector(right)))
    }

    #[inline(always)]
    pub(super) fn ne(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(v128_not(u8x16_eq(vector(left), vector(right))))
    }

    #[inline(always)]
    pub(super) fn lt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(u8x16_lt(vector(left), vector(right)))
    }

    #[inline(always)]
    pub(super) fn le(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(u8x16_le(vector(left), vector(right)))
    }

    #[inline(always)]
    pub(super) fn gt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(u8x16_gt(vector(left), vector(right)))
    }

    #[inline(always)]
    pub(super) fn ge(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(u8x16_ge(vector(left), vector(right)))
    }

    #[inline(always)]
    pub(super) fn any(mask: [u8; 16]) -> bool {
        v128_any_true(vector(mask))
    }

    #[inline(always)]
    pub(super) fn and(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(v128_and(vector(left), vector(right)))
    }

    #[inline(always)]
    pub(super) fn or(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(v128_or(vector(left), vector(right)))
    }

    #[inline(always)]
    pub(super) fn select(
        mask: [u8; 16],
        true_values: [u8; 16],
        false_values: [u8; 16],
    ) -> [u8; 16] {
        array(v128_bitselect(
            vector(true_values),
            vector(false_values),
            vector(mask),
        ))
    }
}

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    all(target_arch = "wasm32", target_feature = "simd128")
)))]
mod backend {
    #[inline(always)]
    pub(super) fn eq(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| u8::from(left[index] == right[index]).wrapping_neg())
    }

    #[inline(always)]
    pub(super) fn ne(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| u8::from(left[index] != right[index]).wrapping_neg())
    }

    #[inline(always)]
    pub(super) fn lt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| u8::from(left[index] < right[index]).wrapping_neg())
    }

    #[inline(always)]
    pub(super) fn le(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| u8::from(left[index] <= right[index]).wrapping_neg())
    }

    #[inline(always)]
    pub(super) fn gt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| u8::from(left[index] > right[index]).wrapping_neg())
    }

    #[inline(always)]
    pub(super) fn ge(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| u8::from(left[index] >= right[index]).wrapping_neg())
    }

    #[inline(always)]
    pub(super) fn any(mask: [u8; 16]) -> bool {
        mask.into_iter().any(|lane| lane != 0)
    }

    #[inline(always)]
    pub(super) fn and(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| left[index] & right[index])
    }

    #[inline(always)]
    pub(super) fn or(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| left[index] | right[index])
    }

    #[inline(always)]
    pub(super) fn select(
        mask: [u8; 16],
        true_values: [u8; 16],
        false_values: [u8; 16],
    ) -> [u8; 16] {
        std::array::from_fn(|index| {
            if mask[index] != 0 {
                true_values[index]
            } else {
                false_values[index]
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::prelude::SimdPartialOrd;
    use super::{mask8x16, u8x16};

    #[test]
    fn comparisons_and_masks_select_bytes() {
        let values = u8x16::from_array(*b"abcdefghijklmno\n");
        let control_whitespace: mask8x16 =
            values.simd_ge(u8x16::splat(b'\t')) & values.simd_le(u8x16::splat(b'\r'));
        let selected = control_whitespace
            .select(values, u8x16::splat(b' '))
            .to_array();

        assert!(control_whitespace.any());
        assert_eq!(selected, *b"               \n");
    }
}
