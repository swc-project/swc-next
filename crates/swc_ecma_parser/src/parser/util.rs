use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ecma_ast::*;

pub trait IsSimpleParameterList {
    fn is_simple_parameter_list(&self) -> bool;
}

impl<'a> IsSimpleParameterList for Vec<'a, Param<'a>> {
    #[inline]
    fn is_simple_parameter_list(&self) -> bool {
        self.iter().all(|param| matches!(param.pat, Pat::Ident(_)))
    }
}

impl<'a> IsSimpleParameterList for Vec<'a, Pat<'a>> {
    #[inline]
    fn is_simple_parameter_list(&self) -> bool {
        self.iter().all(|pat| matches!(pat, Pat::Ident(_)))
    }
}

impl<'a> IsSimpleParameterList for Vec<'a, ParamOrTsParamProp<'a>> {
    #[inline]
    fn is_simple_parameter_list(&self) -> bool {
        self.iter().all(|param| match param {
            ParamOrTsParamProp::Param(param) => matches!(param.pat, Pat::Ident(_)),
        })
    }
}

pub trait IsInvalidClassName {
    fn invalid_class_name(&self) -> Option<Span>;
}

impl IsInvalidClassName for Ident<'_> {
    fn invalid_class_name(&self) -> Option<Span> {
        match self.sym.as_str() {
            "string" | "null" | "number" | "object" | "any" | "unknown" | "boolean" | "bigint"
            | "symbol" | "void" | "never" | "intrinsic" => Some(self.span),
            _ => None,
        }
    }
}

impl IsInvalidClassName for Option<Box<'_, Ident<'_>>> {
    fn invalid_class_name(&self) -> Option<Span> {
        self.as_ref().and_then(|i| i.invalid_class_name())
    }
}

impl IsInvalidClassName for Option<Ident<'_>> {
    fn invalid_class_name(&self) -> Option<Span> {
        self.as_ref().and_then(|i| i.invalid_class_name())
    }
}

pub trait ExprExt {
    fn is_valid_simple_assignment_target(&self, strict: bool) -> bool;
}

impl ExprExt for Expr<'_> {
    /// "IsValidSimpleAssignmentTarget" from spec.
    fn is_valid_simple_assignment_target(&self, strict: bool) -> bool {
        match self {
            Expr::Ident(ident) => {
                if strict && ident.is_reserved_in_strict_bind() {
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
            Expr::Paren(paren) => paren.expr.is_valid_simple_assignment_target(strict),
            Expr::Member(member) => match &member.obj {
                Expr::Member(..) => member.obj.is_valid_simple_assignment_target(strict),
                Expr::OptChain(..) => false,
                _ => true,
            },

            Expr::SuperProp(..) => true,
            Expr::New(..) | Expr::Call(..) => false,
            Expr::MetaProp(..) => false,
            Expr::Update(..) => false,
            Expr::Unary(..) | Expr::Await(..) => false,
            Expr::Bin(..) => false,
            Expr::Cond(..) => false,
            Expr::Yield(..) | Expr::Arrow(..) | Expr::Assign(..) => false,
            Expr::Seq(..) => false,
            Expr::OptChain(..) => false,
            Expr::PrivateName(..) => false,
            Expr::JSXMember(..)
            | Expr::JSXNamespacedName(..)
            | Expr::JSXEmpty(..)
            | Expr::JSXElement(..)
            | Expr::JSXFragment(..) => false,
            Expr::Invalid(..) => false,
        }
    }
}

pub trait FromStmt<'a>: Sized {
    fn from_stmt(ast: &AstBuilder<'a>, stmt: Stmt<'a>) -> Self;
}

impl<'a> FromStmt<'a> for ModuleItem<'a> {
    fn from_stmt(ast: &AstBuilder<'a>, stmt: Stmt<'a>) -> Self {
        ModuleItem::Stmt(ast.allocator.boxed(stmt))
    }
}

impl<'a> FromStmt<'a> for Stmt<'a> {
    fn from_stmt(_: &AstBuilder<'a>, stmt: Stmt<'a>) -> Self {
        stmt
    }
}
