//! Value expressions that can reject a speculative type-argument list.

use swc_next_ecma_ast::{
    ArgumentData, ArrowFunctionBodyData, BinaryOperator, CallExpression, Expr, ExprData,
    ObjectPropertyKindData, PropertyKind, UnaryOperator,
};

use crate::printer::{Printer, PrinterMode};

impl<'a, M: PrinterMode> Printer<'a, M> {
    pub(in crate::printer) fn syntax_rejects_type(&self, mut expr: Expr) -> bool {
        // Complete type forms compose through tuple elements, property types,
        // template holes, type operators and function returns. Computed keys
        // and parameter initializers use value grammar, so do not inspect them.
        // Keep the worklist iterative: a value bitwise chain can be arbitrarily
        // deep even though the ordinary binary printer uses a heap stack.
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum TypeEnd {
            Delimiter,
            Member,
        }
        let mut remaining = Vec::new();
        let mut end = TypeEnd::Delimiter;
        loop {
            match self.ast.expr_data(expr) {
                ExprData::ParenthesizedExpression(e) => {
                    expr = e.expression(self.ast);
                    end = TypeEnd::Delimiter;
                    continue;
                }
                ExprData::CallExpression(call) => {
                    if self.call_has_value_head(call) {
                        // print_call writes `(` immediately after the callee.
                        // It cannot become a separate type call signature by
                        // ASI, even when the source had a line break there.
                        return true;
                    } else if let ArgumentData::Expr(argument) = self
                        .ast
                        .argument_data(call.arguments(self.ast).get_node(self.ast, 0).unwrap())
                    {
                        expr = argument;
                        end = TypeEnd::Delimiter;
                        continue;
                    }
                }
                ExprData::TaggedTemplateExpression(tagged) => {
                    if tagged.type_arguments(self.ast).is_some()
                        || !self.is_type_operator_head(tagged.tag(self.ast))
                    {
                        return true;
                    }
                    expr = Expr::TemplateLiteral(tagged.quasi(self.ast));
                    continue;
                }
                ExprData::ArrayExpression(array) => {
                    for slot in array.elements(self.ast).iter() {
                        let Some(argument) = self.ast.get_node_in_sub_range(slot) else {
                            // A tuple type cannot contain an omitted element.
                            return true;
                        };
                        remaining.push((
                            match self.ast.argument_data(argument) {
                                ArgumentData::Expr(value) => value,
                                ArgumentData::SpreadElement(spread) => spread.argument(self.ast),
                            },
                            TypeEnd::Delimiter,
                        ));
                    }
                }
                ExprData::ObjectExpression(object) => {
                    for slot in object.properties(self.ast).iter() {
                        let ObjectPropertyKindData::ObjectProperty(property) = self
                            .ast
                            .object_property_kind_data(self.ast.get_node_in_sub_range(slot))
                        else {
                            // Type literals have no object spread member.
                            return true;
                        };
                        if property.kind(self.ast) != PropertyKind::Init {
                            // TS type members retain accessor bodies and parse
                            // their contents using value grammar.
                            continue;
                        }
                        if property.method(self.ast) {
                            // A method signature cannot consume a value body.
                            return true;
                        }
                        if !self.can_use_shorthand_property(property) {
                            remaining.push((property.value(self.ast), TypeEnd::Member));
                        }
                    }
                }
                ExprData::TemplateLiteral(template) => {
                    remaining.extend(
                        template
                            .expressions(self.ast)
                            .iter()
                            .map(|slot| (self.ast.get_node_in_sub_range(slot), TypeEnd::Delimiter)),
                    );
                }
                ExprData::Function(_)
                | ExprData::Class(_)
                | ExprData::NewExpression(_)
                | ExprData::RegExpLiteral(_)
                | ExprData::UpdateExpression(_)
                | ExprData::SequenceExpression(_)
                | ExprData::TsAsExpression(_)
                | ExprData::TsSatisfiesExpression(_)
                | ExprData::TsTypeAssertion(_)
                | ExprData::JsxElement(_)
                | ExprData::JsxFragment(_) => return true,
                ExprData::AssignmentExpression(_) => {
                    // A property signature can retain `p: T = value`; its
                    // initializer is value grammar, not another type.
                    if end != TypeEnd::Member {
                        return true;
                    }
                }
                ExprData::AwaitExpression(_) | ExprData::ConditionalExpression(_) => {
                    // Keyword tokens can become following property names
                    // across emitted line breaks. Preserve that possibility.
                    if end != TypeEnd::Member {
                        return true;
                    }
                }
                ExprData::LogicalExpression(_) => return true,
                ExprData::UnaryExpression(e) => {
                    match e.operator(self.ast) {
                        UnaryOperator::Positive | UnaryOperator::BitwiseNot => return true,
                        UnaryOperator::Delete | UnaryOperator::Void => {
                            if end != TypeEnd::Member {
                                return true;
                            }
                        }
                        UnaryOperator::LogicalNot => {
                            expr = e.argument(self.ast);
                            continue;
                        }
                        UnaryOperator::Negate => {
                            let mut argument = e.argument(self.ast);
                            while let ExprData::TsNonNullExpression(non_null) =
                                self.ast.expr_data(argument)
                            {
                                argument = non_null.expression(self.ast);
                            }
                            if !matches!(
                                self.ast.expr_data(argument),
                                ExprData::NumericLiteral(_) | ExprData::BigIntLiteral(_)
                            ) {
                                return true;
                            }
                        }
                        // Type queries retain the conservative entity-name
                        // interpretation until their operand is summarized.
                        _ => {}
                    }
                }
                ExprData::BinaryExpression(binary) => match binary.operator(self.ast) {
                    BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOr => {
                        remaining.push((binary.right(self.ast), end));
                        expr = binary.left(self.ast);
                        end = TypeEnd::Delimiter;
                        continue;
                    }
                    BinaryOperator::In | BinaryOperator::Instanceof if end == TypeEnd::Member => {}
                    _ => return true,
                },
                ExprData::TsNonNullExpression(non_null) => {
                    expr = non_null.expression(self.ast);
                    continue;
                }
                ExprData::ChainExpression(chain) => {
                    expr = chain.expression(self.ast);
                    continue;
                }
                ExprData::ArrowFunctionExpression(arrow) => {
                    if arrow.r#async(self.ast)
                        || arrow.return_type(self.ast).is_some()
                        || arrow.type_parameters(self.ast).is_none()
                            && self.bare_arrow_parameter(arrow.params(self.ast)).is_some()
                    {
                        return true;
                    }
                    if let ArrowFunctionBodyData::Expr(body) =
                        self.ast.arrow_function_body_data(arrow.body(self.ast))
                    {
                        expr = body;
                        continue;
                    }
                }
                _ => {}
            }
            let Some((next, tail)) = remaining.pop() else {
                return false;
            };
            expr = next;
            end = tail;
        }
    }

    fn call_has_value_head(&self, call: CallExpression) -> bool {
        call.optional(self.ast)
            || call.type_arguments(self.ast).is_some()
            || call.arguments(self.ast).len() != 1
            || !self.is_type_operator_head(call.callee(self.ast))
    }

    fn is_type_operator_head(&self, mut callee: Expr) -> bool {
        loop {
            match self.ast.expr_data(callee) {
                ExprData::IdentifierReference(identifier) => {
                    // `keyof(b)` is both a value call and a valid type operator.
                    return matches!(
                        self.ast.get_utf8(identifier.name(self.ast)),
                        "keyof" | "readonly" | "unique"
                    );
                }
                ExprData::TsNonNullExpression(expression) => {
                    callee = expression.expression(self.ast);
                }
                _ => return false,
            }
        }
    }
}
