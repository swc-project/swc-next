#![allow(unused, clippy::useless_conversion, clippy::single_match)]
use crate::visit::{TraverseAction, TraverseCtx};
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
    fn visit_ts_function(&mut self, node: TsFunction) {
        self.enter_node(node.node_id());
        <TsFunction as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_method_definition(&mut self, node: TsMethodDefinition) {
        self.enter_node(node.node_id());
        <TsMethodDefinition as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
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
    fn visit_directives(&mut self, node: TypedSubRange<Directive>) {
        <TypedSubRange<Directive> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_stmts(&mut self, node: TypedSubRange<Stmt>) {
        <TypedSubRange<Stmt> as VisitWith<'a, Self>>::visit_children_with(node, self)
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
        let field_value_1 = self.directives(visitor.ast());
        let field_value_2 = self.body(visitor.ast());
        let field_value_3 = self.hashbang(visitor.ast());
        <TypedSubRange<Directive> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Option<Hashbang> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
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
            DeclData::TsFunction(it) => <TsFunction as VisitWith<'a, V>>::visit_with(it, visitor),
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
            DeclData::TsImportEqualsDeclaration(it) => {
                <TsImportEqualsDeclaration as VisitWith<'a, V>>::visit_with(it, visitor)
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
            ExprData::PrivateIdentifier(it) => {
                <PrivateIdentifier as VisitWith<'a, V>>::visit_with(it, visitor)
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
            BindingPatternData::SimpleAssignmentTarget(it) => {
                <SimpleAssignmentTarget as VisitWith<'a, V>>::visit_with(it, visitor)
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
            ClassElementData::TsMethodDefinition(it) => {
                <TsMethodDefinition as VisitWith<'a, V>>::visit_with(it, visitor)
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
            ExportDefaultDeclarationKindData::TsFunction(it) => {
                <TsFunction as VisitWith<'a, V>>::visit_with(it, visitor)
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
        let field_value_4 = self.params(visitor.ast());
        let field_value_5 = self.body(visitor.ast());
        let field_value_6 = self.type_parameters(visitor.ast());
        let field_value_7 = self.return_type(visitor.ast());
        <Option<BindingIdentifier> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_4, visitor);
        <FunctionBody as VisitWith<'a, V>>::visit_with(field_value_5, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_6,
            visitor,
        );
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_7, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FunctionBody {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_function_body(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.directives(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <TypedSubRange<Directive> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
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
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsFunction {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_function(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        let field_value_4 = self.params(visitor.ast());
        let field_value_5 = self.type_parameters(visitor.ast());
        let field_value_6 = self.return_type(visitor.ast());
        <Option<BindingIdentifier> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <FormalParameters as VisitWith<'a, V>>::visit_with(field_value_4, visitor);
        <Option<TsTypeParameterDeclaration> as VisitWith<'a, V>>::visit_with(
            field_value_5,
            visitor,
        );
        <Option<TsTypeAnnotation> as VisitWith<'a, V>>::visit_with(field_value_6, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TsMethodDefinition {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ts_method_definition(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decorators(visitor.ast());
        let field_value_1 = self.key(visitor.ast());
        let field_value_2 = self.value(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <PropertyKey as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TsFunction as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
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
        let field_value_0 = self.directives(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <TypedSubRange<Directive> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
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
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Directive> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_directives(visitor, self)
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
    fn visit_mut_ts_function(&mut self, node: TsFunction) {
        <TsFunction as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ts_method_definition(&mut self, node: TsMethodDefinition) {
        <TsMethodDefinition as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
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
    fn visit_mut_directives(&mut self, node: TypedSubRange<Directive>) {
        <TypedSubRange<Directive> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_stmts(&mut self, node: TypedSubRange<Stmt>) {
        <TypedSubRange<Stmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
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
        let field_value = self.directives(visitor.ast());
        <TypedSubRange<Directive> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
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
            DeclData::TsFunction(it) => {
                <TsFunction as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
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
            DeclData::TsImportEqualsDeclaration(it) => {
                <TsImportEqualsDeclaration as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
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
            ExprData::PrivateIdentifier(it) => {
                <PrivateIdentifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
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
            BindingPatternData::SimpleAssignmentTarget(it) => {
                <SimpleAssignmentTarget as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
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
            ClassElementData::TsMethodDefinition(it) => {
                <TsMethodDefinition as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
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
            ExportDefaultDeclarationKindData::TsFunction(it) => {
                <TsFunction as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
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
        <FunctionBody as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
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
        let field_value = self.directives(visitor.ast());
        <TypedSubRange<Directive> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
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
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsFunction {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_function(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <Option<BindingIdentifier> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.params(visitor.ast());
        <FormalParameters as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.type_parameters(visitor.ast());
        <Option<TsTypeParameterDeclaration> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.return_type(visitor.ast());
        <Option<TsTypeAnnotation> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TsMethodDefinition {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ts_method_definition(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.key(visitor.ast());
        <PropertyKey as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <TsFunction as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
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
        let field_value = self.directives(visitor.ast());
        <TypedSubRange<Directive> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
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
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Directive> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_directives(visitor, self)
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
pub trait Traverse<'a, State = ()> {
    #[inline]
    fn enter_node(
        &mut self,
        node_id: NodeId,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_node(&mut self, node_id: NodeId, kind: NodeKind, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_program(&mut self, node: Program, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_super(&mut self, node: Super, ctx: &mut TraverseCtx<'_, 'a, State>) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_super(&mut self, node: Super, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_this_expression(
        &mut self,
        node: ThisExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_this_expression(&mut self, node: ThisExpression, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_string_literal(
        &mut self,
        node: StringLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_string_literal(&mut self, node: StringLiteral, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_numeric_literal(
        &mut self,
        node: NumericLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_numeric_literal(&mut self, node: NumericLiteral, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_big_int_literal(
        &mut self,
        node: BigIntLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_big_int_literal(&mut self, node: BigIntLiteral, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_boolean_literal(
        &mut self,
        node: BooleanLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_boolean_literal(&mut self, node: BooleanLiteral, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_null_literal(
        &mut self,
        node: NullLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_null_literal(&mut self, node: NullLiteral, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_reg_exp_literal(
        &mut self,
        node: RegExpLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_template_literal(
        &mut self,
        node: TemplateLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_template_literal(
        &mut self,
        node: TemplateLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_template_element(
        &mut self,
        node: TemplateElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_template_element(
        &mut self,
        node: TemplateElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_private_identifier(
        &mut self,
        node: PrivateIdentifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_private_identifier(
        &mut self,
        node: PrivateIdentifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_identifier_name(
        &mut self,
        node: IdentifierName,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_identifier_name(&mut self, node: IdentifierName, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_label_identifier(
        &mut self,
        node: LabelIdentifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_label_identifier(
        &mut self,
        node: LabelIdentifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_assignment_pattern(
        &mut self,
        node: AssignmentPattern,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_assignment_pattern(
        &mut self,
        node: AssignmentPattern,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_binding_rest_element(
        &mut self,
        node: BindingRestElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_binding_rest_element(
        &mut self,
        node: BindingRestElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_array_pattern(
        &mut self,
        node: ArrayPattern,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_array_pattern(&mut self, node: ArrayPattern, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_object_pattern(
        &mut self,
        node: ObjectPattern,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_object_pattern(&mut self, node: ObjectPattern, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_binding_property(
        &mut self,
        node: BindingProperty,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_binding_property(
        &mut self,
        node: BindingProperty,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_logical_expression(
        &mut self,
        node: LogicalExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_logical_expression(
        &mut self,
        node: LogicalExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_conditional_expression(
        &mut self,
        node: ConditionalExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_conditional_expression(
        &mut self,
        node: ConditionalExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_update_expression(
        &mut self,
        node: UpdateExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_update_expression(
        &mut self,
        node: UpdateExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_variable_declarator(
        &mut self,
        node: VariableDeclarator,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_variable_declarator(
        &mut self,
        node: VariableDeclarator,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_if_statement(
        &mut self,
        node: IfStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_if_statement(&mut self, node: IfStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_switch_statement(
        &mut self,
        node: SwitchStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_switch_statement(
        &mut self,
        node: SwitchStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_switch_case(&mut self, node: SwitchCase, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_for_statement(
        &mut self,
        node: ForStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_for_statement(&mut self, node: ForStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_for_in_statement(
        &mut self,
        node: ForInStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_for_in_statement(
        &mut self,
        node: ForInStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_for_of_statement(
        &mut self,
        node: ForOfStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_for_of_statement(
        &mut self,
        node: ForOfStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_try_statement(
        &mut self,
        node: TryStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_try_statement(&mut self, node: TryStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_catch_clause(&mut self, node: CatchClause, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_break_statement(
        &mut self,
        node: BreakStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_break_statement(&mut self, node: BreakStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_continue_statement(
        &mut self,
        node: ContinueStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_continue_statement(
        &mut self,
        node: ContinueStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_labeled_statement(
        &mut self,
        node: LabeledStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_labeled_statement(
        &mut self,
        node: LabeledStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_expression_statement(
        &mut self,
        node: ExpressionStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_expression_statement(
        &mut self,
        node: ExpressionStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_return_statement(
        &mut self,
        node: ReturnStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_return_statement(
        &mut self,
        node: ReturnStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_throw_statement(
        &mut self,
        node: ThrowStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_throw_statement(&mut self, node: ThrowStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_debugger_statement(
        &mut self,
        node: DebuggerStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_debugger_statement(
        &mut self,
        node: DebuggerStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_empty_statement(
        &mut self,
        node: EmptyStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_empty_statement(&mut self, node: EmptyStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_directive(
        &mut self,
        node: Directive,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_directive(&mut self, node: Directive, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_decorator(
        &mut self,
        node: Decorator,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_decorator(&mut self, node: Decorator, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_class(&mut self, node: Class, ctx: &mut TraverseCtx<'_, 'a, State>) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_class(&mut self, node: Class, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_class_body(&mut self, node: ClassBody, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_method_definition(
        &mut self,
        node: MethodDefinition,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_method_definition(
        &mut self,
        node: MethodDefinition,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_property_definition(
        &mut self,
        node: PropertyDefinition,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_property_definition(
        &mut self,
        node: PropertyDefinition,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_static_block(&mut self, node: StaticBlock, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_function(&mut self, node: Function, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_function_body(&mut self, node: FunctionBody, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_block_statement(&mut self, node: BlockStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_while_statement(
        &mut self,
        node: WhileStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_while_statement(&mut self, node: WhileStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_do_while_statement(
        &mut self,
        node: DoWhileStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_do_while_statement(
        &mut self,
        node: DoWhileStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_with_statement(
        &mut self,
        node: WithStatement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_with_statement(&mut self, node: WithStatement, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_formal_parameters(
        &mut self,
        node: FormalParameters,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_formal_parameters(
        &mut self,
        node: FormalParameters,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_formal_parameter(
        &mut self,
        node: FormalParameter,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_formal_parameter(
        &mut self,
        node: FormalParameter,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_array_expression(
        &mut self,
        node: ArrayExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_array_expression(
        &mut self,
        node: ArrayExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_spread_element(
        &mut self,
        node: SpreadElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_spread_element(&mut self, node: SpreadElement, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_parenthesized_expression(
        &mut self,
        node: ParenthesizedExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_parenthesized_expression(
        &mut self,
        node: ParenthesizedExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_sequence_expression(
        &mut self,
        node: SequenceExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_sequence_expression(
        &mut self,
        node: SequenceExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_call_expression(&mut self, node: CallExpression, ctx: &mut TraverseCtx<'_, 'a, State>) {
    }
    #[inline]
    fn enter_chain_expression(
        &mut self,
        node: ChainExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_chain_expression(
        &mut self,
        node: ChainExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_tagged_template_expression(
        &mut self,
        node: TaggedTemplateExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_tagged_template_expression(
        &mut self,
        node: TaggedTemplateExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_new_expression(
        &mut self,
        node: NewExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_new_expression(&mut self, node: NewExpression, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_await_expression(
        &mut self,
        node: AwaitExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_await_expression(
        &mut self,
        node: AwaitExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_yield_expression(
        &mut self,
        node: YieldExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_yield_expression(
        &mut self,
        node: YieldExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_meta_property(&mut self, node: MetaProperty, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_import_expression(
        &mut self,
        node: ImportExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_import_expression(
        &mut self,
        node: ImportExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_import_declaration(
        &mut self,
        node: ImportDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_import_declaration(
        &mut self,
        node: ImportDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_import_specifier(
        &mut self,
        node: ImportSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_import_specifier(
        &mut self,
        node: ImportSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_import_default_specifier(
        &mut self,
        node: ImportDefaultSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_import_default_specifier(
        &mut self,
        node: ImportDefaultSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_import_namespace_specifier(
        &mut self,
        node: ImportNamespaceSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_import_namespace_specifier(
        &mut self,
        node: ImportNamespaceSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_import_attribute(
        &mut self,
        node: ImportAttribute,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_import_attribute(
        &mut self,
        node: ImportAttribute,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_export_specifier(
        &mut self,
        node: ExportSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_export_specifier(
        &mut self,
        node: ExportSpecifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_hashbang(
        &mut self,
        node: Hashbang,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_hashbang(&mut self, node: Hashbang, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_function(
        &mut self,
        node: TsFunction,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_function(&mut self, node: TsFunction, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_method_definition(
        &mut self,
        node: TsMethodDefinition,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_method_definition(
        &mut self,
        node: TsMethodDefinition,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_annotation(
        &mut self,
        node: TsTypeAnnotation,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_annotation(
        &mut self,
        node: TsTypeAnnotation,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_any_keyword(
        &mut self,
        node: TsAnyKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_any_keyword(&mut self, node: TsAnyKeyword, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_unknown_keyword(
        &mut self,
        node: TsUnknownKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_unknown_keyword(
        &mut self,
        node: TsUnknownKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_never_keyword(
        &mut self,
        node: TsNeverKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_never_keyword(
        &mut self,
        node: TsNeverKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_void_keyword(
        &mut self,
        node: TsVoidKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_void_keyword(&mut self, node: TsVoidKeyword, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_null_keyword(
        &mut self,
        node: TsNullKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_null_keyword(&mut self, node: TsNullKeyword, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_undefined_keyword(
        &mut self,
        node: TsUndefinedKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_undefined_keyword(
        &mut self,
        node: TsUndefinedKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_string_keyword(
        &mut self,
        node: TsStringKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_string_keyword(
        &mut self,
        node: TsStringKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_number_keyword(
        &mut self,
        node: TsNumberKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_number_keyword(
        &mut self,
        node: TsNumberKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_big_int_keyword(
        &mut self,
        node: TsBigIntKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_big_int_keyword(
        &mut self,
        node: TsBigIntKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_boolean_keyword(
        &mut self,
        node: TsBooleanKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_boolean_keyword(
        &mut self,
        node: TsBooleanKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_symbol_keyword(
        &mut self,
        node: TsSymbolKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_symbol_keyword(
        &mut self,
        node: TsSymbolKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_object_keyword(
        &mut self,
        node: TsObjectKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_object_keyword(
        &mut self,
        node: TsObjectKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_intrinsic_keyword(
        &mut self,
        node: TsIntrinsicKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_intrinsic_keyword(
        &mut self,
        node: TsIntrinsicKeyword,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_this_type(
        &mut self,
        node: TsThisType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_this_type(&mut self, node: TsThisType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_type_reference(
        &mut self,
        node: TsTypeReference,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_reference(
        &mut self,
        node: TsTypeReference,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_qualified_name(
        &mut self,
        node: TsQualifiedName,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_qualified_name(
        &mut self,
        node: TsQualifiedName,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_query(
        &mut self,
        node: TsTypeQuery,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_query(&mut self, node: TsTypeQuery, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_import_type(
        &mut self,
        node: TsImportType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_import_type(&mut self, node: TsImportType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_type_parameter(
        &mut self,
        node: TsTypeParameter,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_parameter(
        &mut self,
        node: TsTypeParameter,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_parameter_instantiation(
        &mut self,
        node: TsTypeParameterInstantiation,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_parameter_instantiation(
        &mut self,
        node: TsTypeParameterInstantiation,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_literal_type(
        &mut self,
        node: TsLiteralType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_literal_type(&mut self, node: TsLiteralType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_template_literal_type(
        &mut self,
        node: TsTemplateLiteralType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_template_literal_type(
        &mut self,
        node: TsTemplateLiteralType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_array_type(
        &mut self,
        node: TsArrayType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_array_type(&mut self, node: TsArrayType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_indexed_access_type(
        &mut self,
        node: TsIndexedAccessType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_indexed_access_type(
        &mut self,
        node: TsIndexedAccessType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_tuple_type(
        &mut self,
        node: TsTupleType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_tuple_type(&mut self, node: TsTupleType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_named_tuple_member(
        &mut self,
        node: TsNamedTupleMember,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_named_tuple_member(
        &mut self,
        node: TsNamedTupleMember,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_optional_type(
        &mut self,
        node: TsOptionalType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_optional_type(
        &mut self,
        node: TsOptionalType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_rest_type(
        &mut self,
        node: TsRestType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_rest_type(&mut self, node: TsRestType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_js_doc_nullable_type(
        &mut self,
        node: TsJsDocNullableType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_js_doc_nullable_type(
        &mut self,
        node: TsJsDocNullableType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_js_doc_non_nullable_type(
        &mut self,
        node: TsJsDocNonNullableType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_js_doc_non_nullable_type(
        &mut self,
        node: TsJsDocNonNullableType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_js_doc_unknown_type(
        &mut self,
        node: TsJsDocUnknownType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_js_doc_unknown_type(
        &mut self,
        node: TsJsDocUnknownType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_union_type(
        &mut self,
        node: TsUnionType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_union_type(&mut self, node: TsUnionType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_intersection_type(
        &mut self,
        node: TsIntersectionType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_intersection_type(
        &mut self,
        node: TsIntersectionType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_conditional_type(
        &mut self,
        node: TsConditionalType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_conditional_type(
        &mut self,
        node: TsConditionalType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_infer_type(
        &mut self,
        node: TsInferType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_infer_type(&mut self, node: TsInferType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_type_operator(
        &mut self,
        node: TsTypeOperator,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_operator(
        &mut self,
        node: TsTypeOperator,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_parenthesized_type(
        &mut self,
        node: TsParenthesizedType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_parenthesized_type(
        &mut self,
        node: TsParenthesizedType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_function_type(
        &mut self,
        node: TsFunctionType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_function_type(
        &mut self,
        node: TsFunctionType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_constructor_type(
        &mut self,
        node: TsConstructorType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_constructor_type(
        &mut self,
        node: TsConstructorType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_predicate(
        &mut self,
        node: TsTypePredicate,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_predicate(
        &mut self,
        node: TsTypePredicate,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_literal(
        &mut self,
        node: TsTypeLiteral,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_literal(&mut self, node: TsTypeLiteral, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_mapped_type(
        &mut self,
        node: TsMappedType,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_mapped_type(&mut self, node: TsMappedType, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_property_signature(
        &mut self,
        node: TsPropertySignature,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_property_signature(
        &mut self,
        node: TsPropertySignature,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_method_signature(
        &mut self,
        node: TsMethodSignature,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_method_signature(
        &mut self,
        node: TsMethodSignature,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_index_signature(
        &mut self,
        node: TsIndexSignature,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_index_signature(
        &mut self,
        node: TsIndexSignature,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_interface_body(
        &mut self,
        node: TsInterfaceBody,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_interface_body(
        &mut self,
        node: TsInterfaceBody,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_interface_heritage(
        &mut self,
        node: TsInterfaceHeritage,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_interface_heritage(
        &mut self,
        node: TsInterfaceHeritage,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_class_implements(
        &mut self,
        node: TsClassImplements,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_class_implements(
        &mut self,
        node: TsClassImplements,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_enum_body(
        &mut self,
        node: TsEnumBody,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_enum_body(&mut self, node: TsEnumBody, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_enum_member(
        &mut self,
        node: TsEnumMember,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_enum_member(&mut self, node: TsEnumMember, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_module_block(&mut self, node: TsModuleBlock, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_ts_global_declaration(
        &mut self,
        node: TsGlobalDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_global_declaration(
        &mut self,
        node: TsGlobalDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_parameter_property(
        &mut self,
        node: TsParameterProperty,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_parameter_property(
        &mut self,
        node: TsParameterProperty,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_this_parameter(
        &mut self,
        node: TsThisParameter,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_this_parameter(
        &mut self,
        node: TsThisParameter,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_as_expression(
        &mut self,
        node: TsAsExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_as_expression(
        &mut self,
        node: TsAsExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_satisfies_expression(
        &mut self,
        node: TsSatisfiesExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_satisfies_expression(
        &mut self,
        node: TsSatisfiesExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_type_assertion(
        &mut self,
        node: TsTypeAssertion,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_type_assertion(
        &mut self,
        node: TsTypeAssertion,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_non_null_expression(
        &mut self,
        node: TsNonNullExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_non_null_expression(
        &mut self,
        node: TsNonNullExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_instantiation_expression(
        &mut self,
        node: TsInstantiationExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_instantiation_expression(
        &mut self,
        node: TsInstantiationExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_export_assignment(
        &mut self,
        node: TsExportAssignment,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_export_assignment(
        &mut self,
        node: TsExportAssignment,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_namespace_export_declaration(
        &mut self,
        node: TsNamespaceExportDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_namespace_export_declaration(
        &mut self,
        node: TsNamespaceExportDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_ts_external_module_reference(
        &mut self,
        node: TsExternalModuleReference,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_ts_external_module_reference(
        &mut self,
        node: TsExternalModuleReference,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_element(
        &mut self,
        node: JsxElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_element(&mut self, node: JsxElement, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_jsx_opening_element(
        &mut self,
        node: JsxOpeningElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_opening_element(
        &mut self,
        node: JsxOpeningElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_closing_element(
        &mut self,
        node: JsxClosingElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_closing_element(
        &mut self,
        node: JsxClosingElement,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_fragment(
        &mut self,
        node: JsxFragment,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_fragment(&mut self, node: JsxFragment, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_jsx_opening_fragment(
        &mut self,
        node: JsxOpeningFragment,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_opening_fragment(
        &mut self,
        node: JsxOpeningFragment,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_closing_fragment(
        &mut self,
        node: JsxClosingFragment,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_closing_fragment(
        &mut self,
        node: JsxClosingFragment,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_identifier(
        &mut self,
        node: JsxIdentifier,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_identifier(&mut self, node: JsxIdentifier, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_jsx_namespaced_name(
        &mut self,
        node: JsxNamespacedName,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_namespaced_name(
        &mut self,
        node: JsxNamespacedName,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_member_expression(
        &mut self,
        node: JsxMemberExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_member_expression(
        &mut self,
        node: JsxMemberExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_attribute(
        &mut self,
        node: JsxAttribute,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_attribute(&mut self, node: JsxAttribute, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_jsx_spread_attribute(
        &mut self,
        node: JsxSpreadAttribute,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_spread_attribute(
        &mut self,
        node: JsxSpreadAttribute,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_expression_container(
        &mut self,
        node: JsxExpressionContainer,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_expression_container(
        &mut self,
        node: JsxExpressionContainer,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_empty_expression(
        &mut self,
        node: JsxEmptyExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_empty_expression(
        &mut self,
        node: JsxEmptyExpression,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
    #[inline]
    fn enter_jsx_text(
        &mut self,
        node: JsxText,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_text(&mut self, node: JsxText, ctx: &mut TraverseCtx<'_, 'a, State>) {}
    #[inline]
    fn enter_jsx_spread_child(
        &mut self,
        node: JsxSpreadChild,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        TraverseAction::Proceed
    }
    #[inline]
    fn exit_jsx_spread_child(
        &mut self,
        node: JsxSpreadChild,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) {
    }
}
pub trait TraverseWith<'a, State = ()> {
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction;
}
impl<'a, State> TraverseWith<'a, State> for Program {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Stmt {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Decl {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Expr {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BindingPattern {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for AssignmentTarget {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for SimpleAssignmentTarget {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for PropertyKey {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ModuleExportName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Argument {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ObjectPropertyKind {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ForStatementInit {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ForStatementLeft {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ArrowFunctionBody {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for FormalParameterItem {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for FormalParameterPattern {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ClassElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ImportDeclarationSpecifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ExportDefaultDeclarationKind {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Super {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ThisExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for StringLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for NumericLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BigIntLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BooleanLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for NullLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for RegExpLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TemplateLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TemplateElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for IdentifierReference {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for PrivateIdentifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BindingIdentifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for IdentifierName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for LabelIdentifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for AssignmentPattern {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BindingRestElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ArrayPattern {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ObjectPattern {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BindingProperty {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BinaryExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for LogicalExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ConditionalExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for UnaryExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for UpdateExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for AssignmentExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for VariableDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for VariableDeclarator {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for IfStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for SwitchStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for SwitchCase {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ForStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ForInStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ForOfStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TryStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for CatchClause {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BreakStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ContinueStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for LabeledStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ExpressionStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ReturnStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ThrowStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for DebuggerStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for EmptyStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Directive {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Decorator {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Class {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ClassBody {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for MethodDefinition {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for PropertyDefinition {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for StaticBlock {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Function {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for FunctionBody {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for BlockStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for WhileStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for DoWhileStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for WithStatement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for FormalParameters {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for FormalParameter {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ArrayExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ObjectExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for SpreadElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ObjectProperty {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ParenthesizedExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ArrowFunctionExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for SequenceExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for MemberExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for CallExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ChainExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TaggedTemplateExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for NewExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for AwaitExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for YieldExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for MetaProperty {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ImportExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ImportDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ImportSpecifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ImportDefaultSpecifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ImportNamespaceSpecifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ImportAttribute {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ExportNamedDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ExportDefaultDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ExportAllDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for ExportSpecifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for Hashbang {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsQualifiedNameLeft {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsImportTypeQualifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeQueryExprName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTupleElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypePredicateName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsEnumMemberName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsModuleName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsModuleBody {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsModuleReference {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsFunction {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsMethodDefinition {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeAnnotation {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsAnyKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsUnknownKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsNeverKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsVoidKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsNullKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsUndefinedKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsStringKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsNumberKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsBigIntKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsBooleanKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsSymbolKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsObjectKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsIntrinsicKeyword {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsThisType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeReference {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsQualifiedName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeQuery {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsImportType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeParameter {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeParameterDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeParameterInstantiation {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsLiteralType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTemplateLiteralType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsArrayType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsIndexedAccessType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTupleType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsNamedTupleMember {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsOptionalType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsRestType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsJsDocNullableType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsJsDocNonNullableType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsJsDocUnknownType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsUnionType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsIntersectionType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsConditionalType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsInferType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeOperator {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsParenthesizedType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsFunctionType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsConstructorType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypePredicate {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeLiteral {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsMappedType {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsPropertySignature {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsMethodSignature {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsCallSignatureDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsConstructSignatureDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsIndexSignature {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeAliasDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsInterfaceDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsInterfaceBody {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsInterfaceHeritage {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsClassImplements {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsEnumDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsEnumBody {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsEnumMember {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsModuleDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsModuleBlock {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsGlobalDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsParameterProperty {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsThisParameter {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsAsExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsSatisfiesExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsTypeAssertion {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsNonNullExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsInstantiationExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsExportAssignment {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsNamespaceExportDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsImportEqualsDeclaration {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TsExternalModuleReference {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxElementName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxMemberExpressionObject {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxAttributeName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxAttributeItem {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxAttributeValue {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxChild {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxOpeningElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxClosingElement {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxFragment {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxOpeningFragment {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxClosingFragment {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxIdentifier {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxNamespacedName {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxMemberExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxAttribute {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxSpreadAttribute {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxExpressionContainer {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxEmptyExpression {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxText {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for JsxSpreadChild {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        walk_node(self.node_id(), traverser, ctx)
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<Directive> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <Directive as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<Stmt> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<Hashbang> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <Hashbang as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TemplateElement> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TemplateElement as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<Expr> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<Decorator> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <Decorator as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<TsTypeAnnotation> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <TsTypeAnnotation as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<BindingPattern> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <BindingPattern as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<Option<BindingPattern>> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <Option<BindingPattern> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<BindingRestElement> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <BindingRestElement as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<BindingProperty> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <BindingProperty as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<VariableDeclarator> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <VariableDeclarator as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<Expr> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <Expr as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<Stmt> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <Stmt as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<SwitchCase> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <SwitchCase as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<ForStatementInit> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <ForStatementInit as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<CatchClause> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <CatchClause as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<BlockStatement> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <BlockStatement as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<LabelIdentifier> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <LabelIdentifier as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<BindingIdentifier> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<TsTypeParameterDeclaration> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <TsTypeParameterDeclaration as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<TsTypeParameterInstantiation> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <TsTypeParameterInstantiation as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TsClassImplements> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TsClassImplements as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<ClassElement> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <ClassElement as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<FormalParameterItem> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <FormalParameterItem as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<Argument> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <Argument as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<Option<Argument>> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <Option<Argument> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<ObjectPropertyKind> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <ObjectPropertyKind as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<Argument> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <Argument as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<ImportDeclarationSpecifier> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <ImportDeclarationSpecifier as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<ImportAttribute> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <ImportAttribute as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<Decl> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <Decl as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<ExportSpecifier> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <ExportSpecifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<StringLiteral> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <StringLiteral as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<ModuleExportName> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <ModuleExportName as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<TsImportTypeQualifier> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <TsImportTypeQualifier as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<TsType> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <TsType as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TsTypeParameter> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TsTypeParameter as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TsType> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TsTupleElement> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TsTupleElement as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TsTypeElement> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TsTypeElement as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<BindingIdentifier> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TsInterfaceHeritage> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TsInterfaceHeritage as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<TsEnumMember> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <TsEnumMember as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<TsModuleBody> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <TsModuleBody as TraverseWith<'a, State>>::traverse_with(it, traverser, ctx);
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<JsxChild> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <JsxChild as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<JsxClosingElement> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <JsxClosingElement as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for TypedSubRange<JsxAttributeItem> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        for child_idx in self.iter() {
            let child = ctx.ast.get_node_in_sub_range(child_idx);
            if <JsxAttributeItem as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                return TraverseAction::Stop;
            }
        }
        TraverseAction::Proceed
    }
}
impl<'a, State> TraverseWith<'a, State> for Option<JsxAttributeValue> {
    #[inline]
    fn traverse_with<T: ?Sized + Traverse<'a, State>>(
        self,
        traverser: &mut T,
        ctx: &mut TraverseCtx<'_, 'a, State>,
    ) -> TraverseAction {
        if let Some(it) = self {
            return <JsxAttributeValue as TraverseWith<'a, State>>::traverse_with(
                it, traverser, ctx,
            );
        }
        TraverseAction::Proceed
    }
}
fn walk_node<'a, State, T: ?Sized + Traverse<'a, State>>(
    node_id: NodeId,
    traverser: &mut T,
    ctx: &mut TraverseCtx<'_, 'a, State>,
) -> TraverseAction {
    ctx.push_node(node_id);
    let entered_data = ctx.ast.kind_data(node_id);
    let mut action = traverser.enter_node(node_id, ctx);
    if action == TraverseAction::Proceed {
        action = match entered_data {
            NodeKindData::Program(node) => traverser.enter_program(node, ctx),
            NodeKindData::Super(node) => traverser.enter_super(node, ctx),
            NodeKindData::ThisExpression(node) => traverser.enter_this_expression(node, ctx),
            NodeKindData::StringLiteral(node) => traverser.enter_string_literal(node, ctx),
            NodeKindData::NumericLiteral(node) => traverser.enter_numeric_literal(node, ctx),
            NodeKindData::BigIntLiteral(node) => traverser.enter_big_int_literal(node, ctx),
            NodeKindData::BooleanLiteral(node) => traverser.enter_boolean_literal(node, ctx),
            NodeKindData::NullLiteral(node) => traverser.enter_null_literal(node, ctx),
            NodeKindData::RegExpLiteral(node) => traverser.enter_reg_exp_literal(node, ctx),
            NodeKindData::TemplateLiteral(node) => traverser.enter_template_literal(node, ctx),
            NodeKindData::TemplateElement(node) => traverser.enter_template_element(node, ctx),
            NodeKindData::IdentifierReference(node) => {
                traverser.enter_identifier_reference(node, ctx)
            }
            NodeKindData::PrivateIdentifier(node) => traverser.enter_private_identifier(node, ctx),
            NodeKindData::BindingIdentifier(node) => traverser.enter_binding_identifier(node, ctx),
            NodeKindData::IdentifierName(node) => traverser.enter_identifier_name(node, ctx),
            NodeKindData::LabelIdentifier(node) => traverser.enter_label_identifier(node, ctx),
            NodeKindData::AssignmentPattern(node) => traverser.enter_assignment_pattern(node, ctx),
            NodeKindData::BindingRestElement(node) => {
                traverser.enter_binding_rest_element(node, ctx)
            }
            NodeKindData::ArrayPattern(node) => traverser.enter_array_pattern(node, ctx),
            NodeKindData::ObjectPattern(node) => traverser.enter_object_pattern(node, ctx),
            NodeKindData::BindingProperty(node) => traverser.enter_binding_property(node, ctx),
            NodeKindData::BinaryExpression(node) => traverser.enter_binary_expression(node, ctx),
            NodeKindData::LogicalExpression(node) => traverser.enter_logical_expression(node, ctx),
            NodeKindData::ConditionalExpression(node) => {
                traverser.enter_conditional_expression(node, ctx)
            }
            NodeKindData::UnaryExpression(node) => traverser.enter_unary_expression(node, ctx),
            NodeKindData::UpdateExpression(node) => traverser.enter_update_expression(node, ctx),
            NodeKindData::AssignmentExpression(node) => {
                traverser.enter_assignment_expression(node, ctx)
            }
            NodeKindData::VariableDeclaration(node) => {
                traverser.enter_variable_declaration(node, ctx)
            }
            NodeKindData::VariableDeclarator(node) => {
                traverser.enter_variable_declarator(node, ctx)
            }
            NodeKindData::IfStatement(node) => traverser.enter_if_statement(node, ctx),
            NodeKindData::SwitchStatement(node) => traverser.enter_switch_statement(node, ctx),
            NodeKindData::SwitchCase(node) => traverser.enter_switch_case(node, ctx),
            NodeKindData::ForStatement(node) => traverser.enter_for_statement(node, ctx),
            NodeKindData::ForInStatement(node) => traverser.enter_for_in_statement(node, ctx),
            NodeKindData::ForOfStatement(node) => traverser.enter_for_of_statement(node, ctx),
            NodeKindData::TryStatement(node) => traverser.enter_try_statement(node, ctx),
            NodeKindData::CatchClause(node) => traverser.enter_catch_clause(node, ctx),
            NodeKindData::BreakStatement(node) => traverser.enter_break_statement(node, ctx),
            NodeKindData::ContinueStatement(node) => traverser.enter_continue_statement(node, ctx),
            NodeKindData::LabeledStatement(node) => traverser.enter_labeled_statement(node, ctx),
            NodeKindData::ExpressionStatement(node) => {
                traverser.enter_expression_statement(node, ctx)
            }
            NodeKindData::ReturnStatement(node) => traverser.enter_return_statement(node, ctx),
            NodeKindData::ThrowStatement(node) => traverser.enter_throw_statement(node, ctx),
            NodeKindData::DebuggerStatement(node) => traverser.enter_debugger_statement(node, ctx),
            NodeKindData::EmptyStatement(node) => traverser.enter_empty_statement(node, ctx),
            NodeKindData::Directive(node) => traverser.enter_directive(node, ctx),
            NodeKindData::Decorator(node) => traverser.enter_decorator(node, ctx),
            NodeKindData::Class(node) => traverser.enter_class(node, ctx),
            NodeKindData::ClassBody(node) => traverser.enter_class_body(node, ctx),
            NodeKindData::MethodDefinition(node) => traverser.enter_method_definition(node, ctx),
            NodeKindData::PropertyDefinition(node) => {
                traverser.enter_property_definition(node, ctx)
            }
            NodeKindData::StaticBlock(node) => traverser.enter_static_block(node, ctx),
            NodeKindData::Function(node) => traverser.enter_function(node, ctx),
            NodeKindData::FunctionBody(node) => traverser.enter_function_body(node, ctx),
            NodeKindData::BlockStatement(node) => traverser.enter_block_statement(node, ctx),
            NodeKindData::WhileStatement(node) => traverser.enter_while_statement(node, ctx),
            NodeKindData::DoWhileStatement(node) => traverser.enter_do_while_statement(node, ctx),
            NodeKindData::WithStatement(node) => traverser.enter_with_statement(node, ctx),
            NodeKindData::FormalParameters(node) => traverser.enter_formal_parameters(node, ctx),
            NodeKindData::FormalParameter(node) => traverser.enter_formal_parameter(node, ctx),
            NodeKindData::ArrayExpression(node) => traverser.enter_array_expression(node, ctx),
            NodeKindData::ObjectExpression(node) => traverser.enter_object_expression(node, ctx),
            NodeKindData::SpreadElement(node) => traverser.enter_spread_element(node, ctx),
            NodeKindData::ObjectProperty(node) => traverser.enter_object_property(node, ctx),
            NodeKindData::ParenthesizedExpression(node) => {
                traverser.enter_parenthesized_expression(node, ctx)
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                traverser.enter_arrow_function_expression(node, ctx)
            }
            NodeKindData::SequenceExpression(node) => {
                traverser.enter_sequence_expression(node, ctx)
            }
            NodeKindData::MemberExpression(node) => traverser.enter_member_expression(node, ctx),
            NodeKindData::CallExpression(node) => traverser.enter_call_expression(node, ctx),
            NodeKindData::ChainExpression(node) => traverser.enter_chain_expression(node, ctx),
            NodeKindData::TaggedTemplateExpression(node) => {
                traverser.enter_tagged_template_expression(node, ctx)
            }
            NodeKindData::NewExpression(node) => traverser.enter_new_expression(node, ctx),
            NodeKindData::AwaitExpression(node) => traverser.enter_await_expression(node, ctx),
            NodeKindData::YieldExpression(node) => traverser.enter_yield_expression(node, ctx),
            NodeKindData::MetaProperty(node) => traverser.enter_meta_property(node, ctx),
            NodeKindData::ImportExpression(node) => traverser.enter_import_expression(node, ctx),
            NodeKindData::ImportDeclaration(node) => traverser.enter_import_declaration(node, ctx),
            NodeKindData::ImportSpecifier(node) => traverser.enter_import_specifier(node, ctx),
            NodeKindData::ImportDefaultSpecifier(node) => {
                traverser.enter_import_default_specifier(node, ctx)
            }
            NodeKindData::ImportNamespaceSpecifier(node) => {
                traverser.enter_import_namespace_specifier(node, ctx)
            }
            NodeKindData::ImportAttribute(node) => traverser.enter_import_attribute(node, ctx),
            NodeKindData::ExportNamedDeclaration(node) => {
                traverser.enter_export_named_declaration(node, ctx)
            }
            NodeKindData::ExportDefaultDeclaration(node) => {
                traverser.enter_export_default_declaration(node, ctx)
            }
            NodeKindData::ExportAllDeclaration(node) => {
                traverser.enter_export_all_declaration(node, ctx)
            }
            NodeKindData::ExportSpecifier(node) => traverser.enter_export_specifier(node, ctx),
            NodeKindData::Hashbang(node) => traverser.enter_hashbang(node, ctx),
            NodeKindData::TsFunction(node) => traverser.enter_ts_function(node, ctx),
            NodeKindData::TsMethodDefinition(node) => {
                traverser.enter_ts_method_definition(node, ctx)
            }
            NodeKindData::TsTypeAnnotation(node) => traverser.enter_ts_type_annotation(node, ctx),
            NodeKindData::TsAnyKeyword(node) => traverser.enter_ts_any_keyword(node, ctx),
            NodeKindData::TsUnknownKeyword(node) => traverser.enter_ts_unknown_keyword(node, ctx),
            NodeKindData::TsNeverKeyword(node) => traverser.enter_ts_never_keyword(node, ctx),
            NodeKindData::TsVoidKeyword(node) => traverser.enter_ts_void_keyword(node, ctx),
            NodeKindData::TsNullKeyword(node) => traverser.enter_ts_null_keyword(node, ctx),
            NodeKindData::TsUndefinedKeyword(node) => {
                traverser.enter_ts_undefined_keyword(node, ctx)
            }
            NodeKindData::TsStringKeyword(node) => traverser.enter_ts_string_keyword(node, ctx),
            NodeKindData::TsNumberKeyword(node) => traverser.enter_ts_number_keyword(node, ctx),
            NodeKindData::TsBigIntKeyword(node) => traverser.enter_ts_big_int_keyword(node, ctx),
            NodeKindData::TsBooleanKeyword(node) => traverser.enter_ts_boolean_keyword(node, ctx),
            NodeKindData::TsSymbolKeyword(node) => traverser.enter_ts_symbol_keyword(node, ctx),
            NodeKindData::TsObjectKeyword(node) => traverser.enter_ts_object_keyword(node, ctx),
            NodeKindData::TsIntrinsicKeyword(node) => {
                traverser.enter_ts_intrinsic_keyword(node, ctx)
            }
            NodeKindData::TsThisType(node) => traverser.enter_ts_this_type(node, ctx),
            NodeKindData::TsTypeReference(node) => traverser.enter_ts_type_reference(node, ctx),
            NodeKindData::TsQualifiedName(node) => traverser.enter_ts_qualified_name(node, ctx),
            NodeKindData::TsTypeQuery(node) => traverser.enter_ts_type_query(node, ctx),
            NodeKindData::TsImportType(node) => traverser.enter_ts_import_type(node, ctx),
            NodeKindData::TsTypeParameter(node) => traverser.enter_ts_type_parameter(node, ctx),
            NodeKindData::TsTypeParameterDeclaration(node) => {
                traverser.enter_ts_type_parameter_declaration(node, ctx)
            }
            NodeKindData::TsTypeParameterInstantiation(node) => {
                traverser.enter_ts_type_parameter_instantiation(node, ctx)
            }
            NodeKindData::TsLiteralType(node) => traverser.enter_ts_literal_type(node, ctx),
            NodeKindData::TsTemplateLiteralType(node) => {
                traverser.enter_ts_template_literal_type(node, ctx)
            }
            NodeKindData::TsArrayType(node) => traverser.enter_ts_array_type(node, ctx),
            NodeKindData::TsIndexedAccessType(node) => {
                traverser.enter_ts_indexed_access_type(node, ctx)
            }
            NodeKindData::TsTupleType(node) => traverser.enter_ts_tuple_type(node, ctx),
            NodeKindData::TsNamedTupleMember(node) => {
                traverser.enter_ts_named_tuple_member(node, ctx)
            }
            NodeKindData::TsOptionalType(node) => traverser.enter_ts_optional_type(node, ctx),
            NodeKindData::TsRestType(node) => traverser.enter_ts_rest_type(node, ctx),
            NodeKindData::TsJsDocNullableType(node) => {
                traverser.enter_ts_js_doc_nullable_type(node, ctx)
            }
            NodeKindData::TsJsDocNonNullableType(node) => {
                traverser.enter_ts_js_doc_non_nullable_type(node, ctx)
            }
            NodeKindData::TsJsDocUnknownType(node) => {
                traverser.enter_ts_js_doc_unknown_type(node, ctx)
            }
            NodeKindData::TsUnionType(node) => traverser.enter_ts_union_type(node, ctx),
            NodeKindData::TsIntersectionType(node) => {
                traverser.enter_ts_intersection_type(node, ctx)
            }
            NodeKindData::TsConditionalType(node) => traverser.enter_ts_conditional_type(node, ctx),
            NodeKindData::TsInferType(node) => traverser.enter_ts_infer_type(node, ctx),
            NodeKindData::TsTypeOperator(node) => traverser.enter_ts_type_operator(node, ctx),
            NodeKindData::TsParenthesizedType(node) => {
                traverser.enter_ts_parenthesized_type(node, ctx)
            }
            NodeKindData::TsFunctionType(node) => traverser.enter_ts_function_type(node, ctx),
            NodeKindData::TsConstructorType(node) => traverser.enter_ts_constructor_type(node, ctx),
            NodeKindData::TsTypePredicate(node) => traverser.enter_ts_type_predicate(node, ctx),
            NodeKindData::TsTypeLiteral(node) => traverser.enter_ts_type_literal(node, ctx),
            NodeKindData::TsMappedType(node) => traverser.enter_ts_mapped_type(node, ctx),
            NodeKindData::TsPropertySignature(node) => {
                traverser.enter_ts_property_signature(node, ctx)
            }
            NodeKindData::TsMethodSignature(node) => traverser.enter_ts_method_signature(node, ctx),
            NodeKindData::TsCallSignatureDeclaration(node) => {
                traverser.enter_ts_call_signature_declaration(node, ctx)
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                traverser.enter_ts_construct_signature_declaration(node, ctx)
            }
            NodeKindData::TsIndexSignature(node) => traverser.enter_ts_index_signature(node, ctx),
            NodeKindData::TsTypeAliasDeclaration(node) => {
                traverser.enter_ts_type_alias_declaration(node, ctx)
            }
            NodeKindData::TsInterfaceDeclaration(node) => {
                traverser.enter_ts_interface_declaration(node, ctx)
            }
            NodeKindData::TsInterfaceBody(node) => traverser.enter_ts_interface_body(node, ctx),
            NodeKindData::TsInterfaceHeritage(node) => {
                traverser.enter_ts_interface_heritage(node, ctx)
            }
            NodeKindData::TsClassImplements(node) => traverser.enter_ts_class_implements(node, ctx),
            NodeKindData::TsEnumDeclaration(node) => traverser.enter_ts_enum_declaration(node, ctx),
            NodeKindData::TsEnumBody(node) => traverser.enter_ts_enum_body(node, ctx),
            NodeKindData::TsEnumMember(node) => traverser.enter_ts_enum_member(node, ctx),
            NodeKindData::TsModuleDeclaration(node) => {
                traverser.enter_ts_module_declaration(node, ctx)
            }
            NodeKindData::TsModuleBlock(node) => traverser.enter_ts_module_block(node, ctx),
            NodeKindData::TsGlobalDeclaration(node) => {
                traverser.enter_ts_global_declaration(node, ctx)
            }
            NodeKindData::TsParameterProperty(node) => {
                traverser.enter_ts_parameter_property(node, ctx)
            }
            NodeKindData::TsThisParameter(node) => traverser.enter_ts_this_parameter(node, ctx),
            NodeKindData::TsAsExpression(node) => traverser.enter_ts_as_expression(node, ctx),
            NodeKindData::TsSatisfiesExpression(node) => {
                traverser.enter_ts_satisfies_expression(node, ctx)
            }
            NodeKindData::TsTypeAssertion(node) => traverser.enter_ts_type_assertion(node, ctx),
            NodeKindData::TsNonNullExpression(node) => {
                traverser.enter_ts_non_null_expression(node, ctx)
            }
            NodeKindData::TsInstantiationExpression(node) => {
                traverser.enter_ts_instantiation_expression(node, ctx)
            }
            NodeKindData::TsExportAssignment(node) => {
                traverser.enter_ts_export_assignment(node, ctx)
            }
            NodeKindData::TsNamespaceExportDeclaration(node) => {
                traverser.enter_ts_namespace_export_declaration(node, ctx)
            }
            NodeKindData::TsImportEqualsDeclaration(node) => {
                traverser.enter_ts_import_equals_declaration(node, ctx)
            }
            NodeKindData::TsExternalModuleReference(node) => {
                traverser.enter_ts_external_module_reference(node, ctx)
            }
            NodeKindData::JsxElement(node) => traverser.enter_jsx_element(node, ctx),
            NodeKindData::JsxOpeningElement(node) => traverser.enter_jsx_opening_element(node, ctx),
            NodeKindData::JsxClosingElement(node) => traverser.enter_jsx_closing_element(node, ctx),
            NodeKindData::JsxFragment(node) => traverser.enter_jsx_fragment(node, ctx),
            NodeKindData::JsxOpeningFragment(node) => {
                traverser.enter_jsx_opening_fragment(node, ctx)
            }
            NodeKindData::JsxClosingFragment(node) => {
                traverser.enter_jsx_closing_fragment(node, ctx)
            }
            NodeKindData::JsxIdentifier(node) => traverser.enter_jsx_identifier(node, ctx),
            NodeKindData::JsxNamespacedName(node) => traverser.enter_jsx_namespaced_name(node, ctx),
            NodeKindData::JsxMemberExpression(node) => {
                traverser.enter_jsx_member_expression(node, ctx)
            }
            NodeKindData::JsxAttribute(node) => traverser.enter_jsx_attribute(node, ctx),
            NodeKindData::JsxSpreadAttribute(node) => {
                traverser.enter_jsx_spread_attribute(node, ctx)
            }
            NodeKindData::JsxExpressionContainer(node) => {
                traverser.enter_jsx_expression_container(node, ctx)
            }
            NodeKindData::JsxEmptyExpression(node) => {
                traverser.enter_jsx_empty_expression(node, ctx)
            }
            NodeKindData::JsxText(node) => traverser.enter_jsx_text(node, ctx),
            NodeKindData::JsxSpreadChild(node) => traverser.enter_jsx_spread_child(node, ctx),
        };
    }
    match action {
        TraverseAction::Skip => {
            ctx.pop_node(node_id);
            return TraverseAction::Proceed;
        }
        TraverseAction::Stop => {
            ctx.pop_node(node_id);
            return TraverseAction::Stop;
        }
        TraverseAction::Proceed => {}
    }
    let current_kind = ctx.node_kind(node_id);
    let current_data = ctx.ast.kind_data(node_id);
    let action = match current_data {
        NodeKindData::Program(node) => 'traverse_children: {
            let child = node.directives(ctx.ast);
            if <TypedSubRange<Directive> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <TypedSubRange<Stmt> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.hashbang(ctx.ast);
            if <Option<Hashbang> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::Super(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::ThisExpression(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::StringLiteral(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::NumericLiteral(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::BigIntLiteral(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::BooleanLiteral(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::NullLiteral(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::RegExpLiteral(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TemplateLiteral(node) => 'traverse_children: {
            let child = node.quasis(ctx.ast);
            if <TypedSubRange<TemplateElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.expressions(ctx.ast);
            if <TypedSubRange<Expr> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TemplateElement(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::IdentifierReference(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::PrivateIdentifier(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::BindingIdentifier(node) => 'traverse_children: {
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::IdentifierName(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::LabelIdentifier(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::AssignmentPattern(node) => 'traverse_children: {
            let child = node.left(ctx.ast);
            if <BindingPattern as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.right(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::BindingRestElement(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <BindingPattern as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ArrayPattern(node) => 'traverse_children: {
            let child = node.elements(ctx.ast);
            if <TypedSubRange<Option<BindingPattern>> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.rest(ctx.ast);
            if <Option<BindingRestElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ObjectPattern(node) => 'traverse_children: {
            let child = node.properties(ctx.ast);
            if <TypedSubRange<BindingProperty> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.rest(ctx.ast);
            if <Option<BindingRestElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::BindingProperty(node) => 'traverse_children: {
            let child = node.key(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.value(ctx.ast);
            if <BindingPattern as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::BinaryExpression(node) => 'traverse_children: {
            let child = node.left(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.right(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::LogicalExpression(node) => 'traverse_children: {
            let child = node.left(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.right(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ConditionalExpression(node) => 'traverse_children: {
            let child = node.test(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.consequent(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.alternate(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::UnaryExpression(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::UpdateExpression(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <SimpleAssignmentTarget as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::AssignmentExpression(node) => 'traverse_children: {
            let child = node.left(ctx.ast);
            if <AssignmentTarget as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.right(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::VariableDeclaration(node) => 'traverse_children: {
            let child = node.declarators(ctx.ast);
            if <TypedSubRange<VariableDeclarator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::VariableDeclarator(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <BindingPattern as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.init(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::IfStatement(node) => 'traverse_children: {
            let child = node.test(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.consequent(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.alternate(ctx.ast);
            if <Option<Stmt> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::SwitchStatement(node) => 'traverse_children: {
            let child = node.discriminant(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.cases(ctx.ast);
            if <TypedSubRange<SwitchCase> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::SwitchCase(node) => 'traverse_children: {
            let child = node.test(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.consequent(ctx.ast);
            if <TypedSubRange<Stmt> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ForStatement(node) => 'traverse_children: {
            let child = node.init(ctx.ast);
            if <Option<ForStatementInit> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.test(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.update(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ForInStatement(node) => 'traverse_children: {
            let child = node.left(ctx.ast);
            if <ForStatementLeft as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.right(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ForOfStatement(node) => 'traverse_children: {
            let child = node.left(ctx.ast);
            if <ForStatementLeft as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.right(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TryStatement(node) => 'traverse_children: {
            let child = node.block(ctx.ast);
            if <BlockStatement as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.handler(ctx.ast);
            if <Option<CatchClause> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.finalizer(ctx.ast);
            if <Option<BlockStatement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::CatchClause(node) => 'traverse_children: {
            let child = node.param(ctx.ast);
            if <Option<BindingPattern> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <BlockStatement as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::BreakStatement(node) => 'traverse_children: {
            let child = node.label(ctx.ast);
            if <Option<LabelIdentifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ContinueStatement(node) => 'traverse_children: {
            let child = node.label(ctx.ast);
            if <Option<LabelIdentifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::LabeledStatement(node) => 'traverse_children: {
            let child = node.label(ctx.ast);
            if <LabelIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ExpressionStatement(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ReturnStatement(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ThrowStatement(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::DebuggerStatement(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::EmptyStatement(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::Directive(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <StringLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::Decorator(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::Class(node) => 'traverse_children: {
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.id(ctx.ast);
            if <Option<BindingIdentifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.super_class(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <ClassBody as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.super_type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.implements(ctx.ast);
            if <TypedSubRange<TsClassImplements> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ClassBody(node) => 'traverse_children: {
            let child = node.body(ctx.ast);
            if <TypedSubRange<ClassElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::MethodDefinition(node) => 'traverse_children: {
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.key(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.value(ctx.ast);
            if <Function as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::PropertyDefinition(node) => 'traverse_children: {
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.key(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.value(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::StaticBlock(node) => 'traverse_children: {
            let child = node.body(ctx.ast);
            if <TypedSubRange<Stmt> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::Function(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <Option<BindingIdentifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <FunctionBody as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::FunctionBody(node) => 'traverse_children: {
            let child = node.directives(ctx.ast);
            if <TypedSubRange<Directive> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <TypedSubRange<Stmt> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::BlockStatement(node) => 'traverse_children: {
            let child = node.body(ctx.ast);
            if <TypedSubRange<Stmt> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::WhileStatement(node) => 'traverse_children: {
            let child = node.test(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::DoWhileStatement(node) => 'traverse_children: {
            let child = node.body(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.test(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::WithStatement(node) => 'traverse_children: {
            let child = node.object(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <Stmt as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::FormalParameters(node) => 'traverse_children: {
            let child = node.items(ctx.ast);
            if <TypedSubRange<FormalParameterItem> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.rest(ctx.ast);
            if <Option<BindingRestElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::FormalParameter(node) => 'traverse_children: {
            let child = node.pattern(ctx.ast);
            if <FormalParameterPattern as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ArrayExpression(node) => 'traverse_children: {
            let child = node.elements(ctx.ast);
            if <TypedSubRange<Option<Argument>> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ObjectExpression(node) => 'traverse_children: {
            let child = node.properties(ctx.ast);
            if <TypedSubRange<ObjectPropertyKind> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::SpreadElement(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ObjectProperty(node) => 'traverse_children: {
            let child = node.key(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.value(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ParenthesizedExpression(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ArrowFunctionExpression(node) => 'traverse_children: {
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <ArrowFunctionBody as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::SequenceExpression(node) => 'traverse_children: {
            let child = node.expressions(ctx.ast);
            if <TypedSubRange<Expr> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::MemberExpression(node) => 'traverse_children: {
            let child = node.object(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.property(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::CallExpression(node) => 'traverse_children: {
            let child = node.callee(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.arguments(ctx.ast);
            if <TypedSubRange<Argument> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ChainExpression(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TaggedTemplateExpression(node) => 'traverse_children: {
            let child = node.tag(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.quasi(ctx.ast);
            if <TemplateLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::NewExpression(node) => 'traverse_children: {
            let child = node.callee(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.arguments(ctx.ast);
            if <TypedSubRange<Argument> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::AwaitExpression(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::YieldExpression(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::MetaProperty(node) => 'traverse_children: {
            let child = node.meta(ctx.ast);
            if <IdentifierName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.property(ctx.ast);
            if <IdentifierName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ImportExpression(node) => 'traverse_children: {
            let child = node.source(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.options(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ImportDeclaration(node) => 'traverse_children: {
            let child = node.specifiers(ctx.ast);
            if <TypedSubRange<ImportDeclarationSpecifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.source(ctx.ast);
            if <StringLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.attributes(ctx.ast);
            if <TypedSubRange<ImportAttribute> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ImportSpecifier(node) => 'traverse_children: {
            let child = node.imported(ctx.ast);
            if <ModuleExportName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.local(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ImportDefaultSpecifier(node) => 'traverse_children: {
            let child = node.local(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ImportNamespaceSpecifier(node) => 'traverse_children: {
            let child = node.local(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ImportAttribute(node) => 'traverse_children: {
            let child = node.key(ctx.ast);
            if <ModuleExportName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.value(ctx.ast);
            if <StringLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ExportNamedDeclaration(node) => 'traverse_children: {
            let child = node.declaration(ctx.ast);
            if <Option<Decl> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.specifiers(ctx.ast);
            if <TypedSubRange<ExportSpecifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.source(ctx.ast);
            if <Option<StringLiteral> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.attributes(ctx.ast);
            if <TypedSubRange<ImportAttribute> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ExportDefaultDeclaration(node) => 'traverse_children: {
            let child = node.declaration(ctx.ast);
            if <ExportDefaultDeclarationKind as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ExportAllDeclaration(node) => 'traverse_children: {
            let child = node.exported(ctx.ast);
            if <Option<ModuleExportName> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.source(ctx.ast);
            if <StringLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.attributes(ctx.ast);
            if <TypedSubRange<ImportAttribute> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::ExportSpecifier(node) => 'traverse_children: {
            let child = node.local(ctx.ast);
            if <ModuleExportName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.exported(ctx.ast);
            if <ModuleExportName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::Hashbang(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsFunction(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <Option<BindingIdentifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsMethodDefinition(node) => 'traverse_children: {
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.key(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.value(ctx.ast);
            if <TsFunction as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeAnnotation(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsAnyKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsUnknownKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsNeverKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsVoidKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsNullKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsUndefinedKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsStringKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsNumberKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsBigIntKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsBooleanKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsSymbolKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsObjectKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsIntrinsicKeyword(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsThisType(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeReference(node) => 'traverse_children: {
            let child = node.type_name(ctx.ast);
            if <TsTypeName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsQualifiedName(node) => 'traverse_children: {
            let child = node.left(ctx.ast);
            if <TsQualifiedNameLeft as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.right(ctx.ast);
            if <IdentifierName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeQuery(node) => 'traverse_children: {
            let child = node.expr_name(ctx.ast);
            if <TsTypeQueryExprName as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsImportType(node) => 'traverse_children: {
            let child = node.source(ctx.ast);
            if <StringLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.options(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.qualifier(ctx.ast);
            if <Option<TsImportTypeQualifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeParameter(node) => 'traverse_children: {
            let child = node.name(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.constraint(ctx.ast);
            if <Option<TsType> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.default(ctx.ast);
            if <Option<TsType> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeParameterDeclaration(node) => 'traverse_children: {
            let child = node.params(ctx.ast);
            if <TypedSubRange<TsTypeParameter> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeParameterInstantiation(node) => 'traverse_children: {
            let child = node.params(ctx.ast);
            if <TypedSubRange<TsType> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsLiteralType(node) => 'traverse_children: {
            let child = node.literal(ctx.ast);
            if <TsLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTemplateLiteralType(node) => 'traverse_children: {
            let child = node.quasis(ctx.ast);
            if <TypedSubRange<TemplateElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.types(ctx.ast);
            if <TypedSubRange<TsType> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsArrayType(node) => 'traverse_children: {
            let child = node.element_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsIndexedAccessType(node) => 'traverse_children: {
            let child = node.object_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.index_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTupleType(node) => 'traverse_children: {
            let child = node.element_types(ctx.ast);
            if <TypedSubRange<TsTupleElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsNamedTupleMember(node) => 'traverse_children: {
            let child = node.label(ctx.ast);
            if <IdentifierName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.element_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsOptionalType(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsRestType(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsJsDocNullableType(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsJsDocNonNullableType(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsJsDocUnknownType(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::TsUnionType(node) => 'traverse_children: {
            let child = node.types(ctx.ast);
            if <TypedSubRange<TsType> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsIntersectionType(node) => 'traverse_children: {
            let child = node.types(ctx.ast);
            if <TypedSubRange<TsType> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsConditionalType(node) => 'traverse_children: {
            let child = node.check_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.extends_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.true_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.false_type(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsInferType(node) => 'traverse_children: {
            let child = node.type_parameter(ctx.ast);
            if <TsTypeParameter as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeOperator(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsParenthesizedType(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsFunctionType(node) => 'traverse_children: {
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <TsTypeAnnotation as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsConstructorType(node) => 'traverse_children: {
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <TsTypeAnnotation as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypePredicate(node) => 'traverse_children: {
            let child = node.parameter_name(ctx.ast);
            if <TsTypePredicateName as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeLiteral(node) => 'traverse_children: {
            let child = node.members(ctx.ast);
            if <TypedSubRange<TsTypeElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsMappedType(node) => 'traverse_children: {
            let child = node.key(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.constraint(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.name_type(ctx.ast);
            if <Option<TsType> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsType> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsPropertySignature(node) => 'traverse_children: {
            let child = node.key(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsMethodSignature(node) => 'traverse_children: {
            let child = node.key(ctx.ast);
            if <PropertyKey as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsCallSignatureDeclaration(node) => 'traverse_children: {
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsConstructSignatureDeclaration(node) => 'traverse_children: {
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.params(ctx.ast);
            if <FormalParameters as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.return_type(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsIndexSignature(node) => 'traverse_children: {
            let child = node.parameters(ctx.ast);
            if <TypedSubRange<BindingIdentifier> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <TsTypeAnnotation as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeAliasDeclaration(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsInterfaceDeclaration(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_parameters(ctx.ast);
            if <Option<TsTypeParameterDeclaration> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.extends(ctx.ast);
            if <TypedSubRange<TsInterfaceHeritage> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <TsInterfaceBody as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsInterfaceBody(node) => 'traverse_children: {
            let child = node.body(ctx.ast);
            if <TypedSubRange<TsTypeElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsInterfaceHeritage(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsClassImplements(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsEnumDeclaration(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <TsEnumBody as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsEnumBody(node) => 'traverse_children: {
            let child = node.members(ctx.ast);
            if <TypedSubRange<TsEnumMember> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsEnumMember(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <TsEnumMemberName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.initializer(ctx.ast);
            if <Option<Expr> as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsModuleDeclaration(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <TsModuleName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <Option<TsModuleBody> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsModuleBlock(node) => 'traverse_children: {
            let child = node.directives(ctx.ast);
            if <TypedSubRange<Directive> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <TypedSubRange<Stmt> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsGlobalDeclaration(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <IdentifierName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.body(ctx.ast);
            if <TsModuleBlock as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsParameterProperty(node) => 'traverse_children: {
            let child = node.decorators(ctx.ast);
            if <TypedSubRange<Decorator> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.parameter(ctx.ast);
            if <BindingPattern as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsThisParameter(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <Option<TsTypeAnnotation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsAsExpression(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsSatisfiesExpression(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsTypeAssertion(node) => 'traverse_children: {
            let child = node.type_annotation(ctx.ast);
            if <TsType as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsNonNullExpression(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsInstantiationExpression(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <TsTypeParameterInstantiation as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsExportAssignment(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsNamespaceExportDeclaration(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <IdentifierName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsImportEqualsDeclaration(node) => 'traverse_children: {
            let child = node.id(ctx.ast);
            if <BindingIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.module_reference(ctx.ast);
            if <TsModuleReference as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::TsExternalModuleReference(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <StringLiteral as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxElement(node) => 'traverse_children: {
            let child = node.opening_element(ctx.ast);
            if <JsxOpeningElement as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.children(ctx.ast);
            if <TypedSubRange<JsxChild> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.closing_element(ctx.ast);
            if <Option<JsxClosingElement> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxOpeningElement(node) => 'traverse_children: {
            let child = node.name(ctx.ast);
            if <JsxElementName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.attributes(ctx.ast);
            if <TypedSubRange<JsxAttributeItem> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.type_arguments(ctx.ast);
            if <Option<TsTypeParameterInstantiation> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxClosingElement(node) => 'traverse_children: {
            let child = node.name(ctx.ast);
            if <JsxElementName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxFragment(node) => 'traverse_children: {
            let child = node.opening_fragment(ctx.ast);
            if <JsxOpeningFragment as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.children(ctx.ast);
            if <TypedSubRange<JsxChild> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.closing_fragment(ctx.ast);
            if <JsxClosingFragment as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxOpeningFragment(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::JsxClosingFragment(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::JsxIdentifier(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::JsxNamespacedName(node) => 'traverse_children: {
            let child = node.namespace(ctx.ast);
            if <JsxIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.name(ctx.ast);
            if <JsxIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxMemberExpression(node) => 'traverse_children: {
            let child = node.object(ctx.ast);
            if <JsxMemberExpressionObject as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.property(ctx.ast);
            if <JsxIdentifier as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxAttribute(node) => 'traverse_children: {
            let child = node.name(ctx.ast);
            if <JsxAttributeName as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            let child = node.value(ctx.ast);
            if <Option<JsxAttributeValue> as TraverseWith<'a, State>>::traverse_with(
                child, traverser, ctx,
            ) == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxSpreadAttribute(node) => 'traverse_children: {
            let child = node.argument(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxExpressionContainer(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <JsxExpression as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
        NodeKindData::JsxEmptyExpression(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::JsxText(node) => 'traverse_children: {
            TraverseAction::Proceed
        }
        NodeKindData::JsxSpreadChild(node) => 'traverse_children: {
            let child = node.expression(ctx.ast);
            if <Expr as TraverseWith<'a, State>>::traverse_with(child, traverser, ctx)
                == TraverseAction::Stop
            {
                break 'traverse_children TraverseAction::Stop;
            }
            TraverseAction::Proceed
        }
    };
    if action == TraverseAction::Stop {
        ctx.pop_node(node_id);
        return TraverseAction::Stop;
    }
    match current_data {
        NodeKindData::Program(node) => traverser.exit_program(node, ctx),
        NodeKindData::Super(node) => traverser.exit_super(node, ctx),
        NodeKindData::ThisExpression(node) => traverser.exit_this_expression(node, ctx),
        NodeKindData::StringLiteral(node) => traverser.exit_string_literal(node, ctx),
        NodeKindData::NumericLiteral(node) => traverser.exit_numeric_literal(node, ctx),
        NodeKindData::BigIntLiteral(node) => traverser.exit_big_int_literal(node, ctx),
        NodeKindData::BooleanLiteral(node) => traverser.exit_boolean_literal(node, ctx),
        NodeKindData::NullLiteral(node) => traverser.exit_null_literal(node, ctx),
        NodeKindData::RegExpLiteral(node) => traverser.exit_reg_exp_literal(node, ctx),
        NodeKindData::TemplateLiteral(node) => traverser.exit_template_literal(node, ctx),
        NodeKindData::TemplateElement(node) => traverser.exit_template_element(node, ctx),
        NodeKindData::IdentifierReference(node) => traverser.exit_identifier_reference(node, ctx),
        NodeKindData::PrivateIdentifier(node) => traverser.exit_private_identifier(node, ctx),
        NodeKindData::BindingIdentifier(node) => traverser.exit_binding_identifier(node, ctx),
        NodeKindData::IdentifierName(node) => traverser.exit_identifier_name(node, ctx),
        NodeKindData::LabelIdentifier(node) => traverser.exit_label_identifier(node, ctx),
        NodeKindData::AssignmentPattern(node) => traverser.exit_assignment_pattern(node, ctx),
        NodeKindData::BindingRestElement(node) => traverser.exit_binding_rest_element(node, ctx),
        NodeKindData::ArrayPattern(node) => traverser.exit_array_pattern(node, ctx),
        NodeKindData::ObjectPattern(node) => traverser.exit_object_pattern(node, ctx),
        NodeKindData::BindingProperty(node) => traverser.exit_binding_property(node, ctx),
        NodeKindData::BinaryExpression(node) => traverser.exit_binary_expression(node, ctx),
        NodeKindData::LogicalExpression(node) => traverser.exit_logical_expression(node, ctx),
        NodeKindData::ConditionalExpression(node) => {
            traverser.exit_conditional_expression(node, ctx)
        }
        NodeKindData::UnaryExpression(node) => traverser.exit_unary_expression(node, ctx),
        NodeKindData::UpdateExpression(node) => traverser.exit_update_expression(node, ctx),
        NodeKindData::AssignmentExpression(node) => traverser.exit_assignment_expression(node, ctx),
        NodeKindData::VariableDeclaration(node) => traverser.exit_variable_declaration(node, ctx),
        NodeKindData::VariableDeclarator(node) => traverser.exit_variable_declarator(node, ctx),
        NodeKindData::IfStatement(node) => traverser.exit_if_statement(node, ctx),
        NodeKindData::SwitchStatement(node) => traverser.exit_switch_statement(node, ctx),
        NodeKindData::SwitchCase(node) => traverser.exit_switch_case(node, ctx),
        NodeKindData::ForStatement(node) => traverser.exit_for_statement(node, ctx),
        NodeKindData::ForInStatement(node) => traverser.exit_for_in_statement(node, ctx),
        NodeKindData::ForOfStatement(node) => traverser.exit_for_of_statement(node, ctx),
        NodeKindData::TryStatement(node) => traverser.exit_try_statement(node, ctx),
        NodeKindData::CatchClause(node) => traverser.exit_catch_clause(node, ctx),
        NodeKindData::BreakStatement(node) => traverser.exit_break_statement(node, ctx),
        NodeKindData::ContinueStatement(node) => traverser.exit_continue_statement(node, ctx),
        NodeKindData::LabeledStatement(node) => traverser.exit_labeled_statement(node, ctx),
        NodeKindData::ExpressionStatement(node) => traverser.exit_expression_statement(node, ctx),
        NodeKindData::ReturnStatement(node) => traverser.exit_return_statement(node, ctx),
        NodeKindData::ThrowStatement(node) => traverser.exit_throw_statement(node, ctx),
        NodeKindData::DebuggerStatement(node) => traverser.exit_debugger_statement(node, ctx),
        NodeKindData::EmptyStatement(node) => traverser.exit_empty_statement(node, ctx),
        NodeKindData::Directive(node) => traverser.exit_directive(node, ctx),
        NodeKindData::Decorator(node) => traverser.exit_decorator(node, ctx),
        NodeKindData::Class(node) => traverser.exit_class(node, ctx),
        NodeKindData::ClassBody(node) => traverser.exit_class_body(node, ctx),
        NodeKindData::MethodDefinition(node) => traverser.exit_method_definition(node, ctx),
        NodeKindData::PropertyDefinition(node) => traverser.exit_property_definition(node, ctx),
        NodeKindData::StaticBlock(node) => traverser.exit_static_block(node, ctx),
        NodeKindData::Function(node) => traverser.exit_function(node, ctx),
        NodeKindData::FunctionBody(node) => traverser.exit_function_body(node, ctx),
        NodeKindData::BlockStatement(node) => traverser.exit_block_statement(node, ctx),
        NodeKindData::WhileStatement(node) => traverser.exit_while_statement(node, ctx),
        NodeKindData::DoWhileStatement(node) => traverser.exit_do_while_statement(node, ctx),
        NodeKindData::WithStatement(node) => traverser.exit_with_statement(node, ctx),
        NodeKindData::FormalParameters(node) => traverser.exit_formal_parameters(node, ctx),
        NodeKindData::FormalParameter(node) => traverser.exit_formal_parameter(node, ctx),
        NodeKindData::ArrayExpression(node) => traverser.exit_array_expression(node, ctx),
        NodeKindData::ObjectExpression(node) => traverser.exit_object_expression(node, ctx),
        NodeKindData::SpreadElement(node) => traverser.exit_spread_element(node, ctx),
        NodeKindData::ObjectProperty(node) => traverser.exit_object_property(node, ctx),
        NodeKindData::ParenthesizedExpression(node) => {
            traverser.exit_parenthesized_expression(node, ctx)
        }
        NodeKindData::ArrowFunctionExpression(node) => {
            traverser.exit_arrow_function_expression(node, ctx)
        }
        NodeKindData::SequenceExpression(node) => traverser.exit_sequence_expression(node, ctx),
        NodeKindData::MemberExpression(node) => traverser.exit_member_expression(node, ctx),
        NodeKindData::CallExpression(node) => traverser.exit_call_expression(node, ctx),
        NodeKindData::ChainExpression(node) => traverser.exit_chain_expression(node, ctx),
        NodeKindData::TaggedTemplateExpression(node) => {
            traverser.exit_tagged_template_expression(node, ctx)
        }
        NodeKindData::NewExpression(node) => traverser.exit_new_expression(node, ctx),
        NodeKindData::AwaitExpression(node) => traverser.exit_await_expression(node, ctx),
        NodeKindData::YieldExpression(node) => traverser.exit_yield_expression(node, ctx),
        NodeKindData::MetaProperty(node) => traverser.exit_meta_property(node, ctx),
        NodeKindData::ImportExpression(node) => traverser.exit_import_expression(node, ctx),
        NodeKindData::ImportDeclaration(node) => traverser.exit_import_declaration(node, ctx),
        NodeKindData::ImportSpecifier(node) => traverser.exit_import_specifier(node, ctx),
        NodeKindData::ImportDefaultSpecifier(node) => {
            traverser.exit_import_default_specifier(node, ctx)
        }
        NodeKindData::ImportNamespaceSpecifier(node) => {
            traverser.exit_import_namespace_specifier(node, ctx)
        }
        NodeKindData::ImportAttribute(node) => traverser.exit_import_attribute(node, ctx),
        NodeKindData::ExportNamedDeclaration(node) => {
            traverser.exit_export_named_declaration(node, ctx)
        }
        NodeKindData::ExportDefaultDeclaration(node) => {
            traverser.exit_export_default_declaration(node, ctx)
        }
        NodeKindData::ExportAllDeclaration(node) => {
            traverser.exit_export_all_declaration(node, ctx)
        }
        NodeKindData::ExportSpecifier(node) => traverser.exit_export_specifier(node, ctx),
        NodeKindData::Hashbang(node) => traverser.exit_hashbang(node, ctx),
        NodeKindData::TsFunction(node) => traverser.exit_ts_function(node, ctx),
        NodeKindData::TsMethodDefinition(node) => traverser.exit_ts_method_definition(node, ctx),
        NodeKindData::TsTypeAnnotation(node) => traverser.exit_ts_type_annotation(node, ctx),
        NodeKindData::TsAnyKeyword(node) => traverser.exit_ts_any_keyword(node, ctx),
        NodeKindData::TsUnknownKeyword(node) => traverser.exit_ts_unknown_keyword(node, ctx),
        NodeKindData::TsNeverKeyword(node) => traverser.exit_ts_never_keyword(node, ctx),
        NodeKindData::TsVoidKeyword(node) => traverser.exit_ts_void_keyword(node, ctx),
        NodeKindData::TsNullKeyword(node) => traverser.exit_ts_null_keyword(node, ctx),
        NodeKindData::TsUndefinedKeyword(node) => traverser.exit_ts_undefined_keyword(node, ctx),
        NodeKindData::TsStringKeyword(node) => traverser.exit_ts_string_keyword(node, ctx),
        NodeKindData::TsNumberKeyword(node) => traverser.exit_ts_number_keyword(node, ctx),
        NodeKindData::TsBigIntKeyword(node) => traverser.exit_ts_big_int_keyword(node, ctx),
        NodeKindData::TsBooleanKeyword(node) => traverser.exit_ts_boolean_keyword(node, ctx),
        NodeKindData::TsSymbolKeyword(node) => traverser.exit_ts_symbol_keyword(node, ctx),
        NodeKindData::TsObjectKeyword(node) => traverser.exit_ts_object_keyword(node, ctx),
        NodeKindData::TsIntrinsicKeyword(node) => traverser.exit_ts_intrinsic_keyword(node, ctx),
        NodeKindData::TsThisType(node) => traverser.exit_ts_this_type(node, ctx),
        NodeKindData::TsTypeReference(node) => traverser.exit_ts_type_reference(node, ctx),
        NodeKindData::TsQualifiedName(node) => traverser.exit_ts_qualified_name(node, ctx),
        NodeKindData::TsTypeQuery(node) => traverser.exit_ts_type_query(node, ctx),
        NodeKindData::TsImportType(node) => traverser.exit_ts_import_type(node, ctx),
        NodeKindData::TsTypeParameter(node) => traverser.exit_ts_type_parameter(node, ctx),
        NodeKindData::TsTypeParameterDeclaration(node) => {
            traverser.exit_ts_type_parameter_declaration(node, ctx)
        }
        NodeKindData::TsTypeParameterInstantiation(node) => {
            traverser.exit_ts_type_parameter_instantiation(node, ctx)
        }
        NodeKindData::TsLiteralType(node) => traverser.exit_ts_literal_type(node, ctx),
        NodeKindData::TsTemplateLiteralType(node) => {
            traverser.exit_ts_template_literal_type(node, ctx)
        }
        NodeKindData::TsArrayType(node) => traverser.exit_ts_array_type(node, ctx),
        NodeKindData::TsIndexedAccessType(node) => traverser.exit_ts_indexed_access_type(node, ctx),
        NodeKindData::TsTupleType(node) => traverser.exit_ts_tuple_type(node, ctx),
        NodeKindData::TsNamedTupleMember(node) => traverser.exit_ts_named_tuple_member(node, ctx),
        NodeKindData::TsOptionalType(node) => traverser.exit_ts_optional_type(node, ctx),
        NodeKindData::TsRestType(node) => traverser.exit_ts_rest_type(node, ctx),
        NodeKindData::TsJsDocNullableType(node) => {
            traverser.exit_ts_js_doc_nullable_type(node, ctx)
        }
        NodeKindData::TsJsDocNonNullableType(node) => {
            traverser.exit_ts_js_doc_non_nullable_type(node, ctx)
        }
        NodeKindData::TsJsDocUnknownType(node) => traverser.exit_ts_js_doc_unknown_type(node, ctx),
        NodeKindData::TsUnionType(node) => traverser.exit_ts_union_type(node, ctx),
        NodeKindData::TsIntersectionType(node) => traverser.exit_ts_intersection_type(node, ctx),
        NodeKindData::TsConditionalType(node) => traverser.exit_ts_conditional_type(node, ctx),
        NodeKindData::TsInferType(node) => traverser.exit_ts_infer_type(node, ctx),
        NodeKindData::TsTypeOperator(node) => traverser.exit_ts_type_operator(node, ctx),
        NodeKindData::TsParenthesizedType(node) => traverser.exit_ts_parenthesized_type(node, ctx),
        NodeKindData::TsFunctionType(node) => traverser.exit_ts_function_type(node, ctx),
        NodeKindData::TsConstructorType(node) => traverser.exit_ts_constructor_type(node, ctx),
        NodeKindData::TsTypePredicate(node) => traverser.exit_ts_type_predicate(node, ctx),
        NodeKindData::TsTypeLiteral(node) => traverser.exit_ts_type_literal(node, ctx),
        NodeKindData::TsMappedType(node) => traverser.exit_ts_mapped_type(node, ctx),
        NodeKindData::TsPropertySignature(node) => traverser.exit_ts_property_signature(node, ctx),
        NodeKindData::TsMethodSignature(node) => traverser.exit_ts_method_signature(node, ctx),
        NodeKindData::TsCallSignatureDeclaration(node) => {
            traverser.exit_ts_call_signature_declaration(node, ctx)
        }
        NodeKindData::TsConstructSignatureDeclaration(node) => {
            traverser.exit_ts_construct_signature_declaration(node, ctx)
        }
        NodeKindData::TsIndexSignature(node) => traverser.exit_ts_index_signature(node, ctx),
        NodeKindData::TsTypeAliasDeclaration(node) => {
            traverser.exit_ts_type_alias_declaration(node, ctx)
        }
        NodeKindData::TsInterfaceDeclaration(node) => {
            traverser.exit_ts_interface_declaration(node, ctx)
        }
        NodeKindData::TsInterfaceBody(node) => traverser.exit_ts_interface_body(node, ctx),
        NodeKindData::TsInterfaceHeritage(node) => traverser.exit_ts_interface_heritage(node, ctx),
        NodeKindData::TsClassImplements(node) => traverser.exit_ts_class_implements(node, ctx),
        NodeKindData::TsEnumDeclaration(node) => traverser.exit_ts_enum_declaration(node, ctx),
        NodeKindData::TsEnumBody(node) => traverser.exit_ts_enum_body(node, ctx),
        NodeKindData::TsEnumMember(node) => traverser.exit_ts_enum_member(node, ctx),
        NodeKindData::TsModuleDeclaration(node) => traverser.exit_ts_module_declaration(node, ctx),
        NodeKindData::TsModuleBlock(node) => traverser.exit_ts_module_block(node, ctx),
        NodeKindData::TsGlobalDeclaration(node) => traverser.exit_ts_global_declaration(node, ctx),
        NodeKindData::TsParameterProperty(node) => traverser.exit_ts_parameter_property(node, ctx),
        NodeKindData::TsThisParameter(node) => traverser.exit_ts_this_parameter(node, ctx),
        NodeKindData::TsAsExpression(node) => traverser.exit_ts_as_expression(node, ctx),
        NodeKindData::TsSatisfiesExpression(node) => {
            traverser.exit_ts_satisfies_expression(node, ctx)
        }
        NodeKindData::TsTypeAssertion(node) => traverser.exit_ts_type_assertion(node, ctx),
        NodeKindData::TsNonNullExpression(node) => traverser.exit_ts_non_null_expression(node, ctx),
        NodeKindData::TsInstantiationExpression(node) => {
            traverser.exit_ts_instantiation_expression(node, ctx)
        }
        NodeKindData::TsExportAssignment(node) => traverser.exit_ts_export_assignment(node, ctx),
        NodeKindData::TsNamespaceExportDeclaration(node) => {
            traverser.exit_ts_namespace_export_declaration(node, ctx)
        }
        NodeKindData::TsImportEqualsDeclaration(node) => {
            traverser.exit_ts_import_equals_declaration(node, ctx)
        }
        NodeKindData::TsExternalModuleReference(node) => {
            traverser.exit_ts_external_module_reference(node, ctx)
        }
        NodeKindData::JsxElement(node) => traverser.exit_jsx_element(node, ctx),
        NodeKindData::JsxOpeningElement(node) => traverser.exit_jsx_opening_element(node, ctx),
        NodeKindData::JsxClosingElement(node) => traverser.exit_jsx_closing_element(node, ctx),
        NodeKindData::JsxFragment(node) => traverser.exit_jsx_fragment(node, ctx),
        NodeKindData::JsxOpeningFragment(node) => traverser.exit_jsx_opening_fragment(node, ctx),
        NodeKindData::JsxClosingFragment(node) => traverser.exit_jsx_closing_fragment(node, ctx),
        NodeKindData::JsxIdentifier(node) => traverser.exit_jsx_identifier(node, ctx),
        NodeKindData::JsxNamespacedName(node) => traverser.exit_jsx_namespaced_name(node, ctx),
        NodeKindData::JsxMemberExpression(node) => traverser.exit_jsx_member_expression(node, ctx),
        NodeKindData::JsxAttribute(node) => traverser.exit_jsx_attribute(node, ctx),
        NodeKindData::JsxSpreadAttribute(node) => traverser.exit_jsx_spread_attribute(node, ctx),
        NodeKindData::JsxExpressionContainer(node) => {
            traverser.exit_jsx_expression_container(node, ctx)
        }
        NodeKindData::JsxEmptyExpression(node) => traverser.exit_jsx_empty_expression(node, ctx),
        NodeKindData::JsxText(node) => traverser.exit_jsx_text(node, ctx),
        NodeKindData::JsxSpreadChild(node) => traverser.exit_jsx_spread_child(node, ctx),
    }
    traverser.exit_node(node_id, current_kind, ctx);
    ctx.pop_node(node_id);
    TraverseAction::Proceed
}
