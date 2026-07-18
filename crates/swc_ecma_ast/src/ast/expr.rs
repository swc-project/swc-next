use std::mem;

use swc_experimental_ast_macros::ast;

use crate::{Ast, ExtraData, GetSpan, ast::*, node_id::ExtraDataCompact};

#[ast(repr)]
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
    Str(Str),
    Bool(Bool),
    Null(Null),
    Num(Number),
    BigInt(BigInt),
    Regex(Regex),
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

#[allow(non_snake_case)]
impl Expr {
    /// Compatibility constructor for code which produces a generic literal.
    pub fn Lit(lit: Lit) -> Self {
        match lit {
            Lit::Str(node) => Self::Str(node),
            Lit::Bool(node) => Self::Bool(node),
            Lit::Null(node) => Self::Null(node),
            Lit::Num(node) => Self::Num(node),
            Lit::BigInt(node) => Self::BigInt(node),
            Lit::Regex(node) => Self::Regex(node),
        }
    }
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

#[ast(repr)]
pub enum PropOrSpread {
    // Prop
    Shorthand(Ident),
    KeyValue(KeyValueProp),
    Assign(AssignProp),
    Getter(GetterProp),
    Setter(SetterProp),
    Method(MethodProp),

    // PropOrSpread
    Spread(SpreadElement),
}

#[allow(non_snake_case)]
impl PropOrSpread {
    #[inline]
    pub fn Prop(prop: Prop) -> Self {
        const {
            assert!(std::mem::size_of::<Prop>() == std::mem::size_of::<Self>());
        }
        unsafe { std::mem::transmute(prop) }
    }

    #[inline]
    pub fn into_prop(self) -> Result<Prop, Self> {
        match self {
            Self::Spread(_) => Err(self),
            _ => unsafe { Ok(std::mem::transmute(self)) },
        }
    }
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

#[ast(repr)]
pub enum MemberProp {
    Ident(IdentName),
    PrivateName(PrivateName),
    Computed(ComputedPropName),
}

impl MemberProp {
    pub fn is_ident_with(self, ast: &Ast<'_>, sym: &str) -> bool {
        matches!(self, MemberProp::Ident(i) if i.sym(ast).as_str() == sym)
    }
}

#[ast]
pub struct SuperPropExpr {
    obj: Super,
    prop: SuperProp,
}

#[ast(repr)]
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
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
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

#[ast(repr)]
pub enum Callee {
    // Expr
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
    Str(Str),
    Bool(Bool),
    Null(Null),
    Num(Number),
    BigInt(BigInt),
    Regex(Regex),
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
    PrivateName(PrivateName),
    OptChain(OptChainExpr),
    Invalid(Invalid),

    // Callee
    Super(Super),
}

#[allow(non_snake_case)]
impl Callee {
    #[inline]
    pub fn Expr(expr: Expr) -> Self {
        const {
            assert!(std::mem::size_of::<Expr>() == std::mem::size_of::<Self>());
        }
        unsafe { std::mem::transmute(expr) }
    }

    #[inline]
    pub fn into_expr(self) -> Result<Expr, Self> {
        match self {
            Self::Super(_) => Err(self),
            _ => unsafe { Ok(std::mem::transmute(self)) },
        }
    }
}

#[ast]
pub struct Super {}

#[ast]
pub struct ExprOrSpread {
    spread: Option<Span>,
    expr: Expr,
}

#[ast(repr)]
pub enum BlockStmtOrExpr {
    // Expr
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
    Str(Str),
    Bool(Bool),
    Null(Null),
    Num(Number),
    BigInt(BigInt),
    Regex(Regex),
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
    PrivateName(PrivateName),
    OptChain(OptChainExpr),
    Invalid(Invalid),

    // BlockStmtOrExpr
    BlockStmt(BlockStmt),
}

#[allow(non_snake_case)]
impl BlockStmtOrExpr {
    #[inline]
    pub fn Expr(expr: Expr) -> Self {
        const {
            assert!(std::mem::size_of::<Expr>() == std::mem::size_of::<Self>());
        }
        unsafe { std::mem::transmute(expr) }
    }

    #[inline]
    pub fn into_expr(self) -> Result<Expr, Self> {
        match self {
            Self::BlockStmt(_) => Err(self),
            _ => unsafe { Ok(std::mem::transmute(self)) },
        }
    }
}

#[ast(repr)]
pub enum AssignTarget {
    SimpleIdent(BindingIdent),
    SimpleMember(MemberExpr),
    SimpleSuperProp(SuperPropExpr),
    SimpleParen(ParenExpr),
    SimpleOptChain(OptChainExpr),
    SimpleInvalid(Invalid),
    PatArray(ArrayPat),
    PatObject(ObjectPat),
    PatternInvalid(Invalid),
}

#[allow(non_snake_case)]
impl AssignTarget {
    pub fn Simple(target: SimpleAssignTarget) -> Self {
        match target {
            SimpleAssignTarget::Ident(node) => Self::SimpleIdent(node),
            SimpleAssignTarget::Member(node) => Self::SimpleMember(node),
            SimpleAssignTarget::SuperProp(node) => Self::SimpleSuperProp(node),
            SimpleAssignTarget::Paren(node) => Self::SimpleParen(node),
            SimpleAssignTarget::OptChain(node) => Self::SimpleOptChain(node),
            SimpleAssignTarget::Invalid(node) => Self::SimpleInvalid(node),
        }
    }

    pub fn Pat(target: AssignTargetPat) -> Self {
        match target {
            AssignTargetPat::Array(node) => Self::PatArray(node),
            AssignTargetPat::Object(node) => Self::PatObject(node),
            AssignTargetPat::Invalid(node) => Self::PatternInvalid(node),
        }
    }

    pub fn into_simple(self) -> Option<SimpleAssignTarget> {
        match self {
            Self::SimpleIdent(node) => Some(SimpleAssignTarget::Ident(node)),
            Self::SimpleMember(node) => Some(SimpleAssignTarget::Member(node)),
            Self::SimpleSuperProp(node) => Some(SimpleAssignTarget::SuperProp(node)),
            Self::SimpleParen(node) => Some(SimpleAssignTarget::Paren(node)),
            Self::SimpleOptChain(node) => Some(SimpleAssignTarget::OptChain(node)),
            Self::SimpleInvalid(node) => Some(SimpleAssignTarget::Invalid(node)),
            _ => None,
        }
    }

    pub fn into_pat(self) -> Option<AssignTargetPat> {
        match self {
            Self::PatArray(node) => Some(AssignTargetPat::Array(node)),
            Self::PatObject(node) => Some(AssignTargetPat::Object(node)),
            Self::PatternInvalid(node) => Some(AssignTargetPat::Invalid(node)),
            _ => None,
        }
    }
}

#[ast(repr)]
pub enum AssignTargetPat {
    Array(ArrayPat),
    Object(ObjectPat),
    Invalid(Invalid),
}

#[ast(repr)]
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

#[ast(repr)]
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
            Expr::Ident(i) => i.sym(ast).as_str() == ident.as_ref(),
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
            p => match p.into_expr() {
                Ok(e) => match Self::try_from_expr(ast, e) {
                    Ok(v) => v,
                    Err(e) => return Err(Pat::Expr(e)),
                },
                Err(p) => return Err(p),
            },
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
