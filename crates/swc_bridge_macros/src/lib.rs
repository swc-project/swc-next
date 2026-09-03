use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Expr, FnArg, GenericArgument, ImplItem, Item, ItemEnum, ItemFn, ItemImpl, Lit, LitStr, Meta,
    Pat, PathArguments, ReturnType, Token, Type, Visibility, parse::Parser, parse_macro_input,
    parse_quote, punctuated::Punctuated,
};

#[proc_macro_attribute]
pub fn bridge(args: TokenStream, input: TokenStream) -> TokenStream {
    let arguments = match Punctuated::<Meta, Token![,]>::parse_terminated.parse(args) {
        Ok(arguments) => arguments.into_iter().collect::<Vec<_>>(),
        Err(error) => return error.into_compile_error().into(),
    };
    let item = parse_macro_input!(input as Item);
    match expand_bridge(item, arguments) {
        Ok(expanded) => expanded.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_bridge(item: Item, arguments: Vec<Meta>) -> syn::Result<proc_macro2::TokenStream> {
    match item {
        Item::Enum(enumeration) => expand_enum(enumeration, arguments),
        Item::Fn(function) => expand_function(function, arguments),
        Item::Impl(implementation) => expand_wire_impl(implementation, arguments),
        Item::Struct(structure) => {
            let napi = napi_meta(&arguments);
            Ok(quote! {
                #[#napi]
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
    let Some((_, trait_path, _)) = &implementation.trait_ else {
        return Err(syn::Error::new_spanned(
            &implementation,
            "#[bridge] on an impl requires BridgeDecode or BridgeEncode",
        ));
    };
    let Some(trait_name) = trait_path.segments.last().map(|segment| &segment.ident) else {
        return Err(syn::Error::new_spanned(
            trait_path,
            "missing bridge wire trait name",
        ));
    };
    if trait_name != "BridgeDecode" && trait_name != "BridgeEncode" {
        return Err(syn::Error::new_spanned(
            trait_path,
            "#[bridge] on an impl requires BridgeDecode or BridgeEncode",
        ));
    }

    let mut wire_types = implementation.items.iter().filter_map(|item| {
        let ImplItem::Type(associated_type) = item else {
            return None;
        };
        (associated_type.ident == "Wire").then_some(&associated_type.ty)
    });
    if wire_types.next().is_none() {
        return Err(syn::Error::new_spanned(
            &implementation,
            "#[bridge] wire impl must define `type Wire`",
        ));
    }
    if wire_types.next().is_some() {
        return Err(syn::Error::new_spanned(
            &implementation,
            "#[bridge] wire impl defines `type Wire` more than once",
        ));
    }

    Ok(quote!(#implementation))
}

fn expand_enum(
    enumeration: ItemEnum,
    arguments: Vec<Meta>,
) -> syn::Result<proc_macro2::TokenStream> {
    let napi = napi_meta(&arguments);
    let Some(string_enum_case) = string_enum_case(&arguments)? else {
        return Ok(quote! {
            #[#napi]
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

    let ident = &enumeration.ident;
    Ok(quote! {
        #[#napi]
        #enumeration

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
    })
}

fn expand_function(
    function: ItemFn,
    arguments: Vec<Meta>,
) -> syn::Result<proc_macro2::TokenStream> {
    validate_function(&function)?;

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
        if pattern.by_ref.is_some() || pattern.subpat.is_some() {
            return Err(syn::Error::new_spanned(
                pattern,
                "#[bridge] parameters must use identifier patterns",
            ));
        }

        let ident = &pattern.ident;
        let semantic_type = &argument.ty;
        if is_byte_slice(semantic_type) {
            native_arguments.push(quote!(#ident: ::napi::bindgen_prelude::Buffer));
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
                return Err(::napi::Error::new(
                    ::napi::Status::InvalidArg,
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
    let unsafety = &function.sig.unsafety;
    let ident = &function.sig.ident;
    let native_adapter = format_ident!("__swc_bridge_napi_{ident}");
    let call = if unsafety.is_some() {
        quote!(unsafe { #ident(#(#call_arguments),*) })
    } else {
        quote!(#ident(#(#call_arguments),*))
    };
    let call = if fallible {
        quote! {
            let result = #call
                .map_err(|error| ::napi::Error::from_reason(error.to_string()))?;
        }
    } else {
        quote!(let result = #call;)
    };
    let napi = napi_function_meta(arguments, &function)?;

    Ok(quote! {
        #function

        #(#attributes)*
        #[#napi]
        #visibility fn #native_adapter(
            #(#native_arguments),*
        ) -> ::napi::Result<<<#return_type as ::swc_next_bridge::BridgeEncode>::Wire as
            ::swc_next_bridge::NativeAbi>::Abi> {
            #(#conversions)*
            #call
            let result = <#return_type as ::swc_next_bridge::BridgeEncode>::encode(result);
            Ok(<<#return_type as ::swc_next_bridge::BridgeEncode>::Wire as
                ::swc_next_bridge::NativeAbi>::into_native(result))
        }
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

fn napi_function_meta(
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
        quote!(napi_derive::napi)
    } else {
        quote!(napi_derive::napi(#(#arguments),*))
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

fn return_type(output: &ReturnType) -> Type {
    match output {
        ReturnType::Default => parse_quote!(()),
        ReturnType::Type(_, ty) => (**ty).clone(),
    }
}

fn result_value_type(ty: &Type) -> Option<&Type> {
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

fn is_byte_slice(ty: &Type) -> bool {
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

fn is_byte_vec(ty: &Type) -> bool {
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
