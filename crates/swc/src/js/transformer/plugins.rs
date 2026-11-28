//! JavaScript adapters for the v1 transform hooks.

use swc_next_allocator::{Allocator, vec::ArenaVec};
use swc_next_ecma_ast::{AstCtx, ffi::ast};

use super::{
    bridge,
    v1::{self, Hooks, Stage},
};
use crate::js::{CommentsOption, JsOutput};

const AFTER_PARSE: u32 = 1;
const BEFORE_CODEGEN: u32 = 2;

struct PluginHooks<F, M> {
    stages: u32,
    source_utf16_len: usize,
    output_comments: CommentsOption,
    callback: F,
    make_error: M,
}

impl<E, F: FnMut(u32, Vec<u8>) -> Result<Vec<u8>, E>, M: Fn(String) -> E> Hooks
    for PluginHooks<F, M>
{
    type Error = E;

    fn run_js_plugins<'a>(
        &mut self,
        stage: Stage,
        ast: &mut AstCtx<'a>,
        allocator: &'a Allocator,
    ) -> Result<bool, E> {
        let stage = match stage {
            Stage::AfterParse => AFTER_PARSE,
            Stage::BeforeCodegen => BEFORE_CODEGEN,
        };
        if stage == AFTER_PARSE
            && self.output_comments == CommentsOption::All
            && !ast.comments().is_empty()
        {
            return Err((self.make_error)("Comment preservation across JavaScript plugins is unsupported; set comments: \"none\"".to_owned()));
        }
        if self.stages & stage == 0 {
            return Ok(false);
        }

        let mut input = vec![0; ast::buffer_size(ast, &[])];
        ast::serialize_into(ast, &[], &mut input, self.source_utf16_len);
        let edited = (self.callback)(stage, input)?;
        let mut aligned = vec![0_u64; edited.len().div_ceil(8)];
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(aligned.as_mut_ptr().cast::<u8>(), edited.len())
        };
        bytes.copy_from_slice(&edited);
        let temporary = Allocator::default();
        let edited = unsafe { ast::deserialize_from(bytes, &temporary) }
            .map_err(|error| (self.make_error)(error.to_string()))?;
        let mut replacement = edited.to_owned_with_source(ast.source(), allocator);
        let mut comments = ArenaVec::with_capacity_in(ast.comments().len(), allocator);
        comments.extend_from_slice_copy(ast.comments());
        unsafe { replacement.ast_mut() }.set_attached_comments(comments);
        *ast = replacement;
        Ok(true)
    }
}

/// Run the existing v1 pipeline with synchronous JavaScript AST hooks.
///
/// # Safety
/// The source must be valid UTF-8 with the supplied UTF-16 length. The callback
/// must return a valid AST transfer encoded with that same original source.
unsafe fn run_v1_pipeline<E>(
    source: &[u8],
    source_utf16_len: u32,
    config: &[u8],
    stages: u32,
    callback: impl FnMut(u32, Vec<u8>) -> Result<Vec<u8>, E>,
    make_error: impl Fn(String) -> E,
) -> Result<JsOutput, E> {
    let options = bridge::decode_options(config)
        .ok_or_else(|| make_error("invalid transform configuration".to_owned()))?
        .unwrap_or_default();
    if options.source_map.unwrap_or(false) {
        return Err(make_error(
            "sourceMap is unsupported with JavaScript plugins".to_owned(),
        ));
    }
    let output_comments = options.comments.unwrap_or(CommentsOption::All);
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    v1::transform_with_hooks::<true, _>(
        source,
        options.into(),
        &mut PluginHooks {
            stages,
            source_utf16_len: source_utf16_len as usize,
            output_comments,
            callback,
            make_error,
        },
    )
}

#[cfg(not(target_family = "wasm"))]
pub use native::{NativePluginCallback, NativePluginOutput, transform_to_node_output};

#[cfg(not(target_family = "wasm"))]
mod native {
    use swc_next_bridge::{
        NativeAbi,
        napi::{
            self,
            bindgen_prelude::{Buffer, Either, FnArgs, Function, Uint8Array},
        },
    };

    pub type NativePluginCallback<'a> = Function<'a, FnArgs<(u32, Buffer)>, Uint8Array>;
    pub type NativePluginOutput = Either<<String as NativeAbi>::Abi, Buffer>;

    /// # Safety
    /// Source must be valid UTF-8 with the specified UTF-16 length. The callback
    /// must return a matching AST transfer with the original source.
    pub unsafe fn transform_to_node_output(
        env: &napi::Env,
        source: &[u8],
        source_utf16_len: u32,
        config: &[u8],
        stages: u32,
        callback: NativePluginCallback<'_>,
    ) -> napi::Result<NativePluginOutput> {
        let output = unsafe {
            super::run_v1_pipeline(
                source,
                source_utf16_len,
                config,
                stages,
                |stage, bytes| {
                    callback
                        .call((stage, bytes.into()).into())
                        .map(|bytes| bytes.to_vec())
                },
                napi::Error::from_reason,
            )
        }?;
        if output.diagnostics.is_empty() {
            return Ok(Either::A(<String as NativeAbi>::into_native(output.code)));
        }
        let source = unsafe { std::str::from_utf8_unchecked(source) };
        super::bridge::output_to_buffer(source, source_utf16_len, output, |size| {
            swc_next_bridge::create_buffer(env, size)
        })
        .map(Either::B)
    }
}

#[cfg(target_family = "wasm")]
/// # Safety
/// Source must be valid UTF-8 with the specified UTF-16 length. The host
/// callback must return a matching AST transfer with the original source.
pub unsafe fn transform_to_wasm_buffer(
    source: &[u8],
    source_utf16_len: u32,
    config: &[u8],
    stages: u32,
) -> Result<Vec<u8>, String> {
    use swc_next_bridge::wasm::{FromWasmAbi, free};

    #[link(wasm_import_module = "swc")]
    unsafe extern "C" {
        fn run_js_plugins(stage: u32, input: *const u8, length: usize) -> u32;
    }

    let output = unsafe {
        run_v1_pipeline(
            source,
            source_utf16_len,
            config,
            stages,
            |stage, bytes| {
                let pointer = run_js_plugins(stage, bytes.as_ptr(), bytes.len());
                let edited = Vec::<u8>::from_wasm_abi(pointer);
                free(pointer);
                edited.ok_or_else(|| "JavaScript plugin stopped the transform".to_owned())
            },
            |message| message,
        )
    }?;
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    bridge::output_to_buffer(source, source_utf16_len, output, |size| Ok(vec![0; size]))
}
