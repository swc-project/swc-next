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
            matches!(ast.pat_data(param.pat(ast)), PatData::Ident(_))
        })
    }
}

impl IsSimpleParameterList for TypedSubRange<Pat> {
    #[inline]
    fn is_simple_parameter_list(self, ast: &Ast) -> bool {
        self.iter().all(|pat| {
            let pat = ast.get_node_in_sub_range(pat);
            matches!(ast.pat_data(pat), PatData::Ident(_))
        })
    }
}

impl IsSimpleParameterList for TypedSubRange<ParamOrTsParamProp> {
    #[inline]
    fn is_simple_parameter_list(self, ast: &Ast) -> bool {
        self.iter().all(|param| {
            let param = ast.get_node_in_sub_range(param);
            match ast.param_or_ts_param_prop_data(param) {
                ParamOrTsParamPropData::Param(param) => {
                    matches!(ast.pat_data(param.pat(ast)), PatData::Ident(_))
                }
            }
        })
    }
}

pub trait IsInvalidClassName {
    fn invalid_class_name(&self, ast: &Ast) -> Option<Span>;
}

impl IsInvalidClassName for Ident {
    fn invalid_class_name(&self, ast: &Ast) -> Option<Span> {
        match self.sym(ast).as_str() {
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
        match ast.expr_data(*self) {
            ExprData::Ident(ident) => {
                if strict && ident.is_reserved_in_strict_bind(ast) {
                    return false;
                }
                true
            }

            ExprData::This(..)
            | ExprData::Lit(..)
            | ExprData::Array(..)
            | ExprData::Object(..)
            | ExprData::Fn(..)
            | ExprData::Class(..)
            | ExprData::Tpl(..)
            | ExprData::TaggedTpl(..) => false,
            ExprData::Paren(paren) => paren
                .expr(ast)
                .is_valid_simple_assignment_target(ast, strict),
            ExprData::Member(member) => {
                let obj = member.obj(ast);
                match ast.expr_data(obj) {
                    ExprData::Member(..) => obj.is_valid_simple_assignment_target(ast, strict),
                    ExprData::OptChain(..) => false,
                    _ => true,
                }
            }

            ExprData::SuperProp(..) => true,

            ExprData::New(..) | ExprData::Call(..) | ExprData::Import(..) => false,
            // TODO: Spec only mentions `new.target`
            ExprData::MetaProp(..) => false,

            ExprData::Update(..) => false,

            ExprData::Unary(..) | ExprData::Await(..) => false,

            ExprData::Bin(..) => false,

            ExprData::Cond(..) => false,

            ExprData::Yield(..) | ExprData::Arrow(..) | ExprData::Assign(..) => false,

            ExprData::Seq(..) => false,

            ExprData::OptChain(..) => false,

            // MemberExpression is valid assignment target
            ExprData::PrivateName(..) => false,

            // jsx
            ExprData::JSXMember(..)
            | ExprData::JSXNamespacedName(..)
            | ExprData::JSXEmpty(..)
            | ExprData::JSXElement(..)
            | ExprData::JSXFragment(..) => false,

            // // typescript
            // Expr::TsNonNull(TsNonNullExpr { ref expr, .. })
            // | Expr::TsTypeAssertion(TsTypeAssertion { ref expr, .. })
            // | Expr::TsAs(TsAsExpr { ref expr, .. })
            // | Expr::TsInstantiation(TsInstantiation { ref expr, .. })
            // | Expr::TsSatisfies(TsSatisfiesExpr { ref expr, .. }) => {
            //     expr.is_valid_simple_assignment_target(strict)
            // }

            // Expr::TsConstAssertion(..) => false,
            ExprData::Invalid(..) => false,
            #[cfg(swc_ast_unknown)]
            _ => unreachable!(),
        }
    }
}

pub trait FromStmt<'a>: ExtraDataCompact<'a> {
    fn from_stmt(stmt: Stmt) -> Self;
}

impl<'a> FromStmt<'a> for ModuleItem {
    fn from_stmt(stmt: Stmt) -> Self {
        ModuleItem::Stmt(stmt)
    }
}

impl<'a> FromStmt<'a> for Stmt {
    fn from_stmt(stmt: Stmt) -> Self {
        stmt
    }
}

pub(crate) struct ScratchIndex<N> {
    start: usize,
    _p: PhantomData<N>,
}

impl<N> ScratchIndex<N> {
    pub(crate) fn new(start: usize) -> Self {
        Self {
            start,
            _p: PhantomData,
        }
    }

    pub(crate) fn end<'p, 'a, I: Tokens<'a>>(self, p: &mut Parser<'p, 'a, I>) -> TypedSubRange<N>
    where
        N: ExtraDataCompact<'a>,
    {
        let range = p.ast.add_extra_data_sub_range(&p.scratch[self.start..]);
        p.scratch.truncate(self.start);
        unsafe { std::mem::transmute(range) }
    }

    pub(crate) fn push<'p, 'a, I: Tokens<'a>>(&mut self, p: &mut Parser<'p, 'a, I>, node: N)
    where
        N: ExtraDataCompact<'a>,
    {
        let data = node.to_extra_data(p.ast);
        p.scratch.push(data);
    }
}
