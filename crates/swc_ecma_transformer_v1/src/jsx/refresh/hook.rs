use std::{fmt::Write, mem};

use base64::prelude::{BASE64_STANDARD, Engine};
use sha1::{Digest, Sha1};
use swc_next_ecma_ast::*;

use crate::common::{build, resolver::context::Resolver};

use super::{
    RefreshOptions,
    util::{is_builtin_hook, make_call_expr, make_call_stmt, span_to_snippet},
};

struct HookSig {
    handle: BindingIdentifier,
    hooks: Vec<Hook>,
}

impl HookSig {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L23>.
    fn new(ast: &mut Ast<'_>, resolver: &mut Resolver, hooks: Vec<Hook>) -> Self {
        Self {
            handle: resolver.private_binding(ast, "_s"),
            hooks,
        }
    }
}

struct Hook {
    callee: HookCall,
    key: String,
}

enum HookCall {
    Ident(IdentifierReference),
    Member(Expr, IdentifierName),
}

pub(super) struct HookRegister<'a, 'ast> {
    pub ast: &'a mut Ast<'ast>,
    pub resolver: &'a mut Resolver,
    options: &'a RefreshOptions,
    pub ident: Vec<BindingIdentifier>,
    pub extra_stmt: Vec<Stmt>,
    current_scope: Vec<ScopeId>,
    should_reset: bool,
}

impl<'a, 'ast> HookRegister<'a, 'ast> {
    pub fn new(
        ast: &'a mut Ast<'ast>,
        resolver: &'a mut Resolver,
        options: &'a RefreshOptions,
        should_reset: bool,
    ) -> Self {
        let current_scope = vec![resolver.top_level()];
        Self {
            ast,
            resolver,
            options,
            ident: Vec::new(),
            extra_stmt: Vec::new(),
            current_scope,
            should_reset,
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L52>.
    pub fn gen_hook_handle(&mut self) -> Stmt {
        let decls = mem::take(&mut self.ident)
            .into_iter()
            .map(|id| {
                let callee =
                    self.resolver
                        .marked_reference(self.ast, &self.options.refresh_sig, true);
                let init = make_call_expr(self.ast, callee);
                self.ast.variable_declarator(
                    DUMMY_SP,
                    BindingPattern::BindingIdentifier(id),
                    Some(init),
                    false,
                )
            })
            .collect::<Vec<_>>();
        build::declaration(self.ast, VariableKind::Var, decls)
    }

    // Attach a signature to the function, including nested functions.
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L78>.
    fn wrap_with_register(
        &mut self,
        handle: BindingIdentifier,
        func: Expr,
        hooks: Vec<Hook>,
    ) -> Expr {
        let mut args = vec![func];
        let mut sign = Vec::new();
        let mut custom_hook = Vec::new();
        let ast = &mut *self.ast;

        for hook in hooks {
            let name = match &hook.callee {
                HookCall::Ident(ident) => ident.name(ast),
                HookCall::Member(_, ident) => ident.name(ast),
            };
            let name = ast.get_utf8(name);
            sign.push(format!("{name}{{{}}}", hook.key));
            match &hook.callee {
                HookCall::Ident(_) if !is_builtin_hook(name) => custom_hook.push(hook.callee),
                HookCall::Member(obj, _) if !is_builtin_hook(name) => {
                    if let Some(ident) = obj.as_identifier_reference(ast)
                        && ast.get_utf8(ident.name(ast)) != "React"
                    {
                        custom_hook.push(hook.callee);
                    }
                }
                _ => {}
            }
        }

        let sign = sign.join("\n");
        let sign = if self.options.emit_full_signatures {
            sign
        } else {
            BASE64_STANDARD.encode(Sha1::digest(sign))
        };
        let sign = ast.add_utf8(&sign).into();
        args.push(ast.expr_string_literal(DUMMY_SP, sign, AstUtf8::EMPTY));

        let mut should_reset = self.should_reset;
        let mut custom_hook_in_scope = Vec::new();
        for hook in custom_hook {
            let ident = match &hook {
                HookCall::Ident(ident) => Some(*ident),
                HookCall::Member(obj, _) => obj.as_identifier_reference(ast),
            };
            let in_scope = ident
                .and_then(|ident| self.resolver.scope_of(ident.node_id()))
                .is_some_and(|scope| self.current_scope.contains(&scope));
            if in_scope {
                custom_hook_in_scope.push(hook);
            } else {
                should_reset = true;
            }
        }

        if should_reset || !custom_hook_in_scope.is_empty() {
            args.push(ast.expr_boolean_literal(DUMMY_SP, should_reset));
        }

        if !custom_hook_in_scope.is_empty() {
            let elems = custom_hook_in_scope
                .into_iter()
                .map(|hook| {
                    let expr = match hook {
                        HookCall::Ident(ident) => {
                            build::clone_expr(ast, self.resolver, Expr::IdentifierReference(ident))
                        }
                        HookCall::Member(obj, prop) => {
                            let obj = build::clone_expr(ast, self.resolver, obj);
                            let prop = prop.clone_in(ast);
                            ast.expr_member_expression(
                                DUMMY_SP,
                                obj,
                                PropertyKey::IdentifierName(prop),
                                false,
                                false,
                            )
                        }
                    };
                    Some(Argument::Expr(expr))
                })
                .collect::<Vec<_>>();
            let elems = ast.add_sub_range(elems);
            let array = ast.expr_array_expression(DUMMY_SP, elems);
            let ret = ast.stmt_return_statement(DUMMY_SP, Some(array));
            let stmts = ast.add_sub_range([ret]);
            let body = ast.function_body(DUMMY_SP, SubRange::empty(), stmts);
            let params = ast.formal_parameters(
                DUMMY_SP,
                SubRange::empty(),
                None,
                FormalParameterKind::FormalParameters,
            );
            args.push(Expr::Function(ast.function(
                DUMMY_SP,
                FunctionType::FunctionExpression,
                None,
                None,
                None,
                params,
                None,
                body,
                false,
                false,
            )));
        }

        let callee = self.resolver.reference(ast, handle);
        build::call(ast, callee, args, false)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L196>.
    fn gen_hook_register_stmt(&mut self, ident: BindingIdentifier, sig: HookSig) {
        self.ident.push(sig.handle);
        let expr = self.resolver.reference(self.ast, ident);
        let expr = self.wrap_with_register(sig.handle, expr, sig.hooks);
        self.extra_stmt
            .push(self.ast.stmt_expression_statement(DUMMY_SP, expr));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L207>.
    fn visit_mut_stmt_list_with_scope(
        &mut self,
        scope: Option<ScopeId>,
        stmts: SubRange<Stmt>,
    ) -> SubRange<Stmt> {
        let old_ident = mem::take(&mut self.ident);
        let old_stmts = mem::take(&mut self.extra_stmt);
        self.current_scope.extend(scope);

        let mut items = Vec::with_capacity(stmts.len());
        for slot in stmts.iter() {
            let stmt = self.ast.get_node_in_sub_range(slot);
            let stmt = stmt.visit_mut_children_with(self).unwrap_or(stmt);
            items.push(stmt);
            items.append(&mut self.extra_stmt);
        }

        if !self.ident.is_empty() {
            items.insert(0, self.gen_hook_handle());
        }

        if scope.is_some() {
            self.current_scope.pop();
        }
        self.ident = old_ident;
        self.extra_stmt = old_stmts;
        self.ast.add_sub_range(items)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L232>.
    fn visit_mut_arrow_body_with_scope(&mut self, arrow: ArrowFunctionExpression) {
        match self.ast.arrow_function_body_data(arrow.body(self.ast)) {
            ArrowFunctionBodyData::FunctionBody(body) => self.visit_mut_function_stmts(body),
            ArrowFunctionBodyData::Expr(expr) => {
                if let Some(expr) = expr.visit_mut_with(self) {
                    arrow.set_body(self.ast, ArrowFunctionBody::Expr(expr));
                }
            }
        }
    }

    fn visit_mut_function_stmts(&mut self, body: FunctionBody) {
        let scope = self.resolver.scope_of(body.node_id());
        let stmts = self.visit_mut_stmt_list_with_scope(scope, body.body(self.ast));
        body.set_body(self.ast, stmts);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L387>.
    fn collect_hooks(&mut self, body: FunctionBody, self_ids: &[SymbolId]) -> Option<HookSig> {
        let mut hook = HookCollector::new(self.ast, self.resolver, self_ids);
        body.body(self.ast).visit_with(&mut hook);
        if hook.state.is_empty() {
            return None;
        }

        let sig = HookSig::new(self.ast, self.resolver, hook.state);
        let handle = self.resolver.reference(self.ast, sig.handle);
        let call = make_call_stmt(self.ast, handle);
        let stmts = build::prepend_stmts(self.ast, [call], body.body(self.ast));
        body.set_body(self.ast, stmts);
        Some(sig)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L406>.
    fn collect_hooks_arrow(
        &mut self,
        arrow: ArrowFunctionExpression,
        self_ids: &[SymbolId],
    ) -> Option<HookSig> {
        match self.ast.arrow_function_body_data(arrow.body(self.ast)) {
            ArrowFunctionBodyData::FunctionBody(body) => self.collect_hooks(body, self_ids),
            ArrowFunctionBodyData::Expr(expr) => {
                let mut hook = HookCollector::new(self.ast, self.resolver, self_ids);
                expr.visit_with(&mut hook);
                if hook.state.is_empty() {
                    return None;
                }
                let sig = HookSig::new(self.ast, self.resolver, hook.state);
                let handle = self.resolver.reference(self.ast, sig.handle);
                let call = make_call_stmt(self.ast, handle);
                let span = expr.span(self.ast);
                let ret = self.ast.stmt_return_statement(span, Some(expr));
                let stmts = self.ast.add_sub_range([call, ret]);
                let body = self.ast.function_body(span, SubRange::empty(), stmts);
                arrow.set_body(self.ast, ArrowFunctionBody::FunctionBody(body));
                arrow.set_expression(self.ast, false);
                Some(sig)
            }
        }
    }
}

impl<'ast> VisitMut<'ast> for HookRegister<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L251>.
    fn visit_mut_block_statement(&mut self, block: BlockStatement) -> Option<BlockStatement> {
        let scope = self.resolver.scope_of(block.node_id());
        let stmts = self.visit_mut_stmt_list_with_scope(scope, block.body(self.ast));
        block.set_body(self.ast, stmts);
        None
    }

    fn visit_mut_function_body(&mut self, _: FunctionBody) -> Option<FunctionBody> {
        None
    }

    fn visit_mut_ts_type(&mut self, _: TsType) -> Option<TsType> {
        None
    }

    fn visit_mut_ts_type_element(&mut self, _: TsTypeElement) -> Option<TsTypeElement> {
        None
    }

    fn visit_mut_ts_module_declaration(
        &mut self,
        _: TsModuleDeclaration,
    ) -> Option<TsModuleDeclaration> {
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L257>.
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L375>.
    fn visit_mut_function(&mut self, function: Function) -> Option<Function> {
        function.visit_mut_children_with(self);
        let body = function.body(self.ast);
        self.visit_mut_function_stmts(body);

        if function.function_type(self.ast) == FunctionType::FunctionDeclaration
            && let Some(ident) = function.id(self.ast)
        {
            let self_ids = [self.resolver.symbol_of(ident.node_id())];
            if let Some(sig) = self.collect_hooks(body, &self_ids) {
                self.gen_hook_register_stmt(ident, sig);
            }
        }
        None
    }

    /// SWC represents object methods as functions rather than expressions.
    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        if property.method(self.ast) || property.kind(self.ast) != PropertyKind::Init {
            let key = property.key(self.ast);
            if let Some(key) = key.visit_mut_with(self) {
                property.set_key(self.ast, key);
            }
            if let Some(function) = property.value(self.ast).as_function(self.ast) {
                function.visit_mut_with(self);
            }
            None
        } else {
            property.visit_mut_children_with(self)
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L273>.
    fn visit_mut_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
    ) -> Option<ArrowFunctionExpression> {
        arrow.visit_mut_children_with(self);
        if let Some(body) = arrow.body(self.ast).as_function_body(self.ast) {
            self.visit_mut_function_stmts(body);
        }
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L281>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        let replacement = expr.visit_mut_children_with(self);
        let expr = replacement.unwrap_or(expr);
        let sig = match self.ast.expr_data(expr) {
            ExprData::Function(function) => {
                let self_ids = function
                    .id(self.ast)
                    .and_then(|ident| self.resolver.symbol(ident.node_id()));
                self.collect_hooks(function.body(self.ast), self_ids.as_slice())
            }
            ExprData::ArrowFunctionExpression(arrow) => self.collect_hooks_arrow(arrow, &[]),
            _ => None,
        };
        if let Some(HookSig { handle, hooks }) = sig {
            self.ident.push(handle);
            Some(self.wrap_with_register(handle, expr, hooks))
        } else {
            replacement
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L309>.
    fn visit_mut_variable_declaration(
        &mut self,
        node: VariableDeclaration,
    ) -> Option<VariableDeclaration> {
        for slot in node.declarators(self.ast).iter() {
            let decl = self.ast.get_node_in_sub_range(slot);
            if let Some(id) = decl.id(self.ast).as_binding_identifier(self.ast)
                && let Some(init) = decl.init(self.ast)
            {
                let symbol = self.resolver.symbol_of(id.node_id());
                let sig = match self.ast.expr_data(init) {
                    ExprData::Function(function) => {
                        let body = function.body(self.ast);
                        self.visit_mut_function_stmts(body);
                        let mut self_ids = vec![symbol];
                        if let Some(ident) = function.id(self.ast) {
                            self_ids.push(self.resolver.symbol_of(ident.node_id()));
                        }
                        self.collect_hooks(body, &self_ids)
                    }
                    ExprData::ArrowFunctionExpression(arrow) => {
                        self.visit_mut_arrow_body_with_scope(arrow);
                        self.collect_hooks_arrow(arrow, &[symbol])
                    }
                    _ => {
                        if let Some(init) = init.visit_mut_with(self) {
                            decl.set_init(self.ast, Some(init));
                        }
                        None
                    }
                };
                if let Some(sig) = sig {
                    self.gen_hook_register_stmt(id, sig);
                }
            } else {
                decl.visit_mut_children_with(self);
            }
        }
        None
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L444>.
struct HookCollector<'a, 'ast> {
    ast: &'a Ast<'ast>,
    resolver: &'a Resolver,
    state: Vec<Hook>,
    self_ids: &'a [SymbolId],
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L454>.
fn is_hook_like(name: &str) -> bool {
    name.strip_prefix("use")
        .and_then(|name| name.chars().next())
        .is_some_and(char::is_uppercase)
}

impl<'a, 'ast> HookCollector<'a, 'ast> {
    fn new(ast: &'a Ast<'ast>, resolver: &'a Resolver, self_ids: &'a [SymbolId]) -> Self {
        Self {
            ast,
            resolver,
            state: Vec::new(),
            self_ids,
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L463>.
    fn get_hook_from_call_expr(
        &self,
        expr: CallExpression,
        lhs: Option<BindingPattern>,
    ) -> Option<Hook> {
        let ast = self.ast;
        let (callee, name) = match ast.expr_data(expr.callee(ast)) {
            ExprData::IdentifierReference(ident) => {
                if self
                    .resolver
                    .symbol(ident.node_id())
                    .is_some_and(|id| self.self_ids.contains(&id))
                {
                    return None;
                }
                (HookCall::Ident(ident), ident.name(ast))
            }
            ExprData::MemberExpression(member)
                if !member.computed(ast) && !member.object(ast).is_super(ast) =>
            {
                let ident = member.property(ast).as_identifier_name(ast)?;
                (HookCall::Member(member.object(ast), ident), ident.name(ast))
            }
            _ => return None,
        };
        let name = ast.get_utf8(name);
        if !is_hook_like(name) || build::chain_part(ast, Expr::CallExpression(expr)) {
            return None;
        }
        let mut key = lhs
            .map_or("", |lhs| span_to_snippet(ast, lhs.span(ast)))
            .to_owned();
        let args = expr.arguments(ast);
        let initial_state = match name {
            "useState" => args.get_node(ast, 0),
            "useReducer" => args.get_node(ast, 1),
            _ => None,
        };
        if let Some(arg) = initial_state {
            let _ = write!(key, "({})", span_to_snippet(ast, arg.span(ast)));
        }
        Some(Hook { callee, key })
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L532>.
    fn get_hook_from_expr(&self, expr: Expr, lhs: Option<BindingPattern>) -> Option<Hook> {
        self.get_hook_from_call_expr(expr.as_call_expression(self.ast)?, lhs)
    }
}

impl<'ast> Visit<'ast> for HookCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_arrow_function_body(&mut self, _: ArrowFunctionBody) {}

    fn visit_block_statement(&mut self, _: BlockStatement) {}

    fn visit_function_body(&mut self, _: FunctionBody) {}

    fn visit_ts_type(&mut self, _: TsType) {}

    fn visit_ts_type_element(&mut self, _: TsTypeElement) {}

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L550>.
    fn visit_expr(&mut self, expr: Expr) {
        expr.visit_children_with(self);
        if let Some(hook) = self.get_hook_from_expr(expr, None) {
            self.state.push(hook);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L560>.
    fn visit_stmt(&mut self, stmt: Stmt) {
        let ast = self.ast;
        let expr = match ast.stmt_data(stmt) {
            StmtData::ExpressionStatement(stmt) => Some(stmt.expression(ast)),
            StmtData::ReturnStatement(stmt) => stmt.argument(ast),
            StmtData::Declaration(decl)
                if let Some(var_decl) = decl.as_variable_declaration(ast) =>
            {
                for slot in var_decl.declarators(ast).iter() {
                    let decl = ast.get_node_in_sub_range(slot);
                    let hook = decl
                        .init(ast)
                        .and_then(|init| self.get_hook_from_expr(init, Some(decl.id(ast))));
                    if let Some(hook) = hook {
                        self.state.push(hook);
                    } else {
                        stmt.visit_children_with(self);
                    }
                }
                return;
            }
            _ => None,
        };
        if let Some(hook) = expr.and_then(|expr| self.get_hook_from_expr(expr, None)) {
            self.state.push(hook);
        } else {
            stmt.visit_children_with(self);
        }
    }
}
