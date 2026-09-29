//! Lane-wise comparison traits, matching `std::simd::cmp`.

pub trait SimdPartialEq {
    type Mask;

    fn simd_eq(self, other: Self) -> Self::Mask;
    fn simd_ne(self, other: Self) -> Self::Mask;
}

pub trait SimdPartialOrd: SimdPartialEq {
    fn simd_lt(self, other: Self) -> Self::Mask;
    fn simd_le(self, other: Self) -> Self::Mask;
    fn simd_gt(self, other: Self) -> Self::Mask;
    fn simd_ge(self, other: Self) -> Self::Mask;
}
