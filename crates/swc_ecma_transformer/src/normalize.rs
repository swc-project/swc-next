use swc_next_ecma_ast::{
    Ast, CommentPosition, Expr, ExprData, GetSpan, Program, SequenceExpression, Span, SubRange,
    TsIntersectionType, TsType, TsTypeData, TsUnionType, VisitMut, VisitMutWith,
};

/// Normalizes parser AST shapes for downstream transforms.
///
/// This pass preserves the node IDs of semantic-bearing leaves while the AST
/// representation is normalized.
pub struct Normalizer {}

impl Normalizer {
    pub fn normalize<'ast>(program: Program, ast: &mut Ast<'ast>) {
        let mut pass = NormalizePass {
            ast,
            leading_comment_retargets: Vec::new(),
            trailing_comment_retargets: Vec::new(),
        };
        program.visit_mut_children_with(&mut pass);
        pass.retarget_comments();
    }
}

struct NormalizePass<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    leading_comment_retargets: Vec<(u32, u32)>,
    trailing_comment_retargets: Vec<(u32, u32)>,
}

#[derive(Clone, Copy)]
enum AssociativeType {
    Union(TsUnionType),
    Intersection(TsIntersectionType),
}

impl AssociativeType {
    fn types(self, ast: &Ast<'_>) -> SubRange<TsType> {
        match self {
            Self::Union(node) => node.types(ast),
            Self::Intersection(node) => node.types(ast),
        }
    }

    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Union(node) => node.span(ast),
            Self::Intersection(node) => node.span(ast),
        }
    }

    fn nested_types(self, node: TsType, ast: &Ast<'_>) -> Option<SubRange<TsType>> {
        match (self, ast.ts_type_data(node)) {
            (Self::Union(_), TsTypeData::TsUnionType(node)) => Some(node.types(ast)),
            (Self::Intersection(_), TsTypeData::TsIntersectionType(node)) => Some(node.types(ast)),
            (_, TsTypeData::TsUnionType(node)) if node.types(ast).len() == 1 => {
                Some(node.types(ast))
            }
            (_, TsTypeData::TsIntersectionType(node)) if node.types(ast).len() == 1 => {
                Some(node.types(ast))
            }
            _ => None,
        }
    }

    fn set_types(self, ast: &mut Ast<'_>, types: SubRange<TsType>) {
        match self {
            Self::Union(node) => node.set_types(ast, types),
            Self::Intersection(node) => node.set_types(ast, types),
        }
    }
}

impl NormalizePass<'_, '_> {
    fn record_replacement(&mut self, removed: Span, replacement: Span) {
        if self.ast.comments().is_empty() {
            return;
        }

        if removed.start != replacement.start {
            self.leading_comment_retargets
                .push((removed.start, replacement.start));
        }
        if removed.end != replacement.end {
            self.trailing_comment_retargets
                .push((removed.end, replacement.end));
        }
    }

    fn replacement_span(&self, flattened: &[Expr], start: usize) -> Option<Span> {
        let first = *flattened.get(start)?;
        let last = *flattened.last()?;
        let first_span = first.span(self.ast);
        let last_span = last.span(self.ast);
        Some(Span::new(first_span.start, last_span.end))
    }

    fn append_sequence_elements(&mut self, expression: Expr, flattened: &mut Vec<Expr>) -> bool {
        let start = flattened.len();
        let removed = match self.ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(parenthesized) => {
                let inner = parenthesized.expression(self.ast);
                self.append_sequence_elements(inner, flattened);
                true
            }
            ExprData::SequenceExpression(sequence) => {
                for expression_id in sequence.expressions(self.ast).iter() {
                    let expression = self.ast.get_node_in_sub_range(expression_id);
                    self.append_sequence_elements(expression, flattened);
                }
                true
            }
            _ => {
                flattened.push(expression);
                false
            }
        };

        let replacement = if removed {
            self.replacement_span(flattened, start)
        } else {
            None
        };
        if let Some(replacement) = replacement {
            self.record_replacement(expression.span(self.ast), replacement);
        }

        removed
    }

    fn normalize_sequence(&mut self, sequence: SequenceExpression) -> Option<Expr> {
        let expressions = sequence.expressions(self.ast);
        let mut flattened = Vec::with_capacity(expressions.len());
        let mut changed = false;

        for expression_id in expressions.iter() {
            let expression = self.ast.get_node_in_sub_range(expression_id);
            changed |= self.append_sequence_elements(expression, &mut flattened);
        }

        if flattened.len() == 1 {
            let replacement = flattened.pop().unwrap();
            self.record_replacement(sequence.span(self.ast), replacement.span(self.ast));
            return Some(replacement);
        }

        if changed {
            let expressions = self.ast.add_sub_range(flattened);
            sequence.set_expressions(self.ast, expressions);
        }

        None
    }

    fn append_type_members(
        &mut self,
        outer: AssociativeType,
        node: TsType,
        flattened: &mut Vec<TsType>,
    ) -> Span {
        let removed = node.span(self.ast);
        if let TsTypeData::TsParenthesizedType(parenthesized) = self.ast.ts_type_data(node) {
            let inner = parenthesized.type_annotation(self.ast);
            let replacement = self.append_type_members(outer, inner, flattened);
            self.record_replacement(removed, replacement);
            return replacement;
        }
        let Some(members) = outer.nested_types(node, self.ast) else {
            let member = node.visit_mut_with(self).unwrap_or(node);
            flattened.push(member);
            return member.span(self.ast);
        };
        if members.len() == 1
            && let Some(member) = members.get_node(self.ast, 0)
        {
            let replacement = self.append_type_members(outer, member, flattened);
            self.record_replacement(removed, replacement);
            return replacement;
        }
        let start = flattened.len();
        for id in members.iter() {
            let member = self.ast.get_node_in_sub_range(id);
            self.append_type_members(outer, member, flattened);
        }
        if let (Some(first), Some(last)) = (flattened.get(start), flattened.last()) {
            let replacement = Span::new(first.span(self.ast).start, last.span(self.ast).end);
            self.record_replacement(removed, replacement);
            return replacement;
        }
        removed
    }

    fn normalize_type_list(&mut self, node: AssociativeType) -> Option<TsType> {
        let types = node.types(self.ast);
        if types.len() == 1 {
            let member = types.get_node(self.ast, 0)?;
            let replacement = member.visit_mut_with(self).unwrap_or(member);
            self.record_replacement(node.span(self.ast), replacement.span(self.ast));
            return Some(replacement);
        }

        // Keep flat lists in place. Collect nested members directly into the
        // outer list instead of allocating and copying a list at every level.
        let needs_flattening = types.iter().any(|id| {
            let member = self.ast.get_node_in_sub_range(id);
            matches!(
                self.ast.ts_type_data(member),
                TsTypeData::TsParenthesizedType(_)
            ) || node.nested_types(member, self.ast).is_some()
        });
        if !needs_flattening {
            types.visit_mut_with(self);
            return None;
        }

        let mut flattened = Vec::with_capacity(types.len());
        for id in types.iter() {
            let member = self.ast.get_node_in_sub_range(id);
            self.append_type_members(node, member, &mut flattened);
        }
        if flattened.len() == 1 {
            let replacement = flattened[0];
            self.record_replacement(node.span(self.ast), replacement.span(self.ast));
            return Some(replacement);
        }
        let types = self.ast.add_sub_range(flattened);
        node.set_types(self.ast, types);
        None
    }

    fn compact_retargets(retargets: &mut Vec<(u32, u32)>, select: fn(u32, u32) -> u32) {
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

    fn retarget_comments(mut self) {
        if self.leading_comment_retargets.is_empty() && self.trailing_comment_retargets.is_empty() {
            return;
        }

        Self::compact_retargets(&mut self.leading_comment_retargets, u32::max);
        Self::compact_retargets(&mut self.trailing_comment_retargets, u32::min);

        let leading = &self.leading_comment_retargets;
        let trailing = &self.trailing_comment_retargets;
        self.ast.retarget_comments(|position, attached_to| {
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
}

impl<'ast> VisitMut<'ast> for NormalizePass<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        match self.ast.expr_data(node) {
            ExprData::ParenthesizedExpression(parenthesized) => {
                let expression = parenthesized.expression(self.ast);
                let replacement = expression.visit_mut_with(self).unwrap_or(expression);
                self.record_replacement(node.span(self.ast), replacement.span(self.ast));
                Some(replacement)
            }
            ExprData::SequenceExpression(sequence) => {
                if let Some(expression) = self.normalize_sequence(sequence) {
                    Some(expression.visit_mut_with(self).unwrap_or(expression))
                } else {
                    sequence.visit_mut_children_with(self);
                    None
                }
            }
            _ => node.visit_mut_children_with(self),
        }
    }

    fn visit_mut_ts_type(&mut self, node: TsType) -> Option<TsType> {
        match self.ast.ts_type_data(node) {
            TsTypeData::TsParenthesizedType(parenthesized) => {
                let type_annotation = parenthesized.type_annotation(self.ast);
                let replacement = type_annotation
                    .visit_mut_with(self)
                    .unwrap_or(type_annotation);
                self.record_replacement(node.span(self.ast), replacement.span(self.ast));
                Some(replacement)
            }
            TsTypeData::TsUnionType(union) => {
                self.normalize_type_list(AssociativeType::Union(union))
            }
            TsTypeData::TsIntersectionType(intersection) => {
                self.normalize_type_list(AssociativeType::Intersection(intersection))
            }
            _ => node.visit_mut_children_with(self),
        }
    }
}
