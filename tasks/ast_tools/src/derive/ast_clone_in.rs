use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema, TypeId},
};

pub fn ast_clone_in(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                impls.extend(generate_clone_in_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => impls.extend(generate_clone_in_for_enum(ast_enum, schema)),
            _ => continue,
        };
    }

    let output = quote! {
        #![allow(unused)]
        use swc_experimental_allocator::{Allocator, CloneIn};
        use crate::ast::*;

        #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_clone_in"),
        tokens: output,
    }
    .into()
}

fn generate_clone_in_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = impl_generics(ast.type_id, schema);
    let source_ty = source_type_ref(ast.type_id, schema);
    let cloned_ty = cloned_type_ref(ast.type_id, schema);
    let fields = ast.fields.iter().map(|field| {
        let field_ident = format_ident!("{}", field.name);
        quote!(#field_ident: self.#field_ident.clone_in(allocator))
    });

    quote! {
        impl<#impl_generics> CloneIn<'a> for #source_ty {
            type Cloned = #cloned_ty;

            #[inline]
            fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
                #name {
                    #(#fields),*
                }
            }
        }
    }
}

fn generate_clone_in_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = impl_generics(ast.type_id, schema);
    let source_ty = source_type_ref(ast.type_id, schema);
    let cloned_ty = cloned_type_ref(ast.type_id, schema);

    let mut arms = TokenStream::new();
    for variant in ast.variants.iter() {
        let variant_ident = format_ident!("{}", variant.name);
        if variant.type_id.is_some() {
            arms.extend(quote! {
                Self::#variant_ident(it) => #name::#variant_ident(it.clone_in(allocator)),
            });
        } else {
            arms.extend(quote! {
                Self::#variant_ident => #name::#variant_ident,
            });
        }
    }

    quote! {
        impl<#impl_generics> CloneIn<'a> for #source_ty {
            type Cloned = #cloned_ty;

            #[inline]
            fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
                match self {
                    #arms
                }
            }
        }
    }
}

fn impl_generics(type_id: TypeId, schema: &Schema) -> TokenStream {
    if type_has_lifetime(type_id, schema) {
        quote!('a, 'src)
    } else {
        quote!('a)
    }
}

fn source_type_ref(type_id: TypeId, schema: &Schema) -> TokenStream {
    named_ast_type_ref(type_id, quote!('src), schema)
}

fn cloned_type_ref(type_id: TypeId, schema: &Schema) -> TokenStream {
    named_ast_type_ref(type_id, quote!('a), schema)
}

fn named_ast_type_ref(type_id: TypeId, lifetime: TokenStream, schema: &Schema) -> TokenStream {
    match &schema.types[type_id] {
        AstType::Struct(ast) => {
            named_type_ref(&ast.name, lifetime, type_has_lifetime(type_id, schema))
        }
        AstType::Enum(ast) => {
            named_type_ref(&ast.name, lifetime, type_has_lifetime(type_id, schema))
        }
        _ => unreachable!(),
    }
}

fn named_type_ref(name: &str, lifetime: TokenStream, has_lifetime: bool) -> TokenStream {
    let ident = format_ident!("{}", name);
    if has_lifetime {
        quote!(#ident<#lifetime>)
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
