use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstType, Schema},
};

pub fn ast_extra_compact(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                let ident = format_ident!("{}", ast_struct.name);

                impls.extend(quote! {
                   impl<'a> ExtraDataCompact<'a> for #ident {
                        #[inline]
                        fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
                            ExtraData { node_id: self.node_id() }
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
                            Self(unsafe { data.node_id })
                        }
                   }
                });
            }
            AstType::Enum(ast_enum) => {
                let ident = format_ident!("{}", ast_enum.name);

                impls.extend(quote! {
                   impl<'a> ExtraDataCompact<'a> for #ident {
                        #[inline]
                        fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
                            ExtraData { node_id: self.node_id() }
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
                            Self(unsafe { data.node_id })
                        }
                   }
                });
            }
            AstType::Option(ast_option) => {
                let inner_type = &schema.types[ast_option.inner_type_id];
                // Option<TypedSubRange<T>> is implemented manually
                if matches!(inner_type, AstType::Vec(_) | AstType::Primitive(_)) {
                    continue;
                }

                assert!(matches!(inner_type, AstType::Struct(_) | AstType::Enum(_)));

                let ty_ident = ty.repr_ident(schema);
                let inner_ident = inner_type.repr_ident(schema);
                impls.extend(quote! {
                   impl<'a> ExtraDataCompact<'a> for #ty_ident {
                        #[inline]
                        fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
                            ExtraData {
                                node_id: self.map_or(
                                    NodeId::from_raw_unchecked(u32::MAX),
                                    |node| node.node_id(),
                                )
                            }
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
                            let node_id = unsafe { data.node_id };
                            (node_id.index() != u32::MAX as usize).then_some(#inner_ident(node_id))
                        }
                   }
                });
            }
            _ => {}
        }
    }

    let output = quote! {
        #![allow(unused, clippy::manual_map)]
        use crate::{Ast, CloneIn, ExtraData};
        use crate::{ast::*, node_id::*};
        use oxc_index::Idx;

        #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_extra_compact"),
        tokens: output,
    }
    .into()
}
