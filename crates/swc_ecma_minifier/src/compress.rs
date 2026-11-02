use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, BinaryExpression, BinaryOperator, BindingPatternData, DeclData, Expr, ExprData, GetSpan,
    NodeId, ObjectProperty, Program, Span, Stmt, StmtData, TraverseAction, TraverseMut,
    TraverseMutCtx, TraverseMutWith, UnaryExpression, UnaryOperator, VariableDeclarator,
    VariableKind,
};
use swc_next_ecma_semantic::{
    AnalyzeOptions, NameResolution, SymbolFlags, analyze, scope::ScopeKind,
};

use crate::CompressOptions;

mod effects;
mod value;

use effects::may_have_side_effects;
use value::Value;

#[derive(Default)]
struct Candidates {
    globals: FxHashMap<NodeId, Value<'static>>,
    typeofs: FxHashMap<NodeId, NodeId>,
    unused: FxHashSet<NodeId>,
}

impl Candidates {
    fn build(ast: &Ast<'_>, options: &CompressOptions) -> Self {
        let mut candidates = Self::default();
        if !options.global_defs && !options.typeofs && !options.unused {
            return candidates;
        }
        let semantic = analyze(ast, AnalyzeOptions::default()).semantic;
        // Dynamic lookup can observe declarations absent from the static use index.
        if semantic
            .iter_scopes()
            .any(|entry| entry.scope.kind == ScopeKind::With)
            || semantic
                .iter_references()
                .any(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"eval")
        {
            return candidates;
        }
        let mut local_vars = FxHashMap::default();
        for entry in semantic.iter_symbols() {
            let mut declarations = semantic.declarations(entry.id);
            let Some(declaration) = declarations.next() else {
                continue;
            };
            if declarations.next().is_some() {
                continue;
            }
            let scope = semantic.scope(declaration.scope);
            if options.typeofs
                && declaration.flags == SymbolFlags::FUNCTION_SCOPED_VAR
                && matches!(scope.kind, ScopeKind::Function | ScopeKind::FunctionBody)
                && scope.node != ast.root_program().node_id()
                && semantic
                    .uses(entry.id)
                    .iter()
                    .all(|id| !semantic.reference(*id).flags.is_write())
            {
                local_vars.insert(entry.id, declaration.node);
            }
            let mut ancestor = Some(declaration.scope);
            let mut in_function = false;
            while let Some(id) = ancestor {
                let scope = semantic.scope(id);
                if matches!(scope.kind, ScopeKind::Function | ScopeKind::FunctionBody) {
                    in_function = scope.node != ast.root_program().node_id();
                    break;
                }
                ancestor = scope.parent;
            }
            if options.unused && in_function && semantic.uses(entry.id).is_empty() {
                candidates.unused.insert(declaration.node);
            }
        }
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            match reference.resolution() {
                NameResolution::Unresolved
                    if options.global_defs && reference.flags.is_read_only() =>
                {
                    let global = match ast.get_utf8(reference.name).as_bytes() {
                        b"undefined" => Value::Undefined,
                        b"Infinity" => Value::Number(f64::INFINITY),
                        b"NaN" => Value::Number(f64::NAN),
                        _ => continue,
                    };
                    candidates.globals.insert(reference.node, global);
                }
                NameResolution::Resolved(symbol) if local_vars.contains_key(&symbol) => {
                    candidates
                        .typeofs
                        .insert(reference.node, local_vars[&symbol]);
                }
                _ => {}
            }
        }
        candidates
    }
}

pub(super) fn compress(ast: &mut Ast<'_>, options: CompressOptions) {
    let candidates = Candidates::build(ast, &options);
    let program = ast.root_program();
    let mut compressor = Compressor {
        options,
        candidates,
        delete_depth: 0,
        primitive_bindings: FxHashSet::default(),
        pending_typeofs: Vec::new(),
    };
    let _ = program.traverse_mut_with(&mut compressor, &mut TraverseMutCtx::new(ast, ()));
}

/// Compression hooks for one traversal.
///
/// Children are compressed before their parent's exit hook, so exit rewrites
/// see final children. Rewrites return `ReplaceSkip`; `ReplaceContinue` would
/// traverse the replacement again. Enter hooks decide from original syntax,
/// before any child is rewritten.
struct Compressor {
    options: CompressOptions,
    candidates: Candidates,
    /// Number of enclosing `delete` operators.
    delete_depth: u32,
    primitive_bindings: FxHashSet<NodeId>,
    pending_typeofs: Vec<TypeofRewrite>,
}

struct TypeofRewrite {
    binary: BinaryExpression,
    argument: Expr,
    binding: NodeId,
    string_on_left: bool,
    operator: BinaryOperator,
    span: Span,
}

/// Whether a declared binding starts with a known primitive value.
fn primitive_initializer(ast: &Ast<'_>, declarator: VariableDeclarator) -> bool {
    declarator
        .init(ast)
        .is_none_or(|expr| Value::of(ast, expr).is_some())
}

fn number(ast: &mut Ast<'_>, span: Span, value: f64) -> Expr {
    Expr::NumericLiteral(ast.numeric_literal(span, value, None))
}

fn undefined(ast: &mut Ast<'_>, span: Span) -> Expr {
    let zero = number(ast, span, 0.0);
    Expr::UnaryExpression(ast.unary_expression(span, zero, UnaryOperator::Void))
}

fn is_undefined_string(ast: &Ast<'_>, expr: Expr) -> bool {
    matches!(ast.expr_data(expr), ExprData::StringLiteral(literal)
        if ast.get_wtf8(literal.value(ast)).as_bytes() == b"undefined")
}

impl Compressor {
    fn removable(&self, ast: &Ast<'_>, declarator: VariableDeclarator) -> bool {
        matches!(ast.binding_pattern_data(declarator.id(ast)),
            BindingPatternData::BindingIdentifier(binding)
            if self.candidates.unused.contains(&binding.node_id()))
            && declarator
                .init(ast)
                .is_none_or(|expr| !may_have_side_effects(ast, expr))
    }

    fn typeof_argument(&self, ast: &Ast<'_>, expr: Expr) -> Option<Expr> {
        let ExprData::UnaryExpression(unary) = ast.expr_data(expr) else {
            return None;
        };
        let argument = unary.argument(ast);
        (unary.operator(ast) == UnaryOperator::Typeof
            && self.candidates.typeofs.contains_key(&argument.node_id()))
        .then_some(argument)
    }

    fn record_typeof(&mut self, ast: &Ast<'_>, binary: BinaryExpression) {
        let operator = match binary.operator(ast) {
            BinaryOperator::Equal | BinaryOperator::StrictEqual => BinaryOperator::StrictEqual,
            BinaryOperator::NotEqual | BinaryOperator::StrictNotEqual => {
                BinaryOperator::StrictNotEqual
            }
            _ => return,
        };
        let left = binary.left(ast);
        let right = binary.right(ast);
        let (argument, string, string_on_left) = if is_undefined_string(ast, right)
            && let Some(argument) = self.typeof_argument(ast, left)
        {
            (argument, right, false)
        } else if is_undefined_string(ast, left)
            && let Some(argument) = self.typeof_argument(ast, right)
        {
            (argument, left, true)
        } else {
            return;
        };
        self.pending_typeofs.push(TypeofRewrite {
            binary,
            argument,
            binding: self.candidates.typeofs[&argument.node_id()],
            string_on_left,
            operator,
            span: string.span(ast),
        });
    }

    fn global(&self, ast: &mut Ast<'_>, node: Expr) -> Option<Expr> {
        if self.delete_depth != 0 {
            return None;
        }
        let value = *self.candidates.globals.get(&node.node_id())?;
        let span = node.span(ast);
        self.value_expr(ast, span, value)
    }

    /// Folds a unary or binary expression whose operands are constant.
    ///
    /// Operands are compressed before this runs, so a constant operand is
    /// already in a form `Value::of` reads. The result replaces the
    /// expression only if it prints shorter, which also stops canonical
    /// forms such as `!0`, `-1`, and `void 0` from being rebuilt.
    fn fold(&self, ast: &mut Ast<'_>, node: Expr) -> Option<Expr> {
        let (value, cost) = match ast.expr_data(node) {
            ExprData::UnaryExpression(unary) => {
                let operator = unary.operator(ast);
                let argument = Value::of(ast, unary.argument(ast))?;
                // A non-finite argument prints as a parenthesized division.
                let parentheses = match argument {
                    Value::Number(value) if !value.is_finite() => 2,
                    _ => 0,
                };
                (
                    Value::unary(operator, argument)?,
                    unary_operator_cost(operator) + self.cost(argument) + parentheses,
                )
            }
            ExprData::BinaryExpression(binary) => {
                let operator = binary.operator(ast);
                let left = Value::of(ast, binary.left(ast))?;
                let right = Value::of(ast, binary.right(ast))?;
                (
                    Value::binary(ast, operator, left, right)?,
                    self.cost(left) + binary_operator_cost(operator) + self.cost(right),
                )
            }
            _ => return None,
        };
        if self.cost(value) >= cost {
            return None;
        }
        let span = node.span(ast);
        self.value_expr(ast, span, value)
    }

    /// Approximate printed length of the expression `value_expr` builds.
    fn cost(&self, value: Value<'_>) -> usize {
        match value {
            Value::Undefined => "void 0".len(),
            Value::Null => "null".len(),
            Value::Boolean(_) if self.options.booleans => "!0".len(),
            Value::Boolean(value) => if value { "true" } else { "false" }.len(),
            Value::Number(value) if value.is_nan() => "0/0".len(),
            Value::Number(value) if value.is_infinite() => usize::from(value < 0.0) + "1/0".len(),
            Value::Number(value) => {
                usize::from(value.is_sign_negative())
                    + zmij_ecma::Buffer::new().format_finite(value.abs()).len()
            }
            // Quotes; escapes are not counted.
            Value::String(value) => value.len() + 2,
            // BigInt values are never built. Comparisons that consume one,
            // such as `1n === 1`, still fold.
            Value::BigInt => usize::MAX / 4,
        }
    }

    /// Builds the shortest expression for a constant value. Returns `None`
    /// for BigInt, whose value is not tracked.
    fn value_expr(&self, ast: &mut Ast<'_>, span: Span, value: Value<'_>) -> Option<Expr> {
        Some(match value {
            Value::Undefined => undefined(ast, span),
            Value::Null => Expr::NullLiteral(ast.null_literal(span)),
            Value::Boolean(value) if self.options.booleans => {
                let argument = number(ast, span, if value { 0.0 } else { 1.0 });
                Expr::UnaryExpression(ast.unary_expression(
                    span,
                    argument,
                    UnaryOperator::LogicalNot,
                ))
            }
            Value::Boolean(value) => Expr::BooleanLiteral(ast.boolean_literal(span, value)),
            Value::Number(value) if value.is_finite() => {
                let literal = number(ast, span, value.abs());
                if !value.is_sign_negative() {
                    return Some(literal);
                }
                Expr::UnaryExpression(ast.unary_expression(span, literal, UnaryOperator::Negate))
            }
            Value::Number(value) => {
                let numerator = if value.is_nan() { 0.0 } else { 1.0 };
                let mut left = number(ast, span, numerator);
                if value == f64::NEG_INFINITY {
                    left = Expr::UnaryExpression(ast.unary_expression(
                        span,
                        left,
                        UnaryOperator::Negate,
                    ));
                }
                let right = number(ast, span, 0.0);
                Expr::BinaryExpression(ast.binary_expression(
                    span,
                    left,
                    right,
                    BinaryOperator::Divide,
                ))
            }
            Value::String(value) => {
                let value = ast.add_wtf8(value);
                let raw = ast.empty_utf8();
                Expr::StringLiteral(ast.string_literal(span, value, raw))
            }
            Value::BigInt => return None,
        })
    }
}

fn unary_operator_cost(operator: UnaryOperator) -> usize {
    match operator {
        UnaryOperator::Typeof => "typeof ".len(),
        UnaryOperator::Void => "void ".len(),
        UnaryOperator::Delete => "delete ".len(),
        UnaryOperator::Negate
        | UnaryOperator::Positive
        | UnaryOperator::LogicalNot
        | UnaryOperator::BitwiseNot => 1,
    }
}

fn binary_operator_cost(operator: BinaryOperator) -> usize {
    match operator {
        BinaryOperator::StrictEqual
        | BinaryOperator::StrictNotEqual
        | BinaryOperator::UnsignedRightShift => 3,
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThanOrEqual
        | BinaryOperator::Exponent
        | BinaryOperator::LeftShift
        | BinaryOperator::RightShift
        | BinaryOperator::In => 2,
        BinaryOperator::Instanceof => " instanceof ".len(),
        BinaryOperator::LessThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Modulo
        | BinaryOperator::BitwiseOr
        | BinaryOperator::BitwiseXor
        | BinaryOperator::BitwiseAnd => 1,
    }
}

impl<'ast> TraverseMut<'ast> for Compressor {
    fn exit_program(
        &mut self,
        _node: Program,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Program> {
        // Forward uses can precede their declarations. Patch recorded handles on
        // exit; this worklist never visits a node or walks its children again.
        for rewrite in std::mem::take(&mut self.pending_typeofs) {
            if !self.primitive_bindings.contains(&rewrite.binding) {
                continue;
            }
            let undefined = undefined(ctx.ast, rewrite.span);
            let (left, right) = if rewrite.string_on_left {
                (undefined, rewrite.argument)
            } else {
                (rewrite.argument, undefined)
            };
            rewrite.binary.set_left(ctx.ast, left);
            rewrite.binary.set_right(ctx.ast, right);
            rewrite.binary.set_operator(ctx.ast, rewrite.operator);
        }
        TraverseAction::Continue
    }

    fn enter_stmt(
        &mut self,
        node: Stmt,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Stmt> {
        let ast = &mut *ctx.ast;
        let StmtData::Declaration(declaration) = ast.stmt_data(node) else {
            return TraverseAction::Continue;
        };
        let DeclData::VariableDeclaration(declaration) = ast.decl_data(declaration) else {
            return TraverseAction::Continue;
        };
        if !matches!(
            declaration.kind(ast),
            VariableKind::Var | VariableKind::Let | VariableKind::Const
        ) {
            return TraverseAction::Continue;
        }
        // Initializers are inspected before their children are rewritten.
        // Loop headers are not statements: their implicit writes must not be
        // treated as an undefined initializer.
        let original = declaration.declarators(ast);
        for id in original.iter() {
            let declarator = ast.get_node_in_sub_range(id);
            if primitive_initializer(ast, declarator)
                && let BindingPatternData::BindingIdentifier(binding) =
                    ast.binding_pattern_data(declarator.id(ast))
            {
                self.primitive_bindings.insert(binding.node_id());
            }
        }

        let kept: Vec<_> = original
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .filter(|declarator| !self.removable(ast, *declarator))
            .collect();
        if kept.is_empty() {
            // Keep a barrier against turning a following string into a directive.
            let span = node.span(ast);
            return TraverseAction::ReplaceSkip(Stmt::EmptyStatement(ast.empty_statement(span)));
        }
        if kept.len() != original.len() {
            let declarators = ast.add_sub_range(kept);
            declaration.set_declarators(ast, declarators);
        }
        TraverseAction::Continue
    }

    fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<ObjectProperty> {
        // Compression never replaces a value with an identifier.
        if node.shorthand(ctx.ast)
            && !matches!(
                ctx.ast.expr_data(node.value(ctx.ast)),
                ExprData::IdentifierReference(_)
            )
        {
            node.set_shorthand(ctx.ast, false);
        }
        TraverseAction::Continue
    }

    fn enter_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<UnaryExpression> {
        if node.operator(ctx.ast) == UnaryOperator::Delete {
            self.delete_depth += 1;
        }
        TraverseAction::Continue
    }

    fn exit_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<UnaryExpression> {
        if node.operator(ctx.ast) == UnaryOperator::Delete {
            self.delete_depth -= 1;
        }
        TraverseAction::Continue
    }

    fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<BinaryExpression> {
        self.record_typeof(ctx.ast, node);
        TraverseAction::Continue
    }

    fn exit_expr(
        &mut self,
        node: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Expr> {
        let ast = &mut *ctx.ast;
        let replacement = match ast.expr_data(node) {
            ExprData::IdentifierReference(_) => self.global(ast, node),
            ExprData::UnaryExpression(_) | ExprData::BinaryExpression(_)
                if self.options.evaluate =>
            {
                self.fold(ast, node)
            }
            ExprData::BooleanLiteral(literal) if self.options.booleans => {
                let span = node.span(ast);
                let argument = number(ast, span, if literal.value(ast) { 0.0 } else { 1.0 });
                Some(Expr::UnaryExpression(ast.unary_expression(
                    span,
                    argument,
                    UnaryOperator::LogicalNot,
                )))
            }
            _ => None,
        };
        replacement.map_or(TraverseAction::Continue, TraverseAction::ReplaceSkip)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, Expr, ExprData, GetSpan, StmtData};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use crate::{Minifier, MinifierOptions};

    pub(super) fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
        let result =
            Parser::init(allocator, source, Options::default(), NoTokenParserConfig).parse();
        assert!(result.diagnostics.is_empty());
        result.ast
    }

    pub(super) fn expression(ast: &Ast<'_>) -> Expr {
        let statement =
            ast.get_node_in_sub_range(ast.root_program().body(ast).iter().next().unwrap());
        let StmtData::ExpressionStatement(statement) = ast.stmt_data(statement) else {
            panic!();
        };
        statement.expression(ast)
    }

    #[test]
    fn source_spans() {
        let allocator = Allocator::new();
        let mut ast = parse(&allocator, "  true;");
        let span = expression(&ast).span(&ast);
        Minifier::minify(&mut ast, MinifierOptions::default());
        let ExprData::UnaryExpression(replacement) = ast.expr_data(expression(&ast)) else {
            panic!();
        };
        assert_eq!(replacement.span(&ast), span);
        assert_eq!(replacement.argument(&ast).span(&ast), span);
    }
}
