/// SWC's `class_properties::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Represent private members as ordinary properties instead of WeakMaps
    /// and WeakSets.
    pub private_as_properties: bool,
    /// Assign public fields rather than defining them.
    pub set_public_fields: bool,
    /// Assume a class's superclass does not change.
    pub constant_super: bool,
    /// Assume values cannot be the special `document.all` object.
    pub no_document_all: bool,
    /// Assume property getters have no side effects.
    ///
    /// The singular spelling follows SWC's class-properties configuration.
    pub pure_getter: bool,
}

use crate::common::{TraverseCtx, build as b, constructor, super_field::SuperFieldAccessFolder};
use class_name_tdz::ClassNameTdzFolder;
use member_init::{MemberInit, MemberInitRecord, PrivAccessor, PrivMethod, PrivProp, PubProp};
use private_field::{Private, PrivateKind, PrivateRecord};
use std::collections::HashMap;
use swc_next_ecma_ast::*;
use this_in_static::{NewTargetInProp, ThisInStaticFolder};
use used_name::UsedNameCollector;

mod class_name_tdz;
mod member_init;
mod private_field;
mod this_in_static;
mod used_name;

#[derive(Default)]
struct ClassExtra {
    vars: Vec<VariableDeclarator>,
    lets: Vec<VariableDeclarator>,
    stmts: Vec<Stmt>,
}
impl ClassExtra {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L109>.
    fn merge_with(self, ast: &mut Ast<'_>, output: &mut Vec<Stmt>, class: Stmt) {
        if !self.vars.is_empty() {
            output.push(b::declaration(ast, VariableKind::Var, self.vars));
        }
        if !self.lets.is_empty() {
            output.push(b::declaration(ast, VariableKind::Let, self.lets));
        }
        output.push(class);
        output.extend(self.stmts);
    }
}
struct ClassProperties<'a, 'ast, 'r> {
    ast: &'a mut Ast<'ast>,
    ctx: &'a mut TraverseCtx<'r>,
    options: Config,
    private: PrivateRecord,
    extra: ClassExtra,
    diagnostics: Vec<Diagnostic>,
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L45>.
pub(crate) fn class_properties(
    ast: &mut Ast<'_>,
    ctx: &mut TraverseCtx<'_>,
    options: Config,
) -> Vec<Diagnostic> {
    struct ShouldWork<'a, 'ast> {
        ast: &'a Ast<'ast>,
        found: bool,
    }
    impl<'ast> Visit<'ast> for ShouldWork<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }
        fn visit_method_definition(&mut self, _: MethodDefinition) {
            self.found = true;
        }
        fn visit_property_definition(&mut self, node: PropertyDefinition) {
            if !node.accessor(self.ast) {
                self.found = true;
            }
        }
    }
    let program = ast.root_program();
    let mut finder = ShouldWork { ast, found: false };
    program.visit_with(&mut finder);
    if !finder.found {
        return Vec::new();
    }
    let mut pass = ClassProperties {
        ast,
        ctx,
        options,
        private: PrivateRecord::default(),
        extra: ClassExtra::default(),
        diagnostics: Vec::new(),
    };
    program.visit_mut_with(&mut pass);
    pass.diagnostics
}
impl<'ast, 'r> ClassProperties<'_, 'ast, 'r> {
    fn accessor(&mut self) -> private_field::PrivateAccessVisitor<'_, 'ast, 'r> {
        private_field::PrivateAccessVisitor {
            ast: self.ast,
            ctx: self.ctx,
            private: &self.private,
            options: self.options,
            vars: Vec::new(),
        }
    }
    fn super_fields(
        &mut self,
        class: BindingIdentifier,
        static_: bool,
        superclass: Option<BindingIdentifier>,
    ) -> SuperFieldAccessFolder<'_, 'ast, 'r> {
        SuperFieldAccessFolder::new(
            self.ast,
            self.ctx,
            class,
            static_,
            self.options.constant_super,
            superclass,
        )
    }
    fn binding(&mut self, name: &str) -> BindingIdentifier {
        self.private.curr_class().binding(self.ast, self.ctx, name)
    }
    fn weak(&mut self, binding: BindingIdentifier, collection: &str) -> VariableDeclarator {
        let init = if self.options.private_as_properties {
            let helper = self.ctx.helper(self.ast, "class_private_field_loose_key");
            let name = self.ast.get_utf8(binding.name(self.ast)).to_owned();
            let name = b::string(self.ast, &name);
            b::call(self.ast, helper, [name], true)
        } else {
            let callee = self
                .ctx
                .resolver
                .marked_reference(self.ast, collection, true);
            Expr::NewExpression(
                self.ast
                    .new_expression(DUMMY_SP, callee, None, SubRange::empty()),
            )
        };
        b::declarator(self.ast, binding, Some(init))
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L418>.
    fn visit_mut_class_as_decl(&mut self, class: Class, class_id: BindingIdentifier) -> ClassExtra {
        let mut names: HashMap<String, PrivateKind> = HashMap::new();
        for slot in class.body(self.ast).body(self.ast).iter() {
            let member = self.ast.get_node_in_sub_range(slot);
            let (key, kind) = match self.ast.class_element_data(member) {
                ClassElementData::MethodDefinition(method) => (
                    method.key(self.ast),
                    PrivateKind {
                        is_method: true,
                        is_static: method.r#static(self.ast),
                        has_getter: method.kind(self.ast) == MethodDefinitionKind::Get,
                        has_setter: method.kind(self.ast) == MethodDefinitionKind::Set,
                    },
                ),
                ClassElementData::PropertyDefinition(prop) if !prop.accessor(self.ast) => (
                    prop.key(self.ast),
                    PrivateKind {
                        is_static: prop.r#static(self.ast),
                        ..Default::default()
                    },
                ),
                _ => continue,
            };
            if let Some(name) = b::private_name(self.ast, key) {
                if let Some(previous) = names.get_mut(&name) {
                    let pair = previous.is_method
                        && kind.is_method
                        && previous.is_static == kind.is_static
                        && ((kind.has_getter && !previous.has_getter && previous.has_setter)
                            || (kind.has_setter && !previous.has_setter && previous.has_getter));
                    if pair {
                        previous.has_getter |= kind.has_getter;
                        previous.has_setter |= kind.has_setter;
                    } else {
                        let mut diagnostic =
                            Diagnostic::error("duplicate private name", key.span(self.ast));
                        diagnostic.help = Some(name.into());
                        self.diagnostics.push(diagnostic);
                    }
                } else {
                    names.insert(name, kind);
                }
            }
        }
        let scope = self.ctx.resolver.fresh_scope();
        self.private.push(Private {
            class_name: class_id,
            scope,
            ident: names,
        });
        class.visit_mut_children_with(self);
        let body = class.body(self.ast);
        body.visit_mut_with(&mut private_field::BrandCheckHandler {
            ast: self.ast,
            ctx: self.ctx,
            private: &self.private,
        });
        let members: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let cache_method_names =
            members
                .iter()
                .any(|member| match self.ast.class_element_data(*member) {
                    ClassElementData::PropertyDefinition(prop) => !prop.accessor(self.ast),
                    ClassElementData::StaticBlock(_) => true,
                    ClassElementData::MethodDefinition(method)
                        if method.kind(self.ast) != MethodDefinitionKind::Constructor
                            && b::private_name(self.ast, method.key(self.ast)).is_none() =>
                    {
                        contains_super(self.ast, method.key(self.ast))
                    }
                    _ => false,
                });
        let mut extra = ClassExtra::default();
        let mut constructor_inits = MemberInitRecord::default();
        let mut static_inits = MemberInitRecord::default();
        let mut functions = Vec::new();
        let mut output = Vec::new();
        let mut constructor = None;
        let mut used_keys = Vec::new();
        let mut superclass = None;
        for member in members {
            match self.ast.class_element_data(member) {
                ClassElementData::MethodDefinition(method)
                    if method.kind(self.ast) == MethodDefinitionKind::Constructor =>
                {
                    constructor = Some(method);
                }
                ClassElementData::MethodDefinition(method) => {
                    if let Some(name) = b::private_name(self.ast, method.key(self.ast)) {
                        let fn_name = match method.kind(self.ast) {
                            MethodDefinitionKind::Get => format!("get_{name}"),
                            MethodDefinitionKind::Set => format!("set_{name}"),
                            _ if private_field::is_reserved(&name) => format!("__{name}"),
                            _ => name.clone(),
                        };
                        let function_id = self.binding(&fn_name);
                        let binding = self.binding(&format!("_{name}"));
                        let static_ = method.r#static(self.ast);
                        let record = if static_ {
                            &mut static_inits
                        } else {
                            &mut constructor_inits
                        };
                        let collection = match method.kind(self.ast) {
                            MethodDefinitionKind::Get | MethodDefinitionKind::Set => {
                                let getter = method.kind(self.ast) == MethodDefinitionKind::Get;
                                let inserted = record.push(
                                    self.ast,
                                    MemberInit::PrivAccessor(PrivAccessor {
                                        name: binding,
                                        getter: getter.then_some(function_id),
                                        setter: (!getter).then_some(function_id),
                                    }),
                                );
                                (inserted && (!static_ || self.options.private_as_properties))
                                    .then_some("WeakMap")
                            }
                            _ => {
                                if !static_ || self.options.private_as_properties {
                                    record.push(
                                        self.ast,
                                        MemberInit::PrivMethod(PrivMethod {
                                            name: binding,
                                            fn_name: function_id,
                                        }),
                                    );
                                    Some("WeakSet")
                                } else {
                                    None
                                }
                            }
                        };
                        if let Some(collection) = collection {
                            extra.vars.push(self.weak(binding, collection));
                        }
                        let function = method.value(self.ast);
                        function
                            .visit_mut_with(&mut self.super_fields(class_id, static_, superclass));
                        function.set_id(self.ast, Some(function_id));
                        function.set_function_type(self.ast, FunctionType::FunctionDeclaration);
                        functions.push(Stmt::Declaration(Decl::Function(function)));
                    } else {
                        if method.computed(self.ast) && cache_method_names {
                            let mut key = method.key(self.ast).as_expr(self.ast).unwrap();
                            if !b::is_literal(self.ast, key) {
                                let (expression, vars) =
                                    private_field::visit_private_in_expr(key, self.accessor());
                                key = expression;
                                extra.vars.extend(vars);
                                key = key
                                    .visit_mut_with(&mut ClassNameTdzFolder {
                                        ast: self.ast,
                                        ctx: self.ctx,
                                        class_name: class_id,
                                    })
                                    .unwrap_or(key);
                                let name = b::alias_ident_for(self.ast, key, "tmp");
                                let binding = self.ctx.resolver.private_binding(self.ast, &name);
                                extra.lets.push(b::declarator(self.ast, binding, Some(key)));
                                let reference = self.ctx.resolver.reference(self.ast, binding);
                                method.set_key(self.ast, PropertyKey::Expr(reference));
                            }
                        }
                        output.push(member);
                    }
                }
                ClassElementData::PropertyDefinition(prop) if !prop.accessor(self.ast) => {
                    let private = b::private_name(self.ast, prop.key(self.ast));
                    if private.is_none() {
                        let key = prop
                            .key(self.ast)
                            .visit_mut_with(&mut ClassNameTdzFolder {
                                ast: self.ast,
                                ctx: self.ctx,
                                class_name: class_id,
                            })
                            .unwrap_or(prop.key(self.ast));
                        prop.set_key(self.ast, key);
                        if !prop.r#static(self.ast) {
                            let mut collector = UsedNameCollector {
                                ast: self.ast,
                                used_names: Vec::new(),
                            };
                            key.visit_with(&mut collector);
                            used_keys.extend(collector.used_names);
                        }
                        if prop.computed(self.ast) {
                            let key = prop.key(self.ast).as_expr(self.ast).unwrap();
                            if !b::is_literal(self.ast, key) {
                                let (key, vars) =
                                    private_field::visit_private_in_expr(key, self.accessor());
                                extra.vars.extend(vars);
                                let aliased = !key.is_identifier_reference(self.ast)
                                    || key.as_identifier_reference(self.ast).is_some_and(|id| {
                                        used_keys.iter().any(|name| {
                                            name == self.ast.get_utf8(id.name(self.ast))
                                        })
                                    });
                                let key = if aliased {
                                    let name = b::alias_ident_for(self.ast, key, "_ref");
                                    let binding =
                                        self.ctx.resolver.private_binding(self.ast, &name);
                                    extra.lets.push(b::declarator(self.ast, binding, Some(key)));
                                    self.ctx.resolver.reference(self.ast, binding)
                                } else {
                                    key
                                };
                                prop.set_key(self.ast, PropertyKey::Expr(key));
                            }
                        }
                    }
                    let mut value = prop
                        .value(self.ast)
                        .unwrap_or_else(|| b::undefined(self.ast));
                    value = value
                        .visit_mut_with(&mut NewTargetInProp { ast: self.ast })
                        .unwrap_or(value);
                    if private.is_none() {
                        let (expression, vars) =
                            private_field::visit_private_in_expr(value, self.accessor());
                        value = expression;
                        extra.vars.extend(vars);
                    }
                    if prop.r#static(self.ast) {
                        if private.is_none()
                            && superclass.is_none()
                            && let Some(super_) = class.super_class(self.ast)
                        {
                            let binding = if let Some(id) = super_.as_identifier_reference(self.ast)
                            {
                                self.ctx
                                    .resolver
                                    .binding(self.ast, self.ctx.resolver.symbol_of(id.node_id()))
                            } else {
                                let name = b::alias_ident_for(self.ast, super_, "_ref");
                                let binding = self.ctx.resolver.private_binding(self.ast, &name);
                                extra.vars.push(b::declarator(self.ast, binding, None));
                                let reference = self.ctx.resolver.reference(self.ast, binding);
                                let assignment = b::assign(self.ast, reference, super_);
                                class.set_super_class(self.ast, Some(assignment));
                                binding
                            };
                            superclass = Some(binding);
                        }
                        value = value
                            .visit_mut_with(&mut self.super_fields(class_id, true, superclass))
                            .unwrap_or(value);
                    }
                    if private.is_some() {
                        let (expression, vars) =
                            private_field::visit_private_in_expr(value, self.accessor());
                        value = expression;
                        extra.vars.extend(vars);
                    }
                    if prop.r#static(self.ast) {
                        value = value
                            .visit_mut_with(&mut ThisInStaticFolder {
                                ast: self.ast,
                                resolver: self.ctx.resolver,
                                ident: class_id,
                            })
                            .unwrap_or(value);
                    }
                    let init = if let Some(name) = private {
                        if prop.r#static(self.ast)
                            && self.ctx.static_blocks.contains(&prop.node_id())
                        {
                            static_inits.push(self.ast, MemberInit::StaticBlock(value));
                            continue;
                        }
                        let binding = self.binding(&format!("_{name}"));
                        if self.options.private_as_properties || !prop.r#static(self.ast) {
                            extra.vars.push(self.weak(binding, "WeakMap"));
                        }
                        MemberInit::PrivProp(PrivProp {
                            name: binding,
                            value,
                        })
                    } else {
                        MemberInit::PubProp(PubProp {
                            name: prop.key(self.ast),
                            computed: prop.computed(self.ast),
                            value,
                        })
                    };
                    if prop.r#static(self.ast) {
                        static_inits.push(self.ast, init);
                    } else {
                        constructor_inits.push(self.ast, init);
                    }
                }
                _ => output.push(member),
            }
        }
        if !constructor_inits.is_empty() && constructor.is_none() {
            constructor = Some(constructor::default_constructor(
                self.ast,
                self.ctx.resolver,
                class.super_class(self.ast).is_some(),
            ));
        }
        if let Some(constructor) = constructor {
            let expressions = constructor_inits.into_init(self.ast, self.ctx, self.options);
            constructor::inject_after_super(
                self.ast,
                self.ctx.resolver,
                constructor.value(self.ast).body(self.ast),
                expressions,
            );
            output.push(ClassElement::MethodDefinition(constructor));
        }
        for statement in &functions {
            statement.visit_mut_with(&mut self.accessor());
        }
        extra.stmts = static_inits.into_init_static(self.ast, self.ctx, self.options, class_id);
        extra.stmts.extend(functions);
        for member in &mut output {
            if let Some(replacement) = member.visit_mut_with(&mut self.accessor()) {
                *member = replacement;
            }
        }
        let output = self.ast.add_sub_range(output);
        body.set_body(self.ast, output);
        self.private.pop();
        extra
    }

    fn take_initializer(&mut self, declarator: VariableDeclarator) -> Option<Expr> {
        let init = declarator.init(self.ast)?;
        let pattern = declarator.id(self.ast);
        let binding = pattern.as_binding_identifier(self.ast).unwrap();
        let reference = self.ctx.resolver.reference(self.ast, binding);
        let assignment = b::assign(self.ast, reference, init);
        declarator.set_init(self.ast, None);
        Some(assignment)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L338>.
    fn visit_mut_stmt_like(&mut self, body: SubRange<Stmt>) -> SubRange<Stmt> {
        let old = std::mem::take(&mut self.extra);
        let mut output = Vec::new();
        for slot in body.iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            let (class, default_export) = match self.ast.stmt_data(statement) {
                StmtData::Declaration(decl) => (decl.as_class(self.ast), false),
                StmtData::ExportNamedDeclaration(export) => {
                    let class = export
                        .declaration(self.ast)
                        .and_then(|decl| decl.as_class(self.ast));
                    (class, false)
                }
                StmtData::ExportDefaultDeclaration(export) => {
                    (export.declaration(self.ast).as_class(self.ast), true)
                }
                _ => (None, false),
            };
            let Some(class) = class else {
                output.push(statement.visit_mut_with(self).unwrap_or(statement));
                continue;
            };
            if class.declare(self.ast) {
                output.push(statement);
                continue;
            }
            let id = class
                .id(self.ast)
                .unwrap_or_else(|| self.ctx.resolver.private_binding(self.ast, "_class"));
            let extra = self.visit_mut_class_as_decl(class, id);
            if default_export {
                class.set_id(self.ast, Some(id));
                class.set_class_type(self.ast, ClassType::ClassDeclaration);
                extra.merge_with(self.ast, &mut output, Stmt::Declaration(Decl::Class(class)));

                let local = self.ast.identifier_name(DUMMY_SP, id.name(self.ast));
                self.ctx
                    .resolver
                    .copy_context(id.node_id(), local.node_id());
                let default = self.ast.add_utf8("default");
                let default = self.ast.identifier_name(DUMMY_SP, default);
                let specifier = self.ast.export_specifier(
                    DUMMY_SP,
                    ModuleExportName::IdentifierName(local),
                    ModuleExportName::IdentifierName(default),
                    ImportOrExportKind::Value,
                );
                let specifiers = self.ast.add_sub_range([specifier]);
                output.push(Stmt::ExportNamedDeclaration(
                    self.ast.export_named_declaration(
                        DUMMY_SP,
                        None,
                        specifiers,
                        None,
                        SubRange::empty(),
                        ImportOrExportKind::Value,
                    ),
                ));
            } else {
                extra.merge_with(self.ast, &mut output, statement);
            }
        }
        let extra = std::mem::replace(&mut self.extra, old);
        let mut prefix = Vec::new();
        if !extra.lets.is_empty() {
            prefix.push(b::declaration(self.ast, VariableKind::Let, extra.lets));
        }
        if !extra.vars.is_empty() {
            prefix.push(b::declaration(self.ast, VariableKind::Var, extra.vars));
        }
        prefix.extend(output);
        prefix.extend(extra.stmts);
        self.ast.add_sub_range(prefix)
    }
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L140>.
impl<'ast> VisitMut<'ast> for ClassProperties<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_program(&mut self, node: Program) -> Option<Program> {
        let body = self.visit_mut_stmt_like(node.body(self.ast));
        node.set_body(self.ast, body);
        None
    }
    fn visit_mut_block_statement(&mut self, node: BlockStatement) -> Option<BlockStatement> {
        let body = self.visit_mut_stmt_like(node.body(self.ast));
        node.set_body(self.ast, body);
        None
    }
    fn visit_mut_function_body(&mut self, node: FunctionBody) -> Option<FunctionBody> {
        let body = self.visit_mut_stmt_like(node.body(self.ast));
        node.set_body(self.ast, body);
        None
    }
    fn visit_mut_static_block(&mut self, node: StaticBlock) -> Option<StaticBlock> {
        let body = self.visit_mut_stmt_like(node.body(self.ast));
        node.set_body(self.ast, body);
        None
    }
    fn visit_mut_switch_case(&mut self, node: SwitchCase) -> Option<SwitchCase> {
        if let Some(test) = node.test(self.ast) {
            let test = test.visit_mut_with(self).unwrap_or(test);
            node.set_test(self.ast, Some(test));
        }
        let body = self.visit_mut_stmt_like(node.consequent(self.ast));
        node.set_consequent(self.ast, body);
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
        if let Some(class) = node
            .as_expr(self.ast)
            .and_then(|expr| expr.as_class(self.ast))
        {
            let id = class
                .id(self.ast)
                .unwrap_or_else(|| self.ctx.resolver.private_binding(self.ast, "_class"));
            let extra = self.visit_mut_class_as_decl(class, id);
            class.set_id(self.ast, Some(id));
            class.set_class_type(self.ast, ClassType::ClassDeclaration);
            let mut body = Vec::new();
            extra.merge_with(self.ast, &mut body, Stmt::Declaration(Decl::Class(class)));
            let reference = self.ctx.resolver.reference(self.ast, id);
            body.push(self.ast.stmt_return_statement(DUMMY_SP, Some(reference)));
            return Some(ArrowFunctionBody::FunctionBody(b::function_body(
                self.ast, body,
            )));
        }
        node.visit_mut_children_with(self);
        None
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L194>.
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        let Some(class) = node.as_class(self.ast) else {
            node.visit_mut_children_with(self);
            return None;
        };
        let original = class.id(self.ast);
        let name = original.map_or_else(
            || "_class".to_owned(),
            |id| format!("_{}", self.ast.get_utf8(id.name(self.ast))),
        );
        let id = self.ctx.resolver.private_binding(self.ast, &name);
        let extra = self.visit_mut_class_as_decl(class, id);
        let mut expressions = Vec::new();
        for (kind, declarations) in [
            (VariableKind::Var, extra.vars),
            (VariableKind::Let, extra.lets),
        ] {
            for declaration in declarations {
                expressions.extend(self.take_initializer(declaration));
                if kind == VariableKind::Var {
                    self.extra.vars.push(declaration);
                } else {
                    self.extra.lets.push(declaration);
                }
            }
        }
        if extra.stmts.is_empty() {
            expressions.push(node);
        } else {
            self.extra.vars.push(b::declarator(self.ast, id, None));
            let reference = self.ctx.resolver.reference(self.ast, id);
            expressions.push(b::assign(self.ast, reference, node));
            for statement in extra.stmts {
                if let Some(original) = original {
                    replace_class(self.ast, self.ctx.resolver, statement, original, id);
                }
                if let Some(expression) = statement.as_expression_statement(self.ast) {
                    expressions.push(expression.expression(self.ast));
                } else if let Some(declaration) = statement
                    .as_declaration(self.ast)
                    .and_then(|decl| decl.as_variable_declaration(self.ast))
                {
                    for slot in declaration.declarators(self.ast).iter() {
                        let declarator = self.ast.get_node_in_sub_range(slot);
                        expressions.extend(self.take_initializer(declarator));
                        self.extra.vars.push(declarator);
                    }
                } else {
                    self.extra.stmts.push(statement);
                }
            }
            expressions.push(self.ctx.resolver.reference(self.ast, id));
        }
        Some(b::sequence(self.ast, expressions))
    }
}
fn contains_super<'a, 'ast>(ast: &'a Ast<'ast>, key: PropertyKey) -> bool {
    struct SuperVisitor<'a, 'ast> {
        ast: &'a Ast<'ast>,
        found: bool,
    }
    impl<'ast> Visit<'ast> for SuperVisitor<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }
        fn visit_super(&mut self, _: Super) {
            self.found = true;
        }
    }
    let mut finder = SuperVisitor { ast, found: false };
    key.visit_with(&mut finder);
    finder.found
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L267>.
fn replace_class(
    ast: &mut Ast<'_>,
    resolver: &mut crate::common::resolver::context::Resolver,
    statement: Stmt,
    original: BindingIdentifier,
    replacement: BindingIdentifier,
) {
    struct Replacer<'a, 'ast> {
        ast: &'a mut Ast<'ast>,
        resolver: &'a mut crate::common::resolver::context::Resolver,
        original: BindingIdentifier,
        replacement: BindingIdentifier,
    }
    impl<'ast> VisitMut<'ast> for Replacer<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.ast
        }
        fn visit_mut_identifier_reference(
            &mut self,
            id: IdentifierReference,
        ) -> Option<IdentifierReference> {
            if self
                .resolver
                .same_symbol(id.node_id(), self.original.node_id())
            {
                return self
                    .resolver
                    .reference(self.ast, self.replacement)
                    .as_identifier_reference(self.ast);
            }
            None
        }
    }
    statement.visit_mut_with(&mut Replacer {
        ast,
        resolver,
        original,
        replacement,
    });
}
