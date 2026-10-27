use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, BinaryExpression, BinaryOperator, BindingPatternData, DeclData, Expr, ExprData, GetSpan,
    NodeId, ObjectProperty, Program, Span, Stmt, StmtData, UnaryOperator, VariableDeclarator,
    VariableKind, VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::{
    AnalyzeOptions, NameResolution, SymbolFlags, analyze, scope::ScopeKind,
};

use crate::CompressOptions;

#[derive(Default)]
struct Candidates {
    globals: FxHashMap<NodeId, Global>,
    typeofs: FxHashMap<NodeId, NodeId>,
    unused: FxHashSet<NodeId>,
}

#[derive(Clone, Copy)]
enum Global {
    Undefined,
    Infinity,
    Nan,
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
                        b"undefined" => Global::Undefined,
                        b"Infinity" => Global::Infinity,
                        b"NaN" => Global::Nan,
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
    ast.root_program().visit_mut_with(&mut Compressor {
        ast,
        options,
        candidates,
        in_delete: false,
        primitive_bindings: FxHashSet::default(),
        pending_typeofs: Vec::new(),
    });
}

struct Compressor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    options: CompressOptions,
    candidates: Candidates,
    in_delete: bool,
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

impl Compressor<'_, '_> {
    fn primitive_initializer(&self, declarator: VariableDeclarator) -> bool {
        declarator.init(self.ast).is_none_or(|expr| {
            matches!(
                self.ast.expr_data(expr),
                ExprData::BooleanLiteral(_)
                    | ExprData::NumericLiteral(_)
                    | ExprData::StringLiteral(_)
                    | ExprData::NullLiteral(_)
                    | ExprData::BigIntLiteral(_)
            )
        })
    }

    fn removable(&self, declarator: VariableDeclarator) -> bool {
        matches!(self.ast.binding_pattern_data(declarator.id(self.ast)),
            BindingPatternData::BindingIdentifier(binding)
            if self.candidates.unused.contains(&binding.node_id()))
            && self.primitive_initializer(declarator)
    }

    fn number(&mut self, span: Span, value: f64) -> Expr {
        Expr::NumericLiteral(self.ast.numeric_literal(span, value, None))
    }

    fn undefined(&mut self, span: Span) -> Expr {
        let zero = self.number(span, 0.0);
        Expr::UnaryExpression(self.ast.unary_expression(span, zero, UnaryOperator::Void))
    }

    fn typeof_argument(&self, expr: Expr) -> Option<Expr> {
        let ExprData::UnaryExpression(unary) = self.ast.expr_data(expr) else {
            return None;
        };
        let argument = unary.argument(self.ast);
        (unary.operator(self.ast) == UnaryOperator::Typeof
            && self.candidates.typeofs.contains_key(&argument.node_id()))
        .then_some(argument)
    }

    fn is_undefined_string(&self, expr: Expr) -> bool {
        matches!(self.ast.expr_data(expr), ExprData::StringLiteral(literal)
            if self.ast.get_wtf8(literal.value(self.ast)).as_bytes() == b"undefined")
    }
}

impl<'ast> VisitMut<'ast> for Compressor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_program(&mut self, node: Program) -> Option<Program> {
        node.visit_mut_children_with(self);
        // Forward uses can precede their declarations. Patch recorded handles on
        // exit; this worklist never visits a node or walks its children again.
        for rewrite in std::mem::take(&mut self.pending_typeofs) {
            if !self.primitive_bindings.contains(&rewrite.binding) {
                continue;
            }
            let undefined = self.undefined(rewrite.span);
            let (left, right) = if rewrite.string_on_left {
                (undefined, rewrite.argument)
            } else {
                (rewrite.argument, undefined)
            };
            rewrite.binary.set_left(self.ast, left);
            rewrite.binary.set_right(self.ast, right);
            rewrite.binary.set_operator(self.ast, rewrite.operator);
        }
        None
    }

    fn visit_mut_stmt(&mut self, node: Stmt) -> Option<Stmt> {
        if let StmtData::Declaration(declaration) = self.ast.stmt_data(node)
            && let DeclData::VariableDeclaration(declaration) = self.ast.decl_data(declaration)
            && matches!(
                declaration.kind(self.ast),
                VariableKind::Var | VariableKind::Let | VariableKind::Const
            )
        {
            let original = declaration.declarators(self.ast);
            // Loop headers do not enter visit_mut_stmt: their implicit writes
            // must not be treated as an undefined initializer.
            for id in original.iter() {
                let declarator = self.ast.get_node_in_sub_range(id);
                if self.primitive_initializer(declarator)
                    && let BindingPatternData::BindingIdentifier(binding) =
                        self.ast.binding_pattern_data(declarator.id(self.ast))
                {
                    self.primitive_bindings.insert(binding.node_id());
                }
            }

            let kept: Vec<_> = original
                .iter()
                .map(|id| self.ast.get_node_in_sub_range(id))
                .filter(|declarator| !self.removable(*declarator))
                .collect();
            if kept.is_empty() {
                // Keep a barrier against turning a following string into a directive.
                return Some(Stmt::EmptyStatement(
                    self.ast.empty_statement(node.span(self.ast)),
                ));
            }
            if kept.len() != original.len() {
                let declarators = self.ast.add_sub_range(kept);
                declaration.set_declarators(self.ast, declarators);
            }
        }
        node.visit_mut_children_with(self)
    }

    fn visit_mut_object_property(&mut self, node: ObjectProperty) -> Option<ObjectProperty> {
        let value = node.value(self.ast);
        node.visit_mut_children_with(self);
        if node.shorthand(self.ast) && node.value(self.ast) != value {
            node.set_shorthand(self.ast, false);
        }
        None
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        let span = node.span(self.ast);
        if !self.in_delete
            && let Some(global) = self.candidates.globals.get(&node.node_id()).copied()
        {
            return Some(match global {
                Global::Undefined => self.undefined(span),
                Global::Infinity | Global::Nan => {
                    let left = self.number(
                        span,
                        if matches!(global, Global::Infinity) {
                            1.0
                        } else {
                            0.0
                        },
                    );
                    let right = self.number(span, 0.0);
                    Expr::BinaryExpression(self.ast.binary_expression(
                        span,
                        left,
                        right,
                        BinaryOperator::Divide,
                    ))
                }
            });
        }
        if self.options.booleans
            && let ExprData::BooleanLiteral(literal) = self.ast.expr_data(node)
        {
            let argument = self.number(span, if literal.value(self.ast) { 0.0 } else { 1.0 });
            return Some(Expr::UnaryExpression(self.ast.unary_expression(
                span,
                argument,
                UnaryOperator::LogicalNot,
            )));
        }
        if let ExprData::BinaryExpression(binary) = self.ast.expr_data(node) {
            let operator = match binary.operator(self.ast) {
                BinaryOperator::Equal | BinaryOperator::StrictEqual => {
                    Some(BinaryOperator::StrictEqual)
                }
                BinaryOperator::NotEqual | BinaryOperator::StrictNotEqual => {
                    Some(BinaryOperator::StrictNotEqual)
                }
                _ => None,
            };
            if let Some(operator) = operator {
                let left = binary.left(self.ast);
                let right = binary.right(self.ast);
                if self.is_undefined_string(right)
                    && let Some(argument) = self.typeof_argument(left)
                {
                    self.pending_typeofs.push(TypeofRewrite {
                        binary,
                        argument,
                        binding: self.candidates.typeofs[&argument.node_id()],
                        string_on_left: false,
                        operator,
                        span: right.span(self.ast),
                    });
                } else if self.is_undefined_string(left)
                    && let Some(argument) = self.typeof_argument(right)
                {
                    self.pending_typeofs.push(TypeofRewrite {
                        binary,
                        argument,
                        binding: self.candidates.typeofs[&argument.node_id()],
                        string_on_left: true,
                        operator,
                        span: left.span(self.ast),
                    });
                }
            }
        }
        let previous = self.in_delete;
        if matches!(self.ast.expr_data(node), ExprData::UnaryExpression(unary) if unary.operator(self.ast) == UnaryOperator::Delete)
        {
            self.in_delete = true;
        }
        let result = node.visit_mut_children_with(self);
        self.in_delete = previous;
        result
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, Expr, ExprData, GetSpan, StmtData};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use crate::{Minifier, MinifierOptions};

    fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
        let result =
            Parser::init(allocator, source, Options::default(), NoTokenParserConfig).parse();
        assert!(result.diagnostics.is_empty());
        result.ast
    }

    fn expression(ast: &Ast<'_>) -> Expr {
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
