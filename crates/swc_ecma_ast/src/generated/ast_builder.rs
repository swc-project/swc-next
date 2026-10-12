#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::too_many_arguments
)]
use crate::generated::ast_payload::*;
use crate::*;
use oxc_index::Idx;
impl<'a> Ast<'a> {
    #[inline]
    pub fn program(
        &mut self,
        span: Span,
        directives: SubRange<Directive>,
        body: SubRange<Stmt>,
        hashbang: Option<Hashbang>,
    ) -> Program {
        let _f0 = self.add_extra(body);
        Program(self.add_node(
            span,
            NodeKind::Program,
            NodePayload {
                program: ProgramPayload {
                    directives,
                    hashbang: hashbang.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn stmt_block_statement(&mut self, span: Span, body: SubRange<Stmt>) -> Stmt {
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
        cases: SubRange<SwitchCase>,
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
    pub fn stmt_decl_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<VariableDeclarator>,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::TsFunction(
            self.ts_function(
                span,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                generator,
                r#async,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_type_alias_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        type_annotation: TsType,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::TsTypeAliasDeclaration(
            self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        extends: SubRange<TsInterfaceHeritage>,
        body: TsInterfaceBody,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::TsInterfaceDeclaration(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        body: TsEnumBody,
        is_const: bool,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::TsEnumDeclaration(
            self.ts_enum_declaration(span, id, body, is_const, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName,
        body: Option<TsModuleBody>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::TsModuleDeclaration(
            self.ts_module_declaration(span, id, body, kind, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_global_declaration(
        &mut self,
        span: Span,
        id: IdentifierName,
        body: TsModuleBlock,
        declare: bool,
    ) -> Stmt {
        Stmt::Declaration(Decl::TsGlobalDeclaration(
            self.ts_global_declaration(span, id, body, declare).into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_import_equals_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        module_reference: TsModuleReference,
        import_kind: ImportOrExportKind,
    ) -> Stmt {
        Stmt::Declaration(Decl::TsImportEqualsDeclaration(
            self.ts_import_equals_declaration(span, id, module_reference, import_kind)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_import_declaration(
        &mut self,
        span: Span,
        specifiers: SubRange<ImportDeclarationSpecifier>,
        source: StringLiteral,
        attributes: SubRange<ImportAttribute>,
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
        specifiers: SubRange<ExportSpecifier>,
        source: Option<StringLiteral>,
        attributes: SubRange<ImportAttribute>,
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
        attributes: SubRange<ImportAttribute>,
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
        declarators: SubRange<VariableDeclarator>,
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
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> Decl {
        Decl::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        )
    }
    #[inline]
    pub fn decl_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> Decl {
        Decl::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
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
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> Decl {
        Decl::TsFunction(
            self.ts_function(
                span,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                generator,
                r#async,
                declare,
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
        extends: SubRange<TsInterfaceHeritage>,
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
    pub fn expr_numeric_literal(&mut self, span: Span, value: f64, raw: Option<AstUtf8>) -> Expr {
        Expr::NumericLiteral(self.numeric_literal(span, value, raw).into())
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
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
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
    pub fn expr_sequence_expression(&mut self, span: Span, expressions: SubRange<Expr>) -> Expr {
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
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> Expr {
        Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional, pure)
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
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> Expr {
        Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        )
    }
    #[inline]
    pub fn expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> Expr {
        Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        )
    }
    #[inline]
    pub fn expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> Expr {
        Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        )
    }
    #[inline]
    pub fn expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> Expr {
        Expr::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
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
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> Expr {
        Expr::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        )
    }
    #[inline]
    pub fn expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> Expr {
        Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
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
        sync: bool,
    ) -> Expr {
        Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        )
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
        children: SubRange<JsxChild>,
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
        children: SubRange<JsxChild>,
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
    ) -> BindingPattern {
        BindingPattern::BindingIdentifier(self.binding_identifier(span, name).into())
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> BindingPattern {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> BindingPattern {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> BindingPattern {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> BindingPattern {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> BindingPattern {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> BindingPattern {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern,
        right: Expr,
    ) -> BindingPattern {
        BindingPattern::AssignmentPattern(self.assignment_pattern(span, left, right).into())
    }
    #[inline]
    pub fn binding_pattern_binding_rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern,
    ) -> BindingPattern {
        BindingPattern::BindingRestElement(self.binding_rest_element(span, argument).into())
    }
    #[inline]
    pub fn binding_pattern_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
    ) -> BindingPattern {
        BindingPattern::ArrayPattern(self.array_pattern(span, elements, rest).into())
    }
    #[inline]
    pub fn binding_pattern_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
    ) -> BindingPattern {
        BindingPattern::ObjectPattern(self.object_pattern(span, properties, rest).into())
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> AssignmentTarget {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> AssignmentTarget {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> AssignmentTarget {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> AssignmentTarget {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> AssignmentTarget {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> AssignmentTarget {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn assignment_target_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
    ) -> AssignmentTarget {
        AssignmentTarget::ArrayAssignmentTarget(self.array_pattern(span, elements, rest).into())
    }
    #[inline]
    pub fn assignment_target_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
    ) -> AssignmentTarget {
        AssignmentTarget::ObjectAssignmentTarget(self.object_pattern(span, properties, rest).into())
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
    pub fn property_key_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_this_expression(&mut self, span: Span) -> PropertyKey {
        PropertyKey::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn property_key_expr_super(&mut self, span: Span) -> PropertyKey {
        PropertyKey::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn property_key_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8>,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_big_int_literal(&mut self, span: Span, raw: AstUtf8) -> PropertyKey {
        PropertyKey::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn property_key_expr_boolean_literal(&mut self, span: Span, value: bool) -> PropertyKey {
        PropertyKey::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_null_literal(&mut self, span: Span) -> PropertyKey {
        PropertyKey::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn property_key_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<Expr>,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional, pure)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: SubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> PropertyKey {
        PropertyKey::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
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
    pub fn argument_expr_identifier_reference(&mut self, span: Span, name: AstUtf8) -> Argument {
        Argument::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_this_expression(&mut self, span: Span) -> Argument {
        Argument::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn argument_expr_super(&mut self, span: Span) -> Argument {
        Argument::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn argument_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> Argument {
        Argument::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> Argument {
        Argument::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8>,
    ) -> Argument {
        Argument::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_big_int_literal(&mut self, span: Span, raw: AstUtf8) -> Argument {
        Argument::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn argument_expr_boolean_literal(&mut self, span: Span, value: bool) -> Argument {
        Argument::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_null_literal(&mut self, span: Span) -> Argument {
        Argument::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn argument_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> Argument {
        Argument::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> Argument {
        Argument::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> Argument {
        Argument::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> Argument {
        Argument::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> Argument {
        Argument::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> Argument {
        Argument::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> Argument {
        Argument::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> Argument {
        Argument::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<Expr>,
    ) -> Argument {
        Argument::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> Argument {
        Argument::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> Argument {
        Argument::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional, pure)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_chain_expression(&mut self, span: Span, expression: Expr) -> Argument {
        Argument::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> Argument {
        Argument::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> Argument {
        Argument::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> Argument {
        Argument::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> Argument {
        Argument::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
    ) -> Argument {
        Argument::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> Argument {
        Argument::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> Argument {
        Argument::Expr(Expr::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> Argument {
        Argument::Expr(Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> Argument {
        Argument::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_await_expression(&mut self, span: Span, argument: Expr) -> Argument {
        Argument::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> Argument {
        Argument::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_private_identifier(&mut self, span: Span, name: AstUtf8) -> Argument {
        Argument::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> Argument {
        Argument::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> Argument {
        Argument::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> Argument {
        Argument::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> Argument {
        Argument::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> Argument {
        Argument::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: SubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> Argument {
        Argument::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> Argument {
        Argument::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
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
        declarators: SubRange<VariableDeclarator>,
        declare: bool,
    ) -> ForStatementInit {
        ForStatementInit::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_this_expression(&mut self, span: Span) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_super(&mut self, span: Span) -> ForStatementInit {
        ForStatementInit::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8>,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_null_literal(&mut self, span: Span) -> ForStatementInit {
        ForStatementInit::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<Expr>,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional, pure)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: SubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> ForStatementInit {
        ForStatementInit::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_left_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<VariableDeclarator>,
        declare: bool,
    ) -> ForStatementLeft {
        ForStatementLeft::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(
                self.identifier_reference(span, name).into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberExpression(
                self.member_expression(span, object, property, computed, optional)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsAsExpression(
                self.ts_as_expression(span, expression, type_annotation)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsSatisfiesExpression(
                self.ts_satisfies_expression(span, expression, type_annotation)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsTypeAssertion(
                self.ts_type_assertion(span, type_annotation, expression)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsNonNullExpression(
                self.ts_non_null_expression(span, expression).into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::ArrayAssignmentTarget(
            self.array_pattern(span, elements, rest).into(),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
    ) -> ForStatementLeft {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::ObjectAssignmentTarget(
            self.object_pattern(span, properties, rest).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_function_body(
        &mut self,
        span: Span,
        directives: SubRange<Directive>,
        body: SubRange<Stmt>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::FunctionBody(self.function_body(span, directives, body).into())
    }
    #[inline]
    pub fn arrow_function_body_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_this_expression(&mut self, span: Span) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_super(&mut self, span: Span) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_null_literal(&mut self, span: Span) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<Expr>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional, pure)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: SubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> ArrowFunctionBody {
        ArrowFunctionBody::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn formal_parameter_item_formal_parameter(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
        pattern: FormalParameterPattern,
        type_annotation: Option<TsTypeAnnotation>,
        initializer: Option<Expr>,
        optional: bool,
    ) -> FormalParameterItem {
        FormalParameterItem::FormalParameter(
            self.formal_parameter(
                span,
                decorators,
                pattern,
                type_annotation,
                initializer,
                optional,
            )
            .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_item_ts_parameter_property(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
        parameter: FormalParameter,
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
    ) -> FormalParameterPattern {
        FormalParameterPattern::BindingIdentifier(self.binding_identifier(span, name).into())
    }
    #[inline]
    pub fn formal_parameter_pattern_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
    ) -> FormalParameterPattern {
        FormalParameterPattern::ArrayPattern(self.array_pattern(span, elements, rest).into())
    }
    #[inline]
    pub fn formal_parameter_pattern_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
    ) -> FormalParameterPattern {
        FormalParameterPattern::ObjectPattern(self.object_pattern(span, properties, rest).into())
    }
    #[inline]
    pub fn class_element_method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
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
        decorators: SubRange<Decorator>,
        key: PropertyKey,
        type_annotation: Option<TsTypeAnnotation>,
        value: Option<Expr>,
        computed: bool,
        r#static: bool,
        accessor: bool,
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
                type_annotation,
                value,
                computed,
                r#static,
                accessor,
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
    pub fn class_element_static_block(&mut self, span: Span, body: SubRange<Stmt>) -> ClassElement {
        ClassElement::StaticBlock(self.static_block(span, body).into())
    }
    #[inline]
    pub fn class_element_ts_method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
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
        parameters: SubRange<TsIndexSignatureParameter>,
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
    pub fn export_default_declaration_kind_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_this_expression(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_super(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::BigIntLiteral(
            self.big_int_literal(span, raw).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_null_literal(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<Expr>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional, pure)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: SubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
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
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind::TsFunction(
            self.ts_function(
                span,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                generator,
                r#async,
                declare,
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
        extends: SubRange<TsInterfaceHeritage>,
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
        Super(self.add_node(span, NodeKind::Super, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn this_expression(&mut self, span: Span) -> ThisExpression {
        ThisExpression(self.add_node(span, NodeKind::ThisExpression, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn string_literal(&mut self, span: Span, value: AstWtf8, raw: AstUtf8) -> StringLiteral {
        StringLiteral(self.add_node(
            span,
            NodeKind::StringLiteral,
            NodePayload {
                string_literal: StringLiteralPayload { value, raw },
            },
        ))
    }
    #[inline]
    pub fn numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8>,
    ) -> NumericLiteral {
        NumericLiteral(self.add_node(
            span,
            NodeKind::NumericLiteral,
            NodePayload {
                numeric_literal: NumericLiteralPayload {
                    value,
                    raw: raw.map_or(u64::from(u32::MAX), AstUtf8::raw),
                },
            },
        ))
    }
    #[inline]
    pub fn big_int_literal(&mut self, span: Span, raw: AstUtf8) -> BigIntLiteral {
        BigIntLiteral(self.add_node(
            span,
            NodeKind::BigIntLiteral,
            NodePayload {
                big_int_literal: BigIntLiteralPayload {
                    raw,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn boolean_literal(&mut self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral(self.add_node(
            span,
            NodeKind::BooleanLiteral,
            NodePayload {
                boolean_literal: BooleanLiteralPayload {
                    value,
                    _pad0: [0; 15usize],
                },
            },
        ))
    }
    #[inline]
    pub fn null_literal(&mut self, span: Span) -> NullLiteral {
        NullLiteral(self.add_node(span, NodeKind::NullLiteral, NodePayload { raw: 0 }))
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
            NodePayload {
                reg_exp_literal: RegExpLiteralPayload { pattern, flags },
            },
        ))
    }
    #[inline]
    pub fn template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> TemplateLiteral {
        TemplateLiteral(self.add_node(
            span,
            NodeKind::TemplateLiteral,
            NodePayload {
                template_literal: TemplateLiteralPayload {
                    quasis,
                    expressions,
                },
            },
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
        let _f0 = self.add_extra(raw);
        TemplateElement(self.add_node(
            span,
            NodeKind::TemplateElement,
            NodePayload {
                template_element: TemplateElementPayload {
                    cooked,
                    tail,
                    is_cooked_undefined,
                    _pad0: [0; 2usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn identifier_reference(&mut self, span: Span, name: AstUtf8) -> IdentifierReference {
        IdentifierReference(self.add_node(
            span,
            NodeKind::IdentifierReference,
            NodePayload {
                identifier_reference: IdentifierReferencePayload {
                    name,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn private_identifier(&mut self, span: Span, name: AstUtf8) -> PrivateIdentifier {
        PrivateIdentifier(self.add_node(
            span,
            NodeKind::PrivateIdentifier,
            NodePayload {
                private_identifier: PrivateIdentifierPayload {
                    name,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn binding_identifier(&mut self, span: Span, name: AstUtf8) -> BindingIdentifier {
        BindingIdentifier(self.add_node(
            span,
            NodeKind::BindingIdentifier,
            NodePayload {
                binding_identifier: BindingIdentifierPayload {
                    name,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn identifier_name(&mut self, span: Span, name: AstUtf8) -> IdentifierName {
        IdentifierName(self.add_node(
            span,
            NodeKind::IdentifierName,
            NodePayload {
                identifier_name: IdentifierNamePayload {
                    name,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn label_identifier(&mut self, span: Span, name: AstUtf8) -> LabelIdentifier {
        LabelIdentifier(self.add_node(
            span,
            NodeKind::LabelIdentifier,
            NodePayload {
                label_identifier: LabelIdentifierPayload {
                    name,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern,
        right: Expr,
    ) -> AssignmentPattern {
        AssignmentPattern(self.add_node(
            span,
            NodeKind::AssignmentPattern,
            NodePayload {
                assignment_pattern: AssignmentPatternPayload {
                    left,
                    right,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn binding_rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern,
    ) -> BindingRestElement {
        BindingRestElement(self.add_node(
            span,
            NodeKind::BindingRestElement,
            NodePayload {
                binding_rest_element: BindingRestElementPayload {
                    argument,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<Option<BindingPattern>>,
        rest: Option<BindingRestElement>,
    ) -> ArrayPattern {
        ArrayPattern(self.add_node(
            span,
            NodeKind::ArrayPattern,
            NodePayload {
                array_pattern: ArrayPatternPayload {
                    elements,
                    rest: rest.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<BindingProperty>,
        rest: Option<BindingRestElement>,
    ) -> ObjectPattern {
        ObjectPattern(self.add_node(
            span,
            NodeKind::ObjectPattern,
            NodePayload {
                object_pattern: ObjectPatternPayload {
                    properties,
                    rest: rest.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
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
            NodePayload {
                binding_property: BindingPropertyPayload {
                    key,
                    value,
                    shorthand,
                    computed,
                    _pad0: [0; 6usize],
                },
            },
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
            NodePayload {
                binary_expression: BinaryExpressionPayload {
                    left,
                    right,
                    operator,
                    _pad0: [0; 7usize],
                },
            },
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
            NodePayload {
                logical_expression: LogicalExpressionPayload {
                    left,
                    right,
                    operator,
                    _pad0: [0; 7usize],
                },
            },
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
            NodePayload {
                conditional_expression: ConditionalExpressionPayload {
                    test,
                    consequent,
                    alternate,
                    _pad0: [0; 4usize],
                },
            },
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
            NodePayload {
                unary_expression: UnaryExpressionPayload {
                    argument,
                    operator,
                    _pad0: [0; 11usize],
                },
            },
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
            NodePayload {
                update_expression: UpdateExpressionPayload {
                    argument,
                    operator,
                    prefix,
                    _pad0: [0; 10usize],
                },
            },
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
            NodePayload {
                assignment_expression: AssignmentExpressionPayload {
                    left,
                    right,
                    operator,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<VariableDeclarator>,
        declare: bool,
    ) -> VariableDeclaration {
        VariableDeclaration(self.add_node(
            span,
            NodeKind::VariableDeclaration,
            NodePayload {
                variable_declaration: VariableDeclarationPayload {
                    declarators,
                    kind,
                    declare,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn variable_declarator(
        &mut self,
        span: Span,
        id: BindingPattern,
        type_annotation: Option<TsTypeAnnotation>,
        init: Option<Expr>,
        definite: bool,
    ) -> VariableDeclarator {
        VariableDeclarator(self.add_node(
            span,
            NodeKind::VariableDeclarator,
            NodePayload {
                variable_declarator: VariableDeclaratorPayload {
                    id,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    init: init.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    definite,
                    _pad0: [0; 3usize],
                },
            },
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
            NodePayload {
                if_statement: IfStatementPayload {
                    test,
                    consequent,
                    alternate: alternate.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn switch_statement(
        &mut self,
        span: Span,
        discriminant: Expr,
        cases: SubRange<SwitchCase>,
    ) -> SwitchStatement {
        SwitchStatement(self.add_node(
            span,
            NodeKind::SwitchStatement,
            NodePayload {
                switch_statement: SwitchStatementPayload {
                    cases,
                    discriminant,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn switch_case(
        &mut self,
        span: Span,
        test: Option<Expr>,
        consequent: SubRange<Stmt>,
    ) -> SwitchCase {
        SwitchCase(self.add_node(
            span,
            NodeKind::SwitchCase,
            NodePayload {
                switch_case: SwitchCasePayload {
                    consequent,
                    test: test.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
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
            NodePayload {
                for_statement: ForStatementPayload {
                    init: init.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    test: test.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    update: update.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    body,
                },
            },
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
            NodePayload {
                for_in_statement: ForInStatementPayload {
                    left,
                    right,
                    body,
                    _pad0: [0; 4usize],
                },
            },
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
            NodePayload {
                for_of_statement: ForOfStatementPayload {
                    left,
                    right,
                    body,
                    r#await,
                    _pad0: [0; 3usize],
                },
            },
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
            NodePayload {
                try_statement: TryStatementPayload {
                    block,
                    handler: handler.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    finalizer: finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn catch_clause(
        &mut self,
        span: Span,
        param: Option<BindingPattern>,
        type_annotation: Option<TsTypeAnnotation>,
        body: BlockStatement,
    ) -> CatchClause {
        CatchClause(self.add_node(
            span,
            NodeKind::CatchClause,
            NodePayload {
                catch_clause: CatchClausePayload {
                    param: param.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    body,
                    _pad0: [0; 4usize],
                },
            },
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
            NodePayload {
                break_statement: BreakStatementPayload {
                    label: label.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                continue_statement: ContinueStatementPayload {
                    label: label.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                labeled_statement: LabeledStatementPayload {
                    label,
                    body,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn expression_statement(&mut self, span: Span, expression: Expr) -> ExpressionStatement {
        ExpressionStatement(self.add_node(
            span,
            NodeKind::ExpressionStatement,
            NodePayload {
                expression_statement: ExpressionStatementPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn return_statement(&mut self, span: Span, argument: Option<Expr>) -> ReturnStatement {
        ReturnStatement(self.add_node(
            span,
            NodeKind::ReturnStatement,
            NodePayload {
                return_statement: ReturnStatementPayload {
                    argument: argument.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn throw_statement(&mut self, span: Span, argument: Expr) -> ThrowStatement {
        ThrowStatement(self.add_node(
            span,
            NodeKind::ThrowStatement,
            NodePayload {
                throw_statement: ThrowStatementPayload {
                    argument,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn debugger_statement(&mut self, span: Span) -> DebuggerStatement {
        DebuggerStatement(self.add_node(span, NodeKind::DebuggerStatement, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn empty_statement(&mut self, span: Span) -> EmptyStatement {
        EmptyStatement(self.add_node(span, NodeKind::EmptyStatement, NodePayload { raw: 0 }))
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
            NodePayload {
                directive: DirectivePayload {
                    value,
                    expression,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn decorator(&mut self, span: Span, expression: Expr) -> Decorator {
        Decorator(self.add_node(
            span,
            NodeKind::Decorator,
            NodePayload {
                decorator: DecoratorPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> Class {
        let _extra_data = [
            (decorators).into_extra_data(),
            (super_class).into_extra_data(),
            (super_type_arguments).into_extra_data(),
            (implements).into_extra_data(),
            (body).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        Class(self.add_node(
            span,
            NodeKind::Class,
            NodePayload {
                class: ClassPayload {
                    id: id.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    class_type,
                    r#abstract,
                    declare,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn class_body(&mut self, span: Span, body: SubRange<ClassElement>) -> ClassBody {
        ClassBody(self.add_node(
            span,
            NodeKind::ClassBody,
            NodePayload {
                class_body: ClassBodyPayload {
                    body,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
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
        let _extra_data = [(decorators).into_extra_data(), (value).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        MethodDefinition(self.add_node(
            span,
            NodeKind::MethodDefinition,
            NodePayload {
                method_definition: MethodDefinitionPayload {
                    key,
                    kind,
                    computed,
                    r#static,
                    r#override,
                    optional,
                    r#abstract,
                    accessibility,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn property_definition(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
        key: PropertyKey,
        type_annotation: Option<TsTypeAnnotation>,
        value: Option<Expr>,
        computed: bool,
        r#static: bool,
        accessor: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> PropertyDefinition {
        let _extra_data = [
            (decorators).into_extra_data(),
            (key).into_extra_data(),
            (type_annotation).into_extra_data(),
            (value).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        PropertyDefinition(self.add_node(
            span,
            NodeKind::PropertyDefinition,
            NodePayload {
                property_definition: PropertyDefinitionPayload {
                    computed,
                    r#static,
                    accessor,
                    declare,
                    r#override,
                    optional,
                    definite,
                    readonly,
                    r#abstract,
                    accessibility,
                    _pad0: [0; 2usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn static_block(&mut self, span: Span, body: SubRange<Stmt>) -> StaticBlock {
        StaticBlock(self.add_node(
            span,
            NodeKind::StaticBlock,
            NodePayload {
                static_block: StaticBlockPayload {
                    body,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> Function {
        let _extra_data = [
            (this_param).into_extra_data(),
            (params).into_extra_data(),
            (return_type).into_extra_data(),
            (body).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        Function(self.add_node(
            span,
            NodeKind::Function,
            NodePayload {
                function: FunctionPayload {
                    id: id.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    function_type,
                    generator,
                    r#async,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn function_body(
        &mut self,
        span: Span,
        directives: SubRange<Directive>,
        body: SubRange<Stmt>,
    ) -> FunctionBody {
        FunctionBody(self.add_node(
            span,
            NodeKind::FunctionBody,
            NodePayload {
                function_body: FunctionBodyPayload { directives, body },
            },
        ))
    }
    #[inline]
    pub fn block_statement(&mut self, span: Span, body: SubRange<Stmt>) -> BlockStatement {
        BlockStatement(self.add_node(
            span,
            NodeKind::BlockStatement,
            NodePayload {
                block_statement: BlockStatementPayload {
                    body,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn while_statement(&mut self, span: Span, test: Expr, body: Stmt) -> WhileStatement {
        WhileStatement(self.add_node(
            span,
            NodeKind::WhileStatement,
            NodePayload {
                while_statement: WhileStatementPayload {
                    test,
                    body,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn do_while_statement(&mut self, span: Span, body: Stmt, test: Expr) -> DoWhileStatement {
        DoWhileStatement(self.add_node(
            span,
            NodeKind::DoWhileStatement,
            NodePayload {
                do_while_statement: DoWhileStatementPayload {
                    body,
                    test,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn with_statement(&mut self, span: Span, object: Expr, body: Stmt) -> WithStatement {
        WithStatement(self.add_node(
            span,
            NodeKind::WithStatement,
            NodePayload {
                with_statement: WithStatementPayload {
                    object,
                    body,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn formal_parameters(
        &mut self,
        span: Span,
        items: SubRange<FormalParameterItem>,
        rest: Option<FormalParameterRest>,
        kind: FormalParameterKind,
    ) -> FormalParameters {
        FormalParameters(self.add_node(
            span,
            NodeKind::FormalParameters,
            NodePayload {
                formal_parameters: FormalParametersPayload {
                    items,
                    rest: rest.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    kind,
                    _pad0: [0; 3usize],
                },
            },
        ))
    }
    #[inline]
    pub fn formal_parameter(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
        pattern: FormalParameterPattern,
        type_annotation: Option<TsTypeAnnotation>,
        initializer: Option<Expr>,
        optional: bool,
    ) -> FormalParameter {
        let _extra_data = [
            (decorators).into_extra_data(),
            (initializer).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        FormalParameter(self.add_node(
            span,
            NodeKind::FormalParameter,
            NodePayload {
                formal_parameter: FormalParameterPayload {
                    pattern,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    optional,
                    _pad0: [0; 3usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn formal_parameter_rest(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
        argument: FormalParameterPattern,
        type_annotation: Option<TsTypeAnnotation>,
    ) -> FormalParameterRest {
        FormalParameterRest(self.add_node(
            span,
            NodeKind::FormalParameterRest,
            NodePayload {
                formal_parameter_rest: FormalParameterRestPayload {
                    decorators,
                    argument,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> ArrayExpression {
        ArrayExpression(self.add_node(
            span,
            NodeKind::ArrayExpression,
            NodePayload {
                array_expression: ArrayExpressionPayload {
                    elements,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
    ) -> ObjectExpression {
        ObjectExpression(self.add_node(
            span,
            NodeKind::ObjectExpression,
            NodePayload {
                object_expression: ObjectExpressionPayload {
                    properties,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn spread_element(&mut self, span: Span, argument: Expr) -> SpreadElement {
        SpreadElement(self.add_node(
            span,
            NodeKind::SpreadElement,
            NodePayload {
                spread_element: SpreadElementPayload {
                    argument,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                object_property: ObjectPropertyPayload {
                    key,
                    value,
                    kind,
                    method,
                    shorthand,
                    computed,
                    _pad0: [0; 4usize],
                },
            },
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
            NodePayload {
                parenthesized_expression: ParenthesizedExpressionPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> ArrowFunctionExpression {
        let _extra_data = [(return_type).into_extra_data(), (body).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        ArrowFunctionExpression(self.add_node(
            span,
            NodeKind::ArrowFunctionExpression,
            NodePayload {
                arrow_function_expression: ArrowFunctionExpressionPayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params,
                    expression,
                    r#async,
                    _pad0: [0; 2usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<Expr>,
    ) -> SequenceExpression {
        SequenceExpression(self.add_node(
            span,
            NodeKind::SequenceExpression,
            NodePayload {
                sequence_expression: SequenceExpressionPayload {
                    expressions,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                member_expression: MemberExpressionPayload {
                    object,
                    property,
                    computed,
                    optional,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> CallExpression {
        let _f0 = self.add_extra(arguments);
        CallExpression(self.add_node(
            span,
            NodeKind::CallExpression,
            NodePayload {
                call_expression: CallExpressionPayload {
                    callee,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    optional,
                    pure,
                    _pad0: [0; 2usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn chain_expression(&mut self, span: Span, expression: Expr) -> ChainExpression {
        ChainExpression(self.add_node(
            span,
            NodeKind::ChainExpression,
            NodePayload {
                chain_expression: ChainExpressionPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> TaggedTemplateExpression {
        TaggedTemplateExpression(self.add_node(
            span,
            NodeKind::TaggedTemplateExpression,
            NodePayload {
                tagged_template_expression: TaggedTemplateExpressionPayload {
                    tag,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    quasi,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> NewExpression {
        NewExpression(self.add_node(
            span,
            NodeKind::NewExpression,
            NodePayload {
                new_expression: NewExpressionPayload {
                    arguments,
                    callee,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn await_expression(&mut self, span: Span, argument: Expr) -> AwaitExpression {
        AwaitExpression(self.add_node(
            span,
            NodeKind::AwaitExpression,
            NodePayload {
                await_expression: AwaitExpressionPayload {
                    argument,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                yield_expression: YieldExpressionPayload {
                    argument: argument.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    delegate,
                    _pad0: [0; 11usize],
                },
            },
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
            NodePayload {
                meta_property: MetaPropertyPayload {
                    meta,
                    property,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ImportExpression {
        let _f0 = self.add_extra(phase);
        ImportExpression(self.add_node(
            span,
            NodeKind::ImportExpression,
            NodePayload {
                import_expression: ImportExpressionPayload {
                    source,
                    options: options.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    sync,
                    _pad0: [0; 3usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn import_declaration(
        &mut self,
        span: Span,
        specifiers: SubRange<ImportDeclarationSpecifier>,
        source: StringLiteral,
        attributes: SubRange<ImportAttribute>,
        phase: Option<ImportPhase>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclaration {
        let _extra_data = [
            (specifiers).into_extra_data(),
            (attributes).into_extra_data(),
            (phase).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        ImportDeclaration(self.add_node(
            span,
            NodeKind::ImportDeclaration,
            NodePayload {
                import_declaration: ImportDeclarationPayload {
                    source,
                    import_kind,
                    _pad0: [0; 7usize],
                    extra_data_start: _extra_data_start,
                },
            },
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
            NodePayload {
                import_specifier: ImportSpecifierPayload {
                    imported,
                    local,
                    import_kind,
                    _pad0: [0; 7usize],
                },
            },
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
            NodePayload {
                import_default_specifier: ImportDefaultSpecifierPayload {
                    local,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                import_namespace_specifier: ImportNamespaceSpecifierPayload {
                    local,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                import_attribute: ImportAttributePayload {
                    key,
                    value,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn export_named_declaration(
        &mut self,
        span: Span,
        declaration: Option<Decl>,
        specifiers: SubRange<ExportSpecifier>,
        source: Option<StringLiteral>,
        attributes: SubRange<ImportAttribute>,
        export_kind: ImportOrExportKind,
    ) -> ExportNamedDeclaration {
        let _extra_data = [
            (specifiers).into_extra_data(),
            (attributes).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        ExportNamedDeclaration(self.add_node(
            span,
            NodeKind::ExportNamedDeclaration,
            NodePayload {
                export_named_declaration: ExportNamedDeclarationPayload {
                    declaration: declaration.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    source: source.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    export_kind,
                    _pad0: [0; 3usize],
                    extra_data_start: _extra_data_start,
                },
            },
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
            NodePayload {
                export_default_declaration: ExportDefaultDeclarationPayload {
                    declaration,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn export_all_declaration(
        &mut self,
        span: Span,
        exported: Option<ModuleExportName>,
        source: StringLiteral,
        attributes: SubRange<ImportAttribute>,
        export_kind: ImportOrExportKind,
    ) -> ExportAllDeclaration {
        let _f0 = self.add_extra(attributes);
        ExportAllDeclaration(self.add_node(
            span,
            NodeKind::ExportAllDeclaration,
            NodePayload {
                export_all_declaration: ExportAllDeclarationPayload {
                    exported: exported.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    source,
                    export_kind,
                    _pad0: [0; 3usize],
                    extra_data_start: _f0,
                },
            },
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
            NodePayload {
                export_specifier: ExportSpecifierPayload {
                    local,
                    exported,
                    export_kind,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn hashbang(&mut self, span: Span, value: AstUtf8) -> Hashbang {
        Hashbang(self.add_node(
            span,
            NodeKind::Hashbang,
            NodePayload {
                hashbang: HashbangPayload {
                    value,
                    _pad0: [0; 8usize],
                },
            },
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
        quasis: SubRange<TemplateElement>,
        types: SubRange<TsType>,
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
        element_types: SubRange<TsTupleElement>,
    ) -> TsType {
        TsType::TsTupleType(self.ts_tuple_type(span, element_types).into())
    }
    #[inline]
    pub fn ts_type_ts_optional_type(&mut self, span: Span, type_annotation: TsType) -> TsType {
        TsType::TsOptionalType(self.ts_optional_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_type_ts_rest_type(&mut self, span: Span, type_annotation: TsTupleElement) -> TsType {
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
    pub fn ts_type_ts_union_type(&mut self, span: Span, types: SubRange<TsType>) -> TsType {
        TsType::TsUnionType(self.ts_union_type(span, types).into())
    }
    #[inline]
    pub fn ts_type_ts_intersection_type(&mut self, span: Span, types: SubRange<TsType>) -> TsType {
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
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
    ) -> TsType {
        TsType::TsFunctionType(
            self.ts_function_type(span, type_parameters, this_param, params, return_type)
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
        members: SubRange<TsTypeElement>,
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
    ) -> TsQualifiedNameLeft {
        TsQualifiedNameLeft::BindingIdentifier(self.binding_identifier(span, name).into())
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
    pub fn ts_type_query_expr_name_ts_type_name_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsTypeQueryExprName {
        TsTypeQueryExprName::TsTypeName(TsTypeName::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn ts_type_query_expr_name_ts_type_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsTypeQueryExprName {
        TsTypeQueryExprName::TsTypeName(TsTypeName::TsQualifiedName(
            self.ts_qualified_name(span, left, right).into(),
        ))
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
        value: f64,
        raw: Option<AstUtf8>,
    ) -> TsLiteral {
        TsLiteral::NumericLiteral(self.numeric_literal(span, value, raw).into())
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
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
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
    pub fn ts_tuple_element_ts_type_ts_any_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsAnyKeyword(self.ts_any_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_unknown_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsUnknownKeyword(
            self.ts_unknown_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_never_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsNeverKeyword(self.ts_never_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_void_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsVoidKeyword(self.ts_void_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_null_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsNullKeyword(self.ts_null_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_undefined_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsUndefinedKeyword(
            self.ts_undefined_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_string_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsStringKeyword(self.ts_string_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_number_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsNumberKeyword(self.ts_number_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_big_int_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsBigIntKeyword(
            self.ts_big_int_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_boolean_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsBooleanKeyword(
            self.ts_boolean_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_symbol_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsSymbolKeyword(self.ts_symbol_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_object_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsObjectKeyword(self.ts_object_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_intrinsic_keyword(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsIntrinsicKeyword(
            self.ts_intrinsic_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_this_type(&mut self, span: Span) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsThisType(self.ts_this_type(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_reference(
        &mut self,
        span: Span,
        type_name: TsTypeName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsTypeReference(
            self.ts_type_reference(span, type_name, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_query(
        &mut self,
        span: Span,
        expr_name: TsTypeQueryExprName,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsTypeQuery(
            self.ts_type_query(span, expr_name, type_arguments).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral,
        options: Option<Expr>,
        qualifier: Option<TsImportTypeQualifier>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsImportType(
            self.ts_import_type(span, source, options, qualifier, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_literal_type(
        &mut self,
        span: Span,
        literal: TsLiteral,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsLiteralType(
            self.ts_literal_type(span, literal).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        types: SubRange<TsType>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsTemplateLiteralType(
            self.ts_template_literal_type(span, quasis, types).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_array_type(
        &mut self,
        span: Span,
        element_type: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsArrayType(
            self.ts_array_type(span, element_type).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_indexed_access_type(
        &mut self,
        span: Span,
        object_type: TsType,
        index_type: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsIndexedAccessType(
            self.ts_indexed_access_type(span, object_type, index_type)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_tuple_type(
        &mut self,
        span: Span,
        element_types: SubRange<TsTupleElement>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsTupleType(
            self.ts_tuple_type(span, element_types).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_optional_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsOptionalType(
            self.ts_optional_type(span, type_annotation).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_rest_type(
        &mut self,
        span: Span,
        type_annotation: TsTupleElement,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsRestType(
            self.ts_rest_type(span, type_annotation).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_js_doc_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsJsDocNullableType(
            self.ts_js_doc_nullable_type(span, type_annotation, postfix)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_js_doc_non_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
        postfix: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsJsDocNonNullableType(
            self.ts_js_doc_non_nullable_type(span, type_annotation, postfix)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_js_doc_unknown_type(
        &mut self,
        span: Span,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsJsDocUnknownType(
            self.ts_js_doc_unknown_type(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_union_type(
        &mut self,
        span: Span,
        types: SubRange<TsType>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsUnionType(self.ts_union_type(span, types).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_intersection_type(
        &mut self,
        span: Span,
        types: SubRange<TsType>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsIntersectionType(
            self.ts_intersection_type(span, types).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_conditional_type(
        &mut self,
        span: Span,
        check_type: TsType,
        extends_type: TsType,
        true_type: TsType,
        false_type: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsConditionalType(
            self.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_infer_type(
        &mut self,
        span: Span,
        type_parameter: TsTypeParameter,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsInferType(
            self.ts_infer_type(span, type_parameter).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_operator(
        &mut self,
        span: Span,
        operator: TsTypeOperatorKind,
        type_annotation: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsTypeOperator(
            self.ts_type_operator(span, operator, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_parenthesized_type(
        &mut self,
        span: Span,
        type_annotation: TsType,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsParenthesizedType(
            self.ts_parenthesized_type(span, type_annotation).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsFunctionType(
            self.ts_function_type(span, type_parameters, this_param, params, return_type)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
        r#abstract: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsConstructorType(
            self.ts_constructor_type(span, type_parameters, params, return_type, r#abstract)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_predicate(
        &mut self,
        span: Span,
        parameter_name: TsTypePredicateName,
        type_annotation: Option<TsTypeAnnotation>,
        asserts: bool,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsTypePredicate(
            self.ts_type_predicate(span, parameter_name, type_annotation, asserts)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_literal(
        &mut self,
        span: Span,
        members: SubRange<TsTypeElement>,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsTypeLiteral(
            self.ts_type_literal(span, members).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_mapped_type(
        &mut self,
        span: Span,
        key: BindingIdentifier,
        constraint: TsType,
        name_type: Option<TsType>,
        type_annotation: Option<TsType>,
        optional: TsMappedTypeModifier,
        readonly: TsMappedTypeModifier,
    ) -> TsTupleElement {
        TsTupleElement::TsType(TsType::TsMappedType(
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
        ))
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
        this_param: Option<TsThisParameter>,
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
                this_param,
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
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsTypeElement {
        TsTypeElement::TsCallSignatureDeclaration(
            self.ts_call_signature_declaration(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
            )
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
        parameters: SubRange<TsIndexSignatureParameter>,
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
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> TsEnumMemberName {
        TsEnumMemberName::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn ts_module_name_binding_identifier(&mut self, span: Span, name: AstUtf8) -> TsModuleName {
        TsModuleName::BindingIdentifier(self.binding_identifier(span, name).into())
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
        directives: SubRange<Directive>,
        body: SubRange<Stmt>,
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
    pub fn ts_module_reference_ts_type_name_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> TsModuleReference {
        TsModuleReference::TsTypeName(TsTypeName::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn ts_module_reference_ts_type_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft,
        right: IdentifierName,
    ) -> TsModuleReference {
        TsModuleReference::TsTypeName(TsTypeName::TsQualifiedName(
            self.ts_qualified_name(span, left, right).into(),
        ))
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
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> TsFunction {
        let _extra_data = [
            (this_param).into_extra_data(),
            (params).into_extra_data(),
            (return_type).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsFunction(self.add_node(
            span,
            NodeKind::TsFunction,
            NodePayload {
                ts_function: TsFunctionPayload {
                    id: id.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    generator,
                    r#async,
                    declare,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
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
        let _extra_data = [(decorators).into_extra_data(), (value).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsMethodDefinition(self.add_node(
            span,
            NodeKind::TsMethodDefinition,
            NodePayload {
                ts_method_definition: TsMethodDefinitionPayload {
                    key,
                    kind,
                    computed,
                    r#static,
                    r#override,
                    optional,
                    r#abstract,
                    accessibility,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_annotation(&mut self, span: Span, type_annotation: TsType) -> TsTypeAnnotation {
        TsTypeAnnotation(self.add_node(
            span,
            NodeKind::TsTypeAnnotation,
            NodePayload {
                ts_type_annotation: TsTypeAnnotationPayload {
                    type_annotation,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_any_keyword(&mut self, span: Span) -> TsAnyKeyword {
        TsAnyKeyword(self.add_node(span, NodeKind::TsAnyKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_unknown_keyword(&mut self, span: Span) -> TsUnknownKeyword {
        TsUnknownKeyword(self.add_node(span, NodeKind::TsUnknownKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_never_keyword(&mut self, span: Span) -> TsNeverKeyword {
        TsNeverKeyword(self.add_node(span, NodeKind::TsNeverKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_void_keyword(&mut self, span: Span) -> TsVoidKeyword {
        TsVoidKeyword(self.add_node(span, NodeKind::TsVoidKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_null_keyword(&mut self, span: Span) -> TsNullKeyword {
        TsNullKeyword(self.add_node(span, NodeKind::TsNullKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_undefined_keyword(&mut self, span: Span) -> TsUndefinedKeyword {
        TsUndefinedKeyword(self.add_node(
            span,
            NodeKind::TsUndefinedKeyword,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn ts_string_keyword(&mut self, span: Span) -> TsStringKeyword {
        TsStringKeyword(self.add_node(span, NodeKind::TsStringKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_number_keyword(&mut self, span: Span) -> TsNumberKeyword {
        TsNumberKeyword(self.add_node(span, NodeKind::TsNumberKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_big_int_keyword(&mut self, span: Span) -> TsBigIntKeyword {
        TsBigIntKeyword(self.add_node(span, NodeKind::TsBigIntKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_boolean_keyword(&mut self, span: Span) -> TsBooleanKeyword {
        TsBooleanKeyword(self.add_node(span, NodeKind::TsBooleanKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_symbol_keyword(&mut self, span: Span) -> TsSymbolKeyword {
        TsSymbolKeyword(self.add_node(span, NodeKind::TsSymbolKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_object_keyword(&mut self, span: Span) -> TsObjectKeyword {
        TsObjectKeyword(self.add_node(span, NodeKind::TsObjectKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_intrinsic_keyword(&mut self, span: Span) -> TsIntrinsicKeyword {
        TsIntrinsicKeyword(self.add_node(
            span,
            NodeKind::TsIntrinsicKeyword,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn ts_this_type(&mut self, span: Span) -> TsThisType {
        TsThisType(self.add_node(span, NodeKind::TsThisType, NodePayload { raw: 0 }))
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
            NodePayload {
                ts_type_reference: TsTypeReferencePayload {
                    type_name,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_qualified_name: TsQualifiedNamePayload {
                    left,
                    right,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_type_query: TsTypeQueryPayload {
                    expr_name,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_import_type: TsImportTypePayload {
                    source,
                    options: options.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    qualifier: qualifier.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
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
            NodePayload {
                ts_type_parameter: TsTypeParameterPayload {
                    name,
                    constraint: constraint.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    default: default.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    r#in,
                    out,
                    r#const,
                    _pad0: [0; 1usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_parameter_declaration(
        &mut self,
        span: Span,
        params: SubRange<TsTypeParameter>,
    ) -> TsTypeParameterDeclaration {
        TsTypeParameterDeclaration(self.add_node(
            span,
            NodeKind::TsTypeParameterDeclaration,
            NodePayload {
                ts_type_parameter_declaration: TsTypeParameterDeclarationPayload {
                    params,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_parameter_instantiation(
        &mut self,
        span: Span,
        params: SubRange<TsType>,
    ) -> TsTypeParameterInstantiation {
        TsTypeParameterInstantiation(self.add_node(
            span,
            NodeKind::TsTypeParameterInstantiation,
            NodePayload {
                ts_type_parameter_instantiation: TsTypeParameterInstantiationPayload {
                    params,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_literal_type(&mut self, span: Span, literal: TsLiteral) -> TsLiteralType {
        TsLiteralType(self.add_node(
            span,
            NodeKind::TsLiteralType,
            NodePayload {
                ts_literal_type: TsLiteralTypePayload {
                    literal,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        types: SubRange<TsType>,
    ) -> TsTemplateLiteralType {
        TsTemplateLiteralType(self.add_node(
            span,
            NodeKind::TsTemplateLiteralType,
            NodePayload {
                ts_template_literal_type: TsTemplateLiteralTypePayload { quasis, types },
            },
        ))
    }
    #[inline]
    pub fn ts_array_type(&mut self, span: Span, element_type: TsType) -> TsArrayType {
        TsArrayType(self.add_node(
            span,
            NodeKind::TsArrayType,
            NodePayload {
                ts_array_type: TsArrayTypePayload {
                    element_type,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                ts_indexed_access_type: TsIndexedAccessTypePayload {
                    object_type,
                    index_type,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_tuple_type(
        &mut self,
        span: Span,
        element_types: SubRange<TsTupleElement>,
    ) -> TsTupleType {
        TsTupleType(self.add_node(
            span,
            NodeKind::TsTupleType,
            NodePayload {
                ts_tuple_type: TsTupleTypePayload {
                    element_types,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_named_tuple_member: TsNamedTupleMemberPayload {
                    label,
                    element_type,
                    optional,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_optional_type(&mut self, span: Span, type_annotation: TsType) -> TsOptionalType {
        TsOptionalType(self.add_node(
            span,
            NodeKind::TsOptionalType,
            NodePayload {
                ts_optional_type: TsOptionalTypePayload {
                    type_annotation,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_rest_type(&mut self, span: Span, type_annotation: TsTupleElement) -> TsRestType {
        TsRestType(self.add_node(
            span,
            NodeKind::TsRestType,
            NodePayload {
                ts_rest_type: TsRestTypePayload {
                    type_annotation,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                ts_js_doc_nullable_type: TsJsDocNullableTypePayload {
                    type_annotation,
                    postfix,
                    _pad0: [0; 11usize],
                },
            },
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
            NodePayload {
                ts_js_doc_non_nullable_type: TsJsDocNonNullableTypePayload {
                    type_annotation,
                    postfix,
                    _pad0: [0; 11usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_js_doc_unknown_type(&mut self, span: Span) -> TsJsDocUnknownType {
        TsJsDocUnknownType(self.add_node(
            span,
            NodeKind::TsJsDocUnknownType,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn ts_union_type(&mut self, span: Span, types: SubRange<TsType>) -> TsUnionType {
        TsUnionType(self.add_node(
            span,
            NodeKind::TsUnionType,
            NodePayload {
                ts_union_type: TsUnionTypePayload {
                    types,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_intersection_type(
        &mut self,
        span: Span,
        types: SubRange<TsType>,
    ) -> TsIntersectionType {
        TsIntersectionType(self.add_node(
            span,
            NodeKind::TsIntersectionType,
            NodePayload {
                ts_intersection_type: TsIntersectionTypePayload {
                    types,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_conditional_type: TsConditionalTypePayload {
                    check_type,
                    extends_type,
                    true_type,
                    false_type,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_infer_type(&mut self, span: Span, type_parameter: TsTypeParameter) -> TsInferType {
        TsInferType(self.add_node(
            span,
            NodeKind::TsInferType,
            NodePayload {
                ts_infer_type: TsInferTypePayload {
                    type_parameter,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                ts_type_operator: TsTypeOperatorPayload {
                    type_annotation,
                    operator,
                    _pad0: [0; 11usize],
                },
            },
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
            NodePayload {
                ts_parenthesized_type: TsParenthesizedTypePayload {
                    type_annotation,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: TsTypeAnnotation,
    ) -> TsFunctionType {
        TsFunctionType(self.add_node(
            span,
            NodeKind::TsFunctionType,
            NodePayload {
                ts_function_type: TsFunctionTypePayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    this_param: this_param.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params,
                    return_type,
                },
            },
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
            NodePayload {
                ts_constructor_type: TsConstructorTypePayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params,
                    return_type,
                    r#abstract,
                    _pad0: [0; 3usize],
                },
            },
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
            NodePayload {
                ts_type_predicate: TsTypePredicatePayload {
                    parameter_name,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    asserts,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_literal(
        &mut self,
        span: Span,
        members: SubRange<TsTypeElement>,
    ) -> TsTypeLiteral {
        TsTypeLiteral(self.add_node(
            span,
            NodeKind::TsTypeLiteral,
            NodePayload {
                ts_type_literal: TsTypeLiteralPayload {
                    members,
                    _pad0: [0; 8usize],
                },
            },
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
            (name_type).into_extra_data(),
            (type_annotation).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsMappedType(self.add_node(
            span,
            NodeKind::TsMappedType,
            NodePayload {
                ts_mapped_type: TsMappedTypePayload {
                    key,
                    constraint,
                    optional,
                    readonly,
                    _pad0: [0; 2usize],
                    extra_data_start: _extra_data_start,
                },
            },
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
            NodePayload {
                ts_property_signature: TsPropertySignaturePayload {
                    key,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    computed,
                    optional,
                    readonly,
                    _pad0: [0; 5usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_method_signature(
        &mut self,
        span: Span,
        key: PropertyKey,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        kind: TsMethodSignatureKind,
        computed: bool,
        optional: bool,
    ) -> TsMethodSignature {
        let _extra_data = [
            (this_param).into_extra_data(),
            (params).into_extra_data(),
            (return_type).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsMethodSignature(self.add_node(
            span,
            NodeKind::TsMethodSignature,
            NodePayload {
                ts_method_signature: TsMethodSignaturePayload {
                    key,
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    kind,
                    computed,
                    optional,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_call_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
    ) -> TsCallSignatureDeclaration {
        TsCallSignatureDeclaration(self.add_node(
            span,
            NodeKind::TsCallSignatureDeclaration,
            NodePayload {
                ts_call_signature_declaration: TsCallSignatureDeclarationPayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    this_param: this_param.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params,
                    return_type: return_type.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
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
            NodePayload {
                ts_construct_signature_declaration: TsConstructSignatureDeclarationPayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params,
                    return_type: return_type.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_index_signature(
        &mut self,
        span: Span,
        parameters: SubRange<TsIndexSignatureParameter>,
        type_annotation: TsTypeAnnotation,
        readonly: bool,
        r#static: bool,
    ) -> TsIndexSignature {
        TsIndexSignature(self.add_node(
            span,
            NodeKind::TsIndexSignature,
            NodePayload {
                ts_index_signature: TsIndexSignaturePayload {
                    parameters,
                    type_annotation,
                    readonly,
                    r#static,
                    _pad0: [0; 2usize],
                },
            },
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
            NodePayload {
                ts_type_alias_declaration: TsTypeAliasDeclarationPayload {
                    id,
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_annotation,
                    declare,
                    _pad0: [0; 3usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        type_parameters: Option<TsTypeParameterDeclaration>,
        extends: SubRange<TsInterfaceHeritage>,
        body: TsInterfaceBody,
        declare: bool,
    ) -> TsInterfaceDeclaration {
        let _extra_data = [(extends).into_extra_data(), (body).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsInterfaceDeclaration(self.add_node(
            span,
            NodeKind::TsInterfaceDeclaration,
            NodePayload {
                ts_interface_declaration: TsInterfaceDeclarationPayload {
                    id,
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    declare,
                    _pad0: [0; 3usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_interface_body(
        &mut self,
        span: Span,
        body: SubRange<TsTypeElement>,
    ) -> TsInterfaceBody {
        TsInterfaceBody(self.add_node(
            span,
            NodeKind::TsInterfaceBody,
            NodePayload {
                ts_interface_body: TsInterfaceBodyPayload {
                    body,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_interface_heritage: TsInterfaceHeritagePayload {
                    expression,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_class_implements: TsClassImplementsPayload {
                    expression,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_enum_declaration: TsEnumDeclarationPayload {
                    id,
                    body,
                    is_const,
                    declare,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_enum_body(&mut self, span: Span, members: SubRange<TsEnumMember>) -> TsEnumBody {
        TsEnumBody(self.add_node(
            span,
            NodeKind::TsEnumBody,
            NodePayload {
                ts_enum_body: TsEnumBodyPayload {
                    members,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_enum_member: TsEnumMemberPayload {
                    id,
                    initializer: initializer.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    computed,
                    _pad0: [0; 7usize],
                },
            },
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
            NodePayload {
                ts_module_declaration: TsModuleDeclarationPayload {
                    id,
                    body: body.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    kind,
                    declare,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_module_block(
        &mut self,
        span: Span,
        directives: SubRange<Directive>,
        body: SubRange<Stmt>,
    ) -> TsModuleBlock {
        TsModuleBlock(self.add_node(
            span,
            NodeKind::TsModuleBlock,
            NodePayload {
                ts_module_block: TsModuleBlockPayload { directives, body },
            },
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
            NodePayload {
                ts_global_declaration: TsGlobalDeclarationPayload {
                    id,
                    body,
                    declare,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_parameter_property(
        &mut self,
        span: Span,
        decorators: SubRange<Decorator>,
        parameter: FormalParameter,
        r#override: bool,
        readonly: bool,
        accessibility: Accessibility,
    ) -> TsParameterProperty {
        TsParameterProperty(self.add_node(
            span,
            NodeKind::TsParameterProperty,
            NodePayload {
                ts_parameter_property: TsParameterPropertyPayload {
                    decorators,
                    parameter,
                    r#override,
                    readonly,
                    accessibility,
                    _pad0: [0; 1usize],
                },
            },
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
            NodePayload {
                ts_this_parameter: TsThisParameterPayload {
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                ts_as_expression: TsAsExpressionPayload {
                    expression,
                    type_annotation,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_satisfies_expression: TsSatisfiesExpressionPayload {
                    expression,
                    type_annotation,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                ts_type_assertion: TsTypeAssertionPayload {
                    type_annotation,
                    expression,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_non_null_expression(&mut self, span: Span, expression: Expr) -> TsNonNullExpression {
        TsNonNullExpression(self.add_node(
            span,
            NodeKind::TsNonNullExpression,
            NodePayload {
                ts_non_null_expression: TsNonNullExpressionPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                ts_instantiation_expression: TsInstantiationExpressionPayload {
                    expression,
                    type_arguments,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_export_assignment(&mut self, span: Span, expression: Expr) -> TsExportAssignment {
        TsExportAssignment(self.add_node(
            span,
            NodeKind::TsExportAssignment,
            NodePayload {
                ts_export_assignment: TsExportAssignmentPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                ts_namespace_export_declaration: TsNamespaceExportDeclarationPayload {
                    id,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                ts_import_equals_declaration: TsImportEqualsDeclarationPayload {
                    id,
                    module_reference,
                    import_kind,
                    _pad0: [0; 7usize],
                },
            },
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
            NodePayload {
                ts_external_module_reference: TsExternalModuleReferencePayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_index_signature_parameter(
        &mut self,
        span: Span,
        name: BindingIdentifier,
        type_annotation: TsTypeAnnotation,
    ) -> TsIndexSignatureParameter {
        TsIndexSignatureParameter(self.add_node(
            span,
            NodeKind::TsIndexSignatureParameter,
            NodePayload {
                ts_index_signature_parameter: TsIndexSignatureParameterPayload {
                    name,
                    type_annotation,
                    _pad0: [0; 8usize],
                },
            },
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
        children: SubRange<JsxChild>,
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
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> JsxAttributeValue {
        JsxAttributeValue::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_this_expression(&mut self, span: Span) -> JsxExpression {
        JsxExpression::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_super(&mut self, span: Span) -> JsxExpression {
        JsxExpression::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8,
        raw: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8>,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_null_literal(&mut self, span: Span) -> JsxExpression {
        JsxExpression::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8,
        flags: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        expressions: SubRange<Expr>,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: BinaryOperator,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr,
        right: Expr,
        operator: LogicalOperator,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr,
        operator: UnaryOperator,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget,
        operator: UpdateOperator,
        prefix: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget,
        right: Expr,
        operator: AssignmentOperator,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr,
        consequent: Expr,
        alternate: Expr,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<Expr>,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr,
        property: PropertyKey,
        computed: bool,
        optional: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
        optional: bool,
        pure: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional, pure)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        arguments: SubRange<Argument>,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
        quasi: TemplateLiteral,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: ArrowFunctionBody,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<Option<Argument>>,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<ObjectPropertyKind>,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        this_param: Option<TsThisParameter>,
        params: FormalParameters,
        return_type: Option<TsTypeAnnotation>,
        body: FunctionBody,
        generator: bool,
        r#async: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::Function(
            self.function(
                span,
                function_type,
                id,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                generator,
                r#async,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<Decorator>,
        id: Option<BindingIdentifier>,
        type_parameters: Option<TsTypeParameterDeclaration>,
        super_class: Option<Expr>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
        implements: SubRange<TsClassImplements>,
        body: ClassBody,
        r#abstract: bool,
        declare: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::Class(
            self.class(
                span,
                class_type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr>,
        delegate: bool,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_annotation: TsType,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr,
        type_arguments: TsTypeParameterInstantiation,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement,
        children: SubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> JsxExpression {
        JsxExpression::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
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
        children: SubRange<JsxChild>,
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
        children: SubRange<JsxChild>,
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
        children: SubRange<JsxChild>,
        closing_element: Option<JsxClosingElement>,
    ) -> JsxElement {
        JsxElement(self.add_node(
            span,
            NodeKind::JsxElement,
            NodePayload {
                jsx_element: JsxElementPayload {
                    children,
                    opening_element,
                    closing_element:
                        closing_element.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_opening_element(
        &mut self,
        span: Span,
        name: JsxElementName,
        type_arguments: Option<TsTypeParameterInstantiation>,
        attributes: SubRange<JsxAttributeItem>,
        self_closing: bool,
    ) -> JsxOpeningElement {
        let _f0 = self.add_extra(attributes);
        JsxOpeningElement(self.add_node(
            span,
            NodeKind::JsxOpeningElement,
            NodePayload {
                jsx_opening_element: JsxOpeningElementPayload {
                    name,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    self_closing,
                    _pad0: [0; 3usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_closing_element(&mut self, span: Span, name: JsxElementName) -> JsxClosingElement {
        JsxClosingElement(self.add_node(
            span,
            NodeKind::JsxClosingElement,
            NodePayload {
                jsx_closing_element: JsxClosingElementPayload {
                    name,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment,
        children: SubRange<JsxChild>,
        closing_fragment: JsxClosingFragment,
    ) -> JsxFragment {
        JsxFragment(self.add_node(
            span,
            NodeKind::JsxFragment,
            NodePayload {
                jsx_fragment: JsxFragmentPayload {
                    children,
                    opening_fragment,
                    closing_fragment,
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_opening_fragment(&mut self, span: Span) -> JsxOpeningFragment {
        JsxOpeningFragment(self.add_node(
            span,
            NodeKind::JsxOpeningFragment,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn jsx_closing_fragment(&mut self, span: Span) -> JsxClosingFragment {
        JsxClosingFragment(self.add_node(
            span,
            NodeKind::JsxClosingFragment,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn jsx_identifier(&mut self, span: Span, name: AstUtf8) -> JsxIdentifier {
        JsxIdentifier(self.add_node(
            span,
            NodeKind::JsxIdentifier,
            NodePayload {
                jsx_identifier: JsxIdentifierPayload {
                    name,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                jsx_namespaced_name: JsxNamespacedNamePayload {
                    namespace,
                    name,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                jsx_member_expression: JsxMemberExpressionPayload {
                    object,
                    property,
                    _pad0: [0; 8usize],
                },
            },
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
            NodePayload {
                jsx_attribute: JsxAttributePayload {
                    name,
                    value: value.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_spread_attribute(&mut self, span: Span, argument: Expr) -> JsxSpreadAttribute {
        JsxSpreadAttribute(self.add_node(
            span,
            NodeKind::JsxSpreadAttribute,
            NodePayload {
                jsx_spread_attribute: JsxSpreadAttributePayload {
                    argument,
                    _pad0: [0; 12usize],
                },
            },
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
            NodePayload {
                jsx_expression_container: JsxExpressionContainerPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_empty_expression(&mut self, span: Span) -> JsxEmptyExpression {
        JsxEmptyExpression(self.add_node(
            span,
            NodeKind::JsxEmptyExpression,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn jsx_text(&mut self, span: Span, value: AstUtf8) -> JsxText {
        JsxText(self.add_node(
            span,
            NodeKind::JsxText,
            NodePayload {
                jsx_text: JsxTextPayload {
                    value,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_spread_child(&mut self, span: Span, expression: Expr) -> JsxSpreadChild {
        JsxSpreadChild(self.add_node(
            span,
            NodeKind::JsxSpreadChild,
            NodePayload {
                jsx_spread_child: JsxSpreadChildPayload {
                    expression,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
}
