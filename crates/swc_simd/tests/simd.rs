use swc_next_simd::prelude::{SimdPartialEq, SimdPartialOrd};
use swc_next_simd::{Select, Simd, mask8x16, u8x16};

#[test]
fn vector_conversions() {
    let values = std::array::from_fn(|lane| lane as u8);
    assert_eq!(u8x16::from_array(values).to_array(), values);
    assert_eq!(u8x16::splat(42).to_array(), [42; 16]);
    assert_eq!(u8x16::from_slice(&values).to_array(), values);

    let source: [u8; 32] = std::array::from_fn(|lane| lane as u8);
    assert_eq!(
        u8x16::from_slice(&source[3..]).to_array(),
        std::array::from_fn(|lane| (lane + 3) as u8)
    );
}

#[test]
#[should_panic]
fn from_slice_too_short() {
    let _ = u8x16::from_slice(&[0; 15]);
}

// Follows portable-simd's masks.rs roundtrip_int_conversion test.
#[test]
fn roundtrip_int_conversion() {
    let values = [
        true, false, false, true, false, false, true, false, true, true, false, false, false,
        false, false, true,
    ];
    let mask = mask8x16::from_array(values);
    let int = mask.to_simd();
    assert_eq!(mask.to_array(), values);
    assert_eq!(
        int.to_array(),
        [-1, 0, 0, -1, 0, 0, -1, 0, -1, -1, 0, 0, 0, 0, 0, -1]
    );
    assert_eq!(mask8x16::from_simd(int).to_array(), values);

    // SAFETY: The integer vector came from a mask.
    let mask = unsafe { mask8x16::from_simd_unchecked(int) };
    assert_eq!(mask.to_array(), values);
    assert_eq!(mask8x16::splat(false).to_array(), [false; 16]);
    assert_eq!(mask8x16::splat(true).to_array(), [true; 16]);
}

#[test]
#[should_panic]
fn invalid_mask() {
    let _ = mask8x16::from_simd(Simd::splat(1));
}

// Like portable-simd's first_set test, exhaust the mask bit patterns.
// Construct through from_array because from_bitmask is outside our subset.
#[test]
fn mask_reductions() {
    for bitmask in 0..=u16::MAX {
        let values = std::array::from_fn(|lane| bitmask & (1 << lane) != 0);
        let mask = mask8x16::from_array(values);
        let expected = if bitmask == 0 {
            None
        } else {
            Some(bitmask.trailing_zeros() as usize)
        };
        assert_eq!(mask.first_set(), expected);
        assert_eq!(mask.any(), bitmask != 0);
    }
}

// Compare each lane with scalar operations, as in portable-simd's
// mask_ops_impl and ops_macros tests.
#[test]
fn mask_operations() {
    let left = std::array::from_fn(|lane| lane % 2 == 0);
    let right = std::array::from_fn(|lane| lane % 4 < 2);
    let a = mask8x16::from_array(left);
    let b = mask8x16::from_array(right);
    assert_eq!(
        (a & b).to_array(),
        std::array::from_fn(|lane| left[lane] & right[lane])
    );
    assert_eq!(
        (a | b).to_array(),
        std::array::from_fn(|lane| left[lane] | right[lane])
    );
    assert_eq!((!a).to_array(), left.map(|value| !value));

    let true_values = std::array::from_fn(|lane| lane as u8);
    let false_values = std::array::from_fn(|lane| u8::MAX - lane as u8);
    let selected = a.select(
        u8x16::from_array(true_values),
        u8x16::from_array(false_values),
    );
    assert_eq!(
        selected.to_array(),
        std::array::from_fn(|lane| if left[lane] {
            true_values[lane]
        } else {
            false_values[lane]
        })
    );
}

#[test]
fn unsigned_byte_operations() {
    for left in 0..=u8::MAX {
        for right in (0..=u8::MAX).step_by(16) {
            let values = std::array::from_fn(|lane| right + lane as u8);
            let a = u8x16::splat(left);
            let b = u8x16::from_array(values);
            assert_eq!(a.simd_eq(b).to_array(), values.map(|right| left == right));
            assert_eq!(a.simd_ne(b).to_array(), values.map(|right| left != right));
            assert_eq!(a.simd_lt(b).to_array(), values.map(|right| left < right));
            assert_eq!(a.simd_le(b).to_array(), values.map(|right| left <= right));
            assert_eq!(a.simd_gt(b).to_array(), values.map(|right| left > right));
            assert_eq!(a.simd_ge(b).to_array(), values.map(|right| left >= right));
            assert_eq!(
                (a - b).to_array(),
                values.map(|right| left.wrapping_sub(right))
            );
            assert_eq!((a | b).to_array(), values.map(|right| left | right));
        }
    }
}

#[cfg(feature = "memchr")]
#[test]
fn byte_searches() {
    use swc_next_simd::memchr::memchr4;

    assert_eq!(memchr4(b'w', b'x', b'y', b'z', b""), None);
    assert_eq!(memchr4(b'w', b'x', b'y', b'z', b"abzyxw"), Some(2));
    assert_eq!(memchr4(b'x', b'x', b'x', b'x', b"abx"), Some(2));

    // Exercise every alignment, vector boundary, and tail position.
    let mut source = [0; 80];
    for start in 0..16 {
        for len in 0..=64 {
            let haystack = &mut source[start..start + len];
            assert_eq!(memchr4(0xfc, 0xfd, 0xfe, 0xff, haystack), None);
            for position in 0..len {
                for needle in 0xfc..=0xff {
                    haystack[position] = needle;
                    assert_eq!(memchr4(0xfc, 0xfd, 0xfe, 0xff, haystack), Some(position));
                }
                haystack[position] = 0;
            }
        }
    }
}
