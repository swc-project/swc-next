use std::mem::MaybeUninit;

pub fn allocator_buffer(source_len: usize) -> Box<[MaybeUninit<u8>]> {
    let len = source_len.saturating_mul(16).max(64 * 1024);
    Box::<[u8]>::new_uninit_slice(len)
}
