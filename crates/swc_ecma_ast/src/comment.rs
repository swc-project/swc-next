use std::collections::HashMap;
use std::fmt;

use swc_experimental_allocator::{Allocator, atom::Atom, vec::Vec};

use crate::{DUMMY_SP, Span};

#[derive(Clone)]
pub struct Comments<'a> {
    pub leading: HashMap<u32, Vec<'a, Comment<'a>>>,
    pub trailing: HashMap<u32, Vec<'a, Comment<'a>>>,
    allocator: &'a Allocator,
}

impl fmt::Debug for Comments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Comments")
            .field("leading", &self.leading)
            .field("trailing", &self.trailing)
            .finish()
    }
}

impl PartialEq for Comments<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.leading == other.leading && self.trailing == other.trailing
    }
}

impl Eq for Comments<'_> {}

impl<'a> Comments<'a> {
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            leading: HashMap::default(),
            trailing: HashMap::default(),
            allocator,
        }
    }

    pub fn add_leading(&mut self, pos: u32, cmt: Comment<'a>) {
        let allocator = self.allocator;
        self.leading
            .entry(pos)
            .or_insert_with(|| Vec::new_in(allocator))
            .push(cmt);
    }

    pub fn add_leading_comments(
        &mut self,
        pos: u32,
        comments: impl IntoIterator<Item = Comment<'a>>,
    ) {
        let allocator = self.allocator;
        self.leading
            .entry(pos)
            .or_insert_with(|| Vec::new_in(allocator))
            .extend(comments);
    }

    pub fn has_leading(&self, pos: u32) -> bool {
        self.leading.get(&pos).is_some_and(|v| !v.is_empty())
    }

    pub fn move_leading(&mut self, from: u32, to: u32) {
        move_comments(&mut self.leading, from, to);
    }

    pub fn take_leading(&mut self, pos: u32) -> Option<Vec<'a, Comment<'a>>> {
        self.leading.remove(&pos)
    }

    pub fn get_leading(&self, pos: u32) -> Option<Vec<'a, Comment<'a>>> {
        self.leading.get(&pos).cloned()
    }

    pub fn add_trailing(&mut self, pos: u32, cmt: Comment<'a>) {
        let allocator = self.allocator;
        self.trailing
            .entry(pos)
            .or_insert_with(|| Vec::new_in(allocator))
            .push(cmt);
    }

    pub fn add_trailing_comments(
        &mut self,
        pos: u32,
        comments: impl IntoIterator<Item = Comment<'a>>,
    ) {
        let allocator = self.allocator;
        self.trailing
            .entry(pos)
            .or_insert_with(|| Vec::new_in(allocator))
            .extend(comments);
    }

    pub fn has_trailing(&self, pos: u32) -> bool {
        self.trailing.get(&pos).is_some_and(|v| !v.is_empty())
    }

    pub fn move_trailing(&mut self, from: u32, to: u32) {
        move_comments(&mut self.trailing, from, to);
    }

    pub fn take_trailing(&mut self, pos: u32) -> Option<Vec<'a, Comment<'a>>> {
        self.trailing.remove(&pos)
    }

    pub fn get_trailing(&self, pos: u32) -> Option<Vec<'a, Comment<'a>>> {
        self.trailing.get(&pos).cloned()
    }

    pub fn add_pure_comment(&mut self, pos: u32) {
        assert_ne!(pos, 0, "cannot add pure comment to zero position");

        let allocator = self.allocator;
        let leading = self
            .leading
            .entry(pos)
            .or_insert_with(|| Vec::new_in(allocator));
        let text = Atom::new_const("#__PURE__");

        if !leading.iter().any(|c| c.text == text) {
            leading.push(Comment {
                kind: CommentKind::Block,
                span: DUMMY_SP,
                text,
            });
        }
    }

    pub fn with_leading<F, Ret>(&self, pos: u32, op: F) -> Ret
    where
        F: FnOnce(&[Comment<'a>]) -> Ret,
    {
        if let Some(comments) = self.leading.get(&pos) {
            op(comments)
        } else {
            op(&[])
        }
    }

    pub fn with_trailing<F, Ret>(&self, pos: u32, op: F) -> Ret
    where
        F: FnOnce(&[Comment<'a>]) -> Ret,
    {
        if let Some(comments) = self.trailing.get(&pos) {
            op(comments)
        } else {
            op(&[])
        }
    }

    /// This method is used to check if a comment with the given flag exists.
    ///
    /// If `flag` is `PURE`, this method will look for `@__PURE__` and
    /// `#__PURE__`.
    pub fn has_flag(&self, lo: u32, flag: &str) -> bool {
        self.with_leading(lo, |comments| {
            for c in comments {
                if c.kind == CommentKind::Block {
                    for line in c.text.lines() {
                        let line = line.trim_start_matches(['*', ' ']);
                        let line = line.trim();

                        if line.len() == (flag.len() + 5)
                            && (line.starts_with("#__") || line.starts_with("@__"))
                            && line.ends_with("__")
                            && flag == &line[3..line.len() - 2]
                        {
                            return true;
                        }
                    }
                }
            }

            false
        })
    }
}

fn move_comments<'a>(comments: &mut HashMap<u32, Vec<'a, Comment<'a>>>, from: u32, to: u32) {
    if from == to {
        return;
    }

    let Some(moved) = comments.remove(&from) else {
        return;
    };

    let mut moved = moved;
    if from < to && comments.get(&to).is_some_and(|v| !v.is_empty()) {
        moved.extend(comments.remove(&to).unwrap());
    }

    if let Some(target) = comments.get_mut(&to) {
        target.extend(moved);
    } else {
        comments.insert(to, moved);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment<'a> {
    pub kind: CommentKind,
    pub span: Span,
    /// [`Atom::new_bad`][] is perfectly fine for this value.
    pub text: Atom<'a>,
}

impl<'a> Comment<'a> {
    pub fn new(kind: CommentKind, span: Span, text: Atom<'a>) -> Self {
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
