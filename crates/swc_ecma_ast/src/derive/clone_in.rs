use crate::{
    AssignOp, Ast, Atom, BinaryOp, ImportPhase, MetaPropKind, MethodKind, ScopeId, Span, SymbolId,
    TypedSubRange, UnaryOp, UpdateOp, VarDeclKind, Wtf8Atom, node_id::ExtraDataCompact,
};

pub trait CloneIn<'a>: Sized {
    type Cloned;

    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned;
}

macro_rules! impl_clone_in_trivial {
    ($i:ident) => {
        impl<'a> CloneIn<'a> for $i {
            type Cloned = $i;

            fn clone_in(&self, _ast: &mut Ast<'a>) -> Self::Cloned {
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
impl_clone_in_trivial!(ScopeId);
impl_clone_in_trivial!(SymbolId);

impl<'a, C, T: CloneIn<'a, Cloned = C>> CloneIn<'a> for Vec<T> {
    type Cloned = Vec<C>;

    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let mut cloned = Vec::with_capacity(self.len());
        for item in self {
            cloned.push(item.clone_in(ast));
        }
        cloned
    }
}

impl<'a, C, T: CloneIn<'a, Cloned = C>> CloneIn<'a> for Option<T> {
    type Cloned = Option<C>;

    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in(ast))
    }
}

impl<'a, C, T> CloneIn<'a> for TypedSubRange<T>
where
    C: ExtraDataCompact<'a>,
    T: CloneIn<'a, Cloned = C> + ExtraDataCompact<'a>,
{
    type Cloned = TypedSubRange<C>;

    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let mut ids = Vec::with_capacity(self.len());
        for id in self.iter() {
            let node = ast.get_node_in_sub_range(id);
            ids.push(node.clone_in(ast));
        }
        ast.add_typed_sub_range(ids)
    }
}

impl<'a> CloneIn<'a> for Atom<'a> {
    type Cloned = Self;

    fn clone_in(&self, _ast: &mut Ast<'a>) -> Self::Cloned {
        *self
    }
}

impl<'a> CloneIn<'a> for Wtf8Atom<'a> {
    type Cloned = Self;

    fn clone_in(&self, _ast: &mut Ast<'a>) -> Self::Cloned {
        *self
    }
}
