//! The early-error checker, a semantic-traverser visitor that reports
//! the scope-dependent early errors the specification requires.

use std::{borrow::Cow, fmt::Write};

use swc_next_allocator::{
    Allocator, CloneIn, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{
    ArrayPattern, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    AssignmentPattern, Ast, BinaryOperator, BindingIdentifier, BindingProperty, BindingRestElement,
    BlockStatement, CatchClause, Class, ClassBody, Decorator, Diagnostic, Directive,
    ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, ExportSpecifier,
    ForInStatement, ForOfStatement, ForStatement, FormalParameters, Function, FunctionBody,
    FunctionType, IdentifierReference, ImportDeclaration, ImportOrExportKind, ImportSpecifier,
    JsxClosingElement, JsxOpeningElement, Label, MemberExpression, NodeId, NodeKind, NodeKindData,
    ObjectPattern, ParenthesizedExpression, PropertyDefinition, PropertyKey, ScopeId, Severity,
    Span, StaticBlock, SwitchCase, SwitchStatement, SymbolId, TsArrayType, TsAsExpression,
    TsCallSignatureDeclaration, TsClassImplements, TsConditionalType,
    TsConstructSignatureDeclaration, TsConstructorType, TsEnumBody, TsEnumDeclaration,
    TsEnumMember, TsExportAssignment, TsFunction, TsFunctionType, TsGlobalDeclaration,
    TsImportEqualsDeclaration, TsImportType, TsIndexSignature, TsIndexedAccessType, TsInferType,
    TsInterfaceBody, TsInterfaceDeclaration, TsInterfaceHeritage, TsIntersectionType,
    TsJsDocNonNullableType, TsJsDocNullableType, TsJsDocUnknownType, TsLiteralType, TsMappedType,
    TsMethodDefinition, TsMethodSignature, TsModuleBlock, TsModuleDeclaration, TsNamedTupleMember,
    TsNamespaceExportDeclaration, TsNonNullExpression, TsOptionalType, TsParenthesizedType,
    TsPropertySignature, TsQualifiedName, TsRestType, TsSatisfiesExpression, TsTemplateLiteralType,
    TsTupleType, TsTypeAliasDeclaration, TsTypeAnnotation, TsTypeAssertion, TsTypeLiteral,
    TsTypeOperator, TsTypeParameter, TsTypeParameterDeclaration, TsTypeParameterInstantiation,
    TsTypePredicate, TsTypeQuery, TsTypeReference, TsUnionType, UnaryExpression, UnaryOperator,
    UpdateExpression, VariableDeclaration, VariableDeclarator, VariableKind, Visit, VisitWith,
    WithStatement,
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
    allocator: &'a Allocator,
    exported_names: ArenaHashMap<'a, &'a [u8], NodeId>,
    export_specifiers: ArenaVec<'a, ExportSpecifierInfo<'a>>,
    diagnostics: Vec<Diagnostic<'a>>,
}

impl<'traverse, 'a> Checker<'traverse, 'a> {
    pub fn new(ast: &'traverse Ast<'a>, check_syntax: bool) -> Self {
        Self {
            checks: CheckerState::new(ast.allocator()),
            context: SemanticVisitCtx::new(ast, SemanticContext::init(ast)),
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
        self.leave_node(node.node_id());
    }
}

fn export_expression_space(tree: &Ast<'_>, expression: NodeId) -> ReferenceSpace {
    if matches!(
        tree.kind_data(expression),
        NodeKindData::IdentifierReference(_)
    ) {
        ReferenceSpace::Any
    } else {
        ReferenceSpace::Value
    }
}

#[derive(Clone, Copy)]
struct ExportSpecifierInfo<'a> {
    local_name: &'a [u8],
    node: NodeId,
}

impl<'a> CheckerState<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            exported_names: ArenaHashMap::new_in(allocator),
            export_specifiers: ArenaVec::new_in(allocator),
            diagnostics: Vec::new(),
        }
    }

    /// Post-traversal check that all local export specifiers refer to
    /// declared bindings.
    pub fn check_unresolved_exports(
        &mut self,
        tree: &Ast<'a>,
        symbols: &crate::binder::SymbolTracker<'a>,
    ) {
        if !tree.is_module() {
            return;
        }
        for index in 0..self.export_specifiers.len() {
            let specifier = self.export_specifiers[index];
            if symbols
                .binding(ScopeId::MODULE, specifier.local_name)
                .is_none()
            {
                let message = self.fmt(format_args!(
                    "Export '{}' is not defined",
                    String::from_utf8_lossy(specifier.local_name)
                ));
                self.report(tree.span(specifier.node), message, ReportOptions::default());
            }
        }
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic<'a>> {
        self.diagnostics
    }

    fn report_redeclaration(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        node: NodeId,
        existing_id: SymbolId,
        existing: Symbol,
        context: &SemanticContext<'a>,
    ) {
        let name = tree.get_utf8(identifier.name(tree)).as_bytes();
        let current_span = tree.span(node);
        let existing_span = tree.span(context.symbols.first_decl_of(existing_id));
        let kind = existing.flags.to_string();
        let article = if matches!(kind.as_bytes()[0], b'a' | b'e' | b'i' | b'o' | b'u') {
            "an"
        } else {
            "a"
        };
        let message = self.fmt(format_args!(
            "Identifier '{}' has already been declared",
            String::from_utf8_lossy(name)
        ));
        let first = self.fmt(format_args!(
            "'{}' was first declared as {} {} here",
            String::from_utf8_lossy(name),
            article,
            kind
        ));
        let help = self.fmt(format_args!(
            "Consider removing or renaming this declaration of '{}'",
            String::from_utf8_lossy(name)
        ));
        self.report(
            current_span,
            message,
            ReportOptions {
                help: Some(help),
                labels: self.labels(&[
                    self.label(existing_span, first),
                    self.label(current_span, "cannot be redeclared here"),
                ]),
                ..ReportOptions::default()
            },
        );
    }

    fn report(&mut self, span: Span, message: &'a str, options: ReportOptions<'a>) {
        assert!(span.start <= span.end);
        assert!(!message.is_empty());
        self.diagnostics.push(Diagnostic {
            severity: options.severity,
            message: Cow::Borrowed(message),
            span,
            help: options.help.map(Cow::Borrowed),
            labels: options.labels,
        });
    }

    fn label(&self, span: Span, message: &'a str) -> Label<'a> {
        Label {
            span,
            message: Cow::Borrowed(message),
        }
    }

    fn labels(&self, items: &[Label<'a>]) -> Vec<Label<'a>> {
        items.to_vec()
    }

    fn fmt(&self, arguments: std::fmt::Arguments<'_>) -> &'a str {
        let mut writer = ArenaWriter {
            bytes: ArenaVec::new_in(self.allocator),
        };
        writer
            .write_fmt(arguments)
            .expect("formatting into an arena vector is infallible");
        std::str::from_utf8(writer.bytes.into_bump_slice())
            .expect("Rust formatting produces valid UTF-8")
    }
}

struct ArenaWriter<'a> {
    bytes: ArenaVec<'a, u8>,
}

impl Write for ArenaWriter<'_> {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.bytes.extend_from_slice_copy(value.as_bytes());
        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PrivateElementKind {
    Field,
    Method,
    Getter,
    Setter,
}

/// Returns the StringValue of a class element's PrivateIdentifier key, if any.
fn private_element_name<'a>(tree: &'a Ast<'_>, node: NodeId) -> Option<&'a [u8]> {
    let key = match tree.kind_data(node) {
        NodeKindData::PropertyDefinition(property) => property.key(tree).node_id(),
        NodeKindData::MethodDefinition(method) => method.key(tree).node_id(),
        NodeKindData::TsMethodDefinition(method) => method.key(tree).node_id(),
        _ => return None,
    };
    let NodeKindData::PrivateIdentifier(identifier) = tree.kind_data(key) else {
        return None;
    };
    Some(tree.get_utf8(identifier.name(tree)).as_bytes())
}

fn private_element_kind(tree: &Ast<'_>, node: NodeId) -> PrivateElementKind {
    let kind = match tree.kind_data(node) {
        NodeKindData::MethodDefinition(method) => Some(method.kind(tree)),
        NodeKindData::TsMethodDefinition(method) => Some(method.kind(tree)),
        _ => None,
    };
    match kind {
        Some(swc_next_ecma_ast::MethodDefinitionKind::Get) => PrivateElementKind::Getter,
        Some(swc_next_ecma_ast::MethodDefinitionKind::Set) => PrivateElementKind::Setter,
        Some(_) => PrivateElementKind::Method,
        None => PrivateElementKind::Field,
    }
}

fn private_element_is_static(tree: &Ast<'_>, node: NodeId) -> bool {
    match tree.kind_data(node) {
        NodeKindData::PropertyDefinition(property) => property.r#static(tree),
        NodeKindData::MethodDefinition(method) => method.r#static(tree),
        NodeKindData::TsMethodDefinition(method) => method.r#static(tree),
        _ => false,
    }
}

/// 15.7.7 AllPrivateIdentifiersValid. Walks ancestor ClassBody nodes,
/// collecting PrivateBoundIdentifiers at each level.
fn is_private_name_declared(context: &SemanticVisitCtx<'_, '_>, name: &[u8]) -> bool {
    for ancestor in context.ancestors() {
        let NodeKindData::ClassBody(body) = context.ast.kind_data(ancestor) else {
            continue;
        };
        for child in body.body(context.ast).iter() {
            let child = context.ast.get_node_in_sub_range(child).node_id();
            if private_element_name(context.ast, child) == Some(name) {
                return true;
            }
        }
    }
    false
}

/// <https://tc39.es/ecma262/#sec-keywords-and-reserved-words>
fn match_strict_reserved(name: &[u8]) -> Option<&'static str> {
    match name.len() {
        3 => check(name, "let"),
        5 => check(name, "yield"),
        6 => check(name, "static").or_else(|| check(name, "public")),
        7 => check(name, "private").or_else(|| check(name, "package")),
        9 => check(name, "protected").or_else(|| check(name, "interface")),
        10 => check(name, "implements"),
        _ => None,
    }
}

fn check(name: &[u8], keyword: &'static str) -> Option<&'static str> {
    (name == keyword.as_bytes()).then_some(keyword)
}

fn is_eval_or_arguments(name: &[u8]) -> bool {
    matches!(name, b"eval" | b"arguments")
}

fn is_eval_or_arguments_reference(tree: &Ast<'_>, node: NodeId) -> bool {
    let NodeKindData::IdentifierReference(identifier) = tree.kind_data(node) else {
        return false;
    };
    is_eval_or_arguments(tree.get_utf8(identifier.name(tree)).as_bytes())
}

fn unwrap_parentheses(tree: &Ast<'_>, node: NodeId) -> NodeId {
    let mut current = node;
    let mut depth = 0u32;
    // The parser builds at most one parenthesis node per syntactic
    // level, so this chain is bounded by source nesting depth.
    loop {
        assert!(depth < 1_000_000);
        let NodeKindData::ParenthesizedExpression(parenthesized) = tree.kind_data(current) else {
            return current;
        };
        current = parenthesized.expression(tree).node_id();
        depth += 1;
    }
}

/// Leading zero followed by digits: legacy octal (`077`) or
/// leading-zero decimal (`089`).
fn is_legacy_numeric_literal(raw: &[u8]) -> bool {
    raw.len() >= 2 && raw[0] == b'0' && raw[1].is_ascii_digit()
}

#[derive(Clone, Copy)]
enum SuperCallValidity {
    Valid,
    NotInConstructor,
    NoExtends,
}

/// `super()` is permitted only inside a constructor of a derived class.
/// Arrow functions are transparent (Section 8.5.1); regular functions are opaque.
fn super_call_validity(context: &SemanticVisitCtx<'_, '_>) -> SuperCallValidity {
    let mut ancestors = context.ancestors();
    while let Some(ancestor) = ancestors.next() {
        match context.ast.node_kind(ancestor) {
            NodeKind::ArrowFunctionExpression => {}
            NodeKind::Function => {
                if let Some(parent) = ancestors.next()
                    && let NodeKindData::MethodDefinition(method) = context.ast.kind_data(parent)
                    && method.kind(context.ast)
                        == swc_next_ecma_ast::MethodDefinitionKind::Constructor
                {
                    for class in ancestors {
                        if let NodeKindData::Class(class) = context.ast.kind_data(class) {
                            return if class.super_class(context.ast).is_some() {
                                SuperCallValidity::Valid
                            } else {
                                SuperCallValidity::NoExtends
                            };
                        }
                    }
                }
                return SuperCallValidity::NotInConstructor;
            }
            NodeKind::PropertyDefinition | NodeKind::StaticBlock | NodeKind::Program => {
                return SuperCallValidity::NotInConstructor;
            }
            _ => {}
        }
    }
    SuperCallValidity::NotInConstructor
}

/// `super.property` is valid in class methods, object methods, field
/// initializers, static blocks, and arrows inheriting from those contexts.
fn is_super_property_valid(context: &SemanticVisitCtx<'_, '_>) -> bool {
    let mut ancestors = context.ancestors();
    while let Some(ancestor) = ancestors.next() {
        match context.ast.node_kind(ancestor) {
            NodeKind::ArrowFunctionExpression => {}
            NodeKind::Function => {
                if let Some(parent) = ancestors.next() {
                    match context.ast.kind_data(parent) {
                        NodeKindData::MethodDefinition(_) => return true,
                        NodeKindData::ObjectProperty(property)
                            if property.method(context.ast)
                                || property.kind(context.ast)
                                    != swc_next_ecma_ast::PropertyKind::Init =>
                        {
                            return true;
                        }
                        _ => {}
                    }
                }
                return false;
            }
            NodeKind::PropertyDefinition | NodeKind::StaticBlock => return true,
            NodeKind::Program => return false,
            _ => {}
        }
    }
    false
}

fn is_at_program_level(context: &SemanticVisitCtx<'_, '_>) -> bool {
    context.parent().is_none_or(|parent| {
        matches!(
            context.ast.node_kind(parent),
            NodeKind::Program | NodeKind::TsModuleBlock
        )
    })
}

fn is_function_boundary(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Function | NodeKind::ArrowFunctionExpression | NodeKind::StaticBlock
    )
}

fn is_iteration(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::ForStatement
            | NodeKind::ForInStatement
            | NodeKind::ForOfStatement
            | NodeKind::WhileStatement
            | NodeKind::DoWhileStatement
    )
}

fn is_inside_breakable(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        let kind = context.ast.node_kind(ancestor);
        if is_iteration(kind) || kind == NodeKind::SwitchStatement {
            return true;
        }
        if is_function_boundary(kind) {
            return false;
        }
    }
    false
}

fn is_inside_iteration(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        let kind = context.ast.node_kind(ancestor);
        if is_iteration(kind) {
            return true;
        }
        if is_function_boundary(kind) {
            return false;
        }
    }
    false
}

fn is_new_target_available(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        match context.ast.node_kind(ancestor) {
            NodeKind::Function | NodeKind::StaticBlock | NodeKind::PropertyDefinition => {
                return true;
            }
            NodeKind::ArrowFunctionExpression => {}
            _ => {}
        }
    }
    // CommonJS top level is a function body.
    context.ast.source_type() == swc_next_ecma_ast::SourceType::CommonJs
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum LabelSearch {
    Found,
    NotFound,
    CrossedBoundary,
}

fn find_label(context: &SemanticVisitCtx<'_, '_>, name: &[u8]) -> LabelSearch {
    assert!(!name.is_empty());
    let mut crossed_boundary = false;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::LabeledStatement(statement) => {
                let label = context
                    .ast
                    .get_utf8(statement.label(context.ast).name(context.ast))
                    .as_bytes();
                if label == name {
                    return if crossed_boundary {
                        LabelSearch::CrossedBoundary
                    } else {
                        LabelSearch::Found
                    };
                }
            }
            NodeKindData::Function(_)
            | NodeKindData::ArrowFunctionExpression(_)
            | NodeKindData::StaticBlock(_) => crossed_boundary = true,
            _ => {}
        }
    }
    LabelSearch::NotFound
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ContinueLabelSearch {
    Found,
    NotFound,
    CrossedBoundary,
    NotIteration,
}

fn find_label_for_continue(context: &SemanticVisitCtx<'_, '_>, name: &[u8]) -> ContinueLabelSearch {
    assert!(!name.is_empty());
    let mut crossed_boundary = false;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::LabeledStatement(statement) => {
                let label = context
                    .ast
                    .get_utf8(statement.label(context.ast).name(context.ast))
                    .as_bytes();
                if label == name {
                    if crossed_boundary {
                        return ContinueLabelSearch::CrossedBoundary;
                    }
                    let body = context.ast.node_kind(statement.body(context.ast).node_id());
                    return if is_iteration(body) || body == NodeKind::LabeledStatement {
                        ContinueLabelSearch::Found
                    } else {
                        ContinueLabelSearch::NotIteration
                    };
                }
            }
            NodeKindData::Function(_)
            | NodeKindData::ArrowFunctionExpression(_)
            | NodeKindData::StaticBlock(_) => crossed_boundary = true,
            _ => {}
        }
    }
    ContinueLabelSearch::NotFound
}

// True when the nearest enclosing boundary is a static block.
fn is_inside_static_block(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        match context.ast.node_kind(ancestor) {
            NodeKind::StaticBlock => return true,
            NodeKind::Function | NodeKind::ArrowFunctionExpression => return false,
            _ => {}
        }
    }
    false
}

/// `arguments` is available inside regular functions but not in class field
/// initializers or static blocks (arrow functions are transparent).
fn is_arguments_available(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        match context.ast.node_kind(ancestor) {
            NodeKind::Function => return true,
            NodeKind::PropertyDefinition | NodeKind::StaticBlock => return false,
            NodeKind::ArrowFunctionExpression | NodeKind::Program => {}
            _ => {}
        }
    }
    true
}

fn is_in_formal_parameters(context: &SemanticVisitCtx<'_, '_>) -> bool {
    find_formal_parameters(context).is_some()
}

fn find_formal_parameters(
    context: &SemanticVisitCtx<'_, '_>,
) -> Option<swc_next_ecma_ast::FormalParameters> {
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::FormalParameters(parameters) => {
                return Some(parameters);
            }
            NodeKindData::Program(_)
            | NodeKindData::Function(_)
            | NodeKindData::ArrowFunctionExpression(_) => {
                return None;
            }
            _ => {}
        }
    }
    None
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

fn has_octal_escape(value: &[u8]) -> bool {
    let mut position = 0;
    while let Some(offset) = value[position..].iter().position(|&byte| byte == b'\\') {
        let slash = position + offset;
        let next = slash + 1;
        if next >= value.len() {
            break;
        }
        match value[next] {
            b'1'..=b'9' => return true,
            b'0' if value.get(next + 1).is_some_and(u8::is_ascii_digit) => return true,
            _ => {}
        }
        position = next + 1;
    }
    false
}

struct ReportOptions<'a> {
    severity: Severity,
    help: Option<&'a str>,
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

impl Checker<'_, '_> {
    fn check(&mut self, node: NodeId) {
        let checks = &mut self.checks;
        let context = &mut self.context;
        match context.ast.kind_data(node) {
            NodeKindData::Program(_) => {
                checks.check_unresolved_exports(context.ast, &context.state.symbols);
            }
            NodeKindData::StringLiteral(literal) => {
                if context.state.in_type_position() || !context.state.scope.is_strict() {
                    return;
                }
                let span = context.ast.span(literal.node_id());
                if has_octal_escape(
                    &context.ast.source().as_bytes()[span.start as usize..span.end as usize],
                ) {
                    checks.report(
                        span,
                        "Octal escape sequences are not allowed in strict mode",
                        ReportOptions {
                            help: Some("Use \\xHH (hex) or \\uHHHH (unicode) escape instead"),
                            ..ReportOptions::default()
                        },
                    );
                }
            }
            NodeKindData::NumericLiteral(literal) => {
                if context.state.in_type_position() || !context.state.scope.is_strict() {
                    return;
                }
                let span = context.ast.span(literal.node_id());
                let raw = &context.ast.source().as_bytes()[span.start as usize..span.end as usize];
                if !is_legacy_numeric_literal(raw) {
                    return;
                }
                let is_octal = raw.iter().all(|byte| !matches!(byte, b'8' | b'9'));
                let (message, help) = if is_octal {
                    (
                        "Octal literals are not allowed in strict mode",
                        "Use the 0o prefix for octal literals (e.g., 0o77), or a decimal equivalent",
                    )
                } else {
                    (
                        "Decimals with leading zeros are not allowed in strict mode",
                        "Remove the leading zero, or use 0o for octal, 0x for hex, or 0b for binary notation",
                    )
                };
                checks.report(
                    span,
                    message,
                    ReportOptions {
                        help: Some(help),
                        ..ReportOptions::default()
                    },
                );
            }
            NodeKindData::RegExpLiteral(literal) => {
                let pattern = context.ast.get_utf8(literal.pattern(context.ast));
                let flags = context.ast.get_utf8(literal.flags(context.ast));
                if swc_next_regular_expression::LiteralParser::new(
                    checks.allocator,
                    pattern,
                    Some(flags),
                    swc_next_regular_expression::Options::default(),
                )
                .parse()
                .is_err()
                {
                    checks.report(
                        context.ast.span(literal.node_id()),
                        "Invalid regular expression literal",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::IdentifierReference(identifier) => {
                if context.state.in_type_position() {
                    return;
                }
                let node_id = identifier.node_id();
                let name = context
                    .ast
                    .get_utf8(identifier.name(context.ast))
                    .as_bytes();
                checks.check_strict_reserved(name, node_id, context, "an identifier");
                if name == b"arguments" && !is_arguments_available(context) {
                    checks.report(
                        context.ast.span(node_id),
                        "'arguments' is not allowed in class field initializers or static blocks",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::BindingIdentifier(identifier) => {
                let node_id = identifier.node_id();
                let name = context
                    .ast
                    .get_utf8(identifier.name(context.ast))
                    .as_bytes();
                let flags = context.state.symbols.pending.flags;
                if context.state.in_type_position() && !flags.contains(SymbolFlags::TYPE_PARAMETER)
                {
                    return;
                }
                if context.state.scope.is_strict() && !flags.contains(SymbolFlags::AMBIENT) {
                    checks.check_strict_reserved(name, node_id, context, "a binding identifier");
                    if is_eval_or_arguments(name) {
                        let message = checks.fmt(format_args!(
                            "'{}' is not allowed as a binding identifier in strict mode",
                            String::from_utf8_lossy(name)
                        ));
                        checks.report(context.ast.span(node_id), message, ReportOptions::default());
                    }
                } else if flags.contains(SymbolFlags::BLOCK_SCOPED_VAR) && name == b"let" {
                    checks.report(
                        context.ast.span(node_id),
                        "'let' is not allowed as a variable name in a lexical declaration",
                        ReportOptions::default(),
                    );
                }
                let existing = context
                    .state
                    .symbols
                    .binding(context.state.symbols.pending.scope, name);
                checks.check_redeclaration(identifier, node_id, name, flags, context, existing);
                if context.state.symbols.export_state.named().is_some() {
                    let skip = existing.is_some() && context.ast.is_ts();
                    if !skip {
                        checks.record_exported_name(name, node_id, context);
                    }
                }
            }
            NodeKindData::LabelIdentifier(identifier) => {
                let name = context
                    .ast
                    .get_utf8(identifier.name(context.ast))
                    .as_bytes();
                checks.check_strict_reserved(name, identifier.node_id(), context, "a label");
            }
            NodeKindData::BinaryExpression(expression) => {
                let left = expression.left(context.ast).node_id();
                if expression.operator(context.ast) == BinaryOperator::In
                    && let NodeKindData::PrivateIdentifier(identifier) = context.ast.kind_data(left)
                {
                    let name = context
                        .ast
                        .get_utf8(identifier.name(context.ast))
                        .as_bytes();
                    if !is_private_name_declared(context, name) {
                        let message = checks.fmt(format_args!(
                            "Private field '#{}' must be declared in an enclosing class",
                            String::from_utf8_lossy(name)
                        ));
                        checks.report(context.ast.span(left), message, ReportOptions::default());
                    }
                }
            }
            NodeKindData::UnaryExpression(expression) => {
                if expression.operator(context.ast) != UnaryOperator::Delete {
                    return;
                }
                let mut target =
                    unwrap_parentheses(context.ast, expression.argument(context.ast).node_id());
                if let NodeKindData::ChainExpression(chain) = context.ast.kind_data(target) {
                    target =
                        unwrap_parentheses(context.ast, chain.expression(context.ast).node_id());
                }
                match context.ast.kind_data(target) {
                    NodeKindData::IdentifierReference(_) if context.state.scope.is_strict() => {
                        checks.report(
                            context.ast.span(expression.node_id()),
                            "Deleting a variable in strict mode is not allowed",
                            ReportOptions::default(),
                        );
                    }
                    NodeKindData::MemberExpression(member)
                        if !member.computed(context.ast)
                            && context
                                .ast
                                .node_kind(member.property(context.ast).node_id())
                                == NodeKind::PrivateIdentifier =>
                    {
                        checks.report(
                            context.ast.span(expression.node_id()),
                            "Private fields cannot be deleted",
                            ReportOptions::default(),
                        );
                    }
                    _ => {}
                }
            }
            NodeKindData::UpdateExpression(expression) => {
                if context.state.scope.is_strict()
                    && is_eval_or_arguments_reference(
                        context.ast,
                        expression.argument(context.ast).node_id(),
                    )
                {
                    checks.report(
                        context.ast.span(expression.node_id()),
                        "Cannot assign to 'eval' or 'arguments' in strict mode",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::AssignmentExpression(expression) => {
                if context.state.scope.is_strict() {
                    checks.check_assign_target_eval_arguments(
                        expression.left(context.ast).node_id(),
                        context,
                    );
                }
            }
            NodeKindData::VariableDeclaration(declaration) => {
                let kind = declaration.kind(context.ast);
                if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing)
                    && context.ast.source_type() == swc_next_ecma_ast::SourceType::Script
                    && is_at_program_level(context)
                {
                    let message = checks.fmt(format_args!(
                        "'{}' is not allowed at the top level of a script",
                        kind.as_str()
                    ));
                    checks.report(
                        context.ast.span(declaration.node_id()),
                        message,
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::SwitchStatement(statement) => {
                let mut first_default = None;
                for case in statement.cases(context.ast).iter() {
                    let case = context.ast.get_node_in_sub_range(case);
                    if case.test(context.ast).is_some() {
                        continue;
                    }
                    if let Some(first) = first_default {
                        checks.report(
                            context.ast.span(case.node_id()),
                            "A switch statement can only have one default clause",
                            ReportOptions {
                                labels: checks.labels(&[checks
                                    .label(context.ast.span(first), "first default defined here")]),
                                ..ReportOptions::default()
                            },
                        );
                    } else {
                        first_default = Some(case.node_id());
                    }
                }
            }
            NodeKindData::ForInStatement(statement) => {
                let left = statement.left(context.ast).node_id();
                checks.check_for_in_of_initializer(context, left, true);
                if context.state.scope.is_strict() {
                    checks.check_assign_target_eval_arguments(left, context);
                }
            }
            NodeKindData::ForOfStatement(statement) => {
                if statement.r#await(context.ast)
                    && !context.ast.is_ts()
                    && is_inside_static_block(context)
                {
                    checks.report(
                        context.ast.span(statement.node_id()),
                        "Cannot use 'for await' in class static initialization block",
                        ReportOptions::default(),
                    );
                }
                let left = statement.left(context.ast).node_id();
                checks.check_for_in_of_initializer(context, left, false);
                if context.state.scope.is_strict() {
                    checks.check_assign_target_eval_arguments(left, context);
                }
            }
            NodeKindData::BreakStatement(statement) => {
                if let Some(label) = statement.label(context.ast) {
                    let name = context.ast.get_utf8(label.name(context.ast)).as_bytes();
                    let result = find_label(context, name);
                    if result != LabelSearch::Found {
                        let message = match result {
                            LabelSearch::NotFound => checks.fmt(format_args!(
                                "Use of undefined label '{}'",
                                String::from_utf8_lossy(name)
                            )),
                            LabelSearch::CrossedBoundary => checks.fmt(format_args!(
                                "Cannot break to label '{}' across function boundaries",
                                String::from_utf8_lossy(name)
                            )),
                            LabelSearch::Found => unreachable!(),
                        };
                        checks.report(
                            context.ast.span(statement.node_id()),
                            message,
                            ReportOptions::default(),
                        );
                    }
                } else if !is_inside_breakable(context) {
                    checks.report(
                        context.ast.span(statement.node_id()),
                        "Illegal break statement",
                        ReportOptions {
                            help: Some(
                                "A 'break' statement can only be used within an enclosing iteration or switch statement",
                            ),
                            ..ReportOptions::default()
                        },
                    );
                }
            }
            NodeKindData::ContinueStatement(statement) => {
                if let Some(label) = statement.label(context.ast) {
                    let name = context.ast.get_utf8(label.name(context.ast)).as_bytes();
                    let result = find_label_for_continue(context, name);
                    if result != ContinueLabelSearch::Found {
                        let (message, help) = match result {
                            ContinueLabelSearch::NotFound => (
                                checks.fmt(format_args!(
                                    "Use of undefined label '{}'",
                                    String::from_utf8_lossy(name)
                                )),
                                None,
                            ),
                            ContinueLabelSearch::CrossedBoundary => (
                                checks.fmt(format_args!(
                                    "Cannot continue to label '{}' across function boundaries",
                                    String::from_utf8_lossy(name)
                                )),
                                None,
                            ),
                            ContinueLabelSearch::NotIteration => (
                                checks.fmt(format_args!(
                                    "Label '{}' does not denote an iteration statement",
                                    String::from_utf8_lossy(name)
                                )),
                                Some(
                                    "A 'continue' statement can only jump to a label of an enclosing 'for', 'while', 'do-while', 'for-in', or 'for-of' statement",
                                ),
                            ),
                            ContinueLabelSearch::Found => unreachable!(),
                        };
                        checks.report(
                            context.ast.span(statement.node_id()),
                            message,
                            ReportOptions {
                                help,
                                ..ReportOptions::default()
                            },
                        );
                    }
                } else if !is_inside_iteration(context) {
                    checks.report(
                        context.ast.span(statement.node_id()),
                        "Illegal continue statement",
                        ReportOptions {
                            help: Some(
                                "A 'continue' statement can only be used within an enclosing iteration statement",
                            ),
                            ..ReportOptions::default()
                        },
                    );
                }
            }
            NodeKindData::LabeledStatement(statement) => {
                let name = context
                    .ast
                    .get_utf8(statement.label(context.ast).name(context.ast))
                    .as_bytes();
                let duplicate = {
                    let mut ancestors = context.ancestors();
                    let _ = ancestors.next();
                    let mut duplicate = false;
                    for ancestor in ancestors {
                        match context.ast.kind_data(ancestor) {
                            NodeKindData::LabeledStatement(outer) => {
                                let outer_name = context
                                    .ast
                                    .get_utf8(outer.label(context.ast).name(context.ast))
                                    .as_bytes();
                                if outer_name == name {
                                    duplicate = true;
                                }
                            }
                            NodeKindData::Function(_)
                            | NodeKindData::ArrowFunctionExpression(_)
                            | NodeKindData::StaticBlock(_) => break,
                            _ => {}
                        }
                    }
                    duplicate
                };
                if duplicate {
                    let message = checks.fmt(format_args!(
                        "Duplicate label '{}'",
                        String::from_utf8_lossy(name)
                    ));
                    checks.report(
                        context.ast.span(statement.label(context.ast).node_id()),
                        message,
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::Directive(directive) => {
                if context.ast.get_utf8(directive.value(context.ast)) != "use strict" {
                    return;
                }
                let directive_span = context.ast.span(directive.node_id());
                let mut parameter = None;
                for node in context.ancestors() {
                    match context.ast.kind_data(node) {
                        NodeKindData::Function(function) => {
                            parameter = ecmascript::find_non_simple_parameter(
                                context.ast,
                                function.params(context.ast),
                            );
                            break;
                        }
                        NodeKindData::ArrowFunctionExpression(arrow) => {
                            parameter = ecmascript::find_non_simple_parameter(
                                context.ast,
                                arrow.params(context.ast),
                            );
                            break;
                        }
                        NodeKindData::Program(_) => break,
                        _ => {}
                    }
                }
                if let Some(parameter) = parameter {
                    checks.report(
                        directive_span,
                        "Illegal 'use strict' directive in function with non-simple parameter list",
                        ReportOptions {
                            labels: checks
                                .labels(&[checks
                                    .label(context.ast.span(parameter), "non-simple parameter")]),
                            ..ReportOptions::default()
                        },
                    );
                }
            }
            NodeKindData::ClassBody(body) => {
                let children = body.body(context.ast);
                let mut first_constructor = None;
                for index in 0..children.len() {
                    let child = children
                        .get_node(context.ast, index)
                        .expect("class child exists")
                        .node_id();
                    if let NodeKindData::MethodDefinition(method) = context.ast.kind_data(child)
                        && method.kind(context.ast)
                            == swc_next_ecma_ast::MethodDefinitionKind::Constructor
                    {
                        if let Some(first) = first_constructor {
                            checks.report(
                                context.ast.span(method.key(context.ast).node_id()),
                                "A class can only have one constructor",
                                ReportOptions {
                                    labels: checks.labels(&[checks.label(
                                        context.ast.span(first),
                                        "first constructor defined here",
                                    )]),
                                    ..ReportOptions::default()
                                },
                            );
                        } else {
                            first_constructor = Some(method.key(context.ast).node_id());
                        }
                    }
                    let Some(name) = private_element_name(context.ast, child) else {
                        continue;
                    };
                    let kind = private_element_kind(context.ast, child);
                    for previous_index in 0..index {
                        let previous = children
                            .get_node(context.ast, previous_index)
                            .expect("class child exists")
                            .node_id();
                        let Some(previous_name) = private_element_name(context.ast, previous)
                        else {
                            continue;
                        };
                        if previous_name != name {
                            continue;
                        }
                        let previous_kind = private_element_kind(context.ast, previous);
                        let get_set_pair = matches!(
                            (previous_kind, kind),
                            (PrivateElementKind::Getter, PrivateElementKind::Setter)
                                | (PrivateElementKind::Setter, PrivateElementKind::Getter)
                        );
                        if get_set_pair
                            && private_element_is_static(context.ast, previous)
                                == private_element_is_static(context.ast, child)
                        {
                            continue;
                        }
                        let message = checks.fmt(format_args!(
                            "Duplicate private name '#{}'",
                            String::from_utf8_lossy(name)
                        ));
                        checks.report(
                            context.ast.span(child),
                            message,
                            ReportOptions {
                                labels: checks
                                    .labels(&[checks
                                        .label(context.ast.span(previous), "first declared here")]),
                                ..ReportOptions::default()
                            },
                        );
                        break;
                    }
                }
            }
            NodeKindData::Function(function) => {
                if function.function_type(context.ast) != FunctionType::FunctionDeclaration {
                    return;
                }
                let strict = context.state.scope.is_strict();
                let mut through_labels = false;
                let mut ancestors = context.ancestors();
                let _ = ancestors.next();
                let parent = loop {
                    let Some(parent) = ancestors.next() else {
                        return;
                    };
                    if context.ast.node_kind(parent) != NodeKind::LabeledStatement {
                        break context.ast.node_kind(parent);
                    }
                    through_labels = true;
                };
                let valid = match parent {
                    NodeKind::Program
                    | NodeKind::FunctionBody
                    | NodeKind::ClassBody
                    | NodeKind::StaticBlock
                    | NodeKind::BlockStatement
                    | NodeKind::SwitchCase
                    | NodeKind::ExportNamedDeclaration
                    | NodeKind::ExportDefaultDeclaration
                    | NodeKind::TsModuleBlock => !strict || !through_labels,
                    NodeKind::IfStatement => !strict && !through_labels,
                    _ => false,
                };
                if !valid {
                    let message = if strict {
                        "In strict mode code, functions can only be declared at top level or inside a block"
                    } else {
                        "In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement"
                    };
                    checks.report(
                        context.ast.span(function.node_id()),
                        message,
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::WithStatement(statement) => {
                if context.state.scope.is_strict() {
                    checks.report(
                        context.ast.span(statement.node_id()),
                        "'with' statements are not allowed in strict mode",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::ObjectExpression(object) => {
                let mut first_proto = None;
                for property in object.properties(context.ast).iter() {
                    let property = context.ast.get_node_in_sub_range(property).node_id();
                    let NodeKindData::ObjectProperty(item) = context.ast.kind_data(property) else {
                        continue;
                    };
                    if item.computed(context.ast)
                        || item.method(context.ast)
                        || item.shorthand(context.ast)
                        || item.kind(context.ast) != swc_next_ecma_ast::PropertyKind::Init
                    {
                        continue;
                    }
                    let name = match context.ast.kind_data(item.key(context.ast).node_id()) {
                        NodeKindData::IdentifierName(identifier) => context
                            .ast
                            .get_utf8(identifier.name(context.ast))
                            .as_bytes(),
                        NodeKindData::StringLiteral(string) => {
                            context.ast.get_wtf8(string.value(context.ast)).as_bytes()
                        }
                        _ => continue,
                    };
                    if name != b"__proto__" {
                        continue;
                    }
                    if let Some(first) = first_proto {
                        checks.report(
                            context.ast.span(property),
                            "Duplicate '__proto__' property in object literal",
                            ReportOptions {
                                labels: checks.labels(&[
                                    checks.label(context.ast.span(first), "first defined here")
                                ]),
                                ..ReportOptions::default()
                            },
                        );
                    } else {
                        first_proto = Some(property);
                    }
                }
            }
            NodeKindData::MemberExpression(expression) => {
                let object = expression.object(context.ast).node_id();
                if context.ast.node_kind(object) == NodeKind::Super
                    && !is_super_property_valid(context)
                {
                    checks.report(
                        context.ast.span(expression.node_id()),
                        "'super' property access is only valid inside a method or class body",
                        ReportOptions {
                            help: Some(
                                "Use an arrow function instead of a regular function to inherit the 'super' binding",
                            ),
                            ..ReportOptions::default()
                        },
                    );
                }
                let property = expression.property(context.ast).node_id();
                if !expression.computed(context.ast)
                    && let NodeKindData::PrivateIdentifier(identifier) =
                        context.ast.kind_data(property)
                {
                    let name = context
                        .ast
                        .get_utf8(identifier.name(context.ast))
                        .as_bytes();
                    if context.ast.node_kind(object) == NodeKind::Super {
                        checks.report(
                            context.ast.span(expression.node_id()),
                            "'super' keyword unexpected here",
                            ReportOptions::default(),
                        );
                    } else if !is_private_name_declared(context, name) {
                        let message = checks.fmt(format_args!(
                            "Private field '#{}' must be declared in an enclosing class",
                            String::from_utf8_lossy(name)
                        ));
                        checks.report(
                            context.ast.span(property),
                            message,
                            ReportOptions::default(),
                        );
                    }
                }
            }
            NodeKindData::CallExpression(call) => {
                if context.ast.node_kind(call.callee(context.ast).node_id()) == NodeKind::Super {
                    let message = "'super()' is only valid in a constructor of a derived class";
                    match super_call_validity(context) {
                        SuperCallValidity::Valid => {}
                        SuperCallValidity::NotInConstructor => checks.report(
                            context.ast.span(call.node_id()),
                            message,
                            ReportOptions {
                                help: Some(
                                    "Use an arrow function instead of a regular function to inherit the 'super' binding",
                                ),
                                ..ReportOptions::default()
                            },
                        ),
                        SuperCallValidity::NoExtends => checks.report(
                            context.ast.span(call.node_id()),
                            message,
                            ReportOptions {
                                help: Some(
                                    "Add an 'extends' clause to the class or remove the 'super()' call",
                                ),
                                ..ReportOptions::default()
                            },
                        ),
                    }
                }
            }
            NodeKindData::AwaitExpression(expression) => {
                let message = {
                    let mut message = None;
                    for ancestor in context.ancestors() {
                        match context.ast.node_kind(ancestor) {
                            NodeKind::FormalParameters => {
                                message =
                                    Some("Await expression is not allowed in formal parameters");
                                break;
                            }
                            NodeKind::StaticBlock => {
                                message =
                                    Some("Cannot use await in class static initialization block");
                                break;
                            }
                            NodeKind::Function
                            | NodeKind::ArrowFunctionExpression
                            | NodeKind::Program => break,
                            _ => {}
                        }
                    }
                    message
                };
                if let Some(message) = message {
                    checks.report(
                        context.ast.span(expression.node_id()),
                        message,
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::YieldExpression(expression) => {
                if is_in_formal_parameters(context) {
                    checks.report(
                        context.ast.span(expression.node_id()),
                        "Yield expression is not allowed in formal parameters",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::MetaProperty(property) => {
                let name = context
                    .ast
                    .get_utf8(property.meta(context.ast).name(context.ast));
                if name == "import" && !context.ast.is_module() {
                    checks.report(
                        context.ast.span(property.node_id()),
                        "'import.meta' is only valid in module code",
                        ReportOptions::default(),
                    );
                }
                if name == "new" && !is_new_target_available(context) {
                    checks.report(
                        context.ast.span(property.node_id()),
                        "'new.target' is only valid inside functions, class field initializers, or static blocks",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::ImportDeclaration(declaration) => {
                checks.check_import_export_position(
                    declaration.node_id(),
                    "import statement",
                    "'import' declaration",
                    context,
                );
                checks
                    .check_duplicate_with_attributes(declaration.attributes(context.ast), context);
            }
            NodeKindData::ExportNamedDeclaration(declaration) => {
                checks.check_import_export_position(
                    declaration.node_id(),
                    "'export' declaration",
                    "'export' declaration",
                    context,
                );
                checks
                    .check_duplicate_with_attributes(declaration.attributes(context.ast), context);
            }
            NodeKindData::ExportDefaultDeclaration(declaration) => {
                checks.check_import_export_position(
                    declaration.node_id(),
                    "'export default' declaration",
                    "'export default' declaration",
                    context,
                );
                checks.record_exported_name(b"default", declaration.node_id(), context);
            }
            NodeKindData::ExportAllDeclaration(declaration) => {
                checks.check_import_export_position(
                    declaration.node_id(),
                    "'export *' declaration",
                    "'export *' declaration",
                    context,
                );
                if let Some(exported) = declaration.exported(context.ast) {
                    let name = module_export_name(context.ast, exported.node_id());
                    checks.record_exported_name(name, declaration.node_id(), context);
                }
                checks
                    .check_duplicate_with_attributes(declaration.attributes(context.ast), context);
            }
            NodeKindData::ExportSpecifier(specifier) => {
                let exported_name =
                    module_export_name(context.ast, specifier.exported(context.ast).node_id());
                checks.record_exported_name(exported_name, specifier.node_id(), context);
                if specifier.export_kind(context.ast) == ImportOrExportKind::Type
                    || context.state.in_ts_namespace()
                {
                    return;
                }
                if let Some(parent) = context.parent()
                    && let NodeKindData::ExportNamedDeclaration(declaration) =
                        context.ast.kind_data(parent)
                    && declaration.source(context.ast).is_none()
                    && declaration.export_kind(context.ast) != ImportOrExportKind::Type
                {
                    let local =
                        module_export_name(context.ast, specifier.local(context.ast).node_id());
                    checks.export_specifiers.push(ExportSpecifierInfo {
                        local_name: local.clone_in(checks.allocator),
                        node: specifier.node_id(),
                    });
                }
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Checker<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.context.ast
    }

    fn enter_node(&mut self, node: NodeId) {
        self.context.enter_node(node);
    }

    fn leave_node(&mut self, node: NodeId) {
        if self.check_syntax {
            self.check(node);
        }
        self.context.leave_node(node);
    }
    #[inline]
    fn visit_update_expression(&mut self, node: UpdateExpression) {
        self.context.enter_node(node.node_id());
        let argument = node.argument(self.context.ast);
        let flags = self.assignment_reference_flags(true);
        self.visit_with_reference_flags(argument, flags);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
        self.context.enter_node(node.node_id());
        let left = node.left(self.context.ast);
        let read = node.operator(self.context.ast) != AssignmentOperator::Assign;
        let flags = self.assignment_reference_flags(read);
        self.visit_with_reference_flags(left, flags);
        node.right(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assignment_pattern(&mut self, node: AssignmentPattern) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.left(self.context.ast), node_flags);
        node.right(self.context.ast).visit_with(self);
        node.decorators(self.context.ast).visit_with(self);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binding_rest_element(&mut self, node: BindingRestElement) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.argument(self.context.ast), node_flags);
        node.decorators(self.context.ast).visit_with(self);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_array_pattern(&mut self, node: ArrayPattern) {
        let node_flags = self.context.enter_node(node.node_id());
        for element in node.elements(self.context.ast).iter() {
            if let Some(element) = self.context.ast.get_node_in_sub_range(element) {
                self.visit_with_reference_flags(element, node_flags);
            }
        }
        if let Some(rest) = node.rest(self.context.ast) {
            self.visit_with_reference_flags(rest, node_flags);
        }
        node.decorators(self.context.ast).visit_with(self);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_pattern(&mut self, node: ObjectPattern) {
        let node_flags = self.context.enter_node(node.node_id());
        for property in node.properties(self.context.ast).iter() {
            let property = self.context.ast.get_node_in_sub_range(property);
            self.visit_with_reference_flags(property, node_flags);
        }
        if let Some(rest) = node.rest(self.context.ast) {
            self.visit_with_reference_flags(rest, node_flags);
        }
        node.decorators(self.context.ast).visit_with(self);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binding_property(&mut self, node: BindingProperty) {
        let node_flags = self.context.enter_node(node.node_id());
        node.key(self.context.ast).visit_with(self);
        self.visit_with_reference_flags(node.value(self.context.ast), node_flags);
        self.leave_node(node.node_id());
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
        node.value(self.context.ast).visit_with(self);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_member_expression(&mut self, node: MemberExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        // Assignment writes the property, not its base. Both the base and a
        // computed property are evaluated as reads.
        let object_flags = self.flags_with_space(node_flags.space);
        self.visit_with_reference_flags(node.object(self.context.ast), object_flags);
        node.property(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_parenthesized_expression(&mut self, node: ParenthesizedExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_as_expression(&mut self, node: TsAsExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_satisfies_expression(&mut self, node: TsSatisfiesExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_assertion(&mut self, node: TsTypeAssertion) {
        let node_flags = self.context.enter_node(node.node_id());
        node.type_annotation(self.context.ast).visit_with(self);
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_non_null_expression(&mut self, node: TsNonNullExpression) {
        let node_flags = self.context.enter_node(node.node_id());
        self.visit_with_reference_flags(node.expression(self.context.ast), node_flags);
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        let flags = self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let identifier = node;
        let state = &mut context.state;
        state
            .symbols
            .add_identifier_reference(context.ast, identifier, &state.scope, flags);
        node.visit_children_with(self);
        self.leave_node(node.node_id());
    }
    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        let flags = self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    /// Per section 14.12, the CaseBlock grammar permits at most one DefaultClause.
    fn visit_switch_statement(&mut self, node: SwitchStatement) {
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        self.context.enter_node(node.node_id());
        self.context
            .state
            .symbols
            .enter_export_named_declaration(self.context.ast, node);
        node.visit_children_with(self);
        self.context.state.symbols.exit_export_declaration();
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    fn visit_formal_parameters(&mut self, node: FormalParameters) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_formal_parameters(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_formal_parameters();
        self.leave_node(node.node_id());
    }
    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        self.context.enter_node(node.node_id());
        self.context
            .state
            .symbols
            .enter_import_specifier(self.context.ast, node);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_export_assignment(&mut self, node: TsExportAssignment) {
        self.context.enter_node(node.node_id());
        let expression = node.expression(self.context.ast);
        let space = export_expression_space(self.context.ast, expression.node_id());
        let flags = self.flags_with_space(space);
        self.visit_with_reference_flags(expression, flags);
        self.leave_node(node.node_id());
    }
    fn visit_ts_enum_declaration(&mut self, node: TsEnumDeclaration) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_enum_declaration(self.context.ast, node, &state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    fn visit_ts_global_declaration(&mut self, node: TsGlobalDeclaration) {
        self.context.enter_node(node.node_id());
        self.context.state.symbols.enter_ts_global_declaration();
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(node.node_id());
    }
    fn visit_ts_namespace_export_declaration(&mut self, node: TsNamespaceExportDeclaration) {
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state
            .symbols
            .enter_ts_namespace_export_declaration(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ts_type_annotation(&mut self, node: TsTypeAnnotation) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_type_reference(&mut self, node: TsTypeReference) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_qualified_name(&mut self, node: TsQualifiedName) {
        if self.in_ts_module_name {
            self.context.enter_node(node.node_id());
            node.visit_children_with(self);
            self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_type_query(&mut self, node: TsTypeQuery) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let flags = self.flags_with_space(ReferenceSpace::Typeof);
        self.visit_with_reference_flags(node.expr_name(self.context.ast), flags);
        node.type_arguments(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_import_type(&mut self, node: TsImportType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_type_parameter_declaration(&mut self, node: TsTypeParameterDeclaration) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_type_parameter_instantiation(&mut self, node: TsTypeParameterInstantiation) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_literal_type(&mut self, node: TsLiteralType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_template_literal_type(&mut self, node: TsTemplateLiteralType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_array_type(&mut self, node: TsArrayType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_indexed_access_type(&mut self, node: TsIndexedAccessType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_tuple_type(&mut self, node: TsTupleType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_named_tuple_member(&mut self, node: TsNamedTupleMember) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_optional_type(&mut self, node: TsOptionalType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_rest_type(&mut self, node: TsRestType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_js_doc_nullable_type(&mut self, node: TsJsDocNullableType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_js_doc_non_nullable_type(&mut self, node: TsJsDocNonNullableType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_js_doc_unknown_type(&mut self, node: TsJsDocUnknownType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_union_type(&mut self, node: TsUnionType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_intersection_type(&mut self, node: TsIntersectionType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_infer_type(&mut self, node: TsInferType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_type_operator(&mut self, node: TsTypeOperator) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_parenthesized_type(&mut self, node: TsParenthesizedType) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_type_literal(&mut self, node: TsTypeLiteral) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_property_signature(&mut self, node: TsPropertySignature) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        self.visit_type_member_key(node.key(self.context.ast), node.computed(self.context.ast));
        node.type_annotation(self.context.ast).visit_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_class_implements(&mut self, node: TsClassImplements) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_interface_heritage(&mut self, node: TsInterfaceHeritage) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    #[inline]
    fn visit_ts_interface_body(&mut self, node: TsInterfaceBody) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    fn visit_ts_type_predicate(&mut self, node: TsTypePredicate) {
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let predicate = node;
        let state = &mut context.state;
        state
            .symbols
            .add_ts_type_predicate_reference(context.ast, predicate, &state.scope);
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.context.state.exit_type_position();
    }
    fn visit_switch_case(&mut self, node: SwitchCase) {
        self.context
            .state
            .enter_switch_case(self.context.current_node());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
    }
    fn visit_for_statement(&mut self, node: ForStatement) {
        self.context.state.enter_for_statement(node.node_id());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_catch_clause(&mut self, node: CatchClause) {
        self.context.state.enter_catch_clause(node.node_id());
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_catch_clause(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_decorator(&mut self, node: Decorator) {
        self.context.state.enter_decorator();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_decorator();
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    fn visit_static_block(&mut self, node: StaticBlock) {
        self.context.state.enter_static_block(node.node_id());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_function_body(&mut self, node: FunctionBody) {
        let parent = self.context.current_node();
        self.context
            .state
            .enter_function_body(self.context.ast, node, parent);
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_block_statement(&mut self, node: BlockStatement) {
        self.context
            .state
            .enter_block_statement(self.context.ast, node.node_id());
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_with_statement(&mut self, node: WithStatement) {
        self.context.enter_node(node.node_id());
        node.object(self.context.ast).visit_with(self);
        self.context.state.enter_with_statement(node.node_id());
        node.body(self.context.ast).visit_with(self);
        self.context.state.exit_with_statement(node.node_id());
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    fn visit_ts_function(&mut self, node: TsFunction) {
        self.context.state.enter_ts_function(node.node_id());
        self.context.enter_node(node.node_id());
        let state = &mut self.context.state;
        state.symbols.enter_ts_function(&state.scope);
        node.visit_children_with(self);
        self.context.state.symbols.exit_saved_context();
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_ts_conditional_type(&mut self, node: TsConditionalType) {
        self.context.state.enter_ts_conditional_type(node.node_id());
        self.context.state.enter_type_position();
        self.context.enter_node(node.node_id());
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
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
        self.leave_node(node.node_id());
    }
    fn visit_ts_enum_body(&mut self, node: TsEnumBody) {
        self.context.state.enter_ts_enum_body(node.node_id());
        self.context.enter_node(node.node_id());
        self.context.state.symbols.enter_ts_enum_body();
        node.visit_children_with(self);
        self.context.state.exit_scope(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_ts_enum_member(&mut self, node: TsEnumMember) {
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let member = node;
        let state = &mut context.state;
        state
            .symbols
            .declare_ts_enum_member(context.ast, member, &state.scope);
        node.visit_children_with(self);
        self.leave_node(node.node_id());
    }
    fn visit_ts_module_block(&mut self, node: TsModuleBlock) {
        self.context.state.enter_ts_module_block(node.node_id());
        self.context.enter_node(node.node_id());
        self.context.state.symbols.enter_ts_module_block();
        node.visit_children_with(self);
        self.context.state.exit_ts_module_block(node.node_id());
        self.leave_node(node.node_id());
    }
    fn visit_jsx_opening_element(&mut self, node: JsxOpeningElement) {
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let element = node;
        let state = &mut context.state;
        state
            .symbols
            .add_jsx_opening_element_reference(context.ast, element, &state.scope);
        node.visit_children_with(self);
        self.leave_node(node.node_id());
    }
    fn visit_jsx_closing_element(&mut self, node: JsxClosingElement) {
        self.context.enter_node(node.node_id());
        let context = &mut self.context;
        let element = node;
        let state = &mut context.state;
        state
            .symbols
            .add_jsx_closing_element_reference(context.ast, element, &state.scope);
        node.visit_children_with(self);
        self.leave_node(node.node_id());
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

impl<'a> CheckerState<'a> {
    fn check_redeclaration(
        &mut self,
        identifier: BindingIdentifier,
        node: NodeId,
        name: &[u8],
        flags: SymbolFlags,
        context: &mut SemanticVisitCtx<'_, 'a>,
        existing_id: Option<SymbolId>,
    ) {
        assert!(!name.is_empty());
        let target = context.state.symbols.pending.scope;
        let excludes = context.state.symbols.pending.excludes;
        if let Some(existing_id) = existing_id {
            let existing = context.state.symbols.symbol(existing_id);
            let merging_with_ambient = flags.contains(SymbolFlags::AMBIENT)
                || existing.flags.contains(SymbolFlags::AMBIENT);
            let import_value_conflict = !context.ast.is_ts()
                && context.ast.is_module()
                && target == ScopeId::MODULE
                && ((flags.contains(SymbolFlags::IMPORT) && existing.flags.in_value_space())
                    || (flags.in_value_space() && existing.flags.contains(SymbolFlags::IMPORT)));
            if !merging_with_ambient
                && (existing.flags.intersects(excludes) || import_value_conflict)
                && !context
                    .state
                    .symbols
                    .pending
                    .allows_annex_b_function_duplicate(existing.flags)
            {
                self.report_redeclaration(
                    context.ast,
                    identifier,
                    node,
                    existing_id,
                    existing,
                    &context.state,
                );
                return;
            }

            // https://tc39.es/ecma262/#sec-parameter-lists-static-semantics-early-errors
            // Duplicate parameters are allowed in sloppy simple-parameter-list
            // functions. Re-add the conflict for strict, unique, arrow, and
            // non-simple cases.
            if existing.flags.contains(SymbolFlags::PARAMETER)
                && flags.contains(SymbolFlags::PARAMETER)
                && let Some(parameters) = find_formal_parameters(context)
            {
                let kind = parameters.kind(context.ast);
                let must_be_unique = context.state.scope.is_strict()
                    || matches!(
                        kind,
                        swc_next_ecma_ast::FormalParameterKind::UniqueFormalParameters
                            | swc_next_ecma_ast::FormalParameterKind::ArrowFormalParameters
                    )
                    || ecmascript::find_non_simple_parameter(context.ast, parameters).is_some();
                if must_be_unique {
                    self.report_redeclaration(
                        context.ast,
                        identifier,
                        node,
                        existing_id,
                        existing,
                        &context.state,
                    );
                }
            }
        }

        // A split body environment keeps the early errors: lexical body names
        // may not duplicate parameter names (15.2.1) or catch parameter names
        // (14.15.1) one scope up:
        //
        //   function f(a = 1) { let a }   // still an error
        //   function f(a = 1) { var a }   // legal, var stays out
        if !flags.intersects(SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::FUNCTION) {
            let target_scope = context.state.scope.get(target);
            let outer = target_scope.parent;
            let outer_holds_parameters = outer.is_some_and(|outer| match target_scope.kind {
                ScopeKind::FunctionBody => true,
                ScopeKind::Block => {
                    match context.ast.kind_data(context.state.scope.get(outer).node) {
                        NodeKindData::CatchClause(catch) => {
                            catch.body(context.ast).node_id() == target_scope.node
                        }
                        _ => false,
                    }
                }
                _ => false,
            });
            if outer_holds_parameters
                && let Some(outer) = outer
                && let Some(symbol_id) = context.state.symbols.own_binding(outer, name)
            {
                let existing = context.state.symbols.symbol(symbol_id);
                if existing
                    .flags
                    .intersects(SymbolFlags::PARAMETER | SymbolFlags::CATCH_VAR)
                    && existing.flags.intersects(excludes)
                {
                    self.report_redeclaration(
                        context.ast,
                        identifier,
                        node,
                        symbol_id,
                        existing,
                        &context.state,
                    );
                }
            }
        }

        // Per 14.2.1, a hoisting var conflicts with block-scoped names
        // in any intermediate block it passes through.
        if flags.is_hoisting_var() {
            for scope_id in context
                .state
                .scope
                .ancestors(Some(context.state.scope.current))
            {
                if scope_id == target {
                    break;
                }
                let Some(symbol_id) = context.state.symbols.own_binding(scope_id, name) else {
                    continue;
                };
                let existing = context.state.symbols.symbol(symbol_id);
                if existing.flags.is_block_scoped_like() {
                    self.report_redeclaration(
                        context.ast,
                        identifier,
                        node,
                        symbol_id,
                        existing,
                        &context.state,
                    );
                    break;
                }
                if existing.flags.contains(SymbolFlags::CATCH_VAR) {
                    // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
                    // (14.15.1): a var colliding with a catch parameter is a
                    // syntax error, except for the Annex B.3.4 simple-binding
                    // web-compat carve-out.
                    let catch_node = context.state.scope.get(scope_id).node;
                    let NodeKindData::CatchClause(catch) = context.ast.kind_data(catch_node) else {
                        unreachable!()
                    };
                    let simple_parameter = catch.param(context.ast).is_some_and(|parameter| {
                        context.ast.node_kind(parameter.node_id()) == NodeKind::BindingIdentifier
                    });
                    if !simple_parameter {
                        self.report_redeclaration(
                            context.ast,
                            identifier,
                            node,
                            symbol_id,
                            existing,
                            &context.state,
                        );
                        break;
                    }
                }
            }
        }
    }

    /// <https://tc39.es/ecma262/#sec-keywords-and-reserved-words>
    fn check_strict_reserved(
        &mut self,
        name: &[u8],
        node: NodeId,
        context: &mut SemanticVisitCtx<'_, 'a>,
        as_what: &'static str,
    ) {
        if !context.state.scope.is_strict() {
            return;
        }
        if let Some(word) = match_strict_reserved(name) {
            let message = self.fmt(format_args!(
                "'{}' is reserved in strict mode and cannot be used as {}",
                word, as_what
            ));
            self.report(context.ast.span(node), message, ReportOptions::default());
        }
    }

    /// 14.7.5.1 ForBinding must contain exactly one BoundName and, outside
    /// Annex B.3.5, may not have an Initializer.
    fn check_for_in_of_initializer(
        &mut self,
        context: &mut SemanticVisitCtx<'_, 'a>,
        left: NodeId,
        is_for_in: bool,
    ) {
        let NodeKindData::VariableDeclaration(declaration) = context.ast.kind_data(left) else {
            return;
        };
        let declarators = declaration.declarators(context.ast);
        if declarators.len() > 1 {
            self.report(
                context.ast.span(left),
                "Only a single variable declaration is allowed in a for-in/of statement",
                ReportOptions::default(),
            );
            return;
        }
        for declarator in declarators.iter() {
            let declarator = context.ast.get_node_in_sub_range(declarator);
            if declarator.init(context.ast).is_none() {
                continue;
            }
            let annex_b = is_for_in
                && !context.ast.is_ts()
                && declaration.kind(context.ast) == VariableKind::Var
                && !context.state.scope.is_strict()
                && context.ast.node_kind(declarator.id(context.ast).node_id())
                    == NodeKind::BindingIdentifier;
            if annex_b {
                continue;
            }
            self.report(
                context.ast.span(declarator.node_id()),
                "for-in/of loop variable declaration may not have an initializer",
                ReportOptions::default(),
            );
            return;
        }
    }

    /// Walks a destructuring assignment target to find `eval`/`arguments` references.
    fn check_assign_target_eval_arguments(
        &mut self,
        node: NodeId,
        context: &mut SemanticVisitCtx<'_, 'a>,
    ) {
        match context.ast.kind_data(node) {
            NodeKindData::IdentifierReference(identifier) => {
                if is_eval_or_arguments(
                    context
                        .ast
                        .get_utf8(identifier.name(context.ast))
                        .as_bytes(),
                ) {
                    self.report(
                        context.ast.span(node),
                        "Cannot assign to 'eval' or 'arguments' in strict mode",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::ArrayPattern(array) => {
                let elements = array.elements(context.ast);
                for index in 0..elements.len() {
                    if let Some(Some(element)) = elements.get_node(context.ast, index) {
                        self.check_assign_target_eval_arguments(element.node_id(), context);
                    }
                }
                if let Some(rest) = array.rest(context.ast) {
                    self.check_assign_target_eval_arguments(rest.node_id(), context);
                }
            }
            NodeKindData::ObjectPattern(object) => {
                let properties = object.properties(context.ast);
                for index in 0..properties.len() {
                    let property = properties
                        .get_node(context.ast, index)
                        .expect("object pattern property exists");
                    self.check_assign_target_eval_arguments(
                        property.value(context.ast).node_id(),
                        context,
                    );
                }
                if let Some(rest) = object.rest(context.ast) {
                    self.check_assign_target_eval_arguments(rest.node_id(), context);
                }
            }
            NodeKindData::AssignmentPattern(pattern) => {
                self.check_assign_target_eval_arguments(
                    pattern.left(context.ast).node_id(),
                    context,
                );
            }
            NodeKindData::BindingRestElement(rest) => {
                self.check_assign_target_eval_arguments(
                    rest.argument(context.ast).node_id(),
                    context,
                );
            }
            _ => {}
        }
    }

    fn check_import_export_position(
        &mut self,
        node: NodeId,
        out_of_module_label: &'static str,
        top_level_label: &'static str,
        context: &mut SemanticVisitCtx<'_, 'a>,
    ) {
        let Some(parent) = context.parent() else {
            return;
        };
        let message = match context.ast.node_kind(parent) {
            NodeKind::Program if !context.ast.is_module() => Some(self.fmt(format_args!(
                "Cannot use {} outside a module",
                out_of_module_label
            ))),
            NodeKind::Program | NodeKind::TsModuleBlock => None,
            _ => Some(self.fmt(format_args!(
                "{} may only appear at the top level",
                top_level_label
            ))),
        };
        if let Some(message) = message {
            self.report(context.ast.span(node), message, ReportOptions::default());
        }
    }

    fn check_duplicate_with_attributes(
        &mut self,
        attributes: swc_next_ecma_ast::TypedSubRange<swc_next_ecma_ast::ImportAttribute>,
        context: &mut SemanticVisitCtx<'_, 'a>,
    ) {
        for index in 0..attributes.len() {
            let attribute = attributes
                .get_node(context.ast, index)
                .expect("import attribute exists");
            let key = attribute.key(context.ast).node_id();
            let key_name = match context.ast.kind_data(key) {
                NodeKindData::IdentifierName(identifier) => context
                    .ast
                    .get_utf8(identifier.name(context.ast))
                    .as_bytes(),
                NodeKindData::StringLiteral(string) => {
                    context.ast.get_wtf8(string.value(context.ast)).as_bytes()
                }
                _ => continue,
            };
            for previous_index in 0..index {
                let previous = attributes
                    .get_node(context.ast, previous_index)
                    .expect("import attribute exists");
                let previous_key = previous.key(context.ast).node_id();
                let (previous_key_name, previous_key_is_string_literal) =
                    match context.ast.kind_data(previous_key) {
                        NodeKindData::IdentifierName(identifier) => (
                            context
                                .ast
                                .get_utf8(identifier.name(context.ast))
                                .as_bytes(),
                            false,
                        ),
                        NodeKindData::StringLiteral(string) => (
                            context.ast.get_wtf8(string.value(context.ast)).as_bytes(),
                            true,
                        ),
                        _ => continue,
                    };
                if key_name == previous_key_name {
                    let name = if !previous_key_is_string_literal {
                        previous_key_name
                    } else {
                        key_name
                    };
                    let message = self.fmt(format_args!(
                        "Duplicate import attribute key '{}'",
                        String::from_utf8_lossy(name)
                    ));
                    self.report(
                        context.ast.span(key),
                        message,
                        ReportOptions {
                            labels: self.labels(&[
                                self.label(context.ast.span(previous_key), "first used here")
                            ]),
                            ..ReportOptions::default()
                        },
                    );
                    break;
                }
            }
        }
    }

    /// Records an exported name and reports a duplicate if one already exists.
    fn record_exported_name(
        &mut self,
        name: &[u8],
        node: NodeId,
        context: &mut SemanticVisitCtx<'_, 'a>,
    ) {
        // Exports inside a TS namespace are namespace-scoped, not module-scoped.
        if !context.ast.is_module() || context.state.in_ts_namespace() {
            return;
        }
        if let Some(&first) = self.exported_names.get(name) {
            let message = self.fmt(format_args!(
                "Duplicate export of '{}'",
                String::from_utf8_lossy(name)
            ));
            self.report(
                context.ast.span(node),
                message,
                ReportOptions {
                    labels: self.labels(&[
                        self.label(context.ast.span(first), "first exported here"),
                        self.label(context.ast.span(node), "exported again here"),
                    ]),
                    ..ReportOptions::default()
                },
            );
        } else {
            let key = name.clone_in(self.allocator);
            self.exported_names.insert(key, node);
        }
    }
}
