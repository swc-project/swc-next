//! Decisions shared by dependency collection and declaration emit.
//!
//! `expressions` classifies inferred types and direct returns; `keys` identifies
//! property names; `accessors` pairs members and selects their type sources.
//! `DeclarationSurface` carries annotations and hidden overload implementations
//! discovered during dependency collection into emit.

mod accessors;
mod expressions;
mod keys;
mod overloads;

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, FormalParameter, FormalParameterPatternData, NodeId, SourceType,
    SymbolId, TsModuleDeclaration, TsModuleName, TsModuleNameData, TsQualifiedNameLeftData, TsType,
};
use swc_next_ecma_semantic::Semantic;

pub(crate) use accessors::{
    AccessorKey, AccessorPair, AccessorType, class_accessors, object_accessors,
};
pub(crate) use expressions::{
    ExportExpression, InferredType, export_expression, first_template_quasi, inferred_type,
    return_expression, unparenthesized,
};
pub(crate) use keys::{
    property_identity, signed_numeric_value, supported_computed_key, supported_object_key,
};

#[derive(Clone, Copy)]
struct ParameterType {
    annotation: TsType,
    optional: bool,
}

#[derive(Default)]
pub(crate) struct DeclarationSurface {
    annotations: FxHashMap<SymbolId, ParameterType>,
    pub(crate) hidden_implementations: FxHashSet<NodeId>,
}
impl DeclarationSurface {
    pub(crate) fn record_annotation(
        &mut self,
        parameter: FormalParameter,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        if let FormalParameterPatternData::BindingIdentifier(binding) =
            ast.formal_parameter_pattern_data(parameter.pattern(ast))
            && let Some(annotation) = parameter.type_annotation(ast)
            && let Some(symbol) = semantic.symbol_of(binding.node_id())
        {
            self.annotations.insert(
                symbol,
                ParameterType {
                    annotation: annotation.type_annotation(ast),
                    optional: parameter.optional(ast) && parameter.initializer(ast).is_none(),
                },
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DeclContext {
    pub(crate) declare: bool,
    pub(crate) retain_all: bool,
    pub(crate) ambient: bool,
}

impl DeclContext {
    pub(crate) fn for_program(source_type: SourceType) -> Self {
        let retain_all = matches!(source_type, SourceType::Script | SourceType::Unambiguous);
        Self {
            retain_all,
            ..Self::TOP_LEVEL
        }
    }

    pub(crate) const TOP_LEVEL: Self = Self {
        declare: true,
        retain_all: false,
        ambient: false,
    };

    pub(crate) const NAMESPACE: Self = Self {
        declare: false,
        retain_all: false,
        ambient: false,
    };

    pub(crate) const AMBIENT_MODULE: Self = Self {
        declare: false,
        retain_all: true,
        ambient: true,
    };
    pub(crate) fn namespace(self, module: TsModuleDeclaration, ast: &Ast<'_>) -> Self {
        if self.ambient
            || module.declare(ast)
            || matches!(
                ast.ts_module_name_data(module.id(ast)),
                TsModuleNameData::StringLiteral(_)
            )
        {
            Self::AMBIENT_MODULE
        } else {
            Self::NAMESPACE
        }
    }
}

/// The lexical binding that owns a namespace, including dotted declarations.
pub(crate) fn namespace_binding(name: TsModuleName, ast: &Ast<'_>) -> Option<BindingIdentifier> {
    let mut qualified = match ast.ts_module_name_data(name) {
        TsModuleNameData::BindingIdentifier(binding) => return Some(binding),
        TsModuleNameData::StringLiteral(_) => return None,
        TsModuleNameData::TsQualifiedName(qualified) => qualified,
    };
    loop {
        match ast.ts_qualified_name_left_data(qualified.left(ast)) {
            TsQualifiedNameLeftData::BindingIdentifier(binding) => return Some(binding),
            TsQualifiedNameLeftData::TsQualifiedName(left) => qualified = left,
            _ => return None,
        }
    }
}
