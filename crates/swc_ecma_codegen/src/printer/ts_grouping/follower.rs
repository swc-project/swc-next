//! Followers of closing angles in speculative TypeScript type arguments.

use swc_next_ecma_ast::{
    Ast, AstNode, BinaryExpression, BinaryOperator, Expr, ExprData, GetSpan, NodeId, NodeKind,
    Precedence, UnaryOperator, is_ident_continue_ascii, is_identifier,
};

use crate::{printer::Options, util::next_code_point};

/// Conservatively checks whether the AST can emit a comparison and a committing close.
pub(super) fn needs_trace(ast: &Ast<'_>, options: &Options) -> bool {
    if !binary_expressions(ast).any(|binary| binary.operator(ast) == BinaryOperator::LessThan) {
        return false;
    }
    // Explicit type lists balance their own closing angles. A value trial
    // therefore also needs a runtime close with a committing printed follower.
    // Keep the no-LT scan cheap even when a TS file contains many GT operators.
    let mut comments_ordered = None;
    binary_expressions(ast).any(|binary| match binary.operator(ast) {
        BinaryOperator::GreaterThan
        | BinaryOperator::RightShift
        | BinaryOperator::UnsignedRightShift => {
            close_may_commit(ast, binary, options, &mut comments_ordered)
        }
        _ => false,
    })
}

/// Checks the first token and intervening line terminators in the printed output.
pub(super) fn printed_follower_can_commit(code: &[u8], mut index: usize) -> bool {
    let mut newline = false;
    loop {
        match code.get(index) {
            Some(b' ' | b'\t' | b'\x0b' | b'\x0c') => index += 1,
            Some(b'\n' | b'\r') => {
                newline = true;
                index += 1;
            }
            Some(0xe2)
                if code.get(index + 1) == Some(&0x80)
                    && matches!(code.get(index + 2), Some(0xa8 | 0xa9)) =>
            {
                newline = true;
                index += 3;
            }
            Some(b'/') if code.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index < code.len() && code.get(index..index + 2) != Some(b"*/") {
                    newline |= matches!(code[index], b'\n' | b'\r')
                        || matches!(code.get(index..index + 3), Some([0xe2, 0x80, 0xa8 | 0xa9]));
                    index += 1;
                }
                index += 2;
            }
            Some(b'/') if code.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < code.len()
                    && !matches!(code[index], b'\n' | b'\r')
                    && !matches!(code.get(index..index + 3), Some([0xe2, 0x80, 0xa8 | 0xa9]))
                {
                    index += 1;
                }
            }
            _ => break,
        }
    }
    can_follow_type_arguments(code.get(index..).unwrap_or_default(), newline)
}

fn can_follow_type_arguments(code: &[u8], newline: bool) -> bool {
    match code {
        [b'+' | b'-', b'=', ..] | [b'<', b'<' | b'=', ..] | [b'!', b'=', ..] => true,
        [b'+', b'+', ..] | [b'-', b'-', ..] => newline,
        // Match the parser's unconditional rejections before applying the line break.
        [b'<' | b'>' | b'+' | b'-', ..] => false,
        _ if newline => true,
        [b'a' | b'i' | b's', ..] => binary_keyword_follows(code),
        // The remaining emitted expression starts require a preceding line break.
        [b'.', b'0'..=b'9', ..] | [b'.', b'.', b'.', ..] => false,
        [
            b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'_'
            | b'$'
            | b'\\'
            | 0x80..=0xff
            | b'\''
            | b'"'
            | b'!'
            | b'~'
            | b'['
            | b'{'
            | b'#'
            | b'@',
            ..,
        ] => false,
        _ => true,
    }
}

fn binary_keyword_follows(code: &[u8]) -> bool {
    let end = if code.starts_with(b"as") {
        2
    } else if code.starts_with(b"satisfies") {
        9
    } else if code.starts_with(b"instanceof") {
        10
    } else if code.starts_with(b"in") {
        2
    } else {
        return false;
    };
    match code.get(end) {
        Some(b'\\') => false,
        Some(byte) if byte.is_ascii() => !is_ident_continue_ascii(*byte),
        Some(_) => {
            let (_, width) = next_code_point(code, end);
            // The final ASCII letter supplies a valid identifier start, so
            // the shared Unicode predicate tests just this continuation.
            !is_identifier(&code[end - 1..end + width])
        }
        None => true,
    }
}

#[derive(Clone, Copy)]
enum Follower {
    // These followers reject a type-argument trial even after a line break.
    Reject,
    // An expression-start token rejects the trial unless trivia adds a line break.
    Value(u32),
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
                return if matches!(name, "" | "as" | "satisfies") {
                    Follower::Unknown
                } else {
                    Follower::Value(leading)
                };
            }
            ExprData::ThisExpression(_)
            | ExprData::Super(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_) => return Follower::Value(leading),
            // Compact strings may be re-quoted as tagged-template followers.
            ExprData::StringLiteral(_) if !options.minify => return Follower::Value(leading),
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
        Follower::Unknown => true,
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
