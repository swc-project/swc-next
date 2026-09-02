use crate::{Ast, AstUtf8, AstWtf8, ExtraData, SubRange};

use super::NodeId;

oxc_index::define_index_type! {
    #[repr(transparent)]
    pub struct ExtraDataId = u32;
}

impl ExtraData {
    #[inline]
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[inline]
    pub(crate) fn raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub(crate) fn from_node_id(node_id: NodeId) -> Self {
        Self(node_id.index() as u64)
    }

    #[inline]
    pub(crate) fn node_id(self) -> NodeId {
        unsafe { NodeId::from_raw_unchecked(self.0 as u32) }
    }

    /// Store an optional node ID using `u32::MAX` as the external `None`
    /// sentinel rather than `NodeId`'s internal non-max representation.
    #[inline]
    pub(crate) fn from_optional_node_id(node_id: Option<NodeId>) -> Self {
        Self(node_id.map_or(u64::from(u32::MAX), |node_id| node_id.index() as u64))
    }

    #[inline]
    pub(crate) fn optional_node_id(self) -> Option<NodeId> {
        let raw = self.0 as u32;
        (raw != u32::MAX).then(|| unsafe { NodeId::from_raw_unchecked(raw) })
    }
}

/// Converts compact AST fields to and from the flattening AST's extra-data table.
pub trait ExtraDataCompact<'a>: Sized {
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData;

    /// # Safety
    /// `data` must have been written for `Self`.
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self;
}

impl<'a, T> ExtraDataCompact<'a> for SubRange<T> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self::from_raw(data.raw())
    }
}

impl<'a> ExtraDataCompact<'a> for AstUtf8 {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self::from_raw(data.raw())
    }
}

impl<'a> ExtraDataCompact<'a> for Option<AstUtf8> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        // `AstUtf8` ranges always satisfy `start <= end`, so `(u32::MAX, 0)`
        // is available as an unambiguous `None` sentinel.
        ExtraData::from_raw(self.map_or(u64::from(u32::MAX), AstUtf8::raw))
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let raw = data.raw();
        (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
    }
}

impl<'a> ExtraDataCompact<'a> for f64 {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData::from_raw(self.to_bits())
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self::from_bits(data.raw())
    }
}

impl<'a> ExtraDataCompact<'a> for AstWtf8 {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self::from_raw(data.raw())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExtraData, ExtraDataId, NodeId, SubRange};

    #[test]
    fn canonical_storage() {
        let node_id = unsafe { NodeId::from_raw_unchecked(0x1122_3344) };
        let data = ExtraData::from_node_id(node_id);
        assert_eq!(data.raw(), 0x1122_3344);
        assert_eq!(data.node_id().index(), 0x1122_3344);

        let range = SubRange::<()> {
            start: ExtraDataId::from_raw(0x1122_3344),
            end: ExtraDataId::from_raw(0x5566_7788),
            _phantom: std::marker::PhantomData,
        };
        let data = ExtraData::from_raw(range.raw());
        assert_eq!(data.raw(), 0x5566_7788_1122_3344);
        let decoded = SubRange::<()>::from_raw(data.raw());
        assert_eq!(decoded.start.index(), 0x1122_3344);
        assert_eq!(decoded.end.index(), 0x5566_7788);
    }
}
