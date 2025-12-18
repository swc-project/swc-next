use std::marker::PhantomData;

use swc_experimental_ecma_ast::*;

use crate::{Parser, input::Tokens};

pub trait IsSimpleParameterList {
    fn is_simple_parameter_list(self, ast: &Ast) -> bool;
}

impl IsSimpleParameterList for TypedSubRange<Param> {
    #[inline]
    fn is_simple_parameter_list(self, ast: &Ast) -> bool {
        self.iter().all(|param| {
            let param = ast.get_node_in_sub_range(param);
            matches!(param.pat(ast), Pat::Ident(_))
        })
    }
}

impl IsSimpleParameterList for TypedSubRange<Pat> {
    #[inline]
    fn is_simple_parameter_list(self, ast: &Ast) -> bool {
        self.iter().all(|pat| {
            let pat = ast.get_node_in_sub_range(pat);
            matches!(pat, Pat::Ident(_))
        })
    }
}

impl IsSimpleParameterList for TypedSubRange<ParamOrTsParamProp> {
    #[inline]
    fn is_simple_parameter_list(self, ast: &Ast) -> bool {
        self.iter().all(|param| {
            let param = ast.get_node_in_sub_range(param);
            match param {
                ParamOrTsParamProp::Param(param) => matches!(param.pat(ast), Pat::Ident(_)),
            }
        })
    }
}

pub(crate) fn invalid_class_name(name: &str) -> bool {
    match name {
        "string" | "null" | "number" | "object" | "any" | "unknown" | "boolean" | "bigint"
        | "symbol" | "void" | "never" | "intrinsic" => true,
        _ => false,
    }
}

pub trait FromStmt: NodeIdTrait {
    fn from_stmt(stmt: Stmt) -> Self;
}

impl FromStmt for ModuleItem {
    fn from_stmt(stmt: Stmt) -> Self {
        ModuleItem::Stmt(stmt)
    }
}

impl FromStmt for Stmt {
    fn from_stmt(stmt: Stmt) -> Self {
        stmt
    }
}

pub(crate) struct ScratchIndex<N: NodeIdTrait> {
    start: usize,
    _p: PhantomData<N>,
}

impl<N: NodeIdTrait> ScratchIndex<N> {
    pub(crate) fn new(start: usize) -> Self {
        Self {
            start,
            _p: PhantomData,
        }
    }

    pub(crate) fn end<I: Tokens>(self, p: &mut Parser<I>) -> TypedSubRange<N> {
        let range = p.ast.add_typed_sub_range(&p.scratch[self.start..]);
        unsafe { p.scratch.set_len(self.start) };
        range
    }

    pub(crate) fn push<I: Tokens>(&mut self, p: &mut Parser<I>, node: N) {
        p.scratch.push(node.node_id());
    }
}

impl<I: Tokens> Parser<I> {
    pub(crate) fn is_not_this(&self, p: Param) -> bool {
        let Pat::Ident(ident) = p.pat(&self.ast) else {
            return true;
        };
        self.input
            .iter
            .read_utf8(ident.id(&self.ast).sym(&self.ast))
            != "this"
    }

    /// "IsValidSimpleAssignmentTarget" from spec.
    pub(crate) fn is_expr_valid_simple_assignment_target(&self, expr: Expr, strict: bool) -> bool {
        match expr {
            Expr::Ident(ident) => {
                if strict
                    && is_reserved_in_strict_bind(self.input.iter.read_utf8(ident.sym(&self.ast)))
                {
                    return false;
                }
                true
            }

            Expr::This(..)
            | Expr::Lit(..)
            | Expr::Array(..)
            | Expr::Object(..)
            | Expr::Fn(..)
            | Expr::Class(..)
            | Expr::Tpl(..)
            | Expr::TaggedTpl(..) => false,
            Expr::Paren(paren) => {
                self.is_expr_valid_simple_assignment_target(paren.expr(&self.ast), strict)
            }
            Expr::Member(member) => {
                let obj = member.obj(&self.ast);
                match obj {
                    Expr::Member(..) => self.is_expr_valid_simple_assignment_target(obj, strict),
                    Expr::OptChain(..) => false,
                    _ => true,
                }
            }

            Expr::SuperProp(..) => true,

            Expr::New(..) | Expr::Call(..) => false,
            // TODO: Spec only mentions `new.target`
            Expr::MetaProp(..) => false,

            Expr::Update(..) => false,

            Expr::Unary(..) | Expr::Await(..) => false,

            Expr::Bin(..) => false,

            Expr::Cond(..) => false,

            Expr::Yield(..) | Expr::Arrow(..) | Expr::Assign(..) => false,

            Expr::Seq(..) => false,

            Expr::OptChain(..) => false,

            // MemberExpression is valid assignment target
            Expr::PrivateName(..) => false,

            // jsx
            Expr::JSXMember(..)
            | Expr::JSXNamespacedName(..)
            | Expr::JSXEmpty(..)
            | Expr::JSXElement(..)
            | Expr::JSXFragment(..) => false,

            // // typescript
            // Expr::TsNonNull(TsNonNullExpr { ref expr, .. })
            // | Expr::TsTypeAssertion(TsTypeAssertion { ref expr, .. })
            // | Expr::TsAs(TsAsExpr { ref expr, .. })
            // | Expr::TsInstantiation(TsInstantiation { ref expr, .. })
            // | Expr::TsSatisfies(TsSatisfiesExpr { ref expr, .. }) => {
            //     expr.is_valid_simple_assignment_target(strict)
            // }

            // Expr::TsConstAssertion(..) => false,
            Expr::Invalid(..) => false,
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        }
    }
}
