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
                let field_ident = format_ident!("{}", map_field_type_to_extra_field(ty, schema));

                impls.extend(quote! {
                   impl ExtraDataLayout for #ident {
                        const WORDS: usize = (std::mem::size_of::<Self>() + 3) / 4;
                   }

                   impl<'a> ExtraDataCompact<'a> for #ident {
                        #[inline]
                        fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
                            let mut data = ExtraData { other: 0 };
                            data.#field_ident = self;
                            data
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
                            unsafe { data.#field_ident }
                        }
                   }
                });
            }
            AstType::Enum(ast_enum) => {
                for variant in &ast_enum.variants {
                    if let Some(type_id) = variant.type_id {
                        assert!(
                            !matches!(schema.types[type_id], AstType::Enum(_)),
                            "{}::{} must inline the nested AST enum variants",
                            ast_enum.name,
                            variant.name,
                        );
                    }
                }

                let ident = format_ident!("{}", ast_enum.name);
                let field_ident = format_ident!("{}", map_field_type_to_extra_field(ty, schema));

                impls.extend(quote! {
                   const _: () =
                       assert!(std::mem::size_of::<#ident>() == std::mem::size_of::<u64>());

                   impl ExtraDataLayout for #ident {
                        const WORDS: usize = 2;
                   }

                   impl<'a> ExtraDataCompact<'a> for #ident {
                        #[inline]
                        fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
                            ExtraData {
                                #field_ident: unsafe { std::mem::transmute::<Self, u64>(self) }
                            }
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
                            unsafe {
                                std::mem::transmute::<u64, Self>(data.#field_ident)
                            }
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

                let optional_extra_field = map_field_type_to_extra_field(ty, schema);
                let opt_field_ident = format_ident!("{}", optional_extra_field);
                let is_enum = matches!(inner_type, AstType::Enum(_));
                let (opt_to, opt_from) = if is_enum {
                    (
                        quote!(ExtraData {
                            #opt_field_ident: self.map_or(u64::MAX, |node| unsafe {
                                node.to_extra_data(ast).other
                            })
                        }),
                        quote!({
                            let raw = unsafe { data.#opt_field_ident };
                            (raw != u64::MAX).then(|| unsafe {
                                ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast)
                            })
                        }),
                    )
                } else {
                    (
                        quote!({
                            let mut data = ExtraData { other: 0 };
                            data.#opt_field_ident = self;
                            data
                        }),
                        quote!(unsafe { data.#opt_field_ident }),
                    )
                };

                let ty_ident = ty.repr_ident(schema);
                let words = if is_enum {
                    quote!(2)
                } else {
                    quote!((std::mem::size_of::<Self>() + 3) / 4)
                };
                impls.extend(quote! {
                   impl ExtraDataLayout for #ty_ident {
                        const WORDS: usize = #words;
                   }

                   impl<'a> ExtraDataCompact<'a> for #ty_ident {
                        #[inline]
                        fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
                            #opt_to
                        }

                        #[inline]
                        unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
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
