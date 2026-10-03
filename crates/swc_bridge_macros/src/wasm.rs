use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, LitStr, Pat, Safety, Type, parse_quote};

use crate::{is_byte_slice, result_value_type, return_type};

pub(super) fn expand_function(function: &ItemFn) -> syn::Result<TokenStream> {
    let mut raw_arguments = Vec::new();
    let mut conversions = Vec::new();
    let mut call_arguments = Vec::new();
    for input in &function.sig.inputs {
        let FnArg::Typed(argument) = input else {
            return Err(syn::Error::new_spanned(
                input,
                "#[bridge] functions cannot have a receiver",
            ));
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument.pat,
                "#[bridge] parameters must use identifier patterns",
            ));
        };

        let ident = &pattern.ident;
        let semantic_type = &argument.ty;
        let decode_type: Type = if is_byte_slice(semantic_type) {
            parse_quote!(::std::vec::Vec<u8>)
        } else {
            (**semantic_type).clone()
        };
        raw_arguments.push(quote! {
            #ident: <<#decode_type as ::swc_next_bridge::BridgeDecode>::Wire as
                ::swc_next_bridge::wasm::FromWasmAbi>::Abi
        });
        conversions.push(quote! {
            let #ident = unsafe {
                <<#decode_type as ::swc_next_bridge::BridgeDecode>::Wire as
                    ::swc_next_bridge::wasm::FromWasmAbi>::from_wasm_abi(#ident)
            };
            let Some(#ident) = #ident else {
                ::swc_next_bridge::wasm::invalid_abi();
            };
            let Some(#ident) = <#decode_type as ::swc_next_bridge::BridgeDecode>::decode(#ident)
            else {
                ::swc_next_bridge::wasm::invalid_abi();
            };
        });
        if is_byte_slice(semantic_type) {
            call_arguments.push(quote!(&#ident));
        } else {
            call_arguments.push(quote!(#ident));
        }
    }

    let declared_return_type = return_type(&function.sig.output);
    let (return_type, fallible) = result_value_type(&declared_return_type)
        .map_or((declared_return_type.clone(), false), |ty| {
            (ty.clone(), true)
        });
    let abi_return_type = if fallible {
        quote!(u32)
    } else {
        quote! {
            <<#return_type as ::swc_next_bridge::BridgeEncode>::Wire as
                ::swc_next_bridge::wasm::IntoWasmAbi>::Abi
        }
    };

    let ident = &function.sig.ident;
    let attributes = &function.attrs;
    let adapter = format_ident!("__swc_bridge_wasm_{ident}");
    let export_name = LitStr::new(
        &format!("__swc_{}", ident.to_string().trim_start_matches("r#")),
        ident.span(),
    );
    let call = if matches!(function.sig.safety, Safety::Unsafe(_)) {
        quote!(unsafe { #ident(#(#call_arguments),*) })
    } else {
        quote!(#ident(#(#call_arguments),*))
    };
    let encode = if fallible {
        quote! {
            match #call {
                Ok(result) => {
                    let result = <#return_type as ::swc_next_bridge::BridgeEncode>::encode(result);
                    <<#return_type as ::swc_next_bridge::BridgeEncode>::Wire as
                        ::swc_next_bridge::wasm::IntoWasmResult>::into_wasm_result(result)
                }
                Err(error) => ::swc_next_bridge::wasm::error_result(&error.to_string()),
            }
        }
    } else {
        quote! {
            let result = #call;
            let result = <#return_type as ::swc_next_bridge::BridgeEncode>::encode(result);
            <<#return_type as ::swc_next_bridge::BridgeEncode>::Wire as
                ::swc_next_bridge::wasm::IntoWasmAbi>::into_wasm_abi(result)
        }
    };

    Ok(quote! {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        const _: () = {
            #[used]
            static LINK_RUNTIME: (
                extern "C" fn(u32) -> u32,
                unsafe extern "C" fn(u32),
            ) = (
                ::swc_next_bridge::wasm::alloc,
                ::swc_next_bridge::wasm::free,
            );
        };

        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        #(#attributes)*
        #[doc(hidden)]
        #[allow(improper_ctypes_definitions)]
        #[unsafe(export_name = #export_name)]
        pub unsafe extern "C" fn #adapter(
            #(#raw_arguments),*
        ) -> #abi_return_type {
            #(#conversions)*
            #encode
        }
    })
}
