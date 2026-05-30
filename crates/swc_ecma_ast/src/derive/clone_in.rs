use swc_experimental_allocator::{Allocator, CloneIn};

use crate::{
    AssignOp, BinaryOp, ImportPhase, MetaPropKind, MethodKind, ScopeId, Span, SymbolId, UnaryOp,
    UpdateOp, VarDeclKind,
};

macro_rules! impl_clone_in_trivial {
    ($i:ident) => {
        impl<'a> CloneIn<'a> for $i {
            type Cloned = $i;

            fn clone_in(&self, _allocator: &'a Allocator) -> Self::Cloned {
                self.clone()
            }
        }
    };
}

impl_clone_in_trivial!(Span);
impl_clone_in_trivial!(UnaryOp);
impl_clone_in_trivial!(UpdateOp);
impl_clone_in_trivial!(BinaryOp);
impl_clone_in_trivial!(AssignOp);
impl_clone_in_trivial!(MetaPropKind);
impl_clone_in_trivial!(ImportPhase);
impl_clone_in_trivial!(VarDeclKind);
impl_clone_in_trivial!(MethodKind);
impl_clone_in_trivial!(ScopeId);
impl_clone_in_trivial!(SymbolId);
