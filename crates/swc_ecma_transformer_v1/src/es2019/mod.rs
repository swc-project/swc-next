//! ES2019 transform options.

use swc_next_ecma_ast::Ast;

use crate::common::resolver::context::Resolver;

/// Selects ES2019 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2019Options {
    /// Add a binding to catch clauses that omit one.
    pub optional_catch_binding: bool,
}

mod optional_catch_binding;

pub(crate) fn transform(ast: &mut Ast<'_>, resolver: &mut Resolver, options: ES2019Options) {
    if options.optional_catch_binding {
        optional_catch_binding::pass(ast, resolver);
    }
}
