use swc_next_ecma_ast::VariableKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum ScopeKind {
    Block,
    #[default]
    Fn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclKind {
    Lexical,
    Param,
    Var,
    Function,
}

impl From<VariableKind> for DeclKind {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1741>.
    fn from(kind: VariableKind) -> Self {
        match kind {
            VariableKind::Var => Self::Var,
            VariableKind::Let
            | VariableKind::Const
            | VariableKind::Using
            | VariableKind::AwaitUsing => Self::Lexical,
        }
    }
}
