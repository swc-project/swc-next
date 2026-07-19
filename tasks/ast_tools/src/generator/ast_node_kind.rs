use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema},
};

pub fn ast_node_kind(schema: &Schema) -> RawOutput {
    let variants = schema.types.iter().filter_map(|ty| match ty {
        AstType::Struct(ast) => Some(format_ident!("{}", ast.name)),
        _ => None,
    });
    let variants: TokenStream = quote!(#(#variants,)*);

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_node_kind"),
        tokens: quote! {
            /// The concrete type stored at a node ID.
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum NodeKind {
                #variants
            }
        },
    }
    .into()
}
