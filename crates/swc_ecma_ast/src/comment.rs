use swc_core::atoms::Atom;

use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub kind: CommentKind,
    pub span: Span,
    /// [`Atom::new_bad`][] is perfectly fine for this value.
    pub text: Atom,
    pub position: CommentPosition,
    pub attached_to: u32,
}

impl Comment {
    pub fn new(kind: CommentKind, span: Span, text: Atom) -> Self {
        Self {
            kind,
            span,
            text,
            position: CommentPosition::Trailing,
            attached_to: 0,
        }
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

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CommentPosition {
    /// Comments prior to a token until another token or trailing comment.
    ///
    /// e.g.
    ///
    /// ```ignore
    /// /* leading */ token;
    /// /* leading */
    /// // leading
    /// token;
    /// ```
    #[default]
    Leading = 0,

    /// Comments tailing a token until a newline.
    /// e.g. `token /* trailing */ // trailing`
    Trailing = 1,
}
