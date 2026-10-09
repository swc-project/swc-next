use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{Ast, CommentPosition, GetSpan, Span};

use super::Mutation;

/// Deferred comment attachment changes for one traversal. The flat AST comment
/// list already stores each comment once, so inheritance retains child comments
/// and moves parent attachments without creating duplicate comment records.
#[derive(Default)]
pub(crate) struct CommentState {
    leading: Vec<(u32, u32)>,
    trailing: Vec<(u32, u32)>,
}

impl CommentState {
    fn inherit(&mut self, child: Span, parent: Span) {
        if parent.start != child.start {
            self.leading.push((parent.start, child.start));
        }
        if parent.end != child.end {
            self.trailing.push((parent.end, child.end));
        }
    }

    fn compose(retargets: &mut Vec<(u32, u32)>) -> FxHashMap<u32, u32> {
        let mut targets = FxHashMap::default();
        // Compose in reverse order: a parent -> child transfer must also inherit
        // later transfers from that child. Snapshot the destination's current
        // result so repeated sources and cycles preserve call order.
        for (source, target) in retargets.drain(..).rev() {
            let target = targets.get(&target).copied().unwrap_or(target);
            targets.insert(source, target);
        }
        targets
    }

    pub(crate) fn flush(&mut self, ast: &mut Ast<'_>) {
        if self.leading.is_empty() && self.trailing.is_empty() {
            return;
        }
        let leading = Self::compose(&mut self.leading);
        let trailing = Self::compose(&mut self.trailing);
        // TODO: Make codegen honor retargeted leading-comment attachment anchors.
        // It still consumes leading comments by source position, so inherited
        // comments can print too late (e.g. the static-block name-conflict fixture).
        ast.retarget_comments(|position, attached_to| match position {
            CommentPosition::Leading => leading.get(&attached_to).copied(),
            CommentPosition::Trailing => trailing.get(&attached_to).copied(),
        });
    }
}

impl Mutation<'_, '_, '_> {
    /// Babel: t.inheritsComments(child, parent). Retain the child's attachments,
    /// inherit the parent's leading/trailing attachments, and return the child.
    /// The parent is being replaced; the AST stores one attachment per comment
    /// rather than Babel's shared comment objects on both nodes. Inner comments
    /// retain their existing child/token anchors; there is no inner-comment list.
    /// Nodes use their span boundaries as attachment anchors, as in Normalizer.
    pub(crate) fn inherits_comments<C: GetSpan, P: GetSpan>(&mut self, child: C, parent: P) -> C {
        if !self.ast.comments().is_empty() {
            self.state
                .comments
                .inherit(child.span(self.ast), parent.span(self.ast));
        }
        child
    }
}
