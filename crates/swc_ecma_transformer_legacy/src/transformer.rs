use swc_next_ecma_ast::{AstCtx, Diagnostic, Span};

use crate::{TransformerOptions, jsx::JsxRuntime};

/// Diagnostics produced while transforming an AST in place.
#[derive(Debug, Default)]
pub struct TransformerReturn {
    pub diagnostics: Vec<Diagnostic>,
}

/// Entry point for legacy SWC transforms on the SWC Next AST.
pub struct Transformer;

impl Transformer {
    /// Transforms a program in place without requiring a semantic model.
    ///
    /// Call after parsing. The caller does not need to normalize or analyze the
    /// AST. Any analysis needed by future ports belongs inside this pipeline.
    /// Previously computed semantic indexes must be recomputed after mutation.
    ///
    /// ```
    /// use swc_next_allocator::Allocator;
    /// use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    /// use swc_next_ecma_transformer_legacy::{Transformer, TransformerOptions};
    ///
    /// let allocator = Allocator::new();
    /// let parsed = Parser::init(
    ///     &allocator, "const value = 1;", Options::default(), NoTokenParserConfig,
    /// ).parse();
    /// assert!(parsed.diagnostics.is_empty());
    /// let mut ast = parsed.ast;
    /// let result = Transformer::transform(&mut ast, TransformerOptions::default());
    /// assert!(result.diagnostics.is_empty());
    /// ```
    ///
    /// No transforms have been ported yet: each selected transform produces an
    /// error diagnostic, and the AST is left unchanged. Default options succeed
    /// without changing the AST. This keeps missing ports visible in conformance
    /// results rather than silently treating enabled transforms as successful.
    ///
    /// # Panics
    ///
    /// Panics if the AST root is not a program.
    pub fn transform(ast: &mut AstCtx<'_>, options: TransformerOptions) -> TransformerReturn {
        let _program = ast.root_program();
        let selected = [
            (
                options
                    .jsx
                    .as_ref()
                    .is_some_and(|jsx| jsx.runtime != Some(JsxRuntime::Preserve)),
                "jsx",
            ),
            (options.typescript.is_some(), "typescript"),
            (options.env.es2022.static_blocks, "static_blocks"),
            (
                options.env.es2022.class_properties.is_some(),
                "class_properties",
            ),
            (options.env.es2022.private_in_object, "private_in_object"),
            (
                options.env.es2021.logical_assignments,
                "logical_assignments",
            ),
            (
                options.env.es2020.export_namespace_from,
                "export_namespace_from",
            ),
            (
                options.env.es2020.nullish_coalescing.is_some(),
                "nullish_coalescing",
            ),
            (
                options.env.es2020.optional_chaining.is_some(),
                "optional_chaining",
            ),
            (
                options.env.es2019.optional_catch_binding,
                "optional_catch_binding",
            ),
            (
                options.env.es2018.object_rest_spread.is_some(),
                "object_rest_spread",
            ),
            (
                options.env.es2017.async_to_generator.is_some(),
                "async_to_generator",
            ),
            (options.env.es2016.exponentiation, "exponentiation"),
            (options.env.regexp.dot_all_regex, "dot_all_regex"),
            (options.env.regexp.has_indices, "has_indices"),
            (
                options.env.regexp.lookbehind_assertion,
                "lookbehind_assertion",
            ),
            (
                options.env.regexp.named_capturing_groups_regex,
                "named_capturing_groups_regex",
            ),
            (
                options.env.regexp.duplicate_named_capturing_groups_regex,
                "duplicate_named_capturing_groups_regex",
            ),
            (options.env.regexp.sticky_regex, "sticky_regex"),
            (
                options.env.regexp.unicode_property_regex,
                "unicode_property_regex",
            ),
            (options.env.regexp.unicode_regex, "unicode_regex"),
            (options.env.regexp.unicode_sets_regex, "unicode_sets_regex"),
        ];
        let diagnostics = selected
            .into_iter()
            .filter(|(enabled, _)| *enabled)
            .map(|(_, name)| {
                let mut diagnostic =
                    Diagnostic::error("Legacy transform has not been ported yet", Span::new(0, 0));
                diagnostic.help = Some(name.into());
                diagnostic
            })
            .collect();
        TransformerReturn { diagnostics }
    }
}
