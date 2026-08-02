use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
    util::{
        EXTRA_DATA_ID_BIT_OFFSET, InlineStorageMode, calculate_inline_layout,
        generate_field_to_u32, generate_field_to_u64, safe_ident,
    },
};

pub fn ast_builder(schema: &Schema) -> RawOutput {
    let mut build_functions = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                build_functions.extend(generate_build_function_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => {
                let mut context = RecursiveEnumContext {
                    ret_ty: format_ident!("{}", ast_enum.name).to_token_stream(),
                    ..Default::default()
                };
                build_functions.extend(generate_build_function_for_enum(
                    ast_enum,
                    schema,
                    &mut context,
                ));
            }
            _ => continue,
        };
    }

    let output = quote! {
            #![allow(
                unused,
                clippy::useless_conversion,
                clippy::identity_op,
                clippy::too_many_arguments
            )]
            use crate::*;
            use oxc_index::Idx;

            impl<'a> Ast<'a> {
                #build_functions
            }
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_builder"),
        tokens: output,
    }
    .into()
}

fn generate_build_function_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let fn_name = safe_ident(&ast.name.to_case(Case::Snake));
    let ret_ty = format_ident!("{}", ast.name);
    let fn_params = generate_fn_params_decl(ast, schema);

    // Check if this struct can use inline storage
    if let Some(layout) = calculate_inline_layout(ast, schema) {
        generate_build_function_inline(ast, schema, fn_name, ret_ty, fn_params, &layout)
    } else {
        generate_build_function_extra_data(ast, schema, fn_name, ret_ty, fn_params)
    }
}

/// Generate a builder by packing fields into a u128 payload.
fn generate_build_function_inline(
    ast: &AstStruct,
    schema: &Schema,
    fn_name: syn::Ident,
    ret_ty: syn::Ident,
    fn_params: TokenStream,
    layout: &crate::util::InlineLayout,
) -> TokenStream {
    if layout.fields.is_empty() {
        return quote! {
            #[inline]
            pub fn #fn_name(&mut self, span: Span) -> #ret_ty {
                #ret_ty(self.add_node(
                    span,
                    NodeKind::#ret_ty,
                    NodePayload(0),
                ))
            }
        };
    }

    let mut pack_u128 = TokenStream::new();

    for &(field_idx, byte_offset, byte_size) in &layout.fields {
        let field = &ast.fields[field_idx];
        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id];

        let bit_offset = byte_offset * 8;
        if byte_size == std::mem::size_of::<u64>() {
            let field_u64 = generate_field_to_u64(field_ty, &field_name, quote!(self), schema);
            pack_u128.extend(quote! { | ((#field_u64 as u128) << #bit_offset) });
            continue;
        }

        let field_u32 = generate_field_to_u32(field_ty, &field_name, quote!(self), schema);
        pack_u128.extend(quote! { | ((#field_u32 as u128) << #bit_offset) });
    }

    match layout.mode {
        InlineStorageMode::Full => quote! {
            #[inline]
            pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                #ret_ty(self.add_node(
                    span,
                    NodeKind::#ret_ty,
                    NodePayload(0u128 #pack_u128),
                ))
            }
        },
        InlineStorageMode::Partial => {
            let mut extra_data_values = Vec::new();

            for (field_index, field) in ast.fields.iter().enumerate() {
                if layout
                    .fields
                    .iter()
                    .any(|(inline_index, _, _)| *inline_index == field_index)
                {
                    continue;
                }

                let field_name = format_ident!("{}", field.name);
                extra_data_values.push(quote!(#field_name.to_extra_data(self)));
            }

            let (add_extra_data, start_id) = generate_add_extra_data(&extra_data_values);
            let start_id = start_id.expect("partial layout must contain an extra-data field");

            quote! {
                #[inline]
                pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                    #add_extra_data
                    #ret_ty(self.add_node(
                        span,
                        NodeKind::#ret_ty,
                        NodePayload(
                            0u128
                            #pack_u128
                            | ((#start_id.index() as u128) << #EXTRA_DATA_ID_BIT_OFFSET)
                        ),
                    ))
                }
            }
        }
    }
}

/// Generate build function using extra_data storage (for large nodes)
fn generate_build_function_extra_data(
    ast: &AstStruct,
    _schema: &Schema,
    fn_name: syn::Ident,
    ret_ty: syn::Ident,
    fn_params: TokenStream,
) -> TokenStream {
    let mut extra_data_values = Vec::new();
    for field in &ast.fields {
        let field_name = format_ident!("{}", field.name);
        extra_data_values.push(quote!(#field_name.to_extra_data(self)));
    }

    let (add_extra_data, start_id) = generate_add_extra_data(&extra_data_values);
    let node_payload = match start_id {
        Some(start_id) => {
            quote!(NodePayload((#start_id.index() as u128) << #EXTRA_DATA_ID_BIT_OFFSET))
        }
        None => quote!(NodePayload(0)),
    };

    let tokens = quote! {
        #[inline]
        pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
            #add_extra_data

            #ret_ty(
                self.add_node(
                    span,
                    NodeKind::#ret_ty,
                    #node_payload,
                )
            )
        }
    };

    tokens
}

fn generate_add_extra_data(values: &[TokenStream]) -> (TokenStream, Option<TokenStream>) {
    match values {
        [] => (TokenStream::new(), None),
        [value] => (
            quote! {
                let _f0 = self.add_extra(#value);
            },
            Some(quote!(_f0)),
        ),
        values => (
            quote! {
                let _extra_data = [#(#values),*];
                let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
            },
            Some(quote!(_extra_data_range.start)),
        ),
    }
}

// TODO: Use [crate::util::flat_enum_type]
#[derive(Default)]
struct RecursiveEnumContext {
    ret_ty: TokenStream,
    name: Vec<String>,
    constructor: Vec<TokenStream>,
}

fn generate_build_function_for_enum(
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
                    let has_duplicate_payload = ast
                        .variants
                        .iter()
                        .filter(|candidate| candidate.type_id == Some(payload_ty_id))
                        .count()
                        > 1;
                    if has_duplicate_payload {
                        fn_name.push_str(&variant.name);
                    } else {
                        fn_name.push_str(&ast_struct.name);
                    }
                    safe_ident(&fn_name.to_case(Case::Snake))
                };

                let args = generate_fn_args(ast_struct);
                let constructor = safe_ident(&ast_struct.name.to_case(Case::Snake));
                let body = recursive_context.constructor.iter().rev().fold(
                    quote!( self.#constructor(span, #args).into() ),
                    |acc, constructor| quote!(#constructor(#acc)),
                );

                let fn_params = generate_fn_params_decl(ast_struct, schema);
                build_variants.extend(quote! {
                    #[inline]
                    pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                        #body
                    }
                });
            }
            AstType::Enum(inner_enum) => {
                build_variants.extend(generate_build_function_for_enum(
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

fn generate_fn_params_decl(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let mut fields = Vec::default();
    for field in ast.fields.iter() {
        let field_name = format_ident!("{}", field.name);
        let field_ty = schema.types[field.type_id].repr_ident(schema);
        fields.push(quote!(#field_name: #field_ty));
    }

    quote!( #(#fields),* )
}

fn generate_fn_args(ast: &AstStruct) -> TokenStream {
    let mut fields = Vec::default();
    for field in ast.fields.iter() {
        let field_name = format_ident!("{}", field.name);
        fields.push(quote!(#field_name));
    }

    quote!( #(#fields),* )
}
