//! The early-error checker, a semantic-traverser visitor that reports
//! the scope-dependent early errors the specification requires.

mod js;
mod ts;

use std::{borrow::Cow, collections::HashMap};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    ArrayPattern, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    AssignmentPattern, Ast, AstInternedUtf8, AstUtf8, AstWtf8, BinaryOperator, BindingIdentifier,
    BindingProperty, BindingRestElement, BlockStatement, CallExpression, CatchClause, Class,
    ClassBody, Decorator, Diagnostic, Directive, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportNamedDeclaration, ExportSpecifier, ForInStatement, ForOfStatement, ForStatement,
    FormalParameters, Function, FunctionBody, FunctionType, IdentifierReference, ImportDeclaration,
    ImportOrExportKind, ImportSpecifier, JsxClosingElement, JsxOpeningElement, Label,
    MemberExpression, NodeId, NodeKind, NodeKindData, ObjectPattern, ParenthesizedExpression,
    PropertyDefinition, PropertyKey, ScopeId, Severity, Span, StaticBlock, SwitchCase,
    SwitchStatement, SymbolId, TsArrayType, TsAsExpression, TsCallSignatureDeclaration,
    TsClassImplements, TsConditionalType, TsConstructSignatureDeclaration, TsConstructorType,
    TsEnumBody, TsEnumDeclaration, TsEnumMember, TsExportAssignment, TsFunction, TsFunctionType,
    TsGlobalDeclaration, TsImportEqualsDeclaration, TsImportType, TsIndexSignature,
    TsIndexedAccessType, TsInferType, TsInterfaceBody, TsInterfaceDeclaration, TsInterfaceHeritage,
    TsIntersectionType, TsJsDocNonNullableType, TsJsDocNullableType, TsJsDocUnknownType,
    TsLiteralType, TsMappedType, TsMethodDefinition, TsMethodSignature, TsModuleBlock,
    TsModuleDeclaration, TsNamedTupleMember, TsNamespaceExportDeclaration, TsNonNullExpression,
    TsOptionalType, TsParenthesizedType, TsPropertySignature, TsQualifiedName, TsRestType,
    TsSatisfiesExpression, TsTemplateLiteralType, TsTupleType, TsTypeAliasDeclaration,
    TsTypeAnnotation, TsTypeAssertion, TsTypeLiteral, TsTypeOperator, TsTypeParameter,
    TsTypeParameterDeclaration, TsTypeParameterInstantiation, TsTypePredicate, TsTypeQuery,
    TsTypeReference, TsUnionType, UnaryExpression, UnaryOperator, UpdateExpression,
    VariableDeclaration, VariableDeclarator, VariableKind, Visit, VisitWith, WithStatement,
};

use crate::{
    ReferenceFlags, ReferenceSpace, Semantic, Symbol, SymbolFlags,
    context::{SemanticContext, SemanticVisitCtx},
    ecmascript,
    scope::ScopeKind,
};

pub struct Checker<'traverse, 'a> {
    checks: CheckerState<'a>,
    context: SemanticVisitCtx<'traverse, 'a>,
    check_syntax: bool,
    in_ts_module_name: bool,
}

struct CheckerState<'a> {
    exported_names: HashMap<Vec<u8>, NodeId>,
    export_specifiers: Vec<ExportSpecifierInfo>,
    diagnostics: Vec<Diagnostic<'a>>,
}

#[derive(Clone, Copy)]
enum ImportAttributeName {
    Utf8(AstUtf8),
    Wtf8(AstWtf8),
}

impl ImportAttributeName {
    fn eq(self, other: Self, ast: &Ast<'_>) -> bool {
        match (self, other) {
            (Self::Utf8(left), Self::Utf8(right)) => ast.utf8_eq(left, right),
            (Self::Utf8(utf8), Self::Wtf8(wtf8)) | (Self::Wtf8(wtf8), Self::Utf8(utf8)) => {
                ast.utf8_eq_wtf8(utf8, wtf8)
            }
            (Self::Wtf8(left), Self::Wtf8(right)) => ast.get_wtf8(left) == ast.get_wtf8(right),
        }
    }
}

impl<'traverse, 'a> Checker<'traverse, 'a> {
    pub fn new(ast: &'traverse mut Ast<'a>, check_syntax: bool, build_module_record: bool) -> Self {
        let state = SemanticContext::init(ast, build_module_record);
        Self {
            checks: CheckerState::new(),
            context: SemanticVisitCtx::new(ast, state),
            check_syntax,
            in_ts_module_name: false,
        }
    }

    pub fn finish(self) -> (Semantic<'a>, Vec<Diagnostic<'a>>) {
        let semantic = self.context.finalize();
        (semantic, self.checks.into_diagnostics())
    }

    #[inline]
    fn visit_with_reference_flags<T>(&mut self, node: T, flags: ReferenceFlags)
    where
        T: VisitWith<'a, Self>,
    {
        let previous = self.context.state.replace_reference_flags(flags);
        node.visit_with(self);
        self.context.state.replace_reference_flags(previous);
    }

    #[inline]
    fn assignment_reference_flags(&self, read: bool) -> ReferenceFlags {
        let flags = self.context.state.default_reference_flags();
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
        self.context
            .state
            .default_reference_flags()
            .with_space(space)
    }

    #[inline]
    fn visit_type_member_key(&mut self, key: PropertyKey, computed: bool) {
        if computed {
            // A computed name on a type-only member still resolves a value
            // name (including a type-only import), but is never evaluated.
            let flags = self.flags_with_space(ReferenceSpace::Typeof);
            self.visit_with_reference_flags(key, flags);
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

    fn visit_dynamic_var_declarator(&mut self, node: VariableDeclarator, iteration_write: bool) {
        self.context.enter_node(node.node_id());
        let id = node.id(self.context.ast);
        let init = node.init(self.context.ast);
        let dynamic_write = iteration_write || init.is_some();
        if dynamic_write {
            let flags = self.assignment_reference_flags(false);
            self.visit_with_reference_flags(id, flags);
        } else {
            id.visit_with(self);
        }
        init.visit_with(self);
        self.leave_node(NodeKindData::VariableDeclarator(node));
    }
}

fn export_expression_space(tree: &Ast<'_>, expression: NodeId) -> ReferenceSpace {
    if tree.node_kind(expression) == NodeKind::IdentifierReference {
        ReferenceSpace::Any
    } else {
        ReferenceSpace::Value
    }
}

fn is_direct_eval_call(tree: &Ast<'_>, call: CallExpression) -> bool {
    if call.optional(tree) {
        return false;
    }

    let mut callee = call.callee(tree).node_id();
    loop {
        callee = match tree.kind_data(callee) {
            NodeKindData::IdentifierReference(identifier) => {
                return tree.utf8_eq_bytes(identifier.name(tree), b"eval");
            }
            NodeKindData::ParenthesizedExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsAsExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsSatisfiesExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsTypeAssertion(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsNonNullExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsInstantiationExpression(wrapper) => wrapper.expression(tree).node_id(),
            _ => return false,
        };
    }
}

struct ExportSpecifierInfo {
    local_name: ModuleExportNameValue,
    node: NodeId,
}

#[derive(Clone, Copy)]
enum ModuleExportNameValue {
    Utf8(AstUtf8),
    Wtf8(AstWtf8),
}

impl ModuleExportNameValue {
    fn bytes<'b>(self, tree: &'b Ast<'_>) -> Cow<'b, [u8]> {
        match self {
            Self::Utf8(name) => {
                let chunks = tree.utf8_chunks(name);
                if chunks.suffix().is_none() {
                    Cow::Borrowed(chunks.prefix().as_bytes())
                } else {
                    Cow::Owned(chunks.bytes().collect())
                }
            }
            Self::Wtf8(name) => Cow::Borrowed(tree.get_wtf8(name).as_bytes()),
        }
    }
}

impl<'a> CheckerState<'a> {
    pub fn new() -> Self {
        Self {
            exported_names: HashMap::new(),
            export_specifiers: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic<'a>> {
        self.diagnostics
    }
}

impl Checker<'_, '_> {
    #[inline(always)]
    fn check(&mut self, node_data: NodeKindData) {
        match node_data {
            NodeKindData::Program(_) => js::check_program(self),
            NodeKindData::StringLiteral(node) => js::check_string_literal(self, node),
            NodeKindData::NumericLiteral(node) => js::check_numeric_literal(self, node),
            NodeKindData::RegExpLiteral(node) => js::check_reg_exp_literal(self, node),
            NodeKindData::IdentifierReference(node) => js::check_identifier_reference(self, node),
            NodeKindData::BindingIdentifier(node) => js::check_binding_identifier(self, node),
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
            NodeKindData::Function(node) => js::check_function(self, node),
            NodeKindData::WithStatement(node) => js::check_with_statement(self, node),
            NodeKindData::ObjectExpression(node) => js::check_object_expression(self, node),
            NodeKindData::MemberExpression(node) => js::check_member_expression(self, node),
            NodeKindData::CallExpression(node) => js::check_call_expression(self, node),
            NodeKindData::AwaitExpression(node) => js::check_await_expression(self, node),
            NodeKindData::YieldExpression(node) => js::check_yield_expression(self, node),
            NodeKindData::MetaProperty(node) => js::check_meta_property(self, node),
            NodeKindData::ImportDeclaration(node) => js::check_import_declaration(self, node),
            NodeKindData::ExportNamedDeclaration(node) => {
                js::check_export_named_declaration(self, node)
            }
            NodeKindData::ExportDefaultDeclaration(node) => {
                js::check_export_default_declaration(self, node)
            }
            NodeKindData::ExportAllDeclaration(node) => {
                js::check_export_all_declaration(self, node)
            }
            NodeKindData::ExportSpecifier(node) => js::check_export_specifier(self, node),
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Checker<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.context.ast
    }

    #[inline(always)]
    fn enter_node(&mut self, node: NodeKindData) {
        self.context.enter_node(node.node_id());
    }

    #[inline(always)]
    fn leave_node(&mut self, node: NodeKindData) {
        if self.check_syntax {
            self.check(node);
        }
        self.context.leave_node(node.node_id());
    }

    #[inline]
    fn visit_call_expression(&mut self, node: CallExpression) {
        self.context.enter_node(node.node_id());
        if !self.context.state.symbols.in_ambient_context()
            && self.context.state.default_reference_flags().is_read()
            && is_direct_eval_call(self.context.ast, node)
        {
            self.context.state.scope.mark_direct_eval();
        }
        node.visit_children_with(self);
        self.leave_node(NodeKindData::CallExpression(node));
    }

    #[inline]
    fn visit_update_expression(&mut self, node: UpdateExpression) {
        self.context.enter_node(node.node_id());
        let argument = node.argument(self.context.ast);
        let flags = self.assignment_reference_flags(true);
        self.visit_with_reference_flags(argument, flags);
        self.leave_node(NodeKindData::UpdateExpression(node));
    }

    #[inline]
    fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
        self.context.enter_node(node.node_id());
        let left = node.left(self.context.ast);
        let read = node.operator(self.context.ast) != AssignmentOperator::Assign;
        let flags = self.assignment_reference_flags(read);
        self.visit_with_reference_flags(left, flags);
        node.right(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::AssignmentExpression(node));
    }

    #[inline]
    fn visit_assignment_pattern(&mut self, node: AssignmentPattern) {
        let node_flags = self.context.enter_node(node.node_id());
        node.decorators(self.context.ast).visit_with(self);
        self.visit_with_reference_flags(node.left(self.context.ast), node_flags);
        node.type_annotation(self.context.ast).visit_with(self);
        node.right(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::AssignmentPattern(node));
    }

    #[inline]
    fn visit_binding_rest_element(&mut self, node: BindingRestElement) {
        let node_flags = self.context.enter_node(node.node_id());
        node.decorators(self.context.ast).visit_with(self);
        self.visit_with_reference_flags(node.argument(self.context.ast), node_flags);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::BindingRestElement(node));
    }

    #[inline]
    fn visit_array_pattern(&mut self, node: ArrayPattern) {
        let node_flags = self.context.enter_node(node.node_id());
        node.decorators(self.context.ast).visit_with(self);
        for element in node.elements(self.context.ast).iter() {
            if let Some(element) = self.context.ast.get_node_in_sub_range(element) {
                self.visit_with_reference_flags(element, node_flags);
            }
        }
        if let Some(rest) = node.rest(self.context.ast) {
            self.visit_with_reference_flags(rest, node_flags);
        }
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::ArrayPattern(node));
    }

    #[inline]
    fn visit_object_pattern(&mut self, node: ObjectPattern) {
        let node_flags = self.context.enter_node(node.node_id());
        node.decorators(self.context.ast).visit_with(self);
        for property in node.properties(self.context.ast).iter() {
            let property = self.context.ast.get_node_in_sub_range(property);
            self.visit_with_reference_flags(property, node_flags);
        }
        if let Some(rest) = node.rest(self.context.ast) {
            self.visit_with_reference_flags(rest, node_flags);
        }
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::ObjectPattern(node));
    }

    #[inline]
    fn visit_binding_property(&mut self, node: BindingProperty) {
        let node_flags = self.context.enter_node(node.node_id());
        node.key(self.context.ast).visit_with(self);
        self.visit_with_reference_flags(node.value(self.context.ast), node_flags);
        self.leave_node(NodeKindData::BindingProperty(node));
    }

    #[inline]
    fn visit_property_definition(&mut self, node: PropertyDefinition) {
        self.context.enter_node(node.node_id());
        let decorators = node.decorators(self.context.ast);
        decorators.visit_with(self);
        let key = node.key(self.context.ast);
        let erased = node.declare(self.context.ast) || node.r#abstract(self.context.ast);
        if erased {
            self.visit_erased_member_key(
                key,
                node.computed(self.context.ast),
                !decorators.is_empty(),
            );
        } else {
            key.visit_with(self);
        }
        node.type_annotation(self.context.ast).visit_with(self);
        node.value(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::PropertyDefinition(node));
    }

    #[inline]
    fn visit_ts_method_definition(&mut self, node: TsMethodDefinition) {
        self.context.enter_node(node.node_id());
        let decorators = node.decorators(self.context.ast);
        decorators.visit_with(self);
        self.visit_erased_member_key(
            node.key(self.context.ast),
            node.computed(self.context.ast),
            !decorators.is_empty(),
        );
        node.value(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::TsMethodDefinition(node));
    }

    #[inline]
    fn visit_member_expression(&mut self, node: MemberExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        // Assignment writes the property, not its base. Both the base and a
        // computed property are evaluated as reads.
        let object_flags = self.flags_with_space(node_flags.space);
        self.visit_with_reference_flags(node.object(self.context.ast), object_flags);
        node.property(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::MemberExpression(node));
    }

    #[inline]
    fn visit_parenthesized_expression(&mut self, node: ParenthesizedExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        self.leave_node(NodeKindData::ParenthesizedExpression(node));
    }

    #[inline]
    fn visit_ts_as_expression(&mut self, node: TsAsExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::TsAsExpression(node));
    }

    #[inline]
    fn visit_ts_satisfies_expression(&mut self, node: TsSatisfiesExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::TsSatisfiesExpression(node));
    }

    #[inline]
    fn visit_ts_type_assertion(&mut self, node: TsTypeAssertion) {
        let node_flags = self.context.enter_node(node.node_id());
        node.type_annotation(self.context.ast).visit_with(self);
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        self.leave_node(NodeKindData::TsTypeAssertion(node));
    }

    #[inline]
    fn visit_ts_non_null_expression(&mut self, node: TsNonNullExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        self.leave_node(NodeKindData::TsNonNullExpression(node));
    }

    #[inline]
    fn visit_unary_expression(&mut self, node: UnaryExpression) {
        self.context.enter_node(node.node_id());
        let argument = node.argument(self.context.ast);
        if node.operator(self.context.ast) == UnaryOperator::Delete {
            let flags = self.context.state.default_reference_flags().no_access();
            self.visit_with_reference_flags(argument, flags);
        } else {
            argument.visit_with(self);
        }
        self.leave_node(NodeKindData::UnaryExpression(node));
    }

    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        let flags = self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let state = &mut context.state;
        state
            .symbols
            .add_identifier_reference(context.ast, node, &state.scope, flags);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::IdentifierReference(node));
    }

    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        let flags = self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::BindingIdentifier(node));
        let direct_export = self.context.state.symbols.export_state.named();
        let direct_export = direct_export.filter(|_| !self.context.state.in_ts_namespace());
        let state = &mut self.context.state;
        let declaration = state.symbols.declare_binding_identifier(
            self.context.ast,
            node,
            &state.scope,
            flags.space,
        );
        if declaration.is_some() && flags.is_write_only() {
            state
                .symbols
                .add_var_write_reference(self.context.ast, node, &state.scope, flags);
        }
        if declaration.is_some()
            && let Some(statement) = direct_export
        {
            state
                .module_record
                .add_direct_export(self.context.ast, node.node_id(), statement);
        }
    }

    fn visit_variable_declaration(&mut self, node: VariableDeclaration) {
        let node_flags = self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        let dynamic_var_write =
            state
                .symbols
                .enter_variable_declaration(self.context.ast, node, &state.scope);
        if dynamic_var_write {
            let iteration_write = node_flags.is_write();
            for declarator in node.declarators(self.context.ast).iter() {
                let declarator = self.context.ast.get_node_in_sub_range(declarator);
                self.visit_dynamic_var_declarator(declarator, iteration_write);
            }
        } else {
            node.visit_children_with(self);
        }
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::VariableDeclaration(node));
    }

    /// Per section 14.12, the CaseBlock grammar permits at most one DefaultClause.
    fn visit_switch_statement(&mut self, node: SwitchStatement) {
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::SwitchStatement(node));
        self.context.state.exit_scope(node.node_id());
    }

    /// <https://tc39.es/ecma262/#sec-for-in-and-for-of-statements-static-semantics-early-errors>
    fn visit_for_in_statement(&mut self, node: ForInStatement) {
        self.context.state.enter_for_in_statement(node.node_id());
        self.context.enter_node(node.node_id());
        let flags = self.assignment_reference_flags(false);
        self.visit_with_reference_flags(node.left(self.context.ast), flags);
        node.right(self.context.ast).visit_with(self);
        node.body(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::ForInStatement(node));
        self.context.state.exit_scope(node.node_id());
    }

    /// <https://tc39.es/ecma262/#sec-for-in-and-for-of-statements-static-semantics-early-errors>
    fn visit_for_of_statement(&mut self, node: ForOfStatement) {
        self.context.state.enter_for_of_statement(node.node_id());
        self.context.enter_node(node.node_id());
        let flags = self.assignment_reference_flags(false);
        self.visit_with_reference_flags(node.left(self.context.ast), flags);
        node.right(self.context.ast).visit_with(self);
        node.body(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::ForOfStatement(node));
        self.context.state.exit_scope(node.node_id());
    }

    /// <https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors>
    ///
    /// "It is a Syntax Error if FunctionBodyContainsUseStrict is true and
    /// IsSimpleParameterList of FormalParameters is false."
    fn visit_directive(&mut self, node: Directive) {
        self.context.state.enter_directive(self.context.ast, node);
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::Directive(node));
    }

    /// 15.7.1 Static Semantics: Early Errors.
    /// <https://tc39.es/ecma262/#sec-class-definitions-static-semantics-early-errors>
    ///
    /// `PrototypePropertyNameList` may contain only one `constructor`.
    /// `PrivateBoundIdentifiers` may contain no duplicate entries, except
    /// one getter/setter pair with matching staticness.
    fn visit_class_body(&mut self, node: ClassBody) {
        self.context.enter_node(node.node_id());
        self.context.state.symbols.enter_class_body();
        node.visit_children_with(self);
        self.leave_node(NodeKindData::ClassBody(node));
    }

    // https://tc39.es/ecma262/#sec-labelled-statements-static-semantics-early-errors (14.13.1)
    //   LabelledItem : FunctionDeclaration - syntax error unless non-strict + web host (B.3.1).
    // https://tc39.es/ecma262/#sec-if-statement-static-semantics-early-errors (14.6.1)
    //   IsLabelledFunction prevents labelled functions in if/iteration/with bodies.
    // https://tc39.es/ecma262/#sec-functiondeclarations-in-ifstatement-statement-clauses (B.3.3)
    //   Bare FunctionDeclaration in if/else is allowed in non-strict code only.
    fn visit_function(&mut self, node: Function) {
        let annex_b_if_body =
            is_annex_b_if_function(self.context.ast, self.context.current_node(), node);
        self.context
            .state
            .enter_function(self.context.ast, node, annex_b_if_body);
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_function(self.context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::Function(node));
        self.context.state.exit_scope(node.node_id());
    }

    // https://tc39.es/ecma262/#sec-static-semantics-early-errors
    fn visit_import_declaration(&mut self, node: ImportDeclaration) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_import_declaration(self.context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::ImportDeclaration(node));
    }

    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        self.context.enter_node(node.node_id());
        self.context
            .state
            .symbols
            .enter_export_named_declaration(self.context.ast, node);
        node.visit_children_with(self);
        self.context.state.symbols.exit_export_declaration();
        self.leave_node(NodeKindData::ExportNamedDeclaration(node));
    }

    fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        self.context.enter_node(node.node_id());
        if !self.context.state.in_ts_namespace() {
            self.context
                .state
                .module_record
                .add_default_export(self.context.ast, node);
        }
        self.context
            .state
            .symbols
            .enter_export_default_declaration();
        let declaration = node.declaration(self.context.ast);
        let space = declaration
            .as_expr(self.context.ast)
            .map_or(ReferenceSpace::Value, |expression| {
                export_expression_space(self.context.ast, expression.node_id())
            });
        let flags = self.flags_with_space(space);
        self.visit_with_reference_flags(declaration, flags);
        self.context.state.symbols.exit_export_declaration();
        self.leave_node(NodeKindData::ExportDefaultDeclaration(node));
    }

    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        self.context.enter_node(node.node_id());
        if !self.context.state.in_ts_namespace() {
            self.context
                .state
                .module_record
                .add_export_all(self.context.ast, node);
        }
        node.visit_children_with(self);
        self.leave_node(NodeKindData::ExportAllDeclaration(node));
    }

    #[inline]
    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        self.context.enter_node(node.node_id());
        let declaration = self.context.parent().and_then(|parent| {
            let NodeKindData::ExportNamedDeclaration(declaration) =
                self.context.ast.kind_data(parent)
            else {
                return None;
            };
            Some(declaration)
        });
        if !self.context.state.in_ts_namespace()
            && let Some(declaration) = declaration
        {
            self.context
                .state
                .module_record
                .add_specifier(self.context.ast, declaration, node);
        }
        let type_only = node.export_kind(self.context.ast) == ImportOrExportKind::Type
            || declaration.is_some_and(|declaration| {
                declaration.export_kind(self.context.ast) == ImportOrExportKind::Type
            });
        let mut flags = self.flags_with_space(ReferenceSpace::Any);
        if type_only {
            flags = flags.no_access();
        }
        if declaration.is_some_and(|declaration| declaration.source(self.context.ast).is_none()) {
            let state = &mut self.context.state;
            state
                .symbols
                .add_local_export_reference(self.context.ast, node, &state.scope, flags);
        }
        self.visit_with_reference_flags(node.local(self.context.ast), flags);
        node.exported(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::ExportSpecifier(node));
    }

    fn visit_formal_parameters(&mut self, node: FormalParameters) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_formal_parameters(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_formal_parameters();
        self.leave_node(NodeKindData::FormalParameters(node));
    }

    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        self.context.enter_node(node.node_id());
        self.context
            .state
            .symbols
            .enter_import_specifier(self.context.ast, node);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::ImportSpecifier(node));
    }

    fn visit_ts_import_equals_declaration(&mut self, node: TsImportEqualsDeclaration) {
        self.context.enter_node(node.node_id());
        let import_kind = node.import_kind(self.context.ast);
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_import_equals_declaration(import_kind, &state.scope);
        let mut flags = self.flags_with_space(ReferenceSpace::Any);
        if import_kind == ImportOrExportKind::Type {
            flags = flags.no_access();
        }
        self.visit_with_reference_flags(node.id(self.context.ast), flags);
        self.visit_with_reference_flags(node.module_reference(self.context.ast), flags);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsImportEqualsDeclaration(node));
    }

    #[inline]
    fn visit_ts_export_assignment(&mut self, node: TsExportAssignment) {
        self.context.enter_node(node.node_id());
        let expression = node.expression(self.context.ast);
        let space = export_expression_space(self.context.ast, expression.node_id());
        let flags = self.flags_with_space(space);
        self.visit_with_reference_flags(expression, flags);
        self.leave_node(NodeKindData::TsExportAssignment(node));
    }

    fn visit_ts_enum_declaration(&mut self, node: TsEnumDeclaration) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_enum_declaration(self.context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsEnumDeclaration(node));
    }

    fn visit_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_module_declaration(self.context.ast, node, &state.scope);
        let was_in_ts_module_name = std::mem::replace(&mut self.in_ts_module_name, true);
        node.id(self.context.ast).visit_with(self);
        self.in_ts_module_name = was_in_ts_module_name;
        node.body(self.context.ast).visit_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsModuleDeclaration(node));
    }

    fn visit_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        self.context.enter_node(node.node_id());
        self.context.state.symbols.enter_ts_global_declaration();
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsGlobalDeclaration(node));
    }

    fn visit_ts_namespace_export_declaration(&mut self, node: TsNamespaceExportDeclaration) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_namespace_export_declaration(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsNamespaceExportDeclaration(node));
    }

    #[inline]
    fn visit_ts_type_annotation(&mut self, node: TsTypeAnnotation) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeAnnotation(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_type_reference(&mut self, node: TsTypeReference) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeReference(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_qualified_name(&mut self, node: TsQualifiedName) {
        if self.in_ts_module_name {
            self.context.enter_node(node.node_id());
            node.visit_children_with(self);
            self.leave_node(NodeKindData::TsQualifiedName(node));
            return;
        }
        self.context.state.enter_type_position();
        let node_flags = self.context.enter_node(node.node_id());
        let space = match node_flags.space {
            ReferenceSpace::Value | ReferenceSpace::Type | ReferenceSpace::Namespace => {
                ReferenceSpace::Namespace
            }
            space => space,
        };
        let flags = node_flags.with_space(space);
        self.visit_with_reference_flags(node.left(self.context.ast), flags);
        node.right(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::TsQualifiedName(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_type_query(&mut self, node: TsTypeQuery) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let flags = self.flags_with_space(ReferenceSpace::Typeof);
        self.visit_with_reference_flags(node.expr_name(self.context.ast), flags);
        node.type_arguments(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::TsTypeQuery(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_import_type(&mut self, node: TsImportType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsImportType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_type_parameter(&mut self, node: TsTypeParameter) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let parent = self.context.parent();
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_type_parameter(self.context.ast, parent, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(NodeKindData::TsTypeParameter(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_type_parameter_declaration(&mut self, node: TsTypeParameterDeclaration) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeParameterDeclaration(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_type_parameter_instantiation(&mut self, node: TsTypeParameterInstantiation) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeParameterInstantiation(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_literal_type(&mut self, node: TsLiteralType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsLiteralType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_template_literal_type(&mut self, node: TsTemplateLiteralType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTemplateLiteralType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_array_type(&mut self, node: TsArrayType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsArrayType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_indexed_access_type(&mut self, node: TsIndexedAccessType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsIndexedAccessType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_tuple_type(&mut self, node: TsTupleType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTupleType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_named_tuple_member(&mut self, node: TsNamedTupleMember) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsNamedTupleMember(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_optional_type(&mut self, node: TsOptionalType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsOptionalType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_rest_type(&mut self, node: TsRestType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsRestType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_js_doc_nullable_type(&mut self, node: TsJsDocNullableType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsJsDocNullableType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_js_doc_non_nullable_type(&mut self, node: TsJsDocNonNullableType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsJsDocNonNullableType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_js_doc_unknown_type(&mut self, node: TsJsDocUnknownType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsJsDocUnknownType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_union_type(&mut self, node: TsUnionType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsUnionType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_intersection_type(&mut self, node: TsIntersectionType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsIntersectionType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_infer_type(&mut self, node: TsInferType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsInferType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_type_operator(&mut self, node: TsTypeOperator) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeOperator(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_parenthesized_type(&mut self, node: TsParenthesizedType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsParenthesizedType(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_type_literal(&mut self, node: TsTypeLiteral) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypeLiteral(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_property_signature(&mut self, node: TsPropertySignature) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        self.visit_type_member_key(node.key(self.context.ast), node.computed(self.context.ast));
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::TsPropertySignature(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_class_implements(&mut self, node: TsClassImplements) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsClassImplements(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_interface_heritage(&mut self, node: TsInterfaceHeritage) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsInterfaceHeritage(node));
        self.context.state.exit_type_position();
    }

    #[inline]
    fn visit_ts_interface_body(&mut self, node: TsInterfaceBody) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsInterfaceBody(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_type_predicate(&mut self, node: TsTypePredicate) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let state = &mut context.state;
        state
            .symbols
            .add_ts_type_predicate_reference(context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsTypePredicate(node));
        self.context.state.exit_type_position();
    }

    fn visit_switch_case(&mut self, node: SwitchCase) {
        self.context
            .state
            .enter_switch_case(self.context.current_node());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(NodeKindData::SwitchCase(node));
    }

    fn visit_for_statement(&mut self, node: ForStatement) {
        self.context.state.enter_for_statement(node.node_id());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::ForStatement(node));
    }

    fn visit_catch_clause(&mut self, node: CatchClause) {
        self.context.state.enter_catch_clause(node.node_id());
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_catch_clause(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::CatchClause(node));
    }

    fn visit_decorator(&mut self, node: Decorator) {
        self.context.state.enter_decorator();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_decorator();
        self.leave_node(NodeKindData::Decorator(node));
    }

    fn visit_class(&mut self, node: Class) {
        self.context.state.enter_class(self.context.ast, node);
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_class(self.context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::Class(node));
    }

    fn visit_static_block(&mut self, node: StaticBlock) {
        self.context.state.enter_static_block(node.node_id());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::StaticBlock(node));
    }

    fn visit_function_body(&mut self, node: FunctionBody) {
        let parent = self.context.current_node();
        self.context
            .state
            .enter_function_body(self.context.ast, node, parent);
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::FunctionBody(node));
    }

    fn visit_block_statement(&mut self, node: BlockStatement) {
        self.context
            .state
            .enter_block_statement(self.context.ast, node.node_id());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::BlockStatement(node));
    }

    fn visit_with_statement(&mut self, node: WithStatement) {
        self.context.enter_node(node.node_id());
        node.object(self.context.ast).visit_with(self);
        self.context.state.enter_with_statement(node.node_id());
        node.body(self.context.ast).visit_with(self);
        self.context.state.exit_with_statement(node.node_id());
        self.leave_node(NodeKindData::WithStatement(node));
    }

    fn visit_arrow_function_expression(&mut self, node: ArrowFunctionExpression) {
        self.context
            .state
            .enter_arrow_function_expression(self.context.ast, node);
        let export_state = self.context.state.symbols.enter_arrow_function_expression();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context
            .state
            .symbols
            .exit_arrow_function_expression(export_state);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::ArrowFunctionExpression(node));
    }

    fn visit_ts_function(&mut self, node: TsFunction) {
        self.context.state.enter_ts_function(node.node_id());
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_ts_function(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsFunction(node));
    }

    fn visit_ts_conditional_type(&mut self, node: TsConditionalType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.check_type(self.context.ast).visit_with(self);
        self.context.state.enter_ts_conditional_type(node.node_id());
        node.extends_type(self.context.ast).visit_with(self);
        node.true_type(self.context.ast).visit_with(self);
        self.context.state.exit_scope(node.node_id());
        node.false_type(self.context.ast).visit_with(self);
        self.leave_node(NodeKindData::TsConditionalType(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_function_type(&mut self, node: TsFunctionType) {
        self.context.state.enter_ts_function_type(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_ts_function_type(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsFunctionType(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_constructor_type(&mut self, node: TsConstructorType) {
        self.context.state.enter_ts_constructor_type(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_ts_constructor_type(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsConstructorType(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_mapped_type(&mut self, node: TsMappedType) {
        self.context.state.enter_ts_mapped_type(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let parent = self.context.parent();
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_mapped_type(self.context.ast, parent, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsMappedType(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_method_signature(&mut self, node: TsMethodSignature) {
        self.context.state.enter_ts_method_signature(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_ts_method_signature(&state.scope);
        self.visit_type_member_key(node.key(self.context.ast), node.computed(self.context.ast));
        node.type_parameters(self.context.ast).visit_with(self);
        node.params(self.context.ast).visit_with(self);
        node.return_type(self.context.ast).visit_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsMethodSignature(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_call_signature_declaration(&mut self, node: TsCallSignatureDeclaration) {
        self.context
            .state
            .enter_ts_call_signature_declaration(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_call_signature_declaration(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsCallSignatureDeclaration(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_construct_signature_declaration(&mut self, node: TsConstructSignatureDeclaration) {
        self.context
            .state
            .enter_ts_construct_signature_declaration(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_construct_signature_declaration(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsConstructSignatureDeclaration(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_index_signature(&mut self, node: TsIndexSignature) {
        self.context.state.enter_ts_index_signature(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_ts_index_signature(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsIndexSignature(node));
        self.context.state.exit_type_position();
    }

    fn visit_ts_type_alias_declaration(&mut self, node: TsTypeAliasDeclaration) {
        self.context
            .state
            .enter_ts_type_alias_declaration(node.node_id());
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_type_alias_declaration(self.context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsTypeAliasDeclaration(node));
    }

    fn visit_ts_interface_declaration(&mut self, node: TsInterfaceDeclaration) {
        self.context
            .state
            .enter_ts_interface_declaration(node.node_id());
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_interface_declaration(self.context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsInterfaceDeclaration(node));
    }

    fn visit_ts_enum_body(&mut self, node: TsEnumBody) {
        self.context.state.enter_ts_enum_body(node.node_id());
        self.context.enter_node(node.node_id());
        self.context.state.symbols.enter_ts_enum_body();
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(NodeKindData::TsEnumBody(node));
    }

    fn visit_ts_enum_member(&mut self, node: TsEnumMember) {
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let state = &mut context.state;
        state
            .symbols
            .declare_ts_enum_member(context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::TsEnumMember(node));
    }

    fn visit_ts_module_block(&mut self, node: TsModuleBlock) {
        self.context.state.enter_ts_module_block(node.node_id());
        self.context.enter_node(node.node_id());
        self.context.state.symbols.enter_ts_module_block();
        node.visit_children_with(self);
        self.context.state.exit_ts_module_block(node.node_id());
        self.leave_node(NodeKindData::TsModuleBlock(node));
    }

    fn visit_jsx_opening_element(&mut self, node: JsxOpeningElement) {
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let state = &mut context.state;
        state
            .symbols
            .add_jsx_opening_element_reference(context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.leave_node(NodeKindData::JsxOpeningElement(node));
    }

    fn visit_jsx_closing_element(&mut self, node: JsxClosingElement) {
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let state = &mut context.state;
        state
            .symbols
            .add_jsx_closing_element_reference(context.ast, node, &state.scope);
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
