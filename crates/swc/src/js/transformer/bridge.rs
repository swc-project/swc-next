//! For internal SWC Next use only. These APIs and packed option/result
//! formats do not follow semantic versioning and may change in any release.
//! Use matching `@swc-next/encoder/transform` helpers and native/Wasm bindings.
//!
//! Binary source transformation bridge. Options use a 16-byte little-endian
//! header, optionally followed by a length-prefixed UTF-8 filename. Empty options
//! select defaults. Output starts with code length, optional map length
//! (`u32::MAX` means absent), and diagnostic count, followed by UTF-8 code/map
//! and the parser bridge's packed diagnostic records.

use swc_next_bridge::{BridgeDecode, BridgeDecodeOptional, bridge};
use swc_next_ecma_ast::ffi::ast::{diagnostics_buffer_size, serialize_diagnostics};

use super::transform_js;
use crate::js::{
    CommentsOption, JsOutput, JsTransformApiOptions, bridge::Diagnostic,
    generator::JsGeneratorConfig, parser::JsParserConfig,
};

#[cfg(not(target_family = "wasm"))]
use swc_next_bridge::napi;

#[bridge(
    object,
    js_name = "JsOutput",
    object_from_js = false,
    object_to_js = false
)]
pub struct TransformOutput {
    pub code: String,
    /// Source map as a JSON string. Absent when source maps are disabled.
    pub map: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Options for parsing and printing source.
///
/// Built-in transformations, including TypeScript type erasure and JSX lowering,
/// are not currently implemented by the JavaScript API.
#[bridge(object, object_from_js = false, object_to_js = false)]
#[derive(Default)]
pub struct TransformApiOptions {
    /// Used to infer the language and source type, and for diagnostics and source maps.
    pub filename: Option<String>,
    /// Source map generation.
    ///
    /// @default false
    pub source_map: Option<bool>,
    /// Comment policy.
    pub comments: Option<CommentsOption>,
    /// Parser configuration.
    pub parser: Option<JsParserConfig>,
    /// Generator configuration.
    pub generator: Option<JsGeneratorConfig>,
}

impl From<TransformApiOptions> for JsTransformApiOptions {
    fn from(options: TransformApiOptions) -> Self {
        Self {
            filename: options.filename,
            source_map: options.source_map,
            comments: options.comments,
            parser: options.parser,
            generator: options.generator,
            transformer: None,
        }
    }
}

#[bridge]
impl BridgeDecodeOptional for TransformApiOptions {
    type Wire = Vec<u8>;

    fn decode_optional(value: Self::Wire) -> Option<Option<Self>> {
        decode_options(&value)
    }
}

/// Decode the packed transform configuration without copying the input buffer.
pub fn decode_options(bytes: &[u8]) -> Option<Option<TransformApiOptions>> {
    if bytes.is_empty() {
        return Some(None);
    }
    let header = bytes.get(..16)?;
    let word =
        |index: usize| u32::from_le_bytes(header[index * 4..index * 4 + 4].try_into().unwrap());
    if word(0) != 1 || word(3) & !0xf != 0 {
        return None;
    }
    let parser = <Option<JsParserConfig> as BridgeDecode>::decode(word(1))?;
    let generator = <Option<JsGeneratorConfig> as BridgeDecode>::decode(word(2))?;
    let filename = if bytes.len() == 16 {
        None
    } else {
        let length = u32::from_le_bytes(bytes.get(16..20)?.try_into().ok()?) as usize;
        let value = bytes.get(20..)?;
        if value.len() != length {
            return None;
        }
        Some(std::str::from_utf8(value).ok()?.to_owned())
    };
    let source_map = match word(3) & 3 {
        0 => None,
        1 => Some(false),
        2 => Some(true),
        _ => return None,
    };
    let comments = match (word(3) >> 2) & 3 {
        0 => None,
        1 => Some(CommentsOption::All),
        2 => Some(CommentsOption::None),
        _ => return None,
    };
    Some(Some(TransformApiOptions {
        filename,
        source_map,
        comments,
        parser,
        generator,
    }))
}

/// Transform and serialize into a buffer allocated after its exact size is known.
///
/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
fn output_to_buffer<T: AsMut<[u8]>, E>(
    source: &str,
    source_utf16_len: u32,
    output: JsOutput,
    allocate: impl FnOnce(usize) -> Result<T, E>,
) -> Result<T, E> {
    let map = output.map.as_deref().unwrap_or("");
    let size = 12 + output.code.len() + map.len() + diagnostics_buffer_size(&output.diagnostics);
    let mut buffer = allocate(size)?;
    let bytes = buffer.as_mut();
    let header = [
        output.code.len() as u32,
        output.map.as_ref().map_or(u32::MAX, |map| map.len() as u32),
        output.diagnostics.len() as u32,
    ];
    for (word, bytes) in header.into_iter().zip(bytes[..12].as_chunks_mut::<4>().0) {
        bytes.copy_from_slice(&word.to_le_bytes());
    }
    let code_end = 12 + output.code.len();
    bytes[12..code_end].copy_from_slice(output.code.as_bytes());
    let map_end = code_end + map.len();
    bytes[code_end..map_end].copy_from_slice(map.as_bytes());
    serialize_diagnostics(
        source,
        &output.diagnostics,
        &mut bytes[map_end..],
        source_utf16_len as usize,
    );
    Ok(buffer)
}

/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
pub unsafe fn transform_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<TransformApiOptions>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let output = transform_js::<false>(source, options.unwrap_or_default().into());
    let Ok(buffer) = {
        output_to_buffer(source, source_utf16_len, output, |size| {
            Ok::<_, std::convert::Infallible>(vec![0; size])
        })
    };
    buffer
}

/// Return code through the native string bridge when no metadata is needed.
/// Otherwise serialize into a Node-owned buffer.
///
/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
#[cfg(not(target_family = "wasm"))]
pub unsafe fn transform_to_node_output(
    env: &napi::Env,
    source: &[u8],
    source_utf16_len: u32,
    options: &[u8],
) -> napi::Result<
    napi::bindgen_prelude::Either<
        <String as swc_next_bridge::NativeAbi>::Abi,
        napi::bindgen_prelude::Buffer,
    >,
> {
    let options = decode_options(options).ok_or_else(|| {
        napi::Error::new(
            napi::Status::InvalidArg,
            "invalid bridge value for `options`",
        )
    })?;
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let output = transform_js::<false>(source, options.unwrap_or_default().into());
    if output.map.is_none() && output.diagnostics.is_empty() {
        return Ok(napi::bindgen_prelude::Either::A(
            <String as swc_next_bridge::NativeAbi>::into_native(output.code),
        ));
    }
    output_to_buffer(source, source_utf16_len, output, |size| {
        swc_next_bridge::create_buffer(env, size)
    })
    .map(napi::bindgen_prelude::Either::B)
}
