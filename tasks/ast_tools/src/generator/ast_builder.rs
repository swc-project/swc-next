use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema, TypeId},
    util::safe_ident,
};

pub fn ast_builder(schema: &Schema) -> RawOutput {
    let mut build_functions = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                build_functions.extend(generate_build_functions_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => {
                let mut context = RecursiveEnumContext {
                    ret_ty: type_ref(ast_enum.type_id, schema),
                    ..Default::default()
                };
                build_functions.extend(generate_build_functions_for_enum(
                    ast_enum,
                    schema,
                    &mut context,
                ));
            }
            _ => continue,
        };
    }

    let output = quote! {
        #![allow(unused, clippy::useless_conversion, clippy::identity_op)]
        use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
        use swc_experimental_allocator::boxed::Box;
        use swc_experimental_allocator::vec::Vec;
        use crate::*;

        impl<'a> AstBuilder<'a> {
            #build_functions
        }
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_builder"),
        tokens: output,
    }
    .into()
}

fn generate_build_functions_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let fn_name = safe_ident(&ast.name.to_case(Case::Snake));
    let box_fn_name = safe_ident(&format!("box_{}", ast.name.to_case(Case::Snake)));
    let ret_ty = type_ref(ast.type_id, schema);
    let constructor = format_ident!("{}", ast.name);
    let fn_params = generate_fn_params_decl(ast, schema);
    let fn_args = generate_fn_args(ast);
    let fields = generate_struct_fields(ast);

    quote! {
        #[inline]
        pub fn #fn_name(&self, #fn_params) -> #ret_ty {
            #constructor {
                #fields
            }
        }

        #[inline]
        pub fn #box_fn_name(&self, #fn_params) -> Box<'a, #ret_ty> {
            self.allocator.boxed(self.#fn_name(#fn_args))
        }
    }
}

#[derive(Default)]
struct RecursiveEnumContext {
    ret_ty: TokenStream,
    name: Vec<String>,
    constructor: Vec<TokenStream>,
}

fn generate_build_functions_for_enum(
    ast: &AstEnum,
    schema: &Schema,
    recursive_context: &mut RecursiveEnumContext,
) -> TokenStream {
    let mut build_variants = TokenStream::new();

    let enum_name = format_ident!("{}", ast.name);
    recursive_context.name.push(ast.name.clone());

    for variant in ast.variants.iter() {
        let Some(payload_ty_id) = variant.type_id else {
            continue;
        };

        let variant_name = format_ident!("{}", variant.name);
        recursive_context
            .constructor
            .push(quote!( #enum_name::#variant_name ));

        match &schema.types[payload_ty_id] {
            AstType::Struct(ast_struct) => {
                let ret_ty = &recursive_context.ret_ty;
                let fn_name = {
                    let mut fn_name = String::new();
                    for name in recursive_context.name.iter() {
                        fn_name.push_str(name);
                    }
                    fn_name.push_str(&ast_struct.name);
                    safe_ident(&fn_name.to_case(Case::Snake))
                };

                let args = generate_fn_args(ast_struct);
                let box_constructor =
                    safe_ident(&format!("box_{}", ast_struct.name.to_case(Case::Snake)));
                let body = wrap_enum_constructors(
                    quote!( self.#box_constructor(#args) ),
                    &recursive_context.constructor,
                );

                let fn_params = generate_fn_params_decl(ast_struct, schema);
                build_variants.extend(quote! {
                    #[inline]
                    pub fn #fn_name(&self, #fn_params) -> #ret_ty {
                        #body
                    }
                });
            }
            AstType::Enum(inner_enum) => {
                build_variants.extend(generate_build_functions_for_enum(
                    inner_enum,
                    schema,
                    recursive_context,
                ));
            }
            _ => unreachable!(),
        }

        recursive_context.constructor.pop();
    }
    recursive_context.name.pop();

    build_variants
}

fn wrap_enum_constructors(mut payload: TokenStream, constructors: &[TokenStream]) -> TokenStream {
    for (index, constructor) in constructors.iter().rev().enumerate() {
        payload = if index == 0 {
            quote!(#constructor(#payload))
        } else {
            quote!(#constructor(self.allocator.boxed(#payload)))
        };
    }

    payload
}

fn generate_fn_params_decl(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let mut fields = Vec::default();
    for field in ast.fields.iter() {
        if should_default_in_builder(ast, &field.name) {
            continue;
        }

        let field_name = format_ident!("{}", field.name);
        let field_ty = field_type_ref(field.type_id, schema);
        fields.push(quote!(#field_name: #field_ty));
    }

    quote!( #(#fields),* )
}

fn generate_fn_args(ast: &AstStruct) -> TokenStream {
    let mut fields = Vec::default();
    for field in ast.fields.iter() {
        if should_default_in_builder(ast, &field.name) {
            continue;
        }

        let field_name = format_ident!("{}", field.name);
        fields.push(quote!(#field_name));
    }

    quote!( #(#fields),* )
}

fn generate_struct_fields(ast: &AstStruct) -> TokenStream {
    let fields = ast.fields.iter().map(|field| {
        let field_name = format_ident!("{}", field.name);
        if should_default_in_builder(ast, &field.name) {
            quote!(#field_name: Default::default())
        } else {
            quote!(#field_name)
        }
    });

    quote!(#(#fields),*)
}

fn should_default_in_builder(ast: &AstStruct, field_name: &str) -> bool {
    matches!(
        (ast.name.as_str(), field_name),
        ("Ident", "symbol_id") | ("BlockStmt", "scope_id")
    )
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

fn named_type_ref(name: &str, has_lifetime: bool) -> TokenStream {
    let ident = format_ident!("{}", name);
    if has_lifetime {
        quote!(#ident<'a>)
    } else {
        ident.into_token_stream()
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
