use std::{
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::*;
use swc_next_ecma_semantic::name_resolution::{NameResolution, resolver};

use crate::{
    cases::{Case, swc_resolver::SwcResolverCase},
    runner::{Runner, parse_case_with_recovery},
    suite::TestResult,
};

pub struct ResolverRunner;

impl Runner<SwcResolverCase> for ResolverRunner {
    const NAME: &'static str = "resolver";

    fn run(case: &SwcResolverCase) -> TestResult {
        let path = case.relative_path().to_path_buf();
        let result = catch_unwind(AssertUnwindSafe(|| run_case(case)));
        match result {
            Ok(Ok(())) => TestResult::Passed {
                path,
                negative: false,
            },
            Ok(Err(error)) => TestResult::Failed {
                path,
                error,
                negative: false,
            },
            Err(_) => TestResult::Panic {
                path,
                negative: false,
            },
        }
    }
}

fn run_case(case: &SwcResolverCase) -> Result<(), String> {
    let allocator = Allocator::new();
    // The resolver oracle is generated for every program accepted by the
    // experimental parser, including programs with recoverable early errors.
    let ast = parse_case_with_recovery(&allocator, case);
    let resolution = resolver(&ast);
    let actual = resolver_snapshot(&ast, &resolution);

    if actual == case.expected() {
        Ok(())
    } else {
        Err(format!("{}: name resolution differs", case.name()))
    }
}

fn resolver_snapshot(ast: &Ast<'_>, resolution: &NameResolution<'_>) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "Top level: ScopeId({})",
        context_for_scope(resolution, resolution.top_level_scope_id())
    )
    .unwrap();
    writeln!(
        output,
        "Unresolved: ScopeId({})",
        context_for_scope(resolution, resolution.unresolved_scope_id())
    )
    .unwrap();
    ast.root_program().visit_with(&mut ResolverDisplayVisitor {
        ast,
        resolution,
        output: &mut output,
        export_from: false,
    });
    output
}

fn context_for_scope(resolution: &NameResolution<'_>, scope: ScopeId) -> u32 {
    if scope == resolution.top_level_scope_id() {
        2
    } else if scope == resolution.unresolved_scope_id() {
        1
    } else if scope == resolution.label_scope_id() {
        u32::MAX
    } else {
        scope.raw() - 1
    }
}

struct ResolverDisplayVisitor<'a, 'ast> {
    ast: &'a Ast<'ast>,
    resolution: &'a NameResolution<'ast>,
    output: &'a mut String,
    export_from: bool,
}

impl ResolverDisplayVisitor<'_, '_> {
    fn write_identifier<N: AstNode>(&mut self, node: N, name: AstUtf8) {
        let name = self.ast.get_utf8(name);
        let context = context_for_scope(self.resolution, self.resolution.node_scope(node));
        writeln!(self.output, "{name} ({name:?}) -> ScopeId({context})").unwrap();
    }

    fn write_identifier_name(&mut self, identifier: IdentifierName, context: u32) {
        let name = self.ast.get_utf8(identifier.name(self.ast));
        writeln!(self.output, "{name} ({name:?}) -> ScopeId({context})").unwrap();
    }

    fn write_module_export_name(&mut self, name: ModuleExportName, resolved: bool) {
        if let ModuleExportNameData::IdentifierName(identifier) =
            self.ast.module_export_name_data(name)
        {
            let context = if resolved {
                context_for_scope(self.resolution, self.resolution.node_scope(identifier))
            } else {
                context_for_scope(self.resolution, self.resolution.unresolved_scope_id())
            };
            self.write_identifier_name(identifier, context);
        }
    }

    fn is_shorthand<N: AstNode + GetSpan>(&self, name: ModuleExportName, local: N) -> bool {
        name.span(self.ast) == local.span(self.ast)
    }
}

impl<'ast> Visit<'ast> for ResolverDisplayVisitor<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        self.write_identifier(node, node.name(self.ast));
    }

    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        self.write_identifier(node, node.name(self.ast));
        node.visit_children_with(self);
    }

    fn visit_identifier_name(&mut self, node: IdentifierName) {
        node.visit_children_with(self);
    }

    fn visit_label_identifier(&mut self, node: LabelIdentifier) {
        self.write_identifier(node, node.name(self.ast));
    }

    fn visit_private_identifier(&mut self, node: PrivateIdentifier) {
        node.visit_children_with(self);
    }

    fn visit_jsx_identifier(&mut self, node: JsxIdentifier) {
        node.visit_children_with(self);
    }

    fn visit_module_export_name(&mut self, node: ModuleExportName) {
        self.write_module_export_name(node, true);
    }

    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        let local = node.local(self.ast);
        let imported = node.imported(self.ast);
        local.visit_with(self);
        if !self.is_shorthand(imported, local) {
            self.write_module_export_name(imported, false);
        }
    }

    fn visit_import_attribute(&mut self, _: ImportAttribute) {}

    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        node.declaration(self.ast).visit_with(self);
        let old_export_from = self.export_from;
        self.export_from = node.source(self.ast).is_some();
        node.specifiers(self.ast).visit_with(self);
        self.export_from = old_export_from;
    }

    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        if let Some(exported) = node.exported(self.ast) {
            self.write_module_export_name(exported, false);
        }
    }

    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        let local = node.local(self.ast);
        let exported = node.exported(self.ast);
        self.write_module_export_name(local, !self.export_from);
        if !self.is_shorthand(exported, local) {
            self.write_module_export_name(exported, false);
        }
    }

    fn visit_class(&mut self, node: Class) {
        node.id(self.ast).visit_with(self);
        node.decorators(self.ast).visit_with(self);
        node.body(self.ast).visit_with(self);
        node.super_class(self.ast).visit_with(self);
    }

    fn visit_method_definition(&mut self, node: MethodDefinition) {
        node.key(self.ast).visit_with(self);
        let function = node.value(self.ast);
        function.params(self.ast).visit_with(self);
        node.decorators(self.ast).visit_with(self);
        function.body(self.ast).visit_with(self);
    }

    fn visit_property_definition(&mut self, node: PropertyDefinition) {
        node.key(self.ast).visit_with(self);
        node.value(self.ast).visit_with(self);
        node.decorators(self.ast).visit_with(self);
    }

    fn visit_do_while_statement(&mut self, node: DoWhileStatement) {
        node.test(self.ast).visit_with(self);
        node.body(self.ast).visit_with(self);
    }

    fn visit_jsx_element_name(&mut self, node: JsxElementName) {
        match self.ast.jsx_element_name_data(node) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                self.write_identifier(identifier, identifier.name(self.ast));
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self);
            }
            JsxElementNameData::JsxNamespacedName(_) => {}
        }
    }

    fn visit_jsx_member_expression_object(&mut self, node: JsxMemberExpressionObject) {
        match self.ast.jsx_member_expression_object_data(node) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                self.write_identifier(identifier, identifier.name(self.ast));
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self);
            }
        }
    }
}
