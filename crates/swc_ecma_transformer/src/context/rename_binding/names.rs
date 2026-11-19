//! Preserve observable function/class names when their bindings are renamed.

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    Accessibility, Argument, AssignmentExpression, AssignmentOperator, AssignmentPattern, Ast,
    AstUtf8, Class, ClassElement, DUMMY_SP, Expr, Function, FunctionType, IfStatement, NodeId,
    NodeKindData, PrivateIdentifier, PropertyKey, ScopeId, Stmt, SubRange, SymbolId,
    VariableDeclarator, Visit, VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_semantic::{
    SymbolFlags,
    scope::{ScopeFlags, ScopeKind},
};

use crate::context::{BoundIdentifier, TransformCtx, statement_injector::Anchor};
use crate::{helpers::HelperKind, uid::UidGenerator};

impl TransformCtx<'_, '_> {
    pub(super) fn preserve_renamed_function_names(&mut self, names: &FxHashMap<SymbolId, AstUtf8>) {
        let program = self.ast.root_program();
        let mut preserver = FunctionNames {
            ctx: self,
            names,
            hoisted: Vec::new(),
        };
        program.visit_mut_with(&mut preserver);
        let hoisted = preserver.hoisted;
        for (container, statement) in hoisted {
            self.inject_statement(container, statement, Anchor::First);
            if container != program.node_id() {
                self.inject_statements_to(container);
            }
        }
    }
}

struct FunctionNames<'c, 's, 'a> {
    ctx: &'c mut TransformCtx<'s, 'a>,
    names: &'c FxHashMap<SymbolId, AstUtf8>,
    hoisted: Vec<(NodeId, Stmt)>,
}

impl FunctionNames<'_, '_, '_> {
    fn inferred_name(&mut self, binding: NodeId, expression: Expr) -> Option<Expr> {
        let symbol = self.ctx.semantic().symbol_of(binding).or_else(|| {
            let reference = self.ctx.semantic().reference_of(binding)?;
            self.ctx.semantic().reference(reference).symbol
        })?;
        let &name = self.names.get(&symbol)?;
        let anonymous = match self.ctx.ast.kind_data(expression.node_id()) {
            NodeKindData::ArrowFunctionExpression(_) => true,
            NodeKindData::Function(function) => function.id(self.ctx.ast).is_none(),
            NodeKindData::Class(class) => class.id(self.ctx.ast).is_none(),
            _ => false,
        };
        if !anonymous {
            return None;
        }
        if let Some(class) = expression.as_class(self.ctx.ast) {
            self.set_class_name(class, name);
            return Some(expression);
        }
        let scope = self.ctx.semantic().scope_of(expression.node_id())?;
        let parent = self.ctx.semantic().scope(scope).parent.unwrap();
        let name = Expr::StringLiteral(
            self.ctx
                .add_node(|ast| ast.string_literal(DUMMY_SP, name.into(), ast.empty_utf8())),
        );
        Some(self.ctx.call_set_function_name(expression, name, parent))
    }

    fn set_class_name(&mut self, node: Class, name: AstUtf8) {
        let body = node.body(self.ctx.ast);
        let mut private_names = PrivateNames {
            ast: self.ctx.ast,
            names: UidGenerator::new(self.ctx.ast.allocator()),
        };
        // A private access can refer to a name declared by an enclosing class.
        body.visit_with(&mut private_names);
        let key = self
            .ctx
            .ast
            .add_utf8(private_names.names.next_name("_name", 2));
        let members: Vec<_> = body
            .body(self.ctx.ast)
            .iter()
            .map(|slot| self.ctx.ast.get_node_in_sub_range(slot))
            .collect();
        let key = PropertyKey::Expr(Expr::PrivateIdentifier(
            self.ctx
                .add_node(|ast| ast.private_identifier(DUMMY_SP, key)),
        ));
        let scope = self.ctx.semantic().scope_of(node.node_id()).unwrap();
        let target = Expr::ThisExpression(self.ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
        let name = Expr::StringLiteral(
            self.ctx
                .add_node(|ast| ast.string_literal(DUMMY_SP, name.into(), ast.empty_utf8())),
        );
        let callee = self.ctx.helper_callee(HelperKind::SetClassName, scope);
        let value = self
            .ctx
            .call(callee, [Argument::Expr(target), Argument::Expr(name)]);
        let field = ClassElement::PropertyDefinition(self.ctx.add_node(|ast| {
            ast.property_definition(
                DUMMY_SP,
                SubRange::empty(),
                key,
                None,
                Some(value),
                false,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                Accessibility::None,
            )
        }));
        let members = self
            .ctx
            .ast
            .add_sub_range(std::iter::once(field).chain(members));
        body.set_body(self.ctx.ast, members);
    }

    fn restore_function_name(
        &mut self,
        binding: BoundIdentifier,
        name: AstUtf8,
        scope: ScopeId,
    ) -> Stmt {
        let target = Expr::IdentifierReference(binding.reference_read(self.ctx, scope, DUMMY_SP));
        let name = Expr::StringLiteral(
            self.ctx
                .add_node(|ast| ast.string_literal(DUMMY_SP, name.into(), ast.empty_utf8())),
        );
        let value = self.ctx.call_set_function_name(target, name, scope);
        Stmt::ExpressionStatement(
            self.ctx
                .add_node(|ast| ast.expression_statement(DUMMY_SP, value)),
        )
    }

    fn wrap_annex_b_branch(&mut self, branch: Stmt) -> Option<Stmt> {
        let function = branch
            .as_declaration(self.ctx.ast)?
            .as_function(self.ctx.ast)?;
        let identifier = function.id(self.ctx.ast)?;
        let symbol = self.ctx.semantic().symbol_of(identifier.node_id())?;
        let &name = self.names.get(&symbol)?;
        if !self
            .ctx
            .semantic()
            .symbol(symbol)
            .flags
            .contains(SymbolFlags::ANNEX_B_IF_FUNCTION)
        {
            return None;
        }
        let function_scope = self.ctx.semantic().scope_of_symbol(symbol);
        let parent = self.ctx.semantic().scope(function_scope).parent.unwrap();
        let flags = ScopeFlags {
            strict: self.ctx.semantic().scope(parent).flags.strict,
            ..ScopeFlags::default()
        };
        let block = self
            .ctx
            .add_node(|ast| ast.block_statement(DUMMY_SP, SubRange::empty()));
        let block_scope =
            self.ctx
                .create_child_scope(parent, block.node_id(), ScopeKind::Block, flags);
        self.ctx.reparent_scope(function_scope, block_scope);
        let text = self.ctx.ast.get_utf8(identifier.name(self.ctx.ast));
        self.ctx
            .state
            .semantic
            .move_annex_b_if_binding_to_block(symbol, block_scope, text);
        let restore =
            self.restore_function_name(BoundIdentifier { identifier, symbol }, name, block_scope);
        let statements = self.ctx.ast.add_sub_range([branch, restore]);
        block.set_body(self.ctx.ast, statements);
        Some(Stmt::BlockStatement(block))
    }
}

struct PrivateNames<'s, 'a> {
    ast: &'s Ast<'a>,
    names: UidGenerator<'a>,
}

impl<'a> Visit<'a> for PrivateNames<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn visit_private_identifier(&mut self, node: PrivateIdentifier) {
        self.names
            .observe(self.ast.get_utf8(node.name(self.ast)).as_bytes());
    }
}

impl<'a> VisitMut<'a> for FunctionNames<'_, '_, 'a> {
    fn ast(&mut self) -> &mut Ast<'a> {
        self.ctx.ast
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
        if self
            .ctx
            .semantic()
            .symbol(symbol)
            .flags
            .contains(SymbolFlags::ANNEX_B_IF_FUNCTION)
        {
            return None;
        }
        let scope = self.ctx.semantic().scope_of_symbol(symbol);
        let owner = self.ctx.semantic().scope(scope).node;
        let container = match self.ctx.ast.kind_data(owner) {
            NodeKindData::Function(function) => function.body(self.ctx.ast).node_id(),
            _ => owner,
        };
        let statement =
            self.restore_function_name(BoundIdentifier { identifier, symbol }, name, scope);
        self.hoisted.push((container, statement));
        None
    }

    fn visit_mut_if_statement(&mut self, node: IfStatement) -> Option<IfStatement> {
        node.visit_mut_children_with(self);
        if let Some(branch) = self.wrap_annex_b_branch(node.consequent(self.ctx.ast)) {
            node.set_consequent(self.ctx.ast, branch);
        }
        if let Some(branch) = node
            .alternate(self.ctx.ast)
            .and_then(|branch| self.wrap_annex_b_branch(branch))
        {
            node.set_alternate(self.ctx.ast, Some(branch));
        }
        None
    }

    fn visit_mut_class(&mut self, node: Class) -> Option<Class> {
        node.visit_mut_children_with(self);
        let identifier = node.id(self.ctx.ast)?;
        let symbol = self.ctx.semantic().symbol_of(identifier.node_id())?;
        let &name = self.names.get(&symbol)?;
        self.set_class_name(node, name);
        None
    }
}
