use swc_next_ecma_ast::{Ast, CommentPosition, Span};

/// Collects comment attachment changes while one AST node is replaced by another.
///
/// Transform passes record replacements as they mutate the tree, then apply all
/// retargets together so chained replacements are deterministic.
#[derive(Default)]
pub(crate) struct CommentRetargets {
    leading: Vec<(u32, u32)>,
    trailing: Vec<(u32, u32)>,
}

impl CommentRetargets {
    pub(crate) fn record_replacement(&mut self, ast: &Ast<'_>, removed: Span, replacement: Span) {
        if ast.comments().is_empty() {
            return;
        }

        if removed.start != replacement.start {
            self.leading.push((removed.start, replacement.start));
        }
        if removed.end != replacement.end {
            self.trailing.push((removed.end, replacement.end));
        }
    }

    pub(crate) fn apply(mut self, ast: &mut Ast<'_>) {
        if self.leading.is_empty() && self.trailing.is_empty() {
            return;
        }

        Self::compact(&mut self.leading, u32::max);
        Self::compact(&mut self.trailing, u32::min);

        let leading = &self.leading;
        let trailing = &self.trailing;
        ast.retarget_comments(|position, attached_to| {
            let retargets = match position {
                CommentPosition::Leading => leading,
                CommentPosition::Trailing => trailing,
            };
            let index = retargets
                .binary_search_by_key(&attached_to, |retarget| retarget.0)
                .ok()?;
            Some(retargets[index].1)
        });
    }

    fn compact(retargets: &mut Vec<(u32, u32)>, select: fn(u32, u32) -> u32) {
        retargets.sort_unstable_by_key(|retarget| retarget.0);

        let mut write = 0;
        for read in 0..retargets.len() {
            let (source, target) = retargets[read];
            if write > 0 && retargets[write - 1].0 == source {
                retargets[write - 1].1 = select(retargets[write - 1].1, target);
            } else {
                retargets[write] = (source, target);
                write += 1;
            }
        }
        retargets.truncate(write);
    }
}
