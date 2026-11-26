//! Type grammar roles decide the parentheses around actual borrowed children.

use swc_next_ecma_ast::{
    Ast, FormalParameterItemData, TsLiteralData, TsType, TsTypeData, TsTypeNameData,
};

use crate::document::BodyOpening;
use crate::formatter::{MAX_RECURSION_DEPTH, suppression::ContentMode};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TypeContext {
    Root,
    BeforeKeyword,
    TypeExpressionRhs {
        before_keyword: bool,
    },
    Parenthesized,
    ArrowReturn,
    BeforeFunctionBody,
    AngleAssertion,
    PostfixOperand,
    ModifierOperand,
    ModifierClosedOperand,
    RestOperand,
    JsDocNullableOperand,
    JsDocPrimaryOperand,
    JsDocPostfixOperand,
    JsDocEnclosedPostfixOperand,
    CompoundMember,
    UnionMember,
    IntersectionMember {
        line_continuation: bool,
        body_opening: BodyOpening,
    },
    ConditionalCheck,
    ConditionalExtends,
    TypeConstraint,
    InferConstraint,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeEnclosure {
    None,
    Parentheses,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeFrame {
    None,
    Tight,
    Block,
}

impl TypeFrame {
    pub(super) fn tight(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Tight | Self::Block => Self::Tight,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TypePrecedence {
    Conditional,
    Union,
    Intersection,
    Operator,
    Query,
    Postfix,
    Atomic,
}

impl TypeContext {
    pub(super) fn body_opening(self) -> BodyOpening {
        match self {
            Self::IntersectionMember { body_opening, .. } => body_opening,
            _ => BodyOpening::Plain,
        }
    }

    pub(super) fn without_body_opening(self) -> Self {
        match self {
            Self::IntersectionMember {
                line_continuation, ..
            } => Self::IntersectionMember {
                line_continuation,
                body_opening: BodyOpening::Plain,
            },
            _ => self,
        }
    }

    /// A required union envelope owns the head inside its grouping pair.
    pub(super) fn preserves_union_head(self, mut node: TsType, ast: &Ast<'_>) -> bool {
        if !matches!(
            self,
            Self::PostfixOperand
                | Self::ModifierOperand
                | Self::ModifierClosedOperand
                | Self::RestOperand
                | Self::JsDocNullableOperand
                | Self::JsDocPrimaryOperand
                | Self::JsDocPostfixOperand
                | Self::JsDocEnclosedPostfixOperand
        ) {
            return false;
        }
        loop {
            let data = ast.ts_type_data(node);
            let types = match data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    node = parentheses.type_annotation(ast);
                    continue;
                }
                TsTypeData::TsUnionType(union) => union.types(ast),
                TsTypeData::TsIntersectionType(intersection) => intersection.types(ast),
                _ => return false,
            };
            if types.len() != 1 {
                return matches!(data, TsTypeData::TsUnionType(_))
                    && self.needs_parentheses(data, ast);
            }
            let Some(first) = types.iter().next() else {
                return false;
            };
            node = ast.get_node_in_sub_range(first);
        }
    }

    pub(super) fn enclosure(self) -> TypeEnclosure {
        if matches!(self, Self::Parenthesized) {
            TypeEnclosure::Parentheses
        } else {
            TypeEnclosure::None
        }
    }

    pub(super) fn frame(self, data: TsTypeData, ast: &Ast<'_>) -> TypeFrame {
        if !self.needs_parentheses(data, ast) {
            return TypeFrame::None;
        }
        match (data, self) {
            (TsTypeData::TsUnionType(_), _)
            | (
                TsTypeData::TsConditionalType(_),
                Self::ConditionalCheck | Self::ConditionalExtends,
            ) => TypeFrame::Block,
            _ => TypeFrame::Tight,
        }
    }

    pub(super) fn needs_verbatim_keyword_parentheses(
        self,
        data: TsTypeData,
        ast: &Ast<'_>,
    ) -> bool {
        matches!(
            self,
            Self::TypeExpressionRhs {
                before_keyword: true
            }
        ) && ends_with_bare_asserts(data, ast, ContentMode::Verbatim)
    }

    pub(super) fn needs_parentheses(self, data: TsTypeData, ast: &Ast<'_>) -> bool {
        let minimum = match self {
            Self::Root | Self::Parenthesized => return false,
            Self::BeforeKeyword => {
                return ends_with_bare_asserts(data, ast, ContentMode::Formatted);
            }
            Self::TypeExpressionRhs { before_keyword } => {
                return before_keyword && ends_with_bare_asserts(data, ast, ContentMode::Formatted)
                    || match data {
                        TsTypeData::TsParenthesizedType(_) => false,
                        TsTypeData::TsTypeOperator(node) => {
                            super::type_operator_requires_terminator(node, ast)
                        }
                        _ => super::type_data_requires_terminator(data, ast),
                    };
            }
            Self::ArrowReturn => return matches!(data, TsTypeData::TsFunctionType(_)),
            Self::BeforeFunctionBody => {
                return matches!(
                    super::terminal_type_data(data, ast, true),
                    TsTypeData::TsJsDocUnknownType(_)
                );
            }
            Self::AngleAssertion => {
                return conditional_assertion_type_can_start_arrow(data, ast, MAX_RECURSION_DEPTH);
            }
            Self::ConditionalCheck => {
                return matches!(
                    data,
                    TsTypeData::TsConditionalType(_)
                        | TsTypeData::TsFunctionType(_)
                        | TsTypeData::TsConstructorType(_)
                        | TsTypeData::TsJsDocNullableType(_)
                        | TsTypeData::TsJsDocUnknownType(_)
                ) || ends_with_bare_asserts(data, ast, ContentMode::Formatted);
            }
            Self::ConditionalExtends => {
                return match data {
                    TsTypeData::TsConditionalType(_) | TsTypeData::TsJsDocUnknownType(_) => true,
                    TsTypeData::TsJsDocNullableType(node) => node.postfix(ast),
                    TsTypeData::TsFunctionType(node) => {
                        return_is_constrained_infer(node.return_type(ast).type_annotation(ast), ast)
                            || exposes_conditional_type(data, ast, MAX_RECURSION_DEPTH)
                    }
                    TsTypeData::TsConstructorType(node) => {
                        return_is_constrained_infer(node.return_type(ast).type_annotation(ast), ast)
                            || exposes_conditional_type(data, ast, MAX_RECURSION_DEPTH)
                    }
                    _ => false,
                };
            }
            Self::TypeConstraint => return matches!(data, TsTypeData::TsConditionalType(_)),
            Self::InferConstraint => {
                return match data {
                    TsTypeData::TsConditionalType(_) => true,
                    TsTypeData::TsFunctionType(_) | TsTypeData::TsConstructorType(_) => {
                        exposes_conditional_type(data, ast, MAX_RECURSION_DEPTH)
                    }
                    _ => false,
                };
            }
            Self::PostfixOperand => {
                if matches!(data, TsTypeData::TsJsDocUnknownType(_)) {
                    return true;
                }
                TypePrecedence::Postfix
            }
            Self::ModifierClosedOperand => {
                // The bare question mark remains open to the following type;
                // enclosing the whole operator can parse as a function type.
                if matches!(data, TsTypeData::TsJsDocUnknownType(_)) {
                    return true;
                }
                TypePrecedence::Query
            }
            Self::ModifierOperand => TypePrecedence::Query,
            Self::RestOperand => {
                if matches!(data, TsTypeData::TsInferType(_)) {
                    return false;
                }
                TypePrecedence::Query
            }
            Self::JsDocNullableOperand => {
                if matches!(data, TsTypeData::TsJsDocUnknownType(_)) {
                    return true;
                }
                TypePrecedence::Query
            }
            Self::JsDocPrimaryOperand => {
                // Prefix ! consumes a primary type; postfix children need their own pair.
                if matches!(
                    data,
                    TsTypeData::TsArrayType(_)
                        | TsTypeData::TsIndexedAccessType(_)
                        | TsTypeData::TsJsDocNullableType(_)
                ) || matches!(data, TsTypeData::TsJsDocNonNullableType(node) if node.postfix(ast))
                {
                    return true;
                }
                TypePrecedence::Query
            }
            Self::JsDocPostfixOperand | Self::JsDocEnclosedPostfixOperand => {
                // Close nullable/unknown children before the next marker's type-start lookahead.
                if matches!(
                    data,
                    TsTypeData::TsJsDocNullableType(_) | TsTypeData::TsJsDocUnknownType(_)
                ) || matches!(self, Self::JsDocEnclosedPostfixOperand)
                    && jsdoc_operand_looks_like_parameter(data, ast)
                {
                    return true;
                }
                TypePrecedence::Query
            }
            Self::CompoundMember | Self::UnionMember | Self::IntersectionMember { .. } => {
                if matches!(
                    data,
                    TsTypeData::TsJsDocNullableType(_) | TsTypeData::TsJsDocUnknownType(_)
                ) {
                    return true;
                }
                if let TsTypeData::TsInferType(node) = data {
                    return node.type_parameter(ast).constraint(ast).is_some();
                }
                TypePrecedence::Operator
            }
        };
        let precedence = match data {
            TsTypeData::TsConditionalType(_) => TypePrecedence::Conditional,
            TsTypeData::TsUnionType(_)
            | TsTypeData::TsFunctionType(_)
            | TsTypeData::TsConstructorType(_) => TypePrecedence::Union,
            TsTypeData::TsIntersectionType(_) => TypePrecedence::Intersection,
            TsTypeData::TsTypeOperator(_) | TsTypeData::TsInferType(_) => TypePrecedence::Operator,
            TsTypeData::TsTypeQuery(_) => TypePrecedence::Query,
            TsTypeData::TsJsDocNullableType(node) => {
                if node.postfix(ast) {
                    // A following bracket would make ? look like a conditional separator.
                    TypePrecedence::Query
                } else {
                    TypePrecedence::Conditional
                }
            }
            TsTypeData::TsJsDocNonNullableType(node) if node.postfix(ast) => {
                TypePrecedence::Postfix
            }
            TsTypeData::TsArrayType(_) | TsTypeData::TsIndexedAccessType(_) => {
                TypePrecedence::Postfix
            }
            _ => TypePrecedence::Atomic,
        };
        precedence < minimum
    }
}

/// TypeScript can read the parameters of a conditional branch as a generic
/// arrow's parameters when the conditional immediately follows an opening angle.
fn conditional_assertion_type_can_start_arrow(
    data: TsTypeData,
    ast: &Ast<'_>,
    remaining: usize,
) -> bool {
    if remaining == 0 {
        return false;
    }
    let TsTypeData::TsConditionalType(conditional) = data else {
        return false;
    };
    [conditional.true_type(ast), conditional.false_type(ast)]
        .into_iter()
        .any(|mut branch| {
            while let TsTypeData::TsParenthesizedType(node) = ast.ts_type_data(branch) {
                branch = node.type_annotation(ast);
            }
            let branch = ast.ts_type_data(branch);
            matches!(branch, TsTypeData::TsFunctionType(_))
                || conditional_assertion_type_can_start_arrow(branch, ast, remaining - 1)
        })
}

// An enclosing type pair speculates a function when a single identifier-like
// operand is followed by ?. The child pair changes that actual grammar head.
fn jsdoc_operand_looks_like_parameter(data: TsTypeData, ast: &Ast<'_>) -> bool {
    match data {
        TsTypeData::TsTypeReference(node) => {
            node.type_arguments(ast).is_none()
                && matches!(
                    ast.ts_type_name_data(node.type_name(ast)),
                    TsTypeNameData::IdentifierReference(_)
                )
        }
        TsTypeData::TsAnyKeyword(_)
        | TsTypeData::TsUnknownKeyword(_)
        | TsTypeData::TsNeverKeyword(_)
        | TsTypeData::TsVoidKeyword(_)
        | TsTypeData::TsNullKeyword(_)
        | TsTypeData::TsUndefinedKeyword(_)
        | TsTypeData::TsStringKeyword(_)
        | TsTypeData::TsNumberKeyword(_)
        | TsTypeData::TsBigIntKeyword(_)
        | TsTypeData::TsBooleanKeyword(_)
        | TsTypeData::TsSymbolKeyword(_)
        | TsTypeData::TsObjectKeyword(_)
        | TsTypeData::TsIntrinsicKeyword(_) => true,
        TsTypeData::TsLiteralType(node) => {
            matches!(
                ast.ts_literal_data(node.literal(ast)),
                TsLiteralData::BooleanLiteral(_)
            )
        }
        _ => false,
    }
}

fn type_without_parentheses(mut data: TsTypeData, ast: &Ast<'_>) -> TsTypeData {
    while let TsTypeData::TsParenthesizedType(pair) = data {
        data = ast.ts_type_data(pair.type_annotation(ast));
    }
    data
}

/// A callable's parameters and defaults inherit the surrounding parser restriction.
/// Return types and delimited type containers have their own conditional scope.
fn exposes_conditional_type(data: TsTypeData, ast: &Ast<'_>, depth: usize) -> bool {
    let (parameters, this, type_parameters) = match type_without_parentheses(data, ast) {
        TsTypeData::TsConditionalType(_) => return true,
        TsTypeData::TsFunctionType(node) => (
            node.params(ast),
            node.this_param(ast),
            node.type_parameters(ast),
        ),
        TsTypeData::TsConstructorType(node) => (node.params(ast), None, node.type_parameters(ast)),
        _ => return false,
    };
    if depth == 0 {
        return true;
    }
    let exposes = |node: TsType| exposes_conditional_type(ast.ts_type_data(node), ast, depth - 1);
    if this
        .and_then(|node| node.type_annotation(ast))
        .is_some_and(|annotation| exposes(annotation.type_annotation(ast)))
    {
        return true;
    }
    if parameters.items(ast).iter().any(|id| {
        let parameter = match ast.formal_parameter_item_data(ast.get_node_in_sub_range(id)) {
            FormalParameterItemData::FormalParameter(node) => node,
            FormalParameterItemData::TsParameterProperty(node) => node.parameter(ast),
        };
        parameter
            .type_annotation(ast)
            .is_some_and(|annotation| exposes(annotation.type_annotation(ast)))
    }) || parameters
        .rest(ast)
        .and_then(|rest| rest.type_annotation(ast))
        .is_some_and(|annotation| exposes(annotation.type_annotation(ast)))
    {
        return true;
    }
    type_parameters.is_some_and(|parameters| {
        parameters.params(ast).iter().any(|id| {
            let parameter = ast.get_node_in_sub_range(id);
            parameter.default(ast).is_some_and(exposes)
                || parameter.constraint(ast).is_some_and(|constraint| {
                    let data = type_without_parentheses(ast.ts_type_data(constraint), ast);
                    // A direct conditional constraint already retains its own pair.
                    !matches!(data, TsTypeData::TsConditionalType(_))
                        && exposes_conditional_type(data, ast, depth - 1)
                })
        })
    })
}

// On the same line, a following keyword can turn a bare `asserts` into
// the head of a type predicate. Follow the right edge until a delimiter or
// a grammar pair supplied by the child position closes it.
fn ends_with_bare_asserts(mut data: TsTypeData, ast: &Ast<'_>, mode: ContentMode) -> bool {
    let mut context = TypeContext::Root;
    loop {
        if mode == ContentMode::Formatted {
            data = type_without_parentheses(data, ast);
            if context.needs_parentheses(data, ast) {
                return false;
            }
        }
        let (child, child_context) = match data {
            TsTypeData::TsTypeReference(reference) => {
                if reference.type_arguments(ast).is_some() {
                    return false;
                }
                let TsTypeNameData::IdentifierReference(name) =
                    ast.ts_type_name_data(reference.type_name(ast))
                else {
                    return false;
                };
                return ast.get_utf8(name.name(ast)) == "asserts";
            }
            TsTypeData::TsUnionType(union) => {
                let Some(last) = union.types(ast).last() else {
                    return false;
                };
                (ast.get_node_in_sub_range(last), TypeContext::UnionMember)
            }
            TsTypeData::TsIntersectionType(intersection) => {
                let Some(last) = intersection.types(ast).last() else {
                    return false;
                };
                (ast.get_node_in_sub_range(last), TypeContext::CompoundMember)
            }
            TsTypeData::TsTypeOperator(operator) => (
                operator.type_annotation(ast),
                TypeContext::ModifierClosedOperand,
            ),
            TsTypeData::TsJsDocNullableType(node) if !node.postfix(ast) => {
                (node.type_annotation(ast), TypeContext::JsDocNullableOperand)
            }
            TsTypeData::TsJsDocNonNullableType(node) if !node.postfix(ast) => {
                (node.type_annotation(ast), TypeContext::JsDocPrimaryOperand)
            }
            TsTypeData::TsInferType(infer) => {
                let Some(constraint) = infer.type_parameter(ast).constraint(ast) else {
                    return false;
                };
                (constraint, TypeContext::InferConstraint)
            }
            TsTypeData::TsTypePredicate(predicate) => {
                let Some(annotation) = predicate.type_annotation(ast) else {
                    return false;
                };
                (annotation.type_annotation(ast), TypeContext::Root)
            }
            TsTypeData::TsFunctionType(function) => (
                function.return_type(ast).type_annotation(ast),
                TypeContext::Root,
            ),
            TsTypeData::TsConstructorType(constructor) => (
                constructor.return_type(ast).type_annotation(ast),
                TypeContext::Root,
            ),
            TsTypeData::TsConditionalType(conditional) => {
                (conditional.false_type(ast), TypeContext::Root)
            }
            _ => return false,
        };
        data = ast.ts_type_data(child);
        context = child_context;
    }
}

// A trailing `?` can consume the constraint as a conditional return type.
// Only the path to the printed right edge can expose that ambiguity.
fn return_is_constrained_infer(mut node: TsType, ast: &Ast<'_>) -> bool {
    loop {
        match ast.ts_type_data(node) {
            TsTypeData::TsParenthesizedType(parentheses) => {
                node = parentheses.type_annotation(ast);
            }
            TsTypeData::TsFunctionType(function) => {
                node = function.return_type(ast).type_annotation(ast);
            }
            TsTypeData::TsConstructorType(constructor) => {
                node = constructor.return_type(ast).type_annotation(ast);
            }
            TsTypeData::TsConditionalType(conditional) => {
                node = conditional.false_type(ast);
            }
            TsTypeData::TsTypePredicate(predicate) => {
                let Some(annotation) = predicate.type_annotation(ast) else {
                    return false;
                };
                node = annotation.type_annotation(ast);
            }
            TsTypeData::TsInferType(infer) => {
                return infer.type_parameter(ast).constraint(ast).is_some();
            }
            _ => return false,
        }
    }
}
