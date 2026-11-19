use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Meta, Pat, Safety};

use crate::{is_byte_slice, napi_function_meta, result_value_type, return_type};

pub(super) fn expand_function(function: &ItemFn, arguments: Vec<Meta>) -> syn::Result<TokenStream> {
    let mut native_arguments = Vec::new();
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
        if is_byte_slice(semantic_type) {
            native_arguments.push(quote!(#ident: napi::bindgen_prelude::Buffer));
            call_arguments.push(quote!(&#ident));
            continue;
        }

        let decode_type = (**semantic_type).clone();
        native_arguments.push(quote! {
            #ident: <<#decode_type as ::swc_next_bridge::BridgeDecode>::Wire as
                ::swc_next_bridge::NativeAbi>::Abi
        });
        conversions.push(quote! {
            let #ident = <<#decode_type as ::swc_next_bridge::BridgeDecode>::Wire as
                ::swc_next_bridge::NativeAbi>::from_native(#ident);
            let Some(#ident) = <#decode_type as ::swc_next_bridge::BridgeDecode>::decode(#ident)
            else {
                return Err(napi::Error::new(
                    napi::Status::InvalidArg,
                    concat!("invalid bridge value for `", stringify!(#ident), "`"),
                ));
            };
        });
        call_arguments.push(quote!(#ident));
    }

    let declared_return_type = return_type(&function.sig.output);
    let (return_type, fallible) = result_value_type(&declared_return_type)
        .map_or((declared_return_type.clone(), false), |ty| {
            (ty.clone(), true)
        });

    let attributes = &function.attrs;
    let visibility = &function.vis;
    let ident = &function.sig.ident;
    let native_adapter = format_ident!("__swc_bridge_napi_{ident}");
    let call = if matches!(function.sig.safety, Safety::Unsafe(_)) {
        quote!(unsafe { #ident(#(#call_arguments),*) })
    } else {
        quote!(#ident(#(#call_arguments),*))
    };
    let call = if fallible {
        quote! {
            let result = #call
                .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        }
    } else {
        quote!(let result = #call;)
    };
    let napi = napi_function_meta(arguments, function)?;

    Ok(quote! {
        #[cfg(not(target_family = "wasm"))]
        const _: () = {
            // NAPI-RS emits relative `napi::` paths. Keep its import local to each
            // adapter so callers need neither a dependency nor a module-level import.
            use ::swc_next_bridge::napi;

            #(#attributes)*
            #[#napi]
            #visibility fn #native_adapter(
                #(#native_arguments),*
            ) -> napi::Result<<<#return_type as ::swc_next_bridge::BridgeEncode>::Wire as
                ::swc_next_bridge::NativeAbi>::Abi> {
                #(#conversions)*
                #call
                let result = <#return_type as ::swc_next_bridge::BridgeEncode>::encode(result);
                Ok(<<#return_type as ::swc_next_bridge::BridgeEncode>::Wire as
                    ::swc_next_bridge::NativeAbi>::into_native(result))
            }
        };
    })
}

/// Use the one-byte N-API constructor for enum spellings known to be ASCII.
/// Other spellings and explicit variant overrides retain napi-rs conversion.
pub(super) fn ascii_enum_conversion(enumeration: &syn::ItemEnum) -> Option<TokenStream> {
    if enumeration.variants.is_empty()
        || enumeration.variants.iter().any(|variant| {
            !variant
                .ident
                .to_string()
                .bytes()
                .all(|byte| byte.is_ascii_alphabetic())
                || variant
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().is_ident("napi"))
        })
    {
        return None;
    }
    let ident = &enumeration.ident;
    let arms = enumeration.variants.iter().map(|variant| {
        let name = &variant.ident;
        let spelling = name.to_string().to_ascii_lowercase();
        let bytes = syn::LitByteStr::new(spelling.as_bytes(), name.span());
        quote!(#ident::#name => #bytes)
    });
    Some(quote! {
        #[cfg(not(target_family = "wasm"))]
        const _: () = {
            use ::swc_next_bridge::napi;

            impl napi::bindgen_prelude::ToNapiValue for &#ident {
                unsafe fn to_napi_value(
                    env: napi::sys::napi_env,
                    value: Self,
                ) -> napi::Result<napi::sys::napi_value> {
                    let bytes: &[u8] = match value { #(#arms,)* };
                    let mut result = ::std::ptr::null_mut();
                    napi::check_status!(unsafe {
                        napi::sys::napi_create_string_latin1(
                            env, bytes.as_ptr().cast(), bytes.len() as isize, &mut result,
                        )
                    }, "Failed to convert rust `&str` into napi `string`")?;
                    Ok(result)
                }
            }

            impl napi::bindgen_prelude::ToNapiValue for #ident {
                unsafe fn to_napi_value(
                    env: napi::sys::napi_env,
                    value: Self,
                ) -> napi::Result<napi::sys::napi_value> {
                    unsafe { napi::bindgen_prelude::ToNapiValue::to_napi_value(env, &value) }
                }
            }

            impl napi::bindgen_prelude::ToNapiValue for &mut #ident {
                unsafe fn to_napi_value(
                    env: napi::sys::napi_env,
                    value: Self,
                ) -> napi::Result<napi::sys::napi_value> {
                    unsafe { napi::bindgen_prelude::ToNapiValue::to_napi_value(env, &*value) }
                }
            }
        };
    })
}
