#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::too_many_arguments
)]
use crate::*;
use oxc_index::Idx;
impl<'a> Ast<'a> {
    #[inline]
    pub fn program(
        &mut self,
        span: Span,
        source_type: SourceType,
        directives: TypedSubRange<Directive>,
        body: TypedSubRange<Stmt>,
        hashbang: Option<Hashbang>,
    ) -> Program {
        let _extra_data = [source_type.to_extra_data(self), body.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        Program(self.add_node(
            span,
            NodeKind::Program,
            NodePayload(
                0u128
                    | ((unsafe { directives.to_extra_data(self).other } as u128) << 32usize)
                    | ((hashbang.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn stmt_block_statement(&mut self, span: Span, body: TypedSubRange<Stmt>) -> Stmt {
        Stmt::BlockStatement(self.block_statement(span, body).into())
    }
    #[inline]
    pub fn stmt_break_statement(&mut self, span: Span, label: Option<LabelIdentifier>) -> Stmt {
        Stmt::BreakStatement(self.break_statement(span, label).into())
    }
    #[inline]
    pub fn stmt_continue_statement(&mut self, span: Span, label: Option<LabelIdentifier>) -> Stmt {
        Stmt::ContinueStatement(self.continue_statement(span, label).into())
    }
    #[inline]
    pub fn stmt_debugger_statement(&mut self, span: Span) -> Stmt {
        Stmt::DebuggerStatement(self.debugger_statement(span).into())
    }
    #[inline]
    pub fn stmt_do_while_statement(&mut self, span: Span, body: Stmt, test: Expr) -> Stmt {
        Stmt::DoWhileStatement(self.do_while_statement(span, body, test).into())
    }
    #[inline]
    pub fn stmt_empty_statement(&mut self, span: Span) -> Stmt {
        Stmt::EmptyStatement(self.empty_statement(span).into())
    }
    #[inline]
    pub fn stmt_expression_statement(&mut self, span: Span, expression: Expr) -> Stmt {
        Stmt::ExpressionStatement(self.expression_statement(span, expression).into())
    }
    #[inline]
    pub fn stmt_for_in_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft,
        right: Expr,
        body: Stmt,
    ) -> Stmt {
        Stmt::ForInStatement(self.for_in_statement(span, left, right, body).into())
    }
    #[inline]
    pub fn stmt_for_of_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft,
        right: Expr,
        body: Stmt,
        r#await: bool,
    ) -> Stmt {
        Stmt::ForOfStatement(
            self.for_of_statement(span, left, right, body, r#await)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_for_statement(
        &mut self,
        span: Span,
        init: Option<ForStatementInit>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Stmt,
    ) -> Stmt {
        Stmt::ForStatement(self.for_statement(span, init, test, update, body).into())
    }
    #[inline]
    pub fn stmt_if_statement(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Stmt,
        alternate: Option<Stmt>,
    ) -> Stmt {
        Stmt::IfStatement(self.if_statement(span, test, consequent, alternate).into())
    }
    #[inline]
    pub fn stmt_labeled_statement(
        &mut self,
        span: Span,
        label: LabelIdentifier,
        body: Stmt,
    ) -> Stmt {
        Stmt::LabeledStatement(self.labeled_statement(span, label, body).into())
    }
    #[inline]
    pub fn stmt_return_statement(&mut self, span: Span, argument: Option<Expr>) -> Stmt {
        Stmt::ReturnStatement(self.return_statement(span, argument).into())
    }
    #[inline]
    pub fn stmt_switch_statement(
        &mut self,
        span: Span,
        discriminant: Expr,
        cases: TypedSubRange<SwitchCase>,
    ) -> Stmt {
        Stmt::SwitchStatement(self.switch_statement(span, discriminant, cases).into())
    }
    #[inline]
    pub fn stmt_throw_statement(&mut self, span: Span, argument: Expr) -> Stmt {
        Stmt::ThrowStatement(self.throw_statement(span, argument).into())
    }
    #[inline]
    pub fn stmt_try_statement(
        &mut self,
        span: Span,
        block: BlockStatement,
        handler: Option<CatchClause>,
        finalizer: Option<BlockStatement>,
    ) -> Stmt {
        Stmt::TryStatement(self.try_statement(span, block, handler, finalizer).into())
    }
    #[inline]
    pub fn stmt_while_statement(&mut self, span: Span, test: Expr, body: Stmt) -> Stmt {
        Stmt::WhileStatement(self.while_statement(span, test, body).into())
    }
    #[inline]
    pub fn stmt_with_statement(&mut self, span: Span, object: Expr, body: Stmt) -> Stmt {
        Stmt::WithStatement(self.with_statement(span, object, body).into())
    }
    #[inline]
    pub fn stmt_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: TypedSubRange<VariableDeclarator>,
        declare: bool,
    ) -> Stmt {
        Stmt::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Stmt {
        Stmt::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn stmt_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> Stmt {
        Stmt::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        declare: bool,
        params: FormalParameters,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Stmt {
        Stmt::TsFunction(
            self.ts_function(
                span,
                id,
                generator,
                r#async,
                declare,
                params,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_type_alias_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        type_annotation: TsType,
        declare: bool,
    ) -> Stmt {
        Stmt::TsTypeAliasDeclaration(
            self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        extends: TypedSubRange<TsInterfaceHeritage>,
        body: TsInterfaceBody,
        declare: bool,
    ) -> Stmt {
        Stmt::TsInterfaceDeclaration(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        body: TsEnumBody,
        is_const: bool,
        declare: bool,
    ) -> Stmt {
        Stmt::TsEnumDeclaration(
            self.ts_enum_declaration(span, id, body, is_const, declare)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName,
        body: Option<TsModuleBody>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> Stmt {
        Stmt::TsModuleDeclaration(
            self.ts_module_declaration(span, id, body, kind, declare)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_global_declaration(
        &mut self,
        span: Span,
        id: IdentifierName,
        body: TsModuleBlock,
        declare: bool,
    ) -> Stmt {
        Stmt::TsGlobalDeclaration(self.ts_global_declaration(span, id, body, declare).into())
    }
    #[inline]
    pub fn stmt_ts_import_equals_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        module_reference: TsModuleReference,
        import_kind: ImportOrExportKind,
    ) -> Stmt {
        Stmt::TsImportEqualsDeclaration(
            self.ts_import_equals_declaration(span, id, module_reference, import_kind)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_import_declaration(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ImportDeclarationSpecifier>,
        source: StringLiteral,
        attributes: TypedSubRange<ImportAttribute>,
        phase: Option<ImportPhase>,
        import_kind: ImportOrExportKind,
    ) -> Stmt {
        Stmt::ImportDeclaration(
            self.import_declaration(span, specifiers, source, attributes, phase, import_kind)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_export_named_declaration(
        &mut self,
        span: Span,
        declaration: Option<Decl>,
        specifiers: TypedSubRange<ExportSpecifier>,
        source: Option<StringLiteral>,
        attributes: TypedSubRange<ImportAttribute>,
        export_kind: ImportOrExportKind,
    ) -> Stmt {
        Stmt::ExportNamedDeclaration(
            self.export_named_declaration(
                span,
                declaration,
                specifiers,
                source,
                attributes,
                export_kind,
            )
            .into(),
        )
    }
    #[inline]
    pub fn stmt_export_default_declaration(
        &mut self,
        span: Span,
        declaration: ExportDefaultDeclarationKind,
    ) -> Stmt {
        Stmt::ExportDefaultDeclaration(self.export_default_declaration(span, declaration).into())
    }
    #[inline]
    pub fn stmt_export_all_declaration(
        &mut self,
        span: Span,
        exported: Option<ModuleExportName>,
        source: StringLiteral,
        attributes: TypedSubRange<ImportAttribute>,
        export_kind: ImportOrExportKind,
    ) -> Stmt {
        Stmt::ExportAllDeclaration(
            self.export_all_declaration(span, exported, source, attributes, export_kind)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_export_assignment(&mut self, span: Span, expression: Expr) -> Stmt {
        Stmt::TsExportAssignment(self.ts_export_assignment(span, expression).into())
    }
    #[inline]
    pub fn stmt_ts_namespace_export_declaration(&mut self, span: Span, id: IdentifierName) -> Stmt {
        Stmt::TsNamespaceExportDeclaration(self.ts_namespace_export_declaration(span, id).into())
    }
    #[inline]
    pub fn decl_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: TypedSubRange<VariableDeclarator>,
        declare: bool,
    ) -> Decl {
        Decl::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Decl {
        Decl::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn decl_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> Decl {
        Decl::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn decl_ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        declare: bool,
        params: FormalParameters,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Decl {
        Decl::TsFunction(
            self.ts_function(
                span,
                id,
                generator,
                r#async,
                declare,
                params,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn decl_ts_type_alias_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        type_annotation: TsType,
        declare: bool,
    ) -> Decl {
        Decl::TsTypeAliasDeclaration(
            self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        extends: TypedSubRange<TsInterfaceHeritage>,
        body: TsInterfaceBody,
        declare: bool,
    ) -> Decl {
        Decl::TsInterfaceDeclaration(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        body: TsEnumBody,
        is_const: bool,
        declare: bool,
    ) -> Decl {
        Decl::TsEnumDeclaration(
            self.ts_enum_declaration(span, id, body, is_const, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName,
        body: Option<TsModuleBody>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> Decl {
        Decl::TsModuleDeclaration(
            self.ts_module_declaration(span, id, body, kind, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_global_declaration(
        &mut self,
        span: Span,
        id: IdentifierName,
        body: TsModuleBlock,
        declare: bool,
    ) -> Decl {
        Decl::TsGlobalDeclaration(self.ts_global_declaration(span, id, body, declare).into())
    }
    #[inline]
    pub fn decl_ts_import_equals_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        module_reference: TsModuleReference,
        import_kind: ImportOrExportKind,
    ) -> Decl {
        Decl::TsImportEqualsDeclaration(
            self.ts_import_equals_declaration(span, id, module_reference, import_kind)
                .into(),
        )
    }
    #[inline]
    pub fn expr_identifier_reference(&mut self, span: Span, name: AstUtf8) -> Expr {
        Expr::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn expr_this_expression(&mut self, span: Span) -> Expr {
        Expr::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn expr_super(&mut self, span: Span) -> Expr {
        Expr::Super(self.super_(span).into())
    }
    #[inline]
    pub fn expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> Expr {
        Expr::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn expr_string_literal(&mut self, span: Span, value: AstWtf8, raw: AstUtf8) -> Expr {
        Expr::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn expr_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> Expr {
        Expr::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn expr_big_int_literal(&mut self, span: Span, raw: AstUtf8) -> Expr {
        Expr::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn expr_boolean_literal(&mut self, span: Span, value: bool) -> Expr {
        Expr::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn expr_null_literal(&mut self, span: Span) -> Expr {
        Expr::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn expr_reg_exp_literal(&mut self, span: Span, pattern: AstUtf8, flags: AstUtf8) -> Expr {
        Expr::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn expr_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> Expr {
        Expr::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> Expr {
        Expr::BinaryExpression(self.binary_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> Expr {
        Expr::LogicalExpression(self.logical_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> Expr {
        Expr::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> Expr {
        Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> Expr {
        Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> Expr {
        Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> Expr {
        Expr::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn expr_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> Expr {
        Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Expr {
        Expr::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn expr_chain_expression(&mut self, span: Span, expression: Expr) -> Expr {
        Expr::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Expr {
        Expr::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Expr {
        Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Expr {
        Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn expr_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> Expr {
        Expr::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn expr_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> Expr {
        Expr::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn expr_parenthesized_expression(&mut self, span: Span, expression: Expr) -> Expr {
        Expr::ParenthesizedExpression(self.parenthesized_expression(span, expression).into())
    }
    #[inline]
    pub fn expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Expr {
        Expr::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> Expr {
        Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn expr_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> Expr {
        Expr::ImportExpression(self.import_expression(span, source, options, phase).into())
    }
    #[inline]
    pub fn expr_await_expression(&mut self, span: Span, argument: Expr) -> Expr {
        Expr::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> Expr {
        Expr::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn expr_private_identifier(&mut self, span: Span, name: AstUtf8) -> Expr {
        Expr::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> Expr {
        Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> Expr {
        Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> Expr {
        Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn expr_ts_non_null_expression(&mut self, span: Span, expression: Expr) -> Expr {
        Expr::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> Expr {
        Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> Expr {
        Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> Expr {
        Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> BindingPattern {
        BindingPattern::BindingIdentifier(
            self.binding_identifier(span, name, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> BindingPattern {
        BindingPattern::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn binding_pattern_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> BindingPattern {
        BindingPattern::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> BindingPattern {
        BindingPattern::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> BindingPattern {
        BindingPattern::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> BindingPattern {
        BindingPattern::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> BindingPattern {
        BindingPattern::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn binding_pattern_assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern,
        right: Expr,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> BindingPattern {
        BindingPattern::AssignmentPattern(
            self.assignment_pattern(span, left, right, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_binding_rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> BindingPattern {
        BindingPattern::BindingRestElement(
            self.binding_rest_element(span, argument, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_array_pattern(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> BindingPattern {
        BindingPattern::ArrayPattern(
            self.array_pattern(span, elements, rest, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_object_pattern(
        &mut self,
        span: Span,
        properties: TypedSubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> BindingPattern {
        BindingPattern::ObjectPattern(
            self.object_pattern(
                span,
                properties,
                rest,
                decorators,
                type_annotation,
                optional,
            )
            .into(),
        )
    }
    #[inline]
    pub fn assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> AssignmentTarget {
        AssignmentTarget::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> AssignmentTarget {
        AssignmentTarget::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> AssignmentTarget {
        AssignmentTarget::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> AssignmentTarget {
        AssignmentTarget::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> AssignmentTarget {
        AssignmentTarget::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> AssignmentTarget {
        AssignmentTarget::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn assignment_target_array_pattern(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> AssignmentTarget {
        AssignmentTarget::ArrayAssignmentTarget(
            self.array_pattern(span, elements, rest, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn assignment_target_object_pattern(
        &mut self,
        span: Span,
        properties: TypedSubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> AssignmentTarget {
        AssignmentTarget::ObjectAssignmentTarget(
            self.object_pattern(
                span,
                properties,
                rest,
                decorators,
                type_annotation,
                optional,
            )
            .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn property_key_identifier_name(&mut self, span: Span, name: AstUtf8) -> PropertyKey {
        PropertyKey::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn property_key_private_identifier(&mut self, span: Span, name: AstUtf8) -> PropertyKey {
        PropertyKey::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn property_key_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> PropertyKey {
        PropertyKey::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn property_key_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> PropertyKey {
        PropertyKey::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn property_key_big_int_literal(&mut self, span: Span, raw: AstUtf8) -> PropertyKey {
        PropertyKey::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn property_key_identifier_reference(&mut self, span: Span, name: AstUtf8) -> PropertyKey {
        PropertyKey::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn property_key_this_expression(&mut self, span: Span) -> PropertyKey {
        PropertyKey::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn property_key_super(&mut self, span: Span) -> PropertyKey {
        PropertyKey::Super(self.super_(span).into())
    }
    #[inline]
    pub fn property_key_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> PropertyKey {
        PropertyKey::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn property_key_boolean_literal(&mut self, span: Span, value: bool) -> PropertyKey {
        PropertyKey::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn property_key_null_literal(&mut self, span: Span) -> PropertyKey {
        PropertyKey::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn property_key_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> PropertyKey {
        PropertyKey::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn property_key_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> PropertyKey {
        PropertyKey::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn property_key_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> PropertyKey {
        PropertyKey::BinaryExpression(self.binary_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn property_key_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> PropertyKey {
        PropertyKey::LogicalExpression(self.logical_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn property_key_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> PropertyKey {
        PropertyKey::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn property_key_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> PropertyKey {
        PropertyKey::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> PropertyKey {
        PropertyKey::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> PropertyKey {
        PropertyKey::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> PropertyKey {
        PropertyKey::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn property_key_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> PropertyKey {
        PropertyKey::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> PropertyKey {
        PropertyKey::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_chain_expression(&mut self, span: Span, expression: Expr) -> PropertyKey {
        PropertyKey::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn property_key_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> PropertyKey {
        PropertyKey::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> PropertyKey {
        PropertyKey::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> PropertyKey {
        PropertyKey::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn property_key_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> PropertyKey {
        PropertyKey::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn property_key_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> PropertyKey {
        PropertyKey::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn property_key_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> PropertyKey {
        PropertyKey::ParenthesizedExpression(self.parenthesized_expression(span, expression).into())
    }
    #[inline]
    pub fn property_key_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> PropertyKey {
        PropertyKey::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn property_key_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> PropertyKey {
        PropertyKey::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn property_key_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> PropertyKey {
        PropertyKey::ImportExpression(self.import_expression(span, source, options, phase).into())
    }
    #[inline]
    pub fn property_key_await_expression(&mut self, span: Span, argument: Expr) -> PropertyKey {
        PropertyKey::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn property_key_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> PropertyKey {
        PropertyKey::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn property_key_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> PropertyKey {
        PropertyKey::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> PropertyKey {
        PropertyKey::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> PropertyKey {
        PropertyKey::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> PropertyKey {
        PropertyKey::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn property_key_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> PropertyKey {
        PropertyKey::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> PropertyKey {
        PropertyKey::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn property_key_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> PropertyKey {
        PropertyKey::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn module_export_name_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ModuleExportName {
        ModuleExportName::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn module_export_name_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> ModuleExportName {
        ModuleExportName::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn argument_identifier_reference(&mut self, span: Span, name: AstUtf8) -> Argument {
        Argument::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn argument_this_expression(&mut self, span: Span) -> Argument {
        Argument::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn argument_super(&mut self, span: Span) -> Argument {
        Argument::Super(self.super_(span).into())
    }
    #[inline]
    pub fn argument_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> Argument {
        Argument::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn argument_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> Argument {
        Argument::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn argument_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> Argument {
        Argument::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn argument_big_int_literal(&mut self, span: Span, raw: AstUtf8) -> Argument {
        Argument::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn argument_boolean_literal(&mut self, span: Span, value: bool) -> Argument {
        Argument::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn argument_null_literal(&mut self, span: Span) -> Argument {
        Argument::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn argument_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> Argument {
        Argument::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn argument_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> Argument {
        Argument::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn argument_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> Argument {
        Argument::BinaryExpression(self.binary_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn argument_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> Argument {
        Argument::LogicalExpression(self.logical_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn argument_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> Argument {
        Argument::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn argument_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> Argument {
        Argument::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn argument_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> Argument {
        Argument::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn argument_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> Argument {
        Argument::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn argument_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> Argument {
        Argument::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn argument_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> Argument {
        Argument::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn argument_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Argument {
        Argument::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn argument_chain_expression(&mut self, span: Span, expression: Expr) -> Argument {
        Argument::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn argument_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Argument {
        Argument::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn argument_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> Argument {
        Argument::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn argument_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Argument {
        Argument::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn argument_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> Argument {
        Argument::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn argument_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> Argument {
        Argument::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn argument_parenthesized_expression(&mut self, span: Span, expression: Expr) -> Argument {
        Argument::ParenthesizedExpression(self.parenthesized_expression(span, expression).into())
    }
    #[inline]
    pub fn argument_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Argument {
        Argument::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn argument_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> Argument {
        Argument::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn argument_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> Argument {
        Argument::ImportExpression(self.import_expression(span, source, options, phase).into())
    }
    #[inline]
    pub fn argument_await_expression(&mut self, span: Span, argument: Expr) -> Argument {
        Argument::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn argument_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> Argument {
        Argument::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn argument_private_identifier(&mut self, span: Span, name: AstUtf8) -> Argument {
        Argument::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn argument_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> Argument {
        Argument::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn argument_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> Argument {
        Argument::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn argument_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> Argument {
        Argument::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn argument_ts_non_null_expression(&mut self, span: Span, expression: Expr) -> Argument {
        Argument::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn argument_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> Argument {
        Argument::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn argument_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> Argument {
        Argument::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn argument_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> Argument {
        Argument::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn argument_spread_element(&mut self, span: Span, argument: Expr) -> Argument {
        Argument::SpreadElement(self.spread_element(span, argument).into())
    }
    #[inline]
    pub fn object_property_kind_object_property(
        &mut self,
        span: Span,
        key: PropertyKey,
        value: Expr,
        kind: PropertyKind,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectPropertyKind {
        ObjectPropertyKind::ObjectProperty(
            self.object_property(span, key, value, kind, method, shorthand, computed)
                .into(),
        )
    }
    #[inline]
    pub fn object_property_kind_spread_element(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> ObjectPropertyKind {
        ObjectPropertyKind::SpreadElement(self.spread_element(span, argument).into())
    }
    #[inline]
    pub fn for_statement_init_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: TypedSubRange<VariableDeclarator>,
        declare: bool,
    ) -> ForStatementInit {
        ForStatementInit::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn for_statement_init_this_expression(&mut self, span: Span) -> ForStatementInit {
        ForStatementInit::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn for_statement_init_super(&mut self, span: Span) -> ForStatementInit {
        ForStatementInit::Super(self.super_(span).into())
    }
    #[inline]
    pub fn for_statement_init_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> ForStatementInit {
        ForStatementInit::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn for_statement_init_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn for_statement_init_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn for_statement_init_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn for_statement_init_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ForStatementInit {
        ForStatementInit::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn for_statement_init_null_literal(&mut self, span: Span) -> ForStatementInit {
        ForStatementInit::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn for_statement_init_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn for_statement_init_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> ForStatementInit {
        ForStatementInit::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn for_statement_init_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> ForStatementInit {
        ForStatementInit::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn for_statement_init_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> ForStatementInit {
        ForStatementInit::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn for_statement_init_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> ForStatementInit {
        ForStatementInit::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn for_statement_init_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ForStatementInit {
        ForStatementInit::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> ForStatementInit {
        ForStatementInit::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> ForStatementInit {
        ForStatementInit::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> ForStatementInit {
        ForStatementInit::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn for_statement_init_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ForStatementInit {
        ForStatementInit::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ForStatementInit {
        ForStatementInit::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn for_statement_init_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ForStatementInit {
        ForStatementInit::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ForStatementInit {
        ForStatementInit::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ForStatementInit {
        ForStatementInit::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> ForStatementInit {
        ForStatementInit::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn for_statement_init_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> ForStatementInit {
        ForStatementInit::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn for_statement_init_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn for_statement_init_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ForStatementInit {
        ForStatementInit::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> ForStatementInit {
        ForStatementInit::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> ForStatementInit {
        ForStatementInit::ImportExpression(
            self.import_expression(span, source, options, phase).into(),
        )
    }
    #[inline]
    pub fn for_statement_init_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> ForStatementInit {
        ForStatementInit::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn for_statement_init_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> ForStatementInit {
        ForStatementInit::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn for_statement_init_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn for_statement_init_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementInit {
        ForStatementInit::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementInit {
        ForStatementInit::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn for_statement_init_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> ForStatementInit {
        ForStatementInit::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> ForStatementInit {
        ForStatementInit::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> ForStatementInit {
        ForStatementInit::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: TypedSubRange<VariableDeclarator>,
        declare: bool,
    ) -> ForStatementLeft {
        ForStatementLeft::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ForStatementLeft {
        ForStatementLeft::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn for_statement_left_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ForStatementLeft {
        ForStatementLeft::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementLeft {
        ForStatementLeft::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementLeft {
        ForStatementLeft::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ForStatementLeft {
        ForStatementLeft::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementLeft {
        ForStatementLeft::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn for_statement_left_array_pattern(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> ForStatementLeft {
        ForStatementLeft::ArrayAssignmentTarget(
            self.array_pattern(span, elements, rest, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_object_pattern(
        &mut self,
        span: Span,
        properties: TypedSubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> ForStatementLeft {
        ForStatementLeft::ObjectAssignmentTarget(
            self.object_pattern(
                span,
                properties,
                rest,
                decorators,
                type_annotation,
                optional,
            )
            .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_function_body(
        &mut self,
        span: Span,
        directives: TypedSubRange<Directive>,
        body: TypedSubRange<Stmt>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::FunctionBody(self.function_body(span, directives, body).into())
    }
    #[inline]
    pub fn arrow_function_body_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn arrow_function_body_this_expression(&mut self, span: Span) -> ArrowFunctionBody {
        ArrowFunctionBody::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn arrow_function_body_super(&mut self, span: Span) -> ArrowFunctionBody {
        ArrowFunctionBody::Super(self.super_(span).into())
    }
    #[inline]
    pub fn arrow_function_body_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn arrow_function_body_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn arrow_function_body_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn arrow_function_body_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn arrow_function_body_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn arrow_function_body_null_literal(&mut self, span: Span) -> ArrowFunctionBody {
        ArrowFunctionBody::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn arrow_function_body_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn arrow_function_body_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn arrow_function_body_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn arrow_function_body_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn arrow_function_body_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn arrow_function_body_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn arrow_function_body_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn arrow_function_body_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::ImportExpression(
            self.import_expression(span, source, options, phase).into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn arrow_function_body_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn arrow_function_body_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn arrow_function_body_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn arrow_function_body_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn arrow_function_body_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_item_formal_parameter(
        &mut self,
        span: Span,
        pattern: FormalParameterPattern,
    ) -> FormalParameterItem {
        FormalParameterItem::FormalParameter(self.formal_parameter(span, pattern).into())
    }
    #[inline]
    pub fn formal_parameter_item_ts_parameter_property(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        parameter: BindingPattern,
        r#override: bool,
        readonly: bool,
        accessibility: Accessibility,
    ) -> FormalParameterItem {
        FormalParameterItem::TsParameterProperty(
            self.ts_parameter_property(
                span,
                decorators,
                parameter,
                r#override,
                readonly,
                accessibility,
            )
            .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> FormalParameterPattern {
        FormalParameterPattern::BindingIdentifier(
            self.binding_identifier(span, name, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> FormalParameterPattern {
        FormalParameterPattern::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn formal_parameter_pattern_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> FormalParameterPattern {
        FormalParameterPattern::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> FormalParameterPattern {
        FormalParameterPattern::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> FormalParameterPattern {
        FormalParameterPattern::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> FormalParameterPattern {
        FormalParameterPattern::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> FormalParameterPattern {
        FormalParameterPattern::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern,
        right: Expr,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> FormalParameterPattern {
        FormalParameterPattern::AssignmentPattern(
            self.assignment_pattern(span, left, right, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_binding_rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> FormalParameterPattern {
        FormalParameterPattern::BindingRestElement(
            self.binding_rest_element(span, argument, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_array_pattern(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> FormalParameterPattern {
        FormalParameterPattern::ArrayPattern(
            self.array_pattern(span, elements, rest, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_object_pattern(
        &mut self,
        span: Span,
        properties: TypedSubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> FormalParameterPattern {
        FormalParameterPattern::ObjectPattern(
            self.object_pattern(
                span,
                properties,
                rest,
                decorators,
                type_annotation,
                optional,
            )
            .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_ts_this_parameter(
        &mut self,
        span: Span,
        type_annotation: Option<TsTypeAnnotation>,
    ) -> FormalParameterPattern {
        FormalParameterPattern::TsThisParameter(
            self.ts_this_parameter(span, type_annotation).into(),
        )
    }
    #[inline]
    pub fn class_element_method_definition(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        value: Function,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> ClassElement {
        ClassElement::MethodDefinition(
            self.method_definition(
                span,
                decorators,
                key,
                value,
                kind,
                computed,
                r#static,
                r#override,
                optional,
                r#abstract,
                accessibility,
            )
            .into(),
        )
    }
    #[inline]
    pub fn class_element_property_definition(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        value: Option<Expr>,
        computed: bool,
        r#static: bool,
        accessor: bool,
        type_annotation: Option<TsTypeAnnotation>,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> ClassElement {
        ClassElement::PropertyDefinition(
            self.property_definition(
                span,
                decorators,
                key,
                value,
                computed,
                r#static,
                accessor,
                type_annotation,
                declare,
                r#override,
                optional,
                definite,
                readonly,
                r#abstract,
                accessibility,
            )
            .into(),
        )
    }
    #[inline]
    pub fn class_element_static_block(
        &mut self,
        span: Span,
        body: TypedSubRange<Stmt>,
    ) -> ClassElement {
        ClassElement::StaticBlock(self.static_block(span, body).into())
    }
    #[inline]
    pub fn class_element_ts_method_definition(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        value: TsFunction,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> ClassElement {
        ClassElement::TsMethodDefinition(
            self.ts_method_definition(
                span,
                decorators,
                key,
                value,
                kind,
                computed,
                r#static,
                r#override,
                optional,
                r#abstract,
                accessibility,
            )
            .into(),
        )
    }
    #[inline]
    pub fn class_element_ts_index_signature(
        &mut self,
        span: Span,
        parameters: TypedSubRange<BindingIdentifier>,
        type_annotation: TsTypeAnnotation,
        readonly: bool,
        r#static: bool,
    ) -> ClassElement {
        ClassElement::TsIndexSignature(
            self.ts_index_signature(span, parameters, type_annotation, readonly, r#static)
                .into(),
        )
    }
    #[inline]
    pub fn import_declaration_specifier_import_specifier(
        &mut self,
        span: Span,
        imported: ModuleExportName,
        local: BindingIdentifier,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclarationSpecifier {
        ImportDeclarationSpecifier::ImportSpecifier(
            self.import_specifier(span, imported, local, import_kind)
                .into(),
        )
    }
    #[inline]
    pub fn import_declaration_specifier_import_default_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier,
    ) -> ImportDeclarationSpecifier {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(
            self.import_default_specifier(span, local).into(),
        )
    }
    #[inline]
    pub fn import_declaration_specifier_import_namespace_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier,
    ) -> ImportDeclarationSpecifier {
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(
            self.import_namespace_specifier(span, local).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::IdentifierReference(
            self.identifier_reference(span, name).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_this_expression(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_super(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Super(self.super_(span).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_null_literal(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ChainExpression(
            self.chain_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ObjectExpression(
            self.object_expression(span, properties).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_function_expression(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::FunctionExpression(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_class_expression(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ClassExpression(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::ImportExpression(
            self.import_expression(span, source, options, phase).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        declare: bool,
        params: FormalParameters,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsFunction(
            self.ts_function(
                span,
                id,
                generator,
                r#async,
                declare,
                params,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        extends: TypedSubRange<TsInterfaceHeritage>,
        body: TsInterfaceBody,
        declare: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsInterfaceDeclaration(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
                .into(),
        )
    }
    #[inline]
    pub fn super_(&mut self, span: Span) -> Super {
        Super(self.add_node(span, NodeKind::Super, NodePayload(0)))
    }
    #[inline]
    pub fn this_expression(&mut self, span: Span) -> ThisExpression {
        ThisExpression(self.add_node(span, NodeKind::ThisExpression, NodePayload(0)))
    }
    #[inline]
    pub fn string_literal(&mut self, span: Span, value: AstWtf8, raw: AstUtf8) -> StringLiteral {
        StringLiteral(self.add_node(
            span,
            NodeKind::StringLiteral,
            NodePayload(
                0u128
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { raw.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> NumericLiteral {
        NumericLiteral(self.add_node(
            span,
            NodeKind::NumericLiteral,
            NodePayload(
                0u128
                    | ((kind as u32 as u128) << 64usize)
                    | ((unsafe { raw.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn big_int_literal(&mut self, span: Span, raw: AstUtf8) -> BigIntLiteral {
        BigIntLiteral(self.add_node(
            span,
            NodeKind::BigIntLiteral,
            NodePayload(0u128 | ((unsafe { raw.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn boolean_literal(&mut self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral(self.add_node(
            span,
            NodeKind::BooleanLiteral,
            NodePayload(0u128 | ((value as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn null_literal(&mut self, span: Span) -> NullLiteral {
        NullLiteral(self.add_node(span, NodeKind::NullLiteral, NodePayload(0)))
    }
    #[inline]
    pub fn reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> RegExpLiteral {
        RegExpLiteral(self.add_node(
            span,
            NodeKind::RegExpLiteral,
            NodePayload(
                0u128
                    | ((unsafe { pattern.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { flags.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> TemplateLiteral {
        TemplateLiteral(self.add_node(
            span,
            NodeKind::TemplateLiteral,
            NodePayload(
                0u128
                    | ((unsafe { quasis.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { expressions.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn template_element(
        &mut self,
        span: Span,
        cooked: AstWtf8,
        raw: AstUtf8,
        tail: bool,
        is_cooked_undefined: bool,
    ) -> TemplateElement {
        let _f0 = self.add_extra(raw.to_extra_data(self));
        TemplateElement(self.add_node(
            span,
            NodeKind::TemplateElement,
            NodePayload(
                0u128
                    | ((unsafe { cooked.to_extra_data(self).other } as u128) << 16usize)
                    | ((tail as u32 as u128) << 0usize)
                    | ((is_cooked_undefined as u32 as u128) << 8usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn identifier_reference(&mut self, span: Span, name: AstUtf8) -> IdentifierReference {
        IdentifierReference(self.add_node(
            span,
            NodeKind::IdentifierReference,
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn private_identifier(&mut self, span: Span, name: AstUtf8) -> PrivateIdentifier {
        PrivateIdentifier(self.add_node(
            span,
            NodeKind::PrivateIdentifier,
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> BindingIdentifier {
        let _extra_data = [name.to_extra_data(self), decorators.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        BindingIdentifier(self.add_node(
            span,
            NodeKind::BindingIdentifier,
            NodePayload(
                0u128
                    | ((type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 8usize)
                    | ((optional as u32 as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn identifier_name(&mut self, span: Span, name: AstUtf8) -> IdentifierName {
        IdentifierName(self.add_node(
            span,
            NodeKind::IdentifierName,
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn label_identifier(&mut self, span: Span, name: AstUtf8) -> LabelIdentifier {
        LabelIdentifier(self.add_node(
            span,
            NodeKind::LabelIdentifier,
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern,
        right: Expr,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> AssignmentPattern {
        let _extra_data = [
            decorators.to_extra_data(self),
            type_annotation.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        AssignmentPattern(self.add_node(
            span,
            NodeKind::AssignmentPattern,
            NodePayload(
                0u128
                    | ((left.node_id().index() as u32 as u128) << 8usize)
                    | ((right.node_id().index() as u32 as u128) << 40usize)
                    | ((optional as u32 as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn binding_rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> BindingRestElement {
        let _f0 = self.add_extra(decorators.to_extra_data(self));
        BindingRestElement(self.add_node(
            span,
            NodeKind::BindingRestElement,
            NodePayload(
                0u128
                    | ((argument.node_id().index() as u32 as u128) << 8usize)
                    | ((type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 40usize)
                    | ((optional as u32 as u128) << 0usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn array_pattern(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> ArrayPattern {
        let _extra_data = [elements.to_extra_data(self), decorators.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ArrayPattern(self.add_node(
            span,
            NodeKind::ArrayPattern,
            NodePayload(
                0u128
                    | ((rest.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 8usize)
                    | ((type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 40usize)
                    | ((optional as u32 as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn object_pattern(
        &mut self,
        span: Span,
        properties: TypedSubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> ObjectPattern {
        let _extra_data = [
            properties.to_extra_data(self),
            decorators.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ObjectPattern(self.add_node(
            span,
            NodeKind::ObjectPattern,
            NodePayload(
                0u128
                    | ((rest.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 8usize)
                    | ((type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 40usize)
                    | ((optional as u32 as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn binding_property(
        &mut self,
        span: Span,
        key: PropertyKey,
        value: BindingPattern,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty {
        BindingProperty(self.add_node(
            span,
            NodeKind::BindingProperty,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 0usize)
                    | ((value.node_id().index() as u32 as u128) << 32usize)
                    | ((shorthand as u32 as u128) << 64usize)
                    | ((computed as u32 as u128) << 72usize),
            ),
        ))
    }
    #[inline]
    pub fn binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> BinaryExpression {
        BinaryExpression(self.add_node(
            span,
            NodeKind::BinaryExpression,
            NodePayload(
                0u128
                    | ((left.node_id().index() as u32 as u128) << 0usize)
                    | ((right.node_id().index() as u32 as u128) << 32usize)
                    | ((operator as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> LogicalExpression {
        LogicalExpression(self.add_node(
            span,
            NodeKind::LogicalExpression,
            NodePayload(
                0u128
                    | ((left.node_id().index() as u32 as u128) << 0usize)
                    | ((right.node_id().index() as u32 as u128) << 32usize)
                    | ((operator as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> ConditionalExpression {
        ConditionalExpression(self.add_node(
            span,
            NodeKind::ConditionalExpression,
            NodePayload(
                0u128
                    | ((test.node_id().index() as u32 as u128) << 0usize)
                    | ((consequent.node_id().index() as u32 as u128) << 32usize)
                    | ((alternate.node_id().index() as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> UnaryExpression {
        UnaryExpression(self.add_node(
            span,
            NodeKind::UnaryExpression,
            NodePayload(
                0u128
                    | ((argument.node_id().index() as u32 as u128) << 0usize)
                    | ((operator as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> UpdateExpression {
        UpdateExpression(self.add_node(
            span,
            NodeKind::UpdateExpression,
            NodePayload(
                0u128
                    | ((argument.node_id().index() as u32 as u128) << 0usize)
                    | ((operator as u32 as u128) << 32usize)
                    | ((prefix as u32 as u128) << 40usize),
            ),
        ))
    }
    #[inline]
    pub fn assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> AssignmentExpression {
        AssignmentExpression(self.add_node(
            span,
            NodeKind::AssignmentExpression,
            NodePayload(
                0u128
                    | ((left.node_id().index() as u32 as u128) << 0usize)
                    | ((right.node_id().index() as u32 as u128) << 32usize)
                    | ((operator as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: TypedSubRange<VariableDeclarator>,
        declare: bool,
    ) -> VariableDeclaration {
        VariableDeclaration(self.add_node(
            span,
            NodeKind::VariableDeclaration,
            NodePayload(
                0u128
                    | ((kind as u32 as u128) << 64usize)
                    | ((unsafe { declarators.to_extra_data(self).other } as u128) << 0usize)
                    | ((declare as u32 as u128) << 72usize),
            ),
        ))
    }
    #[inline]
    pub fn variable_declarator(
        &mut self,
        span: Span,
        id: BindingPattern,
        init: Option<Expr>,
        definite: bool,
    ) -> VariableDeclarator {
        VariableDeclarator(self.add_node(
            span,
            NodeKind::VariableDeclarator,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 0usize)
                    | ((init.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((definite as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn if_statement(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Stmt,
        alternate: Option<Stmt>,
    ) -> IfStatement {
        IfStatement(self.add_node(
            span,
            NodeKind::IfStatement,
            NodePayload(
                0u128
                    | ((test.node_id().index() as u32 as u128) << 0usize)
                    | ((consequent.node_id().index() as u32 as u128) << 32usize)
                    | ((alternate.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn switch_statement(
        &mut self,
        span: Span,
        discriminant: Expr,
        cases: TypedSubRange<SwitchCase>,
    ) -> SwitchStatement {
        SwitchStatement(self.add_node(
            span,
            NodeKind::SwitchStatement,
            NodePayload(
                0u128
                    | ((discriminant.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { cases.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn switch_case(
        &mut self,
        span: Span,
        test: Option<Expr>,
        consequent: TypedSubRange<Stmt>,
    ) -> SwitchCase {
        SwitchCase(self.add_node(
            span,
            NodeKind::SwitchCase,
            NodePayload(
                0u128
                    | ((test.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize)
                    | ((unsafe { consequent.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn for_statement(
        &mut self,
        span: Span,
        init: Option<ForStatementInit>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Stmt,
    ) -> ForStatement {
        ForStatement(self.add_node(
            span,
            NodeKind::ForStatement,
            NodePayload(
                0u128
                    | ((init.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize)
                    | ((test.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((update.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize)
                    | ((body.node_id().index() as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn for_in_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft,
        right: Expr,
        body: Stmt,
    ) -> ForInStatement {
        ForInStatement(self.add_node(
            span,
            NodeKind::ForInStatement,
            NodePayload(
                0u128
                    | ((left.node_id().index() as u32 as u128) << 0usize)
                    | ((right.node_id().index() as u32 as u128) << 32usize)
                    | ((body.node_id().index() as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn for_of_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft,
        right: Expr,
        body: Stmt,
        r#await: bool,
    ) -> ForOfStatement {
        ForOfStatement(self.add_node(
            span,
            NodeKind::ForOfStatement,
            NodePayload(
                0u128
                    | ((left.node_id().index() as u32 as u128) << 0usize)
                    | ((right.node_id().index() as u32 as u128) << 32usize)
                    | ((body.node_id().index() as u32 as u128) << 64usize)
                    | ((r#await as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn try_statement(
        &mut self,
        span: Span,
        block: BlockStatement,
        handler: Option<CatchClause>,
        finalizer: Option<BlockStatement>,
    ) -> TryStatement {
        TryStatement(self.add_node(
            span,
            NodeKind::TryStatement,
            NodePayload(
                0u128
                    | ((block.node_id().index() as u32 as u128) << 0usize)
                    | ((handler.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn catch_clause(
        &mut self,
        span: Span,
        param: Option<BindingPattern>,
        body: BlockStatement,
    ) -> CatchClause {
        CatchClause(self.add_node(
            span,
            NodeKind::CatchClause,
            NodePayload(
                0u128
                    | ((param.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn break_statement(
        &mut self,
        span: Span,
        label: Option<LabelIdentifier>,
    ) -> BreakStatement {
        BreakStatement(self.add_node(
            span,
            NodeKind::BreakStatement,
            NodePayload(
                0u128
                    | ((label.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn continue_statement(
        &mut self,
        span: Span,
        label: Option<LabelIdentifier>,
    ) -> ContinueStatement {
        ContinueStatement(self.add_node(
            span,
            NodeKind::ContinueStatement,
            NodePayload(
                0u128
                    | ((label.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn labeled_statement(
        &mut self,
        span: Span,
        label: LabelIdentifier,
        body: Stmt,
    ) -> LabeledStatement {
        LabeledStatement(self.add_node(
            span,
            NodeKind::LabeledStatement,
            NodePayload(
                0u128
                    | ((label.node_id().index() as u32 as u128) << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn expression_statement(&mut self, span: Span, expression: Expr) -> ExpressionStatement {
        ExpressionStatement(self.add_node(
            span,
            NodeKind::ExpressionStatement,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn return_statement(&mut self, span: Span, argument: Option<Expr>) -> ReturnStatement {
        ReturnStatement(self.add_node(
            span,
            NodeKind::ReturnStatement,
            NodePayload(
                0u128
                    | ((argument.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn throw_statement(&mut self, span: Span, argument: Expr) -> ThrowStatement {
        ThrowStatement(self.add_node(
            span,
            NodeKind::ThrowStatement,
            NodePayload(0u128 | ((argument.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn debugger_statement(&mut self, span: Span) -> DebuggerStatement {
        DebuggerStatement(self.add_node(span, NodeKind::DebuggerStatement, NodePayload(0)))
    }
    #[inline]
    pub fn empty_statement(&mut self, span: Span) -> EmptyStatement {
        EmptyStatement(self.add_node(span, NodeKind::EmptyStatement, NodePayload(0)))
    }
    #[inline]
    pub fn directive(
        &mut self,
        span: Span,
        expression: StringLiteral,
        value: AstUtf8,
    ) -> Directive {
        Directive(self.add_node(
            span,
            NodeKind::Directive,
            NodePayload(
                0u128
                    | ((expression.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn decorator(&mut self, span: Span, expression: Expr) -> Decorator {
        Decorator(self.add_node(
            span,
            NodeKind::Decorator,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> Class {
        let _extra_data = [
            decorators.to_extra_data(self),
            body.to_extra_data(self),
            type_parameters.to_extra_data(self),
            super_type_arguments.to_extra_data(self),
            implements.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        Class(self.add_node(
            span,
            NodeKind::Class,
            NodePayload(
                0u128
                    | ((class_type as u32 as u128) << 0usize)
                    | ((id.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 24usize)
                    | ((super_class.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 56usize)
                    | ((r#abstract as u32 as u128) << 8usize)
                    | ((declare as u32 as u128) << 16usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn class_body(&mut self, span: Span, body: TypedSubRange<ClassElement>) -> ClassBody {
        ClassBody(self.add_node(
            span,
            NodeKind::ClassBody,
            NodePayload(0u128 | ((unsafe { body.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn method_definition(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        value: Function,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> MethodDefinition {
        let _extra_data = [decorators.to_extra_data(self), value.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        MethodDefinition(self.add_node(
            span,
            NodeKind::MethodDefinition,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 56usize)
                    | ((kind as u32 as u128) << 0usize)
                    | ((computed as u32 as u128) << 8usize)
                    | ((r#static as u32 as u128) << 16usize)
                    | ((r#override as u32 as u128) << 24usize)
                    | ((optional as u32 as u128) << 32usize)
                    | ((r#abstract as u32 as u128) << 40usize)
                    | ((accessibility as u32 as u128) << 48usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn property_definition(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        value: Option<Expr>,
        computed: bool,
        r#static: bool,
        accessor: bool,
        type_annotation: Option<TsTypeAnnotation>,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> PropertyDefinition {
        let _extra_data = [
            decorators.to_extra_data(self),
            key.to_extra_data(self),
            value.to_extra_data(self),
            type_annotation.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        PropertyDefinition(self.add_node(
            span,
            NodeKind::PropertyDefinition,
            NodePayload(
                0u128
                    | ((computed as u32 as u128) << 0usize)
                    | ((r#static as u32 as u128) << 8usize)
                    | ((accessor as u32 as u128) << 16usize)
                    | ((declare as u32 as u128) << 24usize)
                    | ((r#override as u32 as u128) << 32usize)
                    | ((optional as u32 as u128) << 40usize)
                    | ((definite as u32 as u128) << 48usize)
                    | ((readonly as u32 as u128) << 56usize)
                    | ((r#abstract as u32 as u128) << 64usize)
                    | ((accessibility as u32 as u128) << 72usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn static_block(&mut self, span: Span, body: TypedSubRange<Stmt>) -> StaticBlock {
        StaticBlock(self.add_node(
            span,
            NodeKind::StaticBlock,
            NodePayload(0u128 | ((unsafe { body.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> Function {
        let _extra_data = [
            body.to_extra_data(self),
            type_parameters.to_extra_data(self),
            return_type.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        Function(self.add_node(
            span,
            NodeKind::Function,
            NodePayload(
                0u128
                    | ((function_type as u32 as u128) << 0usize)
                    | ((id.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 24usize)
                    | ((generator as u32 as u128) << 8usize)
                    | ((r#async as u32 as u128) << 16usize)
                    | ((params.node_id().index() as u32 as u128) << 56usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn function_body(
        &mut self,
        span: Span,
        directives: TypedSubRange<Directive>,
        body: TypedSubRange<Stmt>,
    ) -> FunctionBody {
        FunctionBody(self.add_node(
            span,
            NodeKind::FunctionBody,
            NodePayload(
                0u128
                    | ((unsafe { directives.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn block_statement(&mut self, span: Span, body: TypedSubRange<Stmt>) -> BlockStatement {
        BlockStatement(self.add_node(
            span,
            NodeKind::BlockStatement,
            NodePayload(0u128 | ((unsafe { body.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn while_statement(&mut self, span: Span, test: Expr, body: Stmt) -> WhileStatement {
        WhileStatement(self.add_node(
            span,
            NodeKind::WhileStatement,
            NodePayload(
                0u128
                    | ((test.node_id().index() as u32 as u128) << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn do_while_statement(&mut self, span: Span, body: Stmt, test: Expr) -> DoWhileStatement {
        DoWhileStatement(self.add_node(
            span,
            NodeKind::DoWhileStatement,
            NodePayload(
                0u128
                    | ((body.node_id().index() as u32 as u128) << 0usize)
                    | ((test.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn with_statement(&mut self, span: Span, object: Expr, body: Stmt) -> WithStatement {
        WithStatement(self.add_node(
            span,
            NodeKind::WithStatement,
            NodePayload(
                0u128
                    | ((object.node_id().index() as u32 as u128) << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn formal_parameters(
        &mut self,
        span: Span,
        items: TypedSubRange<FormalParameterItem>,
        rest: Option<BindingRestElement>,
        kind: FormalParameterKind,
    ) -> FormalParameters {
        FormalParameters(self.add_node(
            span,
            NodeKind::FormalParameters,
            NodePayload(
                0u128
                    | ((unsafe { items.to_extra_data(self).other } as u128) << 0usize)
                    | ((rest.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize)
                    | ((kind as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn formal_parameter(
        &mut self,
        span: Span,
        pattern: FormalParameterPattern,
    ) -> FormalParameter {
        FormalParameter(self.add_node(
            span,
            NodeKind::FormalParameter,
            NodePayload(0u128 | ((pattern.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> ArrayExpression {
        ArrayExpression(self.add_node(
            span,
            NodeKind::ArrayExpression,
            NodePayload(
                0u128 | ((unsafe { elements.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> ObjectExpression {
        ObjectExpression(self.add_node(
            span,
            NodeKind::ObjectExpression,
            NodePayload(
                0u128 | ((unsafe { properties.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn spread_element(&mut self, span: Span, argument: Expr) -> SpreadElement {
        SpreadElement(self.add_node(
            span,
            NodeKind::SpreadElement,
            NodePayload(0u128 | ((argument.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn object_property(
        &mut self,
        span: Span,
        key: PropertyKey,
        value: Expr,
        kind: PropertyKind,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectProperty {
        ObjectProperty(self.add_node(
            span,
            NodeKind::ObjectProperty,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 0usize)
                    | ((value.node_id().index() as u32 as u128) << 32usize)
                    | ((kind as u32 as u128) << 64usize)
                    | ((method as u32 as u128) << 72usize)
                    | ((shorthand as u32 as u128) << 80usize)
                    | ((computed as u32 as u128) << 88usize),
            ),
        ))
    }
    #[inline]
    pub fn parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ParenthesizedExpression {
        ParenthesizedExpression(self.add_node(
            span,
            NodeKind::ParenthesizedExpression,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> ArrowFunctionExpression {
        let _extra_data = [
            type_parameters.to_extra_data(self),
            return_type.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ArrowFunctionExpression(self.add_node(
            span,
            NodeKind::ArrowFunctionExpression,
            NodePayload(
                0u128
                    | ((expression as u32 as u128) << 0usize)
                    | ((r#async as u32 as u128) << 8usize)
                    | ((params.node_id().index() as u32 as u128) << 16usize)
                    | ((body.node_id().index() as u32 as u128) << 48usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> SequenceExpression {
        SequenceExpression(self.add_node(
            span,
            NodeKind::SequenceExpression,
            NodePayload(
                0u128 | ((unsafe { expressions.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> MemberExpression {
        MemberExpression(self.add_node(
            span,
            NodeKind::MemberExpression,
            NodePayload(
                0u128
                    | ((object.node_id().index() as u32 as u128) << 0usize)
                    | ((property.node_id().index() as u32 as u128) << 32usize)
                    | ((computed as u32 as u128) << 64usize)
                    | ((optional as u32 as u128) << 72usize),
            ),
        ))
    }
    #[inline]
    pub fn call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> CallExpression {
        let _f0 = self.add_extra(arguments.to_extra_data(self));
        CallExpression(self.add_node(
            span,
            NodeKind::CallExpression,
            NodePayload(
                0u128
                    | ((callee.node_id().index() as u32 as u128) << 8usize)
                    | ((optional as u32 as u128) << 0usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 40usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn chain_expression(&mut self, span: Span, expression: Expr) -> ChainExpression {
        ChainExpression(self.add_node(
            span,
            NodeKind::ChainExpression,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TaggedTemplateExpression {
        TaggedTemplateExpression(self.add_node(
            span,
            NodeKind::TaggedTemplateExpression,
            NodePayload(
                0u128
                    | ((tag.node_id().index() as u32 as u128) << 0usize)
                    | ((quasi.node_id().index() as u32 as u128) << 32usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> NewExpression {
        NewExpression(self.add_node(
            span,
            NodeKind::NewExpression,
            NodePayload(
                0u128
                    | ((callee.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { arguments.to_extra_data(self).other } as u128) << 0usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn await_expression(&mut self, span: Span, argument: Expr) -> AwaitExpression {
        AwaitExpression(self.add_node(
            span,
            NodeKind::AwaitExpression,
            NodePayload(0u128 | ((argument.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> YieldExpression {
        YieldExpression(self.add_node(
            span,
            NodeKind::YieldExpression,
            NodePayload(
                0u128
                    | ((argument.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize)
                    | ((delegate as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> MetaProperty {
        MetaProperty(self.add_node(
            span,
            NodeKind::MetaProperty,
            NodePayload(
                0u128
                    | ((meta.node_id().index() as u32 as u128) << 0usize)
                    | ((property.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> ImportExpression {
        let _f0 = self.add_extra(phase.to_extra_data(self));
        ImportExpression(self.add_node(
            span,
            NodeKind::ImportExpression,
            NodePayload(
                0u128
                    | ((source.node_id().index() as u32 as u128) << 0usize)
                    | ((options.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn import_declaration(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ImportDeclarationSpecifier>,
        source: StringLiteral,
        attributes: TypedSubRange<ImportAttribute>,
        phase: Option<ImportPhase>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclaration {
        let _extra_data = [
            specifiers.to_extra_data(self),
            attributes.to_extra_data(self),
            phase.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ImportDeclaration(self.add_node(
            span,
            NodeKind::ImportDeclaration,
            NodePayload(
                0u128
                    | ((source.node_id().index() as u32 as u128) << 8usize)
                    | ((import_kind as u32 as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn import_specifier(
        &mut self,
        span: Span,
        imported: ModuleExportName,
        local: BindingIdentifier,
        import_kind: ImportOrExportKind,
    ) -> ImportSpecifier {
        ImportSpecifier(self.add_node(
            span,
            NodeKind::ImportSpecifier,
            NodePayload(
                0u128
                    | ((imported.node_id().index() as u32 as u128) << 0usize)
                    | ((local.node_id().index() as u32 as u128) << 32usize)
                    | ((import_kind as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn import_default_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier,
    ) -> ImportDefaultSpecifier {
        ImportDefaultSpecifier(self.add_node(
            span,
            NodeKind::ImportDefaultSpecifier,
            NodePayload(0u128 | ((local.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn import_namespace_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier,
    ) -> ImportNamespaceSpecifier {
        ImportNamespaceSpecifier(self.add_node(
            span,
            NodeKind::ImportNamespaceSpecifier,
            NodePayload(0u128 | ((local.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn import_attribute(
        &mut self,
        span: Span,
        key: ModuleExportName,
        value: StringLiteral,
    ) -> ImportAttribute {
        ImportAttribute(self.add_node(
            span,
            NodeKind::ImportAttribute,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 0usize)
                    | ((value.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn export_named_declaration(
        &mut self,
        span: Span,
        declaration: Option<Decl>,
        specifiers: TypedSubRange<ExportSpecifier>,
        source: Option<StringLiteral>,
        attributes: TypedSubRange<ImportAttribute>,
        export_kind: ImportOrExportKind,
    ) -> ExportNamedDeclaration {
        let _extra_data = [
            specifiers.to_extra_data(self),
            attributes.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ExportNamedDeclaration(self.add_node(
            span,
            NodeKind::ExportNamedDeclaration,
            NodePayload(
                0u128
                    | ((declaration.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 8usize)
                    | ((source.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 40usize)
                    | ((export_kind as u32 as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn export_default_declaration(
        &mut self,
        span: Span,
        declaration: ExportDefaultDeclarationKind,
    ) -> ExportDefaultDeclaration {
        ExportDefaultDeclaration(self.add_node(
            span,
            NodeKind::ExportDefaultDeclaration,
            NodePayload(0u128 | ((declaration.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn export_all_declaration(
        &mut self,
        span: Span,
        exported: Option<ModuleExportName>,
        source: StringLiteral,
        attributes: TypedSubRange<ImportAttribute>,
        export_kind: ImportOrExportKind,
    ) -> ExportAllDeclaration {
        let _f0 = self.add_extra(attributes.to_extra_data(self));
        ExportAllDeclaration(self.add_node(
            span,
            NodeKind::ExportAllDeclaration,
            NodePayload(
                0u128
                    | ((exported.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 8usize)
                    | ((source.node_id().index() as u32 as u128) << 40usize)
                    | ((export_kind as u32 as u128) << 0usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn export_specifier(
        &mut self,
        span: Span,
        local: ModuleExportName,
        exported: ModuleExportName,
        export_kind: ImportOrExportKind,
    ) -> ExportSpecifier {
        ExportSpecifier(self.add_node(
            span,
            NodeKind::ExportSpecifier,
            NodePayload(
                0u128
                    | ((local.node_id().index() as u32 as u128) << 0usize)
                    | ((exported.node_id().index() as u32 as u128) << 32usize)
                    | ((export_kind as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn hashbang(&mut self, span: Span, value: AstUtf8) -> Hashbang {
        Hashbang(self.add_node(
            span,
            NodeKind::Hashbang,
            NodePayload(0u128 | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_type_ts_any_keyword(&mut self, span: Span) -> TsType {
        TsType::TsAnyKeyword(self.ts_any_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_unknown_keyword(&mut self, span: Span) -> TsType {
        TsType::TsUnknownKeyword(self.ts_unknown_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_never_keyword(&mut self, span: Span) -> TsType {
        TsType::TsNeverKeyword(self.ts_never_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_void_keyword(&mut self, span: Span) -> TsType {
        TsType::TsVoidKeyword(self.ts_void_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_null_keyword(&mut self, span: Span) -> TsType {
        TsType::TsNullKeyword(self.ts_null_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_undefined_keyword(&mut self, span: Span) -> TsType {
        TsType::TsUndefinedKeyword(self.ts_undefined_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_string_keyword(&mut self, span: Span) -> TsType {
        TsType::TsStringKeyword(self.ts_string_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_number_keyword(&mut self, span: Span) -> TsType {
        TsType::TsNumberKeyword(self.ts_number_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_big_int_keyword(&mut self, span: Span) -> TsType {
        TsType::TsBigIntKeyword(self.ts_big_int_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_boolean_keyword(&mut self, span: Span) -> TsType {
        TsType::TsBooleanKeyword(self.ts_boolean_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_symbol_keyword(&mut self, span: Span) -> TsType {
        TsType::TsSymbolKeyword(self.ts_symbol_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_object_keyword(&mut self, span: Span) -> TsType {
        TsType::TsObjectKeyword(self.ts_object_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_intrinsic_keyword(&mut self, span: Span) -> TsType {
        TsType::TsIntrinsicKeyword(self.ts_intrinsic_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_this_type(&mut self, span: Span) -> TsType {
        TsType::TsThisType(self.ts_this_type(span).into())
    }
    #[inline]
    pub fn ts_type_ts_type_reference(
        &mut self,
        span: Span,
        type_name: TsTypeName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsType {
        TsType::TsTypeReference(
            self.ts_type_reference(span, type_name, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_type_query(
        &mut self,
        span: Span,
        expr_name: TsTypeQueryExprName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsType {
        TsType::TsTypeQuery(self.ts_type_query(span, expr_name, type_arguments).into())
    }
    #[inline]
    pub fn ts_type_ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral,
        options: Option<Expr>,
        qualifier: Option<TsImportTypeQualifier>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsType {
        TsType::TsImportType(
            self.ts_import_type(span, source, options, qualifier, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_literal_type(&mut self, span: Span, literal: TsLiteral) -> TsType {
        TsType::TsLiteralType(self.ts_literal_type(span, literal).into())
    }
    #[inline]
    pub fn ts_type_ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        types: TypedSubRange<TsType>,
    ) -> TsType {
        TsType::TsTemplateLiteralType(self.ts_template_literal_type(span, quasis, types).into())
    }
    #[inline]
    pub fn ts_type_ts_array_type(&mut self, span: Span, element_type: TsType) -> TsType {
        TsType::TsArrayType(self.ts_array_type(span, element_type).into())
    }
    #[inline]
    pub fn ts_type_ts_indexed_access_type(
        &mut self,
        span: Span,
        object_type: TsType,
        index_type: TsType,
    ) -> TsType {
        TsType::TsIndexedAccessType(
            self.ts_indexed_access_type(span, object_type, index_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_tuple_type(
        &mut self,
        span: Span,
        element_types: TypedSubRange<TsTupleElement>,
    ) -> TsType {
        TsType::TsTupleType(self.ts_tuple_type(span, element_types).into())
    }
    #[inline]
    pub fn ts_type_ts_optional_type(&mut self, span: Span, type_annotation: TsType) -> TsType {
        TsType::TsOptionalType(self.ts_optional_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_type_ts_rest_type(&mut self, span: Span, type_annotation: TsType) -> TsType {
        TsType::TsRestType(self.ts_rest_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_type_ts_js_doc_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsType {
        TsType::TsJsDocNullableType(
            self.ts_js_doc_nullable_type(span, type_annotation, postfix)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_js_doc_non_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsType {
        TsType::TsJsDocNonNullableType(
            self.ts_js_doc_non_nullable_type(span, type_annotation, postfix)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_js_doc_unknown_type(&mut self, span: Span) -> TsType {
        TsType::TsJsDocUnknownType(self.ts_js_doc_unknown_type(span).into())
    }
    #[inline]
    pub fn ts_type_ts_union_type(&mut self, span: Span, types: TypedSubRange<TsType>) -> TsType {
        TsType::TsUnionType(self.ts_union_type(span, types).into())
    }
    #[inline]
    pub fn ts_type_ts_intersection_type(
        &mut self,
        span: Span,
        types: TypedSubRange<TsType>,
    ) -> TsType {
        TsType::TsIntersectionType(self.ts_intersection_type(span, types).into())
    }
    #[inline]
    pub fn ts_type_ts_conditional_type(
        &mut self,
        span: Span,
        check_type: TsType,
        extends_type: TsType,
        true_type: TsType,
        false_type: TsType,
    ) -> TsType {
        TsType::TsConditionalType(
            self.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_infer_type(&mut self, span: Span, type_parameter: TsTypeParameter) -> TsType {
        TsType::TsInferType(self.ts_infer_type(span, type_parameter).into())
    }
    #[inline]
    pub fn ts_type_ts_type_operator(
        &mut self,
        span: Span,
        operator: TsTypeOperatorKind,
        type_annotation: TsType,
    ) -> TsType {
        TsType::TsTypeOperator(
            self.ts_type_operator(span, operator, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_parenthesized_type(&mut self, span: Span, type_annotation: TsType) -> TsType {
        TsType::TsParenthesizedType(self.ts_parenthesized_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_type_ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
    ) -> TsType {
        TsType::TsFunctionType(
            self.ts_function_type(span, type_parameters, params, return_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
        r#abstract: bool,
    ) -> TsType {
        TsType::TsConstructorType(
            self.ts_constructor_type(span, type_parameters, params, return_type, r#abstract)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_type_predicate(
        &mut self,
        span: Span,
        parameter_name: TsTypePredicateName,
        type_annotation: Option<TsTypeAnnotation>,
        asserts: bool,
    ) -> TsType {
        TsType::TsTypePredicate(
            self.ts_type_predicate(span, parameter_name, type_annotation, asserts)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_type_literal(
        &mut self,
        span: Span,
        members: TypedSubRange<TsTypeElement>,
    ) -> TsType {
        TsType::TsTypeLiteral(self.ts_type_literal(span, members).into())
    }
    #[inline]
    pub fn ts_type_ts_mapped_type(
        &mut self,
        span: Span,
        key: BindingIdentifier,
        constraint: TsType,
        name_type: Option<TsType>,
        type_annotation: Option<TsType>,
        optional: TsMappedTypeModifier,
        readonly: TsMappedTypeModifier,
    ) -> TsType {
        TsType::TsMappedType(
            self.ts_mapped_type(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_type_name_identifier_reference(&mut self, span: Span, name: AstUtf8) -> TsTypeName {
        TsTypeName::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn ts_type_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsTypeName {
        TsTypeName::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsQualifiedNameLeft {
        TsQualifiedNameLeft::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsQualifiedNameLeft {
        TsQualifiedNameLeft::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> TsQualifiedNameLeft {
        TsQualifiedNameLeft::BindingIdentifier(
            self.binding_identifier(span, name, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn ts_qualified_name_left_this_expression(&mut self, span: Span) -> TsQualifiedNameLeft {
        TsQualifiedNameLeft::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsQualifiedNameLeft {
        TsQualifiedNameLeft::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_import_type_qualifier_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsImportTypeQualifier {
        TsImportTypeQualifier::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_import_type_qualifier_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsImportTypeQualifier {
        TsImportTypeQualifier::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_type_query_expr_name_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsTypeQueryExprName {
        TsTypeQueryExprName::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn ts_type_query_expr_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsTypeQueryExprName {
        TsTypeQueryExprName::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_type_query_expr_name_this_expression(&mut self, span: Span) -> TsTypeQueryExprName {
        TsTypeQueryExprName::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn ts_type_query_expr_name_ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral,
        options: Option<Expr>,
        qualifier: Option<TsImportTypeQualifier>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTypeQueryExprName {
        TsTypeQueryExprName::TsImportType(
            self.ts_import_type(span, source, options, qualifier, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_literal_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> TsLiteral {
        TsLiteral::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn ts_literal_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> TsLiteral {
        TsLiteral::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn ts_literal_big_int_literal(&mut self, span: Span, raw: AstUtf8) -> TsLiteral {
        TsLiteral::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn ts_literal_boolean_literal(&mut self, span: Span, value: bool) -> TsLiteral {
        TsLiteral::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn ts_literal_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> TsLiteral {
        TsLiteral::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn ts_literal_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> TsLiteral {
        TsLiteral::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_any_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsAnyKeyword(self.ts_any_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_unknown_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsUnknownKeyword(self.ts_unknown_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_never_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsNeverKeyword(self.ts_never_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_void_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsVoidKeyword(self.ts_void_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_null_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsNullKeyword(self.ts_null_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_undefined_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsUndefinedKeyword(self.ts_undefined_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_string_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsStringKeyword(self.ts_string_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_number_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsNumberKeyword(self.ts_number_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_big_int_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsBigIntKeyword(self.ts_big_int_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_boolean_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsBooleanKeyword(self.ts_boolean_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_symbol_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsSymbolKeyword(self.ts_symbol_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_object_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsObjectKeyword(self.ts_object_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_intrinsic_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsIntrinsicKeyword(self.ts_intrinsic_keyword(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_this_type(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsThisType(self.ts_this_type(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_reference(
        &mut self,
        span: Span,
        type_name: TsTypeName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTupleElement {
        TsTupleElement::TsTypeReference(
            self.ts_type_reference(span, type_name, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_query(
        &mut self,
        span: Span,
        expr_name: TsTypeQueryExprName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTupleElement {
        TsTupleElement::TsTypeQuery(self.ts_type_query(span, expr_name, type_arguments).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral,
        options: Option<Expr>,
        qualifier: Option<TsImportTypeQualifier>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTupleElement {
        TsTupleElement::TsImportType(
            self.ts_import_type(span, source, options, qualifier, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_literal_type(
        &mut self,
        span: Span,
        literal: TsLiteral,
    ) -> TsTupleElement {
        TsTupleElement::TsLiteralType(self.ts_literal_type(span, literal).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        types: TypedSubRange<TsType>,
    ) -> TsTupleElement {
        TsTupleElement::TsTemplateLiteralType(
            self.ts_template_literal_type(span, quasis, types).into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_array_type(
        &mut self,
        span: Span,
        element_type: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsArrayType(self.ts_array_type(span, element_type).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_indexed_access_type(
        &mut self,
        span: Span,
        object_type: TsType,
        index_type: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsIndexedAccessType(
            self.ts_indexed_access_type(span, object_type, index_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_tuple_type(
        &mut self,
        span: Span,
        element_types: TypedSubRange<TsTupleElement>,
    ) -> TsTupleElement {
        TsTupleElement::TsTupleType(self.ts_tuple_type(span, element_types).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_optional_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsOptionalType(self.ts_optional_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_rest_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsRestType(self.ts_rest_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_js_doc_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsJsDocNullableType(
            self.ts_js_doc_nullable_type(span, type_annotation, postfix)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_js_doc_non_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsJsDocNonNullableType(
            self.ts_js_doc_non_nullable_type(span, type_annotation, postfix)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_js_doc_unknown_type(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsJsDocUnknownType(self.ts_js_doc_unknown_type(span).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_union_type(
        &mut self,
        span: Span,
        types: TypedSubRange<TsType>,
    ) -> TsTupleElement {
        TsTupleElement::TsUnionType(self.ts_union_type(span, types).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_intersection_type(
        &mut self,
        span: Span,
        types: TypedSubRange<TsType>,
    ) -> TsTupleElement {
        TsTupleElement::TsIntersectionType(self.ts_intersection_type(span, types).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_conditional_type(
        &mut self,
        span: Span,
        check_type: TsType,
        extends_type: TsType,
        true_type: TsType,
        false_type: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsConditionalType(
            self.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_infer_type(
        &mut self,
        span: Span,
        type_parameter: TsTypeParameter,
    ) -> TsTupleElement {
        TsTupleElement::TsInferType(self.ts_infer_type(span, type_parameter).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_operator(
        &mut self,
        span: Span,
        operator: TsTypeOperatorKind,
        type_annotation: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsTypeOperator(
            self.ts_type_operator(span, operator, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_parenthesized_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsParenthesizedType(
            self.ts_parenthesized_type(span, type_annotation).into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
    ) -> TsTupleElement {
        TsTupleElement::TsFunctionType(
            self.ts_function_type(span, type_parameters, params, return_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
        r#abstract: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsConstructorType(
            self.ts_constructor_type(span, type_parameters, params, return_type, r#abstract)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_predicate(
        &mut self,
        span: Span,
        parameter_name: TsTypePredicateName,
        type_annotation: Option<TsTypeAnnotation>,
        asserts: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsTypePredicate(
            self.ts_type_predicate(span, parameter_name, type_annotation, asserts)
                .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_literal(
        &mut self,
        span: Span,
        members: TypedSubRange<TsTypeElement>,
    ) -> TsTupleElement {
        TsTupleElement::TsTypeLiteral(self.ts_type_literal(span, members).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_mapped_type(
        &mut self,
        span: Span,
        key: BindingIdentifier,
        constraint: TsType,
        name_type: Option<TsType>,
        type_annotation: Option<TsType>,
        optional: TsMappedTypeModifier,
        readonly: TsMappedTypeModifier,
    ) -> TsTupleElement {
        TsTupleElement::TsMappedType(
            self.ts_mapped_type(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_tuple_element_ts_named_tuple_member(
        &mut self,
        span: Span,
        label: IdentifierName,
        element_type: TsType,
        optional: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsNamedTupleMember(
            self.ts_named_tuple_member(span, label, element_type, optional)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_property_signature(
        &mut self,
        span: Span,
        key: PropertyKey,
        type_annotation: Option<TsTypeAnnotation>,
        computed: bool,
        optional: bool,
        readonly: bool,
    ) -> TsTypeElement {
        TsTypeElement::TsPropertySignature(
            self.ts_property_signature(span, key, type_annotation, computed, optional, readonly)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_method_signature(
        &mut self,
        span: Span,
        key: PropertyKey,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        kind: TsMethodSignatureKind,
        computed: bool,
        optional: bool,
    ) -> TsTypeElement {
        TsTypeElement::TsMethodSignature(
            self.ts_method_signature(
                span,
                key,
                type_parameters,
                params,
                return_type,
                kind,
                computed,
                optional,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_call_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsTypeElement {
        TsTypeElement::TsCallSignatureDeclaration(
            self.ts_call_signature_declaration(span, type_parameters, params, return_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_construct_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsTypeElement {
        TsTypeElement::TsConstructSignatureDeclaration(
            self.ts_construct_signature_declaration(span, type_parameters, params, return_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_index_signature(
        &mut self,
        span: Span,
        parameters: TypedSubRange<BindingIdentifier>,
        type_annotation: TsTypeAnnotation,
        readonly: bool,
        r#static: bool,
    ) -> TsTypeElement {
        TsTypeElement::TsIndexSignature(
            self.ts_index_signature(span, parameters, type_annotation, readonly, r#static)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_predicate_name_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsTypePredicateName {
        TsTypePredicateName::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_type_predicate_name_ts_this_type(&mut self, span: Span) -> TsTypePredicateName {
        TsTypePredicateName::TsThisType(self.ts_this_type(span).into())
    }
    #[inline]
    pub fn ts_enum_member_name_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsEnumMemberName {
        TsEnumMemberName::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_enum_member_name_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> TsEnumMemberName {
        TsEnumMemberName::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn ts_enum_member_name_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn ts_enum_member_name_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsEnumMemberName {
        TsEnumMemberName::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn ts_enum_member_name_this_expression(&mut self, span: Span) -> TsEnumMemberName {
        TsEnumMemberName::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn ts_enum_member_name_super(&mut self, span: Span) -> TsEnumMemberName {
        TsEnumMemberName::Super(self.super_(span).into())
    }
    #[inline]
    pub fn ts_enum_member_name_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> TsEnumMemberName {
        TsEnumMemberName::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn ts_enum_member_name_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> TsEnumMemberName {
        TsEnumMemberName::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn ts_enum_member_name_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> TsEnumMemberName {
        TsEnumMemberName::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn ts_enum_member_name_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> TsEnumMemberName {
        TsEnumMemberName::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn ts_enum_member_name_null_literal(&mut self, span: Span) -> TsEnumMemberName {
        TsEnumMemberName::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn ts_enum_member_name_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> TsEnumMemberName {
        TsEnumMemberName::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn ts_enum_member_name_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> TsEnumMemberName {
        TsEnumMemberName::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> TsEnumMemberName {
        TsEnumMemberName::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> TsEnumMemberName {
        TsEnumMemberName::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn ts_enum_member_name_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> TsEnumMemberName {
        TsEnumMemberName::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> TsEnumMemberName {
        TsEnumMemberName::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> TsEnumMemberName {
        TsEnumMemberName::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn ts_enum_member_name_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> TsEnumMemberName {
        TsEnumMemberName::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> TsEnumMemberName {
        TsEnumMemberName::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn ts_enum_member_name_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn ts_enum_member_name_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn ts_enum_member_name_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> TsEnumMemberName {
        TsEnumMemberName::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> TsEnumMemberName {
        TsEnumMemberName::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::ImportExpression(
            self.import_expression(span, source, options, phase).into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> TsEnumMemberName {
        TsEnumMemberName::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn ts_enum_member_name_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> TsEnumMemberName {
        TsEnumMemberName::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn ts_enum_member_name_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsEnumMemberName {
        TsEnumMemberName::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn ts_enum_member_name_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn ts_enum_member_name_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn ts_enum_member_name_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> TsEnumMemberName {
        TsEnumMemberName::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn ts_module_name_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
        decorators: TypedSubRange<Decorator>,
        type_annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> TsModuleName {
        TsModuleName::BindingIdentifier(
            self.binding_identifier(span, name, decorators, type_annotation, optional)
                .into(),
        )
    }
    #[inline]
    pub fn ts_module_name_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> TsModuleName {
        TsModuleName::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn ts_module_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsModuleName {
        TsModuleName::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_module_body_ts_module_block(
        &mut self,
        span: Span,
        directives: TypedSubRange<Directive>,
        body: TypedSubRange<Stmt>,
    ) -> TsModuleBody {
        TsModuleBody::TsModuleBlock(self.ts_module_block(span, directives, body).into())
    }
    #[inline]
    pub fn ts_module_body_ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName,
        body: Option<TsModuleBody>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> TsModuleBody {
        TsModuleBody::TsModuleDeclaration(
            self.ts_module_declaration(span, id, body, kind, declare)
                .into(),
        )
    }
    #[inline]
    pub fn ts_module_reference_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsModuleReference {
        TsModuleReference::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn ts_module_reference_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsModuleReference {
        TsModuleReference::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_module_reference_ts_external_module_reference(
        &mut self,
        span: Span,
        expression: StringLiteral,
    ) -> TsModuleReference {
        TsModuleReference::TsExternalModuleReference(
            self.ts_external_module_reference(span, expression).into(),
        )
    }
    #[inline]
    pub fn ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        declare: bool,
        params: FormalParameters,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsFunction {
        let _extra_data = [
            type_parameters.to_extra_data(self),
            return_type.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        TsFunction(self.add_node(
            span,
            NodeKind::TsFunction,
            NodePayload(
                0u128
                    | ((id.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 24usize)
                    | ((generator as u32 as u128) << 0usize)
                    | ((r#async as u32 as u128) << 8usize)
                    | ((declare as u32 as u128) << 16usize)
                    | ((params.node_id().index() as u32 as u128) << 56usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_method_definition(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        value: TsFunction,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> TsMethodDefinition {
        let _extra_data = [decorators.to_extra_data(self), value.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        TsMethodDefinition(self.add_node(
            span,
            NodeKind::TsMethodDefinition,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 56usize)
                    | ((kind as u32 as u128) << 0usize)
                    | ((computed as u32 as u128) << 8usize)
                    | ((r#static as u32 as u128) << 16usize)
                    | ((r#override as u32 as u128) << 24usize)
                    | ((optional as u32 as u128) << 32usize)
                    | ((r#abstract as u32 as u128) << 40usize)
                    | ((accessibility as u32 as u128) << 48usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_annotation(&mut self, span: Span, type_annotation: TsType) -> TsTypeAnnotation {
        TsTypeAnnotation(self.add_node(
            span,
            NodeKind::TsTypeAnnotation,
            NodePayload(0u128 | ((type_annotation.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_any_keyword(&mut self, span: Span) -> TsAnyKeyword {
        TsAnyKeyword(self.add_node(span, NodeKind::TsAnyKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_unknown_keyword(&mut self, span: Span) -> TsUnknownKeyword {
        TsUnknownKeyword(self.add_node(span, NodeKind::TsUnknownKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_never_keyword(&mut self, span: Span) -> TsNeverKeyword {
        TsNeverKeyword(self.add_node(span, NodeKind::TsNeverKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_void_keyword(&mut self, span: Span) -> TsVoidKeyword {
        TsVoidKeyword(self.add_node(span, NodeKind::TsVoidKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_null_keyword(&mut self, span: Span) -> TsNullKeyword {
        TsNullKeyword(self.add_node(span, NodeKind::TsNullKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_undefined_keyword(&mut self, span: Span) -> TsUndefinedKeyword {
        TsUndefinedKeyword(self.add_node(span, NodeKind::TsUndefinedKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_string_keyword(&mut self, span: Span) -> TsStringKeyword {
        TsStringKeyword(self.add_node(span, NodeKind::TsStringKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_number_keyword(&mut self, span: Span) -> TsNumberKeyword {
        TsNumberKeyword(self.add_node(span, NodeKind::TsNumberKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_big_int_keyword(&mut self, span: Span) -> TsBigIntKeyword {
        TsBigIntKeyword(self.add_node(span, NodeKind::TsBigIntKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_boolean_keyword(&mut self, span: Span) -> TsBooleanKeyword {
        TsBooleanKeyword(self.add_node(span, NodeKind::TsBooleanKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_symbol_keyword(&mut self, span: Span) -> TsSymbolKeyword {
        TsSymbolKeyword(self.add_node(span, NodeKind::TsSymbolKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_object_keyword(&mut self, span: Span) -> TsObjectKeyword {
        TsObjectKeyword(self.add_node(span, NodeKind::TsObjectKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_intrinsic_keyword(&mut self, span: Span) -> TsIntrinsicKeyword {
        TsIntrinsicKeyword(self.add_node(span, NodeKind::TsIntrinsicKeyword, NodePayload(0)))
    }
    #[inline]
    pub fn ts_this_type(&mut self, span: Span) -> TsThisType {
        TsThisType(self.add_node(span, NodeKind::TsThisType, NodePayload(0)))
    }
    #[inline]
    pub fn ts_type_reference(
        &mut self,
        span: Span,
        type_name: TsTypeName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTypeReference {
        TsTypeReference(self.add_node(
            span,
            NodeKind::TsTypeReference,
            NodePayload(
                0u128
                    | ((type_name.node_id().index() as u32 as u128) << 0usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsQualifiedName {
        TsQualifiedName(self.add_node(
            span,
            NodeKind::TsQualifiedName,
            NodePayload(
                0u128
                    | ((left.node_id().index() as u32 as u128) << 0usize)
                    | ((right.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_query(
        &mut self,
        span: Span,
        expr_name: TsTypeQueryExprName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTypeQuery {
        TsTypeQuery(self.add_node(
            span,
            NodeKind::TsTypeQuery,
            NodePayload(
                0u128
                    | ((expr_name.node_id().index() as u32 as u128) << 0usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral,
        options: Option<Expr>,
        qualifier: Option<TsImportTypeQualifier>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsImportType {
        TsImportType(self.add_node(
            span,
            NodeKind::TsImportType,
            NodePayload(
                0u128
                    | ((source.node_id().index() as u32 as u128) << 0usize)
                    | ((options.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((qualifier.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_parameter(
        &mut self,
        span: Span,
        name: BindingIdentifier,
        constraint: Option<TsType>,
        default: Option<TsType>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> TsTypeParameter {
        TsTypeParameter(self.add_node(
            span,
            NodeKind::TsTypeParameter,
            NodePayload(
                0u128
                    | ((name.node_id().index() as u32 as u128) << 0usize)
                    | ((constraint.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((default.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize)
                    | ((r#in as u32 as u128) << 96usize)
                    | ((out as u32 as u128) << 104usize)
                    | ((r#const as u32 as u128) << 112usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_parameter_declaration(
        &mut self,
        span: Span,
        params: TypedSubRange<TsTypeParameter>,
    ) -> TsTypeParameterDeclaration {
        TsTypeParameterDeclaration(self.add_node(
            span,
            NodeKind::TsTypeParameterDeclaration,
            NodePayload(0u128 | ((unsafe { params.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_type_parameter_instantiation(
        &mut self,
        span: Span,
        params: TypedSubRange<TsType>,
    ) -> TsTypeParameterInstantiation {
        TsTypeParameterInstantiation(self.add_node(
            span,
            NodeKind::TsTypeParameterInstantiation,
            NodePayload(0u128 | ((unsafe { params.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_literal_type(&mut self, span: Span, literal: TsLiteral) -> TsLiteralType {
        TsLiteralType(self.add_node(
            span,
            NodeKind::TsLiteralType,
            NodePayload(0u128 | ((literal.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        types: TypedSubRange<TsType>,
    ) -> TsTemplateLiteralType {
        TsTemplateLiteralType(self.add_node(
            span,
            NodeKind::TsTemplateLiteralType,
            NodePayload(
                0u128
                    | ((unsafe { quasis.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { types.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_array_type(&mut self, span: Span, element_type: TsType) -> TsArrayType {
        TsArrayType(self.add_node(
            span,
            NodeKind::TsArrayType,
            NodePayload(0u128 | ((element_type.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_indexed_access_type(
        &mut self,
        span: Span,
        object_type: TsType,
        index_type: TsType,
    ) -> TsIndexedAccessType {
        TsIndexedAccessType(self.add_node(
            span,
            NodeKind::TsIndexedAccessType,
            NodePayload(
                0u128
                    | ((object_type.node_id().index() as u32 as u128) << 0usize)
                    | ((index_type.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_tuple_type(
        &mut self,
        span: Span,
        element_types: TypedSubRange<TsTupleElement>,
    ) -> TsTupleType {
        TsTupleType(self.add_node(
            span,
            NodeKind::TsTupleType,
            NodePayload(
                0u128 | ((unsafe { element_types.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_named_tuple_member(
        &mut self,
        span: Span,
        label: IdentifierName,
        element_type: TsType,
        optional: bool,
    ) -> TsNamedTupleMember {
        TsNamedTupleMember(self.add_node(
            span,
            NodeKind::TsNamedTupleMember,
            NodePayload(
                0u128
                    | ((label.node_id().index() as u32 as u128) << 0usize)
                    | ((element_type.node_id().index() as u32 as u128) << 32usize)
                    | ((optional as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_optional_type(&mut self, span: Span, type_annotation: TsType) -> TsOptionalType {
        TsOptionalType(self.add_node(
            span,
            NodeKind::TsOptionalType,
            NodePayload(0u128 | ((type_annotation.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_rest_type(&mut self, span: Span, type_annotation: TsType) -> TsRestType {
        TsRestType(self.add_node(
            span,
            NodeKind::TsRestType,
            NodePayload(0u128 | ((type_annotation.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_js_doc_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsJsDocNullableType {
        TsJsDocNullableType(self.add_node(
            span,
            NodeKind::TsJsDocNullableType,
            NodePayload(
                0u128
                    | ((type_annotation.node_id().index() as u32 as u128) << 0usize)
                    | ((postfix as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_js_doc_non_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsJsDocNonNullableType {
        TsJsDocNonNullableType(self.add_node(
            span,
            NodeKind::TsJsDocNonNullableType,
            NodePayload(
                0u128
                    | ((type_annotation.node_id().index() as u32 as u128) << 0usize)
                    | ((postfix as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_js_doc_unknown_type(&mut self, span: Span) -> TsJsDocUnknownType {
        TsJsDocUnknownType(self.add_node(span, NodeKind::TsJsDocUnknownType, NodePayload(0)))
    }
    #[inline]
    pub fn ts_union_type(&mut self, span: Span, types: TypedSubRange<TsType>) -> TsUnionType {
        TsUnionType(self.add_node(
            span,
            NodeKind::TsUnionType,
            NodePayload(0u128 | ((unsafe { types.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_intersection_type(
        &mut self,
        span: Span,
        types: TypedSubRange<TsType>,
    ) -> TsIntersectionType {
        TsIntersectionType(self.add_node(
            span,
            NodeKind::TsIntersectionType,
            NodePayload(0u128 | ((unsafe { types.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_conditional_type(
        &mut self,
        span: Span,
        check_type: TsType,
        extends_type: TsType,
        true_type: TsType,
        false_type: TsType,
    ) -> TsConditionalType {
        TsConditionalType(self.add_node(
            span,
            NodeKind::TsConditionalType,
            NodePayload(
                0u128
                    | ((check_type.node_id().index() as u32 as u128) << 0usize)
                    | ((extends_type.node_id().index() as u32 as u128) << 32usize)
                    | ((true_type.node_id().index() as u32 as u128) << 64usize)
                    | ((false_type.node_id().index() as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_infer_type(&mut self, span: Span, type_parameter: TsTypeParameter) -> TsInferType {
        TsInferType(self.add_node(
            span,
            NodeKind::TsInferType,
            NodePayload(0u128 | ((type_parameter.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_type_operator(
        &mut self,
        span: Span,
        operator: TsTypeOperatorKind,
        type_annotation: TsType,
    ) -> TsTypeOperator {
        TsTypeOperator(self.add_node(
            span,
            NodeKind::TsTypeOperator,
            NodePayload(
                0u128
                    | ((operator as u32 as u128) << 32usize)
                    | ((type_annotation.node_id().index() as u32 as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_parenthesized_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
    ) -> TsParenthesizedType {
        TsParenthesizedType(self.add_node(
            span,
            NodeKind::TsParenthesizedType,
            NodePayload(0u128 | ((type_annotation.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
    ) -> TsFunctionType {
        TsFunctionType(self.add_node(
            span,
            NodeKind::TsFunctionType,
            NodePayload(
                0u128
                    | ((type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 0usize)
                    | ((params.node_id().index() as u32 as u128) << 32usize)
                    | ((return_type.node_id().index() as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
        r#abstract: bool,
    ) -> TsConstructorType {
        TsConstructorType(self.add_node(
            span,
            NodeKind::TsConstructorType,
            NodePayload(
                0u128
                    | ((type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 0usize)
                    | ((params.node_id().index() as u32 as u128) << 32usize)
                    | ((return_type.node_id().index() as u32 as u128) << 64usize)
                    | ((r#abstract as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_predicate(
        &mut self,
        span: Span,
        parameter_name: TsTypePredicateName,
        type_annotation: Option<TsTypeAnnotation>,
        asserts: bool,
    ) -> TsTypePredicate {
        TsTypePredicate(self.add_node(
            span,
            NodeKind::TsTypePredicate,
            NodePayload(
                0u128
                    | ((parameter_name.node_id().index() as u32 as u128) << 0usize)
                    | ((type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize)
                    | ((asserts as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_literal(
        &mut self,
        span: Span,
        members: TypedSubRange<TsTypeElement>,
    ) -> TsTypeLiteral {
        TsTypeLiteral(self.add_node(
            span,
            NodeKind::TsTypeLiteral,
            NodePayload(0u128 | ((unsafe { members.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_mapped_type(
        &mut self,
        span: Span,
        key: BindingIdentifier,
        constraint: TsType,
        name_type: Option<TsType>,
        type_annotation: Option<TsType>,
        optional: TsMappedTypeModifier,
        readonly: TsMappedTypeModifier,
    ) -> TsMappedType {
        let _extra_data = [
            name_type.to_extra_data(self),
            type_annotation.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        TsMappedType(self.add_node(
            span,
            NodeKind::TsMappedType,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 16usize)
                    | ((constraint.node_id().index() as u32 as u128) << 48usize)
                    | ((optional as u32 as u128) << 0usize)
                    | ((readonly as u32 as u128) << 8usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_property_signature(
        &mut self,
        span: Span,
        key: PropertyKey,
        type_annotation: Option<TsTypeAnnotation>,
        computed: bool,
        optional: bool,
        readonly: bool,
    ) -> TsPropertySignature {
        TsPropertySignature(self.add_node(
            span,
            NodeKind::TsPropertySignature,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 0usize)
                    | ((type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize)
                    | ((computed as u32 as u128) << 64usize)
                    | ((optional as u32 as u128) << 72usize)
                    | ((readonly as u32 as u128) << 80usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_method_signature(
        &mut self,
        span: Span,
        key: PropertyKey,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        kind: TsMethodSignatureKind,
        computed: bool,
        optional: bool,
    ) -> TsMethodSignature {
        let _extra_data = [params.to_extra_data(self), return_type.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        TsMethodSignature(self.add_node(
            span,
            NodeKind::TsMethodSignature,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 24usize)
                    | ((type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 56usize)
                    | ((kind as u32 as u128) << 0usize)
                    | ((computed as u32 as u128) << 8usize)
                    | ((optional as u32 as u128) << 16usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_call_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsCallSignatureDeclaration {
        TsCallSignatureDeclaration(self.add_node(
            span,
            NodeKind::TsCallSignatureDeclaration,
            NodePayload(
                0u128
                    | ((type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 0usize)
                    | ((params.node_id().index() as u32 as u128) << 32usize)
                    | ((return_type.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_construct_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsConstructSignatureDeclaration {
        TsConstructSignatureDeclaration(self.add_node(
            span,
            NodeKind::TsConstructSignatureDeclaration,
            NodePayload(
                0u128
                    | ((type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 0usize)
                    | ((params.node_id().index() as u32 as u128) << 32usize)
                    | ((return_type.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_index_signature(
        &mut self,
        span: Span,
        parameters: TypedSubRange<BindingIdentifier>,
        type_annotation: TsTypeAnnotation,
        readonly: bool,
        r#static: bool,
    ) -> TsIndexSignature {
        TsIndexSignature(self.add_node(
            span,
            NodeKind::TsIndexSignature,
            NodePayload(
                0u128
                    | ((unsafe { parameters.to_extra_data(self).other } as u128) << 0usize)
                    | ((type_annotation.node_id().index() as u32 as u128) << 64usize)
                    | ((readonly as u32 as u128) << 96usize)
                    | ((r#static as u32 as u128) << 104usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_alias_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        type_annotation: TsType,
        declare: bool,
    ) -> TsTypeAliasDeclaration {
        TsTypeAliasDeclaration(self.add_node(
            span,
            NodeKind::TsTypeAliasDeclaration,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 0usize)
                    | ((type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize)
                    | ((type_annotation.node_id().index() as u32 as u128) << 64usize)
                    | ((declare as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        extends: TypedSubRange<TsInterfaceHeritage>,
        body: TsInterfaceBody,
        declare: bool,
    ) -> TsInterfaceDeclaration {
        let _extra_data = [extends.to_extra_data(self), body.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        TsInterfaceDeclaration(self.add_node(
            span,
            NodeKind::TsInterfaceDeclaration,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 8usize)
                    | ((type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 40usize)
                    | ((declare as u32 as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_interface_body(
        &mut self,
        span: Span,
        body: TypedSubRange<TsTypeElement>,
    ) -> TsInterfaceBody {
        TsInterfaceBody(self.add_node(
            span,
            NodeKind::TsInterfaceBody,
            NodePayload(0u128 | ((unsafe { body.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_interface_heritage(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsInterfaceHeritage {
        TsInterfaceHeritage(self.add_node(
            span,
            NodeKind::TsInterfaceHeritage,
            NodePayload(
                0u128
                    | ((expression.node_id().index() as u32 as u128) << 0usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_class_implements(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsClassImplements {
        TsClassImplements(self.add_node(
            span,
            NodeKind::TsClassImplements,
            NodePayload(
                0u128
                    | ((expression.node_id().index() as u32 as u128) << 0usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        body: TsEnumBody,
        is_const: bool,
        declare: bool,
    ) -> TsEnumDeclaration {
        TsEnumDeclaration(self.add_node(
            span,
            NodeKind::TsEnumDeclaration,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 32usize)
                    | ((is_const as u32 as u128) << 64usize)
                    | ((declare as u32 as u128) << 72usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_enum_body(&mut self, span: Span, members: TypedSubRange<TsEnumMember>) -> TsEnumBody {
        TsEnumBody(self.add_node(
            span,
            NodeKind::TsEnumBody,
            NodePayload(0u128 | ((unsafe { members.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_enum_member(
        &mut self,
        span: Span,
        id: TsEnumMemberName,
        initializer: Option<Expr>,
        computed: bool,
    ) -> TsEnumMember {
        TsEnumMember(self.add_node(
            span,
            NodeKind::TsEnumMember,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 0usize)
                    | ((initializer.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 32usize)
                    | ((computed as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName,
        body: Option<TsModuleBody>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> TsModuleDeclaration {
        TsModuleDeclaration(self.add_node(
            span,
            NodeKind::TsModuleDeclaration,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 0usize)
                    | ((body.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((kind as u32 as u128) << 64usize)
                    | ((declare as u32 as u128) << 72usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_module_block(
        &mut self,
        span: Span,
        directives: TypedSubRange<Directive>,
        body: TypedSubRange<Stmt>,
    ) -> TsModuleBlock {
        TsModuleBlock(self.add_node(
            span,
            NodeKind::TsModuleBlock,
            NodePayload(
                0u128
                    | ((unsafe { directives.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_global_declaration(
        &mut self,
        span: Span,
        id: IdentifierName,
        body: TsModuleBlock,
        declare: bool,
    ) -> TsGlobalDeclaration {
        TsGlobalDeclaration(self.add_node(
            span,
            NodeKind::TsGlobalDeclaration,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 32usize)
                    | ((declare as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_parameter_property(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        parameter: BindingPattern,
        r#override: bool,
        readonly: bool,
        accessibility: Accessibility,
    ) -> TsParameterProperty {
        TsParameterProperty(self.add_node(
            span,
            NodeKind::TsParameterProperty,
            NodePayload(
                0u128
                    | ((unsafe { decorators.to_extra_data(self).other } as u128) << 0usize)
                    | ((parameter.node_id().index() as u32 as u128) << 64usize)
                    | ((r#override as u32 as u128) << 96usize)
                    | ((readonly as u32 as u128) << 104usize)
                    | ((accessibility as u32 as u128) << 112usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_this_parameter(
        &mut self,
        span: Span,
        type_annotation: Option<TsTypeAnnotation>,
    ) -> TsThisParameter {
        TsThisParameter(self.add_node(
            span,
            NodeKind::TsThisParameter,
            NodePayload(
                0u128
                    | ((type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> TsAsExpression {
        TsAsExpression(self.add_node(
            span,
            NodeKind::TsAsExpression,
            NodePayload(
                0u128
                    | ((expression.node_id().index() as u32 as u128) << 0usize)
                    | ((type_annotation.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> TsSatisfiesExpression {
        TsSatisfiesExpression(self.add_node(
            span,
            NodeKind::TsSatisfiesExpression,
            NodePayload(
                0u128
                    | ((expression.node_id().index() as u32 as u128) << 0usize)
                    | ((type_annotation.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> TsTypeAssertion {
        TsTypeAssertion(self.add_node(
            span,
            NodeKind::TsTypeAssertion,
            NodePayload(
                0u128
                    | ((type_annotation.node_id().index() as u32 as u128) << 0usize)
                    | ((expression.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_non_null_expression(&mut self, span: Span, expression: Expr) -> TsNonNullExpression {
        TsNonNullExpression(self.add_node(
            span,
            NodeKind::TsNonNullExpression,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> TsInstantiationExpression {
        TsInstantiationExpression(self.add_node(
            span,
            NodeKind::TsInstantiationExpression,
            NodePayload(
                0u128
                    | ((expression.node_id().index() as u32 as u128) << 0usize)
                    | ((type_arguments.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_export_assignment(&mut self, span: Span, expression: Expr) -> TsExportAssignment {
        TsExportAssignment(self.add_node(
            span,
            NodeKind::TsExportAssignment,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_namespace_export_declaration(
        &mut self,
        span: Span,
        id: IdentifierName,
    ) -> TsNamespaceExportDeclaration {
        TsNamespaceExportDeclaration(self.add_node(
            span,
            NodeKind::TsNamespaceExportDeclaration,
            NodePayload(0u128 | ((id.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn ts_import_equals_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        module_reference: TsModuleReference,
        import_kind: ImportOrExportKind,
    ) -> TsImportEqualsDeclaration {
        TsImportEqualsDeclaration(self.add_node(
            span,
            NodeKind::TsImportEqualsDeclaration,
            NodePayload(
                0u128
                    | ((id.node_id().index() as u32 as u128) << 0usize)
                    | ((module_reference.node_id().index() as u32 as u128) << 32usize)
                    | ((import_kind as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ts_external_module_reference(
        &mut self,
        span: Span,
        expression: StringLiteral,
    ) -> TsExternalModuleReference {
        TsExternalModuleReference(self.add_node(
            span,
            NodeKind::TsExternalModuleReference,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn jsx_element_name_jsx_identifier(&mut self, span: Span, name: AstUtf8) -> JsxElementName {
        JsxElementName::JsxIdentifier(self.jsx_identifier(span, name).into())
    }
    #[inline]
    pub fn jsx_element_name_jsx_namespaced_name(
        &mut self,
        span: Span,
        namespace: JsxIdentifier,
        name: JsxIdentifier,
    ) -> JsxElementName {
        JsxElementName::JsxNamespacedName(self.jsx_namespaced_name(span, namespace, name).into())
    }
    #[inline]
    pub fn jsx_element_name_jsx_member_expression(
        &mut self,
        span: Span,
        object: JsxMemberExpressionObject,
        property: JsxIdentifier,
    ) -> JsxElementName {
        JsxElementName::JsxMemberExpression(
            self.jsx_member_expression(span, object, property).into(),
        )
    }
    #[inline]
    pub fn jsx_member_expression_object_jsx_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> JsxMemberExpressionObject {
        JsxMemberExpressionObject::JsxIdentifier(self.jsx_identifier(span, name).into())
    }
    #[inline]
    pub fn jsx_member_expression_object_jsx_member_expression(
        &mut self,
        span: Span,
        object: JsxMemberExpressionObject,
        property: JsxIdentifier,
    ) -> JsxMemberExpressionObject {
        JsxMemberExpressionObject::JsxMemberExpression(
            self.jsx_member_expression(span, object, property).into(),
        )
    }
    #[inline]
    pub fn jsx_attribute_name_jsx_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> JsxAttributeName {
        JsxAttributeName::JsxIdentifier(self.jsx_identifier(span, name).into())
    }
    #[inline]
    pub fn jsx_attribute_name_jsx_namespaced_name(
        &mut self,
        span: Span,
        namespace: JsxIdentifier,
        name: JsxIdentifier,
    ) -> JsxAttributeName {
        JsxAttributeName::JsxNamespacedName(self.jsx_namespaced_name(span, namespace, name).into())
    }
    #[inline]
    pub fn jsx_attribute_item_jsx_attribute(
        &mut self,
        span: Span,
        name: JsxAttributeName,
        value: Option<JsxAttributeValue>,
    ) -> JsxAttributeItem {
        JsxAttributeItem::JsxAttribute(self.jsx_attribute(span, name, value).into())
    }
    #[inline]
    pub fn jsx_attribute_item_jsx_spread_attribute(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> JsxAttributeItem {
        JsxAttributeItem::JsxSpreadAttribute(self.jsx_spread_attribute(span, argument).into())
    }
    #[inline]
    pub fn jsx_attribute_value_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> JsxAttributeValue {
        JsxAttributeValue::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_attribute_value_jsx_expression_container(
        &mut self,
        span: Span,
        expression: JsxExpression,
    ) -> JsxAttributeValue {
        JsxAttributeValue::JsxExpressionContainer(
            self.jsx_expression_container(span, expression).into(),
        )
    }
    #[inline]
    pub fn jsx_attribute_value_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> JsxAttributeValue {
        JsxAttributeValue::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_attribute_value_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> JsxAttributeValue {
        JsxAttributeValue::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn jsx_expression_this_expression(&mut self, span: Span) -> JsxExpression {
        JsxExpression::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn jsx_expression_super(&mut self, span: Span) -> JsxExpression {
        JsxExpression::Super(self.super_(span).into())
    }
    #[inline]
    pub fn jsx_expression_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> JsxExpression {
        JsxExpression::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn jsx_expression_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_expression_numeric_literal(
        &mut self,
        span: Span,
        kind: NumericLiteralKind,
        raw: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::NumericLiteral(self.numeric_literal(span, kind, raw).into())
    }
    #[inline]
    pub fn jsx_expression_big_int_literal(&mut self, span: Span, raw: AstUtf8) -> JsxExpression {
        JsxExpression::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn jsx_expression_boolean_literal(&mut self, span: Span, value: bool) -> JsxExpression {
        JsxExpression::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn jsx_expression_null_literal(&mut self, span: Span) -> JsxExpression {
        JsxExpression::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn jsx_expression_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn jsx_expression_template_literal(
        &mut self,
        span: Span,
        quasis: TypedSubRange<TemplateElement>,
        expressions: TypedSubRange<Expr>,
    ) -> JsxExpression {
        JsxExpression::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn jsx_expression_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> JsxExpression {
        JsxExpression::BinaryExpression(self.binary_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn jsx_expression_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> JsxExpression {
        JsxExpression::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        )
    }
    #[inline]
    pub fn jsx_expression_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> JsxExpression {
        JsxExpression::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn jsx_expression_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> JsxExpression {
        JsxExpression::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> JsxExpression {
        JsxExpression::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> JsxExpression {
        JsxExpression::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_sequence_expression(
        &mut self,
        span: Span,
        expressions: TypedSubRange<Expr>,
    ) -> JsxExpression {
        JsxExpression::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn jsx_expression_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> JsxExpression {
        JsxExpression::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        optional: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> JsxExpression {
        JsxExpression::CallExpression(
            self.call_expression(span, callee, arguments, optional, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn jsx_expression_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        arguments: TypedSubRange<Argument>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> JsxExpression {
        JsxExpression::NewExpression(
            self.new_expression(span, callee, arguments, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        quasi: TemplateLiteral,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> JsxExpression {
        JsxExpression::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, quasi, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: FormalParameters,
        body: ArrowFunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> JsxExpression {
        JsxExpression::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_array_expression(
        &mut self,
        span: Span,
        elements: TypedSubRange<Option<Argument>>,
    ) -> JsxExpression {
        JsxExpression::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn jsx_expression_object_expression(
        &mut self,
        span: Span,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) -> JsxExpression {
        JsxExpression::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn jsx_expression_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn jsx_expression_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        generator: bool,
        r#async: bool,
        params: FormalParameters,
        body: FunctionBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        return_type: Option<TsTypeAnnotation>,
    ) -> JsxExpression {
        JsxExpression::Function(
            self.function(
                span,
                function_type,
                id,
                generator,
                r#async,
                params,
                body,
                type_parameters,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: TypedSubRange<Decorator>,
        id: Option<BindingIdentifier>,
        super_class: Option<Expr>,
        body: ClassBody,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: TypedSubRange<TsClassImplements>,
        r#abstract: bool,
        declare: bool,
    ) -> JsxExpression {
        JsxExpression::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                super_class,
                body,
                type_parameters,
                super_type_arguments,
                implements,
                r#abstract,
                declare,
            )
            .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
    ) -> JsxExpression {
        JsxExpression::ImportExpression(self.import_expression(span, source, options, phase).into())
    }
    #[inline]
    pub fn jsx_expression_await_expression(&mut self, span: Span, argument: Expr) -> JsxExpression {
        JsxExpression::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn jsx_expression_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> JsxExpression {
        JsxExpression::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn jsx_expression_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn jsx_expression_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> JsxExpression {
        JsxExpression::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> JsxExpression {
        JsxExpression::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn jsx_expression_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> JsxExpression {
        JsxExpression::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> JsxExpression {
        JsxExpression::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> JsxExpression {
        JsxExpression::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_jsx_empty_expression(&mut self, span: Span) -> JsxExpression {
        JsxExpression::JsxEmptyExpression(self.jsx_empty_expression(span).into())
    }
    #[inline]
    pub fn jsx_child_jsx_text(&mut self, span: Span, value: AstUtf8) -> JsxChild {
        JsxChild::JsxText(self.jsx_text(span, value).into())
    }
    #[inline]
    pub fn jsx_child_jsx_expression_container(
        &mut self,
        span: Span,
        expression: JsxExpression,
    ) -> JsxChild {
        JsxChild::JsxExpressionContainer(self.jsx_expression_container(span, expression).into())
    }
    #[inline]
    pub fn jsx_child_jsx_spread_child(&mut self, span: Span, expression: Expr) -> JsxChild {
        JsxChild::JsxSpreadChild(self.jsx_spread_child(span, expression).into())
    }
    #[inline]
    pub fn jsx_child_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> JsxChild {
        JsxChild::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_child_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> JsxChild {
        JsxChild::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: TypedSubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> JsxElement {
        JsxElement(self.add_node(
            span,
            NodeKind::JsxElement,
            NodePayload(
                0u128
                    | ((opening_element.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { children.to_extra_data(self).other } as u128) << 0usize)
                    | ((closing_element.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_opening_element(
        &mut self,
        span: Span,
        name: JsxElementName,
        attributes: TypedSubRange<JsxAttributeItem>,
        self_closing: bool,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> JsxOpeningElement {
        let _f0 = self.add_extra(attributes.to_extra_data(self));
        JsxOpeningElement(self.add_node(
            span,
            NodeKind::JsxOpeningElement,
            NodePayload(
                0u128
                    | ((name.node_id().index() as u32 as u128) << 8usize)
                    | ((self_closing as u32 as u128) << 0usize)
                    | ((type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32)
                        as u128)
                        << 40usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_closing_element(&mut self, span: Span, name: JsxElementName) -> JsxClosingElement {
        JsxClosingElement(self.add_node(
            span,
            NodeKind::JsxClosingElement,
            NodePayload(0u128 | ((name.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: TypedSubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> JsxFragment {
        JsxFragment(self.add_node(
            span,
            NodeKind::JsxFragment,
            NodePayload(
                0u128
                    | ((opening_fragment.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { children.to_extra_data(self).other } as u128) << 0usize)
                    | ((closing_fragment.node_id().index() as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_opening_fragment(&mut self, span: Span) -> JsxOpeningFragment {
        JsxOpeningFragment(self.add_node(span, NodeKind::JsxOpeningFragment, NodePayload(0)))
    }
    #[inline]
    pub fn jsx_closing_fragment(&mut self, span: Span) -> JsxClosingFragment {
        JsxClosingFragment(self.add_node(span, NodeKind::JsxClosingFragment, NodePayload(0)))
    }
    #[inline]
    pub fn jsx_identifier(&mut self, span: Span, name: AstUtf8) -> JsxIdentifier {
        JsxIdentifier(self.add_node(
            span,
            NodeKind::JsxIdentifier,
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn jsx_namespaced_name(
        &mut self,
        span: Span,
        namespace: JsxIdentifier,
        name: JsxIdentifier,
    ) -> JsxNamespacedName {
        JsxNamespacedName(self.add_node(
            span,
            NodeKind::JsxNamespacedName,
            NodePayload(
                0u128
                    | ((namespace.node_id().index() as u32 as u128) << 0usize)
                    | ((name.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_member_expression(
        &mut self,
        span: Span,
        object: JsxMemberExpressionObject,
        property: JsxIdentifier,
    ) -> JsxMemberExpression {
        JsxMemberExpression(self.add_node(
            span,
            NodeKind::JsxMemberExpression,
            NodePayload(
                0u128
                    | ((object.node_id().index() as u32 as u128) << 0usize)
                    | ((property.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_attribute(
        &mut self,
        span: Span,
        name: JsxAttributeName,
        value: Option<JsxAttributeValue>,
    ) -> JsxAttribute {
        JsxAttribute(self.add_node(
            span,
            NodeKind::JsxAttribute,
            NodePayload(
                0u128
                    | ((name.node_id().index() as u32 as u128) << 0usize)
                    | ((value.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_spread_attribute(&mut self, span: Span, argument: Expr) -> JsxSpreadAttribute {
        JsxSpreadAttribute(self.add_node(
            span,
            NodeKind::JsxSpreadAttribute,
            NodePayload(0u128 | ((argument.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn jsx_expression_container(
        &mut self,
        span: Span,
        expression: JsxExpression,
    ) -> JsxExpressionContainer {
        JsxExpressionContainer(self.add_node(
            span,
            NodeKind::JsxExpressionContainer,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn jsx_empty_expression(&mut self, span: Span) -> JsxEmptyExpression {
        JsxEmptyExpression(self.add_node(span, NodeKind::JsxEmptyExpression, NodePayload(0)))
    }
    #[inline]
    pub fn jsx_text(&mut self, span: Span, value: AstUtf8) -> JsxText {
        JsxText(self.add_node(
            span,
            NodeKind::JsxText,
            NodePayload(0u128 | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn jsx_spread_child(&mut self, span: Span, expression: Expr) -> JsxSpreadChild {
        JsxSpreadChild(self.add_node(
            span,
            NodeKind::JsxSpreadChild,
            NodePayload(0u128 | ((expression.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
}
