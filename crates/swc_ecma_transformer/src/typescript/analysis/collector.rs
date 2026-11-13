use super::super::TypeScriptOptions;
use super::{enums::EnumCollector, modules::ModuleCollector, namespaces::NamespaceCollector};
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, CallExpression, ExprData, Function, Lang, MemberExpression,
    MethodDefinition, NodeId, ObjectProperty, PropertyDefinition, PropertyKind,
    SimpleAssignmentTarget, Stmt, TsEnumDeclaration, TsEnumMember, TsGlobalDeclaration,
    TsImportEqualsDeclaration, TsInterfaceDeclaration, TsModuleDeclaration, TsType,
    UnaryExpression, VariableDeclaration, VariableDeclarator, VariableKind, Visit, VisitWith,
};
use swc_next_ecma_semantic::Semantic;

pub(super) struct AnalysisCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    semantic: &'visit Semantic<'ast>,
    pub(super) enums: EnumCollector,
    ambient: bool,
    current_enum: Option<usize>,
    constant: bool,
    immediately_invoked: Option<NodeId>,
    pub(super) modules: ModuleCollector,
    pub(super) namespaces: NamespaceCollector,
}

impl<'visit, 'ast> AnalysisCollector<'visit, 'ast> {
    pub(super) fn new(
        ast: &'visit Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
        options: &TypeScriptOptions,
    ) -> Self {
        Self {
            ast,
            semantic,
            enums: EnumCollector::new(ast, semantic, options),
            ambient: ast.lang() == Lang::Dts,
            current_enum: None,
            constant: false,
            immediately_invoked: None,
            modules: ModuleCollector::new(options),
            namespaces: NamespaceCollector::default(),
        }
    }
}

impl<'ast> Visit<'ast> for AnalysisCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    // Runtime emission does not depend on expressions inside erased types.
    // Semantic reference flags already distinguish type uses from runtime uses.
    fn visit_ts_type(&mut self, _node: TsType) {}

    fn visit_ts_interface_declaration(&mut self, _node: TsInterfaceDeclaration) {}

    fn visit_call_expression(&mut self, call: CallExpression) {
        let mut callee = call.callee(self.ast);
        while let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(callee) {
            callee = parenthesized.expression(self.ast);
        }
        let previous = self.immediately_invoked;
        self.immediately_invoked = Some(callee.node_id());
        call.visit_children_with(self);
        self.immediately_invoked = previous;
    }

    fn visit_function(&mut self, function: Function) {
        let region = (self.immediately_invoked != Some(function.node_id()))
            .then(|| self.enums.availability.enter_deferred());
        function.visit_children_with(self);
        if let Some(region) = region {
            self.enums.availability.exit_deferred(region);
        }
    }

    fn visit_arrow_function_expression(&mut self, function: ArrowFunctionExpression) {
        let region = (self.immediately_invoked != Some(function.node_id()))
            .then(|| self.enums.availability.enter_deferred());
        function.visit_children_with(self);
        if let Some(region) = region {
            self.enums.availability.exit_deferred(region);
        }
    }

    fn visit_method_definition(&mut self, method: MethodDefinition) {
        method.decorators(self.ast).visit_with(self);
        // TypeScript's function-like boundary includes a method's computed name,
        // which is outside the Function node in our AST. This is only a constant
        // evaluation rule; computed names still execute when the class is created.
        let region = self.enums.availability.enter_deferred();
        method.key(self.ast).visit_with(self);
        method.value(self.ast).visit_with(self);
        self.enums.availability.exit_deferred(region);
    }

    fn visit_object_property(&mut self, property: ObjectProperty) {
        let region = (property.method(self.ast) || property.kind(self.ast) != PropertyKind::Init)
            .then(|| self.enums.availability.enter_deferred());
        property.visit_children_with(self);
        if let Some(region) = region {
            self.enums.availability.exit_deferred(region);
        }
    }

    fn visit_property_definition(&mut self, property: PropertyDefinition) {
        property.decorators(self.ast).visit_with(self);
        property.key(self.ast).visit_with(self);
        property.type_annotation(self.ast).visit_with(self);
        if let Some(initializer) = property.value(self.ast) {
            let region =
                (!property.r#static(self.ast)).then(|| self.enums.availability.enter_deferred());
            initializer.visit_with(self);
            if let Some(region) = region {
                self.enums.availability.exit_deferred(region);
            }
        }
    }

    fn visit_simple_assignment_target(&mut self, target: SimpleAssignmentTarget) {
        let previous = self
            .enums
            .set_assignment_target(Some(target.node_id()), self.ast);
        target.visit_children_with(self);
        self.enums.set_assignment_target(previous, self.ast);
    }

    fn visit_stmt(&mut self, statement: Stmt) {
        self.namespaces.record_statement(statement, self.ast);
        statement.visit_children_with(self);
    }

    fn visit_ts_enum_declaration(&mut self, declaration: TsEnumDeclaration) {
        let owner =
            self.enums
                .record_enum_declaration(declaration, self.ambient, self.ast, self.semantic);
        let previous = std::mem::replace(&mut self.current_enum, owner);
        declaration.visit_children_with(self);
        self.current_enum = previous;
    }

    fn visit_ts_enum_member(&mut self, member: TsEnumMember) {
        self.enums
            .record_member_declaration(member, self.current_enum, self.semantic);
        member.visit_children_with(self);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: TsImportEqualsDeclaration) {
        self.modules
            .record_import_equals(declaration, self.ast, self.semantic);
        declaration.visit_children_with(self);
    }

    fn visit_ts_module_declaration(&mut self, declaration: TsModuleDeclaration) {
        self.namespaces.record_declaration(declaration);
        let ambient = self.ambient;
        self.ambient |= declaration.declare(self.ast);
        declaration.visit_children_with(self);
        self.ambient = ambient;
    }

    fn visit_ts_global_declaration(&mut self, declaration: TsGlobalDeclaration) {
        let ambient = self.ambient;
        self.ambient = true;
        declaration.visit_children_with(self);
        self.ambient = ambient;
    }

    fn visit_unary_expression(&mut self, expression: UnaryExpression) {
        let target = (expression.operator(self.ast) == swc_next_ecma_ast::UnaryOperator::Delete)
            .then(|| expression.argument(self.ast).node_id());
        let previous = self.enums.set_assignment_target(target, self.ast);
        expression.visit_children_with(self);
        self.enums.set_assignment_target(previous, self.ast);
    }

    fn visit_variable_declaration(&mut self, declaration: VariableDeclaration) {
        let constant = declaration.kind(self.ast) == VariableKind::Const;
        let previous = std::mem::replace(&mut self.constant, constant);
        declaration.visit_children_with(self);
        self.constant = previous;
    }

    fn visit_variable_declarator(&mut self, declarator: VariableDeclarator) {
        self.enums
            .record_variable_declarator(declarator, self.constant, self.ast, self.semantic);
        declarator.visit_children_with(self);
    }

    fn visit_member_expression(&mut self, member: MemberExpression) {
        self.enums
            .record_member_expression(member, self.ast, self.semantic);
        member.visit_children_with(self);
    }
}
