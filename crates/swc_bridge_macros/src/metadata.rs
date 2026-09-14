use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::Serialize;
use syn::LitStr;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum BridgeDescriptor {
    Function(BridgeFunction),
    StringEnum(StringEnum),
    WireMapping(WireMapping),
}

#[derive(Debug, Serialize)]
pub(super) struct BridgeFunction {
    pub rust_name: String,
    pub js_name: String,
    pub wasm_export_name: String,
    pub parameters: Vec<BridgeParameter>,
    pub result: String,
    pub fallible: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct BridgeParameter {
    pub rust_name: String,
    pub js_name: String,
    pub semantic_type: String,
}

#[derive(Debug, Serialize)]
pub(super) struct StringEnum {
    pub rust_name: String,
    pub js_name: String,
    pub variants: Vec<StringEnumVariant>,
}

#[derive(Debug, Serialize)]
pub(super) struct StringEnumVariant {
    pub rust_name: String,
    pub js_value: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WireDirection {
    Decode,
    Encode,
}

#[derive(Debug, Serialize)]
pub(super) struct WireMapping {
    pub direction: WireDirection,
    pub semantic_type: String,
    pub wire_type: String,
}

pub(super) fn expand_metadata(descriptors: &[BridgeDescriptor]) -> syn::Result<TokenStream> {
    let json = serde_json::to_string(descriptors)
        .map_err(|error| syn::Error::new(Span::call_site(), error))?;
    let json = LitStr::new(&json, Span::call_site());
    Ok(quote! {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        const _: () = {
            const BRIDGEN_METADATA: &str = #json;

            #[unsafe(link_section = "__wasm_bindgen_unstable")]
            static _GENERATED: [u8; BRIDGEN_METADATA.len()] = {
                let mut bytes = [0; BRIDGEN_METADATA.len()];
                let mut index = 0;
                while index < bytes.len() {
                    bytes[index] = BRIDGEN_METADATA.as_bytes()[index];
                    index += 1;
                }
                bytes
            };
        };
    })
}
