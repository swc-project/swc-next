//! Preserve shorthand property syntax and observable function/class names after renaming bindings.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, AssignmentPattern, Ast, AstUtf8, BindingProperty,
    DUMMY_SP, DeclData, Expr, Function, FunctionType, NodeId, NodeKind, NodeKindData,
    ObjectProperty, ScopeId, Stmt, StmtData, SymbolId, VariableDeclarator, VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::SymbolFlags;

use crate::context::{
    BoundIdentifier, TransformCtx, statement_injector::Anchor, unwrap_transparent_expr,
};

impl TransformCtx<'_, '_> {
    pub(super) fn preserve_renamed_syntax(&mut self, names: &FxHashMap<SymbolId, AstUtf8>) {
        let program = self.ast.root_program();
        let mut preserver = RenamedSyntax {
            ctx: self,
            names,
            hoisted: Vec::new(),
            restored_functions: FxHashSet::default(),
        };
        program.visit_mut_with(&mut preserver);

        let mut containers = Vec::new();
        for (container, statement) in preserver.hoisted {
            if !containers.contains(&container) {
                containers.push(container);
            }
            self.inject_statement(container, statement, Anchor::First);
        }

        let program_id = program.node_id();
        let program_scope = self.program_scope();
        for slot in program.body(self.ast).iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            let class = match self.ast.stmt_data(statement) {
                StmtData::Declaration(decl) => match self.ast.decl_data(decl) {
                    DeclData::Class(class) => Some(class),
                    _ => None,
                },
                StmtData::ExportDefaultDeclaration(export) => {
                    export.declaration(self.ast).as_class(self.ast)
                }
                _ => None,
            };
            if let Some(class) = class
                && let Some(identifier) = class.id(self.ast)
                && let Some(symbol) = self.semantic().symbol_of(identifier.node_id())
                && let Some(&name) = names.get(&symbol)
            {
                let restore = restore_binding_name(
                    self,
                    BoundIdentifier { identifier, symbol },
                    name,
                    program_scope,
                );
                if !containers.contains(&program_id) {
                    containers.push(program_id);
                }
                self.inject_statement(program_id, restore, Anchor::After(statement));
            }
        }

        for container in containers {
            self.inject_statements_to(container);
        }
    }
}

fn restore_binding_name(
    ctx: &mut TransformCtx<'_, '_>,
    binding: BoundIdentifier,
    name: AstUtf8,
    scope: ScopeId,
) -> Stmt {
    let target = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
    let name = Expr::StringLiteral(
        ctx.add_node(|ast| ast.string_literal(DUMMY_SP, name.into(), ast.empty_utf8())),
    );
    let value = ctx.call_set_function_name(target, name, scope);
    Stmt::ExpressionStatement(ctx.add_node(|ast| ast.expression_statement(DUMMY_SP, value)))
}

struct RenamedSyntax<'c, 's, 'a> {
    ctx: &'c mut TransformCtx<'s, 'a>,
    names: &'c FxHashMap<SymbolId, AstUtf8>,
    hoisted: Vec<(NodeId, Stmt)>,
    restored_functions: FxHashSet<SymbolId>,
}

impl RenamedSyntax<'_, '_, '_> {
    fn inferred_name(&mut self, binding: NodeId, expression: Expr) -> Option<Expr> {
        let symbol = self.ctx.semantic().symbol_of(binding).or_else(|| {
            let reference = self.ctx.semantic().reference_of(binding)?;
            self.ctx.semantic().reference(reference).symbol
        })?;
        let &name = self.names.get(&symbol)?;
        let target = unwrap_transparent_expr(expression, self.ctx.ast);
        let scope = match self.ctx.ast.kind_data(target.node_id()) {
            NodeKindData::ArrowFunctionExpression(_) => {
                self.ctx.semantic().scope_of(target.node_id())?
            }
            NodeKindData::Function(function) if function.id(self.ctx.ast).is_none() => {
                self.ctx.semantic().scope_of(target.node_id())?
            }
            NodeKindData::Class(class) if class.id(self.ctx.ast).is_none() => {
                self.ctx.semantic().scope_of(target.node_id())?
            }
            _ => return None,
        };
        let parent = self.ctx.semantic().scope(scope).parent.unwrap();
        let name = Expr::StringLiteral(
            self.ctx
                .add_node(|ast| ast.string_literal(DUMMY_SP, name.into(), ast.empty_utf8())),
        );
        Some(self.ctx.call_set_function_name(expression, name, parent))
    }
}

impl<'a> VisitMut<'a> for RenamedSyntax<'_, '_, 'a> {
    fn ast(&mut self) -> &mut Ast<'a> {
        self.ctx.ast
    }

    fn visit_mut_object_property(&mut self, node: ObjectProperty) -> Option<ObjectProperty> {
        node.visit_mut_children_with(self);
        if node.shorthand(self.ctx.ast)
            && let Some(key) = node.key(self.ctx.ast).as_identifier_name(self.ctx.ast)
            && let Some(value) = node
                .value(self.ctx.ast)
                .as_identifier_reference(self.ctx.ast)
            && self.ctx.ast.get_utf8(key.name(self.ctx.ast))
                != self.ctx.ast.get_utf8(value.name(self.ctx.ast))
        {
            node.set_shorthand(self.ctx.ast, false);
        }
        None
    }

    fn visit_mut_binding_property(&mut self, node: BindingProperty) -> Option<BindingProperty> {
        node.visit_mut_children_with(self);
        if !node.shorthand(self.ctx.ast) {
            return None;
        }
        let value = node.value(self.ctx.ast);
        let target = value
            .as_assignment_pattern(self.ctx.ast)
            .map_or(value, |pattern| pattern.left(self.ctx.ast));
        let name = target
            .as_binding_identifier(self.ctx.ast)
            .map(|identifier| identifier.name(self.ctx.ast))
            .or_else(|| {
                target
                    .as_simple_assignment_target(self.ctx.ast)?
                    .as_identifier_reference(self.ctx.ast)
                    .map(|identifier| identifier.name(self.ctx.ast))
            });
        if let Some(key) = node.key(self.ctx.ast).as_identifier_name(self.ctx.ast)
            && let Some(name) = name
            && self.ctx.ast.get_utf8(key.name(self.ctx.ast)) != self.ctx.ast.get_utf8(name)
        {
            node.set_shorthand(self.ctx.ast, false);
        }
        None
    }

    fn visit_mut_variable_declarator(
        &mut self,
        node: VariableDeclarator,
    ) -> Option<VariableDeclarator> {
        node.visit_mut_children_with(self);
        if let Some(value) = node.init(self.ctx.ast)
            && let Some(value) = self.inferred_name(node.id(self.ctx.ast).node_id(), value)
        {
            node.set_init(self.ctx.ast, Some(value));
        }
        None
    }

    fn visit_mut_assignment_pattern(
        &mut self,
        node: AssignmentPattern,
    ) -> Option<AssignmentPattern> {
        node.visit_mut_children_with(self);
        if let Some(value) =
            self.inferred_name(node.left(self.ctx.ast).node_id(), node.right(self.ctx.ast))
        {
            node.set_right(self.ctx.ast, value);
        }
        None
    }

    fn visit_mut_assignment_expression(
        &mut self,
        node: AssignmentExpression,
    ) -> Option<AssignmentExpression> {
        node.visit_mut_children_with(self);
        if matches!(
            node.operator(self.ctx.ast),
            AssignmentOperator::Assign
                | AssignmentOperator::LogicalAndAssign
                | AssignmentOperator::LogicalOrAssign
                | AssignmentOperator::NullishAssign
        ) && let Some(value) =
            self.inferred_name(node.left(self.ctx.ast).node_id(), node.right(self.ctx.ast))
        {
            node.set_right(self.ctx.ast, value);
        }
        None
    }

    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        node.visit_mut_children_with(self);
        if node.function_type(self.ctx.ast) != FunctionType::FunctionDeclaration {
            return None;
        }
        let identifier = node.id(self.ctx.ast)?;
        let symbol = self.ctx.semantic().symbol_of(identifier.node_id())?;
        let &name = self.names.get(&symbol)?;
        debug_assert!(
            !self
                .ctx
                .semantic()
                .symbol(symbol)
                .flags
                .contains(SymbolFlags::ANNEX_B_IF_FUNCTION)
        );
        if !self.restored_functions.insert(symbol) {
            return None;
        }
        let scope = self.ctx.semantic().scope_of_symbol(symbol);
        let owner = self.ctx.semantic().scope(scope).node;
        debug_assert_ne!(self.ctx.ast.node_kind(owner), NodeKind::SwitchStatement);
        let container = match self.ctx.ast.kind_data(owner) {
            NodeKindData::Function(function) => function.body(self.ctx.ast).node_id(),
            _ => owner,
        };
        let statement = restore_binding_name(
            self.ctx,
            BoundIdentifier { identifier, symbol },
            name,
            scope,
        );
        self.hoisted.push((container, statement));
        None
    }
}
