use swc_next_ecma_ast::{Ast, GetSpan, NodeId, NodeKindData, ScopeId, Stmt, TraverseMutCtx};

mod comments;
mod container;
mod helpers;
mod semantic;
mod statement_injector;

pub(crate) use comments::CommentState;
pub(crate) use container::{Container, ContainerOwner, ContainerOwnerChild};
pub(crate) use semantic::BoundIdentifier;
pub(crate) use statement_injector::StatementInjector;

use crate::state::TransformerState;
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
    pub fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    pub(crate) fn current_scope(&self) -> ScopeId {
        self.semantic()
            .scope_at(*self.state.path.last().expect("active traversal node"))
            .expect("active node scope")
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

    pub(crate) fn finish(&mut self) {
        self.state.comments.flush(self.ast);
        debug_assert!(self.state.statement_injector.is_empty());
        debug_assert!(self.state.helpers.is_empty());
        debug_assert!(self.state.container.is_empty());
    }
}

impl Mutation<'_, '_, '_> {
    /// Babel: t.inheritsComments(child, parent). Retain the child's attachments,
    /// inherit the parent's leading/trailing attachments, and return the child.
    /// The parent is being replaced; the AST stores one attachment per comment
    /// rather than Babel's shared comment objects on both nodes. Inner comments
    /// retain their existing child/token anchors; there is no inner-comment list.
    /// Nodes use their span boundaries as attachment anchors, as in Normalizer.
    pub fn inherits_comments<C: GetSpan, P: GetSpan>(&mut self, child: C, parent: P) -> C {
        if !self.ast.comments().is_empty() {
            self.state
                .comments
                .inherit(child.span(self.ast), parent.span(self.ast));
        }
        child
    }

    /// Append a prepared member; container_flush attaches the final range.
    pub fn push_container(&mut self, owner: ContainerOwnerChild) {
        match owner {
            ContainerOwnerChild::ClassBody((class_body, class_element)) => self
                .container_class_body_mut(class_body)
                .push(class_element),
        }
    }

    /// Prepend a prepared member, visible immediately through container_class_body.
    /// The caller must register new scope owners and references before insertion.
    #[expect(dead_code, reason = "Retain generic container insertion support")]
    pub fn unshift_container(&mut self, owner: ContainerOwnerChild) {
        match owner {
            ContainerOwnerChild::ClassBody((class_body, class_element)) => self
                .container_class_body_mut(class_body)
                .insert(0, class_element),
        }
    }

    /// Register a fresh var binding in the hoist target and queue its declaration.
    /// `base` is the complete identifier stem, including any leading underscore.
    pub fn generate_var(&mut self, scope: ScopeId, base: &str) -> BoundIdentifier {
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

        let id = BoundIdentifier::named(self, scope, base, SymbolFlags::FUNCTION_SCOPED_VAR);
        self.inject_variable(container, id.identifier);

        // BoundIdentifier retains the declaration identity; callers create fresh
        // registered references with id.reference_read(), rather than cloneNode(id).
        id
    }

    /// Queue a prepared statement after a path on the current traversal stack.
    /// Forward through expression statements, labels and exports to
    /// anchor the insertion after the containing statement. The caller prepares
    /// the statement's semantic information in its destination scope before
    /// queueing it; the statement injector only defers attachment to the list.
    ///
    /// This covers statement-list insertion used by static-block lowering.
    /// Expression rewriting and wrapping single-statement bodies require
    /// separate mutation support.
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
                _ => unreachable!("insert_after encountered an unsupported parent node kind"),
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
