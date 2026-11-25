use std::{iter, mem};

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::*;

use crate::common::{TraverseCtx, build};

use self::metadata::{Metadata, ParamMetadata, remove_span};
use super::contains_decorator;

mod metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnumKind {
    Mixed,
    Str,
    Num,
}

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L24>.
pub(crate) fn transform(ast: &mut Ast<'_>, ctx: &mut TraverseCtx<'_>, metadata: bool) {
    let mut enums = FxHashMap::default();
    let program = ast.root_program();
    {
        let mut collector = EnumCollector {
            ast,
            enums: &mut enums,
        };
        program.visit_with(&mut collector);
    }

    let mut pass = TscDecorator {
        ast,
        ctx,
        metadata,
        enums,
        vars: Vec::new(),
        appended_exprs: Vec::new(),
        appended_private_access_exprs: Vec::new(),
        prepended_exprs: Vec::new(),
        class_name: None,
        assign_class_expr_to: None,
    };
    program.visit_mut_with(&mut pass);
}

struct EnumCollector<'a, 'ast> {
    ast: &'a Ast<'ast>,
    enums: &'a mut FxHashMap<String, EnumKind>,
}

impl<'ast> Visit<'ast> for EnumCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L220>.
    fn visit_ts_enum_declaration(&mut self, decl: TsEnumDeclaration) {
        let enum_kind = decl
            .body(self.ast)
            .members(self.ast)
            .iter()
            .map(|slot| {
                let member = self.ast.get_node_in_sub_range(slot);
                match member.initializer(self.ast) {
                    Some(expr) => match self.ast.expr_data(expr) {
                        ExprData::UnaryExpression(unary)
                            if unary.operator(self.ast) == UnaryOperator::Negate =>
                        {
                            EnumKind::Num
                        }
                        ExprData::StringLiteral(_) => EnumKind::Str,
                        ExprData::NumericLiteral(_) => EnumKind::Num,
                        _ => EnumKind::Mixed,
                    },
                    None => EnumKind::Num,
                }
            })
            .fold(None, |opt: Option<EnumKind>, item| {
                if item == EnumKind::Mixed {
                    return Some(EnumKind::Mixed);
                }
                match opt {
                    Some(EnumKind::Mixed) => Some(EnumKind::Mixed),
                    Some(prev) if prev == item => Some(item),
                    Some(_) => Some(EnumKind::Mixed),
                    None => Some(item),
                }
            });

        if let Some(kind) = enum_kind {
            let name = self
                .ast
                .get_utf8(decl.id(self.ast).name(self.ast))
                .to_owned();
            self.enums.insert(name, kind);
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L38>.
struct TscDecorator<'a, 'ast, 'r> {
    ast: &'a mut Ast<'ast>,
    ctx: &'a mut TraverseCtx<'r>,
    metadata: bool,
    enums: FxHashMap<String, EnumKind>,

    /// Used for computed keys and class expressions; variables are uninitialized.
    vars: Vec<VariableDeclarator>,
    appended_exprs: Vec<Expr>,
    appended_private_access_exprs: Vec<Expr>,
    prepended_exprs: Vec<Expr>,

    class_name: Option<BindingIdentifier>,
    assign_class_expr_to: Option<BindingIdentifier>,
}

impl TscDecorator<'_, '_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L55>.
    fn visit_mut_stmt_list(&mut self, stmts: SubRange<Stmt>) -> SubRange<Stmt> {
        let old_vars = mem::take(&mut self.vars);
        let old_appended = mem::take(&mut self.appended_exprs);
        let old_prepended = mem::take(&mut self.prepended_exprs);

        let items: Vec<_> = stmts
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let mut new_stmts = Vec::with_capacity(items.len());

        for stmt in items {
            debug_assert!(self.appended_exprs.is_empty());

            let stmt = stmt.visit_mut_with(self).unwrap_or(stmt);

            if !self.vars.is_empty() {
                let vars = mem::take(&mut self.vars);
                new_stmts.push(build::declaration(self.ast, VariableKind::Var, vars));
            }

            for expr in self.prepended_exprs.drain(..) {
                new_stmts.push(self.ast.stmt_expression_statement(DUMMY_SP, expr));
            }

            new_stmts.push(stmt);

            for expr in self.appended_exprs.drain(..) {
                new_stmts.push(self.ast.stmt_expression_statement(DUMMY_SP, expr));
            }
        }

        self.prepended_exprs = old_prepended;
        self.appended_exprs = old_appended;
        self.vars = old_vars;

        self.ast.add_sub_range(new_stmts)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L119>.
    fn key(&mut self, key: PropertyKey, computed: bool) -> (Expr, Option<PropertyKey>) {
        if computed
            && let Some(expr) = key.as_expr(self.ast)
            && !is_simple_lit(self.ast, expr)
        {
            let var_name = self.ctx.resolver.private_binding(self.ast, "_key");
            self.vars.push(build::declarator(self.ast, var_name, None));

            let lhs = self.ctx.resolver.reference(self.ast, var_name);
            self.prepended_exprs
                .push(build::assign(self.ast, lhs, expr));

            let replacement_key = self.ctx.resolver.reference(self.ast, var_name);
            let key_arg = self.ctx.resolver.reference(self.ast, var_name);
            return (key_arg, Some(PropertyKey::Expr(replacement_key)));
        }

        match self.ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(ident) => {
                let name = self.ast.get_utf8(ident.name(self.ast)).to_owned();
                (build::string(self.ast, &name), None)
            }
            PropertyKeyData::Expr(expr) => {
                let cloned = build::clone_expr(self.ast, self.ctx.resolver, expr);
                (cloned, None)
            }
        }
    }

    fn has_private_access(ast: &Ast<'_>, mut expr: Expr) -> bool {
        while let Some(member) = expr.as_member_expression(ast) {
            if build::private_name(ast, member.property(ast)).is_some() {
                return true;
            }
            expr = member.object(ast);
        }
        false
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L173>.
    fn add_decorate_call(
        &mut self,
        decorators: impl IntoIterator<Item = Expr>,
        target: Expr,
        key: Expr,
        desc: Expr,
    ) {
        let mut has_private_access = false;
        let elems: Vec<_> = decorators
            .into_iter()
            .map(|expr| {
                if !has_private_access {
                    has_private_access = Self::has_private_access(self.ast, expr);
                }
                remove_span(self.ast, expr);
                Some(Argument::Expr(expr))
            })
            .collect();
        let elements = self.ast.add_sub_range(elems);
        let decorators = self.ast.expr_array_expression(DUMMY_SP, elements);

        remove_span(self.ast, target);
        remove_span(self.ast, desc);

        let callee = self.ctx.helper(self.ast, "ts_decorate");
        let call = build::call(self.ast, callee, [decorators, target, key, desc], false);

        if has_private_access {
            self.appended_private_access_exprs.push(call);
        } else {
            self.appended_exprs.push(call);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L265>.
    fn visit_class_inner(&mut self, cls: Class) {
        let appended_private = mem::take(&mut self.appended_private_access_exprs);

        {
            let mut param_meta = ParamMetadata {
                ast: self.ast,
                ctx: self.ctx,
            };
            cls.visit_mut_with(&mut param_meta);
        }

        if self.metadata {
            let class_name = self
                .class_name
                .map(|id| self.ast.get_utf8(id.name(self.ast)).to_owned());
            let mut meta = Metadata::new(self.ast, self.ctx, &self.enums, class_name);
            cls.visit_mut_with(&mut meta);
        }

        cls.visit_mut_children_with(self);

        let appended_private =
            mem::replace(&mut self.appended_private_access_exprs, appended_private);

        if !appended_private.is_empty() {
            let expr = build::sequence(self.ast, appended_private);
            let stmt = self.ast.stmt_expression_statement(DUMMY_SP, expr);
            let block_body = self.ast.add_sub_range([stmt]);
            let static_block =
                ClassElement::StaticBlock(self.ast.static_block(DUMMY_SP, block_body));
            let body = cls.body(self.ast);
            let mut members: Vec<_> = body
                .body(self.ast)
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot))
                .collect();
            members.push(static_block);
            let range = self.ast.add_sub_range(members);
            body.set_body(self.ast, range);
        }

        if let Some(class_name) = self.class_name {
            let decs = cls.decorators(self.ast);
            if !decs.is_empty() {
                let elems: Vec<_> = decs
                    .iter()
                    .map(|slot| {
                        let expr = self.ast.get_node_in_sub_range(slot).expression(self.ast);
                        remove_span(self.ast, expr);
                        Some(Argument::Expr(expr))
                    })
                    .collect();
                cls.set_decorators(self.ast, SubRange::empty());

                let elements = self.ast.add_sub_range(elems);
                let decorators = self.ast.expr_array_expression(DUMMY_SP, elements);
                let arg_class = self.ctx.resolver.reference(self.ast, class_name);
                let callee = self.ctx.helper(self.ast, "ts_decorate");
                let decorated = build::call(self.ast, callee, [decorators, arg_class], false);

                let lhs = self.ctx.resolver.reference(self.ast, class_name);
                self.appended_exprs
                    .push(build::assign(self.ast, lhs, decorated));
            }
        }
    }
}

fn is_simple_lit(ast: &Ast<'_>, expr: Expr) -> bool {
    matches!(
        ast.expr_data(expr),
        ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
    )
}

impl<'ast> VisitMut<'ast> for TscDecorator<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_program(&mut self, program: Program) -> Option<Program> {
        let body = self.visit_mut_stmt_list(program.body(self.ast));
        program.set_body(self.ast, body);
        None
    }

    fn visit_mut_block_statement(&mut self, block: BlockStatement) -> Option<BlockStatement> {
        let body = self.visit_mut_stmt_list(block.body(self.ast));
        block.set_body(self.ast, body);
        None
    }

    fn visit_mut_function_body(&mut self, body: FunctionBody) -> Option<FunctionBody> {
        let stmts = self.visit_mut_stmt_list(body.body(self.ast));
        body.set_body(self.ast, stmts);
        None
    }

    fn visit_mut_static_block(&mut self, block: StaticBlock) -> Option<StaticBlock> {
        let stmts = self.visit_mut_stmt_list(block.body(self.ast));
        block.set_body(self.ast, stmts);
        None
    }

    fn visit_mut_switch_case(&mut self, case: SwitchCase) -> Option<SwitchCase> {
        if let Some(test) = case.test(self.ast)
            && let Some(replacement) = test.visit_mut_with(self)
        {
            case.set_test(self.ast, Some(replacement));
        }
        let consequent = self.visit_mut_stmt_list(case.consequent(self.ast));
        case.set_consequent(self.ast, consequent);
        None
    }

    fn visit_mut_ts_module_block(&mut self, block: TsModuleBlock) -> Option<TsModuleBlock> {
        let body = self.visit_mut_stmt_list(block.body(self.ast));
        block.set_body(self.ast, body);
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L348>.
    fn visit_mut_decl(&mut self, decl: Decl) -> Option<Decl> {
        if let Some(cls) = decl.as_class(self.ast) {
            let old = self.class_name;
            self.class_name = cls.id(self.ast);
            self.visit_class_inner(cls);
            self.class_name = old;
            return None;
        }
        decl.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L405>.
    fn visit_mut_export_default_declaration(
        &mut self,
        export: ExportDefaultDeclaration,
    ) -> Option<ExportDefaultDeclaration> {
        let decl = export.declaration(self.ast);
        if let Some(cls) = decl.as_class(self.ast) {
            if !contains_decorator(self.ast, cls) {
                return None;
            }
            let ident = cls.id(self.ast).unwrap_or_else(|| {
                let id = self.ctx.resolver.private_binding(self.ast, "_class");
                cls.set_id(self.ast, Some(id));
                id
            });
            let old = self.class_name.replace(ident);
            self.visit_class_inner(cls);
            self.class_name = old;
            // `export default class` is not an expression.
            self.assign_class_expr_to = None;
            return None;
        }
        export.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L356>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        let appended_exprs = mem::take(&mut self.appended_exprs);

        let expr = if let Some(cls) = expr.as_class(self.ast) {
            if contains_decorator(self.ast, cls) {
                let ident = cls.id(self.ast).unwrap_or_else(|| {
                    let id = self.ctx.resolver.private_binding(self.ast, "_class");
                    cls.set_id(self.ast, Some(id));
                    id
                });
                let old = self.class_name.replace(ident);
                self.visit_class_inner(cls);
                self.assign_class_expr_to = Some(ident);
                self.class_name = old;
            }
            expr
        } else {
            expr.visit_mut_children_with(self).unwrap_or(expr)
        };

        let appended_exprs = mem::replace(&mut self.appended_exprs, appended_exprs);

        if let Some(var_name) = self.assign_class_expr_to.take() {
            self.vars.push(build::declarator(self.ast, var_name, None));

            let lhs = self.ctx.resolver.reference(self.ast, var_name);
            let assign = build::assign(self.ast, lhs, expr);
            let tail = self.ctx.resolver.reference(self.ast, var_name);
            let exprs: Vec<_> = iter::once(assign)
                .chain(appended_exprs)
                .chain(iter::once(tail))
                .collect();
            return Some(build::sequence(self.ast, exprs));
        }

        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L411>.
    fn visit_mut_method_definition(
        &mut self,
        method: MethodDefinition,
    ) -> Option<MethodDefinition> {
        method.visit_mut_children_with(self);

        if method.kind(self.ast) == MethodDefinitionKind::Constructor {
            return None;
        }
        if let Some(class_name) = self.class_name {
            let decs = method.decorators(self.ast);
            if !decs.is_empty() {
                let (key_arg, new_key) = self.key(method.key(self.ast), method.computed(self.ast));
                if let Some(new_key) = new_key {
                    method.set_key(self.ast, new_key);
                }

                let target = if method.r#static(self.ast) {
                    self.ctx.resolver.reference(self.ast, class_name)
                } else {
                    let cls_ref = self.ctx.resolver.reference(self.ast, class_name);
                    build::member(self.ast, cls_ref, "prototype")
                };

                let exprs: Vec<_> = decs
                    .iter()
                    .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
                    .collect();
                method.set_decorators(self.ast, SubRange::empty());

                let desc = self.ast.expr_null_literal(DUMMY_SP);
                self.add_decorate_call(exprs, target, key_arg, desc);
            }
        }
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L434>.
    fn visit_mut_property_definition(
        &mut self,
        prop: PropertyDefinition,
    ) -> Option<PropertyDefinition> {
        prop.visit_mut_children_with(self);

        if build::private_name(self.ast, prop.key(self.ast)).is_some() {
            return None;
        }
        if let Some(class_name) = self.class_name {
            let decs = prop.decorators(self.ast);
            if !decs.is_empty() {
                let (key_arg, new_key) = self.key(prop.key(self.ast), prop.computed(self.ast));
                if let Some(new_key) = new_key {
                    prop.set_key(self.ast, new_key);
                }

                let target = if prop.r#static(self.ast) {
                    self.ctx.resolver.reference(self.ast, class_name)
                } else {
                    let cls_ref = self.ctx.resolver.reference(self.ast, class_name);
                    build::member(self.ast, cls_ref, "prototype")
                };

                let exprs: Vec<_> = decs
                    .iter()
                    .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
                    .collect();
                prop.set_decorators(self.ast, SubRange::empty());

                let desc = build::undefined(self.ast);
                self.add_decorate_call(exprs, target, key_arg, desc);
            }
        }
        None
    }
}
