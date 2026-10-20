use super::super::JsxRuntime;
use super::JsxLowerer;
use crate::syntax::{FollowingTransform, NamespaceParameter};
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_ast::VisitMutWith;
use swc_next_ecma_ast::{Ast, Expr, Lang, Program, VisitMut};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic, scope::ScopeKind};

impl<'ast> FollowingTransform<'ast> for JsxLowerer<'_, 'ast> {
    fn runtime_dependencies(&mut self, semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
        let mut dependencies = FxHashSet::default();
        if self.options.runtime != JsxRuntime::Classic || semantic.jsx_usages().is_empty() {
            return dependencies;
        }
        let has_namespaces = semantic
            .iter_scopes()
            .any(|entry| entry.scope.kind == ScopeKind::TsModule);
        let mut scope = None;
        let mut factory = None;
        let mut fragment = None;
        for usage in semantic.jsx_usages() {
            if scope != Some(usage.scope) {
                scope = Some(usage.scope);
                factory = self
                    .pragma
                    .root()
                    .and_then(|root| semantic.lookup(usage.scope, root, ReferenceSpace::Value));
                fragment = self
                    .pragma_frag
                    .root()
                    .and_then(|root| semantic.lookup(usage.scope, root, ReferenceSpace::Value));
                if let Some(symbol) = factory {
                    dependencies.insert(symbol);
                }
            }
            if has_namespaces && (factory.is_some() || fragment.is_some()) {
                self.runtime_references
                    .insert(usage.node, (factory, fragment));
            }
            if usage.is_fragment
                && let Some(symbol) = fragment
            {
                dependencies.insert(symbol);
            }
        }
        dependencies
    }

    fn exit_expression(
        &mut self,
        expression: Expr,
        ast: &mut Ast<'ast>,
        namespace_parameter: &NamespaceParameter<'_>,
    ) -> Option<Expr> {
        if let Some(&(factory, fragment)) = self.runtime_references.get(&expression.node_id()) {
            // Direct JSX children share this environment. JSX inside embedded
            // expressions has already been lowered with its own source bindings.
            self.pragma_namespace = factory
                .zip(self.pragma.root())
                .and_then(|(symbol, name)| namespace_parameter(ast, name, symbol));
            self.pragma_frag_namespace = fragment
                .zip(self.pragma_frag.root())
                .and_then(|(symbol, name)| namespace_parameter(ast, name, symbol));
        } else {
            self.pragma_namespace = None;
            self.pragma_frag_namespace = None;
        }
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
