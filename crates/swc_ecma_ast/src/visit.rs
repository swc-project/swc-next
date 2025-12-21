use crate::{
    AssignOp, BigIntId, BinaryOp, ImportPhase, MetaPropKind, MethodKind, OptionalUtf8Ref,
    OptionalWtf8Ref, UnaryOp, UpdateOp, Utf8Ref, VarDeclKind, Wtf8Ref,
};
use swc_core::common::Span;

use crate::{Visit, VisitMut, VisitMutWith, VisitWith};

macro_rules! dummy_visit_mut_impl {
    ($ident:ident) => {
        impl<V: ?Sized + Visit> VisitWith<V> for $ident {
            #[inline]
            fn visit_with(self, _visitor: &mut V) {}

            #[inline]
            fn visit_children_with(self, _visitor: &mut V) {}
        }

        impl<V: ?Sized + VisitMut> VisitMutWith<V> for $ident {
            #[inline]
            fn visit_mut_with(self, _visitor: &mut V) -> Self {
                self
            }

            #[inline]
            fn visit_mut_children_with(self, _visitor: &mut V) {}
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
