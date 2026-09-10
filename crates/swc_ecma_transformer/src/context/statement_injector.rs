use crate::context::Mutation;
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    ArrowFunctionBody, ArrowFunctionExpression, AstUtf8, BindingPattern, DUMMY_SP, Decl, NodeId,
    Stmt, SubRange, VariableKind,
};

/// Collects statements to prepend to scopes or insert after existing statements.
///
/// Transforms enqueue edits while visiting descendants. All after-insertions
/// for a statement container are applied in one rebuild, and all generated
/// variables for a scope are prepended when that scope exits.
#[derive(Default)]
struct PendingStatements {
    /// Preserve enqueue order for statements sharing an anchor.
    insertions: Vec<Insertion>,
    prepend_vars: Vec<AstUtf8>,
    prepend: Vec<Stmt>,
}

struct Insertion {
    anchor: NodeId,
    statement: Stmt,
}

struct StatementContainer {
    node: NodeId,
    depth: usize,
}

#[derive(Default)]
pub(crate) struct StatementInjector {
    pending: FxHashMap<NodeId, PendingStatements>,
    statement_containers: Vec<StatementContainer>,
    var_scopes: Vec<NodeId>,
}

impl StatementInjector {
    pub(super) fn prepend_statement(&mut self, container: NodeId, statement: Stmt) {
        self.pending
            .entry(container)
            .or_default()
            .prepend
            .push(statement);
    }

    pub(super) fn current_var_scope(&self) -> NodeId {
        *self.var_scopes.last().expect("var scope exists")
    }

    pub(super) fn enter_statement_container(&mut self, node: NodeId, depth: usize) {
        self.statement_containers
            .push(StatementContainer { node, depth });
    }

    pub(super) fn exit_statement_container(&mut self, expected: NodeId) {
        let actual = self
            .statement_containers
            .pop()
            .map(|container| container.node);
        debug_assert_eq!(actual, Some(expected));
    }

    pub(super) fn enter_var_scope(&mut self, container: NodeId) {
        self.var_scopes.push(container);
    }

    pub(super) fn exit_var_scope(&mut self, expected: NodeId) {
        let actual = self.var_scopes.pop();
        debug_assert_eq!(actual, Some(expected));
    }

    pub(super) fn current_insertion_target(&self, path: &[NodeId]) -> (NodeId, NodeId) {
        let container = self
            .statement_containers
            .last()
            .expect("a class declaration must belong to a statement list");
        let anchor = path
            .get(container.depth + 1)
            .copied()
            .expect("a class declaration must be inside its statement container");
        (container.node, anchor)
    }

    pub(super) fn prepend_var_current(&mut self, name: AstUtf8) {
        let container = *self
            .var_scopes
            .last()
            .expect("a class must belong to a generated-name var scope");
        self.pending
            .entry(container)
            .or_default()
            .prepend_vars
            .push(name);
    }

    fn insert_after(&mut self, container: NodeId, anchor: NodeId, statement: Stmt) {
        self.pending
            .entry(container)
            .or_default()
            .insertions
            .push(Insertion { anchor, statement });
    }

    pub(super) fn flush(&mut self, container: NodeId, ctx: &mut Mutation<'_, '_, '_>) {
        if self.pending.is_empty() {
            return;
        }
        let Some(pending) = self.pending.remove(&container) else {
            return;
        };
        let list = ctx.statement_list(container);
        let body = list.statements(ctx.ast);
        let insertion_count = pending.insertions.len();
        let prepend_count = pending.prepend.len() + usize::from(!pending.prepend_vars.is_empty());
        let mut statements = Vec::with_capacity(body.len() + insertion_count + prepend_count);
        statements.extend(pending.prepend);
        if !pending.prepend_vars.is_empty() {
            statements.push(var_statement(pending.prepend_vars, ctx));
        }

        let mut insertions = FxHashMap::<NodeId, Vec<Stmt>>::default();
        for insertion in pending.insertions {
            insertions
                .entry(insertion.anchor)
                .or_default()
                .push(insertion.statement);
        }
        for id in body.iter() {
            let statement = ctx.ast.get_node_in_sub_range(id);
            statements.push(statement);
            if let Some(after) = insertions.remove(&statement.node_id()) {
                statements.extend(after);
            }
        }
        assert!(
            insertions.is_empty(),
            "pending insertion anchors must remain in their container"
        );
        ctx.attach_statements(list, &statements);
    }

    pub(super) fn flush_arrow_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self.pending.is_empty() {
            return;
        }
        let Some(pending) = self.pending.remove(&arrow.node_id()) else {
            return;
        };
        assert!(
            pending.insertions.is_empty() && pending.prepend.is_empty(),
            "an expression-bodied arrow cannot contain statement insertions"
        );
        let statement = var_statement(pending.prepend_vars, ctx);
        let body = arrow.body(ctx.ast);
        let expression = body
            .as_expr(ctx.ast)
            .expect("queued arrow scope must have an expression body");
        let scope = ctx
            .scope_for_node(arrow.node_id())
            .expect("arrow scope exists");
        let return_statement = Stmt::ReturnStatement(ctx.add_node(scope, |ast| {
            ast.return_statement(DUMMY_SP, Some(expression))
        }));
        let statements = ctx.ast.add_sub_range([statement, return_statement]);
        let function_body = ctx.add_node(scope, |ast| {
            ast.function_body(DUMMY_SP, SubRange::empty(), statements)
        });
        arrow.set_expression(ctx.ast, false);
        arrow.set_body(ctx.ast, ArrowFunctionBody::FunctionBody(function_body));
    }

    pub(super) fn is_empty(&self) -> bool {
        self.pending.is_empty()
            && self.statement_containers.is_empty()
            && self.var_scopes.is_empty()
    }
}

fn var_statement<'a>(names: Vec<AstUtf8>, ctx: &mut Mutation<'_, '_, 'a>) -> Stmt {
    let scope = ctx
        .state
        .semantic
        .scope_of(ctx.state.generated_bindings[&names[0]].symbol);
    let declarators: Vec<_> = names
        .into_iter()
        .map(|name| {
            let identifier = ctx.generated_declaration(name);
            let pattern = BindingPattern::BindingIdentifier(identifier);
            ctx.add_node(scope, |ast| {
                ast.variable_declarator(DUMMY_SP, pattern, None, None, false)
            })
        })
        .collect();
    let declarators = ctx.ast.add_sub_range(declarators);
    let declaration = ctx.add_node(scope, |ast| {
        ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false)
    });
    Stmt::Declaration(Decl::VariableDeclaration(declaration))
}

impl Mutation<'_, '_, '_> {
    /// Queue a prepared statement without changing AST nodes or statement lists.
    /// Its bindings, references and scopes have already been registered by the
    /// context construction APIs. Flush only attaches the prepared statement.
    pub(crate) fn inject_statement(&mut self, container: NodeId, statement: Stmt) {
        self.assert_statement_scope(self.statement_list(container), statement);
        self.state
            .statement_injector
            .prepend_statement(container, statement);
    }

    /// Queue the declaration of a generated binding registered at allocation.
    /// The declaration statement itself is built when the scope is flushed.
    pub(crate) fn inject_var(&mut self, name: AstUtf8) {
        debug_assert!(self.state.generated_bindings.contains_key(&name));
        self.state.statement_injector.prepend_var_current(name);
    }

    pub(crate) fn inject_statement_after_current(&mut self, statement: Stmt) {
        let (container, anchor) = self
            .state
            .statement_injector
            .current_insertion_target(self.path());
        self.inject_statement_after(container, anchor, statement);
    }

    pub(crate) fn inject_statement_after(
        &mut self,
        container: NodeId,
        anchor: NodeId,
        statement: Stmt,
    ) {
        let list = self.statement_list(container);
        self.assert_statement_scope(list, statement);
        assert!(
            self.path()
                .windows(2)
                .any(|edge| edge == [container, anchor])
                || list.statements(self.ast).iter().any(|slot| self
                    .ast
                    .get_node_in_sub_range(slot)
                    .node_id()
                    == anchor),
            "insertion anchor must be a direct statement in the container"
        );
        self.state
            .statement_injector
            .insert_after(container, anchor, statement);
    }

    pub(crate) fn enter_statement_container(&mut self, node: NodeId) {
        self.state
            .statement_injector
            .enter_statement_container(node, self.depth());
    }

    pub(crate) fn exit_statement_container(&mut self, node: NodeId) {
        self.state.statement_injector.exit_statement_container(node);
    }

    pub(crate) fn enter_var_scope(&mut self, node: NodeId) {
        self.state.statement_injector.enter_var_scope(node);
    }

    pub(crate) fn exit_var_scope(&mut self, node: NodeId) {
        self.state.statement_injector.exit_var_scope(node);
    }
}
