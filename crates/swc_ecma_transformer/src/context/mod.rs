use swc_next_ecma_ast::{Ast, NodeId, NodeKindData, ScopeId, Stmt, TraverseMutCtx};

mod bound_identifier;
mod comments;
mod container;
mod helpers;
mod memoisation;
mod statement_injector;

pub(crate) use bound_identifier::BoundIdentifier;
pub(crate) use comments::CommentState;
pub(crate) use container::{Container, ContainerOwner, ContainerOwnerChild};
pub(crate) use statement_injector::StatementInjector;

use crate::{assumptions::Assumptions, state::TransformerState};
use swc_next_ecma_semantic::{ScopeModel, SymbolFlags};

/// Borrow AST and transform state together for synchronized mutations.
pub trait Mutate<'s, 'a> {
    fn mutate(&mut self) -> Mutation<'_, 's, 'a>;
}

impl<'s, 'a> Mutate<'s, 'a> for TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>> {
    fn mutate(&mut self) -> Mutation<'_, 's, 'a> {
        Mutation {
            ast: self.ast,
            state: &mut self.state,
        }
    }
}

/// An atomic mutation interface borrowing the traversal's AST, state and path.
pub struct Mutation<'c, 's, 'a> {
    pub ast: &'c mut Ast<'a>,
    state: &'c mut TransformerState<'s, 'a>,
}

impl<'c, 's, 'a> Mutation<'c, 's, 'a> {
    pub fn assumptions(&self) -> &Assumptions {
        &self.state.assumptions
    }

    pub fn semantic(&self) -> &ScopeModel<'a> {
        self.state.semantic
    }

    pub(crate) fn parent(&self) -> Option<NodeId> {
        self.state.path.iter().rev().nth(1).copied()
    }

    pub(crate) fn ancestors(&self) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        self.state.path.iter().rev().copied()
    }

    pub(crate) fn program_scope(&self) -> ScopeId {
        self.semantic()
            .scope_at(self.ast.root_program().node_id())
            .expect("program scope exists")
    }

    /// Allocate nodes in their final evaluation scope and register only the new
    /// NodeId range. The closure must only create nodes; existing AST edges are
    /// edited through mutation APIs. Bindings, references and scope owners use
    /// their dedicated construction methods.
    pub(crate) fn add_node<N>(&mut self, scope: ScopeId, f: impl FnOnce(&mut Ast<'a>) -> N) -> N {
        debug_assert!(
            self.state.semantic.scope(scope).active,
            "target scope must be active"
        );
        let node = f(self.ast);
        self.state.semantic.grow_to(self.ast.node_count(), scope);
        node
    }

    pub(crate) fn finish(&mut self) {
        self.state.comments.flush(self.ast);
        debug_assert!(self.state.statement_injector.is_empty());
        debug_assert!(self.state.helpers.is_empty());
        debug_assert!(self.state.container.is_empty());
    }
}

impl<'c, 's, 'a> Mutation<'c, 's, 'a> {
    /// Babel: classBody.pushContainer("body", [t.classPrivateProperty(...)]) .
    /// Append a prepared member; container_flush attaches the final range.
    pub fn push_container(&mut self, owner: ContainerOwnerChild) {
        match owner {
            ContainerOwnerChild::ClassBody((class_body, class_element)) => self
                .container_class_body_mut(class_body)
                .push(class_element),
        }
    }

    /// Babel: classPath.get("body").unshiftContainer("body", t.staticBlock([...])) .
    /// Prepend a prepared member, visible immediately through container_class_body.
    /// The caller must register new scope owners and references before insertion.
    pub fn unshift_container(&mut self, owner: ContainerOwnerChild) {
        match owner {
            ContainerOwnerChild::ClassBody((class_body, class_element)) => self
                .container_class_body_mut(class_body)
                .insert(0, class_element),
        }
    }

    /// Babel: scope.generateDeclaredUidIdentifier.
    /// Generate a unique identifier and add it to the current scope.
    ///
    /// `name` is an identifier stem, e.g. `staticBlock`. Reuse the shared name
    /// allocator for Babel's `generateUidIdentifier`, and queue an uninitialized
    /// `var` for `scope.push({ id })`. The declaration goes in Semantic's var
    /// hoist target; Babel's declaration grouping and IIFE parameter optimization
    /// are not needed for this equivalent implementation.
    pub fn generate_declared_uid_identifier(
        &mut self,
        scope: ScopeId,
        name: &str,
    ) -> BoundIdentifier {
        let scope = self.semantic().scope(scope).hoist_target;
        // Parameter expressions can give the function body its own var scope.
        let scope = self.semantic().scope(scope).function_body.unwrap_or(scope);
        let owner = self.semantic().scope(scope).node;
        let container = match self.ast.kind_data(owner) {
            NodeKindData::Function(function) => function.body(self.ast).node_id(),
            NodeKindData::ArrowFunctionExpression(arrow) => arrow
                .body(self.ast)
                .as_function_body(self.ast)
                .map_or(owner, |body| body.node_id()),
            _ => owner,
        };

        let base = format!("_{name}");
        let id = BoundIdentifier::named(self, scope, &base, SymbolFlags::FUNCTION_SCOPED_VAR);
        self.inject_variable(container, id.identifier);

        // BoundIdentifier retains the declaration identity; callers create fresh
        // registered references with id.reference(), rather than cloneNode(id).
        id
    }

    /// Babel: classBody.parentPath.insertAfter(t.expressionStatement(call)).
    /// Queue a prepared statement after a path on the current traversal stack.
    /// Like Babel, forward through expression statements, labels and exports to
    /// anchor the insertion after the containing statement. The caller prepares
    /// the statement's semantic information in its destination scope before
    /// queueing it; the statement injector only defers attachment to the list.
    ///
    /// This covers statement-list insertion used by static-block lowering.
    /// Babel's expression rewriting and wrapping of single-statement bodies
    /// still require separate mutation support.
    pub fn insert_after(&mut self, path: NodeId, statement: Stmt) {
        let index = self
            .state
            .path
            .iter()
            .rposition(|&node| node == path)
            .expect("insert_after target must be on the current traversal path");
        let mut anchor = path;
        let mut parents = self.state.path[..index].iter().rev().copied();
        let (container, anchor) = loop {
            let parent = parents
                .next()
                .expect("insert_after needs a statement container");
            let statements = match self.ast.kind_data(parent) {
                NodeKindData::Program(node) => node.body(self.ast),
                NodeKindData::BlockStatement(node) => node.body(self.ast),
                NodeKindData::FunctionBody(node) => node.body(self.ast),
                NodeKindData::StaticBlock(node) => node.body(self.ast),
                NodeKindData::SwitchCase(node) => node.consequent(self.ast),
                NodeKindData::TsModuleBlock(node) => node.body(self.ast),
                NodeKindData::ExpressionStatement(_)
                | NodeKindData::LabeledStatement(_)
                | NodeKindData::ExportNamedDeclaration(_)
                | NodeKindData::ExportDefaultDeclaration(_) => {
                    anchor = parent;
                    continue;
                }
                _ => panic!("insert_after requires a target in a statement list"),
            };
            let anchor = statements
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot))
                .find(|node| node.node_id() == anchor)
                .expect("insert_after anchor must belong to its statement container");
            break (parent, anchor);
        };
        self.inject_statement(
            container,
            statement,
            statement_injector::Anchor::After(anchor),
        );
    }
}
