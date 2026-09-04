use crate::{AstNode, AstUtf8, AstWtf8, ExtraData, SubRange};

use super::NodeId;

oxc_index::define_index_type! {
    #[repr(transparent)]
    pub struct ExtraDataId = u32;
}

impl ExtraData {
    #[inline]
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self(raw.to_le())
    }

    #[inline]
    pub(crate) fn raw(self) -> u64 {
        u64::from_le(self.0)
    }

    #[inline]
    fn from_parts(low: u32, high: u32) -> Self {
        Self::from_raw(u64::from(low) | (u64::from(high) << 32))
    }

    #[inline]
    fn parts(&self) -> [u32; 2] {
        let parts = std::ptr::from_ref(self).cast::<u32>();
        // SAFETY: `ExtraData` is transparent over a `u64`, so both four-byte
        // reads are initialized and in bounds.
        unsafe {
            [
                u32::from_le(parts.read_unaligned()),
                u32::from_le(parts.add(1).read_unaligned()),
            ]
        }
    }
}

/// A value that has an eight-byte representation in the AST extra-data table.
pub trait ExtraDataValue: Sized {
    fn into_extra_data(self) -> ExtraData;

    /// # Safety
    /// `data` must have been written for `Self`.
    unsafe fn from_extra_data(data: &ExtraData) -> Self;
}

impl<T: AstNode> ExtraDataValue for T {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.node_id().index() as u64)
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        unsafe { Self::from_node_id_unchecked(NodeId::from_raw_unchecked(data.raw() as u32)) }
    }
}

impl<T: AstNode> ExtraDataValue for Option<T> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.map_or(u64::from(u32::MAX), |node| node.node_id().index() as u64))
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let raw = data.raw() as u32;
        (raw != u32::MAX)
            .then(|| unsafe { T::from_node_id_unchecked(NodeId::from_raw_unchecked(raw)) })
    }
}

impl<T> ExtraDataValue for SubRange<T> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_parts(self.start.index() as u32, self.end.index() as u32)
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let [start, end] = data.parts();
        Self::new(ExtraDataId::from_raw(start), ExtraDataId::from_raw(end))
    }
}

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
    use crate::{ExtraDataId, ExtraDataValue, NodeId, Program, SubRange};

    #[test]
    fn canonical_storage() {
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
