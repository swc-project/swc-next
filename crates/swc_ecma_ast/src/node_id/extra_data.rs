use crate::{AstInternedUtf8, AstNode, AstUtf8, AstWtf8, SubRange};

use super::{NodeId, sealed::ExtraDataValue as SealedExtraDataValue};

oxc_index::define_index_type! {
    #[repr(transparent)]
    pub struct ExtraDataId = u32;
    MAX_INDEX = u32::MAX as usize;
}

/// Compact values stored in the flattening AST's extra-data table.
///
/// The AST schema supplies the type information, so the table stores only its
/// 64-bit representation without a runtime discriminant. The low and high
/// words are stored in that order, each in native byte order.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ExtraData {
    node_id: NodeId,
    sub_range: SubRange<()>,
    alignment: u64,
}

impl ExtraData {
    #[inline]
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self {
            sub_range: SubRange::new(
                ExtraDataId::from_raw(raw as u32),
                ExtraDataId::from_raw((raw >> 32) as u32),
            ),
        }
    }

    #[inline]
    pub(crate) fn raw(self) -> u64 {
        // SAFETY: Both range fields accept every u32 bit pattern, and all
        // constructors initialize the full eight-byte slot.
        let range = unsafe { self.sub_range };
        (range.start.index() as u64) | ((range.end.index() as u64) << 32)
    }
}

/// A value that has an eight-byte representation in the AST extra-data table.
/// Implementations are restricted to the storage types supported by this crate.
pub trait ExtraDataValue: Sized + SealedExtraDataValue {
    fn into_extra_data(self) -> ExtraData;

    /// # Safety
    /// `data` must have been written for `Self`.
    unsafe fn from_extra_data(data: &ExtraData) -> Self;
}

impl<T: AstNode> SealedExtraDataValue for T {}

impl<T: AstNode> ExtraDataValue for T {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.node_id().index() as u64)
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node_id) }
    }
}

impl<T: AstNode> SealedExtraDataValue for Option<T> {}

impl<T: AstNode> ExtraDataValue for Option<T> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.map_or(u64::from(u32::MAX), |node| node.node_id().index() as u64))
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let raw = unsafe { data.node_id }.index() as u32;
        (raw != u32::MAX)
            .then(|| unsafe { T::from_node_id_unchecked(NodeId::from_raw_unchecked(raw)) })
    }
}

impl<T> SealedExtraDataValue for SubRange<T> {}

impl<T> ExtraDataValue for SubRange<T> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData {
            sub_range: SubRange::new(self.start, self.end),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let range = unsafe { data.sub_range };
        Self::new(range.start, range.end)
    }
}

impl SealedExtraDataValue for AstUtf8 {}

impl ExtraDataValue for AstUtf8 {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        Self::from_raw(data.raw())
    }
}

impl SealedExtraDataValue for AstInternedUtf8 {}

impl ExtraDataValue for AstInternedUtf8 {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        Self::from_raw(data.raw())
    }
}

impl SealedExtraDataValue for Option<AstUtf8> {}

impl ExtraDataValue for Option<AstUtf8> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        // `AstUtf8` ranges always satisfy `start <= end`, so `(u32::MAX, 0)`
        // is available as an unambiguous `None` sentinel.
        ExtraData::from_raw(self.map_or(u64::from(u32::MAX), AstUtf8::raw))
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let raw = data.raw();
        (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
    }
}

impl SealedExtraDataValue for f64 {}

impl ExtraDataValue for f64 {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.to_bits())
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        Self::from_bits(data.raw())
    }
}

impl SealedExtraDataValue for AstWtf8 {}

impl ExtraDataValue for AstWtf8 {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        Self::from_raw(data.raw())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExtraData, ExtraDataId, ExtraDataValue, NodeId, Program, SubRange};

    #[test]
    fn canonical_storage() {
        assert_eq!(
            ExtraData::from_raw(0x0123_4567_89ab_cdef).raw(),
            0x0123_4567_89ab_cdef
        );
        assert_eq!(ExtraData::from_raw(u64::MAX).raw(), u64::MAX);

        let data = None::<Program>.into_extra_data();
        assert_eq!(data.raw(), u64::from(u32::MAX));
        assert!(unsafe { Option::<Program>::from_extra_data(&data) }.is_none());

        let node = Program(NodeId::from_raw_unchecked(0x1122_3344));
        let data = node.into_extra_data();
        assert_eq!(data.raw(), 0x1122_3344);
        let decoded = unsafe { Program::from_extra_data(&data) };
        assert_eq!(decoded.0.index(), 0x1122_3344);

        let range = SubRange::<()>::new(
            ExtraDataId::from_raw(0x1122_3344),
            ExtraDataId::from_raw(0x5566_7788),
        );
        let data = range.into_extra_data();
        assert_eq!(data.raw(), 0x5566_7788_1122_3344);
        let decoded = unsafe { SubRange::<()>::from_extra_data(&data) };
        assert_eq!(decoded.start.index(), 0x1122_3344);
        assert_eq!(decoded.end.index(), 0x5566_7788);
    }
}
