use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput},
    schema::{AstEnum, AstStruct, AstStructField, AstType, Schema, TypeId},
    util::{is_span_field, should_skip_field, simple_enum_flag_width, type_has_lifetime},
};

pub fn ast_measure(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();

    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast) => impls.extend(generate_measure_for_struct(ast, schema)),
            AstType::Enum(ast) => impls.extend(generate_measure_for_enum(ast, schema)),
            _ => {}
        }
    }

    let output = quote! {
        #![allow(unused)]
        use crate::{
            *,
            codec::{AstMeasure, Header},
        };
        use swc_experimental_allocator::boxed::Box;

        impl<'a, M: AstMeasure> AstMeasure for Box<'a, M> {
            fn measure(&self, header: &mut Header) {
                (**self).measure(header)
            }
        }

        impl<M: AstMeasure> AstMeasure for Option<M> {
            fn measure(&self, header: &mut Header) {
                if let Some(it) = self {
                    it.measure(header);
                }
            }
        }

        #impls
    };

    RustOutput {
        path: format!("crates/{AST_CRATE_PATH}/src/codec/ast_measure.rs"),
        tokens: output,
    }
    .into()
}

fn generate_measure_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
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
                Self::#variant_name(it) => it.measure(header),
            });
        } else {
            arms.extend(quote! {
                Self::#variant_name => header.node(),
            });
        }
    }

    quote! {
        impl #impl_generics AstMeasure for #name #ty_generics {
            fn measure(&self, header: &mut Header) {
                match self {
                    #arms
                }
            }
        }
    }
}

fn generate_measure_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let impl_generics = if type_has_lifetime(ast.type_id, schema) {
        quote!(<'a>)
    } else {
        TokenStream::new()
    };
    let ty_generics = impl_generics.clone();

    let mut field_measure = TokenStream::new();
    for field in ast.fields.iter() {
        if is_span_field(field, schema) || should_skip_field(field, schema) {
            continue;
        }

        if is_flag_field(field, schema) {
            continue;
        }

        if let Some(stmt) = measure_field_stmt(field, schema) {
            field_measure.extend(stmt);
        }
    }

    quote! {
        impl #impl_generics AstMeasure for #name #ty_generics {
            fn measure(&self, header: &mut Header) {
                header.node();
                #field_measure
            }
        }
    }
}

fn measure_field_stmt(field: &AstStructField, schema: &Schema) -> Option<TokenStream> {
    let field_name = format_ident!("{}", field.name);
    Some(match &schema.types[field.type_id] {
        AstType::Struct(_) | AstType::Enum(_) | AstType::Box(_) => {
            quote!(self.#field_name.measure(header);)
        }
        AstType::Vec(_) => quote!(header.nodes(&self.#field_name);),
        AstType::Option(ast) => measure_option_field_stmt(&field_name, ast.inner_type_id, schema)?,
        AstType::Primitive(ast) => measure_primitive_field_stmt(&field_name, ast.name)?,
    })
}

fn measure_option_field_stmt(
    field_name: &proc_macro2::Ident,
    inner_type_id: TypeId,
    schema: &Schema,
) -> Option<TokenStream> {
    Some(match &schema.types[inner_type_id] {
        AstType::Primitive(ast) if matches!(ast.name, "Atom") => {
            quote!(header.opt_utf8(self.#field_name);)
        }
        AstType::Primitive(ast) if matches!(ast.name, "Wtf8Atom") => {
            quote!(header.opt_wtf8(self.#field_name);)
        }
        AstType::Struct(_) | AstType::Enum(_) | AstType::Box(_) => {
            quote!(self.#field_name.measure(header);)
        }
        AstType::Vec(_) => quote! {
            if let Some(it) = &self.#field_name {
                header.nodes(it);
            }
        },
        _ => return None,
    })
}

fn measure_primitive_field_stmt(
    field_name: &proc_macro2::Ident,
    name: &str,
) -> Option<TokenStream> {
    Some(match name {
        "Atom" => quote!(header.utf8(self.#field_name);),
        "Wtf8Atom" => quote!(header.wtf8(self.#field_name);),
        _ => return None,
    })
}

fn is_flag_field(field: &AstStructField, schema: &Schema) -> bool {
    let AstType::Primitive(ast) = &schema.types[field.type_id] else {
        return false;
    };

    ast.name == "bool" || simple_enum_flag_width(ast.name).is_some()
}
