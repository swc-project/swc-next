use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, AstUtf8, BinaryExpression, BinaryOperator, BindingIdentifier,
    BindingPattern, BindingPatternData, BindingProperty, BlockStatement, CallExpression,
    ConditionalExpression, DeclData, Expr, ExprData, ForStatement, ForStatementInitData, Function,
    FunctionBody, FunctionType, GetSpan, IdentifierReference, IfStatement, LogicalExpression,
    LogicalOperator, NodeId, NodeKindData, ObjectProperty, Program, PropertyKey, PropertyKeyData,
    SimpleAssignmentTargetData, Span, StaticBlock, Stmt, StmtData, SubRange, SwitchCase,
    TaggedTemplateExpression, TraverseAction, TraverseMut, TraverseMutCtx, TraverseMutWith,
    UnaryExpression, UnaryOperator, VariableDeclaration, VariableDeclarator, VariableKind,
    WhileStatement,
};
use swc_next_ecma_semantic::{
    AnalyzeOptions, NameResolution, SymbolFlags, analyze, scope::ScopeKind,
};

use crate::{CompressOptions, mangle::Renames};

mod conditionals;
mod dead_code;
mod effects;
mod side_effects;
mod statements;
mod value;

use dead_code::{Branch, Hoisted};
use effects::may_have_side_effects;
use statements::StatementList;
use value::{Value, skip_parentheses};

#[derive(Default)]
struct Candidates {
    globals: FxHashMap<NodeId, Value<'static>>,
    typeofs: FxHashMap<NodeId, NodeId>,
    unused: FxHashSet<NodeId>,
    /// Reads that cannot throw and whose removal cannot make a declaration
    /// removable.
    pure_reads: FxHashSet<NodeId>,
    renames: Renames,
}

impl Candidates {
    fn build(ast: &Ast<'_>, options: &CompressOptions, mangle: bool) -> Self {
        let mut candidates = Self::default();
        if !options.global_defs
            && !options.comparisons
            && !options.side_effects
            && !options.typeofs
            && !options.unused
            && !mangle
        {
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
        if mangle {
            candidates.renames = Renames::build(ast, &semantic);
        }
        let mut local_vars = FxHashMap::default();
        // Pure symbols, each with the offset from which its reads are pure.
        let mut pure_symbols = FxHashMap::default();
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
            // A removed read could leave a local `var` without uses, which
            // only a later pass would remove.
            let flags = entry.symbol.flags;
            let removable_var = options.unused
                && in_function
                && flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR)
                && !flags.contains(SymbolFlags::PARAMETER);
            if options.side_effects
                && is_initialized(flags)
                && !removable_var
                && !is_global_var(scope.kind, declaration.flags)
                && let Some(start) = initialized_from(ast, flags, scope.node)
            {
                pure_symbols.insert(entry.id, start);
            }
        }
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            if let NameResolution::Resolved(symbol) = reference.resolution()
                && reference.flags.is_read_only()
                && pure_symbols
                    .get(&symbol)
                    .is_some_and(|start| ast.span(reference.node).start >= *start)
            {
                candidates.pure_reads.insert(reference.node);
            }
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

pub(super) fn compress(ast: &mut Ast<'_>, options: CompressOptions, mangle: bool) {
    let mut candidates = Candidates::build(ast, &options, mangle);
    let names = std::mem::take(&mut candidates.renames.names)
        .iter()
        .map(|name| ast.add_utf8(name))
        .collect();
    let program = ast.root_program();
    let mut compressor = Compressor {
        options,
        candidates,
        names,
        delete_depth: 0,
        primitive_bindings: FxHashSet::default(),
        pending_typeofs: Vec::new(),
        hoisted: Vec::new(),
        function_starts: Vec::new(),
        branches: Vec::new(),
        reference_positions: FxHashSet::default(),
        statement_list: StatementList::default(),
    };
    let _ = program.traverse_mut_with(&mut compressor, &mut TraverseMutCtx::new(ast, ()));
}

/// Compression and mangling hooks for one traversal.
///
/// Children are compressed and renamed before their parent's exit hook, so
/// exit rewrites see final children. Rewrites return `ReplaceSkip`;
/// `ReplaceContinue` would traverse the replacement again. Enter hooks decide
/// from original syntax, before any child is rewritten.
struct Compressor {
    options: CompressOptions,
    candidates: Candidates,
    /// Name of each mangling slot.
    names: Vec<AstUtf8>,
    /// Number of enclosing `delete` operators.
    delete_depth: u32,
    primitive_bindings: FxHashSet<NodeId>,
    pending_typeofs: Vec<TypeofRewrite>,
    /// Hoisted declarations in open functions. Each function's entries are
    /// dropped when it exits.
    hoisted: Vec<Hoisted>,
    /// Length of `hoisted` when each open function was entered.
    function_starts: Vec<usize>,
    /// Open statements whose branches may be removed.
    branches: Vec<Branch>,
    /// Conditional, logical, and sequence expressions used as a callee, a
    /// tag, or a `delete` or `typeof` operand, where a reference keeps its
    /// receiver.
    reference_positions: FxHashSet<NodeId>,
    statement_list: StatementList,
}

struct TypeofRewrite {
    binary: BinaryExpression,
    argument: Expr,
    binding: NodeId,
    string_on_left: bool,
    span: Span,
}

/// Whether every declaration of a symbol initializes it before code can read
/// it, so a read cannot throw. Lexical declarations have a temporal dead zone,
/// and imports can be read before their module runs.
fn is_initialized(flags: SymbolFlags) -> bool {
    flags.intersects(
        SymbolFlags::FUNCTION_SCOPED_VAR
            | SymbolFlags::FUNCTION
            | SymbolFlags::PARAMETER
            | SymbolFlags::CATCH_VAR
            | SymbolFlags::FUNCTION_EXPRESSION,
    ) && !flags.intersects(
        SymbolFlags::BLOCK_SCOPED_VAR
            | SymbolFlags::CONST_VAR
            | SymbolFlags::CLASS
            | SymbolFlags::CLASS_EXPRESSION
            | SymbolFlags::IMPORT
            | SymbolFlags::IMPORT_EQUALS,
    )
}

/// Whether a declaration is a `var` in the global scope of a script. Such a
/// binding is a property of the global object, which can be an accessor
/// defined before the script runs.
fn is_global_var(scope: ScopeKind, flags: SymbolFlags) -> bool {
    scope == ScopeKind::Global && flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR)
}

/// The offset from which reads of a binding declared in `scope` are
/// initialized. A parameter or a catch binding is uninitialized while its
/// list or pattern is evaluated, as in `function f(a = b, b) {}`.
fn initialized_from(ast: &Ast<'_>, flags: SymbolFlags, scope: NodeId) -> Option<u32> {
    if !flags.intersects(SymbolFlags::PARAMETER | SymbolFlags::CATCH_VAR) {
        return Some(0);
    }
    let pattern = match ast.kind_data(scope) {
        NodeKindData::Function(function) => function.params(ast).node_id(),
        NodeKindData::ArrowFunctionExpression(arrow) => arrow.params(ast).node_id(),
        NodeKindData::CatchClause(catch) => catch.param(ast)?.node_id(),
        _ => return None,
    };
    Some(ast.span(pattern).end)
}

/// The strict form of an equality operator.
fn strict_equality(operator: BinaryOperator) -> Option<BinaryOperator> {
    match operator {
        BinaryOperator::Equal | BinaryOperator::StrictEqual => Some(BinaryOperator::StrictEqual),
        BinaryOperator::NotEqual | BinaryOperator::StrictNotEqual => {
            Some(BinaryOperator::StrictNotEqual)
        }
        _ => None,
    }
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

/// The name of the identifier that a shorthand binding property binds.
fn binding_name(ast: &Ast<'_>, pattern: BindingPattern) -> Option<AstUtf8> {
    match ast.binding_pattern_data(pattern) {
        BindingPatternData::BindingIdentifier(binding) => Some(binding.name(ast)),
        BindingPatternData::SimpleAssignmentTarget(target) => {
            match ast.simple_assignment_target_data(target) {
                SimpleAssignmentTargetData::IdentifierReference(reference) => {
                    Some(reference.name(ast))
                }
                _ => None,
            }
        }
        BindingPatternData::AssignmentPattern(pattern) => binding_name(ast, pattern.left(ast)),
        _ => None,
    }
}

/// Whether a shorthand property's key still spells `name`.
fn key_is(ast: &Ast<'_>, key: PropertyKey, name: Option<AstUtf8>) -> bool {
    matches!(ast.property_key_data(key), PropertyKeyData::IdentifierName(key)
        if name.is_some_and(|name| ast.get_utf8(name) == ast.get_utf8(key.name(ast))))
}

impl Compressor {
    /// The mangled name of a binding or reference node.
    fn rename(&self, node: NodeId) -> Option<AstUtf8> {
        Some(self.names[self.candidates.renames.slot(node)?])
    }

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
        if strict_equality(binary.operator(ast)).is_none() {
            return;
        }
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

    /// Marks a conditional, logical, or sequence callee, tag, or operand,
    /// whose kept branch or last item must not become a bare reference.
    fn mark_reference_position(&mut self, ast: &Ast<'_>, expr: Expr) {
        let expr = skip_parentheses(ast, expr);
        let marked = match ast.expr_data(expr) {
            ExprData::ConditionalExpression(_) | ExprData::LogicalExpression(_) => {
                self.options.dead_code
            }
            ExprData::SequenceExpression(_) => self.options.side_effects,
            _ => false,
        };
        if marked {
            self.reference_positions.insert(expr.node_id());
        }
    }

    /// Returns the branch that replaces a conditional or logical expression.
    ///
    /// In a reference position, a reference becomes `(0, a.b)`, so the call
    /// receiver stays `undefined`, `delete` still sees a value, and `typeof`
    /// still throws for an undeclared name.
    fn kept_branch(&mut self, ast: &mut Ast<'_>, node: Expr, kept: Expr) -> Expr {
        if !self.reference_positions.remove(&node.node_id()) || !dead_code::is_reference(ast, kept)
        {
            return kept;
        }
        let span = node.span(ast);
        let zero = number(ast, span, 0.0);
        let expressions = ast.add_sub_range([zero, kept]);
        Expr::SequenceExpression(ast.sequence_expression(span, expressions))
    }

    fn conditional(
        &mut self,
        ast: &mut Ast<'_>,
        node: Expr,
        conditional: ConditionalExpression,
    ) -> Option<Expr> {
        let test = Value::of(ast, conditional.test(ast))?.to_boolean()?;
        let kept = if test {
            conditional.consequent(ast)
        } else {
            conditional.alternate(ast)
        };
        Some(self.kept_branch(ast, node, kept))
    }

    fn logical(
        &mut self,
        ast: &mut Ast<'_>,
        node: Expr,
        logical: LogicalExpression,
    ) -> Option<Expr> {
        let left = Value::of(ast, logical.left(ast))?;
        let keep_left = match logical.operator(ast) {
            LogicalOperator::And => !left.to_boolean()?,
            LogicalOperator::Or => left.to_boolean()?,
            LogicalOperator::NullishCoalescing => !matches!(left, Value::Undefined | Value::Null),
        };
        let kept = if keep_left {
            logical.left(ast)
        } else {
            logical.right(ast)
        };
        Some(self.kept_branch(ast, node, kept))
    }

    /// Records where an `if`, `while`, or `for` statement starts, and where
    /// the alternate of the innermost open `if` starts.
    fn enter_branch(&mut self, ast: &Ast<'_>, node: Stmt) {
        let start = self.hoisted.len();
        if let Some(branch) = self.branches.last_mut()
            && branch.alternate == Some(node.node_id())
        {
            branch.middle = Some(start);
        }
        let alternate = match ast.stmt_data(node) {
            StmtData::IfStatement(statement) => statement.alternate(ast).map(|s| s.node_id()),
            StmtData::WhileStatement(_) | StmtData::ForStatement(_) => None,
            _ => return,
        };
        self.branches.push(Branch {
            start,
            alternate,
            middle: None,
        });
    }

    /// Replaces an `if`, `while`, or `for` statement whose condition is
    /// constant with the statements that still run.
    fn exit_branch(&mut self, ast: &mut Ast<'_>, node: Stmt) -> Option<Stmt> {
        let data = ast.stmt_data(node);
        if !matches!(
            data,
            StmtData::IfStatement(_) | StmtData::WhileStatement(_) | StmtData::ForStatement(_)
        ) {
            return None;
        }
        // `enter_branch` pushed this statement's entry.
        let branch = self.branches.pop()?;
        let span = node.span(ast);
        match data {
            StmtData::IfStatement(statement) => self.remove_if(ast, span, statement, &branch),
            StmtData::WhileStatement(statement) => self.remove_while(ast, span, statement, &branch),
            StmtData::ForStatement(statement) => self.remove_for(ast, span, statement, &branch),
            _ => None,
        }
    }

    fn remove_if(
        &self,
        ast: &mut Ast<'_>,
        span: Span,
        statement: IfStatement,
        branch: &Branch,
    ) -> Option<Stmt> {
        let test = Value::of(ast, statement.test(ast))?.to_boolean()?;
        let end = self.hoisted.len();
        let (kept, removed) = if test {
            (Some(statement.consequent(ast)), branch.alternate(end))
        } else {
            (statement.alternate(ast), branch.consequent(end))
        };
        let hoisted = self.hoist(ast, span, removed, None, Vec::new())?;
        Some(dead_code::join(ast, span, hoisted, kept))
    }

    fn remove_while(
        &self,
        ast: &mut Ast<'_>,
        span: Span,
        statement: WhileStatement,
        branch: &Branch,
    ) -> Option<Stmt> {
        if Value::of(ast, statement.test(ast))?.to_boolean()? {
            return None;
        }
        let removed = branch.consequent(self.hoisted.len());
        let hoisted = self.hoist(ast, span, removed, None, Vec::new())?;
        Some(dead_code::join(ast, span, hoisted, None))
    }

    /// Replaces a `for` loop whose test is falsy with its initializer.
    fn remove_for(
        &self,
        ast: &mut Ast<'_>,
        span: Span,
        statement: ForStatement,
        branch: &Branch,
    ) -> Option<Stmt> {
        if Value::of(ast, statement.test(ast)?)?.to_boolean()? {
            return None;
        }
        let (init, declaration) = match statement
            .init(ast)
            .map(|init| ast.for_statement_init_data(init))
        {
            None => (None, None),
            Some(ForStatementInitData::Expr(init)) => {
                let init = may_have_side_effects(ast, init)
                    .then(|| ast.stmt_expression_statement(init.span(ast), init));
                (init, None)
            }
            Some(ForStatementInitData::VariableDeclaration(declaration))
                if declaration.kind(ast) == VariableKind::Var =>
            {
                (None, Some(declaration))
            }
            // A lexical initializer is scoped to the loop.
            Some(ForStatementInitData::VariableDeclaration(_)) => return None,
        };
        // A `var` initializer still runs; body names join its declaration.
        let prefix = declaration.map_or_else(Vec::new, |declaration| {
            declaration
                .declarators(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id))
                .collect()
        });
        let removed = branch.consequent(self.hoisted.len());
        let hoisted = self.hoist(ast, span, removed, declaration, prefix)?;
        Some(dead_code::join(ast, span, hoisted, init))
    }

    /// Builds a `var` statement for `prefix` and the used names that the
    /// removed `entries` declare. Returns `Some(None)` when nothing is
    /// declared, and `None` when the entries cannot be removed.
    fn hoist(
        &self,
        ast: &mut Ast<'_>,
        span: Span,
        entries: std::ops::Range<usize>,
        exclude: Option<VariableDeclaration>,
        mut prefix: Vec<VariableDeclarator>,
    ) -> Option<Option<Stmt>> {
        let bindings =
            dead_code::hoisted_bindings(ast, &self.hoisted[entries], exclude, |binding| {
                !self.candidates.unused.contains(&binding.node_id())
            })?;
        prefix.extend(dead_code::declarators(ast, &bindings));
        Some((!prefix.is_empty()).then(|| dead_code::var_statement(ast, span, prefix)))
    }

    fn compress_list(&mut self, ast: &mut Ast<'_>, list: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        self.statement_list.compress(ast, &self.options, list)
    }

    /// Compresses a block that `exit_block_statement` left to the node that
    /// holds it.
    ///
    /// An enclosing list merges the statements of a block without
    /// block-scoped declarations. If the block built its own sequences and
    /// declarator lists first, the enclosing list would copy them again, and
    /// nested blocks would copy them once for each level.
    fn compress_held_block(&mut self, ast: &mut Ast<'_>, block: BlockStatement) {
        if self.options.blocks
            && !statements::declares_lexical(ast, block)
            && let Some(body) = self.compress_list(ast, block.body(ast))
        {
            block.set_body(ast, body);
        }
    }

    /// Compresses a block body, and replaces it with its only statement.
    fn compress_body(&mut self, ast: &mut Ast<'_>, body: Stmt) -> Stmt {
        if let StmtData::BlockStatement(block) = ast.stmt_data(body) {
            self.compress_held_block(ast, block);
        }
        statements::unwrap_block(ast, body)
    }

    /// Compresses the block bodies of a statement, and replaces them with
    /// their only statement.
    fn compress_bodies(&mut self, ast: &mut Ast<'_>, node: Stmt) {
        match ast.stmt_data(node) {
            StmtData::IfStatement(statement) => {
                for body in [Some(statement.consequent(ast)), statement.alternate(ast)]
                    .into_iter()
                    .flatten()
                {
                    if let StmtData::BlockStatement(block) = ast.stmt_data(body) {
                        self.compress_held_block(ast, block);
                    }
                }
                statements::unwrap_if(ast, statement);
            }
            StmtData::WhileStatement(statement) => {
                let body = self.compress_body(ast, statement.body(ast));
                statement.set_body(ast, body);
            }
            // `do{a()}while(x)` is not longer than `do a();while(x)`, and
            // some engines misparse a `do` body without braces.
            StmtData::DoWhileStatement(statement) => {
                if let StmtData::BlockStatement(block) = ast.stmt_data(statement.body(ast)) {
                    self.compress_held_block(ast, block);
                }
            }
            StmtData::ForStatement(statement) => {
                let body = self.compress_body(ast, statement.body(ast));
                statement.set_body(ast, body);
            }
            StmtData::ForInStatement(statement) => {
                let body = self.compress_body(ast, statement.body(ast));
                statement.set_body(ast, body);
            }
            StmtData::ForOfStatement(statement) => {
                let body = self.compress_body(ast, statement.body(ast));
                statement.set_body(ast, body);
            }
            StmtData::LabeledStatement(statement) => {
                let body = self.compress_body(ast, statement.body(ast));
                statement.set_body(ast, body);
            }
            StmtData::WithStatement(statement) => {
                let body = self.compress_body(ast, statement.body(ast));
                statement.set_body(ast, body);
            }
            StmtData::TryStatement(statement) => {
                self.compress_held_block(ast, statement.block(ast));
                if let Some(handler) = statement.handler(ast) {
                    self.compress_held_block(ast, handler.body(ast));
                }
                if let Some(finalizer) = statement.finalizer(ast) {
                    self.compress_held_block(ast, finalizer);
                }
            }
            _ => {}
        }
    }

    fn enter_function_scope(&mut self) {
        if self.options.dead_code {
            self.function_starts.push(self.hoisted.len());
        }
    }

    fn exit_function_scope(&mut self) {
        if let Some(start) = self.function_starts.pop() {
            self.hoisted.truncate(start);
        }
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
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Program> {
        // Forward uses can precede their declarations. Patch recorded handles on
        // exit; this worklist never visits a node or walks its children again.
        for rewrite in std::mem::take(&mut self.pending_typeofs) {
            if !self.primitive_bindings.contains(&rewrite.binding) {
                continue;
            }
            // A later rewrite may have negated the comparison.
            let Some(operator) = strict_equality(rewrite.binary.operator(ctx.ast)) else {
                continue;
            };
            let undefined = undefined(ctx.ast, rewrite.span);
            let (left, right) = if rewrite.string_on_left {
                (undefined, rewrite.argument)
            } else {
                (rewrite.argument, undefined)
            };
            rewrite.binary.set_left(ctx.ast, left);
            rewrite.binary.set_right(ctx.ast, right);
            rewrite.binary.set_operator(ctx.ast, operator);
        }
        let ast = &mut *ctx.ast;
        if let Some(body) = self.compress_list(ast, node.body(ast)) {
            node.set_body(ast, body);
        }
        TraverseAction::Continue
    }

    fn enter_stmt(
        &mut self,
        node: Stmt,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Stmt> {
        let ast = &mut *ctx.ast;
        if self.options.dead_code {
            self.enter_branch(ast, node);
        }
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

    fn exit_stmt(
        &mut self,
        node: Stmt,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Stmt> {
        if self.options.dead_code
            && let Some(replacement) = self.exit_branch(ctx.ast, node)
        {
            return TraverseAction::ReplaceSkip(replacement);
        }
        if (self.options.side_effects || self.options.negate_iife)
            && let Some(replacement) = self.drop_statement_values(ctx.ast, node)
        {
            return TraverseAction::ReplaceSkip(replacement);
        }
        if self.options.blocks {
            self.compress_bodies(ctx.ast, node);
        }
        if self.options.conditionals
            && let Some(replacement) = self.compress_if(ctx.ast, node)
        {
            let replacement = if self.options.side_effects || self.options.negate_iife {
                self.drop_statement_values(ctx.ast, replacement)
                    .unwrap_or(replacement)
            } else {
                replacement
            };
            return TraverseAction::ReplaceSkip(replacement);
        }
        TraverseAction::Continue
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<BlockStatement> {
        // A block without block-scoped declarations is compressed by the
        // node that holds it; see `compress_held_block`.
        let ast = &mut *ctx.ast;
        if (!self.options.blocks || statements::declares_lexical(ast, node))
            && let Some(body) = self.compress_list(ast, node.body(ast))
        {
            node.set_body(ast, body);
        }
        TraverseAction::Continue
    }

    fn exit_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<FunctionBody> {
        let ast = &mut *ctx.ast;
        if self.options.if_return
            && let Some(body) = self.compress_function_tail(ast, node.body(ast))
        {
            node.set_body(ast, body);
        }
        if let Some(body) = self.compress_list(ast, node.body(ast)) {
            node.set_body(ast, body);
        }
        TraverseAction::Continue
    }

    fn exit_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<SwitchCase> {
        let ast = &mut *ctx.ast;
        let consequent = node.consequent(ast);
        if let Some(consequent) = self.compress_list(ast, consequent) {
            node.set_consequent(ast, consequent);
        }
        TraverseAction::Continue
    }

    fn enter_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<VariableDeclaration> {
        if self.options.dead_code && node.kind(ctx.ast) == VariableKind::Var {
            self.hoisted.push(Hoisted::Var(node));
        }
        TraverseAction::Continue
    }

    fn enter_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Function> {
        if self.options.dead_code
            && node.function_type(ctx.ast) == FunctionType::FunctionDeclaration
        {
            self.hoisted.push(Hoisted::Function);
        }
        self.enter_function_scope();
        TraverseAction::Continue
    }

    fn exit_function(
        &mut self,
        _node: Function,
        _ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<Function> {
        self.exit_function_scope();
        TraverseAction::Continue
    }

    fn enter_arrow_function_expression(
        &mut self,
        _node: ArrowFunctionExpression,
        _ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        self.enter_function_scope();
        TraverseAction::Continue
    }

    fn exit_arrow_function_expression(
        &mut self,
        _node: ArrowFunctionExpression,
        _ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        self.exit_function_scope();
        TraverseAction::Continue
    }

    fn enter_static_block(
        &mut self,
        _node: StaticBlock,
        _ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<StaticBlock> {
        self.enter_function_scope();
        TraverseAction::Continue
    }

    fn exit_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<StaticBlock> {
        let ast = &mut *ctx.ast;
        if let Some(body) = self.compress_list(ast, node.body(ast)) {
            node.set_body(ast, body);
        }
        self.exit_function_scope();
        TraverseAction::Continue
    }

    fn enter_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<CallExpression> {
        self.mark_reference_position(ctx.ast, node.callee(ctx.ast));
        TraverseAction::Continue
    }

    fn enter_tagged_template_expression(
        &mut self,
        node: TaggedTemplateExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<TaggedTemplateExpression> {
        self.mark_reference_position(ctx.ast, node.tag(ctx.ast));
        TraverseAction::Continue
    }

    fn enter_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<BindingIdentifier> {
        if let Some(name) = self.rename(node.node_id()) {
            node.set_name(ctx.ast, name);
        }
        TraverseAction::Continue
    }

    fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<IdentifierReference> {
        if let Some(name) = self.rename(node.node_id()) {
            node.set_name(ctx.ast, name);
        }
        TraverseAction::Continue
    }

    fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<ObjectProperty> {
        // The value may have been replaced or renamed.
        let ast = &mut *ctx.ast;
        if node.shorthand(ast) {
            let name = match ast.expr_data(node.value(ast)) {
                ExprData::IdentifierReference(reference) => Some(reference.name(ast)),
                _ => None,
            };
            if !key_is(ast, node.key(ast), name) {
                node.set_shorthand(ast, false);
            }
        }
        TraverseAction::Continue
    }

    fn exit_binding_property(
        &mut self,
        node: BindingProperty,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<BindingProperty> {
        // The bound identifier may have been renamed.
        let ast = &mut *ctx.ast;
        if node.shorthand(ast) && !key_is(ast, node.key(ast), binding_name(ast, node.value(ast))) {
            node.set_shorthand(ast, false);
        }
        TraverseAction::Continue
    }

    fn enter_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast>,
    ) -> TraverseAction<UnaryExpression> {
        match node.operator(ctx.ast) {
            UnaryOperator::Delete => {
                self.delete_depth += 1;
                self.mark_reference_position(ctx.ast, node.argument(ctx.ast));
            }
            UnaryOperator::Typeof => self.mark_reference_position(ctx.ast, node.argument(ctx.ast)),
            _ => {}
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
        if self.options.comparisons {
            self.loosen_equality(ctx.ast, node);
        }
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
            ExprData::SequenceExpression(_) if self.options.side_effects => {
                self.trim_sequence(ast, node)
            }
            ExprData::UnaryExpression(unary)
                if self.options.side_effects && unary.operator(ast) == UnaryOperator::Void =>
            {
                self.drop_void_operand(ast, node, unary).or_else(|| {
                    self.options
                        .evaluate
                        .then(|| self.fold(ast, node))
                        .flatten()
                })
            }
            ExprData::ConditionalExpression(conditional) => {
                let removed = if self.options.dead_code {
                    self.conditional(ast, node, conditional)
                } else {
                    None
                };
                removed.or_else(|| {
                    self.options
                        .conditionals
                        .then(|| self.compress_conditional(ast, node, conditional))
                        .flatten()
                })
            }
            ExprData::LogicalExpression(logical) if self.options.dead_code => {
                self.logical(ast, node, logical)
            }
            ExprData::UnaryExpression(unary)
                if self.options.comparisons
                    && unary.operator(ast) == UnaryOperator::LogicalNot
                    && let ExprData::BinaryExpression(binary) =
                        ast.expr_data(skip_parentheses(ast, unary.argument(ast)))
                    && let Some(operator) =
                        conditionals::negated_equality(binary.operator(ast)) =>
            {
                binary.set_operator(ast, operator);
                Some(unary.argument(ast))
            }
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
    use swc_next_ecma_ast::{Ast, Expr, ExprData, GetSpan, StmtData, UnaryOperator};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use crate::{CompressOptions, Minifier, MinifierOptions};

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
        let compress = CompressOptions {
            side_effects: false,
            ..CompressOptions::default()
        };
        let options = MinifierOptions {
            compress: Some(compress),
            ..MinifierOptions::default()
        };
        Minifier::minify(&mut ast, options);
        let ExprData::UnaryExpression(replacement) = ast.expr_data(expression(&ast)) else {
            panic!();
        };
        assert_eq!(replacement.span(&ast), span);
        assert_eq!(replacement.argument(&ast).span(&ast), span);
    }

    #[test]
    fn parenthesized_operands() {
        let allocator = Allocator::new();
        let options = Options {
            preserve_parens: true,
            ..Options::default()
        };
        let result = Parser::init(&allocator, "typeof (x);", options, NoTokenParserConfig).parse();
        let mut ast = result.ast;
        Minifier::minify(&mut ast, MinifierOptions::default());
        let ExprData::UnaryExpression(unary) = ast.expr_data(expression(&ast)) else {
            panic!();
        };
        assert_eq!(unary.operator(&ast), UnaryOperator::Typeof);
    }
}
