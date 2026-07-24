use swc_next_ecma_ast::{
    Ast, AttachedComment, AttachedCommentPosition, Comment, NodeId, NodeKind, Program, Span,
    Traverse, TraverseAction, TraverseCtx, TraverseWith,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ChildInfo {
    node: NodeId,
    start: u32,
    end: u32,
}

struct ChildCollector;

impl<'a> Traverse<'a, Vec<Vec<ChildInfo>>> for ChildCollector {
    fn enter_node(
        &mut self,
        node: NodeId,
        kind: NodeKind,
        ctx: &mut TraverseCtx<'_, 'a, Vec<Vec<ChildInfo>>>,
    ) -> TraverseAction {
        // Hashbangs are represented as AST nodes, but comments after a hashbang
        // belong to the program body just as they do in the source grammar.
        if kind == NodeKind::Hashbang {
            return TraverseAction::Skip;
        }
        if let Some(parent) = ctx.parent() {
            let span = ctx.ast.span(node);
            ctx.state[parent.index()].push(ChildInfo {
                node,
                start: span.start,
                end: span.end,
            });
        }
        TraverseAction::Proceed
    }
}

pub(crate) fn attach(ast: &mut Ast<'_>, program: Program, raw: &[Comment]) {
    if raw.is_empty() {
        ast.set_attached_comments(&[]);
        return;
    }

    let node_count = ast.node_count();
    let children = {
        let state = vec![Vec::new(); node_count];
        let mut ctx = TraverseCtx::new(ast, state);
        let action = program.traverse_with(&mut ChildCollector, &mut ctx);
        debug_assert_ne!(action, TraverseAction::Stop);
        ctx.state
    };

    let root = program.node_id();
    let root_span = ast.span(root);
    let mut ctx = AttachContext {
        ast,
        raw,
        children,
        out: Vec::with_capacity(raw.len()),
        cursor: 0,
    };
    ctx.walk_at(root, root_span);

    while ctx.cursor < raw.len() {
        ctx.write(root, AttachedCommentPosition::Inside, false);
        ctx.cursor += 1;
    }

    debug_assert_eq!(ctx.out.len(), raw.len());
    ctx.out.sort_by_key(|comment| comment.attached_to.index());
    ctx.ast.set_attached_comments(&ctx.out);
}

struct AttachContext<'comments, 'ast, 'allocator> {
    ast: &'ast mut Ast<'allocator>,
    raw: &'comments [Comment],
    children: Vec<Vec<ChildInfo>>,
    out: Vec<AttachedComment>,
    cursor: usize,
}

impl AttachContext<'_, '_, '_> {
    fn walk_at(&mut self, node: NodeId, node_span: Span) {
        if self.cursor >= self.raw.len() || self.raw[self.cursor].span.start >= node_span.end {
            return;
        }

        let mut children = self.children[node.index()].clone();
        sort_by_start(&mut children);

        let mut prev = None;
        let mut prev_end = 0;
        for child in children {
            self.consume_between(node, prev, prev_end, Some(child.node), child.start);
            self.walk_at(child.node, Span::new(child.start, child.end));
            prev = Some(child.node);
            prev_end = child.end;
        }
        self.consume_between(node, prev, prev_end, None, node_span.end);
    }

    fn consume_between(
        &mut self,
        host: NodeId,
        prev: Option<NodeId>,
        prev_end: u32,
        next: Option<NodeId>,
        next_start: u32,
    ) {
        while self.cursor < self.raw.len() {
            let comment = self.raw[self.cursor];
            if comment.span.start >= next_start {
                return;
            }

            match (prev, next) {
                (Some(prev), Some(next)) => {
                    if self.same_line(comment.span.end, next_start) {
                        self.write(next, AttachedCommentPosition::Before, true);
                    } else if self.same_line(prev_end, comment.span.start) {
                        self.write(prev, AttachedCommentPosition::After, true);
                    } else {
                        self.write(next, AttachedCommentPosition::Before, false);
                    }
                }
                (None, Some(next)) => self.write(
                    next,
                    AttachedCommentPosition::Before,
                    self.same_line(comment.span.end, next_start),
                ),
                (Some(prev), None) => self.write(
                    prev,
                    AttachedCommentPosition::After,
                    self.same_line(prev_end, comment.span.start),
                ),
                (None, None) => {
                    self.write(host, AttachedCommentPosition::Inside, false);
                }
            }
            self.cursor += 1;
        }
    }

    fn write(&mut self, host: NodeId, position: AttachedCommentPosition, same_line: bool) {
        let raw = self.raw[self.cursor];
        self.out.push(AttachedComment {
            attached_to: host,
            comment_type: raw.comment_type,
            position,
            same_line,
            value: raw.value,
        });
    }

    fn same_line(&self, a: u32, b: u32) -> bool {
        let lo = a.min(b) as usize;
        let hi = a.max(b) as usize;
        !self.ast.source()[lo..hi].contains(&b'\n')
    }
}

fn sort_by_start(children: &mut [ChildInfo]) {
    let mut index = 1;
    while index < children.len() {
        let child = children[index];
        let mut insert_at = index;
        while insert_at > 0 && children[insert_at - 1].start > child.start {
            children[insert_at] = children[insert_at - 1];
            insert_at -= 1;
        }
        children[insert_at] = child;
        index += 1;
    }
}
