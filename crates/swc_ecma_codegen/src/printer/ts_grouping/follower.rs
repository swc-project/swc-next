//! Followers of closing angles in speculative TypeScript type arguments.

use swc_next_ecma_ast::{
    Ast, AstNode, BinaryExpression, BinaryOperator, Expr, ExprData, GetSpan, NodeId, NodeKind,
    Precedence, StringLiteral, UnaryOperator,
};

use super::super::{Options, Printer, PrinterMode};

pub(super) enum Risk {
    None,
    Single(BinaryExpression),
    Multiple,
}

/// Conservatively finds up to two runtime closes that can commit a type-argument trial.
pub(super) fn comparison_risk(ast: &Ast<'_>, options: &Options) -> Risk {
    if !binary_expressions(ast).any(|binary| binary.operator(ast) == BinaryOperator::LessThan) {
        return Risk::None;
    }
    // Explicit type lists balance their own closing angles. A value trial
    // therefore also needs a runtime close with a committing printed follower.
    // Keep the no-LT scan cheap even when a TS file contains many GT operators.
    let mut comments_ordered = None;
    let mut candidates = binary_expressions(ast).filter(|&binary| match binary.operator(ast) {
        BinaryOperator::GreaterThan
        | BinaryOperator::RightShift
        | BinaryOperator::UnsignedRightShift => {
            close_may_commit(ast, binary, options, &mut comments_ordered)
        }
        _ => false,
    });
    let Some(first) = candidates.next() else {
        return Risk::None;
    };
    if candidates.next().is_some() {
        Risk::Multiple
    } else {
        Risk::Single(first)
    }
}

#[derive(Clone, Copy)]
enum Follower {
    // These followers reject a type-argument trial even after a line break.
    Reject,
    // An expression-start token rejects the trial unless trivia adds a line break.
    Value(u32),
    CompactString(StringLiteral, u32),
    Unknown,
}

fn close_follower(
    ast: &Ast<'_>,
    mut expr: Expr,
    mut level: Precedence,
    options: &Options,
) -> Follower {
    let mut leading = 0;
    loop {
        leading = leading.max(expr.span(ast).start);
        match ast.expr_data(expr) {
            ExprData::IdentifierReference(identifier) => {
                let name = ast.get_utf8(identifier.name(ast));
                return if matches!(name, "" | "as" | "satisfies" | "in" | "instanceof") {
                    Follower::Unknown
                } else {
                    Follower::Value(leading)
                };
            }
            ExprData::ThisExpression(_)
            | ExprData::Super(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::ArrayExpression(_)
            | ExprData::ObjectExpression(_)
            | ExprData::Function(_)
            | ExprData::Class(_)
            | ExprData::PrivateIdentifier(_)
            | ExprData::MetaProperty(_) => return Follower::Value(leading),
            ExprData::StringLiteral(string) => {
                return if options.minify {
                    Follower::CompactString(string, leading)
                } else {
                    Follower::Value(leading)
                };
            }
            ExprData::UpdateExpression(update)
                if update.prefix(ast) && level <= Precedence::Prefix =>
            {
                return Follower::Value(leading);
            }
            ExprData::AwaitExpression(_) if level <= Precedence::Prefix => {
                return Follower::Value(leading);
            }
            ExprData::NewExpression(_) if level <= Precedence::New => {
                return Follower::Value(leading);
            }
            ExprData::NumericLiteral(_) | ExprData::BigIntLiteral(_)
                if level < Precedence::Prefix =>
            {
                return Follower::Value(leading);
            }
            ExprData::UnaryExpression(unary) if level <= Precedence::Prefix => {
                return match unary.operator(ast) {
                    UnaryOperator::Positive | UnaryOperator::Negate => Follower::Reject,
                    _ => Follower::Value(leading),
                };
            }
            ExprData::MemberExpression(member) => {
                let object = member.object(ast);
                if member.computed(ast)
                    && let ExprData::IdentifierReference(identifier) = ast.expr_data(object)
                    && ast.get_utf8(identifier.name(ast)) == "let"
                {
                    return Follower::Unknown;
                }
                expr = object;
                level = Precedence::Call;
            }
            ExprData::CallExpression(call) if level <= Precedence::Call => {
                if options.comments && call.pure(ast) {
                    return Follower::Unknown;
                }
                expr = call.callee(ast);
                level = Precedence::Call;
            }
            ExprData::ChainExpression(chain) if level < Precedence::Call => {
                expr = chain.expression(ast);
                level = Precedence::Lowest;
            }
            ExprData::BinaryExpression(binary) => {
                let operator = binary.operator(ast);
                let precedence = operator.precedence();
                if level > precedence || operator == BinaryOperator::Exponent {
                    return Follower::Unknown;
                }
                expr = binary.left(ast);
                level = precedence;
            }
            _ => return Follower::Unknown,
        }
    }
}

fn close_may_commit(
    ast: &Ast<'_>,
    binary: BinaryExpression,
    options: &Options,
    comments_ordered: &mut Option<bool>,
) -> bool {
    let mut left = binary.left(ast);
    while let ExprData::ParenthesizedExpression(group) = ast.expr_data(left) {
        left = group.expression(ast);
    }
    // The printer emits `as`/`satisfies` as value operators without a preceding
    // line break. They cannot extend a type argument. Any value expression
    // inside their type annotation is enclosed by its own physical delimiters.
    if matches!(
        ast.expr_data(left),
        ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_)
    ) {
        return false;
    }
    match close_follower(
        ast,
        binary.right(ast),
        binary.operator(ast).precedence().next(),
        options,
    ) {
        Follower::Reject => false,
        Follower::CompactString(..) | Follower::Unknown => true,
        Follower::Value(end) => {
            if !options.comments || ast.comments().is_empty() {
                return false;
            }
            // The binary's leading comments have already been emitted. Track
            // every leading-comment offset visited before the first RHS token,
            // rather than assuming transformed node spans enclose their children.
            let start = binary.span(ast).start;
            if end <= start {
                return false;
            }
            let comments = ast.comments();
            if !*comments_ordered.get_or_insert_with(|| {
                comments
                    .windows(2)
                    .all(|pair| pair[0].span.end <= pair[1].span.end)
            }) {
                return true;
            }
            let next = comments.partition_point(|comment| comment.span.end <= start);
            comments
                .get(next)
                .is_some_and(|comment| comment.span.end <= end)
        }
    }
}

/// Checks a runtime close at its printed precedence. The preflight only tests
/// for possible trivia and defers compact re-quoting to this check.
pub(super) fn close_can_commit<M: PrinterMode>(
    printer: &Printer<'_, M>,
    binary: BinaryExpression,
    right_level: Precedence,
    comments_ordered: bool,
) -> bool {
    let ast = printer.ast;
    let end = match close_follower(ast, binary.right(ast), right_level, &printer.options) {
        Follower::Reject => return false,
        Follower::Unknown => return true,
        Follower::Value(end) => end,
        Follower::CompactString(string, end) => {
            if printer.string_quote(ast.get_wtf8(string.value(ast)), true) == b'`' {
                return true;
            }
            end
        }
    };
    let start = binary.span(ast).start.max(binary.left(ast).span(ast).start);
    if end <= start || printer.comments.is_empty() {
        return false;
    }
    if !comments_ordered {
        return true;
    }
    let next = printer
        .comments
        .partition_point(|comment| comment.span.end <= start);
    printer.comments[next..]
        .iter()
        .take_while(|comment| comment.span.end <= end)
        .any(|comment| printer.comment_has_line_terminator(*comment))
}

fn binary_expressions<'a>(ast: &'a Ast<'_>) -> impl Iterator<Item = BinaryExpression> + 'a {
    const { assert!(std::mem::size_of::<NodeKind>() == 1) };
    let kinds = ast.node_kinds();
    // SAFETY: NodeKind has a one-byte representation and every initialized
    // discriminant is valid to read as a byte. The slice has the same lifetime.
    let bytes = unsafe { std::slice::from_raw_parts(kinds.as_ptr().cast(), kinds.len()) };
    memchr::memchr_iter(NodeKind::BinaryExpression as u8, bytes).map(|index| {
        let id = NodeId::from_raw_unchecked(index as u32);
        // SAFETY: The tag was checked in this same AST's initialized table.
        unsafe { BinaryExpression::from_node_id_unchecked(id) }
    })
}
