use anyhow::{Result, bail, ensure};
use wasmparser::{BinaryReader, Encoding, Imports, Parser, Payload};

use super::descriptor::BridgeDescriptor;

const METADATA_SECTION: &str = "__wasm_bindgen_unstable";

pub(super) fn bind_plugin_import(bytes: &mut Vec<u8>, runtime: &str) -> Result<()> {
    if let Some(output) = replace_plugin_import(bytes, runtime)? {
        *bytes = output;
    }
    Ok(())
}

fn replace_plugin_import(bytes: &[u8], runtime: &str) -> Result<Option<Vec<u8>>> {
    let mut section_start = 8;
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload?;
        let Some((id, range)) = payload.as_section() else {
            continue;
        };
        let range = usize::try_from(range.start)?..usize::try_from(range.end)?;
        if let Payload::ImportSection(imports) = payload {
            for entry in imports {
                let Imports::Single(offset, import) = entry? else {
                    continue;
                };
                if import.module != "swc" || import.name != "run_js_plugins" {
                    continue;
                }
                let offset = usize::try_from(offset)?;
                let mut reader = BinaryReader::new(&bytes[offset..range.end], offset as u64);
                reader.read_string()?;
                let replaced_end = usize::try_from(reader.original_position())?;
                let mut module = Vec::new();
                write_leb(&mut module, runtime.len());
                module.extend_from_slice(runtime.as_bytes());
                let mut output = Vec::with_capacity(bytes.len() + module.len());
                output.extend_from_slice(&bytes[..section_start]);
                output.push(id);
                write_leb(
                    &mut output,
                    range.len() - (replaced_end - offset) + module.len(),
                );
                output.extend_from_slice(&bytes[range.start..offset]);
                output.extend_from_slice(&module);
                output.extend_from_slice(&bytes[replaced_end..]);
                return Ok(Some(output));
            }
            break;
        }
        section_start = range.end;
    }
    Ok(None)
}

fn write_leb(output: &mut Vec<u8>, mut value: usize) {
    while value >= 0x80 {
        output.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

pub(super) fn extract_metadata(bytes: &[u8]) -> Result<(Vec<BridgeDescriptor>, Vec<u8>)> {
    let mut descriptors = Vec::new();
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut retained_start = 0;
    let mut section_start = 8;
    let mut found_metadata = false;

    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload?;
        if let Payload::Version { num, encoding, .. } = &payload
            && (*encoding != Encoding::Module || *num != 1)
        {
            bail!("expected a core WebAssembly module");
        }
        let Some((_, range)) = payload.as_section() else {
            continue;
        };
        let section_end = usize::try_from(range.end)?;
        if let Payload::CustomSection(section) = &payload
            && section.name() == METADATA_SECTION
        {
            found_metadata = true;
            // The linker concatenates the JSON arrays emitted by individual items.
            for item in serde_json::Deserializer::from_slice(section.data())
                .into_iter::<Vec<BridgeDescriptor>>()
            {
                descriptors.extend(item?);
            }
            // Preserve all other sections byte-for-byte, including their headers.
            stripped.extend_from_slice(&bytes[retained_start..section_start]);
            retained_start = section_end;
        }
        section_start = section_end;
    }

    ensure!(found_metadata, "missing bridge metadata");
    stripped.extend_from_slice(&bytes[retained_start..]);
    Ok((descriptors, stripped))
}
