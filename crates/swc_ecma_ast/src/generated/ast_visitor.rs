#![allow(unused, clippy::useless_conversion, clippy::single_match)]
use crate::*;
pub trait Visit<'a> {
    fn ast(&self) -> &Ast<'a>;
    #[inline]
    fn enter_node(&mut self, node_id: NodeId) {}
    #[inline]
    fn leave_node(&mut self, node_id: NodeId) {}
    #[inline]
    fn visit_program(&mut self, node: Program) {
        self.enter_node(node.node_id());
        <Program as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_stmt(&mut self, node: Stmt) {
        <Stmt as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_statement_list_item(&mut self, node: StatementListItem) {
        <StatementListItem as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_decl(&mut self, node: Decl) {
        <Decl as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_expr(&mut self, node: Expr) {
        <Expr as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_binding_pattern(&mut self, node: BindingPattern) {
        <BindingPattern as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_assignment_target(&mut self, node: AssignmentTarget) {
        <AssignmentTarget as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_simple_assignment_target(&mut self, node: SimpleAssignmentTarget) {
        <SimpleAssignmentTarget as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_property_key(&mut self, node: PropertyKey) {
        <PropertyKey as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_module_export_name(&mut self, node: ModuleExportName) {
        <ModuleExportName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_argument(&mut self, node: Argument) {
        <Argument as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_object_property_kind(&mut self, node: ObjectPropertyKind) {
        <ObjectPropertyKind as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_for_statement_init(&mut self, node: ForStatementInit) {
        <ForStatementInit as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_for_statement_left(&mut self, node: ForStatementLeft) {
        <ForStatementLeft as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_arrow_function_body(&mut self, node: ArrowFunctionBody) {
        <ArrowFunctionBody as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_formal_parameter_item(&mut self, node: FormalParameterItem) {
        <FormalParameterItem as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_formal_parameter_pattern(&mut self, node: FormalParameterPattern) {
        <FormalParameterPattern as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class_element(&mut self, node: ClassElement) {
        <ClassElement as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_declaration_specifier(&mut self, node: ImportDeclarationSpecifier) {
        <ImportDeclarationSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_export_default_declaration_kind(&mut self, node: ExportDefaultDeclarationKind) {
        <ExportDefaultDeclarationKind as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_super(&mut self, node: Super) {
        self.enter_node(node.node_id());
        <Super as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_this_expression(&mut self, node: ThisExpression) {
        self.enter_node(node.node_id());
        <ThisExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_string_literal(&mut self, node: StringLiteral) {
        self.enter_node(node.node_id());
        <StringLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_numeric_literal(&mut self, node: NumericLiteral) {
        self.enter_node(node.node_id());
        <NumericLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_big_int_literal(&mut self, node: BigIntLiteral) {
        self.enter_node(node.node_id());
        <BigIntLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_boolean_literal(&mut self, node: BooleanLiteral) {
        self.enter_node(node.node_id());
        <BooleanLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_null_literal(&mut self, node: NullLiteral) {
        self.enter_node(node.node_id());
        <NullLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_reg_exp_literal(&mut self, node: RegExpLiteral) {
        self.enter_node(node.node_id());
        <RegExpLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_template_literal(&mut self, node: TemplateLiteral) {
        self.enter_node(node.node_id());
        <TemplateLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_template_element(&mut self, node: TemplateElement) {
        self.enter_node(node.node_id());
        <TemplateElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        self.enter_node(node.node_id());
        <IdentifierReference as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_private_identifier(&mut self, node: PrivateIdentifier) {
        self.enter_node(node.node_id());
        <PrivateIdentifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        self.enter_node(node.node_id());
        <BindingIdentifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_identifier_name(&mut self, node: IdentifierName) {
        self.enter_node(node.node_id());
        <IdentifierName as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_label_identifier(&mut self, node: LabelIdentifier) {
        self.enter_node(node.node_id());
        <LabelIdentifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assignment_pattern(&mut self, node: AssignmentPattern) {
        self.enter_node(node.node_id());
        <AssignmentPattern as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binding_rest_element(&mut self, node: BindingRestElement) {
        self.enter_node(node.node_id());
        <BindingRestElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_array_pattern(&mut self, node: ArrayPattern) {
        self.enter_node(node.node_id());
        <ArrayPattern as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_pattern(&mut self, node: ObjectPattern) {
        self.enter_node(node.node_id());
        <ObjectPattern as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binding_property(&mut self, node: BindingProperty) {
        self.enter_node(node.node_id());
        <BindingProperty as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binary_expression(&mut self, node: BinaryExpression) {
        self.enter_node(node.node_id());
        <BinaryExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_logical_expression(&mut self, node: LogicalExpression) {
        self.enter_node(node.node_id());
        <LogicalExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_conditional_expression(&mut self, node: ConditionalExpression) {
        self.enter_node(node.node_id());
        <ConditionalExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_unary_expression(&mut self, node: UnaryExpression) {
        self.enter_node(node.node_id());
        <UnaryExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_update_expression(&mut self, node: UpdateExpression) {
        self.enter_node(node.node_id());
        <UpdateExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
        self.enter_node(node.node_id());
        <AssignmentExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_variable_declaration(&mut self, node: VariableDeclaration) {
        self.enter_node(node.node_id());
        <VariableDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_variable_declarator(&mut self, node: VariableDeclarator) {
        self.enter_node(node.node_id());
        <VariableDeclarator as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_if_statement(&mut self, node: IfStatement) {
        self.enter_node(node.node_id());
        <IfStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_switch_statement(&mut self, node: SwitchStatement) {
        self.enter_node(node.node_id());
        <SwitchStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_switch_case(&mut self, node: SwitchCase) {
        self.enter_node(node.node_id());
        <SwitchCase as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_statement(&mut self, node: ForStatement) {
        self.enter_node(node.node_id());
        <ForStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_in_statement(&mut self, node: ForInStatement) {
        self.enter_node(node.node_id());
        <ForInStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_of_statement(&mut self, node: ForOfStatement) {
        self.enter_node(node.node_id());
        <ForOfStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_try_statement(&mut self, node: TryStatement) {
        self.enter_node(node.node_id());
        <TryStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_catch_clause(&mut self, node: CatchClause) {
        self.enter_node(node.node_id());
        <CatchClause as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_break_statement(&mut self, node: BreakStatement) {
        self.enter_node(node.node_id());
        <BreakStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_continue_statement(&mut self, node: ContinueStatement) {
        self.enter_node(node.node_id());
        <ContinueStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_labeled_statement(&mut self, node: LabeledStatement) {
        self.enter_node(node.node_id());
        <LabeledStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_expression_statement(&mut self, node: ExpressionStatement) {
        self.enter_node(node.node_id());
        <ExpressionStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_return_statement(&mut self, node: ReturnStatement) {
        self.enter_node(node.node_id());
        <ReturnStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_throw_statement(&mut self, node: ThrowStatement) {
        self.enter_node(node.node_id());
        <ThrowStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_debugger_statement(&mut self, node: DebuggerStatement) {
        self.enter_node(node.node_id());
        <DebuggerStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_empty_statement(&mut self, node: EmptyStatement) {
        self.enter_node(node.node_id());
        <EmptyStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_directive(&mut self, node: Directive) {
        self.enter_node(node.node_id());
        <Directive as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_decorator(&mut self, node: Decorator) {
        self.enter_node(node.node_id());
        <Decorator as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class(&mut self, node: Class) {
        self.enter_node(node.node_id());
        <Class as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_body(&mut self, node: ClassBody) {
        self.enter_node(node.node_id());
        <ClassBody as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_method_definition(&mut self, node: MethodDefinition) {
        self.enter_node(node.node_id());
        <MethodDefinition as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_property_definition(&mut self, node: PropertyDefinition) {
        self.enter_node(node.node_id());
        <PropertyDefinition as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_static_block(&mut self, node: StaticBlock) {
        self.enter_node(node.node_id());
        <StaticBlock as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_function(&mut self, node: Function) {
        self.enter_node(node.node_id());
        <Function as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_function_body(&mut self, node: FunctionBody) {
        self.enter_node(node.node_id());
        <FunctionBody as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_block_statement(&mut self, node: BlockStatement) {
        self.enter_node(node.node_id());
        <BlockStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_while_statement(&mut self, node: WhileStatement) {
        self.enter_node(node.node_id());
        <WhileStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_do_while_statement(&mut self, node: DoWhileStatement) {
        self.enter_node(node.node_id());
        <DoWhileStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_with_statement(&mut self, node: WithStatement) {
        self.enter_node(node.node_id());
        <WithStatement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_formal_parameters(&mut self, node: FormalParameters) {
        self.enter_node(node.node_id());
        <FormalParameters as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_formal_parameter(&mut self, node: FormalParameter) {
        self.enter_node(node.node_id());
        <FormalParameter as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_array_expression(&mut self, node: ArrayExpression) {
        self.enter_node(node.node_id());
        <ArrayExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_expression(&mut self, node: ObjectExpression) {
        self.enter_node(node.node_id());
        <ObjectExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_spread_element(&mut self, node: SpreadElement) {
        self.enter_node(node.node_id());
        <SpreadElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_property(&mut self, node: ObjectProperty) {
        self.enter_node(node.node_id());
        <ObjectProperty as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_parenthesized_expression(&mut self, node: ParenthesizedExpression) {
        self.enter_node(node.node_id());
        <ParenthesizedExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_arrow_function_expression(&mut self, node: ArrowFunctionExpression) {
        self.enter_node(node.node_id());
        <ArrowFunctionExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_sequence_expression(&mut self, node: SequenceExpression) {
        self.enter_node(node.node_id());
        <SequenceExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_member_expression(&mut self, node: MemberExpression) {
        self.enter_node(node.node_id());
        <MemberExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_call_expression(&mut self, node: CallExpression) {
        self.enter_node(node.node_id());
        <CallExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_chain_expression(&mut self, node: ChainExpression) {
        self.enter_node(node.node_id());
        <ChainExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_tagged_template_expression(&mut self, node: TaggedTemplateExpression) {
        self.enter_node(node.node_id());
        <TaggedTemplateExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_new_expression(&mut self, node: NewExpression) {
        self.enter_node(node.node_id());
        <NewExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_await_expression(&mut self, node: AwaitExpression) {
        self.enter_node(node.node_id());
        <AwaitExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_yield_expression(&mut self, node: YieldExpression) {
        self.enter_node(node.node_id());
        <YieldExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_meta_property(&mut self, node: MetaProperty) {
        self.enter_node(node.node_id());
        <MetaProperty as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_expression(&mut self, node: ImportExpression) {
        self.enter_node(node.node_id());
        <ImportExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_declaration(&mut self, node: ImportDeclaration) {
        self.enter_node(node.node_id());
        <ImportDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        self.enter_node(node.node_id());
        <ImportSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_default_specifier(&mut self, node: ImportDefaultSpecifier) {
        self.enter_node(node.node_id());
        <ImportDefaultSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_namespace_specifier(&mut self, node: ImportNamespaceSpecifier) {
        self.enter_node(node.node_id());
        <ImportNamespaceSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_attribute(&mut self, node: ImportAttribute) {
        self.enter_node(node.node_id());
        <ImportAttribute as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        self.enter_node(node.node_id());
        <ExportNamedDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        self.enter_node(node.node_id());
        <ExportDefaultDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        self.enter_node(node.node_id());
        <ExportAllDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        self.enter_node(node.node_id());
        <ExportSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_hashbang(&mut self, node: Hashbang) {
        self.enter_node(node.node_id());
        <Hashbang as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type(&mut self, node: TsType) {
        <TsType as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_type_name(&mut self, node: TsTypeName) {
        <TsTypeName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_qualified_name_left(&mut self, node: TsQualifiedNameLeft) {
        <TsQualifiedNameLeft as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_import_type_qualifier(&mut self, node: TsImportTypeQualifier) {
        <TsImportTypeQualifier as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_type_query_expr_name(&mut self, node: TsTypeQueryExprName) {
        <TsTypeQueryExprName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_literal(&mut self, node: TsLiteral) {
        <TsLiteral as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_tuple_element(&mut self, node: TsTupleElement) {
        <TsTupleElement as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_type_element(&mut self, node: TsTypeElement) {
        <TsTypeElement as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_type_predicate_name(&mut self, node: TsTypePredicateName) {
        <TsTypePredicateName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_enum_member_name(&mut self, node: TsEnumMemberName) {
        <TsEnumMemberName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_module_name(&mut self, node: TsModuleName) {
        <TsModuleName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_module_body(&mut self, node: TsModuleBody) {
        <TsModuleBody as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_module_reference(&mut self, node: TsModuleReference) {
        <TsModuleReference as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_type_annotation(&mut self, node: TsTypeAnnotation) {
        self.enter_node(node.node_id());
        <TsTypeAnnotation as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_any_keyword(&mut self, node: TsAnyKeyword) {
        self.enter_node(node.node_id());
        <TsAnyKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_unknown_keyword(&mut self, node: TsUnknownKeyword) {
        self.enter_node(node.node_id());
        <TsUnknownKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_never_keyword(&mut self, node: TsNeverKeyword) {
        self.enter_node(node.node_id());
        <TsNeverKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_void_keyword(&mut self, node: TsVoidKeyword) {
        self.enter_node(node.node_id());
        <TsVoidKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_null_keyword(&mut self, node: TsNullKeyword) {
        self.enter_node(node.node_id());
        <TsNullKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_undefined_keyword(&mut self, node: TsUndefinedKeyword) {
        self.enter_node(node.node_id());
        <TsUndefinedKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_string_keyword(&mut self, node: TsStringKeyword) {
        self.enter_node(node.node_id());
        <TsStringKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_number_keyword(&mut self, node: TsNumberKeyword) {
        self.enter_node(node.node_id());
        <TsNumberKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_big_int_keyword(&mut self, node: TsBigIntKeyword) {
        self.enter_node(node.node_id());
        <TsBigIntKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_boolean_keyword(&mut self, node: TsBooleanKeyword) {
        self.enter_node(node.node_id());
        <TsBooleanKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_symbol_keyword(&mut self, node: TsSymbolKeyword) {
        self.enter_node(node.node_id());
        <TsSymbolKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_object_keyword(&mut self, node: TsObjectKeyword) {
        self.enter_node(node.node_id());
        <TsObjectKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_intrinsic_keyword(&mut self, node: TsIntrinsicKeyword) {
        self.enter_node(node.node_id());
        <TsIntrinsicKeyword as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_this_type(&mut self, node: TsThisType) {
        self.enter_node(node.node_id());
        <TsThisType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_reference(&mut self, node: TsTypeReference) {
        self.enter_node(node.node_id());
        <TsTypeReference as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_qualified_name(&mut self, node: TsQualifiedName) {
        self.enter_node(node.node_id());
        <TsQualifiedName as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_query(&mut self, node: TsTypeQuery) {
        self.enter_node(node.node_id());
        <TsTypeQuery as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_import_type(&mut self, node: TsImportType) {
        self.enter_node(node.node_id());
        <TsImportType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_parameter(&mut self, node: TsTypeParameter) {
        self.enter_node(node.node_id());
        <TsTypeParameter as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_parameter_declaration(&mut self, node: TsTypeParameterDeclaration) {
        self.enter_node(node.node_id());
        <TsTypeParameterDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_parameter_instantiation(&mut self, node: TsTypeParameterInstantiation) {
        self.enter_node(node.node_id());
        <TsTypeParameterInstantiation as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_literal_type(&mut self, node: TsLiteralType) {
        self.enter_node(node.node_id());
        <TsLiteralType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_template_literal_type(&mut self, node: TsTemplateLiteralType) {
        self.enter_node(node.node_id());
        <TsTemplateLiteralType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_array_type(&mut self, node: TsArrayType) {
        self.enter_node(node.node_id());
        <TsArrayType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_indexed_access_type(&mut self, node: TsIndexedAccessType) {
        self.enter_node(node.node_id());
        <TsIndexedAccessType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_tuple_type(&mut self, node: TsTupleType) {
        self.enter_node(node.node_id());
        <TsTupleType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_named_tuple_member(&mut self, node: TsNamedTupleMember) {
        self.enter_node(node.node_id());
        <TsNamedTupleMember as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_optional_type(&mut self, node: TsOptionalType) {
        self.enter_node(node.node_id());
        <TsOptionalType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_rest_type(&mut self, node: TsRestType) {
        self.enter_node(node.node_id());
        <TsRestType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_js_doc_nullable_type(&mut self, node: TsJsDocNullableType) {
        self.enter_node(node.node_id());
        <TsJsDocNullableType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_js_doc_non_nullable_type(&mut self, node: TsJsDocNonNullableType) {
        self.enter_node(node.node_id());
        <TsJsDocNonNullableType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_js_doc_unknown_type(&mut self, node: TsJsDocUnknownType) {
        self.enter_node(node.node_id());
        <TsJsDocUnknownType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_union_type(&mut self, node: TsUnionType) {
        self.enter_node(node.node_id());
        <TsUnionType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_intersection_type(&mut self, node: TsIntersectionType) {
        self.enter_node(node.node_id());
        <TsIntersectionType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_conditional_type(&mut self, node: TsConditionalType) {
        self.enter_node(node.node_id());
        <TsConditionalType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_infer_type(&mut self, node: TsInferType) {
        self.enter_node(node.node_id());
        <TsInferType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_operator(&mut self, node: TsTypeOperator) {
        self.enter_node(node.node_id());
        <TsTypeOperator as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_parenthesized_type(&mut self, node: TsParenthesizedType) {
        self.enter_node(node.node_id());
        <TsParenthesizedType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_function_type(&mut self, node: TsFunctionType) {
        self.enter_node(node.node_id());
        <TsFunctionType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_constructor_type(&mut self, node: TsConstructorType) {
        self.enter_node(node.node_id());
        <TsConstructorType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_predicate(&mut self, node: TsTypePredicate) {
        self.enter_node(node.node_id());
        <TsTypePredicate as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_literal(&mut self, node: TsTypeLiteral) {
        self.enter_node(node.node_id());
        <TsTypeLiteral as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_mapped_type(&mut self, node: TsMappedType) {
        self.enter_node(node.node_id());
        <TsMappedType as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_property_signature(&mut self, node: TsPropertySignature) {
        self.enter_node(node.node_id());
        <TsPropertySignature as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_method_signature(&mut self, node: TsMethodSignature) {
        self.enter_node(node.node_id());
        <TsMethodSignature as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_call_signature_declaration(&mut self, node: TsCallSignatureDeclaration) {
        self.enter_node(node.node_id());
        <TsCallSignatureDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_construct_signature_declaration(&mut self, node: TsConstructSignatureDeclaration) {
        self.enter_node(node.node_id());
        <TsConstructSignatureDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_index_signature(&mut self, node: TsIndexSignature) {
        self.enter_node(node.node_id());
        <TsIndexSignature as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_alias_declaration(&mut self, node: TsTypeAliasDeclaration) {
        self.enter_node(node.node_id());
        <TsTypeAliasDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_interface_declaration(&mut self, node: TsInterfaceDeclaration) {
        self.enter_node(node.node_id());
        <TsInterfaceDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_interface_body(&mut self, node: TsInterfaceBody) {
        self.enter_node(node.node_id());
        <TsInterfaceBody as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_interface_heritage(&mut self, node: TsInterfaceHeritage) {
        self.enter_node(node.node_id());
        <TsInterfaceHeritage as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_class_implements(&mut self, node: TsClassImplements) {
        self.enter_node(node.node_id());
        <TsClassImplements as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_enum_declaration(&mut self, node: TsEnumDeclaration) {
        self.enter_node(node.node_id());
        <TsEnumDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_enum_body(&mut self, node: TsEnumBody) {
        self.enter_node(node.node_id());
        <TsEnumBody as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_enum_member(&mut self, node: TsEnumMember) {
        self.enter_node(node.node_id());
        <TsEnumMember as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        self.enter_node(node.node_id());
        <TsModuleDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_module_block(&mut self, node: TsModuleBlock) {
        self.enter_node(node.node_id());
        <TsModuleBlock as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        self.enter_node(node.node_id());
        <TsGlobalDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_parameter_property(&mut self, node: TsParameterProperty) {
        self.enter_node(node.node_id());
        <TsParameterProperty as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_this_parameter(&mut self, node: TsThisParameter) {
        self.enter_node(node.node_id());
        <TsThisParameter as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_as_expression(&mut self, node: TsAsExpression) {
        self.enter_node(node.node_id());
        <TsAsExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_satisfies_expression(&mut self, node: TsSatisfiesExpression) {
        self.enter_node(node.node_id());
        <TsSatisfiesExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_assertion(&mut self, node: TsTypeAssertion) {
        self.enter_node(node.node_id());
        <TsTypeAssertion as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_non_null_expression(&mut self, node: TsNonNullExpression) {
        self.enter_node(node.node_id());
        <TsNonNullExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_instantiation_expression(&mut self, node: TsInstantiationExpression) {
        self.enter_node(node.node_id());
        <TsInstantiationExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_export_assignment(&mut self, node: TsExportAssignment) {
        self.enter_node(node.node_id());
        <TsExportAssignment as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_namespace_export_declaration(&mut self, node: TsNamespaceExportDeclaration) {
        self.enter_node(node.node_id());
        <TsNamespaceExportDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        self.enter_node(node.node_id());
        <TsImportEqualsDeclaration as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_external_module_reference(&mut self, node: TsExternalModuleReference) {
        self.enter_node(node.node_id());
        <TsExternalModuleReference as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_element_name(&mut self, node: JsxElementName) {
        <JsxElementName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_member_expression_object(&mut self, node: JsxMemberExpressionObject) {
        <JsxMemberExpressionObject as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_attribute_name(&mut self, node: JsxAttributeName) {
        <JsxAttributeName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_attribute_item(&mut self, node: JsxAttributeItem) {
        <JsxAttributeItem as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_attribute_value(&mut self, node: JsxAttributeValue) {
        <JsxAttributeValue as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_expression(&mut self, node: JsxExpression) {
        <JsxExpression as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_child(&mut self, node: JsxChild) {
        <JsxChild as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_element(&mut self, node: JsxElement) {
        self.enter_node(node.node_id());
        <JsxElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_opening_element(&mut self, node: JsxOpeningElement) {
        self.enter_node(node.node_id());
        <JsxOpeningElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_closing_element(&mut self, node: JsxClosingElement) {
        self.enter_node(node.node_id());
        <JsxClosingElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_fragment(&mut self, node: JsxFragment) {
        self.enter_node(node.node_id());
        <JsxFragment as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_opening_fragment(&mut self, node: JsxOpeningFragment) {
        self.enter_node(node.node_id());
        <JsxOpeningFragment as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_closing_fragment(&mut self, node: JsxClosingFragment) {
        self.enter_node(node.node_id());
        <JsxClosingFragment as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_identifier(&mut self, node: JsxIdentifier) {
        self.enter_node(node.node_id());
        <JsxIdentifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_namespaced_name(&mut self, node: JsxNamespacedName) {
        self.enter_node(node.node_id());
        <JsxNamespacedName as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_member_expression(&mut self, node: JsxMemberExpression) {
        self.enter_node(node.node_id());
        <JsxMemberExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_attribute(&mut self, node: JsxAttribute) {
        self.enter_node(node.node_id());
        <JsxAttribute as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_spread_attribute(&mut self, node: JsxSpreadAttribute) {
        self.enter_node(node.node_id());
        <JsxSpreadAttribute as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_expression_container(&mut self, node: JsxExpressionContainer) {
        self.enter_node(node.node_id());
        <JsxExpressionContainer as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_empty_expression(&mut self, node: JsxEmptyExpression) {
        self.enter_node(node.node_id());
        <JsxEmptyExpression as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_text(&mut self, node: JsxText) {
        self.enter_node(node.node_id());
        <JsxText as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_spread_child(&mut self, node: JsxSpreadChild) {
        self.enter_node(node.node_id());
        <JsxSpreadChild as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_statement_list_items(&mut self, node: TypedSubRange<StatementListItem>) {
        <TypedSubRange<StatementListItem> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_hashbang(&mut self, node: Option<Hashbang>) {
        <Option<Hashbang> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_template_elements(&mut self, node: TypedSubRange<TemplateElement>) {
        <TypedSubRange<TemplateElement> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_exprs(&mut self, node: TypedSubRange<Expr>) {
        <TypedSubRange<Expr> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_decorators(&mut self, node: TypedSubRange<Decorator>) {
        <TypedSubRange<Decorator> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_ts_type_annotation(&mut self, node: Option<TsTypeAnnotation>) {
        <Option<TsTypeAnnotation> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_binding_pattern(&mut self, node: Option<BindingPattern>) {
        <Option<BindingPattern> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_vec_binding_patterns(&mut self, node: TypedSubRange<Option<BindingPattern>>) {
        <TypedSubRange<Option<BindingPattern>> as VisitWith<'a, Self>>::visit_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_opt_binding_rest_element(&mut self, node: Option<BindingRestElement>) {
        <Option<BindingRestElement> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_binding_propertys(&mut self, node: TypedSubRange<BindingProperty>) {
        <TypedSubRange<BindingProperty> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_variable_declarators(&mut self, node: TypedSubRange<VariableDeclarator>) {
        <TypedSubRange<VariableDeclarator> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_expr(&mut self, node: Option<Expr>) {
        <Option<Expr> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_stmt(&mut self, node: Option<Stmt>) {
        <Option<Stmt> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_switch_cases(&mut self, node: TypedSubRange<SwitchCase>) {
        <TypedSubRange<SwitchCase> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_stmts(&mut self, node: TypedSubRange<Stmt>) {
        <TypedSubRange<Stmt> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_for_statement_init(&mut self, node: Option<ForStatementInit>) {
        <Option<ForStatementInit> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_catch_clause(&mut self, node: Option<CatchClause>) {
        <Option<CatchClause> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_block_statement(&mut self, node: Option<BlockStatement>) {
        <Option<BlockStatement> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_label_identifier(&mut self, node: Option<LabelIdentifier>) {
        <Option<LabelIdentifier> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_binding_identifier(&mut self, node: Option<BindingIdentifier>) {
        <Option<BindingIdentifier> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_ts_type_parameter_declaration(
        &mut self,
        node: Option<TsTypeParameterDeclaration>,
    ) {
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, Self>>::visit_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_opt_ts_type_parameter_instantiation(
        &mut self,
        node: Option<TsTypeParameterInstantiation>,
    ) {
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, Self>>::visit_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_ts_class_implementss(&mut self, node: TypedSubRange<TsClassImplements>) {
        <TypedSubRange<TsClassImplements> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class_elements(&mut self, node: TypedSubRange<ClassElement>) {
        <TypedSubRange<ClassElement> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_function_body(&mut self, node: Option<FunctionBody>) {
        <Option<FunctionBody> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_formal_parameter_items(&mut self, node: TypedSubRange<FormalParameterItem>) {
        <TypedSubRange<FormalParameterItem> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_argument(&mut self, node: Option<Argument>) {
        <Option<Argument> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_vec_arguments(&mut self, node: TypedSubRange<Option<Argument>>) {
        <TypedSubRange<Option<Argument>> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_object_property_kinds(&mut self, node: TypedSubRange<ObjectPropertyKind>) {
        <TypedSubRange<ObjectPropertyKind> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_arguments(&mut self, node: TypedSubRange<Argument>) {
        <TypedSubRange<Argument> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_declaration_specifiers(
        &mut self,
        node: TypedSubRange<ImportDeclarationSpecifier>,
    ) {
        <TypedSubRange<ImportDeclarationSpecifier> as VisitWith<'a, Self>>::visit_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_import_attributes(&mut self, node: TypedSubRange<ImportAttribute>) {
        <TypedSubRange<ImportAttribute> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_decl(&mut self, node: Option<Decl>) {
        <Option<Decl> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_export_specifiers(&mut self, node: TypedSubRange<ExportSpecifier>) {
        <TypedSubRange<ExportSpecifier> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_string_literal(&mut self, node: Option<StringLiteral>) {
        <Option<StringLiteral> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_module_export_name(&mut self, node: Option<ModuleExportName>) {
        <Option<ModuleExportName> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_ts_import_type_qualifier(&mut self, node: Option<TsImportTypeQualifier>) {
        <Option<TsImportTypeQualifier> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_ts_type(&mut self, node: Option<TsType>) {
        <Option<TsType> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_ts_type_parameters(&mut self, node: TypedSubRange<TsTypeParameter>) {
        <TypedSubRange<TsTypeParameter> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_types(&mut self, node: TypedSubRange<TsType>) {
        <TypedSubRange<TsType> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_tuple_elements(&mut self, node: TypedSubRange<TsTupleElement>) {
        <TypedSubRange<TsTupleElement> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_type_elements(&mut self, node: TypedSubRange<TsTypeElement>) {
        <TypedSubRange<TsTypeElement> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_binding_identifiers(&mut self, node: TypedSubRange<BindingIdentifier>) {
        <TypedSubRange<BindingIdentifier> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_interface_heritages(&mut self, node: TypedSubRange<TsInterfaceHeritage>) {
        <TypedSubRange<TsInterfaceHeritage> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_ts_enum_members(&mut self, node: TypedSubRange<TsEnumMember>) {
        <TypedSubRange<TsEnumMember> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_ts_module_body(&mut self, node: Option<TsModuleBody>) {
        <Option<TsModuleBody> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_jsx_childs(&mut self, node: TypedSubRange<JsxChild>) {
        <TypedSubRange<JsxChild> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_jsx_closing_element(&mut self, node: Option<JsxClosingElement>) {
        <Option<JsxClosingElement> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_jsx_attribute_items(&mut self, node: TypedSubRange<JsxAttributeItem>) {
        <TypedSubRange<JsxAttributeItem> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_jsx_attribute_value(&mut self, node: Option<JsxAttributeValue>) {
        <Option<JsxAttributeValue> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
}
pub trait VisitWith<'a, V: ?Sized + Visit<'a>> {
    fn visit_with(self, visitor: &mut V);
    fn visit_children_with(self, visitor: &mut V);
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Program {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_program(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.body(visitor.ast());
        let field_value_2 = self.hashbang(visitor.ast());
        <TypedSubRange<StatementListItem> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Hashbang> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Stmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().stmt_data(self) {
            StmtData::BlockStatement(it) => {
                <BlockStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::BreakStatement(it) => {
                <BreakStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ContinueStatement(it) => {
                <ContinueStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::DebuggerStatement(it) => {
                <DebuggerStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::DoWhileStatement(it) => {
                <DoWhileStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::EmptyStatement(it) => {
                <EmptyStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ExpressionStatement(it) => {
                <ExpressionStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ForInStatement(it) => {
                <ForInStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ForOfStatement(it) => {
                <ForOfStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ForStatement(it) => {
                <ForStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::IfStatement(it) => <IfStatement as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::LabeledStatement(it) => {
                <LabeledStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ReturnStatement(it) => {
                <ReturnStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::SwitchStatement(it) => {
                <SwitchStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ThrowStatement(it) => {
                <ThrowStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::TryStatement(it) => {
                <TryStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::WhileStatement(it) => {
                <WhileStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::WithStatement(it) => {
                <WithStatement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::Declaration(it) => <Decl as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::ImportDeclaration(it) => {
                <ImportDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ExportNamedDeclaration(it) => {
                <ExportNamedDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ExportDefaultDeclaration(it) => {
                <ExportDefaultDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::ExportAllDeclaration(it) => {
                <ExportAllDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::TsExportAssignment(it) => {
                <TsExportAssignment as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::TsNamespaceExportDeclaration(it) => {
                <TsNamespaceExportDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            StmtData::TsImportEqualsDeclaration(it) => {
                <TsImportEqualsDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for StatementListItem {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_statement_list_item(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().statement_list_item_data(self) {
            StatementListItemData::Stmt(it) => <Stmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StatementListItemData::Directive(it) => {
                <Directive as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Decl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().decl_data(self) {
            DeclData::VariableDeclaration(it) => {
                <VariableDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            DeclData::Function(it) => <Function as VisitWith<'a, V>>::visit_with(it, visitor),
            DeclData::Class(it) => <Class as VisitWith<'a, V>>::visit_with(it, visitor),
            DeclData::TsTypeAliasDeclaration(it) => {
                <TsTypeAliasDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            DeclData::TsInterfaceDeclaration(it) => {
                <TsInterfaceDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            DeclData::TsEnumDeclaration(it) => {
                <TsEnumDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            DeclData::TsModuleDeclaration(it) => {
                <TsModuleDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            DeclData::TsGlobalDeclaration(it) => {
                <TsGlobalDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Expr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().expr_data(self) {
            ExprData::IdentifierReference(it) => {
                <IdentifierReference as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::ThisExpression(it) => {
                <ThisExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::Super(it) => <Super as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::MetaProperty(it) => {
                <MetaProperty as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::StringLiteral(it) => {
                <StringLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::NumericLiteral(it) => {
                <NumericLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::BigIntLiteral(it) => {
                <BigIntLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::BooleanLiteral(it) => {
                <BooleanLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::NullLiteral(it) => <NullLiteral as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::RegExpLiteral(it) => {
                <RegExpLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::TemplateLiteral(it) => {
                <TemplateLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::BinaryExpression(it) => {
                <BinaryExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::LogicalExpression(it) => {
                <LogicalExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::UnaryExpression(it) => {
                <UnaryExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::UpdateExpression(it) => {
                <UpdateExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::AssignmentExpression(it) => {
                <AssignmentExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::ConditionalExpression(it) => {
                <ConditionalExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::SequenceExpression(it) => {
                <SequenceExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::MemberExpression(it) => {
                <MemberExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::CallExpression(it) => {
                <CallExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::ChainExpression(it) => {
                <ChainExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::NewExpression(it) => {
                <NewExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::TaggedTemplateExpression(it) => {
                <TaggedTemplateExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::ArrowFunctionExpression(it) => {
                <ArrowFunctionExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::ArrayExpression(it) => {
                <ArrayExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::ObjectExpression(it) => {
                <ObjectExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::ParenthesizedExpression(it) => {
                <ParenthesizedExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::Function(it) => <Function as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Class(it) => <Class as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::ImportExpression(it) => {
                <ImportExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::AwaitExpression(it) => {
                <AwaitExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::YieldExpression(it) => {
                <YieldExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::TsAsExpression(it) => {
                <TsAsExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::TsSatisfiesExpression(it) => {
                <TsSatisfiesExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::TsTypeAssertion(it) => {
                <TsTypeAssertion as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::TsNonNullExpression(it) => {
                <TsNonNullExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::TsInstantiationExpression(it) => {
                <TsInstantiationExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::JsxElement(it) => <JsxElement as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::JsxFragment(it) => <JsxFragment as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BindingPattern {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_pattern(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().binding_pattern_data(self) {
            BindingPatternData::BindingIdentifier(it) => {
                <BindingIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            BindingPatternData::AssignmentPattern(it) => {
                <AssignmentPattern as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            BindingPatternData::BindingRestElement(it) => {
                <BindingRestElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            BindingPatternData::ArrayPattern(it) => {
                <ArrayPattern as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            BindingPatternData::ObjectPattern(it) => {
                <ObjectPattern as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignmentTarget {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assignment_target(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().assignment_target_data(self) {
            AssignmentTargetData::SimpleAssignmentTarget(it) => {
                <SimpleAssignmentTarget as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            AssignmentTargetData::ArrayAssignmentTarget(it) => {
                <ArrayPattern as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            AssignmentTargetData::ObjectAssignmentTarget(it) => {
                <ObjectPattern as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SimpleAssignmentTarget {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_simple_assignment_target(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().simple_assignment_target_data(self) {
            SimpleAssignmentTargetData::IdentifierReference(it) => {
                <IdentifierReference as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignmentTargetData::MemberExpression(it) => {
                <MemberExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsAsExpression(it) => {
                <TsAsExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(it) => {
                <TsSatisfiesExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsTypeAssertion(it) => {
                <TsTypeAssertion as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsNonNullExpression(it) => {
                <TsNonNullExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PropertyKey {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_property_key(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().property_key_data(self) {
            PropertyKeyData::IdentifierName(it) => {
                <IdentifierName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            PropertyKeyData::PrivateIdentifier(it) => {
                <PrivateIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            PropertyKeyData::StringLiteral(it) => {
                <StringLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            PropertyKeyData::NumericLiteral(it) => {
                <NumericLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            PropertyKeyData::BigIntLiteral(it) => {
                <BigIntLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            PropertyKeyData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ModuleExportName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().module_export_name_data(self) {
            ModuleExportNameData::IdentifierName(it) => {
                <IdentifierName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ModuleExportNameData::StringLiteral(it) => {
                <StringLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Argument {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_argument(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().argument_data(self) {
            ArgumentData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
            ArgumentData::SpreadElement(it) => {
                <SpreadElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectPropertyKind {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_property_kind(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().object_property_kind_data(self) {
            ObjectPropertyKindData::ObjectProperty(it) => {
                <ObjectProperty as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ObjectPropertyKindData::SpreadElement(it) => {
                <SpreadElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForStatementInit {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_statement_init(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().for_statement_init_data(self) {
            ForStatementInitData::VariableDeclaration(it) => {
                <VariableDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ForStatementInitData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForStatementLeft {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_statement_left(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().for_statement_left_data(self) {
            ForStatementLeftData::VariableDeclaration(it) => {
                <VariableDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ForStatementLeftData::AssignmentTarget(it) => {
                <AssignmentTarget as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrowFunctionBody {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_arrow_function_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().arrow_function_body_data(self) {
            ArrowFunctionBodyData::FunctionBody(it) => {
                <FunctionBody as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ArrowFunctionBodyData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FormalParameterItem {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_formal_parameter_item(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().formal_parameter_item_data(self) {
            FormalParameterItemData::FormalParameter(it) => {
                <FormalParameter as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            FormalParameterItemData::TsParameterProperty(it) => {
                <TsParameterProperty as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FormalParameterPattern {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_formal_parameter_pattern(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().formal_parameter_pattern_data(self) {
            FormalParameterPatternData::BindingPattern(it) => {
                <BindingPattern as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            FormalParameterPatternData::TsThisParameter(it) => {
                <TsThisParameter as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().class_element_data(self) {
            ClassElementData::MethodDefinition(it) => {
                <MethodDefinition as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassElementData::PropertyDefinition(it) => {
                <PropertyDefinition as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassElementData::StaticBlock(it) => {
                <StaticBlock as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassElementData::TsIndexSignature(it) => {
                <TsIndexSignature as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportDeclarationSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_declaration_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().import_declaration_specifier_data(self) {
            ImportDeclarationSpecifierData::ImportSpecifier(it) => {
                <ImportSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ImportDeclarationSpecifierData::ImportDefaultSpecifier(it) => {
                <ImportDefaultSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ImportDeclarationSpecifierData::ImportNamespaceSpecifier(it) => {
                <ImportNamespaceSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultDeclarationKind {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_declaration_kind(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().export_default_declaration_kind_data(self) {
            ExportDefaultDeclarationKindData::Expr(it) => {
                <Expr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExportDefaultDeclarationKindData::Function(it) => {
                <Function as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExportDefaultDeclarationKindData::Class(it) => {
                <Class as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(it) => {
                <TsInterfaceDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Super {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_super(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ThisExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_this_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for StringLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_string_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for NumericLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_numeric_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BigIntLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_big_int_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BooleanLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_boolean_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for NullLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_null_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for RegExpLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_reg_exp_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TemplateLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_template_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.quasis(visitor.ast());
        let field_value_1 = self.expressions(visitor.ast());
        <TypedSubRange<TemplateElement> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TemplateElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_template_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for IdentifierReference {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_identifier_reference(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PrivateIdentifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_private_identifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BindingIdentifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_identifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.decorators(visitor.ast());
        let field_value_2 = self.type_annotation(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for IdentifierName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_identifier_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for LabelIdentifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_label_identifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignmentPattern {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assignment_pattern(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        let field_value_2 = self.decorators(visitor.ast());
        let field_value_3 = self.type_annotation(visitor.ast());
        <BindingPattern as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BindingRestElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_rest_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        let field_value_1 = self.decorators(visitor.ast());
        let field_value_2 = self.type_annotation(visitor.ast());
        <BindingPattern as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrayPattern {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_array_pattern(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.elements(visitor.ast());
        let field_value_1 = self.rest(visitor.ast());
        let field_value_2 = self.decorators(visitor.ast());
        let field_value_3 = self.type_annotation(visitor.ast());
        <TypedSubRange<Option<BindingPattern>> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
        <Option<BindingRestElement> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectPattern {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_pattern(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.properties(visitor.ast());
        let field_value_1 = self.rest(visitor.ast());
        let field_value_2 = self.decorators(visitor.ast());
        let field_value_3 = self.type_annotation(visitor.ast());
        <TypedSubRange<BindingProperty> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<BindingRestElement> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BindingProperty {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_property(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <BindingPattern as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BinaryExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binary_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for LogicalExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_logical_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ConditionalExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_conditional_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.consequent(visitor.ast());
        let field_value_2 = self.alternate(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UnaryExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_unary_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UpdateExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_update_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <SimpleAssignmentTarget as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignmentExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assignment_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        <AssignmentTarget as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VariableDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_variable_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.declarators(visitor.ast());
        <TypedSubRange<VariableDeclarator> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VariableDeclarator {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_variable_declarator(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.init(visitor.ast());
        <BindingPattern as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for IfStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_if_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.consequent(visitor.ast());
        let field_value_2 = self.alternate(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Stmt> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SwitchStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.discriminant(visitor.ast());
        let field_value_1 = self.cases(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<SwitchCase> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SwitchCase {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_case(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.consequent(visitor.ast());
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.init(visitor.ast());
        let field_value_1 = self.test(visitor.ast());
        let field_value_2 = self.update(visitor.ast());
        let field_value_3 = self.body(visitor.ast());
        <Option<ForStatementInit> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForInStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_in_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        let field_value_2 = self.body(visitor.ast());
        <ForStatementLeft as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForOfStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_of_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        let field_value_2 = self.body(visitor.ast());
        <ForStatementLeft as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TryStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_try_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.block(visitor.ast());
        let field_value_1 = self.handler(visitor.ast());
        let field_value_2 = self.finalizer(visitor.ast());
        <BlockStatement as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<CatchClause> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<BlockStatement> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CatchClause {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.param(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Option<BindingPattern> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <BlockStatement as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BreakStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_break_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.label(visitor.ast());
        <Option<LabelIdentifier> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ContinueStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_continue_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.label(visitor.ast());
        <Option<LabelIdentifier> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for LabeledStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_labeled_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.label(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <LabelIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExpressionStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expression_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ReturnStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_return_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ThrowStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_throw_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DebuggerStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_debugger_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for EmptyStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_empty_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Directive {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_directive(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <StringLiteral as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Decorator {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decorator(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Class {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.decorators(visitor.ast());
        let field_value_2 = self.id(visitor.ast());
        let field_value_3 = self.super_class(visitor.ast());
        let field_value_4 = self.body(visitor.ast());
        let field_value_5 = self.type_parameters(visitor.ast());
        let field_value_6 = self.super_type_arguments(visitor.ast());
        let field_value_7 = self.implements(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<BindingIdentifier> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
        <ClassBody as VisitWith<'a, V>>::visit_with(field_value_4, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_5,
            visitor,
        );
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_6,
            visitor,
        );
        <TypedSubRange<TsClassImplements> as VisitWith<'a, V>>::visit_with(field_value_7, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassBody {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<ClassElement> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MethodDefinition {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_method_definition(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decorators(visitor.ast());
        let field_value_1 = self.key(visitor.ast());
        let field_value_2 = self.value(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PropertyDefinition {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_property_definition(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decorators(visitor.ast());
        let field_value_1 = self.key(visitor.ast());
        let field_value_2 = self.value(visitor.ast());
        let field_value_6 = self.type_annotation(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_6, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for StaticBlock {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_static_block(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Function {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_function(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.id(visitor.ast());
        let field_value_5 = self.params(visitor.ast());
        let field_value_6 = self.body(visitor.ast());
        let field_value_7 = self.type_parameters(visitor.ast());
        let field_value_8 = self.return_type(visitor.ast());
        <Option<BindingIdentifier> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_5, visitor);
        <Option<FunctionBody> as VisitWith<'a, V>>::visit_with(field_value_6, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_7,
            visitor,
        );
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_8, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FunctionBody {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_function_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<StatementListItem> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BlockStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_block_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for WhileStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_while_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DoWhileStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_do_while_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        let field_value_1 = self.test(visitor.ast());
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for WithStatement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_with_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.object(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FormalParameters {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_formal_parameters(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.items(visitor.ast());
        let field_value_1 = self.rest(visitor.ast());
        <TypedSubRange<FormalParameterItem> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
        <Option<BindingRestElement> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FormalParameter {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_formal_parameter(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.pattern(visitor.ast());
        <FormalParameterPattern as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrayExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_array_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.elements(visitor.ast());
        <TypedSubRange<Option<Argument>> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.properties(visitor.ast());
        <TypedSubRange<ObjectPropertyKind> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SpreadElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_spread_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectProperty {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_property(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ParenthesizedExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_parenthesized_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrowFunctionExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_arrow_function_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_2 = self.params(visitor.ast());
        let field_value_3 = self.body(visitor.ast());
        let field_value_4 = self.type_parameters(visitor.ast());
        let field_value_5 = self.return_type(visitor.ast());
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <ArrowFunctionBody as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_4,
            visitor,
        );
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_5, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SequenceExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_sequence_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expressions(visitor.ast());
        <TypedSubRange<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MemberExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_member_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.object(visitor.ast());
        let field_value_1 = self.property(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CallExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_call_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.callee(visitor.ast());
        let field_value_1 = self.arguments(visitor.ast());
        let field_value_3 = self.type_arguments(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Argument> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_3,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ChainExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_chain_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TaggedTemplateExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tagged_template_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.tag(visitor.ast());
        let field_value_1 = self.quasi(visitor.ast());
        let field_value_2 = self.type_arguments(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TemplateLiteral as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_2,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for NewExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_new_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.callee(visitor.ast());
        let field_value_1 = self.arguments(visitor.ast());
        let field_value_2 = self.type_arguments(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Argument> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_2,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AwaitExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_await_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for YieldExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_yield_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MetaProperty {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_meta_property(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.meta(visitor.ast());
        let field_value_1 = self.property(visitor.ast());
        <IdentifierName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <IdentifierName as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.source(visitor.ast());
        let field_value_1 = self.options(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.specifiers(visitor.ast());
        let field_value_1 = self.source(visitor.ast());
        let field_value_2 = self.attributes(visitor.ast());
        <TypedSubRange<ImportDeclarationSpecifier> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
        <StringLiteral as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<ImportAttribute> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.imported(visitor.ast());
        let field_value_1 = self.local(visitor.ast());
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportDefaultSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.local(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportNamespaceSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_namespace_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.local(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportAttribute {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_attribute(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <StringLiteral as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportNamedDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_named_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.declaration(visitor.ast());
        let field_value_1 = self.specifiers(visitor.ast());
        let field_value_2 = self.source(visitor.ast());
        let field_value_3 = self.attributes(visitor.ast());
        <Option<Decl> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<ExportSpecifier> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<StringLiteral> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <TypedSubRange<ImportAttribute> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.declaration(visitor.ast());
        <ExportDefaultDeclarationKind as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportAllDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_all_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.exported(visitor.ast());
        let field_value_1 = self.source(visitor.ast());
        let field_value_2 = self.attributes(visitor.ast());
        <Option<ModuleExportName> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <StringLiteral as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<ImportAttribute> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.local(visitor.ast());
        let field_value_1 = self.exported(visitor.ast());
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Hashbang {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_hashbang(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_type_data(self) {
            TsTypeData::TsAnyKeyword(it) => {
                <TsAnyKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsUnknownKeyword(it) => {
                <TsUnknownKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsNeverKeyword(it) => {
                <TsNeverKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsVoidKeyword(it) => {
                <TsVoidKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsNullKeyword(it) => {
                <TsNullKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsUndefinedKeyword(it) => {
                <TsUndefinedKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsStringKeyword(it) => {
                <TsStringKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsNumberKeyword(it) => {
                <TsNumberKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsBigIntKeyword(it) => {
                <TsBigIntKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsBooleanKeyword(it) => {
                <TsBooleanKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsSymbolKeyword(it) => {
                <TsSymbolKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsObjectKeyword(it) => {
                <TsObjectKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsIntrinsicKeyword(it) => {
                <TsIntrinsicKeyword as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsThisType(it) => <TsThisType as VisitWith<'a, V>>::visit_with(it, visitor),
            TsTypeData::TsTypeReference(it) => {
                <TsTypeReference as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsTypeQuery(it) => {
                <TsTypeQuery as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsImportType(it) => {
                <TsImportType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsLiteralType(it) => {
                <TsLiteralType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsTemplateLiteralType(it) => {
                <TsTemplateLiteralType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsArrayType(it) => {
                <TsArrayType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsIndexedAccessType(it) => {
                <TsIndexedAccessType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsTupleType(it) => {
                <TsTupleType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsOptionalType(it) => {
                <TsOptionalType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsRestType(it) => <TsRestType as VisitWith<'a, V>>::visit_with(it, visitor),
            TsTypeData::TsJsDocNullableType(it) => {
                <TsJsDocNullableType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsJsDocNonNullableType(it) => {
                <TsJsDocNonNullableType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsJsDocUnknownType(it) => {
                <TsJsDocUnknownType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsUnionType(it) => {
                <TsUnionType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsIntersectionType(it) => {
                <TsIntersectionType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsConditionalType(it) => {
                <TsConditionalType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsInferType(it) => {
                <TsInferType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsTypeOperator(it) => {
                <TsTypeOperator as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsParenthesizedType(it) => {
                <TsParenthesizedType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsFunctionType(it) => {
                <TsFunctionType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsConstructorType(it) => {
                <TsConstructorType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsTypePredicate(it) => {
                <TsTypePredicate as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsTypeLiteral(it) => {
                <TsTypeLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeData::TsMappedType(it) => {
                <TsMappedType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_type_name_data(self) {
            TsTypeNameData::IdentifierReference(it) => {
                <IdentifierReference as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeNameData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsQualifiedNameLeft {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_qualified_name_left(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_qualified_name_left_data(self) {
            TsQualifiedNameLeftData::IdentifierReference(it) => {
                <IdentifierReference as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsQualifiedNameLeftData::IdentifierName(it) => {
                <IdentifierName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsQualifiedNameLeftData::BindingIdentifier(it) => {
                <BindingIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsQualifiedNameLeftData::ThisExpression(it) => {
                <ThisExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsQualifiedNameLeftData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsImportTypeQualifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_import_type_qualifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_import_type_qualifier_data(self) {
            TsImportTypeQualifierData::IdentifierName(it) => {
                <IdentifierName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsImportTypeQualifierData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeQueryExprName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_query_expr_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_type_query_expr_name_data(self) {
            TsTypeQueryExprNameData::TsTypeName(it) => {
                <TsTypeName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeQueryExprNameData::ThisExpression(it) => {
                <ThisExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeQueryExprNameData::TsImportType(it) => {
                <TsImportType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_literal_data(self) {
            TsLiteralData::StringLiteral(it) => {
                <StringLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsLiteralData::NumericLiteral(it) => {
                <NumericLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsLiteralData::BigIntLiteral(it) => {
                <BigIntLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsLiteralData::BooleanLiteral(it) => {
                <BooleanLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsLiteralData::TemplateLiteral(it) => {
                <TemplateLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsLiteralData::UnaryExpression(it) => {
                <UnaryExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTupleElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_tuple_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_tuple_element_data(self) {
            TsTupleElementData::TsType(it) => <TsType as VisitWith<'a, V>>::visit_with(it, visitor),
            TsTupleElementData::TsNamedTupleMember(it) => {
                <TsNamedTupleMember as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTupleElementData::TsOptionalType(it) => {
                <TsOptionalType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTupleElementData::TsRestType(it) => {
                <TsRestType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_type_element_data(self) {
            TsTypeElementData::TsPropertySignature(it) => {
                <TsPropertySignature as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeElementData::TsMethodSignature(it) => {
                <TsMethodSignature as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeElementData::TsCallSignatureDeclaration(it) => {
                <TsCallSignatureDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeElementData::TsConstructSignatureDeclaration(it) => {
                <TsConstructSignatureDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypeElementData::TsIndexSignature(it) => {
                <TsIndexSignature as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypePredicateName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_predicate_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_type_predicate_name_data(self) {
            TsTypePredicateNameData::IdentifierName(it) => {
                <IdentifierName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsTypePredicateNameData::TsThisType(it) => {
                <TsThisType as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsEnumMemberName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_enum_member_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_enum_member_name_data(self) {
            TsEnumMemberNameData::IdentifierName(it) => {
                <IdentifierName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsEnumMemberNameData::StringLiteral(it) => {
                <StringLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsEnumMemberNameData::TemplateLiteral(it) => {
                <TemplateLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsEnumMemberNameData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsModuleName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_module_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_module_name_data(self) {
            TsModuleNameData::BindingIdentifier(it) => {
                <BindingIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsModuleNameData::StringLiteral(it) => {
                <StringLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsModuleNameData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsModuleBody {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_module_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_module_body_data(self) {
            TsModuleBodyData::TsModuleBlock(it) => {
                <TsModuleBlock as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsModuleBodyData::TsModuleDeclaration(it) => {
                <TsModuleDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsModuleReference {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_module_reference(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().ts_module_reference_data(self) {
            TsModuleReferenceData::TsTypeName(it) => {
                <TsTypeName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            TsModuleReferenceData::TsExternalModuleReference(it) => {
                <TsExternalModuleReference as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeAnnotation {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_annotation(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsAnyKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_any_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsUnknownKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_unknown_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsNeverKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_never_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsVoidKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_void_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsNullKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_null_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsUndefinedKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_undefined_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsStringKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_string_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsNumberKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_number_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsBigIntKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_big_int_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsBooleanKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_boolean_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsSymbolKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_symbol_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsObjectKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_object_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsIntrinsicKeyword {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_intrinsic_keyword(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsThisType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_this_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeReference {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_reference(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_name(visitor.ast());
        let field_value_1 = self.type_arguments(visitor.ast());
        <TsTypeName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_1,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsQualifiedName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_qualified_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        <TsQualifiedNameLeft as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <IdentifierName as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeQuery {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_query(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr_name(visitor.ast());
        let field_value_1 = self.type_arguments(visitor.ast());
        <TsTypeQueryExprName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_1,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsImportType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_import_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.source(visitor.ast());
        let field_value_1 = self.options(visitor.ast());
        let field_value_2 = self.qualifier(visitor.ast());
        let field_value_3 = self.type_arguments(visitor.ast());
        <StringLiteral as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsImportTypeQualifier> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_3,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeParameter {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_parameter(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        let field_value_1 = self.constraint(visitor.ast());
        let field_value_2 = self.default(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsType> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsType> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeParameterDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_parameter_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.params(visitor.ast());
        <TypedSubRange<TsTypeParameter> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeParameterInstantiation {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_parameter_instantiation(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.params(visitor.ast());
        <TypedSubRange<TsType> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsLiteralType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_literal_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.literal(visitor.ast());
        <TsLiteral as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTemplateLiteralType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_template_literal_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.quasis(visitor.ast());
        let field_value_1 = self.types(visitor.ast());
        <TypedSubRange<TemplateElement> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<TsType> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsArrayType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_array_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.element_type(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsIndexedAccessType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_indexed_access_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.object_type(visitor.ast());
        let field_value_1 = self.index_type(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTupleType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_tuple_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.element_types(visitor.ast());
        <TypedSubRange<TsTupleElement> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsNamedTupleMember {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_named_tuple_member(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.label(visitor.ast());
        let field_value_1 = self.element_type(visitor.ast());
        <IdentifierName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsOptionalType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_optional_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsRestType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_rest_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsJsDocNullableType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_js_doc_nullable_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsJsDocNonNullableType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_js_doc_non_nullable_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsJsDocUnknownType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_js_doc_unknown_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsUnionType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_union_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.types(visitor.ast());
        <TypedSubRange<TsType> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsIntersectionType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_intersection_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.types(visitor.ast());
        <TypedSubRange<TsType> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsConditionalType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_conditional_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.check_type(visitor.ast());
        let field_value_1 = self.extends_type(visitor.ast());
        let field_value_2 = self.true_type(visitor.ast());
        let field_value_3 = self.false_type(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsInferType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_infer_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_parameter(visitor.ast());
        <TsTypeParameter as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeOperator {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_operator(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.type_annotation(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsParenthesizedType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_parenthesized_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsFunctionType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_function_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_parameters(visitor.ast());
        let field_value_1 = self.params(visitor.ast());
        let field_value_2 = self.return_type(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TsTypeAnnotation as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsConstructorType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_constructor_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_parameters(visitor.ast());
        let field_value_1 = self.params(visitor.ast());
        let field_value_2 = self.return_type(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TsTypeAnnotation as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypePredicate {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_predicate(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.parameter_name(visitor.ast());
        let field_value_1 = self.type_annotation(visitor.ast());
        <TsTypePredicateName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeLiteral {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.members(visitor.ast());
        <TypedSubRange<TsTypeElement> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsMappedType {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_mapped_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.constraint(visitor.ast());
        let field_value_2 = self.name_type(visitor.ast());
        let field_value_3 = self.type_annotation(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsType> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<TsType> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsPropertySignature {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_property_signature(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.type_annotation(visitor.ast());
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsMethodSignature {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_method_signature(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.type_parameters(visitor.ast());
        let field_value_2 = self.params(visitor.ast());
        let field_value_3 = self.return_type(visitor.ast());
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_1,
            visitor,
        );
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsCallSignatureDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_call_signature_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_parameters(visitor.ast());
        let field_value_1 = self.params(visitor.ast());
        let field_value_2 = self.return_type(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsConstructSignatureDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_construct_signature_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_parameters(visitor.ast());
        let field_value_1 = self.params(visitor.ast());
        let field_value_2 = self.return_type(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsIndexSignature {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_index_signature(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.parameters(visitor.ast());
        let field_value_1 = self.type_annotation(visitor.ast());
        <TypedSubRange<BindingIdentifier> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsTypeAnnotation as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeAliasDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_alias_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.type_parameters(visitor.ast());
        let field_value_2 = self.type_annotation(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_1,
            visitor,
        );
        <TsType as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsInterfaceDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_interface_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.type_parameters(visitor.ast());
        let field_value_2 = self.extends(visitor.ast());
        let field_value_3 = self.body(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_1,
            visitor,
        );
        <TypedSubRange<TsInterfaceHeritage> as VisitWith<'a, V>>::visit_with(
            field_value_2,
            visitor,
        );
        <TsInterfaceBody as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsInterfaceBody {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_interface_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<TsTypeElement> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsInterfaceHeritage {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_interface_heritage(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        let field_value_1 = self.type_arguments(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_1,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsClassImplements {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_class_implements(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        let field_value_1 = self.type_arguments(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_1,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsEnumDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_enum_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsEnumBody as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsEnumBody {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_enum_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.members(visitor.ast());
        <TypedSubRange<TsEnumMember> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsEnumMember {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_enum_member(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.initializer(visitor.ast());
        <TsEnumMemberName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsModuleDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_module_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <TsModuleName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<TsModuleBody> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsModuleBlock {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_module_block(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<StatementListItem> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsGlobalDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_global_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <IdentifierName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsModuleBlock as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsParameterProperty {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_parameter_property(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decorators(visitor.ast());
        let field_value_1 = self.parameter(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <BindingPattern as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsThisParameter {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_this_parameter(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsAsExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_as_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        let field_value_1 = self.type_annotation(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsSatisfiesExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_satisfies_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        let field_value_1 = self.type_annotation(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsType as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsTypeAssertion {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_assertion(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.type_annotation(visitor.ast());
        let field_value_1 = self.expression(visitor.ast());
        <TsType as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsNonNullExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_non_null_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsInstantiationExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_instantiation_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        let field_value_1 = self.type_arguments(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsTypeParameterInstantiation as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsExportAssignment {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_export_assignment(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsNamespaceExportDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_namespace_export_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        <IdentifierName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsImportEqualsDeclaration {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_import_equals_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_1 = self.module_reference(visitor.ast());
        <BindingIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TsModuleReference as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsExternalModuleReference {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_external_module_reference(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <StringLiteral as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxElementName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_element_name_data(self) {
            JsxElementNameData::JsxIdentifier(it) => {
                <JsxIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxElementNameData::JsxNamespacedName(it) => {
                <JsxNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxElementNameData::JsxMemberExpression(it) => {
                <JsxMemberExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxMemberExpressionObject {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_member_expression_object(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_member_expression_object_data(self) {
            JsxMemberExpressionObjectData::JsxIdentifier(it) => {
                <JsxIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(it) => {
                <JsxMemberExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxAttributeName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attribute_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_attribute_name_data(self) {
            JsxAttributeNameData::JsxIdentifier(it) => {
                <JsxIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxAttributeNameData::JsxNamespacedName(it) => {
                <JsxNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxAttributeItem {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attribute_item(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_attribute_item_data(self) {
            JsxAttributeItemData::JsxAttribute(it) => {
                <JsxAttribute as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxAttributeItemData::JsxSpreadAttribute(it) => {
                <JsxSpreadAttribute as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxAttributeValue {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attribute_value(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_attribute_value_data(self) {
            JsxAttributeValueData::StringLiteral(it) => {
                <StringLiteral as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxAttributeValueData::JsxExpressionContainer(it) => {
                <JsxExpressionContainer as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxAttributeValueData::JsxElement(it) => {
                <JsxElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxAttributeValueData::JsxFragment(it) => {
                <JsxFragment as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_expression_data(self) {
            JsxExpressionData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
            JsxExpressionData::JsxEmptyExpression(it) => {
                <JsxEmptyExpression as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxChild {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_child(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_child_data(self) {
            JsxChildData::JsxText(it) => <JsxText as VisitWith<'a, V>>::visit_with(it, visitor),
            JsxChildData::JsxExpressionContainer(it) => {
                <JsxExpressionContainer as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxChildData::JsxSpreadChild(it) => {
                <JsxSpreadChild as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxChildData::JsxElement(it) => {
                <JsxElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JsxChildData::JsxFragment(it) => {
                <JsxFragment as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.opening_element(visitor.ast());
        let field_value_1 = self.children(visitor.ast());
        let field_value_2 = self.closing_element(visitor.ast());
        <JsxOpeningElement as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<JsxChild> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<JsxClosingElement> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxOpeningElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_opening_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        let field_value_1 = self.attributes(visitor.ast());
        let field_value_3 = self.type_arguments(visitor.ast());
        <JsxElementName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<JsxAttributeItem> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<TsTypeParameterInstantiation> as VisitWith<'a, V>>::visit_with(
            field_value_3,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxClosingElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        <JsxElementName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxFragment {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.opening_fragment(visitor.ast());
        let field_value_1 = self.children(visitor.ast());
        let field_value_2 = self.closing_fragment(visitor.ast());
        <JsxOpeningFragment as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<JsxChild> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <JsxClosingFragment as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxOpeningFragment {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_opening_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxClosingFragment {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_closing_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxIdentifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_identifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxNamespacedName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_namespaced_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.namespace(visitor.ast());
        let field_value_1 = self.name(visitor.ast());
        <JsxIdentifier as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <JsxIdentifier as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxMemberExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_member_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.object(visitor.ast());
        let field_value_1 = self.property(visitor.ast());
        <JsxMemberExpressionObject as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <JsxIdentifier as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxAttribute {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attribute(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <JsxAttributeName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<JsxAttributeValue> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxSpreadAttribute {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_spread_attribute(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.argument(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxExpressionContainer {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_expression_container(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <JsxExpression as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxEmptyExpression {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_empty_expression(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxText {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_text(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JsxSpreadChild {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_spread_child(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expression(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<StatementListItem> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_statement_list_items(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Hashbang> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_hashbang(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TemplateElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_template_elements(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Expr> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_exprs(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Decorator> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decorators(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<TsTypeAnnotation> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ts_type_annotation(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<BindingPattern> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_binding_pattern(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Option<BindingPattern>> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_vec_binding_patterns(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<BindingRestElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_binding_rest_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<BindingProperty> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_propertys(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<VariableDeclarator> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_variable_declarators(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Expr> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Stmt> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<SwitchCase> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_cases(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Stmt> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_stmts(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<ForStatementInit> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_for_statement_init(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<CatchClause> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<BlockStatement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_block_statement(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<LabelIdentifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_label_identifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<BindingIdentifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_binding_identifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<TsTypeParameterDeclaration> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ts_type_parameter_declaration(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<TsTypeParameterInstantiation> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ts_type_parameter_instantiation(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TsClassImplements> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_class_implementss(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ClassElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_elements(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<FunctionBody> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_function_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<FormalParameterItem> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_formal_parameter_items(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Argument> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_argument(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Option<Argument>> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_vec_arguments(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ObjectPropertyKind> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_property_kinds(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Argument> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_arguments(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ImportDeclarationSpecifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_declaration_specifiers(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ImportAttribute> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_attributes(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Decl> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ExportSpecifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_specifiers(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<StringLiteral> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_string_literal(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<ModuleExportName> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<TsImportTypeQualifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ts_import_type_qualifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<TsType> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ts_type(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TsTypeParameter> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_parameters(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TsType> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_types(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TsTupleElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_tuple_elements(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TsTypeElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_type_elements(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<BindingIdentifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_identifiers(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TsInterfaceHeritage> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_interface_heritages(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TsEnumMember> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_enum_members(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<TsModuleBody> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ts_module_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<JsxChild> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_childs(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<JsxClosingElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<JsxAttributeItem> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attribute_items(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<JsxAttributeValue> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_jsx_attribute_value(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
pub trait VisitMut<'a> {
    fn ast(&mut self) -> &mut Ast<'a>;
    #[inline]
    fn visit_mut_program(&mut self, node: Program) {
        <Program as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_stmt(&mut self, node: Stmt) {
        <Stmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_statement_list_item(&mut self, node: StatementListItem) {
        <StatementListItem as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_decl(&mut self, node: Decl) {
        <Decl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_expr(&mut self, node: Expr) {
        <Expr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_binding_pattern(&mut self, node: BindingPattern) {
        <BindingPattern as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assignment_target(&mut self, node: AssignmentTarget) {
        <AssignmentTarget as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_simple_assignment_target(&mut self, node: SimpleAssignmentTarget) {
        <SimpleAssignmentTarget as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_property_key(&mut self, node: PropertyKey) {
        <PropertyKey as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module_export_name(&mut self, node: ModuleExportName) {
        <ModuleExportName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_argument(&mut self, node: Argument) {
        <Argument as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_object_property_kind(&mut self, node: ObjectPropertyKind) {
        <ObjectPropertyKind as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_statement_init(&mut self, node: ForStatementInit) {
        <ForStatementInit as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_statement_left(&mut self, node: ForStatementLeft) {
        <ForStatementLeft as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_arrow_function_body(&mut self, node: ArrowFunctionBody) {
        <ArrowFunctionBody as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_formal_parameter_item(&mut self, node: FormalParameterItem) {
        <FormalParameterItem as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_formal_parameter_pattern(&mut self, node: FormalParameterPattern) {
        <FormalParameterPattern as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_element(&mut self, node: ClassElement) {
        <ClassElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_declaration_specifier(&mut self, node: ImportDeclarationSpecifier) {
        <ImportDeclarationSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_declaration_kind(&mut self, node: ExportDefaultDeclarationKind) {
        <ExportDefaultDeclarationKind as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_super(&mut self, node: Super) {
        <Super as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_this_expression(&mut self, node: ThisExpression) {
        <ThisExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_string_literal(&mut self, node: StringLiteral) {
        <StringLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_numeric_literal(&mut self, node: NumericLiteral) {
        <NumericLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_big_int_literal(&mut self, node: BigIntLiteral) {
        <BigIntLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_boolean_literal(&mut self, node: BooleanLiteral) {
        <BooleanLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_null_literal(&mut self, node: NullLiteral) {
        <NullLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_reg_exp_literal(&mut self, node: RegExpLiteral) {
        <RegExpLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_template_literal(&mut self, node: TemplateLiteral) {
        <TemplateLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_template_element(&mut self, node: TemplateElement) {
        <TemplateElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_identifier_reference(&mut self, node: IdentifierReference) {
        <IdentifierReference as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_private_identifier(&mut self, node: PrivateIdentifier) {
        <PrivateIdentifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_binding_identifier(&mut self, node: BindingIdentifier) {
        <BindingIdentifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_identifier_name(&mut self, node: IdentifierName) {
        <IdentifierName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_label_identifier(&mut self, node: LabelIdentifier) {
        <LabelIdentifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assignment_pattern(&mut self, node: AssignmentPattern) {
        <AssignmentPattern as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_binding_rest_element(&mut self, node: BindingRestElement) {
        <BindingRestElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_array_pattern(&mut self, node: ArrayPattern) {
        <ArrayPattern as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_object_pattern(&mut self, node: ObjectPattern) {
        <ObjectPattern as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_binding_property(&mut self, node: BindingProperty) {
        <BindingProperty as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_binary_expression(&mut self, node: BinaryExpression) {
        <BinaryExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_logical_expression(&mut self, node: LogicalExpression) {
        <LogicalExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_conditional_expression(&mut self, node: ConditionalExpression) {
        <ConditionalExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_unary_expression(&mut self, node: UnaryExpression) {
        <UnaryExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_update_expression(&mut self, node: UpdateExpression) {
        <UpdateExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assignment_expression(&mut self, node: AssignmentExpression) {
        <AssignmentExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_variable_declaration(&mut self, node: VariableDeclaration) {
        <VariableDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_variable_declarator(&mut self, node: VariableDeclarator) {
        <VariableDeclarator as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_if_statement(&mut self, node: IfStatement) {
        <IfStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_statement(&mut self, node: SwitchStatement) {
        <SwitchStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_case(&mut self, node: SwitchCase) {
        <SwitchCase as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_statement(&mut self, node: ForStatement) {
        <ForStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_in_statement(&mut self, node: ForInStatement) {
        <ForInStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_of_statement(&mut self, node: ForOfStatement) {
        <ForOfStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_try_statement(&mut self, node: TryStatement) {
        <TryStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_catch_clause(&mut self, node: CatchClause) {
        <CatchClause as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_break_statement(&mut self, node: BreakStatement) {
        <BreakStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_continue_statement(&mut self, node: ContinueStatement) {
        <ContinueStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_labeled_statement(&mut self, node: LabeledStatement) {
        <LabeledStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_expression_statement(&mut self, node: ExpressionStatement) {
        <ExpressionStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_return_statement(&mut self, node: ReturnStatement) {
        <ReturnStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_throw_statement(&mut self, node: ThrowStatement) {
        <ThrowStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_debugger_statement(&mut self, node: DebuggerStatement) {
        <DebuggerStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_empty_statement(&mut self, node: EmptyStatement) {
        <EmptyStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_directive(&mut self, node: Directive) {
        <Directive as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_decorator(&mut self, node: Decorator) {
        <Decorator as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class(&mut self, node: Class) {
        <Class as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_body(&mut self, node: ClassBody) {
        <ClassBody as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_method_definition(&mut self, node: MethodDefinition) {
        <MethodDefinition as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_property_definition(&mut self, node: PropertyDefinition) {
        <PropertyDefinition as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_static_block(&mut self, node: StaticBlock) {
        <StaticBlock as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_function(&mut self, node: Function) {
        <Function as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_function_body(&mut self, node: FunctionBody) {
        <FunctionBody as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_block_statement(&mut self, node: BlockStatement) {
        <BlockStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_while_statement(&mut self, node: WhileStatement) {
        <WhileStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_do_while_statement(&mut self, node: DoWhileStatement) {
        <DoWhileStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_with_statement(&mut self, node: WithStatement) {
        <WithStatement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_formal_parameters(&mut self, node: FormalParameters) {
        <FormalParameters as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_formal_parameter(&mut self, node: FormalParameter) {
        <FormalParameter as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_array_expression(&mut self, node: ArrayExpression) {
        <ArrayExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_object_expression(&mut self, node: ObjectExpression) {
        <ObjectExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_spread_element(&mut self, node: SpreadElement) {
        <SpreadElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_object_property(&mut self, node: ObjectProperty) {
        <ObjectProperty as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_parenthesized_expression(&mut self, node: ParenthesizedExpression) {
        <ParenthesizedExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_arrow_function_expression(&mut self, node: ArrowFunctionExpression) {
        <ArrowFunctionExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_sequence_expression(&mut self, node: SequenceExpression) {
        <SequenceExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_member_expression(&mut self, node: MemberExpression) {
        <MemberExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_call_expression(&mut self, node: CallExpression) {
        <CallExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_chain_expression(&mut self, node: ChainExpression) {
        <ChainExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_tagged_template_expression(&mut self, node: TaggedTemplateExpression) {
        <TaggedTemplateExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_new_expression(&mut self, node: NewExpression) {
        <NewExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_await_expression(&mut self, node: AwaitExpression) {
        <AwaitExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_yield_expression(&mut self, node: YieldExpression) {
        <YieldExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_meta_property(&mut self, node: MetaProperty) {
        <MetaProperty as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_expression(&mut self, node: ImportExpression) {
        <ImportExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_declaration(&mut self, node: ImportDeclaration) {
        <ImportDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_specifier(&mut self, node: ImportSpecifier) {
        <ImportSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_default_specifier(&mut self, node: ImportDefaultSpecifier) {
        <ImportDefaultSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_namespace_specifier(&mut self, node: ImportNamespaceSpecifier) {
        <ImportNamespaceSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_attribute(&mut self, node: ImportAttribute) {
        <ImportAttribute as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        <ExportNamedDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        <ExportDefaultDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        <ExportAllDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_specifier(&mut self, node: ExportSpecifier) {
        <ExportSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_hashbang(&mut self, node: Hashbang) {
        <Hashbang as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type(&mut self, node: TsType) {
        <TsType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_name(&mut self, node: TsTypeName) {
        <TsTypeName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_qualified_name_left(&mut self, node: TsQualifiedNameLeft) {
        <TsQualifiedNameLeft as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_import_type_qualifier(&mut self, node: TsImportTypeQualifier) {
        <TsImportTypeQualifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_query_expr_name(&mut self, node: TsTypeQueryExprName) {
        <TsTypeQueryExprName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_literal(&mut self, node: TsLiteral) {
        <TsLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_tuple_element(&mut self, node: TsTupleElement) {
        <TsTupleElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_element(&mut self, node: TsTypeElement) {
        <TsTypeElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_predicate_name(&mut self, node: TsTypePredicateName) {
        <TsTypePredicateName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_enum_member_name(&mut self, node: TsEnumMemberName) {
        <TsEnumMemberName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_module_name(&mut self, node: TsModuleName) {
        <TsModuleName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_module_body(&mut self, node: TsModuleBody) {
        <TsModuleBody as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_module_reference(&mut self, node: TsModuleReference) {
        <TsModuleReference as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_annotation(&mut self, node: TsTypeAnnotation) {
        <TsTypeAnnotation as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_any_keyword(&mut self, node: TsAnyKeyword) {
        <TsAnyKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_unknown_keyword(&mut self, node: TsUnknownKeyword) {
        <TsUnknownKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_never_keyword(&mut self, node: TsNeverKeyword) {
        <TsNeverKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_void_keyword(&mut self, node: TsVoidKeyword) {
        <TsVoidKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_null_keyword(&mut self, node: TsNullKeyword) {
        <TsNullKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_undefined_keyword(&mut self, node: TsUndefinedKeyword) {
        <TsUndefinedKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_string_keyword(&mut self, node: TsStringKeyword) {
        <TsStringKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_number_keyword(&mut self, node: TsNumberKeyword) {
        <TsNumberKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_big_int_keyword(&mut self, node: TsBigIntKeyword) {
        <TsBigIntKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_boolean_keyword(&mut self, node: TsBooleanKeyword) {
        <TsBooleanKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_symbol_keyword(&mut self, node: TsSymbolKeyword) {
        <TsSymbolKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_object_keyword(&mut self, node: TsObjectKeyword) {
        <TsObjectKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_intrinsic_keyword(&mut self, node: TsIntrinsicKeyword) {
        <TsIntrinsicKeyword as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_this_type(&mut self, node: TsThisType) {
        <TsThisType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_reference(&mut self, node: TsTypeReference) {
        <TsTypeReference as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_qualified_name(&mut self, node: TsQualifiedName) {
        <TsQualifiedName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_query(&mut self, node: TsTypeQuery) {
        <TsTypeQuery as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_import_type(&mut self, node: TsImportType) {
        <TsImportType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_parameter(&mut self, node: TsTypeParameter) {
        <TsTypeParameter as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_parameter_declaration(&mut self, node: TsTypeParameterDeclaration) {
        <TsTypeParameterDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_parameter_instantiation(&mut self, node: TsTypeParameterInstantiation) {
        <TsTypeParameterInstantiation as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_ts_literal_type(&mut self, node: TsLiteralType) {
        <TsLiteralType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_template_literal_type(&mut self, node: TsTemplateLiteralType) {
        <TsTemplateLiteralType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_array_type(&mut self, node: TsArrayType) {
        <TsArrayType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_indexed_access_type(&mut self, node: TsIndexedAccessType) {
        <TsIndexedAccessType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_tuple_type(&mut self, node: TsTupleType) {
        <TsTupleType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_named_tuple_member(&mut self, node: TsNamedTupleMember) {
        <TsNamedTupleMember as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_optional_type(&mut self, node: TsOptionalType) {
        <TsOptionalType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_rest_type(&mut self, node: TsRestType) {
        <TsRestType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_js_doc_nullable_type(&mut self, node: TsJsDocNullableType) {
        <TsJsDocNullableType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_js_doc_non_nullable_type(&mut self, node: TsJsDocNonNullableType) {
        <TsJsDocNonNullableType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_js_doc_unknown_type(&mut self, node: TsJsDocUnknownType) {
        <TsJsDocUnknownType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_union_type(&mut self, node: TsUnionType) {
        <TsUnionType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_intersection_type(&mut self, node: TsIntersectionType) {
        <TsIntersectionType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_conditional_type(&mut self, node: TsConditionalType) {
        <TsConditionalType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_infer_type(&mut self, node: TsInferType) {
        <TsInferType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_operator(&mut self, node: TsTypeOperator) {
        <TsTypeOperator as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_parenthesized_type(&mut self, node: TsParenthesizedType) {
        <TsParenthesizedType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_function_type(&mut self, node: TsFunctionType) {
        <TsFunctionType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_constructor_type(&mut self, node: TsConstructorType) {
        <TsConstructorType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_predicate(&mut self, node: TsTypePredicate) {
        <TsTypePredicate as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_literal(&mut self, node: TsTypeLiteral) {
        <TsTypeLiteral as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_mapped_type(&mut self, node: TsMappedType) {
        <TsMappedType as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_property_signature(&mut self, node: TsPropertySignature) {
        <TsPropertySignature as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_method_signature(&mut self, node: TsMethodSignature) {
        <TsMethodSignature as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_call_signature_declaration(&mut self, node: TsCallSignatureDeclaration) {
        <TsCallSignatureDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
    ) {
        <TsConstructSignatureDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_ts_index_signature(&mut self, node: TsIndexSignature) {
        <TsIndexSignature as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_alias_declaration(&mut self, node: TsTypeAliasDeclaration) {
        <TsTypeAliasDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_interface_declaration(&mut self, node: TsInterfaceDeclaration) {
        <TsInterfaceDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_interface_body(&mut self, node: TsInterfaceBody) {
        <TsInterfaceBody as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_interface_heritage(&mut self, node: TsInterfaceHeritage) {
        <TsInterfaceHeritage as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_class_implements(&mut self, node: TsClassImplements) {
        <TsClassImplements as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_enum_declaration(&mut self, node: TsEnumDeclaration) {
        <TsEnumDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_enum_body(&mut self, node: TsEnumBody) {
        <TsEnumBody as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_enum_member(&mut self, node: TsEnumMember) {
        <TsEnumMember as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        <TsModuleDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_module_block(&mut self, node: TsModuleBlock) {
        <TsModuleBlock as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        <TsGlobalDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_parameter_property(&mut self, node: TsParameterProperty) {
        <TsParameterProperty as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_this_parameter(&mut self, node: TsThisParameter) {
        <TsThisParameter as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_as_expression(&mut self, node: TsAsExpression) {
        <TsAsExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_satisfies_expression(&mut self, node: TsSatisfiesExpression) {
        <TsSatisfiesExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_assertion(&mut self, node: TsTypeAssertion) {
        <TsTypeAssertion as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_non_null_expression(&mut self, node: TsNonNullExpression) {
        <TsNonNullExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_instantiation_expression(&mut self, node: TsInstantiationExpression) {
        <TsInstantiationExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_export_assignment(&mut self, node: TsExportAssignment) {
        <TsExportAssignment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_namespace_export_declaration(&mut self, node: TsNamespaceExportDeclaration) {
        <TsNamespaceExportDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        <TsImportEqualsDeclaration as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_external_module_reference(&mut self, node: TsExternalModuleReference) {
        <TsExternalModuleReference as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element_name(&mut self, node: JsxElementName) {
        <JsxElementName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_member_expression_object(&mut self, node: JsxMemberExpressionObject) {
        <JsxMemberExpressionObject as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attribute_name(&mut self, node: JsxAttributeName) {
        <JsxAttributeName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attribute_item(&mut self, node: JsxAttributeItem) {
        <JsxAttributeItem as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attribute_value(&mut self, node: JsxAttributeValue) {
        <JsxAttributeValue as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_expression(&mut self, node: JsxExpression) {
        <JsxExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_child(&mut self, node: JsxChild) {
        <JsxChild as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element(&mut self, node: JsxElement) {
        <JsxElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_opening_element(&mut self, node: JsxOpeningElement) {
        <JsxOpeningElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_closing_element(&mut self, node: JsxClosingElement) {
        <JsxClosingElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_fragment(&mut self, node: JsxFragment) {
        <JsxFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_opening_fragment(&mut self, node: JsxOpeningFragment) {
        <JsxOpeningFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_closing_fragment(&mut self, node: JsxClosingFragment) {
        <JsxClosingFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_identifier(&mut self, node: JsxIdentifier) {
        <JsxIdentifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_namespaced_name(&mut self, node: JsxNamespacedName) {
        <JsxNamespacedName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_member_expression(&mut self, node: JsxMemberExpression) {
        <JsxMemberExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attribute(&mut self, node: JsxAttribute) {
        <JsxAttribute as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_spread_attribute(&mut self, node: JsxSpreadAttribute) {
        <JsxSpreadAttribute as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_expression_container(&mut self, node: JsxExpressionContainer) {
        <JsxExpressionContainer as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_empty_expression(&mut self, node: JsxEmptyExpression) {
        <JsxEmptyExpression as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_text(&mut self, node: JsxText) {
        <JsxText as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_spread_child(&mut self, node: JsxSpreadChild) {
        <JsxSpreadChild as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_statement_list_items(&mut self, node: TypedSubRange<StatementListItem>) {
        <TypedSubRange<StatementListItem> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_hashbang(&mut self, node: Option<Hashbang>) {
        <Option<Hashbang> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_template_elements(&mut self, node: TypedSubRange<TemplateElement>) {
        <TypedSubRange<TemplateElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_exprs(&mut self, node: TypedSubRange<Expr>) {
        <TypedSubRange<Expr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_decorators(&mut self, node: TypedSubRange<Decorator>) {
        <TypedSubRange<Decorator> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_opt_ts_type_annotation(&mut self, node: Option<TsTypeAnnotation>) {
        <Option<TsTypeAnnotation> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_binding_pattern(&mut self, node: Option<BindingPattern>) {
        <Option<BindingPattern> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_vec_binding_patterns(&mut self, node: TypedSubRange<Option<BindingPattern>>) {
        <TypedSubRange<Option<BindingPattern>> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_binding_rest_element(&mut self, node: Option<BindingRestElement>) {
        <Option<BindingRestElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_binding_propertys(&mut self, node: TypedSubRange<BindingProperty>) {
        <TypedSubRange<BindingProperty> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_variable_declarators(&mut self, node: TypedSubRange<VariableDeclarator>) {
        <TypedSubRange<VariableDeclarator> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_expr(&mut self, node: Option<Expr>) {
        <Option<Expr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_stmt(&mut self, node: Option<Stmt>) {
        <Option<Stmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_cases(&mut self, node: TypedSubRange<SwitchCase>) {
        <TypedSubRange<SwitchCase> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_stmts(&mut self, node: TypedSubRange<Stmt>) {
        <TypedSubRange<Stmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_opt_for_statement_init(&mut self, node: Option<ForStatementInit>) {
        <Option<ForStatementInit> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_catch_clause(&mut self, node: Option<CatchClause>) {
        <Option<CatchClause> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_block_statement(&mut self, node: Option<BlockStatement>) {
        <Option<BlockStatement> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_label_identifier(&mut self, node: Option<LabelIdentifier>) {
        <Option<LabelIdentifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_binding_identifier(&mut self, node: Option<BindingIdentifier>) {
        <Option<BindingIdentifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_ts_type_parameter_declaration(
        &mut self,
        node: Option<TsTypeParameterDeclaration>,
    ) {
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_opt_ts_type_parameter_instantiation(
        &mut self,
        node: Option<TsTypeParameterInstantiation>,
    ) {
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_ts_class_implementss(&mut self, node: TypedSubRange<TsClassImplements>) {
        <TypedSubRange<TsClassImplements> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_class_elements(&mut self, node: TypedSubRange<ClassElement>) {
        <TypedSubRange<ClassElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_function_body(&mut self, node: Option<FunctionBody>) {
        <Option<FunctionBody> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_formal_parameter_items(&mut self, node: TypedSubRange<FormalParameterItem>) {
        <TypedSubRange<FormalParameterItem> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_argument(&mut self, node: Option<Argument>) {
        <Option<Argument> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_vec_arguments(&mut self, node: TypedSubRange<Option<Argument>>) {
        <TypedSubRange<Option<Argument>> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_object_property_kinds(&mut self, node: TypedSubRange<ObjectPropertyKind>) {
        <TypedSubRange<ObjectPropertyKind> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_arguments(&mut self, node: TypedSubRange<Argument>) {
        <TypedSubRange<Argument> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_import_declaration_specifiers(
        &mut self,
        node: TypedSubRange<ImportDeclarationSpecifier>,
    ) {
        <TypedSubRange<
            ImportDeclarationSpecifier,
        > as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_import_attributes(&mut self, node: TypedSubRange<ImportAttribute>) {
        <TypedSubRange<ImportAttribute> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_decl(&mut self, node: Option<Decl>) {
        <Option<Decl> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_specifiers(&mut self, node: TypedSubRange<ExportSpecifier>) {
        <TypedSubRange<ExportSpecifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_string_literal(&mut self, node: Option<StringLiteral>) {
        <Option<StringLiteral> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_module_export_name(&mut self, node: Option<ModuleExportName>) {
        <Option<ModuleExportName> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_ts_import_type_qualifier(&mut self, node: Option<TsImportTypeQualifier>) {
        <Option<TsImportTypeQualifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_opt_ts_type(&mut self, node: Option<TsType>) {
        <Option<TsType> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_type_parameters(&mut self, node: TypedSubRange<TsTypeParameter>) {
        <TypedSubRange<TsTypeParameter> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_ts_types(&mut self, node: TypedSubRange<TsType>) {
        <TypedSubRange<TsType> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_ts_tuple_elements(&mut self, node: TypedSubRange<TsTupleElement>) {
        <TypedSubRange<TsTupleElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_ts_type_elements(&mut self, node: TypedSubRange<TsTypeElement>) {
        <TypedSubRange<TsTypeElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_binding_identifiers(&mut self, node: TypedSubRange<BindingIdentifier>) {
        <TypedSubRange<BindingIdentifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_ts_interface_heritages(&mut self, node: TypedSubRange<TsInterfaceHeritage>) {
        <TypedSubRange<TsInterfaceHeritage> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_ts_enum_members(&mut self, node: TypedSubRange<TsEnumMember>) {
        <TypedSubRange<TsEnumMember> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_ts_module_body(&mut self, node: Option<TsModuleBody>) {
        <Option<TsModuleBody> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_childs(&mut self, node: TypedSubRange<JsxChild>) {
        <TypedSubRange<JsxChild> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_opt_jsx_closing_element(&mut self, node: Option<JsxClosingElement>) {
        <Option<JsxClosingElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attribute_items(&mut self, node: TypedSubRange<JsxAttributeItem>) {
        <TypedSubRange<JsxAttributeItem> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_jsx_attribute_value(&mut self, node: Option<JsxAttributeValue>) {
        <Option<JsxAttributeValue> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
}
pub trait VisitMutWith<'a, V: ?Sized + VisitMut<'a>> {
    fn visit_mut_with(self, visitor: &mut V);
    fn visit_mut_children_with(self, visitor: &mut V);
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Program {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_program(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<StatementListItem> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.hashbang(visitor.ast());
        <Option<Hashbang> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Stmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().stmt_data(self);
        match data {
            StmtData::BlockStatement(it) => {
                <BlockStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::BreakStatement(it) => {
                <BreakStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ContinueStatement(it) => {
                <ContinueStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::DebuggerStatement(it) => {
                <DebuggerStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::DoWhileStatement(it) => {
                <DoWhileStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::EmptyStatement(it) => {
                <EmptyStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ExpressionStatement(it) => {
                <ExpressionStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ForInStatement(it) => {
                <ForInStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ForOfStatement(it) => {
                <ForOfStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ForStatement(it) => {
                <ForStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::IfStatement(it) => {
                <IfStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::LabeledStatement(it) => {
                <LabeledStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ReturnStatement(it) => {
                <ReturnStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::SwitchStatement(it) => {
                <SwitchStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ThrowStatement(it) => {
                <ThrowStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::TryStatement(it) => {
                <TryStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::WhileStatement(it) => {
                <WhileStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::WithStatement(it) => {
                <WithStatement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::Declaration(it) => <Decl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::ImportDeclaration(it) => {
                <ImportDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ExportNamedDeclaration(it) => {
                <ExportNamedDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ExportDefaultDeclaration(it) => {
                <ExportDefaultDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::ExportAllDeclaration(it) => {
                <ExportAllDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::TsExportAssignment(it) => {
                <TsExportAssignment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::TsNamespaceExportDeclaration(it) => {
                <TsNamespaceExportDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::TsImportEqualsDeclaration(it) => {
                <TsImportEqualsDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for StatementListItem {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_statement_list_item(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().statement_list_item_data(self);
        match data {
            StatementListItemData::Stmt(it) => {
                <Stmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StatementListItemData::Directive(it) => {
                <Directive as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().decl_data(self);
        match data {
            DeclData::VariableDeclaration(it) => {
                <VariableDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            DeclData::Function(it) => {
                <Function as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            DeclData::Class(it) => <Class as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            DeclData::TsTypeAliasDeclaration(it) => {
                <TsTypeAliasDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            DeclData::TsInterfaceDeclaration(it) => {
                <TsInterfaceDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            DeclData::TsEnumDeclaration(it) => {
                <TsEnumDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            DeclData::TsModuleDeclaration(it) => {
                <TsModuleDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            DeclData::TsGlobalDeclaration(it) => {
                <TsGlobalDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Expr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().expr_data(self);
        match data {
            ExprData::IdentifierReference(it) => {
                <IdentifierReference as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::ThisExpression(it) => {
                <ThisExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Super(it) => <Super as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::MetaProperty(it) => {
                <MetaProperty as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::StringLiteral(it) => {
                <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::NumericLiteral(it) => {
                <NumericLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::BigIntLiteral(it) => {
                <BigIntLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::BooleanLiteral(it) => {
                <BooleanLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::NullLiteral(it) => {
                <NullLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::RegExpLiteral(it) => {
                <RegExpLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::TemplateLiteral(it) => {
                <TemplateLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::BinaryExpression(it) => {
                <BinaryExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::LogicalExpression(it) => {
                <LogicalExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::UnaryExpression(it) => {
                <UnaryExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::UpdateExpression(it) => {
                <UpdateExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::AssignmentExpression(it) => {
                <AssignmentExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::ConditionalExpression(it) => {
                <ConditionalExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::SequenceExpression(it) => {
                <SequenceExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::MemberExpression(it) => {
                <MemberExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::CallExpression(it) => {
                <CallExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::ChainExpression(it) => {
                <ChainExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::NewExpression(it) => {
                <NewExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::TaggedTemplateExpression(it) => {
                <TaggedTemplateExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::ArrowFunctionExpression(it) => {
                <ArrowFunctionExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::ArrayExpression(it) => {
                <ArrayExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::ObjectExpression(it) => {
                <ObjectExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::ParenthesizedExpression(it) => {
                <ParenthesizedExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Function(it) => {
                <Function as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Class(it) => <Class as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::ImportExpression(it) => {
                <ImportExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::AwaitExpression(it) => {
                <AwaitExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::YieldExpression(it) => {
                <YieldExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::TsAsExpression(it) => {
                <TsAsExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::TsSatisfiesExpression(it) => {
                <TsSatisfiesExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::TsTypeAssertion(it) => {
                <TsTypeAssertion as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::TsNonNullExpression(it) => {
                <TsNonNullExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::TsInstantiationExpression(it) => {
                <TsInstantiationExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::JsxElement(it) => {
                <JsxElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::JsxFragment(it) => {
                <JsxFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BindingPattern {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_pattern(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().binding_pattern_data(self);
        match data {
            BindingPatternData::BindingIdentifier(it) => {
                <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            BindingPatternData::AssignmentPattern(it) => {
                <AssignmentPattern as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            BindingPatternData::BindingRestElement(it) => {
                <BindingRestElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            BindingPatternData::ArrayPattern(it) => {
                <ArrayPattern as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            BindingPatternData::ObjectPattern(it) => {
                <ObjectPattern as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignmentTarget {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assignment_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().assignment_target_data(self);
        match data {
            AssignmentTargetData::SimpleAssignmentTarget(it) => {
                <SimpleAssignmentTarget as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            AssignmentTargetData::ArrayAssignmentTarget(it) => {
                <ArrayPattern as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            AssignmentTargetData::ObjectAssignmentTarget(it) => {
                <ObjectPattern as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SimpleAssignmentTarget {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_simple_assignment_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().simple_assignment_target_data(self);
        match data {
            SimpleAssignmentTargetData::IdentifierReference(it) => {
                <IdentifierReference as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignmentTargetData::MemberExpression(it) => {
                <MemberExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsAsExpression(it) => {
                <TsAsExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(it) => {
                <TsSatisfiesExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsTypeAssertion(it) => {
                <TsTypeAssertion as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignmentTargetData::TsNonNullExpression(it) => {
                <TsNonNullExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropertyKey {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_property_key(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().property_key_data(self);
        match data {
            PropertyKeyData::IdentifierName(it) => {
                <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropertyKeyData::PrivateIdentifier(it) => {
                <PrivateIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropertyKeyData::StringLiteral(it) => {
                <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropertyKeyData::NumericLiteral(it) => {
                <NumericLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropertyKeyData::BigIntLiteral(it) => {
                <BigIntLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropertyKeyData::Expr(it) => <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleExportName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().module_export_name_data(self);
        match data {
            ModuleExportNameData::IdentifierName(it) => {
                <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleExportNameData::StringLiteral(it) => {
                <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Argument {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_argument(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().argument_data(self);
        match data {
            ArgumentData::Expr(it) => <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ArgumentData::SpreadElement(it) => {
                <SpreadElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPropertyKind {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_property_kind(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().object_property_kind_data(self);
        match data {
            ObjectPropertyKindData::ObjectProperty(it) => {
                <ObjectProperty as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ObjectPropertyKindData::SpreadElement(it) => {
                <SpreadElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForStatementInit {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_statement_init(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().for_statement_init_data(self);
        match data {
            ForStatementInitData::VariableDeclaration(it) => {
                <VariableDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ForStatementInitData::Expr(it) => {
                <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForStatementLeft {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_statement_left(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().for_statement_left_data(self);
        match data {
            ForStatementLeftData::VariableDeclaration(it) => {
                <VariableDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ForStatementLeftData::AssignmentTarget(it) => {
                <AssignmentTarget as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrowFunctionBody {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_arrow_function_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().arrow_function_body_data(self);
        match data {
            ArrowFunctionBodyData::FunctionBody(it) => {
                <FunctionBody as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ArrowFunctionBodyData::Expr(it) => {
                <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FormalParameterItem {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_formal_parameter_item(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().formal_parameter_item_data(self);
        match data {
            FormalParameterItemData::FormalParameter(it) => {
                <FormalParameter as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            FormalParameterItemData::TsParameterProperty(it) => {
                <TsParameterProperty as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FormalParameterPattern {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_formal_parameter_pattern(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().formal_parameter_pattern_data(self);
        match data {
            FormalParameterPatternData::BindingPattern(it) => {
                <BindingPattern as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            FormalParameterPatternData::TsThisParameter(it) => {
                <TsThisParameter as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().class_element_data(self);
        match data {
            ClassElementData::MethodDefinition(it) => {
                <MethodDefinition as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassElementData::PropertyDefinition(it) => {
                <PropertyDefinition as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassElementData::StaticBlock(it) => {
                <StaticBlock as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassElementData::TsIndexSignature(it) => {
                <TsIndexSignature as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDeclarationSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_declaration_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().import_declaration_specifier_data(self);
        match data {
            ImportDeclarationSpecifierData::ImportSpecifier(it) => {
                <ImportSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ImportDeclarationSpecifierData::ImportDefaultSpecifier(it) => {
                <ImportDefaultSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ImportDeclarationSpecifierData::ImportNamespaceSpecifier(it) => {
                <ImportNamespaceSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultDeclarationKind {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_declaration_kind(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().export_default_declaration_kind_data(self);
        match data {
            ExportDefaultDeclarationKindData::Expr(it) => {
                <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExportDefaultDeclarationKindData::Function(it) => {
                <Function as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExportDefaultDeclarationKindData::Class(it) => {
                <Class as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(it) => {
                <TsInterfaceDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Super {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_super(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThisExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_this_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for StringLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_string_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NumericLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_numeric_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BigIntLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_big_int_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BooleanLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_boolean_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NullLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_null_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for RegExpLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_reg_exp_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TemplateLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_template_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.quasis(visitor.ast());
        <TypedSubRange<TemplateElement> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.expressions(visitor.ast());
        <TypedSubRange<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TemplateElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_template_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IdentifierReference {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_identifier_reference(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateIdentifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_private_identifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BindingIdentifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_identifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IdentifierName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_identifier_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for LabelIdentifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_label_identifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignmentPattern {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assignment_pattern(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <BindingPattern as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BindingRestElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_rest_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <BindingPattern as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayPattern {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_array_pattern(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.elements(visitor.ast());
        <TypedSubRange<Option<BindingPattern>> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.rest(visitor.ast());
        <Option<BindingRestElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPattern {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_pattern(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.properties(visitor.ast());
        <TypedSubRange<BindingProperty> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.rest(visitor.ast());
        <Option<BindingRestElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BindingProperty {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_property(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <BindingPattern as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BinaryExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binary_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for LogicalExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_logical_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ConditionalExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_conditional_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.consequent(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.alternate(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UnaryExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_unary_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UpdateExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_update_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <SimpleAssignmentTarget as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignmentExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assignment_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <AssignmentTarget as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VariableDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_variable_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.declarators(visitor.ast());
        <TypedSubRange<VariableDeclarator> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VariableDeclarator {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_variable_declarator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <BindingPattern as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.init(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IfStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_if_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.consequent(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.alternate(visitor.ast());
        <Option<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.discriminant(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.cases(visitor.ast());
        <TypedSubRange<SwitchCase> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchCase {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_case(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.consequent(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.init(visitor.ast());
        <Option<ForStatementInit> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.test(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.update(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForInStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_in_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <ForStatementLeft as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForOfStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_of_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <ForStatementLeft as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TryStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_try_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.block(visitor.ast());
        <BlockStatement as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.handler(visitor.ast());
        <Option<CatchClause> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.finalizer(visitor.ast());
        <Option<BlockStatement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CatchClause {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.param(visitor.ast());
        <Option<BindingPattern> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <BlockStatement as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BreakStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_break_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Option<LabelIdentifier> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ContinueStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_continue_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Option<LabelIdentifier> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for LabeledStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_labeled_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <LabelIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExpressionStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expression_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ReturnStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_return_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThrowStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_throw_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DebuggerStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_debugger_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for EmptyStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_empty_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Directive {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_directive(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decorator {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decorator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Class {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.id(visitor.ast());
        <Option<BindingIdentifier> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.super_class(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <ClassBody as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.super_type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.implements(visitor.ast());
        <TypedSubRange<TsClassImplements> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassBody {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<ClassElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MethodDefinition {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_method_definition(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.key(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropertyDefinition {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_property_definition(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.key(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for StaticBlock {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_static_block(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Function {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_function(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <Option<BindingIdentifier> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Option<FunctionBody> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.return_type(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FunctionBody {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_function_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<StatementListItem> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BlockStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_block_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WhileStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_while_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DoWhileStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_do_while_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WithStatement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_with_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.object(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FormalParameters {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_formal_parameters(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.items(visitor.ast());
        <TypedSubRange<FormalParameterItem> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.rest(visitor.ast());
        <Option<BindingRestElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FormalParameter {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_formal_parameter(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.pattern(visitor.ast());
        <FormalParameterPattern as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_array_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.elements(visitor.ast());
        <TypedSubRange<Option<Argument>> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.properties(visitor.ast());
        <TypedSubRange<ObjectPropertyKind> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SpreadElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_spread_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectProperty {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_property(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ParenthesizedExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_parenthesized_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrowFunctionExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_arrow_function_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <ArrowFunctionBody as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.return_type(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SequenceExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_sequence_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expressions(visitor.ast());
        <TypedSubRange<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MemberExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_member_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.object(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.property(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CallExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_call_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.arguments(visitor.ast());
        <TypedSubRange<Argument> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ChainExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_chain_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TaggedTemplateExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tagged_template_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.tag(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.quasi(visitor.ast());
        <TemplateLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NewExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_new_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.arguments(visitor.ast());
        <TypedSubRange<Argument> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AwaitExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_await_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for YieldExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_yield_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MetaProperty {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_meta_property(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.meta(visitor.ast());
        <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.property(visitor.ast());
        <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.source(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.options(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.specifiers(visitor.ast());
        <TypedSubRange<ImportDeclarationSpecifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.source(visitor.ast());
        <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.attributes(visitor.ast());
        <TypedSubRange<ImportAttribute> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.imported(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.local(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDefaultSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportNamespaceSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_namespace_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportAttribute {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_attribute(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportNamedDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_named_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.declaration(visitor.ast());
        <Option<Decl> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.specifiers(visitor.ast());
        <TypedSubRange<ExportSpecifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.source(visitor.ast());
        <Option<StringLiteral> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.attributes(visitor.ast());
        <TypedSubRange<ImportAttribute> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.declaration(visitor.ast());
        <ExportDefaultDeclarationKind as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportAllDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_all_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.exported(visitor.ast());
        <Option<ModuleExportName> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.source(visitor.ast());
        <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.attributes(visitor.ast());
        <TypedSubRange<ImportAttribute> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.exported(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Hashbang {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_hashbang(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_type_data(self);
        match data {
            TsTypeData::TsAnyKeyword(it) => {
                <TsAnyKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsUnknownKeyword(it) => {
                <TsUnknownKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsNeverKeyword(it) => {
                <TsNeverKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsVoidKeyword(it) => {
                <TsVoidKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsNullKeyword(it) => {
                <TsNullKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsUndefinedKeyword(it) => {
                <TsUndefinedKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsStringKeyword(it) => {
                <TsStringKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsNumberKeyword(it) => {
                <TsNumberKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsBigIntKeyword(it) => {
                <TsBigIntKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsBooleanKeyword(it) => {
                <TsBooleanKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsSymbolKeyword(it) => {
                <TsSymbolKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsObjectKeyword(it) => {
                <TsObjectKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsIntrinsicKeyword(it) => {
                <TsIntrinsicKeyword as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsThisType(it) => {
                <TsThisType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsTypeReference(it) => {
                <TsTypeReference as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsTypeQuery(it) => {
                <TsTypeQuery as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsImportType(it) => {
                <TsImportType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsLiteralType(it) => {
                <TsLiteralType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsTemplateLiteralType(it) => {
                <TsTemplateLiteralType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsArrayType(it) => {
                <TsArrayType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsIndexedAccessType(it) => {
                <TsIndexedAccessType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsTupleType(it) => {
                <TsTupleType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsOptionalType(it) => {
                <TsOptionalType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsRestType(it) => {
                <TsRestType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsJsDocNullableType(it) => {
                <TsJsDocNullableType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsJsDocNonNullableType(it) => {
                <TsJsDocNonNullableType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsJsDocUnknownType(it) => {
                <TsJsDocUnknownType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsUnionType(it) => {
                <TsUnionType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsIntersectionType(it) => {
                <TsIntersectionType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsConditionalType(it) => {
                <TsConditionalType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsInferType(it) => {
                <TsInferType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsTypeOperator(it) => {
                <TsTypeOperator as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsParenthesizedType(it) => {
                <TsParenthesizedType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsFunctionType(it) => {
                <TsFunctionType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsConstructorType(it) => {
                <TsConstructorType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsTypePredicate(it) => {
                <TsTypePredicate as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsTypeLiteral(it) => {
                <TsTypeLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeData::TsMappedType(it) => {
                <TsMappedType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_type_name_data(self);
        match data {
            TsTypeNameData::IdentifierReference(it) => {
                <IdentifierReference as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeNameData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsQualifiedNameLeft {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_qualified_name_left(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_qualified_name_left_data(self);
        match data {
            TsQualifiedNameLeftData::IdentifierReference(it) => {
                <IdentifierReference as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsQualifiedNameLeftData::IdentifierName(it) => {
                <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsQualifiedNameLeftData::BindingIdentifier(it) => {
                <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsQualifiedNameLeftData::ThisExpression(it) => {
                <ThisExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsQualifiedNameLeftData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsImportTypeQualifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_import_type_qualifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_import_type_qualifier_data(self);
        match data {
            TsImportTypeQualifierData::IdentifierName(it) => {
                <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsImportTypeQualifierData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeQueryExprName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_query_expr_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_type_query_expr_name_data(self);
        match data {
            TsTypeQueryExprNameData::TsTypeName(it) => {
                <TsTypeName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeQueryExprNameData::ThisExpression(it) => {
                <ThisExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeQueryExprNameData::TsImportType(it) => {
                <TsImportType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_literal_data(self);
        match data {
            TsLiteralData::StringLiteral(it) => {
                <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsLiteralData::NumericLiteral(it) => {
                <NumericLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsLiteralData::BigIntLiteral(it) => {
                <BigIntLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsLiteralData::BooleanLiteral(it) => {
                <BooleanLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsLiteralData::TemplateLiteral(it) => {
                <TemplateLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsLiteralData::UnaryExpression(it) => {
                <UnaryExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTupleElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_tuple_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_tuple_element_data(self);
        match data {
            TsTupleElementData::TsType(it) => {
                <TsType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTupleElementData::TsNamedTupleMember(it) => {
                <TsNamedTupleMember as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTupleElementData::TsOptionalType(it) => {
                <TsOptionalType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTupleElementData::TsRestType(it) => {
                <TsRestType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_type_element_data(self);
        match data {
            TsTypeElementData::TsPropertySignature(it) => {
                <TsPropertySignature as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeElementData::TsMethodSignature(it) => {
                <TsMethodSignature as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeElementData::TsCallSignatureDeclaration(it) => {
                <TsCallSignatureDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypeElementData::TsConstructSignatureDeclaration(it) => {
                <TsConstructSignatureDeclaration as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                )
            }
            TsTypeElementData::TsIndexSignature(it) => {
                <TsIndexSignature as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypePredicateName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_predicate_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_type_predicate_name_data(self);
        match data {
            TsTypePredicateNameData::IdentifierName(it) => {
                <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsTypePredicateNameData::TsThisType(it) => {
                <TsThisType as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsEnumMemberName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_enum_member_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_enum_member_name_data(self);
        match data {
            TsEnumMemberNameData::IdentifierName(it) => {
                <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsEnumMemberNameData::StringLiteral(it) => {
                <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsEnumMemberNameData::TemplateLiteral(it) => {
                <TemplateLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsEnumMemberNameData::Expr(it) => {
                <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsModuleName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_module_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_module_name_data(self);
        match data {
            TsModuleNameData::BindingIdentifier(it) => {
                <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsModuleNameData::StringLiteral(it) => {
                <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsModuleNameData::TsQualifiedName(it) => {
                <TsQualifiedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsModuleBody {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_module_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_module_body_data(self);
        match data {
            TsModuleBodyData::TsModuleBlock(it) => {
                <TsModuleBlock as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsModuleBodyData::TsModuleDeclaration(it) => {
                <TsModuleDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsModuleReference {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_module_reference(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().ts_module_reference_data(self);
        match data {
            TsModuleReferenceData::TsTypeName(it) => {
                <TsTypeName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            TsModuleReferenceData::TsExternalModuleReference(it) => {
                <TsExternalModuleReference as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeAnnotation {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_annotation(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsAnyKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_any_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsUnknownKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_unknown_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsNeverKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_never_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsVoidKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_void_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsNullKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_null_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsUndefinedKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_undefined_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsStringKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_string_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsNumberKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_number_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsBigIntKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_big_int_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsBooleanKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_boolean_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsSymbolKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_symbol_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsObjectKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_object_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsIntrinsicKeyword {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_intrinsic_keyword(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsThisType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_this_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeReference {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_reference(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_name(visitor.ast());
        <TsTypeName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsQualifiedName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_qualified_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <TsQualifiedNameLeft as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeQuery {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_query(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr_name(visitor.ast());
        <TsTypeQueryExprName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsImportType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_import_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.source(visitor.ast());
        <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.options(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.qualifier(visitor.ast());
        <Option<TsImportTypeQualifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeParameter {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_parameter(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.constraint(visitor.ast());
        <Option<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.default(visitor.ast());
        <Option<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeParameterDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_parameter_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.params(visitor.ast());
        <TypedSubRange<TsTypeParameter> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeParameterInstantiation {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_parameter_instantiation(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.params(visitor.ast());
        <TypedSubRange<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsLiteralType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_literal_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.literal(visitor.ast());
        <TsLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTemplateLiteralType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_template_literal_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.quasis(visitor.ast());
        <TypedSubRange<TemplateElement> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.types(visitor.ast());
        <TypedSubRange<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsArrayType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_array_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.element_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsIndexedAccessType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_indexed_access_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.object_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.index_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTupleType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_tuple_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.element_types(visitor.ast());
        <TypedSubRange<TsTupleElement> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsNamedTupleMember {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_named_tuple_member(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.element_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsOptionalType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_optional_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsRestType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_rest_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsJsDocNullableType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_js_doc_nullable_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsJsDocNonNullableType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_js_doc_non_nullable_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsJsDocUnknownType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_js_doc_unknown_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsUnionType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_union_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.types(visitor.ast());
        <TypedSubRange<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsIntersectionType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_intersection_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.types(visitor.ast());
        <TypedSubRange<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsConditionalType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_conditional_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.check_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.extends_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.true_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.false_type(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsInferType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_infer_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_parameter(visitor.ast());
        <TsTypeParameter as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeOperator {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_operator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsParenthesizedType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_parenthesized_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsFunctionType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_function_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.return_type(visitor.ast());
        <TsTypeAnnotation as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsConstructorType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_constructor_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.return_type(visitor.ast());
        <TsTypeAnnotation as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypePredicate {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_predicate(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.parameter_name(visitor.ast());
        <TsTypePredicateName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeLiteral {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.members(visitor.ast());
        <TypedSubRange<TsTypeElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsMappedType {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_mapped_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.constraint(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.name_type(visitor.ast());
        <Option<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsType> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsPropertySignature {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_property_signature(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsMethodSignature {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_method_signature(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.return_type(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsCallSignatureDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_call_signature_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.return_type(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsConstructSignatureDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_construct_signature_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.return_type(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsIndexSignature {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_index_signature(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.parameters(visitor.ast());
        <TypedSubRange<BindingIdentifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.type_annotation(visitor.ast());
        <TsTypeAnnotation as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeAliasDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_alias_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsInterfaceDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_interface_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.extends(visitor.ast());
        <TypedSubRange<TsInterfaceHeritage> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.body(visitor.ast());
        <TsInterfaceBody as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsInterfaceBody {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_interface_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<TsTypeElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsInterfaceHeritage {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_interface_heritage(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsClassImplements {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_class_implements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsEnumDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_enum_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <TsEnumBody as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsEnumBody {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_enum_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.members(visitor.ast());
        <TypedSubRange<TsEnumMember> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsEnumMember {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_enum_member(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <TsEnumMemberName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.initializer(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsModuleDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_module_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <TsModuleName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Option<TsModuleBody> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsModuleBlock {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_module_block(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<StatementListItem> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsGlobalDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_global_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <TsModuleBlock as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsParameterProperty {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_parameter_property(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.parameter(visitor.ast());
        <BindingPattern as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsThisParameter {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_this_parameter(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsAsExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_as_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsSatisfiesExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_satisfies_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsTypeAssertion {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_assertion(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.type_annotation(visitor.ast());
        <TsType as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsNonNullExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_non_null_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsInstantiationExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_instantiation_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_arguments(visitor.ast());
        <TsTypeParameterInstantiation as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsExportAssignment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_export_assignment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsNamespaceExportDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_namespace_export_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <IdentifierName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsImportEqualsDeclaration {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_import_equals_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <BindingIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.module_reference(visitor.ast());
        <TsModuleReference as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsExternalModuleReference {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_external_module_reference(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxElementName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_element_name_data(self);
        match data {
            JsxElementNameData::JsxIdentifier(it) => {
                <JsxIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxElementNameData::JsxNamespacedName(it) => {
                <JsxNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxElementNameData::JsxMemberExpression(it) => {
                <JsxMemberExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxMemberExpressionObject {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_member_expression_object(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_member_expression_object_data(self);
        match data {
            JsxMemberExpressionObjectData::JsxIdentifier(it) => {
                <JsxIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(it) => {
                <JsxMemberExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxAttributeName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attribute_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_attribute_name_data(self);
        match data {
            JsxAttributeNameData::JsxIdentifier(it) => {
                <JsxIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxAttributeNameData::JsxNamespacedName(it) => {
                <JsxNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxAttributeItem {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attribute_item(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_attribute_item_data(self);
        match data {
            JsxAttributeItemData::JsxAttribute(it) => {
                <JsxAttribute as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxAttributeItemData::JsxSpreadAttribute(it) => {
                <JsxSpreadAttribute as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxAttributeValue {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attribute_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_attribute_value_data(self);
        match data {
            JsxAttributeValueData::StringLiteral(it) => {
                <StringLiteral as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxAttributeValueData::JsxExpressionContainer(it) => {
                <JsxExpressionContainer as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxAttributeValueData::JsxElement(it) => {
                <JsxElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxAttributeValueData::JsxFragment(it) => {
                <JsxFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_expression_data(self);
        match data {
            JsxExpressionData::Expr(it) => {
                <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxExpressionData::JsxEmptyExpression(it) => {
                <JsxEmptyExpression as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxChild {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_child_data(self);
        match data {
            JsxChildData::JsxText(it) => {
                <JsxText as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxChildData::JsxExpressionContainer(it) => {
                <JsxExpressionContainer as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxChildData::JsxSpreadChild(it) => {
                <JsxSpreadChild as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxChildData::JsxElement(it) => {
                <JsxElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JsxChildData::JsxFragment(it) => {
                <JsxFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.opening_element(visitor.ast());
        <JsxOpeningElement as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.children(visitor.ast());
        <TypedSubRange<JsxChild> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.closing_element(visitor.ast());
        <Option<JsxClosingElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxOpeningElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JsxElementName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.attributes(visitor.ast());
        <TypedSubRange<JsxAttributeItem> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.type_arguments(visitor.ast());
        <Option<TsTypeParameterInstantiation> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxClosingElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JsxElementName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.opening_fragment(visitor.ast());
        <JsxOpeningFragment as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.children(visitor.ast());
        <TypedSubRange<JsxChild> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.closing_fragment(visitor.ast());
        <JsxClosingFragment as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxOpeningFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxClosingFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxIdentifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_identifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxNamespacedName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_namespaced_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.namespace(visitor.ast());
        <JsxIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.name(visitor.ast());
        <JsxIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxMemberExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_member_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.object(visitor.ast());
        <JsxMemberExpressionObject as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.property(visitor.ast());
        <JsxIdentifier as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxAttribute {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attribute(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JsxAttributeName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<JsxAttributeValue> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxSpreadAttribute {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_spread_attribute(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.argument(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxExpressionContainer {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_expression_container(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <JsxExpression as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxEmptyExpression {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_empty_expression(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxText {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_text(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JsxSpreadChild {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_spread_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expression(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<StatementListItem> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_statement_list_items(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Hashbang> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_hashbang(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TemplateElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_template_elements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Expr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_exprs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Decorator> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decorators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<TsTypeAnnotation> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ts_type_annotation(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<BindingPattern> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_binding_pattern(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Option<BindingPattern>> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_vec_binding_patterns(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<BindingRestElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_binding_rest_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<BindingProperty> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_propertys(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<VariableDeclarator> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_variable_declarators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Expr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Stmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<SwitchCase> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_cases(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Stmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_stmts(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<ForStatementInit> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_for_statement_init(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<CatchClause> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<BlockStatement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_block_statement(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<LabelIdentifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_label_identifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<BindingIdentifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_binding_identifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<TsTypeParameterDeclaration> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ts_type_parameter_declaration(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<TsTypeParameterInstantiation> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ts_type_parameter_instantiation(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TsClassImplements> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_class_implementss(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ClassElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_elements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<FunctionBody> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_function_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<FormalParameterItem> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_formal_parameter_items(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Argument> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_argument(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Option<Argument>> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_vec_arguments(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ObjectPropertyKind> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_property_kinds(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Argument> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_arguments(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V>
    for TypedSubRange<ImportDeclarationSpecifier>
{
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_declaration_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ImportAttribute> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_attributes(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Decl> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ExportSpecifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<StringLiteral> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_string_literal(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<ModuleExportName> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<TsImportTypeQualifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ts_import_type_qualifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<TsType> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ts_type(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TsTypeParameter> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_parameters(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TsType> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_types(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TsTupleElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_tuple_elements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TsTypeElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_type_elements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<BindingIdentifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_identifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TsInterfaceHeritage> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_interface_heritages(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TsEnumMember> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_enum_members(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<TsModuleBody> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ts_module_body(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<JsxChild> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_childs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<JsxClosingElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<JsxAttributeItem> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attribute_items(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<JsxAttributeValue> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_attribute_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
