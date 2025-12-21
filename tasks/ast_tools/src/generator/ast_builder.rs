use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
    util::{
        INLINE_DATA_U32_SIZE, InlineStorageMode, calculate_inline_layout, generate_field_to_u32,
        map_field_type_to_extra_field, safe_ident,
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
            #![allow(unused, clippy::useless_conversion, clippy::identity_op)]
            use swc_core::common::Span;

            use crate::{Ast, AstNode, ExtraData, NodeData, NodeKind, ast::*, node_id::*};

            impl Ast {
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

/// Generate build function using inline storage (for small nodes)
/// Layout (7 bytes total):
/// - Bytes 0-3: NodeData.inline_data (u32)
/// - Bytes 4-6: AstNode.inline_data ([u8; 3])
fn generate_build_function_inline(
    ast: &AstStruct,
    schema: &Schema,
    fn_name: syn::Ident,
    ret_ty: syn::Ident,
    fn_params: TokenStream,
    layout: &crate::util::InlineLayout,
) -> TokenStream {
    // Optimization: zero-field nodes use empty: () for shorter generated code
    if layout.fields.is_empty() {
        return quote! {
            #[inline]
            pub fn #fn_name(&mut self, span: Span) -> #ret_ty {
                #ret_ty(self.add_node(AstNode {
                    span,
                    kind: NodeKind::#ret_ty,
                    _inline_data: 0u32.into(),
                    data: NodeData { empty: () },
                }))
            }
        };
    }

    // Optimization: if FourBytes mode with a single field at offset 0 (1-4 bytes),
    // we can directly assign the u32 value without byte conversion
    let is_single_u32_field = layout.mode == InlineStorageMode::FourBytes
        && layout.fields.len() == 1
        && layout.fields[0].1 == 0   // byte_offset == 0
        && layout.fields[0].2 <= INLINE_DATA_U32_SIZE; // byte_size <= 4

    if is_single_u32_field {
        let (field_idx, _, _) = layout.fields[0];
        let field = &ast.fields[field_idx];
        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id];

        // Generate direct u32 value
        let u32_value = generate_field_to_u32(field_ty, &field_name, schema);

        return quote! {
            #[inline]
            pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                #ret_ty(
                    self.add_node(AstNode {
                        span,
                        kind: NodeKind::#ret_ty,
                        _inline_data: 0u32.into(),
                        data: NodeData {
                            inline_data: #u32_value,
                        },
                    })
                )
            }
        };
    }

    // General case: pack fields using bit operations
    // inline_u32: bytes 0-3 (node.data.inline_data)
    // inline_u24: bytes 4-6 (node.inline_data as U24)
    let mut pack_u32 = TokenStream::new();
    let mut pack_u24 = TokenStream::new();

    for &(field_idx, byte_offset, byte_size) in &layout.fields {
        let field = &ast.fields[field_idx];
        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id];

        // Generate u32 value for this field
        let field_u32 = generate_field_to_u32(field_ty, &field_name, schema);
        let bit_offset = byte_offset * 8;
        let field_end = byte_offset + byte_size;

        if field_end <= INLINE_DATA_U32_SIZE {
            // Field entirely in bytes 0-3 (inline_u32)
            if byte_offset == 0 {
                pack_u32.extend(quote! { | #field_u32 });
            } else {
                pack_u32.extend(quote! { | ((#field_u32) << #bit_offset) });
            }
        } else if byte_offset >= INLINE_DATA_U32_SIZE {
            // Field entirely in bytes 4-6 (inline_u24)
            let u24_bit_offset = (byte_offset - INLINE_DATA_U32_SIZE) * 8;
            if u24_bit_offset == 0 {
                pack_u24.extend(quote! { | #field_u32 });
            } else {
                pack_u24.extend(quote! { | ((#field_u32) << #u24_bit_offset) });
            }
        } else {
            // Field spans the boundary (bytes in both u32 and u24)
            // Split: lower bits go to u32, upper bits go to u24
            let bits_in_u32 = (INLINE_DATA_U32_SIZE - byte_offset) * 8;
            let mask_u32 = (1u32 << bits_in_u32) - 1;

            if byte_offset == 0 {
                pack_u32.extend(quote! { | ((#field_u32) & #mask_u32) });
            } else {
                pack_u32.extend(quote! { | (((#field_u32) & #mask_u32) << #bit_offset) });
            }
            pack_u24.extend(quote! { | ((#field_u32) >> #bits_in_u32) });
        }
    }

    match layout.mode {
        InlineStorageMode::FourBytes => {
            quote! {
                #[inline]
                pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                    #ret_ty(
                        self.add_node(AstNode {
                            span,
                            kind: NodeKind::#ret_ty,
                            _inline_data: 0u32.into(),
                            data: NodeData {
                                inline_data: 0u32 #pack_u32,
                            },
                        })
                    )
                }
            }
        }
        InlineStorageMode::Full => {
            quote! {
                #[inline]
                pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
                    #ret_ty(
                        self.add_node(AstNode {
                            span,
                            kind: NodeKind::#ret_ty,
                            _inline_data: (0u32 #pack_u24).into(),
                            data: NodeData {
                                inline_data: 0u32 #pack_u32,
                            },
                        })
                    )
                }
            }
        }
    }
}

/// Generate build function using extra_data storage (for large nodes)
fn generate_build_function_extra_data(
    ast: &AstStruct,
    schema: &Schema,
    fn_name: syn::Ident,
    ret_ty: syn::Ident,
    fn_params: TokenStream,
) -> TokenStream {
    let mut add_extra_data = TokenStream::new();
    for (index, field) in ast.fields.iter().enumerate() {
        let extra_data_id = format_ident!("_f{}", index);
        let field_name = format_ident!("{}", field.name);

        let field_ty = &schema.types[field.type_id];
        let extra_data_field = map_field_type_to_extra_field(field_ty, schema);
        let field_value = match field_ty {
            AstType::Option(ast_option) => {
                let inner_ty = &schema.types[ast_option.inner_type_id];
                match inner_ty {
                    AstType::Vec(_) => quote!(#field_name.map(|n| n.inner).into()),
                    AstType::Enum(_) => quote!(#field_name),
                    _ => quote!(#field_name.map(|n| n.node_id()).into()),
                }
            }
            AstType::Struct(_) => {
                quote!( #field_name.node_id() )
            }
            AstType::Enum(ast_enum) if ast_enum.name == "AssignTarget" => {
                quote!( #field_name.node_id().into() )
            }
            AstType::Enum(_) => quote!( #field_name ),
            _ if extra_data_field == "other" => quote!( #field_name.to_extra_data() ),
            _ => quote!( #field_name.into() ),
        };
        let extra_data_field = format_ident!("{}", extra_data_field);

        add_extra_data.extend(quote! {
            let #extra_data_id = self.add_extra(ExtraData { #extra_data_field: #field_value });
        });
    }

    let node_data = match ast.fields.len() {
        0 => quote!(empty: ()),
        _ => quote!(extra_data_start: _f0),
    };

    let tokens = quote! {
        #[inline]
        pub fn #fn_name(&mut self, span: Span, #fn_params) -> #ret_ty {
            #add_extra_data

            #ret_ty(
                self.add_node(AstNode {
                    span,
                    kind: NodeKind::#ret_ty,
                    _inline_data: 0u32.into(),
                    data: NodeData {
                        #node_data
                    },
                })
            )
        }
    };

    tokens
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
                    fn_name.push_str(&ast_struct.name);
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
