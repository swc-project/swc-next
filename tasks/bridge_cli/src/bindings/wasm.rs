use wasmparser::{Encoding, Parser, Payload};

use super::descriptor::BridgeDescriptor;

const METADATA_SECTION: &str = "__wasm_bindgen_unstable";

pub(super) fn extract_metadata(bytes: &[u8]) -> Result<(Vec<BridgeDescriptor>, Vec<u8>), String> {
    let mut descriptors = Vec::new();
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut retained_start = 0;
    let mut section_start = 8;
    let mut found_metadata = false;

    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|error| format!("invalid WebAssembly module: {error}"))?;
        if let Payload::Version { num, encoding, .. } = &payload
            && (*encoding != Encoding::Module || *num != 1)
        {
            return Err("expected a core WebAssembly module".to_owned());
        }
        let Some((_, range)) = payload.as_section() else {
            continue;
        };
        if let Payload::CustomSection(section) = &payload
            && section.name() == METADATA_SECTION
        {
            found_metadata = true;
            // The linker concatenates the JSON arrays emitted by individual items.
            for item in serde_json::Deserializer::from_slice(section.data())
                .into_iter::<Vec<BridgeDescriptor>>()
            {
                descriptors.extend(item.map_err(|error| {
                    format!(
                        "invalid bridge metadata at offset {}: {error}",
                        section.data_offset()
                    )
                })?);
            }
            // Preserve all other sections byte-for-byte, including their headers.
            stripped.extend_from_slice(&bytes[retained_start..section_start]);
            retained_start = range.end;
        }
        section_start = range.end;
    }

    if !found_metadata {
        return Err(format!(
            "missing WebAssembly custom section {METADATA_SECTION}"
        ));
    }
    stripped.extend_from_slice(&bytes[retained_start..]);
    Ok((descriptors, stripped))
}
