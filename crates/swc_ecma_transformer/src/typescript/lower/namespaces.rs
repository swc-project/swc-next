use rustc_hash::{FxHashMap, FxHashSet};
use std::rc::Rc;

use swc_next_ecma_ast::{
    AssignmentOperator, AssignmentTarget, Ast, AstUtf8, BindingIdentifier, BindingPattern,
    BindingPatternData, BindingProperty, DUMMY_SP, Decl, DeclData, Expr, GetSpan,
    IdentifierReference, ImportOrExportKind, JsxElementName, JsxElementNameData, JsxIdentifier,
    JsxMemberExpression, JsxMemberExpressionObject, JsxMemberExpressionObjectData, PropertyKey,
    SimpleAssignmentTarget, Span, Stmt, StmtData, SubRange, SymbolId, TsImportEqualsDeclaration,
    TsModuleBody, TsModuleBodyData, TsModuleDeclaration, TsModuleName, TsModuleNameData,
    TsQualifiedName, TsQualifiedNameLeftData, VariableDeclaration, VisitMut, VisitMutWith,
};

use super::{StatementExpansion, TypeScriptLowerer, runtime::RuntimeTarget};
use swc_next_ecma_semantic::Semantic;

#[derive(Default)]
pub(super) struct NamespaceState {
    members: FxHashMap<SymbolId, Rc<NamespaceMembers>>,
    contexts: Vec<NamespaceContext>,
    export_alias_depth: usize,
}

impl NamespaceState {
    fn has_export_aliases(&self) -> bool {
        self.export_alias_depth != 0
    }

    pub(super) fn parameter_for_symbol(
        &self,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
        name: &[u8],
        symbol: SymbolId,
    ) -> Option<AstUtf8> {
        if self.has_export_aliases() {
            return self.parameter_for_export_alias(ast, semantic, name, symbol);
        }
        for context in self.contexts.iter().rev() {
            if context.members.declared.contains(&symbol) {
                return (!context.local_symbols.contains(&symbol)).then_some(context.parameter);
            }
        }
        None
    }

    // Inlining the export lookup increases stack/register work for every ordinary reference.
    #[inline(never)]
    fn parameter_for_export_alias(
        &self,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
        name: &[u8],
        symbol: SymbolId,
    ) -> Option<AstUtf8> {
        // Export specifiers can expose a declaration under a different name,
        // including when its original name has a local runtime binding.
        let same_name = name == ast.get_wtf8(semantic.symbol(symbol).name).as_bytes();
        for context in self.contexts.iter().rev() {
            let direct = same_name && context.members.declared.contains(&symbol);
            let exported = direct
                || context.members.export_alias_owner.is_some_and(|owner| {
                    semantic
                        .ts_namespace_value_targets(owner, name)
                        .any(|target| target.candidate() == Some(symbol))
                });
            if exported {
                let local = same_name && context.local_symbols.contains(&symbol);
                return (!local).then_some(context.parameter);
            }
        }
        None
    }

    fn enter(
        &mut self,
        parameter: AstUtf8,
        members: Rc<NamespaceMembers>,
        local_symbols: FxHashSet<SymbolId>,
    ) {
        self.export_alias_depth += usize::from(members.export_alias_owner.is_some());
        self.contexts.push(NamespaceContext {
            parameter,
            members,
            local_symbols,
        });
    }

    fn exit(&mut self) {
        let context = self
            .contexts
            .pop()
            .expect("a namespace lowering context exists");
        self.export_alias_depth -= usize::from(context.members.export_alias_owner.is_some());
    }
}

pub(super) struct NamespaceContext {
    pub(super) parameter: AstUtf8,
    members: Rc<NamespaceMembers>,
    local_symbols: FxHashSet<SymbolId>,
}

pub(super) struct NamespaceMembers {
    declared: FxHashSet<SymbolId>,
    export_alias_owner: Option<SymbolId>,
}

#[derive(Clone, Copy)]
struct NamespacePathPart {
    symbol: SymbolId,
    name: AstUtf8,
    span: Span,
}

struct NamespacePath {
    binding: BindingIdentifier,
    root: NamespacePathPart,
    properties: Vec<NamespacePathPart>,
}

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn lower_namespace(
        &mut self,
        declaration: TsModuleDeclaration,
        exported: bool,
    ) -> StatementExpansion {
        if !self
            .analysis
            .namespace_is_instantiated(declaration.node_id())
        {
            return StatementExpansion::Remove;
        }
        let Some(body) = declaration.body(self.ast_ref()) else {
            return StatementExpansion::Remove;
        };
        let Some(path) = self.namespace_path(declaration.id(self.ast_ref())) else {
            return StatementExpansion::Remove;
        };
        let root = path.root;

        let parent_namespace = self
            .namespaces
            .contexts
            .last()
            .map(|context| context.parameter);
        let prefix = self.register_namespace_binding(
            path.binding,
            root,
            exported && parent_namespace.is_none(),
        );
        let target = if exported && let Some(parent) = parent_namespace {
            RuntimeTarget::IdentifierAndProperty {
                identifier: root.name,
                object: parent,
                property: root.name,
            }
        } else {
            RuntimeTarget::Identifier(root.name)
        };

        let statement = self.build_namespace_layer(
            root,
            &path.properties,
            body,
            target,
            declaration.span(self.ast_ref()),
        );
        StatementExpansion::with_prefix(prefix, statement)
    }

    pub(super) fn namespace_identifier_reference(
        &mut self,
        identifier: IdentifierReference,
    ) -> Option<Expr> {
        let parameter = self.namespace_parameter_for_reference(identifier.node_id())?;
        let name = identifier.name(self.ast_ref());
        let span = identifier.span(self.ast_ref());
        Some(Expr::MemberExpression(
            self.namespace_member(parameter, name, span),
        ))
    }

    pub(super) fn namespace_identifier_target(
        &mut self,
        identifier: IdentifierReference,
    ) -> Option<SimpleAssignmentTarget> {
        let parameter = self.namespace_parameter_for_reference(identifier.node_id())?;
        let name = identifier.name(self.ast_ref());
        let span = identifier.span(self.ast_ref());
        Some(SimpleAssignmentTarget::MemberExpression(
            self.namespace_member(parameter, name, span),
        ))
    }

    pub(super) fn namespace_jsx_name(&mut self, name: JsxElementName) -> Option<JsxElementName> {
        let root = self.jsx_name_root(name)?;
        let parameter = self.namespace_parameter_for_reference(root.node_id())?;
        let span = name.span(self.ast_ref());
        match self.ast_ref().jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                let object = self
                    .ast()
                    .jsx_member_expression_object_jsx_identifier(DUMMY_SP, parameter);
                Some(
                    self.ast()
                        .jsx_element_name_jsx_member_expression(span, object, identifier),
                )
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                let object = member.object(self.ast_ref());
                let object = self.prepend_jsx_namespace(object, parameter);
                member.set_object(self.ast(), object);
                Some(name)
            }
            JsxElementNameData::JsxNamespacedName(_) => None,
        }
    }

    fn register_namespace_binding(
        &mut self,
        binding: BindingIdentifier,
        part: NamespacePathPart,
        exported: bool,
    ) -> Option<Stmt> {
        let kind = self.register_runtime_binding(part.symbol)?;
        Some(self.runtime_binding_statement(binding, part.span, kind, exported))
    }

    fn build_namespace_layer(
        &mut self,
        part: NamespacePathPart,
        properties: &[NamespacePathPart],
        body: TsModuleBody,
        target: RuntimeTarget,
        span: Span,
    ) -> Stmt {
        let parameter = self.container_parameter(part.name, part.symbol);
        let members = self
            .namespaces
            .members
            .entry(part.symbol)
            .or_insert_with(|| {
                Rc::new(NamespaceMembers {
                    declared: self.semantic.ts_namespace_members(part.symbol).collect(),
                    export_alias_owner: self
                        .semantic
                        .ts_namespace_has_export_aliases(part.symbol)
                        .then_some(part.symbol),
                })
            });
        let members = Rc::clone(members);
        // Each dotted segment is an implicit exported namespace declaration
        // in its parent's IIFE, with the same local binding as an explicit one.
        let local_symbols = if let Some(next) = properties.first() {
            FxHashSet::from_iter([next.symbol])
        } else {
            self.namespace_body_local_symbols(body)
        };
        self.namespaces
            .enter(parameter, Rc::clone(&members), local_symbols);

        self.enter_binding_scope();
        let (directives, statements) = if let Some((&next, remaining)) = properties.split_first() {
            let binding = self.ast().binding_identifier(next.span, next.name);
            let prefix = self.register_namespace_binding(binding, next, false);
            let target = RuntimeTarget::IdentifierAndProperty {
                identifier: next.name,
                object: parameter,
                property: next.name,
            };
            let statement = self.build_namespace_layer(next, remaining, body, target, span);
            let statements = self
                .ast()
                .add_sub_range(prefix.into_iter().chain([statement]));
            (SubRange::empty(), statements)
        } else {
            self.lower_namespace_body(body, parameter, &members)
        };
        self.exit_binding_scope();
        self.namespaces.exit();

        self.object_iife_statement(parameter, directives, statements, target, span)
    }

    fn lower_namespace_body(
        &mut self,
        body: TsModuleBody,
        parameter: AstUtf8,
        members: &NamespaceMembers,
    ) -> (SubRange<swc_next_ecma_ast::Directive>, SubRange<Stmt>) {
        match self.ast_ref().ts_module_body_data(body) {
            TsModuleBodyData::TsModuleBlock(block) => {
                let directives = block.directives(self.ast_ref());
                let statements = self.lower_namespace_statement_list(
                    block.body(self.ast_ref()),
                    parameter,
                    members,
                );
                (directives, statements)
            }
            TsModuleBodyData::TsModuleDeclaration(declaration) => {
                let mut statements = Vec::new();
                self.lower_namespace(declaration, true)
                    .extend_into(&mut statements);
                (SubRange::empty(), self.ast().add_sub_range(statements))
            }
        }
    }

    fn lower_namespace_statement_list(
        &mut self,
        statements: SubRange<Stmt>,
        parameter: AstUtf8,
        members: &NamespaceMembers,
    ) -> SubRange<Stmt> {
        if self.analysis.has_merged_runtime_bindings() {
            self.register_scope_runtime_bindings(statements);
        }
        let mut lowered = Vec::with_capacity(statements.len());
        for statement_id in statements.iter() {
            let statement = self.ast_ref().get_node_in_sub_range(statement_id);
            match self.ast_ref().stmt_data(statement) {
                StmtData::ExportNamedDeclaration(export) => {
                    self.lower_namespace_export(export, parameter, members, &mut lowered);
                }
                StmtData::ImportDeclaration(_)
                | StmtData::ExportDefaultDeclaration(_)
                | StmtData::ExportAllDeclaration(_)
                | StmtData::TsNamespaceExportDeclaration(_) => {}
                _ => match self.lower_statement(statement) {
                    Some(replacement) => replacement.extend_into(&mut lowered),
                    None => lowered.push(statement),
                },
            }
        }
        self.ast().add_sub_range(lowered)
    }

    fn lower_namespace_export(
        &mut self,
        export: swc_next_ecma_ast::ExportNamedDeclaration,
        parameter: AstUtf8,
        members: &NamespaceMembers,
        output: &mut Vec<Stmt>,
    ) {
        if export.export_kind(self.ast_ref()) == ImportOrExportKind::Type {
            return;
        }
        let Some(declaration) = export.declaration(self.ast_ref()) else {
            return;
        };
        match self.ast_ref().decl_data(declaration) {
            DeclData::VariableDeclaration(variable) => {
                self.lower_namespace_variable(variable, parameter, &members.declared, output);
            }
            DeclData::Function(function) => {
                let Some(binding) = function.id(self.ast_ref()) else {
                    return;
                };
                declaration.visit_mut_children_with(self);
                output.push(Stmt::Declaration(declaration));
                output.push(self.namespace_export_assignment(parameter, binding));
            }
            DeclData::Class(class) if !class.declare(self.ast_ref()) => {
                let Some(binding) = class.id(self.ast_ref()) else {
                    return;
                };
                declaration.visit_mut_children_with(self);
                output.push(Stmt::Declaration(declaration));
                output.push(self.namespace_export_assignment(parameter, binding));
            }
            DeclData::TsEnumDeclaration(enum_declaration) => {
                self.lower_namespace_enum(enum_declaration, parameter, output);
            }
            DeclData::TsModuleDeclaration(namespace) => {
                self.lower_namespace(namespace, true).extend_into(output);
            }
            DeclData::TsImportEqualsDeclaration(import_equals) => {
                self.lower_namespace_import_equals(import_equals, parameter, output);
            }
            _ => {}
        }
    }

    fn lower_namespace_variable(
        &mut self,
        declaration: VariableDeclaration,
        parameter: AstUtf8,
        property_symbols: &FxHashSet<SymbolId>,
        output: &mut Vec<Stmt>,
    ) {
        if declaration.declare(self.ast_ref()) {
            return;
        }
        let mut assignments = Vec::with_capacity(declaration.declarators(self.ast_ref()).len());
        for declarator_id in declaration.declarators(self.ast_ref()).iter() {
            let declarator = self.ast_ref().get_node_in_sub_range(declarator_id);
            let Some(initializer) = declarator.init(self.ast_ref()) else {
                continue;
            };
            let pattern = declarator.id(self.ast_ref());
            let pattern = pattern.visit_mut_with(self).unwrap_or(pattern);
            let pattern = {
                let mut rewriter = NamespaceBindingPatternRewriter {
                    ast: self.ast,
                    semantic: self.semantic,
                    property_symbols,
                    parameter,
                };
                pattern.visit_mut_with(&mut rewriter).unwrap_or(pattern)
            };
            let target = binding_pattern_to_assignment_target(pattern, self.ast_ref());
            let initializer = initializer.visit_mut_with(self).unwrap_or(initializer);
            let span = declarator.span(self.ast_ref());
            assignments.push(self.ast().expr_assignment_expression(
                span,
                target,
                initializer,
                AssignmentOperator::Assign,
            ));
        }
        if assignments.is_empty() {
            return;
        }
        let expression = if assignments.len() == 1 {
            assignments[0]
        } else {
            let expressions = self.ast().add_sub_range(assignments);
            self.ast().expr_sequence_expression(DUMMY_SP, expressions)
        };
        let span = declaration.span(self.ast_ref());
        output.push(self.ast().stmt_expression_statement(span, expression));
    }

    fn lower_namespace_import_equals(
        &mut self,
        declaration: TsImportEqualsDeclaration,
        parameter: AstUtf8,
        output: &mut Vec<Stmt>,
    ) {
        if self.import_equals_is_erased(declaration) {
            return;
        }
        let binding = declaration.id(self.ast_ref());
        let initializer = self.lower_import_equals_initializer(declaration);
        let initializer = initializer.visit_mut_with(self).unwrap_or(initializer);
        output.push(self.namespace_property_assignment_statement(
            parameter,
            binding.name(self.ast_ref()),
            initializer,
            declaration.span(self.ast_ref()),
        ));
    }

    fn namespace_export_assignment(
        &mut self,
        parameter: AstUtf8,
        binding: BindingIdentifier,
    ) -> Stmt {
        let name = binding.name(self.ast_ref());
        let span = binding.span(self.ast_ref());
        let value = self.ast().expr_identifier_reference(span, name);
        self.namespace_property_assignment_statement(parameter, name, value, DUMMY_SP)
    }

    fn namespace_property_assignment_statement(
        &mut self,
        parameter: AstUtf8,
        name: AstUtf8,
        value: Expr,
        span: Span,
    ) -> Stmt {
        let member = self.namespace_member(parameter, name, span);
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberExpression(member),
        );
        let assignment =
            self.ast()
                .expr_assignment_expression(span, target, value, AssignmentOperator::Assign);
        self.ast().stmt_expression_statement(span, assignment)
    }

    fn namespace_member(
        &mut self,
        object: AstUtf8,
        property: AstUtf8,
        span: Span,
    ) -> swc_next_ecma_ast::MemberExpression {
        let object = self.ast().expr_identifier_reference(DUMMY_SP, object);
        let property = self.ast().identifier_name(span, property);
        self.ast().member_expression(
            span,
            object,
            PropertyKey::IdentifierName(property),
            false,
            false,
        )
    }

    fn namespace_parameter_for_reference(
        &self,
        node: swc_next_ecma_ast::NodeId,
    ) -> Option<AstUtf8> {
        if self.namespaces.contexts.is_empty() || node.index() >= self.semantic_node_count {
            return None;
        }
        let reference = self.semantic.reference_of(node)?;
        let reference = self.semantic.reference(reference);
        // Qualification preserves the selected binding, even when an opaque
        // alias has uncertain value meaning. It does not evaluate the target or
        // decide whether its declaration emits code.
        let symbol = reference.resolution().candidate()?;
        self.namespaces.parameter_for_symbol(
            self.ast_ref(),
            self.semantic,
            self.ast_ref().get_utf8(reference.name).as_bytes(),
            symbol,
        )
    }

    fn namespace_body_local_symbols(&self, body: TsModuleBody) -> FxHashSet<SymbolId> {
        let TsModuleBodyData::TsModuleBlock(block) = self.ast_ref().ts_module_body_data(body)
        else {
            return FxHashSet::default();
        };
        let mut symbols = FxHashSet::default();
        for statement_id in block.body(self.ast_ref()).iter() {
            let statement = self.ast_ref().get_node_in_sub_range(statement_id);
            let StmtData::ExportNamedDeclaration(export) = self.ast_ref().stmt_data(statement)
            else {
                continue;
            };
            let Some(declaration) = export.declaration(self.ast_ref()) else {
                continue;
            };
            if let Some(symbol) = self.namespace_export_local_symbol(declaration) {
                symbols.insert(symbol);
            }
        }
        symbols
    }

    fn namespace_export_local_symbol(&self, declaration: Decl) -> Option<SymbolId> {
        let binding = match self.ast_ref().decl_data(declaration) {
            DeclData::Function(function) => function.id(self.ast_ref()),
            DeclData::Class(class) if !class.declare(self.ast_ref()) => class.id(self.ast_ref()),
            DeclData::TsEnumDeclaration(enum_declaration)
                if !enum_declaration.declare(self.ast_ref()) =>
            {
                let binding = enum_declaration.id(self.ast_ref());
                let symbol = self.semantic.symbol_of(binding.node_id())?;
                return (!self.analysis.const_enum_is_removable(symbol)).then_some(symbol);
            }
            DeclData::TsModuleDeclaration(namespace)
                if self.analysis.namespace_is_instantiated(namespace.node_id()) =>
            {
                return self
                    .namespace_path(namespace.id(self.ast_ref()))
                    .map(|path| path.root.symbol);
            }
            _ => None,
        }?;
        self.semantic.symbol_of(binding.node_id())
    }

    fn jsx_name_root(&self, name: JsxElementName) -> Option<JsxIdentifier> {
        match self.ast_ref().jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => Some(identifier),
            JsxElementNameData::JsxMemberExpression(member) => self.jsx_member_root(member),
            JsxElementNameData::JsxNamespacedName(_) => None,
        }
    }

    fn jsx_member_root(&self, member: JsxMemberExpression) -> Option<JsxIdentifier> {
        match self
            .ast_ref()
            .jsx_member_expression_object_data(member.object(self.ast_ref()))
        {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => Some(identifier),
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                self.jsx_member_root(member)
            }
        }
    }

    fn prepend_jsx_namespace(
        &mut self,
        object: JsxMemberExpressionObject,
        parameter: AstUtf8,
    ) -> JsxMemberExpressionObject {
        match self.ast_ref().jsx_member_expression_object_data(object) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                let namespace = self
                    .ast()
                    .jsx_member_expression_object_jsx_identifier(DUMMY_SP, parameter);
                self.ast()
                    .jsx_member_expression_object_jsx_member_expression(
                        DUMMY_SP, namespace, identifier,
                    )
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                let inner = member.object(self.ast_ref());
                let inner = self.prepend_jsx_namespace(inner, parameter);
                member.set_object(self.ast(), inner);
                object
            }
        }
    }

    fn namespace_path(&self, name: TsModuleName) -> Option<NamespacePath> {
        let mut properties = Vec::new();
        let binding = match self.ast_ref().ts_module_name_data(name) {
            TsModuleNameData::BindingIdentifier(binding) => binding,
            TsModuleNameData::TsQualifiedName(qualified) => {
                self.collect_qualified_namespace_path(qualified, &mut properties)?
            }
            TsModuleNameData::StringLiteral(_) => return None,
        };
        let symbol = self
            .semantic
            .symbol_of(binding.node_id())
            .expect("a namespace declaration has a symbol");
        Some(NamespacePath {
            binding,
            root: NamespacePathPart {
                symbol,
                name: binding.name(self.ast_ref()),
                span: binding.span(self.ast_ref()),
            },
            properties,
        })
    }

    fn collect_qualified_namespace_path(
        &self,
        qualified: TsQualifiedName,
        output: &mut Vec<NamespacePathPart>,
    ) -> Option<BindingIdentifier> {
        let left = qualified.left(self.ast_ref());
        let binding = match self.ast_ref().ts_qualified_name_left_data(left) {
            TsQualifiedNameLeftData::BindingIdentifier(binding) => binding,
            TsQualifiedNameLeftData::TsQualifiedName(qualified) => {
                self.collect_qualified_namespace_path(qualified, output)?
            }
            TsQualifiedNameLeftData::IdentifierName(_)
            | TsQualifiedNameLeftData::IdentifierReference(_)
            | TsQualifiedNameLeftData::ThisExpression(_) => return None,
        };
        let identifier = qualified.right(self.ast_ref());
        let symbol = self
            .semantic
            .symbol_of(identifier.node_id())
            .expect("a qualified namespace member has a symbol");
        output.push(NamespacePathPart {
            symbol,
            name: identifier.name(self.ast_ref()),
            span: identifier.span(self.ast_ref()),
        });
        Some(binding)
    }
}

struct NamespaceBindingPatternRewriter<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    semantic: &'visit swc_next_ecma_semantic::Semantic<'ast>,
    property_symbols: &'visit FxHashSet<SymbolId>,
    parameter: AstUtf8,
}

impl<'ast> VisitMut<'ast> for NamespaceBindingPatternRewriter<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_binding_pattern(&mut self, pattern: BindingPattern) -> Option<BindingPattern> {
        let BindingPatternData::BindingIdentifier(identifier) =
            self.ast.binding_pattern_data(pattern)
        else {
            return pattern.visit_mut_children_with(self);
        };
        let symbol = self.semantic.symbol_of(identifier.node_id());
        if !symbol.is_some_and(|symbol| self.property_symbols.contains(&symbol)) {
            return None;
        }
        let span = identifier.span(self.ast);
        let name = identifier.name(self.ast);
        let object = self.ast.expr_identifier_reference(DUMMY_SP, self.parameter);
        let property = self.ast.identifier_name(span, name);
        let member = self.ast.member_expression(
            span,
            object,
            PropertyKey::IdentifierName(property),
            false,
            false,
        );
        Some(BindingPattern::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberExpression(member),
        ))
    }

    fn visit_mut_binding_property(&mut self, property: BindingProperty) -> Option<BindingProperty> {
        property.visit_mut_children_with(self);
        super::erase::update_binding_property_shorthand(self.ast, property);
        None
    }
}

fn binding_pattern_to_assignment_target(
    pattern: BindingPattern,
    ast: &Ast<'_>,
) -> AssignmentTarget {
    match ast.binding_pattern_data(pattern) {
        swc_next_ecma_ast::BindingPatternData::SimpleAssignmentTarget(target) => {
            AssignmentTarget::SimpleAssignmentTarget(target)
        }
        swc_next_ecma_ast::BindingPatternData::ArrayPattern(pattern) => {
            AssignmentTarget::ArrayAssignmentTarget(pattern)
        }
        swc_next_ecma_ast::BindingPatternData::ObjectPattern(pattern) => {
            AssignmentTarget::ObjectAssignmentTarget(pattern)
        }
        _ => unreachable!("an exported namespace variable has an assignment-compatible pattern"),
    }
}
