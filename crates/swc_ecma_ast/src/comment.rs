use swc_atoms::Atom;

use crate::Span;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Comments {
    pub leading: hashbrown::HashMap<u32, Vec<Comment>>,
    pub trailing: hashbrown::HashMap<u32, Vec<Comment>>,
}

impl Comments {
    pub fn move_leading(&mut self, from: u32, to: u32) {
        move_comments(&mut self.leading, from, to);
    }

    pub fn move_trailing(&mut self, from: u32, to: u32) {
        move_comments(&mut self.trailing, from, to);
    }
}

fn move_comments(comments: &mut hashbrown::HashMap<u32, Vec<Comment>>, from: u32, to: u32) {
    if from == to {
        return;
    }

    let Some(moved) = comments.remove(&from) else {
        return;
    };

    comments.entry(to).or_default().extend(moved);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub kind: CommentKind,
    pub span: Span,
    /// [`Atom::new_bad`][] is perfectly fine for this value.
    pub text: Atom,
}

impl Comment {
    pub fn new(kind: CommentKind, span: Span, text: Atom) -> Self {
        Self { kind, span, text }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    /// Line comment
    #[default]
    Line = 0,
    // Block comment
    Block = 1,
}
