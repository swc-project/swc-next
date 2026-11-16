use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Diagnostic, Severity};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_minifier::{CompressOptions, Minifier, MinifierOptions};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};

use super::{CommentsOption, JsMinifyApiOptions, JsOutput, serialize_source_map};

use crate::BoolOr;

/// JavaScript minification options.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsMinifierConfig {
    /// `false` disables compression. `true` uses default options.
    ///
    /// @default true
    pub compress: Option<BoolOr<JsCompressConfig>>,
    /// `false` disables mangling. `true` uses default options.
    ///
    /// @default true
    pub mangle: Option<bool>,
    // Extend the boolean shorthand when mangler options are supported.
    // pub mangle: Option<BoolOr<JsMangleOptions>>,
}

impl From<JsMinifierConfig> for MinifierOptions {
    fn from(options: JsMinifierConfig) -> Self {
        let defaults = Self::default();
        Self {
            compress: match options.compress {
                None => defaults.compress,
                Some(BoolOr::Bool(false)) => None,
                Some(BoolOr::Bool(true)) => Some(CompressOptions::default()),
                Some(BoolOr::Options(options)) => Some(options.into()),
            },
            mangle: options.mangle.unwrap_or(defaults.mangle),
        }
    }
}

/// Compression options. All default to true.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsCompressConfig {
    /// Remove empty statements and unnecessary blocks.
    pub blocks: Option<bool>,
    /// Shorten boolean literals.
    pub booleans: Option<bool>,
    /// Shorten comparisons whose operand types permit it.
    pub comparisons: Option<bool>,
    /// Shorten conditional statements and expressions.
    pub conditionals: Option<bool>,
    /// Remove code that a constant condition never runs.
    pub dead_code: Option<bool>,
    /// Fold constant expressions when the result is shorter.
    pub evaluate: Option<bool>,
    /// Shorten unshadowed global constants.
    pub global_defs: Option<bool>,
    /// Shorten conditional returns and remove redundant trailing returns.
    pub if_return: Option<bool>,
    /// Join adjacent declarations of the same kind.
    pub join_vars: Option<bool>,
    /// Negate immediately invoked function expressions in statement position.
    pub negate_iife: Option<bool>,
    /// Inline safe local constants when the replacement is shorter.
    pub reduce_vars: Option<bool>,
    /// Join expression statements with comma operators.
    pub sequences: Option<bool>,
    /// Remove unused computations that have no side effects.
    pub side_effects: Option<bool>,
    /// Shorten undefined `typeof` comparisons for local `var` bindings.
    pub typeofs: Option<bool>,
    /// Remove unused variables whose initializers have no side effects.
    pub unused: Option<bool>,
}

impl From<JsCompressConfig> for CompressOptions {
    fn from(options: JsCompressConfig) -> Self {
        let defaults = Self::default();
        Self {
            blocks: options.blocks.unwrap_or(defaults.blocks),
            booleans: options.booleans.unwrap_or(defaults.booleans),
            comparisons: options.comparisons.unwrap_or(defaults.comparisons),
            conditionals: options.conditionals.unwrap_or(defaults.conditionals),
            dead_code: options.dead_code.unwrap_or(defaults.dead_code),
            evaluate: options.evaluate.unwrap_or(defaults.evaluate),
            global_defs: options.global_defs.unwrap_or(defaults.global_defs),
            if_return: options.if_return.unwrap_or(defaults.if_return),
            join_vars: options.join_vars.unwrap_or(defaults.join_vars),
            negate_iife: options.negate_iife.unwrap_or(defaults.negate_iife),
            reduce_vars: options.reduce_vars.unwrap_or(defaults.reduce_vars),
            sequences: options.sequences.unwrap_or(defaults.sequences),
            side_effects: options.side_effects.unwrap_or(defaults.side_effects),
            typeofs: options.typeofs.unwrap_or(defaults.typeofs),
            unused: options.unused.unwrap_or(defaults.unused),
        }
    }
}

// pub struct JsMangleOptions {
//     pub top_level: Option<bool>,
//     pub keep_class_names: Option<bool>,
//     pub keep_fn_names: Option<bool>,
// }

/// Minify JavaScript source without lowering TypeScript or JSX syntax.
///
/// Compression and mangling use the minifier configuration. Compact printing
/// is controlled independently by `generator.minify` and defaults to false.
pub fn minify_js(source: &str, options: JsMinifyApiOptions) -> JsOutput {
    let mut output = JsOutput::default();
    fn has_errors(diagnostics: &[Diagnostic]) -> bool {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    // ======================= Parse =======================
    let allocator = Allocator::default();
    let mut ast = {
        let comments = options.comments.unwrap_or(CommentsOption::None);
        let parser = ParserOptions {
            comments: comments.into(),
            ..ParserOptions::default()
        };
        let parsed = Parser::init(&allocator, source, parser, NoTokenParserConfig).parse();
        output.diagnostics.extend(parsed.diagnostics);
        if has_errors(&output.diagnostics) {
            return output;
        }
        parsed.ast
    };

    // ======================= Analyze =======================
    {
        let diagnostics = check_syntax(&ast, SyntaxChecks::all());
        output.diagnostics.extend(diagnostics);
        if has_errors(&output.diagnostics) {
            return output;
        }
    }

    // ======================= Minify =======================
    {
        let minifier = options.minifier.unwrap_or_default().into();
        Minifier::minify(&mut ast, minifier);
    }

    // ======================= Generate =======================
    {
        let mut generator = GeneratorOptions::from(options.generator.unwrap_or_default());
        let source_map = options.source_map.unwrap_or(false);
        let filename = options.filename.as_deref().unwrap_or("input.js");
        generator.source_map_path = source_map.then(|| filename.into());
        generator.comments =
            options.comments.unwrap_or(CommentsOption::None) == CommentsOption::All;
        let generated = generate(&ast, generator);
        output.code = generated.code;
        output.map = generated.map.as_ref().map(serialize_source_map);
    }

    output
}
