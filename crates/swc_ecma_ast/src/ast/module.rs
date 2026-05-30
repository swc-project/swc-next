use crate::Span;
use swc_experimental_allocator::atom::Atom;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::ast::{ModuleDecl, Stmt};

#[ast]
#[derive(Debug)]
pub enum Program<'a> {
    Module(Box<'a, Module<'a>>),
    Script(Box<'a, Script<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct Module<'a> {
    pub span: Span,
    pub body: Vec<'a, ModuleItem<'a>>,
    pub shebang: Option<Atom<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct Script<'a> {
    pub span: Span,
    pub body: Vec<'a, Stmt<'a>>,
    pub shebang: Option<Atom<'a>>,
}

#[ast]
#[derive(Debug)]
pub enum ModuleItem<'a> {
    ModuleDecl(Box<'a, ModuleDecl<'a>>),
    Stmt(Box<'a, Stmt<'a>>),
}
