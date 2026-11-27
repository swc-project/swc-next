//! RegExp transform options, independent of the ECMAScript edition groups.

use swc_next_ecma_ast::*;

use crate::common::{build as b, resolver::context::Resolver};

/// SWC's `swc_ecma_compat_common::regexp::Config` feature switches.
///
/// All flags default to `false`. Supported switches rewrite selected literals
/// as `RegExp(...)` calls, preserving the original pattern and flags for the
/// runtime to interpret.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegExpOptions {
    /// Rewrite literals with the `s` (dotAll) flag.
    pub dot_all_regex: bool,
    /// Reserved for named capture group lowering. Currently ignored.
    pub named_capturing_groups_regex: bool,
    /// Reserved for duplicate named capture group lowering. Currently ignored.
    pub duplicate_named_capturing_groups_regex: bool,
    /// Rewrite literals with the `y` (sticky) flag.
    pub sticky_regex: bool,
    /// Reserved for Unicode property escape lowering. Currently ignored.
    pub unicode_property_regex: bool,
    /// Rewrite literals with the `u` (Unicode) flag.
    pub unicode_regex: bool,
    /// Rewrite literals with the `v` (Unicode sets) flag.
    pub unicode_sets_regex: bool,
}

impl RegExpOptions {
    fn is_enabled(self) -> bool {
        self.dot_all_regex || self.sticky_regex || self.unicode_regex || self.unicode_sets_regex
    }
}

pub(crate) fn transform(ast: &mut Ast<'_>, resolver: &mut Resolver, options: RegExpOptions) {
    if options.is_enabled() {
        ast.root_program().visit_mut_with(&mut RegexpPass {
            ast,
            resolver,
            options,
        });
    }
}

struct RegexpPass<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    options: RegExpOptions,
}

impl RegexpPass<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/regexp.rs#L188>.
    fn needs_regexp_constructor(&self, flags: &str) -> bool {
        (self.options.dot_all_regex && flags.contains('s'))
            || (self.options.sticky_regex && flags.contains('y'))
            || (self.options.unicode_regex && flags.contains('u'))
            || (self.options.unicode_sets_regex && flags.contains('v'))
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/regexp.rs#L227>.
impl<'ast> VisitMut<'ast> for RegexpPass<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        let Some(regex) = node.as_reg_exp_literal(self.ast) else {
            return node.visit_mut_children_with(self);
        };
        let pattern = self.ast.get_utf8(regex.pattern(self.ast));
        let flags = self.ast.get_utf8(regex.flags(self.ast));
        if !self.needs_regexp_constructor(flags) {
            return None;
        }

        let mut args = vec![b::string(self.ast, pattern)];
        if !flags.is_empty() {
            args.push(b::string(self.ast, flags));
        }
        let callee = self.resolver.marked_reference(self.ast, "RegExp", true);
        Some(b::call(self.ast, callee, args, false))
    }
}
