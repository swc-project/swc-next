//! Overload implementations omitted from the declaration surface.

use std::mem::take;

use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    Ast, Class, ClassElementData, Decl, DeclData, ExportDefaultDeclarationKindData,
    MethodDefinitionKind, Stmt, StmtData, SubRange, SymbolId,
};
use swc_next_ecma_semantic::Semantic;

use super::{DeclarationSurface, property_identity};

#[derive(Debug, Clone, Copy)]
enum FunctionOverloadInfo {
    Signature(SymbolId),
    Implementation(SymbolId),
    DefaultSignature,
    DefaultImplementation,
}

impl DeclarationSurface {
    pub(crate) fn record_statement_overloads(
        &mut self,
        statements: SubRange<Stmt>,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        let mut overloads = FxHashSet::default();
        let mut has_default_overload = false;
        for statement in statements.iter() {
            let statement = ast.get_node_in_sub_range(statement);
            let hidden = match self.function_overload_info(statement, ast, semantic) {
                Some(FunctionOverloadInfo::Signature(symbol)) => {
                    overloads.insert(symbol);
                    false
                }
                Some(FunctionOverloadInfo::Implementation(symbol)) => overloads.contains(&symbol),
                Some(FunctionOverloadInfo::DefaultSignature) => {
                    has_default_overload = true;
                    false
                }
                Some(FunctionOverloadInfo::DefaultImplementation) => {
                    take(&mut has_default_overload)
                }
                _ => false,
            };
            if hidden {
                self.hidden_implementations.insert(statement.node_id());
            }
        }
    }

    pub(crate) fn record_class_overloads(
        &mut self,
        class: Class,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        let mut overloads = FxHashSet::default();
        let mut constructor_overload = false;
        let elements = class.body(ast).body(ast);
        for element in elements.iter() {
            let element = ast.get_node_in_sub_range(element);
            let ClassElementData::TsMethodDefinition(method) = ast.class_element_data(element)
            else {
                continue;
            };
            match method.kind(ast) {
                MethodDefinitionKind::Constructor => constructor_overload = true,
                MethodDefinitionKind::Method
                    if !method.r#abstract(ast) && !method.optional(ast) =>
                {
                    if let Some(key) = property_identity(method.key(ast), ast, semantic) {
                        overloads.insert((key, method.r#static(ast)));
                    }
                }
                _ => {}
            }
        }
        if overloads.is_empty() && !constructor_overload {
            return;
        }
        for element in elements.iter() {
            let element = ast.get_node_in_sub_range(element);
            let ClassElementData::MethodDefinition(method) = ast.class_element_data(element) else {
                continue;
            };
            let hidden = match method.kind(ast) {
                MethodDefinitionKind::Constructor => constructor_overload,
                MethodDefinitionKind::Method => property_identity(method.key(ast), ast, semantic)
                    .is_some_and(|key| overloads.contains(&(key, method.r#static(ast)))),
                _ => false,
            };
            if hidden {
                self.hidden_implementations.insert(method.node_id());
            }
        }
    }

    fn function_overload_info(
        &self,
        statement: Stmt,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) -> Option<FunctionOverloadInfo> {
        match ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => {
                Self::decl_overload_info(declaration, ast, semantic)
            }
            StmtData::ExportNamedDeclaration(export) => export
                .declaration(ast)
                .and_then(|declaration| Self::decl_overload_info(declaration, ast, semantic)),
            StmtData::ExportDefaultDeclaration(export) => {
                match ast.export_default_declaration_kind_data(export.declaration(ast)) {
                    ExportDefaultDeclarationKindData::TsFunction(_) => {
                        Some(FunctionOverloadInfo::DefaultSignature)
                    }
                    ExportDefaultDeclarationKindData::Function(_) => {
                        Some(FunctionOverloadInfo::DefaultImplementation)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn decl_overload_info(
        declaration: Decl,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) -> Option<FunctionOverloadInfo> {
        let (binding, signature) = match ast.decl_data(declaration) {
            DeclData::TsFunction(function) => (function.id(ast)?, true),
            DeclData::Function(function) => (function.id(ast)?, false),
            _ => return None,
        };
        let symbol = semantic.symbol_of(binding.node_id())?;
        if signature {
            Some(FunctionOverloadInfo::Signature(symbol))
        } else {
            Some(FunctionOverloadInfo::Implementation(symbol))
        }
    }
}
