use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema, TypeId},
    util::safe_ident,
};

pub fn ast_import(schema: &Schema) -> RawOutput {
    let arms = schema.types.iter().filter_map(|ty| {
        let AstType::Struct(node) = ty else {
            return None;
        };
        let kind = format_ident!("{}", node.name);
        let builder = safe_ident(&node.name.to_case(Case::Snake));
        let fields: Vec<_> = node
            .fields
            .iter()
            .map(|field| safe_ident(&field.name.to_case(Case::Snake)))
            .collect();
        let values = node.fields.iter().zip(&fields).map(|(field, name)| {
            let value = import_value(schema, field.type_id, quote!(node.#name(source_ast)));
            quote!(let #name = #value;)
        });
        Some(quote! {
            NodeKindData::#kind(node) => {
                #(#values)*
                let copied = self.#builder(DUMMY_SP, #(#fields),*);
                // SAFETY: The detached slot has the same node kind as copied.
                // No temporary builder handle is retained.
                unsafe { self.replace_data(mapping.node(node), copied); }
            }
        })
    });
    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_import"),
        tokens: quote! {
            use crate::{Ast, DUMMY_SP, ImportedAst, NodeKindData};

            impl Ast<'_> {
                pub(crate) fn import_nodes(&mut self, source_ast: &Ast<'_>, mapping: ImportedAst) {
                    for index in 0..source_ast.node_count() {
                        let id = crate::NodeId::from_usize_unchecked(index);
                        match source_ast.kind_data(id) {
                            #(#arms)*
                        }
                    }
                }
            }
        },
    }
    .into()
}

fn import_value(schema: &Schema, ty: TypeId, value: TokenStream) -> TokenStream {
    match &schema.types[ty] {
        AstType::Struct(_) | AstType::Enum(_) => quote!(mapping.node(#value)),
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" => quote!(self.add_utf8(source_ast.get_utf8(#value))),
            "AstWtf8" => quote!(self.add_wtf8(source_ast.get_wtf8(#value))),
            "Span" => quote!(DUMMY_SP),
            _ => value,
        },
        AstType::Option(option) => {
            if let AstType::Primitive(primitive) = &schema.types[option.inner_type_id]
                && !matches!(primitive.name, "AstUtf8" | "AstWtf8" | "Span")
            {
                return value;
            }
            let inner = import_value(schema, option.inner_type_id, quote!(value));
            quote!(#value.map(|value| #inner))
        }
        AstType::SubRange(range) => {
            let inner = import_value(
                schema,
                range.inner_type_id,
                quote!(source_ast.get_node_in_sub_range(slot)),
            );
            quote!({
                let values = #value.iter().map(|slot| #inner).collect::<Vec<_>>();
                self.add_sub_range(values)
            })
        }
    }
}
