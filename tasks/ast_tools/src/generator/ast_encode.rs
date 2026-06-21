use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput},
    schema::{AstEnum, AstStruct, AstStructField, AstType, Schema, SpanKind, TypeId},
};

pub fn ast_encode(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();

    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast) => impls.extend(generate_encode_for_struct(ast, schema)),
            AstType::Enum(ast) => impls.extend(generate_encode_for_enum(ast, schema)),
            _ => {}
        }
    }

    let output = quote! {
        #![allow(unused)]
        use crate::{
            *,
            codec::{AstEncode, NodeKind, PackedNode, NONE},
        };
        use swc_experimental_allocator::boxed::Box;

        impl<'a, E: AstEncode> AstEncode for Box<'a, E> {
            fn encode(&self, encoder: &mut Encoder) -> u32 {
                (**self).encode(encoder)
            }
        }

        impl<E: AstEncode> AstEncode for Option<E> {
            fn encode(&self, encoder: &mut Encoder) -> u32 {
                match self {
                    Some(it) => it.encode(encoder),
                    None => NONE,
                }
            }
        }

        #impls
    };

    RustOutput {
        path: format!("crates/{AST_CRATE_PATH}/src/codec/ast_encode.rs"),
        tokens: output,
    }
    .into()
}

fn generate_encode_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };
    let ty_generics = impl_generics.clone();

    let mut arms = TokenStream::new();
    for variant in ast.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);
        if variant.type_id.is_some() {
            arms.extend(quote! {
                Self::#variant_name(it) => it.encode(encoder),
            });
        } else {
            let node_kind = format_ident!("{}{}", ast.name, variant.name);
            arms.extend(quote! {
                Self::#variant_name => encoder.packed_node(PackedNode {
                    kind: NodeKind::#node_kind,
                    ..Default::default()
                }),
            });
        }
    }

    quote! {
        impl #impl_generics AstEncode for #name #ty_generics {
            fn encode(&self, encoder: &mut Encoder) -> u32 {
                match self {
                    #arms
                }
            }
        }
    }
}

fn generate_encode_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };
    let ty_generics = impl_generics.clone();
    let node_kind = format_ident!("{}", ast.name);

    let mut flag_offset = 0usize;
    let mut flags = Vec::new();
    let mut pre_stmts = Vec::new();
    let mut fields = Vec::new();

    for field in ast.fields.iter() {
        if is_span_field(field, schema) || should_skip_field(field, schema) {
            continue;
        }

        if let Some((expr, width)) = flag_expr(field, flag_offset, schema) {
            flags.push(expr);
            flag_offset += width;
            continue;
        }

        if let Some(encoded) = field_exprs(field, schema) {
            pre_stmts.push(encoded.stmts);
            let exprs = encoded.exprs;
            fields.extend(exprs);
        } else {
            return todo_encode_impl(&name, impl_generics, ty_generics);
        }
    }

    if fields.len() > 5 || flag_offset > u16::BITS as usize {
        return todo_encode_impl(&name, impl_generics, ty_generics);
    }

    let field_stmts = fields.iter().enumerate().map(|(index, expr)| {
        let field = format_ident!("field{index}");
        quote! {
            let #field = #expr;
        }
    });
    let field_inits = (0..fields.len()).map(|index| {
        let field = format_ident!("field{index}");
        quote!(#field)
    });

    let flags_expr = if flags.is_empty() {
        quote!(0)
    } else {
        quote!(#(#flags)|*)
    };

    let (span_lo, span_hi) = struct_span_exprs(ast, schema);

    quote! {
        impl #impl_generics AstEncode for #name #ty_generics {
            fn encode(&self, encoder: &mut Encoder) -> u32 {
                #(#pre_stmts)*
                #(#field_stmts)*
                let flags = #flags_expr;
                encoder.packed_node(PackedNode {
                    kind: NodeKind::#node_kind,
                    flags,
                    span_lo: #span_lo,
                    span_hi: #span_hi,
                    #(#field_inits,)*
                    ..Default::default()
                })
            }
        }
    }
}

fn todo_encode_impl(
    name: &proc_macro2::Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
) -> TokenStream {
    quote! {
        impl #impl_generics AstEncode for #name #ty_generics {
            fn encode(&self, _encoder: &mut Encoder) -> u32 {
                todo!()
            }
        }
    }
}

struct EncodedField {
    stmts: TokenStream,
    exprs: Vec<TokenStream>,
}

fn encoded_expr(expr: TokenStream) -> EncodedField {
    EncodedField {
        stmts: TokenStream::new(),
        exprs: vec![expr],
    }
}

fn field_exprs(field: &AstStructField, schema: &Schema) -> Option<EncodedField> {
    let field_name = format_ident!("{}", field.name);
    Some(match &schema.types[field.type_id] {
        AstType::Struct(_) | AstType::Enum(_) | AstType::Box(_) => {
            encoded_expr(quote!(self.#field_name.encode(encoder)))
        }
        AstType::Vec(_) => list_field_exprs(&field.name, quote!(&self.#field_name)),
        AstType::Option(ast) => option_field_exprs(&field_name, ast.inner_type_id, schema)?,
        AstType::Primitive(ast) => primitive_field_exprs(&field_name, ast.name)?,
    })
}

fn list_field_exprs(name: &str, value: TokenStream) -> EncodedField {
    let start = format_ident!("__{}_start", name);
    let len = format_ident!("__{}_len", name);
    EncodedField {
        stmts: quote! {
            let (#start, #len) = encoder.nodes(#value);
        },
        exprs: vec![quote!(#start), quote!(#len)],
    }
}

fn string_field_exprs(name: &str, callee: TokenStream) -> EncodedField {
    let start = format_ident!("__{}_start", name);
    let len = format_ident!("__{}_len", name);
    EncodedField {
        stmts: quote! {
            let (#start, #len) = #callee;
        },
        exprs: vec![quote!(#start), quote!(#len)],
    }
}

fn option_field_exprs(
    field_name: &proc_macro2::Ident,
    inner_type_id: TypeId,
    schema: &Schema,
) -> Option<EncodedField> {
    Some(match &schema.types[inner_type_id] {
        AstType::Primitive(ast) if matches!(ast.name, "Atom" | "Utf8Ref" | "AtomRef") => {
            string_field_exprs(
                &field_name.to_string(),
                quote!(encoder.opt_utf8(self.#field_name)),
            )
        }
        AstType::Primitive(ast) if matches!(ast.name, "Wtf8Atom" | "Wtf8Ref") => {
            string_field_exprs(
                &field_name.to_string(),
                quote!(encoder.opt_wtf8(self.#field_name)),
            )
        }
        AstType::Primitive(ast) if ast.name == "Span" => EncodedField {
            stmts: TokenStream::new(),
            exprs: vec![
                quote!(self.#field_name.map_or(NONE, |span| span.start)),
                quote!(self.#field_name.map_or(0, |span| span.end)),
            ],
        },
        AstType::Struct(_) | AstType::Enum(_) | AstType::Box(_) => encoded_expr(quote! {
            match &self.#field_name {
                Some(it) => it.encode(encoder),
                None => NONE,
            }
        }),
        AstType::Vec(_) => {
            let start = format_ident!("__{}_start", field_name);
            let len = format_ident!("__{}_len", field_name);
            EncodedField {
                stmts: quote! {
                    let (#start, #len) = match &self.#field_name {
                        Some(it) => encoder.nodes(it),
                        None => (NONE, 0),
                    };
                },
                exprs: vec![quote!(#start), quote!(#len)],
            }
        }
        _ => return None,
    })
}

fn primitive_field_exprs(field_name: &proc_macro2::Ident, name: &str) -> Option<EncodedField> {
    Some(match name {
        "Atom" | "Utf8Ref" | "AtomRef" => string_field_exprs(
            &field_name.to_string(),
            quote!(encoder.utf8(self.#field_name)),
        ),
        "Wtf8Atom" | "Wtf8Ref" => string_field_exprs(
            &field_name.to_string(),
            quote!(encoder.wtf8(self.#field_name)),
        ),
        "OptionalUtf8Ref" => string_field_exprs(
            &field_name.to_string(),
            quote!(encoder.opt_utf8(self.#field_name)),
        ),
        "OptionalWtf8Ref" => string_field_exprs(
            &field_name.to_string(),
            quote!(encoder.opt_wtf8(self.#field_name)),
        ),
        "u8" | "u16" | "u32" | "usize" => encoded_expr(quote!(self.#field_name as u32)),
        "u64" => EncodedField {
            stmts: TokenStream::new(),
            exprs: vec![
                quote!(self.#field_name as u32),
                quote!((self.#field_name >> 32) as u32),
            ],
        },
        "f64" => EncodedField {
            stmts: TokenStream::new(),
            exprs: vec![
                quote!(self.#field_name.to_bits() as u32),
                quote!((self.#field_name.to_bits() >> 32) as u32),
            ],
        },
        _ => return None,
    })
}

fn flag_expr(
    field: &AstStructField,
    offset: usize,
    schema: &Schema,
) -> Option<(TokenStream, usize)> {
    let field_name = format_ident!("{}", field.name);
    let AstType::Primitive(ast) = &schema.types[field.type_id] else {
        return None;
    };

    if ast.name == "bool" {
        let offset = offset as u16;
        return Some((
            quote! {
                if self.#field_name {
                    1u16 << #offset
                } else {
                    0
                }
            },
            1,
        ));
    }

    let width = simple_enum_flag_width(ast.name)?;
    let offset = offset as u16;
    Some((
        quote! {
            (self.#field_name as u16) << #offset
        },
        width,
    ))
}

fn simple_enum_flag_width(name: &str) -> Option<usize> {
    Some(match name {
        "ImportPhase" => 2,
        "VarDeclKind" => 2,
        "UnaryOp" => 3,
        "UpdateOp" => 1,
        "AssignOp" => 5,
        "BinaryOp" => 5,
        "MetaPropKind" => 1,
        "MethodKind" => 2,
        _ => return None,
    })
}

fn should_skip_field(field: &AstStructField, schema: &Schema) -> bool {
    match &schema.types[field.type_id] {
        AstType::Primitive(ast) => matches!(ast.name, "ScopeId" | "SymbolId"),
        AstType::Option(ast) => {
            matches!(&schema.types[ast.inner_type_id], AstType::Primitive(inner) if matches!(inner.name, "ScopeId" | "SymbolId"))
        }
        _ => false,
    }
}

fn is_span_field(field: &AstStructField, schema: &Schema) -> bool {
    field.span_kind.is_some()
        || matches!(&schema.types[field.type_id], AstType::Primitive(ast) if ast.name == "Span")
}

fn struct_span_exprs(ast: &AstStruct, schema: &Schema) -> (TokenStream, TokenStream) {
    if ast.skip_span {
        return (quote!(0), quote!(0));
    }

    if let Some(field) = ast
        .fields
        .iter()
        .find(|field| field.span_kind == Some(SpanKind::Full))
        .or_else(|| {
            ast.fields
                .iter()
                .find(|field| field.name == "span" && is_span_type(field.type_id, schema))
        })
    {
        let expr = span_expr(field, schema);
        return (quote!(#expr.start), quote!(#expr.end));
    }

    let lo = ast
        .fields
        .iter()
        .find(|field| field.span_kind == Some(SpanKind::Lo));
    let hi = ast
        .fields
        .iter()
        .find(|field| field.span_kind == Some(SpanKind::Hi));

    match (lo, hi) {
        (Some(lo), Some(hi)) => {
            let lo_expr = span_endpoint_expr(lo, SpanKind::Lo, schema);
            let hi_expr = span_endpoint_expr(hi, SpanKind::Hi, schema);
            (lo_expr, hi_expr)
        }
        _ => (quote!(0), quote!(0)),
    }
}

fn span_expr(field: &AstStructField, schema: &Schema) -> TokenStream {
    let field_name = format_ident!("{}", field.name);
    if is_span_type(field.type_id, schema) {
        quote!(self.#field_name)
    } else {
        quote!(self.#field_name.span())
    }
}

fn span_endpoint_expr(field: &AstStructField, span_kind: SpanKind, schema: &Schema) -> TokenStream {
    let field_name = format_ident!("{}", field.name);
    match span_kind {
        SpanKind::Lo => {
            if is_span_type(field.type_id, schema) {
                quote!(self.#field_name.start)
            } else {
                quote!(self.#field_name.span_lo())
            }
        }
        SpanKind::Hi => {
            if is_span_type(field.type_id, schema) {
                quote!(self.#field_name.end)
            } else {
                quote!(self.#field_name.span_hi())
            }
        }
        SpanKind::Full => unreachable!(),
    }
}

fn is_span_type(type_id: TypeId, schema: &Schema) -> bool {
    matches!(&schema.types[type_id], AstType::Primitive(ast) if ast.name == "Span")
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
            | "AtomRef"
            | "Wtf8Ref"
            | "Wtf8Atom"
            | "OptionalUtf8Ref"
            | "OptionalWtf8Ref"
            | "swc_experimental_num_bigint::BigInt"
    )
}
