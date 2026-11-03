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

// The vector and mask operations below use `#[inline(always)]`. Size-optimized
// builds such as `opt-level = "z"` otherwise keep them as calls, and the
// scalar backend then passes every 16-byte vector through memory.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Simd<T, const N: usize>([T; N]);

impl<T: Copy, const N: usize> Simd<T, N> {
    pub const LEN: usize = N;

    #[inline(always)]
    pub const fn splat(value: T) -> Self {
        Self([value; N])
    }

    #[inline(always)]
    pub const fn from_array(array: [T; N]) -> Self {
        Self(array)
    }

    #[inline(always)]
    pub fn from_slice(slice: &[T]) -> Self {
        let array = slice
            .first_chunk::<N>()
            .expect("source slice is shorter than the SIMD vector");
        Self(*array)
    }

    #[inline(always)]
    pub const fn to_array(self) -> [T; N] {
        self.0
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Mask<T, const N: usize>(Simd<T, N>);

impl<const N: usize> Mask<i8, N> {
    #[inline(always)]
    pub const fn splat(value: bool) -> Self {
        Self(Simd::splat(if value { -1 } else { 0 }))
    }

    #[inline(always)]
    pub fn from_array(array: [bool; N]) -> Self {
        let lanes = array.map(|value| if value { -1 } else { 0 });
        Self(Simd::from_array(lanes))
    }

    #[inline(always)]
    pub fn to_array(self) -> [bool; N] {
        self.0.to_array().map(|lane| lane != 0)
    }

    /// Converts an integer vector to a mask: 0 is false and -1 is true.
    ///
    /// # Panics
    /// Panics if any element is neither 0 nor -1.
    #[inline(always)]
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
    #[inline(always)]
    #[must_use]
    pub unsafe fn from_simd_unchecked(value: Simd<i8, N>) -> Self {
        Self(value)
    }

    /// Returns the integer vector, with 0 for false and -1 for true.
    #[inline(always)]
    pub fn to_simd(self) -> Simd<i8, N> {
        self.0
    }
}

/// Choose elements from two vectors using a mask.
pub trait Select<T> {
    fn select(self, true_values: T, false_values: T) -> T;
}

impl Select<Simd<u8, 16>> for Mask<i8, 16> {
    #[inline(always)]
    fn select(self, true_values: Simd<u8, 16>, false_values: Simd<u8, 16>) -> Simd<u8, 16> {
        let mask = as_unsigned(self.to_simd().to_array());
        Simd::from_array(backend::select(mask, true_values.0, false_values.0))
    }
}

impl Mask<i8, 16> {
    #[inline(always)]
    pub fn any(self) -> bool {
        let lanes = as_unsigned(self.to_simd().to_array());
        backend::any(lanes)
    }

    /// The first true lane, or `None` when every lane is false.
    #[inline(always)]
    pub fn first_set(self) -> Option<usize> {
        let lanes = as_unsigned(self.to_simd().to_array());
        let lane = backend::first_set(lanes);
        (lane < 16).then_some(lane)
    }
}

impl Not for Mask<i8, 16> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        let lanes = as_unsigned(self.to_simd().to_array());
        let result = as_signed(backend::eq(lanes, [0; 16]));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Self::from_simd_unchecked(Simd::from_array(result)) }
    }
}

impl BitOr for Simd<u8, 16> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, other: Self) -> Self {
        Self(backend::or(self.0, other.0))
    }
}

impl Sub for Simd<u8, 16> {
    type Output = Self;

    /// Lane-wise wrapping subtraction, matching portable SIMD arithmetic.
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Self(backend::sub(self.0, other.0))
    }
}

impl BitAnd for Mask<i8, 16> {
    type Output = Self;

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    fn simd_eq(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::eq(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline(always)]
    fn simd_ne(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::ne(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }
}

impl SimdPartialOrd for Simd<u8, 16> {
    #[inline(always)]
    fn simd_lt(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::lt(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline(always)]
    fn simd_le(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::le(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline(always)]
    fn simd_gt(self, other: Self) -> Self::Mask {
        let lanes = as_signed(backend::gt(self.0, other.0));
        // SAFETY: Byte comparisons produce only 0 or -1 lanes.
        unsafe { Mask::from_simd_unchecked(Simd::from_array(lanes)) }
    }

    #[inline(always)]
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
use scalar as backend;

/// Portable backend that processes each vector as two `u64` words (SWAR).
///
/// Lane `i` is byte `i` of a little-endian load, on every target. It is
/// compiled for tests on every host so that it can be checked against a
/// per-lane reference.
#[cfg(any(
    test,
    not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        all(target_arch = "wasm32", target_feature = "simd128")
    ))
))]
mod scalar {
    /// The high bit of each lane.
    const HIGH: u64 = 0x8080_8080_8080_8080;
    /// The low seven bits of each lane.
    const LOW: u64 = 0x7F7F_7F7F_7F7F_7F7F;

    #[inline(always)]
    fn words(lanes: [u8; 16]) -> [u64; 2] {
        let (low, high) = lanes.split_at(8);
        [
            u64::from_le_bytes(low.try_into().unwrap()),
            u64::from_le_bytes(high.try_into().unwrap()),
        ]
    }

    #[inline(always)]
    fn lanes(words: [u64; 2]) -> [u8; 16] {
        let mut lanes = [0; 16];
        lanes[..8].copy_from_slice(&words[0].to_le_bytes());
        lanes[8..].copy_from_slice(&words[1].to_le_bytes());
        lanes
    }

    #[inline(always)]
    fn map(left: [u8; 16], right: [u8; 16], op: impl Fn(u64, u64) -> u64) -> [u8; 16] {
        let [left_low, left_high] = words(left);
        let [right_low, right_high] = words(right);
        lanes([op(left_low, right_low), op(left_high, right_high)])
    }

    /// Sets the high bit of each nonzero lane. No carry crosses a lane
    /// because `(x & LOW) + LOW` is at most `0xFE` per lane.
    #[inline(always)]
    fn nonzero(word: u64) -> u64 {
        (((word & LOW) + LOW) | word) & HIGH
    }

    /// Expands high bits to `0xFF` lanes; other bits must be clear.
    #[inline(always)]
    fn expand(high_bits: u64) -> u64 {
        (high_bits >> 7) * 0xFF
    }

    /// Sets the high bit of each lane where `left >= right` (unsigned).
    #[inline(always)]
    fn ge_bits(left: u64, right: u64) -> u64 {
        // The high bit of each lane is set when the low seven bits of `left`
        // are not less than those of `right`. No borrow crosses a lane.
        let low_ge = (left | HIGH) - (right & LOW);
        ((left & !right) | (!(left ^ right) & low_ge)) & HIGH
    }

    #[inline(always)]
    pub(super) fn first_set(mask: [u8; 16]) -> usize {
        let [low, high] = words(mask);
        let low = nonzero(low);
        if low != 0 {
            return low.trailing_zeros() as usize / 8;
        }
        let high = nonzero(high);
        if high != 0 {
            return 8 + high.trailing_zeros() as usize / 8;
        }
        16
    }

    #[inline(always)]
    pub(super) fn sub(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| {
            ((left | HIGH) - (right & LOW)) ^ ((left ^ !right) & HIGH)
        })
    }

    #[inline(always)]
    pub(super) fn eq(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| {
            expand(!nonzero(left ^ right) & HIGH)
        })
    }

    #[inline(always)]
    pub(super) fn ne(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| expand(nonzero(left ^ right)))
    }

    #[inline(always)]
    pub(super) fn lt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| {
            expand(!ge_bits(left, right) & HIGH)
        })
    }

    #[inline(always)]
    pub(super) fn le(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| expand(ge_bits(right, left)))
    }

    #[inline(always)]
    pub(super) fn gt(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| {
            expand(!ge_bits(right, left) & HIGH)
        })
    }

    #[inline(always)]
    pub(super) fn ge(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| expand(ge_bits(left, right)))
    }

    #[inline(always)]
    pub(super) fn any(mask: [u8; 16]) -> bool {
        let [low, high] = words(mask);
        low | high != 0
    }

    #[inline(always)]
    pub(super) fn and(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| left & right)
    }

    #[inline(always)]
    pub(super) fn or(left: [u8; 16], right: [u8; 16]) -> [u8; 16] {
        map(left, right, |left, right| left | right)
    }

    /// Every mask lane is `0x00` or `0xFF`.
    #[inline(always)]
    pub(super) fn select(
        mask: [u8; 16],
        true_values: [u8; 16],
        false_values: [u8; 16],
    ) -> [u8; 16] {
        let [mask_low, mask_high] = words(mask);
        let [true_low, true_high] = words(true_values);
        let [false_low, false_high] = words(false_values);
        lanes([
            (mask_low & true_low) | (!mask_low & false_low),
            (mask_high & true_high) | (!mask_high & false_high),
        ])
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn lane_mask(value: bool) -> u8 {
            u8::from(value).wrapping_neg()
        }

        /// Places every `(left, right)` byte pair in every lane position.
        fn for_each_pair_vector(mut check: impl FnMut([u8; 16], [u8; 16])) {
            for chunk in 0..4096_u32 {
                for rotation in 0..16 {
                    let mut left = [0; 16];
                    let mut right = [0; 16];
                    for lane in 0..16 {
                        let pair = chunk * 16 + lane as u32;
                        let position = (lane + rotation) % 16;
                        left[position] = (pair >> 8) as u8;
                        right[position] = pair as u8;
                    }
                    check(left, right);
                }
            }
        }

        fn assert_lanes(
            name: &str,
            left: [u8; 16],
            right: [u8; 16],
            actual: [u8; 16],
            expected: impl Fn(u8, u8) -> u8,
        ) {
            for lane in 0..16 {
                assert_eq!(
                    actual[lane],
                    expected(left[lane], right[lane]),
                    "{name}({:#04x}, {:#04x}) in lane {lane}",
                    left[lane],
                    right[lane],
                );
            }
        }

        #[test]
        fn lane_operations() {
            for_each_pair_vector(|left, right| {
                assert_lanes("eq", left, right, eq(left, right), |l, r| lane_mask(l == r));
                assert_lanes("ne", left, right, ne(left, right), |l, r| lane_mask(l != r));
                assert_lanes("lt", left, right, lt(left, right), |l, r| lane_mask(l < r));
                assert_lanes("le", left, right, le(left, right), |l, r| lane_mask(l <= r));
                assert_lanes("gt", left, right, gt(left, right), |l, r| lane_mask(l > r));
                assert_lanes("ge", left, right, ge(left, right), |l, r| lane_mask(l >= r));
                assert_lanes("sub", left, right, sub(left, right), u8::wrapping_sub);
                assert_lanes("and", left, right, and(left, right), |l, r| l & r);
                assert_lanes("or", left, right, or(left, right), |l, r| l | r);

                let mask = eq(left, right);
                let selected = select(mask, left, right);
                for lane in 0..16 {
                    let expected = if mask[lane] != 0 {
                        left[lane]
                    } else {
                        right[lane]
                    };
                    assert_eq!(selected[lane], expected, "select in lane {lane}");
                }
            });
        }

        #[test]
        fn lane_boundaries() {
            let mut left = [0; 16];
            let mut right = [0; 16];
            for lane in 0..16 {
                left[lane] = [0x00, 0xFF, 0x7F, 0x80][lane % 4];
                right[lane] = [0xFF, 0x00, 0x80, 0x7F][lane % 4];
            }
            for (left, right) in [(left, right), (right, left)] {
                assert_lanes("sub", left, right, sub(left, right), u8::wrapping_sub);
                assert_lanes("lt", left, right, lt(left, right), |l, r| lane_mask(l < r));
                assert_lanes("le", left, right, le(left, right), |l, r| lane_mask(l <= r));
                assert_lanes("gt", left, right, gt(left, right), |l, r| lane_mask(l > r));
                assert_lanes("ge", left, right, ge(left, right), |l, r| lane_mask(l >= r));
            }
        }

        #[test]
        fn masks() {
            assert_eq!(first_set([0; 16]), 16);
            assert!(!any([0; 16]));

            for lane in 0..16 {
                let mut mask = [0; 16];
                mask[lane] = 0xFF;
                assert_eq!(first_set(mask), lane);
                assert!(any(mask));

                mask[15] = 0xFF;
                assert_eq!(first_set(mask), lane);
            }

            let mut mask = [0; 16];
            mask[7] = 0xFF;
            mask[8] = 0xFF;
            assert_eq!(first_set(mask), 7);
            mask[7] = 0;
            assert_eq!(first_set(mask), 8);
        }
    }
}
