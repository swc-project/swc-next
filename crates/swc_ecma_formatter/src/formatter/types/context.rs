//! Type grammar roles decide the parentheses around actual borrowed children.

use swc_next_ecma_ast::{Ast, TsLiteralData, TsType, TsTypeData, TsTypeNameData};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TypeContext {
    Root,
    Parenthesized,
    ArrowReturn,
    PostfixOperand,
    ModifierOperand,
    RestOperand,
    JsDocNullableOperand,
    JsDocPrimaryOperand,
    JsDocPostfixOperand,
    JsDocEnclosedPostfixOperand,
    CompoundMember,
    UnionMember,
    IntersectionLineContinuation,
    ConditionalCheck,
    ConditionalExtends,
    TypeConstraint,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeEnclosure {
    None,
    Parentheses,
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
    /// A required union envelope owns the head inside its grouping pair.
    pub(super) fn preserves_union_head(self, mut node: TsType, ast: &Ast<'_>) -> bool {
        if !matches!(
            self,
            Self::PostfixOperand
                | Self::ModifierOperand
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

    pub(super) fn needs_parentheses(self, data: TsTypeData, ast: &Ast<'_>) -> bool {
        let minimum = match self {
            Self::Root | Self::Parenthesized => return false,
            Self::ArrowReturn => return matches!(data, TsTypeData::TsFunctionType(_)),
            Self::ConditionalCheck => {
                return matches!(
                    data,
                    TsTypeData::TsConditionalType(_)
                        | TsTypeData::TsFunctionType(_)
                        | TsTypeData::TsConstructorType(_)
                        | TsTypeData::TsJsDocNullableType(_)
                        | TsTypeData::TsJsDocUnknownType(_)
                );
            }
            Self::ConditionalExtends => {
                return match data {
                    TsTypeData::TsConditionalType(_) | TsTypeData::TsJsDocUnknownType(_) => true,
                    TsTypeData::TsJsDocNullableType(node) => node.postfix(ast),
                    TsTypeData::TsFunctionType(node) => {
                        return_is_constrained_infer(node.return_type(ast).type_annotation(ast), ast)
                    }
                    TsTypeData::TsConstructorType(node) => {
                        return_is_constrained_infer(node.return_type(ast).type_annotation(ast), ast)
                    }
                    _ => false,
                };
            }
            Self::TypeConstraint => return matches!(data, TsTypeData::TsConditionalType(_)),
            Self::PostfixOperand => {
                if matches!(data, TsTypeData::TsJsDocUnknownType(_)) {
                    return true;
                }
                TypePrecedence::Postfix
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
            Self::CompoundMember | Self::UnionMember | Self::IntersectionLineContinuation => {
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

fn return_is_constrained_infer(mut node: TsType, ast: &Ast<'_>) -> bool {
    loop {
        match ast.ts_type_data(node) {
            TsTypeData::TsParenthesizedType(parentheses) => {
                node = parentheses.type_annotation(ast);
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
