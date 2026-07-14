use std::{alloc::Layout, ptr::NonNull};

use allocator_api2::alloc::{AllocError, Allocator};

unsafe impl Allocator for crate::Allocator<'_> {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.try_alloc_layout(layout)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { self.deallocate_layout(ptr, layout) }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.shrink_layout(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.grow_layout(ptr, old_layout, new_layout, false) }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.grow_layout(ptr, old_layout, new_layout, true) }
    }
}
