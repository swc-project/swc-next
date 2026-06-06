use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstStructField, AstType, Schema, SpanKind, TypeId},
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
    let mut as_mut_variant = TokenStream::new();

    for variant in ast.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);
        let is_fn_name = format_ident!("is_{}", variant.name.to_case(Case::Snake));
        is_variant.extend(quote! {
            #[inline]
            pub const fn #is_fn_name(&self) -> bool {
                matches!(self, Self::#variant_name { .. })
            }
        });

        let Some(payload_ty_id) = variant.type_id else {
            continue;
        };

        let as_fn_name = format_ident!("as_{}", variant.name.to_case(Case::Snake));
        let as_mut_fn_name = format_ident!("as_mut_{}", variant.name.to_case(Case::Snake));
        let payload_ty = type_ref(payload_ty_id, schema);
        as_variant.extend(quote! {
            #[inline]
            pub fn #as_fn_name(&self) -> Option<&#payload_ty> {
                match self {
                    Self::#variant_name(it) => Some(it),
                    _ => None,
                }
            }
        });
        as_mut_variant.extend(quote! {
            #[inline]
            pub fn #as_mut_fn_name(&mut self) -> Option<&mut #payload_ty> {
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
            #as_mut_variant
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

    let span_spec = struct_span_spec(ast, schema);
    let get_span = generate_struct_get_span(&span_spec, schema);
    let set_span = generate_struct_set_span(&span_spec, schema);

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

fn is_span_type(type_id: TypeId, schema: &Schema) -> bool {
    matches!(&schema.types[type_id], AstType::Primitive(ast) if ast.name == "Span")
}

enum StructSpanSpec<'a> {
    Field(&'a AstStructField),
    Range {
        lo: &'a AstStructField,
        hi: &'a AstStructField,
    },
    None,
}

fn struct_span_spec<'a>(ast: &'a AstStruct, schema: &Schema) -> StructSpanSpec<'a> {
    let full_fields = ast
        .fields
        .iter()
        .filter(|field| field.span_kind == Some(SpanKind::Full))
        .collect::<Vec<_>>();
    let lo_fields = ast
        .fields
        .iter()
        .filter(|field| field.span_kind == Some(SpanKind::Lo))
        .collect::<Vec<_>>();
    let hi_fields = ast
        .fields
        .iter()
        .filter(|field| field.span_kind == Some(SpanKind::Hi))
        .collect::<Vec<_>>();

    assert!(
        full_fields.len() <= 1,
        "{} has multiple #[span] fields",
        ast.name
    );
    assert!(
        lo_fields.len() <= 1,
        "{} has multiple #[span(lo)] fields",
        ast.name
    );
    assert!(
        hi_fields.len() <= 1,
        "{} has multiple #[span(hi)] fields",
        ast.name
    );
    assert!(
        full_fields.is_empty() || (lo_fields.is_empty() && hi_fields.is_empty()),
        "{} mixes #[span] with #[span(lo)] / #[span(hi)]",
        ast.name
    );

    if let Some(field) = full_fields.first() {
        return StructSpanSpec::Field(field);
    }

    if !lo_fields.is_empty() || !hi_fields.is_empty() {
        assert!(
            !lo_fields.is_empty() && !hi_fields.is_empty(),
            "{} must have both #[span(lo)] and #[span(hi)]",
            ast.name
        );
        return StructSpanSpec::Range {
            lo: lo_fields[0],
            hi: hi_fields[0],
        };
    }

    ast.fields
        .iter()
        .find(|field| field.name == "span" && is_span_type(field.type_id, schema))
        .map_or(StructSpanSpec::None, StructSpanSpec::Field)
}

fn generate_struct_get_span(span_spec: &StructSpanSpec<'_>, schema: &Schema) -> TokenStream {
    match span_spec {
        StructSpanSpec::Field(field) => field_span_expr(field, schema),
        StructSpanSpec::Range { lo, hi } => {
            let lo_expr = field_span_endpoint_expr(lo, SpanKind::Lo, schema);
            let hi_expr = field_span_endpoint_expr(hi, SpanKind::Hi, schema);
            quote!(Span::new(#lo_expr, #hi_expr))
        }
        StructSpanSpec::None => quote!(DUMMY_SP),
    }
}

fn generate_struct_set_span(span_spec: &StructSpanSpec<'_>, schema: &Schema) -> TokenStream {
    match span_spec {
        StructSpanSpec::Field(field) => field_set_span_stmt(field, schema),
        StructSpanSpec::Range { lo, hi } => {
            let lo_stmt = field_set_span_endpoint_stmt(lo, SpanKind::Lo, schema);
            let hi_stmt = field_set_span_endpoint_stmt(hi, SpanKind::Hi, schema);
            quote! {
                #lo_stmt
                #hi_stmt
            }
        }
        StructSpanSpec::None => TokenStream::new(),
    }
}

fn field_span_expr(field: &AstStructField, schema: &Schema) -> TokenStream {
    let field_name = format_ident!("{}", field.name);
    if is_span_type(field.type_id, schema) {
        quote!(self.#field_name)
    } else {
        quote!(self.#field_name.span())
    }
}

fn field_set_span_stmt(field: &AstStructField, schema: &Schema) -> TokenStream {
    let field_name = format_ident!("{}", field.name);
    if is_span_type(field.type_id, schema) {
        quote!(self.#field_name = span;)
    } else {
        quote!(self.#field_name.set_span(span);)
    }
}

fn field_span_endpoint_expr(
    field: &AstStructField,
    span_kind: SpanKind,
    schema: &Schema,
) -> TokenStream {
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
        SpanKind::Full => unreachable!("full span is not an endpoint"),
    }
}

fn field_set_span_endpoint_stmt(
    field: &AstStructField,
    span_kind: SpanKind,
    schema: &Schema,
) -> TokenStream {
    let field_name = format_ident!("{}", field.name);
    match span_kind {
        SpanKind::Lo => {
            if is_span_type(field.type_id, schema) {
                quote!(self.#field_name.start = span.start;)
            } else {
                quote! {
                    let current = self.#field_name.span();
                    self.#field_name.set_span(Span::new(span.start, current.end));
                }
            }
        }
        SpanKind::Hi => {
            if is_span_type(field.type_id, schema) {
                quote!(self.#field_name.end = span.end;)
            } else {
                quote! {
                    let current = self.#field_name.span();
                    self.#field_name.set_span(Span::new(current.start, span.end));
                }
            }
        }
        SpanKind::Full => unreachable!("full span is not an endpoint"),
    }
}
