use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Expr, FnArg, GenericArgument, ImplItem, Item, ItemEnum, ItemFn, ItemImpl, Lit, LitInt, LitStr,
    Meta, Pat, PathArguments, ReturnType, Token, Type, Visibility, parse::Parser,
    parse_macro_input, parse_quote, punctuated::Punctuated,
};

use crate::metadata::{
    BridgeDescriptor, BridgeFunction, BridgeParameter, StringEnum, StringEnumVariant,
    WireDirection, WireMapping, expand_metadata,
};

mod metadata;
mod napi;
mod wasm;

#[proc_macro_attribute]
pub fn bridge(args: TokenStream, input: TokenStream) -> TokenStream {
    let arguments = match Punctuated::<Meta, Token![,]>::parse_terminated.parse(args) {
        Ok(arguments) => arguments.into_iter().collect::<Vec<_>>(),
        Err(error) => return error.into_compile_error().into(),
    };
    let item = parse_macro_input!(input as Item);
    let descriptors = match describe_bridge_item(&item, &arguments) {
        Ok(descriptors) => descriptors,
        Err(error) => return error.into_compile_error().into(),
    };
    let metadata = match expand_metadata(&descriptors) {
        Ok(metadata) => metadata,
        Err(error) => return error.into_compile_error().into(),
    };
    match expand_bridge(item, arguments) {
        Ok(expanded) => quote! {
            #expanded
            #metadata
        }
        .into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn describe_bridge_item(item: &Item, arguments: &[Meta]) -> syn::Result<Vec<BridgeDescriptor>> {
    match item {
        Item::Enum(enumeration) => {
            describe_enum(enumeration, arguments).map(|descriptor| descriptor.into_iter().collect())
        }
        Item::Fn(function) => {
            describe_function(function, arguments).map(|descriptor| vec![descriptor])
        }
        Item::Impl(implementation) => {
            describe_wire_mapping(implementation, arguments).map(|descriptor| vec![descriptor])
        }
        Item::Struct(_) => Ok(Vec::new()),
        _ => Ok(Vec::new()),
    }
}

fn describe_function(function: &ItemFn, arguments: &[Meta]) -> syn::Result<BridgeDescriptor> {
    validate_function(function)?;
    let rust_name = function.sig.ident.to_string();
    let js_name = napi_js_name(arguments)?.unwrap_or_else(|| to_lower_camel_case(&rust_name));
    let wasm_export_name = format!("__swc_{}", rust_name.trim_start_matches("r#"));
    let parameters = function
        .sig
        .inputs
        .iter()
        .map(|input| {
            let syn::FnArg::Typed(argument) = input else {
                return Err(syn::Error::new_spanned(
                    input,
                    "#[bridge] functions cannot have a receiver",
                ));
            };
            let syn::Pat::Ident(pattern) = argument.pat.as_ref() else {
                return Err(syn::Error::new_spanned(
                    &argument.pat,
                    "#[bridge] parameters must use identifier patterns",
                ));
            };
            let semantic_type = if is_byte_slice(&argument.ty) {
                "Vec<u8>".to_owned()
            } else {
                type_key(&argument.ty)
            };
            Ok(BridgeParameter {
                rust_name: pattern.ident.to_string(),
                js_name: to_lower_camel_case(&pattern.ident.to_string()),
                semantic_type,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let declared_result = return_type(&function.sig.output);
    let (result, fallible) = result_value_type(&declared_result)
        .map_or((&declared_result, false), |result| (result, true));

    Ok(BridgeDescriptor::Function(BridgeFunction {
        rust_name,
        js_name,
        wasm_export_name,
        parameters,
        result: type_key(result),
        fallible,
    }))
}

fn describe_enum(
    enumeration: &ItemEnum,
    arguments: &[Meta],
) -> syn::Result<Option<BridgeDescriptor>> {
    if string_enum_case(arguments)?.is_none() {
        return Ok(None);
    }
    let rust_name = enumeration.ident.to_string();
    let js_name = napi_js_name(arguments)?.unwrap_or_else(|| rust_name.clone());
    let variants = enumeration
        .variants
        .iter()
        .map(|variant| StringEnumVariant {
            rust_name: variant.ident.to_string(),
            js_value: variant.ident.to_string().to_lowercase(),
        })
        .collect();
    Ok(Some(BridgeDescriptor::StringEnum(StringEnum {
        rust_name,
        js_name,
        variants,
    })))
}

fn expand_bridge(item: Item, arguments: Vec<Meta>) -> syn::Result<proc_macro2::TokenStream> {
    match item {
        Item::Enum(enumeration) => expand_enum(enumeration, arguments),
        Item::Fn(function) => expand_function(function, arguments),
        Item::Impl(implementation) => expand_wire_impl(implementation, arguments),
        Item::Struct(structure) => {
            let napi = napi_meta(&arguments);
            Ok(quote! {
                #[cfg_attr(not(target_family = "wasm"), #napi)]
                #structure
            })
        }
        _ => Err(syn::Error::new_spanned(
            item,
            "#[bridge] supports functions, enums, structs, and bridge wire impls",
        )),
    }
}

fn expand_wire_impl(
    implementation: ItemImpl,
    arguments: Vec<Meta>,
) -> syn::Result<proc_macro2::TokenStream> {
    describe_wire_mapping(&implementation, &arguments)?;
    Ok(quote!(#implementation))
}

fn describe_wire_mapping(
    implementation: &ItemImpl,
    arguments: &[Meta],
) -> syn::Result<BridgeDescriptor> {
    if !arguments.is_empty() {
        return Err(syn::Error::new_spanned(
            implementation,
            "#[bridge] wire impls do not accept arguments",
        ));
    }
    if !implementation.generics.params.is_empty() || implementation.generics.where_clause.is_some()
    {
        return Err(syn::Error::new_spanned(
            &implementation.generics,
            "#[bridge] wire impls cannot be generic",
        ));
    }
    let Some((trait_path, _)) = &implementation.trait_ else {
        return Err(syn::Error::new_spanned(
            implementation,
            "#[bridge] on an impl requires BridgeDecode or BridgeEncode",
        ));
    };
    let Some(trait_name) = trait_path.segments.last().map(|segment| &segment.ident) else {
        return Err(syn::Error::new_spanned(
            trait_path,
            "missing bridge wire trait name",
        ));
    };
    let direction = if trait_name == "BridgeDecode" {
        WireDirection::Decode
    } else if trait_name == "BridgeEncode" {
        WireDirection::Encode
    } else {
        return Err(syn::Error::new_spanned(
            trait_path,
            "#[bridge] on an impl requires BridgeDecode or BridgeEncode",
        ));
    };

    let mut wire_types = implementation.items.iter().filter_map(|item| {
        let ImplItem::Type(associated_type) = item else {
            return None;
        };
        (associated_type.ident == "Wire").then_some(&associated_type.ty)
    });
    let Some(wire_type) = wire_types.next() else {
        return Err(syn::Error::new_spanned(
            implementation,
            "#[bridge] wire impl must define `type Wire`",
        ));
    };
    if wire_types.next().is_some() {
        return Err(syn::Error::new_spanned(
            implementation,
            "#[bridge] wire impl defines `type Wire` more than once",
        ));
    }

    Ok(BridgeDescriptor::WireMapping(WireMapping {
        direction,
        semantic_type: type_key(&implementation.self_ty),
        wire_type: type_key(wire_type),
    }))
}

fn expand_enum(
    enumeration: ItemEnum,
    arguments: Vec<Meta>,
) -> syn::Result<proc_macro2::TokenStream> {
    let napi = napi_meta(&arguments);
    let Some(string_enum_case) = string_enum_case(&arguments)? else {
        return Ok(quote! {
            #[cfg_attr(not(target_family = "wasm"), #napi)]
            #enumeration
        });
    };
    if string_enum_case != "lowercase" {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[bridge] currently supports only `string_enum = \"lowercase\"`",
        ));
    }

    for variant in &enumeration.variants {
        if !matches!(variant.fields, syn::Fields::Unit) || variant.discriminant.is_some() {
            return Err(syn::Error::new_spanned(
                variant,
                "#[bridge(string_enum)] requires unit variants without discriminants",
            ));
        }
    }

    let ascii_conversion = if arguments
        .iter()
        .any(|argument| argument.path().is_ident("object_to_js"))
    {
        None
    } else {
        napi::ascii_enum_conversion(&enumeration)
    };
    let napi = if ascii_conversion.is_some() {
        let mut arguments = arguments.clone();
        arguments.push(parse_quote!(object_to_js = false));
        napi_meta(&arguments)
    } else {
        napi
    };
    let ident = &enumeration.ident;
    let variants = enumeration.variants.iter().map(|variant| &variant.ident);
    let encode_arms = variants.clone().enumerate().map(|(index, variant)| {
        let index = LitInt::new(&format!("{index}u32"), variant.span());
        quote!(Self::#variant => #index)
    });
    let decode_arms = variants.enumerate().map(|(index, variant)| {
        let index = LitInt::new(&format!("{index}u32"), variant.span());
        quote!(#index => Some(Self::#variant))
    });

    Ok(quote! {
        #[cfg_attr(not(target_family = "wasm"), #napi)]
        #enumeration
        #ascii_conversion

        impl ::swc_next_bridge::private::Sealed for #ident {}
        impl ::swc_next_bridge::BridgeWire for #ident {}

        impl ::swc_next_bridge::BridgeDecode for #ident {
            type Wire = Self;

            fn decode(value: Self::Wire) -> Option<Self> {
                Some(value)
            }
        }

        impl ::swc_next_bridge::BridgeEncode for #ident {
            type Wire = Self;

            fn encode(self) -> Self::Wire {
                self
            }
        }

        #[cfg(not(target_family = "wasm"))]
        impl ::swc_next_bridge::NativeAbi for #ident {
            type Abi = Self;

            fn from_native(value: Self::Abi) -> Self {
                value
            }

            fn into_native(self) -> Self::Abi {
                self
            }
        }

        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        unsafe impl ::swc_next_bridge::wasm::FromWasmAbi for #ident {
            type Abi = u32;

            unsafe fn from_wasm_abi(value: Self::Abi) -> Option<Self> {
                match value {
                    #(#decode_arms,)*
                    _ => None,
                }
            }
        }

        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        impl ::swc_next_bridge::wasm::IntoWasmAbi for #ident {
            type Abi = u32;

            fn into_wasm_abi(self) -> Self::Abi {
                match self {
                    #(#encode_arms,)*
                }
            }
        }
    })
}

fn expand_function(
    function: ItemFn,
    arguments: Vec<Meta>,
) -> syn::Result<proc_macro2::TokenStream> {
    validate_function(&function)?;
    let napi = napi::expand_function(&function, arguments)?;
    let wasm = wasm::expand_function(&function)?;

    Ok(quote! {
        #function
        #napi
        #wasm
    })
}

fn validate_function(function: &ItemFn) -> syn::Result<()> {
    let signature = &function.sig;
    if !matches!(function.vis, Visibility::Public(_)) {
        return Err(syn::Error::new_spanned(
            &function.vis,
            "#[bridge] functions must be public",
        ));
    }
    if signature.constness.is_some()
        || signature.asyncness.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
    {
        return Err(syn::Error::new_spanned(
            signature,
            "#[bridge] requires a non-generic, non-async Rust function",
        ));
    }
    Ok(())
}

pub(crate) fn napi_function_meta(
    mut arguments: Vec<Meta>,
    function: &ItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
    if napi_js_name(&arguments)?.is_none() {
        let ident = &function.sig.ident;
        let js_name = LitStr::new(&to_lower_camel_case(&ident.to_string()), ident.span());
        arguments.push(parse_quote!(js_name = #js_name));
    }

    let overrides_entire_type = arguments
        .iter()
        .any(|argument| argument.path().is_ident("ts_type"));
    if !overrides_entire_type
        && !arguments
            .iter()
            .any(|argument| argument.path().is_ident("ts_args_type"))
    {
        let value = LitStr::new(&infer_ts_arguments(function)?, function.sig.ident.span());
        arguments.push(parse_quote!(ts_args_type = #value));
    }
    if !overrides_entire_type
        && !arguments
            .iter()
            .any(|argument| argument.path().is_ident("ts_return_type"))
    {
        let value = LitStr::new(
            &infer_ts_return_type(&function.sig.output)?,
            function.sig.ident.span(),
        );
        arguments.push(parse_quote!(ts_return_type = #value));
    }

    Ok(napi_meta(&arguments))
}

fn infer_ts_arguments(function: &ItemFn) -> syn::Result<String> {
    let arguments = function
        .sig
        .inputs
        .iter()
        .map(|input| {
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
            let optional = option_inner_type(&argument.ty);
            let ty = infer_ts_type(optional.unwrap_or(&argument.ty), TypePosition::Argument)?;
            Ok((
                to_lower_camel_case(&pattern.ident.to_string()),
                ty,
                optional.is_some(),
            ))
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let last_required = arguments.iter().rposition(|(_, _, optional)| !optional);

    Ok(arguments
        .into_iter()
        .enumerate()
        .map(|(index, (name, mut ty, optional))| {
            if optional {
                ty.push_str(" | undefined | null");
            }
            let optional_marker = optional && last_required.is_none_or(|required| index > required);
            format!("{name}{}: {ty}", if optional_marker { "?" } else { "" })
        })
        .collect::<Vec<_>>()
        .join(", "))
}

fn infer_ts_return_type(output: &ReturnType) -> syn::Result<String> {
    let ty = return_type(output);
    let ty = result_value_type(&ty).unwrap_or(&ty);
    if let Some(inner) = option_inner_type(ty) {
        return Ok(format!(
            "{} | undefined | null",
            infer_ts_type(inner, TypePosition::Return)?
        ));
    }
    infer_ts_type(ty, TypePosition::Return)
}

#[derive(Clone, Copy)]
enum TypePosition {
    Argument,
    Return,
}

fn infer_ts_type(ty: &Type, position: TypePosition) -> syn::Result<String> {
    match ty {
        Type::Group(group) => infer_ts_type(&group.elem, position),
        Type::Paren(paren) => infer_ts_type(&paren.elem, position),
        Type::Reference(reference) => infer_ts_type(&reference.elem, position),
        Type::Slice(slice) if is_ident_type(&slice.elem, "u8") => Ok("Buffer".into()),
        Type::Tuple(tuple) if tuple.elems.is_empty() => match position {
            TypePosition::Argument => Err(syn::Error::new_spanned(
                ty,
                "unit is not supported as a #[bridge] parameter",
            )),
            TypePosition::Return => Ok("void".into()),
        },
        Type::Path(path) if path.qself.is_none() => {
            let Some(segment) = path.path.segments.last() else {
                return cannot_infer_ts_type(ty);
            };
            let ident = segment.ident.to_string();
            match ident.as_str() {
                "u32" | "i32" | "f64" if matches!(segment.arguments, PathArguments::None) => {
                    Ok("number".into())
                }
                "bool" if matches!(segment.arguments, PathArguments::None) => Ok("boolean".into()),
                "String" | "str" if matches!(segment.arguments, PathArguments::None) => {
                    Ok("string".into())
                }
                "Vec" if is_byte_vec(ty) => Ok("Buffer".into()),
                _ if matches!(segment.arguments, PathArguments::None) => Ok(ident),
                _ => cannot_infer_ts_type(ty),
            }
        }
        _ => cannot_infer_ts_type(ty),
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    single_type_argument(&segment.arguments)
}

fn cannot_infer_ts_type<T>(ty: &Type) -> syn::Result<T> {
    Err(syn::Error::new_spanned(
        ty,
        "cannot infer a TypeScript bridge type; specify `ts_args_type` or `ts_return_type`",
    ))
}

fn napi_meta(arguments: &[Meta]) -> proc_macro2::TokenStream {
    if arguments.is_empty() {
        quote!(::swc_next_bridge::napi_derive::napi)
    } else {
        quote!(::swc_next_bridge::napi_derive::napi(#(#arguments),*))
    }
}

fn napi_js_name(arguments: &[Meta]) -> syn::Result<Option<String>> {
    let mut names = arguments
        .iter()
        .filter(|argument| argument.path().is_ident("js_name"))
        .map(|argument| meta_string_value(argument, "js_name"));
    let name = names.next().transpose()?;
    if let Some(duplicate) = names.next() {
        duplicate?;
        return Err(syn::Error::new(Span::call_site(), "duplicate `js_name`"));
    }
    Ok(name)
}

fn string_enum_case(arguments: &[Meta]) -> syn::Result<Option<String>> {
    let mut cases = arguments
        .iter()
        .filter(|argument| argument.path().is_ident("string_enum"))
        .map(|argument| meta_string_value(argument, "string_enum"));
    let case = cases.next().transpose()?;
    if let Some(duplicate) = cases.next() {
        duplicate?;
        return Err(syn::Error::new(
            Span::call_site(),
            "duplicate `string_enum`",
        ));
    }
    Ok(case)
}

fn meta_string_value(meta: &Meta, name: &str) -> syn::Result<String> {
    let Meta::NameValue(argument) = meta else {
        return Err(syn::Error::new_spanned(
            meta,
            format!("`{name}` must be a string literal"),
        ));
    };
    let Expr::Lit(value) = &argument.value else {
        return Err(syn::Error::new_spanned(
            &argument.value,
            format!("`{name}` must be a string literal"),
        ));
    };
    let Lit::Str(value) = &value.lit else {
        return Err(syn::Error::new_spanned(
            &value.lit,
            format!("`{name}` must be a string literal"),
        ));
    };
    Ok(value.value())
}

fn type_key(ty: &Type) -> String {
    ty.to_token_stream()
        .to_string()
        .split_whitespace()
        .collect()
}

pub(crate) fn return_type(output: &ReturnType) -> Type {
    match output {
        ReturnType::Default => parse_quote!(()),
        ReturnType::Type(_, ty) => (**ty).clone(),
    }
}

pub(crate) fn result_value_type(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Result" {
        return None;
    }
    single_type_argument(&segment.arguments)
}

fn single_type_argument(arguments: &PathArguments) -> Option<&Type> {
    let PathArguments::AngleBracketed(arguments) = arguments else {
        return None;
    };
    match arguments.args.first()? {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

pub(crate) fn is_byte_slice(ty: &Type) -> bool {
    let Type::Reference(reference) = ty else {
        return false;
    };
    if reference.mutability.is_some() {
        return false;
    }
    let Type::Slice(slice) = reference.elem.as_ref() else {
        return false;
    };
    is_ident_type(&slice.elem, "u8")
}

pub(crate) fn is_byte_vec(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    segment.ident == "Vec"
        && single_type_argument(&segment.arguments).is_some_and(|ty| is_ident_type(ty, "u8"))
}

fn is_ident_type(ty: &Type, expected: &str) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none()
        && path.path.segments.len() == 1
        && path.path.segments[0].ident == expected
        && matches!(path.path.segments[0].arguments, PathArguments::None)
}

fn to_lower_camel_case(value: &str) -> String {
    let value = value.strip_prefix("r#").unwrap_or(value);
    let mut words = value.split('_');
    let mut result = words.next().unwrap_or_default().to_owned();
    for word in words {
        let mut characters = word.chars();
        if let Some(first) = characters.next() {
            result.extend(first.to_uppercase());
            result.extend(characters);
        }
    }
    result
}
