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
                            ExtraDataValue::from_extra_data(
                                extra_data.get_unchecked(offset)
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
            }
            AstType::Option(ast) => {
                if !should_visit_type(ast.inner_type_id, schema) {
                    continue;
                }

                let inner_type = &schema.types[ast.inner_type_id];
                let (fn_name, fn_mut_name) = match inner_type {
                    AstType::SubRange(range) => {
                        let inner_type = &schema.types[range.inner_type_id];
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
            }
            AstType::SubRange(ast) => {
                if !should_visit_type(ast.inner_type_id, schema) {
                    continue;
                }

                let inner_type = &schema.types[ast.inner_type_id];
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
            }
            _ => continue,
        };
    }

    let traverse = generate_traverse(schema, false);
    let traverse_mut = generate_traverse(schema, true);

    let output = quote! {
        #![allow(unused, clippy::useless_conversion, clippy::single_match)]
        use crate::*;
        use crate::visit::{TraverseCtx, TraverseMutCtx};

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

            #traverse
            #traverse_mut
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_visitor"),
        tokens: output,
    }
    .into()
}

fn generate_traverse(schema: &Schema, mutable: bool) -> TokenStream {
    let (trait_name, with_trait, with_fn, ctx_ty, walk_prefix, docs) = if mutable {
        (
            format_ident!("TraverseMut"),
            format_ident!("TraverseMutWith"),
            format_ident!("traverse_mut_with"),
            format_ident!("TraverseMutCtx"),
            "walk_mut",
            "Hooks invoked before and after each concrete node, with mutable AST access.",
        )
    } else {
        (
            format_ident!("Traverse"),
            format_ident!("TraverseWith"),
            format_ident!("traverse_with"),
            format_ident!("TraverseCtx"),
            "walk",
            "Hooks invoked before and after each concrete node, with immutable AST access.",
        )
    };
    let mut hooks = TokenStream::new();
    let mut impls = TokenStream::new();
    let mut walks = TokenStream::new();

    for ty in schema.types.iter() {
        let ty_ident = ty.repr_ident(schema);
        if matches!(ty, AstType::Struct(_) | AstType::Enum(_)) {
            let name = ty.name().to_case(Case::Snake);
            let enter_fn = format_ident!("enter_{name}");
            let exit_fn = format_ident!("exit_{name}");
            hooks.extend(quote! {
                #[inline]
                fn #enter_fn(&mut self, node: #ty_ident, ctx: &mut #ctx_ty<'_, 'a, State>) {}

                #[inline]
                fn #exit_fn(&mut self, node: #ty_ident, ctx: &mut #ctx_ty<'_, 'a, State>) {}
            });
        }
        let body = match ty {
            AstType::Struct(ast) => {
                let name = ast.name.to_case(Case::Snake);
                let enter_fn = format_ident!("enter_{name}");
                let exit_fn = format_ident!("exit_{name}");
                let walk_fn = format_ident!("{walk_prefix}_{name}");
                let mut children = TokenStream::new();
                for field in &ast.fields {
                    if !should_visit_type(field.type_id, schema) {
                        continue;
                    }
                    let getter = safe_ident(&field.name.to_case(Case::Snake));
                    let traverse = traverse_child(
                        &schema.types[field.type_id],
                        schema,
                        walk_prefix,
                        quote!(child),
                        true,
                    );
                    children.extend(quote! {
                        let child = node.#getter(ctx.ast);
                        #traverse
                    });
                }

                walks.extend(quote! {
                    #[inline]
                    fn #walk_fn<'a, State, T: ?Sized + #trait_name<'a, State>>(
                        node: #ty_ident,
                        traverser: &mut T,
                        ctx: &mut #ctx_ty<'_, 'a, State>,
                    ) {
                        traverser.#enter_fn(node, ctx);
                        #children
                        traverser.#exit_fn(node, ctx);
                    }
                });
                traverse_child(ty, schema, walk_prefix, quote!(self), true)
            }
            AstType::Enum(ast) => {
                let walk_fn = format_ident!("{walk_prefix}_{}", ast.name.to_case(Case::Snake));
                let mut arms = TokenStream::new();
                let mut seen = HashSet::new();
                for variant in &ast.variants {
                    let Some(type_id) = variant.type_id else {
                        continue;
                    };
                    let variant_ty = &schema.types[type_id];
                    let variant_ident = variant_ty.repr_ident(schema);
                    let traverse = traverse_child(
                        variant_ty,
                        schema,
                        walk_prefix,
                        quote!(#variant_ident(node.0)),
                        false,
                    );
                    let mut node_kinds = Vec::new();
                    for kind in concrete_node_kinds(type_id, schema) {
                        if !seen.insert(kind.clone()) {
                            continue;
                        }
                        let concrete_ty = format_ident!("{kind}");
                        // These variants share a node kind with Expr but are declarations
                        // only when their contextual type says so, just as in VisitWith.
                        if ast.name == "ExportDefaultDeclarationKind"
                            && variant.name == "Expr"
                            && matches!(kind.as_str(), "Function" | "Class")
                        {
                            let concrete_walk =
                                format_ident!("{walk_prefix}_{}", kind.to_case(Case::Snake));
                            let (getter, declaration) = if kind == "Function" {
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
                            arms.extend(quote! {
                                NodeKind::#concrete_ty => {
                                    let concrete = #concrete_ty(node.0);
                                    if concrete.#getter(ctx.ast) == #declaration {
                                        #concrete_walk(concrete, traverser, ctx);
                                    } else {
                                        #traverse
                                    }
                                }
                            });
                        } else {
                            node_kinds.push(concrete_ty);
                        }
                    }
                    if !node_kinds.is_empty() {
                        arms.extend(quote! {
                            #(NodeKind::#node_kinds)|* => { #traverse }
                        });
                    }
                }
                walks.extend(quote! {
                    #[inline]
                    fn #walk_fn<'a, State, T: ?Sized + #trait_name<'a, State>>(
                        node: #ty_ident,
                        traverser: &mut T,
                        ctx: &mut #ctx_ty<'_, 'a, State>,
                    ) {
                        let kind = unsafe { *ctx.ast.nodes.kind_unchecked(node.0) };
                        match kind {
                            #arms
                            _ => unsafe { std::hint::unreachable_unchecked() },
                        }
                    }
                });
                traverse_child(ty, schema, walk_prefix, quote!(self), true)
            }
            AstType::Option(ast) => {
                if !should_visit_type(ast.inner_type_id, schema) {
                    continue;
                }
                traverse_child(ty, schema, walk_prefix, quote!(self), true)
            }
            AstType::SubRange(ast) => {
                if !should_visit_type(ast.inner_type_id, schema) {
                    continue;
                }
                traverse_child(ty, schema, walk_prefix, quote!(self), true)
            }
            AstType::Primitive(_) => continue,
        };
        impls.extend(quote! {
            impl<'a, State> #with_trait<'a, State> for #ty_ident {
                #[inline]
                fn #with_fn<T: ?Sized + #trait_name<'a, State>>(
                    self,
                    traverser: &mut T,
                    ctx: &mut #ctx_ty<'_, 'a, State>,
                ) {
                    #body
                }
            }
        });
    }

    quote! {
        #[doc = #docs]
        /// Children are always traversed between the enter and exit hooks.
        /// Enum hooks follow the field and variant types along the traversal path.
        /// Each node's enter/exit hooks enclose its enum and concrete hooks, even
        /// when nested enums describe the same node. Direct traversal uses the
        /// same ordering for the root node.
        pub trait #trait_name<'a, State = ()> {
            #[inline]
            fn enter_node(&mut self, node_id: NodeId, ctx: &mut #ctx_ty<'_, 'a, State>) {}

            #[inline]
            fn exit_node(&mut self, node_id: NodeId, ctx: &mut #ctx_ty<'_, 'a, State>) {}

            #hooks
        }

        /// Walks AST nodes and invokes their traversal hooks.
        pub trait #with_trait<'a, State = ()> {
            fn #with_fn<T: ?Sized + #trait_name<'a, State>>(
                self,
                traverser: &mut T,
                ctx: &mut #ctx_ty<'_, 'a, State>,
            );
        }

        #impls
        #walks
    }
}

/// Generate the caller's traversal boundary. Enum dispatch passes `false` since
/// its variants describe the same node; fields and public entry points pass `true`.
fn traverse_child(
    ty: &AstType,
    schema: &Schema,
    walk_prefix: &str,
    node: TokenStream,
    node_hooks: bool,
) -> TokenStream {
    match ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            let name = ty.name().to_case(Case::Snake);
            let walk_fn = format_ident!("{walk_prefix}_{name}");
            let walk = quote!(#walk_fn(#node, traverser, ctx););
            let walk = if matches!(ty, AstType::Enum(_)) {
                let enter_fn = format_ident!("enter_{name}");
                let exit_fn = format_ident!("exit_{name}");
                quote! {
                    traverser.#enter_fn(#node, ctx);
                    #walk
                    traverser.#exit_fn(#node, ctx);
                }
            } else {
                walk
            };
            let walk = if node_hooks {
                quote! {
                    let node_id = (#node).node_id();
                    traverser.enter_node(node_id, ctx);
                    #walk
                    traverser.exit_node(node_id, ctx);
                }
            } else {
                walk
            };
            quote! {{
                #walk
            }}
        }
        AstType::Option(ast) => {
            let traverse = traverse_child(
                &schema.types[ast.inner_type_id],
                schema,
                walk_prefix,
                quote!(child),
                node_hooks,
            );
            quote! {
                if let Some(child) = #node {
                    #traverse
                }
            }
        }
        AstType::SubRange(ast) => {
            let traverse = traverse_child(
                &schema.types[ast.inner_type_id],
                schema,
                walk_prefix,
                quote!(child),
                node_hooks,
            );
            quote! {
                for child_idx in #node.iter() {
                    let child = ctx.ast.get_node_in_sub_range(child_idx);
                    #traverse
                }
            }
        }
        AstType::Primitive(_) => TokenStream::new(),
    }
}

fn should_visit_type(type_id: TypeId, schema: &Schema) -> bool {
    match &schema.types[type_id] {
        AstType::Struct(_) | AstType::Enum(_) => true,
        AstType::SubRange(ast) => should_visit_type(ast.inner_type_id, schema),
        AstType::Option(ast) => should_visit_type(ast.inner_type_id, schema),
        AstType::Primitive(_) => false,
    }
}
