//! Declarations that the traversal removes, and reads that it replaces with
//! constants, decided before the traversal.
//!
//! Removing a declaration removes the reads in its initializer or body,
//! which can leave other declarations without uses. Use counts that ignore
//! reads in removed code are iterated to a fixed point here, so the one
//! traversal removes the whole chain, and a repeated pass finds nothing more
//! to remove.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, AstWtf8, BindingPatternData, DeclData, Expr, ExprData, ForStatementInitData,
    ForStatementLeft, ForStatementLeftData, FunctionType, NodeId, NodeKind, NodeKindData, ScopeId,
    Span, StmtData, SymbolId, VariableDeclaration, VariableDeclarator, VariableKind,
};
use swc_next_ecma_semantic::{
    NameResolution, ReferenceId, Semantic, SymbolFlags, scope::ScopeKind,
};

use super::{effects::may_have_side_effects, value::Value};
use crate::CompressOptions;

/// A constant that replaces the reads of an inlined binding.
#[derive(Clone, Copy)]
pub(super) enum Constant {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(AstWtf8),
}

impl Constant {
    pub(super) fn value<'a>(self, ast: &Ast<'a>) -> Value<'a> {
        match self {
            Self::Undefined => Value::Undefined,
            Self::Null => Value::Null,
            Self::Boolean(value) => Value::Boolean(value),
            Self::Number(value) => Value::Number(value),
            Self::String(value) => Value::String(ast.get_wtf8(value)),
        }
    }
}

#[derive(Default)]
pub(super) struct Reduction {
    /// Bindings whose declarations the traversal removes.
    pub(super) removed: FxHashSet<NodeId>,
    /// Names of the removed function declarations.
    pub(super) functions: FxHashSet<NodeId>,
    /// Symbols of a kind whose declarations can be removed when they lose
    /// their uses. The kind does not depend on where the declaration is, so
    /// moving it does not change how a repeated pass treats its reads.
    pub(super) removable: FxHashSet<SymbolId>,
    /// The constant that replaces each read of an inlined binding.
    pub(super) inlined: FxHashMap<NodeId, Constant>,
    /// Removed bindings whose initializers are constants written with reads,
    /// such as `a + 1` for an inlined `a`. The traversal removes them before
    /// it folds the initializers.
    pub(super) constant_bindings: FxHashSet<NodeId>,
    removed_symbols: FxHashSet<SymbolId>,
    inlined_symbols: FxHashSet<SymbolId>,
    /// The code that removing each declaration removes.
    containers: FxHashMap<SymbolId, Span>,
    /// For containers that lost a use, the number of their uses outside
    /// removed code and outside the container itself.
    outside: FxHashMap<SymbolId, usize>,
    /// Whether each reference is in removed code, by index.
    removed_references: Vec<bool>,
    /// Number of references in removed code.
    removed_count: usize,
    /// All references, by start position, collected when code is first
    /// removed.
    references: Vec<(u32, ReferenceId)>,
    /// Removable symbols that are not containers yet, because their
    /// initializers are not constant yet.
    pending: Vec<SymbolId>,
    /// Bindings that may be inlined, in source order.
    inline_candidates: Vec<InlineCandidate>,
}

/// A binding declared once, with an initializer that runs before every
/// statement that follows it.
#[derive(Clone, Copy)]
struct InlineCandidate {
    span: Span,
    symbol: SymbolId,
    binding: NodeId,
    scope: ScopeId,
    init: Expr,
}

/// The declarations of the AST that removal and inlining inspect. The
/// arena is scanned once, without walking the tree.
#[derive(Default)]
struct Declarations {
    /// The declarator of each binding identifier, or `None` when more than
    /// one declarator holds it.
    declarators: FxHashMap<NodeId, Option<VariableDeclarator>>,
    /// The declaration that holds each declarator.
    holders: FxHashMap<NodeId, Option<VariableDeclaration>>,
    /// Declarations in loop heads, which are not statements.
    loop_heads: FxHashSet<NodeId>,
    /// `var` declarations directly in a function body, which run before
    /// any statement that follows them.
    top_level: FxHashSet<NodeId>,
    /// The span of each function declaration, by its name.
    functions: FxHashMap<NodeId, Option<Span>>,
    /// Names of function declarations in blocks. Outside strict code, such
    /// a function also binds its name in the enclosing function unless a
    /// `let` or `const` of the name in an enclosing block prevents it.
    block_functions: FxHashSet<Vec<u8>>,
}

/// Records `value` for `key`, or `None` when another value was recorded.
fn record<K: std::hash::Hash + Eq, V: Copy + PartialEq>(
    map: &mut FxHashMap<K, Option<V>>,
    key: K,
    value: V,
) {
    map.entry(key)
        .and_modify(|entry| {
            if *entry != Some(value) {
                *entry = None;
            }
        })
        .or_insert(Some(value));
}

impl Declarations {
    fn scan(ast: &Ast<'_>) -> Self {
        let mut declarations = Self::default();
        for index in 0..ast.node_count() {
            let node = NodeId::from_usize(index);
            match ast.kind_data(node) {
                NodeKindData::VariableDeclaration(declaration) => {
                    for id in declaration.declarators(ast).iter() {
                        let declarator = ast.get_node_in_sub_range(id);
                        record(&mut declarations.holders, declarator.node_id(), declaration);
                        if let BindingPatternData::BindingIdentifier(binding) =
                            ast.binding_pattern_data(declarator.id(ast))
                        {
                            record(&mut declarations.declarators, binding.node_id(), declarator);
                        }
                    }
                }
                NodeKindData::ForStatement(statement) => {
                    if let Some(init) = statement.init(ast)
                        && let ForStatementInitData::VariableDeclaration(declaration) =
                            ast.for_statement_init_data(init)
                    {
                        declarations.loop_heads.insert(declaration.node_id());
                    }
                }
                NodeKindData::ForInStatement(statement) => {
                    declarations.loop_head(ast, statement.left(ast));
                }
                NodeKindData::ForOfStatement(statement) => {
                    declarations.loop_head(ast, statement.left(ast));
                }
                NodeKindData::FunctionBody(body) => {
                    for id in body.body(ast).iter() {
                        if let StmtData::Declaration(declaration) =
                            ast.stmt_data(ast.get_node_in_sub_range(id))
                            && let DeclData::VariableDeclaration(declaration) =
                                ast.decl_data(declaration)
                            && declaration.kind(ast) == VariableKind::Var
                        {
                            declarations.top_level.insert(declaration.node_id());
                        }
                    }
                }
                NodeKindData::Function(function)
                    if function.function_type(ast) == FunctionType::FunctionDeclaration =>
                {
                    if let Some(binding) = function.id(ast) {
                        record(
                            &mut declarations.functions,
                            binding.node_id(),
                            ast.span(node),
                        );
                    }
                }
                _ => {}
            }
        }
        declarations
    }

    fn loop_head(&mut self, ast: &Ast<'_>, left: ForStatementLeft) {
        if let ForStatementLeftData::VariableDeclaration(declaration) =
            ast.for_statement_left_data(left)
        {
            self.loop_heads.insert(declaration.node_id());
        }
    }

    /// Whether a `let` or `const` named `name` can keep a function in a
    /// nested block from binding its name in the enclosing function.
    fn blocks_function(&self, ast: &Ast<'_>, name: AstWtf8) -> bool {
        self.block_functions.contains(ast.get_wtf8(name).as_bytes())
    }

    /// The declarator of a binding and the declaration statement that holds
    /// it. Declarators in loop heads are excluded.
    fn statement(&self, binding: NodeId) -> Option<(VariableDeclarator, VariableDeclaration)> {
        let declarator = (*self.declarators.get(&binding)?)?;
        let declaration = (*self.holders.get(&declarator.node_id())?)?;
        (!self.loop_heads.contains(&declaration.node_id())).then_some((declarator, declaration))
    }
}

pub(super) struct Context<'s, 'a> {
    pub(super) semantic: &'s Semantic<'a>,
    pub(super) options: &'s CompressOptions,
    /// Whether each symbol is declared inside a function.
    pub(super) in_function: &'s FxHashSet<SymbolId>,
    /// Constants of global reads.
    pub(super) globals: &'s FxHashMap<NodeId, Value<'static>>,
    /// The printed length of each binding's name.
    pub(super) name_length: &'s dyn Fn(&Ast<'_>, NodeId) -> usize,
}

impl Reduction {
    pub(super) fn build(ast: &mut Ast<'_>, context: &Context<'_, '_>) -> Self {
        let mut reduction = Self::default();
        let options = context.options;
        if !options.unused && !options.reduce_vars {
            return reduction;
        }
        let mut declarations = Declarations::scan(ast);
        for entry in context.semantic.iter_symbols() {
            let flags = entry.symbol.flags;
            let in_block = || {
                context.semantic.declarations(entry.id).any(|declaration| {
                    !matches!(
                        context.semantic.scope(declaration.scope).kind,
                        ScopeKind::Function
                            | ScopeKind::FunctionBody
                            | ScopeKind::Global
                            | ScopeKind::Module
                    )
                })
            };
            if flags.contains(SymbolFlags::FUNCTION)
                && (flags.contains(SymbolFlags::ANNEX_B_IF_FUNCTION) || in_block())
            {
                let name = ast.get_wtf8(entry.symbol.name).as_bytes().to_vec();
                declarations.block_functions.insert(name);
            }
        }
        if options.unused {
            for entry in context.semantic.iter_symbols() {
                if is_removable_kind(ast, context, &declarations, entry.id) {
                    reduction.removable.insert(entry.id);
                }
            }
            reduction.pending = reduction.removable.iter().copied().collect();
        }
        if options.reduce_vars {
            reduction.collect_inline_candidates(ast, context, &declarations);
        }
        // Removing code can remove the writes that keep a binding from being
        // inlined, and inlining can make an initializer constant. Within one
        // round, an initializer reads only bindings inlined before it.
        let mut removed_references = usize::MAX;
        loop {
            if options.unused {
                reduction.remove_unused(ast, context, &declarations);
            }
            if !options.reduce_vars || reduction.removed_count == removed_references {
                break;
            }
            removed_references = reduction.removed_count;
            if !reduction.inline(ast, context) || !options.unused {
                break;
            }
        }
        let semantic = context.semantic;
        for &symbol in &reduction.removed_symbols {
            let declaration = semantic.declarations(symbol).next().unwrap();
            reduction.removed.insert(declaration.node);
            if let Some((_, true)) = reduction.container(ast, context, &declarations, symbol) {
                reduction.constant_bindings.insert(declaration.node);
            }
            if semantic
                .symbol(symbol)
                .flags
                .contains(SymbolFlags::FUNCTION)
            {
                reduction.functions.insert(declaration.node);
            }
        }
        reduction
    }

    fn is_removed(&self, id: ReferenceId) -> bool {
        self.removed_references
            .get(id.index())
            .copied()
            .unwrap_or(false)
    }

    /// The uses of `symbol` outside removed code.
    fn live_uses<'s>(
        &'s self,
        semantic: &'s Semantic<'_>,
        symbol: SymbolId,
    ) -> impl Iterator<Item = ReferenceId> + 's {
        semantic
            .uses(symbol)
            .iter()
            .copied()
            .filter(move |id| !self.is_removed(*id))
    }

    /// Removes declarations without uses, and declarations whose uses are
    /// all in removed code or in the declaration itself, iterated to a fixed
    /// point. Each reference in removed code is visited once.
    fn remove_unused(
        &mut self,
        ast: &Ast<'_>,
        context: &Context<'_, '_>,
        declarations: &Declarations,
    ) {
        let semantic = context.semantic;
        let mut worklist = Vec::new();
        let mut pending = std::mem::take(&mut self.pending);
        pending.retain(|&symbol| {
            let Some((span, _)) = self.container(ast, context, declarations, symbol) else {
                return true;
            };
            self.containers.insert(symbol, span);
            // Uses in the declaration itself, such as recursive calls, are
            // removed with it.
            if self
                .live_uses(semantic, symbol)
                .all(|id| contains(span, ast.span(semantic.reference(id).node)))
            {
                worklist.push(symbol);
            }
            false
        });
        self.pending = pending;
        if !worklist.is_empty() && self.references.is_empty() {
            self.references = semantic
                .iter_references()
                .map(|entry| (ast.span(entry.reference.node).start, entry.id))
                .collect();
            // References are listed in source order.
            if !self.references.is_sorted_by_key(|entry| entry.0) {
                self.references.sort_unstable_by_key(|entry| entry.0);
            }
            let count = self
                .references
                .iter()
                .map(|(_, id)| id.index() + 1)
                .max()
                .unwrap_or(0);
            self.removed_references = vec![false; count];
        }
        while let Some(symbol) = worklist.pop() {
            if !self.removed_symbols.insert(symbol) {
                continue;
            }
            let span = self.containers[&symbol];
            let first = self
                .references
                .partition_point(|(start, _)| *start < span.start);
            for &(start, id) in &self.references[first..] {
                if start >= span.end {
                    break;
                }
                if self.removed_references[id.index()] {
                    continue;
                }
                self.removed_references[id.index()] = true;
                self.removed_count += 1;
                let reference = semantic.reference(id);
                let Some(target) = reference.symbol else {
                    continue;
                };
                if self.removed_symbols.contains(&target) {
                    continue;
                }
                let Some(&container) = self.containers.get(&target) else {
                    continue;
                };
                if contains(container, ast.span(reference.node)) {
                    continue;
                }
                // The count is taken when a use is first removed, after
                // this one.
                let outside = self.outside.entry(target).or_insert_with(|| {
                    semantic
                        .uses(target)
                        .iter()
                        .filter(|id| {
                            !self.removed_references[id.index()]
                                && !contains(container, ast.span(semantic.reference(**id).node))
                        })
                        .count()
                        + 1
                });
                *outside -= 1;
                if *outside == 0 {
                    worklist.push(target);
                }
            }
        }
    }

    /// The span of the code that removing an unused declaration of `symbol`
    /// removes, and whether the initializer is a constant written with
    /// reads, or `None` when the traversal keeps the declaration.
    ///
    /// A local function declaration is removed, and so is a declarator of a
    /// single name in a declaration statement whose initializer has no side
    /// effects.
    fn container(
        &self,
        ast: &Ast<'_>,
        context: &Context<'_, '_>,
        declarations: &Declarations,
        symbol: SymbolId,
    ) -> Option<(Span, bool)> {
        let declaration = context.semantic.declarations(symbol).next()?;
        if let Some(span) = declarations.functions.get(&declaration.node) {
            return Some(((*span)?, false));
        }
        let (declarator, _) = declarations.statement(declaration.node)?;
        let constant = match declarator.init(ast) {
            Some(init) if may_have_side_effects(ast, init) => {
                self.constant(ast, context, init)?;
                true
            }
            _ => false,
        };
        Some((ast.span(declarator.node_id()), constant))
    }

    /// Collects the bindings declared once by a declarator that runs before
    /// every statement that follows it.
    fn collect_inline_candidates(
        &mut self,
        ast: &Ast<'_>,
        context: &Context<'_, '_>,
        declarations: &Declarations,
    ) {
        let semantic = context.semantic;
        for entry in semantic.iter_symbols() {
            // `arguments` can also name the arguments object, and a value
            // in place of `__proto__` in `{ __proto__ }` sets the prototype.
            if !context.in_function.contains(&entry.id)
                || semantic.uses(entry.id).is_empty()
                || matches!(
                    ast.get_wtf8(entry.symbol.name).as_bytes(),
                    b"arguments" | b"__proto__"
                )
            {
                continue;
            }
            let mut iter = semantic.declarations(entry.id);
            let (Some(declaration), None) = (iter.next(), iter.next()) else {
                continue;
            };
            let Some((declarator, holder)) = declarations.statement(declaration.node) else {
                continue;
            };
            let Some(init) = declarator.init(ast) else {
                continue;
            };
            let dominates = match holder.kind(ast) {
                VariableKind::Var => declarations.top_level.contains(&holder.node_id()),
                // A `case` can jump past the declaration.
                VariableKind::Let | VariableKind::Const => {
                    let scope = semantic.scope(declaration.scope);
                    ast.node_kind(scope.node) != NodeKind::SwitchStatement
                        && !declarations.blocks_function(ast, entry.symbol.name)
                }
                _ => false,
            };
            if dominates {
                self.inline_candidates.push(InlineCandidate {
                    span: ast.span(declarator.node_id()),
                    symbol: entry.id,
                    binding: declaration.node,
                    scope: declaration.scope,
                    init,
                });
            }
        }
        // An initializer can read a binding inlined before it.
        self.inline_candidates
            .sort_unstable_by_key(|candidate| candidate.span.start);
    }

    /// Inlines bindings that hold a constant for every read outside removed
    /// code. Returns whether a binding was inlined.
    fn inline(&mut self, ast: &mut Ast<'_>, context: &Context<'_, '_>) -> bool {
        let semantic = context.semantic;
        let mut changed = false;
        let candidates = std::mem::take(&mut self.inline_candidates);
        for candidate in &candidates {
            let InlineCandidate {
                span,
                symbol,
                binding,
                scope,
                init,
            } = *candidate;
            if self.removed_symbols.contains(&symbol) || self.inlined_symbols.contains(&symbol) {
                continue;
            }
            // Cheap checks first: most initializers are not constant.
            let Some(value) = self.constant(ast, context, init) else {
                continue;
            };
            let Some(cost) = cost(value, context.options.booleans) else {
                continue;
            };
            let uses: Vec<_> = self.live_uses(semantic, symbol).collect();
            let name = (context.name_length)(ast, binding);
            // The declarator `a=value,` is removed.
            if uses.is_empty()
                || uses.len() * cost > uses.len() * name + name + cost + 2
                || !reads_follow(ast, semantic, &uses, symbol, scope, span)
            {
                continue;
            }
            let constant = match value {
                Value::Undefined => Constant::Undefined,
                Value::Null => Constant::Null,
                Value::Boolean(value) => Constant::Boolean(value),
                Value::Number(value) => Constant::Number(value),
                Value::String(value) => Constant::String(ast.add_wtf8(value)),
                Value::BigInt => continue,
            };
            self.removed.insert(binding);
            self.constant_bindings.insert(binding);
            self.inlined_symbols.insert(symbol);
            changed = true;
            for id in uses {
                self.inlined.insert(semantic.reference(id).node, constant);
            }
        }
        self.inline_candidates = candidates;
        changed
    }

    /// The value of an initializer, as the traversal would compute it.
    fn constant<'a>(
        &self,
        ast: &Ast<'a>,
        context: &Context<'_, '_>,
        expr: Expr,
    ) -> Option<Value<'a>> {
        let options = context.options;
        match ast.expr_data(expr) {
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.constant(ast, context, parenthesized.expression(ast))
            }
            ExprData::IdentifierReference(_) => {
                if let Some(constant) = self.inlined.get(&expr.node_id()) {
                    return Some(constant.value(ast));
                }
                if options.global_defs {
                    return context.globals.get(&expr.node_id()).copied();
                }
                None
            }
            ExprData::UnaryExpression(unary) if options.evaluate => {
                let argument = self.constant(ast, context, unary.argument(ast))?;
                Value::unary(unary.operator(ast), argument)
            }
            ExprData::BinaryExpression(binary) if options.evaluate => {
                let left = self.constant(ast, context, binary.left(ast))?;
                let right = self.constant(ast, context, binary.right(ast))?;
                Value::binary(ast, binary.operator(ast), left, right)
            }
            ExprData::ConditionalExpression(conditional) if options.dead_code => {
                let test = self.constant(ast, context, conditional.test(ast))?;
                let kept = if test.to_boolean()? {
                    conditional.consequent(ast)
                } else {
                    conditional.alternate(ast)
                };
                self.constant(ast, context, kept)
            }
            ExprData::SequenceExpression(sequence) if options.side_effects => {
                let mut items = sequence.expressions(ast).iter();
                let last = ast.get_node_in_sub_range(items.next_back()?);
                if items.any(|id| may_have_side_effects(ast, ast.get_node_in_sub_range(id))) {
                    return None;
                }
                self.constant(ast, context, last)
            }
            _ => Value::of(ast, expr),
        }
    }
}

/// Whether every use of `symbol` is a read that runs after the
/// declarator at `span`: it follows the declarator in the source, and it
/// is not in a function declaration hoisted above it.
fn reads_follow(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    uses: &[ReferenceId],
    symbol: SymbolId,
    declaration_scope: ScopeId,
    span: Span,
) -> bool {
    uses.iter().all(|id| {
        let reference = semantic.reference(*id);
        if reference.resolution() != NameResolution::Resolved(symbol)
            || !reference.flags.is_read_only()
            || ast.span(reference.node).start < span.end
        {
            return false;
        }
        let mut current = reference.scope;
        while current != declaration_scope {
            let scope = semantic.scope(current);
            if scope.parent == Some(declaration_scope)
                && let NodeKindData::Function(function) = ast.kind_data(scope.node)
                && function.function_type(ast) == FunctionType::FunctionDeclaration
            {
                return false;
            }
            let Some(parent) = scope.parent else {
                return false;
            };
            current = parent;
        }
        true
    })
}

/// Whether `symbol` is a local function declaration, or a local variable
/// declared once with a single name.
fn is_removable_kind(
    ast: &Ast<'_>,
    context: &Context<'_, '_>,
    declarations: &Declarations,
    symbol: SymbolId,
) -> bool {
    let semantic = context.semantic;
    if !context.in_function.contains(&symbol) {
        return false;
    }
    let mut iter = semantic.declarations(symbol);
    let (Some(declaration), None) = (iter.next(), iter.next()) else {
        return false;
    };
    if semantic
        .symbol(symbol)
        .flags
        .contains(SymbolFlags::FUNCTION)
    {
        let scope = semantic.scope(declaration.scope);
        // A function in a block can also bind a name in the function scope.
        // The name of a function that is the body of an `if` is declared in
        // the function's own scope.
        return scope.kind == ScopeKind::Function
            && declarations
                .functions
                .get(&declaration.node)
                .copied()
                .flatten()
                .is_some_and(|span| ast.span(scope.node) != span);
    }
    declarations
        .declarators
        .get(&declaration.node)
        .copied()
        .flatten()
        .and_then(|declarator| *declarations.holders.get(&declarator.node_id())?)
        .is_some_and(|holder| match holder.kind(ast) {
            VariableKind::Var => true,
            VariableKind::Let | VariableKind::Const => {
                !declarations.blocks_function(ast, semantic.symbol(symbol).name)
            }
            _ => false,
        })
}

/// Whether `inner` is within `outer`.
fn contains(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

/// Approximate printed length of a constant, or `None` for a BigInt.
pub(super) fn cost(value: Value<'_>, booleans: bool) -> Option<usize> {
    Some(match value {
        Value::Undefined => "void 0".len(),
        Value::Null => "null".len(),
        Value::Boolean(_) if booleans => "!0".len(),
        Value::Boolean(value) => if value { "true" } else { "false" }.len(),
        Value::Number(value) if value.is_nan() => "0/0".len(),
        Value::Number(value) if value.is_infinite() => usize::from(value < 0.0) + "1/0".len(),
        Value::Number(value) => {
            usize::from(value.is_sign_negative())
                + zmij_ecma::Buffer::new().format_finite(value.abs()).len()
        }
        // Quotes; escapes are not counted.
        Value::String(value) => value.len() + 2,
        Value::BigInt => return None,
    })
}
