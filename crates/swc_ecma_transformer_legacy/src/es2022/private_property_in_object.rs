//! SWC private-brand checks. Each class gets one syntax context for its brands.
use crate::common::{
    TraverseCtx, build as b, constructor::default_constructor, resolver::context::Id,
};
use std::collections::HashSet;
use swc_next_ecma_ast::*;

pub(crate) struct PrivatePropertyInObjectPass {
    classes: Vec<ClassData>,
    prepends: Vec<Vec<Expr>>,
    injected: HashSet<Id>,
}
struct ClassData {
    ident: Option<BindingIdentifier>,
    scope: ScopeId,
    expression: bool,
    privates: HashSet<String>,
    methods: HashSet<String>,
    statics: HashSet<String>,
    used: HashSet<String>,
    variables: Vec<VariableDeclarator>,
    initializers: Vec<Expr>,
    constructor: Vec<Expr>,
}
impl PrivatePropertyInObjectPass {
    pub(crate) fn new() -> Self {
        Self {
            classes: Vec::new(),
            prepends: Vec::new(),
            injected: HashSet::new(),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L163>.
    fn var_name_for_brand_check(&self, ast: &Ast<'_>, name: &str) -> Id {
        let class = self.classes.last().unwrap();
        let name = if !class.statics.contains(name) && class.methods.contains(name) {
            class.ident.map_or_else(
                || name.to_owned(),
                |id| ast.get_utf8(id.name(ast)).to_owned(),
            )
        } else {
            name.to_owned()
        };
        Id {
            name: format!("_brand_check_{name}"),
            scope: class.scope,
        }
    }
    fn push_var(
        &mut self,
        ast: &mut Ast<'_>,
        resolver: &mut crate::common::resolver::context::Resolver,
        binding: BindingIdentifier,
        init: Option<Expr>,
    ) {
        let class = self.classes.last_mut().unwrap();
        let init = if class.expression {
            if let Some(init) = init {
                let reference = resolver.reference(ast, binding);
                class.initializers.push(b::assign(ast, reference, init));
            }
            None
        } else {
            init
        };
        class.variables.push(ast.variable_declarator(
            DUMMY_SP,
            BindingPattern::BindingIdentifier(binding),
            None,
            init,
            false,
        ));
    }
    fn wrap_default(&self, ast: &mut Ast<'_>, expression: Expr) -> Expr {
        if names(ast, expression, false).is_empty() {
            return expression;
        }
        let ret = ast.stmt_return_statement(DUMMY_SP, Some(expression));
        let body = ast.add_sub_range([ret]);
        let body = ast.function_body(DUMMY_SP, SubRange::empty(), body);
        let arrow = b::arrow(ast, [], ArrowFunctionBody::FunctionBody(body));
        b::call(ast, arrow, [], false)
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L261>.
impl<'ast> TraverseMut<'ast, TraverseCtx<'_>> for PrivatePropertyInObjectPass {
    fn enter_class(
        &mut self,
        node: Class,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Class> {
        let mut class = ClassData {
            ident: node.id(ctx.ast),
            scope: ctx.state.resolver.fresh_scope(),
            expression: node.class_type(ctx.ast) == ClassType::ClassExpression,
            privates: HashSet::new(),
            methods: HashSet::new(),
            statics: HashSet::new(),
            used: names(ctx.ast, node.body(ctx.ast), true),
            variables: Vec::new(),
            initializers: Vec::new(),
            constructor: Vec::new(),
        };
        for slot in node.body(ctx.ast).body(ctx.ast).iter() {
            let member = ctx.ast.get_node_in_sub_range(slot);
            let (key, method, static_) = match ctx.ast.class_element_data(member) {
                ClassElementData::MethodDefinition(method) => {
                    (method.key(ctx.ast), true, method.r#static(ctx.ast))
                }
                ClassElementData::PropertyDefinition(prop) if !prop.accessor(ctx.ast) => {
                    (prop.key(ctx.ast), false, prop.r#static(ctx.ast))
                }
                _ => continue,
            };
            if let Some(name) = b::private_name(ctx.ast, key) {
                if method {
                    class.methods.insert(name.clone());
                }
                if static_ {
                    class.statics.insert(name.clone());
                }
                class.privates.insert(name);
            }
        }
        self.classes.push(class);
        TraverseAction::Continue
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L215>.
    fn exit_class(
        &mut self,
        node: Class,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Class> {
        let class = self.classes.pop().unwrap();
        if !class.constructor.is_empty() {
            let body = node.body(ctx.ast);
            let mut members: Vec<_> = body
                .body(ctx.ast)
                .iter()
                .map(|slot| ctx.ast.get_node_in_sub_range(slot))
                .collect();
            let constructor = members.iter().find_map(|member| {
                member
                    .as_method_definition(ctx.ast)
                    .filter(|method| method.kind(ctx.ast) == MethodDefinitionKind::Constructor)
            });
            let constructor = constructor.unwrap_or_else(|| {
                let method = default_constructor(
                    ctx.ast,
                    ctx.state.resolver,
                    node.super_class(ctx.ast).is_some(),
                );
                members.push(ClassElement::MethodDefinition(method));
                method
            });
            let function_body = constructor.value(ctx.ast).body(ctx.ast);
            let statements: Vec<_> = class
                .constructor
                .into_iter()
                .map(|expr| ctx.ast.stmt_expression_statement(DUMMY_SP, expr))
                .collect();
            let statements: Vec<_> = function_body
                .body(ctx.ast)
                .iter()
                .map(|slot| ctx.ast.get_node_in_sub_range(slot))
                .chain(statements)
                .collect();
            let statements = ctx.ast.add_sub_range(statements);
            function_body.set_body(ctx.ast, statements);
            let members = ctx.ast.add_sub_range(members);
            body.set_body(ctx.ast, members);
        }
        for variable in class.variables {
            ctx.state.var_declarations.insert_var_declarator(variable);
        }
        if class.expression {
            self.prepends.last_mut().unwrap().extend(class.initializers);
        }
        TraverseAction::Continue
    }
    fn enter_assignment_pattern(
        &mut self,
        node: AssignmentPattern,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<AssignmentPattern> {
        let right = self.wrap_default(ctx.ast, node.right(ctx.ast));
        node.set_right(ctx.ast, right);
        TraverseAction::Continue
    }
    fn enter_formal_parameter(
        &mut self,
        node: FormalParameter,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<FormalParameter> {
        if let Some(init) = node.initializer(ctx.ast) {
            let init = self.wrap_default(ctx.ast, init);
            node.set_initializer(ctx.ast, Some(init));
        }
        TraverseAction::Continue
    }
    fn enter_expr(
        &mut self,
        _: Expr,
        _: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        self.prepends.push(Vec::new());
        TraverseAction::Continue
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L350>.
    fn exit_expr(
        &mut self,
        node: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        let mut prepend = self.prepends.pop().unwrap();
        if !prepend.is_empty() {
            if let Some(sequence) = node.as_sequence_expression(ctx.ast) {
                prepend.extend(
                    sequence
                        .expressions(ctx.ast)
                        .iter()
                        .map(|slot| ctx.ast.get_node_in_sub_range(slot)),
                );
            } else {
                prepend.push(node);
            }
            return TraverseAction::ReplaceSkip(b::sequence(ctx.ast, prepend));
        }
        let Some(binary) = node.as_binary_expression(ctx.ast) else {
            return TraverseAction::Continue;
        };
        if binary.operator(ctx.ast) != BinaryOperator::In {
            return TraverseAction::Continue;
        }
        let Some(private) = binary.left(ctx.ast).as_private_identifier(ctx.ast) else {
            return TraverseAction::Continue;
        };
        let name = ctx.ast.get_utf8(private.name(ctx.ast)).to_owned();
        let class = self.classes.last().unwrap();
        let method = class.methods.contains(&name);
        if class.statics.contains(&name)
            && method
            && let Some(ident) = class.ident
        {
            let left = ctx.state.resolver.reference(ctx.ast, ident);
            return TraverseAction::ReplaceSkip(ctx.ast.expr_binary_expression(
                binary.span(ctx.ast),
                left,
                binary.right(ctx.ast),
                BinaryOperator::StrictEqual,
            ));
        }
        let brand = self.var_name_for_brand_check(ctx.ast, &name);
        if class.privates.contains(&name) && self.injected.insert(brand.clone()) {
            let binding = ctx.state.resolver.binding(ctx.ast, &brand);
            let weak_set = ctx
                .state
                .resolver
                .marked_reference(ctx.ast, "WeakSet", true);
            let init = Expr::NewExpression(ctx.ast.new_expression(
                DUMMY_SP,
                weak_set,
                None,
                SubRange::empty(),
            ));
            self.push_var(ctx.ast, ctx.state.resolver, binding, Some(init));
            if method {
                let object = b::reference(ctx.ast, ctx.state.resolver, &brand);
                let callee = b::member(ctx.ast, object, "add");
                let this = ctx.ast.expr_this_expression(DUMMY_SP);
                let add = b::call(ctx.ast, callee, [this], false);
                self.classes.last_mut().unwrap().constructor.push(add);
            }
        }
        let object = b::reference(ctx.ast, ctx.state.resolver, &brand);
        let callee = b::member(ctx.ast, object, "has");
        TraverseAction::ReplaceSkip(b::call(ctx.ast, callee, [binary.right(ctx.ast)], false))
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L474>.
    fn exit_property_definition(
        &mut self,
        node: PropertyDefinition,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<PropertyDefinition> {
        let Some(name) = b::private_name(ctx.ast, node.key(ctx.ast)) else {
            return TraverseAction::Continue;
        };
        if node.accessor(ctx.ast) || !self.classes.last().unwrap().used.contains(&name) {
            return TraverseAction::Continue;
        }
        let brand = self.var_name_for_brand_check(ctx.ast, &name);
        let object = b::reference(ctx.ast, ctx.state.resolver, &brand);
        let callee = b::member(ctx.ast, object, "add");
        let this = ctx.ast.expr_this_expression(DUMMY_SP);
        let add = b::call(ctx.ast, callee, [this], false);
        let value = if let Some(value) = node.value(ctx.ast) {
            let tmp = ctx.state.resolver.private_binding(ctx.ast, "_tmp");
            self.push_var(ctx.ast, ctx.state.resolver, tmp, None);
            let reference = ctx.state.resolver.reference(ctx.ast, tmp);
            let assign = b::assign(ctx.ast, reference, value);
            let reference = ctx.state.resolver.reference(ctx.ast, tmp);
            b::sequence(ctx.ast, vec![assign, add, reference])
        } else {
            ctx.ast
                .expr_unary_expression(DUMMY_SP, add, UnaryOperator::Void)
        };
        node.set_value(ctx.ast, Some(value));
        TraverseAction::Continue
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L622>.
struct PrivateInFinder<'a, 'ast> {
    ast: &'a Ast<'ast>,
    names: HashSet<String>,
    ignore_class: bool,
}
impl<'ast> Visit<'ast> for PrivateInFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }
    fn visit_class(&mut self, node: Class) {
        if !self.ignore_class {
            node.visit_children_with(self);
        }
    }
    fn visit_binary_expression(&mut self, node: BinaryExpression) {
        node.visit_children_with(self);
        if node.operator(self.ast) == BinaryOperator::In
            && let Some(name) = node.left(self.ast).as_private_identifier(self.ast)
        {
            self.names
                .insert(self.ast.get_utf8(name.name(self.ast)).to_owned());
        }
    }
}
fn names<'a, 'ast>(
    ast: &'a Ast<'ast>,
    node: impl VisitWith<'ast, PrivateInFinder<'a, 'ast>>,
    ignore_class: bool,
) -> HashSet<String> {
    let mut finder = PrivateInFinder {
        ast,
        names: HashSet::new(),
        ignore_class,
    };
    // The class itself is the analysis root; only nested classes are skipped.
    node.visit_with(&mut finder);
    finder.names
}
