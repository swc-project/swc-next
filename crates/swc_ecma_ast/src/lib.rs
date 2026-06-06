mod ast;
mod comment;
mod common;
mod derive;
mod semantic;
mod span;
mod visit;

mod generated {
    mod ast_builder;
    mod ast_clone_in;
    mod ast_property;
    pub(crate) mod ast_visitor;
}

pub use ast::*;
pub use comment::*;
pub use common::*;
pub use generated::ast_visitor::*;
pub use semantic::{ScopeId, SymbolId};
pub use span::{DUMMY_SP, GetSpan, SetSpan, Span};
use swc_experimental_allocator::Allocator;

#[cfg(target_pointer_width = "64")]
const _: () = {
    use std::mem::size_of;

    use ast::*;

    macro_rules! assert_size {
        ($ty:ty, $size:expr) => {
            assert!(size_of::<$ty>() == $size);
        };
    }

    assert_size!(Program<'_>, 16);
    assert_size!(ModuleItem<'_>, 16);

    assert_size!(ClassMember<'_>, 16);
    assert_size!(MethodKind, 1);
    assert_size!(Key<'_>, 16);

    assert_size!(Decl<'_>, 16);
    assert_size!(VarDeclKind, 1);

    assert_size!(ParamOrTsParamProp<'_>, 8);

    assert_size!(BinaryOp, 1);
    assert_size!(AssignOp, 1);
    assert_size!(UpdateOp, 1);
    assert_size!(UnaryOp, 1);

    assert_size!(Prop<'_>, 16);
    assert_size!(PropName<'_>, 16);

    assert_size!(Lit<'_>, 16);

    assert_size!(JSXObject<'_>, 16);
    assert_size!(JSXExpr<'_>, 16);
    assert_size!(JSXElementName<'_>, 16);
    assert_size!(JSXAttrOrSpread<'_>, 16);
    assert_size!(JSXAttrName<'_>, 16);
    assert_size!(JSXAttrValue<'_>, 16);
    assert_size!(JSXElementChild<'_>, 16);

    assert_size!(ModuleDecl<'_>, 16);
    assert_size!(ImportPhase, 1);
    assert_size!(ImportSpecifier<'_>, 16);
    assert_size!(ExportSpecifier<'_>, 16);
    assert_size!(ModuleExportName<'_>, 16);
    assert_size!(DefaultDecl<'_>, 16);

    assert_size!(Expr<'_>, 16);
    assert_size!(PropOrSpread<'_>, 16);
    assert_size!(MemberProp<'_>, 16);
    assert_size!(SuperProp<'_>, 16);
    assert_size!(MetaPropKind, 1);
    assert_size!(Callee<'_>, 16);
    assert_size!(BlockStmtOrExpr<'_>, 16);
    assert_size!(AssignTarget<'_>, 16);
    assert_size!(AssignTargetPat<'_>, 16);
    assert_size!(SimpleAssignTarget<'_>, 16);
    assert_size!(OptChainBase<'_>, 16);

    assert_size!(Pat<'_>, 16);
    assert_size!(ObjectPatProp<'_>, 16);

    assert_size!(Stmt<'_>, 16);
    assert_size!(ForHead<'_>, 16);
    assert_size!(VarDeclOrExpr<'_>, 16);
};

pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}
