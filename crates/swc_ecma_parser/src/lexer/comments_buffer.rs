use swc_experimental_allocator::{Allocator, vec::Vec};
use swc_experimental_ecma_ast::Comment;

#[derive(Debug, Clone)]
pub struct BufferedComment<'a> {
    pub kind: BufferedCommentKind,
    pub pos: u32,
    pub comment: Comment<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum BufferedCommentKind {
    Leading,
    Trailing,
}

#[derive(Clone)]
pub struct CommentsBuffer<'a> {
    comments: Vec<'a, BufferedComment<'a>>,
    pending_leading: Vec<'a, Comment<'a>>,
}

#[derive(Debug, Default)]
pub struct CommentsBufferCheckpoint {
    comments_pos: usize,
    pending_leading: usize,
}

impl<'a> CommentsBuffer<'a> {
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            comments: Vec::new_in(allocator),
            pending_leading: Vec::new_in(allocator),
        }
    }

    pub fn checkpoint_save(&self) -> CommentsBufferCheckpoint {
        CommentsBufferCheckpoint {
            comments_pos: self.comments.len(),
            pending_leading: self.pending_leading.len(),
        }
    }

    pub fn checkpoint_load(&mut self, checkpoint: CommentsBufferCheckpoint) {
        self.comments.truncate(checkpoint.comments_pos);
        self.pending_leading.truncate(checkpoint.pending_leading);
    }
    #[inline(always)]
    pub fn push_comment(&mut self, comment: BufferedComment<'a>) {
        self.comments.push(comment);
    }

    #[inline(always)]
    pub fn push_pending(&mut self, comment: Comment<'a>) {
        self.pending_leading.push(comment);
    }

    #[inline(always)]
    pub fn has_pending(&self) -> bool {
        !self.pending_leading.is_empty()
    }

    #[inline(always)]
    pub fn pending_to_comment(&mut self, kind: BufferedCommentKind, pos: u32) {
        // Most tokens have no pending comments; avoid creating an empty drain on
        // this lexer hot path.
        match self.pending_leading.len() {
            0 => return,
            1 => {
                let comment = self.pending_leading.pop().unwrap();
                let comment = BufferedComment { kind, pos, comment };
                self.comments.push(comment);
                return;
            }
            _ => {}
        }

        for comment in self.pending_leading.drain(..) {
            let comment = BufferedComment { kind, pos, comment };
            self.comments.push(comment);
        }
    }

    #[inline(always)]
    pub fn take_comments(&mut self) -> impl Iterator<Item = BufferedComment<'a>> + '_ {
        self.comments.drain(..)
    }
}
