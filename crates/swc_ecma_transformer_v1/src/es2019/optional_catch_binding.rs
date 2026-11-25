use crate::common::resolver::context::Resolver;
use swc_next_ecma_ast::*;

/// SWC's standalone pass uses an ordinary visitor.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2019/optional_catch_binding.rs#L33>.
pub(crate) fn pass(ast: &mut Ast<'_>, resolver: &mut Resolver) {
    ast.root_program()
        .visit_mut_with(&mut OptionalCatchBindingPass { ast, resolver });
}

struct OptionalCatchBindingPass<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
}
impl<'ast> VisitMut<'ast> for OptionalCatchBindingPass<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_catch_clause(&mut self, clause: CatchClause) -> Option<CatchClause> {
        self.transform_catch_clause(clause);
        clause.body(self.ast).visit_mut_with(self);
        None
    }
}

impl OptionalCatchBindingPass<'_, '_> {
    fn transform_catch_clause(&mut self, clause: CatchClause) {
        if clause.param(self.ast).is_none() {
            let binding = self.resolver.private_binding(self.ast, "unused");
            clause.set_param(self.ast, Some(BindingPattern::BindingIdentifier(binding)));
        }
    }
}
