use std::{iter, mem};

use crate::common::{TraverseCtx, build as b};
use swc_next_ecma_ast::*;

/// Only the method/static-field mode of SWC SuperFieldAccessFolder is selected
/// by class-properties. Constructor-specific branches belong to the ES2015 pass.
pub(crate) struct SuperFieldAccessFolder<'a, 'ast, 'r> {
    pub ast: &'a mut Ast<'ast>,
    pub ctx: &'a mut TraverseCtx<'r>,
    pub class: BindingIdentifier,
    pub static_: bool,
    pub constant: bool,
    pub superclass: Option<BindingIdentifier>,
    in_pattern: bool,
}
impl<'a, 'ast, 'r> SuperFieldAccessFolder<'a, 'ast, 'r> {
    pub(crate) fn new(
        ast: &'a mut Ast<'ast>,
        ctx: &'a mut TraverseCtx<'r>,
        class: BindingIdentifier,
        static_: bool,
        constant: bool,
        superclass: Option<BindingIdentifier>,
    ) -> Self {
        Self {
            ast,
            ctx,
            class,
            static_,
            constant,
            superclass,
            in_pattern: false,
        }
    }
    fn proto_arg(&mut self) -> Expr {
        let class = self.ctx.resolver.reference(self.ast, self.class);
        let class = if self.static_ {
            class
        } else {
            b::member(self.ast, class, "prototype")
        };
        if self.constant {
            class
        } else {
            let helper = self.ctx.helper(self.ast, "get_prototype_of");
            b::call(self.ast, helper, [class], false)
        }
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_classes/src/super_field.rs#L436>.
    fn super_to_get_call(&mut self, member: MemberExpression) -> Expr {
        if self.constant {
            let base = if let Some(id) = self.superclass {
                self.ctx.resolver.reference(self.ast, id)
            } else {
                self.ctx.resolver.marked_reference(
                    self.ast,
                    if self.static_ { "Function" } else { "Object" },
                    true,
                )
            };
            let base = if self.static_ && self.superclass.is_some() {
                base
            } else {
                b::member(self.ast, base, "prototype")
            };
            self.ast.expr_member_expression(
                DUMMY_SP,
                base,
                member.property(self.ast),
                member.computed(self.ast),
                false,
            )
        } else {
            let proto = self.proto_arg();
            let key = prop_arg(self.ast, member.property(self.ast));
            let this = self.ast.expr_this_expression(DUMMY_SP);
            let helper = self.ctx.helper(self.ast, "get");
            b::call(self.ast, helper, [proto, key, this], false)
        }
    }
    fn super_to_set_call(&mut self, member: MemberExpression, right: Expr) -> Expr {
        let this = self.ast.expr_this_expression(DUMMY_SP);
        if self.constant {
            let target = self.ast.expr_member_expression(
                DUMMY_SP,
                this,
                member.property(self.ast),
                member.computed(self.ast),
                false,
            );
            b::assign(self.ast, target, right)
        } else {
            let proto = self.proto_arg();
            let key = prop_arg(self.ast, member.property(self.ast));
            let true_ = self.ast.expr_boolean_literal(DUMMY_SP, true);
            let helper = self.ctx.helper(self.ast, "set");
            b::call(self.ast, helper, [proto, key, right, this, true_], false)
        }
    }

    fn super_to_update_call(&mut self, member: MemberExpression) -> Expr {
        let proto = self.proto_arg();
        let key = prop_arg(self.ast, member.property(self.ast));
        let this = self.ast.expr_this_expression(DUMMY_SP);
        let true_ = self.ast.expr_boolean_literal(DUMMY_SP, true);
        let helper = self.ctx.helper(self.ast, "update");
        let call = b::call(self.ast, helper, [proto, key, this, true_], false);
        b::member(self.ast, call, "_")
    }
    fn is_super(&self, member: MemberExpression) -> bool {
        member.object(self.ast).is_super(self.ast)
    }
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_classes/src/super_field.rs#L78>.
impl<'ast> VisitMut<'ast> for SuperFieldAccessFolder<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_class_element(&mut self, node: ClassElement) -> Option<ClassElement> {
        match self.ast.class_element_data(node) {
            ClassElementData::MethodDefinition(method) => {
                if let Some(key) = method.key(self.ast).visit_mut_with(self) {
                    method.set_key(self.ast, key);
                }
            }
            ClassElementData::PropertyDefinition(prop) => {
                if let Some(key) = prop.key(self.ast).visit_mut_with(self) {
                    prop.set_key(self.ast, key);
                }
            }
            _ => {}
        }
        None
    }
    fn visit_mut_binding_pattern(&mut self, node: BindingPattern) -> Option<BindingPattern> {
        let old = mem::replace(&mut self.in_pattern, true);
        let replacement = node.visit_mut_children_with(self);
        self.in_pattern = old;
        replacement
    }
    fn visit_mut_simple_assignment_target(
        &mut self,
        node: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        if let Some(member) = node
            .as_member_expression(self.ast)
            .filter(|member| self.is_super(*member))
        {
            let expr = self.super_to_update_call(member);
            return Some(b::target(self.ast, expr));
        }
        node.visit_mut_children_with(self);
        None
    }
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if let Some(assign) = node.as_assignment_expression(self.ast)
            && let Some(member) = assign
                .left(self.ast)
                .as_simple_assignment_target(self.ast)
                .and_then(|target| target.as_member_expression(self.ast))
                .filter(|member| self.is_super(*member))
        {
            let right = assign
                .right(self.ast)
                .visit_mut_with(self)
                .unwrap_or(assign.right(self.ast));
            assign.set_right(self.ast, right);
            if assign.operator(self.ast) != AssignmentOperator::Assign {
                let left = self.super_to_update_call(member);
                assign.set_left(
                    self.ast,
                    AssignmentTarget::SimpleAssignmentTarget(b::target(self.ast, left)),
                );
                return None;
            }
            return Some(self.super_to_set_call(member, right));
        }
        if let Some(call) = node.as_call_expression(self.ast)
            && let Some(member) = call
                .callee(self.ast)
                .as_member_expression(self.ast)
                .filter(|member| self.is_super(*member))
        {
            let args = call
                .arguments(self.ast)
                .visit_mut_with(self)
                .unwrap_or(call.arguments(self.ast));
            let callee = self.super_to_get_call(member);
            let this = self.ast.expr_this_expression(DUMMY_SP);
            let original: Vec<_> = args
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot))
                .collect();
            let rest = if original.len() == 1 {
                original[0]
                    .as_spread_element(self.ast)
                    .map(|spread| spread.argument(self.ast))
                    .filter(|expr| {
                        expr.as_identifier_reference(self.ast)
                            .is_some_and(|id| self.ast.get_utf8(id.name(self.ast)) == "arguments")
                    })
            } else {
                None
            };
            let (callee, args) = if let Some(rest) = rest {
                (
                    b::member(self.ast, callee, "apply"),
                    vec![Argument::Expr(this), Argument::Expr(rest)],
                )
            } else {
                (
                    b::member(self.ast, callee, "call"),
                    iter::once(Argument::Expr(this)).chain(original).collect(),
                )
            };
            let args = self.ast.add_sub_range(args);
            return Some(
                self.ast
                    .expr_call_expression(DUMMY_SP, callee, None, args, false, false),
            );
        }
        if let Some(member) = node
            .as_member_expression(self.ast)
            .filter(|member| self.is_super(*member))
        {
            if let Some(key) = member.property(self.ast).visit_mut_with(self) {
                member.set_property(self.ast, key);
            }
            return Some(if self.in_pattern {
                self.super_to_update_call(member)
            } else {
                self.super_to_get_call(member)
            });
        }
        node.visit_mut_children_with(self);
        None
    }
}

pub(crate) fn prop_arg(ast: &mut Ast<'_>, key: PropertyKey) -> Expr {
    if let Some(name) = key.as_identifier_name(ast) {
        let name = ast.get_utf8(name.name(ast)).to_owned();
        b::string(ast, &name)
    } else {
        key.as_expr(ast).unwrap()
    }
}

/// Rewrites `super` for moved static decorator members without crossing into
/// nested home-object scopes.
/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_classes/src/super_field.rs#L167>.
pub(crate) fn rewrite_super_in_moved_static_member(
    ast: &mut Ast<'_>,
    ctx: &mut TraverseCtx<'_>,
    function: Function,
    class: BindingIdentifier,
) {
    let mut folder = MovedStaticSuperFieldAccessFolder {
        folder: SuperFieldAccessFolder::new(ast, ctx, class, true, false, None),
    };
    let params = function.params(folder.folder.ast);
    params.visit_mut_with(&mut folder);
    let body = function.body(folder.folder.ast);
    body.visit_mut_with(&mut folder);
}

struct MovedStaticSuperFieldAccessFolder<'a, 'ast, 'r> {
    folder: SuperFieldAccessFolder<'a, 'ast, 'r>,
}

impl<'ast> VisitMut<'ast> for MovedStaticSuperFieldAccessFolder<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.folder.ast
    }

    fn visit_mut_class(&mut self, _: Class) -> Option<Class> {
        // `super` inside a nested class belongs to that class.
        None
    }

    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        // Nested regular functions have their own `super` binding rules.
        None
    }

    fn visit_mut_binding_pattern(&mut self, node: BindingPattern) -> Option<BindingPattern> {
        let old = mem::replace(&mut self.folder.in_pattern, true);
        let replacement = node.visit_mut_children_with(self);
        self.folder.in_pattern = old;
        replacement
    }

    fn visit_mut_simple_assignment_target(
        &mut self,
        node: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        if let Some(member) = node
            .as_member_expression(self.folder.ast)
            .filter(|member| self.folder.is_super(*member))
        {
            let expr = self.folder.super_to_update_call(member);
            return Some(b::target(self.folder.ast, expr));
        }
        node.visit_mut_children_with(self);
        None
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if let Some(update) = node.as_update_expression(self.folder.ast)
            && let Some(member) = update
                .argument(self.folder.ast)
                .as_member_expression(self.folder.ast)
                .filter(|member| self.folder.is_super(*member))
        {
            let replacement = self.folder.super_to_update_call(member);
            update.set_argument(self.folder.ast, b::target(self.folder.ast, replacement));
            return None;
        }
        if let Some(assign) = node.as_assignment_expression(self.folder.ast)
            && let Some(member) = assign
                .left(self.folder.ast)
                .as_simple_assignment_target(self.folder.ast)
                .and_then(|target| target.as_member_expression(self.folder.ast))
                .filter(|member| self.folder.is_super(*member))
        {
            let right = assign
                .right(self.folder.ast)
                .visit_mut_with(self)
                .unwrap_or(assign.right(self.folder.ast));
            assign.set_right(self.folder.ast, right);
            if assign.operator(self.folder.ast) != AssignmentOperator::Assign {
                let left = self.folder.super_to_update_call(member);
                assign.set_left(
                    self.folder.ast,
                    AssignmentTarget::SimpleAssignmentTarget(b::target(self.folder.ast, left)),
                );
                return None;
            }
            return Some(self.folder.super_to_set_call(member, right));
        }
        if let Some(call) = node.as_call_expression(self.folder.ast)
            && let Some(member) = call
                .callee(self.folder.ast)
                .as_member_expression(self.folder.ast)
                .filter(|member| self.folder.is_super(*member))
        {
            let args = call
                .arguments(self.folder.ast)
                .visit_mut_with(self)
                .unwrap_or(call.arguments(self.folder.ast));
            let callee = self.folder.super_to_get_call(member);
            let this = self.folder.ast.expr_this_expression(DUMMY_SP);
            let original: Vec<_> = args
                .iter()
                .map(|slot| self.folder.ast.get_node_in_sub_range(slot))
                .collect();
            let rest = if original.len() == 1 {
                original[0]
                    .as_spread_element(self.folder.ast)
                    .map(|spread| spread.argument(self.folder.ast))
                    .filter(|expr| {
                        expr.as_identifier_reference(self.folder.ast)
                            .is_some_and(|id| {
                                self.folder.ast.get_utf8(id.name(self.folder.ast)) == "arguments"
                            })
                    })
            } else {
                None
            };
            let (callee, args) = if let Some(rest) = rest {
                (
                    b::member(self.folder.ast, callee, "apply"),
                    vec![Argument::Expr(this), Argument::Expr(rest)],
                )
            } else {
                (
                    b::member(self.folder.ast, callee, "call"),
                    iter::once(Argument::Expr(this)).chain(original).collect(),
                )
            };
            let args = self.folder.ast.add_sub_range(args);
            return Some(self.folder.ast.expr_call_expression(
                DUMMY_SP, callee, None, args, false, false,
            ));
        }
        if let Some(member) = node
            .as_member_expression(self.folder.ast)
            .filter(|member| self.folder.is_super(*member))
        {
            if let Some(key) = member.property(self.folder.ast).visit_mut_with(self) {
                member.set_property(self.folder.ast, key);
            }
            return Some(if self.folder.in_pattern {
                self.folder.super_to_update_call(member)
            } else {
                self.folder.super_to_get_call(member)
            });
        }
        node.visit_mut_children_with(self);
        None
    }
}
