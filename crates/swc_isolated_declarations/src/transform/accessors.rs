//! Resolve accessor types and apply them to emitted class and type signatures.

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    Ast, ClassElement, CloneIn, Diagnostic, MethodDefinitionKind, PropertyKey, Span, SubRange,
    TsMethodSignatureKind, TsTypeAnnotation, TsTypeElement, TsTypeElementData,
};

use super::Transformer;
use super::diagnostics::ACCESSOR_TYPE_REQUIRED;
use crate::surface::{AccessorKey, AccessorPair, AccessorType, class_accessors, property_identity};

#[derive(Debug, Clone, Copy)]
pub(super) struct AccessorAnnotations {
    pub(super) span: Span,
    pub(super) has_getter: bool,
    pub(super) has_setter: bool,
    pub(super) getter_explicit: Option<TsTypeAnnotation>,
    pub(super) setter_explicit: Option<TsTypeAnnotation>,
    pub(super) getter_inferred: Option<TsTypeAnnotation>,
}

impl AccessorAnnotations {
    pub(super) fn separate_signatures(&self) -> bool {
        self.getter_explicit.is_some() && self.setter_explicit.is_some()
    }
}

#[derive(Default)]
struct AccessorSignatureAnnotations {
    getter: Option<TsTypeAnnotation>,
    setter: Option<TsTypeAnnotation>,
}

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn collect_accessor_annotations(
        &mut self,
        elements: SubRange<ClassElement>,
        ast: &mut Ast<'ast>,
    ) -> FxHashMap<AccessorKey<'ast>, AccessorAnnotations> {
        let pairs = class_accessors(
            elements
                .iter()
                .map(|element| ast.get_node_in_sub_range(element)),
            ast,
            self.semantic,
        );
        let annotations: FxHashMap<_, _> = pairs
            .into_iter()
            .map(|(key, pair)| (key, self.resolve_accessor_pair(pair, ast)))
            .collect();
        for entry in annotations.values() {
            if entry.getter_explicit.is_none()
                && entry.setter_explicit.is_none()
                && entry.getter_inferred.is_none()
            {
                self.diagnostics
                    .push(Diagnostic::error(ACCESSOR_TYPE_REQUIRED, entry.span));
            }
        }
        annotations
    }

    pub(super) fn resolve_accessor_pair(
        &mut self,
        pair: AccessorPair,
        ast: &mut Ast<'ast>,
    ) -> AccessorAnnotations {
        let getter_explicit = pair.getter_annotation(ast);
        let setter_explicit = pair.setter_annotation(ast);
        let getter_inferred = match pair.read_type(ast) {
            Some(AccessorType::Inferred(function)) => {
                self.infer_function_return_type(function, ast)
            }
            _ => None,
        };
        AccessorAnnotations {
            span: pair.span,
            has_getter: pair.getter.is_some(),
            has_setter: pair.setter.is_some(),
            getter_explicit,
            setter_explicit,
            getter_inferred,
        }
    }

    pub(super) fn accessor_annotation<'map>(
        &self,
        property: PropertyKey,
        is_static: bool,
        kind: MethodDefinitionKind,
        annotations: &'map FxHashMap<AccessorKey<'ast>, AccessorAnnotations>,
        ast: &Ast<'ast>,
    ) -> Option<&'map TsTypeAnnotation> {
        if !matches!(kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set) {
            return None;
        }
        let key = self.accessor_key(property, is_static, ast)?;
        let entry = annotations.get(&key)?;
        match kind {
            MethodDefinitionKind::Get => entry
                .getter_explicit
                .as_ref()
                .or(entry.setter_explicit.as_ref())
                .or(entry.getter_inferred.as_ref()),
            MethodDefinitionKind::Set => entry
                .setter_explicit
                .as_ref()
                .or(entry.getter_explicit.as_ref())
                .or(entry.getter_inferred.as_ref()),
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => None,
        }
    }

    pub(super) fn transform_type_accessor_elements(
        &self,
        elements: SubRange<TsTypeElement>,
        ast: &mut Ast<'ast>,
    ) {
        let mut annotations = FxHashMap::default();

        for element in elements.iter() {
            let element = ast.get_node_in_sub_range(element);
            let TsTypeElementData::TsMethodSignature(signature) = ast.ts_type_element_data(element)
            else {
                continue;
            };
            if signature.kind(ast) == TsMethodSignatureKind::Method {
                continue;
            }
            let Some(key) = self.accessor_key(signature.key(ast), false, ast) else {
                continue;
            };
            let entry: &mut AccessorSignatureAnnotations = annotations.entry(key).or_default();
            match signature.kind(ast) {
                TsMethodSignatureKind::Get => {
                    entry.getter = signature.return_type(ast);
                }
                TsMethodSignatureKind::Set => {
                    entry.setter =
                        self.first_parameter_annotation_in_params(signature.params(ast), ast);
                }
                TsMethodSignatureKind::Method => {}
            }
        }

        if annotations.is_empty() {
            return;
        }

        for element in elements.iter() {
            let element = ast.get_node_in_sub_range(element);
            let TsTypeElementData::TsMethodSignature(signature) = ast.ts_type_element_data(element)
            else {
                continue;
            };
            if signature.kind(ast) == TsMethodSignatureKind::Method {
                continue;
            }
            let Some(key) = self.accessor_key(signature.key(ast), false, ast) else {
                continue;
            };
            let Some(entry) = annotations.get(&key) else {
                continue;
            };
            match signature.kind(ast) {
                TsMethodSignatureKind::Get if signature.return_type(ast).is_none() => {
                    if let Some(annotation) = entry.setter {
                        let annotation = annotation.clone_in(ast);
                        signature.set_return_type(ast, Some(annotation));
                    }
                }
                TsMethodSignatureKind::Set
                    if self
                        .first_parameter_annotation_in_params(signature.params(ast), ast)
                        .is_none() =>
                {
                    if let Some(annotation) = entry.getter {
                        self.set_first_parameter_annotation_in_params(
                            signature.params(ast),
                            &annotation,
                            ast,
                        );
                    }
                }
                TsMethodSignatureKind::Method
                | TsMethodSignatureKind::Get
                | TsMethodSignatureKind::Set => {}
            }
        }
    }

    pub(super) fn accessor_key(
        &self,
        key: PropertyKey,
        r#static: bool,
        ast: &Ast<'ast>,
    ) -> Option<AccessorKey<'ast>> {
        let name = property_identity(key, ast, self.semantic)?;
        Some(AccessorKey { name, r#static })
    }
}
