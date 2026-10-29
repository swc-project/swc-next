use crate::AstUtf8;
use crate::Span;

bitflags::bitflags! {
    /// Line terminators in the trivia surrounding a comment.
    #[repr(transparent)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct CommentNewlines: u8 {
        /// The trivia before the comment contains a line terminator.
        const LEADING = 1 << 0;
        /// The trivia after the comment contains a line terminator.
        /// Line comments always set this flag, including at the end of a file.
        const TRAILING = 1 << 1;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    Line = 0,
    Block = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentPosition {
    Leading = 0,
    Trailing = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comment {
    pub span: Span,
    pub attached_to: u32,
    pub kind: CommentKind,
    pub position: CommentPosition,
    pub newlines: CommentNewlines,
}

impl Comment {
    pub fn new(span: Span, kind: CommentKind) -> Self {
        Self {
            span,
            kind,
            attached_to: 0,
            position: CommentPosition::Trailing,
            newlines: if kind == CommentKind::Line {
                CommentNewlines::TRAILING
            } else {
                CommentNewlines::empty()
            },
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
