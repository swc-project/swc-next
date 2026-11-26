//! For internal SWC Next use only. These APIs and packed option/result
//! formats do not follow semantic versioning and may change in any release.
//! Use matching `@swc-next/protocol/transform` helpers and native/Wasm bindings.
//!
//! Binary source transformation bridge. Options use a 16-byte little-endian
//! header, optionally followed by a length-prefixed UTF-8 filename and/or a
//! binary-encoded transformer config. Empty options select defaults.
//! Output starts with code length, optional map length (`u32::MAX` means absent),
//! and diagnostic count, followed by UTF-8 code/map and the parser bridge's
//! packed diagnostic records.

use std::collections::BTreeMap;

use swc_next_bridge::{BridgeDecode, bridge};
use swc_next_ecma_ast::ffi::ast::{diagnostics_buffer_size, serialize_diagnostics};

use super::{
    JsAssumptions, JsDecoratorConfig, JsDecoratorVersion, JsFeature, JsModuleConfig, JsModuleType,
    JsReactRefreshConfig, JsTarget, JsTransformerConfig, JsTypeScriptConfig, JsxConfig, JsxRuntime,
    transform_js_v1 as transform_js,
};

use crate::{
    BoolOr,
    js::{
        CommentsOption, JsOutput, JsTransformApiOptions, bridge::Diagnostic,
        generator::JsGeneratorConfig, parser::JsParserConfig,
    },
};

#[cfg(not(target_family = "wasm"))]
use swc_next_bridge::napi;

#[bridge(object, js_name = "JsOutput")]
pub struct TransformOutput {
    pub code: String,
    /// Source map as a JSON string. Absent when source maps are disabled.
    pub map: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Options for transforming JavaScript and TypeScript source.
#[bridge(object)]
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
    /// Transformer configuration.
    #[bridge(ts_type = "TransformerConfig")]
    pub transformer: Option<JsTransformerConfig>,
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
            transformer: options.transformer,
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], offset: usize) -> Self {
        Self { bytes, offset }
    }

    fn read_u8(&mut self) -> Option<u8> {
        let value = *self.bytes.get(self.offset)?;
        self.offset += 1;
        Some(value)
    }

    fn read_u16(&mut self) -> Option<u16> {
        let slice: [u8; 2] = self.bytes.get(self.offset..self.offset + 2)?.try_into().ok()?;
        self.offset += 2;
        Some(u16::from_le_bytes(slice))
    }

    fn read_u32(&mut self) -> Option<u32> {
        let slice: [u8; 4] = self.bytes.get(self.offset..self.offset + 4)?.try_into().ok()?;
        self.offset += 4;
        Some(u32::from_le_bytes(slice))
    }

    fn read_string(&mut self) -> Option<String> {
        let length = self.read_u32()? as usize;
        let end = self.offset.checked_add(length)?;
        let value = std::str::from_utf8(self.bytes.get(self.offset..end)?).ok()?;
        self.offset = end;
        Some(value.to_owned())
    }
}

fn decode_opt_bool(bits: u32) -> Option<Option<bool>> {
    match bits & 3 {
        0 => Some(None),
        1 => Some(Some(false)),
        2 => Some(Some(true)),
        _ => None,
    }
}

fn decode_jsx_runtime(bits: u32) -> Option<Option<JsxRuntime>> {
    match bits & 3 {
        0 => Some(None),
        1 => Some(Some(JsxRuntime::Classic)),
        2 => Some(Some(JsxRuntime::Automatic)),
        _ => None,
    }
}

fn decode_transformer(reader: &mut Reader<'_>) -> Option<JsTransformerConfig> {
    let flags = reader.read_u32()?;
    if flags & !0x3fff != 0 {
        return None;
    }

    let filename = if (flags & 0x1) != 0 {
        Some(reader.read_string()?)
    } else {
        None
    };

    let target = match (flags >> 1) & 3 {
        0 => None,
        1 => Some(JsTarget::Query(reader.read_string()?)),
        2 => {
            let count = reader.read_u32()? as usize;
            let mut queries = Vec::with_capacity(count.min(64));
            for _ in 0..count {
                queries.push(reader.read_string()?);
            }
            Some(JsTarget::Queries(queries))
        }
        3 => {
            let count = reader.read_u32()? as usize;
            let mut engines = BTreeMap::new();
            for _ in 0..count {
                let key = reader.read_string()?;
                let value = reader.read_string()?;
                engines.insert(key, value);
            }
            Some(JsTarget::Engines(engines))
        }
        _ => unreachable!(),
    };

    let typescript = match (flags >> 3) & 3 {
        0 => None,
        1 => Some(BoolOr::Bool(false)),
        2 => Some(BoolOr::Bool(true)),
        3 => {
            let ts_flags = u32::from(reader.read_u8()?);
            if ts_flags & !0x3f != 0 {
                return None;
            }
            let verbatim_module_syntax = decode_opt_bool(ts_flags)?;
            let jsx_runtime = decode_jsx_runtime(ts_flags >> 2)?;
            let jsx_pragma = if (ts_flags & 0x10) != 0 {
                Some(reader.read_string()?)
            } else {
                None
            };
            let jsx_pragma_frag = if (ts_flags & 0x20) != 0 {
                Some(reader.read_string()?)
            } else {
                None
            };
            Some(BoolOr::Options(JsTypeScriptConfig {
                verbatim_module_syntax,
                #[cfg(feature = "js_transformer_v2")]
                preserve_const_enums: None,
                jsx_runtime,
                jsx_pragma,
                jsx_pragma_frag,
            }))
        }
        _ => unreachable!(),
    };

    let jsx = match (flags >> 5) & 3 {
        0 => None,
        1 => Some(BoolOr::Bool(false)),
        2 => Some(BoolOr::Bool(true)),
        3 => {
            let jsx_flags = reader.read_u32()?;
            if jsx_flags & !0x1_ffff != 0 {
                return None;
            }
            let runtime = decode_jsx_runtime(jsx_flags)?;
            let development = decode_opt_bool(jsx_flags >> 2)?;
            let source_self = decode_opt_bool(jsx_flags >> 4)?;
            let throw_if_namespace = decode_opt_bool(jsx_flags >> 6)?;
            let pure = decode_opt_bool(jsx_flags >> 8)?;
            let display_name = decode_opt_bool(jsx_flags >> 10)?;
            let import_source = if (jsx_flags & 0x4000) != 0 {
                Some(reader.read_string()?)
            } else {
                None
            };
            let pragma = if (jsx_flags & 0x8000) != 0 {
                Some(reader.read_string()?)
            } else {
                None
            };
            let pragma_frag = if (jsx_flags & 0x1_0000) != 0 {
                Some(reader.read_string()?)
            } else {
                None
            };
            let refresh = match (jsx_flags >> 12) & 3 {
                0 => None,
                1 => Some(BoolOr::Bool(false)),
                2 => Some(BoolOr::Bool(true)),
                3 => {
                    let refresh_flags = u32::from(reader.read_u8()?);
                    if refresh_flags & !0x0f != 0 {
                        return None;
                    }
                    let emit_full_signatures = decode_opt_bool(refresh_flags)?;
                    let refresh_reg = if (refresh_flags & 0x04) != 0 {
                        Some(reader.read_string()?)
                    } else {
                        None
                    };
                    let refresh_sig = if (refresh_flags & 0x08) != 0 {
                        Some(reader.read_string()?)
                    } else {
                        None
                    };
                    Some(BoolOr::Options(JsReactRefreshConfig {
                        refresh_reg,
                        refresh_sig,
                        emit_full_signatures,
                    }))
                }
                _ => unreachable!(),
            };
            Some(BoolOr::Options(JsxConfig {
                runtime,
                development,
                source_self,
                import_source,
                pragma,
                pragma_frag,
                throw_if_namespace,
                pure,
                display_name,
                refresh,
            }))
        }
        _ => unreachable!(),
    };

    let decorators = match (flags >> 7) & 3 {
        0 => None,
        1 => Some(BoolOr::Bool(false)),
        2 => Some(BoolOr::Bool(true)),
        3 => {
            let dec_flags = u32::from(reader.read_u8()?);
            if dec_flags & !0x0f != 0 {
                return None;
            }
            let version = match dec_flags & 3 {
                0 => None,
                1 => Some(JsDecoratorVersion::V2023_11),
                2 => Some(JsDecoratorVersion::Legacy),
                _ => return None,
            };
            let metadata = decode_opt_bool(dec_flags >> 2)?;
            Some(BoolOr::Options(JsDecoratorConfig { version, metadata }))
        }
        _ => unreachable!(),
    };

    let module = match (flags >> 9) & 7 {
        0 => None,
        1 => Some(JsModuleConfig { r#type: None }),
        2 => Some(JsModuleConfig {
            r#type: Some(JsModuleType::Preserve),
        }),
        3 => Some(JsModuleConfig {
            r#type: Some(JsModuleType::CommonJs),
        }),
        4 => Some(JsModuleConfig {
            r#type: Some(JsModuleType::Amd),
        }),
        5 => Some(JsModuleConfig {
            r#type: Some(JsModuleType::Umd),
        }),
        6 => Some(JsModuleConfig {
            r#type: Some(JsModuleType::SystemJs),
        }),
        _ => return None,
    };

    let features = if (flags & 0x1000) != 0 {
        let feature_flags = u32::from(reader.read_u8()?);
        if feature_flags & !0x0f != 0 {
            return None;
        }
        let mut map = BTreeMap::new();
        if let Some(enabled) = decode_opt_bool(feature_flags)? {
            map.insert(JsFeature::ClassStaticBlock, enabled);
        }
        if let Some(enabled) = decode_opt_bool(feature_flags >> 2)? {
            map.insert(JsFeature::OptionalCatchBinding, enabled);
        }
        Some(map)
    } else {
        None
    };

    let assumptions = if (flags & 0x2000) != 0 {
        let a = u32::from(reader.read_u16()?);
        Some(JsAssumptions {
            constant_super: decode_opt_bool(a)?,
            ignore_function_length: decode_opt_bool(a >> 2)?,
            no_document_all: decode_opt_bool(a >> 4)?,
            object_rest_no_symbols: decode_opt_bool(a >> 6)?,
            private_fields_as_properties: decode_opt_bool(a >> 8)?,
            pure_getters: decode_opt_bool(a >> 10)?,
            set_public_class_fields: decode_opt_bool(a >> 12)?,
            set_spread_properties: decode_opt_bool(a >> 14)?,
        })
    } else {
        None
    };

    Some(JsTransformerConfig {
        filename,
        target,
        typescript,
        jsx,
        decorators,
        module,
        features,
        assumptions,
    })
}

/// Decode the packed transform configuration without copying the input buffer.
pub fn decode_options(bytes: &[u8]) -> Option<Option<TransformApiOptions>> {
    if bytes.is_empty() {
        return Some(None);
    }
    let header = bytes.get(..16)?;
    let word =
        |index: usize| u32::from_le_bytes(header[index * 4..index * 4 + 4].try_into().unwrap());
    if word(0) != 1 || word(3) & !0x3f != 0 {
        return None;
    }
    let parser = <Option<JsParserConfig> as BridgeDecode>::decode(word(1))?;
    let generator = <Option<JsGeneratorConfig> as BridgeDecode>::decode(word(2))?;
    let source_map = decode_opt_bool(word(3))?;
    let comments = match (word(3) >> 2) & 3 {
        0 => None,
        1 => Some(CommentsOption::All),
        2 => Some(CommentsOption::None),
        _ => return None,
    };
    let mut reader = Reader::new(bytes, 16);
    let filename = if (word(3) & 0x10) != 0 {
        Some(reader.read_string()?)
    } else {
        None
    };
    let transformer = if (word(3) & 0x20) != 0 {
        Some(decode_transformer(&mut reader)?)
    } else {
        None
    };
    if reader.offset != bytes.len() {
        return None;
    }
    Some(Some(TransformApiOptions {
        filename,
        source_map,
        comments,
        parser,
        transformer,
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
    options: &[u8],
) -> Result<Vec<u8>, String> {
    let options = decode_options(options).ok_or_else(|| "invalid bridge value for `options`".to_owned())?;
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let output = transform_js::<true>(source, options.unwrap_or_default().into());
    let Ok(buffer) = {
        output_to_buffer(source, source_utf16_len, output, |size| {
            Ok::<_, std::convert::Infallible>(vec![0; size])
        })
    };
    Ok(buffer)
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
    let output = transform_js::<true>(source, options.unwrap_or_default().into());
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
