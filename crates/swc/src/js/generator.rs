use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi;
use swc_next_ecma_codegen::{Options, generate};

#[cfg(feature = "bridge")]
use swc_next_bridge::{BridgeDecodeOptional, BridgeEncode, bridge};

#[cfg(all(feature = "bridge", not(target_family = "wasm")))]
use swc_next_bridge::napi;

pub use ffi::DeserializeError;

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

/// Generate from a base buffer and its optional edit extension.
/// Both byte buffers are validated before native AST fields are accessed.
pub fn generate_from_buffers(
    base: &[u8],
    extension: &[u8],
    options: Option<JsGenerateApiOptions>,
) -> Result<GenerateOutput, ffi::DeserializeError> {
    let allocator = Allocator::default();
    let tree = ffi::deserialize(base, extension, &allocator)?;
    let result = generate(&tree, options.unwrap_or_default().into());
    Ok(GenerateOutput { code: result.code })
}

/// Generate from AST tables produced by the matching parser and incremental encoder.
///
/// # Safety
/// Both buffers must obey [`ffi::deserialize_unchecked`]'s validity and immutability
/// contract, including any edits made by the matching incremental encoder. They
/// must remain unchanged throughout this call.
pub unsafe fn generate_from_buffers_unchecked(
    base: &[u8],
    extension: &[u8],
    options: Option<JsGenerateApiOptions>,
) -> Result<GenerateOutput, ffi::DeserializeError> {
    let allocator = Allocator::default();
    let tree = unsafe { ffi::deserialize_unchecked(base, extension, &allocator) }?;
    let result = generate(&tree, options.unwrap_or_default().into());
    Ok(GenerateOutput { code: result.code })
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

    // Internal option word paired with packages/protocol/generator.js. Its layout
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
