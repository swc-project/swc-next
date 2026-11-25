use rustc_hash::FxHashMap;
use swc_next_ecma_ast::*;

use crate::common::{TraverseCtx, build};

use super::EnumKind;

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L17>.
pub(super) struct ParamMetadata<'a, 'ast, 'r> {
    pub(super) ast: &'a mut Ast<'ast>,
    pub(super) ctx: &'a mut TraverseCtx<'r>,
}

impl ParamMetadata<'_, '_, '_> {
    fn create_param_decorator(&mut self, param_index: usize, decorator_expr: Expr) -> Decorator {
        remove_span(self.ast, decorator_expr);
        let callee = self.ctx.helper(self.ast, "ts_param");
        let index = self
            .ast
            .expr_numeric_literal(DUMMY_SP, param_index as f64, None);
        let expr = build::call(self.ast, callee, [index, decorator_expr], false);
        self.ast.decorator(DUMMY_SP, expr)
    }

    fn drain_formal_param_decorators(
        &mut self,
        param: FormalParameter,
        idx: usize,
        out: &mut Vec<Decorator>,
    ) {
        let decs = param.decorators(self.ast);
        if decs.is_empty() {
            return;
        }
        let exprs: Vec<_> = decs
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
            .collect();
        param.set_decorators(self.ast, SubRange::empty());
        for expr in exprs {
            out.push(self.create_param_decorator(idx, expr));
        }
    }

    fn drain_param_prop_decorators(
        &mut self,
        prop: TsParameterProperty,
        idx: usize,
        out: &mut Vec<Decorator>,
    ) {
        let decs = prop.decorators(self.ast);
        if !decs.is_empty() {
            let exprs: Vec<_> = decs
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
                .collect();
            prop.set_decorators(self.ast, SubRange::empty());
            for expr in exprs {
                out.push(self.create_param_decorator(idx, expr));
            }
        }
        let param = prop.parameter(self.ast);
        self.drain_formal_param_decorators(param, idx, out);
    }

    fn drain_rest_param_decorators(
        &mut self,
        rest: FormalParameterRest,
        idx: usize,
        out: &mut Vec<Decorator>,
    ) {
        let decs = rest.decorators(self.ast);
        if decs.is_empty() {
            return;
        }
        let exprs: Vec<_> = decs
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot).expression(self.ast))
            .collect();
        rest.set_decorators(self.ast, SubRange::empty());
        for expr in exprs {
            out.push(self.create_param_decorator(idx, expr));
        }
    }
}

impl<'ast> VisitMut<'ast> for ParamMetadata<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L20>.
    fn visit_mut_class(&mut self, cls: Class) -> Option<Class> {
        cls.visit_mut_children_with(self);

        let mut decorators: Vec<_> = cls
            .decorators(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let initial_len = decorators.len();

        let body = cls.body(self.ast).body(self.ast);
        for slot in body.iter() {
            let elem = self.ast.get_node_in_sub_range(slot);
            let Some(method) = elem.as_method_definition(self.ast) else {
                continue;
            };
            if method.kind(self.ast) != MethodDefinitionKind::Constructor {
                continue;
            }
            let params = method.value(self.ast).params(self.ast);
            let items = params.items(self.ast);
            let mut idx = 0;
            for param_slot in items.iter() {
                match self
                    .ast
                    .formal_parameter_item_data(self.ast.get_node_in_sub_range(param_slot))
                {
                    FormalParameterItemData::TsParameterProperty(p) => {
                        self.drain_param_prop_decorators(p, idx, &mut decorators);
                        idx += 1;
                    }
                    FormalParameterItemData::FormalParameter(p) => {
                        self.drain_formal_param_decorators(p, idx, &mut decorators);
                        idx += 1;
                    }
                }
            }
            if let Some(rest) = params.rest(self.ast) {
                self.drain_rest_param_decorators(rest, idx, &mut decorators);
            }
        }

        if decorators.len() != initial_len {
            let range = self.ast.add_sub_range(decorators);
            cls.set_decorators(self.ast, range);
        }
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L54>.
    fn visit_mut_method_definition(&mut self, method: MethodDefinition) -> Option<MethodDefinition> {
        if method.kind(self.ast) == MethodDefinitionKind::Constructor {
            return None;
        }
        let mut decorators: Vec<_> = method
            .decorators(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let initial_len = decorators.len();

        let params = method.value(self.ast).params(self.ast);
        let items = params.items(self.ast);
        let mut idx = 0;
        for param_slot in items.iter() {
            match self
                .ast
                .formal_parameter_item_data(self.ast.get_node_in_sub_range(param_slot))
            {
                FormalParameterItemData::FormalParameter(p) => {
                    self.drain_formal_param_decorators(p, idx, &mut decorators);
                    idx += 1;
                }
                FormalParameterItemData::TsParameterProperty(p) => {
                    self.drain_param_prop_decorators(p, idx, &mut decorators);
                    idx += 1;
                }
            }
        }
        if let Some(rest) = params.rest(self.ast) {
            self.drain_rest_param_decorators(rest, idx, &mut decorators);
        }

        if decorators.len() != initial_len {
            let range = self.ast.add_sub_range(decorators);
            method.set_decorators(self.ast, range);
        }
        None
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L85>.
pub(super) fn remove_span(ast: &mut Ast<'_>, expr: Expr) {
    match ast.expr_data(expr) {
        ExprData::MemberExpression(member) => {
            member.set_span(ast, DUMMY_SP);
            let obj = member.object(ast);
            remove_span(ast, obj);
        }
        ExprData::CallExpression(call) => {
            call.set_span(ast, DUMMY_SP);
            let callee = call.callee(ast);
            remove_span(ast, callee);
            let args = call.arguments(ast);
            for slot in args.iter() {
                let arg = ast.get_node_in_sub_range(slot);
                match ast.argument_data(arg) {
                    ArgumentData::Expr(e) => remove_span(ast, e),
                    ArgumentData::SpreadElement(s) => {
                        let e = s.argument(ast);
                        remove_span(ast, e);
                    }
                }
            }
        }
        _ => {
            expr.set_span(ast, DUMMY_SP);
        }
    }
}

type EnumMap = FxHashMap<String, EnumKind>;

fn get_enum_kind_as_str(
    ast: &Ast<'_>,
    enums: &EnumMap,
    param: Option<TsTypeAnnotation>,
) -> Option<&'static str> {
    let type_ref = param?.type_annotation(ast).as_ts_type_reference(ast)?;
    let ident = type_ref.type_name(ast).as_identifier_reference(ast)?;
    let name = ast.get_utf8(ident.name(ast));
    enums.get(name).map(|kind| match kind {
        EnumKind::Mixed => "Object",
        EnumKind::Str => "String",
        EnumKind::Num => "Number",
    })
}

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L133>.
pub(super) struct Metadata<'a, 'ast, 'r> {
    ast: &'a mut Ast<'ast>,
    ctx: &'a mut TraverseCtx<'r>,
    enums: &'a EnumMap,
    class_name: Option<String>,
}

impl<'a, 'ast, 'r> Metadata<'a, 'ast, 'r> {
    pub(super) fn new(
        ast: &'a mut Ast<'ast>,
        ctx: &'a mut TraverseCtx<'r>,
        enums: &'a EnumMap,
        class_name: Option<String>,
    ) -> Self {
        Self {
            ast,
            ctx,
            enums,
            class_name,
        }
    }

    fn unresolved(&mut self, name: &str) -> Expr {
        self.ctx.resolver.marked_reference(self.ast, name, true)
    }

    fn create_metadata_design_decorator(&mut self, design: &str, type_arg: Expr) -> Decorator {
        let callee = self.ctx.helper(self.ast, "ts_metadata");
        let key = build::string(self.ast, design);
        let expr = build::call(self.ast, callee, [key, type_arg], false);
        self.ast.decorator(DUMMY_SP, expr)
    }

    fn serialize_param_type(&mut self, ann: Option<TsTypeAnnotation>) -> Expr {
        if let Some(kind) = get_enum_kind_as_str(self.ast, self.enums, ann) {
            self.unresolved(kind)
        } else {
            self.serialize_type(ann)
        }
    }

    fn serialize_formal_params(&mut self, params: FormalParameters) -> Expr {
        let items = params.items(self.ast);
        let mut elems = Vec::with_capacity(items.len() + usize::from(params.rest(self.ast).is_some()));
        for slot in items.iter() {
            let ann = match self
                .ast
                .formal_parameter_item_data(self.ast.get_node_in_sub_range(slot))
            {
                FormalParameterItemData::TsParameterProperty(p) => {
                    p.parameter(self.ast).type_annotation(self.ast)
                }
                FormalParameterItemData::FormalParameter(p) => p.type_annotation(self.ast),
            };
            let expr = self.serialize_param_type(ann);
            elems.push(Some(Argument::Expr(expr)));
        }
        if let Some(rest) = params.rest(self.ast) {
            let ann = rest.type_annotation(self.ast);
            let expr = self.serialize_param_type(ann);
            elems.push(Some(Argument::Expr(expr)));
        }
        let elements = self.ast.add_sub_range(elems);
        self.ast.expr_array_expression(DUMMY_SP, elements)
    }

    fn check_object_existed(&mut self, expr: Expr) -> Expr {
        if let Some(member) = expr.as_member_expression(self.ast) {
            let obj = build::clone_expr(self.ast, self.ctx.resolver, member.object(self.ast));
            let left = self.check_object_existed(obj);
            let typeof_expr = self
                .ast
                .expr_unary_expression(DUMMY_SP, expr, UnaryOperator::Typeof);
            let undef_str = build::string(self.ast, "undefined");
            let right = self.ast.expr_binary_expression(
                DUMMY_SP,
                typeof_expr,
                undef_str,
                BinaryOperator::StrictEqual,
            );
            self.ast
                .expr_logical_expression(DUMMY_SP, left, right, LogicalOperator::Or)
        } else {
            let typeof_expr = self
                .ast
                .expr_unary_expression(DUMMY_SP, expr, UnaryOperator::Typeof);
            let undef_str = build::string(self.ast, "undefined");
            self.ast.expr_binary_expression(
                DUMMY_SP,
                typeof_expr,
                undef_str,
                BinaryOperator::StrictEqual,
            )
        }
    }

    fn serialize_bigint_type(&mut self) -> Expr {
        let bigint_test = self.unresolved("BigInt");
        let test = self.check_object_existed(bigint_test);
        let cons = self.unresolved("Object");
        let alt = self.unresolved("BigInt");
        self.ast
            .expr_conditional_expression(DUMMY_SP, test, cons, alt)
    }

    fn ts_entity_to_member_expr(&mut self, type_name: TsTypeName) -> Expr {
        match self.ast.ts_type_name_data(type_name) {
            TsTypeNameData::TsQualifiedName(q) => self.ts_qualified_name_to_member_expr(q),
            TsTypeNameData::IdentifierReference(id) => {
                let reference = self.ast.identifier_reference(DUMMY_SP, id.name(self.ast));
                self.ctx
                    .resolver
                    .copy_context(id.node_id(), reference.node_id());
                Expr::IdentifierReference(reference)
            }
        }
    }

    fn ts_qualified_name_to_member_expr(&mut self, q: TsQualifiedName) -> Expr {
        let left = q.left(self.ast);
        let obj = match self.ast.ts_qualified_name_left_data(left) {
            TsQualifiedNameLeftData::TsQualifiedName(inner) => {
                self.ts_qualified_name_to_member_expr(inner)
            }
            TsQualifiedNameLeftData::IdentifierReference(id) => {
                let reference = self.ast.identifier_reference(DUMMY_SP, id.name(self.ast));
                self.ctx
                    .resolver
                    .copy_context(id.node_id(), reference.node_id());
                Expr::IdentifierReference(reference)
            }
            TsQualifiedNameLeftData::IdentifierName(id) => {
                let reference = self.ast.identifier_reference(DUMMY_SP, id.name(self.ast));
                self.ctx
                    .resolver
                    .copy_context(id.node_id(), reference.node_id());
                Expr::IdentifierReference(reference)
            }
            TsQualifiedNameLeftData::BindingIdentifier(id) => {
                let reference = self.ast.identifier_reference(DUMMY_SP, id.name(self.ast));
                self.ctx
                    .resolver
                    .copy_context(id.node_id(), reference.node_id());
                Expr::IdentifierReference(reference)
            }
            TsQualifiedNameLeftData::ThisExpression(_) => self.ast.expr_this_expression(DUMMY_SP),
        };
        let right = q.right(self.ast);
        let prop = self.ast.identifier_name(DUMMY_SP, right.name(self.ast));
        self.ast.expr_member_expression(
            DUMMY_SP,
            obj,
            PropertyKey::IdentifierName(prop),
            false,
            false,
        )
    }

    fn serialize_type_ref(&mut self, ty: TsTypeReference) -> Expr {
        let type_name = ty.type_name(self.ast);
        if let Some(ident) = type_name.as_identifier_reference(self.ast)
            && self.class_name.as_deref() == Some(self.ast.get_utf8(ident.name(self.ast)))
        {
            return self.unresolved("Object");
        }

        let member_expr = self.ts_entity_to_member_expr(type_name);
        let test_expr = build::clone_expr(self.ast, self.ctx.resolver, member_expr);
        let test = self.check_object_existed(test_expr);
        let cons = self.unresolved("Object");
        self.ast
            .expr_conditional_expression(DUMMY_SP, test, cons, member_expr)
    }

    fn serialize_type_list(&mut self, types: &[TsType]) -> Expr {
        let mut unified: Option<Expr> = None;
        for &ty in types {
            let ty = match self.ast.ts_type_data(ty) {
                TsTypeData::TsParenthesizedType(paren) => paren.type_annotation(self.ast),
                _ => ty,
            };
            match self.ast.ts_type_data(ty) {
                TsTypeData::TsNeverKeyword(_) => continue,
                TsTypeData::TsNullKeyword(_) | TsTypeData::TsUndefinedKeyword(_) => {
                    return self.unresolved("Object");
                }
                _ => {}
            }

            let item = self.serialize_type_node(ty);

            if item
                .as_identifier_reference(self.ast)
                .is_some_and(|id| self.ast.get_utf8(id.name(self.ast)) == "Object")
            {
                return item;
            }

            match unified {
                None => unified = Some(item),
                Some(prev) => match (
                    prev.as_identifier_reference(self.ast),
                    item.as_identifier_reference(self.ast),
                ) {
                    (Some(prev_id), Some(item_id))
                        if self.ast.get_utf8(prev_id.name(self.ast))
                            == self.ast.get_utf8(item_id.name(self.ast)) => {}
                    _ => return self.unresolved("Object"),
                },
            }
        }

        unified.unwrap_or_else(|| self.unresolved("Object"))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L478>.
    fn serialize_type_node(&mut self, ty: TsType) -> Expr {
        if is_str(self.ast, ty) {
            return self.unresolved("String");
        }
        match self.ast.ts_type_data(ty) {
            TsTypeData::TsVoidKeyword(_)
            | TsTypeData::TsUndefinedKeyword(_)
            | TsTypeData::TsNullKeyword(_)
            | TsTypeData::TsNeverKeyword(_) => build::undefined(self.ast),

            TsTypeData::TsParenthesizedType(paren) => {
                let inner = paren.type_annotation(self.ast);
                self.serialize_type_node(inner)
            }

            TsTypeData::TsFunctionType(_) | TsTypeData::TsConstructorType(_) => {
                self.unresolved("Function")
            }

            TsTypeData::TsArrayType(_) | TsTypeData::TsTupleType(_) => self.unresolved("Array"),

            TsTypeData::TsTypePredicate(_) | TsTypeData::TsBooleanKeyword(_) => {
                self.unresolved("Boolean")
            }

            TsTypeData::TsLiteralType(lit) => match self.ast.ts_literal_data(lit.literal(self.ast))
            {
                TsLiteralData::BooleanLiteral(_) => self.unresolved("Boolean"),
                TsLiteralData::NumericLiteral(_) => self.unresolved("Number"),
                TsLiteralData::BigIntLiteral(_) => self.serialize_bigint_type(),
                TsLiteralData::StringLiteral(_) | TsLiteralData::TemplateLiteral(_) => {
                    self.unresolved("String")
                }
                TsLiteralData::UnaryExpression(unary) => {
                    match self.ast.expr_data(unary.argument(self.ast)) {
                        ExprData::NumericLiteral(_) => self.unresolved("Number"),
                        ExprData::BigIntLiteral(_) => self.serialize_bigint_type(),
                        _ => self.unresolved("Object"),
                    }
                }
            },

            TsTypeData::TsTemplateLiteralType(_) => self.unresolved("String"),

            TsTypeData::TsObjectKeyword(_) => self.unresolved("Object"),

            TsTypeData::TsNumberKeyword(_) => self.unresolved("Number"),

            TsTypeData::TsBigIntKeyword(_) => self.serialize_bigint_type(),

            TsTypeData::TsSymbolKeyword(_) => self.unresolved("Symbol"),

            TsTypeData::TsTypeQuery(_)
            | TsTypeData::TsTypeOperator(_)
            | TsTypeData::TsIndexedAccessType(_)
            | TsTypeData::TsTypeLiteral(_)
            | TsTypeData::TsMappedType(_)
            | TsTypeData::TsAnyKeyword(_)
            | TsTypeData::TsUnknownKeyword(_)
            | TsTypeData::TsThisType(_)
            | TsTypeData::TsImportType(_) => self.unresolved("Object"),

            TsTypeData::TsUnionType(union) => {
                let types: Vec<_> = union
                    .types(self.ast)
                    .iter()
                    .map(|slot| self.ast.get_node_in_sub_range(slot))
                    .collect();
                self.serialize_type_list(&types)
            }

            TsTypeData::TsIntersectionType(intersection) => {
                let types: Vec<_> = intersection
                    .types(self.ast)
                    .iter()
                    .map(|slot| self.ast.get_node_in_sub_range(slot))
                    .collect();
                self.serialize_type_list(&types)
            }

            TsTypeData::TsConditionalType(cond) => {
                let types = [cond.true_type(self.ast), cond.false_type(self.ast)];
                self.serialize_type_list(&types)
            }

            TsTypeData::TsTypeReference(type_ref) => self.serialize_type_ref(type_ref),

            _ => self.unresolved("Object"),
        }
    }

    fn serialize_type(&mut self, param: Option<TsTypeAnnotation>) -> Expr {
        let Some(ann) = param else {
            return build::undefined(self.ast);
        };
        let ty = ann.type_annotation(self.ast);
        self.serialize_type_node(ty)
    }
}

fn is_str(ast: &Ast<'_>, ty: TsType) -> bool {
    match ast.ts_type_data(ty) {
        TsTypeData::TsStringKeyword(_) | TsTypeData::TsTemplateLiteralType(_) => true,
        TsTypeData::TsLiteralType(lit) => matches!(
            ast.ts_literal_data(lit.literal(ast)),
            TsLiteralData::StringLiteral(_) | TsLiteralData::TemplateLiteral(_)
        ),
        TsTypeData::TsUnionType(union) => union
            .types(ast)
            .iter()
            .all(|slot| is_str(ast, ast.get_node_in_sub_range(slot))),
        _ => false,
    }
}

impl<'ast> VisitMut<'ast> for Metadata<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L140>.
    fn visit_mut_class(&mut self, cls: Class) -> Option<Class> {
        cls.visit_mut_children_with(self);

        let decs = cls.decorators(self.ast);
        if decs.is_empty() {
            return None;
        }

        let constructor = cls.body(self.ast).body(self.ast).iter().find_map(|slot| {
            let method = self
                .ast
                .get_node_in_sub_range(slot)
                .as_method_definition(self.ast)?;
            (method.kind(self.ast) == MethodDefinitionKind::Constructor).then_some(method)
        });
        let constructor = constructor?;

        let mut decorators: Vec<_> = decs
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();

        let fn_type = self.unresolved("Function");
        decorators.push(self.create_metadata_design_decorator("design:type", fn_type));

        let params = constructor.value(self.ast).params(self.ast);
        let param_types = self.serialize_formal_params(params);
        decorators.push(self.create_metadata_design_decorator("design:paramtypes", param_types));

        let range = self.ast.add_sub_range(decorators);
        cls.set_decorators(self.ast, range);
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L203>.
    fn visit_mut_method_definition(&mut self, method: MethodDefinition) -> Option<MethodDefinition> {
        let kind = method.kind(self.ast);
        if kind == MethodDefinitionKind::Constructor {
            return None;
        }
        let decs = method.decorators(self.ast);
        if decs.is_empty() {
            return None;
        }

        let mut decorators: Vec<_> = decs
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();

        let func = method.value(self.ast);
        let type_arg = match kind {
            MethodDefinitionKind::Method => self.unresolved("Function"),
            MethodDefinitionKind::Get => {
                let return_type = func.return_type(self.ast);
                self.serialize_param_type(return_type)
            }
            MethodDefinitionKind::Set => {
                let first_ann = func.params(self.ast).items(self.ast).iter().next().and_then(
                    |slot| match self
                        .ast
                        .formal_parameter_item_data(self.ast.get_node_in_sub_range(slot))
                    {
                        FormalParameterItemData::FormalParameter(p) => p.type_annotation(self.ast),
                        FormalParameterItemData::TsParameterProperty(p) => {
                            p.parameter(self.ast).type_annotation(self.ast)
                        }
                    },
                );
                self.serialize_type(first_ann)
            }
            MethodDefinitionKind::Constructor => unreachable!(),
        };
        decorators.push(self.create_metadata_design_decorator("design:type", type_arg));

        let params = func.params(self.ast);
        let param_types = self.serialize_formal_params(params);
        decorators.push(self.create_metadata_design_decorator("design:paramtypes", param_types));

        if kind == MethodDefinitionKind::Method {
            let return_arg = if func.r#async(self.ast) {
                self.unresolved("Promise")
            } else {
                let return_type = func.return_type(self.ast);
                self.serialize_param_type(return_type)
            };
            decorators.push(self.create_metadata_design_decorator("design:returntype", return_arg));
        }

        let range = self.ast.add_sub_range(decorators);
        method.set_decorators(self.ast, range);
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L278>.
    fn visit_mut_property_definition(
        &mut self,
        prop: PropertyDefinition,
    ) -> Option<PropertyDefinition> {
        let decs = prop.decorators(self.ast);
        let type_ann = prop.type_annotation(self.ast);
        if decs.is_empty() || type_ann.is_none() {
            return None;
        }
        // Skip private fields.
        if build::private_name(self.ast, prop.key(self.ast)).is_some() {
            return None;
        }

        let mut decorators: Vec<_> = decs
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let type_arg = self.serialize_param_type(type_ann);
        decorators.push(self.create_metadata_design_decorator("design:type", type_arg));

        let range = self.ast.add_sub_range(decorators);
        prop.set_decorators(self.ast, range);
        None
    }
}
