use rustc_hash::FxHashSet;

use swc_next_ecma_ast::{
    AssignmentOperator, AssignmentTarget, Ast, AstUtf8, AstWtf8, DUMMY_SP, Expr, ExprData, GetSpan,
    ObjectProperty, PropertyKey, SimpleAssignmentTarget, Stmt, SubRange, SymbolId,
    TsEnumDeclaration, UnaryOperator, VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::Semantic;

use super::{StatementExpansion, TypeScriptLowerer, runtime::RuntimeTarget};
use crate::typescript::analysis::TsEnumConstantValue;

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn lower_enum(
        &mut self,
        declaration: TsEnumDeclaration,
        exported: bool,
    ) -> StatementExpansion {
        if declaration.declare(self.ast_ref()) {
            return StatementExpansion::Remove;
        }
        let binding = declaration.id(self.ast_ref());
        let owner = self
            .semantic
            .symbol_of(binding.node_id())
            .expect("an enum declaration has a symbol");
        if self.analysis.const_enum_is_removable(owner) {
            return StatementExpansion::Remove;
        }

        let enum_name = binding.name(self.ast_ref());
        let span = declaration.span(self.ast_ref());
        let prefix = self
            .register_runtime_binding(owner)
            .map(|kind| self.runtime_binding_statement(binding, span, kind, exported));
        let statement = self.enum_iife_statement(
            declaration,
            owner,
            enum_name,
            RuntimeTarget::Identifier(enum_name),
        );
        StatementExpansion::with_prefix(prefix, statement)
    }

    pub(super) fn lower_namespace_enum(
        &mut self,
        declaration: TsEnumDeclaration,
        namespace: AstUtf8,
        output: &mut Vec<Stmt>,
    ) {
        if declaration.declare(self.ast_ref()) {
            return;
        }
        let binding = declaration.id(self.ast_ref());
        let owner = self
            .semantic
            .symbol_of(binding.node_id())
            .expect("an enum declaration has a symbol");
        if self.analysis.const_enum_is_removable(owner) {
            return;
        }
        let enum_name = binding.name(self.ast_ref());
        let span = declaration.span(self.ast_ref());
        if let Some(kind) = self.register_runtime_binding(owner) {
            output.push(self.runtime_binding_statement(binding, span, kind, false));
        }
        output.push(self.enum_iife_statement(
            declaration,
            owner,
            enum_name,
            RuntimeTarget::IdentifierAndProperty {
                identifier: enum_name,
                object: namespace,
                property: enum_name,
            },
        ));
    }

    pub(super) fn inline_enum_constant(
        &mut self,
        expression: Expr,
        value: TsEnumConstantValue<'_>,
    ) -> Expr {
        let span = expression.span(self.ast_ref());
        self.enum_constant_expression(value, span)
    }

    fn enum_iife_statement(
        &mut self,
        declaration: TsEnumDeclaration,
        owner: SymbolId,
        enum_name: AstUtf8,
        target: RuntimeTarget,
    ) -> Stmt {
        let parameter = self.container_parameter(enum_name, owner);
        let members = declaration.body(self.ast_ref()).members(self.ast_ref());
        let mut statements = Vec::with_capacity(members.len());
        for member_id in members.iter() {
            let member = self.ast_ref().get_node_in_sub_range(member_id);
            let Some(member_symbol) = self.semantic.symbol_of(member.node_id()) else {
                if let Some(initializer) = member.initializer(self.ast_ref()) {
                    let initializer = initializer.visit_mut_with(self).unwrap_or(initializer);
                    let span = member.span(self.ast_ref());
                    statements.push(self.ast().stmt_expression_statement(span, initializer));
                }
                continue;
            };
            let member_name = self.semantic.symbol(member_symbol).name;
            let value = self.analysis.enum_member_value(member_symbol);
            let initializer = if let Some(constant) = value.constant() {
                self.enum_constant_expression(constant, member.span(self.ast_ref()))
            } else if let Some(initializer) = member.initializer(self.ast_ref()) {
                let initializer = initializer.visit_mut_with(self).unwrap_or(initializer);
                self.rewrite_enum_initializer(initializer, owner, parameter)
            } else {
                let zero = self.ast().expr_numeric_literal(DUMMY_SP, 0.0, None);
                self.ast()
                    .expr_unary_expression(DUMMY_SP, zero, UnaryOperator::Void)
            };
            statements.push(self.enum_member_statement(
                parameter,
                member_name.into(),
                initializer,
                value.is_string(),
            ));
        }

        let statements = self.ast().add_sub_range(statements);
        self.object_iife_statement(parameter, SubRange::empty(), statements, target, DUMMY_SP)
    }

    fn enum_member_statement(
        &mut self,
        enum_name: AstUtf8,
        member_name: AstWtf8,
        initializer: Expr,
        string_member: bool,
    ) -> Stmt {
        let member = self.enum_member_access(enum_name, member_name);
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberExpression(member),
        );
        let assignment = self.ast().expr_assignment_expression(
            DUMMY_SP,
            target,
            initializer,
            AssignmentOperator::Assign,
        );
        if string_member {
            return self.ast().stmt_expression_statement(DUMMY_SP, assignment);
        }

        let object = self.ast().expr_identifier_reference(DUMMY_SP, enum_name);
        let outer_member = self.ast().member_expression(
            DUMMY_SP,
            object,
            PropertyKey::Expr(assignment),
            true,
            false,
        );
        let outer_target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberExpression(outer_member),
        );
        let name = self.enum_member_name_literal(member_name);
        let outer_assignment = self.ast().expr_assignment_expression(
            DUMMY_SP,
            outer_target,
            Expr::StringLiteral(name),
            AssignmentOperator::Assign,
        );
        self.ast()
            .stmt_expression_statement(DUMMY_SP, outer_assignment)
    }

    fn enum_member_access(
        &mut self,
        enum_name: AstUtf8,
        member_name: AstWtf8,
    ) -> swc_next_ecma_ast::MemberExpression {
        let object = self.ast().expr_identifier_reference(DUMMY_SP, enum_name);
        let property = self.enum_member_name_literal(member_name);
        self.ast().member_expression(
            DUMMY_SP,
            object,
            PropertyKey::Expr(Expr::StringLiteral(property)),
            true,
            false,
        )
    }

    fn enum_member_name_literal(
        &mut self,
        member_name: AstWtf8,
    ) -> swc_next_ecma_ast::StringLiteral {
        self.ast()
            .string_literal(DUMMY_SP, member_name, AstUtf8::EMPTY)
    }

    fn enum_constant_expression(
        &mut self,
        value: TsEnumConstantValue<'_>,
        span: swc_next_ecma_ast::Span,
    ) -> Expr {
        match value {
            TsEnumConstantValue::String(value) => {
                let value = self.ast().add_wtf8(value);
                Expr::StringLiteral(self.ast().string_literal(span, value, AstUtf8::EMPTY))
            }
            TsEnumConstantValue::Number { value, raw } => {
                // TypeScript prints either zero as `0`, but evaluation must retain its sign.
                let value = if value == 0.0 { 0.0 } else { value };
                Expr::NumericLiteral(self.ast().numeric_literal(span, value, raw))
            }
        }
    }

    fn rewrite_enum_initializer(
        &mut self,
        initializer: Expr,
        owner: SymbolId,
        enum_name: AstUtf8,
    ) -> Expr {
        let members = self
            .enum_member_symbols
            .entry(owner)
            .or_insert_with(|| self.semantic.ts_enum_members(owner).collect());
        let mut rewriter = EnumInitializerRewriter {
            ast: self.ast,
            semantic: self.semantic,
            semantic_node_count: self.semantic_node_count,
            members,
            enum_name,
        };
        initializer
            .visit_mut_with(&mut rewriter)
            .unwrap_or(initializer)
    }
}

struct EnumInitializerRewriter<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    semantic: &'visit Semantic<'ast>,
    semantic_node_count: usize,
    members: &'visit FxHashSet<SymbolId>,
    enum_name: AstUtf8,
}

impl<'ast> VisitMut<'ast> for EnumInitializerRewriter<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        let ExprData::IdentifierReference(identifier) = self.ast.expr_data(expression) else {
            return expression.visit_mut_children_with(self);
        };
        if identifier.node_id().index() >= self.semantic_node_count {
            return None;
        }
        let symbol = self
            .semantic
            .reference_of(identifier.node_id())
            .and_then(|reference| self.semantic.reference(reference).resolution().symbol());
        if !symbol.is_some_and(|symbol| self.members.contains(&symbol)) {
            return None;
        }

        let span = identifier.span(self.ast);
        let name = identifier.name(self.ast);
        let object = self.ast.expr_identifier_reference(DUMMY_SP, self.enum_name);
        let property = self.ast.identifier_name(span, name);
        Some(self.ast.expr_member_expression(
            span,
            object,
            PropertyKey::IdentifierName(property),
            false,
            false,
        ))
    }

    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        property.visit_mut_children_with(self);
        let shorthand_value_is_identifier = matches!(
            self.ast.expr_data(property.value(self.ast)),
            ExprData::IdentifierReference(_)
        );
        if property.shorthand(self.ast) && !shorthand_value_is_identifier {
            property.set_shorthand(self.ast, false);
        }
        None
    }
}
