//! TypeScript can rescan the two tokens in a left shift as the start of
//! a type argument containing a generic function type. Its type parameter
//! list can consume another binding (and its default) across a line break.

use swc_next_ecma_ast::{
    AssignmentOperator, Ast, BinaryOperator, BindingPatternData, ClassBody, DeclData, Expr,
    ExprData, FunctionBody, JsxElement, JsxFragment, StmtData, TsType, TsTypeAssertion,
    TsTypeParameterDeclaration, TsTypeParameterInstantiation, UnaryOperator, Visit, VisitWith,
};

use super::super::{ExpressionOrType, StatementItem};
use super::StatementGap;
use crate::formatter::Formatter;
use crate::formatter::suppression::ContentMode;

/// A bare shift count can leave a type parameter list open across several statements.
#[derive(Default)]
pub(in crate::formatter::statements) struct ShiftContinuation {
    open: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeParameterLookahead {
    Ends,
    Continues,
    ClosingAngle,
}

impl ShiftContinuation {
    pub(super) fn needs_terminator(
        &mut self,
        gap: &StatementGap<'_, '_>,
        formatter: &Formatter<'_, '_>,
        mode: ContentMode,
    ) -> bool {
        let Some(next) = gap.next else { return false };
        if !self.open
            && !matches!(gap.current.item.tail_syntax(formatter.ast),
                Some(ExpressionOrType::Expression(expr)) if formatter.ends_in_shift_parameter(expr, mode))
        {
            return false;
        }
        let lookahead = following_type_parameter(formatter.ast, next.item);
        self.open = lookahead == TypeParameterLookahead::Continues;
        lookahead == TypeParameterLookahead::ClosingAngle
    }
}

fn following_type_parameter(ast: &Ast<'_>, statement: StatementItem) -> TypeParameterLookahead {
    use TypeParameterLookahead::*;
    let StatementItem::Statement(statement) = statement else {
        return Ends;
    };
    match ast.stmt_data(statement) {
        StmtData::ExpressionStatement(node) => expression_lookahead(ast, node.expression(ast)),
        StmtData::Declaration(decl) => match ast.decl_data(decl) {
            DeclData::VariableDeclaration(decl) => {
                for id in decl.declarators(ast).iter() {
                    let binding = ast.get_node_in_sub_range(id);
                    if !matches!(
                        ast.binding_pattern_data(binding.id(ast)),
                        BindingPatternData::BindingIdentifier(_)
                    ) {
                        return Ends;
                    }
                    if let Some(init) = binding.init(ast) {
                        let result = expression_lookahead(ast, init);
                        if result != Continues {
                            return result;
                        }
                    }
                }
                Continues
            }
            DeclData::TsTypeAliasDeclaration(_) => Continues,
            _ => Ends,
        },
        _ => Ends,
    }
}

/// A simple name or default can extend the parameter list. A second
/// angle expression or a keyword operand can supply its closing token.
fn expression_lookahead(ast: &Ast<'_>, mut expr: Expr) -> TypeParameterLookahead {
    use TypeParameterLookahead::*;
    let mut tail = Continues;
    let mut may_have_angle = true;
    loop {
        expr = match ast.expr_data(expr) {
            ExprData::ParenthesizedExpression(node) => node.expression(ast),
            ExprData::AssignmentExpression(node)
                if node.operator(ast) == AssignmentOperator::Assign =>
            {
                node.right(ast)
            }
            ExprData::BinaryExpression(node) => {
                match node.operator(ast) {
                    BinaryOperator::LeftShift | BinaryOperator::LessThan => {
                        if may_have_angle && operand_has_angle(ast, node.right(ast)) {
                            return ClosingAngle;
                        }
                    }
                    // The same tokens can continue a union or intersection type.
                    BinaryOperator::BitwiseOr | BinaryOperator::BitwiseAnd => {}
                    _ => tail = Ends,
                }
                node.left(ast)
            }
            ExprData::UnaryExpression(node)
                if matches!(
                    node.operator(ast),
                    UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
                ) =>
            {
                if may_have_angle && operand_has_angle(ast, node.argument(ast)) {
                    return ClosingAngle;
                }
                // The search covered the whole operand, including nested unary expressions.
                may_have_angle = false;
                node.argument(ast)
            }
            ExprData::UnaryExpression(node) if node.operator(ast) == UnaryOperator::Negate => {
                node.argument(ast)
            }
            ExprData::MemberExpression(node) if !node.optional(ast) => node.object(ast),
            ExprData::TsNonNullExpression(node) => node.expression(ast),
            ExprData::TsAsExpression(node) => node.expression(ast),
            ExprData::TsSatisfiesExpression(node) => node.expression(ast),
            // Tuple and object literal types may also serve as defaults.
            ExprData::IdentifierReference(_)
            | ExprData::ArrayExpression(_)
            | ExprData::ObjectExpression(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::StringLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::TsInstantiationExpression(_) => return tail,
            _ => return Ends,
        }
    }
}

/// Inspect actual syntax tokens. Strings, comments, and the contents of a
/// nested function body cannot close the speculative type argument.
fn operand_has_angle(ast: &Ast<'_>, expr: Expr) -> bool {
    let mut search = ClosingAngleSearch {
        ast,
        pending: Vec::new(),
        found: false,
    };
    let mut node = ExpressionOrType::Expression(expr);
    loop {
        match node {
            ExpressionOrType::Expression(expr) => expr.visit_children_with(&mut search),
            ExpressionOrType::Type(ty) => ty.visit_children_with(&mut search),
        }
        if search.found {
            return true;
        }
        let Some(next) = search.pending.pop() else {
            return false;
        };
        node = next;
    }
}

struct ClosingAngleSearch<'ast, 'source> {
    ast: &'ast Ast<'source>,
    pending: Vec<ExpressionOrType>,
    found: bool,
}

impl<'source> Visit<'source> for ClosingAngleSearch<'_, 'source> {
    fn ast(&self) -> &Ast<'source> {
        self.ast
    }

    // Long expression chains are handled iteratively by the binary printer.
    fn visit_expr(&mut self, expr: Expr) {
        self.pending.push(ExpressionOrType::Expression(expr));
    }
    fn visit_ts_type(&mut self, ty: TsType) {
        self.pending.push(ExpressionOrType::Type(ty));
    }

    fn visit_ts_type_parameter_instantiation(&mut self, _: TsTypeParameterInstantiation) {
        self.found = true;
    }
    fn visit_ts_type_parameter_declaration(&mut self, _: TsTypeParameterDeclaration) {
        self.found = true;
    }
    fn visit_ts_type_assertion(&mut self, _: TsTypeAssertion) {
        self.found = true;
    }
    fn visit_jsx_element(&mut self, _: JsxElement) {
        self.found = true;
    }
    fn visit_jsx_fragment(&mut self, _: JsxFragment) {
        self.found = true;
    }
    fn visit_function_body(&mut self, _: FunctionBody) {}
    fn visit_class_body(&mut self, _: ClassBody) {}
}
