//! The semantic builder, a visitor that builds the semantic model and reports
//! the scope-dependent early errors the specification requires.

mod declarations;
mod js;
mod ts;

use std::{borrow::Cow, collections::hash_map::Entry};

use rustc_hash::FxHashMap;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    ArrayPattern, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    AssignmentPattern, Ast, BinaryOperator, BindingIdentifier, BindingProperty, BindingRestElement,
    BlockStatement, BooleanLiteral, CallExpression, CatchClause, Class, ClassBody, ClassType,
    Decorator, Diagnostic, Directive, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationKindData, ExportNamedDeclaration, ExportSpecifier, Expr,
    ForInStatement, ForOfStatement, ForStatement, FormalParameters, Function, FunctionBody,
    FunctionType, IdentifierName, IdentifierReference, ImportDeclaration, ImportOrExportKind,
    ImportSpecifier, JsxClosingElement, JsxIdentifier, JsxOpeningElement, JsxText, Label,
    MemberExpression, MethodDefinition, NodeId, NodeKind, NodeKindData, ObjectPattern,
    ParenthesizedExpression, PropertyDefinition, PropertyKey, PropertyKeyData, ScopeId, Severity,
    Span, StaticBlock, SwitchCase, SwitchStatement, ThisExpression, TsArrayType, TsAsExpression,
    TsCallSignatureDeclaration, TsClassImplements, TsConditionalType,
    TsConstructSignatureDeclaration, TsConstructorType, TsEnumBody, TsEnumDeclaration,
    TsEnumMember, TsExportAssignment, TsFunction, TsFunctionType, TsGlobalDeclaration,
    TsImportEqualsDeclaration, TsImportType, TsIndexSignature, TsIndexedAccessType, TsInferType,
    TsInterfaceBody, TsInterfaceDeclaration, TsInterfaceHeritage, TsIntersectionType,
    TsJsDocNonNullableType, TsJsDocNullableType, TsJsDocUnknownType, TsLiteralType, TsMappedType,
    TsMethodDefinition, TsMethodSignature, TsModuleBlock, TsModuleDeclaration, TsModuleNameData,
    TsNamedTupleMember, TsNamespaceExportDeclaration, TsNonNullExpression, TsOptionalType,
    TsParenthesizedType, TsPropertySignature, TsQualifiedName, TsRestType, TsSatisfiesExpression,
    TsTemplateLiteralType, TsThisParameter, TsTupleType, TsType, TsTypeAliasDeclaration,
    TsTypeAnnotation, TsTypeAssertion, TsTypeLiteral, TsTypeOperator, TsTypeParameter,
    TsTypeParameterDeclaration, TsTypeParameterInstantiation, TsTypePredicate, TsTypeQuery,
    TsTypeReference, TsUnionType, UnaryExpression, UnaryOperator, UpdateExpression,
    VariableDeclaration, VariableKind, Visit, VisitWith, WithStatement,
};

use crate::{
    ReferenceFlags, ReferenceSpace, SymbolFlags, SyntaxChecks,
    binder::PreparedDeclaration,
    context::SemanticContext,
    ecmascript,
    scope::{DecoratorSite, ScopeKind},
};

pub struct SemanticBuilder<'traverse, 'a> {
    exported_names: FxHashMap<&'traverse [u8], NodeId>,
    export_specifiers: Vec<ExportSpecifier>,
    diagnostics: Vec<Diagnostic<'a>>,
    state: SemanticContext<'a>,
    ast: &'traverse Ast<'a>,
    path: Vec<NodeId>,
    check_syntax: SyntaxChecks,
    in_ts_module_name: bool,
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    pub fn new(
        ast: &'traverse Ast<'a>,
        check_syntax: SyntaxChecks,
        build_module_record: bool,
    ) -> Self {
        Self {
            exported_names: FxHashMap::default(),
            export_specifiers: Vec::new(),
            diagnostics: Vec::new(),
            state: SemanticContext::init(ast, build_module_record),
            ast,
            path: Vec::with_capacity(64),
            check_syntax,
            in_ts_module_name: false,
        }
    }

    pub(crate) fn finish(mut self) -> (Vec<Diagnostic<'a>>, SemanticContext<'a>) {
        let check_syntax = self.check_syntax;
        let (export_alias_issues, import_alias_issues) = {
            let state = &mut self.state;
            state.symbols.bind_annex_b_functions(self.ast, &state.scope);
            state
                .symbols
                .resolve_ts_aliases(self.ast, &state.scope, check_syntax)
        };
        if check_syntax.ts_aliases() {
            ts::check_ts_alias_issues(&mut self, export_alias_issues, import_alias_issues);
        }
        debug_assert!(self.path.is_empty());
        (self.diagnostics, self.state)
    }

    fn enter_node(&mut self, node: NodeId) -> ReferenceFlags {
        self.path.push(node);
        self.state.enter_node()
    }

    fn parent(&self) -> Option<NodeId> {
        self.path.iter().rev().nth(1).copied()
    }

    fn current_node(&self) -> Option<NodeId> {
        self.path.last().copied()
    }

    fn ancestors(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.path.iter().rev().copied()
    }

    fn record_ts_expando_assignment(&mut self, node: swc_next_ecma_ast::AssignmentExpression) {
        self.state.symbols.record_ts_expando_assignment(
            self.ast,
            node,
            &self.state.scope,
            self.path.iter().rev().copied(),
        );
    }

    #[inline]
    fn record_identifier_reference(&mut self, node: IdentifierReference, flags: ReferenceFlags) {
        let state = &mut self.state;
        state
            .symbols
            .add_identifier_reference(self.ast, node, &state.scope, flags);
    }

    #[inline]
    fn visit_with_reference_flags<T>(&mut self, node: T, flags: ReferenceFlags)
    where
        T: VisitWith<'a, Self>,
    {
        let previous = self.state.replace_reference_flags(flags);
        node.visit_with(self);
        self.state.replace_reference_flags(previous);
    }

    #[inline]
    fn visit_with_reference_context<T>(&mut self, node: T, flags: ReferenceFlags)
    where
        T: VisitWith<'a, Self>,
    {
        let previous = self.state.replace_default_reference_flags(flags);
        self.visit_with_reference_flags(node, flags);
        self.state.replace_default_reference_flags(previous);
    }

    #[inline]
    fn assignment_reference_flags(&self, read: bool) -> ReferenceFlags {
        let flags = self.state.default_reference_flags();
        if !flags.is_read() {
            flags
        } else if read {
            flags.read_write()
        } else {
            flags.write_only()
        }
    }

    #[inline]
    fn flags_with_space(&self, space: ReferenceSpace) -> ReferenceFlags {
        let flags = self.state.default_reference_flags().with_space(space);
        if space.in_type_position() {
            flags.no_access()
        } else {
            flags
        }
    }

    fn visit_assertion_type(&mut self, annotation: TsType) {
        let tree = self.ast;
        if let NodeKindData::TsTypeReference(reference) = tree.kind_data(annotation.node_id())
            && reference.type_arguments(tree).is_none()
            && let NodeKindData::IdentifierReference(name) =
                tree.kind_data(reference.type_name(tree).node_id())
            && tree.get_utf8(name.name(tree)) == "const"
        {
            // Like TS7's isConstTypeReferenceName, this applies only to the
            // bare marker in an assertion, never to an ordinary type name.
            return;
        }
        annotation.visit_with(self);
    }

    #[inline]
    fn visit_type_member_key(&mut self, key: PropertyKey, computed: bool) {
        if computed {
            // TypeScript checks computed names as expressions in value space,
            // even on erased declarations. Apply that context to the whole key.
            let flags = self.flags_with_space(ReferenceSpace::Typeof);
            self.visit_with_reference_context(key, flags);
        } else {
            key.visit_with(self);
        }
    }

    #[inline]
    fn visit_erased_member_key(&mut self, key: PropertyKey, computed: bool, decorated: bool) {
        if decorated {
            // Legacy decorators evaluate a computed key even when the member
            // declaration itself is erased.
            key.visit_with(self);
        } else {
            self.visit_type_member_key(key, computed);
        }
    }

    fn visit_ts_heritage_expression(
        &mut self,
        expression: Expr,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let space = if self.ast.node_kind(expression.node_id()) == NodeKind::MemberExpression {
            ReferenceSpace::Namespace
        } else {
            ReferenceSpace::Type
        };
        let flags = self.flags_with_space(space);
        self.visit_with_reference_flags(expression, flags);
        type_arguments.visit_with(self);
    }
}

impl SemanticBuilder<'_, '_> {
    #[inline(always)]
    fn check(&mut self, node_data: NodeKindData) {
        match node_data {
            NodeKindData::Program(_) => js::check_program(self),
            NodeKindData::StringLiteral(node) => js::check_string_literal(self, node),
            NodeKindData::NumericLiteral(node) => js::check_numeric_literal(self, node),
            NodeKindData::RegExpLiteral(node) => js::check_reg_exp_literal(self, node),
            NodeKindData::IdentifierReference(node) => {
                let tree = self.ast;
                let name = tree.get_utf8(node.name(tree)).as_bytes();
                js::check_identifier_reference(self, node, name);
                if tree.is_ts() && name == b"await" {
                    ts::check_contextual_await(
                        self,
                        node.node_id(),
                        "'await' cannot be used as an identifier here",
                    );
                }
            }
            NodeKindData::LabelIdentifier(node) => js::check_label_identifier(self, node),
            NodeKindData::BinaryExpression(node) => js::check_binary_expression(self, node),
            NodeKindData::UnaryExpression(node) => js::check_unary_expression(self, node),
            NodeKindData::UpdateExpression(node) => js::check_update_expression(self, node),
            NodeKindData::AssignmentExpression(node) => js::check_assignment_expression(self, node),
            NodeKindData::VariableDeclaration(node) => js::check_variable_declaration(self, node),
            NodeKindData::SwitchStatement(node) => js::check_switch_statement(self, node),
            NodeKindData::ForInStatement(node) => js::check_for_in_statement(self, node),
            NodeKindData::ForOfStatement(node) => js::check_for_of_statement(self, node),
            NodeKindData::BreakStatement(node) => js::check_break_statement(self, node),
            NodeKindData::ContinueStatement(node) => js::check_continue_statement(self, node),
            NodeKindData::LabeledStatement(node) => js::check_labeled_statement(self, node),
            NodeKindData::Directive(node) => js::check_directive(self, node),
            NodeKindData::ClassBody(node) => js::check_class_body(self, node),
            NodeKindData::MethodDefinition(node) => ts::check_method_definition(self, node),
            NodeKindData::Function(node) => js::check_function(self, node),
            NodeKindData::WithStatement(node) => js::check_with_statement(self, node),
            NodeKindData::ObjectExpression(node) => js::check_object_expression(self, node),
            NodeKindData::MemberExpression(node) => js::check_member_expression(self, node),
            NodeKindData::CallExpression(node) => js::check_call_expression(self, node),
            NodeKindData::AwaitExpression(node) => js::check_await_expression(self, node),
            NodeKindData::YieldExpression(node) => js::check_yield_expression(self, node),
            NodeKindData::TsParameterProperty(node) => ts::check_parameter_property(self, node),
            NodeKindData::MetaProperty(node) => js::check_meta_property(self, node),
            NodeKindData::ImportDeclaration(node) => js::check_import_declaration(self, node),
            NodeKindData::ExportNamedDeclaration(node) => {
                js::check_export_named_declaration(self, node)
            }
            NodeKindData::ExportDefaultDeclaration(node) => {
                js::check_export_default_declaration(self, node)
            }
            NodeKindData::TsExportAssignment(node) => ts::check_ts_export_assignment(self, node),
            NodeKindData::TsModuleDeclaration(node) => ts::check_ts_module_declaration(self, node),
            NodeKindData::TsGlobalDeclaration(node) => ts::check_ts_global_declaration(self, node),
            NodeKindData::ExportAllDeclaration(node) => {
                js::check_export_all_declaration(self, node)
            }
            NodeKindData::ExportSpecifier(node) => js::check_export_specifier(self, node),
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for SemanticBuilder<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    #[inline(always)]
    fn enter_node(&mut self, node: NodeKindData) {
        self.enter_node(node.node_id());
    }

    #[inline(always)]
    fn leave_node(&mut self, node: NodeKindData) {
        if !self.check_syntax.is_empty() {
            self.check(node);
        }
        let current = self.path.pop();
        debug_assert_eq!(current, Some(node.node_id()));
    }

    #[inline]
    fn visit_update_expression(&mut self, node: UpdateExpression) {
        self.enter_node(node.node_id());
        let argument = node.argument(self.ast);
        let flags = self.assignment_reference_flags(true);
        self.visit_with_reference_flags(argument, flags);
        self.leave_node(NodeKindData::UpdateExpression(node));
    }

    #[inline]
    fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
        self.enter_node(node.node_id());
        self.record_ts_expando_assignment(node);
        let left = node.left(self.ast);
        let read = node.operator(self.ast) != AssignmentOperator::Assign;
        let flags = self.assignment_reference_flags(read);
        self.visit_with_reference_flags(left, flags);
        node.right(self.ast).visit_with(self);
        self.leave_node(NodeKindData::AssignmentExpression(node));
    }

    #[inline]
    fn visit_member_expression(&mut self, node: MemberExpression) {
        let node_flags = self.enter_node(node.node_id());
        // Assignment writes the property, not its base. Both the base and a
        // computed property are evaluated as reads.
        let object_flags = self.flags_with_space(node_flags.space);
        self.visit_with_reference_flags(node.object(self.ast), object_flags);
        node.property(self.ast).visit_with(self);
        self.leave_node(NodeKindData::MemberExpression(node));
    }

    #[inline]
    fn visit_parenthesized_expression(&mut self, node: ParenthesizedExpression) {
        let node_flags = self.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.ast), node_flags);
        self.leave_node(NodeKindData::ParenthesizedExpression(node));
    }

    #[inline]
    fn visit_ts_as_expression(&mut self, node: TsAsExpression) {
        let node_flags = self.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.ast), node_flags);
        self.visit_assertion_type(node.type_annotation(self.ast));
        self.leave_node(NodeKindData::TsAsExpression(node));
    }

    #[inline]
    fn visit_ts_satisfies_expression(&mut self, node: TsSatisfiesExpression) {
        let node_flags = self.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.ast), node_flags);
        node.type_annotation(self.ast).visit_with(self);
        self.leave_node(NodeKindData::TsSatisfiesExpression(node));
    }

    #[inline]
    fn visit_ts_type_assertion(&mut self, node: TsTypeAssertion) {
        let node_flags = self.enter_node(node.node_id());
        self.visit_assertion_type(node.type_annotation(self.ast));
        self.visit_with_reference_flags(node.expression(self.ast), node_flags);
        self.leave_node(NodeKindData::TsTypeAssertion(node));
    }

    #[inline]
    fn visit_ts_non_null_expression(&mut self, node: TsNonNullExpression) {
        let node_flags = self.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.ast), node_flags);
        self.leave_node(NodeKindData::TsNonNullExpression(node));
    }

    #[inline]
    fn visit_unary_expression(&mut self, node: UnaryExpression) {
        self.enter_node(node.node_id());
        let argument = node.argument(self.ast);
        if node.operator(self.ast) == UnaryOperator::Delete {
            let flags = self.state.default_reference_flags().no_access();
            self.visit_with_reference_flags(argument, flags);
        } else {
            argument.visit_with(self);
        }
        self.leave_node(NodeKindData::UnaryExpression(node));
    }

    // These leaves have neither binding actions nor syntax checks. Consume
    // pending reference flags without adding an unused ancestry entry.
    #[inline]
    fn visit_this_expression(&mut self, _node: ThisExpression) {
        self.state.enter_node();
    }

    #[inline]
    fn visit_boolean_literal(&mut self, _node: BooleanLiteral) {
        self.state.enter_node();
    }

    #[inline]
    fn visit_identifier_name(&mut self, _node: IdentifierName) {
        self.state.enter_node();
    }

    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        // This leaf has no children. Only syntax checks inspect its ancestry;
        // binding still consumes the pending reference flags in either mode.
        if self.check_syntax.identifier_references() {
            let flags = self.enter_node(node.node_id());
            self.record_identifier_reference(node, flags);
            self.check(NodeKindData::IdentifierReference(node));
            let current = self.path.pop();
            debug_assert_eq!(current, Some(node.node_id()));
        } else {
            let flags = self.state.enter_node();
            self.record_identifier_reference(node, flags);
        }
    }

    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        let prepared = if self.check_syntax.binding_identifiers() {
            let tree = self.ast;
            let name = tree.get_utf8(node.name(tree)).as_bytes();
            let prepared = js::check_binding_identifier(self, node, name);
            if tree.is_ts() && name == b"await" {
                ts::check_contextual_await(
                    self,
                    node.node_id(),
                    "'await' cannot be used as a binding identifier here",
                );
            }
            prepared
        } else {
            None
        };
        let current = self.path.pop();
        debug_assert_eq!(current, Some(node.node_id()));
        let direct_export = self.state.symbols.export_state.named();
        let direct_export = direct_export.filter(|_| !self.state.in_ts_namespace());
        let state = &mut self.state;
        let declared = if let Some(prepared) = prepared {
            state.symbols.declare_prepared_binding_identifier(
                self.ast,
                node,
                &state.scope,
                prepared,
            )
        } else {
            state
                .symbols
                .declare_binding_identifier(self.ast, node, &state.scope)
        };
        if declared && let Some(statement) = direct_export {
            state
                .module_record
                .add_direct_export(self.ast, node.node_id(), statement);
        }
    }

    #[inline]
    fn visit_assignment_pattern(&mut self, node: AssignmentPattern) {
        let node_flags = self.enter_node(node.node_id());
        self.visit_with_reference_flags(node.left(self.ast), node_flags);
        node.right(self.ast).visit_with(self);
        self.leave_node(NodeKindData::AssignmentPattern(node));
    }

    #[inline]
    fn visit_binding_rest_element(&mut self, node: BindingRestElement) {
        let node_flags = self.enter_node(node.node_id());
        self.visit_with_reference_flags(node.argument(self.ast), node_flags);
        self.leave_node(NodeKindData::BindingRestElement(node));
    }

    #[inline]
    fn visit_array_pattern(&mut self, node: ArrayPattern) {
        let node_flags = self.enter_node(node.node_id());
        for element in node.elements(self.ast).iter() {
            if let Some(element) = self.ast.get_node_in_sub_range(element) {
                self.visit_with_reference_flags(element, node_flags);
            }
        }
        if let Some(rest) = node.rest(self.ast) {
            self.visit_with_reference_flags(rest, node_flags);
        }
        self.leave_node(NodeKindData::ArrayPattern(node));
    }

    #[inline]
    fn visit_object_pattern(&mut self, node: ObjectPattern) {
        let node_flags = self.enter_node(node.node_id());
        for property in node.properties(self.ast).iter() {
            let property = self.ast.get_node_in_sub_range(property);
            self.visit_with_reference_flags(property, node_flags);
        }
        if let Some(rest) = node.rest(self.ast) {
            self.visit_with_reference_flags(rest, node_flags);
        }
        self.leave_node(NodeKindData::ObjectPattern(node));
    }

    #[inline]
    fn visit_binding_property(&mut self, node: BindingProperty) {
        let node_flags = self.enter_node(node.node_id());
        if node_flags.space.in_type_position() {
            self.visit_type_member_key(node.key(self.ast), node.computed(self.ast));
        } else {
            node.key(self.ast).visit_with(self);
        }
        self.visit_with_reference_flags(node.value(self.ast), node_flags);
        self.leave_node(NodeKindData::BindingProperty(node));
    }

    fn visit_variable_declaration(&mut self, node: VariableDeclaration) {
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_variable_declaration(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::VariableDeclaration(node));
    }

    fn visit_block_statement(&mut self, node: BlockStatement) {
        self.state.enter_block_statement(self.ast, node.node_id());
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::BlockStatement(node));
    }

    /// Per section 14.12, the CaseBlock grammar permits at most one DefaultClause.
    fn visit_switch_statement(&mut self, node: SwitchStatement) {
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::SwitchStatement(node));
        self.state.exit_scope(node.node_id());
    }

    fn visit_switch_case(&mut self, node: SwitchCase) {
        self.state.enter_switch_case(self.current_node());
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::SwitchCase(node));
    }

    fn visit_for_statement(&mut self, node: ForStatement) {
        self.state.enter_for_statement(node.node_id());
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::ForStatement(node));
    }

    /// <https://tc39.es/ecma262/#sec-for-in-and-for-of-statements-static-semantics-early-errors>
    fn visit_for_in_statement(&mut self, node: ForInStatement) {
        self.state.enter_for_in_statement(node.node_id());
        self.enter_node(node.node_id());
        let flags = self.assignment_reference_flags(false);
        self.visit_with_reference_flags(node.left(self.ast), flags);
        node.right(self.ast).visit_with(self);
        node.body(self.ast).visit_with(self);
        self.leave_node(NodeKindData::ForInStatement(node));
        self.state.exit_scope(node.node_id());
    }

    /// <https://tc39.es/ecma262/#sec-for-in-and-for-of-statements-static-semantics-early-errors>
    fn visit_for_of_statement(&mut self, node: ForOfStatement) {
        self.state.enter_for_of_statement(node.node_id());
        self.enter_node(node.node_id());
        let flags = self.assignment_reference_flags(false);
        self.visit_with_reference_flags(node.left(self.ast), flags);
        node.right(self.ast).visit_with(self);
        node.body(self.ast).visit_with(self);
        self.leave_node(NodeKindData::ForOfStatement(node));
        self.state.exit_scope(node.node_id());
    }

    fn visit_catch_clause(&mut self, node: CatchClause) {
        self.state.enter_catch_clause(node.node_id());
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_catch_clause(&state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::CatchClause(node));
    }

    fn visit_with_statement(&mut self, node: WithStatement) {
        self.enter_node(node.node_id());
        node.object(self.ast).visit_with(self);
        self.state.enter_with_statement(node.node_id());
        node.body(self.ast).visit_with(self);
        self.state.exit_with_statement(node.node_id());
        self.leave_node(NodeKindData::WithStatement(node));
    }

    /// <https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors>
    ///
    /// "It is a Syntax Error if FunctionBodyContainsUseStrict is true and
    /// IsSimpleParameterList of FormalParameters is false."
    fn visit_directive(&mut self, node: Directive) {
        self.state.enter_directive(self.ast, node);
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::Directive(node));
    }

    // https://tc39.es/ecma262/#sec-labelled-statements-static-semantics-early-errors (14.13.1)
    //   LabelledItem : FunctionDeclaration - syntax error unless non-strict + web host (B.3.1).
    // https://tc39.es/ecma262/#sec-if-statement-static-semantics-early-errors (14.6.1)
    //   IsLabelledFunction prevents labelled functions in if/iteration/with bodies.
    // https://tc39.es/ecma262/#sec-functiondeclarations-in-ifstatement-statement-clauses (B.3.3)
    //   Bare FunctionDeclaration in if/else is allowed in non-strict code only.
    fn visit_function(&mut self, node: Function) {
        let annex_b_if_body = is_annex_b_if_function(self.ast, self.current_node(), node);
        self.state.enter_function(self.ast, node);
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_function(self.ast, node, &state.scope, annex_b_if_body);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::Function(node));
        self.state.exit_scope(node.node_id());
    }

    fn visit_ts_function(&mut self, node: TsFunction) {
        self.state.enter_ts_function(node.node_id());
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_function(self.ast, node, &state.scope);
        node.id(self.ast).visit_with(self);
        node.type_parameters(self.ast).visit_with(self);
        node.this_param(self.ast).visit_with(self);
        let flags = self.flags_with_space(ReferenceSpace::Typeof);
        self.visit_with_reference_context(node.params(self.ast), flags);
        node.return_type(self.ast).visit_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsFunction(node));
    }

    fn visit_arrow_function_expression(&mut self, node: ArrowFunctionExpression) {
        self.state.enter_arrow_function_expression(self.ast, node);
        let export_state = self.state.symbols.enter_arrow_function_expression();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.state
            .symbols
            .exit_arrow_function_expression(export_state);
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::ArrowFunctionExpression(node));
    }

    fn visit_ts_this_parameter(&mut self, node: TsThisParameter) {
        self.enter_node(node.node_id());
        // The receiver annotation has the same lookup rules as parameter annotations,
        // although it is stored on the signature and creates no runtime binding.
        self.state
            .scope
            .enter_ts_lookup_view(node.node_id(), ScopeKind::TsSignatureParameter);
        node.visit_children_with(self);
        self.state.scope.exit_ts_lookup_view(node.node_id());
        self.leave_node(NodeKindData::TsThisParameter(node));
    }

    fn visit_formal_parameters(&mut self, node: FormalParameters) {
        self.enter_node(node.node_id());
        if node.items(self.ast).is_empty() && node.rest(self.ast).is_none() {
            // The function body no longer belongs to the direct export declaration.
            self.state.symbols.export_state = Default::default();
            self.leave_node(NodeKindData::FormalParameters(node));
            return;
        }
        let state = &mut self.state;
        // Bind parameters to their owner before entering the lookup-only region.
        state.symbols.enter_formal_parameters(&state.scope);
        let parameter_view = self.ast.is_ts();
        if parameter_view {
            state
                .scope
                .enter_ts_lookup_view(node.node_id(), ScopeKind::TsSignatureParameter);
        }
        node.visit_children_with(self);
        if parameter_view {
            self.state.scope.exit_ts_lookup_view(node.node_id());
        }
        self.state.symbols.exit_formal_parameters();
        self.leave_node(NodeKindData::FormalParameters(node));
    }

    fn visit_function_body(&mut self, node: FunctionBody) {
        let parent = self.current_node();
        self.state.enter_function_body(self.ast, node, parent);
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::FunctionBody(node));
    }

    fn visit_class(&mut self, node: Class) {
        self.state.enter_class(node);
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_class(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::Class(node));
    }

    /// 15.7.1 Static Semantics: Early Errors.
    /// <https://tc39.es/ecma262/#sec-class-definitions-static-semantics-early-errors>
    ///
    /// `PrototypePropertyNameList` may contain only one `constructor`.
    /// `PrivateBoundIdentifiers` may contain no duplicate entries, except
    /// one getter/setter pair with matching staticness.
    fn visit_class_body(&mut self, node: ClassBody) {
        self.enter_node(node.node_id());
        self.state.symbols.enter_class_body();
        node.visit_children_with(self);
        self.leave_node(NodeKindData::ClassBody(node));
    }

    #[inline]
    fn visit_property_definition(&mut self, node: PropertyDefinition) {
        self.enter_node(node.node_id());
        let decorators = node.decorators(self.ast);
        decorators.visit_with(self);
        let key = node.key(self.ast);
        let erased = node.declare(self.ast) || node.r#abstract(self.ast);
        if erased {
            self.visit_erased_member_key(key, node.computed(self.ast), !decorators.is_empty());
        } else {
            key.visit_with(self);
        }
        node.type_annotation(self.ast).visit_with(self);
        node.value(self.ast).visit_with(self);
        self.leave_node(NodeKindData::PropertyDefinition(node));
    }

    #[inline]
    fn visit_ts_method_definition(&mut self, node: TsMethodDefinition) {
        self.enter_node(node.node_id());
        let decorators = node.decorators(self.ast);
        decorators.visit_with(self);
        self.visit_erased_member_key(
            node.key(self.ast),
            node.computed(self.ast),
            !decorators.is_empty(),
        );
        node.value(self.ast).visit_with(self);
        self.leave_node(NodeKindData::TsMethodDefinition(node));
    }

    fn visit_static_block(&mut self, node: StaticBlock) {
        self.state.enter_static_block(node.node_id());
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::StaticBlock(node));
    }

    fn visit_decorator(&mut self, node: Decorator) {
        // TS resolves members and parameters at the class, skipping method
        // locals. Only a class declaration's own decorators skip the class;
        // class expressions retain their type parameters and name environment.
        let site = match self.current_node().map(|parent| self.ast.kind_data(parent)) {
            Some(NodeKindData::Class(class))
                if class.class_type(self.ast) == ClassType::ClassDeclaration =>
            {
                DecoratorSite::ClassDeclaration
            }
            Some(NodeKindData::Class(_)) => DecoratorSite::ClassExpression,
            _ => DecoratorSite::Member,
        };
        self.state.enter_decorator(node.node_id(), site);
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.state.exit_decorator();
        self.leave_node(NodeKindData::Decorator(node));
    }

    #[inline]
    fn visit_ts_class_implements(&mut self, node: TsClassImplements) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        self.visit_ts_heritage_expression(node.expression(self.ast), node.type_arguments(self.ast));
        self.leave_node(NodeKindData::TsClassImplements(node));
        self.state.exit_type_position(type_position);
    }

    // https://tc39.es/ecma262/#sec-static-semantics-early-errors
    fn visit_import_declaration(&mut self, node: ImportDeclaration) {
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_import_declaration(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::ImportDeclaration(node));
    }

    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        self.enter_node(node.node_id());
        self.state.symbols.enter_import_specifier(self.ast, node);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::ImportSpecifier(node));
    }

    fn visit_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        self.enter_node(node.node_id());
        let import_kind = node.import_kind(self.ast);
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_import_equals_declaration(import_kind, &state.scope);
        let mut flags = self.flags_with_space(ReferenceSpace::Any);
        if import_kind == ImportOrExportKind::Type {
            flags = flags.no_access();
        }
        self.visit_with_reference_flags(node.id(self.ast), flags);
        let target_flags = flags.with_space(ReferenceSpace::Namespace);
        self.visit_with_reference_flags(node.module_reference(self.ast), target_flags);
        let state = &mut self.state;
        state.symbols.record_ts_import_equals_target(self.ast, node);
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsImportEqualsDeclaration(node));
    }

    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        self.enter_node(node.node_id());
        self.state
            .symbols
            .enter_export_named_declaration(self.ast, node);
        node.visit_children_with(self);
        self.state.symbols.exit_export_declaration();
        self.leave_node(NodeKindData::ExportNamedDeclaration(node));
    }

    fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        self.enter_node(node.node_id());
        if !self.state.in_ts_namespace() {
            self.state.module_record.add_default_export(self.ast, node);
        }
        self.state.symbols.enter_export_default_declaration();
        let declaration = node.declaration(self.ast);
        let space = declaration
            .as_expr(self.ast)
            .map_or(ReferenceSpace::Value, |expression| {
                export_expression_space(self.ast, expression.node_id())
            });
        let flags = self.flags_with_space(space);
        self.visit_with_reference_flags(declaration, flags);
        self.state.symbols.exit_export_declaration();
        self.leave_node(NodeKindData::ExportDefaultDeclaration(node));
    }

    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        self.enter_node(node.node_id());
        if !self.state.in_ts_namespace() {
            self.state.module_record.add_export_all(self.ast, node);
        }
        node.visit_children_with(self);
        self.leave_node(NodeKindData::ExportAllDeclaration(node));
    }

    #[inline]
    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        self.enter_node(node.node_id());
        let declaration = self.parent().and_then(|parent| {
            let NodeKindData::ExportNamedDeclaration(declaration) = self.ast.kind_data(parent)
            else {
                return None;
            };
            Some(declaration)
        });
        if !self.state.in_ts_namespace()
            && let Some(declaration) = declaration
        {
            self.state
                .module_record
                .add_specifier(self.ast, declaration, node);
        }
        let type_only = node.export_kind(self.ast) == ImportOrExportKind::Type
            || declaration.is_some_and(|declaration| {
                declaration.export_kind(self.ast) == ImportOrExportKind::Type
            });
        let mut flags = self.flags_with_space(ReferenceSpace::Any);
        if type_only {
            flags = flags.no_access();
        }
        if declaration.is_some_and(|declaration| declaration.source(self.ast).is_none()) {
            let state = &mut self.state;
            state.symbols.add_local_export_reference(
                self.ast,
                node,
                &state.scope,
                flags,
                type_only,
            );
        }
        self.visit_with_reference_flags(node.local(self.ast), flags);
        node.exported(self.ast).visit_with(self);
        self.leave_node(NodeKindData::ExportSpecifier(node));
    }

    #[inline]
    fn visit_ts_export_assignment(&mut self, node: TsExportAssignment) {
        self.enter_node(node.node_id());
        let expression = node.expression(self.ast);
        let space = export_expression_space(self.ast, expression.node_id());
        let flags = self.flags_with_space(space);
        self.visit_with_reference_flags(expression, flags);
        self.leave_node(NodeKindData::TsExportAssignment(node));
    }

    fn visit_ts_enum_declaration(&mut self, node: TsEnumDeclaration) {
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_enum_declaration(self.ast, node, &state.scope);
        let id = node.id(self.ast);
        id.visit_with(self);
        let owner = self
            .state
            .symbols
            .symbol_of_node(id.node_id())
            .expect("an enum declaration has a symbol");
        self.state.symbols.enter_ts_enum_members(owner);
        node.body(self.ast).visit_with(self);
        self.state.symbols.exit_ts_enum_members(owner);
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsEnumDeclaration(node));
    }

    fn visit_ts_enum_body(&mut self, node: TsEnumBody) {
        self.state.enter_ts_enum_body(node.node_id());
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_ts_enum_body(&state.scope);
        node.visit_children_with(self);
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsEnumBody(node));
    }

    fn visit_ts_enum_member(&mut self, node: TsEnumMember) {
        self.enter_node(node.node_id());
        let state = &mut self.state;
        let prepared = state
            .symbols
            .prepare_ts_enum_member(self.ast, node, &state.scope);
        if let Some(prepared) = prepared {
            if self.check_syntax.redeclarations() {
                declarations::check_prepared_redeclaration(self, prepared);
            }
            self.state
                .symbols
                .declare_prepared_ts_enum_member(self.ast, prepared);
        }
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsEnumMember(node));
    }

    fn visit_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_module_declaration(self.ast, node, &state.scope);
        let name = node.id(self.ast);
        let was_in_ts_module_name = std::mem::replace(&mut self.in_ts_module_name, true);
        name.visit_with(self);
        self.in_ts_module_name = was_in_ts_module_name;
        let declarations = self.state.symbols.enter_ts_module_body(self.ast, node);
        if self.check_syntax.redeclarations() {
            for prepared in declarations {
                declarations::check_prepared_redeclaration(self, prepared);
            }
        }
        node.body(self.ast).visit_with(self);
        self.state.symbols.exit_ts_module_body();
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsModuleDeclaration(node));
    }

    fn visit_ts_module_block(&mut self, node: TsModuleBlock) {
        self.state.enter_ts_module_block(node.node_id());
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_ts_module_block(&state.scope);
        node.visit_children_with(self);
        self.state.exit_ts_module_block(node.node_id());
        self.leave_node(NodeKindData::TsModuleBlock(node));
    }

    fn visit_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        self.enter_node(node.node_id());
        self.state
            .symbols
            .enter_ts_global_declaration(self.ast, node);
        node.visit_children_with(self);
        self.state.symbols.exit_ts_global_declaration();
        self.leave_node(NodeKindData::TsGlobalDeclaration(node));
    }

    fn visit_ts_namespace_export_declaration(&mut self, node: TsNamespaceExportDeclaration) {
        self.enter_node(node.node_id());
        if self.state.symbols.in_unsupported_ts_external_module() {
            self.leave_node(NodeKindData::TsNamespaceExportDeclaration(node));
            return;
        }
        let state = &mut self.state;
        state
            .symbols
            .record_ts_global_export(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsNamespaceExportDeclaration(node));
    }

    fn visit_ts_type_alias_declaration(&mut self, node: TsTypeAliasDeclaration) {
        self.state.enter_ts_type_alias_declaration(node.node_id());
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_type_alias_declaration(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsTypeAliasDeclaration(node));
    }

    fn visit_ts_interface_declaration(&mut self, node: TsInterfaceDeclaration) {
        self.state.enter_ts_interface_declaration(node.node_id());
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_interface_declaration(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsInterfaceDeclaration(node));
    }

    #[inline]
    fn visit_ts_interface_heritage(&mut self, node: TsInterfaceHeritage) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        self.visit_ts_heritage_expression(node.expression(self.ast), node.type_arguments(self.ast));
        self.leave_node(NodeKindData::TsInterfaceHeritage(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_interface_body(&mut self, node: TsInterfaceBody) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsInterfaceBody(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_type_annotation(&mut self, node: TsTypeAnnotation) {
        let signature_return = self.current_node().is_some_and(|parent| {
            matches!(
                self.ast.node_kind(parent),
                NodeKind::Function
                    | NodeKind::TsFunction
                    | NodeKind::ArrowFunctionExpression
                    | NodeKind::TsFunctionType
                    | NodeKind::TsConstructorType
                    | NodeKind::TsMethodSignature
                    | NodeKind::TsCallSignatureDeclaration
                    | NodeKind::TsConstructSignatureDeclaration
                    | NodeKind::TsIndexSignature
            )
        });
        let view = signature_return.then_some(ScopeKind::TsSignatureReturn);
        if let Some(view) = view {
            self.state.scope.enter_ts_lookup_view(node.node_id(), view);
        }
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        if view.is_some() {
            self.state.scope.exit_ts_lookup_view(node.node_id());
        }
        self.leave_node(NodeKindData::TsTypeAnnotation(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_type_reference(&mut self, node: TsTypeReference) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeReference(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_qualified_name(&mut self, node: TsQualifiedName) {
        if self.in_ts_module_name {
            self.enter_node(node.node_id());
            node.visit_children_with(self);
            self.leave_node(NodeKindData::TsQualifiedName(node));
            return;
        }
        // Qualified names also occur in runtime import-equals targets. The
        // enclosing syntax determines access; qualification only narrows lookup.
        let node_flags = self.enter_node(node.node_id());
        let space = match node_flags.space {
            ReferenceSpace::Value | ReferenceSpace::Type | ReferenceSpace::Namespace => {
                ReferenceSpace::Namespace
            }
            space => space,
        };
        let flags = node_flags.with_space(space);
        self.visit_with_reference_flags(node.left(self.ast), flags);
        node.right(self.ast).visit_with(self);
        self.leave_node(NodeKindData::TsQualifiedName(node));
    }

    #[inline]
    fn visit_ts_type_query(&mut self, node: TsTypeQuery) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let flags = self.flags_with_space(ReferenceSpace::Typeof);
        self.visit_with_reference_flags(node.expr_name(self.ast), flags);
        node.type_arguments(self.ast).visit_with(self);
        self.leave_node(NodeKindData::TsTypeQuery(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_import_type(&mut self, node: TsImportType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsImportType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_type_parameter(&mut self, node: TsTypeParameter) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let parent = self.parent();
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_type_parameter(self.ast, parent, &state.scope);
        node.name(self.ast).visit_with(self);
        let infer =
            parent.is_some_and(|parent| self.ast.node_kind(parent) == NodeKind::TsInferType);
        if infer {
            let state = &mut self.state;
            state
                .scope
                .enter_ts_lookup_view(node.node_id(), ScopeKind::TsInferConstraint);
            state
                .symbols
                .expose_ts_infer_parameter(self.ast, node.name(self.ast), &state.scope);
        }
        node.constraint(self.ast).visit_with(self);
        node.default(self.ast).visit_with(self);
        if infer {
            self.state.scope.exit_ts_lookup_view(node.node_id());
        }
        self.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsTypeParameter(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_type_parameter_declaration(&mut self, node: TsTypeParameterDeclaration) {
        let type_position = self.state.enter_type_position();
        self.state
            .scope
            .enter_ts_lookup_view(node.node_id(), ScopeKind::TsTypeParameters);
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.state.scope.exit_ts_lookup_view(node.node_id());
        self.leave_node(NodeKindData::TsTypeParameterDeclaration(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_type_parameter_instantiation(&mut self, node: TsTypeParameterInstantiation) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeParameterInstantiation(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_literal_type(&mut self, node: TsLiteralType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsLiteralType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_template_literal_type(&mut self, node: TsTemplateLiteralType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTemplateLiteralType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_array_type(&mut self, node: TsArrayType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsArrayType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_indexed_access_type(&mut self, node: TsIndexedAccessType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsIndexedAccessType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_tuple_type(&mut self, node: TsTupleType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTupleType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_named_tuple_member(&mut self, node: TsNamedTupleMember) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsNamedTupleMember(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_optional_type(&mut self, node: TsOptionalType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        if !self.check_syntax.is_empty() {
            ts::check_tuple_optional_type(self, node);
        }
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsOptionalType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_rest_type(&mut self, node: TsRestType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        if !self.check_syntax.is_empty() {
            ts::check_tuple_rest_type(self, node);
        }
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsRestType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_js_doc_nullable_type(&mut self, node: TsJsDocNullableType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsJsDocNullableType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_js_doc_non_nullable_type(&mut self, node: TsJsDocNonNullableType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsJsDocNonNullableType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_js_doc_unknown_type(&mut self, node: TsJsDocUnknownType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsJsDocUnknownType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_union_type(&mut self, node: TsUnionType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsUnionType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_intersection_type(&mut self, node: TsIntersectionType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsIntersectionType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_infer_type(&mut self, node: TsInferType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsInferType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_type_operator(&mut self, node: TsTypeOperator) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeOperator(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_parenthesized_type(&mut self, node: TsParenthesizedType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsParenthesizedType(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_type_literal(&mut self, node: TsTypeLiteral) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeLiteral(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_ts_property_signature(&mut self, node: TsPropertySignature) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        self.visit_type_member_key(node.key(self.ast), node.computed(self.ast));
        node.type_annotation(self.ast).visit_with(self);
        self.leave_node(NodeKindData::TsPropertySignature(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_type_predicate(&mut self, node: TsTypePredicate) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .add_ts_type_predicate_reference(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypePredicate(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_conditional_type(&mut self, node: TsConditionalType) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        node.check_type(self.ast).visit_with(self);
        self.state.enter_ts_conditional_type(node.node_id());
        let extends = node.extends_type(self.ast);
        self.state
            .scope
            .enter_ts_lookup_view(extends.node_id(), ScopeKind::TsConditionalExtends);
        extends.visit_with(self);
        self.state.scope.exit_ts_lookup_view(extends.node_id());
        node.true_type(self.ast).visit_with(self);
        self.state.exit_scope(node.node_id());
        node.false_type(self.ast).visit_with(self);
        self.leave_node(NodeKindData::TsConditionalType(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_function_type(&mut self, node: TsFunctionType) {
        self.state.enter_ts_function_type(node.node_id());
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_ts_function_type(&state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsFunctionType(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_constructor_type(&mut self, node: TsConstructorType) {
        self.state.enter_ts_constructor_type(node.node_id());
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_ts_constructor_type(&state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsConstructorType(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_mapped_type(&mut self, node: TsMappedType) {
        self.state.enter_ts_mapped_type(node.node_id());
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let parent = self.parent();
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_mapped_type(self.ast, parent, &state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsMappedType(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_method_signature(&mut self, node: TsMethodSignature) {
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        self.visit_type_member_key(node.key(self.ast), node.computed(self.ast));
        self.state.enter_ts_method_signature(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_ts_method_signature(&state.scope);
        node.type_parameters(self.ast).visit_with(self);
        node.this_param(self.ast).visit_with(self);
        node.params(self.ast).visit_with(self);
        node.return_type(self.ast).visit_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsMethodSignature(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_call_signature_declaration(&mut self, node: TsCallSignatureDeclaration) {
        self.state
            .enter_ts_call_signature_declaration(node.node_id());
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_call_signature_declaration(&state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsCallSignatureDeclaration(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_construct_signature_declaration(&mut self, node: TsConstructSignatureDeclaration) {
        self.state
            .enter_ts_construct_signature_declaration(node.node_id());
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_construct_signature_declaration(&state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsConstructSignatureDeclaration(node));
        self.state.exit_type_position(type_position);
    }

    fn visit_ts_index_signature(&mut self, node: TsIndexSignature) {
        self.state.enter_ts_index_signature(node.node_id());
        let type_position = self.state.enter_type_position();
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_ts_index_signature(&state.scope);
        node.visit_children_with(self);
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsIndexSignature(node));
        self.state.exit_type_position(type_position);
    }

    #[inline]
    fn visit_jsx_identifier(&mut self, _node: JsxIdentifier) {
        self.state.enter_node();
    }

    #[inline]
    fn visit_jsx_text(&mut self, _node: JsxText) {
        self.state.enter_node();
    }

    fn visit_jsx_element(&mut self, node: swc_next_ecma_ast::JsxElement) {
        self.enter_node(node.node_id());
        self.state
            .symbols
            .record_jsx_usage(node.node_id(), self.state.scope.current, false);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::JsxElement(node));
    }

    fn visit_jsx_fragment(&mut self, node: swc_next_ecma_ast::JsxFragment) {
        self.enter_node(node.node_id());
        self.state
            .symbols
            .record_jsx_usage(node.node_id(), self.state.scope.current, true);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::JsxFragment(node));
    }

    fn visit_jsx_opening_element(&mut self, node: JsxOpeningElement) {
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .add_jsx_opening_element_reference(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::JsxOpeningElement(node));
    }

    fn visit_jsx_closing_element(&mut self, node: JsxClosingElement) {
        self.enter_node(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .add_jsx_closing_element_reference(self.ast, node, &state.scope);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::JsxClosingElement(node));
    }
}

fn is_annex_b_if_function(tree: &Ast<'_>, parent: Option<NodeId>, function: Function) -> bool {
    if function.function_type(tree) != FunctionType::FunctionDeclaration {
        return false;
    }
    let Some(parent) = parent else {
        return false;
    };
    let NodeKindData::IfStatement(statement) = tree.kind_data(parent) else {
        return false;
    };
    let function = function.node_id();
    statement.consequent(tree).node_id() == function
        || statement
            .alternate(tree)
            .is_some_and(|alternate| alternate.node_id() == function)
}

fn export_expression_space(tree: &Ast<'_>, expression: NodeId) -> ReferenceSpace {
    let expression = crate::ecmascript::unwrap_parentheses(tree, expression);
    if tree.node_kind(expression) == NodeKind::IdentifierReference {
        ReferenceSpace::Any
    } else {
        ReferenceSpace::Value
    }
}

/// Extracts the name from a ModuleExportName: an IdentifierName,
/// IdentifierReference, or StringLiteral.
fn module_export_name<'b>(tree: &'b Ast<'_>, node: NodeId) -> &'b [u8] {
    match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) => tree.get_utf8(identifier.name(tree)).as_bytes(),
        NodeKindData::IdentifierReference(identifier) => {
            tree.get_utf8(identifier.name(tree)).as_bytes()
        }
        NodeKindData::StringLiteral(literal) => tree.get_wtf8(literal.value(tree)).as_bytes(),
        _ => b"",
    }
}

struct ReportOptions<'a> {
    severity: Severity,
    help: Option<Cow<'a, str>>,
    labels: Vec<Label<'a>>,
}

impl Default for ReportOptions<'_> {
    fn default() -> Self {
        Self {
            severity: Severity::Error,
            help: None,
            labels: Vec::new(),
        }
    }
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    fn report_unresolved_export(&mut self, tree: &Ast<'a>, name: &[u8], node: NodeId) {
        let message = self.unresolved_export_message(name);
        self.report(tree.span(node), message, ReportOptions::default());
    }

    fn report_non_local_export(&mut self, tree: &Ast<'a>, name: &[u8], node: NodeId) {
        let message = self.fmt(format_args!(
            "Cannot export '{}'. Only local declarations can be exported from a module",
            String::from_utf8_lossy(name)
        ));
        self.report(tree.span(node), message, ReportOptions::default());
    }

    fn unresolved_export_message(&self, name: &[u8]) -> String {
        self.fmt(format_args!(
            "Export '{}' is not defined",
            String::from_utf8_lossy(name)
        ))
    }

    #[cold]
    fn report_redeclaration(
        &mut self,
        tree: &Ast<'a>,
        name: &[u8],
        node: NodeId,
        existing_node: NodeId,
        existing_flags: SymbolFlags,
    ) {
        let name = String::from_utf8_lossy(name);
        let current_span = tree.span(node);
        let existing_span = tree.span(existing_node);
        let kind = existing_flags.diagnostic_kind();
        let article = if matches!(kind.as_bytes()[0], b'a' | b'e' | b'i' | b'o' | b'u') {
            "an"
        } else {
            "a"
        };
        let name = name.as_ref();
        let message = ["Identifier '", name, "' has already been declared"].concat();
        let first = [
            "'",
            name,
            "' was first declared as ",
            article,
            " ",
            kind,
            " here",
        ]
        .concat();
        let help = [
            "Consider removing or renaming this declaration of '",
            name,
            "'",
        ]
        .concat();
        self.report(
            current_span,
            message,
            ReportOptions {
                help: Some(Cow::Owned(help)),
                labels: vec![
                    self.label(existing_span, first),
                    self.label(current_span, "cannot be redeclared here"),
                ],
                ..ReportOptions::default()
            },
        );
    }

    #[cold]
    fn report(&mut self, span: Span, message: impl Into<Cow<'a, str>>, options: ReportOptions<'a>) {
        let message = message.into();
        debug_assert!(span.start <= span.end);
        debug_assert!(!message.is_empty());
        self.diagnostics.push(Diagnostic {
            severity: options.severity,
            message,
            span,
            help: options.help,
            labels: options.labels,
        });
    }

    fn label(&self, span: Span, message: impl Into<Cow<'a, str>>) -> Label<'a> {
        Label {
            span,
            message: message.into(),
        }
    }

    fn labels(&self, items: &[Label<'a>]) -> Vec<Label<'a>> {
        items.to_vec()
    }

    fn fmt(&self, arguments: std::fmt::Arguments<'_>) -> String {
        arguments.to_string()
    }
}
