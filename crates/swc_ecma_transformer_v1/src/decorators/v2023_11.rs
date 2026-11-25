use std::{collections::VecDeque, iter::once, mem};

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::*;

use crate::common::{
    TraverseCtx, build, constructor::inject_after_super, resolver::context::Resolver,
    super_field::rewrite_super_in_moved_static_member,
};

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L20>.
pub(crate) fn transform(ast: &mut Ast<'_>, ctx: &mut TraverseCtx<'_>) {
    let program = ast.root_program();
    let mut pass = DecoratorPass {
        ast,
        ctx,
        extra_vars: Vec::new(),
        extra_lets: Vec::new(),
        state: ClassState::default(),
        pre_class_inits: Vec::new(),
        rename_map: FxHashMap::default(),
        extra_exports: Vec::new(),
        current_class_name: None,
        class_member_class_ref_alias: None,
        implicit_globals: FxHashSet::default(),
        marked_implicit_globals: FxHashSet::default(),
        in_implicit_global_rewrite_expr: false,
        decorator_binding_ids_for_implicit_global_rewrite: FxHashSet::default(),
    };
    program.visit_mut_with(&mut pass);
}

#[derive(Default)]
struct ClassState {
    private_id_index: u32,

    static_non_field_lhs: Vec<BindingIdentifier>,
    proto_non_field_lhs: Vec<BindingIdentifier>,
    static_field_lhs: Vec<BindingIdentifier>,
    proto_field_lhs: Vec<BindingIdentifier>,
    pending_instance_field_extra_inits: Vec<Expr>,

    init_proto: Option<BindingIdentifier>,
    init_proto_args: Vec<Option<Argument>>,

    init_static: Option<BindingIdentifier>,
    init_static_args: Vec<Option<Argument>>,

    extra_stmts: Vec<Stmt>,
    computed_key_inits: Vec<Expr>,

    class_lhs: Vec<Option<BindingPattern>>,
    class_decorators: Vec<Option<Argument>>,
    class_decorators_have_this: bool,
    instance_brand: Option<String>,

    super_class: Option<BindingIdentifier>,
}

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L28>.
struct DecoratorPass<'a, 'ast, 'r> {
    ast: &'a mut Ast<'ast>,
    ctx: &'a mut TraverseCtx<'r>,

    extra_vars: Vec<VariableDeclarator>,
    extra_lets: Vec<VariableDeclarator>,
    state: ClassState,
    pre_class_inits: Vec<Expr>,
    rename_map: FxHashMap<SymbolId, SymbolId>,
    extra_exports: Vec<ExportSpecifier>,

    current_class_name: Option<BindingIdentifier>,
    class_member_class_ref_alias: Option<BindingIdentifier>,

    implicit_globals: FxHashSet<String>,
    marked_implicit_globals: FxHashSet<NodeId>,
    in_implicit_global_rewrite_expr: bool,
    decorator_binding_ids_for_implicit_global_rewrite: FxHashSet<SymbolId>,
}

// ======================= Analysis Visitors =======================

struct PrivateNameFinder<'a, 'ast> {
    ast: &'a Ast<'ast>,
    found: bool,
}

impl<'ast> Visit<'ast> for PrivateNameFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_private_identifier(&mut self, _: PrivateIdentifier) {
        self.found = true;
    }
}

struct YieldAwaitFinder<'a, 'ast> {
    ast: &'a Ast<'ast>,
    found: bool,
}

impl<'ast> Visit<'ast> for YieldAwaitFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_yield_expression(&mut self, _: YieldExpression) {
        self.found = true;
    }

    fn visit_await_expression(&mut self, _: AwaitExpression) {
        self.found = true;
    }
}

struct InstancePrivateNameFinder<'a, 'ast> {
    ast: &'a Ast<'ast>,
    names: &'a FxHashSet<String>,
    found: bool,
}

impl<'ast> Visit<'ast> for InstancePrivateNameFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_private_identifier(&mut self, ident: PrivateIdentifier) {
        if self.names.contains(self.ast.get_utf8(ident.name(self.ast))) {
            self.found = true;
        }
    }
}

struct CurrentClassNameFinder<'a, 'ast> {
    ast: &'a Ast<'ast>,
    resolver: &'a Resolver,
    target: SymbolId,
    found: bool,
}

impl<'ast> Visit<'ast> for CurrentClassNameFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, ident: IdentifierReference) {
        if self.resolver.symbol(ident.node_id()) == Some(self.target) {
            self.found = true;
        }
    }

    fn visit_binding_identifier(&mut self, ident: BindingIdentifier) {
        if self.resolver.symbol(ident.node_id()) == Some(self.target) {
            self.found = true;
        }
    }
}

struct MovedStaticSuperFinder<'a, 'ast> {
    ast: &'a Ast<'ast>,
    found: bool,
}

impl<'ast> Visit<'ast> for MovedStaticSuperFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_class(&mut self, _: Class) {}

    fn visit_function(&mut self, _: Function) {}

    fn visit_super(&mut self, _: Super) {
        self.found = true;
    }
}

struct SymbolReplacer<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    map: &'a FxHashMap<SymbolId, SymbolId>,
}

impl<'ast> VisitMut<'ast> for SymbolReplacer<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_identifier_reference(
        &mut self,
        ident: IdentifierReference,
    ) -> Option<IdentifierReference> {
        if let Some(sym) = self.resolver.symbol(ident.node_id())
            && let Some(&target) = self.map.get(&sym)
        {
            let name = self.resolver.symbol_name(target);
            ident.set_name(self.ast, name);
            self.resolver.set_symbol(ident.node_id(), target);
        }
        None
    }

    fn visit_mut_binding_identifier(
        &mut self,
        ident: BindingIdentifier,
    ) -> Option<BindingIdentifier> {
        if let Some(sym) = self.resolver.symbol(ident.node_id())
            && let Some(&target) = self.map.get(&sym)
        {
            let name = self.resolver.symbol_name(target);
            ident.set_name(self.ast, name);
            self.resolver.set_symbol(ident.node_id(), target);
        }
        None
    }

    fn visit_mut_object_property(&mut self, prop: ObjectProperty) -> Option<ObjectProperty> {
        if prop.shorthand(self.ast)
            && let Some(id) = prop.value(self.ast).as_identifier_reference(self.ast)
            && let Some(sym) = self.resolver.symbol(id.node_id())
            && self.map.contains_key(&sym)
        {
            prop.set_shorthand(self.ast, false);
        }
        prop.visit_mut_children_with(self)
    }

    fn visit_mut_binding_property(&mut self, prop: BindingProperty) -> Option<BindingProperty> {
        prop.visit_mut_children_with(self);
        build::binding_shorthand(self.ast, prop);
        None
    }

    fn visit_mut_export_named_declaration(
        &mut self,
        export: ExportNamedDeclaration,
    ) -> Option<ExportNamedDeclaration> {
        if export.source(self.ast).is_none() {
            let specs = export.specifiers(self.ast);
            for slot in specs.iter() {
                let spec = self.ast.get_node_in_sub_range(slot);
                if let Some(local) = spec.local(self.ast).as_identifier_name(self.ast)
                    && let Some(sym) = self.resolver.symbol(local.node_id())
                    && let Some(&target) = self.map.get(&sym)
                {
                    let name = self.resolver.symbol_name(target);
                    local.set_name(self.ast, name);
                    self.resolver.set_symbol(local.node_id(), target);
                }
            }
        }
        export.visit_mut_children_with(self)
    }
}

fn replace_symbol<'ast, N>(
    ast: &mut Ast<'ast>,
    resolver: &mut Resolver,
    node: N,
    from: SymbolId,
    to: SymbolId,
) where
    for<'a> N: VisitMutWith<'ast, SymbolReplacer<'a, 'ast>>,
{
    let mut map = FxHashMap::default();
    map.insert(from, to);
    rename_symbols(ast, resolver, node, &map);
}

fn rename_symbols<'ast, N>(
    ast: &mut Ast<'ast>,
    resolver: &mut Resolver,
    node: N,
    map: &FxHashMap<SymbolId, SymbolId>,
) where
    for<'a> N: VisitMutWith<'ast, SymbolReplacer<'a, 'ast>>,
{
    if map.is_empty() {
        return;
    }
    let mut replacer = SymbolReplacer { ast, resolver, map };
    node.visit_mut_with(&mut replacer);
}

// ======================= Helper Builders =======================

fn clone_function(ast: &mut Ast<'_>, resolver: &mut Resolver, func: Function) -> Function {
    let expr = build::clone_expr(ast, resolver, Expr::Function(func));
    expr.as_function(ast).unwrap()
}

fn clone_property_key(ast: &mut Ast<'_>, resolver: &mut Resolver, key: PropertyKey) -> PropertyKey {
    match ast.property_key_data(key) {
        PropertyKeyData::IdentifierName(id) => {
            PropertyKey::IdentifierName(ast.identifier_name(id.span(ast), id.name(ast)))
        }
        PropertyKeyData::Expr(expr) => PropertyKey::Expr(build::clone_expr(ast, resolver, expr)),
    }
}

fn prop_key_to_expr_value(ast: &mut Ast<'_>, resolver: &mut Resolver, key: PropertyKey) -> Expr {
    match ast.property_key_data(key) {
        PropertyKeyData::IdentifierName(id) => {
            let name = ast.get_utf8(id.name(ast)).to_owned();
            build::string(ast, &name)
        }
        PropertyKeyData::Expr(expr) => build::clone_expr(ast, resolver, expr),
    }
}

fn make_private_key(ast: &mut Ast<'_>, name: &str) -> PropertyKey {
    let sym = ast.add_utf8(name);
    PropertyKey::Expr(Expr::PrivateIdentifier(
        ast.private_identifier(DUMMY_SP, sym),
    ))
}

fn is_maybe_directive(ast: &Ast<'_>, stmt: Stmt) -> bool {
    stmt.as_expression_statement(ast)
        .is_some_and(|expr_stmt| expr_stmt.expression(ast).is_string_literal(ast))
}

fn default_constructor(ast: &mut Ast<'_>, resolver: &mut Resolver, is_derived: bool) -> MethodDefinition {
    let (params, stmts) = if is_derived {
        let args_binding = resolver.private_binding(ast, "args");
        let rest = ast.formal_parameter_rest(
            DUMMY_SP,
            SubRange::empty(),
            FormalParameterPattern::BindingIdentifier(args_binding),
            None,
        );
        let params = ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            Some(rest),
            FormalParameterKind::FormalParameters,
        );
        let super_expr = ast.expr_super(DUMMY_SP);
        let args_ref = resolver.reference(ast, args_binding);
        let spread = Argument::SpreadElement(ast.spread_element(DUMMY_SP, args_ref));
        let call_args = ast.add_sub_range([spread]);
        let super_call = ast.expr_call_expression(DUMMY_SP, super_expr, None, call_args, false, false);
        let stmt = ast.stmt_expression_statement(DUMMY_SP, super_call);
        (params, vec![stmt])
    } else {
        let params = ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::FormalParameters,
        );
        (params, Vec::new())
    };
    let body = build::function_body(ast, stmts);
    let func = ast.function(
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
    );
    let ctor_name = ast.add_utf8("constructor");
    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, ctor_name));
    ast.method_definition(
        DUMMY_SP,
        SubRange::empty(),
        key,
        func,
        MethodDefinitionKind::Constructor,
        false,
        false,
        false,
        false,
        false,
        Accessibility::None,
    )
}

fn identity_constructor(ast: &mut Ast<'_>) -> MethodDefinition {
    let params = ast.formal_parameters(
        DUMMY_SP,
        SubRange::empty(),
        None,
        FormalParameterKind::FormalParameters,
    );
    let body = build::function_body(ast, Vec::new());
    let func = ast.function(
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
    );
    let ctor_name = ast.add_utf8("constructor");
    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, ctor_name));
    ast.method_definition(
        DUMMY_SP,
        SubRange::empty(),
        key,
        func,
        MethodDefinitionKind::Constructor,
        false,
        false,
        false,
        false,
        false,
        Accessibility::None,
    )
}

// ======================= DecoratorPass Methods =======================

impl DecoratorPass<'_, '_, '_> {
    fn rewrite_implicit_globals_in_expr(&mut self, expr: Expr) -> Expr {
        let old = self.in_implicit_global_rewrite_expr;
        self.in_implicit_global_rewrite_expr = true;
        let expr = expr.visit_mut_with(self).unwrap_or(expr);
        self.in_implicit_global_rewrite_expr = old;
        expr
    }

    fn global_this_member_expr(&mut self, name: &str) -> Expr {
        let global_this = self.ctx.resolver.marked_reference(self.ast, "globalThis", true);
        build::member(self.ast, global_this, name)
    }

    fn mark_ident_for_implicit_global_rewrite(&mut self, id: IdentifierReference) {
        if self.ctx.resolver.is_unresolved(id.node_id()) {
            self.marked_implicit_globals.insert(id.node_id());
        }
    }

    fn maybe_rewrite_implicit_globals_in_var_decl(&mut self, var_decl: VariableDeclaration) {
        let decls = var_decl.declarators(self.ast);
        for slot in decls.iter() {
            let decl = self.ast.get_node_in_sub_range(slot);
            let mut is_decorator_binding = false;
            build::for_each_binding_ident(self.ast, decl.id(self.ast), |binding| {
                if let Some(sym) = self.ctx.resolver.symbol(binding.node_id())
                    && self
                        .decorator_binding_ids_for_implicit_global_rewrite
                        .contains(&sym)
                {
                    is_decorator_binding = true;
                }
            });
            if !is_decorator_binding {
                continue;
            }
            if let Some(init) = decl.init(self.ast) {
                let rewritten = self.rewrite_implicit_globals_in_expr(init);
                decl.set_init(self.ast, Some(rewritten));
            }
        }
    }

    fn maybe_rewrite_implicit_globals_in_stmt(&mut self, stmt: Stmt) {
        match self.ast.stmt_data(stmt) {
            StmtData::Declaration(decl) => {
                if let Some(var_decl) = decl.as_variable_declaration(self.ast) {
                    self.maybe_rewrite_implicit_globals_in_var_decl(var_decl);
                }
            }
            StmtData::ExportNamedDeclaration(export) => {
                if let Some(decl) = export.declaration(self.ast)
                    && let Some(var_decl) = decl.as_variable_declaration(self.ast)
                {
                    self.maybe_rewrite_implicit_globals_in_var_decl(var_decl);
                }
            }
            _ => {}
        }
    }

    fn rewrite_implicit_globals_in_class_expr(&mut self, cls: Class) {
        let decs = cls.decorators(self.ast);
        for slot in decs.iter() {
            let dec = self.ast.get_node_in_sub_range(slot);
            let expr = dec.expression(self.ast);
            if let Some(replacement) = expr.visit_mut_with(self) {
                dec.set_expression(self.ast, replacement);
            }
        }
        if let Some(super_class) = cls.super_class(self.ast)
            && let Some(replacement) = super_class.visit_mut_with(self)
        {
            cls.set_super_class(self.ast, Some(replacement));
        }
        let body = cls.body(self.ast).body(self.ast);
        for slot in body.iter() {
            let elem = self.ast.get_node_in_sub_range(slot);
            match self.ast.class_element_data(elem) {
                ClassElementData::PropertyDefinition(prop) => {
                    if prop.computed(self.ast)
                        && let Some(key_expr) = prop.key(self.ast).as_expr(self.ast)
                        && let Some(replacement) = key_expr.visit_mut_with(self)
                    {
                        prop.set_key(self.ast, PropertyKey::Expr(replacement));
                    }
                    let decs = prop.decorators(self.ast);
                    for d_slot in decs.iter() {
                        let dec = self.ast.get_node_in_sub_range(d_slot);
                        let expr = dec.expression(self.ast);
                        if let Some(replacement) = expr.visit_mut_with(self) {
                            dec.set_expression(self.ast, replacement);
                        }
                    }
                    if prop.r#static(self.ast)
                        && let Some(value) = prop.value(self.ast)
                        && let Some(replacement) = value.visit_mut_with(self)
                    {
                        prop.set_value(self.ast, Some(replacement));
                    }
                }
                ClassElementData::MethodDefinition(method) => {
                    if method.computed(self.ast)
                        && let Some(key_expr) = method.key(self.ast).as_expr(self.ast)
                        && let Some(replacement) = key_expr.visit_mut_with(self)
                    {
                        method.set_key(self.ast, PropertyKey::Expr(replacement));
                    }
                    let decs = method.decorators(self.ast);
                    for d_slot in decs.iter() {
                        let dec = self.ast.get_node_in_sub_range(d_slot);
                        let expr = dec.expression(self.ast);
                        if let Some(replacement) = expr.visit_mut_with(self) {
                            dec.set_expression(self.ast, replacement);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn call_to_property_key(&mut self, expr: Expr) -> Expr {
        let helper = self.ctx.helper(self.ast, "to_property_key");
        build::call(self.ast, helper, [expr], false)
    }

    fn expr_uses_private_name(&self, expr: Expr) -> bool {
        let mut finder = PrivateNameFinder {
            ast: self.ast,
            found: false,
        };
        expr.visit_with(&mut finder);
        finder.found
    }

    fn expr_uses_yield_or_await(&self, expr: Expr) -> bool {
        let mut finder = YieldAwaitFinder {
            ast: self.ast,
            found: false,
        };
        expr.visit_with(&mut finder);
        finder.found
    }

    fn expr_uses_current_class_name(&self, expr: Expr) -> bool {
        let Some(class_name) = self.current_class_name else {
            return false;
        };
        let Some(target) = self.ctx.resolver.symbol(class_name.node_id()) else {
            return false;
        };
        let mut finder = CurrentClassNameFinder {
            ast: self.ast,
            resolver: self.ctx.resolver,
            target,
            found: false,
        };
        expr.visit_with(&mut finder);
        finder.found
    }

    fn should_queue_class_key_init(&self, expr: Expr, allow_class_name_queue: bool) -> bool {
        if self.expr_uses_private_name(expr) && self.expr_uses_yield_or_await(expr) {
            return true;
        }
        allow_class_name_queue && self.expr_uses_current_class_name(expr)
    }

    fn queue_pre_class_init(&mut self, expr: Expr, prefer_class_key: bool) {
        if prefer_class_key {
            self.state.computed_key_inits.push(expr);
        } else {
            self.pre_class_inits.push(expr);
        }
    }

    fn take_computed_key_init_member(&mut self) -> Option<ClassElement> {
        if self.state.computed_key_inits.is_empty() {
            return None;
        }
        let mut exprs = mem::take(&mut self.state.computed_key_inits);
        exprs.push(build::string(self.ast, "_"));
        let key_expr = build::sequence(self.ast, exprs);
        let params = self.ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::FormalParameters,
        );
        let body = build::function_body(self.ast, Vec::new());
        let func = self.ast.function(
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
        );
        Some(ClassElement::MethodDefinition(self.ast.method_definition(
            DUMMY_SP,
            SubRange::empty(),
            PropertyKey::Expr(key_expr),
            func,
            MethodDefinitionKind::Method,
            true,
            true,
            false,
            false,
            false,
            Accessibility::None,
        )))
    }

    fn collect_instance_private_names(&self, body: ClassBody) -> FxHashSet<String> {
        let mut names = FxHashSet::default();
        for slot in body.body(self.ast).iter() {
            let elem = self.ast.get_node_in_sub_range(slot);
            match self.ast.class_element_data(elem) {
                ClassElementData::PropertyDefinition(prop) if !prop.r#static(self.ast) => {
                    if let Some(name) = build::private_name(self.ast, prop.key(self.ast)) {
                        names.insert(name);
                    }
                }
                ClassElementData::MethodDefinition(method) if !method.r#static(self.ast) => {
                    if let Some(name) = build::private_name(self.ast, method.key(self.ast)) {
                        names.insert(name);
                    }
                }
                _ => {}
            }
        }
        names
    }

    fn expr_needs_moved_static_rewrite(
        &self,
        expr: Expr,
        instance_private_names: &FxHashSet<String>,
    ) -> bool {
        if !instance_private_names.is_empty() {
            let mut finder = InstancePrivateNameFinder {
                ast: self.ast,
                names: instance_private_names,
                found: false,
            };
            expr.visit_with(&mut finder);
            if finder.found {
                return true;
            }
        }
        let mut super_finder = MovedStaticSuperFinder {
            ast: self.ast,
            found: false,
        };
        expr.visit_with(&mut super_finder);
        super_finder.found
    }

    fn stmts_need_moved_static_rewrite(
        &self,
        stmts: &[Stmt],
        instance_private_names: &FxHashSet<String>,
    ) -> bool {
        if !instance_private_names.is_empty() {
            let mut finder = InstancePrivateNameFinder {
                ast: self.ast,
                names: instance_private_names,
                found: false,
            };
            for &stmt in stmts {
                stmt.visit_with(&mut finder);
                if finder.found {
                    return true;
                }
            }
        }
        let mut super_finder = MovedStaticSuperFinder {
            ast: self.ast,
            found: false,
        };
        for &stmt in stmts {
            stmt.visit_with(&mut super_finder);
            if super_finder.found {
                return true;
            }
        }
        false
    }

    fn function_needs_moved_static_rewrite(
        &self,
        function: Function,
        instance_private_names: &FxHashSet<String>,
    ) -> bool {
        if !instance_private_names.is_empty() {
            let mut finder = InstancePrivateNameFinder {
                ast: self.ast,
                names: instance_private_names,
                found: false,
            };
            function.visit_with(&mut finder);
            if finder.found {
                return true;
            }
        }
        let mut super_finder = MovedStaticSuperFinder {
            ast: self.ast,
            found: false,
        };
        function.params(self.ast).visit_with(&mut super_finder);
        function.body(self.ast).visit_with(&mut super_finder);
        super_finder.found
    }

    fn current_class_ref_for_super(&mut self) -> Option<BindingIdentifier> {
        let class_name = self.current_class_name?;
        if let Some(sym) = self.ctx.resolver.symbol(class_name.node_id())
            && let Some(&renamed) = self.rename_map.get(&sym)
        {
            return Some(self.ctx.resolver.binding(self.ast, renamed));
        }
        Some(class_name)
    }

    fn active_class_ref_for_super(&mut self) -> Option<BindingIdentifier> {
        if let Some(Some(pat)) = self.state.class_lhs.first()
            && let Some(binding) = pat.as_binding_identifier(self.ast)
        {
            return Some(binding);
        }
        self.current_class_ref_for_super()
    }

    fn maybe_alias_current_class_name_expr(&mut self, expr: Expr) {
        let (Some(class_name), Some(alias)) =
            (self.current_class_name, self.class_member_class_ref_alias)
        else {
            return;
        };
        let from = self.ctx.resolver.symbol_of(class_name.node_id());
        let to = self.ctx.resolver.symbol_of(alias.node_id());
        replace_symbol(self.ast, self.ctx.resolver, expr, from, to);
    }

    fn memoize_static_closure(
        &mut self,
        hint: &str,
        expr: Expr,
        assignments: &mut Vec<Stmt>,
    ) -> BindingIdentifier {
        let id = self.ctx.resolver.private_binding(self.ast, &format!("_{hint}"));
        self.extra_vars.push(build::declarator(self.ast, id, None));
        let lhs = self.ctx.resolver.reference(self.ast, id);
        let assign = build::assign(self.ast, lhs, expr);
        assignments.push(self.ast.stmt_expression_statement(DUMMY_SP, assign));
        id
    }

    fn call_closure_with_this(&mut self, closure: BindingIdentifier) -> Expr {
        let closure_ref = self.ctx.resolver.reference(self.ast, closure);
        let callee = build::member(self.ast, closure_ref, "call");
        let this = self.ast.expr_this_expression(DUMMY_SP);
        build::call(self.ast, callee, [this], false)
    }

    fn maybe_wrap_static_initializer_with_closure(
        &mut self,
        value: Option<Expr>,
        class_name: BindingIdentifier,
        instance_private_names: &FxHashSet<String>,
        assignments: &mut Vec<Stmt>,
    ) -> Option<Expr> {
        let value = value?;
        if !self.expr_needs_moved_static_rewrite(value, instance_private_names) {
            return Some(value);
        }
        let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(value));
        let fn_expr = build::function_with(self.ast, [], [ret]);
        let closure_fn = fn_expr.as_function(self.ast).unwrap();
        rewrite_super_in_moved_static_member(self.ast, self.ctx, closure_fn, class_name);
        let closure = self.memoize_static_closure("fieldValue", fn_expr, assignments);
        Some(self.call_closure_with_this(closure))
    }

    fn push_lhs(&mut self, ident: BindingIdentifier, is_static: bool, is_field: bool) {
        match (is_static, is_field) {
            (true, false) => self.state.static_non_field_lhs.push(ident),
            (false, false) => self.state.proto_non_field_lhs.push(ident),
            (true, true) => self.state.static_field_lhs.push(ident),
            (false, true) => self.state.proto_field_lhs.push(ident),
        }
    }

    fn prepend_pending_instance_field_extras(
        &mut self,
        value: Option<Expr>,
        is_static: bool,
    ) -> Option<Expr> {
        if is_static || self.state.pending_instance_field_extra_inits.is_empty() {
            return value;
        }
        let tail = value.unwrap_or_else(|| build::undefined(self.ast));
        let exprs: Vec<_> = self
            .state
            .pending_instance_field_extra_inits
            .drain(..)
            .chain(once(tail))
            .collect();
        Some(build::sequence(self.ast, exprs))
    }

    fn queue_pending_instance_field_extra(&mut self, expr: Expr, is_static: bool) {
        if !is_static {
            self.state.pending_instance_field_extra_inits.push(expr);
        }
    }

    fn preserve_decorator_this_target(&mut self, target: Expr) -> Expr {
        let target = self.rewrite_implicit_globals_in_expr(target);
        if target.is_identifier_reference(self.ast) || target.is_this_expression(self.ast) {
            return target;
        }
        let ident = self.ctx.resolver.private_binding(self.ast, "_dec_this");
        self.extra_vars
            .push(build::declarator(self.ast, ident, None));
        let lhs = self.ctx.resolver.reference(self.ast, ident);
        self.pre_class_inits
            .push(build::assign(self.ast, lhs, target));
        self.ctx.resolver.reference(self.ast, ident)
    }

    fn preserve_side_effect_of_decorator_with_options(
        &mut self,
        dec: Expr,
        allow_class_name_queue: bool,
    ) -> Expr {
        if let Some(ident) = dec.as_identifier_reference(self.ast)
            && let Some(sym) = self.ctx.resolver.symbol(ident.node_id())
        {
            self.decorator_binding_ids_for_implicit_global_rewrite
                .insert(sym);
        }

        let return_directly = dec.is_identifier_reference(self.ast)
            || dec.is_arrow_function_expression(self.ast)
            || dec.is_function(self.ast);

        let dec = self.rewrite_implicit_globals_in_expr(dec);

        if return_directly {
            return dec;
        }

        let prefer_class_key = self.should_queue_class_key_init(dec, allow_class_name_queue);
        let ident = self.ctx.resolver.private_binding(self.ast, "_dec");
        self.extra_vars
            .push(build::declarator(self.ast, ident, None));
        let lhs = self.ctx.resolver.reference(self.ast, ident);
        let assign = build::assign(self.ast, lhs, dec);
        self.queue_pre_class_init(assign, prefer_class_key);
        self.ctx.resolver.reference(self.ast, ident)
    }

    fn preserve_side_effect_of_decorator(&mut self, dec: Expr) -> Expr {
        self.preserve_side_effect_of_decorator_with_options(dec, true)
    }

    fn preserve_side_effect_of_decorator_2023(&mut self, dec: Expr) -> (Option<Argument>, Argument) {
        if let Some(member) = dec.as_member_expression(self.ast) {
            let obj = member.object(self.ast);
            let this_target = self.preserve_decorator_this_target(obj);
            let this_clone = build::clone_expr(self.ast, self.ctx.resolver, this_target);
            let new_member = self.ast.expr_member_expression(
                member.span(self.ast),
                this_clone,
                member.property(self.ast),
                member.computed(self.ast),
                member.optional(self.ast),
            );
            return (Some(Argument::Expr(this_target)), Argument::Expr(new_member));
        }

        if self.expr_uses_private_name(dec) && !self.expr_uses_yield_or_await(dec) {
            return (None, Argument::Expr(dec));
        }

        let dec = self.preserve_side_effect_of_decorator(dec);
        (None, Argument::Expr(dec))
    }

    fn merge_decorators_for_member(&mut self, decorators: Vec<Expr>) -> (Option<Argument>, bool) {
        let mut has_this = false;
        let mut preserved = Vec::with_capacity(decorators.len());
        for dec in decorators {
            let (dec_this, dec) = self.preserve_side_effect_of_decorator_2023(dec);
            has_this |= dec_this.is_some();
            preserved.push((dec_this, dec));
        }

        let merged = if has_this {
            let mut elems = Vec::with_capacity(preserved.len() * 2);
            for (dec_this, dec) in preserved {
                let this_arg = dec_this.unwrap_or_else(|| Argument::Expr(build::undefined(self.ast)));
                elems.push(Some(this_arg));
                elems.push(Some(dec));
            }
            let elements = self.ast.add_sub_range(elems);
            Some(Argument::Expr(
                self.ast.expr_array_expression(DUMMY_SP, elements),
            ))
        } else {
            let elems: Vec<_> = preserved.into_iter().map(|(_, dec)| Some(dec)).collect();
            self.merge_decorators(elems)
        };

        (merged, has_this)
    }

    fn merge_decorators(&mut self, decorators: Vec<Option<Argument>>) -> Option<Argument> {
        if decorators.len() == 1 {
            return decorators.into_iter().next().unwrap();
        }
        let elements = self.ast.add_sub_range(decorators);
        Some(Argument::Expr(
            self.ast.expr_array_expression(DUMMY_SP, elements),
        ))
    }

    fn preserve_side_effect_of_decorators_for_class(
        &mut self,
        decorators: Vec<Expr>,
    ) -> (Vec<Option<Argument>>, bool) {
        let mut has_this = false;
        let mut preserved = Vec::with_capacity(decorators.len());
        for dec in decorators {
            let (dec_this, dec) = if let Some(member) = dec.as_member_expression(self.ast) {
                let obj = member.object(self.ast);
                let this_target = self.preserve_decorator_this_target(obj);
                let this_clone = build::clone_expr(self.ast, self.ctx.resolver, this_target);
                let member_expr = self.ast.expr_member_expression(
                    member.span(self.ast),
                    this_clone,
                    member.property(self.ast),
                    member.computed(self.ast),
                    member.optional(self.ast),
                );
                let dec = self.preserve_side_effect_of_decorator_with_options(member_expr, false);
                (Some(Argument::Expr(this_target)), Argument::Expr(dec))
            } else {
                let dec = self.preserve_side_effect_of_decorator_with_options(dec, false);
                (None, Argument::Expr(dec))
            };
            has_this |= dec_this.is_some();
            preserved.push((dec_this, dec));
        }

        let decorators = if has_this {
            let mut elems = Vec::with_capacity(preserved.len() * 2);
            for (dec_this, dec) in preserved {
                let this_arg = dec_this.unwrap_or_else(|| Argument::Expr(build::undefined(self.ast)));
                elems.push(Some(this_arg));
                elems.push(Some(dec));
            }
            elems
        } else {
            preserved.into_iter().map(|(_, dec)| Some(dec)).collect()
        };

        (decorators, has_this)
    }

    fn method_kind_code(
        is_static: bool,
        kind: MethodDefinitionKind,
        decorators_have_this: bool,
    ) -> i32 {
        let base = match kind {
            MethodDefinitionKind::Method => 2,
            MethodDefinitionKind::Get => 3,
            MethodDefinitionKind::Set => 4,
            MethodDefinitionKind::Constructor => unreachable!(),
        };
        let base = if is_static { base | 8 } else { base };
        if decorators_have_this { base | 16 } else { base }
    }

    fn field_kind_code(is_static: bool, decorators_have_this: bool) -> i32 {
        let base = if is_static { 8 } else { 0 };
        if decorators_have_this { base | 16 } else { base }
    }

    fn accessor_kind_code(is_static: bool, decorators_have_this: bool) -> i32 {
        let base = if is_static { 9 } else { 1 };
        if decorators_have_this { base | 16 } else { base }
    }

    fn int_expr(&mut self, value: i32) -> Expr {
        self.ast.expr_numeric_literal(DUMMY_SP, value as f64, None)
    }

    fn instance_brand_arg(&mut self) -> Option<Expr> {
        let brand = self.state.instance_brand.clone()?;
        let arg_binding = self.ctx.resolver.private_binding(self.ast, "o");
        let brand_key = self.ast.add_utf8(&brand);
        let left = Expr::PrivateIdentifier(self.ast.private_identifier(DUMMY_SP, brand_key));
        let right = self.ctx.resolver.reference(self.ast, arg_binding);
        let body_expr = self
            .ast
            .expr_binary_expression(DUMMY_SP, left, right, BinaryOperator::In);
        Some(build::arrow(
            self.ast,
            [arg_binding],
            ArrowFunctionBody::Expr(body_expr),
        ))
    }

    fn field_init_call_expr(
        &mut self,
        init: BindingIdentifier,
        is_static: bool,
        value: Option<Expr>,
    ) -> Expr {
        let callee = self.ctx.resolver.reference(self.ast, init);
        let args: Vec<Expr> = if is_static {
            value.into_iter().collect()
        } else {
            let this = self.ast.expr_this_expression(DUMMY_SP);
            once(this).chain(value).collect()
        };
        build::call(self.ast, callee, args, false)
    }

    fn extra_init_call_expr(&mut self, init_extra: BindingIdentifier, is_static: bool) -> Expr {
        let callee = self.ctx.resolver.reference(self.ast, init_extra);
        if is_static {
            build::call(self.ast, callee, [], false)
        } else {
            let this = self.ast.expr_this_expression(DUMMY_SP);
            build::call(self.ast, callee, [this], false)
        }
    }

    fn wrap_init_with_extra(&mut self, init_expr: Expr, extra_expr: Expr) -> Expr {
        let value_ident = self.ctx.resolver.private_binding(self.ast, "_value");
        let const_stmt = Stmt::Declaration(build::variable(
            self.ast,
            VariableKind::Const,
            value_ident,
            Some(init_expr),
            DUMMY_SP,
        ));
        let extra_stmt = self.ast.stmt_expression_statement(DUMMY_SP, extra_expr);
        let value_ref = self.ctx.resolver.reference(self.ast, value_ident);
        let ret_stmt = self.ast.stmt_return_statement(DUMMY_SP, Some(value_ref));
        let body = build::function_body(self.ast, [const_stmt, extra_stmt, ret_stmt]);
        let arrow = build::arrow(self.ast, [], ArrowFunctionBody::FunctionBody(body));
        build::call(self.ast, arrow, [], false)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L1227>.
    fn consume_inits(&mut self) {
        if self.state.init_proto_args.is_empty()
            && self.state.init_static_args.is_empty()
            && self.state.init_proto.is_none()
            && self.state.init_static.is_none()
            && self.state.class_decorators.is_empty()
        {
            return;
        }

        let mut e_lhs: Vec<Option<BindingPattern>> = Vec::new();
        let lhs_ids: Vec<_> = self
            .state
            .static_non_field_lhs
            .drain(..)
            .chain(self.state.proto_non_field_lhs.drain(..))
            .chain(self.state.static_field_lhs.drain(..))
            .chain(self.state.proto_field_lhs.drain(..))
            .collect();

        for id in lhs_ids {
            e_lhs.push(Some(BindingPattern::BindingIdentifier(id)));
        }

        if let Some(init) = self.state.init_proto {
            self.extra_vars.push(build::declarator(self.ast, init, None));
            e_lhs.push(Some(BindingPattern::BindingIdentifier(init)));
        }

        if let Some(init) = self.state.init_static {
            self.extra_vars.push(build::declarator(self.ast, init, None));
            e_lhs.push(Some(BindingPattern::BindingIdentifier(init)));
        }

        let member_elems: Vec<_> = self
            .state
            .init_static_args
            .drain(..)
            .chain(self.state.init_proto_args.drain(..))
            .collect();
        let member_range = self.ast.add_sub_range(member_elems);
        let member_decs = self.ast.expr_array_expression(DUMMY_SP, member_range);

        let class_elems = mem::take(&mut self.state.class_decorators);
        let class_range = self.ast.add_sub_range(class_elems);
        let class_decs = self.ast.expr_array_expression(DUMMY_SP, class_range);

        let class_decorators_have_this = self.state.class_decorators_have_this;
        let instance_brand = self.instance_brand_arg();
        let super_class = self.state.super_class;

        let mut combined_args = vec![
            self.ast.expr_this_expression(DUMMY_SP),
            class_decs,
            member_decs,
        ];

        if class_decorators_have_this || instance_brand.is_some() || super_class.is_some() {
            combined_args.push(self.int_expr(i32::from(class_decorators_have_this)));
        }

        if instance_brand.is_some() || super_class.is_some() {
            combined_args.push(instance_brand.unwrap_or_else(|| build::undefined(self.ast)));
        }

        if let Some(super_class) = super_class {
            combined_args.push(self.ctx.resolver.reference(self.ast, super_class));
        }

        self.state.class_decorators_have_this = false;
        self.state.instance_brand = None;

        let mut props = Vec::new();
        if !e_lhs.is_empty() {
            let elems = self.ast.add_sub_range(e_lhs);
            let value = BindingPattern::ArrayPattern(self.ast.array_pattern(DUMMY_SP, elems, None));
            let key_name = self.ast.add_utf8("e");
            let key = PropertyKey::IdentifierName(self.ast.identifier_name(DUMMY_SP, key_name));
            props.push(self.ast.binding_property(DUMMY_SP, key, value, false, false));
        }
        if !self.state.class_lhs.is_empty() {
            let class_lhs = mem::take(&mut self.state.class_lhs);
            let elems = self.ast.add_sub_range(class_lhs);
            let value = BindingPattern::ArrayPattern(self.ast.array_pattern(DUMMY_SP, elems, None));
            let key_name = self.ast.add_utf8("c");
            let key = PropertyKey::IdentifierName(self.ast.identifier_name(DUMMY_SP, key_name));
            props.push(self.ast.binding_property(DUMMY_SP, key, value, false, false));
        }

        let props_range = self.ast.add_sub_range(props);
        let obj_pat =
            BindingPattern::ObjectPattern(self.ast.object_pattern(DUMMY_SP, props_range, None));
        let left = build::assignment_target(self.ast, self.ctx.resolver, obj_pat);
        let callee = self.ctx.helper(self.ast, "apply_decs_2311");
        let call = build::call(self.ast, callee, combined_args, false);
        let assign = self
            .ast
            .expr_assignment_expression(DUMMY_SP, left, call, AssignmentOperator::Assign);

        self.state
            .extra_stmts
            .push(self.ast.stmt_expression_statement(DUMMY_SP, assign));

        if let Some(init) = self.state.init_static.take() {
            let callee = self.ctx.resolver.reference(self.ast, init);
            let this = self.ast.expr_this_expression(DUMMY_SP);
            let call = build::call(self.ast, callee, [this], false);
            self.state
                .extra_stmts
                .push(self.ast.stmt_expression_statement(DUMMY_SP, call));
        }
    }

    /// Returns `(key_expr, initializer_binding, optional_replacement_key)`.
    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L1432>.
    fn initializer_name(
        &mut self,
        key: PropertyKey,
        computed: bool,
        prefix: &str,
    ) -> (Expr, BindingIdentifier, Option<(PropertyKey, bool)>) {
        if !computed && let Some(ident) = key.as_identifier_name(self.ast) {
            let sym = self.ast.get_utf8(ident.name(self.ast)).to_owned();
            let init = self
                .ctx
                .resolver
                .private_binding(self.ast, &format!("_{prefix}_{sym}"));
            return (build::string(self.ast, &sym), init, None);
        }
        if computed
            && let Some(expr) = key.as_expr(self.ast)
            && let Some(ident) = expr.as_identifier_reference(self.ast)
        {
            let sym = self.ast.get_utf8(ident.name(self.ast)).to_owned();
            let cloned = build::clone_expr(self.ast, self.ctx.resolver, expr);
            let init = self
                .ctx
                .resolver
                .private_binding(self.ast, &format!("_{prefix}_{sym}"));
            return (cloned, init, None);
        }

        let key_ident = self.ctx.resolver.private_binding(self.ast, "_computedKey");
        self.extra_vars
            .push(build::declarator(self.ast, key_ident, None));

        let raw_expr = prop_key_to_expr_value(self.ast, self.ctx.resolver, key);
        let key_expr = self.call_to_property_key(raw_expr);
        self.maybe_alias_current_class_name_expr(key_expr);
        let key_expr = self.rewrite_implicit_globals_in_expr(key_expr);
        let prefer_class_key = self.should_queue_class_key_init(key_expr, true);

        let lhs = self.ctx.resolver.reference(self.ast, key_ident);
        let assign = build::assign(self.ast, lhs, key_expr);
        self.queue_pre_class_init(assign, prefer_class_key);

        let replacement_key = PropertyKey::Expr(self.ctx.resolver.reference(self.ast, key_ident));
        let key_arg = self.ctx.resolver.reference(self.ast, key_ident);
        let init = self
            .ctx
            .resolver
            .private_binding(self.ast, &format!("_{prefix}_computedKey"));

        (key_arg, init, Some((replacement_key, true)))
    }

    fn ensure_constructor(&mut self, cls: Class) -> MethodDefinition {
        let body = cls.body(self.ast);
        let members: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let mut insert_index = 0;
        for (i, &member) in members.iter().enumerate() {
            match self.ast.class_element_data(member) {
                ClassElementData::MethodDefinition(method)
                    if method.kind(self.ast) == MethodDefinitionKind::Constructor =>
                {
                    return method;
                }
                ClassElementData::TsMethodDefinition(method)
                    if method.kind(self.ast) == MethodDefinitionKind::Constructor =>
                {
                    insert_index = i + 1;
                }
                _ => {}
            }
        }
        let ctor = default_constructor(self.ast, self.ctx.resolver, cls.super_class(self.ast).is_some());
        let mut new_members = members;
        new_members.insert(insert_index, ClassElement::MethodDefinition(ctor));
        let range = self.ast.add_sub_range(new_members);
        body.set_body(self.ast, range);
        ctor
    }

    fn ensure_identity_constructor(&mut self, cls: Class) -> MethodDefinition {
        let body = cls.body(self.ast);
        let members: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let mut insert_index = 0;
        for (i, &member) in members.iter().enumerate() {
            match self.ast.class_element_data(member) {
                ClassElementData::MethodDefinition(method)
                    if method.kind(self.ast) == MethodDefinitionKind::Constructor =>
                {
                    return method;
                }
                ClassElementData::TsMethodDefinition(method)
                    if method.kind(self.ast) == MethodDefinitionKind::Constructor =>
                {
                    insert_index = i + 1;
                }
                _ => {}
            }
        }
        let ctor = identity_constructor(self.ast);
        let mut new_members = members;
        new_members.insert(insert_index, ClassElement::MethodDefinition(ctor));
        let range = self.ast.add_sub_range(new_members);
        body.set_body(self.ast, range);
        ctor
    }

    fn handle_super_class(&mut self, cls: Class) {
        if let Some(super_class) = cls.super_class(self.ast) {
            let super_class = self.rewrite_implicit_globals_in_expr(super_class);
            let alias = build::alias_ident_for(self.ast, super_class, "_super");
            let id = self.ctx.resolver.private_binding(self.ast, &alias);
            self.extra_vars.push(build::declarator(self.ast, id, None));

            let lhs = self.ctx.resolver.reference(self.ast, id);
            self.pre_class_inits
                .push(build::assign(self.ast, lhs, super_class));
            let super_ref = self.ctx.resolver.reference(self.ast, id);
            cls.set_super_class(self.ast, Some(super_ref));
            self.state.super_class = Some(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L1606>.
    fn handle_class_expr(&mut self, cls: Class) -> BindingIdentifier {
        let init_class = self.ctx.resolver.private_binding(self.ast, "_initClass");
        self.extra_vars
            .push(build::declarator(self.ast, init_class, None));

        let ident = cls.id(self.ast);
        let new_class_name = match ident {
            Some(id) => {
                let name = format!("_{}", self.ast.get_utf8(id.name(self.ast)));
                self.ctx.resolver.private_binding(self.ast, &name)
            }
            None => self.ctx.resolver.private_binding(self.ast, "_class"),
        };

        if let Some(ident) = ident {
            let from = self.ctx.resolver.symbol_of(ident.node_id());
            let to = self.ctx.resolver.symbol_of(new_class_name.node_id());
            replace_symbol(self.ast, self.ctx.resolver, cls.body(self.ast), from, to);
        }

        self.state
            .class_lhs
            .push(Some(BindingPattern::BindingIdentifier(new_class_name)));
        self.state
            .class_lhs
            .push(Some(BindingPattern::BindingIdentifier(init_class)));

        self.extra_vars
            .push(build::declarator(self.ast, new_class_name, None));

        let dec_exprs: Vec<_> = cls
            .decorators(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
            .collect();
        cls.set_decorators(self.ast, SubRange::empty());

        let (decorators, decorators_have_this) =
            self.preserve_side_effect_of_decorators_for_class(dec_exprs);
        self.state.class_decorators.extend(decorators);
        self.state.class_decorators_have_this |= decorators_have_this;
        self.handle_super_class(cls);

        let callee = self.ctx.resolver.reference(self.ast, init_class);
        let call = build::call(self.ast, callee, [], false);
        let call_stmt = self.ast.stmt_expression_statement(DUMMY_SP, call);
        let block_body = self.ast.add_sub_range([call_stmt]);
        let static_block = ClassElement::StaticBlock(self.ast.static_block(DUMMY_SP, block_body));

        let body = cls.body(self.ast);
        let mut members: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        members.push(static_block);
        let range = self.ast.add_sub_range(members);
        body.set_body(self.ast, range);

        new_class_name
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L1671>.
    fn handle_class_decl(&mut self, cls: Class) -> Stmt {
        let old_state = mem::take(&mut self.state);
        let orig_ident = cls.id(self.ast).expect("class declaration has id");
        let original_class_sym = self.ctx.resolver.symbol_of(orig_ident.node_id());
        let orig_name_str = self.ast.get_utf8(orig_ident.name(self.ast)).to_owned();

        let dec_exprs: Vec<_> = cls
            .decorators(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
            .collect();
        cls.set_decorators(self.ast, SubRange::empty());

        let (decorators, decorators_have_this) =
            self.preserve_side_effect_of_decorators_for_class(dec_exprs);

        let init_class = self.ctx.resolver.private_binding(self.ast, "_initClass");
        self.extra_vars
            .push(build::declarator(self.ast, init_class, None));

        let preserved_class_name = self.ctx.resolver.private_binding(self.ast, &orig_name_str);
        let new_class_name = self
            .ctx
            .resolver
            .private_binding(self.ast, &format!("_{orig_name_str}"));
        let old_class_name = self.current_class_name.replace(orig_ident);
        let member_class_ref_alias = self
            .ctx
            .resolver
            .private_binding(self.ast, &format!("_{orig_name_str}_member"));
        let old_member_class_ref_alias = self
            .class_member_class_ref_alias
            .replace(member_class_ref_alias);

        self.extra_lets
            .push(build::declarator(self.ast, new_class_name, None));
        self.extra_lets
            .push(build::declarator(self.ast, member_class_ref_alias, None));

        let new_class_sym = self.ctx.resolver.symbol_of(new_class_name.node_id());
        self.rename_map.insert(original_class_sym, new_class_sym);

        self.state
            .class_lhs
            .push(Some(BindingPattern::BindingIdentifier(new_class_name)));
        self.state
            .class_lhs
            .push(Some(BindingPattern::BindingIdentifier(init_class)));

        self.state.class_decorators.extend(decorators);
        self.state.class_decorators_have_this |= decorators_have_this;
        self.handle_super_class(cls);

        let class_body = cls.body(self.ast);
        let mut body: Vec<_> = class_body
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        class_body.set_body(self.ast, SubRange::empty());

        let has_static_member = body.iter().any(|&m| match self.ast.class_element_data(m) {
            ClassElementData::MethodDefinition(m) => m.r#static(self.ast),
            ClassElementData::PropertyDefinition(p) => p.r#static(self.ast),
            ClassElementData::StaticBlock(_) => true,
            _ => false,
        });

        if has_static_member {
            let mut last_static_block: Option<Vec<Stmt>> = None;

            self.process_decorators_of_class_members(&body);

            if let Some(member) = self.take_computed_key_init_member() {
                body.insert(0, member);
            }

            // Move static blocks into property initializers.
            for &m in &body {
                match self.ast.class_element_data(m) {
                    ClassElementData::PropertyDefinition(prop)
                        if !prop.accessor(self.ast) && prop.r#static(self.ast) =>
                    {
                        if let Some(stmts) = last_static_block.take() {
                            let fn_body = build::function_body(self.ast, stmts);
                            let arrow =
                                build::arrow(self.ast, [], ArrowFunctionBody::FunctionBody(fn_body));
                            let static_iife = build::call(self.ast, arrow, [], false);
                            let field_value = prop
                                .value(self.ast)
                                .unwrap_or_else(|| build::undefined(self.ast));
                            let seq = build::sequence(self.ast, vec![static_iife, field_value]);
                            prop.set_value(self.ast, Some(seq));
                        }
                    }
                    ClassElementData::StaticBlock(s) => {
                        let stmts: Vec<_> = s
                            .body(self.ast)
                            .iter()
                            .map(|slot| self.ast.get_node_in_sub_range(slot))
                            .collect();
                        s.set_body(self.ast, SubRange::empty());
                        match &mut last_static_block {
                            None => last_static_block = Some(stmts),
                            Some(v) => v.extend(stmts),
                        }
                    }
                    _ => {}
                }
            }

            // Drop static blocks.
            body.retain(|&m| !m.is_static_block(self.ast));

            for &m in &body {
                match self.ast.class_element_data(m) {
                    ClassElementData::PropertyDefinition(_)
                    | ClassElementData::MethodDefinition(_) => {
                        replace_symbol(
                            self.ast,
                            self.ctx.resolver,
                            m,
                            original_class_sym,
                            new_class_sym,
                        );
                    }
                    _ => {}
                }
            }

            let inner_body_range = self.ast.add_sub_range(body);
            let inner_body = self.ast.class_body(DUMMY_SP, inner_body_range);
            let inner_super = cls.super_class(self.ast);
            cls.set_super_class(self.ast, None);

            let inner_id = self.ctx.resolver.binding(self.ast, original_class_sym);
            let inner_class = self.ast.class(
                DUMMY_SP,
                ClassType::ClassExpression,
                SubRange::empty(),
                Some(inner_id),
                None,
                inner_super,
                None,
                SubRange::empty(),
                inner_body,
                false,
                false,
            );

            self.visit_class_body_and_finalize(inner_class);

            let inner_body = inner_class.body(self.ast);
            let instance_private_names = self.collect_instance_private_names(inner_body);
            let mut static_closure_assignments = Vec::new();
            let mut moved_members = Vec::new();
            let mut kept_inner_members = Vec::new();

            let inner_elems: Vec<_> = inner_body
                .body(self.ast)
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot))
                .collect();

            for m in inner_elems {
                let mut should_move = false;
                match self.ast.class_element_data(m) {
                    ClassElementData::PropertyDefinition(p) if p.r#static(self.ast) => {
                        let new_val = self.maybe_wrap_static_initializer_with_closure(
                            p.value(self.ast),
                            new_class_name,
                            &instance_private_names,
                            &mut static_closure_assignments,
                        );
                        p.set_value(self.ast, new_val);
                        p.set_static(self.ast, false);
                        should_move = true;
                    }
                    ClassElementData::MethodDefinition(p)
                        if p.r#static(self.ast)
                            && build::private_name(self.ast, p.key(self.ast)).is_some() =>
                    {
                        let func = p.value(self.ast);
                        let needs_rewrite = self
                            .function_needs_moved_static_rewrite(func, &instance_private_names);
                        if needs_rewrite {
                            let delegated = clone_function(self.ast, self.ctx.resolver, func);
                            rewrite_super_in_moved_static_member(
                                self.ast,
                                self.ctx,
                                delegated,
                                new_class_name,
                            );
                            let priv_name =
                                build::private_name(self.ast, p.key(self.ast)).unwrap();
                            let delegate = self.memoize_static_closure(
                                &priv_name,
                                Expr::Function(delegated),
                                &mut static_closure_assignments,
                            );

                            let orig_params = func.params(self.ast);
                            let item_count = orig_params.items(self.ast).len();
                            let has_rest = orig_params.rest(self.ast).is_some();

                            let mut new_param_items = Vec::with_capacity(item_count);
                            for idx in 0..item_count {
                                let binding = self
                                    .ctx
                                    .resolver
                                    .private_binding(self.ast, &format!("_{idx}"));
                                new_param_items.push(FormalParameterItem::FormalParameter(
                                    self.ast.formal_parameter(
                                        DUMMY_SP,
                                        SubRange::empty(),
                                        FormalParameterPattern::BindingIdentifier(binding),
                                        None,
                                        None,
                                        false,
                                    ),
                                ));
                            }
                            let (rest_param, forwarded_args) = if has_rest {
                                let arg = self.ctx.resolver.private_binding(self.ast, "arg");
                                let rest = self.ast.formal_parameter_rest(
                                    DUMMY_SP,
                                    SubRange::empty(),
                                    FormalParameterPattern::BindingIdentifier(arg),
                                    None,
                                );
                                (Some(rest), self.ctx.resolver.reference(self.ast, arg))
                            } else {
                                (
                                    None,
                                    self.ctx.resolver.marked_reference(self.ast, "arguments", true),
                                )
                            };
                            let items_range = self.ast.add_sub_range(new_param_items);
                            let new_params = self.ast.formal_parameters(
                                DUMMY_SP,
                                items_range,
                                rest_param,
                                FormalParameterKind::FormalParameters,
                            );
                            func.set_params(self.ast, new_params);

                            let delegate_ref = self.ctx.resolver.reference(self.ast, delegate);
                            let apply_callee = build::member(self.ast, delegate_ref, "apply");
                            let this = self.ast.expr_this_expression(DUMMY_SP);
                            let apply_call =
                                build::call(self.ast, apply_callee, [this, forwarded_args], false);
                            let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(apply_call));
                            let fn_body = build::function_body(self.ast, [ret]);
                            func.set_body(self.ast, fn_body);
                        }
                        p.set_static(self.ast, false);
                        should_move = true;
                    }
                    _ => {}
                }

                if should_move {
                    moved_members.push(m);
                } else {
                    kept_inner_members.push(m);
                }
            }

            let static_call = last_static_block.map(|last| {
                if self.stmts_need_moved_static_rewrite(&last, &instance_private_names) {
                    let fn_expr = build::function_with(self.ast, [], last);
                    let closure_fn = fn_expr.as_function(self.ast).unwrap();
                    rewrite_super_in_moved_static_member(
                        self.ast,
                        self.ctx,
                        closure_fn,
                        new_class_name,
                    );
                    let closure = self.memoize_static_closure(
                        "staticBlock",
                        fn_expr,
                        &mut static_closure_assignments,
                    );
                    self.call_closure_with_this(closure)
                } else {
                    let fn_body = build::function_body(self.ast, last);
                    let arrow =
                        build::arrow(self.ast, [], ArrowFunctionBody::FunctionBody(fn_body));
                    build::call(self.ast, arrow, [], false)
                }
            });

            if !static_closure_assignments.is_empty() {
                let stmts_range = self.ast.add_sub_range(static_closure_assignments);
                kept_inner_members.push(ClassElement::StaticBlock(
                    self.ast.static_block(DUMMY_SP, stmts_range),
                ));
            }

            let kept_range = self.ast.add_sub_range(kept_inner_members);
            inner_body.set_body(self.ast, kept_range);

            let inner_class_prop = ClassElement::PropertyDefinition(self.ast.property_definition(
                DUMMY_SP,
                SubRange::empty(),
                PropertyKey::Expr(Expr::Class(inner_class)),
                None,
                None,
                true,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                Accessibility::None,
            ));

            let mut outer_members = Vec::with_capacity(moved_members.len() + 1);
            outer_members.push(inner_class_prop);
            outer_members.extend(moved_members);
            let outer_range = self.ast.add_sub_range(outer_members);
            class_body.set_body(self.ast, outer_range);

            let preserved_sym = self.ctx.resolver.symbol_of(preserved_class_name.node_id());
            replace_symbol(self.ast, self.ctx.resolver, cls, original_class_sym, preserved_sym);

            {
                let ctor = self.ensure_identity_constructor(cls);
                let super_callee = self.ast.expr_super(DUMMY_SP);
                let orig_ref = self.ctx.resolver.symbol_reference(self.ast, original_class_sym);
                let super_call = build::call(self.ast, super_callee, [orig_ref], false);

                let init_callee = self.ctx.resolver.reference(self.ast, init_class);
                let init_class_call = build::call(self.ast, init_callee, [], false);

                let alias_lhs = self.ctx.resolver.reference(self.ast, member_class_ref_alias);
                let new_cls_ref = self.ctx.resolver.reference(self.ast, new_class_name);
                let alias_assign = build::assign(self.ast, alias_lhs, new_cls_ref);

                let seq = build::sequence(
                    self.ast,
                    once(super_call)
                        .chain(static_call)
                        .chain(once(init_class_call))
                        .chain(once(alias_assign))
                        .collect(),
                );
                let stmt = self.ast.stmt_expression_statement(DUMMY_SP, seq);
                let ctor_body = ctor.value(self.ast).body(self.ast);
                let new_body = build::prepend_stmts(self.ast, [stmt], ctor_body.body(self.ast));
                ctor_body.set_body(self.ast, new_body);
            }

            let identity = self.ctx.helper(self.ast, "identity");
            let wrapper_class = self.ast.class(
                DUMMY_SP,
                ClassType::ClassExpression,
                SubRange::empty(),
                None,
                None,
                Some(identity),
                None,
                SubRange::empty(),
                class_body,
                false,
                false,
            );

            self.state = old_state;
            self.current_class_name = old_class_name;
            self.class_member_class_ref_alias = old_member_class_ref_alias;

            let new_expr = self.ast.expr_new_expression(
                DUMMY_SP,
                Expr::Class(wrapper_class),
                None,
                SubRange::empty(),
            );
            return self.ast.stmt_expression_statement(DUMMY_SP, new_expr);
        }

        // No static members.
        let mut ctors = Vec::new();
        let mut rest = Vec::new();
        for m in body {
            if let Some(method) = m.as_method_definition(self.ast)
                && method.kind(self.ast) == MethodDefinitionKind::Constructor
            {
                ctors.push(m);
            } else {
                rest.push(m);
            }
        }
        ctors.extend(rest);
        let body_range = self.ast.add_sub_range(ctors);
        class_body.set_body(self.ast, body_range);

        let preserved_sym = self.ctx.resolver.symbol_of(preserved_class_name.node_id());
        cls.set_id(self.ast, Some(preserved_class_name));
        replace_symbol(self.ast, self.ctx.resolver, cls, original_class_sym, preserved_sym);

        self.visit_class_body_and_finalize(cls);

        let init_callee = self.ctx.resolver.reference(self.ast, init_class);
        let init_call = build::call(self.ast, init_callee, [], false);
        let init_stmt = self.ast.stmt_expression_statement(DUMMY_SP, init_call);

        let alias_lhs = self.ctx.resolver.reference(self.ast, member_class_ref_alias);
        let new_cls_ref = self.ctx.resolver.reference(self.ast, new_class_name);
        let alias_assign = build::assign(self.ast, alias_lhs, new_cls_ref);
        let alias_stmt = self.ast.stmt_expression_statement(DUMMY_SP, alias_assign);

        let sb_stmts = self.ast.add_sub_range([init_stmt, alias_stmt]);
        let static_block = ClassElement::StaticBlock(self.ast.static_block(DUMMY_SP, sb_stmts));

        let mut final_members: Vec<_> = class_body
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        final_members.push(static_block);
        let final_range = self.ast.add_sub_range(final_members);
        class_body.set_body(self.ast, final_range);

        self.state = old_state;
        self.current_class_name = old_class_name;
        self.class_member_class_ref_alias = old_member_class_ref_alias;

        Stmt::Declaration(Decl::Class(cls))
    }

    fn process_decorators(&mut self, decorators: SubRange<Decorator>) {
        for slot in decorators.iter() {
            let dec = self.ast.get_node_in_sub_range(slot);
            let expr = dec.expression(self.ast);
            if let Some(ident) = expr.as_identifier_reference(self.ast)
                && let Some(sym) = self.ctx.resolver.symbol(ident.node_id())
            {
                self.decorator_binding_ids_for_implicit_global_rewrite
                    .insert(sym);
            }
            self.maybe_alias_current_class_name_expr(expr);

            let new_expr = if expr.is_member_expression(self.ast)
                || (self.expr_uses_private_name(expr) && !self.expr_uses_yield_or_await(expr))
            {
                expr
            } else {
                self.preserve_side_effect_of_decorator(expr)
            };
            dec.set_expression(self.ast, new_expr);
        }
    }

    fn process_prop_name(&mut self, key: PropertyKey, computed: bool) -> Option<(PropertyKey, bool)> {
        if !computed && key.is_identifier_name(self.ast) {
            return None;
        }
        if computed
            && key
                .as_expr(self.ast)
                .is_some_and(|e| e.is_identifier_reference(self.ast))
        {
            return None;
        }

        let ident = self.ctx.resolver.private_binding(self.ast, "_computedKey");
        self.extra_vars
            .push(build::declarator(self.ast, ident, None));

        let raw_expr = prop_key_to_expr_value(self.ast, self.ctx.resolver, key);
        let key_expr = self.call_to_property_key(raw_expr);
        self.maybe_alias_current_class_name_expr(key_expr);
        let key_expr = self.rewrite_implicit_globals_in_expr(key_expr);
        let prefer_class_key = self.should_queue_class_key_init(key_expr, true);

        let lhs = self.ctx.resolver.reference(self.ast, ident);
        let assign = build::assign(self.ast, lhs, key_expr);
        self.queue_pre_class_init(assign, prefer_class_key);

        let key_ref = self.ctx.resolver.reference(self.ast, ident);
        Some((PropertyKey::Expr(key_ref), true))
    }

    fn process_decorators_of_class_members(&mut self, members: &[ClassElement]) {
        for &m in members {
            match self.ast.class_element_data(m) {
                ClassElementData::MethodDefinition(method) => {
                    if method.kind(self.ast) == MethodDefinitionKind::Constructor {
                        continue;
                    }
                    self.process_decorators(method.decorators(self.ast));
                    if build::private_name(self.ast, method.key(self.ast)).is_none()
                        && let Some((new_key, new_computed)) =
                            self.process_prop_name(method.key(self.ast), method.computed(self.ast))
                    {
                        method.set_key(self.ast, new_key);
                        method.set_computed(self.ast, new_computed);
                    }
                }
                ClassElementData::PropertyDefinition(prop) => {
                    if prop.declare(self.ast) && !prop.accessor(self.ast) {
                        continue;
                    }
                    self.process_decorators(prop.decorators(self.ast));
                    if build::private_name(self.ast, prop.key(self.ast)).is_none()
                        && let Some((new_key, new_computed)) =
                            self.process_prop_name(prop.key(self.ast), prop.computed(self.ast))
                    {
                        prop.set_key(self.ast, new_key);
                        prop.set_computed(self.ast, new_computed);
                    }
                }
                _ => {}
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L2310>.
    fn visit_class_body_and_finalize(&mut self, cls: Class) {
        let old_stmts = mem::take(&mut self.state.extra_stmts);
        let old_pending_instance = mem::take(&mut self.state.pending_instance_field_extra_inits);
        let old_computed_key_inits = mem::take(&mut self.state.computed_key_inits);

        if let Some(super_class) = cls.super_class(self.ast)
            && let Some(replacement) = super_class.visit_mut_with(self)
        {
            cls.set_super_class(self.ast, Some(replacement));
        }

        let body = cls.body(self.ast);
        self.visit_class_members(body);

        if let Some(init_proto) = self.state.init_proto {
            let callee = self.ctx.resolver.reference(self.ast, init_proto);
            let this = self.ast.expr_this_expression(DUMMY_SP);
            let init_proto_expr = build::call(self.ast, callee, [this], false);

            let mut proto_inited = false;
            let elems = body.body(self.ast);
            for slot in elems.iter() {
                let elem = self.ast.get_node_in_sub_range(slot);
                if let Some(prop) = elem.as_property_definition(self.ast)
                    && !prop.r#static(self.ast)
                    && let Some(value) = prop.value(self.ast)
                {
                    let call_clone = build::clone_expr(self.ast, self.ctx.resolver, init_proto_expr);
                    let seq = build::sequence(self.ast, vec![call_clone, value]);
                    prop.set_value(self.ast, Some(seq));
                    proto_inited = true;
                    break;
                }
            }

            if !proto_inited {
                let ctor = self.ensure_constructor(cls);
                let ctor_body = ctor.value(self.ast).body(self.ast);
                inject_after_super(self.ast, self.ctx.resolver, ctor_body, vec![init_proto_expr]);
            }
        }

        if !self.state.pending_instance_field_extra_inits.is_empty() {
            let pending_instance = mem::take(&mut self.state.pending_instance_field_extra_inits);
            let is_derived = cls.super_class(self.ast).is_some();
            let ctor = self.ensure_constructor(cls);
            let ctor_body = ctor.value(self.ast).body(self.ast);
            if is_derived {
                inject_after_super(self.ast, self.ctx.resolver, ctor_body, pending_instance);
            } else {
                let stmts: Vec<_> = pending_instance
                    .into_iter()
                    .map(|expr| self.ast.stmt_expression_statement(DUMMY_SP, expr))
                    .collect();
                let new_body = build::prepend_stmts(self.ast, stmts, ctor_body.body(self.ast));
                ctor_body.set_body(self.ast, new_body);
            }
        }

        self.consume_inits();

        let computed_key_init_member = self.take_computed_key_init_member();
        let has_computed_key_init = computed_key_init_member.is_some();
        let has_extra_stmts = !self.state.extra_stmts.is_empty();

        if has_computed_key_init || has_extra_stmts {
            let mut members: Vec<_> = body
                .body(self.ast)
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot))
                .collect();
            let mut insert_at = 0;
            if let Some(member) = computed_key_init_member {
                members.insert(insert_at, member);
                insert_at += 1;
            }
            if has_extra_stmts {
                let stmts = mem::take(&mut self.state.extra_stmts);
                let stmts_range = self.ast.add_sub_range(stmts);
                let sb = ClassElement::StaticBlock(self.ast.static_block(DUMMY_SP, stmts_range));
                members.insert(insert_at, sb);
            }
            let range = self.ast.add_sub_range(members);
            body.set_body(self.ast, range);
        }

        self.state.init_proto = None;
        self.state.extra_stmts = old_stmts;
        self.state.pending_instance_field_extra_inits = old_pending_instance;
        self.state.computed_key_inits = old_computed_key_inits;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L2614>.
    fn visit_class_members(&mut self, body: ClassBody) {
        let members: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();

        self.process_decorators_of_class_members(&members);

        let mut new_members = Vec::with_capacity(members.len());

        for m in members {
            match self.ast.class_element_data(m) {
                ClassElementData::PropertyDefinition(accessor) if accessor.accessor(self.ast) => {
                    self.lower_auto_accessor(accessor, &mut new_members);
                    continue;
                }
                ClassElementData::MethodDefinition(method) => {
                    if method.kind(self.ast) != MethodDefinitionKind::Constructor {
                        self.visit_method_definition(method);
                        new_members.push(m);
                        continue;
                    }
                }
                ClassElementData::PropertyDefinition(prop) => {
                    self.visit_property_definition(prop);
                    new_members.push(m);
                    continue;
                }
                _ => {}
            }
            new_members.push(m);
        }

        // Visit constructors, static blocks, etc. (matching upstream second loop).
        for &m in &new_members {
            match self.ast.class_element_data(m) {
                ClassElementData::MethodDefinition(method)
                    if method.kind(self.ast) == MethodDefinitionKind::Constructor =>
                {
                    if m.span(self.ast) != DUMMY_SP {
                        method.visit_mut_children_with(self);
                    }
                }
                ClassElementData::StaticBlock(sb) if m.span(self.ast) != DUMMY_SP => {
                    sb.visit_mut_children_with(self);
                }
                _ => {}
            }
        }

        let range = self.ast.add_sub_range(new_members);
        body.set_body(self.ast, range);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L2621>.
    fn lower_auto_accessor(&mut self, accessor: PropertyDefinition, out: &mut Vec<ClassElement>) {
        if let Some(value) = accessor.value(self.ast)
            && let Some(replacement) = value.visit_mut_with(self)
        {
            accessor.set_value(self.ast, Some(replacement));
        }

        let is_static = accessor.r#static(self.ast);
        let priv_key_name = build::private_name(self.ast, accessor.key(self.ast));
        let is_private = priv_key_name.is_some();

        let (name_expr, init, init_extra, field_name_like, private_slot_name) =
            if let Some(ref k_name) = priv_key_name {
                let name = build::string(self.ast, k_name);
                let init = self
                    .ctx
                    .resolver
                    .private_binding(self.ast, &format!("_init_{k_name}"));
                let init_sym = self.ast.get_utf8(init.name(self.ast)).to_owned();
                let init_extra = self
                    .ctx
                    .resolver
                    .private_binding(self.ast, &format!("_init_extra_{init_sym}"));
                let field_name_like = format!("__{k_name}");
                self.state.private_id_index += 1;
                let slot_name = format!("__{k_name}_{}", self.state.private_id_index);
                (name, init, init_extra, field_name_like, slot_name)
            } else {
                let (name, init, replacement_key) = self.initializer_name(
                    accessor.key(self.ast),
                    accessor.computed(self.ast),
                    "init",
                );
                if let Some((new_key, new_computed)) = replacement_key {
                    accessor.set_key(self.ast, new_key);
                    accessor.set_computed(self.ast, new_computed);
                }
                let init_sym = self.ast.get_utf8(init.name(self.ast)).to_owned();
                let init_extra = self
                    .ctx
                    .resolver
                    .private_binding(self.ast, &format!("_init_extra_{init_sym}"));
                let field_name_like = format!("__{init_sym}").replacen("init", "private", 1);
                self.state.private_id_index += 1;
                let slot_name = format!("{field_name_like}_{}", self.state.private_id_index);
                (name, init, init_extra, field_name_like, slot_name)
            };

        let dec_exprs: Vec<_> = accessor
            .decorators(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
            .collect();
        accessor.set_decorators(self.ast, SubRange::empty());

        let orig_value = accessor.value(self.ast);
        let field_value = if dec_exprs.is_empty() {
            self.prepend_pending_instance_field_extras(orig_value, is_static)
        } else {
            let init_call = self.field_init_call_expr(init, is_static, orig_value);
            let init_call = self
                .prepend_pending_instance_field_extras(Some(init_call), is_static)
                .unwrap();
            let extra_call = self.extra_init_call_expr(init_extra, is_static);
            Some(self.wrap_init_with_extra(init_call, extra_call))
        };

        let slot_key = make_private_key(self.ast, &private_slot_name);
        let private_field = self.ast.property_definition(
            DUMMY_SP,
            SubRange::empty(),
            slot_key,
            None,
            field_value,
            false,
            is_static,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            Accessibility::None,
        );

        let make_storage_obj = |ast: &mut Ast<'_>, resolver: &mut Resolver, cls: Option<BindingIdentifier>| -> Expr {
            if is_static && !is_private {
                cls.map(|c| resolver.reference(ast, c))
                    .unwrap_or_else(|| ast.expr_this_expression(DUMMY_SP))
            } else {
                ast.expr_this_expression(DUMMY_SP)
            }
        };

        let mut getter_function = {
            let obj = make_storage_obj(self.ast, self.ctx.resolver, self.current_class_name);
            let key = make_private_key(self.ast, &private_slot_name);
            let member = self
                .ast
                .expr_member_expression(DUMMY_SP, obj, key, false, false);
            let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(member));
            build::function_with(self.ast, [], [ret])
                .as_function(self.ast)
                .unwrap()
        };

        let mut setter_function = {
            let param = self.ctx.resolver.private_binding(self.ast, "_v");
            let obj = make_storage_obj(self.ast, self.ctx.resolver, self.current_class_name);
            let key = make_private_key(self.ast, &private_slot_name);
            let member = self
                .ast
                .expr_member_expression(DUMMY_SP, obj, key, false, false);
            let rhs = self.ctx.resolver.reference(self.ast, param);
            let assign = build::assign(self.ast, member, rhs);
            let stmt = self.ast.stmt_expression_statement(DUMMY_SP, assign);
            build::function_with(self.ast, [param], [stmt])
                .as_function(self.ast)
                .unwrap()
        };

        if !dec_exprs.is_empty() {
            let (dec, decorators_have_this) = self.merge_decorators_for_member(dec_exprs);

            if !is_static && let Some(ref k_name) = priv_key_name {
                self.state
                    .instance_brand
                    .get_or_insert_with(|| k_name.clone());
            }

            self.extra_vars.push(build::declarator(self.ast, init, None));
            self.extra_vars
                .push(build::declarator(self.ast, init_extra, None));

            let (getter_var, setter_var) = if is_private {
                (
                    Some(
                        self.ctx
                            .resolver
                            .private_binding(self.ast, &format!("_get_{field_name_like}")),
                    ),
                    Some(
                        self.ctx
                            .resolver
                            .private_binding(self.ast, &format!("_set_{field_name_like}")),
                    ),
                )
            } else {
                (None, None)
            };

            let kind_expr = self.int_expr(Self::accessor_kind_code(is_static, decorators_have_this));

            let dec_elems = if is_private {
                let receiver_g = self.ctx.resolver.private_binding(self.ast, "_this");
                let recv_g_ref = self.ctx.resolver.reference(self.ast, receiver_g);
                let slot_g = make_private_key(self.ast, &private_slot_name);
                let mem_g = self
                    .ast
                    .expr_member_expression(DUMMY_SP, recv_g_ref, slot_g, false, false);
                let ret_g = self.ast.stmt_return_statement(DUMMY_SP, Some(mem_g));
                let private_slot_getter = build::function_with(self.ast, [receiver_g], [ret_g]);

                let receiver_s = self.ctx.resolver.private_binding(self.ast, "_this");
                let setter_arg = self.ctx.resolver.private_binding(self.ast, "_v");
                let recv_s_ref = self.ctx.resolver.reference(self.ast, receiver_s);
                let slot_s = make_private_key(self.ast, &private_slot_name);
                let mem_s = self
                    .ast
                    .expr_member_expression(DUMMY_SP, recv_s_ref, slot_s, false, false);
                let val_s = self.ctx.resolver.reference(self.ast, setter_arg);
                let assign_s = build::assign(self.ast, mem_s, val_s);
                let stmt_s = self.ast.stmt_expression_statement(DUMMY_SP, assign_s);
                let private_slot_setter =
                    build::function_with(self.ast, [receiver_s, setter_arg], [stmt_s]);

                let g_var = getter_var.unwrap();
                let s_var = setter_var.unwrap();
                self.extra_vars
                    .push(build::declarator(self.ast, g_var, None));
                self.extra_vars
                    .push(build::declarator(self.ast, s_var, None));

                // Replace getter/setter methods to delegate to `_get_*` / `_set_*`.
                let g_callee = self.ctx.resolver.reference(self.ast, g_var);
                let g_args: Vec<Expr> = if is_static {
                    Vec::new()
                } else {
                    vec![self.ast.expr_this_expression(DUMMY_SP)]
                };
                let g_call = build::call(self.ast, g_callee, g_args, false);
                let g_ret = self.ast.stmt_return_statement(DUMMY_SP, Some(g_call));
                getter_function = build::function_with(self.ast, [], [g_ret])
                    .as_function(self.ast)
                    .unwrap();

                let s_param = self.ctx.resolver.private_binding(self.ast, "_v");
                let s_callee = self.ctx.resolver.reference(self.ast, s_var);
                let s_param_ref = self.ctx.resolver.reference(self.ast, s_param);
                let s_args: Vec<Expr> = if is_static {
                    vec![s_param_ref]
                } else {
                    vec![self.ast.expr_this_expression(DUMMY_SP), s_param_ref]
                };
                let s_call = build::call(self.ast, s_callee, s_args, false);
                let s_stmt = self.ast.stmt_expression_statement(DUMMY_SP, s_call);
                setter_function = build::function_with(self.ast, [s_param], [s_stmt])
                    .as_function(self.ast)
                    .unwrap();

                vec![
                    dec,
                    Some(Argument::Expr(kind_expr)),
                    Some(Argument::Expr(name_expr)),
                    Some(Argument::Expr(private_slot_getter)),
                    Some(Argument::Expr(private_slot_setter)),
                ]
            } else {
                vec![
                    dec,
                    Some(Argument::Expr(kind_expr)),
                    Some(Argument::Expr(name_expr)),
                ]
            };

            let dec_range = self.ast.add_sub_range(dec_elems);
            let initialize_init = Argument::Expr(self.ast.expr_array_expression(DUMMY_SP, dec_range));

            self.push_lhs(init, is_static, false);
            for ident in getter_var.into_iter().chain(setter_var) {
                self.push_lhs(ident, is_static, false);
            }
            self.push_lhs(init_extra, is_static, false);

            if is_static {
                self.state.init_static_args.push(Some(initialize_init));
            } else {
                self.state.init_proto_args.push(Some(initialize_init));
            }
        }

        let computed = !is_private && accessor.computed(self.ast);
        let orig_key = accessor.key(self.ast);
        let getter_key = clone_property_key(self.ast, self.ctx.resolver, orig_key);
        let setter_key = clone_property_key(self.ast, self.ctx.resolver, orig_key);

        let getter = self.ast.method_definition(
            DUMMY_SP,
            SubRange::empty(),
            getter_key,
            getter_function,
            MethodDefinitionKind::Get,
            computed,
            is_static,
            false,
            false,
            false,
            Accessibility::None,
        );
        let setter = self.ast.method_definition(
            DUMMY_SP,
            SubRange::empty(),
            setter_key,
            setter_function,
            MethodDefinitionKind::Set,
            computed,
            is_static,
            false,
            false,
            false,
            Accessibility::None,
        );

        out.push(ClassElement::PropertyDefinition(private_field));
        out.push(ClassElement::MethodDefinition(getter));
        out.push(ClassElement::MethodDefinition(setter));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L2479>.
    fn visit_method_definition(&mut self, method: MethodDefinition) {
        method.visit_mut_children_with(self);

        let decs = method.decorators(self.ast);
        if decs.is_empty() {
            return;
        }
        let dec_exprs: Vec<_> = decs
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
            .collect();
        method.set_decorators(self.ast, SubRange::empty());

        let is_static = method.r#static(self.ast);
        let kind = method.kind(self.ast);

        if let Some(priv_name) = build::private_name(self.ast, method.key(self.ast)) {
            let (dec, decorators_have_this) = self.merge_decorators_for_member(dec_exprs);
            let init = self
                .ctx
                .resolver
                .private_binding(self.ast, &format!("_call_{priv_name}"));
            self.extra_vars.push(build::declarator(self.ast, init, None));

            if is_static {
                self.state
                    .init_static
                    .get_or_insert_with(|| self.ctx.resolver.private_binding(self.ast, "_initStatic"));
            } else {
                self.state
                    .instance_brand
                    .get_or_insert_with(|| priv_name.clone());
                self.state
                    .init_proto
                    .get_or_insert_with(|| self.ctx.resolver.private_binding(self.ast, "_initProto"));
            }

            let func = method.value(self.ast);
            let caller_fn = clone_function(self.ast, self.ctx.resolver, func);
            if is_static && let Some(class_name) = self.active_class_ref_for_super() {
                rewrite_super_in_moved_static_member(self.ast, self.ctx, caller_fn, class_name);
            }

            let kind_expr = self.int_expr(Self::method_kind_code(is_static, kind, decorators_have_this));
            let name_expr = build::string(self.ast, &priv_name);
            let elems = self.ast.add_sub_range([
                dec,
                Some(Argument::Expr(kind_expr)),
                Some(Argument::Expr(name_expr)),
                Some(Argument::Expr(Expr::Function(caller_fn))),
            ]);
            let arg = Some(Argument::Expr(self.ast.expr_array_expression(DUMMY_SP, elems)));
            if is_static {
                self.state.init_static_args.push(arg);
            } else {
                self.state.init_proto_args.push(arg);
            }

            self.push_lhs(init, is_static, false);

            match kind {
                MethodDefinitionKind::Method => {
                    let init_ref = self.ctx.resolver.reference(self.ast, init);
                    let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(init_ref));
                    method.set_kind(self.ast, MethodDefinitionKind::Get);
                    func.set_async(self.ast, false);
                    func.set_generator(self.ast, false);
                    let empty_params = self.ast.formal_parameters(
                        DUMMY_SP,
                        SubRange::empty(),
                        None,
                        FormalParameterKind::FormalParameters,
                    );
                    func.set_params(self.ast, empty_params);
                    let fn_body = build::function_body(self.ast, [ret]);
                    func.set_body(self.ast, fn_body);
                }
                MethodDefinitionKind::Get => {
                    let callee = self.ctx.resolver.reference(self.ast, init);
                    let this = self.ast.expr_this_expression(DUMMY_SP);
                    let call = build::call(self.ast, callee, [this], false);
                    let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(call));
                    let fn_body = build::function_body(self.ast, [ret]);
                    func.set_body(self.ast, fn_body);
                }
                MethodDefinitionKind::Set => {
                    let first_item = func
                        .params(self.ast)
                        .items(self.ast)
                        .iter()
                        .next()
                        .map(|slot| self.ast.get_node_in_sub_range(slot))
                        .and_then(|item| item.as_formal_parameter(self.ast))
                        .and_then(|p| p.pattern(self.ast).as_binding_identifier(self.ast))
                        .expect("setter parameter should be an identifier");
                    let value_arg = self.ctx.resolver.reference(self.ast, first_item);
                    let callee = self.ctx.resolver.reference(self.ast, init);
                    let this = self.ast.expr_this_expression(DUMMY_SP);
                    let call = build::call(self.ast, callee, [this, value_arg], false);
                    let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(call));
                    let fn_body = build::function_body(self.ast, [ret]);
                    func.set_body(self.ast, fn_body);
                }
                MethodDefinitionKind::Constructor => unreachable!(),
            }
            return;
        }

        // Public method.
        let (dec, decorators_have_this) = self.merge_decorators_for_member(dec_exprs);
        let (name, _init, replacement_key) =
            self.initializer_name(method.key(self.ast), method.computed(self.ast), "call");
        if let Some((new_key, new_computed)) = replacement_key {
            method.set_key(self.ast, new_key);
            method.set_computed(self.ast, new_computed);
        }

        if is_static {
            self.state
                .init_static
                .get_or_insert_with(|| self.ctx.resolver.private_binding(self.ast, "_initStatic"));
        } else {
            self.state
                .init_proto
                .get_or_insert_with(|| self.ctx.resolver.private_binding(self.ast, "_initProto"));
        }

        let kind_expr = self.int_expr(Self::method_kind_code(is_static, kind, decorators_have_this));
        let elems = self.ast.add_sub_range([
            dec,
            Some(Argument::Expr(kind_expr)),
            Some(Argument::Expr(name)),
        ]);
        let arg = Some(Argument::Expr(self.ast.expr_array_expression(DUMMY_SP, elems)));
        if is_static {
            self.state.init_static_args.push(arg);
        } else {
            self.state.init_proto_args.push(arg);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L3200>.
    fn visit_property_definition(&mut self, prop: PropertyDefinition) {
        if prop.declare(self.ast) {
            return;
        }
        prop.visit_mut_children_with(self);

        let is_static = prop.r#static(self.ast);
        let decs = prop.decorators(self.ast);
        if decs.is_empty() {
            let new_val = self.prepend_pending_instance_field_extras(prop.value(self.ast), is_static);
            prop.set_value(self.ast, new_val);
            return;
        }

        let dec_exprs: Vec<_> = decs
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
            .collect();
        prop.set_decorators(self.ast, SubRange::empty());

        let (dec, decorators_have_this) = self.merge_decorators_for_member(dec_exprs);

        if let Some(priv_name) = build::private_name(self.ast, prop.key(self.ast)) {
            let init = self
                .ctx
                .resolver
                .private_binding(self.ast, &format!("_init_{priv_name}"));
            let init_extra = self
                .ctx
                .resolver
                .private_binding(self.ast, &format!("_init_extra_{priv_name}"));

            self.extra_vars.push(build::declarator(self.ast, init, None));
            self.extra_vars
                .push(build::declarator(self.ast, init_extra, None));

            let init_call = self.field_init_call_expr(init, is_static, prop.value(self.ast));
            let field_init_expr = self
                .prepend_pending_instance_field_extras(Some(init_call), is_static)
                .unwrap();

            let final_val = if !is_static {
                let extra_call = self.extra_init_call_expr(init_extra, false);
                self.queue_pending_instance_field_extra(extra_call, false);
                field_init_expr
            } else {
                let extra_call = self.extra_init_call_expr(init_extra, true);
                self.wrap_init_with_extra(field_init_expr, extra_call)
            };
            prop.set_value(self.ast, Some(final_val));

            let receiver_g = self.ctx.resolver.private_binding(self.ast, "_this");
            let recv_g_ref = self.ctx.resolver.reference(self.ast, receiver_g);
            let key_g = make_private_key(self.ast, &priv_name);
            let mem_g = self
                .ast
                .expr_member_expression(DUMMY_SP, recv_g_ref, key_g, false, false);
            let ret_g = self.ast.stmt_return_statement(DUMMY_SP, Some(mem_g));
            let getter = build::function_with(self.ast, [receiver_g], [ret_g]);

            let receiver_s = self.ctx.resolver.private_binding(self.ast, "_this");
            let setter_arg = self.ctx.resolver.private_binding(self.ast, "value");
            let recv_s_ref = self.ctx.resolver.reference(self.ast, receiver_s);
            let key_s = make_private_key(self.ast, &priv_name);
            let mem_s = self
                .ast
                .expr_member_expression(DUMMY_SP, recv_s_ref, key_s, false, false);
            let val_s = self.ctx.resolver.reference(self.ast, setter_arg);
            let assign_s = build::assign(self.ast, mem_s, val_s);
            let stmt_s = self.ast.stmt_expression_statement(DUMMY_SP, assign_s);
            let setter = build::function_with(self.ast, [receiver_s, setter_arg], [stmt_s]);

            let kind_expr = self.int_expr(Self::field_kind_code(is_static, decorators_have_this));
            let name_expr = build::string(self.ast, &priv_name);
            let elems = self.ast.add_sub_range([
                dec,
                Some(Argument::Expr(kind_expr)),
                Some(Argument::Expr(name_expr)),
                Some(Argument::Expr(getter)),
                Some(Argument::Expr(setter)),
            ]);
            let initialize_init = Some(Argument::Expr(self.ast.expr_array_expression(DUMMY_SP, elems)));

            if !is_static {
                self.state
                    .instance_brand
                    .get_or_insert_with(|| priv_name.clone());
            }
            self.push_lhs(init, is_static, true);
            self.push_lhs(init_extra, is_static, true);
            if is_static {
                self.state.init_static_args.push(initialize_init);
            } else {
                self.state.init_proto_args.push(initialize_init);
            }
            return;
        }

        // Public field.
        let (name, init, replacement_key) =
            self.initializer_name(prop.key(self.ast), prop.computed(self.ast), "init");
        if let Some((new_key, new_computed)) = replacement_key {
            prop.set_key(self.ast, new_key);
            prop.set_computed(self.ast, new_computed);
        }
        let init_sym = self.ast.get_utf8(init.name(self.ast)).to_owned();
        let init_extra = self
            .ctx
            .resolver
            .private_binding(self.ast, &format!("_init_extra_{init_sym}"));

        self.extra_vars.push(build::declarator(self.ast, init, None));
        self.extra_vars
            .push(build::declarator(self.ast, init_extra, None));

        let init_call = self.field_init_call_expr(init, is_static, prop.value(self.ast));
        let field_init_expr = self
            .prepend_pending_instance_field_extras(Some(init_call), is_static)
            .unwrap();

        let final_val = if !is_static {
            let extra_call = self.extra_init_call_expr(init_extra, false);
            self.queue_pending_instance_field_extra(extra_call, false);
            field_init_expr
        } else {
            let extra_call = self.extra_init_call_expr(init_extra, true);
            self.wrap_init_with_extra(field_init_expr, extra_call)
        };
        prop.set_value(self.ast, Some(final_val));

        let kind_expr = self.int_expr(Self::field_kind_code(is_static, decorators_have_this));
        let elems = self.ast.add_sub_range([
            dec,
            Some(Argument::Expr(kind_expr)),
            Some(Argument::Expr(name)),
        ]);
        let initialize_init = Some(Argument::Expr(self.ast.expr_array_expression(DUMMY_SP, elems)));

        self.push_lhs(init, is_static, true);
        self.push_lhs(init_extra, is_static, true);
        if is_static {
            self.state.init_static_args.push(initialize_init);
        } else {
            self.state.init_proto_args.push(initialize_init);
        }
    }

    fn visit_stmt_list(&mut self, stmts: SubRange<Stmt>, is_program: bool) -> SubRange<Stmt> {
        let old_state = if is_program {
            None
        } else {
            Some(mem::take(&mut self.state))
        };
        let old_extra_vars = mem::take(&mut self.extra_vars);
        let old_extra_lets = mem::take(&mut self.extra_lets);
        let old_pre_class_inits = mem::take(&mut self.pre_class_inits);
        let old_extra_exports = if is_program {
            Some(mem::take(&mut self.extra_exports))
        } else {
            None
        };

        let mut items: Vec<_> = stmts
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let mut insert_builder = InsertPassBuilder::new();

        for (index, item) in items.iter_mut().enumerate() {
            let stmt = *item;
            *item = stmt.visit_mut_with(self).unwrap_or(stmt);

            if !self.pre_class_inits.is_empty() {
                let exprs = mem::take(&mut self.pre_class_inits);
                let seq = build::sequence(self.ast, exprs);
                insert_builder.push_back(index, self.ast.stmt_expression_statement(DUMMY_SP, seq));
            }
            if !self.extra_lets.is_empty() {
                let lets = mem::take(&mut self.extra_lets);
                insert_builder.push_back(index, build::declaration(self.ast, VariableKind::Let, lets));
            }
        }

        for &item in &items {
            self.maybe_rewrite_implicit_globals_in_stmt(item);
        }

        if !self.extra_vars.is_empty() {
            let insert_pos = items
                .iter()
                .position(|&stmt| !is_maybe_directive(self.ast, stmt))
                .unwrap_or(0);
            let vars = mem::take(&mut self.extra_vars);
            insert_builder.push_front(
                insert_pos,
                build::declaration(self.ast, VariableKind::Let, vars),
            );
        }

        if is_program && !self.extra_exports.is_empty() {
            let specs = mem::take(&mut self.extra_exports);
            let specs_range = self.ast.add_sub_range(specs);
            let export_stmt = Stmt::ExportNamedDeclaration(self.ast.export_named_declaration(
                DUMMY_SP,
                None,
                specs_range,
                None,
                SubRange::empty(),
                ImportOrExportKind::Value,
            ));
            insert_builder.push_back(items.len() + 1, export_stmt);
        }

        let built = insert_builder.build(items);
        let range = self.ast.add_sub_range(built);

        if is_program && !self.rename_map.is_empty() {
            let rename_map = mem::take(&mut self.rename_map);
            for slot in range.iter() {
                let stmt = self.ast.get_node_in_sub_range(slot);
                rename_symbols(self.ast, self.ctx.resolver, stmt, &rename_map);
            }
        }

        self.extra_vars = old_extra_vars;
        self.extra_lets = old_extra_lets;
        self.pre_class_inits = old_pre_class_inits;
        if let Some(exports) = old_extra_exports {
            self.extra_exports = exports;
        }
        if let Some(state) = old_state {
            self.state = state;
        }

        range
    }
}

impl<'ast> VisitMut<'ast> for DecoratorPass<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_program(&mut self, program: Program) -> Option<Program> {
        let body = self.visit_stmt_list(program.body(self.ast), true);
        program.set_body(self.ast, body);
        None
    }

    fn visit_mut_block_statement(&mut self, block: BlockStatement) -> Option<BlockStatement> {
        let body = self.visit_stmt_list(block.body(self.ast), false);
        block.set_body(self.ast, body);
        None
    }

    fn visit_mut_function_body(&mut self, body: FunctionBody) -> Option<FunctionBody> {
        let stmts = self.visit_stmt_list(body.body(self.ast), false);
        body.set_body(self.ast, stmts);
        None
    }

    fn visit_mut_static_block(&mut self, block: StaticBlock) -> Option<StaticBlock> {
        let stmts = self.visit_stmt_list(block.body(self.ast), false);
        block.set_body(self.ast, stmts);
        None
    }

    fn visit_mut_switch_case(&mut self, case: SwitchCase) -> Option<SwitchCase> {
        if let Some(test) = case.test(self.ast)
            && let Some(replacement) = test.visit_mut_with(self)
        {
            case.set_test(self.ast, Some(replacement));
        }
        let consequent = self.visit_stmt_list(case.consequent(self.ast), false);
        case.set_consequent(self.ast, consequent);
        None
    }

    fn visit_mut_ts_module_block(&mut self, block: TsModuleBlock) -> Option<TsModuleBlock> {
        let body = self.visit_stmt_list(block.body(self.ast), false);
        block.set_body(self.ast, body);
        None
    }

    fn visit_mut_stmt(&mut self, stmt: Stmt) -> Option<Stmt> {
        match self.ast.stmt_data(stmt) {
            StmtData::Declaration(decl) => {
                if let Some(cls) = decl.as_class(self.ast)
                    && !cls.decorators(self.ast).is_empty()
                {
                    let old_name = self.current_class_name.take();
                    self.current_class_name = cls.id(self.ast);
                    let new_stmt = self.handle_class_decl(cls);
                    self.current_class_name = old_name;
                    return Some(new_stmt);
                }
            }
            StmtData::ExportNamedDeclaration(export) => {
                if let Some(decl) = export.declaration(self.ast)
                    && let Some(cls) = decl.as_class(self.ast)
                    && !cls.decorators(self.ast).is_empty()
                {
                    let ident = cls.id(self.ast).expect("exported class has id");
                    let local_sym = self.ctx.resolver.symbol_of(ident.node_id());
                    let orig_name = ident.name(self.ast);
                    let span = export.span(self.ast);
                    let old_name = self.current_class_name.take();
                    self.current_class_name = Some(ident);
                    let new_stmt = self.handle_class_decl(cls);
                    self.current_class_name = old_name;

                    let local_name = self.ast.identifier_name(DUMMY_SP, orig_name);
                    self.ctx.resolver.set_symbol(local_name.node_id(), local_sym);
                    let exported_name = self.ast.identifier_name(DUMMY_SP, orig_name);
                    self.extra_exports.push(self.ast.export_specifier(
                        span,
                        ModuleExportName::IdentifierName(local_name),
                        ModuleExportName::IdentifierName(exported_name),
                        ImportOrExportKind::Value,
                    ));
                    return Some(new_stmt);
                }
            }
            StmtData::ExportDefaultDeclaration(export) => {
                if let Some(cls) = export.declaration(self.ast).as_class(self.ast)
                    && !cls.decorators(self.ast).is_empty()
                {
                    let ident = cls.id(self.ast).unwrap_or_else(|| {
                        let id = self.ctx.resolver.private_binding(self.ast, "_default");
                        cls.set_id(self.ast, Some(id));
                        id
                    });
                    let local_sym = self.ctx.resolver.symbol_of(ident.node_id());
                    let orig_name = ident.name(self.ast);
                    cls.set_class_type(self.ast, ClassType::ClassDeclaration);
                    let span = export.span(self.ast);
                    let old_name = self.current_class_name.take();
                    self.current_class_name = Some(ident);
                    let new_stmt = self.handle_class_decl(cls);
                    self.current_class_name = old_name;

                    let local_name = self.ast.identifier_name(DUMMY_SP, orig_name);
                    self.ctx.resolver.set_symbol(local_name.node_id(), local_sym);
                    let default_sym = self.ast.add_utf8("default");
                    let exported_name = self.ast.identifier_name(DUMMY_SP, default_sym);
                    self.extra_exports.push(self.ast.export_specifier(
                        span,
                        ModuleExportName::IdentifierName(local_name),
                        ModuleExportName::IdentifierName(exported_name),
                        ImportOrExportKind::Value,
                    ));
                    return Some(new_stmt);
                }
            }
            _ => {}
        }
        stmt.visit_mut_children_with(self)
    }

    fn visit_mut_class(&mut self, cls: Class) -> Option<Class> {
        let old_name = self.current_class_name.take();
        self.current_class_name = cls.id(self.ast);
        self.visit_class_body_and_finalize(cls);
        self.current_class_name = old_name;
        None
    }

    fn visit_mut_identifier_reference(
        &mut self,
        ident: IdentifierReference,
    ) -> Option<IdentifierReference> {
        if self.in_implicit_global_rewrite_expr {
            self.mark_ident_for_implicit_global_rewrite(ident);
        }
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L3298>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        if let Some(assign) = expr.as_assignment_expression(self.ast) {
            assign.visit_mut_children_with(self);
            if assign.operator(self.ast) == AssignmentOperator::Assign
                && let Some(id) = assign
                    .left(self.ast)
                    .as_simple_assignment_target(self.ast)
                    .and_then(|t| t.as_identifier_reference(self.ast))
                && self.marked_implicit_globals.contains(&id.node_id())
            {
                let name = self.ast.get_utf8(id.name(self.ast)).to_owned();
                self.implicit_globals.insert(name.clone());
                let member = self.global_this_member_expr(&name);
                assign.set_left(
                    self.ast,
                    AssignmentTarget::SimpleAssignmentTarget(build::target(self.ast, member)),
                );
            }
            return None;
        }

        if let Some(ident) = expr.as_identifier_reference(self.ast) {
            self.visit_mut_identifier_reference(ident);
            if self.marked_implicit_globals.contains(&ident.node_id()) {
                let name = self.ast.get_utf8(ident.name(self.ast)).to_owned();
                if self.implicit_globals.contains(&name) {
                    return Some(self.global_this_member_expr(&name));
                }
            }
            return None;
        }

        if self.in_implicit_global_rewrite_expr {
            if let Some(cls) = expr.as_class(self.ast) {
                self.rewrite_implicit_globals_in_class_expr(cls);
                return None;
            }
            expr.visit_mut_children_with(self);
            return None;
        }

        if let Some(cls) = expr.as_class(self.ast) {
            if !cls.decorators(self.ast).is_empty() {
                let new_id = self.handle_class_expr(cls);
                let old_name = self.current_class_name.take();
                self.current_class_name = cls.id(self.ast);
                self.visit_class_body_and_finalize(cls);
                self.current_class_name = old_name;

                let tail = self.ctx.resolver.reference(self.ast, new_id);
                return Some(build::sequence(self.ast, vec![expr, tail]));
            }

            let mut old_state = mem::take(&mut self.state);
            let private_id_index = old_state.private_id_index;
            self.state = ClassState {
                private_id_index,
                ..Default::default()
            };
            let old_name = self.current_class_name.take();
            self.current_class_name = cls.id(self.ast);
            self.visit_class_body_and_finalize(cls);
            self.current_class_name = old_name;
            let nested_state = mem::take(&mut self.state);
            old_state.private_id_index = nested_state.private_id_index;
            self.state = old_state;
            return None;
        }

        expr.visit_mut_children_with(self)
    }
}

struct InsertPassBuilder<T> {
    inserts: VecDeque<(usize, T)>,
}

impl<T> InsertPassBuilder<T> {
    fn new() -> Self {
        Self {
            inserts: VecDeque::new(),
        }
    }

    fn push_front(&mut self, index: usize, item: T) {
        debug_assert!(self.inserts.front().is_none_or(|(past, _)| *past >= index));
        self.inserts.push_front((index, item));
    }

    fn push_back(&mut self, index: usize, item: T) {
        debug_assert!(self.inserts.back().is_none_or(|(past, _)| *past <= index));
        self.inserts.push_back((index, item));
    }

    fn build(mut self, original: Vec<T>) -> Vec<T> {
        let capacity = original.len() + self.inserts.len();
        let mut new = Vec::with_capacity(capacity);
        for (index, item) in original.into_iter().enumerate() {
            while self
                .inserts
                .front()
                .is_some_and(|(item_index, _)| *item_index == index)
            {
                new.push(self.inserts.pop_front().unwrap().1);
            }
            new.push(item);
        }
        new.extend(self.inserts.into_iter().map(|(_, item)| item));
        debug_assert_eq!(new.len(), capacity);
        new
    }
}
