//! RegExp transform options, independent of the ECMAScript edition groups.

use swc_next_ecma_ast::*;

use crate::common::{TraverseCtx, build as b};

/// SWC's `swc_ecma_compat_common::regexp::Config` feature switches.
///
/// All flags default to `false`. Supported switches rewrite selected literals
/// as `RegExp(...)` calls, preserving the original pattern and flags for the
/// runtime to interpret.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegExpOptions {
    /// Rewrite literals with the `s` (dotAll) flag.
    pub dot_all_regex: bool,
    /// Rewrite literals with the `d` (match indices) flag.
    pub has_indices: bool,
    /// Rewrite literals with lookbehind assertions.
    pub lookbehind_assertion: bool,
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
    pub(crate) fn is_enabled(self) -> bool {
        self.dot_all_regex
            || self.has_indices
            || self.lookbehind_assertion
            || self.sticky_regex
            || self.unicode_regex
            || self.unicode_sets_regex
    }
}

pub(crate) struct RegexpPass(pub(crate) RegExpOptions);

impl RegexpPass {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/regexp.rs#L188>.
    fn needs_regexp_constructor(&self, pattern: &str, flags: &str) -> bool {
        (self.0.dot_all_regex && flags.contains('s'))
            || (self.0.sticky_regex && flags.contains('y'))
            || (self.0.unicode_regex && flags.contains('u'))
            || (self.0.unicode_sets_regex && flags.contains('v'))
            || (self.0.has_indices && flags.contains('d'))
            || (self.0.lookbehind_assertion
                && (pattern.contains("(?<=") || pattern.contains("(?<!")))
    }

    fn transform_expr(
        &self,
        node: Expr,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        let Some(regex) = node.as_reg_exp_literal(ctx.ast) else {
            return TraverseAction::Continue;
        };
        let pattern = ctx.ast.get_utf8(regex.pattern(ctx.ast));
        let flags = ctx.ast.get_utf8(regex.flags(ctx.ast));
        if !self.needs_regexp_constructor(pattern, flags) {
            return TraverseAction::Continue;
        }

        let mut args = vec![b::string(ctx.ast, pattern)];
        if !flags.is_empty() {
            args.push(b::string(ctx.ast, flags));
        }
        let callee = ctx.state.resolver.marked_reference(ctx.ast, "RegExp", true);
        TraverseAction::ReplaceSkip(b::call(ctx.ast, callee, args, false))
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/regexp.rs#L227>.
impl<'ast> TraverseMut<'ast, TraverseCtx<'_>> for RegexpPass {
    fn exit_expr(
        &mut self,
        node: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        self.transform_expr(node, ctx)
    }
}
