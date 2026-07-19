use crate::{
    AssignOp, Atom, BinaryOp, ImportPhase, MetaPropKind, MethodKind, ScopeId, Span, SymbolId,
    UnaryOp, UpdateOp, VarDeclKind, Visit, VisitMut, VisitMutWith, VisitWith, Wtf8Atom,
};

macro_rules! dummy_visit_mut_impl {
    ($ty:ty) => {
        impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for $ty {
            #[inline]
            fn visit_with(self, _visitor: &mut V) {}

            #[inline]
            fn visit_children_with(self, _visitor: &mut V) {}
        }

        impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for $ty {
            #[inline]
            fn visit_mut_with(self, _visitor: &mut V) {}

            #[inline]
            fn visit_mut_children_with(self, _visitor: &mut V) {}
        }
    };
}

dummy_visit_mut_impl!(bool);
dummy_visit_mut_impl!(f64);
dummy_visit_mut_impl!(Span);
dummy_visit_mut_impl!(Atom<'a>);
dummy_visit_mut_impl!(Wtf8Atom<'a>);
dummy_visit_mut_impl!(ScopeId);
dummy_visit_mut_impl!(SymbolId);
dummy_visit_mut_impl!(ImportPhase);
dummy_visit_mut_impl!(VarDeclKind);
dummy_visit_mut_impl!(UnaryOp);
dummy_visit_mut_impl!(UpdateOp);
dummy_visit_mut_impl!(BinaryOp);
dummy_visit_mut_impl!(AssignOp);
dummy_visit_mut_impl!(MetaPropKind);
dummy_visit_mut_impl!(MethodKind);
