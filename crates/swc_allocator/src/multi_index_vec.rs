/// Generates an arena-backed struct-of-arrays (SoA) container.
///
/// All field arrays share one allocation with the following layout:
/// `[Field1 × cap][Field2 × cap]...`, including alignment padding between
/// fields. Growing allocates a new block in the arena and moves the initialized
/// prefix of every field into it. The arena reclaims both current and superseded
/// blocks together.
///
/// As with the other SWC arena collections, field types must not implement
/// [`Drop`].
#[macro_export]
macro_rules! multi_index_vec {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident<$alloc:lifetime, $idx:ty> {
            $(
                $fvis:vis $fname:ident
                    => $fname_mut:ident
                    => $fname_unchecked:ident
                    => $fname_mut_unchecked:ident
                    : $fty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name<$alloc> {
            allocator: &$alloc $crate::Allocator,
            $( $fname: ::core::ptr::NonNull<$fty>, )*
            len: u32,
            cap: u32,
        }

        #[expect(
            clippy::allow_attributes,
            reason = "macro-generated methods may not all be used"
        )]
        impl<$alloc> $name<$alloc> {
            #[inline(always)]
            fn max_len() -> usize {
                <$idx as ::oxc_index::Idx>::MAX
                    .saturating_add(1)
                    .min(u32::MAX as usize)
            }

            fn compute_layout(cap: usize) -> ::core::alloc::Layout {
                let mut layout = ::core::alloc::Layout::new::<()>();
                $(
                    let field_layout = ::core::alloc::Layout::array::<$fty>(cap)
                        .expect("capacity overflow");
                    layout = layout
                        .extend(field_layout)
                        .expect("capacity overflow")
                        .0;
                )*
                layout.pad_to_align()
            }

            #[inline]
            $vis fn new_in(
                allocator: &$alloc $crate::Allocator,
            ) -> Self {
                const {
                    $(
                        assert!(
                            !::core::mem::needs_drop::<$fty>(),
                            concat!(
                                "arena-backed multi-index vec cannot contain a Drop type: ",
                                stringify!($fty),
                            ),
                        );
                    )*
                }

                Self {
                    allocator,
                    $( $fname: ::core::ptr::NonNull::dangling(), )*
                    len: 0,
                    cap: 0,
                }
            }

            #[inline]
            $vis fn with_capacity_in(
                capacity: usize,
                allocator: &$alloc $crate::Allocator,
            ) -> Self {
                let mut result = Self::new_in(allocator);
                result.reserve(capacity);
                result
            }

            #[inline]
            $vis fn len(&self) -> usize {
                self.len as usize
            }

            #[inline]
            #[allow(dead_code)]
            $vis fn is_empty(&self) -> bool {
                self.len == 0
            }

            #[inline]
            #[allow(dead_code)]
            $vis fn capacity(&self) -> usize {
                self.cap as usize
            }

            #[inline]
            #[allow(dead_code)]
            $vis fn truncate(&mut self, len: usize) {
                if len < self.len as usize {
                    self.len = u32::try_from(len).expect("length exceeds u32");
                }
            }

            $vis fn reserve(&mut self, additional: usize) {
                let required = (self.len as usize)
                    .checked_add(additional)
                    .expect("capacity overflow");
                assert!(
                    required <= Self::max_len(),
                    "capacity exceeds index type range"
                );
                if required > self.cap as usize {
                    self.grow_to(required);
                }
            }

            #[cold]
            fn grow_to(&mut self, min_cap: usize) {
                let max_len = Self::max_len();
                assert!(min_cap <= max_len, "capacity exceeds index type range");

                let new_cap = if self.cap == 0 {
                    min_cap.max(4)
                } else {
                    (self.cap as usize)
                        .saturating_mul(2)
                        .max(min_cap)
                }
                .min(max_len);

                let new_layout = Self::compute_layout(new_cap);
                let new_base = self.allocator.alloc_layout(new_layout).as_ptr();
                let old_len = self.len as usize;
                let mut layout = ::core::alloc::Layout::new::<()>();

                // SAFETY: `new_base` was allocated with `new_layout`. Each
                // offset is produced by the same sequence of `Layout::extend`
                // calls as `compute_layout`, and `old_len <= old cap`.
                unsafe {
                    $(
                        let field_layout = ::core::alloc::Layout::array::<$fty>(new_cap)
                            .expect("capacity overflow");
                        let (next_layout, offset) = layout
                            .extend(field_layout)
                            .expect("capacity overflow");
                        let new_ptr = new_base.add(offset).cast::<$fty>();
                        ::core::ptr::copy_nonoverlapping(
                            self.$fname.as_ptr(),
                            new_ptr,
                            old_len,
                        );
                        self.$fname = ::core::ptr::NonNull::new_unchecked(new_ptr);
                        layout = next_layout;
                    )*
                }
                let _ = layout;

                self.cap = u32::try_from(new_cap).expect("capacity exceeds u32");
            }

            #[inline(always)]
            $vis fn push(&mut self, $( $fname: $fty ),*) -> $idx {
                if self.len == self.cap {
                    let min_cap = (self.len as usize)
                        .checked_add(1)
                        .expect("capacity overflow");
                    self.grow_to(min_cap);
                }

                let idx = self.len as usize;
                // SAFETY: `idx < cap`; all field pointers are aligned and point
                // to allocations valid for `cap` elements.
                unsafe {
                    $(
                        self.$fname.as_ptr().add(idx).write($fname);
                    )*
                }
                self.len += 1;

                // SAFETY: `idx < len <= max_len`, so it is representable by
                // the configured index type.
                unsafe { <$idx as ::oxc_index::Idx>::from_usize_unchecked(idx) }
            }

            #[inline(always)]
            fn checked_idx(&self, id: $idx) -> usize {
                let idx = ::oxc_index::Idx::index(id);
                assert!(idx < self.len as usize, "index out of bounds");
                idx
            }

            $(
                #[inline]
                #[allow(dead_code)]
                $fvis fn $fname(&self, id: $idx) -> &$fty {
                    let idx = self.checked_idx(id);
                    // SAFETY: `checked_idx` proved this element initialized.
                    unsafe { &*self.$fname.as_ptr().add(idx) }
                }

                #[inline]
                #[allow(dead_code)]
                $fvis fn $fname_mut(&mut self, id: $idx) -> &mut $fty {
                    let idx = self.checked_idx(id);
                    // SAFETY: `checked_idx` proved this element initialized,
                    // and `&mut self` guarantees exclusive access.
                    unsafe { &mut *self.$fname.as_ptr().add(idx) }
                }

                #[inline]
                #[allow(dead_code)]
                $fvis unsafe fn $fname_unchecked(&self, id: $idx) -> &$fty {
                    let idx = ::oxc_index::Idx::index(id);
                    unsafe { &*self.$fname.as_ptr().add(idx) }
                }

                #[inline]
                #[allow(dead_code)]
                $fvis unsafe fn $fname_mut_unchecked(&mut self, id: $idx) -> &mut $fty {
                    let idx = ::oxc_index::Idx::index(id);
                    unsafe { &mut *self.$fname.as_ptr().add(idx) }
                }
            )*
        }

        impl ::core::fmt::Debug for $name<'_> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("len", &self.len)
                    .field("cap", &self.cap)
                    .finish()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::Allocator;

    oxc_index::define_index_type! {
        struct TestId = u32;
        MAX_INDEX = u32::MAX as usize;
    }

    multi_index_vec! {
        struct TestData<'a, TestId> {
            first => first_mut => first_unchecked => first_mut_unchecked: u64,
            second => second_mut => second_unchecked => second_mut_unchecked: u8,
            third => third_mut => third_unchecked => third_mut_unchecked: u32,
        }
    }

    oxc_index::define_index_type! {
        struct SmallId = u8;
        MAX_INDEX = 3;
    }

    multi_index_vec! {
        struct SmallData<'a, SmallId> {
            value => value_mut => value_unchecked => value_mut_unchecked: u8,
        }
    }

    #[test]
    fn grows_and_preserves_all_fields() {
        let allocator = Allocator::new();
        let mut data = TestData::with_capacity_in(1, &allocator);

        for value in 0..100 {
            let id = data.push(value, value as u8, value as u32);
            assert_eq!(id.index(), value as usize);
        }

        assert_eq!(data.len(), 100);
        assert!(data.capacity() >= data.len());
        for value in 0..100 {
            let id = TestId::from(value as usize);
            assert_eq!(*data.first(id), value as u64);
            assert_eq!(*data.second(id), value as u8);
            assert_eq!(*data.third(id), value);
        }
    }

    #[test]
    fn mutable_access_updates_one_field() {
        let allocator = Allocator::new();
        let mut data = TestData::new_in(&allocator);
        let id = data.push(1, 2, 3);

        *data.second_mut(id) = 4;

        assert_eq!(*data.first(id), 1);
        assert_eq!(*data.second(id), 4);
        assert_eq!(*data.third(id), 3);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn checked_access_rejects_invalid_index() {
        let allocator = Allocator::new();
        let mut data = TestData::new_in(&allocator);
        data.push(1, 2, 3);
        let _ = data.first(TestId::from(1usize));
    }

    #[test]
    #[should_panic(expected = "capacity exceeds index type range")]
    fn reserve_rejects_capacity_beyond_index_range() {
        let allocator = Allocator::new();
        let mut data = SmallData::with_capacity_in(1, &allocator);
        let id = data.push(1);
        assert_eq!(data.len(), 1);
        assert_eq!(*data.value(id), 1);
        data.reserve(4);
    }
}
