mod options;
mod options_wire;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{DeserializeError, deserialize_from};
use swc_next_ecma_codegen::print;

use crate::{BridgeEncode, bridge};

pub use self::options::CodegenOptions;

#[bridge(object)]
pub struct CodegenOutput {
    pub code: String,
}

#[bridge]
impl BridgeEncode for CodegenOutput {
    type Wire = String;

    fn encode(self) -> Self::Wire {
        self.code
    }
}

/// Print a trusted AST transfer buffer.
///
/// # Safety
///
/// `buffer` must be produced by `@swc-next/encoder` matching the version of this
/// crate and follow the [AST transfer protocol](https://github.com/swc-project/swc-next/blob/main/crates/swc_ecma_ast/src/ffi/ast.rs).
/// It must satisfy the alignment requirements of [`deserialize_from`] and remain
/// immutable throughout this call.
pub unsafe fn print_from_buffer(
    buffer: &[u8],
    options: Option<CodegenOptions>,
) -> Result<CodegenOutput, DeserializeError> {
    let allocator = Allocator::default();
    // SAFETY: The caller guarantees a valid, immutable AST transfer buffer.
    let tree = unsafe { deserialize_from(buffer, &allocator) }?;
    let result = print(&tree, options.unwrap_or_default().into());
    Ok(CodegenOutput { code: result.code })
}

/// Print separate, trusted JavaScript encoder sections.
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
pub unsafe fn print_from_parts(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: Option<CodegenOptions>,
) -> Result<CodegenOutput, DeserializeError> {
    #[cfg(all(not(target_family = "wasm"), target_endian = "little"))]
    {
        let allocator = Allocator::default();
        // SAFETY: The caller guarantees the section layout, alignment, and immutability.
        let tree = unsafe {
            swc_next_ecma_ast::ffi::ast::deserialize_parts(
                header, spans, kinds, payloads, extra, strings, &allocator,
            )
        }?;
        let result = print(&tree, options.unwrap_or_default().into());
        Ok(CodegenOutput { code: result.code })
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
        unsafe { print_from_buffer(buffer, options) }
    }
}
