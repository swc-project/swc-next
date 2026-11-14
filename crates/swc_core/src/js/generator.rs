use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast;
use swc_next_ecma_codegen::{Options, generate};

#[cfg(feature = "bridge")]
use swc_next_bridge::{BridgeDecodeOptional, BridgeEncode, bridge};

#[cfg(all(feature = "bridge", not(target_family = "wasm")))]
use swc_next_bridge::napi;

pub use ast::DeserializeError;

/// JavaScript code generation options.
///
/// Standalone generation prints a caller-supplied AST, which commonly lacks
/// original source text and reliable comment attachments. Comment preservation
/// and source maps are therefore provided by the source-to-source transform and
/// minify APIs instead of this AST-only API.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "GeneratorConfig"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsGeneratorConfig {
    pub ascii_only: Option<bool>,
    /// Escape closing script tags. Tagged template raw text is preserved.
    pub inline_script: Option<bool>,
    /// Print compact code independently of AST compression and mangling.
    pub minify: Option<bool>,
    // pub preamble: Option<String>,
}

/// Options accepted by the standalone generator API.
pub type JsGenerateApiOptions = JsGeneratorConfig;

impl From<JsGeneratorConfig> for Options {
    fn from(options: JsGeneratorConfig) -> Self {
        let defaults = Self::default();
        Self {
            ascii_only: options.ascii_only.unwrap_or(defaults.ascii_only),
            inline_script: options.inline_script.unwrap_or(defaults.inline_script),
            minify: options.minify.unwrap_or(defaults.minify),
            comments: false,
            ..defaults
        }
    }
}

#[cfg_attr(feature = "bridge", bridge(object))]
pub struct GenerateOutput {
    pub code: String,
}

/// Print a trusted AST transfer buffer.
///
/// For internal SWC Next use only. This API and binary format do not follow
/// semantic versioning and may change in any release.
///
/// # Safety
///
/// `buffer` must be produced by `@swc-next/encoder` matching the version of this
/// crate and follow the [AST transfer protocol](https://github.com/swc-project/swc-next/blob/main/crates/swc_ecma_ast/src/ffi/ast.rs).
/// It must satisfy the alignment requirements of [`ast::deserialize_from`] and remain
/// immutable throughout this call.
pub unsafe fn generate_from_buffer(
    buffer: &[u8],
    options: Option<JsGenerateApiOptions>,
) -> Result<GenerateOutput, DeserializeError> {
    let allocator = Allocator::default();
    // SAFETY: The caller guarantees a valid, immutable AST transfer buffer.
    let tree = unsafe { ast::deserialize_from(buffer, &allocator) }?;
    let result = generate(&tree, options.unwrap_or_default().into());
    Ok(GenerateOutput { code: result.code })
}

/// Print separate, trusted JavaScript encoder sections.
///
/// For internal SWC Next use only. This API and binary format do not follow
/// semantic versioning and may change in any release.
///
/// Native little-endian targets borrow the sections. WebAssembly and big-endian
/// targets first join them into an aligned transfer buffer.
///
/// # Safety
///
/// Sections must contain the initialized header, spans, kinds, payloads, extra
/// data, and strings described by the [AST transfer protocol](swc_next_ecma_ast::ffi::ast),
/// without section padding. They must come from the matching JavaScript encoder
/// and remain immutable throughout this call. On native little-endian targets,
/// spans must be aligned to four bytes, payloads and extra data to eight bytes.
pub unsafe fn generate_from_parts(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: Option<JsGenerateApiOptions>,
) -> Result<GenerateOutput, DeserializeError> {
    #[cfg(all(not(target_family = "wasm"), target_endian = "little"))]
    {
        let allocator = Allocator::default();
        // SAFETY: The caller guarantees the section layout, alignment, and immutability.
        let tree = unsafe {
            ast::deserialize_parts(header, spans, kinds, payloads, extra, strings, &allocator)
        }?;
        let result = generate(&tree, options.unwrap_or_default().into());
        Ok(GenerateOutput { code: result.code })
    }
    #[cfg(any(target_family = "wasm", target_endian = "big"))]
    {
        let payload_offset = (header.len() + spans.len() + kinds.len() + 7) & !7;
        let extra_offset = payload_offset + payloads.len();
        let strings_offset = extra_offset + extra.len();
        let length = (strings_offset + strings.len() + 3) & !3;
        // u64 storage preserves the alignment required by the borrowed AST reader.
        let mut words = vec![0_u64; length.div_ceil(8)];
        // SAFETY: Every bit pattern is valid for u64 and the view stays in words.
        let buffer =
            unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr().cast::<u8>(), length) };
        let mut offset = 0;
        for part in [header, spans, kinds] {
            buffer[offset..offset + part.len()].copy_from_slice(part);
            offset += part.len();
        }
        offset = payload_offset;
        for part in [payloads, extra, strings] {
            buffer[offset..offset + part.len()].copy_from_slice(part);
            offset += part.len();
        }
        // SAFETY: The trusted sections now form an aligned transfer buffer.
        unsafe { generate_from_buffer(buffer, options) }
    }
}

#[cfg(feature = "bridge")]
const _: () = {
    use self::JsGeneratorConfig as GeneratorConfig;

    #[bridge]
    impl BridgeEncode for GenerateOutput {
        type Wire = String;

        fn encode(self) -> Self::Wire {
            self.code
        }
    }

    // Internal option word paired with packages/encoder/options.js. Its layout
    // does not follow semver; update both ends together.
    const OPTIONS_PRESENT: u32 = 1 << 0;

    const ASCII_ONLY_SHIFT: u32 = 1;
    const ASCII_ONLY_MASK: u32 = 0b11 << ASCII_ONLY_SHIFT;

    const MINIFY_SHIFT: u32 = 3;
    const MINIFY_MASK: u32 = 0b11 << MINIFY_SHIFT;

    const INLINE_SCRIPT_SHIFT: u32 = 5;
    const INLINE_SCRIPT_MASK: u32 = 0b11 << INLINE_SCRIPT_SHIFT;

    const KNOWN_MASK: u32 = OPTIONS_PRESENT | ASCII_ONLY_MASK | MINIFY_MASK | INLINE_SCRIPT_MASK;

    #[bridge]
    impl BridgeDecodeOptional for GeneratorConfig {
        type Wire = u32;

        fn decode_optional(value: Self::Wire) -> Option<Option<Self>> {
            if value == 0 {
                return Some(None);
            }
            if value & OPTIONS_PRESENT == 0 || value & !KNOWN_MASK != 0 {
                return None;
            }

            let ascii_only = boolean_field(value, ASCII_ONLY_MASK, ASCII_ONLY_SHIFT)?;
            let minify = boolean_field(value, MINIFY_MASK, MINIFY_SHIFT)?;

            let inline_script = boolean_field(value, INLINE_SCRIPT_MASK, INLINE_SCRIPT_SHIFT)?;
            Some(Some(JsGeneratorConfig {
                ascii_only,
                minify,
                inline_script,
            }))
        }
    }

    fn boolean_field(value: u32, mask: u32, shift: u32) -> Option<Option<bool>> {
        match (value & mask) >> shift {
            0 => Some(None),
            1 => Some(Some(false)),
            2 => Some(Some(true)),
            _ => None,
        }
    }
};
