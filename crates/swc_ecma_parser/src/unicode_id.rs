// Lookup implementation based on Yuku's src/util/unicode_id.zig, modified for SWC Next.
// See unicode_id_tables.rs for the fixed source reference.

const CHUNK_SIZE: u32 = 512;
const BITS_PER_WORD: u32 = 32;
const LEAF_CHUNK_WIDTH: usize = 16;

include!("unicode_id_tables.rs");

pub fn can_start_id(cp: u32) -> bool {
    cp == u32::from(b'$')
        || cp == u32::from(b'_')
        || query_bit_table(cp, &ID_START_ROOT, &ID_START_LEAF)
}

pub fn can_continue_id(cp: u32) -> bool {
    cp == u32::from(b'$')
        || cp == u32::from(b'_')
        || cp == 0x200c
        || cp == 0x200d
        || query_bit_table(cp, &ID_CONTINUE_ROOT, &ID_CONTINUE_LEAF)
}

fn query_bit_table(cp: u32, root: &[u8], leaf: &[u64]) -> bool {
    let chunk_idx = cp / CHUNK_SIZE;
    let leaf_base = usize::from(root[chunk_idx as usize]) * LEAF_CHUNK_WIDTH;
    let offset_in_chunk = cp - (chunk_idx * CHUNK_SIZE);
    let word_idx = leaf_base + (offset_in_chunk / BITS_PER_WORD) as usize;
    let bit_position = offset_in_chunk % BITS_PER_WORD;
    let word = leaf[word_idx];
    ((word >> bit_position) & 1) == 1
}
