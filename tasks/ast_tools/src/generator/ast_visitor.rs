use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema, TypeId},
    util::{EXTRA_DATA_ID_BIT_OFFSET, calculate_inline_layout, safe_ident},
};

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
        fn enter_node(&mut self, node_id: NodeId) {}
        #[inline]
        fn leave_node(&mut self, node_id: NodeId) {}
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
                        self.enter_node(node.node_id());
                        <#ty_ident as VisitWith<'a, Self>>::visit_children_with(node, self);
                        self.leave_node(node.node_id());
                    }
                });

                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
                    }
                });
                traverse_functions.extend(quote! {
                    #[inline]
                    fn #enter_fn_name(
                        &mut self,
                        node: #ty_ident,
                        ctx: &mut TraverseCtx<'_, 'a, State>,
                    ) -> TraverseAction {
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
                        <#field_ty_ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
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
                        fn visit_mut_with(self, visitor: &mut V) {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) {
                            #visit_mut_children
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
                    traverse_children.extend(quote! {
                        let child = node.#getter_name(ctx.ast);
                        if <#field_ty_ident as TraverseWith<'a, State>>::traverse_with(
                            child,
                            traverser,
                            ctx,
                        ) == TraverseAction::Stop {
                            break 'traverse_children TraverseAction::Stop;
                        }
                    });
                }

                traverse_walk_functions.extend(quote! {
                    #[inline]
                    fn #walk_fn_name<'a, State, T: ?Sized + Traverse<'a, State>>(
                        node: #ty_ident,
                        traverser: &mut T,
                        ctx: &mut TraverseCtx<'_, 'a, State>,
                    ) -> TraverseAction {
                        let node_id = node.node_id();
                        ctx.push_node(node_id);

                        let mut action = traverser.enter_node(node_id, ctx);
                        if action == TraverseAction::Proceed {
                            action = traverser.#enter_fn_name(node, ctx);
                        }

                        match action {
                            TraverseAction::Skip => {
                                ctx.pop_node(node_id);
                                return TraverseAction::Proceed;
                            }
                            TraverseAction::Stop => {
                                ctx.pop_node(node_id);
                                return TraverseAction::Stop;
                            }
                            TraverseAction::Proceed => {}
                        }

                        let children_action = 'traverse_children: {
                        #traverse_children
                        TraverseAction::Proceed
                        };
                        if children_action == TraverseAction::Stop {
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
                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction {
                            #walk_fn_name(self, traverser, ctx)
                        }
                    }
                });
            }
            AstType::Enum(ast) => {
                let fn_name = format_ident!("visit_{}", &ast.name.to_case(Case::Snake));
                let fn_mut_name = format_ident!("visit_mut_{}", &ast.name.to_case(Case::Snake));
                let data_fn_name = format_ident!("{}_data", &ast.name.to_case(Case::Snake));
                let ty_ident = ty.repr_ident(schema);
                let data_ident = ty.data_ident();

                // Visit/VisitMut
                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitWith<'a, Self>>::visit_children_with(node, self)
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
                    }
                });
                // VisitWith/VisitMutWith
                let mut visit_children_arms = TokenStream::new();
                let mut visit_mut_children_arms = TokenStream::new();
                let mut traverse_arms = TokenStream::new();

                for variant in ast.variants.iter() {
                    let Some(variant_type_id) = variant.type_id else {
                        continue;
                    };

                    let variant_ty = &schema.types[variant_type_id];
                    let variant_ty_ident = variant_ty.repr_ident(schema);
                    let variant_name = format_ident!("{}", variant.name);
                    visit_children_arms.extend(quote! {
                        #data_ident::#variant_name(it) => <#variant_ty_ident as VisitWith<'a, V>>::visit_with(it, visitor),
                    });
                    visit_mut_children_arms.extend(quote! {
                        #data_ident::#variant_name(it) => {
                            <#variant_ty_ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
                        },
                    });
                    traverse_arms.extend(quote! {
                        #data_ident::#variant_name(it) => {
                            <#variant_ty_ident as TraverseWith<'a, State>>::traverse_with(
                                it,
                                traverser,
                                ctx,
                            )
                        },
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
                            match visitor.ast().#data_fn_name(self) {
                                #visit_children_arms
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ident {
                        #[inline]
                        fn visit_mut_with(self, visitor: &mut V) {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) {
                            let data = visitor.ast().#data_fn_name(self);
                            match data {
                                #visit_mut_children_arms
                            }
                        }
                    }
                });

                traverse_with_impls.extend(quote! {
                    impl<'a, State> TraverseWith<'a, State> for #ty_ident {
                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction {
                            match ctx.ast.#data_fn_name(self) {
                                #traverse_arms
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
                    fn #fn_mut_name(&mut self, node: #ty_ident) {
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
                        fn visit_mut_with(self, visitor: &mut V) {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) {
                            if let Some(it) = self {
                                it.visit_mut_with(visitor);
                            }
                        }
                    }
                });

                traverse_with_impls.extend(quote! {
                    impl<'a, State> TraverseWith<'a, State> for #ty_ident {
                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction {
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

                // Visit/VisitMut
                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitWith<'a, Self>>::visit_children_with(node, self)
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) {
                        <#ty_ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
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
                        fn visit_mut_with(self, visitor: &mut V) {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(self, visitor: &mut V) {
                            for child_idx in self.iter() {
                                #get_node
                                child.visit_mut_with(visitor);
                            }
                        }
                    }
                });

                traverse_with_impls.extend(quote! {
                    impl<'a, State> TraverseWith<'a, State> for #ty_ident {
                        #[inline]
                        fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                            self,
                            traverser: &mut T,
                            ctx: &mut TraverseCtx<'_, 'a, State>,
                        ) -> TraverseAction {
                            for child_idx in self.iter() {
                                let child = ctx.ast.get_node_in_sub_range(child_idx);
                                if <#inner_ty_ident as TraverseWith<'a, State>>::traverse_with(
                                    child,
                                    traverser,
                                    ctx,
                                ) == TraverseAction::Stop {
                                    return TraverseAction::Stop;
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

            pub trait VisitMut<'a> {
                #visit_mut_functions
            }

            pub trait VisitMutWith<'a, V: ?Sized + VisitMut<'a>> {
                fn visit_mut_with(self, visitor: &mut V);
                fn visit_mut_children_with(self, visitor: &mut V);
            }

            #visit_mut_with_impls

            pub trait Traverse<'a, State = ()> {
                #[inline]
                fn enter_node(
                    &mut self,
                    node_id: NodeId,
                    ctx: &mut TraverseCtx<'_, 'a, State>,
                ) -> TraverseAction {
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
                fn traverse_with<T: ?Sized + Traverse<'a, State>>(
                    self,
                    traverser: &mut T,
                    ctx: &mut TraverseCtx<'_, 'a, State>,
                ) -> TraverseAction;
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
