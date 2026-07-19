use swc_experimental_ecma_ast::*;

use crate::{
    diagnostic::Diagnostic,
    scope::{ScopeFlags, ScopeKind, ScopeTree},
    symbol::{ReferenceKind, SymbolFlags, SymbolTable},
};

pub(crate) struct AnalysisParts<'a> {
    pub scopes: ScopeTree,
    pub symbols: SymbolTable<'a>,
    pub node_scopes: Vec<ScopeId>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
struct BindingContext {
    flags: SymbolFlags,
    excludes: SymbolFlags,
    target: ScopeId,
}

struct SemanticBuilder<'a> {
    scopes: ScopeTree,
    symbols: SymbolTable<'a>,
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
    labels: Vec<(Atom<'a>, bool, Span)>,
    is_module: bool,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> SemanticBuilder<'a> {
    fn new(program: Program, ast: &Ast<'a>) -> Self {
        let scopes = ScopeTree::new(
            program.node_id(),
            program.source_type(ast) == SourceType::Module,
            body_has_use_strict(program.body(ast), ast),
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
            is_module: program.source_type(ast) == SourceType::Module,
            diagnostics: Vec::new(),
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

    fn open_scope(&mut self, node: NodeId, kind: ScopeKind, flags: ScopeFlags) {
        self.enter_scope(node, kind, flags, 1);
    }

    fn open_expression_scope(&mut self, node: NodeId, body_kind: ScopeKind, flags: ScopeFlags) {
        self.open_scope(node, ScopeKind::ExpressionName, flags);
        self.enter_scope(node, body_kind, flags, 2);
    }

    fn close_scope(&mut self, node: NodeId) {
        let Some(&(frame_node, count)) = self.scope_frames.last() else {
            return;
        };
        if frame_node != node {
            return;
        }
        self.scope_frames.pop();
        for _ in 0..count {
            self.current_scope = self
                .scopes
                .get(self.current_scope)
                .parent
                .expect("a semantic child scope always has a parent");
        }
    }

    fn inherited_flags(&self) -> ScopeFlags {
        ScopeFlags {
            strict: self.scopes.get(self.current_scope).flags.strict,
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

    fn visible_labels(&self) -> &[(Atom<'a>, bool, Span)] {
        let start = self
            .control_frames
            .last()
            .map_or(0, |(_, _, labels)| *labels);
        &self.labels[start..]
    }

    fn check_module_declaration_position(
        &mut self,
        node: impl AstNode + GetSpan,
        description: &str,
        ctx: &TraverseCtx<'_, 'a>,
    ) {
        let valid_parent = ctx.parent().is_some_and(|parent| {
            matches!(
                ctx.node_kind(parent),
                NodeKind::Program | NodeKind::TsModuleBlock
            )
        });
        if !valid_parent {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                format!("{description} may only appear at the top level"),
            ));
        }
    }

    fn push_binding(&mut self, node: NodeId, context: BindingContext) {
        self.binding_frames.push(node);
        self.binding_contexts.push(context);
    }

    fn pop_binding(&mut self, node: NodeId) {
        if self.binding_frames.last().copied() == Some(node) {
            self.binding_frames.pop();
            self.binding_contexts.pop();
        }
    }

    fn declare_binding(&mut self, node: BindingIdentifier, ctx: &mut TraverseCtx<'_, 'a>) {
        if is_write_position(ctx, &self.write_targets) {
            return;
        }
        let Some(mut binding) = self.binding_contexts.last().copied() else {
            return;
        };
        if ctx
            .ancestors()
            .skip(1)
            .any(|ancestor| is_type_node(ctx.node_kind(ancestor)))
            && !binding.flags.intersects(SymbolFlags::TYPE_PARAMETER)
        {
            return;
        }
        let export_kind =
            ctx.ancestors()
                .skip(1)
                .find_map(|ancestor| match ctx.node_kind(ancestor) {
                    NodeKind::FunctionBody | NodeKind::ClassBody | NodeKind::TsModuleBlock => {
                        Some(None)
                    }
                    NodeKind::ExportNamedDeclaration => Some(Some(false)),
                    NodeKind::ExportDefaultDeclaration => Some(Some(true)),
                    _ => None,
                });
        if let Some(Some(is_default)) = export_kind {
            binding.flags = binding.flags.union(SymbolFlags::EXPORTED);
            if is_default {
                binding.flags = binding.flags.union(SymbolFlags::DEFAULT_EXPORT);
            }
        }

        let name = node.name(ctx.ast());
        let result = self.symbols.declare(
            name,
            binding.flags,
            binding.excludes,
            binding.target,
            node.node_id(),
            node.span(ctx.ast()),
        );
        if binding.flags.is_hoisting_var() {
            for scope in self.scopes.ancestors(self.current_scope) {
                if scope == binding.target {
                    break;
                }
                if let Some(existing) = self.symbols.find_in_scope(scope, name.as_str()) {
                    let symbol = self.symbols.symbol(existing);
                    if symbol.flags.is_block_scoped_like() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                node.span(ctx.ast()),
                                format!(
                                    "Variable '{}' conflicts with a lexical declaration",
                                    name.as_str()
                                ),
                            )
                            .with_label(symbol.declaration_spans[0], "lexical declaration is here"),
                        );
                    }
                }
            }
            self.symbols.register_hoist(
                &self.scopes,
                self.current_scope,
                binding.target,
                name,
                result.symbol_id,
            );
        }
        node.set_symbol_id(ctx.ast_mut(), Some(result.symbol_id));

        if let Some(previous) = result.conflict {
            self.diagnostics.push(
                Diagnostic::error(
                    node.span(ctx.ast()),
                    format!("Identifier '{}' has already been declared", name.as_str()),
                )
                .with_label(previous, "previous declaration is here"),
            );
        }

        let strict = self.scopes.get(self.current_scope).flags.strict;
        if strict && matches!(name.as_str(), "eval" | "arguments") {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                format!(
                    "'{}' is not allowed as a binding identifier in strict mode",
                    name.as_str()
                ),
            ));
        }
        if strict && is_strict_reserved(name.as_str()) {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                format!(
                    "'{}' is reserved in strict mode and cannot be used as a binding identifier",
                    name.as_str()
                ),
            ));
        }
    }

    fn function_is_strict(&self, function: Function, ast: &Ast<'a>) -> bool {
        self.inherited_flags().strict
            || function
                .body(ast)
                .is_some_and(|body| body_has_use_strict(body.body(ast), ast))
    }

    fn arrow_is_strict(&self, arrow: ArrowFunctionExpression, ast: &Ast<'a>) -> bool {
        self.inherited_flags().strict
            || match ast.arrow_function_body_data(arrow.body(ast)) {
                ArrowFunctionBodyData::FunctionBody(body) => {
                    body_has_use_strict(body.body(ast), ast)
                }
                ArrowFunctionBodyData::Expr(_) => false,
            }
    }
}

impl<'a> Traverse<'a> for SemanticBuilder<'a> {
    fn enter_node(
        &mut self,
        node_id: NodeId,
        _kind: NodeKind,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.node_scopes[node_id.index()] = self.current_scope;
        TraverseAction::Proceed
    }

    fn enter_function(&mut self, node: Function, ctx: &mut TraverseCtx<'_, 'a>) -> TraverseAction {
        self.enter_control_boundary();
        let outer_scope = self.current_scope;
        let strict = self.function_is_strict(node, ctx.ast());
        let flags = ScopeFlags { strict };
        let function_type = node.function_type(ctx.ast());
        let binding_target =
            if function_type == FunctionType::FunctionExpression && node.id(ctx.ast()).is_some() {
                self.open_expression_scope(node.node_id(), ScopeKind::Function, flags);
                self.scopes
                    .get(self.current_scope)
                    .parent
                    .expect("named function expression has a name scope")
            } else {
                self.open_scope(node.node_id(), ScopeKind::Function, flags);
                outer_scope
            };
        let excludes = SymbolFlags::VALUE_SPACE
            .without(SymbolFlags::FUNCTION_SCOPED_VAR)
            .without(SymbolFlags::FUNCTION);
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::FUNCTION,
                excludes,
                target: binding_target,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_function(&mut self, node: Function, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
        self.close_scope(node.node_id());
        self.exit_control_boundary();
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.enter_control_boundary();
        let strict = self.arrow_is_strict(node, ctx.ast());
        self.open_scope(node.node_id(), ScopeKind::Function, ScopeFlags { strict });
        TraverseAction::Proceed
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.close_scope(node.node_id());
        self.exit_control_boundary();
    }

    fn enter_formal_parameters(
        &mut self,
        node: FormalParameters,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let excludes = if self.scopes.get(self.current_scope).flags.strict {
            SymbolFlags::VALUE_SPACE
        } else {
            SymbolFlags::VALUE_SPACE.without(SymbolFlags::FUNCTION_SCOPED_VAR)
        };
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::FUNCTION_SCOPED_VAR.union(SymbolFlags::PARAMETER),
                excludes,
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_formal_parameters(&mut self, node: FormalParameters, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
    }

    fn enter_block_statement(
        &mut self,
        node: BlockStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if self.catch_bodies.last().copied() != Some(node.node_id()) {
            self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        }
        TraverseAction::Proceed
    }

    fn exit_block_statement(&mut self, node: BlockStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.close_scope(node.node_id());
    }

    fn enter_for_statement(
        &mut self,
        node: ForStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        TraverseAction::Proceed
    }

    fn exit_for_statement(&mut self, node: ForStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
        self.close_scope(node.node_id());
    }

    fn enter_for_in_statement(
        &mut self,
        node: ForInStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        self.write_targets
            .push(for_left_write_target(node.left(_ctx.ast()), _ctx.ast()));
        TraverseAction::Proceed
    }

    fn exit_for_in_statement(&mut self, node: ForInStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
        self.close_scope(node.node_id());
    }

    fn enter_for_of_statement(
        &mut self,
        node: ForOfStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        self.iteration_depth += 1;
        self.write_targets
            .push(for_left_write_target(node.left(_ctx.ast()), _ctx.ast()));
        TraverseAction::Proceed
    }

    fn exit_for_of_statement(&mut self, node: ForOfStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
        self.breakable_depth -= 1;
        self.iteration_depth -= 1;
        self.close_scope(node.node_id());
    }

    fn enter_switch_statement(
        &mut self,
        node: SwitchStatement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.breakable_depth += 1;
        let mut default_span = None;
        for case_id in node.cases(_ctx.ast()).iter() {
            let case = _ctx.ast().get_node_in_sub_range(case_id);
            if case.test(_ctx.ast()).is_some() {
                continue;
            }
            if let Some(previous) = default_span {
                self.diagnostics.push(
                    Diagnostic::error(
                        case.span(_ctx.ast()),
                        "A switch statement can only have one default clause",
                    )
                    .with_label(previous, "previous default clause is here"),
                );
            } else {
                default_span = Some(case.span(_ctx.ast()));
            }
        }
        TraverseAction::Proceed
    }

    fn exit_switch_statement(&mut self, node: SwitchStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.breakable_depth -= 1;
        self.close_scope(node.node_id());
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
        let label = node.label(ctx.ast());
        let name = label.name(ctx.ast());
        if let Some((_, _, previous)) = self
            .visible_labels()
            .iter()
            .rev()
            .find(|(existing, _, _)| *existing == name)
        {
            self.diagnostics.push(
                Diagnostic::error(
                    label.span(ctx.ast()),
                    format!("Label '{}' has already been declared", name.as_str()),
                )
                .with_label(*previous, "previous label is here"),
            );
        }
        let iteration = matches!(
            ctx.ast().stmt_data(node.body(ctx.ast())),
            StmtData::DoWhileStatement(_)
                | StmtData::ForStatement(_)
                | StmtData::ForInStatement(_)
                | StmtData::ForOfStatement(_)
                | StmtData::WhileStatement(_)
        );
        self.labels.push((name, iteration, label.span(ctx.ast())));
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
        if let Some(label) = node.label(ctx.ast()) {
            let name = label.name(ctx.ast());
            if !self
                .visible_labels()
                .iter()
                .any(|(candidate, _, _)| *candidate == name)
            {
                self.diagnostics.push(Diagnostic::error(
                    node.span(ctx.ast()),
                    format!("Undefined label '{}'", name.as_str()),
                ));
            }
        } else if self.breakable_depth == 0 {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                "A 'break' statement can only be used within a loop or switch",
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_continue_statement(
        &mut self,
        node: ContinueStatement,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if let Some(label) = node.label(ctx.ast()) {
            let name = label.name(ctx.ast());
            match self
                .visible_labels()
                .iter()
                .rev()
                .find(|(candidate, _, _)| *candidate == name)
            {
                Some((_, true, _)) => {}
                Some(_) => self.diagnostics.push(Diagnostic::error(
                    node.span(ctx.ast()),
                    format!(
                        "Label '{}' does not denote an iteration statement",
                        name.as_str()
                    ),
                )),
                None => self.diagnostics.push(Diagnostic::error(
                    node.span(ctx.ast()),
                    format!("Undefined label '{}'", name.as_str()),
                )),
            }
        } else if self.iteration_depth == 0 {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                "A 'continue' statement can only be used within a loop",
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
                node.span(ctx.ast()),
                "'with' statements are not allowed in strict mode",
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let meta = node.meta(ctx.ast()).name(ctx.ast());
        let property = node.property(ctx.ast()).name(ctx.ast());
        if meta == "import" && property == "meta" && !self.is_module {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                "'import.meta' is only valid in module code",
            ));
        }
        if meta == "new"
            && property == "target"
            && !ctx.ancestors().skip(1).any(|ancestor| {
                matches!(
                    ctx.node_kind(ancestor),
                    NodeKind::Function
                        | NodeKind::ArrowFunctionExpression
                        | NodeKind::PropertyDefinition
                        | NodeKind::StaticBlock
                )
            })
        {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                "'new.target' is only valid inside a function or class field initializer",
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.catch_bodies.push(node.body(ctx.ast()).node_id());
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::FUNCTION_SCOPED_VAR.union(SymbolFlags::CATCH_VAR),
                excludes: SymbolFlags::VALUE_SPACE,
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_catch_clause(&mut self, node: CatchClause, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
        self.catch_bodies.pop();
        self.close_scope(node.node_id());
    }

    fn enter_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let (flags, excludes, target) = match node.kind(ctx.ast()) {
            VariableKind::Var => (
                SymbolFlags::FUNCTION_SCOPED_VAR,
                SymbolFlags::VALUE_SPACE
                    .without(SymbolFlags::FUNCTION_SCOPED_VAR)
                    .without(SymbolFlags::FUNCTION),
                self.scopes.get(self.current_scope).hoist_target,
            ),
            VariableKind::Let | VariableKind::Using | VariableKind::AwaitUsing => (
                SymbolFlags::BLOCK_SCOPED_VAR,
                SymbolFlags::VALUE_SPACE,
                self.current_scope,
            ),
            VariableKind::Const => (
                SymbolFlags::BLOCK_SCOPED_VAR.union(SymbolFlags::CONST_VAR),
                SymbolFlags::VALUE_SPACE,
                self.current_scope,
            ),
        };
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags,
                excludes,
                target,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_binding(node.node_id());
    }

    fn enter_class(&mut self, node: Class, ctx: &mut TraverseCtx<'_, 'a>) -> TraverseAction {
        let outer_scope = self.current_scope;
        let flags = ScopeFlags { strict: true };
        let binding_target = if node.class_type(ctx.ast()) == ClassType::ClassExpression
            && node.id(ctx.ast()).is_some()
        {
            self.open_expression_scope(node.node_id(), ScopeKind::Class, flags);
            self.scopes
                .get(self.current_scope)
                .parent
                .expect("named class expression has a name scope")
        } else {
            self.open_scope(node.node_id(), ScopeKind::Class, flags);
            outer_scope
        };
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::CLASS,
                excludes: SymbolFlags::VALUE_SPACE
                    .union(SymbolFlags::TYPE_SPACE)
                    .without(SymbolFlags::INTERFACE)
                    .without(SymbolFlags::VALUE_MODULE),
                target: binding_target,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_class(&mut self, node: Class, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
        self.close_scope(node.node_id());
    }

    fn enter_static_block(
        &mut self,
        node: StaticBlock,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(
            node.node_id(),
            ScopeKind::StaticBlock,
            ScopeFlags { strict: true },
        );
        TraverseAction::Proceed
    }

    fn exit_static_block(&mut self, node: StaticBlock, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.close_scope(node.node_id());
    }

    fn enter_import_declaration(
        &mut self,
        node: ImportDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_module_declaration_position(node, "An import declaration", ctx);
        let flags = if node.import_kind(ctx.ast()) == ImportOrExportKind::Type {
            SymbolFlags::TYPE_IMPORT
        } else {
            SymbolFlags::IMPORT
        };
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags,
                excludes: SymbolFlags::IMPORT.union(SymbolFlags::TYPE_IMPORT),
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_import_declaration(&mut self, node: ImportDeclaration, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
    }

    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_module_declaration_position(node, "An import declaration", ctx);
        let flags = if node.import_kind(ctx.ast()) == ImportOrExportKind::Type {
            SymbolFlags::TYPE_IMPORT
        } else {
            SymbolFlags::IMPORT
        };
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags,
                excludes: SymbolFlags::IMPORT.union(SymbolFlags::TYPE_IMPORT),
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_binding(node.node_id());
    }

    fn enter_import_specifier(
        &mut self,
        node: ImportSpecifier,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if node.import_kind(ctx.ast()) == ImportOrExportKind::Type {
            self.push_binding(
                node.node_id(),
                BindingContext {
                    flags: SymbolFlags::TYPE_IMPORT,
                    excludes: SymbolFlags::IMPORT.union(SymbolFlags::TYPE_IMPORT),
                    target: self.current_scope,
                },
            );
        }
        TraverseAction::Proceed
    }

    fn exit_import_specifier(&mut self, node: ImportSpecifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
    }

    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_module_declaration_position(node, "An export declaration", ctx);
        TraverseAction::Proceed
    }

    fn enter_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_module_declaration_position(node, "An export declaration", ctx);
        TraverseAction::Proceed
    }

    fn enter_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.check_module_declaration_position(node, "An export declaration", ctx);
        TraverseAction::Proceed
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        let target = self
            .scopes
            .get(self.current_scope)
            .parent
            .expect("type alias scope has a parent");
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::TYPE_ALIAS,
                excludes: SymbolFlags::TYPE_SPACE,
                target,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_binding(node.node_id());
        self.close_scope(node.node_id());
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        let target = self
            .scopes
            .get(self.current_scope)
            .parent
            .expect("interface scope has a parent");
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::INTERFACE,
                excludes: SymbolFlags::TYPE_SPACE
                    .without(SymbolFlags::INTERFACE)
                    .without(SymbolFlags::CLASS),
                target,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_binding(node.node_id());
        self.close_scope(node.node_id());
    }

    fn enter_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        let flags = if node.is_const(ctx.ast()) {
            SymbolFlags::CONST_ENUM
        } else {
            SymbolFlags::REGULAR_ENUM
        };
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags,
                excludes: SymbolFlags::VALUE_SPACE.union(SymbolFlags::TYPE_SPACE),
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_binding(node.node_id());
    }

    fn enter_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::VALUE_MODULE.union(SymbolFlags::NAMESPACE_MODULE),
                excludes: SymbolFlags::VALUE_SPACE
                    .without(SymbolFlags::FUNCTION)
                    .without(SymbolFlags::CLASS)
                    .without(SymbolFlags::REGULAR_ENUM)
                    .without(SymbolFlags::VALUE_MODULE),
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_binding(node.node_id());
    }

    fn enter_ts_type_parameter(
        &mut self,
        node: TsTypeParameter,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::TYPE_PARAMETER,
                excludes: SymbolFlags::TYPE_SPACE.without(SymbolFlags::TYPE_PARAMETER),
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_type_parameter(&mut self, node: TsTypeParameter, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
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
        self.write_targets
            .push(Some(node.left(ctx.ast()).node_id()));
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
            .push(Some(node.argument(ctx.ast()).node_id()));
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
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_function_type(&mut self, node: TsFunctionType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.close_scope(node.node_id());
    }

    fn enter_ts_constructor_type(
        &mut self,
        node: TsConstructorType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_constructor_type(
        &mut self,
        node: TsConstructorType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.close_scope(node.node_id());
    }

    fn enter_ts_method_signature(
        &mut self,
        node: TsMethodSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_method_signature(
        &mut self,
        node: TsMethodSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.close_scope(node.node_id());
    }

    fn enter_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.close_scope(node.node_id());
    }

    fn enter_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.close_scope(node.node_id());
    }

    fn enter_ts_index_signature(
        &mut self,
        node: TsIndexSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_index_signature(&mut self, node: TsIndexSignature, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.close_scope(node.node_id());
    }

    fn enter_ts_mapped_type(
        &mut self,
        node: TsMappedType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        self.push_binding(
            node.node_id(),
            BindingContext {
                flags: SymbolFlags::TYPE_PARAMETER,
                excludes: SymbolFlags::TYPE_SPACE.without(SymbolFlags::TYPE_PARAMETER),
                target: self.current_scope,
            },
        );
        TraverseAction::Proceed
    }

    fn exit_ts_mapped_type(&mut self, node: TsMappedType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_binding(node.node_id());
        self.close_scope(node.node_id());
    }

    fn enter_ts_conditional_type(
        &mut self,
        node: TsConditionalType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::Block, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_conditional_type(
        &mut self,
        node: TsConditionalType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.close_scope(node.node_id());
    }

    fn enter_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.open_scope(node.node_id(), ScopeKind::TsModule, self.inherited_flags());
        TraverseAction::Proceed
    }

    fn exit_ts_module_block(&mut self, node: TsModuleBlock, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.close_scope(node.node_id());
    }
}

struct ReferenceResolver<'semantic, 'a> {
    scopes: &'semantic ScopeTree,
    symbols: &'semantic mut SymbolTable<'a>,
    node_scopes: &'semantic [ScopeId],
    type_depth: u32,
    write_targets: Vec<Option<NodeId>>,
    diagnostics: &'semantic mut Vec<Diagnostic>,
}

impl<'semantic, 'a> ReferenceResolver<'semantic, 'a> {
    fn new(
        scopes: &'semantic ScopeTree,
        symbols: &'semantic mut SymbolTable<'a>,
        node_scopes: &'semantic [ScopeId],
        diagnostics: &'semantic mut Vec<Diagnostic>,
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
}

impl<'a> Traverse<'a> for ReferenceResolver<'_, 'a> {
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
        let name = node.name(ctx.ast());
        let scope = self.node_scopes[node.node_id().index()];
        let is_write = is_write_position(ctx, &self.write_targets);
        let kind = if self.type_depth > 0 {
            ReferenceKind::Type
        } else {
            ReferenceKind::Value
        };
        let reference_id = self
            .symbols
            .add_reference(name, scope, node.node_id(), kind, is_write);
        let symbol_id = self.symbols.resolve_reference(self.scopes, reference_id);
        node.set_symbol_id(ctx.ast_mut(), symbol_id);
        let strict = self.scopes.get(scope).flags.strict;
        if self.type_depth == 0 && strict && is_strict_reserved(name.as_str()) {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                format!("'{}' is reserved in strict mode", name.as_str()),
            ));
        }
        if is_write && strict && matches!(name.as_str(), "eval" | "arguments") {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                format!("Cannot assign to '{}' in strict mode", name.as_str()),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        if !is_write_position(ctx, &self.write_targets) {
            return TraverseAction::Proceed;
        }
        let name = node.name(ctx.ast());
        let scope = self.node_scopes[node.node_id().index()];
        let reference_id =
            self.symbols
                .add_reference(name, scope, node.node_id(), ReferenceKind::Value, true);
        let symbol_id = self.symbols.resolve_reference(self.scopes, reference_id);
        node.set_symbol_id(ctx.ast_mut(), symbol_id);
        if self.scopes.get(scope).flags.strict && matches!(name.as_str(), "eval" | "arguments") {
            self.diagnostics.push(Diagnostic::error(
                node.span(ctx.ast()),
                format!("Cannot assign to '{}' in strict mode", name.as_str()),
            ));
        }
        TraverseAction::Proceed
    }

    fn enter_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) -> TraverseAction {
        self.write_targets
            .push(Some(node.left(ctx.ast()).node_id()));
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
            .push(Some(node.argument(ctx.ast()).node_id()));
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
            .push(for_left_write_target(node.left(ctx.ast()), ctx.ast()));
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
            .push(for_left_write_target(node.left(ctx.ast()), ctx.ast()));
        TraverseAction::Proceed
    }

    fn exit_for_of_statement(&mut self, _node: ForOfStatement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.write_targets.pop();
    }
}

pub(crate) fn analyze_program<'a>(program: Program, ast: &mut Ast<'a>) -> AnalysisParts<'a> {
    let mut builder = SemanticBuilder::new(program, ast);
    let mut traverse_ctx = TraverseCtx::new(ast);
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
    let mut traverse_ctx = TraverseCtx::new(ast);
    let _ = program.traverse_with(&mut resolver, &mut traverse_ctx);

    AnalysisParts {
        scopes,
        symbols,
        node_scopes,
        diagnostics,
    }
}

fn body_has_use_strict(body: TypedSubRange<StatementListItem>, ast: &Ast<'_>) -> bool {
    for index in 0..body.len() {
        let item = body
            .get_node(ast, index)
            .expect("typed statement-list ranges contain valid nodes");
        match ast.statement_list_item_data(item) {
            StatementListItemData::Directive(directive) => {
                if directive.value(ast) == "use strict" {
                    return true;
                }
            }
            StatementListItemData::Stmt(_) => break,
        }
    }
    false
}

fn is_strict_reserved(name: &str) -> bool {
    matches!(
        name,
        "implements"
            | "interface"
            | "let"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "static"
            | "yield"
    )
}

fn is_write_position(ctx: &TraverseCtx<'_, '_>, targets: &[Option<NodeId>]) -> bool {
    ctx.ancestors()
        .any(|ancestor| targets.contains(&Some(ancestor)))
}

fn for_left_write_target(left: ForStatementLeft, ast: &Ast<'_>) -> Option<NodeId> {
    match ast.for_statement_left_data(left) {
        ForStatementLeftData::AssignmentTarget(target) => Some(target.node_id()),
        ForStatementLeftData::VariableDeclaration(_) => None,
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
            | NodeKind::TsTypeReference
            | NodeKind::TsTypeQuery
            | NodeKind::TsImportType
            | NodeKind::TsLiteralType
            | NodeKind::TsTemplateLiteralType
            | NodeKind::TsArrayType
            | NodeKind::TsIndexedAccessType
            | NodeKind::TsTupleType
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
            | NodeKind::TsMappedType
            | NodeKind::TsInterfaceHeritage
            | NodeKind::TsClassImplements
    )
}
