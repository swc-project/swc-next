use crate::{Atom, Span};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentType {
    Line,
    Block,
}

impl CommentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::Block => "Block",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comment<'a> {
    pub comment_type: CommentType,
    pub value: Atom<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AttachedCommentPosition {
    Before,
    After,
    Inside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedComment<'a> {
    pub comment_type: CommentType,
    pub position: AttachedCommentPosition,
    pub same_line: bool,
    pub value: Atom<'a>,
}
