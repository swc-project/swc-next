use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema},
    util::map_field_type_to_extra_field,
};

pub fn ast_extra_data(schema: &Schema) -> RawOutput {
    let mut typed_fields = TokenStream::new();

    for ty in &schema.types {
        match ty {
            AstType::Struct(ast_struct) => {
                let field = format_ident!("{}", map_field_type_to_extra_field(ty, schema));
                let ident = format_ident!("{}", ast_struct.name);
                typed_fields.extend(quote!(#field: #ident,));
            }
            AstType::Enum(ast_enum) if !schema.repr_sizes.contains_key(&ast_enum.name) => {
                let field = format_ident!("{}", map_field_type_to_extra_field(ty, schema));
                typed_fields.extend(quote!(#field: u64,));
            }
            AstType::Option(ast_option) => {
                let inner = &schema.types[ast_option.inner_type_id];
                match inner {
                    AstType::Struct(_) => {
                        let field = format_ident!("{}", map_field_type_to_extra_field(ty, schema));
                        let ident = inner.repr_ident(schema);
                        typed_fields.extend(quote!(#field: Option<#ident>,));
                    }
                    AstType::Enum(ast_enum) if !schema.repr_sizes.contains_key(&ast_enum.name) => {
                        let field = format_ident!("{}", map_field_type_to_extra_field(ty, schema));
                        typed_fields.extend(quote!(#field: u64,));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let output = quote! {
        /// Compact values stored in the flattening AST's extra-data table.
        #[derive(Clone, Copy)]
        pub union ExtraData {
            atom: AtomId,
            wtf8_atom: Wtf8AtomId,

            span: Span,
            optional_span: OptionalSpan,
            bool: bool,
            number: f64,
            sub_range: SubRange,
            optional_sub_range: OptionalSubRange,
            optional_scope_id: Option<ScopeId>,
            optional_symbol_id: Option<SymbolId>,

            #typed_fields

            /// Any scalar enum or other value representable within eight bytes.
            other: u64,
        }
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_extra_data"),
        tokens: output,
    }
    .into()
}
