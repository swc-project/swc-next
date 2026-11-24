use std::collections::{HashMap, HashSet};
use swc_next_ecma_ast::*;

use super::analyzer::scope::Scope;
use crate::common::resolver::context::Resolver;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L537>.
pub(super) fn analyzer_and_collect_unresolved(
    ast: &Ast<'_>,
    resolver: &Resolver,
) -> HashMap<SymbolId, String> {
    let mut analyzer = AnalyzerAndCollector {
        ast,
        resolver,
        scope: Scope {
            function: true,
            ..Default::default()
        },
        hoisted: Vec::new(),
        ids: HashSet::new(),
        declarations: HashSet::new(),
        pattern: false,
        hoist: false,
    };
    ast.root_program().visit_with(&mut analyzer);
    let unresolved = analyzer
        .ids
        .difference(&analyzer.declarations)
        .map(|&id| ast.get_utf8(resolver.symbol_name(id)))
        .collect();
    analyzer.scope.prepare_renaming();
    let mut renames = HashMap::new();
    analyzer.scope.rename_in_normal_mode(
        ast,
        resolver,
        &unresolved,
        &mut HashMap::new(),
        &mut renames,
    );
    renames.retain(|id, name| ast.get_utf8(resolver.symbol_name(*id)) != *name);
    renames
}

struct AnalyzerAndCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    resolver: &'visit Resolver,
    scope: Scope,
    hoisted: Vec<SymbolId>,
    ids: HashSet<SymbolId>,
    declarations: HashSet<SymbolId>,
    pattern: bool,
    hoist: bool,
}

impl AnalyzerAndCollector<'_, '_> {
    /// SWC scope entry: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L89>.
    /// SWC hoisted declarations: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L117>.
    fn with_scope(&mut self, function: bool, op: impl FnOnce(&mut Self)) {
        let parent = std::mem::replace(
            &mut self.scope,
            Scope {
                function,
                ..Default::default()
            },
        );
        let hoisted = std::mem::take(&mut self.hoisted);
        let flags = (self.pattern, self.hoist);
        self.hoist = false;
        op(self);
        let mut child = std::mem::replace(&mut self.scope, parent);
        let child_hoisted = std::mem::replace(&mut self.hoisted, hoisted);
        (self.pattern, self.hoist) = flags;
        for id in child_hoisted {
            child.add_usage(id);
            self.add_decl(id, true);
        }
        self.scope.children.push(child);
    }

    /// SWC declaration queue: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L52>.
    /// SWC var hoisting: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L53>.
    fn add_decl(&mut self, id: SymbolId, hoist: bool) {
        if hoist && !self.scope.function {
            self.hoisted.push(id);
        } else {
            self.scope.add_decl(id);
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L128>.
    fn binding(&mut self, binding: BindingIdentifier, hoist: bool) {
        let Some(id) = self.resolver.symbol(binding.node_id()) else {
            return;
        };
        self.ids.insert(id);
        self.declarations.insert(id);
        if self.ast.get_utf8(binding.name(self.ast)) != "arguments" {
            self.add_decl(id, hoist);
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L183>.
    fn usage(&mut self, node: NodeId, name: AstUtf8) {
        if let Some(id) = self.resolver.symbol(node) {
            self.ids.insert(id);
            if self.ast.get_utf8(name) != "arguments" {
                self.scope.add_usage(id);
            }
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L382>.
    fn function_contents(&mut self, function: Function) {
        function.params(self.ast).visit_with(self);
        function.body(self.ast).visit_with(self);
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L261>.
    fn has_rest_or_default(&self, parameters: FormalParameters) -> bool {
        parameters.rest(self.ast).is_some()
            || parameters.items(self.ast).iter().any(|id| {
                let parameter = self.ast.get_node_in_sub_range(id);
                match self.ast.formal_parameter_item_data(parameter) {
                    FormalParameterItemData::FormalParameter(parameter) => {
                        parameter.initializer(self.ast).is_some()
                    }
                    FormalParameterItemData::TsParameterProperty(property) => {
                        property.parameter(self.ast).initializer(self.ast).is_some()
                    }
                }
            })
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L368>.
    fn loop_body(&mut self, statement: Stmt) {
        self.with_scope(false, |child| {
            if let Some(block) = statement.as_block_statement(child.ast) {
                block.body(child.ast).visit_with(child);
            } else {
                statement.visit_with(child);
            }
        });
    }
}

impl<'a> Visit<'a> for AnalyzerAndCollector<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L183>.
    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        self.usage(node.node_id(), node.name(self.ast));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L128>.
    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        if self.pattern {
            self.binding(node, self.hoist);
        } else {
            self.usage(node.node_id(), node.name(self.ast));
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L246>.
    fn visit_expr(&mut self, node: Expr) {
        let pattern = std::mem::replace(&mut self.pattern, false);
        node.visit_children_with(self);
        self.pattern = pattern;
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L183>.
    fn visit_numeric_literal(&mut self, node: NumericLiteral) {
        if !node.value(self.ast).is_finite()
            && let Some(raw) = node.raw(self.ast)
        {
            let raw = self.ast.get_utf8(raw);
            if matches!(raw, "NaN" | "Infinity") {
                self.scope.reserved.insert(raw.into());
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L113>.
    fn visit_assignment_target(&mut self, node: AssignmentTarget) {
        let pattern = std::mem::replace(&mut self.pattern, false);
        node.visit_children_with(self);
        self.pattern = pattern;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L461>.
    fn visit_variable_declaration(&mut self, node: VariableDeclaration) {
        let hoist = std::mem::replace(&mut self.hoist, node.kind(self.ast) == VariableKind::Var);
        node.visit_children_with(self);
        self.hoist = hoist;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L470>.
    fn visit_variable_declarator(&mut self, node: VariableDeclarator) {
        let pattern = std::mem::replace(&mut self.pattern, true);
        node.id(self.ast).visit_with(self);
        self.pattern = false;
        node.init(self.ast).visit_with(self);
        self.pattern = pattern;
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L413>.
    fn visit_formal_parameter(&mut self, node: FormalParameter) {
        node.decorators(self.ast).visit_with(self);
        let flags = (self.pattern, self.hoist);
        self.pattern = true;
        self.hoist = true;
        node.pattern(self.ast).visit_with(self);
        node.initializer(self.ast).visit_with(self);
        (self.pattern, self.hoist) = flags;
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L413>.
    fn visit_formal_parameter_rest(&mut self, node: FormalParameterRest) {
        node.decorators(self.ast).visit_with(self);
        let flags = (self.pattern, self.hoist);
        self.pattern = true;
        self.hoist = true;
        node.argument(self.ast).visit_with(self);
        (self.pattern, self.hoist) = flags;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L261>.
    fn visit_function(&mut self, node: Function) {
        if node.function_type(self.ast) == FunctionType::FunctionDeclaration {
            let binding = node.id(self.ast);
            let has_rest = self.has_rest_or_default(node.params(self.ast));
            if let Some(binding) = binding {
                self.binding(binding, true);
                if has_rest {
                    self.usage(binding.node_id(), binding.name(self.ast));
                }
            }
            self.with_scope(true, |child| {
                if has_rest && let Some(binding) = binding {
                    child.usage(binding.node_id(), binding.name(child.ast));
                }
                child.function_contents(node);
            });
        } else if let Some(binding) = node.id(self.ast) {
            self.with_scope(true, |child| {
                child.binding(binding, true);
                child.with_scope(true, |child| {
                    if child.has_rest_or_default(node.params(child.ast)) {
                        child.usage(binding.node_id(), binding.name(child.ast));
                    }
                    child.function_contents(node);
                });
            });
        } else {
            self.with_scope(true, |child| child.function_contents(node));
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L86>.
    fn visit_arrow_function_expression(&mut self, node: ArrowFunctionExpression) {
        self.with_scope(true, |child| {
            node.params(child.ast).visit_with(child);
            let pattern = std::mem::replace(&mut child.pattern, false);
            node.body(child.ast).visit_with(child);
            child.pattern = pattern;
        });
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L135>.
    fn visit_function_body(&mut self, node: FunctionBody) {
        self.with_scope(false, |child| node.body(child.ast).visit_with(child));
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L135>.
    fn visit_block_statement(&mut self, node: BlockStatement) {
        self.with_scope(false, |child| node.body(child.ast).visit_with(child));
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L439>.
    fn visit_static_block(&mut self, node: StaticBlock) {
        self.with_scope(true, |child| node.body(child.ast).visit_with(child));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L147>.
    fn visit_catch_clause(&mut self, node: CatchClause) {
        self.with_scope(false, |child| {
            let flags = (child.pattern, child.hoist);
            child.pattern = false;
            node.body(child.ast).body(child.ast).visit_with(child);
            child.pattern = true;
            child.hoist = false;
            node.param(child.ast).visit_with(child);
            (child.pattern, child.hoist) = flags;
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L173>.
    fn visit_class(&mut self, node: Class) {
        if node.class_type(self.ast) == ClassType::ClassDeclaration {
            if let Some(binding) = node.id(self.ast) {
                self.binding(binding, false);
            }
            node.decorators(self.ast).visit_with(self);
            node.super_class(self.ast).visit_with(self);
            node.body(self.ast).visit_with(self);
        } else {
            self.with_scope(false, |child| {
                if let Some(binding) = node.id(child.ast) {
                    child.binding(binding, false);
                }
                node.decorators(child.ast).visit_with(child);
                node.super_class(child.ast).visit_with(child);
                node.body(child.ast).visit_with(child);
            });
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L193>.
    fn visit_method_definition(&mut self, node: MethodDefinition) {
        if node.kind(self.ast) != MethodDefinitionKind::Constructor {
            node.key(self.ast).visit_with(self);
        }
        self.with_scope(true, |child| {
            if node.kind(child.ast) == MethodDefinitionKind::Constructor {
                node.key(child.ast).visit_with(child);
            }
            node.decorators(child.ast).visit_with(child);
            child.function_contents(node.value(child.ast));
        });
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L434>.
    fn visit_object_property(&mut self, node: ObjectProperty) {
        node.key(self.ast).visit_with(self);
        node.value(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L368>.
    fn visit_for_statement(&mut self, node: ForStatement) {
        self.with_scope(false, |child| {
            node.init(child.ast).visit_with(child);
            node.test(child.ast).visit_with(child);
            node.update(child.ast).visit_with(child);
            child.loop_body(node.body(child.ast));
        });
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L342>.
    fn visit_for_in_statement(&mut self, node: ForInStatement) {
        self.with_scope(false, |child| {
            node.left(child.ast).visit_with(child);
            node.right(child.ast).visit_with(child);
            child.loop_body(node.body(child.ast));
        });
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L355>.
    fn visit_for_of_statement(&mut self, node: ForOfStatement) {
        self.with_scope(false, |child| {
            node.left(child.ast).visit_with(child);
            node.right(child.ast).visit_with(child);
            child.loop_body(node.body(child.ast));
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L399>.
    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        self.binding(node.local(self.ast), true);
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L392>.
    fn visit_import_default_specifier(&mut self, node: ImportDefaultSpecifier) {
        self.binding(node.local(self.ast), true);
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L406>.
    fn visit_import_namespace_specifier(&mut self, node: ImportNamespaceSpecifier) {
        self.binding(node.local(self.ast), true);
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L523>.
    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        if node.source(self.ast).is_none() {
            node.declaration(self.ast).visit_with(self);
            node.specifiers(self.ast).visit_with(self);
        }
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L235>.
    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        if let Some(local) = node.local(self.ast).as_identifier_name(self.ast) {
            self.usage(local.node_id(), local.name(self.ast));
        }
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L214>.
    fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        match self
            .ast
            .export_default_declaration_kind_data(node.declaration(self.ast))
        {
            ExportDefaultDeclarationKindData::Class(class) => {
                if let Some(binding) = class.id(self.ast) {
                    self.binding(binding, false);
                }
                self.with_scope(true, |child| {
                    class.decorators(child.ast).visit_with(child);
                    class.super_class(child.ast).visit_with(child);
                    class.body(child.ast).visit_with(child);
                });
            }
            ExportDefaultDeclarationKindData::Function(function) => {
                if let Some(binding) = function.id(self.ast) {
                    self.binding(binding, true);
                }
                function.visit_with(self);
            }
            ExportDefaultDeclarationKindData::Expr(expression) => expression.visit_with(self),
            _ => {}
        }
    }
    /// JSX identifiers reach SWC's identifier collector: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L519>.
    fn visit_jsx_element_name(&mut self, node: JsxElementName) {
        match self.ast.jsx_element_name_data(node) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                self.ids.extend(self.resolver.symbol(identifier.node_id()));
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self)
            }
            JsxElementNameData::JsxNamespacedName(_) => {}
        }
    }

    /// JSX object roots reach SWC's identifier collector: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L519>.
    fn visit_jsx_member_expression_object(&mut self, node: JsxMemberExpressionObject) {
        match self.ast.jsx_member_expression_object_data(node) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                self.ids.extend(self.resolver.symbol(identifier.node_id()));
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self)
            }
        }
    }

    /// SWC analyzer skips TypeScript types: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L83>.
    fn visit_ts_type(&mut self, _: TsType) {}
}
