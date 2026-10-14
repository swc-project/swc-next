use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum BridgeDescriptor {
    Function(BridgeFunction),
    StringEnum(StringEnum),
    WireMapping(WireMapping),
}

#[derive(Debug, Deserialize)]
pub(super) struct BridgeFunction {
    pub rust_name: String,
    pub js_name: String,
    pub wasm_export_name: String,
    pub parameters: Vec<BridgeParameter>,
    pub result: String,
    pub fallible: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct BridgeParameter {
    pub rust_name: String,
    pub js_name: String,
    pub semantic_type: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct StringEnum {
    pub rust_name: String,
    pub js_name: String,
    pub variants: Vec<StringEnumVariant>,
}

#[derive(Debug, Deserialize)]
pub(super) struct StringEnumVariant {
    pub js_value: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WireDirection {
    Decode,
    Encode,
}

#[derive(Debug, Deserialize)]
pub(super) struct WireMapping {
    pub direction: WireDirection,
    pub semantic_type: String,
    pub wire_type: String,
}
