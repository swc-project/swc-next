use super::super::{JsxImportOptions, bindings::value_binding};
use crate::jsx::pragma::{PragmaKind, pragmas};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, ClassType, Decorator, NodeId, NodeKindData, Program, ScopeId, SwitchStatement, SymbolId,
    Visit, VisitWith, WithStatement,
};
use swc_next_ecma_semantic2::{ReferenceSpace, ScopeKind, Semantic};

pub(in crate::typescript) fn dependencies(
    program: Program,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    options: &JsxImportOptions,
) -> FxHashSet<SymbolId> {
    if !ast.is_jsx() {
        return FxHashSet::default();
    }
    let mut automatic = options.automatic;
    let mut pragma = options.pragma.as_deref().unwrap_or("React.createElement");
    let mut fragment = options.pragma_frag.as_deref().unwrap_or("React.Fragment");
    for (kind, value) in pragmas(program, ast) {
        match kind {
            PragmaKind::Jsx => pragma = value,
            PragmaKind::Fragment => fragment = value,
            PragmaKind::Runtime => match value {
                "automatic" => automatic = true,
                "classic" => automatic = false,
                _ => {}
            },
            PragmaKind::ImportSource => {}
        }
    }
    if automatic {
        return FxHashSet::default();
    }

    fn root(name: &str) -> &[u8] {
        name.split_once('.')
            .map_or(name, |(root, _)| root)
            .as_bytes()
    }
    let factory = root(pragma);
    let fragment = root(fragment);
    let mut collector = JsxDependencies {
        ast,
        semantic,
        scope: ScopeId::ROOT,
        scope_stack: Vec::new(),
        usages: FxHashMap::default(),
    };
    program.visit_with(&mut collector);

    let mut dependencies = FxHashSet::default();
    for (scope, has_fragment) in collector.usages {
        if let Some(symbol) = semantic.lookup(scope, factory, ReferenceSpace::Value) {
            dependencies.insert(value_binding(semantic, symbol));
        }
        if has_fragment
            && let Some(symbol) = semantic.lookup(scope, fragment, ReferenceSpace::Value)
        {
            dependencies.insert(value_binding(semantic, symbol));
        }
    }
    dependencies
}

struct JsxDependencies<'s, 'a> {
    ast: &'s Ast<'a>,
    semantic: &'s Semantic<'a>,
    scope: ScopeId,
    scope_stack: Vec<(NodeId, ScopeId)>,
    usages: FxHashMap<ScopeId, bool>,
}

impl<'a> JsxDependencies<'_, 'a> {
    fn visit_in_scope(&mut self, owner: NodeId, subtree: impl VisitWith<'a, Self>) {
        let previous = self.scope;
        if let Some(scope) = self.semantic.scope_of(owner) {
            self.scope = scope;
        }
        subtree.visit_with(self);
        self.scope = previous;
    }
}

impl<'a> Visit<'a> for JsxDependencies<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeKindData) {
        let id = node.node_id();
        if let Some(scope) = self.semantic.scope_of(id) {
            self.scope_stack.push((id, self.scope));
            self.scope = scope;
        }
        match node {
            NodeKindData::JsxElement(_) => {
                self.usages.entry(self.scope).or_insert(false);
            }
            NodeKindData::JsxFragment(_) => {
                self.usages.insert(self.scope, true);
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, node: NodeKindData) {
        if let Some(&(id, previous)) = self.scope_stack.last()
            && id == node.node_id()
        {
            self.scope = previous;
            self.scope_stack.pop();
        }
    }

    fn visit_switch_statement(&mut self, node: SwitchStatement) {
        // The discriminant is outside the shared case scope.
        node.discriminant(self.ast).visit_with(self);
        self.visit_in_scope(node.node_id(), node.cases(self.ast));
    }

    fn visit_with_statement(&mut self, node: WithStatement) {
        node.object(self.ast).visit_with(self);
        self.visit_in_scope(node.node_id(), node.body(self.ast));
    }

    fn visit_decorator(&mut self, node: Decorator) {
        let previous = self.scope;
        let mut current = Some(previous);
        while let Some(scope) = current {
            let environment = self.semantic.scope(scope);
            if environment.kind == ScopeKind::Class {
                let NodeKindData::Class(class) = self.ast.kind_data(environment.node) else {
                    unreachable!();
                };
                self.scope = if class.class_type(self.ast) == ClassType::ClassDeclaration
                    && class
                        .decorators(self.ast)
                        .iter()
                        .any(|id| self.ast.get_node_in_sub_range(id) == node)
                {
                    environment.parent.unwrap()
                } else {
                    scope
                };
                break;
            }
            current = environment.parent;
        }
        self.visit_in_scope(node.node_id(), node.expression(self.ast));
        self.scope = previous;
    }
}
