use std::{mem, slice};

const HEADER_SIZE: usize = mem::size_of::<u32>();

// Owned values use `[payload length][payload]`. Fallible results place a
// status word at the start of that payload: zero for success, one for error.

#[doc(hidden)]
#[unsafe(export_name = "__swc_alloc")]
pub extern "C" fn alloc(payload_len: u32) -> u32 {
    allocate(payload_len as usize)
}

/// # Safety
///
/// `pointer` must have been returned by [`alloc`] or another bridge result
/// allocation and must not have been freed already.
#[doc(hidden)]
#[unsafe(export_name = "__swc_free")]
pub unsafe extern "C" fn free(pointer: u32) {
    if pointer == 0 {
        return;
    }

    let payload_len = unsafe { *(pointer as *const u32) } as usize;
    let word_len = allocation_word_len(payload_len);
    let words = unsafe { slice::from_raw_parts_mut(pointer as *mut u32, word_len) };
    drop(unsafe { Box::from_raw(words) });
}

pub(super) fn allocate_payload(payload: &[u8]) -> u32 {
    let pointer = allocate(payload.len());
    let output =
        unsafe { slice::from_raw_parts_mut((pointer as *mut u8).add(HEADER_SIZE), payload.len()) };
    output.copy_from_slice(payload);
    pointer
}

pub(super) fn allocate_tagged(tag: u32, payload: &[u8]) -> u32 {
    let payload_len = HEADER_SIZE
        .checked_add(payload.len())
        .unwrap_or_else(|| super::invalid_abi());
    let pointer = allocate(payload_len);
    let tag_pointer = unsafe { (pointer as *mut u8).add(HEADER_SIZE).cast::<u32>() };
    unsafe { tag_pointer.write(tag) };
    let output = unsafe {
        slice::from_raw_parts_mut((pointer as *mut u8).add(HEADER_SIZE * 2), payload.len())
    };
    output.copy_from_slice(payload);
    pointer
}

/// # Safety
///
/// `pointer` must identify a live bridge allocation.
pub(super) unsafe fn borrow_payload<'a>(pointer: u32) -> Option<&'a [u8]> {
    if pointer == 0 {
        return None;
    }

    let payload_len = unsafe { *(pointer as *const u32) } as usize;
    Some(unsafe { slice::from_raw_parts((pointer as *const u8).add(HEADER_SIZE), payload_len) })
}

fn allocate(payload_len: usize) -> u32 {
    let word_len = allocation_word_len(payload_len);
    let mut words = vec![0_u32; word_len].into_boxed_slice();
    words[0] = u32::try_from(payload_len).unwrap_or_else(|_| super::invalid_abi());
    Box::into_raw(words).cast::<u32>() as u32
}

fn allocation_word_len(payload_len: usize) -> usize {
    payload_len
        .checked_add(HEADER_SIZE)
        .unwrap_or_else(|| super::invalid_abi())
        .div_ceil(HEADER_SIZE)
}
