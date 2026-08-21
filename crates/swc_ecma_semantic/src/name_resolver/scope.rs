use swc_next_ecma_ast::VariableKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeKind {
    Block,
    #[default]
    Fn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    Lexical,
    Param,
    Var,
    Function,
    /// Type-only declarations are recorded but are not visible to value references.
    Type,
}

impl From<VariableKind> for DeclKind {
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
