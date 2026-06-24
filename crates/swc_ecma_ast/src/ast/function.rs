use crate::Span;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::ast::{BlockStmt, Decorator, Pat};

#[ast]
#[derive(Debug)]
pub struct Function<'a> {
    pub span: Span,
    pub params: Vec<'a, Param<'a>>,
    pub decorators: Vec<'a, Decorator<'a>>,
    pub body: Box<'a, BlockStmt<'a>>,
    pub is_generator: bool,
    pub is_async: bool,
    // pub type_params: Option<Box<TsTypeParamDecl>>,
    // pub return_type: Option<Box<TsTypeAnn>>,
}

#[ast]
#[derive(Debug)]
pub struct Param<'a> {
    pub span: Span,
    pub decorators: Vec<'a, Decorator<'a>>,
    pub pat: Pat<'a>,
}

#[ast]
#[derive(Debug)]
pub enum ParamOrTsParamProp<'a> {
    // TsParamProp(TsParamProp),
    Param(Box<'a, Param<'a>>),
}
