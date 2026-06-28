use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput},
    schema::{AstEnum, AstStruct, AstStructField, AstType, Schema, TypeId},
    util::{
        is_span_field, safe_ident, should_skip_field, simple_enum_flag_width, type_has_lifetime,
    },
};

pub fn ast_decode(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();

    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast) => impls.extend(generate_decode_for_struct(ast, schema)),
            AstType::Enum(ast) => impls.extend(generate_decode_for_enum(ast, schema)),
            _ => {}
        }
    }

    let output = quote! {
        #![allow(unused)]
        use crate::{
            *,
            codec::{decode::Decoder, AstDecode, NodeKind, NONE},
        };

        #impls
    };

    RustOutput {
        path: format!("crates/{AST_CRATE_PATH}/src/codec/generated/decode.rs"),
        tokens: output,
    }
    .into()
}

fn generate_decode_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = quote!(<'a>);
    let ty_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };

    let mut arms = TokenStream::new();
    for variant in ast.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);
        match variant.type_id {
            Some(type_id) => {
                for node_kind in concrete_node_kinds(type_id, schema) {
                    let node_kind = format_ident!("{node_kind}");
                    arms.extend(quote! {
                        value if value == NodeKind::#node_kind as u8 => Self::#variant_name(AstDecode::decode(decoder, id)),
                    });
                }
            }
            None => {
                let node_kind = format_ident!("{}{}", ast.name, variant.name);
                arms.extend(quote! {
                    value if value == NodeKind::#node_kind as u8 => Self::#variant_name,
                });
            }
        }
    }

    quote! {
        impl #impl_generics AstDecode<'a> for #name #ty_generics {
            fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
                let node = decoder.node(id);
                match node.kind {
                    #arms
                    _ => panic!("unexpected node kind {} for {}", node.kind, stringify!(#name)),
                }
            }
        }
    }
}

fn generate_decode_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = quote!(<'a>);
    let ty_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };
    let node_kind = format_ident!("{}", ast.name);

    let mut field_index = 0usize;
    let mut flag_offset = 0usize;
    let mut fields = TokenStream::new();

    for field in ast.fields.iter() {
        let field_name = safe_ident(&field.name);

        if should_skip_field(field, schema) {
            fields.extend(quote! {
                #field_name: std::cell::Cell::new(None),
            });
            continue;
        }

        if is_span_field(field, schema) {
            fields.extend(quote! {
                #field_name: decoder.span(node),
            });
            continue;
        }

        if let Some((expr, width)) = flag_decode_expr(field, flag_offset, schema) {
            fields.extend(quote! {
                #field_name: #expr,
            });
            flag_offset += width;
            continue;
        }

        let Some(decoded) = decode_field(field, field_index, schema) else {
            unreachable!("{} has unsupported field {}", ast.name, field.name);
        };
        field_index += decoded.width;
        let expr = decoded.expr;
        fields.extend(quote! {
            #field_name: #expr,
        });
    }

    if field_index > 5 || flag_offset > u16::BITS as usize {
        unreachable!("{} cannot fit in PackedNode", ast.name);
    }

    quote! {
        impl #impl_generics AstDecode<'a> for #name #ty_generics {
            fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
                let node = decoder.node(id);
                debug_assert_eq!(node.kind, NodeKind::#node_kind as u8);
                Self {
                    #fields
                }
            }
        }
    }
}

struct DecodedField {
    width: usize,
    expr: TokenStream,
}

fn decode_field(
    field: &AstStructField,
    field_index: usize,
    schema: &Schema,
) -> Option<DecodedField> {
    let slot = |offset| {
        let field = format_ident!("field{}", field_index + offset);
        quote!(node.#field)
    };
    let slot0 = slot(0);
    let slot1 = slot(1);

    Some(match &schema.types[field.type_id] {
        AstType::Struct(_) | AstType::Enum(_) | AstType::Box(_) => DecodedField {
            width: 1,
            expr: quote!(AstDecode::decode(decoder, #slot0)),
        },
        AstType::Vec(_) => DecodedField {
            width: 2,
            expr: quote!(decoder.nodes(#slot0, #slot1)),
        },
        AstType::Option(ast) => decode_option_field(ast.inner_type_id, field_index, schema)?,
        AstType::Primitive(ast) => decode_primitive_field(ast.name, field_index)?,
    })
}

fn decode_option_field(
    inner_type_id: TypeId,
    field_index: usize,
    schema: &Schema,
) -> Option<DecodedField> {
    let slot = |offset| {
        let field = format_ident!("field{}", field_index + offset);
        quote!(node.#field)
    };
    let slot0 = slot(0);
    let slot1 = slot(1);

    Some(match &schema.types[inner_type_id] {
        AstType::Primitive(ast) if ast.name == "Atom" => DecodedField {
            width: 2,
            expr: quote!(decoder.opt_atom(#slot0, #slot1)),
        },
        AstType::Primitive(ast) if ast.name == "Wtf8Atom" => DecodedField {
            width: 2,
            expr: quote!(decoder.opt_wtf8_atom(#slot0, #slot1)),
        },
        AstType::Primitive(ast) if ast.name == "Span" => DecodedField {
            width: 2,
            expr: quote! {
                if #slot0 == NONE {
                    None
                } else {
                    Some(Span {
                        start: #slot0,
                        end: #slot1,
                    })
                }
            },
        },
        AstType::Struct(_) | AstType::Enum(_) | AstType::Box(_) => DecodedField {
            width: 1,
            expr: quote!(AstDecode::decode(decoder, #slot0)),
        },
        AstType::Vec(_) => DecodedField {
            width: 2,
            expr: quote! {
                if #slot0 == NONE {
                    None
                } else {
                    Some(decoder.nodes(#slot0, #slot1))
                }
            },
        },
        _ => return None,
    })
}

fn decode_primitive_field(name: &str, field_index: usize) -> Option<DecodedField> {
    let slot = |offset| {
        let field = format_ident!("field{}", field_index + offset);
        quote!(node.#field)
    };
    let slot0 = slot(0);
    let slot1 = slot(1);

    Some(match name {
        "Atom" => DecodedField {
            width: 2,
            expr: quote!(decoder.atom(#slot0, #slot1)),
        },
        "Wtf8Atom" => DecodedField {
            width: 2,
            expr: quote!(decoder.wtf8_atom(#slot0, #slot1)),
        },
        "f64" => DecodedField {
            width: 2,
            expr: quote!(f64::from_bits((#slot0 as u64) | ((#slot1 as u64) << 32))),
        },
        _ => return None,
    })
}

fn flag_decode_expr(
    field: &AstStructField,
    offset: usize,
    schema: &Schema,
) -> Option<(TokenStream, usize)> {
    let AstType::Primitive(ast) = &schema.types[field.type_id] else {
        return None;
    };

    if ast.name == "bool" {
        let offset = offset as u16;
        return Some((quote!((node.flags & (1u16 << #offset)) != 0), 1));
    }

    let width = simple_enum_flag_width(ast.name)?;
    let name = format_ident!("{}", ast.name);
    let mask = ((1u32 << width) - 1) as u16;
    let offset = offset as u16;
    let variants = ast.variants.as_ref()?;
    let arms = variants.iter().enumerate().map(|(index, variant)| {
        let index = index as u16;
        let variant = format_ident!("{variant}");
        quote!(#index => #name::#variant,)
    });

    Some((
        quote! {
            match (node.flags >> #offset) & #mask {
                #(#arms)*
                value => panic!("invalid {} flag value {value}", stringify!(#name)),
            }
        },
        width,
    ))
}

fn concrete_node_kinds(type_id: TypeId, schema: &Schema) -> Vec<String> {
    match &schema.types[type_id] {
        AstType::Struct(ast) => vec![ast.name.clone()],
        AstType::Enum(ast) => ast
            .variants
            .iter()
            .flat_map(|variant| match variant.type_id {
                Some(type_id) => concrete_node_kinds(type_id, schema),
                None => vec![format!("{}{}", ast.name, variant.name)],
            })
            .collect(),
        AstType::Box(ast) => concrete_node_kinds(ast.inner_type_id, schema),
        _ => unreachable!(
            "unsupported enum variant type {}",
            schema.types[type_id].name()
        ),
    }
}
