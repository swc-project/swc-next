use std::num::NonZeroU32;

use oxc_index::Idx;
use swc_core::common::SyntaxContext;
use swc_experimental_ecma_ast::VarDeclKind;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopeId(pub(crate) NonZeroU32);

impl Idx for ScopeId {
    const MAX: usize = u32::MAX as usize;

    unsafe fn from_usize_unchecked(idx: usize) -> Self {
        unsafe { Self(NonZeroU32::new_unchecked(idx as u32 + 1)) }
    }

    fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}

impl ScopeId {
    pub fn to_ctxt(self) -> SyntaxContext {
        SyntaxContext::from_u32(self.0.get())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeKind {
    Block,
    #[default]
    Fn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentType {
    Binding,
    Ref,
    Label,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    Lexical,
    Param,
    Var,
    Function,
    /// don't actually get stored
    Type,
}

impl From<VarDeclKind> for DeclKind {
    fn from(kind: VarDeclKind) -> Self {
        match kind {
            VarDeclKind::Const | VarDeclKind::Let => Self::Lexical,
            VarDeclKind::Var => Self::Var,
        }
    }
}
