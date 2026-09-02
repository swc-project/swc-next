use crate::{Ast, AstUtf8, AstWtf8, ExtraData, TypedSubRange};

use super::{NodeId, SubRange};

oxc_index::define_index_type! {
    #[repr(transparent)]
    pub struct ExtraDataId = u32;
}

#[cfg(target_endian = "little")]
impl ExtraData {
    #[inline]
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self { other: raw }
    }

    #[inline]
    pub(crate) fn raw(self) -> u64 {
        unsafe { self.other }
    }

    #[inline]
    pub(crate) fn from_node_id(node_id: NodeId) -> Self {
        let mut data = Self { other: 0 };
        data.node_id = node_id;
        data
    }

    #[inline]
    pub(crate) fn node_id(self) -> NodeId {
        unsafe { self.node_id }
    }

    #[inline]
    pub(crate) fn from_sub_range(range: SubRange) -> Self {
        Self { sub_range: range }
    }

    #[inline]
    pub(crate) fn sub_range(self) -> SubRange {
        unsafe { self.sub_range }
    }
}

#[cfg(target_endian = "big")]
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
        NodeId::from_raw_unchecked(self.0 as u32)
    }

    #[inline]
    pub(crate) fn from_sub_range(range: SubRange) -> Self {
        Self(range.start.index() as u64 | ((range.end.index() as u64) << 32))
    }

    #[inline]
    pub(crate) fn sub_range(self) -> SubRange {
        SubRange {
            start: ExtraDataId::from_raw(self.0 as u32),
            end: ExtraDataId::from_raw((self.0 >> 32) as u32),
        }
    }
}

/// Converts compact AST fields to and from the flattening AST's extra-data table.
pub trait ExtraDataCompact<'a>: Sized {
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData;

    /// # Safety
    /// `data` must have been written for `Self`.
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self;
}

impl<'a, T> ExtraDataCompact<'a> for TypedSubRange<T> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData::from_sub_range(self.inner)
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.sub_range().cast_to_typed() }
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
        let node_id = NodeId::from_raw_unchecked(0x1122_3344);
        let data = ExtraData::from_node_id(node_id);
        assert_eq!(data.raw(), 0x1122_3344);
        assert_eq!(data.node_id().index(), 0x1122_3344);

        let range = SubRange {
            start: ExtraDataId::from_raw(0x1122_3344),
            end: ExtraDataId::from_raw(0x5566_7788),
        };
        let data = ExtraData::from_sub_range(range);
        assert_eq!(data.raw(), 0x5566_7788_1122_3344);
        assert_eq!(data.sub_range().start.index(), 0x1122_3344);
        assert_eq!(data.sub_range().end.index(), 0x5566_7788);
    }
}
