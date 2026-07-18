use crate::{
    Ast, Atom, ExtraData, TypedSubRange, Wtf8Atom, node_id::OptionalSubRange, strings::AtomStore,
};

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

impl_extra_data_copy!(crate::Span, span);
impl_extra_data_copy!(bool, bool);
impl_extra_data_copy!(f64, number);
impl_extra_data_copy!(crate::NodeId, node);
impl_extra_data_copy!(crate::OptionalNodeId, optional_node);
impl_extra_data_copy!(u64, other);
impl_extra_data_copy!(Option<crate::ScopeId>, optional_scope_id);
impl_extra_data_copy!(Option<crate::SymbolId>, optional_symbol_id);

impl<'a> ExtraDataCompact<'a> for Option<crate::Span> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_span: self.into(),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.optional_span.into() }
    }
}

impl<'a> ExtraDataCompact<'a> for ExtraData {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        self
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        data
    }
}

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

impl<'a, T> ExtraDataCompact<'a> for Option<TypedSubRange<T>> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        let optional_sub_range = self.map_or_else(OptionalSubRange::new_none, |range| {
            OptionalSubRange::new_some(range.inner)
        });
        ExtraData { optional_sub_range }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.optional_sub_range.cast_to_typed() }
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

impl<'a> ExtraDataCompact<'a> for Option<Atom<'a>> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            atom: self.map_or_else(AtomStore::none_atom, |atom| ast.store_atom(atom)),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        let id = unsafe { data.atom };
        (!AtomStore::is_none_atom(id)).then(|| ast.stored_atom(id))
    }
}

impl<'a> ExtraDataCompact<'a> for Option<Wtf8Atom<'a>> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            wtf8_atom: self
                .map_or_else(AtomStore::none_wtf8_atom, |atom| ast.store_wtf8_atom(atom)),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        let id = unsafe { data.wtf8_atom };
        (!AtomStore::is_none_wtf8_atom(id)).then(|| ast.stored_wtf8_atom(id))
    }
}
