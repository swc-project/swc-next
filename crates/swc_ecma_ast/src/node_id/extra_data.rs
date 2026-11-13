use crate::*;

swc_next_allocator::define_index_type! {
    #[repr(transparent)]
    pub struct ExtraDataId = u32;
    MAX_INDEX = u32::MAX as usize;
}

/// Typed values stored in the flattening AST's extra-data table.
///
/// The AST schema supplies the active field without a runtime discriminant.
/// Every slot initializes all eight bytes for the binary AST format, with the
/// low and high words stored in that order in native byte order.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ExtraData {
    pub(crate) words: [u32; 2],
    pub(crate) node_id: NodeId,
    pub(crate) optional_node_id: u32,
    pub(crate) sub_range: SubRange<()>,
    pub(crate) utf8: AstUtf8,
    pub(crate) optional_import_phase: u32,
}

impl ExtraData {
    pub(crate) const ZERO: Self = Self { words: [0; 2] };

    #[inline]
    pub(crate) fn raw(self) -> u64 {
        // SAFETY: All constructors initialize both words.
        let [low, high] = unsafe { self.words };
        u64::from(low) | (u64::from(high) << 32)
    }

    #[inline]
    pub(crate) fn from_raw_node_id(raw: u32) -> Self {
        Self { words: [raw, 0] }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExtraData, ExtraDataId, SubRange};

    #[test]
    fn canonical_storage() {
        let data = ExtraData {
            words: [0x89ab_cdef, 0x0123_4567],
        };
        assert_eq!(data.raw(), 0x0123_4567_89ab_cdef);

        let data = ExtraData::from_raw_node_id(u32::MAX);
        assert_eq!(data.raw(), u64::from(u32::MAX));

        let data = ExtraData::from_raw_node_id(0x1122_3344);
        assert_eq!(data.raw(), 0x1122_3344);
        assert_eq!(unsafe { data.node_id }.index(), 0x1122_3344);

        let data = ExtraData {
            sub_range: SubRange::new(
                ExtraDataId::from_raw(0x1122_3344),
                ExtraDataId::from_raw(0x5566_7788),
            ),
        };
        assert_eq!(data.raw(), 0x5566_7788_1122_3344);
        assert_eq!(unsafe { data.sub_range }.start.index(), 0x1122_3344);
        assert_eq!(unsafe { data.sub_range }.end.index(), 0x5566_7788);
    }
}
