use crate::{Ast, Atom, ExtraData, TypedSubRange, Wtf8Atom};

oxc_index::define_index_type! {
    pub struct ExtraDataId = u32;
}

/// Converts compact AST fields to and from the flattening AST's extra-data table.
pub trait ExtraDataCompact<'a>: Sized {
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData;

    /// # Safety
    /// `data` must have been written for `Self`.
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self;
}

macro_rules! impl_extra_data_copy {
    ($ty:ty, $field:ident) => {
        impl<'a> ExtraDataCompact<'a> for $ty {
            #[inline]
            fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
                ExtraData { $field: self }
            }

            #[inline]
            unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
                unsafe { data.$field }
            }
        }
    };
}

impl_extra_data_copy!(Option<crate::ScopeId>, optional_scope_id);
impl_extra_data_copy!(Option<crate::SymbolId>, optional_symbol_id);

impl<'a, T> ExtraDataCompact<'a> for TypedSubRange<T> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            sub_range: self.inner,
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.sub_range.cast_to_typed() }
    }
}

impl<'a> ExtraDataCompact<'a> for Atom<'a> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            atom: ast.store_atom(self),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        ast.stored_atom(unsafe { data.atom })
    }
}

impl<'a> ExtraDataCompact<'a> for Wtf8Atom<'a> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            wtf8_atom: ast.store_wtf8_atom(self),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        ast.stored_wtf8_atom(unsafe { data.wtf8_atom })
    }
}
