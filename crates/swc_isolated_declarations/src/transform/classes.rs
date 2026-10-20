//! Class members, private declarations, and constructor parameter properties.

use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    Accessibility, Ast, Class, ClassElement, ClassElementData, CloneIn, Diagnostic, Expr, ExprData,
    FormalParameterItemData, FormalParameters, GetSpan, MethodDefinitionKind, PropertyDefinition,
    PropertyKey, Span, SubRange, TsParameterProperty, TsTypeAnnotation,
};

use super::diagnostics::{
    COMPUTED_PROPERTY_NOT_SUPPORTED, EXTENDS_EXPRESSION_NOT_SUPPORTED, PROPERTY_TYPE_REQUIRED,
};
use super::{DeclContext, Transformer};
use crate::surface::{property_identity, unparenthesized};

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn transform_class(
        &mut self,
        class: Class,
        context: DeclContext,
        ast: &mut Ast<'ast>,
    ) {
        class.set_declare(ast, context.declare);
        class.set_decorators(ast, SubRange::empty());
        if let Some(super_class) = class.super_class(ast)
            && !matches!(
                ast.expr_data(unparenthesized(super_class, ast)),
                ExprData::NullLiteral(_)
            )
            && !self.is_heritage_expression(super_class, ast)
        {
            self.diagnostics.push(Diagnostic::error(
                EXTENDS_EXPRESSION_NOT_SUPPORTED,
                super_class.span(ast),
            ));
        }
        let body = class.body(ast);
        let elements = body.body(ast);
        let accessor_annotations = self.collect_accessor_annotations(elements, ast);
        let mut transformed = Vec::with_capacity(elements.len());
        let mut parameter_properties = Vec::new();
        let mut parameter_property_insert = None;
        let mut has_private_key = false;
        let mut private_methods = FxHashSet::default();

        for element in elements.iter() {
            let element = ast.get_node_in_sub_range(element);
            match ast.class_element_data(element) {
                ClassElementData::MethodDefinition(method) => {
                    let hidden = self
                        .surface
                        .hidden_implementations
                        .contains(&method.node_id());
                    if hidden && method.kind(ast) != MethodDefinitionKind::Constructor {
                        continue;
                    }
                    if Self::property_key_is_private(method.key(ast), ast) {
                        has_private_key = true;
                        continue;
                    }
                    if method.computed(ast) && !self.is_supported_computed_key(method.key(ast), ast)
                    {
                        self.diagnostics.push(Diagnostic::error(
                            COMPUTED_PROPERTY_NOT_SUPPORTED,
                            method.key(ast).span(ast),
                        ));
                        continue;
                    }
                    let kind = method.kind(ast);
                    let accessibility = self.declaration_accessibility(method.accessibility(ast));
                    if accessibility == Accessibility::Private
                        && kind == MethodDefinitionKind::Method
                    {
                        transformed.push(self.private_method_property(
                            method.key(ast),
                            method.span(ast),
                            method.computed(ast),
                            method.r#static(ast),
                            ast,
                        ));
                        continue;
                    }
                    let function = method.value(ast);
                    let return_type = if accessibility == Accessibility::Private
                        && matches!(kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set)
                    {
                        function.set_this_param(ast, None);
                        self.strip_private_accessor_parameters(function.params(ast), kind, ast);
                        None
                    } else if kind == MethodDefinitionKind::Constructor {
                        if !hidden && accessibility != Accessibility::Private {
                            self.report_untyped_this_parameter(function.this_param(ast), ast);
                        }
                        let properties = self.transform_constructor_parameters(
                            function.params(ast),
                            hidden || accessibility == Accessibility::Private,
                            ast,
                        );
                        if !properties.is_empty() {
                            parameter_property_insert.get_or_insert(transformed.len());
                            parameter_properties.extend(properties);
                        }
                        if hidden {
                            continue;
                        }
                        None
                    } else if kind == MethodDefinitionKind::Set {
                        if self.first_parameter_annotation(function, ast).is_none()
                            && let Some(annotation) = self.accessor_annotation(
                                method.key(ast),
                                method.r#static(ast),
                                kind,
                                &accessor_annotations,
                                ast,
                            )
                        {
                            self.set_first_parameter_annotation(function, annotation, ast);
                        }
                        self.transform_formal_parameters_with_diagnostics(
                            function.params(ast),
                            false,
                            ast,
                        );
                        self.report_untyped_this_parameter(function.this_param(ast), ast);
                        None
                    } else if accessibility == Accessibility::Private {
                        None
                    } else if kind == MethodDefinitionKind::Get {
                        let mut return_type = function.return_type(ast);
                        if return_type.is_none()
                            && let Some(annotation) = self.accessor_annotation(
                                method.key(ast),
                                method.r#static(ast),
                                kind,
                                &accessor_annotations,
                                ast,
                            )
                        {
                            return_type = Some(annotation.clone_in(ast));
                        }
                        if return_type.is_none() {
                            return_type = self.infer_function_return_type(function, ast);
                        }
                        self.report_untyped_this_parameter(function.this_param(ast), ast);
                        self.transform_formal_parameters(function.params(ast), ast);
                        return_type
                    } else {
                        self.transform_method_signature(function, method.key(ast).span(ast), ast)
                    };
                    let function = ast.ts_function(
                        function.span(ast),
                        None,
                        function.type_parameters(ast),
                        function.this_param(ast),
                        function.params(ast),
                        return_type,
                        false,
                        false,
                        false,
                    );
                    let key = self.declaration_property_key(method.key(ast), ast);
                    let method = ast.ts_method_definition(
                        method.span(ast),
                        SubRange::empty(),
                        key,
                        function,
                        kind,
                        method.computed(ast),
                        method.r#static(ast),
                        false,
                        method.optional(ast),
                        method.r#abstract(ast),
                        accessibility,
                    );
                    transformed.push(ClassElement::TsMethodDefinition(method));
                }
                ClassElementData::PropertyDefinition(property) => {
                    if Self::property_key_is_private(property.key(ast), ast) {
                        has_private_key = true;
                        continue;
                    }
                    if property.computed(ast)
                        && !self.is_supported_computed_key(property.key(ast), ast)
                    {
                        self.diagnostics.push(Diagnostic::error(
                            COMPUTED_PROPERTY_NOT_SUPPORTED,
                            property.key(ast).span(ast),
                        ));
                        continue;
                    }
                    parameter_property_insert.get_or_insert(transformed.len());
                    self.transform_property(property, ast);
                    transformed.push(element);
                }
                ClassElementData::TsMethodDefinition(method) => {
                    method.set_override(ast, false);
                    if Self::property_key_is_private(method.key(ast), ast) {
                        has_private_key = true;
                        continue;
                    }
                    if method.computed(ast) && !self.is_supported_computed_key(method.key(ast), ast)
                    {
                        self.diagnostics.push(Diagnostic::error(
                            COMPUTED_PROPERTY_NOT_SUPPORTED,
                            method.key(ast).span(ast),
                        ));
                        continue;
                    }
                    if method.accessibility(ast) == Accessibility::Private
                        && method.kind(ast) == MethodDefinitionKind::Method
                    {
                        if let Some(identity) =
                            property_identity(method.key(ast), ast, self.semantic)
                            && !private_methods.insert((identity, method.r#static(ast)))
                        {
                            continue;
                        }
                        transformed.push(self.private_method_property(
                            method.key(ast),
                            method.span(ast),
                            method.computed(ast),
                            method.r#static(ast),
                            ast,
                        ));
                        continue;
                    }
                    if method.accessibility(ast) == Accessibility::Public {
                        method.set_accessibility(ast, Accessibility::None);
                    }
                    let function = method.value(ast);
                    if function.r#async(ast) {
                        function.set_async(ast, false);
                    }
                    if method.accessibility(ast) == Accessibility::Private
                        && matches!(
                            method.kind(ast),
                            MethodDefinitionKind::Get | MethodDefinitionKind::Set
                        )
                    {
                        function.set_this_param(ast, None);
                        function.set_return_type(ast, None);
                        self.strip_private_accessor_parameters(
                            function.params(ast),
                            method.kind(ast),
                            ast,
                        );
                        transformed.push(element);
                        continue;
                    }
                    if let Some(annotation) = self
                        .accessor_annotation(
                            method.key(ast),
                            method.r#static(ast),
                            method.kind(ast),
                            &accessor_annotations,
                            ast,
                        )
                        .copied()
                    {
                        match method.kind(ast) {
                            MethodDefinitionKind::Get if function.return_type(ast).is_none() => {
                                let annotation = annotation.clone_in(ast);
                                function.set_return_type(ast, Some(annotation));
                            }
                            MethodDefinitionKind::Set
                                if self
                                    .first_parameter_annotation_in_params(function.params(ast), ast)
                                    .is_none() =>
                            {
                                self.set_first_parameter_annotation_in_params(
                                    function.params(ast),
                                    &annotation,
                                    ast,
                                );
                            }
                            _ => {}
                        }
                    }
                    if method.kind(ast) == MethodDefinitionKind::Constructor {
                        parameter_property_insert.get_or_insert(transformed.len());
                        if method.accessibility(ast) == Accessibility::Private {
                            let params = function.params(ast);
                            params.set_items(ast, SubRange::empty());
                            params.set_rest(ast, None);
                        } else {
                            self.report_untyped_this_parameter(function.this_param(ast), ast);
                            self.transform_formal_parameters(function.params(ast), ast);
                        }
                    } else {
                        self.report_untyped_this_parameter(function.this_param(ast), ast);
                        self.transform_formal_parameters(function.params(ast), ast);
                    }
                    transformed.push(element);
                }
                ClassElementData::TsIndexSignature(_) => transformed.push(element),
                ClassElementData::StaticBlock(_) => {}
            }
        }

        let insert = parameter_property_insert.unwrap_or(transformed.len());
        transformed.splice(insert..insert, parameter_properties);

        if has_private_key {
            transformed.insert(0, self.private_key_marker(ast));
        }

        let elements = ast.add_sub_range(transformed);
        body.set_body(ast, elements);
    }

    fn is_heritage_expression(&self, expression: Expr, ast: &Ast<'ast>) -> bool {
        match ast.expr_data(expression) {
            ExprData::IdentifierReference(_) => true,
            ExprData::MemberExpression(member) => {
                !member.computed(ast) && self.is_heritage_expression(member.object(ast), ast)
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.is_heritage_expression(parenthesized.expression(ast), ast)
            }
            ExprData::ChainExpression(chain) => {
                self.is_heritage_expression(chain.expression(ast), ast)
            }
            _ => false,
        }
    }

    fn transform_property(&mut self, property: PropertyDefinition, ast: &mut Ast<'ast>) {
        property.set_decorators(ast, SubRange::empty());
        property.set_declare(ast, false);
        property.set_definite(ast, false);
        property.set_override(ast, false);
        if property.accessibility(ast) == Accessibility::Public {
            property.set_accessibility(ast, Accessibility::None);
        }
        if property.accessibility(ast) == Accessibility::Private {
            property.set_type_annotation(ast, None);
            property.set_value(ast, None);
            return;
        }
        if property.type_annotation(ast).is_some() {
            property.set_value(ast, None);
            return;
        }
        let Some(value) = property.value(ast) else {
            self.diagnostics.push(Diagnostic::error(
                PROPERTY_TYPE_REQUIRED,
                property.span(ast),
            ));
            return;
        };
        let readonly = property.readonly(ast);
        if readonly {
            let normalized_value = self.normalize_literal_initializer(value, ast);
            if self.is_literal_initializer(normalized_value, ast) {
                property.set_value(ast, Some(normalized_value));
                return;
            }
        }
        let value = unparenthesized(value, ast);
        let inferred = match ast.expr_data(value) {
            ExprData::TemplateLiteral(template)
                if readonly && !template.expressions(ast).is_empty() =>
            {
                None
            }
            ExprData::TsNonNullExpression(_) => None,
            _ => self.infer_type_from_expr(value, ast),
        };
        let Some(inferred) = inferred else {
            self.diagnostics
                .push(Diagnostic::error(PROPERTY_TYPE_REQUIRED, value.span(ast)));
            property.set_value(ast, None);
            return;
        };
        let annotation = ast.ts_type_annotation(value.span(ast), inferred);
        property.set_type_annotation(ast, Some(annotation));
        property.set_value(ast, None);
    }

    fn private_method_property(
        &self,
        key: PropertyKey,
        span: Span,
        computed: bool,
        is_static: bool,
        ast: &mut Ast<'ast>,
    ) -> ClassElement {
        let property = ast.property_definition(
            span,
            SubRange::empty(),
            key,
            None,
            None,
            computed,
            is_static,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            Accessibility::Private,
        );
        ClassElement::PropertyDefinition(property)
    }

    fn private_key_marker(&self, ast: &mut Ast<'ast>) -> ClassElement {
        let span = Span::new(0, 0);
        let name = ast.add_interned_utf8("private");
        let key = ast.property_key_expr_private_identifier(span, name);
        let property = ast.property_definition(
            span,
            SubRange::empty(),
            key,
            None,
            None,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            Accessibility::None,
        );
        ClassElement::PropertyDefinition(property)
    }

    fn transform_constructor_parameters(
        &mut self,
        params: FormalParameters,
        omit_signature: bool,
        ast: &mut Ast<'ast>,
    ) -> Vec<ClassElement> {
        let items = params.items(ast);
        // Infer the shared annotation before defaults change the call signature.
        for item in items.iter() {
            let item = ast.get_node_in_sub_range(item);
            let FormalParameterItemData::TsParameterProperty(property) =
                ast.formal_parameter_item_data(item)
            else {
                continue;
            };
            let parameter = property.parameter(ast);
            if property.accessibility(ast) != Accessibility::Private
                && parameter.type_annotation(ast).is_none()
                && let Some(initializer) = parameter.initializer(ast)
            {
                let Some(ty) = self.infer_type_from_expr(initializer, ast) else {
                    continue;
                };
                let annotation = ast.ts_type_annotation(initializer.span(ast), ty);
                parameter.set_type_annotation(ast, Some(annotation));
            }
        }
        let properties = items
            .iter()
            .filter_map(|item| {
                let item = ast.get_node_in_sub_range(item);
                let FormalParameterItemData::TsParameterProperty(property) =
                    ast.formal_parameter_item_data(item)
                else {
                    return None;
                };
                let parameter = property.parameter(ast);
                Some((
                    property,
                    parameter.optional(ast),
                    parameter.type_annotation(ast),
                ))
            })
            .collect::<Vec<_>>();

        if omit_signature {
            let mut required_after = false;
            for item in items.iter().rev() {
                let item = ast.get_node_in_sub_range(item);
                match ast.formal_parameter_item_data(item) {
                    FormalParameterItemData::TsParameterProperty(property) => {
                        let parameter = property.parameter(ast);
                        let required = self.formal_parameter_is_required(parameter, ast);
                        let _ = self.transform_formal_parameter_with_diagnostics(
                            parameter,
                            required_after,
                            property.accessibility(ast) != Accessibility::Private,
                            ast,
                        );
                        required_after |= required;
                    }
                    FormalParameterItemData::FormalParameter(parameter) => {
                        required_after |= self.formal_parameter_is_required(parameter, ast);
                    }
                }
            }
            params.set_items(ast, SubRange::empty());
            params.set_rest(ast, None);
        } else {
            self.transform_formal_parameters(params, ast);
        }

        for &(property, optional, _) in &properties {
            if !optional {
                continue;
            }
            self.add_explicit_undefined_to_parameter(property.parameter(ast), ast);
        }

        properties
            .into_iter()
            .filter_map(|(property, optional, original_type)| {
                self.parameter_property_to_class_element(property, optional, original_type, ast)
            })
            .collect()
    }

    fn parameter_property_to_class_element(
        &self,
        property: TsParameterProperty,
        optional: bool,
        original_type: Option<TsTypeAnnotation>,
        ast: &mut Ast<'ast>,
    ) -> Option<ClassElement> {
        let parameter = property.parameter(ast);
        let binding = parameter.pattern(ast).as_binding_identifier(ast)?;
        let accessibility = property.accessibility(ast);
        let type_annotation = if accessibility == Accessibility::Private {
            None
        } else if optional {
            parameter.type_annotation(ast)
        } else {
            original_type
        };
        let accessibility = if accessibility == Accessibility::Public {
            Accessibility::None
        } else {
            accessibility
        };
        let key =
            PropertyKey::IdentifierName(ast.identifier_name(binding.span(ast), binding.name(ast)));
        let property = ast.property_definition(
            property.span(ast),
            SubRange::empty(),
            key,
            type_annotation,
            None,
            false,
            false,
            false,
            false,
            false,
            optional,
            false,
            property.readonly(ast),
            false,
            accessibility,
        );
        Some(ClassElement::PropertyDefinition(property))
    }

    fn strip_private_accessor_parameters(
        &self,
        params: FormalParameters,
        kind: MethodDefinitionKind,
        ast: &mut Ast<'ast>,
    ) {
        params.set_rest(ast, None);
        if kind == MethodDefinitionKind::Get {
            params.set_items(ast, SubRange::empty());
            return;
        }
        let items = params.items(ast);
        for item in items.iter() {
            let item = ast.get_node_in_sub_range(item);
            let FormalParameterItemData::FormalParameter(parameter) =
                ast.formal_parameter_item_data(item)
            else {
                continue;
            };
            parameter.set_type_annotation(ast, None);
            if kind == MethodDefinitionKind::Set
                && let Some(binding) = parameter.pattern(ast).as_binding_identifier(ast)
            {
                let name = ast.add_interned_utf8("value");
                binding.set_name(ast, name);
            }
        }
    }

    fn declaration_accessibility(&self, accessibility: Accessibility) -> Accessibility {
        if accessibility == Accessibility::Public {
            Accessibility::None
        } else {
            accessibility
        }
    }
}
