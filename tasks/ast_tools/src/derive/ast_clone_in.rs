use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
    util::safe_ident,
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
        #![allow(unused, clippy::manual_map)]
        use crate::{Ast, CloneIn, GetSpan, NodeKind};
        use crate::{ast::*, node_id::*};

        #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_clone_in"),
        tokens: output,
    }
    .into()
}

fn generate_clone_in_for_struct(ast: &AstStruct, _schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    let mut fields_clone_in = TokenStream::new();
    let mut fields_params = Vec::new();
    fields_clone_in.extend(quote! { let span = self.span(ast).clone_in(ast); });
    fields_params.push(quote!(span));
    for field in ast.fields.iter() {
        let field_ident = safe_ident(&field.name.to_case(Case::Snake));
        fields_clone_in.extend(quote! {
            let #field_ident = self.#field_ident(ast).clone_in(ast);
        });
        fields_params.push(quote!(#field_ident));
    }

    let fn_name = safe_ident(&ast.name.to_case(Case::Snake));
    let clone_in = quote! {
        impl CloneIn for #name {
            type Cloned = #name;

            #[inline]
            fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
                #fields_clone_in
                ast.#fn_name( #(#fields_params),* )
            }
        }
    };

    quote! {
        #clone_in
    }
}

fn generate_clone_in_for_enum(ast: &AstEnum, _schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    let mut arms = TokenStream::new();
    for variant in ast.variants.iter() {
        let variant_ident = format_ident!("{}", variant.name);
        arms.extend(quote! {
            Self::#variant_ident(it) => Self::#variant_ident(it.clone_in(ast)),
        });
    }

    let node_id_getter = quote! {
        impl CloneIn for #name {
            type Cloned = #name;

            #[inline]
            fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
                match self {
                    #arms
                }
            }
        }
    };

    quote! {
        #node_id_getter
    }
}
