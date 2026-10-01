//! Object literal declaration inference and accessor pair consumption.

use std::collections::hash_map::Entry;

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    Ast, Diagnostic, ExprData, GetSpan, ObjectExpression, ObjectPropertyKindData, PropertyKind,
    TsMethodSignatureKind, TsType,
};

use super::Transformer;
use super::diagnostics::{
    COMPUTED_PROPERTY_NOT_SUPPORTED, EXPRESSION_TYPE_REQUIRED, OBJECT_SHORTHAND_NOT_SUPPORTED,
    OBJECT_SPREAD_NOT_SUPPORTED,
};
use crate::surface::{object_accessors, supported_object_key};

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn infer_object_type(
        &mut self,
        object: ObjectExpression,
        is_const: bool,
        ast: &mut Ast<'ast>,
    ) -> TsType {
        let span = object.span(ast);
        let properties = object.properties(ast);
        let mut accessor_annotations = object_accessors(object, ast, self.semantic)
            .into_iter()
            .map(|(key, pair)| (key, self.resolve_accessor_pair(pair, ast)))
            .collect::<FxHashMap<_, _>>();
        let mut members = Vec::with_capacity(properties.len());
        for property in properties.iter() {
            let property = ast.get_node_in_sub_range(property);
            let property = match ast.object_property_kind_data(property) {
                ObjectPropertyKindData::ObjectProperty(property) => property,
                ObjectPropertyKindData::SpreadElement(spread) => {
                    self.diagnostics.push(Diagnostic::error(
                        OBJECT_SPREAD_NOT_SUPPORTED,
                        spread.span(ast),
                    ));
                    continue;
                }
            };
            if property.shorthand(ast) {
                self.diagnostics.push(Diagnostic::error(
                    OBJECT_SHORTHAND_NOT_SUPPORTED,
                    property.span(ast),
                ));
                continue;
            }
            if property.computed(ast) && !supported_object_key(property.key(ast), ast) {
                self.diagnostics.push(Diagnostic::error(
                    COMPUTED_PROPERTY_NOT_SUPPORTED,
                    property.key(ast).span(ast),
                ));
                continue;
            }

            let value = property.value(ast);
            if matches!(property.kind(ast), PropertyKind::Get | PropertyKind::Set)
                && let Some(key) = self.accessor_key(property.key(ast), false, ast)
            {
                let Entry::Occupied(entry) = accessor_annotations.entry(key) else {
                    continue;
                };
                if entry.get().separate_signatures()
                    && let ExprData::Function(function) = ast.expr_data(value)
                {
                    let (declaration_key, computed) = self.object_declaration_key(
                        property.key(ast),
                        property.computed(ast),
                        true,
                        ast,
                    );
                    let kind = if property.kind(ast) == PropertyKind::Get {
                        TsMethodSignatureKind::Get
                    } else {
                        TsMethodSignatureKind::Set
                    };
                    self.transform_formal_parameters(function.params(ast), ast);
                    let member = ast.ts_type_element_ts_method_signature(
                        property.span(ast),
                        declaration_key,
                        function.type_parameters(ast),
                        function.this_param(ast),
                        function.params(ast),
                        function.return_type(ast),
                        kind,
                        computed,
                        false,
                    );
                    members.push(member);
                    continue;
                }
                // Consume each accessor pair once, including pairs that fail inference.
                let entry = entry.remove();
                let Some(ty) = entry
                    .getter_explicit
                    .or(entry.setter_explicit)
                    .or(entry.getter_inferred)
                    .map(|annotation| annotation.type_annotation(ast))
                else {
                    self.diagnostics
                        .push(Diagnostic::error(EXPRESSION_TYPE_REQUIRED, value.span(ast)));
                    if entry.has_getter && entry.has_setter {
                        self.diagnostics
                            .push(Diagnostic::error(EXPRESSION_TYPE_REQUIRED, entry.span));
                    }
                    continue;
                };
                let preserve_syntax =
                    property.computed(ast) && entry.has_getter && entry.has_setter;
                let (declaration_key, declaration_computed) = self.object_declaration_key(
                    property.key(ast),
                    property.computed(ast),
                    preserve_syntax,
                    ast,
                );
                let annotation = ast.ts_type_annotation(value.span(ast), ty);
                let member = ast.ts_type_element_ts_property_signature(
                    property.span(ast),
                    declaration_key,
                    Some(annotation),
                    declaration_computed,
                    false,
                    entry.has_getter && !entry.has_setter,
                );
                members.push(member);
                continue;
            }
            if property.kind(ast) == PropertyKind::Init
                && property.method(ast)
                && !is_const
                && let ExprData::Function(function) = ast.expr_data(value)
            {
                let (declaration_key, declaration_computed) = self.object_declaration_key(
                    property.key(ast),
                    property.computed(ast),
                    true,
                    ast,
                );
                let return_type =
                    self.transform_method_signature(function, property.key(ast).span(ast), ast);
                let member = ast.ts_type_element_ts_method_signature(
                    property.span(ast),
                    declaration_key,
                    function.type_parameters(ast),
                    function.this_param(ast),
                    function.params(ast),
                    return_type,
                    TsMethodSignatureKind::Method,
                    declaration_computed,
                    false,
                );
                self.validated_type_methods.insert(member.node_id());
                members.push(member);
                continue;
            }
            let inferred = match property.kind(ast) {
                PropertyKind::Init => {
                    if is_const {
                        self.infer_const_type(value, ast)
                    } else {
                        self.infer_type_from_expr(value, ast)
                    }
                }
                PropertyKind::Get => match ast.expr_data(value) {
                    ExprData::Function(function) => function
                        .return_type(ast)
                        .or_else(|| self.infer_function_return_type(function, ast))
                        .map(|annotation| annotation.type_annotation(ast)),
                    _ => None,
                },
                PropertyKind::Set => match ast.expr_data(value) {
                    ExprData::Function(function) => self
                        .first_parameter_annotation(function, ast)
                        .map(|annotation| annotation.type_annotation(ast)),
                    _ => None,
                },
            };
            let Some(inferred) = inferred else {
                self.diagnostics
                    .push(Diagnostic::error(EXPRESSION_TYPE_REQUIRED, value.span(ast)));
                continue;
            };
            let (declaration_key, declaration_computed) =
                self.object_declaration_key(property.key(ast), property.computed(ast), false, ast);
            let annotation = ast.ts_type_annotation(value.span(ast), inferred);
            let member = ast.ts_type_element_ts_property_signature(
                property.span(ast),
                declaration_key,
                Some(annotation),
                declaration_computed,
                false,
                is_const,
            );
            members.push(member);
        }
        let members = ast.add_sub_range(members);
        ast.ts_type_ts_type_literal(span, members)
    }
}
