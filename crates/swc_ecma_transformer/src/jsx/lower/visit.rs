use super::super::JsxRuntime;
use super::JsxLowerer;
use crate::syntax::FollowingTransform;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_ast::VisitMutWith;
use swc_next_ecma_ast::{Ast, Expr, Lang, Program, VisitMut};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic};

impl<'ast> FollowingTransform<'ast> for JsxLowerer<'_, 'ast> {
    fn runtime_dependencies(&self, semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
        let mut dependencies = FxHashSet::default();
        if self.options.runtime != JsxRuntime::Classic {
            return dependencies;
        }
        for usage in semantic.jsx_usages() {
            if let Some(root) = self.pragma.root()
                && let Some(symbol) = semantic.lookup(usage.scope, root, ReferenceSpace::Value)
            {
                dependencies.insert(symbol);
            }
            if usage.is_fragment
                && let Some(root) = self.pragma_frag.root()
                && let Some(symbol) = semantic.lookup(usage.scope, root, ReferenceSpace::Value)
            {
                dependencies.insert(symbol);
            }
        }
        dependencies
    }

    fn exit_expression(&mut self, expression: Expr, ast: &mut Ast<'ast>) -> Option<Expr> {
        self.lower_expression(expression, ast)
    }

    fn finish_program(&mut self, program: Program, ast: &mut Ast<'ast>) {
        self.insert_program_bindings(program, ast);
        let source_type = ast.source_type();
        let lang = match ast.lang() {
            Lang::Jsx | Lang::Tsx => Lang::Js,
            lang => lang,
        };
        ast.set_source_metadata(source_type, lang);
    }
}

pub(in crate::jsx) struct JsxVisitor<'visit, 'options, 'ast> {
    ast: &'visit mut Ast<'ast>,
    lowerer: &'visit mut JsxLowerer<'options, 'ast>,
}

impl<'visit, 'options, 'ast> JsxVisitor<'visit, 'options, 'ast> {
    pub(in crate::jsx) fn new(
        ast: &'visit mut Ast<'ast>,
        lowerer: &'visit mut JsxLowerer<'options, 'ast>,
    ) -> Self {
        Self { ast, lowerer }
    }
}

impl<'ast> VisitMut<'ast> for JsxVisitor<'_, '_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        expression.visit_mut_children_with(self);
        self.lowerer.lower_expression(expression, self.ast)
    }
}
