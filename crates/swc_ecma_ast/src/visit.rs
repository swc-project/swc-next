use crate::{
    AssignOp, BinaryOp, ImportPhase, MetaPropKind, MethodKind, Span, UnaryOp, UpdateOp, VarDeclKind,
};

use crate::{Visit, VisitMut, VisitMutWith, VisitWith};
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};

macro_rules! noop_visit_impl {
    ($ty:ty) => {
        impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for $ty {
            #[inline]
            fn visit_with(&self, _visitor: &mut V) {}

            #[inline]
            fn visit_children_with(&self, _visitor: &mut V) {}
        }

        impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for $ty {
            #[inline]
            fn visit_mut_with(&mut self, _visitor: &mut V) {}

            #[inline]
            fn visit_mut_children_with(&mut self, _visitor: &mut V) {}
        }
    };
}

noop_visit_impl!(bool);
noop_visit_impl!(f64);
noop_visit_impl!(Span);
noop_visit_impl!(Atom<'_>);
noop_visit_impl!(Wtf8Atom<'_>);
noop_visit_impl!(ImportPhase);
noop_visit_impl!(VarDeclKind);
noop_visit_impl!(UnaryOp);
noop_visit_impl!(UpdateOp);
noop_visit_impl!(BinaryOp);
noop_visit_impl!(AssignOp);
noop_visit_impl!(MetaPropKind);
noop_visit_impl!(MethodKind);
