// Based on `bumpalo::boxed` from the `bumpalo` crate, with some modifications to fit our use case:
// - Remove `Drop` implementation, since we don't want to run `Drop` implementations when the box goes out of scope.
//
// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(unstable_name_collisions)]
#![allow(dead_code)]
#![allow(unused_unsafe)]

use core::cmp;
use core::mem;
use core::ptr::{self, NonNull};
use std::alloc::{Layout, handle_alloc_error};

use crate::Allocator;

/// Augments allocator errors with a capacity overflow variant.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CollectionAllocErr {
    CapacityOverflow,
    AllocErr,
}

use self::CollectionAllocErr::*;

// use boxed::Box;

/// A low-level utility for more ergonomically allocating, reallocating, and deallocating
/// a buffer of memory on the heap without having to worry about all the corner cases
/// involved. This type is excellent for building your own data structures like Vec and VecDeque.
/// In particular:
///
/// * Produces Unique::empty() on zero-sized types
/// * Produces Unique::empty() on zero-length allocations
/// * Catches all overflows in capacity computations (promotes them to "capacity overflow" panics)
/// * Guards against 32-bit systems allocating more than isize::MAX bytes
/// * Guards against overflowing your length
/// * Aborts on OOM
/// * Avoids freeing Unique::empty()
/// * Contains a ptr::Unique and thus endows the user with all related benefits
///
/// This type does not in anyway inspect the memory that it manages. When dropped it *will*
/// free its memory, but it *won't* try to Drop its contents. It is up to the user of RawVec
/// to handle the actual things *stored* inside of a RawVec.
///
/// Note that a RawVec always forces its capacity to be usize::MAX for zero-sized types.
/// This enables you to use capacity growing logic catch the overflows in your length
/// that might occur with zero-sized types.
///
/// However this means that you need to be careful when round-tripping this type
/// with a `Box<[T]>`: `cap()` won't yield the len. However `with_capacity`,
/// `shrink_to_fit`, and `from_box` will actually set RawVec's private capacity
/// field. This allows zero-sized types to not be special-cased by consumers of
/// this type.
#[allow(missing_debug_implementations)]
pub struct RawVec<'a, T> {
    ptr: NonNull<T>,
    len: u32,
    cap: u32,
    a: &'a Allocator,
}

impl<'a, T> RawVec<'a, T> {
    /// Like `new` but parameterized over the choice of allocator for
    /// the returned RawVec.
    pub fn new_in(a: &'a Allocator) -> Self {
        // `cap: 0` means "unallocated". zero-sized types are ignored.
        RawVec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
            a,
        }
    }

    /// Like `with_capacity` but parameterized over the choice of
    /// allocator for the returned RawVec.
    #[inline]
    pub fn with_capacity_in(cap: usize, a: &'a Allocator) -> Self {
        RawVec::allocate_in(cap, false, a)
    }

    /// Like `with_capacity_zeroed` but parameterized over the choice
    /// of allocator for the returned RawVec.
    #[inline]
    pub fn with_capacity_zeroed_in(cap: usize, a: &'a Allocator) -> Self {
        RawVec::allocate_in(cap, true, a)
    }

    fn allocate_in(cap: usize, zeroed: bool, a: &'a Allocator) -> Self {
        if mem::size_of::<T>() != 0 && cap > u32::MAX as usize {
            capacity_overflow();
        }

        let elem_size = mem::size_of::<T>();
        let alloc_size = cap
            .checked_mul(elem_size)
            .unwrap_or_else(|| capacity_overflow());
        alloc_guard(alloc_size).unwrap_or_else(|_| capacity_overflow());

        let ptr = if alloc_size == 0 {
            NonNull::<T>::dangling()
        } else {
            let align = mem::align_of::<T>();
            let layout = Layout::from_size_align(alloc_size, align).unwrap();
            let ptr = a
                .arena
                .try_alloc_layout(layout)
                .unwrap_or_else(|_| handle_alloc_error(layout));
            if zeroed {
                unsafe {
                    ptr::write_bytes(ptr.as_ptr(), 0, alloc_size);
                }
            }
            ptr.cast()
        };

        let cap = cap as u32;
        RawVec {
            ptr,
            len: 0,
            cap,
            a,
        }
    }
}

impl<'a, T> RawVec<'a, T> {
    /// Reconstitutes a RawVec from a pointer, length, capacity, and allocator.
    ///
    /// # Undefined Behavior
    ///
    /// The ptr must be allocated (via the given allocator `a`), and with the given capacity. The
    /// capacity cannot exceed `u32::MAX` elements, and the allocation cannot exceed
    /// `isize::MAX` bytes (only a concern on 32-bit systems).
    /// If the ptr and capacity come from a RawVec created via `a`, then this is guaranteed.
    ///
    /// # Safety
    ///
    /// `ptr` must have been allocated by `a` for `cap` values of `T`, and the
    /// allocation must satisfy `RawVec`'s alignment and capacity invariants.
    /// `len` must be less than or equal to `cap`.
    /// `len` must be less than or equal to `u32::MAX`, as length is stored as `u32`.
    /// `cap` must be less than or equal to `u32::MAX`, as capacity is stored as `u32`.
    pub unsafe fn from_raw_parts_in(ptr: *mut T, len: usize, cap: usize, a: &'a Allocator) -> Self {
        let len = len as u32;
        let cap = cap as u32;
        RawVec {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len,
            cap,
            a,
        }
    }
}

impl<'a, T> RawVec<'a, T> {
    /// Gets a raw pointer to the start of the allocation. Note that this is
    /// Unique::empty() if `cap = 0` or T is zero-sized. In the former case, you must
    /// be careful.
    pub fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Gets the number of elements as `u32`.
    #[inline(always)]
    pub fn len_u32(&self) -> u32 {
        self.len
    }

    /// Gets the number of elements as `usize`.
    #[inline(always)]
    pub fn len_usize(&self) -> usize {
        self.len as usize
    }

    /// Set the number of elements.
    #[inline(always)]
    pub fn set_len(&mut self, new_len: u32) {
        self.len = new_len;
    }

    /// Increase the number of elements by `increment`.
    #[inline(always)]
    pub fn increase_len(&mut self, increment: u32) {
        self.len += increment;
    }

    /// Decrease the number of elements by `decrement`.
    #[inline(always)]
    pub fn decrease_len(&mut self, decrement: u32) {
        self.len -= decrement;
    }

    /// Gets the capacity of the allocation.
    ///
    /// This will always be `usize::MAX` if `T` is zero-sized.
    #[inline(always)]
    pub fn cap(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            !0
        } else {
            self.cap as usize
        }
    }

    /// Gets the capacity of the allocation as `u32`.
    ///
    /// This will always be `u32::MAX` if `T` is zero-sized.
    #[inline(always)]
    pub fn cap_u32(&self) -> u32 {
        if mem::size_of::<T>() == 0 {
            !0
        } else {
            self.cap
        }
    }

    /// Returns a shared reference to the allocator backing this RawVec.
    pub fn bump(&self) -> &'a Allocator {
        self.a
    }

    fn current_layout(&self) -> Option<Layout> {
        if self.cap == 0 {
            None
        } else {
            // We have an allocated chunk of memory, so we can bypass runtime
            // checks to get our current layout.
            unsafe {
                let align = mem::align_of::<T>();
                let size = mem::size_of::<T>() * self.cap as usize;
                Some(Layout::from_size_align_unchecked(size, align))
            }
        }
    }

    /// The same as `reserve_exact`, but returns on errors instead of panicking or aborting.
    pub fn try_reserve_exact(
        &mut self,
        used_cap: u32,
        needed_extra_cap: usize,
    ) -> Result<(), CollectionAllocErr> {
        self.fallible_reserve_internal(used_cap, needed_extra_cap, Exact)
    }

    /// Ensures that the buffer contains at least enough space to hold
    /// `used_cap + needed_extra_cap` elements. If it doesn't already,
    /// will reallocate the minimum possible amount of memory necessary.
    /// Generally this will be exactly the amount of memory necessary,
    /// but in principle the allocator is free to give back more than
    /// we asked for.
    ///
    /// If `used_cap` exceeds `self.cap()`, this may fail to actually allocate
    /// the requested space. This is not really unsafe, but the unsafe
    /// code *you* write that relies on the behavior of this function may break.
    ///
    /// # Panics
    ///
    /// * Panics if the requested capacity exceeds `usize::MAX` bytes.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM
    pub fn reserve_exact(&mut self, used_cap: u32, needed_extra_cap: usize) {
        self.infallible_reserve_internal(used_cap, needed_extra_cap, Exact)
    }

    /// Calculates the buffer's new size given that it'll hold `used_cap +
    /// needed_extra_cap` elements. This logic is used in amortized reserve methods.
    /// Returns `(new_capacity, new_alloc_size)`.
    fn amortized_new_size(
        &self,
        used_cap: u32,
        needed_extra_cap: usize,
    ) -> Result<usize, CollectionAllocErr> {
        // Nothing we can really do about these checks :(
        let required_cap = (used_cap as usize)
            .checked_add(needed_extra_cap)
            .ok_or(CapacityOverflow)?;
        // Cannot overflow on supported platforms because `cap` is `u32`.
        let double_cap = (self.cap as usize).checked_mul(2).ok_or(CapacityOverflow)?;
        // `double_cap` guarantees exponential growth.
        Ok(cmp::max(double_cap, required_cap))
    }

    /// The same as `reserve`, but returns on errors instead of panicking or aborting.
    pub fn try_reserve(
        &mut self,
        used_cap: u32,
        needed_extra_cap: usize,
    ) -> Result<(), CollectionAllocErr> {
        self.fallible_reserve_internal(used_cap, needed_extra_cap, Amortized)
    }

    /// Ensures that the buffer contains at least enough space to hold
    /// `used_cap + needed_extra_cap` elements. If it doesn't already have
    /// enough capacity, will reallocate enough space plus comfortable slack
    /// space to get amortized `O(1)` behavior. Will limit this behavior
    /// if it would needlessly cause itself to panic.
    ///
    /// If `used_cap` exceeds `self.cap()`, this may fail to actually allocate
    /// the requested space. This is not really unsafe, but the unsafe
    /// code *you* write that relies on the behavior of this function may break.
    ///
    /// This is ideal for implementing a bulk-push operation like `extend`.
    ///
    /// # Panics
    ///
    /// * Panics if the requested capacity exceeds `usize::MAX` bytes.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #![feature(alloc, raw_vec_internals)]
    /// # extern crate alloc;
    /// # use std::ptr;
    /// # use alloc::raw_vec::RawVec;
    /// struct MyVec<T> {
    ///     buf: RawVec<T>,
    ///     len: usize,
    /// }
    ///
    /// impl<T: Clone> MyVec<T> {
    ///     pub fn push_all(&mut self, elems: &[T]) {
    ///         self.buf.reserve(self.len, elems.len());
    ///         // reserve would have aborted or panicked if the len exceeded
    ///         // `isize::MAX` so this is safe to do unchecked now.
    ///         for x in elems {
    ///             unsafe {
    ///                 ptr::write(self.buf.ptr().add(self.len), x.clone());
    ///             }
    ///             self.len += 1;
    ///         }
    ///     }
    /// }
    /// # fn main() {
    /// #   let mut vector = MyVec { buf: RawVec::new(), len: 0 };
    /// #   vector.push_all(&[1, 3, 5, 7, 9]);
    /// # }
    /// ```
    #[inline(always)]
    pub fn reserve(&mut self, used_cap: u32, needed_extra_cap: usize) {
        self.infallible_reserve_internal(used_cap, needed_extra_cap, Amortized)
    }

    /// Attempts to ensure that the buffer contains at least enough space to hold
    /// `used_cap + needed_extra_cap` elements. If it doesn't already have
    /// enough capacity, will reallocate in place enough space plus comfortable slack
    /// space to get amortized `O(1)` behavior. Will limit this behaviour
    /// if it would needlessly cause itself to panic.
    ///
    /// If `used_cap` exceeds `self.cap()`, this may fail to actually allocate
    /// the requested space. This is not really unsafe, but the unsafe
    /// code *you* write that relies on the behavior of this function may break.
    ///
    /// Returns true if the reallocation attempt has succeeded, or false otherwise.
    ///
    /// # Panics
    ///
    /// * Panics if the requested capacity exceeds `usize::MAX` bytes.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    pub fn reserve_in_place(&mut self, used_cap: u32, needed_extra_cap: usize) -> bool {
        unsafe {
            // NOTE: we don't early branch on ZSTs here because we want this
            // to actually catch "asking for more than usize::MAX" in that case.
            // If we make it past the first branch then we are guaranteed to
            // panic.

            // Don't actually need any more capacity. If the current `cap` is 0, we can't
            // reallocate in place.
            // Wrapping in case they give a bad `used_cap`
            let old_layout = match self.current_layout() {
                Some(layout) => layout,
                None => return false,
            };
            if self.cap_u32().wrapping_sub(used_cap) as usize >= needed_extra_cap {
                return false;
            }

            let new_cap = self
                .amortized_new_size(used_cap, needed_extra_cap)
                .unwrap_or_else(|_| capacity_overflow());

            // Here, `cap < used_cap + needed_extra_cap <= new_cap`
            // (regardless of whether `self.cap - used_cap` wrapped).
            // Therefore we can safely call grow_in_place.

            let new_layout = Layout::array::<T>(new_cap).unwrap();
            // FIXME: may crash and burn on over-reserve
            alloc_guard(new_layout.size()).unwrap_or_else(|_| capacity_overflow());
            match self.try_grow_in_place(old_layout, new_layout.size()) {
                Ok(()) => {
                    if new_cap > u32::MAX as usize {
                        capacity_overflow();
                    }
                    self.cap = new_cap as u32;
                    true
                }
                Err(_) => false,
            }
        }
    }

    /// Shrinks the allocation down to the specified amount. If the given amount
    /// is 0, actually completely deallocates.
    ///
    /// # Panics
    ///
    /// Panics if the given amount is *larger* than the current capacity.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM.
    pub fn shrink_to_fit(&mut self, amount: u32) {
        let elem_size = mem::size_of::<T>();

        // Set the `cap` because they might be about to promote to a `Box<[T]>`
        if elem_size == 0 {
            self.cap = amount;
            return;
        }

        // This check is my waterloo; it's the only thing Vec wouldn't have to do.
        assert!(self.cap >= amount, "Tried to shrink to a larger capacity");

        if amount == 0 {
            // We want to create a new zero-length vector within the
            // same allocator.  We use ptr::write to avoid an
            // erroneous attempt to drop the contents, and we use
            // ptr::read to sidestep condition against destructuring
            // types that implement Drop.

            unsafe {
                let a = self.a;
                self.dealloc_buffer();
                ptr::write(self, RawVec::new_in(a));
            }
        } else if self.cap != amount {
            unsafe {
                // We know here that our `amount` is greater than zero. This
                // implies, via the assert above, that capacity is also greater
                // than zero, which means that we've got a current layout that
                // "fits"
                //
                // We also know that `self.cap` is greater than `amount`, and
                // consequently we don't need runtime checks for creating either
                // layout
                let old_size = elem_size * self.cap as usize;
                let new_size = elem_size * amount as usize;
                let align = mem::align_of::<T>();
                let old_layout = Layout::from_size_align_unchecked(old_size, align);
                match self.realloc_buffer(old_layout, new_size) {
                    Ok(p) => self.ptr = p.cast(),
                    Err(_) => {
                        handle_alloc_error(Layout::from_size_align_unchecked(new_size, align))
                    }
                }
            }
            self.cap = amount;
        }
    }
}

enum Fallibility {
    Fallible,
    Infallible,
}

use self::Fallibility::*;

enum ReserveStrategy {
    Exact,
    Amortized,
}

use self::ReserveStrategy::*;

impl<'a, T> RawVec<'a, T> {
    #[inline(always)]
    fn fallible_reserve_internal(
        &mut self,
        used_cap: u32,
        needed_extra_cap: usize,
        strategy: ReserveStrategy,
    ) -> Result<(), CollectionAllocErr> {
        // This portion of the method should always be inlined.
        if self.cap_u32().wrapping_sub(used_cap) as usize >= needed_extra_cap {
            return Ok(());
        }
        // This portion of the method should never be inlined, and will only be called when
        // the check above has confirmed that it is necessary.
        self.reserve_internal_or_error(used_cap, needed_extra_cap, Fallible, strategy)
    }

    #[inline(always)]
    fn infallible_reserve_internal(
        &mut self,
        used_cap: u32,
        needed_extra_cap: usize,
        strategy: ReserveStrategy,
    ) {
        // This portion of the method should always be inlined.
        if self.cap_u32().wrapping_sub(used_cap) as usize >= needed_extra_cap {
            return;
        }
        // This portion of the method should never be inlined, and will only be called when
        // the check above has confirmed that it is necessary.
        self.reserve_internal_or_panic(used_cap, needed_extra_cap, strategy)
    }

    #[inline(never)]
    fn reserve_internal_or_panic(
        &mut self,
        used_cap: u32,
        needed_extra_cap: usize,
        strategy: ReserveStrategy,
    ) {
        // Delegates the call to `reserve_internal_or_error` and panics in the event of an error.
        // This allows the method to have a return type of `()`, simplifying the assembly at the
        // call site.
        match self.reserve_internal(used_cap, needed_extra_cap, Infallible, strategy) {
            Err(CapacityOverflow) => capacity_overflow(),
            Err(AllocErr) => unreachable!(),
            Ok(()) => { /* yay */ }
        }
    }

    #[inline(never)]
    fn reserve_internal_or_error(
        &mut self,
        used_cap: u32,
        needed_extra_cap: usize,
        fallibility: Fallibility,
        strategy: ReserveStrategy,
    ) -> Result<(), CollectionAllocErr> {
        // Delegates the call to `reserve_internal`, which can be inlined.
        self.reserve_internal(used_cap, needed_extra_cap, fallibility, strategy)
    }

    /// Helper method to reserve additional space, reallocating the backing memory.
    /// The caller is responsible for confirming that there is not already enough space available.
    fn reserve_internal(
        &mut self,
        used_cap: u32,
        needed_extra_cap: usize,
        fallibility: Fallibility,
        strategy: ReserveStrategy,
    ) -> Result<(), CollectionAllocErr> {
        unsafe {
            // NOTE: we don't early branch on ZSTs here because we want this
            // to actually catch "asking for more than usize::MAX" in that case.
            // If we make it past the first branch then we are guaranteed to
            // panic.

            // Nothing we can really do about these checks :(
            let new_cap = match strategy {
                Exact => (used_cap as usize)
                    .checked_add(needed_extra_cap)
                    .ok_or(CapacityOverflow)?,
                Amortized => self.amortized_new_size(used_cap, needed_extra_cap)?,
            };
            if new_cap > u32::MAX as usize {
                return Err(CapacityOverflow);
            }

            let new_layout = Layout::array::<T>(new_cap).map_err(|_| CapacityOverflow)?;

            alloc_guard(new_layout.size())?;

            let res = match self.current_layout() {
                Some(layout) => {
                    debug_assert!(new_layout.align() == layout.align());
                    self.realloc_buffer(layout, new_layout.size())
                }
                None => self
                    .a
                    .arena
                    .try_alloc_layout(new_layout)
                    .map_err(|_| CollectionAllocErr::AllocErr),
            };

            if let (Err(AllocErr), Infallible) = (&res, fallibility) {
                handle_alloc_error(new_layout);
            }

            self.ptr = res?.cast();
            self.cap = new_cap as u32;

            Ok(())
        }
    }

    unsafe fn realloc_buffer(
        &self,
        old_layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<u8>, CollectionAllocErr> {
        let new_layout =
            Layout::from_size_align(new_size, old_layout.align()).map_err(|_| CapacityOverflow)?;
        let new_ptr = self
            .a
            .arena
            .try_alloc_layout(new_layout)
            .map_err(|_| CollectionAllocErr::AllocErr)?;
        unsafe {
            ptr::copy_nonoverlapping(
                self.ptr.cast::<u8>().as_ptr(),
                new_ptr.as_ptr(),
                cmp::min(old_layout.size(), new_size),
            );
        }
        Ok(new_ptr)
    }

    fn try_grow_in_place(
        &self,
        _old_layout: Layout,
        _new_size: usize,
    ) -> Result<(), CollectionAllocErr> {
        Err(CollectionAllocErr::AllocErr)
    }
}

impl<'a, T> RawVec<'a, T> {
    /// Frees the memory owned by the RawVec *without* trying to Drop its contents.
    ///
    /// # Safety
    ///
    /// The buffer must not be used after this call.
    pub unsafe fn dealloc_buffer(&mut self) {
        let elem_size = mem::size_of::<T>();
        if elem_size != 0
            && let Some(layout) = self.current_layout()
        {
            let _ = layout;
        }
    }
}

// We need to guarantee the following:
// * We don't ever allocate `> isize::MAX` byte-size objects
// * We don't overflow `u32::MAX` and actually allocate too little
//
// On 64-bit we need to check for allocations which no longer fit in our `u32`
// capacity representation. On 32-bit and 16-bit we need to guard `isize::MAX`
// in case we're running on a platform which can use all 4GB in user-space,
// e.g. PAE or x32.

#[inline]
fn alloc_guard(alloc_size: usize) -> Result<(), CollectionAllocErr> {
    if mem::size_of::<usize>() < 8 {
        if alloc_size > isize::MAX as usize {
            return Err(CapacityOverflow);
        }
    } else if alloc_size > u32::MAX as usize {
        return Err(CapacityOverflow);
    }
    Ok(())
}

// One central function responsible for reporting capacity overflows. This'll
// ensure that the code generation related to these panics is minimal as there's
// only one location which panics rather than a bunch throughout the module.
fn capacity_overflow() -> ! {
    panic!("capacity overflow")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_does_not_overallocate() {
        let bump = Allocator::new();
        {
            let mut v: RawVec<u32> = RawVec::new_in(&bump);
            // First `reserve` allocates like `reserve_exact`
            v.reserve(0, 9);
            assert_eq!(9, v.cap());
        }

        {
            let mut v: RawVec<u32> = RawVec::new_in(&bump);
            v.reserve(0, 7);
            assert_eq!(7, v.cap());
            // 97 if more than double of 7, so `reserve` should work
            // like `reserve_exact`.
            v.reserve(7, 90);
            assert_eq!(97, v.cap());
        }

        {
            let mut v: RawVec<u32> = RawVec::new_in(&bump);
            v.reserve(0, 12);
            assert_eq!(12, v.cap());
            v.reserve(12, 3);
            // 3 is less than half of 12, so `reserve` must grow
            // exponentially. At the time of writing this test grow
            // factor is 2, so new capacity is 24, however, grow factor
            // of 1.5 is OK too. Hence `>= 18` in assert.
            assert!(v.cap() >= 12 + 12 / 2);
        }
    }

    #[test]
    fn try_reserve_reports_capacity_overflow_above_u32_max() {
        let bump = Allocator::new();
        let mut v: RawVec<u8> = RawVec::new_in(&bump);

        assert_eq!(
            v.try_reserve_exact(u32::MAX, 2),
            Err(CollectionAllocErr::CapacityOverflow)
        );
        assert_eq!(
            v.try_reserve(u32::MAX, 2),
            Err(CollectionAllocErr::CapacityOverflow)
        );
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn with_capacity_panics_above_u32_max() {
        let bump = Allocator::new();
        let _ = RawVec::<u8>::with_capacity_in(u32::MAX as usize + 1, &bump);
    }
}
