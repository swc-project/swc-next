use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    ArrowFunctionBody, BindingIdentifier, BindingPattern, DUMMY_SP, Decl, NodeId, NodeKindData,
    Stmt, SubRange, VariableKind,
};

use crate::context::TransformCtx;

struct Insertion {
    statement: Stmt,
    anchor: Anchor,
}

pub(crate) enum Anchor {
    First,
    // Last,
    // Before(Stmt),
    After(Stmt),
}

#[derive(Default)]
pub(crate) struct StatementInjector {
    /// Container node id -> list of statements to be inserted.
    stmts: FxHashMap<NodeId, Vec<Insertion>>,
    /// Container node id -> list of variable declarations to be inserted.
    vars: FxHashMap<NodeId, Vec<BindingIdentifier>>,
}

impl StatementInjector {
    pub fn is_empty(&self) -> bool {
        self.stmts.values().all(|v| v.is_empty()) && self.vars.values().all(|v| v.is_empty())
    }
}

impl TransformCtx<'_, '_> {
    pub fn inject_statement(&mut self, container_id: NodeId, statement: Stmt, anchor: Anchor) {
        let insertion = Insertion { statement, anchor };
        let injector = &mut self.state.statement_injector;
        let stmts = injector.stmts.entry(container_id).or_default();
        stmts.push(insertion);
    }

    pub fn inject_variable(&mut self, container_id: NodeId, var: BindingIdentifier) {
        // Retain the registered declaration identity, rather than rebuilding an
        // identifier from its name when the container is flushed.
        let vars = self
            .state
            .statement_injector
            .vars
            .entry(container_id)
            .or_default();
        if !vars.contains(&var) {
            vars.push(var);
        }
    }

    /// Flushes statements queued for `container_id`. Most containers have
    /// nothing queued, so skip the map lookups while both queues are empty.
    #[inline]
    pub fn inject_statements_to(&mut self, container_id: NodeId) {
        let injector = &self.state.statement_injector;
        if injector.stmts.is_empty() && injector.vars.is_empty() {
            return;
        }
        self.flush_injected_statements(container_id);
    }

    #[inline(never)]
    fn flush_injected_statements(&mut self, container_id: NodeId) {
        let insertions = self
            .state
            .statement_injector
            .stmts
            .remove(&container_id)
            .unwrap_or_default();
        let vars = self
            .state
            .statement_injector
            .vars
            .remove(&container_id)
            .unwrap_or_default();
        if insertions.is_empty() && vars.is_empty() {
            return;
        }

        let mut container = self.ast.kind_data(container_id);
        if let NodeKindData::ArrowFunctionExpression(arrow) = container {
            let arrow_body = arrow.body(self.ast);
            let body = match arrow_body.as_function_body(self.ast) {
                Some(body) => body,
                None => {
                    let expression = arrow_body
                        .as_expr(self.ast)
                        .expect("concise arrow expression");
                    // scope.push in a concise arrow needs a statement body while
                    // retaining the expression as the arrow's completion value.
                    let ret = self.add_node(|ast| ast.return_statement(DUMMY_SP, Some(expression)));
                    let statements = self.ast.add_sub_range([Stmt::ReturnStatement(ret)]);
                    self.add_node(|ast| ast.function_body(DUMMY_SP, SubRange::empty(), statements))
                }
            };
            arrow.set_body(self.ast, ArrowFunctionBody::FunctionBody(body));
            arrow.set_expression(self.ast, false);
            container = NodeKindData::FunctionBody(body);
        }

        let body = match container {
            NodeKindData::Program(node) => node.body(self.ast),
            NodeKindData::FunctionBody(node) => node.body(self.ast),
            NodeKindData::BlockStatement(node) => node.body(self.ast),
            NodeKindData::StaticBlock(node) => node.body(self.ast),
            NodeKindData::SwitchCase(node) => node.consequent(self.ast),
            NodeKindData::TsModuleBlock(node) => node.body(self.ast),
            _ => panic!("statement injection requires a statement container"),
        };
        let mut first = Vec::new();
        // let mut last = Vec::new();
        let mut before: FxHashMap<NodeId, Vec<Stmt>> = FxHashMap::default();
        let mut after: FxHashMap<NodeId, Vec<Stmt>> = FxHashMap::default();
        for insertion in insertions {
            match insertion.anchor {
                Anchor::First => first.push(insertion.statement),
                // Anchor::Last => last.push(insertion.statement),
                // Anchor::Before(anchor) => before
                //     .entry(anchor.node_id())
                //     .or_default()
                //     .push(insertion.statement),
                Anchor::After(anchor) => after
                    .entry(anchor.node_id())
                    .or_default()
                    .push(insertion.statement),
            }
        }

        if !vars.is_empty() {
            let declarators: Vec<_> = vars
                .into_iter()
                .map(|binding| {
                    self.add_node(|ast| {
                        ast.variable_declarator(
                            DUMMY_SP,
                            BindingPattern::BindingIdentifier(binding),
                            None,
                            None,
                            false,
                        )
                    })
                })
                .collect();
            let declarators = self.ast.add_sub_range(declarators);
            let declaration = self.add_node(|ast| {
                ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false)
            });
            first.push(Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        }

        // Program/FunctionBody directives have a separate range and stay ahead
        // of injected statements. Preserve queue order for each anchor.
        let mut statements = first;
        for slot in body.iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            statements.extend(before.remove(&statement.node_id()).into_iter().flatten());
            statements.push(statement);
            statements.extend(after.remove(&statement.node_id()).into_iter().flatten());
        }
        debug_assert!(
            before.is_empty() && after.is_empty(),
            "statement injection anchor must belong to its container"
        );
        // statements.extend(last);
        let body = self.ast.add_sub_range(statements);
        match container {
            NodeKindData::Program(node) => node.set_body(self.ast, body),
            NodeKindData::FunctionBody(node) => node.set_body(self.ast, body),
            NodeKindData::BlockStatement(node) => node.set_body(self.ast, body),
            NodeKindData::StaticBlock(node) => node.set_body(self.ast, body),
            NodeKindData::SwitchCase(node) => node.set_consequent(self.ast, body),
            NodeKindData::TsModuleBlock(node) => node.set_body(self.ast, body),
            _ => unreachable!(),
        }
    }
}
