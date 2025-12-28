use swc_experimental_ast_macros::ast;

use crate::TsParamProp;

#[ast]
pub struct Function {
    params: Vec<Param>,
    decorators: Vec<Decorator>,
    body: Option<BlockStmt>,
    is_generator: bool,
    is_async: bool,

    // Typescript
    type_params: Option<TsTypeParamDecl>,
    return_type: Option<TsTypeAnn>,
}

#[ast]
pub struct Param {
    decorators: Vec<Decorator>,
    pat: Pat,
}

#[ast]
pub enum ParamOrTsParamProp {
    Param(Param),
    // Typescript
    TsParamProp(TsParamProp),
}
