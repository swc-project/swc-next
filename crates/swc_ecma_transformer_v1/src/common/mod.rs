//! Shared infrastructure for the v1 transformer passes.

pub(crate) mod build;
pub(crate) mod constructor;
pub(crate) mod function;
pub(crate) mod helpers;
mod hygiene;
mod rename;
pub(crate) mod resolver;
pub(crate) mod super_field;
pub(crate) mod var_declarations;

use std::collections::HashSet;

use self::{resolver::context::Resolver, var_declarations::VarDeclarationsStore};
use helpers::Helpers;
use swc_next_ecma_ast::{Ast, BindingIdentifier, BindingPattern, DUMMY_SP, Expr, NodeId};

pub(crate) struct TraverseCtx<'a> {
    pub(crate) helpers: Helpers,
    pub(crate) resolver: &'a mut Resolver,
    pub(crate) var_declarations: VarDeclarationsStore,
    pub(crate) static_blocks: HashSet<NodeId>,
}

impl<'a> TraverseCtx<'a> {
    pub(crate) fn new(resolver: &'a mut Resolver) -> Self {
        Self {
            resolver,
            helpers: Helpers::default(),
            var_declarations: VarDeclarationsStore::default(),
            static_blocks: Default::default(),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/common/var_declarations.rs#L197>.
    pub(crate) fn insert_var(&mut self, ast: &mut Ast<'_>, name: &str) -> BindingIdentifier {
        let binding = self.resolver.private_binding(ast, name);
        let declaration = ast.variable_declarator(
            DUMMY_SP,
            BindingPattern::BindingIdentifier(binding),
            None,
            None,
            false,
        );
        self.var_declarations.insert_var_declarator(declaration);
        binding
    }

    pub(crate) fn helper(&mut self, ast: &mut Ast<'_>, name: &'static str) -> Expr {
        self.helpers.helper_expr(ast, self.resolver, name)
    }
}
