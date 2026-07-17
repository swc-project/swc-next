use crate::{Ast, Atom, ExtraData, TypedSubRange, Wtf8Atom, node_id::OptionalSubRange};

oxc_index::define_index_type! {
    pub struct ExtraDataId = u32;
}

/// Converts compact AST fields to and from the flattening AST's extra-data table.
pub trait ExtraDataCompact<'a>: Sized {
    fn to_extra_data(self) -> ExtraData<'a>;

    /// # Safety
    /// `data` must have been written for `Self`.
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self;
}

macro_rules! impl_extra_data_copy {
    ($ty:ty, $field:ident) => {
        impl<'a> ExtraDataCompact<'a> for $ty {
            #[inline]
            fn to_extra_data(self) -> ExtraData<'a> {
                ExtraData { $field: self }
            }

            #[inline]
            unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
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
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_span: self.into(),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        unsafe { data.optional_span.into() }
    }
}

impl<'a> ExtraDataCompact<'a> for ExtraData<'a> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        self
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        data
    }
}

impl<'a, T> ExtraDataCompact<'a> for TypedSubRange<T> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            sub_range: self.inner,
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        unsafe { data.sub_range.cast_to_typed() }
    }
}

impl<'a, T> ExtraDataCompact<'a> for Option<TypedSubRange<T>> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        let optional_sub_range = self.map_or_else(OptionalSubRange::new_none, |range| {
            OptionalSubRange::new_some(range.inner)
        });
        ExtraData { optional_sub_range }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        unsafe { data.optional_sub_range.cast_to_typed() }
    }
}

impl<'a> ExtraDataCompact<'a> for Atom<'a> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { atom: self }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        unsafe { data.atom }
    }
}

impl<'a> ExtraDataCompact<'a> for Wtf8Atom<'a> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { wtf8_atom: self }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        unsafe { data.wtf8_atom }
    }
}

impl<'a> ExtraDataCompact<'a> for Option<Atom<'a>> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            atom: self.unwrap_or(Atom::EMPTY),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        let atom = unsafe { data.atom };
        (!atom.is_empty()).then_some(atom)
    }
}

impl<'a> ExtraDataCompact<'a> for Option<Wtf8Atom<'a>> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            wtf8_atom: self.unwrap_or(Wtf8Atom::EMPTY),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        let atom = unsafe { data.wtf8_atom };
        (!atom.is_empty()).then_some(atom)
    }
}
