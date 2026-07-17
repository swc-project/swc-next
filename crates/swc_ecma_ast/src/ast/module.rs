use swc_experimental_ast_macros::ast;

use crate::ast::{ModuleDecl, Stmt};

#[ast]
pub enum Program {
    Module(Module),
    Script(Script),
}

#[ast]
pub struct Module {
    body: Vec<ModuleItem>,
    shebang: Option<Atom>,
}

#[ast]
pub struct Script {
    body: Vec<Stmt>,
    shebang: Option<Atom>,
}

#[ast]
pub enum ModuleItem {
    ModuleDecl(ModuleDecl),
    Stmt(Stmt),
}
