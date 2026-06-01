use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema, TypeId},
};

pub fn ast_visitor(schema: &Schema) -> RawOutput {
    let mut visit_functions = TokenStream::new();
    let mut visit_with_impls = TokenStream::new();
    let mut visit_mut_functions = TokenStream::new();
    let mut visit_mut_with_impls = TokenStream::new();

    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast) => {
                let fn_name = format_ident!("visit_{}", &ast.name.to_case(Case::Snake));
                let fn_mut_name = format_ident!("visit_mut_{}", &ast.name.to_case(Case::Snake));
                let ty_ref = ast_type_ref(ast.type_id, schema);

                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: &#ty_ref) {
                        node.visit_children_with(self);
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: &mut #ty_ref) {
                        node.visit_mut_children_with(self);
                    }
                });

                let mut visit_children = TokenStream::new();
                let mut visit_mut_children = TokenStream::new();

                for field in ast.fields.iter() {
                    if !should_visit_type(field.type_id, schema) {
                        continue;
                    }

                    let field_name = format_ident!("{}", field.name);
                    visit_children.extend(quote! {
                        self.#field_name.visit_with(visitor);
                    });
                    visit_mut_children.extend(quote! {
                        self.#field_name.visit_mut_with(visitor);
                    });
                }

                visit_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for #ty_ref {
                        #[inline]
                        fn visit_with(&self, visitor: &mut V) {
                            <V as Visit<'a>>::#fn_name(visitor, self)
                        }

                        #[inline]
                        fn visit_children_with(&self, visitor: &mut V) {
                            #visit_children
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ref {
                        #[inline]
                        fn visit_mut_with(&mut self, visitor: &mut V) {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(&mut self, visitor: &mut V) {
                            #visit_mut_children
                        }
                    }
                });
            }
            AstType::Enum(ast) => {
                let fn_name = format_ident!("visit_{}", &ast.name.to_case(Case::Snake));
                let fn_mut_name = format_ident!("visit_mut_{}", &ast.name.to_case(Case::Snake));
                let ty_ref = ast_type_ref(ast.type_id, schema);

                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: &#ty_ref) {
                        node.visit_children_with(self);
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: &mut #ty_ref) {
                        node.visit_mut_children_with(self);
                    }
                });

                let mut visit_children_arms = TokenStream::new();
                let mut visit_mut_children_arms = TokenStream::new();

                for variant in ast.variants.iter() {
                    let variant_name = format_ident!("{}", variant.name);
                    if variant.type_id.is_some() {
                        visit_children_arms.extend(quote! {
                            Self::#variant_name(it) => it.visit_with(visitor),
                        });
                        visit_mut_children_arms.extend(quote! {
                            Self::#variant_name(it) => it.visit_mut_with(visitor),
                        });
                    } else {
                        visit_children_arms.extend(quote! {
                            Self::#variant_name => {}
                        });
                        visit_mut_children_arms.extend(quote! {
                            Self::#variant_name => {}
                        });
                    }
                }

                visit_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for #ty_ref {
                        #[inline]
                        fn visit_with(&self, visitor: &mut V) {
                            <V as Visit<'a>>::#fn_name(visitor, self)
                        }

                        #[inline]
                        fn visit_children_with(&self, visitor: &mut V) {
                            match self {
                                #visit_children_arms
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ref {
                        #[inline]
                        fn visit_mut_with(&mut self, visitor: &mut V) {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(&mut self, visitor: &mut V) {
                            match self {
                                #visit_mut_children_arms
                            }
                        }
                    }
                });
            }
            AstType::Vec(ast) => {
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
                let ty_ref = type_ref(ast.type_id, schema);

                visit_functions.extend(quote! {
                    #[inline]
                    fn #fn_name(&mut self, node: &#ty_ref) {
                        node.visit_children_with(self);
                    }
                });
                visit_mut_functions.extend(quote! {
                    #[inline]
                    fn #fn_mut_name(&mut self, node: &mut #ty_ref) {
                        node.visit_mut_children_with(self);
                    }
                });

                visit_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for #ty_ref {
                        #[inline]
                        fn visit_with(&self, visitor: &mut V) {
                            <V as Visit<'a>>::#fn_name(visitor, self)
                        }

                        #[inline]
                        fn visit_children_with(&self, visitor: &mut V) {
                            for node in self {
                                node.visit_with(visitor);
                            }
                        }
                    }
                });
                visit_mut_with_impls.extend(quote! {
                    impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for #ty_ref {
                        #[inline]
                        fn visit_mut_with(&mut self, visitor: &mut V) {
                            <V as VisitMut<'a>>::#fn_mut_name(visitor, self)
                        }

                        #[inline]
                        fn visit_mut_children_with(&mut self, visitor: &mut V) {
                            for node in self {
                                node.visit_mut_with(visitor);
                            }
                        }
                    }
                });
            }
            _ => continue,
        };
    }

    let output = quote! {
        #![allow(unused, clippy::useless_conversion, clippy::single_match)]
        use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
        use swc_experimental_allocator::boxed::Box;
        use swc_experimental_allocator::vec::Vec;
        use crate::*;

        pub trait Visit<'a> {
            #visit_functions
        }

        pub trait VisitWith<'a, V: ?Sized + Visit<'a>> {
            fn visit_with(&self, visitor: &mut V);
            fn visit_children_with(&self, visitor: &mut V);
        }

        impl<'a, T, V> VisitWith<'a, V> for Box<'a, T>
        where
            T: VisitWith<'a, V>,
            V: ?Sized + Visit<'a>,
        {
            #[inline]
            fn visit_with(&self, visitor: &mut V) {
                (**self).visit_with(visitor)
            }

            #[inline]
            fn visit_children_with(&self, visitor: &mut V) {
                (**self).visit_children_with(visitor)
            }
        }

        impl<'a, T, V> VisitWith<'a, V> for Option<T>
        where
            T: VisitWith<'a, V>,
            V: ?Sized + Visit<'a>,
        {
            #[inline]
            fn visit_with(&self, visitor: &mut V) {
                if let Some(node) = self {
                    node.visit_with(visitor);
                }
            }

            #[inline]
            fn visit_children_with(&self, visitor: &mut V) {
                if let Some(node) = self {
                    node.visit_children_with(visitor);
                }
            }
        }

        #visit_with_impls

        pub trait VisitMut<'a> {
            #visit_mut_functions
        }

        pub trait VisitMutWith<'a, V: ?Sized + VisitMut<'a>> {
            fn visit_mut_with(&mut self, visitor: &mut V);
            fn visit_mut_children_with(&mut self, visitor: &mut V);
        }

        impl<'a, T, V> VisitMutWith<'a, V> for Box<'a, T>
        where
            T: VisitMutWith<'a, V>,
            V: ?Sized + VisitMut<'a>,
        {
            #[inline]
            fn visit_mut_with(&mut self, visitor: &mut V) {
                (**self).visit_mut_with(visitor)
            }

            #[inline]
            fn visit_mut_children_with(&mut self, visitor: &mut V) {
                (**self).visit_mut_children_with(visitor)
            }
        }

        impl<'a, T, V> VisitMutWith<'a, V> for Option<T>
        where
            T: VisitMutWith<'a, V>,
            V: ?Sized + VisitMut<'a>,
        {
            #[inline]
            fn visit_mut_with(&mut self, visitor: &mut V) {
                if let Some(node) = self {
                    node.visit_mut_with(visitor);
                }
            }

            #[inline]
            fn visit_mut_children_with(&mut self, visitor: &mut V) {
                if let Some(node) = self {
                    node.visit_mut_children_with(visitor);
                }
            }
        }

        #visit_mut_with_impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_visitor"),
        tokens: output,
    }
    .into()
}

fn ast_type_ref(type_id: TypeId, schema: &Schema) -> TokenStream {
    match &schema.types[type_id] {
        AstType::Struct(ast) => named_type_ref(&ast.name, type_has_lifetime(type_id, schema)),
        AstType::Enum(ast) => named_type_ref(&ast.name, type_has_lifetime(type_id, schema)),
        _ => unreachable!(),
    }
}

fn type_ref(type_id: TypeId, schema: &Schema) -> TokenStream {
    match &schema.types[type_id] {
        AstType::Struct(ast) => named_type_ref(&ast.name, type_has_lifetime(type_id, schema)),
        AstType::Enum(ast) => named_type_ref(&ast.name, type_has_lifetime(type_id, schema)),
        AstType::Box(ast) => {
            let inner_ty = type_ref(ast.inner_type_id, schema);
            quote!(Box<'a, #inner_ty>)
        }
        AstType::Vec(ast) => {
            let inner_ty = type_ref(ast.inner_type_id, schema);
            quote!(Vec<'a, #inner_ty>)
        }
        AstType::Option(ast) => {
            let inner_ty = field_type_ref(ast.inner_type_id, schema);
            quote!(Option<#inner_ty>)
        }
        AstType::Primitive(ast) => primitive_type_ref(ast.name),
    }
}

fn field_type_ref(type_id: TypeId, schema: &Schema) -> TokenStream {
    match &schema.types[type_id] {
        AstType::Struct(_) => {
            let inner_ty = type_ref(type_id, schema);
            quote!(Box<'a, #inner_ty>)
        }
        AstType::Enum(_) => type_ref(type_id, schema),
        AstType::Box(ast) => {
            let inner_ty = type_ref(ast.inner_type_id, schema);
            quote!(Box<'a, #inner_ty>)
        }
        AstType::Vec(ast) => {
            let inner_ty = type_ref(ast.inner_type_id, schema);
            quote!(Vec<'a, #inner_ty>)
        }
        AstType::Option(ast) => {
            let inner_ty = field_type_ref(ast.inner_type_id, schema);
            quote!(Option<#inner_ty>)
        }
        AstType::Primitive(ast) => primitive_type_ref(ast.name),
    }
}

fn primitive_type_ref(name: &str) -> TokenStream {
    match name {
        "Utf8Ref" | "Atom" => quote!(Atom<'a>),
        "Wtf8Ref" | "Wtf8Atom" => quote!(Wtf8Atom<'a>),
        "OptionalUtf8Ref" => quote!(Option<Atom<'a>>),
        "OptionalWtf8Ref" => quote!(Option<Wtf8Atom<'a>>),
        _ => format_ident!("{}", name).into_token_stream(),
    }
}

fn named_type_ref(name: &str, has_lifetime: bool) -> TokenStream {
    let ident = format_ident!("{}", name);
    if has_lifetime {
        quote!(#ident<'a>)
    } else {
        ident.into_token_stream()
    }
}

fn type_has_lifetime(type_id: TypeId, schema: &Schema) -> bool {
    match &schema.types[type_id] {
        AstType::Struct(ast) => ast
            .fields
            .iter()
            .any(|field| field_type_needs_lifetime(field.type_id, schema)),
        AstType::Enum(ast) => ast.variants.iter().any(|variant| variant.type_id.is_some()),
        AstType::Box(_) => true,
        AstType::Vec(_) => true,
        AstType::Option(ast) => field_type_needs_lifetime(ast.inner_type_id, schema),
        AstType::Primitive(ast) => primitive_type_needs_lifetime(ast.name),
    }
}

fn field_type_needs_lifetime(type_id: TypeId, schema: &Schema) -> bool {
    match &schema.types[type_id] {
        AstType::Struct(_) => true,
        AstType::Enum(_) => type_has_lifetime(type_id, schema),
        AstType::Box(_) => true,
        AstType::Vec(_) => true,
        AstType::Option(ast) => field_type_needs_lifetime(ast.inner_type_id, schema),
        AstType::Primitive(ast) => primitive_type_needs_lifetime(ast.name),
    }
}

fn should_visit_type(type_id: TypeId, schema: &Schema) -> bool {
    match &schema.types[type_id] {
        AstType::Struct(_) | AstType::Enum(_) => true,
        AstType::Box(ast) => should_visit_type(ast.inner_type_id, schema),
        AstType::Vec(ast) => should_visit_type(ast.inner_type_id, schema),
        AstType::Option(ast) => should_visit_type(ast.inner_type_id, schema),
        AstType::Primitive(_) => false,
    }
}

fn primitive_type_needs_lifetime(name: &str) -> bool {
    matches!(
        name,
        "Utf8Ref"
            | "Atom"
            | "Wtf8Ref"
            | "Wtf8Atom"
            | "OptionalUtf8Ref"
            | "OptionalWtf8Ref"
            | "swc_experimental_num_bigint::BigInt"
    )
}
