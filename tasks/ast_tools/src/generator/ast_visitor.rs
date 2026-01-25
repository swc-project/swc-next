use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema},
    util::safe_ident,
};

pub fn ast_visitor(schema: &Schema) -> RawOutput {
    let mut visit_functions = TokenStream::new();
    let mut visit_with_impls = TokenStream::new();
    let mut visit_mut_functions = TokenStream::new();
    let mut visit_mut_with_impls = TokenStream::new();

    visit_functions.extend(quote! {
        fn ast(&self) -> &Ast;

        #[inline]
        fn enter_node(&mut self, node_id: NodeId) {}
        #[inline]
        fn leave_node(&mut self, node_id: NodeId) {}
    });

    visit_mut_functions.extend(quote! {
        fn ast(&mut self) -> &mut Ast;

        #[inline]
        fn enter_node(&mut self, node_kind: NodeKind) {}
        #[inline]
        fn leave_node(&mut self, node_kind: NodeKind) {}
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
                        self.enter_node(node.node_id());
                        <#ty_ident as VisitWith<Self>>::visit_children_with(node, self);
                        self.leave_node(node.node_id());
                    }
                });

                let node_kind = format_ident!("{}", ast.name);
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> #ty_ident {
                        self.enter_node(NodeKind::#node_kind);
                        let node = <#ty_ident as VisitMutWith<Self>>::visit_mut_children_with(node, self);
                        self.leave_node(NodeKind::#node_kind);
                        node
                    }
                });

                // VisitWith/VisitMutWith
                // Use typed getter methods instead of directly accessing extra_data.
                // This handles both inline and extra_data storage transparently.
                let mut visit_children = TokenStream::new();
                let mut visit_mut_children = TokenStream::new();

                for field in ast.fields.iter() {
                    let field_ty = &schema.types[field.type_id];
                    let field_ty_ident = field_ty.repr_ident(schema);
                    let getter_name = safe_ident(&field.name.to_case(Case::Snake));
                    let setter_name = format_ident!("set_{}", &field.name);

                    visit_children.extend(quote! {
                        let field_value = self.#getter_name(visitor.ast());
                        <#field_ty_ident as VisitWith<V>>::visit_with(field_value, visitor);
                    });
                    visit_mut_children.extend(quote! {
                        let field_value = self.#getter_name(visitor.ast());
                        let new_node = <#field_ty_ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
                        self.#setter_name(visitor.ast(), new_node);
                    });
                }

                visit_with_impls.extend(quote! {
                    impl<V: ?Sized + Visit> VisitWith<V> for #ty_ident {
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit>::#fn_name(visitor, self)
                        }

                        fn visit_children_with(self, visitor: &mut V) {
                            #visit_children
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<V: ?Sized + VisitMut> VisitMutWith<V> for #ty_ident {
                        fn visit_mut_with(self, visitor: &mut V) -> Self {
                            <V as VisitMut>::#fn_mut_name(visitor, self)
                        }

                        fn visit_mut_children_with(self, visitor: &mut V) -> Self {
                            #visit_mut_children
                            self
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
                        <#ty_ident as VisitWith<Self>>::visit_children_with(node, self)
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> #ty_ident {
                        <#ty_ident as VisitMutWith<Self>>::visit_mut_children_with(node, self)
                    }
                });

                // VisitWith/VisitMutWith
                let mut visit_children_arms = TokenStream::new();
                let mut visit_mut_children_arms = TokenStream::new();

                for variant in ast.variants.iter() {
                    let Some(variant_type_id) = variant.type_id else {
                        continue;
                    };

                    let variant_ty = &schema.types[variant_type_id];
                    let variant_ty_ident = variant_ty.repr_ident(schema);
                    let variant_name = format_ident!("{}", variant.name);
                    visit_children_arms.extend(quote! {
                        Self::#variant_name(it) => <#variant_ty_ident as VisitWith<V>>::visit_with(it, visitor),
                    });
                    visit_mut_children_arms.extend(quote! {
                        Self::#variant_name(it) => {
                            Self::#variant_name(<#variant_ty_ident as VisitMutWith<V>>::visit_mut_with(it, visitor))
                        },
                    });
                }

                visit_with_impls.extend(quote! {
                    impl<V: ?Sized + Visit> VisitWith<V> for #ty_ident {
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit>::#fn_name(visitor, self)
                        }

                        fn visit_children_with(self, visitor: &mut V) {
                            match self {
                                #visit_children_arms
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<V: ?Sized + VisitMut> VisitMutWith<V> for #ty_ident {
                        fn visit_mut_with(self, visitor: &mut V) -> Self {
                            <V as VisitMut>::#fn_mut_name(visitor, self)
                        }

                        fn visit_mut_children_with(self, visitor: &mut V) -> Self {
                            match self {
                                #visit_mut_children_arms
                            }
                        }
                    }
                });
            }
            AstType::Option(ast) => {
                let inner_type = &schema.types[ast.inner_type_id];
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
                        <#ty_ident as VisitWith<Self>>::visit_children_with(node, self);
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> #ty_ident {
                        <#ty_ident as VisitMutWith<Self>>::visit_mut_children_with(node, self)
                    }
                });

                // VisitWith/VisitMutWith
                visit_with_impls.extend(quote! {
                    impl<V: ?Sized + Visit> VisitWith<V> for #ty_ident {
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit>::#fn_name(visitor, self)
                        }

                        fn visit_children_with(self, visitor: &mut V) {
                            match self {
                                Some(it) => it.visit_with(visitor),
                                None => {}
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<V: ?Sized + VisitMut> VisitMutWith<V> for #ty_ident {
                        fn visit_mut_with(self, visitor: &mut V) -> Self {
                            <V as VisitMut>::#fn_mut_name(visitor, self)
                        }

                        fn visit_mut_children_with(self, visitor: &mut V) -> Self {
                            match self {
                                Some(it) => {
                                    Some(it.visit_mut_with(visitor))
                                },
                                None => None,
                            }
                        }
                    }
                });
            }
            AstType::Vec(ast) => {
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
                        <#ty_ident as VisitWith<Self>>::visit_children_with(node, self)
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: #ty_ident) -> #ty_ident {
                        <#ty_ident as VisitMutWith<Self>>::visit_mut_children_with(node, self);
                        node
                    }
                });

                // VisitWith/VisitMutWith
                let get_node =
                    quote! ( let child = visitor.ast().get_node_in_sub_range(child_idx); );
                visit_with_impls.extend(quote! {
                    impl<V: ?Sized + Visit> VisitWith<V> for #ty_ident {
                        fn visit_with(self, visitor: &mut V) {
                            <V as Visit>::#fn_name(visitor, self)
                        }

                        fn visit_children_with(self, visitor: &mut V) {
                            for child_idx in self.iter() {
                                #get_node
                                child.visit_with(visitor);
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<V: ?Sized + VisitMut> VisitMutWith<V> for #ty_ident {
                        fn visit_mut_with(self, visitor: &mut V) -> Self {
                            <V as VisitMut>::#fn_mut_name(visitor, self)
                        }

                        fn visit_mut_children_with(self, visitor: &mut V) -> Self {
                            for child_idx in self.iter() {
                                #get_node
                                let new_child = child.visit_mut_with(visitor);
                                self.replace_slot(visitor.ast(), child_idx, new_child);
                            }
                            self
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
            use swc_core::common::Span;

            pub trait Visit {
                #visit_functions
            }

            pub trait VisitWith<V: ?Sized + Visit> {
                fn visit_with(self, visitor: &mut V);
                fn visit_children_with(self, visitor: &mut V);
            }

            #visit_with_impls

            pub trait VisitMut {
                #visit_mut_functions
            }

            pub trait VisitMutWith<V: ?Sized + VisitMut> {
                fn visit_mut_with(self, visitor: &mut V) -> Self;
                fn visit_mut_children_with(self, visitor: &mut V) -> Self;
            }

            #visit_mut_with_impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_visitor"),
        tokens: output,
    }
    .into()
}
