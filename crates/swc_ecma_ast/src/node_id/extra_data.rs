use crate::{Ast, ExtraData, TypedSubRange, node_id::OptionalSubRange};

oxc_index::define_index_type! {
    pub struct ExtraDataId = u32;
}

/// Some AST field can be represented as a single u64 value to store it as [crate::ExtraData],
/// and this trait is used to convert between the AST field and the u64 value.
pub(crate) trait ExtraDataCompact: Sized {
    fn to_extra_data(self) -> ExtraData;
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self;
}

impl ExtraDataCompact for crate::Span {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { span: self }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.span }
    }
}

impl ExtraDataCompact for bool {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { bool: self }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.bool }
    }
}

impl ExtraDataCompact for f64 {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { number: self }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.number }
    }
}

impl<T> ExtraDataCompact for crate::TypedSubRange<T> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            sub_range: self.inner,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe {
            let sub_range = data.sub_range;
            sub_range.cast_to_typed()
        }
    }
}

impl<T> ExtraDataCompact for Option<TypedSubRange<T>> {
    fn to_extra_data(self) -> ExtraData {
        let optional_sub_range = match self {
            Some(it) => OptionalSubRange::new_some(it.inner),
            None => OptionalSubRange::new_none(),
        };
        ExtraData { optional_sub_range }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.optional_sub_range.cast_to_typed() }
    }
}

impl ExtraDataCompact for crate::Utf8Ref {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { utf8: self }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.utf8 }
    }
}

impl ExtraDataCompact for crate::Wtf8Ref {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { wtf8: self }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.wtf8 }
    }
}

impl ExtraDataCompact for crate::OptionalUtf8Ref {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_utf8: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.optional_utf8 }
    }
}

impl ExtraDataCompact for crate::OptionalWtf8Ref {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_wtf8: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.optional_wtf8 }
    }
}

impl ExtraDataCompact for crate::NodeId {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { node: self }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.node }
    }
}

impl ExtraDataCompact for crate::BigIntId {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { bigint: self }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.bigint }
    }
}

impl ExtraDataCompact for crate::OptionalNodeId {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self,
        }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.optional_node }
    }
}

impl ExtraDataCompact for u64 {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self }
    }
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { data.other }
    }
}
