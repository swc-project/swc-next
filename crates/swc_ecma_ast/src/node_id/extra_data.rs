use crate::{Ast, AstUtf8, AstWtf8, ExtraData, TypedSubRange};

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

impl<'a> ExtraDataCompact<'a> for AstUtf8 {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { other: self.raw() }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self::from_raw(unsafe { data.other })
    }
}

impl<'a> ExtraDataCompact<'a> for Option<AstUtf8> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            // `AstUtf8` ranges always satisfy `start <= end`, so `(u32::MAX, 0)`
            // is available as an unambiguous `None` sentinel.
            other: self.map_or(u64::from(u32::MAX), AstUtf8::raw),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let raw = unsafe { data.other };
        (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
    }
}

impl<'a> ExtraDataCompact<'a> for f64 {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            other: self.to_bits(),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self::from_bits(unsafe { data.other })
    }
}

impl<'a> ExtraDataCompact<'a> for AstWtf8 {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { other: self.raw() }
    }

    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self::from_raw(unsafe { data.other })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AstNode, BindingIdentifier, Decorator, Span};
    use swc_next_allocator::Allocator;

    #[test]
    fn optional_node_uses_explicit_eight_byte_encoding() {
        let allocator = Allocator::default();
        let mut ast = Ast::new("", &allocator);
        let decorators = ast.add_typed_sub_range(std::iter::empty::<Decorator>());
        let binding =
            ast.binding_identifier(Span::new(0, 0), AstUtf8::EMPTY, decorators, None, false);
        let node_id = binding.node_id();

        let some = Some(binding).to_extra_data(&ast);
        assert_eq!(unsafe { some.other }, ((node_id.index() as u64) << 32) | 1);
        let decoded =
            unsafe { <Option<BindingIdentifier> as ExtraDataCompact>::from_extra_data(some, &ast) };
        assert_eq!(decoded.map(AstNode::node_id), Some(node_id));

        let none = Option::<BindingIdentifier>::None.to_extra_data(&ast);
        assert_eq!(unsafe { none.other }, 0);
        let decoded =
            unsafe { <Option<BindingIdentifier> as ExtraDataCompact>::from_extra_data(none, &ast) };
        assert!(decoded.is_none());
    }
}
