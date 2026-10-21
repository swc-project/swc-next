use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    generator::ast_payload::{direct_payload_field, payload_member},
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Layout, LayoutMode, Schema},
    util::{generate_field_to_u32, generate_field_to_u64, safe_ident},
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
            use crate::generated::ast_payload::*;
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

    generate_build_function_inline(ast, schema, fn_name, ret_ty, fn_params, &ast.layout)
}

/// Generate a builder that constructs the node's concrete payload layout.
fn generate_build_function_inline(
    ast: &AstStruct,
    schema: &Schema,
    fn_name: syn::Ident,
    ret_ty: syn::Ident,
    fn_params: TokenStream,
    layout: &Layout,
) -> TokenStream {
    if ast.fields.is_empty() {
        return quote! {
            #[inline]
            pub fn #fn_name(&mut self, span: Span) -> #ret_ty {
                #ret_ty(self.add_node(
                    span,
                    NodeKind::#ret_ty,
                    NodePayload { raw: [0; 16] },
                ))
            }
        };
    }

    match layout.mode {
        LayoutMode::Full => {
            let payload = generate_payload_constructor(ast, schema, layout, None);
            quote! {
                #[inline]
                pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                    #ret_ty(self.add_node(span, NodeKind::#ret_ty, #payload))
                }
            }
        }
        LayoutMode::Partial => {
            let extra_data_values = layout
                .extra_fields()
                .map(|(field_index, _)| {
                    let field_name = safe_ident(&ast.fields[field_index].name);
                    quote!(#field_name)
                })
                .collect::<Vec<_>>();

            let (add_extra_data, start_id) = generate_add_extra_data(&extra_data_values);
            let start_id = start_id.expect("partial layout must contain an extra-data field");
            let payload = generate_payload_constructor(ast, schema, layout, Some(start_id));

            quote! {
                #[inline]
                pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                    #add_extra_data
                    #ret_ty(self.add_node(span, NodeKind::#ret_ty, #payload))
                }
            }
        }
    }
}

fn generate_payload_constructor(
    ast: &AstStruct,
    schema: &Schema,
    layout: &Layout,
    extra_data_start: Option<TokenStream>,
) -> TokenStream {
    let member = payload_member(ast);
    let layout_name = format_ident!("{}Payload", ast.name);
    let mut fields = layout.payload_fields().collect::<Vec<_>>();
    fields.sort_by_key(|&(_, offset, _)| offset);

    let mut initializers = TokenStream::new();
    let mut cursor = 0;
    let mut pad_index = 0;
    for (field_index, offset, size) in fields {
        if cursor < offset {
            let pad = format_ident!("_pad{pad_index}");
            let len = offset - cursor;
            initializers.extend(quote!(#pad: [0; #len],));
            pad_index += 1;
        }
        let field = &ast.fields[field_index];
        let field_name = safe_ident(&field.name);
        let field_ty = &schema.types[field.type_id];
        let value = if direct_payload_field(field_ty, schema) {
            quote!(#field_name)
        } else if size == 8 {
            generate_field_to_u64(field_ty, &field_name, schema)
        } else {
            let value = generate_field_to_u32(field_ty, &field_name, quote!(self), schema);
            if size == 4 {
                value
            } else {
                quote!(#value as _)
            }
        };
        initializers.extend(quote!(#field_name: #value,));
        cursor = offset + size;
    }

    let end = layout.extra_start_offset().unwrap_or(16);
    if cursor < end {
        let pad = format_ident!("_pad{pad_index}");
        let len = end - cursor;
        initializers.extend(quote!(#pad: [0; #len],));
    }
    if let Some(start) = extra_data_start {
        initializers.extend(quote!(extra_data_start: #start,));
    }

    quote! {
        NodePayload {
            #member: #layout_name { #initializers }
        }
    }
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
                let _extra_data = [#((#values).into_extra_data()),*];
                let _extra_data_start = self.add_extra_data_slice(&_extra_data);
            },
            Some(quote!(_extra_data_start)),
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
