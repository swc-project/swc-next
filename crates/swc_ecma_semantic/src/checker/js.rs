use swc_next_ecma_ast::{
    AwaitExpression, BinaryExpression, BreakStatement, ContinueStatement, LabelIdentifier,
    LabeledStatement, MetaProperty, NumericLiteral, ObjectExpression, RegExpLiteral, StringLiteral,
    YieldExpression,
};

use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
enum PrivateElementKind {
    Field,
    Method,
    Getter,
    Setter,
}

/// Returns the StringValue of a class element's PrivateIdentifier key, if any.
fn private_element_name(tree: &Ast<'_>, node: NodeId) -> Option<AstUtf8> {
    let key = match tree.kind_data(node) {
        NodeKindData::PropertyDefinition(property) => property.key(tree).node_id(),
        NodeKindData::MethodDefinition(method) => method.key(tree).node_id(),
        NodeKindData::TsMethodDefinition(method) => method.key(tree).node_id(),
        _ => return None,
    };
    let NodeKindData::PrivateIdentifier(identifier) = tree.kind_data(key) else {
        return None;
    };
    Some(identifier.name(tree))
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
fn is_private_name_declared(context: &SemanticVisitCtx<'_, '_>, name: AstUtf8) -> bool {
    for ancestor in context.ancestors() {
        let NodeKindData::ClassBody(body) = context.ast.kind_data(ancestor) else {
            continue;
        };
        for child in body.body(context.ast).iter() {
            let child = context.ast.get_node_in_sub_range(child).node_id();
            if private_element_name(context.ast, child)
                .is_some_and(|candidate| context.ast.utf8_eq(candidate, name))
            {
                return true;
            }
        }
    }
    false
}

/// <https://tc39.es/ecma262/#sec-keywords-and-reserved-words>
fn match_strict_reserved(tree: &Ast<'_>, name: AstUtf8) -> Option<&'static str> {
    const WORDS: [&str; 9] = [
        "let",
        "yield",
        "static",
        "public",
        "private",
        "package",
        "protected",
        "interface",
        "implements",
    ];
    WORDS
        .into_iter()
        .find(|word| tree.utf8_eq_bytes(name, word.as_bytes()))
}

fn is_eval_or_arguments(tree: &Ast<'_>, name: AstUtf8) -> bool {
    tree.utf8_eq_bytes(name, b"eval") || tree.utf8_eq_bytes(name, b"arguments")
}

fn is_eval_or_arguments_reference(tree: &Ast<'_>, node: NodeId) -> bool {
    let NodeKindData::IdentifierReference(identifier) = tree.kind_data(node) else {
        return false;
    };
    is_eval_or_arguments(tree, identifier.name(tree))
}

fn unwrap_parentheses(tree: &Ast<'_>, node: NodeId) -> NodeId {
    let mut current = node;
    let mut depth = 0u32;
    // The parser builds at most one parenthesis node per syntactic
    // level, so this chain is bounded by source nesting depth.
    loop {
        debug_assert!(depth < 1_000_000);
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

fn find_label(context: &SemanticVisitCtx<'_, '_>, name: AstUtf8) -> LabelSearch {
    debug_assert!(!name.is_empty());
    let mut crossed_boundary = false;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::LabeledStatement(statement) => {
                let label = statement.label(context.ast).name(context.ast);
                if context.ast.utf8_eq(label, name) {
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

fn find_label_for_continue(
    context: &SemanticVisitCtx<'_, '_>,
    name: AstUtf8,
) -> ContinueLabelSearch {
    debug_assert!(!name.is_empty());
    let mut crossed_boundary = false;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::LabeledStatement(statement) => {
                let label = statement.label(context.ast).name(context.ast);
                if context.ast.utf8_eq(label, name) {
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
    let mut child = None;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::Function(_) => return true,
            NodeKindData::PropertyDefinition(property)
                if property
                    .value(context.ast)
                    .is_some_and(|value| Some(value.node_id()) == child) =>
            {
                return false;
            }
            NodeKindData::StaticBlock(_) => return false,
            NodeKindData::ArrowFunctionExpression(_) | NodeKindData::Program(_) => {}
            _ => {}
        }
        child = Some(ancestor);
    }
    true
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
fn module_export_name_id(tree: &Ast<'_>, node: NodeId) -> ModuleExportNameValue {
    match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) => {
            ModuleExportNameValue::Utf8(identifier.name(tree))
        }
        NodeKindData::IdentifierReference(identifier) => {
            ModuleExportNameValue::Utf8(identifier.name(tree))
        }
        NodeKindData::StringLiteral(literal) => ModuleExportNameValue::Wtf8(literal.value(tree)),
        _ => ModuleExportNameValue::Wtf8(AstWtf8::EMPTY),
    }
}

fn module_export_name<'b>(tree: &'b Ast<'_>, node: NodeId) -> Cow<'b, [u8]> {
    module_export_name_id(tree, node).bytes(tree)
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

impl<'a> CheckerState<'a> {
    /// Post-traversal check that all local export specifiers refer to
    /// declared bindings.
    pub fn check_unresolved_exports(
        &mut self,
        tree: &mut Ast<'a>,
        symbols: &crate::binder::SymbolTracker<'a>,
    ) {
        if !tree.is_module() {
            return;
        }
        for index in 0..self.export_specifiers.len() {
            let specifier = &self.export_specifiers[index];
            let local_name = match specifier.local_name {
                ModuleExportNameValue::Utf8(name) => Some(tree.intern_name(name)),
                ModuleExportNameValue::Wtf8(name) => {
                    tree.as_utf8(name).map(|name| tree.intern_name(name))
                }
            };
            let binding = local_name.and_then(|name| symbols.binding(ScopeId::MODULE, name));
            if binding.is_none() {
                let local_name = specifier.local_name.bytes(tree);
                let message = self.fmt(format_args!(
                    "Export '{}' is not defined",
                    String::from_utf8_lossy(&local_name)
                ));
                self.report(tree.span(specifier.node), message, ReportOptions::default());
            }
        }
    }

    #[cold]
    fn report_redeclaration(
        &mut self,
        tree: &Ast<'a>,
        identifier: BindingIdentifier,
        node: NodeId,
        existing_id: SymbolId,
        existing: Symbol,
        context: &SemanticContext<'a>,
    ) {
        let name = tree.utf8_chunks(identifier.name(tree)).to_cow();
        let name = name.as_bytes();
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
                help: Some(Cow::Owned(help)),
                labels: self.labels(&[
                    self.label(existing_span, first),
                    self.label(current_span, "cannot be redeclared here"),
                ]),
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

    #[allow(clippy::too_many_arguments)]
    fn check_redeclaration(
        &mut self,
        identifier: BindingIdentifier,
        node: NodeId,
        name_id: AstInternedUtf8,
        flags: SymbolFlags,
        context: &SemanticVisitCtx<'_, 'a>,
        existing_id: Option<SymbolId>,
    ) {
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
                && let Some(symbol_id) = context.state.symbols.own_binding(outer, name_id)
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
                let Some(symbol_id) = context.state.symbols.own_binding(scope_id, name_id) else {
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
        name: AstUtf8,
        node: NodeId,
        context: &SemanticVisitCtx<'_, 'a>,
        as_what: &'static str,
    ) {
        if !context.state.scope.is_strict() {
            return;
        }
        if let Some(word) = match_strict_reserved(context.ast, name) {
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
        context: &SemanticVisitCtx<'_, 'a>,
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
            NodeKindData::IdentifierReference(identifier)
                if is_eval_or_arguments(context.ast, identifier.name(context.ast)) =>
            {
                self.report(
                    context.ast.span(node),
                    "Cannot assign to 'eval' or 'arguments' in strict mode",
                    ReportOptions::default(),
                );
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
                NodeKindData::IdentifierName(identifier) => {
                    ImportAttributeName::Utf8(identifier.name(context.ast))
                }
                NodeKindData::StringLiteral(string) => {
                    ImportAttributeName::Wtf8(string.value(context.ast))
                }
                _ => continue,
            };
            for previous_index in 0..index {
                let previous = attributes
                    .get_node(context.ast, previous_index)
                    .expect("import attribute exists");
                let previous_key = previous.key(context.ast).node_id();
                let previous_key_name = match context.ast.kind_data(previous_key) {
                    NodeKindData::IdentifierName(identifier) => {
                        ImportAttributeName::Utf8(identifier.name(context.ast))
                    }
                    NodeKindData::StringLiteral(string) => {
                        ImportAttributeName::Wtf8(string.value(context.ast))
                    }
                    _ => continue,
                };
                if key_name.eq(previous_key_name, context.ast) {
                    let message = match key_name {
                        ImportAttributeName::Utf8(name) => self.fmt(format_args!(
                            "Duplicate import attribute key '{}'",
                            context.ast.utf8_chunks(name)
                        )),
                        ImportAttributeName::Wtf8(name) => self.fmt(format_args!(
                            "Duplicate import attribute key '{}'",
                            String::from_utf8_lossy(context.ast.get_wtf8(name).as_bytes())
                        )),
                    };
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
        context: &SemanticVisitCtx<'_, 'a>,
    ) {
        // TypeScript declarations can merge across the type and value namespaces. Duplicate
        // exports are therefore checker diagnostics rather than parser-level early errors.
        if context.ast.is_ts() {
            return;
        }
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
            self.exported_names.insert(name.to_vec(), node);
        }
    }
}

pub(super) fn check_program(checker: &mut Checker<'_, '_>) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_unresolved_exports(context.ast, &context.state.symbols);
}

pub(super) fn check_string_literal(checker: &mut Checker<'_, '_>, literal: StringLiteral) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.in_type_position() || !context.state.scope.is_strict() {
        return;
    }
    let span = context.ast.span(literal.node_id());
    if has_octal_escape(&context.ast.source().as_bytes()[span.start as usize..span.end as usize]) {
        checks.report(
            span,
            "Octal escape sequences are not allowed in strict mode",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Use \\xHH (hex) or \\uHHHH (unicode) escape instead",
                )),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_numeric_literal(checker: &mut Checker<'_, '_>, literal: NumericLiteral) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
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
            help: Some(Cow::Borrowed(help)),
            ..ReportOptions::default()
        },
    );
}

pub(super) fn check_reg_exp_literal(checker: &mut Checker<'_, '_>, literal: RegExpLiteral) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let pattern = context.ast.get_flat_utf8(literal.pattern(context.ast));
    let flags = context.ast.get_flat_utf8(literal.flags(context.ast));
    let allocator = Allocator::new();
    if swc_next_regular_expression::LiteralParser::new(
        &allocator,
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

pub(super) fn check_identifier_reference(
    checker: &mut Checker<'_, '_>,
    identifier: IdentifierReference,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.in_type_position() {
        return;
    }
    let node_id = identifier.node_id();
    let name = identifier.name(context.ast);
    checks.check_strict_reserved(name, node_id, context, "an identifier");
    if context.ast.utf8_eq_bytes(name, b"arguments") && !is_arguments_available(context) {
        checks.report(
            context.ast.span(node_id),
            "'arguments' is not allowed in class field initializers or static blocks",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_binding_identifier(
    checker: &mut Checker<'_, '_>,
    identifier: BindingIdentifier,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let node_id = identifier.node_id();
    let name_id = context
        .state
        .symbols
        .canonicalize_name(context.ast, identifier.name(context.ast));
    let name_handle = name_id.into();
    let flags = context.state.symbols.pending.flags;
    if context.state.in_type_position() && !flags.contains(SymbolFlags::TYPE_PARAMETER) {
        return;
    }
    if context.state.scope.is_strict() && !flags.contains(SymbolFlags::AMBIENT) {
        checks.check_strict_reserved(name_handle, node_id, context, "a binding identifier");
        if is_eval_or_arguments(context.ast, name_handle) {
            let name = context.ast.utf8_chunks(name_handle).to_cow();
            let message = checks.fmt(format_args!(
                "'{}' is not allowed as a binding identifier in strict mode",
                name
            ));
            checks.report(context.ast.span(node_id), message, ReportOptions::default());
        }
    } else if flags.contains(SymbolFlags::BLOCK_SCOPED_VAR)
        && context.ast.utf8_eq_bytes(name_handle, b"let")
    {
        checks.report(
            context.ast.span(node_id),
            "'let' is not allowed as a variable name in a lexical declaration",
            ReportOptions::default(),
        );
    }
    let existing = context
        .state
        .symbols
        .binding(context.state.symbols.pending.scope, name_id);
    checks.check_redeclaration(identifier, node_id, name_id, flags, context, existing);
    if context.state.symbols.export_state.named().is_some() {
        let skip = existing.is_some() && context.ast.is_ts();
        if !skip {
            let name = context.ast.utf8_chunks(name_handle).to_cow();
            checks.record_exported_name(name.as_bytes(), node_id, context);
        }
    }
}

pub(super) fn check_label_identifier(checker: &mut Checker<'_, '_>, identifier: LabelIdentifier) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_strict_reserved(
        identifier.name(context.ast),
        identifier.node_id(),
        context,
        "a label",
    );
}

pub(super) fn check_binary_expression(checker: &mut Checker<'_, '_>, expression: BinaryExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let left = expression.left(context.ast).node_id();
    if expression.operator(context.ast) == BinaryOperator::In
        && let NodeKindData::PrivateIdentifier(identifier) = context.ast.kind_data(left)
    {
        let name = identifier.name(context.ast);
        if !is_private_name_declared(context, name) {
            let name = context.ast.utf8_chunks(name).to_cow();
            let message = checks.fmt(format_args!(
                "Private field '#{}' must be declared in an enclosing class",
                name
            ));
            checks.report(context.ast.span(left), message, ReportOptions::default());
        }
    }
}

pub(super) fn check_unary_expression(checker: &mut Checker<'_, '_>, expression: UnaryExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if expression.operator(context.ast) != UnaryOperator::Delete {
        return;
    }
    let mut target = unwrap_parentheses(context.ast, expression.argument(context.ast).node_id());
    if let NodeKindData::ChainExpression(chain) = context.ast.kind_data(target) {
        target = unwrap_parentheses(context.ast, chain.expression(context.ast).node_id());
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

pub(super) fn check_update_expression(checker: &mut Checker<'_, '_>, expression: UpdateExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.scope.is_strict()
        && is_eval_or_arguments_reference(context.ast, expression.argument(context.ast).node_id())
    {
        checks.report(
            context.ast.span(expression.node_id()),
            "Cannot assign to 'eval' or 'arguments' in strict mode",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_assignment_expression(
    checker: &mut Checker<'_, '_>,
    expression: AssignmentExpression,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.scope.is_strict() {
        checks.check_assign_target_eval_arguments(expression.left(context.ast).node_id(), context);
    }
}

pub(super) fn check_variable_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: VariableDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
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

pub(super) fn check_switch_statement(checker: &mut Checker<'_, '_>, statement: SwitchStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
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
                    labels: checks.labels(&[
                        checks.label(context.ast.span(first), "first default defined here")
                    ]),
                    ..ReportOptions::default()
                },
            );
        } else {
            first_default = Some(case.node_id());
        }
    }
}

pub(super) fn check_for_in_statement(checker: &mut Checker<'_, '_>, statement: ForInStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let left = statement.left(context.ast).node_id();
    checks.check_for_in_of_initializer(context, left, true);
    if context.state.scope.is_strict() {
        checks.check_assign_target_eval_arguments(left, context);
    }
}

pub(super) fn check_for_of_statement(checker: &mut Checker<'_, '_>, statement: ForOfStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if statement.r#await(context.ast) && !context.ast.is_ts() && is_inside_static_block(context) {
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

pub(super) fn check_break_statement(checker: &mut Checker<'_, '_>, statement: BreakStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if let Some(label) = statement.label(context.ast) {
        let name = label.name(context.ast);
        let result = find_label(context, name);
        if result != LabelSearch::Found {
            let name = context.ast.utf8_chunks(name).to_cow();
            let message = match result {
                LabelSearch::NotFound => {
                    checks.fmt(format_args!("Use of undefined label '{}'", name))
                }
                LabelSearch::CrossedBoundary => checks.fmt(format_args!(
                    "Cannot break to label '{}' across function boundaries",
                    name
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
                help: Some(Cow::Borrowed(
                    "A 'break' statement can only be used within an enclosing iteration or switch statement",
                )),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_continue_statement(
    checker: &mut Checker<'_, '_>,
    statement: ContinueStatement,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if let Some(label) = statement.label(context.ast) {
        let name = label.name(context.ast);
        let result = find_label_for_continue(context, name);
        if result != ContinueLabelSearch::Found {
            let name = context.ast.utf8_chunks(name).to_cow();
            let (message, help) = match result {
                ContinueLabelSearch::NotFound => (
                    checks.fmt(format_args!("Use of undefined label '{}'", name)),
                    None,
                ),
                ContinueLabelSearch::CrossedBoundary => (
                    checks.fmt(format_args!(
                        "Cannot continue to label '{}' across function boundaries",
                        name
                    )),
                    None,
                ),
                ContinueLabelSearch::NotIteration => (
                    checks.fmt(format_args!(
                        "Label '{}' does not denote an iteration statement",
                        name
                    )),
                    Some(Cow::Borrowed(
                        "A 'continue' statement can only jump to a label of an enclosing 'for', 'while', 'do-while', 'for-in', or 'for-of' statement",
                    )),
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
                help: Some(Cow::Borrowed(
                    "A 'continue' statement can only be used within an enclosing iteration statement",
                )),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_labeled_statement(checker: &mut Checker<'_, '_>, statement: LabeledStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let name = statement.label(context.ast).name(context.ast);
    let duplicate = {
        let mut ancestors = context.ancestors();
        let _ = ancestors.next();
        let mut duplicate = false;
        for ancestor in ancestors {
            match context.ast.kind_data(ancestor) {
                NodeKindData::LabeledStatement(outer) => {
                    let outer_name = outer.label(context.ast).name(context.ast);
                    if context.ast.utf8_eq(outer_name, name) {
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
        let name = context.ast.utf8_chunks(name).to_cow();
        let message = checks.fmt(format_args!("Duplicate label '{}'", name));
        checks.report(
            context.ast.span(statement.label(context.ast).node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_directive(checker: &mut Checker<'_, '_>, directive: Directive) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if !context
        .ast
        .utf8_eq_bytes(directive.value(context.ast), b"use strict")
    {
        return;
    }
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
                parameter =
                    ecmascript::find_non_simple_parameter(context.ast, arrow.params(context.ast));
                break;
            }
            NodeKindData::Program(_) => break,
            _ => {}
        }
    }
    if let Some(parameter) = parameter {
        checks.report(
            context.ast.span(directive.node_id()),
            "Illegal 'use strict' directive in function with non-simple parameter list",
            ReportOptions {
                labels: checks
                    .labels(&[checks.label(context.ast.span(parameter), "non-simple parameter")]),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_class_body(checker: &mut Checker<'_, '_>, body: ClassBody) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let children = body.body(context.ast);
    let mut first_constructor = None;
    for index in 0..children.len() {
        let child = children
            .get_node(context.ast, index)
            .expect("class child exists")
            .node_id();
        // TypeScript permits constructor overload declarations and leaves duplicate
        // implementation diagnostics to its type checker.
        if !context.ast.is_ts()
            && let NodeKindData::MethodDefinition(method) = context.ast.kind_data(child)
            && method.kind(context.ast) == swc_next_ecma_ast::MethodDefinitionKind::Constructor
        {
            if let Some(first) = first_constructor {
                checks.report(
                    context.ast.span(method.key(context.ast).node_id()),
                    "A class can only have one constructor",
                    ReportOptions {
                        labels: checks
                            .labels(&[checks
                                .label(context.ast.span(first), "first constructor defined here")]),
                        ..ReportOptions::default()
                    },
                );
            } else {
                first_constructor = Some(method.key(context.ast).node_id());
            }
        }
        // Body-less TypeScript methods are erased overload signatures. They do not
        // introduce runtime private names and must not conflict with the implementation.
        if context.ast.node_kind(child) == NodeKind::TsMethodDefinition {
            continue;
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
            if context.ast.node_kind(previous) == NodeKind::TsMethodDefinition {
                continue;
            }
            let Some(previous_name) = private_element_name(context.ast, previous) else {
                continue;
            };
            if !context.ast.utf8_eq(previous_name, name) {
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
                context.ast.utf8_chunks(name)
            ));
            checks.report(
                context.ast.span(child),
                message,
                ReportOptions {
                    labels: checks
                        .labels(&[checks.label(context.ast.span(previous), "first declared here")]),
                    ..ReportOptions::default()
                },
            );
            break;
        }
    }
}

pub(super) fn check_function(checker: &mut Checker<'_, '_>, function: Function) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
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

pub(super) fn check_with_statement(checker: &mut Checker<'_, '_>, statement: WithStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.scope.is_strict() {
        checks.report(
            context.ast.span(statement.node_id()),
            "'with' statements are not allowed in strict mode",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_object_expression(checker: &mut Checker<'_, '_>, object: ObjectExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
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
        let is_proto = match context.ast.kind_data(item.key(context.ast).node_id()) {
            NodeKindData::IdentifierName(identifier) => context
                .ast
                .utf8_eq_bytes(identifier.name(context.ast), b"__proto__"),
            NodeKindData::StringLiteral(string) => {
                context.ast.get_wtf8(string.value(context.ast)).as_bytes() == b"__proto__"
            }
            _ => continue,
        };
        if !is_proto {
            continue;
        }
        if let Some(first) = first_proto {
            checks.report(
                context.ast.span(property),
                "Duplicate '__proto__' property in object literal",
                ReportOptions {
                    labels: checks
                        .labels(&[checks.label(context.ast.span(first), "first defined here")]),
                    ..ReportOptions::default()
                },
            );
        } else {
            first_proto = Some(property);
        }
    }
}

pub(super) fn check_member_expression(checker: &mut Checker<'_, '_>, expression: MemberExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let object = expression.object(context.ast).node_id();
    if context.ast.node_kind(object) == NodeKind::Super && !is_super_property_valid(context) {
        checks.report(
            context.ast.span(expression.node_id()),
            "'super' property access is only valid inside a method or class body",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Use an arrow function instead of a regular function to inherit the 'super' binding",
                )),
                ..ReportOptions::default()
            },
        );
    }
    let property = expression.property(context.ast).node_id();
    if !expression.computed(context.ast)
        && let NodeKindData::PrivateIdentifier(identifier) = context.ast.kind_data(property)
    {
        let name = identifier.name(context.ast);
        if context.ast.node_kind(object) == NodeKind::Super {
            checks.report(
                context.ast.span(expression.node_id()),
                "'super' keyword unexpected here",
                ReportOptions::default(),
            );
        } else if !is_private_name_declared(context, name) {
            let message = checks.fmt(format_args!(
                "Private field '#{}' must be declared in an enclosing class",
                context.ast.utf8_chunks(name)
            ));
            checks.report(
                context.ast.span(property),
                message,
                ReportOptions::default(),
            );
        }
    }
}

pub(super) fn check_call_expression(checker: &mut Checker<'_, '_>, call: CallExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.ast.node_kind(call.callee(context.ast).node_id()) == NodeKind::Super {
        let message = "'super()' is only valid in a constructor of a derived class";
        match super_call_validity(context) {
            SuperCallValidity::Valid => {}
            SuperCallValidity::NotInConstructor => checks.report(
                context.ast.span(call.node_id()),
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Use an arrow function instead of a regular function to inherit the 'super' binding",
                    )),
                    ..ReportOptions::default()
                },
            ),
            SuperCallValidity::NoExtends => checks.report(
                context.ast.span(call.node_id()),
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add an 'extends' clause to the class or remove the 'super()' call",
                    )),
                    ..ReportOptions::default()
                },
            ),
        }
    }
}

pub(super) fn check_await_expression(checker: &mut Checker<'_, '_>, expression: AwaitExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let message = await_expression_error(context);
    if let Some(message) = message {
        checks.report(
            context.ast.span(expression.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_yield_expression(checker: &mut Checker<'_, '_>, expression: YieldExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if let Some(message) = yield_expression_error(context) {
        checks.report(
            context.ast.span(expression.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

fn await_expression_error(context: &SemanticVisitCtx<'_, '_>) -> Option<&'static str> {
    const OUTSIDE_ASYNC: &str =
        "Await expressions are only allowed within async functions and at the top level of modules";

    let mut child = context.current_node()?;
    for ancestor in context.ancestors().skip(1) {
        match context.ast.kind_data(ancestor) {
            NodeKindData::FormalParameters(_) => {
                return Some("Await expression is not allowed in formal parameters");
            }
            NodeKindData::StaticBlock(_) => {
                return Some("Cannot use await in class static initialization block");
            }
            NodeKindData::TsEnumDeclaration(_)
            | NodeKindData::TsModuleBlock(_)
            | NodeKindData::TsModuleDeclaration(_) => return Some(OUTSIDE_ASYNC),
            NodeKindData::PropertyDefinition(property)
                if property
                    .value(context.ast)
                    .is_some_and(|value| value.node_id() == child) =>
            {
                return Some(OUTSIDE_ASYNC);
            }
            NodeKindData::Function(function) => {
                return (!function.r#async(context.ast)).then_some(OUTSIDE_ASYNC);
            }
            NodeKindData::ArrowFunctionExpression(function) => {
                return (!function.r#async(context.ast)).then_some(OUTSIDE_ASYNC);
            }
            NodeKindData::Program(_) => {
                return (context.ast.source_type() != swc_next_ecma_ast::SourceType::Module)
                    .then_some(OUTSIDE_ASYNC);
            }
            _ => {}
        }
        child = ancestor;
    }
    Some(OUTSIDE_ASYNC)
}

fn yield_expression_error(context: &SemanticVisitCtx<'_, '_>) -> Option<&'static str> {
    const OUTSIDE_GENERATOR: &str = "A yield expression is only allowed in a generator body";

    let mut child = context.current_node()?;
    for ancestor in context.ancestors().skip(1) {
        match context.ast.kind_data(ancestor) {
            NodeKindData::FormalParameters(_) => {
                return Some("Yield expression is not allowed in formal parameters");
            }
            NodeKindData::TsEnumDeclaration(_)
            | NodeKindData::TsModuleBlock(_)
            | NodeKindData::TsModuleDeclaration(_)
            | NodeKindData::StaticBlock(_)
            | NodeKindData::ArrowFunctionExpression(_)
            | NodeKindData::Program(_) => return Some(OUTSIDE_GENERATOR),
            NodeKindData::PropertyDefinition(property)
                if property
                    .value(context.ast)
                    .is_some_and(|value| value.node_id() == child) =>
            {
                return Some(OUTSIDE_GENERATOR);
            }
            NodeKindData::Function(function) => {
                return (!function.generator(context.ast)).then_some(OUTSIDE_GENERATOR);
            }
            _ => {}
        }
        child = ancestor;
    }
    Some(OUTSIDE_GENERATOR)
}

pub(super) fn check_meta_property(checker: &mut Checker<'_, '_>, property: MetaProperty) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let name = property.meta(context.ast).name(context.ast);
    if context.ast.utf8_eq_bytes(name, b"import") && !context.ast.is_module() {
        checks.report(
            context.ast.span(property.node_id()),
            "'import.meta' is only valid in module code",
            ReportOptions::default(),
        );
    }
    if context.ast.utf8_eq_bytes(name, b"new") && !is_new_target_available(context) {
        checks.report(
            context.ast.span(property.node_id()),
            "'new.target' is only valid inside functions, class field initializers, or static blocks",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_import_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ImportDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "import statement",
        "'import' declaration",
        context,
    );
    checks.check_duplicate_with_attributes(declaration.attributes(context.ast), context);
}

pub(super) fn check_export_named_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ExportNamedDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "'export' declaration",
        "'export' declaration",
        context,
    );
    checks.check_duplicate_with_attributes(declaration.attributes(context.ast), context);
}

pub(super) fn check_export_default_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ExportDefaultDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "'export default' declaration",
        "'export default' declaration",
        context,
    );
    checks.record_exported_name(b"default", declaration.node_id(), context);
}

pub(super) fn check_export_all_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ExportAllDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "'export *' declaration",
        "'export *' declaration",
        context,
    );
    if let Some(exported) = declaration.exported(context.ast) {
        let name = module_export_name(context.ast, exported.node_id());
        checks.record_exported_name(&name, declaration.node_id(), context);
    }
    checks.check_duplicate_with_attributes(declaration.attributes(context.ast), context);
}

pub(super) fn check_export_specifier(checker: &mut Checker<'_, '_>, specifier: ExportSpecifier) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let exported_name = module_export_name(context.ast, specifier.exported(context.ast).node_id());
    checks.record_exported_name(&exported_name, specifier.node_id(), context);
    if specifier.export_kind(context.ast) == ImportOrExportKind::Type
        || context.state.in_ts_namespace()
    {
        return;
    }
    if let Some(parent) = context.parent()
        && let NodeKindData::ExportNamedDeclaration(declaration) = context.ast.kind_data(parent)
        && declaration.source(context.ast).is_none()
        && declaration.export_kind(context.ast) != ImportOrExportKind::Type
    {
        let local_name = module_export_name_id(context.ast, specifier.local(context.ast).node_id());
        checks.export_specifiers.push(ExportSpecifierInfo {
            local_name,
            node: specifier.node_id(),
        });
    }
}
