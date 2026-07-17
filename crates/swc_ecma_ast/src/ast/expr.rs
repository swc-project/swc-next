use std::mem;

use swc_experimental_ast_macros::ast;

use crate::{Ast, ExtraData, GetSpan, ast::*, node_id::ExtraDataCompact};

#[ast]
pub enum Expr {
    This(ThisExpr),
    Array(ArrayLit),
    Object(ObjectLit),
    Fn(FnExpr),
    Unary(UnaryExpr),
    Update(UpdateExpr),
    Bin(BinExpr),
    Assign(AssignExpr),
    Member(MemberExpr),
    SuperProp(SuperPropExpr),
    Cond(CondExpr),
    Import(ImportExpr),
    Call(CallExpr),
    New(NewExpr),
    Seq(SeqExpr),
    Ident(Ident),
    Lit(Lit),
    Tpl(Tpl),
    TaggedTpl(TaggedTpl),
    Arrow(ArrowExpr),
    Class(ClassExpr),
    Yield(YieldExpr),
    MetaProp(MetaPropExpr),
    Await(AwaitExpr),
    Paren(ParenExpr),
    JSXMember(JSXMemberExpr),
    JSXNamespacedName(JSXNamespacedName),
    JSXEmpty(JSXEmptyExpr),
    JSXElement(JSXElement),
    JSXFragment(JSXFragment),
    // TsTypeAssertion(TsTypeAssertion),
    // TsConstAssertion(TsConstAssertion),
    // TsNonNull(TsNonNullExpr),
    // TsAs(TsAsExpr),
    // TsInstantiation(TsInstantiation),
    // TsSatisfies(TsSatisfiesExpr),
    PrivateName(PrivateName),
    OptChain(OptChainExpr),
    Invalid(Invalid),
}

#[ast]
pub struct ThisExpr {}

#[ast]
pub struct ArrayLit {
    elems: Vec<Option<ExprOrSpread>>,
}

#[ast]
pub struct ObjectLit {
    props: Vec<PropOrSpread>,
}

#[ast]
pub enum PropOrSpread {
    Spread(SpreadElement),
    Prop(Prop),
}

#[ast]
pub struct SpreadElement {
    dot3_token: Span,
    expr: Expr,
}

#[ast]
pub struct UnaryExpr {
    op: UnaryOp,
    arg: Expr,
}

#[ast]
pub struct UpdateExpr {
    op: UpdateOp,
    prefix: bool,
    arg: SimpleAssignTarget,
}

#[ast]
pub struct BinExpr {
    op: BinaryOp,
    left: Expr,
    right: Expr,
}
#[ast]
pub struct FnExpr {
    ident: Option<Ident>,
    function: Function,
}

#[ast]
pub struct ClassExpr {
    ident: Option<Ident>,
    class: Class,
}

#[ast]
pub struct AssignExpr {
    op: AssignOp,
    left: AssignTarget,
    right: Expr,
}

#[ast]
pub struct MemberExpr {
    obj: Expr,
    prop: MemberProp,
}

#[ast]
pub enum MemberProp {
    Ident(IdentName),
    PrivateName(PrivateName),
    Computed(ComputedPropName),
}

impl MemberProp {
    pub fn is_ident_with(self, ast: &Ast<'_>, sym: &str) -> bool {
        matches!(self, MemberProp::Ident(i) if i.sym(ast).as_str(ast) == sym)
    }
}

#[ast]
pub struct SuperPropExpr {
    obj: Super,
    prop: SuperProp,
}

#[ast]
pub enum SuperProp {
    Ident(IdentName),
    Computed(ComputedPropName),
}

#[ast]
pub struct CondExpr {
    test: Expr,
    cons: Expr,
    alt: Expr,
}

#[ast]
pub struct ImportExpr {
    import_span: Span,
    source: Expr,
    options: Option<Expr>,
    phase: ImportPhase,
}

#[ast]
pub struct CallExpr {
    callee: Callee,
    args: Vec<ExprOrSpread>,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
    // pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast]
pub struct NewExpr {
    callee: Expr,
    args: Vec<ExprOrSpread>,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
    // pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast]
pub struct SeqExpr {
    exprs: Vec<Expr>,
}

#[ast]
pub struct ArrowExpr {
    params: Vec<Pat>,
    body: BlockStmtOrExpr,
    is_async: bool,
    // type_params: Option<Box<TsTypeParamDecl>>,
    // return_type: Option<Box<TsTypeAnn>>,
}

#[ast]
pub struct YieldExpr {
    arg: Option<Expr>,
    delegate: bool,
}

#[ast]
pub struct MetaPropExpr {
    kind: MetaPropKind,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MetaPropKind {
    /// `new.target`
    NewTarget,
    /// `import.meta`
    ImportMeta,
}

impl<'a> ExtraDataCompact<'a> for MetaPropKind {
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData<'a>, _ast: &Ast<'a>) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub struct AwaitExpr {
    arg: Expr,
}

#[ast]
pub struct Tpl {
    exprs: Vec<Expr>,
    quasis: Vec<TplElement>,
}

#[ast]
pub struct TaggedTpl {
    tag: Expr,
    // type_params: Option<TsTypeParamInstantiation>,
    tpl: Tpl,
}

#[ast]
pub struct TplElement {
    tail: bool,
    cooked: Option<Wtf8Atom>,
    raw: Atom,
}

#[ast]
pub struct ParenExpr {
    expr: Expr,
}

#[ast]
pub enum Callee {
    Super(Super),
    Expr(Expr),
}

#[ast]
pub struct Super {}

#[ast]
pub struct ExprOrSpread {
    spread: Option<Span>,
    expr: Expr,
}

#[ast]
pub enum BlockStmtOrExpr {
    BlockStmt(BlockStmt),
    Expr(Expr),
}

#[ast]
pub enum AssignTarget {
    Simple(SimpleAssignTarget),
    Pat(AssignTargetPat),
}

#[ast]
pub enum AssignTargetPat {
    Array(ArrayPat),
    Object(ObjectPat),
    Invalid(Invalid),
}

#[ast]
pub enum SimpleAssignTarget {
    Ident(BindingIdent),
    Member(MemberExpr),
    SuperProp(SuperPropExpr),
    Paren(ParenExpr),
    OptChain(OptChainExpr),
    // TsAs(TsAsExpr),
    // TsSatisfies(TsSatisfiesExpr),
    // TsNonNull(TsNonNullExpr),
    // TsTypeAssertion(TsTypeAssertion),
    // TsInstantiation(TsInstantiation),
    Invalid(Invalid),
}

#[ast]
pub struct OptChainExpr {
    optional: bool,
    base: OptChainBase,
}

#[ast]
pub enum OptChainBase {
    Member(MemberExpr),
    Call(OptCall),
}

#[ast]
pub struct OptCall {
    callee: Expr,
    args: Vec<ExprOrSpread>,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
    // pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast]
pub struct Invalid {}

impl Expr {
    pub fn is_ident_ref_to<S>(self, ast: &Ast<'_>, ident: &S) -> bool
    where
        S: ?Sized + AsRef<str>,
    {
        match self {
            Expr::Ident(i) => i.sym(ast).as_str(ast) == ident.as_ref(),
            _ => false,
        }
    }
}

impl AssignTarget {
    pub fn try_from_pat(ast: &mut Ast<'_>, p: Pat) -> Result<Self, Pat> {
        Ok(match p {
            Pat::Array(a) => AssignTarget::Pat(AssignTargetPat::Array(a)),
            Pat::Object(o) => AssignTarget::Pat(AssignTargetPat::Object(o)),
            Pat::Ident(i) => AssignTarget::Simple(SimpleAssignTarget::Ident(i)),
            Pat::Invalid(i) => AssignTarget::Simple(SimpleAssignTarget::Invalid(i)),
            Pat::Expr(e) => match Self::try_from_expr(ast, e) {
                Ok(v) => v,
                Err(e) => return Err(Pat::Expr(e)),
            },
            _ => return Err(p),
        })
    }
}

impl AssignTarget {
    pub fn try_from_expr(ast: &mut Ast<'_>, e: Expr) -> Result<Self, Expr> {
        Ok(Self::Simple(SimpleAssignTarget::try_from_expr(ast, e)?))
    }
}

impl SimpleAssignTarget {
    pub fn try_from_expr(ast: &mut Ast<'_>, e: Expr) -> Result<Self, Expr> {
        Ok(match e {
            Expr::Ident(i) => SimpleAssignTarget::Ident(ast.binding_ident(i.span(ast), i)),
            Expr::Member(m) => SimpleAssignTarget::Member(m),
            Expr::SuperProp(s) => SimpleAssignTarget::SuperProp(s),
            Expr::OptChain(s) => SimpleAssignTarget::OptChain(s),
            Expr::Paren(s) => SimpleAssignTarget::Paren(s),
            // Expr::TsAs(a) => SimpleAssignTarget::TsAs(a),
            // Expr::TsSatisfies(s) => SimpleAssignTarget::TsSatisfies(s),
            // Expr::TsNonNull(n) => SimpleAssignTarget::TsNonNull(n),
            // Expr::TsTypeAssertion(a) => SimpleAssignTarget::TsTypeAssertion(a),
            // Expr::TsInstantiation(a) => SimpleAssignTarget::TsInstantiation(a),
            _ => return Err(e),
        })
    }
}
