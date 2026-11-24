//! The super/new.target subset of SWC FnEnvHoister selected by async lowering.
use crate::common::{build as b, resolver::context::Resolver};
use swc_next_ecma_ast::*;

struct SuperField {
    name: Option<String>,
    binding: BindingIdentifier,
}
pub(crate) struct FnEnvHoister<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    this: Option<BindingIdentifier>,
    new_target: Option<BindingIdentifier>,
    get: Vec<SuperField>,
    set: Vec<SuperField>,
    update: Vec<SuperField>,
    extra: Vec<VariableDeclarator>,
    in_pattern: bool,
}
impl FnEnvHoister<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/function/fn_env_hoister.rs#L202>.
    fn field(&mut self, name: Option<String>, kind: usize) -> BindingIdentifier {
        if kind == 2 {
            self.field(name.clone(), 0);
            self.field(name.clone(), 1);
        }
        let fields = match kind {
            0 => &mut self.get,
            1 => &mut self.set,
            _ => &mut self.update,
        };
        if let Some(field) = fields.iter().find(|field| field.name == name) {
            return field.binding;
        }
        let prefix = match kind {
            0 => "_superprop_get",
            1 => "_superprop_set",
            _ => "_superprop_update",
        };
        let binding = self.resolver.private_binding(
            self.ast,
            &name
                .as_ref()
                .map_or_else(|| prefix.to_owned(), |name| format!("{prefix}_{name}")),
        );
        fields.push(SuperField { name, binding });
        binding
    }
    fn name(&self, member: MemberExpression) -> Option<String> {
        if member.computed(self.ast) {
            None
        } else {
            Some(
                self.ast
                    .get_utf8(
                        member
                            .property(self.ast)
                            .as_identifier_name(self.ast)
                            .unwrap()
                            .name(self.ast),
                    )
                    .to_owned(),
            )
        }
    }
    fn access(&mut self, member: MemberExpression, kind: usize) -> Expr {
        let name = self.name(member);
        let binding = self.field(name.clone(), kind);
        let reference = self.resolver.reference(self.ast, binding);
        let expression = if kind == 2 && name.is_some() {
            reference
        } else {
            let args: Vec<_> = if name.is_none() {
                vec![member.property(self.ast).as_expr(self.ast).unwrap()]
            } else {
                Vec::new()
            };
            b::call(self.ast, reference, args, false)
        };
        if kind == 2 {
            b::member(self.ast, expression, "_")
        } else {
            expression
        }
    }
    fn flush(&mut self, body: SubRange<Stmt>) -> SubRange<Stmt> {
        if self.extra.is_empty() {
            return body;
        }
        let variables =
            b::declaration(self.ast, VariableKind::Var, std::mem::take(&mut self.extra));
        b::prepend_stmts(self.ast, [variables], body)
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/function/fn_env_hoister.rs#L635>.
    fn to_stmt(&mut self) -> Vec<Stmt> {
        let mut variables = Vec::new();
        if let Some(binding) = self.this {
            let value = self.ast.expr_this_expression(DUMMY_SP);
            variables.push(b::declarator(self.ast, binding, Some(value)));
        }
        if let Some(binding) = self.new_target {
            let new = self.ast.add_utf8("new");
            let new = self.ast.identifier_name(DUMMY_SP, new);
            let target = self.ast.add_utf8("target");
            let target = self.ast.identifier_name(DUMMY_SP, target);
            let value = Expr::MetaProperty(self.ast.meta_property(DUMMY_SP, new, target));
            variables.push(b::declarator(self.ast, binding, Some(value)));
        }
        // SWC emits named fields before the computed field for each operation.
        let mut updates = std::mem::take(&mut self.update);
        updates.sort_by_key(|field| field.name.is_none());
        for field in updates {
            let prop = field
                .name
                .is_none()
                .then(|| self.resolver.private_binding(self.ast, "_prop"));
            let value = self.resolver.private_binding(self.ast, "v");
            let getter = self
                .get
                .iter()
                .find(|get| get.name == field.name)
                .unwrap()
                .binding;
            let setter = self
                .set
                .iter()
                .find(|set| set.name == field.name)
                .unwrap()
                .binding;
            let callee = self.resolver.reference(self.ast, getter);
            let args: Vec<_> = prop
                .map(|prop| self.resolver.reference(self.ast, prop))
                .into_iter()
                .collect();
            let get = b::call(self.ast, callee, args, false);
            let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(get));
            let get = b::function_with(self.ast, [], [ret]);
            let callee = self.resolver.reference(self.ast, setter);
            let mut args: Vec<_> = prop
                .map(|prop| self.resolver.reference(self.ast, prop))
                .into_iter()
                .collect();
            args.push(self.resolver.reference(self.ast, value));
            let set = b::call(self.ast, callee, args, false);
            let set = if prop.is_some() {
                self.ast.stmt_return_statement(DUMMY_SP, Some(set))
            } else {
                self.ast.stmt_expression_statement(DUMMY_SP, set)
            };
            let set = b::function_with(self.ast, [value], [set]);
            let name = self.ast.add_utf8("_");
            let key = PropertyKey::IdentifierName(self.ast.identifier_name(DUMMY_SP, name));
            let get = ObjectPropertyKind::ObjectProperty(self.ast.object_property(
                DUMMY_SP,
                key,
                get,
                PropertyKind::Get,
                false,
                false,
                false,
            ));
            let name = self.ast.add_utf8("_");
            let key = PropertyKey::IdentifierName(self.ast.identifier_name(DUMMY_SP, name));
            let set = ObjectPropertyKind::ObjectProperty(self.ast.object_property(
                DUMMY_SP,
                key,
                set,
                PropertyKind::Set,
                false,
                false,
                false,
            ));
            let properties = self.ast.add_sub_range([get, set]);
            let mut init = self.ast.expr_object_expression(DUMMY_SP, properties);
            if let Some(prop) = prop {
                init = b::arrow(self.ast, [prop], ArrowFunctionBody::Expr(init));
            }
            variables.push(b::declarator(self.ast, field.binding, Some(init)));
        }
        for (kind, mut fields) in [
            (0, std::mem::take(&mut self.get)),
            (1, std::mem::take(&mut self.set)),
        ] {
            fields.sort_by_key(|field| field.name.is_none());
            for field in fields {
                let mut bindings = Vec::new();
                let super_ = self.ast.expr_super(DUMMY_SP);
                let member = if let Some(name) = field.name {
                    b::member(self.ast, super_, &name)
                } else {
                    let prop = self.resolver.private_binding(self.ast, "_prop");
                    bindings.push(prop);
                    let key = self.resolver.reference(self.ast, prop);
                    b::computed(self.ast, super_, key)
                };
                let init = if kind == 0 {
                    member
                } else {
                    let value = self.resolver.private_binding(self.ast, "_value");
                    bindings.push(value);
                    let value = self.resolver.reference(self.ast, value);
                    b::assign(self.ast, member, value)
                };
                let init = b::arrow(self.ast, bindings, ArrowFunctionBody::Expr(init));
                variables.push(b::declarator(self.ast, field.binding, Some(init)));
            }
        }
        if variables.is_empty() {
            Vec::new()
        } else {
            vec![b::declaration(self.ast, VariableKind::Var, variables)]
        }
    }
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/function/fn_env_hoister.rs#L364>.
impl<'ast> VisitMut<'ast> for FnEnvHoister<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        None
    }
    fn visit_mut_class(&mut self, _: Class) -> Option<Class> {
        None
    }
    fn visit_mut_binding_pattern(&mut self, node: BindingPattern) -> Option<BindingPattern> {
        let old = std::mem::replace(&mut self.in_pattern, true);
        let replacement = node.visit_mut_children_with(self);
        self.in_pattern = old;
        replacement
    }
    fn visit_mut_assignment_target(&mut self, node: AssignmentTarget) -> Option<AssignmentTarget> {
        let old = std::mem::replace(&mut self.in_pattern, true);
        let replacement = node.visit_mut_children_with(self);
        self.in_pattern = old;
        replacement
    }
    fn visit_mut_block_statement(&mut self, node: BlockStatement) -> Option<BlockStatement> {
        node.visit_mut_children_with(self);
        let body = self.flush(node.body(self.ast));
        node.set_body(self.ast, body);
        None
    }
    fn visit_mut_function_body(&mut self, node: FunctionBody) -> Option<FunctionBody> {
        node.visit_mut_children_with(self);
        let body = self.flush(node.body(self.ast));
        node.set_body(self.ast, body);
        None
    }
    fn visit_mut_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
    ) -> Option<ArrowFunctionExpression> {
        node.visit_mut_children_with(self);
        node.set_expression(self.ast, node.body(self.ast).is_expr(self.ast));
        None
    }
    fn visit_mut_arrow_function_body(
        &mut self,
        node: ArrowFunctionBody,
    ) -> Option<ArrowFunctionBody> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        if let Some(expr) = node.as_expr(self.ast)
            && !self.extra.is_empty()
        {
            let ret = self.ast.stmt_return_statement(DUMMY_SP, Some(expr));
            let stmts = self.ast.add_sub_range([ret]);
            let stmts = self.flush(stmts);
            return Some(ArrowFunctionBody::FunctionBody(self.ast.function_body(
                DUMMY_SP,
                SubRange::empty(),
                stmts,
            )));
        }
        Some(node)
    }
    fn visit_mut_simple_assignment_target(
        &mut self,
        node: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        let member = node.as_member_expression(self.ast)?;
        if member.object(self.ast).is_super(self.ast) {
            let expr = self.access(member, 2);
            return Some(b::target(self.ast, expr));
        }
        node.visit_mut_children_with(self);
        None
    }
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if let Some(meta) = node.as_meta_property(self.ast)
            && self.ast.get_utf8(meta.meta(self.ast).name(self.ast)) == "new"
        {
            let binding = *self
                .new_target
                .get_or_insert_with(|| self.resolver.private_binding(self.ast, "_newtarget"));
            return Some(self.resolver.reference(self.ast, binding));
        }
        if let Some(assign) = node.as_assignment_expression(self.ast)
            && let Some(member) = assign
                .left(self.ast)
                .as_simple_assignment_target(self.ast)
                .and_then(|target| target.as_member_expression(self.ast))
                .filter(|member| member.object(self.ast).is_super(self.ast))
        {
            let name = self.name(member);
            let setter = self.field(name.clone(), 1);
            let mut args = Vec::new();
            let right = if assign.operator(self.ast) == AssignmentOperator::Assign {
                if name.is_none() {
                    args.push(member.property(self.ast).as_expr(self.ast).unwrap());
                }
                assign.right(self.ast)
            } else {
                let getter = self.field(name.clone(), 0);
                let mut get_args = Vec::new();
                if name.is_none() {
                    let tmp = self.resolver.private_binding(self.ast, "tmp");
                    self.extra.push(b::declarator(self.ast, tmp, None));
                    let reference = self.resolver.reference(self.ast, tmp);
                    args.push(b::assign(
                        self.ast,
                        reference,
                        member.property(self.ast).as_expr(self.ast).unwrap(),
                    ));
                    get_args.push(self.resolver.reference(self.ast, tmp));
                }
                let getter = self.resolver.reference(self.ast, getter);
                let get = b::call(self.ast, getter, get_args, false);
                b::compound(
                    self.ast,
                    assign.operator(self.ast),
                    get,
                    assign.right(self.ast),
                )
            };
            args.push(right);
            let setter = self.resolver.reference(self.ast, setter);
            let result = b::call(self.ast, setter, args, false);
            result.visit_mut_children_with(self);
            return Some(result);
        }
        if let Some(call) = node.as_call_expression(self.ast)
            && let Some(member) = call
                .callee(self.ast)
                .as_member_expression(self.ast)
                .filter(|member| member.object(self.ast).is_super(self.ast))
        {
            let get = self.access(member, 0);
            let callee = b::member(self.ast, get, "call");
            let binding = *self
                .this
                .get_or_insert_with(|| self.resolver.private_binding(self.ast, "_this"));
            let reference = self.resolver.reference(self.ast, binding);
            let args: Vec<_> = std::iter::once(Argument::Expr(reference))
                .chain(
                    call.arguments(self.ast)
                        .iter()
                        .map(|slot| self.ast.get_node_in_sub_range(slot)),
                )
                .collect();
            let args = self.ast.add_sub_range(args);
            let result = self.ast.expr_call_expression(
                call.span(self.ast),
                callee,
                None,
                args,
                false,
                false,
            );
            result.visit_mut_children_with(self);
            return Some(result);
        }
        if let Some(member) = node
            .as_member_expression(self.ast)
            .filter(|member| member.object(self.ast).is_super(self.ast))
        {
            // SWC visits the computed expression's children before replacing it.
            if member.computed(self.ast) {
                member
                    .property(self.ast)
                    .as_expr(self.ast)
                    .unwrap()
                    .visit_mut_children_with(self);
            }
            return Some(self.access(member, if self.in_pattern { 2 } else { 0 }));
        }
        node.visit_mut_children_with(self);
        None
    }
}

impl<'a, 'ast> FnEnvHoister<'a, 'ast> {
    pub(crate) fn visit<N: Copy + VisitMutWith<'ast, Self>>(
        ast: &'a mut Ast<'ast>,
        resolver: &'a mut Resolver,
        node: N,
    ) -> (N, Vec<Stmt>) {
        let mut hoister = Self {
            ast,
            resolver,
            this: None,
            new_target: None,
            get: Vec::new(),
            set: Vec::new(),
            update: Vec::new(),
            extra: Vec::new(),
            in_pattern: false,
        };
        let node = node.visit_mut_with(&mut hoister).unwrap_or(node);
        (node, hoister.to_stmt())
    }
}

struct InitThis<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    binding: BindingIdentifier,
}
impl<'ast> VisitMut<'ast> for InitThis<'_, 'ast> {
    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        node.visit_mut_children_with(self);
        None
    }

    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_class(&mut self, _: Class) -> Option<Class> {
        None
    }
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        node.visit_mut_children_with(self);
        if !node
            .as_call_expression(self.ast)
            .is_some_and(|call| call.callee(self.ast).is_super(self.ast))
        {
            return None;
        }
        let left = self.resolver.reference(self.ast, self.binding);
        let right = self.ast.expr_this_expression(DUMMY_SP);
        let assign = b::assign(self.ast, left, right);
        let result = b::sequence(self.ast, vec![node, assign]);
        Some(self.ast.expr_parenthesized_expression(DUMMY_SP, result))
    }
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/function/fn_env_hoister.rs#L589>.
pub(crate) fn init_this(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    node: FunctionBody,
    binding: BindingIdentifier,
) {
    node.visit_mut_with(&mut InitThis {
        ast,
        resolver,
        binding,
    });
}
