use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema, TypeId},
};

pub fn ast_property(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                impls.extend(generate_span_for_struct(ast_struct, schema));
            }
            AstType::Enum(ast_enum) => {
                impls.extend(generate_property_for_enum(ast_enum, schema));
                impls.extend(generate_span_for_enum(ast_enum, schema));
            }
            _ => {}
        }
    }

    let output = quote! {
        #![allow(unused, clippy::useless_conversion, clippy::identity_op)]
        use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
        use swc_experimental_allocator::boxed::Box;
        use crate::*;
        use crate::span::{DUMMY_SP, GetSpan, SetSpan};

        #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_property"),
        tokens: output,
    }
    .into()
}

fn generate_property_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };
    let type_generics = impl_generics.clone();

    let mut is_variant = TokenStream::new();
    let mut as_variant = TokenStream::new();

    for variant in ast.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);
        let is_fn_name = format_ident!("is_{}", variant.name.to_case(Case::Snake));
        is_variant.extend(quote! {
            #[inline]
            pub fn #is_fn_name(&self) -> bool {
                matches!(self, Self::#variant_name(_))
            }
        });

        let Some(payload_ty_id) = variant.type_id else {
            continue;
        };

        let as_fn_name = format_ident!("as_{}", variant.name.to_case(Case::Snake));
        let payload_ty = type_ref(payload_ty_id, schema);
        as_variant.extend(quote! {
            #[inline]
            pub fn #as_fn_name(self) -> Option<Box<'a, #payload_ty>> {
                match self {
                    Self::#variant_name(it) => Some(it),
                    _ => None,
                }
            }
        });
    }

    quote! {
        impl #impl_generics #name #type_generics {
            #is_variant
            #as_variant
        }
    }
}

fn generate_span_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };
    let type_generics = impl_generics.clone();

    let get_span = if struct_has_span(ast, schema) {
        quote!(self.span)
    } else {
        quote!(DUMMY_SP)
    };

    let set_span = if struct_has_span(ast, schema) {
        quote!(self.span = span;)
    } else {
        TokenStream::new()
    };

    quote! {
        impl #impl_generics GetSpan for #name #type_generics {
            #[inline]
            fn span(&self) -> Span {
                #get_span
            }
        }

        impl #impl_generics SetSpan for #name #type_generics {
            #[inline]
            fn set_span(&mut self, span: Span) {
                #set_span
            }
        }
    }
}

fn generate_span_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };
    let type_generics = impl_generics.clone();

    let mut get_arms = TokenStream::new();
    let mut set_arms = TokenStream::new();
    for variant in ast.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);

        if variant.type_id.is_some() {
            get_arms.extend(quote! {
                Self::#variant_name(it) => it.span(),
            });
            set_arms.extend(quote! {
                Self::#variant_name(it) => it.set_span(span),
            });
        } else {
            get_arms.extend(quote! {
                Self::#variant_name => DUMMY_SP,
            });
            set_arms.extend(quote! {
                Self::#variant_name => {}
            });
        }
    }

    quote! {
        impl #impl_generics GetSpan for #name #type_generics {
            #[inline]
            fn span(&self) -> Span {
                match self {
                    #get_arms
                }
            }
        }

        impl #impl_generics SetSpan for #name #type_generics {
            #[inline]
            fn set_span(&mut self, span: Span) {
                match self {
                    #set_arms
                }
            }
        }
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
            let inner_ty = type_ref(ast.inner_type_id, schema);
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
        "swc_experimental_num_bigint::BigInt" => quote!(swc_experimental_num_bigint::BigInt<'a>),
        "ScopeId" => quote!(ScopeId),
        "SymbolId" => quote!(SymbolId),
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

fn struct_has_span(ast: &AstStruct, schema: &Schema) -> bool {
    ast.fields
        .iter()
        .any(|field| field.name == "span" && is_span_type(field.type_id, schema))
}

fn is_span_type(type_id: TypeId, schema: &Schema) -> bool {
    matches!(&schema.types[type_id], AstType::Primitive(ast) if ast.name == "Span")
}
