//! Grammar roles for borrowed type operands, including opaque source children.
use swc_next_ecma_ast::{Ast, TsType, TsTypeData};

#[derive(Clone, Copy)]
pub(super) enum TypeContext {
    Root,
    Parenthesized,
    PostfixOperand,
    ModifierOperand,
    RestOperand,
    JsDocPostfixOperand,
    CompoundMember,
    UnionMember,
    IntersectionLineContinuation,
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
    pub(super) fn preserves_union_head(self, mut node: TsType, ast: &Ast<'_>) -> bool {
        if !matches!(
            self,
            Self::PostfixOperand
                | Self::ModifierOperand
                | Self::RestOperand
                | Self::JsDocPostfixOperand
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
            let Some(first) = types.first() else {
                return false;
            };
            node = ast.get_node_in_sub_range(first);
        }
    }

    pub(super) fn needs_parentheses(self, data: TsTypeData, ast: &Ast<'_>) -> bool {
        let minimum = match self {
            Self::Root | Self::Parenthesized => return false,
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
            Self::JsDocPostfixOperand => {
                if matches!(
                    data,
                    TsTypeData::TsJsDocNullableType(_) | TsTypeData::TsJsDocUnknownType(_)
                ) {
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
