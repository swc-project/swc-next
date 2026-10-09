use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{AstUtf8, NodeId, Stmt};

use crate::context::Mutation;

struct Insertion {
    statement: Stmt,
    anchor: Anchor,
}

pub(crate) enum Anchor {
    First,
    Last,
    Before(Stmt),
    After(Stmt),
}

#[derive(Default)]
pub(crate) struct StatementInjector {
    /// Container node id -> list of statements to be inserted.
    stmts: FxHashMap<NodeId, Vec<Insertion>>,
    /// Container node id -> list of variable declarations to be inserted.
    vars: FxHashMap<NodeId, Vec<AstUtf8>>,
}

impl StatementInjector {
    pub fn is_empty(&self) -> bool {
        self.stmts.values().all(|v| v.is_empty()) && self.vars.values().all(|v| v.is_empty())
    }
}

impl Mutation<'_, '_, '_> {
    pub fn inject_statement(&mut self, container_id: NodeId, statement: Stmt, anchor: Anchor) {
        let insertion = Insertion { statement, anchor };
        let injector = &mut self.state.statement_injector;
        let stmts = injector.stmts.entry(container_id).or_default();
        stmts.push(insertion);
    }

    pub fn inject_variable(&mut self, _container_id: NodeId, _var: AstUtf8) {
        todo!()
    }

    pub fn inject_statements_to(&mut self, _container_id: NodeId) {
        todo!()
    }
}
