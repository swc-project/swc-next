use crate::{Ast, DUMMY_SP, ImportedAst, NodeKindData};
impl Ast<'_> {
    pub(crate) fn import_nodes(&mut self, source_ast: &Ast<'_>, mapping: ImportedAst) {
        for index in 0..source_ast.node_count() {
            let id = crate::NodeId::from_usize_unchecked(index);
            match source_ast.kind_data(id) {
                NodeKindData::Program(node) => {
                    let directives = {
                        let values = node
                            .directives(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let body = {
                        let values = node
                            .body(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let hashbang = node.hashbang(source_ast).map(|value| mapping.node(value));
                    let copied = self.program(DUMMY_SP, directives, body, hashbang);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::Super(node) => {
                    let copied = self.super_(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ThisExpression(node) => {
                    let copied = self.this_expression(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::StringLiteral(node) => {
                    let value = self.add_wtf8(source_ast.get_wtf8(node.value(source_ast)));
                    let raw = self.add_utf8(source_ast.get_utf8(node.raw(source_ast)));
                    let copied = self.string_literal(DUMMY_SP, value, raw);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::NumericLiteral(node) => {
                    let value = node.value(source_ast);
                    let raw = node
                        .raw(source_ast)
                        .map(|value| self.add_utf8(source_ast.get_utf8(value)));
                    let copied = self.numeric_literal(DUMMY_SP, value, raw);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BigIntLiteral(node) => {
                    let raw = self.add_utf8(source_ast.get_utf8(node.raw(source_ast)));
                    let copied = self.big_int_literal(DUMMY_SP, raw);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BooleanLiteral(node) => {
                    let value = node.value(source_ast);
                    let copied = self.boolean_literal(DUMMY_SP, value);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::NullLiteral(node) => {
                    let copied = self.null_literal(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::RegExpLiteral(node) => {
                    let pattern = self.add_utf8(source_ast.get_utf8(node.pattern(source_ast)));
                    let flags = self.add_utf8(source_ast.get_utf8(node.flags(source_ast)));
                    let copied = self.reg_exp_literal(DUMMY_SP, pattern, flags);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TemplateLiteral(node) => {
                    let quasis = {
                        let values = node
                            .quasis(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let expressions = {
                        let values = node
                            .expressions(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.template_literal(DUMMY_SP, quasis, expressions);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TemplateElement(node) => {
                    let cooked = self.add_wtf8(source_ast.get_wtf8(node.cooked(source_ast)));
                    let raw = self.add_utf8(source_ast.get_utf8(node.raw(source_ast)));
                    let tail = node.tail(source_ast);
                    let is_cooked_undefined = node.is_cooked_undefined(source_ast);
                    let copied =
                        self.template_element(DUMMY_SP, cooked, raw, tail, is_cooked_undefined);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::IdentifierReference(node) => {
                    let name = self.add_utf8(source_ast.get_utf8(node.name(source_ast)));
                    let copied = self.identifier_reference(DUMMY_SP, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::PrivateIdentifier(node) => {
                    let name = self.add_utf8(source_ast.get_utf8(node.name(source_ast)));
                    let copied = self.private_identifier(DUMMY_SP, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BindingIdentifier(node) => {
                    let name = self.add_utf8(source_ast.get_utf8(node.name(source_ast)));
                    let copied = self.binding_identifier(DUMMY_SP, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::IdentifierName(node) => {
                    let name = self.add_utf8(source_ast.get_utf8(node.name(source_ast)));
                    let copied = self.identifier_name(DUMMY_SP, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::LabelIdentifier(node) => {
                    let name = self.add_utf8(source_ast.get_utf8(node.name(source_ast)));
                    let copied = self.label_identifier(DUMMY_SP, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::AssignmentPattern(node) => {
                    let left = mapping.node(node.left(source_ast));
                    let right = mapping.node(node.right(source_ast));
                    let copied = self.assignment_pattern(DUMMY_SP, left, right);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BindingRestElement(node) => {
                    let argument = mapping.node(node.argument(source_ast));
                    let copied = self.binding_rest_element(DUMMY_SP, argument);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ArrayPattern(node) => {
                    let elements = {
                        let values = node
                            .elements(source_ast)
                            .iter()
                            .map(|slot| {
                                source_ast
                                    .get_node_in_sub_range(slot)
                                    .map(|value| mapping.node(value))
                            })
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let rest = node.rest(source_ast).map(|value| mapping.node(value));
                    let copied = self.array_pattern(DUMMY_SP, elements, rest);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ObjectPattern(node) => {
                    let properties = {
                        let values = node
                            .properties(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let rest = node.rest(source_ast).map(|value| mapping.node(value));
                    let copied = self.object_pattern(DUMMY_SP, properties, rest);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BindingProperty(node) => {
                    let key = mapping.node(node.key(source_ast));
                    let value = mapping.node(node.value(source_ast));
                    let shorthand = node.shorthand(source_ast);
                    let computed = node.computed(source_ast);
                    let copied = self.binding_property(DUMMY_SP, key, value, shorthand, computed);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BinaryExpression(node) => {
                    let left = mapping.node(node.left(source_ast));
                    let right = mapping.node(node.right(source_ast));
                    let operator = node.operator(source_ast);
                    let copied = self.binary_expression(DUMMY_SP, left, right, operator);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::LogicalExpression(node) => {
                    let left = mapping.node(node.left(source_ast));
                    let right = mapping.node(node.right(source_ast));
                    let operator = node.operator(source_ast);
                    let copied = self.logical_expression(DUMMY_SP, left, right, operator);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ConditionalExpression(node) => {
                    let test = mapping.node(node.test(source_ast));
                    let consequent = mapping.node(node.consequent(source_ast));
                    let alternate = mapping.node(node.alternate(source_ast));
                    let copied = self.conditional_expression(DUMMY_SP, test, consequent, alternate);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::UnaryExpression(node) => {
                    let argument = mapping.node(node.argument(source_ast));
                    let operator = node.operator(source_ast);
                    let copied = self.unary_expression(DUMMY_SP, argument, operator);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::UpdateExpression(node) => {
                    let argument = mapping.node(node.argument(source_ast));
                    let operator = node.operator(source_ast);
                    let prefix = node.prefix(source_ast);
                    let copied = self.update_expression(DUMMY_SP, argument, operator, prefix);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::AssignmentExpression(node) => {
                    let left = mapping.node(node.left(source_ast));
                    let right = mapping.node(node.right(source_ast));
                    let operator = node.operator(source_ast);
                    let copied = self.assignment_expression(DUMMY_SP, left, right, operator);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::VariableDeclaration(node) => {
                    let kind = node.kind(source_ast);
                    let declarators = {
                        let values = node
                            .declarators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let declare = node.declare(source_ast);
                    let copied = self.variable_declaration(DUMMY_SP, kind, declarators, declare);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::VariableDeclarator(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let init = node.init(source_ast).map(|value| mapping.node(value));
                    let definite = node.definite(source_ast);
                    let copied =
                        self.variable_declarator(DUMMY_SP, id, type_annotation, init, definite);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::IfStatement(node) => {
                    let test = mapping.node(node.test(source_ast));
                    let consequent = mapping.node(node.consequent(source_ast));
                    let alternate = node.alternate(source_ast).map(|value| mapping.node(value));
                    let copied = self.if_statement(DUMMY_SP, test, consequent, alternate);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::SwitchStatement(node) => {
                    let discriminant = mapping.node(node.discriminant(source_ast));
                    let cases = {
                        let values = node
                            .cases(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.switch_statement(DUMMY_SP, discriminant, cases);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::SwitchCase(node) => {
                    let test = node.test(source_ast).map(|value| mapping.node(value));
                    let consequent = {
                        let values = node
                            .consequent(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.switch_case(DUMMY_SP, test, consequent);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ForStatement(node) => {
                    let init = node.init(source_ast).map(|value| mapping.node(value));
                    let test = node.test(source_ast).map(|value| mapping.node(value));
                    let update = node.update(source_ast).map(|value| mapping.node(value));
                    let body = mapping.node(node.body(source_ast));
                    let copied = self.for_statement(DUMMY_SP, init, test, update, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ForInStatement(node) => {
                    let left = mapping.node(node.left(source_ast));
                    let right = mapping.node(node.right(source_ast));
                    let body = mapping.node(node.body(source_ast));
                    let copied = self.for_in_statement(DUMMY_SP, left, right, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ForOfStatement(node) => {
                    let left = mapping.node(node.left(source_ast));
                    let right = mapping.node(node.right(source_ast));
                    let body = mapping.node(node.body(source_ast));
                    let r#await = node.r#await(source_ast);
                    let copied = self.for_of_statement(DUMMY_SP, left, right, body, r#await);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TryStatement(node) => {
                    let block = mapping.node(node.block(source_ast));
                    let handler = node.handler(source_ast).map(|value| mapping.node(value));
                    let finalizer = node.finalizer(source_ast).map(|value| mapping.node(value));
                    let copied = self.try_statement(DUMMY_SP, block, handler, finalizer);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::CatchClause(node) => {
                    let param = node.param(source_ast).map(|value| mapping.node(value));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let body = mapping.node(node.body(source_ast));
                    let copied = self.catch_clause(DUMMY_SP, param, type_annotation, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BreakStatement(node) => {
                    let label = node.label(source_ast).map(|value| mapping.node(value));
                    let copied = self.break_statement(DUMMY_SP, label);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ContinueStatement(node) => {
                    let label = node.label(source_ast).map(|value| mapping.node(value));
                    let copied = self.continue_statement(DUMMY_SP, label);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::LabeledStatement(node) => {
                    let label = mapping.node(node.label(source_ast));
                    let body = mapping.node(node.body(source_ast));
                    let copied = self.labeled_statement(DUMMY_SP, label, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ExpressionStatement(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.expression_statement(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ReturnStatement(node) => {
                    let argument = node.argument(source_ast).map(|value| mapping.node(value));
                    let copied = self.return_statement(DUMMY_SP, argument);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ThrowStatement(node) => {
                    let argument = mapping.node(node.argument(source_ast));
                    let copied = self.throw_statement(DUMMY_SP, argument);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::DebuggerStatement(node) => {
                    let copied = self.debugger_statement(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::EmptyStatement(node) => {
                    let copied = self.empty_statement(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::Directive(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let value = self.add_utf8(source_ast.get_utf8(node.value(source_ast)));
                    let copied = self.directive(DUMMY_SP, expression, value);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::Decorator(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.decorator(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::Class(node) => {
                    let class_type = node.class_type(source_ast);
                    let decorators = {
                        let values = node
                            .decorators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let id = node.id(source_ast).map(|value| mapping.node(value));
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let super_class = node
                        .super_class(source_ast)
                        .map(|value| mapping.node(value));
                    let super_type_arguments = node
                        .super_type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let implements = {
                        let values = node
                            .implements(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let body = mapping.node(node.body(source_ast));
                    let r#abstract = node.r#abstract(source_ast);
                    let declare = node.declare(source_ast);
                    let copied = self.class(
                        DUMMY_SP,
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
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ClassBody(node) => {
                    let body = {
                        let values = node
                            .body(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.class_body(DUMMY_SP, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::MethodDefinition(node) => {
                    let decorators = {
                        let values = node
                            .decorators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let key = mapping.node(node.key(source_ast));
                    let value = mapping.node(node.value(source_ast));
                    let kind = node.kind(source_ast);
                    let computed = node.computed(source_ast);
                    let r#static = node.r#static(source_ast);
                    let r#override = node.r#override(source_ast);
                    let optional = node.optional(source_ast);
                    let r#abstract = node.r#abstract(source_ast);
                    let accessibility = node.accessibility(source_ast);
                    let copied = self.method_definition(
                        DUMMY_SP,
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
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::PropertyDefinition(node) => {
                    let decorators = {
                        let values = node
                            .decorators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let key = mapping.node(node.key(source_ast));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let value = node.value(source_ast).map(|value| mapping.node(value));
                    let computed = node.computed(source_ast);
                    let r#static = node.r#static(source_ast);
                    let accessor = node.accessor(source_ast);
                    let declare = node.declare(source_ast);
                    let r#override = node.r#override(source_ast);
                    let optional = node.optional(source_ast);
                    let definite = node.definite(source_ast);
                    let readonly = node.readonly(source_ast);
                    let r#abstract = node.r#abstract(source_ast);
                    let accessibility = node.accessibility(source_ast);
                    let copied = self.property_definition(
                        DUMMY_SP,
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
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::StaticBlock(node) => {
                    let body = {
                        let values = node
                            .body(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.static_block(DUMMY_SP, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::Function(node) => {
                    let function_type = node.function_type(source_ast);
                    let id = node.id(source_ast).map(|value| mapping.node(value));
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let this_param = node.this_param(source_ast).map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = node
                        .return_type(source_ast)
                        .map(|value| mapping.node(value));
                    let body = mapping.node(node.body(source_ast));
                    let generator = node.generator(source_ast);
                    let r#async = node.r#async(source_ast);
                    let copied = self.function(
                        DUMMY_SP,
                        function_type,
                        id,
                        type_parameters,
                        this_param,
                        params,
                        return_type,
                        body,
                        generator,
                        r#async,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::FunctionBody(node) => {
                    let directives = {
                        let values = node
                            .directives(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let body = {
                        let values = node
                            .body(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.function_body(DUMMY_SP, directives, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::BlockStatement(node) => {
                    let body = {
                        let values = node
                            .body(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.block_statement(DUMMY_SP, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::WhileStatement(node) => {
                    let test = mapping.node(node.test(source_ast));
                    let body = mapping.node(node.body(source_ast));
                    let copied = self.while_statement(DUMMY_SP, test, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::DoWhileStatement(node) => {
                    let body = mapping.node(node.body(source_ast));
                    let test = mapping.node(node.test(source_ast));
                    let copied = self.do_while_statement(DUMMY_SP, body, test);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::WithStatement(node) => {
                    let object = mapping.node(node.object(source_ast));
                    let body = mapping.node(node.body(source_ast));
                    let copied = self.with_statement(DUMMY_SP, object, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::FormalParameters(node) => {
                    let items = {
                        let values = node
                            .items(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let rest = node.rest(source_ast).map(|value| mapping.node(value));
                    let kind = node.kind(source_ast);
                    let copied = self.formal_parameters(DUMMY_SP, items, rest, kind);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::FormalParameter(node) => {
                    let decorators = {
                        let values = node
                            .decorators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let pattern = mapping.node(node.pattern(source_ast));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let initializer = node
                        .initializer(source_ast)
                        .map(|value| mapping.node(value));
                    let optional = node.optional(source_ast);
                    let copied = self.formal_parameter(
                        DUMMY_SP,
                        decorators,
                        pattern,
                        type_annotation,
                        initializer,
                        optional,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::FormalParameterRest(node) => {
                    let decorators = {
                        let values = node
                            .decorators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let argument = mapping.node(node.argument(source_ast));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let copied =
                        self.formal_parameter_rest(DUMMY_SP, decorators, argument, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ArrayExpression(node) => {
                    let elements = {
                        let values = node
                            .elements(source_ast)
                            .iter()
                            .map(|slot| {
                                source_ast
                                    .get_node_in_sub_range(slot)
                                    .map(|value| mapping.node(value))
                            })
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.array_expression(DUMMY_SP, elements);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ObjectExpression(node) => {
                    let properties = {
                        let values = node
                            .properties(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.object_expression(DUMMY_SP, properties);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::SpreadElement(node) => {
                    let argument = mapping.node(node.argument(source_ast));
                    let copied = self.spread_element(DUMMY_SP, argument);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ObjectProperty(node) => {
                    let key = mapping.node(node.key(source_ast));
                    let value = mapping.node(node.value(source_ast));
                    let kind = node.kind(source_ast);
                    let method = node.method(source_ast);
                    let shorthand = node.shorthand(source_ast);
                    let computed = node.computed(source_ast);
                    let copied = self
                        .object_property(DUMMY_SP, key, value, kind, method, shorthand, computed);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ParenthesizedExpression(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.parenthesized_expression(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ArrowFunctionExpression(node) => {
                    let expression = node.expression(source_ast);
                    let r#async = node.r#async(source_ast);
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = node
                        .return_type(source_ast)
                        .map(|value| mapping.node(value));
                    let body = mapping.node(node.body(source_ast));
                    let copied = self.arrow_function_expression(
                        DUMMY_SP,
                        expression,
                        r#async,
                        type_parameters,
                        params,
                        return_type,
                        body,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::SequenceExpression(node) => {
                    let expressions = {
                        let values = node
                            .expressions(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.sequence_expression(DUMMY_SP, expressions);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::MemberExpression(node) => {
                    let object = mapping.node(node.object(source_ast));
                    let property = mapping.node(node.property(source_ast));
                    let computed = node.computed(source_ast);
                    let optional = node.optional(source_ast);
                    let copied =
                        self.member_expression(DUMMY_SP, object, property, computed, optional);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::CallExpression(node) => {
                    let callee = mapping.node(node.callee(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let arguments = {
                        let values = node
                            .arguments(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let optional = node.optional(source_ast);
                    let pure = node.pure(source_ast);
                    let copied = self.call_expression(
                        DUMMY_SP,
                        callee,
                        type_arguments,
                        arguments,
                        optional,
                        pure,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ChainExpression(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.chain_expression(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TaggedTemplateExpression(node) => {
                    let tag = mapping.node(node.tag(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let quasi = mapping.node(node.quasi(source_ast));
                    let copied =
                        self.tagged_template_expression(DUMMY_SP, tag, type_arguments, quasi);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::NewExpression(node) => {
                    let callee = mapping.node(node.callee(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let arguments = {
                        let values = node
                            .arguments(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.new_expression(DUMMY_SP, callee, type_arguments, arguments);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::AwaitExpression(node) => {
                    let argument = mapping.node(node.argument(source_ast));
                    let copied = self.await_expression(DUMMY_SP, argument);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::YieldExpression(node) => {
                    let argument = node.argument(source_ast).map(|value| mapping.node(value));
                    let delegate = node.delegate(source_ast);
                    let copied = self.yield_expression(DUMMY_SP, argument, delegate);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::MetaProperty(node) => {
                    let meta = mapping.node(node.meta(source_ast));
                    let property = mapping.node(node.property(source_ast));
                    let copied = self.meta_property(DUMMY_SP, meta, property);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ImportExpression(node) => {
                    let source = mapping.node(node.source(source_ast));
                    let options = node.options(source_ast).map(|value| mapping.node(value));
                    let phase = node.phase(source_ast);
                    let sync = node.sync(source_ast);
                    let copied = self.import_expression(DUMMY_SP, source, options, phase, sync);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ImportDeclaration(node) => {
                    let specifiers = {
                        let values = node
                            .specifiers(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let source = mapping.node(node.source(source_ast));
                    let attributes = {
                        let values = node
                            .attributes(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let phase = node.phase(source_ast);
                    let import_kind = node.import_kind(source_ast);
                    let copied = self.import_declaration(
                        DUMMY_SP,
                        specifiers,
                        source,
                        attributes,
                        phase,
                        import_kind,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ImportSpecifier(node) => {
                    let imported = mapping.node(node.imported(source_ast));
                    let local = mapping.node(node.local(source_ast));
                    let import_kind = node.import_kind(source_ast);
                    let copied = self.import_specifier(DUMMY_SP, imported, local, import_kind);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ImportDefaultSpecifier(node) => {
                    let local = mapping.node(node.local(source_ast));
                    let copied = self.import_default_specifier(DUMMY_SP, local);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ImportNamespaceSpecifier(node) => {
                    let local = mapping.node(node.local(source_ast));
                    let copied = self.import_namespace_specifier(DUMMY_SP, local);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ImportAttribute(node) => {
                    let key = mapping.node(node.key(source_ast));
                    let value = mapping.node(node.value(source_ast));
                    let copied = self.import_attribute(DUMMY_SP, key, value);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ExportNamedDeclaration(node) => {
                    let declaration = node
                        .declaration(source_ast)
                        .map(|value| mapping.node(value));
                    let specifiers = {
                        let values = node
                            .specifiers(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let source = node.source(source_ast).map(|value| mapping.node(value));
                    let attributes = {
                        let values = node
                            .attributes(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let export_kind = node.export_kind(source_ast);
                    let copied = self.export_named_declaration(
                        DUMMY_SP,
                        declaration,
                        specifiers,
                        source,
                        attributes,
                        export_kind,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ExportDefaultDeclaration(node) => {
                    let declaration = mapping.node(node.declaration(source_ast));
                    let copied = self.export_default_declaration(DUMMY_SP, declaration);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ExportAllDeclaration(node) => {
                    let exported = node.exported(source_ast).map(|value| mapping.node(value));
                    let source = mapping.node(node.source(source_ast));
                    let attributes = {
                        let values = node
                            .attributes(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let export_kind = node.export_kind(source_ast);
                    let copied = self.export_all_declaration(
                        DUMMY_SP,
                        exported,
                        source,
                        attributes,
                        export_kind,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::ExportSpecifier(node) => {
                    let local = mapping.node(node.local(source_ast));
                    let exported = mapping.node(node.exported(source_ast));
                    let export_kind = node.export_kind(source_ast);
                    let copied = self.export_specifier(DUMMY_SP, local, exported, export_kind);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::Hashbang(node) => {
                    let value = self.add_utf8(source_ast.get_utf8(node.value(source_ast)));
                    let copied = self.hashbang(DUMMY_SP, value);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsFunction(node) => {
                    let id = node.id(source_ast).map(|value| mapping.node(value));
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let this_param = node.this_param(source_ast).map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = node
                        .return_type(source_ast)
                        .map(|value| mapping.node(value));
                    let generator = node.generator(source_ast);
                    let r#async = node.r#async(source_ast);
                    let declare = node.declare(source_ast);
                    let copied = self.ts_function(
                        DUMMY_SP,
                        id,
                        type_parameters,
                        this_param,
                        params,
                        return_type,
                        generator,
                        r#async,
                        declare,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsMethodDefinition(node) => {
                    let decorators = {
                        let values = node
                            .decorators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let key = mapping.node(node.key(source_ast));
                    let value = mapping.node(node.value(source_ast));
                    let kind = node.kind(source_ast);
                    let computed = node.computed(source_ast);
                    let r#static = node.r#static(source_ast);
                    let r#override = node.r#override(source_ast);
                    let optional = node.optional(source_ast);
                    let r#abstract = node.r#abstract(source_ast);
                    let accessibility = node.accessibility(source_ast);
                    let copied = self.ts_method_definition(
                        DUMMY_SP,
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
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeAnnotation(node) => {
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied = self.ts_type_annotation(DUMMY_SP, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsAnyKeyword(node) => {
                    let copied = self.ts_any_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsUnknownKeyword(node) => {
                    let copied = self.ts_unknown_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsNeverKeyword(node) => {
                    let copied = self.ts_never_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsVoidKeyword(node) => {
                    let copied = self.ts_void_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsNullKeyword(node) => {
                    let copied = self.ts_null_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsUndefinedKeyword(node) => {
                    let copied = self.ts_undefined_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsStringKeyword(node) => {
                    let copied = self.ts_string_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsNumberKeyword(node) => {
                    let copied = self.ts_number_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsBigIntKeyword(node) => {
                    let copied = self.ts_big_int_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsBooleanKeyword(node) => {
                    let copied = self.ts_boolean_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsSymbolKeyword(node) => {
                    let copied = self.ts_symbol_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsObjectKeyword(node) => {
                    let copied = self.ts_object_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsIntrinsicKeyword(node) => {
                    let copied = self.ts_intrinsic_keyword(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsThisType(node) => {
                    let copied = self.ts_this_type(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeReference(node) => {
                    let type_name = mapping.node(node.type_name(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let copied = self.ts_type_reference(DUMMY_SP, type_name, type_arguments);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsQualifiedName(node) => {
                    let left = mapping.node(node.left(source_ast));
                    let right = mapping.node(node.right(source_ast));
                    let copied = self.ts_qualified_name(DUMMY_SP, left, right);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeQuery(node) => {
                    let expr_name = mapping.node(node.expr_name(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let copied = self.ts_type_query(DUMMY_SP, expr_name, type_arguments);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsImportType(node) => {
                    let source = mapping.node(node.source(source_ast));
                    let options = node.options(source_ast).map(|value| mapping.node(value));
                    let qualifier = node.qualifier(source_ast).map(|value| mapping.node(value));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let copied =
                        self.ts_import_type(DUMMY_SP, source, options, qualifier, type_arguments);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeParameter(node) => {
                    let name = mapping.node(node.name(source_ast));
                    let constraint = node.constraint(source_ast).map(|value| mapping.node(value));
                    let default = node.default(source_ast).map(|value| mapping.node(value));
                    let r#in = node.r#in(source_ast);
                    let out = node.out(source_ast);
                    let r#const = node.r#const(source_ast);
                    let copied = self
                        .ts_type_parameter(DUMMY_SP, name, constraint, default, r#in, out, r#const);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeParameterDeclaration(node) => {
                    let params = {
                        let values = node
                            .params(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_type_parameter_declaration(DUMMY_SP, params);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeParameterInstantiation(node) => {
                    let params = {
                        let values = node
                            .params(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_type_parameter_instantiation(DUMMY_SP, params);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsLiteralType(node) => {
                    let literal = mapping.node(node.literal(source_ast));
                    let copied = self.ts_literal_type(DUMMY_SP, literal);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTemplateLiteralType(node) => {
                    let quasis = {
                        let values = node
                            .quasis(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let types = {
                        let values = node
                            .types(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_template_literal_type(DUMMY_SP, quasis, types);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsArrayType(node) => {
                    let element_type = mapping.node(node.element_type(source_ast));
                    let copied = self.ts_array_type(DUMMY_SP, element_type);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsIndexedAccessType(node) => {
                    let object_type = mapping.node(node.object_type(source_ast));
                    let index_type = mapping.node(node.index_type(source_ast));
                    let copied = self.ts_indexed_access_type(DUMMY_SP, object_type, index_type);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTupleType(node) => {
                    let element_types = {
                        let values = node
                            .element_types(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_tuple_type(DUMMY_SP, element_types);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsNamedTupleMember(node) => {
                    let label = mapping.node(node.label(source_ast));
                    let element_type = mapping.node(node.element_type(source_ast));
                    let optional = node.optional(source_ast);
                    let copied =
                        self.ts_named_tuple_member(DUMMY_SP, label, element_type, optional);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsOptionalType(node) => {
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied = self.ts_optional_type(DUMMY_SP, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsRestType(node) => {
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied = self.ts_rest_type(DUMMY_SP, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsJsDocNullableType(node) => {
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let postfix = node.postfix(source_ast);
                    let copied = self.ts_js_doc_nullable_type(DUMMY_SP, type_annotation, postfix);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsJsDocNonNullableType(node) => {
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let postfix = node.postfix(source_ast);
                    let copied =
                        self.ts_js_doc_non_nullable_type(DUMMY_SP, type_annotation, postfix);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsJsDocUnknownType(node) => {
                    let copied = self.ts_js_doc_unknown_type(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsUnionType(node) => {
                    let types = {
                        let values = node
                            .types(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_union_type(DUMMY_SP, types);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsIntersectionType(node) => {
                    let types = {
                        let values = node
                            .types(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_intersection_type(DUMMY_SP, types);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsConditionalType(node) => {
                    let check_type = mapping.node(node.check_type(source_ast));
                    let extends_type = mapping.node(node.extends_type(source_ast));
                    let true_type = mapping.node(node.true_type(source_ast));
                    let false_type = mapping.node(node.false_type(source_ast));
                    let copied = self.ts_conditional_type(
                        DUMMY_SP,
                        check_type,
                        extends_type,
                        true_type,
                        false_type,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsInferType(node) => {
                    let type_parameter = mapping.node(node.type_parameter(source_ast));
                    let copied = self.ts_infer_type(DUMMY_SP, type_parameter);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeOperator(node) => {
                    let operator = node.operator(source_ast);
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied = self.ts_type_operator(DUMMY_SP, operator, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsParenthesizedType(node) => {
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied = self.ts_parenthesized_type(DUMMY_SP, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsFunctionType(node) => {
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let this_param = node.this_param(source_ast).map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = mapping.node(node.return_type(source_ast));
                    let copied = self.ts_function_type(
                        DUMMY_SP,
                        type_parameters,
                        this_param,
                        params,
                        return_type,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsConstructorType(node) => {
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = mapping.node(node.return_type(source_ast));
                    let r#abstract = node.r#abstract(source_ast);
                    let copied = self.ts_constructor_type(
                        DUMMY_SP,
                        type_parameters,
                        params,
                        return_type,
                        r#abstract,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypePredicate(node) => {
                    let parameter_name = mapping.node(node.parameter_name(source_ast));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let asserts = node.asserts(source_ast);
                    let copied =
                        self.ts_type_predicate(DUMMY_SP, parameter_name, type_annotation, asserts);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeLiteral(node) => {
                    let members = {
                        let values = node
                            .members(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_type_literal(DUMMY_SP, members);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsMappedType(node) => {
                    let key = mapping.node(node.key(source_ast));
                    let constraint = mapping.node(node.constraint(source_ast));
                    let name_type = node.name_type(source_ast).map(|value| mapping.node(value));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let optional = node.optional(source_ast);
                    let readonly = node.readonly(source_ast);
                    let copied = self.ts_mapped_type(
                        DUMMY_SP,
                        key,
                        constraint,
                        name_type,
                        type_annotation,
                        optional,
                        readonly,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsPropertySignature(node) => {
                    let key = mapping.node(node.key(source_ast));
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let computed = node.computed(source_ast);
                    let optional = node.optional(source_ast);
                    let readonly = node.readonly(source_ast);
                    let copied = self.ts_property_signature(
                        DUMMY_SP,
                        key,
                        type_annotation,
                        computed,
                        optional,
                        readonly,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsMethodSignature(node) => {
                    let key = mapping.node(node.key(source_ast));
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let this_param = node.this_param(source_ast).map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = node
                        .return_type(source_ast)
                        .map(|value| mapping.node(value));
                    let kind = node.kind(source_ast);
                    let computed = node.computed(source_ast);
                    let optional = node.optional(source_ast);
                    let copied = self.ts_method_signature(
                        DUMMY_SP,
                        key,
                        type_parameters,
                        this_param,
                        params,
                        return_type,
                        kind,
                        computed,
                        optional,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsCallSignatureDeclaration(node) => {
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let this_param = node.this_param(source_ast).map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = node
                        .return_type(source_ast)
                        .map(|value| mapping.node(value));
                    let copied = self.ts_call_signature_declaration(
                        DUMMY_SP,
                        type_parameters,
                        this_param,
                        params,
                        return_type,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsConstructSignatureDeclaration(node) => {
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let params = mapping.node(node.params(source_ast));
                    let return_type = node
                        .return_type(source_ast)
                        .map(|value| mapping.node(value));
                    let copied = self.ts_construct_signature_declaration(
                        DUMMY_SP,
                        type_parameters,
                        params,
                        return_type,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsIndexSignature(node) => {
                    let parameters = {
                        let values = node
                            .parameters(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let readonly = node.readonly(source_ast);
                    let r#static = node.r#static(source_ast);
                    let copied = self.ts_index_signature(
                        DUMMY_SP,
                        parameters,
                        type_annotation,
                        readonly,
                        r#static,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeAliasDeclaration(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let declare = node.declare(source_ast);
                    let copied = self.ts_type_alias_declaration(
                        DUMMY_SP,
                        id,
                        type_parameters,
                        type_annotation,
                        declare,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsInterfaceDeclaration(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let type_parameters = node
                        .type_parameters(source_ast)
                        .map(|value| mapping.node(value));
                    let extends = {
                        let values = node
                            .extends(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let body = mapping.node(node.body(source_ast));
                    let declare = node.declare(source_ast);
                    let copied = self.ts_interface_declaration(
                        DUMMY_SP,
                        id,
                        type_parameters,
                        extends,
                        body,
                        declare,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsInterfaceBody(node) => {
                    let body = {
                        let values = node
                            .body(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_interface_body(DUMMY_SP, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsInterfaceHeritage(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let copied = self.ts_interface_heritage(DUMMY_SP, expression, type_arguments);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsClassImplements(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let copied = self.ts_class_implements(DUMMY_SP, expression, type_arguments);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsEnumDeclaration(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let body = mapping.node(node.body(source_ast));
                    let is_const = node.is_const(source_ast);
                    let declare = node.declare(source_ast);
                    let copied = self.ts_enum_declaration(DUMMY_SP, id, body, is_const, declare);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsEnumBody(node) => {
                    let members = {
                        let values = node
                            .members(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_enum_body(DUMMY_SP, members);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsEnumMember(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let initializer = node
                        .initializer(source_ast)
                        .map(|value| mapping.node(value));
                    let computed = node.computed(source_ast);
                    let copied = self.ts_enum_member(DUMMY_SP, id, initializer, computed);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsModuleDeclaration(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let attributes = node.attributes(source_ast).map(|value| mapping.node(value));
                    let body = node.body(source_ast).map(|value| mapping.node(value));
                    let kind = node.kind(source_ast);
                    let declare = node.declare(source_ast);
                    let copied =
                        self.ts_module_declaration(DUMMY_SP, id, attributes, body, kind, declare);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsModuleBlock(node) => {
                    let directives = {
                        let values = node
                            .directives(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let body = {
                        let values = node
                            .body(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let copied = self.ts_module_block(DUMMY_SP, directives, body);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsGlobalDeclaration(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let body = mapping.node(node.body(source_ast));
                    let declare = node.declare(source_ast);
                    let copied = self.ts_global_declaration(DUMMY_SP, id, body, declare);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsParameterProperty(node) => {
                    let decorators = {
                        let values = node
                            .decorators(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let parameter = mapping.node(node.parameter(source_ast));
                    let r#override = node.r#override(source_ast);
                    let readonly = node.readonly(source_ast);
                    let accessibility = node.accessibility(source_ast);
                    let copied = self.ts_parameter_property(
                        DUMMY_SP,
                        decorators,
                        parameter,
                        r#override,
                        readonly,
                        accessibility,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsThisParameter(node) => {
                    let type_annotation = node
                        .type_annotation(source_ast)
                        .map(|value| mapping.node(value));
                    let copied = self.ts_this_parameter(DUMMY_SP, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsAsExpression(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied = self.ts_as_expression(DUMMY_SP, expression, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsSatisfiesExpression(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied =
                        self.ts_satisfies_expression(DUMMY_SP, expression, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsTypeAssertion(node) => {
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.ts_type_assertion(DUMMY_SP, type_annotation, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsNonNullExpression(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.ts_non_null_expression(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsInstantiationExpression(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let type_arguments = mapping.node(node.type_arguments(source_ast));
                    let copied =
                        self.ts_instantiation_expression(DUMMY_SP, expression, type_arguments);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsExportAssignment(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.ts_export_assignment(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsNamespaceExportDeclaration(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let copied = self.ts_namespace_export_declaration(DUMMY_SP, id);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsImportEqualsDeclaration(node) => {
                    let id = mapping.node(node.id(source_ast));
                    let module_reference = mapping.node(node.module_reference(source_ast));
                    let import_kind = node.import_kind(source_ast);
                    let copied = self.ts_import_equals_declaration(
                        DUMMY_SP,
                        id,
                        module_reference,
                        import_kind,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsExternalModuleReference(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.ts_external_module_reference(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::TsIndexSignatureParameter(node) => {
                    let name = mapping.node(node.name(source_ast));
                    let type_annotation = mapping.node(node.type_annotation(source_ast));
                    let copied = self.ts_index_signature_parameter(DUMMY_SP, name, type_annotation);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxElement(node) => {
                    let opening_element = mapping.node(node.opening_element(source_ast));
                    let children = {
                        let values = node
                            .children(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let closing_element = node
                        .closing_element(source_ast)
                        .map(|value| mapping.node(value));
                    let copied =
                        self.jsx_element(DUMMY_SP, opening_element, children, closing_element);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxOpeningElement(node) => {
                    let name = mapping.node(node.name(source_ast));
                    let type_arguments = node
                        .type_arguments(source_ast)
                        .map(|value| mapping.node(value));
                    let attributes = {
                        let values = node
                            .attributes(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let self_closing = node.self_closing(source_ast);
                    let copied = self.jsx_opening_element(
                        DUMMY_SP,
                        name,
                        type_arguments,
                        attributes,
                        self_closing,
                    );
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxClosingElement(node) => {
                    let name = mapping.node(node.name(source_ast));
                    let copied = self.jsx_closing_element(DUMMY_SP, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxFragment(node) => {
                    let opening_fragment = mapping.node(node.opening_fragment(source_ast));
                    let children = {
                        let values = node
                            .children(source_ast)
                            .iter()
                            .map(|slot| mapping.node(source_ast.get_node_in_sub_range(slot)))
                            .collect::<Vec<_>>();
                        self.add_sub_range(values)
                    };
                    let closing_fragment = mapping.node(node.closing_fragment(source_ast));
                    let copied =
                        self.jsx_fragment(DUMMY_SP, opening_fragment, children, closing_fragment);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxOpeningFragment(node) => {
                    let copied = self.jsx_opening_fragment(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxClosingFragment(node) => {
                    let copied = self.jsx_closing_fragment(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxIdentifier(node) => {
                    let name = self.add_utf8(source_ast.get_utf8(node.name(source_ast)));
                    let copied = self.jsx_identifier(DUMMY_SP, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxNamespacedName(node) => {
                    let namespace = mapping.node(node.namespace(source_ast));
                    let name = mapping.node(node.name(source_ast));
                    let copied = self.jsx_namespaced_name(DUMMY_SP, namespace, name);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxMemberExpression(node) => {
                    let object = mapping.node(node.object(source_ast));
                    let property = mapping.node(node.property(source_ast));
                    let copied = self.jsx_member_expression(DUMMY_SP, object, property);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxAttribute(node) => {
                    let name = mapping.node(node.name(source_ast));
                    let value = node.value(source_ast).map(|value| mapping.node(value));
                    let copied = self.jsx_attribute(DUMMY_SP, name, value);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxSpreadAttribute(node) => {
                    let argument = mapping.node(node.argument(source_ast));
                    let copied = self.jsx_spread_attribute(DUMMY_SP, argument);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxExpressionContainer(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.jsx_expression_container(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxEmptyExpression(node) => {
                    let copied = self.jsx_empty_expression(DUMMY_SP);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxText(node) => {
                    let value = self.add_utf8(source_ast.get_utf8(node.value(source_ast)));
                    let copied = self.jsx_text(DUMMY_SP, value);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
                NodeKindData::JsxSpreadChild(node) => {
                    let expression = mapping.node(node.expression(source_ast));
                    let copied = self.jsx_spread_child(DUMMY_SP, expression);
                    unsafe {
                        self.replace_data(mapping.node(node), copied);
                    }
                }
            }
        }
    }
}
