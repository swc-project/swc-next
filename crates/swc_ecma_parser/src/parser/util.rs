use std::marker::PhantomData;

use swc_core::common::Span;
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

pub trait IsInvalidClassName {
    fn invalid_class_name(&self, ast: &Ast) -> Option<Span>;
}

impl IsInvalidClassName for Ident {
    fn invalid_class_name(&self, ast: &Ast) -> Option<Span> {
        match ast.get_utf8(self.sym(ast)) {
            "string" | "null" | "number" | "object" | "any" | "unknown" | "boolean" | "bigint"
            | "symbol" | "void" | "never" | "intrinsic" => Some(self.span(ast)),
            _ => None,
        }
    }
}
impl IsInvalidClassName for Option<Ident> {
    fn invalid_class_name(&self, ast: &Ast) -> Option<Span> {
        self.as_ref().and_then(|i| i.invalid_class_name(ast))
    }
}

pub trait ExprExt {
    fn is_valid_simple_assignment_target(&self, ast: &Ast, strict: bool) -> bool;
}

impl ExprExt for Expr {
    /// "IsValidSimpleAssignmentTarget" from spec.
    fn is_valid_simple_assignment_target(&self, ast: &Ast, strict: bool) -> bool {
        match self {
            Expr::Ident(ident) => {
                if strict && ident.is_reserved_in_strict_bind(ast) {
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
            Expr::Paren(paren) => paren
                .expr(ast)
                .is_valid_simple_assignment_target(ast, strict),
            Expr::Member(member) => {
                let obj = member.obj(ast);
                match obj {
                    Expr::Member(..) => obj.is_valid_simple_assignment_target(ast, strict),
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

pub trait FromStmt: ExtraDataCompact {
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

pub(crate) struct ScratchIndex<N: ExtraDataCompact> {
    start: usize,
    _p: PhantomData<N>,
}

impl<N: ExtraDataCompact> ScratchIndex<N> {
    pub(crate) fn new(start: usize) -> Self {
        Self {
            start,
            _p: PhantomData,
        }
    }

    pub(crate) fn end<I: Tokens>(self, p: &mut Parser<I>) -> TypedSubRange<N> {
        let range = p.ast.add_typed_sub_range(p.scratch.drain(self.start..));
        unsafe { std::mem::transmute(range) }
    }

    pub(crate) fn push<I: Tokens>(&mut self, p: &mut Parser<I>, node: N) {
        p.scratch.push(node.to_extra_data());
    }
}
