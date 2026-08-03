use crate::AstUtf8;
use crate::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    Line,
    Block,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentPosition {
    Leading,
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comment {
    pub span: Span,
    pub kind: CommentKind,
    pub attached_to: u32,
    pub position: CommentPosition,
}

impl Comment {
    pub fn new(span: Span, kind: CommentKind) -> Self {
        Self {
            span,
            kind,
            attached_to: 0,
            position: CommentPosition::Trailing,
        }
    }

    pub fn value(&self, source: &[u8]) -> AstUtf8 {
        let start = self.span.start;
        let head = match source[start as usize] {
            b'<' => 4,
            b'-' => 3,
            _ => 2,
        };
        let tail = if self.kind == CommentKind::Block {
            2
        } else {
            0
        };

        AstUtf8 {
            start: self.span.start + head,
            end: self.span.end - tail,
        }
    }
}
