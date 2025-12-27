use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema},
    util::map_field_type_to_extra_field,
};

pub fn ast_extra_compact(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                let ident = format_ident!("{}", ast_struct.name);
                let (to_expr, from_expr) = (
                    quote!(ExtraData {
                        node: self.node_id()
                    }),
                    quote!(unsafe { Self::from_node_id_unchecked(data.node, ast) }),
                );

                impls.extend(quote! {
                   impl ExtraDataCompact for #ident {
                        #[inline]
                        fn to_extra_data(self) -> ExtraData {
                            #to_expr
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
                            #from_expr
                        }
                   }
                });
            }
            AstType::Enum(ast_enum) => {
                let ident = format_ident!("{}", ast_enum.name);
                let extra_field = map_field_type_to_extra_field(ty, schema);

                let (to_expr, from_expr) = match extra_field.as_str() {
                    "node" => (
                        quote!(ExtraData {
                            node: self.node_id()
                        }),
                        quote!(unsafe { Self::from_node_id_unchecked(data.node, ast) }),
                    ),
                    "other" => {
                        if let Some(&size) = schema.repr_sizes.get(&ast_enum.name) {
                            let cast_ty = match size {
                                1 => quote!(u8),
                                2 => quote!(u16),
                                4 => quote!(u32),
                                8 => quote!(u64),
                                _ => quote!(u64),
                            };

                            (
                                quote!(ExtraData { other: self as u64 }),
                                quote!(unsafe { std::mem::transmute(data.other as #cast_ty) }),
                            )
                        } else {
                            panic!("Invalid type size {}", ast_enum.name);
                        }
                    }
                    _ => {
                        let field_ident = format_ident!("{}", extra_field);
                        (
                            quote!(ExtraData { #field_ident: self }),
                            quote!(unsafe { data.#field_ident }),
                        )
                    }
                };

                impls.extend(quote! {
                   impl ExtraDataCompact for #ident {
                        #[inline]
                        fn to_extra_data(self) -> ExtraData {
                            #to_expr
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
                            #from_expr
                        }
                   }
                });
            }
            AstType::Option(ast_option) => {
                let inner_type = &schema.types[ast_option.inner_type_id];
                // Option<TypedSubRange<T>> is implemented manually
                if matches!(inner_type, AstType::Vec(_)) {
                    continue;
                }

                let optional_extra_field = map_field_type_to_extra_field(ty, schema);
                let inner_ty_ident = inner_type.repr_ident(schema);
                let opt_field_ident = format_ident!("{}", optional_extra_field);
                let (opt_to, opt_from) = if optional_extra_field == "optional_node" {
                    (
                        quote!(ExtraData {
                            optional_node: self.map(|n| n.node_id()).into()
                        }),
                        quote!(unsafe {
                            data.optional_node
                                .to_option()
                                .map(|id| #inner_ty_ident::from_node_id_unchecked(id, ast))
                        }),
                    )
                } else {
                    (
                        quote!(ExtraData { #opt_field_ident: self }),
                        quote!(unsafe { data.#opt_field_ident }),
                    )
                };

                let ty_ident = ty.repr_ident(schema);
                impls.extend(quote! {
                   impl ExtraDataCompact for #ty_ident {
                        #[inline]
                        fn to_extra_data(self) -> ExtraData {
                            #opt_to
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
                            #opt_from
                        }
                   }
                });
            }
            _ => {}
        }
    }

    let output = quote! {
        #![allow(unused, clippy::manual_map)]
        use crate::{Ast, CloneIn, NodeKind, ExtraData};
        use crate::{ast::*, node_id::*};

        #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_extra_compact"),
        tokens: output,
    }
    .into()
}
