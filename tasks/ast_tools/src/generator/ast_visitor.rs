use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema, TypeId},
    util::{EXTRA_DATA_ID_BIT_OFFSET, calculate_inline_layout, safe_ident},
};

use super::ast_node::concrete_node_kinds;

pub fn ast_visitor(schema: &Schema) -> RawOutput {
    let mut visit_functions = TokenStream::new();
    let mut visit_with_impls = TokenStream::new();
    let mut visit_mut_functions = TokenStream::new();
    let mut visit_mut_with_impls = TokenStream::new();
    let mut traverse_functions = TokenStream::new();
    let mut traverse_with_impls = TokenStream::new();
    let mut traverse_walk_functions = TokenStream::new();

    visit_functions.extend(quote! {
        fn ast(&self) -> &Ast<'a>;

        #[inline]
        fn enter_node(&mut self, node: NodeKindData) {}
        #[inline]
        fn leave_node(&mut self, node: NodeKindData) {}
    });

    visit_mut_functions.extend(quote! {
        fn ast(&mut self) -> &mut Ast<'a>;
    });

    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast) => {
                let fn_name = format_ident!("visit_{}", &ast.name.to_case(Case::Snake));
                let fn_mut_name = format_ident!("visit_mut_{}", &ast.name.to_case(Case::Snake));
                let enter_fn_name = format_ident!("enter_{}", &ast.name.to_case(Case::Snake));
                let exit_fn_name = format_ident!("exit_{}", &ast.name.to_case(Case::Snake));
                let walk_fn_name = format_ident!("walk_{}", &ast.name.to_case(Case::Snake));
                let ty_ident = ty.repr_ident(schema);

                // Visit/VisitMut
                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: #ty_ident) {
                        self.enter_node(NodeKindData::#ty_ident(node));
                        <#ty_ident as VisitWith<'a, Self>>::visit_children_with(node, self);
                        self.leave_node(NodeKindData::#ty_ident(node));
                    }
                });

                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> Option<#ty_ident> {
                        <#ty_ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
                    }
                });
                traverse_functions.extend(quote! {
                    #[inline]
                    fn #enter_fn_name(
                        &mut self,
                        node: #ty_ident,
                        ctx: &mut TraverseCtx<'_, 'a, State>,
                    ) -> TraverseAction<#ty_ident> {
                        TraverseAction::Proceed
                    }

                    #[inline]
                    fn #exit_fn_name(
                        &mut self,
                        node: #ty_ident,
                        ctx: &mut TraverseCtx<'_, 'a, State>,
                    ) {
                    }
                });

                // VisitWith/VisitMutWith
                let uses_extra_data = calculate_inline_layout(ast, schema).is_none();
                let mut getter_field_loads = TokenStream::new();
                let mut extra_data_field_loads = TokenStream::new();
                let mut field_values = Vec::new();
                let mut visit_children = TokenStream::new();
                let mut visit_mut_children = TokenStream::new();

                for (field_index, field) in ast.fields.iter().enumerate() {
                    if !should_visit_type(field.type_id, schema) {
                        continue;
                    }

                    let field_ty = &schema.types[field.type_id];
                    let field_ty_ident = field_ty.repr_ident(schema);
                    let getter_name = safe_ident(&field.name.to_case(Case::Snake));
                    let setter_name = format_ident!("set_{}", field.name.to_case(Case::Snake));
                    let field_value = format_ident!("field_value_{field_index}");
                    getter_field_loads.extend(quote! {
                        let #field_value = self.#getter_name(visitor.ast());
                    });
                    extra_data_field_loads.extend(quote! {
                        let offset = extra_data_start.wrapping_add(#field_index);
                        debug_assert!(offset < extra_data.len());
                        let #field_value: #field_ty_ident = unsafe {
                            ExtraDataCompact::from_extra_data(
                                *extra_data.get_unchecked(offset),
                                ast,
                            )
                        };
                    });
                    field_values.push(field_value.clone());
                    visit_children.extend(quote! {
                        <#field_ty_ident as VisitWith<'a, V>>::visit_with(#field_value, visitor);
                    });
                    visit_mut_children.extend(quote! {
                        let field_value = self.#getter_name(visitor.ast());
                        if let Some(field_value) = <#field_ty_ident as VisitMutWith<'a, V>>::visit_mut_with(
                            field_value,
                            visitor,
                        ) {
                            self.#setter_name(visitor.ast(), field_value);
                        }
                    });
                }

                // Keep inline-data access behind the typed getters. For extra-data nodes, decode
                // all visitable fields in one block so the node's base offset and the extra-data
                // slice are loaded once. The block also releases the immutable AST borrow before
                // calling back into the mutable visitor.
                let field_loads = if uses_extra_data && field_values.len() > 1 {
                    quote! {
                        let (#(#field_values,)*) = {
                            let ast = visitor.ast();
                            let extra_data_start = unsafe {
                                (ast.nodes.data_unchecked(self.0).0
                                    >> #EXTRA_DATA_ID_BIT_OFFSET)
                                    as u32 as usize
                            };
                            let extra_data = ast.extra_data.as_slice();
                            #extra_data_field_loads
                            (#(#field_values,)*)
                        };
                    }
                } else {
                    getter_field_loads
                };

                visit_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit<'a>>::#fn_name(visitor, self)
                        }

                        #[inline]
                        fn visit_children_with(self, visitor: &mut V) {
                            #field_loads
                            #visit_children
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_mut_with(self, visitor: &mut V) -> Option<Self> {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) -> Option<Self> {
                            #visit_mut_children
                            None
                        }
                    }
                });

                let mut traverse_children = TokenStream::new();
                for field in ast.fields.iter() {
                    if !should_visit_type(field.type_id, schema) {
                        continue;
                    }

                    let field_ty = &schema.types[field.type_id];
                    let field_ty_ident = field_ty.repr_ident(schema);
                    let getter_name = safe_ident(&field.name.to_case(Case::Snake));
                    let setter_name = format_ident!("set_{}", field.name.to_case(Case::Snake));
                    let replacement = wrap_replacement(field.type_id, quote!(replacement), schema);
                    let replace_arm = if matches!(field_ty, AstType::Vec(_)) {
                        quote! {
                            TraverseAction::Replace(_) => {
                                unreachable!("sub-range traversal consumes child replacements")
                            }
                        }
                    } else {
                        quote! {
                            TraverseAction::Replace(replacement) => {
                                node.#setter_name(ctx.ast, #replacement);
                            }
                        }
                    };
                    traverse_children.extend(quote! {
                        let child = node.#getter_name(ctx.ast);
                        match <#field_ty_ident as TraverseWith<'a, State>>::traverse_with(
                            child,
                            traverser,
                            ctx,
                        ) {
                            TraverseAction::Stop => {
                                break 'traverse_children true;
                            }
                            #replace_arm
                            TraverseAction::Proceed | TraverseAction::Skip => {}
                        }
                    });
                }

                traverse_walk_functions.extend(quote! {
                    #[inline]
                    fn #walk_fn_name<'a, State, T: ?Sized + Traverse<'a, State>>(
                        node: #ty_ident,
                        traverser: &mut T,
                        ctx: &mut TraverseCtx<'_, 'a, State>,
                    ) -> TraverseAction<#ty_ident> {
                        let node_id = node.node_id();
                        ctx.push_node(node_id);

                        let action = match traverser.enter_node::<#ty_ident>(node_id, ctx) {
                            TraverseAction::Proceed => traverser.#enter_fn_name(node, ctx),
                            action => action,
                        };

                        match action {
                            TraverseAction::Skip => {
                                ctx.pop_node(node_id);
                                return TraverseAction::Proceed;
                            }
                            TraverseAction::Stop => {
                                ctx.pop_node(node_id);
                                return TraverseAction::Stop;
                            }
                            TraverseAction::Replace(replacement) => {
                                ctx.pop_node(node_id);
                                return TraverseAction::Replace(replacement);
                            }
                            TraverseAction::Proceed => {}
                        }

                        let children_stopped = 'traverse_children: {
                        #traverse_children
                        false
                        };
                        if children_stopped {
                            ctx.pop_node(node_id);
                            return TraverseAction::Stop;
                        }

                        traverser.#exit_fn_name(node, ctx);
                        traverser.exit_node(node_id, ctx);
                        ctx.pop_node(node_id);
                        TraverseAction::Proceed
                    }
                });
                traverse_with_impls.extend(quote! {
                    impl<'a, State> TraverseWith<'a, State> for #ty_ident {
                        type Node = #ty_ident;

                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction<Self::Node> {
                            #walk_fn_name(self, traverser, ctx)
                        }
                    }
                });
            }
            AstType::Enum(ast) => {
                let fn_name = format_ident!("visit_{}", &ast.name.to_case(Case::Snake));
                let fn_mut_name = format_ident!("visit_mut_{}", &ast.name.to_case(Case::Snake));
                let ty_ident = ty.repr_ident(schema);

                // Visit/VisitMut
                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitWith<'a, Self>>::visit_children_with(node, self)
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> Option<#ty_ident> {
                        <#ty_ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
                    }
                });
                // VisitWith/VisitMutWith
                let mut visit_children_arms = TokenStream::new();
                let mut visit_mut_children_arms = TokenStream::new();
                let mut traverse_arms = TokenStream::new();
                let mut seen_node_kinds = HashSet::new();

                for variant in ast.variants.iter() {
                    let Some(variant_type_id) = variant.type_id else {
                        continue;
                    };

                    let variant_ty = &schema.types[variant_type_id];
                    let variant_ty_ident = variant_ty.repr_ident(schema);
                    let variant_name = format_ident!("{}", variant.name);
                    let mut node_kind_names: Vec<_> = concrete_node_kinds(variant_type_id, schema)
                        .into_iter()
                        .filter(|node_kind| seen_node_kinds.insert(node_kind.clone()))
                        .collect();

                    if ast.name == "ExportDefaultDeclarationKind" && variant.name == "Expr" {
                        for contextual_kind in ["Function", "Class"] {
                            let Some(index) = node_kind_names
                                .iter()
                                .position(|node_kind| node_kind == contextual_kind)
                            else {
                                continue;
                            };
                            node_kind_names.remove(index);

                            let node_kind = format_ident!("{contextual_kind}");
                            let concrete_ty = format_ident!("{contextual_kind}");
                            let (type_getter, declaration_type) = if contextual_kind == "Function" {
                                (
                                    format_ident!("function_type"),
                                    quote!(FunctionType::FunctionDeclaration),
                                )
                            } else {
                                (
                                    format_ident!("class_type"),
                                    quote!(ClassType::ClassDeclaration),
                                )
                            };
                            visit_children_arms.extend(quote! {
                                NodeKind::#node_kind => {
                                    let concrete = #concrete_ty(self.0);
                                    if concrete.#type_getter(visitor.ast()) == #declaration_type {
                                        <#concrete_ty as VisitWith<'a, V>>::visit_with(concrete, visitor)
                                    } else {
                                        let nested = #variant_ty_ident(self.0);
                                        <#variant_ty_ident as VisitWith<'a, V>>::visit_with(nested, visitor)
                                    }
                                }
                            });
                            visit_mut_children_arms.extend(quote! {
                                NodeKind::#node_kind => {
                                    let concrete = #concrete_ty(self.0);
                                    if concrete.#type_getter(visitor.ast()) == #declaration_type {
                                        <#concrete_ty as VisitMutWith<'a, V>>::visit_mut_with(
                                            concrete,
                                            visitor,
                                        ).map(#ty_ident::#node_kind)
                                    } else {
                                        let nested = #variant_ty_ident(self.0);
                                        <#variant_ty_ident as VisitMutWith<'a, V>>::visit_mut_with(
                                            nested,
                                            visitor,
                                        ).map(#ty_ident::#variant_name)
                                    }
                                }
                            });
                            traverse_arms.extend(quote! {
                                NodeKind::#node_kind => {
                                    let concrete = #concrete_ty(self.0);
                                    if concrete.#type_getter(ctx.ast) == #declaration_type {
                                        match <#concrete_ty as TraverseWith<'a, State>>::traverse_with(
                                            concrete,
                                            traverser,
                                            ctx,
                                        ) {
                                            TraverseAction::Proceed => TraverseAction::Proceed,
                                            TraverseAction::Skip => TraverseAction::Skip,
                                            TraverseAction::Stop => TraverseAction::Stop,
                                            TraverseAction::Replace(replacement) => {
                                                TraverseAction::Replace(#ty_ident::#node_kind(replacement))
                                            }
                                        }
                                    } else {
                                        let nested = #variant_ty_ident(self.0);
                                        match <#variant_ty_ident as TraverseWith<'a, State>>::traverse_with(
                                            nested,
                                            traverser,
                                            ctx,
                                        ) {
                                            TraverseAction::Proceed => TraverseAction::Proceed,
                                            TraverseAction::Skip => TraverseAction::Skip,
                                            TraverseAction::Stop => TraverseAction::Stop,
                                            TraverseAction::Replace(replacement) => {
                                                TraverseAction::Replace(#ty_ident::#variant_name(replacement))
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }

                    if node_kind_names.is_empty() {
                        continue;
                    }
                    let node_kinds: Vec<_> = node_kind_names
                        .iter()
                        .map(|node_kind| format_ident!("{node_kind}"))
                        .collect();
                    visit_children_arms.extend(quote! {
                        #(NodeKind::#node_kinds)|* => {
                            let it = #variant_ty_ident(self.0);
                            <#variant_ty_ident as VisitWith<'a, V>>::visit_with(it, visitor)
                        }
                    });
                    visit_mut_children_arms.extend(quote! {
                        #(NodeKind::#node_kinds)|* => {
                            let it = #variant_ty_ident(self.0);
                            <#variant_ty_ident as VisitMutWith<'a, V>>::visit_mut_with(
                                it,
                                visitor,
                            ).map(#ty_ident::#variant_name)
                        }
                    });
                    traverse_arms.extend(quote! {
                        #(NodeKind::#node_kinds)|* => {
                            let it = #variant_ty_ident(self.0);
                            match <#variant_ty_ident as TraverseWith<'a, State>>::traverse_with(
                                it,
                                traverser,
                                ctx,
                            ) {
                                TraverseAction::Proceed => TraverseAction::Proceed,
                                TraverseAction::Skip => TraverseAction::Skip,
                                TraverseAction::Stop => TraverseAction::Stop,
                                TraverseAction::Replace(replacement) => {
                                    TraverseAction::Replace(#ty_ident::#variant_name(replacement))
                                }
                            }
                        }
                    });
                }

                visit_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit<'a>>::#fn_name(visitor, self)
                        }

                        #[inline]
                        fn visit_children_with(self, visitor: &mut V) {
                            let kind = unsafe { *visitor.ast().nodes.kind_unchecked(self.0) };
                            match kind {
                                #visit_children_arms
                                _ => unsafe { std::hint::unreachable_unchecked() },
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_mut_with(self, visitor: &mut V) -> Option<Self> {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) -> Option<Self> {
                            let kind = unsafe { *visitor.ast().nodes.kind_unchecked(self.0) };
                            match kind {
                                #visit_mut_children_arms
                                _ => unsafe { std::hint::unreachable_unchecked() },
                            }
                        }
                    }
                });

                traverse_with_impls.extend(quote! {
                    impl<'a, State> TraverseWith<'a, State> for #ty_ident {
                        type Node = #ty_ident;

                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction<Self::Node> {
                            let kind = unsafe { *ctx.ast.nodes.kind_unchecked(self.0) };
                            match kind {
                                #traverse_arms
                                _ => unsafe { std::hint::unreachable_unchecked() },
                            }
                        }
                    }
                });
            }
            AstType::Option(ast) => {
                if !should_visit_type(ast.inner_type_id, schema) {
                    continue;
                }

                let inner_type = &schema.types[ast.inner_type_id];
                let inner_ty_ident = inner_type.repr_ident(schema);
                let (fn_name, fn_mut_name) = match inner_type {
                    AstType::Vec(vec) => {
                        let inner_type = &schema.types[vec.inner_type_id];
                        let fn_name =
                            format_ident!("visit_opt_{}s", &inner_type.name().to_case(Case::Snake));
                        let fn_mut_name = format_ident!(
                            "visit_mut_opt_{}s",
                            &inner_type.name().to_case(Case::Snake)
                        );
                        (fn_name, fn_mut_name)
                    }
                    _ => {
                        let fn_name =
                            format_ident!("visit_opt_{}", &inner_type.name().to_case(Case::Snake));
                        let fn_mut_name = format_ident!(
                            "visit_mut_opt_{}",
                            &inner_type.name().to_case(Case::Snake)
                        );
                        (fn_name, fn_mut_name)
                    }
                };

                let ty_ident = ty.repr_ident(schema);

                // Visit/VisitMut
                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitWith<'a, Self>>::visit_children_with(node, self);
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> Option<#ty_ident> {
                        <#ty_ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
                    }
                });

                // VisitWith/VisitMutWith
                visit_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit<'a>>::#fn_name(visitor, self)
                        }

                        #[inline]
                        fn visit_children_with(self, visitor: &mut V) {
                            match self {
                                Some(it) => it.visit_with(visitor),
                                None => {}
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_mut_with(self, visitor: &mut V) -> Option<Self> {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) -> Option<Self> {
                            self.and_then(|it| it.visit_mut_with(visitor).map(Some))
                        }
                    }
                });

                traverse_with_impls.extend(quote! {
                    impl<'a, State> TraverseWith<'a, State> for #ty_ident {
                        type Node = <#inner_ty_ident as TraverseWith<'a, State>>::Node;

                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction<Self::Node> {
                            if let Some(it) = self {
                                return <#inner_ty_ident as TraverseWith<'a, State>>::traverse_with(
                                    it,
                                    traverser,
                                    ctx,
                                );
                            }
                            TraverseAction::Proceed
                        }
                    }
                });
            }
            AstType::Vec(ast) => {
                if !should_visit_type(ast.inner_type_id, schema) {
                    continue;
                }

                let inner_type = &schema.types[ast.inner_type_id];
                let inner_ty_ident = inner_type.repr_ident(schema);
                let (fn_name, fn_mut_name) = match inner_type {
                    AstType::Option(opt) => {
                        let inner_type = &schema.types[opt.inner_type_id];
                        let fn_name = format_ident!(
                            "visit_opt_vec_{}s",
                            &inner_type.name().to_case(Case::Snake)
                        );
                        let fn_mut_name = format_ident!(
                            "visit_mut_opt_vec_{}s",
                            &inner_type.name().to_case(Case::Snake)
                        );
                        (fn_name, fn_mut_name)
                    }
                    _ => {
                        let fn_name =
                            format_ident!("visit_{}s", &inner_type.name().to_case(Case::Snake));
                        let fn_mut_name =
                            format_ident!("visit_mut_{}s", &inner_type.name().to_case(Case::Snake));
                        (fn_name, fn_mut_name)
                    }
                };
                let ty_ident = ty.repr_ident(schema);
                let replacement = wrap_replacement(ast.inner_type_id, quote!(replacement), schema);

                // Visit/VisitMut
                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitWith<'a, Self>>::visit_children_with(node, self)
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> Option<#ty_ident> {
                        <#ty_ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
                    }
                });

                // VisitWith/VisitMutWith
                let get_node =
                    quote! ( let child = visitor.ast().get_node_in_sub_range(child_idx); );
                visit_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit<'a>>::#fn_name(visitor, self)
                        }

                        #[inline]
                        fn visit_children_with(self, visitor: &mut V) {
                            for child_idx in self.iter() {
                                #get_node
                                child.visit_with(visitor);
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_mut_with(self, visitor: &mut V) -> Option<Self> {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) -> Option<Self> {
                            for child_idx in self.iter() {
                                #get_node
                                if let Some(replacement) = child.visit_mut_with(visitor) {
                                    self.replace_slot(visitor.ast(), child_idx, replacement);
                                }
                            }
                            None
                        }
                    }
                });

                traverse_with_impls.extend(quote! {
                    impl<'a, State> TraverseWith<'a, State> for #ty_ident {
                        type Node = <#inner_ty_ident as TraverseWith<'a, State>>::Node;

                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction<Self::Node> {
                            for child_idx in self.iter() {
                                let child = ctx.ast.get_node_in_sub_range(child_idx);
                                match <#inner_ty_ident as TraverseWith<'a, State>>::traverse_with(
                                    child,
                                    traverser,
                                    ctx,
                                ) {
                                    TraverseAction::Stop => return TraverseAction::Stop,
                                    TraverseAction::Replace(replacement) => {
                                        self.replace_slot(ctx.ast, child_idx, #replacement);
                                    }
                                    TraverseAction::Proceed | TraverseAction::Skip => {}
                                }
                            }
                            TraverseAction::Proceed
                        }
                    }
                });
            }
            _ => continue,
        };
    }

    let output = quote! {
        #![allow(unused, clippy::useless_conversion, clippy::single_match)]
        use crate::*;
        use crate::visit::{TraverseAction, TraverseCtx};

            pub trait Visit<'a> {
                #visit_functions
            }

            pub trait VisitWith<'a, V: ?Sized + Visit<'a>> {
                fn visit_with(self, visitor: &mut V);
                fn visit_children_with(self, visitor: &mut V);
            }

            #visit_with_impls

            /// Mutably visits AST nodes.
            ///
            /// A visit method returns `Some(node)` to replace the node in its parent, or `None`
            /// to keep the existing node. Child replacements are applied directly to their
            /// containing node while walking it.
            pub trait VisitMut<'a> {
                #visit_mut_functions
            }

            /// Dispatches mutable visits and reports an optional replacement for `self`.
            pub trait VisitMutWith<'a, V: ?Sized + VisitMut<'a>>: Sized {
                fn visit_mut_with(self, visitor: &mut V) -> Option<Self>;
                fn visit_mut_children_with(self, visitor: &mut V) -> Option<Self>;
            }

            #visit_mut_with_impls

            pub trait Traverse<'a, State = ()> {
                #[inline]
                fn enter_node<T: AstNode>(
                    &mut self,
                    node_id: NodeId,
                    ctx: &mut TraverseCtx<'_, 'a, State>,
                ) -> TraverseAction<T> {
                    TraverseAction::Proceed
                }

                #[inline]
                fn exit_node(
                    &mut self,
                    node_id: NodeId,
                    ctx: &mut TraverseCtx<'_, 'a, State>,
                ) {
                }

                #traverse_functions
            }

            pub trait TraverseWith<'a, State = ()> {
                type Node: AstNode;

                fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                    self,
                    traverser: &mut T,
                    ctx: &mut TraverseCtx<'_, 'a, State>,
                ) -> TraverseAction<Self::Node>;
            }

            #traverse_with_impls

            #traverse_walk_functions
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_visitor"),
        tokens: output,
    }
    .into()
}

fn should_visit_type(type_id: TypeId, schema: &Schema) -> bool {
    match &schema.types[type_id] {
        AstType::Struct(_) | AstType::Enum(_) => true,
        AstType::Vec(ast) => should_visit_type(ast.inner_type_id, schema),
        AstType::Option(ast) => should_visit_type(ast.inner_type_id, schema),
        AstType::Primitive(_) => false,
    }
}

fn wrap_replacement(type_id: TypeId, replacement: TokenStream, schema: &Schema) -> TokenStream {
    match &schema.types[type_id] {
        AstType::Option(ast) => {
            let replacement = wrap_replacement(ast.inner_type_id, replacement, schema);
            quote!(Some(#replacement))
        }
        AstType::Struct(_) | AstType::Enum(_) | AstType::Vec(_) => replacement,
        AstType::Primitive(_) => unreachable!("primitive fields are not traversed"),
    }
}
