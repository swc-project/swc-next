use swc_core::common::Span;

use crate::{
    Accessibility, AssignOp, BigIntId, BinaryOp, ImportPhase, MetaPropKind, MethodKind,
    OptionalUtf8Ref, OptionalWtf8Ref, TruePlusMinus, TsKeywordTypeKind, TsTypeOperatorOp, UnaryOp,
    UpdateOp, Utf8Ref, VarDeclKind, Wtf8Ref,
};

use crate::{Visit, VisitMut, VisitMutWith, VisitWith};

macro_rules! dummy_visit_mut_impl {
    ($ty:ty) => {
        impl<V: ?Sized + Visit> VisitWith<V> for $ty {
            #[inline]
            fn visit_with(self, _visitor: &mut V) {}

            #[inline]
            fn visit_children_with(self, _visitor: &mut V) {}
        }

        impl<V: ?Sized + VisitMut> VisitMutWith<V> for $ty {
            #[inline]
            fn visit_mut_with(self, _visitor: &mut V) -> Self {
                self
            }

            #[inline]
            fn visit_mut_children_with(self, _visitor: &mut V) -> Self {
                self
            }
        }
    };
}

dummy_visit_mut_impl!(bool);
dummy_visit_mut_impl!(f64);
dummy_visit_mut_impl!(Span);
dummy_visit_mut_impl!(BigIntId);
dummy_visit_mut_impl!(Utf8Ref);
dummy_visit_mut_impl!(OptionalUtf8Ref);
dummy_visit_mut_impl!(Wtf8Ref);
dummy_visit_mut_impl!(OptionalWtf8Ref);
dummy_visit_mut_impl!(ImportPhase);
dummy_visit_mut_impl!(VarDeclKind);
dummy_visit_mut_impl!(UnaryOp);
dummy_visit_mut_impl!(UpdateOp);
dummy_visit_mut_impl!(BinaryOp);
dummy_visit_mut_impl!(AssignOp);
dummy_visit_mut_impl!(MetaPropKind);
dummy_visit_mut_impl!(MethodKind);
dummy_visit_mut_impl!(TsKeywordTypeKind);
dummy_visit_mut_impl!(TsTypeOperatorOp);
dummy_visit_mut_impl!(Option<TruePlusMinus>);
dummy_visit_mut_impl!(Option<Accessibility>);
