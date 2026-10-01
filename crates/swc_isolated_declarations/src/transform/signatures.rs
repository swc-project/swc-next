//! Function signatures, parameter defaults, and explicit undefined types.

use swc_next_ecma_ast::{
    Ast, BindingPattern, BindingPatternData, CloneIn, Diagnostic, FormalParameter,
    FormalParameterItem, FormalParameterItemData, FormalParameters, Function, GetSpan, Span,
    SubRange, TsFunction, TsMethodSignature, TsThisParameter, TsType, TsTypeAnnotation, TsTypeData,
};

use self::UndefinedPresence::{Absent, Any, Never, Present, UnknownKeyword, Unresolved, Void};
use super::Transformer;
use super::diagnostics::{
    FUNCTION_RETURN_TYPE_REQUIRED, METHOD_RETURN_TYPE_REQUIRED, PARAMETER_TYPE_REQUIRED,
};

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn transform_ts_function_signature(
        &mut self,
        function: TsFunction,
        ast: &mut Ast<'ast>,
    ) {
        if function.r#async(ast) {
            function.set_async(ast, false);
        }
        if function.return_type(ast).is_none() {
            self.diagnostics.push(Diagnostic::error(
                FUNCTION_RETURN_TYPE_REQUIRED,
                function.span(ast),
            ));
        }
        self.report_untyped_this_parameter(function.this_param(ast), ast);
        self.transform_formal_parameters(function.params(ast), ast);
    }

    pub(super) fn validate_type_method_signature(
        &mut self,
        signature: TsMethodSignature,
        ast: &mut Ast<'ast>,
    ) {
        self.report_untyped_this_parameter(signature.this_param(ast), ast);
        self.transform_formal_parameters_with_diagnostics(signature.params(ast), true, ast);
        if signature.return_type(ast).is_none() {
            self.diagnostics.push(Diagnostic::error(
                METHOD_RETURN_TYPE_REQUIRED,
                signature.span(ast),
            ));
        }
    }

    pub(super) fn transform_function_signature(
        &mut self,
        function: Function,
        ast: &mut Ast<'ast>,
    ) -> Option<TsTypeAnnotation> {
        let mut return_type = function.return_type(ast);
        if return_type.is_none() && !function.r#async(ast) && !function.generator(ast) {
            return_type = self.infer_function_return_type(function, ast);
        }
        if return_type.is_none() {
            self.diagnostics.push(Diagnostic::error(
                FUNCTION_RETURN_TYPE_REQUIRED,
                function.span(ast),
            ));
        }
        self.report_untyped_this_parameter(function.this_param(ast), ast);
        self.transform_formal_parameters(function.params(ast), ast);
        return_type
    }

    pub(super) fn transform_method_signature(
        &mut self,
        function: Function,
        diagnostic_span: Span,
        ast: &mut Ast<'ast>,
    ) -> Option<TsTypeAnnotation> {
        let mut return_type = function.return_type(ast);
        if return_type.is_none() && !function.r#async(ast) && !function.generator(ast) {
            return_type = self.infer_function_return_type(function, ast);
        }
        if return_type.is_none() {
            self.diagnostics.push(Diagnostic::error(
                METHOD_RETURN_TYPE_REQUIRED,
                diagnostic_span,
            ));
        }
        self.report_untyped_this_parameter(function.this_param(ast), ast);
        self.transform_formal_parameters(function.params(ast), ast);
        return_type
    }

    pub(super) fn report_untyped_this_parameter(
        &mut self,
        this_parameter: Option<TsThisParameter>,
        ast: &Ast<'ast>,
    ) {
        if let Some(this_parameter) = this_parameter
            && this_parameter.type_annotation(ast).is_none()
        {
            self.diagnostics.push(Diagnostic::error(
                PARAMETER_TYPE_REQUIRED,
                this_parameter.span(ast),
            ));
        }
    }

    pub(super) fn transform_formal_parameters(
        &mut self,
        params: FormalParameters,
        ast: &mut Ast<'ast>,
    ) {
        self.transform_formal_parameters_with_diagnostics(params, true, ast);
    }

    pub(super) fn transform_formal_parameters_with_diagnostics(
        &mut self,
        params: FormalParameters,
        report_missing_types: bool,
        ast: &mut Ast<'ast>,
    ) {
        let items = params.items(ast);
        let mut transformed = Vec::with_capacity(items.len());
        let mut required_after = false;

        for item in items.iter().rev() {
            let item = ast.get_node_in_sub_range(item);
            let parameter = match ast.formal_parameter_item_data(item) {
                FormalParameterItemData::FormalParameter(parameter) => parameter,
                FormalParameterItemData::TsParameterProperty(property) => property.parameter(ast),
            };
            let required = self.formal_parameter_is_required(parameter, ast);
            if !self.transform_formal_parameter_with_diagnostics(
                parameter,
                required_after,
                report_missing_types,
                ast,
            ) {
                continue;
            }
            required_after |= required;
            transformed.push(FormalParameterItem::FormalParameter(parameter));
        }
        transformed.reverse();

        if let Some(rest) = params.rest(ast) {
            rest.set_decorators(ast, SubRange::empty());
            if report_missing_types && rest.type_annotation(ast).is_none() {
                self.diagnostics
                    .push(Diagnostic::error(PARAMETER_TYPE_REQUIRED, rest.span(ast)));
            }
            self.strip_binding_defaults(rest.argument(ast).into(), ast);
        }

        let items = ast.add_sub_range(transformed);
        params.set_items(ast, items);
    }

    pub(super) fn transform_formal_parameter_with_diagnostics(
        &mut self,
        parameter: FormalParameter,
        required_after: bool,
        report_missing_type: bool,
        ast: &mut Ast<'ast>,
    ) -> bool {
        let pattern = parameter.pattern(ast);
        let initializer = parameter.initializer(ast);
        let has_initializer = initializer.is_some();
        let destructuring = !pattern.is_binding_identifier(ast);
        if has_initializer && destructuring && parameter.type_annotation(ast).is_none() {
            if report_missing_type {
                self.diagnostics.push(Diagnostic::error(
                    PARAMETER_TYPE_REQUIRED,
                    parameter.span(ast),
                ));
            }
            return false;
        }

        self.strip_binding_defaults(pattern.into(), ast);
        parameter.set_decorators(ast, SubRange::empty());

        if let Some(initializer) = initializer {
            if parameter.type_annotation(ast).is_none() {
                if let Some(inferred) = self.infer_type_from_expr(initializer, ast) {
                    let annotation = ast.ts_type_annotation(parameter.span(ast), inferred);
                    parameter.set_type_annotation(ast, Some(annotation));
                } else if report_missing_type {
                    self.diagnostics.push(Diagnostic::error(
                        PARAMETER_TYPE_REQUIRED,
                        parameter.span(ast),
                    ));
                }
            }

            if required_after {
                self.add_undefined_to_parameter(parameter, ast);
            } else {
                parameter.set_optional(ast, true);
            }
            parameter.set_initializer(ast, None);
        } else if report_missing_type && parameter.type_annotation(ast).is_none() {
            self.diagnostics.push(Diagnostic::error(
                PARAMETER_TYPE_REQUIRED,
                parameter.span(ast),
            ));
        }

        true
    }

    pub(super) fn formal_parameter_is_required(
        &self,
        parameter: FormalParameter,
        ast: &Ast<'ast>,
    ) -> bool {
        !parameter.optional(ast) && parameter.initializer(ast).is_none()
    }

    pub(super) fn first_parameter_annotation(
        &self,
        function: Function,
        ast: &Ast<'ast>,
    ) -> Option<TsTypeAnnotation> {
        self.first_parameter_annotation_in_params(function.params(ast), ast)
    }

    pub(super) fn first_parameter_annotation_in_params(
        &self,
        params: FormalParameters,
        ast: &Ast<'ast>,
    ) -> Option<TsTypeAnnotation> {
        let item = ast.get_node_in_sub_range(params.items(ast).first()?);
        match ast.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => parameter.type_annotation(ast),
            FormalParameterItemData::TsParameterProperty(property) => {
                property.parameter(ast).type_annotation(ast)
            }
        }
    }

    pub(super) fn set_first_parameter_annotation(
        &self,
        function: Function,
        annotation: &TsTypeAnnotation,
        ast: &mut Ast<'ast>,
    ) {
        self.set_first_parameter_annotation_in_params(function.params(ast), annotation, ast);
    }

    pub(super) fn set_first_parameter_annotation_in_params(
        &self,
        params: FormalParameters,
        annotation: &TsTypeAnnotation,
        ast: &mut Ast<'ast>,
    ) {
        let Some(item) = params.items(ast).first() else {
            return;
        };
        let item = ast.get_node_in_sub_range(item);
        let parameter = match ast.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => parameter,
            FormalParameterItemData::TsParameterProperty(property) => property.parameter(ast),
        };
        let annotation = annotation.clone_in(ast);
        parameter.set_type_annotation(ast, Some(annotation));
    }

    fn add_undefined_to_parameter(&mut self, parameter: FormalParameter, ast: &mut Ast<'ast>) {
        let Some(annotation) = parameter.type_annotation(ast) else {
            return;
        };
        let ty = annotation.type_annotation(ast);
        if self.has_explicit_undefined_union_member(ty, ast) {
            return;
        }
        // Without a type checker, unresolved aliases and indexed accesses may already
        // include undefined. Adding a redundant union is safe and preserves defaults
        // before required parameters without rejecting valid TS7 declarations.
        if matches!(self.undefined_presence(ty, ast), UndefinedPresence::Present) {
            return;
        }
        let span = parameter.span(ast);
        let undefined = ast.ts_type_ts_undefined_keyword(span);
        let types = ast.add_sub_range([ty, undefined]);
        let union = ast.ts_type_ts_union_type(span, types);
        let annotation = ast.ts_type_annotation(span, union);
        parameter.set_type_annotation(ast, Some(annotation));
    }

    pub(super) fn add_explicit_undefined_to_parameter(
        &self,
        parameter: FormalParameter,
        ast: &mut Ast<'ast>,
    ) {
        let Some(annotation) = parameter.type_annotation(ast) else {
            return;
        };
        let ty = annotation.type_annotation(ast);
        if self.type_maybe_undefined(ty, ast) {
            return;
        }
        let span = parameter.span(ast);
        let undefined = ast.ts_type_ts_undefined_keyword(span);
        let types = ast.add_sub_range([ty, undefined]);
        let union = ast.ts_type_ts_union_type(span, types);
        let annotation = ast.ts_type_annotation(span, union);
        parameter.set_type_annotation(ast, Some(annotation));
    }

    fn type_maybe_undefined(&self, ty: TsType, ast: &Ast<'ast>) -> bool {
        match ast.ts_type_data(ty) {
            TsTypeData::TsAnyKeyword(_)
            | TsTypeData::TsUnknownKeyword(_)
            | TsTypeData::TsUndefinedKeyword(_) => true,
            TsTypeData::TsUnionType(union) => union
                .types(ast)
                .iter()
                .map(|ty| ast.get_node_in_sub_range(ty))
                .any(|ty| self.type_maybe_undefined(ty, ast)),
            _ => false,
        }
    }

    fn undefined_presence(&self, ty: TsType, ast: &Ast<'ast>) -> UndefinedPresence {
        match ast.ts_type_data(ty) {
            TsTypeData::TsUndefinedKeyword(_) => UndefinedPresence::Present,
            TsTypeData::TsAnyKeyword(_) => UndefinedPresence::Any,
            TsTypeData::TsUnknownKeyword(_) => UndefinedPresence::UnknownKeyword,
            TsTypeData::TsNeverKeyword(_) => UndefinedPresence::Never,
            TsTypeData::TsVoidKeyword(_) => UndefinedPresence::Void,
            TsTypeData::TsBigIntKeyword(_)
            | TsTypeData::TsBooleanKeyword(_)
            | TsTypeData::TsIntrinsicKeyword(_)
            | TsTypeData::TsNullKeyword(_)
            | TsTypeData::TsNumberKeyword(_)
            | TsTypeData::TsObjectKeyword(_)
            | TsTypeData::TsStringKeyword(_)
            | TsTypeData::TsSymbolKeyword(_)
            | TsTypeData::TsLiteralType(_)
            | TsTypeData::TsFunctionType(_)
            | TsTypeData::TsConstructorType(_)
            | TsTypeData::TsArrayType(_)
            | TsTypeData::TsTupleType(_)
            | TsTypeData::TsMappedType(_)
            | TsTypeData::TsTypeLiteral(_)
            | TsTypeData::TsTypeOperator(_)
            | TsTypeData::TsTemplateLiteralType(_)
            | TsTypeData::TsThisType(_) => UndefinedPresence::Absent,
            TsTypeData::TsParenthesizedType(parenthesized) => {
                self.undefined_presence(parenthesized.type_annotation(ast), ast)
            }
            TsTypeData::TsUnionType(union) => union
                .types(ast)
                .iter()
                .map(|ty| self.undefined_presence(ast.get_node_in_sub_range(ty), ast))
                .fold(UndefinedPresence::Never, UndefinedPresence::union),
            TsTypeData::TsIntersectionType(intersection) => intersection
                .types(ast)
                .iter()
                .map(|ty| self.undefined_presence(ast.get_node_in_sub_range(ty), ast))
                .fold(
                    UndefinedPresence::UnknownKeyword,
                    UndefinedPresence::intersection,
                ),
            TsTypeData::TsConditionalType(_)
            | TsTypeData::TsImportType(_)
            | TsTypeData::TsIndexedAccessType(_)
            | TsTypeData::TsInferType(_)
            | TsTypeData::TsTypePredicate(_)
            | TsTypeData::TsTypeQuery(_)
            | TsTypeData::TsTypeReference(_)
            | TsTypeData::TsOptionalType(_)
            | TsTypeData::TsRestType(_)
            | TsTypeData::TsJsDocNullableType(_)
            | TsTypeData::TsJsDocNonNullableType(_)
            | TsTypeData::TsJsDocUnknownType(_) => UndefinedPresence::Unresolved,
        }
    }

    fn has_explicit_undefined_union_member(&self, ty: TsType, ast: &Ast<'ast>) -> bool {
        match ast.ts_type_data(ty) {
            TsTypeData::TsUndefinedKeyword(_) => true,
            TsTypeData::TsParenthesizedType(parenthesized) => {
                self.has_explicit_undefined_union_member(parenthesized.type_annotation(ast), ast)
            }
            TsTypeData::TsUnionType(union) => union
                .types(ast)
                .iter()
                .map(|ty| ast.get_node_in_sub_range(ty))
                .any(|ty| self.has_explicit_undefined_union_member(ty, ast)),
            _ => false,
        }
    }

    fn strip_binding_defaults(
        &self,
        pattern: BindingPattern,
        ast: &mut Ast<'ast>,
    ) -> BindingPattern {
        match ast.binding_pattern_data(pattern) {
            BindingPatternData::AssignmentPattern(assignment) => {
                self.strip_binding_defaults(assignment.left(ast), ast)
            }
            BindingPatternData::BindingRestElement(rest) => {
                let argument = self.strip_binding_defaults(rest.argument(ast), ast);
                rest.set_argument(ast, argument);
                pattern
            }
            BindingPatternData::ArrayPattern(array) => {
                let elements = array
                    .elements(ast)
                    .iter()
                    .map(|element| {
                        ast.get_node_in_sub_range(element)
                            .map(|element| self.strip_binding_defaults(element, ast))
                    })
                    .collect::<Vec<_>>();
                let elements = ast.add_sub_range(elements);
                array.set_elements(ast, elements);
                if let Some(rest) = array.rest(ast) {
                    let argument = self.strip_binding_defaults(rest.argument(ast), ast);
                    rest.set_argument(ast, argument);
                }
                pattern
            }
            BindingPatternData::ObjectPattern(object) => {
                let properties = object.properties(ast);
                for property in properties.iter() {
                    let property = ast.get_node_in_sub_range(property);
                    let value = self.strip_binding_defaults(property.value(ast), ast);
                    property.set_value(ast, value);
                }
                if let Some(rest) = object.rest(ast) {
                    let argument = self.strip_binding_defaults(rest.argument(ast), ast);
                    rest.set_argument(ast, argument);
                }
                pattern
            }
            _ => pattern,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UndefinedPresence {
    Present,
    Absent,
    Any,
    Never,
    UnknownKeyword,
    Void,
    Unresolved,
}

impl UndefinedPresence {
    fn union(self, other: Self) -> Self {
        match (self, other) {
            (Unresolved, _) | (_, Unresolved) => Unresolved,
            (Any, _) | (_, Any) => Any,
            (UnknownKeyword, _) | (_, UnknownKeyword) => UnknownKeyword,
            (Present, _) | (_, Present) => Present,
            (Void, _) | (_, Void) => Void,
            (Absent, _) | (_, Absent) => Absent,
            (Never, Never) => Never,
        }
    }

    fn intersection(self, other: Self) -> Self {
        match (self, other) {
            (Never, _) | (_, Never) => Never,
            (Unresolved | Any, _) | (_, Unresolved | Any) => Unresolved,
            (Absent, _) | (_, Absent) => Absent,
            (Present, _) | (_, Present) => Present,
            (Void, _) | (_, Void) => Void,
            (UnknownKeyword, UnknownKeyword) => UnknownKeyword,
        }
    }
}
