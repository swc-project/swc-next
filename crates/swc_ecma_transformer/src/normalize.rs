use swc_next_ecma_ast::{
    Ast, Expr, ExprData, GetSpan, Program, SequenceExpression, Span, SubRange,
    TsInterfaceDeclaration, TsIntersectionType, TsType, TsTypeAliasDeclaration, TsTypeData,
    TsUnionType, VisitMut, VisitMutWith,
};

use crate::context::CommentState;

/// Normalizes parser AST shapes for downstream transforms.
///
/// Run this pass before semantic analysis so scopes and export targets are
/// collected from the normalized tree.
pub struct Normalizer {}

impl Normalizer {
    pub fn normalize<'ast>(program: Program, ast: &mut Ast<'ast>) {
        Self::run::<true>(program, ast);
    }

    /// Normalize runtime expressions while retaining TypeScript declaration shapes.
    pub fn normalize_runtime<'ast>(program: Program, ast: &mut Ast<'ast>) {
        Self::run::<false>(program, ast);
    }

    fn run<'ast, const NORMALIZE_TYPES: bool>(program: Program, ast: &mut Ast<'ast>) {
        let mut pass = NormalizePass::<NORMALIZE_TYPES> {
            ast,
            comments: CommentState::default(),
        };
        program.visit_mut_children_with(&mut pass);
        pass.comments.flush(pass.ast);
    }
}

struct NormalizePass<'visit, 'ast, const NORMALIZE_TYPES: bool> {
    ast: &'visit mut Ast<'ast>,
    comments: CommentState,
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

impl<const NORMALIZE_TYPES: bool> NormalizePass<'_, '_, NORMALIZE_TYPES> {
    fn record_replacement(&mut self, removed: Span, replacement: Span) {
        if !self.ast.comments().is_empty() {
            self.comments.inherit(replacement, removed);
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
}

impl<'ast, const NORMALIZE_TYPES: bool> VisitMut<'ast>
    for NormalizePass<'_, 'ast, NORMALIZE_TYPES>
{
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

    fn visit_mut_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
    ) -> Option<TsTypeAliasDeclaration> {
        if NORMALIZE_TYPES {
            node.visit_mut_children_with(self);
        }
        None
    }

    fn visit_mut_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
    ) -> Option<TsInterfaceDeclaration> {
        if NORMALIZE_TYPES {
            node.visit_mut_children_with(self);
        }
        None
    }

    fn visit_mut_ts_type(&mut self, node: TsType) -> Option<TsType> {
        if !NORMALIZE_TYPES {
            return None;
        }
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
