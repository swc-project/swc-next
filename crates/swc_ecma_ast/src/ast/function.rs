use swc_experimental_ast_macros::ast;

#[ast]
pub struct Function {
    params: Vec<Param>,
    decorators: Vec<Decorator>,
    body: BlockStmt,
    is_generator: bool,
    is_async: bool,
    // pub type_params: Option<Box<TsTypeParamDecl>>,
    // pub return_type: Option<Box<TsTypeAnn>>,
}

#[ast]
pub struct Param {
    decorators: Vec<Decorator>,
    pat: Pat,
}

#[ast]
pub enum ParamOrTsParamProp {
    // TsParamProp(TsParamProp),
    Param(Param),
}
