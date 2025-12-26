use crate::{Ast, ExtraData};

oxc_index::define_index_type! {
    pub struct ExtraDataId = u32;
}

/// Some AST field can be represented as a single u64 value to store it as [crate::ExtraData],
/// and this trait is used to convert between the AST field and the u64 value.
pub(crate) trait ExtraDataCompact: Sized {
    fn to_extra_data(self) -> ExtraData;
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self;
}
