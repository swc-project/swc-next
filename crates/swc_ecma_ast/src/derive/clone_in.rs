use swc_core::common::Span;

use crate::{
    AssignOp, Ast, BigIntId, BinaryOp, ImportPhase, MetaPropKind, MethodKind, OptionalUtf8Ref,
    OptionalWtf8Ref, TypedSubRange, UnaryOp, UpdateOp, Utf8Ref, VarDeclKind, Wtf8Ref,
    node_id::ExtraDataCompact,
};

pub trait CloneIn: Sized {
    type Cloned;

    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned;
}

macro_rules! impl_clone_in_trivial {
    ($i:ident) => {
        impl CloneIn for $i {
            type Cloned = $i;

            fn clone_in(&self, _ast: &mut Ast) -> Self::Cloned {
                self.clone()
            }
        }
    };
}

impl_clone_in_trivial!(Span);
impl_clone_in_trivial!(bool);
impl_clone_in_trivial!(f64);
impl_clone_in_trivial!(UnaryOp);
impl_clone_in_trivial!(UpdateOp);
impl_clone_in_trivial!(BinaryOp);
impl_clone_in_trivial!(AssignOp);
impl_clone_in_trivial!(MetaPropKind);
impl_clone_in_trivial!(ImportPhase);
impl_clone_in_trivial!(VarDeclKind);
impl_clone_in_trivial!(MethodKind);
impl_clone_in_trivial!(BigIntId);
impl_clone_in_trivial!(Utf8Ref);
impl_clone_in_trivial!(Wtf8Ref);
impl_clone_in_trivial!(OptionalUtf8Ref);
impl_clone_in_trivial!(OptionalWtf8Ref);

impl<C, T: CloneIn<Cloned = C>> CloneIn for Vec<T> {
    type Cloned = Vec<C>;

    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let mut cloned = Vec::with_capacity(self.len());
        for item in self {
            cloned.push(item.clone_in(ast));
        }
        cloned
    }
}

impl<C, T: CloneIn<Cloned = C>> CloneIn for Option<T> {
    type Cloned = Option<C>;

    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in(ast))
    }
}

impl<C: ExtraDataCompact, T: CloneIn<Cloned = C> + ExtraDataCompact> CloneIn for TypedSubRange<T> {
    type Cloned = TypedSubRange<C>;

    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let mut ids = Vec::with_capacity(self.len());
        for id in self.iter() {
            let node = ast.get_node_in_sub_range(id);
            ids.push(node.clone_in(ast));
        }
        ast.add_typed_sub_range(ids)
    }
}
