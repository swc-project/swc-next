use super::Config;
use crate::common::{TraverseCtx, build as b};
use std::collections::HashMap;
use swc_next_ecma_ast::*;

#[derive(Clone, Copy, Default)]
pub(super) struct PrivateKind {
    pub is_static: bool,
    pub is_method: bool,
    pub has_getter: bool,
    pub has_setter: bool,
}
impl PrivateKind {
    fn is_method(self) -> bool {
        self.is_method && !self.has_getter && !self.has_setter
    }
    fn is_readonly(self) -> bool {
        self.is_method && !self.has_setter
    }
    fn is_writeonly(self) -> bool {
        self.is_method && !self.has_getter && self.has_setter
    }
}
pub(super) struct Private {
    pub class_name: BindingIdentifier,
    pub scope: ScopeId,
    pub ident: HashMap<String, PrivateKind>,
}
impl Private {
    pub(super) fn binding(
        &self,
        ast: &mut Ast<'_>,
        ctx: &mut TraverseCtx<'_>,
        name: &str,
    ) -> BindingIdentifier {
        let symbol = ctx.resolver.scoped_symbol(ast, self.scope, name);
        ctx.resolver.binding(ast, symbol)
    }
}
#[derive(Default)]
pub(super) struct PrivateRecord(Vec<Private>);

impl PrivateRecord {
    pub(super) fn push(&mut self, private: Private) {
        self.0.push(private);
    }

    pub(super) fn pop(&mut self) {
        self.0.pop();
    }

    pub(super) fn curr_class(&self) -> &Private {
        self.0.last().unwrap()
    }

    fn get(&self, name: &str) -> &Private {
        self.0
            .iter()
            .rev()
            .find(|private| private.ident.contains_key(name))
            .unwrap()
    }
}

#[derive(Clone, Copy)]
enum PrivateAccessType {
    Get,
    DestructureSet,
    Update,
}
pub(super) struct PrivateAccessVisitor<'a, 'ast, 'r> {
    pub ast: &'a mut Ast<'ast>,
    pub ctx: &'a mut TraverseCtx<'r>,
    pub private: &'a PrivateRecord,
    pub options: Config,
    pub vars: Vec<VariableDeclarator>,
}
impl PrivateAccessVisitor<'_, '_, '_> {
    fn info(
        &mut self,
        name: &str,
    ) -> (
        PrivateKind,
        BindingIdentifier,
        BindingIdentifier,
        BindingIdentifier,
    ) {
        let private = self.private.get(name);
        let kind = private.ident[name];
        let class = private.class_name;
        let binding = private.binding(self.ast, self.ctx, &format!("_{name}"));
        let method_name = if is_reserved(name) {
            format!("__{name}")
        } else {
            name.to_owned()
        };
        let method = private.binding(self.ast, self.ctx, &method_name);
        (kind, class, binding, method)
    }
    fn temp(&mut self, object: Expr) -> BindingIdentifier {
        let name = b::alias_ident_for(self.ast, object, "_ref");
        let binding = self.ctx.resolver.private_binding(self.ast, &name);
        self.vars.push(b::declarator(self.ast, binding, None));
        binding
    }
    fn helper(&mut self, name: &'static str, arguments: impl IntoIterator<Item = Expr>) -> Expr {
        let helper = self.ctx.helper(self.ast, name);
        b::call(self.ast, helper, arguments, false)
    }
    fn loose(&mut self, member: MemberExpression) -> Option<Expr> {
        let name = b::private_name(self.ast, member.property(self.ast))?;
        // SWC visits receiver children, rather than redispatching its root.
        member.object(self.ast).visit_mut_children_with(self);
        let (_, _, binding, _) = self.info(&name);
        let reference = self.ctx.resolver.reference(self.ast, binding);
        let object = self.helper(
            "class_private_field_loose_base",
            [member.object(self.ast), reference],
        );
        let reference = self.ctx.resolver.reference(self.ast, binding);
        Some(b::computed(self.ast, object, reference))
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L585>.
    fn convert_private_member(
        &mut self,
        member: MemberExpression,
        access: PrivateAccessType,
        alias: Option<BindingIdentifier>,
    ) -> (Expr, Option<Expr>) {
        let Some(name) = b::private_name(self.ast, member.property(self.ast)) else {
            return (Expr::MemberExpression(member), None);
        };
        let (kind, class, binding, method) = self.info(&name);
        let object = member.object(self.ast);
        let reference = self.ctx.resolver.reference(self.ast, binding);
        if kind.is_static {
            let class = self.ctx.resolver.reference(self.ast, class);
            let helper = match access {
                PrivateAccessType::DestructureSet => "class_static_private_field_destructure",
                PrivateAccessType::Update => "class_static_private_field_update",
                PrivateAccessType::Get if kind.is_method() => "class_static_private_method_get",
                _ => "class_static_private_field_spec_get",
            };
            let third = if matches!(access, PrivateAccessType::Get) && kind.is_method() {
                self.ctx.resolver.reference(self.ast, method)
            } else {
                reference
            };
            let class_copy = b::clone_expr(self.ast, self.ctx.resolver, class);
            let result = self.helper(helper, [object, class, third]);
            return if matches!(access, PrivateAccessType::Get) {
                (result, Some(class_copy))
            } else {
                (b::member(self.ast, result, "value"), None)
            };
        }
        if !matches!(access, PrivateAccessType::Get) {
            let helper = if matches!(access, PrivateAccessType::DestructureSet) {
                "class_private_field_destructure"
            } else {
                "class_private_field_update"
            };
            let result = self.helper(helper, [object, reference]);
            return (b::member(self.ast, result, "value"), None);
        }
        if kind.is_writeonly() {
            let name = b::string(self.ast, &format!("#{name}"));
            let error = self.helper("write_only_error", [name]);
            let receiver = b::clone_expr(self.ast, self.ctx.resolver, object);
            return (b::sequence(self.ast, vec![object, error]), Some(receiver));
        }
        let (object, receiver) = if object.is_this_expression(self.ast) {
            let receiver = self.ast.expr_this_expression(DUMMY_SP);
            (object, receiver)
        } else if let Some(alias) = alias {
            (
                self.ctx.resolver.reference(self.ast, alias),
                self.ctx.resolver.reference(self.ast, alias),
            )
        } else if object.is_identifier_reference(self.ast) {
            (object, b::clone_expr(self.ast, self.ctx.resolver, object))
        } else {
            let alias = self.temp(object);
            let reference = self.ctx.resolver.reference(self.ast, alias);
            (
                b::assign(self.ast, reference, object),
                self.ctx.resolver.reference(self.ast, alias),
            )
        };
        let mut args = vec![object, reference];
        let helper = if kind.is_method() {
            args.push(self.ctx.resolver.reference(self.ast, method));
            "class_private_method_get"
        } else {
            "class_private_field_get"
        };
        (self.helper(helper, args), Some(receiver))
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L282>.
    fn assignment(&mut self, assign: AssignmentExpression, member: MemberExpression) -> Expr {
        member.visit_mut_children_with(self);
        let right = assign.right(self.ast);
        let right = right.visit_mut_with(self).unwrap_or(right);
        let Some(name) = b::private_name(self.ast, member.property(self.ast)) else {
            assign.set_right(self.ast, right);
            return Expr::AssignmentExpression(assign);
        };
        let (kind, class, binding, _) = self.info(&name);
        let object = member.object(self.ast);
        let mut alias = None;
        let receiver = if object.is_this_expression(self.ast)
            || assign.operator(self.ast) == AssignmentOperator::Assign
        {
            b::clone_expr(self.ast, self.ctx.resolver, object)
        } else {
            let binding = self.temp(object);
            alias = Some(binding);
            let reference = self.ctx.resolver.reference(self.ast, binding);
            b::assign(self.ast, reference, object)
        };
        let value = if assign.operator(self.ast) == AssignmentOperator::Assign {
            right
        } else {
            let read = self
                .convert_private_member(member, PrivateAccessType::Get, alias)
                .0;
            b::compound(self.ast, assign.operator(self.ast), read, right)
        };
        if kind.is_static {
            let class = self.ctx.resolver.reference(self.ast, class);
            let name = self.ctx.resolver.reference(self.ast, binding);
            self.helper(
                "class_static_private_field_spec_set",
                [receiver, class, name, value],
            )
        } else if kind.is_readonly() {
            let name = b::string(self.ast, &format!("#{name}"));
            let error = self.helper("read_only_error", [name]);
            b::sequence(self.ast, vec![receiver, value, error])
        } else {
            let name = self.ctx.resolver.reference(self.ast, binding);
            self.helper("class_private_field_set", [receiver, name, value])
        }
    }

    fn lower_private_chain(&mut self, chain: ChainExpression) -> Expr {
        let base = self.ast.expr_data(chain.expression(self.ast));
        let (prefix, terminal_member) = match base {
            ExprData::MemberExpression(member) => (member.object(self.ast), Some(member)),
            ExprData::CallExpression(call) => {
                let callee = call.callee(self.ast);
                let member = if optional_spine(self.ast, callee) {
                    None
                } else {
                    callee.as_member_expression(self.ast)
                };
                (callee, member)
            }
            _ => return Expr::ChainExpression(chain),
        };
        let private = terminal_member
            .is_some_and(|member| b::private_name(self.ast, member.property(self.ast)).is_some());
        if private {
            let options = crate::es2020::OptionalChainingOptions {
                no_document_all: self.options.no_document_all,
                pure_getter: self.options.pure_getter,
            };
            let (expression, vars) = crate::es2022::optional_chaining_impl::expression(
                self.ast,
                self.ctx.resolver,
                options,
                Expr::ChainExpression(chain),
            );
            self.vars.extend(vars);
            return expression;
        }

        // SWC visits each chain link separately. Wrap the prefix for that
        // visit, then fold any remaining chain back into the original spine.
        if optional_spine(self.ast, prefix) && private_spine(self.ast, prefix) {
            let prefix = self.ast.expr_chain_expression(DUMMY_SP, prefix);
            let prefix = prefix.visit_mut_with(self).unwrap_or(prefix);
            let prefix = prefix
                .as_chain_expression(self.ast)
                .map_or(prefix, |chain| chain.expression(self.ast));
            match base {
                ExprData::MemberExpression(member) => member.set_object(self.ast, prefix),
                ExprData::CallExpression(call) => call.set_callee(self.ast, prefix),
                _ => unreachable!(),
            }
        }
        Expr::ChainExpression(chain)
    }
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L195>.
impl<'ast> VisitMut<'ast> for PrivateAccessVisitor<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        let old = std::mem::take(&mut self.vars);
        node.visit_mut_children_with(self);
        if !self.vars.is_empty() {
            let decl = b::declaration(self.ast, VariableKind::Var, std::mem::take(&mut self.vars));
            let body = node.body(self.ast);
            let stmts = b::prepend_stmts(self.ast, [decl], body.body(self.ast));
            body.set_body(self.ast, stmts);
        }
        self.vars = old;
        None
    }
    fn visit_mut_simple_assignment_target(
        &mut self,
        node: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        if let Some(member) = node.as_member_expression(self.ast) {
            if self.options.private_as_properties {
                if let Some(result) = self.loose(member) {
                    return Some(b::target(self.ast, result));
                }
            } else {
                member.visit_mut_children_with(self);
                let result = self
                    .convert_private_member(member, PrivateAccessType::DestructureSet, None)
                    .0;
                return Some(b::target(self.ast, result));
            }
        }
        node.visit_mut_children_with(self);
        None
    }
    fn visit_mut_update_expression(&mut self, node: UpdateExpression) -> Option<UpdateExpression> {
        if !self.options.private_as_properties
            && let Some(member) = node.argument(self.ast).as_member_expression(self.ast)
        {
            member.visit_mut_children_with(self);
            let result = self
                .convert_private_member(member, PrivateAccessType::Update, None)
                .0;
            node.set_argument(self.ast, b::target(self.ast, result));
            return None;
        }
        node.visit_mut_children_with(self);
        None
    }
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        let node = if let Some(chain) = node.as_chain_expression(self.ast) {
            self.lower_private_chain(chain)
        } else {
            node
        };
        if self.options.private_as_properties {
            if let Some(member) = node.as_member_expression(self.ast)
                && let Some(result) = self.loose(member)
            {
                return Some(result);
            }
            node.visit_mut_children_with(self);
            return Some(node);
        }
        if let Some(assign) = node.as_assignment_expression(self.ast) {
            if let Some(target) = assign.left(self.ast).as_simple_assignment_target(self.ast)
                && let Some(member) = target.as_member_expression(self.ast)
            {
                return Some(self.assignment(assign, member));
            }
            // SWC visits a destructuring RHS before its assignment pattern.
            if let Some(right) = assign.right(self.ast).visit_mut_with(self) {
                assign.set_right(self.ast, right);
            }
            if let Some(left) = assign.left(self.ast).visit_mut_with(self) {
                assign.set_left(self.ast, left);
            }
            return Some(node);
        }
        if let Some(call) = node.as_call_expression(self.ast)
            && let Some(member) = call.callee(self.ast).as_member_expression(self.ast)
        {
            member.visit_mut_children_with(self);
            let args = call.arguments(self.ast);
            let args = args.visit_mut_with(self).unwrap_or(args);
            let (callee, receiver) =
                self.convert_private_member(member, PrivateAccessType::Get, None);
            let (callee, args) = if let Some(receiver) = receiver {
                let callee = b::member(self.ast, callee, "call");
                let mut arguments = Vec::with_capacity(args.len() + 1);
                arguments.push(Argument::Expr(receiver));
                arguments.extend(args.iter().map(|slot| self.ast.get_node_in_sub_range(slot)));
                (callee, self.ast.add_sub_range(arguments))
            } else {
                (callee, args)
            };
            call.set_callee(self.ast, callee);
            call.set_arguments(self.ast, args);
            return Some(node);
        }
        if let Some(tagged) = node.as_tagged_template_expression(self.ast)
            && let Some(member) = tagged.tag(self.ast).as_member_expression(self.ast)
        {
            member.visit_mut_children_with(self);
            tagged.quasi(self.ast).visit_mut_with(self);
            let (tag, receiver) = self.convert_private_member(member, PrivateAccessType::Get, None);
            let tag = if let Some(receiver) = receiver {
                let callee = b::member(self.ast, tag, "bind");
                b::call(self.ast, callee, [receiver], false)
            } else {
                tag
            };
            tagged.set_tag(self.ast, tag);
            return Some(node);
        }
        if let Some(member) = node.as_member_expression(self.ast) {
            member.visit_mut_children_with(self);
            return Some(
                self.convert_private_member(member, PrivateAccessType::Get, None)
                    .0,
            );
        }
        node.visit_mut_children_with(self);
        Some(node)
    }
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L104>.
pub(super) struct BrandCheckHandler<'a, 'ast, 'r> {
    pub(super) ast: &'a mut Ast<'ast>,
    pub(super) ctx: &'a mut TraverseCtx<'r>,
    pub(super) private: &'a PrivateRecord,
}
impl<'ast> VisitMut<'ast> for BrandCheckHandler<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        node.visit_mut_children_with(self);
        let binary = node.as_binary_expression(self.ast)?;
        if binary.operator(self.ast) != BinaryOperator::In {
            return None;
        }
        let name = binary.left(self.ast).as_private_identifier(self.ast)?;
        let name = self.ast.get_utf8(name.name(self.ast));
        let current = self.private.curr_class();
        let right = binary.right(self.ast);
        if right.as_identifier_reference(self.ast).is_some_and(|id| {
            self.ctx
                .resolver
                .same_symbol(id.node_id(), current.class_name.node_id())
        }) {
            let left = self.ctx.resolver.reference(self.ast, current.class_name);
            return Some(self.ast.expr_binary_expression(
                DUMMY_SP,
                left,
                right,
                BinaryOperator::StrictEqual,
            ));
        }
        let private = self.private.get(name);
        if private.ident[name].is_static {
            let class = self.ctx.resolver.reference(self.ast, private.class_name);
            return Some(self.ast.expr_binary_expression(
                DUMMY_SP,
                right,
                class,
                BinaryOperator::StrictEqual,
            ));
        }
        let binding = private.binding(self.ast, self.ctx, &format!("_{name}"));
        let reference = self.ctx.resolver.reference(self.ast, binding);
        let callee = b::member(self.ast, reference, "has");
        Some(b::call(self.ast, callee, [right], false))
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L566>.
pub(super) fn visit_private_in_expr(
    expr: Expr,
    mut visitor: PrivateAccessVisitor<'_, '_, '_>,
) -> (Expr, Vec<VariableDeclarator>) {
    let expr = expr.visit_mut_with(&mut visitor).unwrap_or(expr);
    (expr, visitor.vars)
}

// SWC Atom::is_reserved_in_any, used for moved private-method declarations.
pub(super) fn is_reserved(name: &str) -> bool {
    matches!(
        name,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

fn optional_spine(ast: &Ast<'_>, expression: Expr) -> bool {
    if let Some(member) = expression.as_member_expression(ast) {
        member.optional(ast) || optional_spine(ast, member.object(ast))
    } else if let Some(call) = expression.as_call_expression(ast) {
        call.optional(ast) || optional_spine(ast, call.callee(ast))
    } else {
        false
    }
}
fn private_spine(ast: &Ast<'_>, expression: Expr) -> bool {
    if let Some(member) = expression.as_member_expression(ast) {
        b::private_name(ast, member.property(ast)).is_some()
            || private_spine(ast, member.object(ast))
    } else if let Some(call) = expression.as_call_expression(ast) {
        private_spine(ast, call.callee(ast))
    } else {
        false
    }
}
