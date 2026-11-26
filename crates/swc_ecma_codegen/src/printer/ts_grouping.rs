//! Plan value comparison groups in TypeScript before emitting text.
//!
//! Scan each delimiter region backwards. A committing `>`, `>>`, or `>>>`
//! gives the number of openings needed before a trial can reach that close.
//! Group the first eligible comparison encountered. Its delimiters preserve the
//! comparison and reject earlier speculative lists that would cross it.

mod follower;
mod value_syntax;

use swc_next_ecma_ast::*;

use self::follower::{Risk, close_can_commit, comparison_risk};
use super::{Binaryish, BinaryishOperator, GenerateResult, Options, Printer, PrinterMode, Typed};
use crate::operator::needs_nullish_wrap;

pub(super) fn generate<'a>(ast: &'a Ast<'a>, options: Options) -> GenerateResult<'a> {
    let risk = comparison_risk(ast, &options);
    let scope = match risk {
        Risk::None => return Printer::<Typed>::init(ast, options).generate(),
        Risk::Single(close) => ComparisonScope::find(ast, close),
        Risk::Multiple => None,
    };
    let mut printer = Printer::<Planned>::init(ast, options);
    if let Some(scope) = scope {
        printer.mode = plan(&printer, Some(scope.statement));
        printer.mode.scope = Some(scope);
        printer.print_program(ast.root_program());
        if !printer.mode.missed_comparison_region() {
            return printer.finish();
        }

        // Discard the tentative output if an edited AST placed the close in
        // another statement. The complete plan uses the actual AST edges.
        let Printer { options, .. } = printer;
        printer = Printer::init(ast, options);
    }
    printer.mode = plan(&printer, None);
    printer.generate()
}

fn plan(printer: &Printer<'_, Planned>, statement: Option<Stmt>) -> Planned {
    let mut planner = Planner {
        printer,
        groups: Planned::default(),
        comments_ordered: printer
            .comments
            .windows(2)
            .all(|pair| pair[0].span.end <= pair[1].span.end),
    };
    if let Some(statement) = statement {
        statement.visit_with(&mut planner);
    } else {
        printer.ast.root_program().visit_with(&mut planner);
    }
    planner.groups
}

#[derive(Default)]
struct Planned {
    groups: Vec<u64>,
    scope: Option<ComparisonScope>,
}

struct ComparisonScope {
    statement: Stmt,
    close: NodeId,
    entered: bool,
    missed: bool,
}

impl ComparisonScope {
    fn find(ast: &Ast<'_>, close: BinaryExpression) -> Option<Self> {
        // Use spans to find a likely statement without walking its children.
        // Printing checks the actual ownership before returning the output.
        let target = close.span(ast);
        let statement = ast
            .root_program()
            .body(ast)
            .iter()
            .map(|slot| ast.get_node_in_sub_range(slot))
            .find(|statement| {
                let span = statement.span(ast);
                span.start <= target.start && target.end <= span.end
            })?;
        Some(Self {
            statement,
            close: close.node_id(),
            entered: false,
            missed: false,
        })
    }
}

impl Planned {
    fn group(&mut self, comparison: BinaryExpression, node_count: usize) {
        if self.groups.is_empty() {
            self.groups
                .resize(node_count.div_ceil(u64::BITS as usize), 0);
        }
        let index = comparison.node_id().index();
        self.groups[index / u64::BITS as usize] |= 1 << (index % u64::BITS as usize);
    }
}

impl PrinterMode for Planned {
    const TYPESCRIPT: bool = true;
    const PLANNED_COMPARISONS: bool = true;

    fn groups_comparison(&self, comparison: BinaryExpression) -> bool {
        let index = comparison.node_id().index();
        self.groups
            .get(index / u64::BITS as usize)
            .is_some_and(|word| word & (1 << (index % u64::BITS as usize)) != 0)
    }

    fn enter_top_level_statement(&mut self, statement: Stmt) {
        if let Some(scope) = &mut self.scope {
            scope.entered = statement.node_id() == scope.statement.node_id();
        }
    }

    fn observe_binary(&mut self, binary: Binaryish) {
        if let Some(scope) = &mut self.scope
            && !scope.entered
            && let Binaryish::Binary(binary) = binary
        {
            scope.missed |= binary.node_id() == scope.close;
        }
    }

    fn missed_comparison_region(&self) -> bool {
        self.scope.as_ref().is_some_and(|scope| scope.missed)
    }
}

#[derive(Clone, Copy)]
struct Position {
    level: Precedence,
    forbid_in: bool,
}

impl Position {
    fn at(level: Precedence) -> Self {
        Self {
            level,
            forbid_in: false,
        }
    }
}

struct Planner<'p, 'a> {
    printer: &'p Printer<'a, Planned>,
    groups: Planned,
    comments_ordered: bool,
}

impl<'a> Visit<'a> for Planner<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.printer.ast
    }

    fn visit_expr(&mut self, expr: Expr) {
        self.scan(expr, Position::at(Precedence::Lowest), 0);
    }

    fn visit_for_statement_init(&mut self, init: ForStatementInit) {
        let ast = self.printer.ast;
        match ast.for_statement_init_data(init) {
            ForStatementInitData::Expr(expr) => {
                self.scan(
                    expr,
                    Position {
                        level: Precedence::Lowest,
                        forbid_in: true,
                    },
                    0,
                );
            }
            ForStatementInitData::VariableDeclaration(declaration) => {
                for slot in declaration.declarators(ast).iter() {
                    let declarator = ast.get_node_in_sub_range(slot);
                    declarator.id(ast).visit_with(self);
                    declarator.type_annotation(ast).visit_with(self);
                    if let Some(initializer) = declarator.init(ast) {
                        self.scan(
                            initializer,
                            Position {
                                level: Precedence::Assignment,
                                forbid_in: true,
                            },
                            0,
                        );
                    }
                }
            }
        }
    }

    fn visit_arguments(&mut self, arguments: SubRange<Argument>) {
        let ast = self.printer.ast;
        let mut suffix = 0;
        for slot in arguments.iter().rev() {
            suffix = self.scan_argument(ast.get_node_in_sub_range(slot), suffix);
        }
    }

    fn visit_array_expression(&mut self, array: ArrayExpression) {
        let ast = self.printer.ast;
        let mut suffix = 0;
        for slot in array.elements(ast).iter().rev() {
            suffix = ast
                .get_node_in_sub_range(slot)
                .map_or(0, |argument| self.scan_argument(argument, suffix));
        }
    }

    fn visit_import_expression(&mut self, import: ImportExpression) {
        let ast = self.printer.ast;
        let position = Position::at(Precedence::Assignment);
        let suffix = import
            .options(ast)
            .map_or(0, |options| self.scan(options, position, 0));
        self.scan(import.source(ast), position, suffix);
    }
}

impl Planner<'_, '_> {
    fn scan_argument(&mut self, argument: Argument, suffix: u8) -> u8 {
        let ast = self.printer.ast;
        let position = Position::at(Precedence::Assignment);
        match ast.argument_data(argument) {
            ArgumentData::Expr(expr) => self.scan(expr, position, suffix),
            ArgumentData::SpreadElement(spread) => {
                self.scan(spread.argument(ast), position, suffix);
                0
            }
        }
    }

    #[inline]
    fn scan(&mut self, expr: Expr, position: Position, suffix: u8) -> u8 {
        let data = self.printer.ast.expr_data(expr);
        if is_type_atom(data) {
            return suffix;
        }
        self.scan_expression(expr, data, position, suffix)
    }

    // A zero suffix means no later close can commit in the current region.
    // A nonzero suffix counts the openings still needed to reach such a close.
    fn scan_expression(
        &mut self,
        mut expr: Expr,
        mut data: ExprData,
        mut position: Position,
        mut suffix: u8,
    ) -> u8 {
        let ast = self.printer.ast;
        // As in the printer, follow the left spine iteratively. Only the first
        // group encountered on the spine can return a suffix to the caller.
        let mut outer_suffix = None;
        loop {
            let binary = match data {
                ExprData::BinaryExpression(binary) => Binaryish::Binary(binary),
                ExprData::LogicalExpression(logical) => Binaryish::Logical(logical),
                _ => {
                    let inner = self.scan_non_binary(expr, data, position, suffix);
                    return outer_suffix.unwrap_or(inner);
                }
            };
            let operator = binary.operator(ast);
            let (mut left_level, mut right_level) = operator.operand_levels();
            let left = binary.left(ast);
            let right = binary.right(ast);
            if operator.needs_group(position.level, position.forbid_in) {
                outer_suffix.get_or_insert_with(|| {
                    if suffix != 0 && !self.printer.syntax_rejects_type(expr) {
                        suffix
                    } else {
                        0
                    }
                });
                suffix = 0;
                position.forbid_in = false;
            }
            if operator == BinaryishOperator::Logical(LogicalOperator::NullishCoalescing) {
                if needs_nullish_wrap(ast, left) {
                    left_level = Precedence::Prefix;
                }
                if needs_nullish_wrap(ast, right) {
                    right_level = Precedence::Prefix;
                }
            }
            suffix = self.scan(
                right,
                Position {
                    level: right_level,
                    ..position
                },
                suffix,
            );
            let left_data = ast.expr_data(left);
            match operator {
                BinaryishOperator::Binary(BinaryOperator::LessThan) => {
                    if suffix == 1 {
                        let Binaryish::Binary(comparison) = binary else {
                            unreachable!()
                        };
                        self.groups.group(comparison, ast.node_kinds().len());
                        outer_suffix.get_or_insert(0);
                        suffix = 0;
                    } else if suffix > 1 {
                        suffix -= 1;
                        if self.grouped_type_head(left_data, position.forbid_in) {
                            suffix = 0;
                        }
                    }
                }
                BinaryishOperator::Binary(
                    op @ (BinaryOperator::GreaterThan
                    | BinaryOperator::RightShift
                    | BinaryOperator::UnsignedRightShift),
                ) => {
                    let width = op.as_str().len() as u8;
                    if (suffix == 0 || width < suffix)
                        && let Binaryish::Binary(close) = binary
                        && close_can_commit(self.printer, close, right_level, self.comments_ordered)
                    {
                        suffix = width;
                    }
                }
                BinaryishOperator::Binary(
                    BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOr,
                ) => {}
                _ => suffix = 0,
            }
            (expr, data, position.level) = (left, left_data, left_level);
        }
    }

    fn scan_non_binary(
        &mut self,
        expr: Expr,
        data: ExprData,
        position: Position,
        suffix: u8,
    ) -> u8 {
        let ast = self.printer.ast;
        match data {
            ExprData::SequenceExpression(sequence) => {
                if position.level >= Precedence::Comma {
                    self.scan_sequence(sequence, false, 0);
                    0
                } else {
                    self.scan_sequence(sequence, position.forbid_in, suffix)
                }
            }
            ExprData::AssignmentExpression(assignment)
                if position.level <= Precedence::Assignment =>
            {
                self.scan(
                    assignment.right(ast),
                    Position {
                        level: Precedence::Assignment,
                        ..position
                    },
                    suffix,
                );
                assignment.left(ast).visit_with(self);
                0
            }
            ExprData::ConditionalExpression(conditional)
                if position.level <= Precedence::Conditional =>
            {
                self.scan(
                    conditional.alternate(ast),
                    Position {
                        level: Precedence::Yield,
                        ..position
                    },
                    suffix,
                );
                self.scan(
                    conditional.consequent(ast),
                    Position::at(Precedence::Yield),
                    0,
                );
                self.scan(
                    conditional.test(ast),
                    Position {
                        level: Precedence::Conditional.next(),
                        ..position
                    },
                    0,
                )
            }
            ExprData::ArrowFunctionExpression(arrow)
                if position.level <= Precedence::Assignment =>
            {
                let body = arrow.body(ast);
                let suffix = match ast.arrow_function_body_data(body) {
                    ArrowFunctionBodyData::Expr(value) => self.scan(
                        value,
                        Position {
                            level: Precedence::Assignment,
                            ..position
                        },
                        suffix,
                    ),
                    ArrowFunctionBodyData::FunctionBody(_) => {
                        body.visit_with(self);
                        suffix
                    }
                };
                arrow.params(ast).visit_with(self);
                arrow.type_parameters(ast).visit_with(self);
                arrow.return_type(ast).visit_with(self);
                if arrow.r#async(ast)
                    || arrow.return_type(ast).is_some()
                    || arrow.type_parameters(ast).is_none()
                        && self
                            .printer
                            .bare_arrow_parameter(arrow.params(ast))
                            .is_some()
                {
                    0
                } else {
                    suffix
                }
            }
            _ if is_type_atom(data) => suffix,
            _ => {
                let rejects = suffix != 0 && self.printer.syntax_rejects_type(expr);
                expr.visit_children_with(self);
                if rejects { 0 } else { suffix }
            }
        }
    }

    fn scan_sequence(
        &mut self,
        sequence: SequenceExpression,
        forbid_in: bool,
        mut suffix: u8,
    ) -> u8 {
        let ast = self.printer.ast;
        for slot in sequence.expressions(ast).iter().rev() {
            suffix = self.scan(
                ast.get_node_in_sub_range(slot),
                Position {
                    level: Precedence::Assignment,
                    forbid_in,
                },
                suffix,
            );
        }
        suffix
    }

    // A parenthesized head cannot take nested type arguments. The left operand
    // of `<` is printed at comparison precedence with an additional cast rule.
    fn grouped_type_head(&self, data: ExprData, forbid_in: bool) -> bool {
        match data {
            ExprData::ParenthesizedExpression(_)
            | ExprData::SequenceExpression(_)
            | ExprData::ArrowFunctionExpression(_)
            | ExprData::AssignmentExpression(_)
            | ExprData::YieldExpression(_)
            | ExprData::ConditionalExpression(_)
            | ExprData::TsAsExpression(_)
            | ExprData::TsSatisfiesExpression(_)
            | ExprData::LogicalExpression(_) => true,
            ExprData::BinaryExpression(binary) => {
                BinaryishOperator::Binary(binary.operator(self.printer.ast))
                    .needs_group(Precedence::Compare, forbid_in)
            }
            _ => false,
        }
    }
}

fn is_type_atom(data: ExprData) -> bool {
    matches!(
        data,
        ExprData::IdentifierReference(_)
            | ExprData::ThisExpression(_)
            | ExprData::Super(_)
            | ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
    )
}
