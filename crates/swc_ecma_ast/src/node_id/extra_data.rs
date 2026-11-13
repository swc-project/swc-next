use crate::*;

swc_next_allocator::define_index_type! {
    #[repr(transparent)]
    pub struct ExtraDataId = u32;
    MAX_INDEX = u32::MAX as usize;
}

/// One four-byte storage unit in the AST extra-data table.
///
/// Node IDs and optional-node sentinels use one unit. Ranges and strings use
/// two consecutive units, with their start followed by their end.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ExtraDataUnit {
    pub(crate) word: u32,
    pub(crate) node_id: NodeId,
    pub(crate) optional_node_id: u32,
    pub(crate) optional_import_phase: u32,
}

impl ExtraDataUnit {
    #[inline]
    pub(crate) fn raw(self) -> u32 {
        // SAFETY: Every field initializes the full unit and accepts a u32 representation.
        unsafe { self.word }
    }

    #[inline]
    pub(crate) fn from_raw_node_id(raw: u32) -> Self {
        Self {
            optional_node_id: raw,
        }
    }
}

/// A typed view over two consecutive extra-data units.
///
/// Both words retain their order on every target; each word uses native byte order.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ExtraData {
    pub(crate) units: [ExtraDataUnit; 2],
    pub(crate) sub_range: SubRange<()>,
    pub(crate) utf8: AstUtf8,
}

#[cfg(test)]
mod tests {
    use crate::{ExtraData, ExtraDataId, ExtraDataUnit, SubRange};

    #[test]
    fn canonical_storage() {
        let data = ExtraData {
            units: [
                ExtraDataUnit { word: 0x89ab_cdef },
                ExtraDataUnit { word: 0x0123_4567 },
            ],
        };
        assert_eq!(
            unsafe { data.units }.map(ExtraDataUnit::raw),
            [0x89ab_cdef, 0x0123_4567]
        );

        let data = ExtraDataUnit::from_raw_node_id(u32::MAX);
        assert_eq!(data.raw(), u32::MAX);

        let data = ExtraDataUnit::from_raw_node_id(0x1122_3344);
        assert_eq!(data.raw(), 0x1122_3344);
        assert_eq!(unsafe { data.node_id }.index(), 0x1122_3344);

        let data = ExtraData {
            sub_range: SubRange::new(
                ExtraDataId::from_raw(0x1122_3344),
                ExtraDataId::from_raw(0x5566_7788),
            ),
        };
        assert_eq!(
            unsafe { data.units }.map(ExtraDataUnit::raw),
            [0x1122_3344, 0x5566_7788]
        );
        assert_eq!(unsafe { data.sub_range }.start.index(), 0x1122_3344);
        assert_eq!(unsafe { data.sub_range }.end.index(), 0x5566_7788);
    }
}
