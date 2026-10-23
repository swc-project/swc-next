//! A stable Rust implementation of the `std::simd` subset used by this workspace.
//!
//! This crate follows the standard library's API and semantics so that, once
//! portable SIMD is stabilized, `std::simd` can serve as a drop-in replacement.
//!
//! The optional `memchr` module extends the `memchr` crate with searches for any
//! of four bytes.

pub mod cmp;

#[cfg(feature = "memchr")]
pub mod memchr;

use cmp::{SimdPartialEq, SimdPartialOrd};
use std::ops::{BitAnd, BitOr, Not, Sub};

/// Reinterprets mask lanes as bytes. Unlike `array::map`, this compiles to no code.
#[inline(always)]
fn as_unsigned(lanes: [i8; 16]) -> [u8; 16] {
    // SAFETY: `i8` and `u8` have the same size and every bit pattern is valid for both.
    unsafe { std::mem::transmute(lanes) }
}

/// Reinterprets bytes as mask lanes. Unlike `array::map`, this compiles to no code.
#[inline(always)]
fn as_signed(lanes: [u8; 16]) -> [i8; 16] {
    // SAFETY: `i8` and `u8` have the same size and every bit pattern is valid for both.
    unsafe { std::mem::transmute(lanes) }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Simd<T, const N: usize>([T; N]);

impl<T: Copy, const N: usize> Simd<T, N> {
    pub const LEN: usize = N;

    #[inline]
    pub const fn splat(value: T) -> Self {
        Self([value; N])
    }

    #[inline]
    pub const fn from_array(array: [T; N]) -> Self {
        Self(array)
    }

    #[inline]
    pub fn from_slice(slice: &[T]) -> Self {
        let array = slice
            .first_chunk::<N>()
            .expect("source slice is shorter than the SIMD vector");
        Self(*array)
    }

    #[inline]
    pub const fn to_array(self) -> [T; N] {
        self.0
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Mask<T, const N: usize>(Simd<T, N>);

impl<const N: usize> Mask<i8, N> {
    #[inline]
    pub const fn splat(value: bool) -> Self {
        Self(Simd::splat(if value { -1 } else { 0 }))
    }

    #[inline]
    pub fn from_array(array: [bool; N]) -> Self {
        let lanes = array.map(|value| if value { -1 } else { 0 });
        Self(Simd::from_array(lanes))
    }

    #[inline]
    pub fn to_array(self) -> [bool; N] {
        self.0.to_array().map(|lane| lane != 0)
    }

    /// Converts an integer vector to a mask: 0 is false and -1 is true.
    ///
    /// # Panics
    /// Panics if any element is neither 0 nor -1.
    #[inline]
    #[track_caller]
    #[must_use]
    pub fn from_simd(value: Simd<i8, N>) -> Self {
        assert!(
            value.0.iter().all(|&lane| lane == 0 || lane == -1),
            "all mask values must be either 0 or -1"
        );
        // SAFETY: Every lane was checked above.
        unsafe { Self::from_simd_unchecked(value) }
    }

    /// Converts an integer vector to a mask without checking its elements.
    ///
    /// # Safety
    /// Every element must be either 0 or -1.
    #[inline]
    #[must_use]
    pub unsafe fn from_simd_unchecked(value: Simd<i8, N>) -> Self {
        Self(value)
    }

    /// Returns the integer vector, with 0 for false and -1 for true.
    #[inline]
    pub fn to_simd(self) -> Simd<i8, N> {
        self.0
    }
}

/// Choose elements from two vectors using a mask.
pub trait Select<T> {
    fn select(self, true_values: T, false_values: T) -> T;
}

impl Select<Simd<u8, 16>> for Mask<i8, 16> {
    #[inline]
    fn select(self, true_values: Simd<u8, 16>, false_values: Simd<u8, 16>) -> Simd<u8, 16> {
        let mask = as_unsigned(self.to_simd().to_array());
        Simd::from_array(backend::select(mask, true_values.0, false_values.0))
    }
}

impl Mask<i8, 16> {
    #[inline]
    pub fn any(self) -> bool {
        let lanes = as_unsigned(self.to_simd().to_array());
        backend::any(lanes)
    }

    /// The first true lane, or `None` when every lane is false.
    #[inline]
    pub fn first_set(self) -> Option<usize> {
        let lanes = as_unsigned(self.to_simd().to_array());
        let lane = backend::first_set(lanes);
        (lane < 16).then_some(lane)
    }
}

impl Not for Mask<i8, 16> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        let lanes = as_unsigned(self.to_simd().to_array());
        let result = as_signed(backend::eq(lanes, [0; 16]));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Self::from_simd_unchecked(Simd::from_array(result)) }
    }
}

impl BitOr for Simd<u8, 16> {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        Self(backend::or(self.0, other.0))
    }
}

impl Sub for Simd<u8, 16> {
    type Output = Self;

    /// Lane-wise wrapping subtraction, matching portable SIMD arithmetic.
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(backend::sub(self.0, other.0))
    }
}

impl BitAnd for Mask<i8, 16> {
    type Output = Self;

    #[inline]
    fn bitand(self, other: Self) -> Self::Output {
        let left = as_unsigned(self.to_simd().to_array());
        let right = as_unsigned(other.to_simd().to_array());
        let result = as_signed(backend::and(left, right));
        // SAFETY: Bitwise operations preserve the 0/-1 mask invariant.
        unsafe { Self::from_simd_unchecked(Simd::from_array(result)) }
    }
}

impl BitOr for Mask<i8, 16> {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self::Output {
        let left = as_unsigned(self.to_simd().to_array());
        let right = as_unsigned(other.to_simd().to_array());
        let result = as_signed(backend::or(left, right));
        // SAFETY: Bitwise operations preserve the 0/-1 mask invariant.
        unsafe { Self::from_simd_unchecked(Simd::from_array(result)) }
    }
}

impl SimdPartialEq for Simd<u8, 16> {
    type Mask = Mask<i8, 16>;

    #[inline]
    fn simd_eq(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::eq(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline]
    fn simd_ne(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::ne(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }
}

impl SimdPartialOrd for Simd<u8, 16> {
    #[inline]
    fn simd_lt(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::lt(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline]
    fn simd_le(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::le(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline]
    fn simd_gt(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::gt(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline]
    fn simd_ge(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::ge(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }
}

#[allow(non_camel_case_types)]
pub type u8x16 = Simd<u8, 16>;

#[allow(non_camel_case_types)]
pub type mask8x16 = Mask<i8, 16>;

pub mod prelude {
    pub use super::cmp::{SimdPartialEq, SimdPartialOrd};
    pub use super::{Mask, Select, Simd, mask8x16, u8x16};
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
    pub(super) fn first_set(mask: [u8; 16]) -> usize {
        let bits = sse2!(_mm_movemask_epi8(vector(mask))) as u16;
        bits.trailing_zeros() as usize
    }

    #[inline(always)]
    pub(super) fn sub(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(sse2!(std::arch::x86_64::_mm_sub_epi8(
            vector(left),
            vector(right)
        )))
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
    pub(super) fn first_set(mask: [u8; 16]) -> usize {
        use std::arch::aarch64::{vreinterpretq_u16_u8, vshrn_n_u16};
        // Pack four bits per lane with SHRN: AArch64 has no byte movemask.
        // SAFETY: NEON is mandatory on AArch64; the shift is in range.
        let nibbles = unsafe { vshrn_n_u16::<4>(vreinterpretq_u16_u8(vector(mask))) };
        // SAFETY: both representations are eight bytes. Reading in native
        // byte order below preserves lane order on big-endian targets too.
        let bytes: [u8; 8] = unsafe { std::mem::transmute(nibbles) };
        let bits = u64::from_ne_bytes(bytes);
        if cfg!(target_endian = "little") {
            bits.trailing_zeros() as usize / 4
        } else {
            bits.leading_zeros() as usize / 4
        }
    }

    #[inline(always)]
    pub(super) fn sub(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        // SAFETY: NEON is mandatory on AArch64.
        array(unsafe { std::arch::aarch64::vsubq_u8(vector(left), vector(right)) })
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
    pub(super) fn first_set(mask: [u8; 16]) -> usize {
        let bits = std::arch::wasm32::i8x16_bitmask(vector(mask)) as u16;
        bits.trailing_zeros() as usize
    }

    #[inline(always)]
    pub(super) fn sub(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        array(std::arch::wasm32::i8x16_sub(vector(left), vector(right)))
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
    pub(super) fn first_set(mask: [u8; 16]) -> usize {
        mask.iter().position(|&lane| lane != 0).unwrap_or(16)
    }

    #[inline(always)]
    pub(super) fn sub(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        std::array::from_fn(|index| left[index].wrapping_sub(right[index]))
    }

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
