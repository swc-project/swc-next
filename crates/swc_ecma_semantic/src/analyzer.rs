use std::{borrow::Cow, marker::PhantomData, string::String};

use swc_experimental_ecma_ast::String as AstString;
use swc_experimental_ecma_ast::*;

use crate::{
    scope::{ScopeFlags, ScopeKind, ScopeTree},
    symbol::{ReferenceKind, SymbolFlags, SymbolTable},
};

pub(crate) struct AnalysisParts {
    pub scopes: ScopeTree,
    pub symbols: SymbolTable,
    pub node_scopes: Vec<ScopeId>,
    pub diagnostics: Vec<Diagnostic<'static>>,
}

#[derive(Clone, Copy)]
struct BindingContext {
    flags: SymbolFlags,
    excludes: SymbolFlags,
    target: ScopeId,
    export_state: ExportState,
    ambient: bool,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum ExportState {
    #[default]
    None,
    Named,
    Default,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PrivateElementKind {
    Field,
    Method,
    Getter,
    Setter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SuperCallValidity {
    Valid,
    NotInConstructor,
    NoExtends,
}

struct SemanticBuilder<'a> {
    scopes: ScopeTree,
    symbols: SymbolTable,
    node_scopes: Vec<ScopeId>,
    current_scope: ScopeId,
    scope_frames: Vec<(NodeId, u8)>,
    binding_contexts: Vec<BindingContext>,
    binding_frames: Vec<NodeId>,
    catch_bodies: Vec<NodeId>,
    write_targets: Vec<Option<NodeId>>,
    breakable_depth: u32,
    iteration_depth: u32,
    control_frames: Vec<(u32, u32, usize)>,
    labels: Vec<(AstString, bool, Span)>,
    // A declaration file is ambient in its entirety.
    ambient: bool,
    diagnostics: Vec<Diagnostic<'static>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> SemanticBuilder<'a> {
    fn new(program: Program, ast: &Ast<'a>) -> Self {
        let scopes = ScopeTree::new(
            program.node_id(),
            program.source_type(ast) == SourceType::Module,
            has_use_strict_directive(program.body(ast), ast),
        );
        let current_scope = scopes.top_level_scope_id();
        let mut node_scopes = vec![current_scope; ast.node_count()];
        node_scopes[program.node_id().index()] = current_scope;
        let symbols = SymbolTable::new(scopes.len());
        Self {
            scopes,
            symbols,
            node_scopes,
            current_scope,
            scope_frames: Vec::new(),
            binding_contexts: Vec::new(),
            binding_frames: Vec::new(),
            catch_bodies: Vec::new(),
            write_targets: Vec::new(),
            breakable_depth: 0,
            iteration_depth: 0,
            control_frames: Vec::new(),
            labels: Vec::new(),
            ambient: ast.lang() == Lang::Dts,
            diagnostics: Vec::new(),
            marker: PhantomData,
        }
    }

    fn enter_scope(&mut self, node: NodeId, kind: ScopeKind, flags: ScopeFlags, count: u8) {
        let scope = self.scopes.push(self.current_scope, node, kind, flags);
        self.symbols.add_scope();
        self.current_scope = scope;
        self.node_scopes[node.index()] = scope;
        if count == 1 {
            self.scope_frames.push((node, 1));
        } else {
            let last = self
                .scope_frames
                .last_mut()
                .expect("the first scope in a multi-scope frame exists");
            debug_assert_eq!(last.0, node);
            last.1 = count;
        }
    }

    fn push_scope(&mut self, node: NodeId, kind: ScopeKind, flags: ScopeFlags) {
        self.enter_scope(node, kind, flags, 1);
    }

    fn expr_name_scope(&mut self, node: NodeId, body_kind: ScopeKind, flags: ScopeFlags) {
        // A named function/class expression's name lives in an intermediate
        // scope between the outer scope and the body scope.
        self.push_scope(node, ScopeKind::ExpressionName, flags);
        self.enter_scope(node, body_kind, flags, 2);
    }

    fn exit_scope(&mut self, node: NodeId) {
        let Some(&(frame_node, count)) = self.scope_frames.last() else {
            return;
        };
        if frame_node != node {
            return;
        }
        self.scope_frames.pop();
        for _ in 0..count {
            self.pop_scope();
        }
    }

    fn pop_scope(&mut self) {
        self.current_scope = self
            .current_scope()
            .parent
            .expect("a semantic child scope always has a parent");
    }

    fn current_scope(&self) -> &crate::scope::Scope {
        self.scopes.get(self.current_scope)
    }

    fn current_hoist_scope_id(&self) -> ScopeId {
        self.current_scope().hoist_target
    }

    fn is_strict(&self) -> bool {
        self.current_scope().flags.strict
    }

    fn inherited_flags(&self) -> ScopeFlags {
        ScopeFlags {
            strict: self.is_strict(),
        }
    }

    fn enter_control_boundary(&mut self) {
        self.control_frames.push((
            self.breakable_depth,
            self.iteration_depth,
            self.labels.len(),
        ));
        self.breakable_depth = 0;
        self.iteration_depth = 0;
    }

    fn exit_control_boundary(&mut self) {
        let (breakable, iteration, labels) = self
            .control_frames
            .pop()
            .expect("function control-flow boundary is balanced");
        self.breakable_depth = breakable;
        self.iteration_depth = iteration;
        self.labels.truncate(labels);
    }

    fn visible_labels(&self) -> &[(AstString, bool, Span)] {
        let start = self
            .control_frames
            .last()
            .map_or(0, |(_, _, labels)| *labels);
        &self.labels[start..]
    }

    fn is_inside_breakable(&self) -> bool {
        self.breakable_depth > 0
    }

    fn is_inside_iteration(&self) -> bool {
        self.iteration_depth > 0
    }

    fn find_label(&self, name: AstString, ast: &Ast<'_>) -> bool {
        self.visible_labels()
            .iter()
            .any(|(candidate, _, _)| ast.string(*candidate) == ast.string(name))
    }

    fn find_label_for_continue(&self, name: AstString, ast: &Ast<'_>) -> Option<bool> {
        self.visible_labels()
            .iter()
            .rev()
            .find(|(candidate, _, _)| ast.string(*candidate) == ast.string(name))
            .map(|(_, iteration, _)| *iteration)
    }

    fn check_label_identifier(&mut self, label: LabelIdentifier, ast: &Ast<'a>) {
        if self.is_strict() {
            check_strict_reserved(
                &mut self.diagnostics,
                ast.string(label.name(ast)),
                label.span(ast),
                "a label",
            );
        }
    }

    fn enter_label_identifier(&mut self, label: LabelIdentifier, ast: &Ast<'a>) {
        self.check_label_identifier(label, ast);
    }

    fn check_import_export_position(
        &mut self,
        node: impl AstNode + GetSpan,
        description: &str,
        ctx: &TraverseCtx<'_, 'a>,
    ) {
        if !is_at_program_level(ctx) {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Owned(format!("{description} may only appear at the top level")),
                span: node.span(ctx.ast),
                help: None,
                labels: Vec::new(),
            });
        } else if self.scopes.get(self.scopes.top_level_scope_id()).kind != ScopeKind::Module
            && !is_in_ts_namespace(ctx)
        {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Owned(format!("{description} is only valid in module code")),
                span: node.span(ctx.ast),
                help: None,
                labels: Vec::new(),
            });
        }
    }

    fn check_duplicate_with_clause_attributes(
        &mut self,
        attributes: TypedSubRange<ImportAttribute>,
        ast: &Ast<'a>,
    ) {
        let mut seen = Vec::<(String, Span)>::new();
        for index in 0..attributes.len() {
            let attribute = attributes
                .get_node(ast, index)
                .expect("import attribute ranges contain their nodes");
            let Some(key) = get_module_export_name(attribute.key(ast), ast) else {
                continue;
            };
            if let Some((_, previous)) = seen.iter().find(|(existing, _)| existing == &key) {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Borrowed("Duplicate import attribute"),
                    span: attribute.key(ast).span(ast),
                    help: None,
                    labels: vec![Label {
                        span: *previous,
                        message: Cow::Borrowed("first attribute is here"),
                    }],
                });
            } else {
                seen.push((key, attribute.key(ast).span(ast)));
            }
        }
    }

    fn set_binding_context(&mut self, node: NodeId, context: BindingContext) {
        self.push_saved_context(node, context);
    }

    fn push_saved_context(&mut self, node: NodeId, context: BindingContext) {
        self.binding_frames.push(node);
        self.binding_contexts.push(context);
    }

    fn exit_binding_context(&mut self, node: NodeId) {
        self.pop_saved_context(node);
    }

    fn pop_saved_context(&mut self, node: NodeId) {
        if self.binding_frames.last().copied() == Some(node) {
            self.binding_frames.pop();
            self.binding_contexts.pop();
        }
    }

    /// Materializes the pending binding context into a symbol.
    ///
    /// Compatible declarations merge into an existing symbol. Conflicting
    /// declarations keep the existing flags but still attach the new node to
    /// that symbol so tooling can map bindings in invalid source.
    fn declare_binding(&mut self, node: BindingIdentifier, ctx: &mut TraverseCtx<'_, 'a>) {
        if is_write_position(node.node_id(), ctx, &self.write_targets) {
            return;
        }
        let Some(mut binding) = self.binding_contexts.last().copied() else {
            return;
        };
        if binding.ambient || self.ambient {
            binding.flags = binding.flags.union(SymbolFlags::AMBIENT);
        }
        let in_type_position = ctx
            .ancestors()
            .skip(1)
            .any(|ancestor| is_type_node(ctx.node_kind(ancestor)));
        if in_type_position {
            // Identifiers in type-position parameter lists are labels. Only
            // type parameters and mapped-type keys are actual declarations.
            let is_type_parameter_name =
                ctx.parent()
                    .is_some_and(|parent| match ctx.ast.kind_data(parent) {
                        NodeKindData::TsTypeParameter(parameter) => {
                            parameter.name(ctx.ast).node_id() == node.node_id()
                        }
                        NodeKindData::TsMappedType(mapped_type) => {
                            mapped_type.key(ctx.ast).node_id() == node.node_id()
                        }
                        _ => false,
                    });
            if !is_type_parameter_name {
                return;
            }
            debug_assert!(binding.flags.intersects(SymbolFlags::TYPE_PARAMETER));
        }
        let export_kind =
            ctx.ancestors()
                .skip(1)
                .find_map(|ancestor| match ctx.node_kind(ancestor) {
                    NodeKind::Function
                    | NodeKind::ArrowFunctionExpression
                    | NodeKind::FunctionBody
                    | NodeKind::ClassBody
                    | NodeKind::TsModuleBlock => Some(None),
                    NodeKind::ExportNamedDeclaration => Some(Some(false)),
                    NodeKind::ExportDefaultDeclaration => Some(Some(true)),
                    _ => None,
                });
        binding.export_state = match export_kind {
            Some(Some(true)) => ExportState::Default,
            Some(Some(false)) => ExportState::Named,
            _ => binding.export_state,
        };
        if binding.export_state != ExportState::None {
            binding.flags = binding.flags.union(SymbolFlags::EXPORTED);
            if binding.export_state == ExportState::Default {
                binding.flags = binding.flags.union(SymbolFlags::DEFAULT_EXPORT);
            }
        }

        let name = node.name(ctx.ast);
        let name_bytes = ctx.ast.string(name);
        let name_text = String::from_utf8_lossy(name_bytes).into_owned();
        if name_bytes == b"let" && binding.flags.intersects(SymbolFlags::BLOCK_SCOPED_VAR) {
            self.diagnostics.push(Diagnostic::error(
                "'let' may not be used as a lexical binding name",
                node.span(ctx.ast),
            ));
        }
        let result = self.symbols.declare(
            name,
            name_bytes,
            binding.flags,
            binding.excludes,
            binding.target,
            node.node_id(),
            node.span(ctx.ast),
        );
        if binding.flags.is_hoisting_var() {
            // Register the hoisting `var` in every block it passes through so
            // block-scoped redeclarations can detect the conflict.
            if let Some(previous) = self.find_hoisting_conflict(name_bytes, binding.target) {
                self.report(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Owned(format!(
                        "Variable '{}' conflicts with a lexical declaration",
                        name_text
                    )),
                    span: node.span(ctx.ast),
                    help: None,
                    labels: vec![Label {
                        span: previous,
                        message: Cow::Borrowed("lexical declaration is here"),
                    }],
                });
            }
            self.symbols.register_hoist(
                &self.scopes,
                self.current_scope,
                binding.target,
                name_bytes,
                result.symbol_id,
            );
        }
        let annex_b_duplicate_function = result.conflict.is_some()
            && !self.scopes.get(self.current_scope).flags.strict
            && self.scopes.get(binding.target).kind == ScopeKind::Block
            && self
                .symbols
                .symbol(result.symbol_id)
                .flags
                .intersects(SymbolFlags::FUNCTION)
            && self
                .symbols
                .symbol(result.symbol_id)
                .declarations
                .iter()
                .copied()
                .all(|declaration| {
                    scope_function_declaration_is_ordinary(
                        self.scopes.get(binding.target).node,
                        declaration,
                        ctx.ast,
                    )
                })
            && ctx
                .parent()
                .is_some_and(|parent| match ctx.ast.kind_data(parent) {
                    NodeKindData::Function(function) => {
                        function.function_type(ctx.ast) == FunctionType::FunctionDeclaration
                            && !function.r#async(ctx.ast)
                            && !function.generator(ctx.ast)
                    }
                    _ => false,
                });
        if let Some(previous) = result.conflict.filter(|_| !annex_b_duplicate_function) {
            self.report_redeclaration(&name_text, node.span(ctx.ast), previous);
        }

        let strict = self.scopes.get(self.current_scope).flags.strict;
        if strict && is_eval_or_arguments(name_bytes) {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Owned(format!(
                    "'{}' is not allowed as a binding identifier in strict mode",
                    name_text
                )),
                span: node.span(ctx.ast),
                help: None,
                labels: Vec::new(),
            });
        }
        if strict {
            check_strict_reserved(
                &mut self.diagnostics,
                name_bytes,
                node.span(ctx.ast),
                "a binding identifier",
            );
        }
        node.set_symbol_id(ctx.ast, Some(result.symbol_id));
    }

    /// Finds a block-scoped declaration crossed by a hoisting `var`.
    ///
    /// ECMAScript 14.2.1 makes this an early error in every intermediate
    /// lexical scope.
    fn find_hoisting_conflict(&self, name: &[u8], target: ScopeId) -> Option<Span> {
        for scope in self.scopes.ancestors(self.current_scope) {
            if scope == target {
                break;
            }
            if let Some(existing) = self.symbols.find_in_scope(scope, name) {
                let symbol = self.symbols.symbol(existing);
                if symbol.flags.is_block_scoped_like() {
                    return symbol.declaration_spans.first().copied();
                }
            }
        }
        None
    }

    fn report_redeclaration(&mut self, name: &str, span: Span, previous: Span) {
        self.report(Diagnostic {
            severity: Severity::Error,
            message: Cow::Owned(format!("Identifier '{name}' has already been declared")),
            span,
            help: None,
            labels: vec![Label {
                span: previous,
                message: Cow::Borrowed("previous declaration is here"),
            }],
        });
    }

    fn report(&mut self, diagnostic: Diagnostic<'static>) {
        self.diagnostics.push(diagnostic);
    }

    fn poison_visible_scopes(&mut self, start: ScopeId) {
        let scopes = self.scopes.ancestors(start).collect::<Vec<_>>();
        for scope in scopes {
            self.scopes.get_mut(scope).rename_unsafe = true;
        }
    }

    fn mark_direct_eval(&mut self) {
        self.scopes.get_mut(self.current_scope).has_direct_eval = true;
        self.poison_visible_scopes(self.current_scope);
    }

    fn function_is_strict(&self, function: Function, ast: &Ast<'a>) -> bool {
        self.inherited_flags().strict || self.has_retroactive_use_strict(function.body(ast), ast)
    }

    fn has_retroactive_use_strict(&self, body: Option<FunctionBody>, ast: &Ast<'a>) -> bool {
        // "use strict" applies retroactively to the whole function, including
        // its parameter list, so inspect the body before entering parameters.
        body.is_some_and(|body| has_use_strict_directive(body.body(ast), ast))
    }

    fn arrow_is_strict(&self, arrow: ArrowFunctionExpression, ast: &Ast<'a>) -> bool {
        self.inherited_flags().strict
            || match ast.arrow_function_body_data(arrow.body(ast)) {
                ArrowFunctionBodyData::FunctionBody(body) => {
                    has_use_strict_directive(body.body(ast), ast)
                }
                ArrowFunctionBodyData::Expr(_) => false,
            }
    }

    /// Checks the `ForBinding` restrictions from ECMAScript 14.7.5.1:
    /// exactly one bound declaration and no initializer.
    fn check_for_in_of_initializer(&mut self, left: ForStatementLeft, ast: &Ast<'a>) {
        let ForStatementLeftData::VariableDeclaration(declaration) =
            ast.for_statement_left_data(left)
        else {
            return;
        };
        let declarators = declaration.declarators(ast);
        if declarators.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                "Only one variable declaration is allowed in a for-in/of statement",
                declaration.span(ast),
            ));
            return;
        }
        let declarator = declarators
            .get_node(ast, 0)
            .expect("a one-element declarator range contains its node");
        if declarator.init(ast).is_some() {
            self.diagnostics.push(Diagnostic::error(
                "A for-in/of variable declaration may not have an initializer",
                declarator.span(ast),
            ));
        }
    }

    /// Checks whether `super()` is inside a derived constructor.
    ///
    /// Arrow functions are transparent; regular functions stop the search.
    fn super_call_validity(&self, ctx: &TraverseCtx<'_, 'a>) -> SuperCallValidity {
        let mut ancestors = ctx.ancestors().skip(1);
        while let Some(ancestor) = ancestors.next() {
            match ctx.ast.kind_data(ancestor) {
                NodeKindData::ArrowFunctionExpression(_) => {}
                NodeKindData::Function(_) => {
                    if let Some(parent) = ancestors.next()
                        && let NodeKindData::MethodDefinition(method) = ctx.ast.kind_data(parent)
                        && method.kind(ctx.ast) == MethodDefinitionKind::Constructor
                    {
                        return if ancestors.any(|ancestor| {
                            matches!(
                                ctx.ast.kind_data(ancestor),
                                NodeKindData::Class(class) if class.super_class(ctx.ast).is_some()
                            )
                        }) {
                            SuperCallValidity::Valid
                        } else {
                            SuperCallValidity::NoExtends
                        };
                    }
                    return SuperCallValidity::NotInConstructor;
                }
                NodeKindData::PropertyDefinition(_)
                | NodeKindData::StaticBlock(_)
                | NodeKindData::Program(_) => {
                    return SuperCallValidity::NotInConstructor;
                }
                _ => {}
            }
        }
        SuperCallValidity::NotInConstructor
    }

    /// Checks whether `super.property` inherits a valid method or class-field
    /// environment.
    fn is_super_property_valid(&self, ctx: &TraverseCtx<'_, 'a>) -> bool {
        let mut ancestors = ctx.ancestors().skip(1);
        while let Some(ancestor) = ancestors.next() {
            match ctx.ast.kind_data(ancestor) {
                NodeKindData::ArrowFunctionExpression(_) => {}
                NodeKindData::Function(_) => {
                    return ancestors.next().is_some_and(|parent| {
                        match ctx.ast.kind_data(parent) {
                            NodeKindData::MethodDefinition(_) => true,
                            NodeKindData::ObjectProperty(property) => {
                                property.method(ctx.ast)
                                    || property.kind(ctx.ast) != PropertyKind::Init
                            }
                            _ => false,
                        }
                    });
                }
                NodeKindData::PropertyDefinition(_) | NodeKindData::StaticBlock(_) => return true,
                NodeKindData::Program(_) => return false,
                _ => {}
            }
        }
        false
    }

    fn enter(&mut self, node_id: NodeId) {
        self.node_scopes[node_id.index()] = self.current_scope;
    }

    fn exit(&mut self, node_id: NodeId) {
        debug_assert!(node_id.index() < self.node_scopes.len());
    }
}

impl<'a> Traverse<'a> for SemanticBuilder<'a> {
    fn enter_node(
        &mut self,
        node_id: NodeId,
        _kind: NodeKind,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.enter(node_id);
        TraverseAction::Proceed
    }

    fn exit_node(&mut self, node_id: NodeId, _kind: NodeKind, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit(node_id);
    }

    fn enter_function(&mut self, node: Function, ctx: &mut TraverseCtx<'_, 'a>) -> TraverseAction {
        self.enter_control_boundary();
        let outer_scope = self.current_scope;
        let strict = self.function_is_strict(node, ctx.ast);
        if node.function_type(ctx.ast) == FunctionType::FunctionDeclaration {
            let mut through_label = false;
            let parent = ctx.ancestors().skip(1).find(|ancestor| {
                if ctx.node_kind(*ancestor) == NodeKind::LabeledStatement {
                    through_label = true;
                    false
                } else {
                    true
                }
            });
            let valid = parent.is_some_and(|parent| match ctx.node_kind(parent) {
                NodeKind::Program
                | NodeKind::FunctionBody
                | NodeKind::ClassBody
                | NodeKind::StaticBlock
                | NodeKind::BlockStatement
                | NodeKind::SwitchCase
                | NodeKind::ExportNamedDeclaration
                | NodeKind::ExportDefaultDeclaration
                | NodeKind::TsModuleBlock => !strict || !through_label,
                NodeKind::IfStatement => !strict && !through_label,
                _ => false,
            });
            if !valid {
                self.diagnostics.push(Diagnostic::error(
                    "A function declaration is not allowed in this statement position",
                    node.span(ctx.ast),
                ));
            }
        }
        let flags = ScopeFlags { strict };
        let binding_target = if is_named_function_expression(node, ctx.ast) {
            // Expression names bind outside the function body but inside the
            // surrounding scope.
            self.expr_name_scope(node.node_id(), ScopeKind::Function, flags);
            self.scopes
                .get(self.current_scope)
                .parent
                .expect("named function expression has a name scope")
        } else {
            self.push_scope(node.node_id(), ScopeKind::Function, flags);
            outer_scope
        };
        let target_kind = self.scopes.get(binding_target).kind;
        let allow_overload = ctx.ast.lang().is_typescript()
            || matches!(
                target_kind,
                ScopeKind::Function | ScopeKind::Global | ScopeKind::StaticBlock
            );
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::FUNCTION,
                excludes: SymbolFlags::function_excludes(allow_overload),
                target: binding_target,
                export_state: ExportState::None,
                ambient: node.declare(ctx.ast) || self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_function(&mut self, node: Function, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
        self.exit_scope(node.node_id());
        self.exit_control_boundary();
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.enter_control_boundary();
        let strict = self.arrow_is_strict(node, ctx.ast);
        self.push_scope(node.node_id(), ScopeKind::Function, ScopeFlags { strict });
        TraverseAction::Proceed
    }

    fn enter_directive(
        &mut self,
        node: Directive,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if ctx.ast.string(node.value(ctx.ast)) != b"use strict" {
            return TraverseAction::Proceed;
        }
        let params =
            ctx.ancestors()
                .skip(1)
                .find_map(|ancestor| match ctx.ast.kind_data(ancestor) {
                    NodeKindData::Function(function) => Some(function.params(ctx.ast)),
                    NodeKindData::ArrowFunctionExpression(function) => {
                        Some(function.params(ctx.ast))
                    }
                    NodeKindData::Program(_) => None,
                    _ => None,
                });
        if let Some(params) =
            params.filter(|params| formal_parameters_are_non_simple(*params, ctx.ast))
        {
            // FunctionBodyContainsUseStrict with a non-simple parameter list
            // is an early error.
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Borrowed(
                    "Illegal 'use strict' directive in function with non-simple parameter list",
                ),
                span: node.span(ctx.ast),
                help: None,
                labels: vec![Label {
                    span: params.span(ctx.ast),
                    message: Cow::Borrowed("non-simple parameters"),
                }],
            });
        }
        TraverseAction::Proceed
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_scope(node.node_id());
        self.exit_control_boundary();
    }

    fn enter_formal_parameters(
        &mut self,
        node: FormalParameters,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        // Duplicate parameters are only allowed for sloppy functions with a
        // simple parameter list. Strict, arrow, unique, and non-simple lists
        // use normal block-scoped conflict rules.
        let excludes = if self.scopes.get(self.current_scope).flags.strict
            || formal_parameters_are_non_simple(node, ctx.ast)
            || matches!(
                node.kind(ctx.ast),
                FormalParameterKind::UniqueFormalParameters
                    | FormalParameterKind::ArrowFormalParameters
            ) {
            SymbolFlags::block_var_excludes()
        } else {
            SymbolFlags::parameter_excludes()
        };
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::FUNCTION_SCOPED_VAR.union(SymbolFlags::PARAMETER),
                excludes,
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_formal_parameters(&mut self, node: FormalParameters, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_block_statement(
        &mut self,
        node: BlockStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        // The catch parameter and its block body share one environment, so a
        // catch body reuses the scope created by the catch clause.
        if self.catch_bodies.last().copied() != Some(node.node_id()) {
            self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        }
        TraverseAction::Proceed
    }

    fn exit_block_statement(&mut self, node: BlockStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_scope(node.node_id());
    }

    fn enter_for_statement(
        &mut self,
        node: ForStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        TraverseAction::Proceed
    }

    fn exit_for_statement(&mut self, node: ForStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
        self.exit_scope(node.node_id());
    }

    fn enter_for_in_statement(
        &mut self,
        node: ForInStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_for_in_of_initializer(node.left(_ctx.ast), _ctx.ast);
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        self.write_targets
            .push(for_left_write_target(node.left(_ctx.ast), _ctx.ast));
        TraverseAction::Proceed
    }

    fn exit_for_in_statement(&mut self, node: ForInStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
        self.exit_scope(node.node_id());
    }

    fn enter_for_of_statement(
        &mut self,
        node: ForOfStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_for_in_of_initializer(node.left(_ctx.ast), _ctx.ast);
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        self.write_targets
            .push(for_left_write_target(node.left(_ctx.ast), _ctx.ast));
        TraverseAction::Proceed
    }

    fn exit_for_of_statement(&mut self, node: ForOfStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
        self.exit_scope(node.node_id());
    }

    fn enter_switch_statement(
        &mut self,
        node: SwitchStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        let mut default_span = None;
        for case_id in node.cases(_ctx.ast).iter() {
            let case = _ctx.ast.get_node_in_sub_range(case_id);
            if case.test(_ctx.ast).is_some() {
                continue;
            }
            if let Some(previous) = default_span {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Borrowed("A switch statement can only have one default clause"),
                    span: case.span(_ctx.ast),
                    help: None,
                    labels: vec![Label {
                        span: previous,
                        message: Cow::Borrowed("previous default clause is here"),
                    }],
                });
            } else {
                default_span = Some(case.span(_ctx.ast));
            }
        }
        TraverseAction::Proceed
    }

    fn exit_switch_statement(&mut self, node: SwitchStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.breakable_depth -= 1;
        self.exit_scope(node.node_id());
    }

    fn enter_while_statement(
        &mut self,
        _node: WhileStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        TraverseAction::Proceed
    }

    fn exit_while_statement(&mut self, _node: WhileStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
    }

    fn enter_do_while_statement(
        &mut self,
        _node: DoWhileStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        TraverseAction::Proceed
    }

    fn exit_do_while_statement(&mut self, _node: DoWhileStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
    }

    fn enter_labeled_statement(
        &mut self,
        node: LabeledStatement,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let label = node.label(ctx.ast);
        let name = label.name(ctx.ast);
        self.enter_label_identifier(label, ctx.ast);
        if let Some((_, _, previous)) = self
            .visible_labels()
            .iter()
            .rev()
            .find(|(existing, _, _)| ctx.ast.string(*existing) == ctx.ast.string(name))
        {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Owned(format!(
                    "Label '{}' has already been declared",
                    String::from_utf8_lossy(ctx.ast.string(name))
                )),
                span: label.span(ctx.ast),
                help: None,
                labels: vec![Label {
                    span: *previous,
                    message: Cow::Borrowed("previous label is here"),
                }],
            });
        }
        let iteration = is_iteration(node.body(ctx.ast), ctx.ast);
        self.labels.push((name, iteration, label.span(ctx.ast)));
        TraverseAction::Proceed
    }

    fn exit_labeled_statement(&mut self, _node: LabeledStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.labels.pop();
    }

    fn enter_break_statement(
        &mut self,
        node: BreakStatement,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if let Some(label) = node.label(ctx.ast) {
            let name = label.name(ctx.ast);
            if !self.find_label(name, ctx.ast) {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Owned(format!(
                        "Undefined label '{}'",
                        String::from_utf8_lossy(ctx.ast.string(name))
                    )),
                    span: node.span(ctx.ast),
                    help: None,
                    labels: Vec::new(),
                });
            }
        } else if !self.is_inside_breakable() {
            self.diagnostics.push(Diagnostic::error(
                "A 'break' statement can only be used within a loop or switch",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_continue_statement(
        &mut self,
        node: ContinueStatement,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if let Some(label) = node.label(ctx.ast) {
            let name = label.name(ctx.ast);
            match self.find_label_for_continue(name, ctx.ast) {
                Some(true) => {}
                Some(false) => self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Owned(format!(
                        "Label '{}' does not denote an iteration statement",
                        String::from_utf8_lossy(ctx.ast.string(name))
                    )),
                    span: node.span(ctx.ast),
                    help: None,
                    labels: Vec::new(),
                }),
                None => self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Owned(format!(
                        "Undefined label '{}'",
                        String::from_utf8_lossy(ctx.ast.string(name))
                    )),
                    span: node.span(ctx.ast),
                    help: None,
                    labels: Vec::new(),
                }),
            }
        } else if !self.is_inside_iteration() {
            self.diagnostics.push(Diagnostic::error(
                "A 'continue' statement can only be used within a loop",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_with_statement(
        &mut self,
        node: WithStatement,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if self.scopes.get(self.current_scope).flags.strict {
            self.diagnostics.push(Diagnostic::error(
                "'with' statements are not allowed in strict mode",
                node.span(ctx.ast),
            ));
        }
        self.scopes.get_mut(self.current_scope).has_with = true;
        self.poison_visible_scopes(self.current_scope);
        TraverseAction::Proceed
    }

    fn enter_yield_expression(
        &mut self,
        node: YieldExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if is_in_formal_parameters(ctx) {
            self.diagnostics.push(Diagnostic::error(
                "A yield expression is not allowed in formal parameters",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_await_expression(
        &mut self,
        node: AwaitExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if is_in_formal_parameters(ctx) {
            self.diagnostics.push(Diagnostic::error(
                "An await expression is not allowed in formal parameters",
                node.span(ctx.ast),
            ));
        }
        if ctx
            .ancestors()
            .skip(1)
            .take_while(|ancestor| {
                !matches!(
                    ctx.node_kind(*ancestor),
                    NodeKind::Function | NodeKind::ArrowFunctionExpression | NodeKind::Program
                )
            })
            .any(|ancestor| ctx.node_kind(ancestor) == NodeKind::StaticBlock)
        {
            self.diagnostics.push(Diagnostic::error(
                "Await is not allowed in a class static block",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if ctx
            .ancestors()
            .skip(1)
            .any(|ancestor| is_type_node(ctx.node_kind(ancestor)))
        {
            return TraverseAction::Proceed;
        }
        let name = node.name(ctx.ast);
        let name = ctx.ast.string(name);
        if self.inherited_flags().strict {
            check_strict_reserved(
                &mut self.diagnostics,
                name,
                node.span(ctx.ast),
                "an identifier reference",
            );
        }
        if name == b"arguments" && !is_arguments_available(ctx) {
            self.diagnostics.push(Diagnostic::error(
                "'arguments' is not allowed in a class field initializer or static block",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if direct_eval_callee(node, ctx.ast) {
            self.mark_direct_eval();
        }
        if ctx.ast.node_kind(node.callee(ctx.ast).node_id()) == NodeKind::Super {
            let message = match self.super_call_validity(ctx) {
                SuperCallValidity::Valid => None,
                SuperCallValidity::NotInConstructor => {
                    Some("'super()' is only valid in a derived class constructor")
                }
                SuperCallValidity::NoExtends => {
                    Some("'super()' cannot be used in a class without an extends clause")
                }
            };
            if let Some(message) = message {
                self.diagnostics
                    .push(Diagnostic::error(message, node.span(ctx.ast)));
            }
        }
        TraverseAction::Proceed
    }

    fn enter_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let property_is_private =
            ctx.ast.node_kind(node.property(ctx.ast).node_id()) == NodeKind::PrivateIdentifier;
        if ctx.ast.node_kind(node.object(ctx.ast).node_id()) == NodeKind::Super {
            // `super` only accepts an IdentifierName or computed expression as
            // its property, never a PrivateIdentifier.
            if property_is_private {
                self.diagnostics.push(Diagnostic::error(
                    "A private name may not be accessed on 'super'",
                    node.span(ctx.ast),
                ));
            } else if !self.is_super_property_valid(ctx) {
                self.diagnostics.push(Diagnostic::error(
                    "A super property is only valid inside a method or class field",
                    node.span(ctx.ast),
                ));
            }
        }
        // AllPrivateIdentifiersValid: walk outward through class bodies until
        // a matching private declaration is found.
        if let PropertyKeyData::PrivateIdentifier(private) =
            ctx.ast.property_key_data(node.property(ctx.ast))
        {
            let name = private.name(ctx.ast);
            let name = ctx.ast.string(name);
            if !is_private_name_declared(ctx, name) {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Owned(format!(
                        "Private name '#{}' is not declared",
                        String::from_utf8_lossy(name)
                    )),
                    span: private.span(ctx.ast),
                    help: None,
                    labels: Vec::new(),
                });
            }
        }
        TraverseAction::Proceed
    }

    fn enter_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if node.operator(ctx.ast) == BinaryOperator::In
            && let ExprData::PrivateIdentifier(private) = ctx.ast.expr_data(node.left(ctx.ast))
        {
            let name = private.name(ctx.ast);
            let name = ctx.ast.string(name);
            if !is_private_name_declared(ctx, name) {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Owned(format!(
                        "Private name '#{}' is not declared",
                        String::from_utf8_lossy(name)
                    )),
                    span: private.span(ctx.ast),
                    help: None,
                    labels: Vec::new(),
                });
            }
        }
        TraverseAction::Proceed
    }

    fn enter_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if node.operator(ctx.ast) == UnaryOperator::Delete
            && expression_is_private_reference(node.argument(ctx.ast), ctx.ast)
        {
            self.diagnostics.push(Diagnostic::error(
                "Private fields may not be deleted",
                node.span(ctx.ast),
            ));
        }
        if node.operator(ctx.ast) == UnaryOperator::Delete
            && self.inherited_flags().strict
            && expression_is_identifier_reference(node.argument(ctx.ast), ctx.ast)
        {
            self.diagnostics.push(Diagnostic::error(
                "Deleting an unqualified identifier is forbidden in strict mode",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_string_literal(
        &mut self,
        node: StringLiteral,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if ctx
            .ancestors()
            .skip(1)
            .any(|ancestor| is_type_node(ctx.node_kind(ancestor)))
        {
            return TraverseAction::Proceed;
        }
        if self.inherited_flags().strict
            && string_has_legacy_escape(ctx.ast.string(node.raw(ctx.ast)))
        {
            self.diagnostics.push(Diagnostic::error(
                "Legacy octal and non-octal decimal escapes are forbidden in strict mode",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_numeric_literal(
        &mut self,
        node: NumericLiteral,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if ctx
            .ancestors()
            .skip(1)
            .any(|ancestor| is_type_node(ctx.node_kind(ancestor)))
        {
            return TraverseAction::Proceed;
        }
        let raw = ctx.ast.string(node.raw(ctx.ast));
        if self.inherited_flags().strict && is_legacy_numeric_literal(raw) {
            self.diagnostics.push(Diagnostic::error(
                "Legacy octal and leading-zero decimal literals are forbidden in strict mode",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let meta = node.meta(ctx.ast).name(ctx.ast);
        let property = node.property(ctx.ast).name(ctx.ast);
        if ctx.ast.string(meta) == b"import"
            && ctx.ast.string(property) == b"meta"
            && self.scopes.get(self.scopes.top_level_scope_id()).kind != ScopeKind::Module
        {
            self.diagnostics.push(Diagnostic::error(
                "'import.meta' is only valid in module code",
                node.span(ctx.ast),
            ));
        }
        if ctx.ast.string(meta) == b"new"
            && ctx.ast.string(property) == b"target"
            && !is_new_target_available(ctx)
        {
            self.diagnostics.push(Diagnostic::error(
                "'new.target' is only valid inside a function or class field initializer",
                node.span(ctx.ast),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.catch_bodies.push(node.body(ctx.ast).node_id());
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::FUNCTION_SCOPED_VAR.union(SymbolFlags::CATCH_VAR),
                excludes: SymbolFlags::catch_param_excludes(),
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_catch_clause(&mut self, node: CatchClause, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
        self.catch_bodies.pop();
        self.exit_scope(node.node_id());
    }

    fn enter_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let kind = node.kind(ctx.ast);
        if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
            let direct_parent = ctx.ancestors().nth(1);
            if direct_parent.is_some_and(|parent| ctx.node_kind(parent) == NodeKind::Program)
                && self.scopes.get(self.scopes.top_level_scope_id()).kind != ScopeKind::Module
            {
                self.diagnostics.push(Diagnostic::error(
                    "A using declaration may not appear at the top level of a script",
                    node.span(ctx.ast),
                ));
            }
        }
        let (flags, excludes, target) = match node.kind(ctx.ast) {
            VariableKind::Var => (
                SymbolFlags::FUNCTION_SCOPED_VAR,
                SymbolFlags::function_var_excludes(),
                self.current_hoist_scope_id(),
            ),
            VariableKind::Let | VariableKind::Using | VariableKind::AwaitUsing => (
                SymbolFlags::BLOCK_SCOPED_VAR,
                SymbolFlags::block_var_excludes(),
                self.current_scope,
            ),
            VariableKind::Const => (
                SymbolFlags::BLOCK_SCOPED_VAR.union(SymbolFlags::CONST_VAR),
                SymbolFlags::block_var_excludes(),
                self.current_scope,
            ),
        };
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags,
                excludes,
                target,
                export_state: ExportState::None,
                ambient: node.declare(ctx.ast) || self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_class(&mut self, node: Class, ctx: &mut TraverseCtx<'_, 'a>) -> TraverseAction {
        let outer_scope = self.current_scope;
        // Classes are always strict.
        let flags = ScopeFlags { strict: true };
        let binding_target = if is_named_class_expression(node, ctx.ast) {
            self.expr_name_scope(node.node_id(), ScopeKind::Class, flags);
            self.scopes
                .get(self.current_scope)
                .parent
                .expect("named class expression has a name scope")
        } else {
            self.push_scope(node.node_id(), ScopeKind::Class, flags);
            outer_scope
        };
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::CLASS,
                excludes: SymbolFlags::class_excludes(),
                target: binding_target,
                export_state: ExportState::None,
                ambient: node.declare(ctx.ast) || self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_class(&mut self, node: Class, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
        self.exit_scope(node.node_id());
    }

    fn enter_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        // Class early errors allow at most one implemented constructor.
        // Private names must be unique except for a matching static or
        // instance getter/setter pair.
        let mut first_constructor = None;
        let mut private_elements: Vec<(AstString, PrivateElementKind, bool, Span)> = Vec::new();
        for id in node.body(ctx.ast).iter() {
            let element = ctx.ast.get_node_in_sub_range(id);
            if let ClassElementData::MethodDefinition(method) = ctx.ast.class_element_data(element)
                && method.kind(ctx.ast) == MethodDefinitionKind::Constructor
                && method.value(ctx.ast).body(ctx.ast).is_some()
            {
                if let Some(previous) = first_constructor {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        message: Cow::Borrowed("A class can only have one constructor"),
                        span: method.key(ctx.ast).span(ctx.ast),
                        help: None,
                        labels: vec![Label {
                            span: previous,
                            message: Cow::Borrowed("first constructor is here"),
                        }],
                    });
                } else {
                    first_constructor = Some(method.span(ctx.ast));
                }
            }
            let Some((name, key_span)) = private_element_name(element, ctx.ast) else {
                continue;
            };
            let kind = private_element_kind(element, ctx.ast);
            let is_static = private_element_is_static(element, ctx.ast);
            if let Some((_, previous_kind, previous_static, previous_span)) =
                private_elements.iter().find(|(previous_name, _, _, _)| {
                    ctx.ast.string(*previous_name) == ctx.ast.string(name)
                })
            {
                let accessor_pair = matches!(
                    (*previous_kind, kind),
                    (PrivateElementKind::Getter, PrivateElementKind::Setter)
                        | (PrivateElementKind::Setter, PrivateElementKind::Getter)
                ) && *previous_static == is_static;
                if !accessor_pair {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        message: Cow::Owned(format!(
                            "Duplicate private name '#{}'",
                            String::from_utf8_lossy(ctx.ast.string(name))
                        )),
                        span: key_span,
                        help: None,
                        labels: vec![Label {
                            span: *previous_span,
                            message: Cow::Borrowed("first declaration is here"),
                        }],
                    });
                }
            } else {
                private_elements.push((name, kind, is_static, key_span));
            }
        }
        TraverseAction::Proceed
    }

    fn enter_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        // A colon-form "__proto__" property mutates the object prototype, so
        // ECMAScript permits at most one per object initializer.
        let mut first_proto = None;
        for id in node.properties(ctx.ast).iter() {
            let property = ctx.ast.get_node_in_sub_range(id);
            let ObjectPropertyKindData::ObjectProperty(property) =
                ctx.ast.object_property_kind_data(property)
            else {
                continue;
            };
            if property.computed(ctx.ast)
                || property.method(ctx.ast)
                || property.shorthand(ctx.ast)
                || property.kind(ctx.ast) != PropertyKind::Init
                || !property_key_name_is(property.key(ctx.ast), "__proto__", ctx.ast)
            {
                continue;
            }
            if let Some(previous) = first_proto {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: Cow::Borrowed("Duplicate '__proto__' property in object literal"),
                    span: property.span(ctx.ast),
                    help: None,
                    labels: vec![Label {
                        span: previous,
                        message: Cow::Borrowed("first property is here"),
                    }],
                });
            } else {
                first_proto = Some(property.span(ctx.ast));
            }
        }
        TraverseAction::Proceed
    }

    fn enter_static_block(
        &mut self,
        node: StaticBlock,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.enter_control_boundary();
        self.push_scope(
            node.node_id(),
            ScopeKind::StaticBlock,
            ScopeFlags { strict: true },
        );
        TraverseAction::Proceed
    }

    fn exit_static_block(&mut self, node: StaticBlock, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_scope(node.node_id());
        self.exit_control_boundary();
    }

    fn enter_import_declaration(
        &mut self,
        node: ImportDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_import_export_position(node, "An import declaration", ctx);
        self.check_duplicate_with_clause_attributes(node.attributes(ctx.ast), ctx.ast);
        let flags = if node.import_kind(ctx.ast) == ImportOrExportKind::Type {
            SymbolFlags::TYPE_IMPORT
        } else {
            SymbolFlags::IMPORT
        };
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags,
                excludes: SymbolFlags::import_binding_excludes(),
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_import_declaration(&mut self, node: ImportDeclaration, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_import_export_position(node, "An import declaration", ctx);
        let flags = if node.import_kind(ctx.ast) == ImportOrExportKind::Type {
            SymbolFlags::TYPE_IMPORT
        } else {
            SymbolFlags::IMPORT
        };
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags,
                excludes: SymbolFlags::import_binding_excludes(),
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_import_specifier(
        &mut self,
        node: ImportSpecifier,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if node.import_kind(ctx.ast) == ImportOrExportKind::Type {
            self.set_binding_context(
                node.node_id(),
                BindingContext {
                    flags: SymbolFlags::TYPE_IMPORT,
                    excludes: SymbolFlags::import_binding_excludes(),
                    target: self.current_scope,
                    export_state: ExportState::None,
                    ambient: self.ambient,
                },
            );
        }
        TraverseAction::Proceed
    }

    fn exit_import_specifier(&mut self, node: ImportSpecifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_import_export_position(node, "An export declaration", ctx);
        self.check_duplicate_with_clause_attributes(node.attributes(ctx.ast), ctx.ast);
        TraverseAction::Proceed
    }

    fn enter_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_import_export_position(node, "An export declaration", ctx);
        TraverseAction::Proceed
    }

    fn enter_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_import_export_position(node, "An export declaration", ctx);
        self.check_duplicate_with_clause_attributes(node.attributes(ctx.ast), ctx.ast);
        TraverseAction::Proceed
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        let target = self
            .scopes
            .get(self.current_scope)
            .parent
            .expect("type alias scope has a parent");
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::TYPE_ALIAS,
                excludes: SymbolFlags::type_alias_excludes(),
                target,
                export_state: ExportState::None,
                ambient: node.declare(ctx.ast) || self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_binding_context(node.node_id());
        self.exit_scope(node.node_id());
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        let target = self
            .scopes
            .get(self.current_scope)
            .parent
            .expect("interface scope has a parent");
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::INTERFACE,
                excludes: SymbolFlags::interface_excludes(),
                target,
                export_state: ExportState::None,
                ambient: node.declare(ctx.ast) || self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_binding_context(node.node_id());
        self.exit_scope(node.node_id());
    }

    fn enter_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let flags = if node.is_const(ctx.ast) {
            SymbolFlags::CONST_ENUM
        } else {
            SymbolFlags::REGULAR_ENUM
        };
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags,
                excludes: if node.is_const(ctx.ast) {
                    SymbolFlags::const_enum_excludes()
                } else {
                    SymbolFlags::regular_enum_excludes()
                },
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: node.declare(ctx.ast) || self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        // A namespace occupies value space when its body contains a
        // value-producing statement. Body-less ambient modules are
        // instantiated by specification.
        let instantiated = is_namespace_instantiated(node.body(ctx.ast), ctx.ast);
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: if instantiated {
                    SymbolFlags::VALUE_MODULE
                } else {
                    SymbolFlags::NAMESPACE_MODULE
                },
                excludes: if instantiated {
                    SymbolFlags::value_module_excludes()
                } else {
                    SymbolFlags::namespace_module_excludes()
                },
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: node.declare(ctx.ast) || self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_ts_type_parameter(
        &mut self,
        node: TsTypeParameter,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::TYPE_PARAMETER,
                excludes: SymbolFlags::type_parameter_excludes(),
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_type_parameter(&mut self, node: TsTypeParameter, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
    }

    fn enter_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.declare_binding(node, ctx);
        TraverseAction::Proceed
    }

    fn enter_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.write_targets.push(Some(node.left(ctx.ast).node_id()));
        TraverseAction::Proceed
    }

    fn exit_assignment_expression(
        &mut self,
        _node: AssignmentExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.write_targets.pop();
    }

    fn enter_update_expression(
        &mut self,
        node: UpdateExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.write_targets
            .push(Some(node.argument(ctx.ast).node_id()));
        TraverseAction::Proceed
    }

    fn exit_update_expression(&mut self, _node: UpdateExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
    }

    fn enter_ts_function_type(
        &mut self,
        node: TsFunctionType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_function_type(&mut self, node: TsFunctionType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_scope(node.node_id());
    }

    fn enter_ts_constructor_type(
        &mut self,
        node: TsConstructorType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_constructor_type(
        &mut self,
        node: TsConstructorType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_scope(node.node_id());
    }

    fn enter_ts_method_signature(
        &mut self,
        node: TsMethodSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_method_signature(
        &mut self,
        node: TsMethodSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_scope(node.node_id());
    }

    fn enter_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_scope(node.node_id());
    }

    fn enter_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_scope(node.node_id());
    }

    fn enter_ts_index_signature(
        &mut self,
        node: TsIndexSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_index_signature(&mut self, node: TsIndexSignature, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_scope(node.node_id());
    }

    fn enter_ts_mapped_type(
        &mut self,
        node: TsMappedType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.set_binding_context(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::TYPE_PARAMETER,
                excludes: SymbolFlags::type_parameter_excludes(),
                target: self.current_scope,
                export_state: ExportState::None,
                ambient: self.ambient,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_mapped_type(&mut self, node: TsMappedType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_binding_context(node.node_id());
        self.exit_scope(node.node_id());
    }

    fn enter_ts_conditional_type(
        &mut self,
        node: TsConditionalType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_conditional_type(
        &mut self,
        node: TsConditionalType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_scope(node.node_id());
    }

    fn enter_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_scope(node.node_id(), ScopeKind::TsModule, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_module_block(&mut self, node: TsModuleBlock, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_scope(node.node_id());
    }
}

struct ReferenceResolver<'semantic> {
    scopes: &'semantic ScopeTree,
    symbols: &'semantic mut SymbolTable,
    node_scopes: &'semantic [ScopeId],
    type_depth: u32,
    write_targets: Vec<Option<NodeId>>,
    diagnostics: &'semantic mut Vec<Diagnostic<'static>>,
}

impl<'semantic> ReferenceResolver<'semantic> {
    fn new(
        scopes: &'semantic ScopeTree,
        symbols: &'semantic mut SymbolTable,
        node_scopes: &'semantic [ScopeId],
        diagnostics: &'semantic mut Vec<Diagnostic<'static>>,
    ) -> Self {
        Self {
            scopes,
            symbols,
            node_scopes,
            type_depth: 0,
            write_targets: Vec::new(),
            diagnostics,
        }
    }

    /// Records and resolves one identifier use after all declarations and
    /// hoisted bindings are known.
    fn check_identifier_reference<'a>(
        &mut self,
        node: IdentifierReference,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let name = node.name(ctx.ast);
        let mut scope = self.node_scopes[node.node_id().index()];
        if ctx
            .ancestors()
            .skip(1)
            .take_while(|ancestor| ctx.node_kind(*ancestor) != NodeKind::Class)
            .any(|ancestor| ctx.node_kind(ancestor) == NodeKind::Decorator)
            && self.scopes.get(scope).kind == ScopeKind::Class
        {
            scope = self
                .scopes
                .get(scope)
                .parent
                .expect("a class scope always has an outer scope");
        }
        let is_write = is_write_position(node.node_id(), ctx, &self.write_targets);
        let is_read = !is_write || is_read_write_position(node.node_id(), ctx);
        let dynamic = ctx
            .ancestors()
            .skip(1)
            .any(|ancestor| ctx.node_kind(ancestor) == NodeKind::WithStatement);
        let kind = if self.type_depth > 0 {
            ReferenceKind::Type
        } else {
            ReferenceKind::Value
        };
        let reference_id = self.symbols.add_reference(
            name,
            scope,
            node.node_id(),
            node.span(ctx.ast),
            kind,
            is_read,
            is_write,
            dynamic,
        );
        let symbol_id =
            self.symbols
                .resolve_reference(self.scopes, reference_id, ctx.ast.string(name));
        node.set_symbol_id(ctx.ast, symbol_id);
        let strict = self.scopes.get(scope).flags.strict;
        if self.type_depth == 0 && strict {
            check_strict_reserved(
                self.diagnostics,
                ctx.ast.string(name),
                node.span(ctx.ast),
                "an identifier reference",
            );
        }
        if is_write && strict {
            check_assign_target_eval_arguments(self.diagnostics, node, ctx.ast);
        }
    }
}

impl<'a> Traverse<'a> for ReferenceResolver<'_> {
    fn enter_node(
        &mut self,
        _node_id: NodeId,
        kind: NodeKind,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if is_type_node(kind) {
            self.type_depth += 1;
        }
        TraverseAction::Proceed
    }

    fn exit_node(&mut self, _node_id: NodeId, kind: NodeKind, _ctx: &mut TraverseCtx<'_, 'a>) {
        if is_type_node(kind) {
            self.type_depth -= 1;
        }
    }

    fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_identifier_reference(node, ctx);
        TraverseAction::Proceed
    }

    fn enter_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if !is_write_position(node.node_id(), ctx, &self.write_targets) {
            return TraverseAction::Proceed;
        }
        let name = node.name(ctx.ast);
        let scope = self.node_scopes[node.node_id().index()];
        let reference_id = self.symbols.add_reference(
            name,
            scope,
            node.node_id(),
            node.span(ctx.ast),
            ReferenceKind::Value,
            false,
            true,
            false,
        );
        let symbol_id =
            self.symbols
                .resolve_reference(self.scopes, reference_id, ctx.ast.string(name));
        node.set_symbol_id(ctx.ast, symbol_id);
        let name_bytes = ctx.ast.string(name);
        if self.scopes.get(scope).flags.strict && is_eval_or_arguments(name_bytes) {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Owned(format!(
                    "Cannot assign to '{}' in strict mode",
                    String::from_utf8_lossy(name_bytes)
                )),
                span: node.span(ctx.ast),
                help: None,
                labels: Vec::new(),
            });
        }
        TraverseAction::Proceed
    }

    fn enter_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.write_targets.push(Some(node.left(ctx.ast).node_id()));
        TraverseAction::Proceed
    }

    fn exit_assignment_expression(
        &mut self,
        _node: AssignmentExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.write_targets.pop();
    }

    fn enter_update_expression(
        &mut self,
        node: UpdateExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.write_targets
            .push(Some(node.argument(ctx.ast).node_id()));
        TraverseAction::Proceed
    }

    fn exit_update_expression(&mut self, _node: UpdateExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
    }

    fn enter_for_in_statement(
        &mut self,
        node: ForInStatement,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.write_targets
            .push(for_left_write_target(node.left(ctx.ast), ctx.ast));
        TraverseAction::Proceed
    }

    fn exit_for_in_statement(&mut self, _node: ForInStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
    }

    fn enter_for_of_statement(
        &mut self,
        node: ForOfStatement,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.write_targets
            .push(for_left_write_target(node.left(ctx.ast), ctx.ast));
        TraverseAction::Proceed
    }

    fn exit_for_of_statement(&mut self, _node: ForOfStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
    }
}

pub(crate) fn analyze_program<'a>(program: Program, ast: &mut Ast<'a>) -> AnalysisParts {
    // First build scopes and declarations. References are resolved in a second
    // traversal after every hoisted and merged binding is known.
    let mut builder = SemanticBuilder::new(program, ast);
    let mut traverse_ctx = TraverseCtx::new(ast, ());
    let _ = program.traverse_with(&mut builder, &mut traverse_ctx);

    let SemanticBuilder {
        scopes,
        mut symbols,
        node_scopes,
        mut diagnostics,
        ..
    } = builder;
    let mut resolver =
        ReferenceResolver::new(&scopes, &mut symbols, &node_scopes, &mut diagnostics);
    let mut traverse_ctx = TraverseCtx::new(ast, ());
    let _ = program.traverse_with(&mut resolver, &mut traverse_ctx);

    AnalysisParts {
        scopes,
        symbols,
        node_scopes,
        diagnostics,
    }
}

fn has_use_strict_directive(body: TypedSubRange<StatementListItem>, ast: &Ast<'_>) -> bool {
    for index in 0..body.len() {
        let item = body
            .get_node(ast, index)
            .expect("typed statement-list ranges contain valid nodes");
        match ast.statement_list_item_data(item) {
            StatementListItemData::Directive(directive) => {
                if ast.string(directive.value(ast)) == b"use strict" {
                    return true;
                }
            }
            StatementListItemData::Stmt(_) => break,
        }
    }
    false
}

fn is_named_function_expression(function: Function, ast: &Ast<'_>) -> bool {
    matches!(
        function.function_type(ast),
        FunctionType::FunctionExpression | FunctionType::TsEmptyBodyFunctionExpression
    ) && function.id(ast).is_some()
}

fn is_named_class_expression(class: Class, ast: &Ast<'_>) -> bool {
    class.class_type(ast) == ClassType::ClassExpression && class.id(ast).is_some()
}

fn is_namespace_instantiated(body: Option<TsModuleBody>, ast: &Ast<'_>) -> bool {
    let Some(body) = body else {
        return true;
    };
    match ast.ts_module_body_data(body) {
        TsModuleBodyData::TsModuleDeclaration(declaration) => {
            is_namespace_instantiated(declaration.body(ast), ast)
        }
        TsModuleBodyData::TsModuleBlock(block) => block
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .any(|item| is_instantiating_statement(item, ast)),
    }
}

fn is_instantiating_statement(item: StatementListItem, ast: &Ast<'_>) -> bool {
    let StatementListItemData::Stmt(statement) = ast.statement_list_item_data(item) else {
        return true;
    };
    match ast.stmt_data(statement) {
        StmtData::TsImportEqualsDeclaration(_) => false,
        StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
            DeclData::TsInterfaceDeclaration(_) | DeclData::TsTypeAliasDeclaration(_) => false,
            DeclData::TsEnumDeclaration(declaration) => !declaration.is_const(ast),
            DeclData::TsModuleDeclaration(declaration) => {
                is_namespace_instantiated(declaration.body(ast), ast)
            }
            _ => true,
        },
        StmtData::ExportNamedDeclaration(declaration) => {
            if declaration.export_kind(ast) == ImportOrExportKind::Type {
                false
            } else if let Some(declaration) = declaration.declaration(ast) {
                let statement = Stmt::Declaration(declaration);
                let item = StatementListItem::Stmt(statement);
                is_instantiating_statement(item, ast)
            } else {
                declaration.source(ast).is_some()
            }
        }
        _ => true,
    }
}

fn formal_parameters_are_non_simple(params: FormalParameters, ast: &Ast<'_>) -> bool {
    if params.rest(ast).is_some() {
        return true;
    }
    params.items(ast).iter().any(|id| {
        let item = ast.get_node_in_sub_range(id);
        let pattern = match ast.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => {
                match ast.formal_parameter_pattern_data(parameter.pattern(ast)) {
                    FormalParameterPatternData::BindingPattern(pattern) => Some(pattern),
                    FormalParameterPatternData::TsThisParameter(_) => None,
                }
            }
            FormalParameterItemData::TsParameterProperty(property) => Some(property.parameter(ast)),
        };
        pattern
            .is_some_and(|pattern| ast.node_kind(pattern.node_id()) != NodeKind::BindingIdentifier)
    })
}

fn property_key_name_is(key: PropertyKey, expected: &str, ast: &Ast<'_>) -> bool {
    match ast.property_key_data(key) {
        PropertyKeyData::IdentifierName(identifier) => {
            ast.string(identifier.name(ast)) == expected.as_bytes()
        }
        PropertyKeyData::StringLiteral(string) => {
            ast.string(string.value(ast)) == expected.as_bytes()
        }
        _ => false,
    }
}

fn get_module_export_name(name: ModuleExportName, ast: &Ast<'_>) -> Option<String> {
    match ast.module_export_name_data(name) {
        ModuleExportNameData::IdentifierName(identifier) => {
            Some(String::from_utf8_lossy(ast.string(identifier.name(ast))).into_owned())
        }
        ModuleExportNameData::StringLiteral(string) => {
            Some(String::from_utf8_lossy(ast.string(string.value(ast))).into_owned())
        }
    }
}

fn is_in_formal_parameters(ctx: &TraverseCtx<'_, '_>) -> bool {
    for ancestor in ctx.ancestors().skip(1) {
        let kind = ctx.node_kind(ancestor);
        match kind {
            NodeKind::FormalParameters => return true,
            NodeKind::Program => {
                return false;
            }
            _ => {}
        }
        if is_function_boundary(kind) {
            return false;
        }
    }
    false
}

fn is_iteration(statement: Stmt, ast: &Ast<'_>) -> bool {
    matches!(
        ast.node_kind(statement.node_id()),
        NodeKind::DoWhileStatement
            | NodeKind::ForStatement
            | NodeKind::ForInStatement
            | NodeKind::ForOfStatement
            | NodeKind::WhileStatement
    )
}

fn is_function_boundary(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Function | NodeKind::ArrowFunctionExpression | NodeKind::StaticBlock
    )
}

fn is_at_program_level(ctx: &TraverseCtx<'_, '_>) -> bool {
    ctx.parent().is_some_and(|parent| {
        matches!(
            ctx.node_kind(parent),
            NodeKind::Program | NodeKind::TsModuleBlock
        )
    })
}

fn is_in_ts_namespace(ctx: &TraverseCtx<'_, '_>) -> bool {
    ctx.ancestors()
        .skip(1)
        .any(|ancestor| ctx.node_kind(ancestor) == NodeKind::TsModuleBlock)
}

fn is_arguments_available(ctx: &TraverseCtx<'_, '_>) -> bool {
    // `arguments` is introduced by regular functions, while arrow functions
    // are transparent. Class fields and static blocks form forbidden
    // boundaries.
    for ancestor in ctx.ancestors().skip(1) {
        match ctx.node_kind(ancestor) {
            NodeKind::Function => return true,
            NodeKind::PropertyDefinition | NodeKind::StaticBlock => return false,
            NodeKind::ArrowFunctionExpression | NodeKind::Program => {}
            _ => {}
        }
    }
    true
}

fn is_private_name_declared(ctx: &TraverseCtx<'_, '_>, name: &[u8]) -> bool {
    // ECMAScript AllPrivateIdentifiersValid checks ancestor ClassBody nodes
    // and accepts a name declared by any enclosing class.
    for ancestor in ctx.ancestors().skip(1) {
        let NodeKindData::ClassBody(body) = ctx.ast.kind_data(ancestor) else {
            continue;
        };
        for id in body.body(ctx.ast).iter() {
            let element = ctx.ast.get_node_in_sub_range(id);
            let key = match ctx.ast.class_element_data(element) {
                ClassElementData::MethodDefinition(method) => method.key(ctx.ast),
                ClassElementData::PropertyDefinition(property) => property.key(ctx.ast),
                ClassElementData::StaticBlock(_) | ClassElementData::TsIndexSignature(_) => {
                    continue;
                }
            };
            if let PropertyKeyData::PrivateIdentifier(private) = ctx.ast.property_key_data(key)
                && ctx.ast.string(private.name(ctx.ast)) == name
            {
                return true;
            }
        }
    }
    false
}

fn private_element_name(element: ClassElement, ast: &Ast<'_>) -> Option<(AstString, Span)> {
    let key = match ast.class_element_data(element) {
        ClassElementData::MethodDefinition(method) => method.key(ast),
        ClassElementData::PropertyDefinition(property) => property.key(ast),
        ClassElementData::StaticBlock(_) | ClassElementData::TsIndexSignature(_) => return None,
    };
    let PropertyKeyData::PrivateIdentifier(private) = ast.property_key_data(key) else {
        return None;
    };
    Some((private.name(ast), key.span(ast)))
}

fn private_element_kind(element: ClassElement, ast: &Ast<'_>) -> PrivateElementKind {
    match ast.class_element_data(element) {
        ClassElementData::MethodDefinition(method) => match method.kind(ast) {
            MethodDefinitionKind::Get => PrivateElementKind::Getter,
            MethodDefinitionKind::Set => PrivateElementKind::Setter,
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {
                PrivateElementKind::Method
            }
        },
        ClassElementData::PropertyDefinition(_) => PrivateElementKind::Field,
        ClassElementData::StaticBlock(_) | ClassElementData::TsIndexSignature(_) => {
            unreachable!("static blocks and index signatures do not have private names")
        }
    }
}

fn private_element_is_static(element: ClassElement, ast: &Ast<'_>) -> bool {
    match ast.class_element_data(element) {
        ClassElementData::MethodDefinition(method) => method.r#static(ast),
        ClassElementData::PropertyDefinition(property) => property.r#static(ast),
        ClassElementData::StaticBlock(_) | ClassElementData::TsIndexSignature(_) => {
            unreachable!("static blocks and index signatures do not have private names")
        }
    }
}

fn expression_is_private_reference(expression: Expr, ast: &Ast<'_>) -> bool {
    match ast.expr_data(unwrap_parens(expression, ast)) {
        ExprData::MemberExpression(member) => {
            ast.node_kind(member.property(ast).node_id()) == NodeKind::PrivateIdentifier
        }
        ExprData::CallExpression(call) => expression_is_private_reference(call.callee(ast), ast),
        ExprData::ChainExpression(chain) => {
            expression_is_private_reference(chain.expression(ast), ast)
        }
        _ => false,
    }
}

fn expression_is_identifier_reference(expression: Expr, ast: &Ast<'_>) -> bool {
    ast.node_kind(unwrap_parens(expression, ast).node_id()) == NodeKind::IdentifierReference
}

fn unwrap_parens(mut expression: Expr, ast: &Ast<'_>) -> Expr {
    // The parser creates at most one wrapper per syntactic nesting level, so
    // this walk is bounded by source nesting depth.
    let mut depth = 0u32;
    loop {
        assert!(depth < 1_000_000);
        depth += 1;
        match ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(parenthesized) => {
                expression = parenthesized.expression(ast);
            }
            _ => return expression,
        }
    }
}

fn string_has_legacy_escape(bytes: &[u8]) -> bool {
    let mut index = 1;
    while index + 1 < bytes.len() {
        if bytes[index] != b'\\' {
            index += 1;
            continue;
        }
        index += 1;
        match bytes[index] {
            b'1'..=b'9' => return true,
            b'0' if index + 1 < bytes.len() - 1 && bytes[index + 1].is_ascii_digit() => {
                return true;
            }
            b'\r' if index + 1 < bytes.len() - 1 && bytes[index + 1] == b'\n' => index += 1,
            _ => {}
        }
        index += 1;
    }
    false
}

fn check_strict_reserved(
    diagnostics: &mut Vec<Diagnostic<'static>>,
    name: &[u8],
    span: Span,
    as_what: &'static str,
) {
    let Some(word) = match_strict_reserved(name) else {
        return;
    };
    diagnostics.push(Diagnostic {
        severity: Severity::Error,
        message: Cow::Owned(format!(
            "'{word}' is reserved in strict mode and cannot be used as {as_what}"
        )),
        span,
        help: None,
        labels: Vec::new(),
    });
}

fn match_strict_reserved(name: &[u8]) -> Option<&'static str> {
    match name.len() {
        3 => check(name, b"let", "let"),
        5 => check(name, b"yield", "yield"),
        6 => check(name, b"static", "static").or_else(|| check(name, b"public", "public")),
        7 => check(name, b"private", "private").or_else(|| check(name, b"package", "package")),
        9 => check(name, b"protected", "protected")
            .or_else(|| check(name, b"interface", "interface")),
        10 => check(name, b"implements", "implements"),
        _ => None,
    }
}

fn check(name: &[u8], keyword: &[u8], word: &'static str) -> Option<&'static str> {
    (name == keyword).then_some(word)
}

fn is_eval_or_arguments(name: &[u8]) -> bool {
    name == b"eval" || name == b"arguments"
}

fn is_eval_or_arguments_ref(node: IdentifierReference, ast: &Ast<'_>) -> bool {
    is_eval_or_arguments(ast.string(node.name(ast)))
}

fn check_assign_target_eval_arguments(
    diagnostics: &mut Vec<Diagnostic<'static>>,
    node: IdentifierReference,
    ast: &Ast<'_>,
) {
    if is_eval_or_arguments_ref(node, ast) {
        diagnostics.push(Diagnostic::error(
            "Cannot assign to 'eval' or 'arguments' in strict mode",
            node.span(ast),
        ));
    }
}

fn is_legacy_numeric_literal(raw: &[u8]) -> bool {
    // A leading zero followed by another digit is either legacy octal (`077`)
    // or a leading-zero decimal literal (`089`).
    raw.len() > 1 && raw[0] == b'0' && raw[1].is_ascii_digit()
}

fn is_new_target_available(ctx: &TraverseCtx<'_, '_>) -> bool {
    ctx.ancestors().skip(1).any(|ancestor| {
        matches!(
            ctx.node_kind(ancestor),
            NodeKind::Function | NodeKind::PropertyDefinition | NodeKind::StaticBlock
        )
    })
}

fn direct_eval_callee(call: CallExpression, ast: &Ast<'_>) -> bool {
    matches!(
        ast.expr_data(unwrap_parens(call.callee(ast), ast)),
        ExprData::IdentifierReference(identifier)
            if ast.string(identifier.name(ast)) == b"eval"
    )
}

fn is_write_position(node: NodeId, ctx: &TraverseCtx<'_, '_>, targets: &[Option<NodeId>]) -> bool {
    if targets.contains(&Some(node)) {
        return true;
    }
    for ancestor in ctx.ancestors().skip(1) {
        if ctx.node_kind(ancestor) == NodeKind::MemberExpression {
            return false;
        }
        if targets.contains(&Some(ancestor)) {
            return true;
        }
    }
    false
}

fn is_read_write_position(node: NodeId, ctx: &TraverseCtx<'_, '_>) -> bool {
    for ancestor in ctx.ancestors().skip(1) {
        match ctx.ast.kind_data(ancestor) {
            NodeKindData::UpdateExpression(_) => return true,
            NodeKindData::AssignmentExpression(assignment) => {
                return assignment.operator(ctx.ast) != AssignmentOperator::Assign;
            }
            NodeKindData::MemberExpression(_)
            | NodeKindData::Function(_)
            | NodeKindData::ArrowFunctionExpression(_)
            | NodeKindData::Program(_) => return false,
            _ if ancestor == node => {}
            _ => {}
        }
    }
    false
}

fn for_left_write_target(left: ForStatementLeft, ast: &Ast<'_>) -> Option<NodeId> {
    match ast.for_statement_left_data(left) {
        ForStatementLeftData::AssignmentTarget(target) => Some(target.node_id()),
        ForStatementLeftData::VariableDeclaration(_) => None,
    }
}

fn scope_function_declaration_is_ordinary(
    scope_node: NodeId,
    declaration: NodeId,
    ast: &Ast<'_>,
) -> bool {
    let statement_is_match = |statement: Stmt| {
        let StmtData::Declaration(declaration_node) = ast.stmt_data(statement) else {
            return None;
        };
        let DeclData::Function(function) = ast.decl_data(declaration_node) else {
            return None;
        };
        (function
            .id(ast)
            .is_some_and(|id| id.node_id() == declaration))
        .then(|| {
            function.function_type(ast) == FunctionType::FunctionDeclaration
                && !function.r#async(ast)
                && !function.generator(ast)
        })
    };

    match ast.kind_data(scope_node) {
        NodeKindData::BlockStatement(block) => block
            .body(ast)
            .iter()
            .find_map(|id| statement_is_match(ast.get_node_in_sub_range(id)))
            .unwrap_or(false),
        NodeKindData::SwitchStatement(switch) => switch.cases(ast).iter().any(|case_id| {
            let case = ast.get_node_in_sub_range(case_id);
            case.consequent(ast)
                .iter()
                .find_map(|id| statement_is_match(ast.get_node_in_sub_range(id)))
                .is_some_and(|ordinary| ordinary)
        }),
        _ => false,
    }
}

fn is_type_node(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::TsAnyKeyword
            | NodeKind::TsUnknownKeyword
            | NodeKind::TsNeverKeyword
            | NodeKind::TsVoidKeyword
            | NodeKind::TsNullKeyword
            | NodeKind::TsUndefinedKeyword
            | NodeKind::TsStringKeyword
            | NodeKind::TsNumberKeyword
            | NodeKind::TsBigIntKeyword
            | NodeKind::TsBooleanKeyword
            | NodeKind::TsSymbolKeyword
            | NodeKind::TsObjectKeyword
            | NodeKind::TsIntrinsicKeyword
            | NodeKind::TsThisType
            | NodeKind::TsTypeAnnotation
            | NodeKind::TsTypeReference
            | NodeKind::TsQualifiedName
            | NodeKind::TsTypeQuery
            | NodeKind::TsImportType
            | NodeKind::TsTypeParameter
            | NodeKind::TsTypeParameterDeclaration
            | NodeKind::TsTypeParameterInstantiation
            | NodeKind::TsLiteralType
            | NodeKind::TsTemplateLiteralType
            | NodeKind::TsArrayType
            | NodeKind::TsIndexedAccessType
            | NodeKind::TsTupleType
            | NodeKind::TsNamedTupleMember
            | NodeKind::TsOptionalType
            | NodeKind::TsRestType
            | NodeKind::TsJsDocNullableType
            | NodeKind::TsJsDocNonNullableType
            | NodeKind::TsJsDocUnknownType
            | NodeKind::TsUnionType
            | NodeKind::TsIntersectionType
            | NodeKind::TsConditionalType
            | NodeKind::TsInferType
            | NodeKind::TsTypeOperator
            | NodeKind::TsParenthesizedType
            | NodeKind::TsFunctionType
            | NodeKind::TsConstructorType
            | NodeKind::TsTypePredicate
            | NodeKind::TsTypeLiteral
            | NodeKind::TsPropertySignature
            | NodeKind::TsMethodSignature
            | NodeKind::TsCallSignatureDeclaration
            | NodeKind::TsConstructSignatureDeclaration
            | NodeKind::TsIndexSignature
            | NodeKind::TsMappedType
            | NodeKind::TsInterfaceHeritage
            | NodeKind::TsClassImplements
            | NodeKind::TsInterfaceBody
    )
}
