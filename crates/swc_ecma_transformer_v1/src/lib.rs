//! SWC JavaScript transform passes ported to the SWC Next AST.
//!
//! Options follow SWC Next's per-edition and language grouping, with the fields
//! and defaults of legacy SWC's transform configurations. [`Transformer::transform`]
//! accepts a mutable AST without a semantic model. JSX, TypeScript, ES2016–ES2022,
//! RegExp, and module lowering are available. The source inventory is in
//! `docs/transform-inventory.md`.
//!
//! Configuring one edition does not enable another edition or RegExp lowering.
//! Switches default to `false`; optional transforms default to `None` (disabled).
//! Use `Some(Default::default())` to enable a configurable transform with SWC's
//! defaults. Per-transform SWC flags remain local to their transform options.
//!
//! ```
//! use swc_next_ecma_transformer_v1::{
//!     TransformerOptions,
//!     env::EnvOptions,
//!     es2016::ES2016Options,
//!     es2017::{AsyncToGeneratorOptions, ES2017Options},
//!     es2018::{ES2018Options, ObjectRestSpreadOptions},
//!     es2019::ES2019Options,
//!     es2020::{ES2020Options, NullishCoalescingOptions, OptionalChainingOptions},
//!     es2021::ES2021Options,
//!     es2022::{ClassPropertiesOptions, ES2022Options},
//!     jsx::{JsxOptions, JsxRuntime, ReactRefreshOptions},
//!     modules::{ModuleType, ModulesOptions},
//!     regexp::RegExpOptions,
//!     typescript::{TsImportExportAssignConfig, TypeScriptOptions},
//! };
//!
//! let options = TransformerOptions {
//!     filename: Some("input.tsx".into()),
//!     env: EnvOptions {
//!         es2016: ES2016Options { exponentiation: true },
//!         es2017: ES2017Options {
//!             async_to_generator: Some(AsyncToGeneratorOptions::default()),
//!             ..Default::default()
//!         },
//!         es2018: ES2018Options {
//!             object_rest_spread: Some(ObjectRestSpreadOptions::default()),
//!         },
//!         es2019: ES2019Options { optional_catch_binding: true },
//!         es2020: ES2020Options {
//!             export_namespace_from: true,
//!             nullish_coalescing: Some(NullishCoalescingOptions::default()),
//!             optional_chaining: Some(OptionalChainingOptions::default()),
//!         },
//!         es2021: ES2021Options { logical_assignments: true },
//!         es2022: ES2022Options {
//!             class_fields_use_set: None,
//!             static_blocks: true,
//!             class_properties: Some(ClassPropertiesOptions::default()),
//!             private_in_object: true,
//!         },
//!         regexp: RegExpOptions { dot_all_regex: true, ..Default::default() },
//!         ..Default::default()
//!     },
//!     jsx: Some(JsxOptions {
//!         runtime: Some(JsxRuntime::Automatic),
//!         development: Some(true),
//!         refresh: Some(ReactRefreshOptions::default()),
//!         ..Default::default()
//!     }),
//!     modules: ModulesOptions { r#type: ModuleType::CommonJs },
//!     typescript: Some(TypeScriptOptions {
//!         import_export_assign_config: TsImportExportAssignConfig::EsNext,
//!         ..Default::default()
//!     }),
//!     ..Default::default()
//! };
//! ```

use swc_next_ecma_ast::{AstCtx, Diagnostic, TraverseMutCtx};

use self::{
    common::{TraverseCtx, resolver::context::Resolver},
    modules::ModulesOptions,
};

pub mod decorators;
pub mod env;
pub mod es2016;
pub mod es2017;
pub mod es2018;
pub mod es2019;
pub mod es2020;
pub mod es2021;
pub mod es2022;
pub mod jsx;
pub mod modules;
pub mod regexp;
pub mod typescript;

mod common;

/// A boolean shorthand or explicit configuration, as in SWC Next's public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOr<T> {
    Bool(bool),
    Options(T),
}

/// Selects ported SWC transforms using SWC Next's option grouping.
///
/// All transforms are disabled by default. Edition groups select individual
/// passes. Use [`env::EnvConfig`] to select transforms for a target.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TransformerOptions {
    /// Source filename for transforms such as JSX development metadata.
    pub filename: Option<String>,
    /// Decorator transform options.
    pub decorators: Option<decorators::DecoratorOptions>,
    /// Resolved ECMAScript and RegExp transform selections.
    pub env: env::EnvOptions,
    /// SWC's React preset options, including JSX and React Refresh.
    pub jsx: Option<jsx::JsxOptions>,
    /// TypeScript stripping/lowering and JSX import-retention options.
    pub typescript: Option<typescript::TypeScriptOptions>,
    /// Module output format. Defaults to preserving ECMAScript modules.
    pub modules: ModulesOptions,
}

/// Diagnostics produced while transforming an AST in place.
#[derive(Debug, Default)]
pub struct TransformerReturn {
    pub diagnostics: Vec<Diagnostic>,
}

/// Entry point for ported SWC transforms on the SWC Next AST.
pub struct Transformer;

impl Transformer {
    /// Transforms a program in place without requiring a semantic model.
    ///
    /// Call after parsing. The caller does not need to normalize or analyze the
    /// AST. Analysis needed by the selected passes belongs inside this pipeline.
    /// Previously computed semantic indexes must be recomputed after mutation.
    /// Name resolution runs before the selected passes; hygiene runs once after
    /// all passes have finished.
    ///
    /// ```
    /// use swc_next_allocator::Allocator;
    /// use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    /// use swc_next_ecma_transformer_v1::{Transformer, TransformerOptions};
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
    /// Supports the selected JSX, TypeScript, ES2016–ES2022, RegExp, and module passes.
    /// Runtime helpers are loaded from `@swc/helpers`.
    /// React Refresh reports a diagnostic because it is not implemented.
    ///
    /// # Panics
    ///
    /// Panics if the AST root is not a program.
    ///
    /// SWC pipeline ordering: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc/src/config/mod.rs#L805>.
    pub fn transform(ast: &mut AstCtx<'_>, options: TransformerOptions) -> TransformerReturn {
        // SAFETY: The program and all handles used by these passes belong to
        // this context. Passes only construct well-typed nodes in the same AST.
        let ast = unsafe { ast.ast_mut() };
        let mut diagnostics = Vec::new();
        let mut resolver = Resolver::new(ast);
        let mut context = TraverseMutCtx::new(ast, TraverseCtx::new(&mut resolver));
        if let Some(decorators) = options.decorators {
            decorators::transform(context.ast, &mut context.state, decorators);
        }
        if let Some(typescript) = options.typescript {
            diagnostics.extend(typescript::typescript(
                context.ast,
                typescript,
                options.jsx.is_some(),
                options.modules.r#type,
                context.state.resolver,
            ));
        }
        if let Some(jsx) = &options.jsx {
            diagnostics.extend(jsx::transform(
                context.ast,
                jsx,
                options.filename.as_deref(),
                context.state.resolver,
            ));
        }

        diagnostics.extend(es2022::transform(&mut context, options.env.es2022));
        es2021::transform(&mut context, options.env.es2021);
        es2020::transform(&mut context, options.env.es2020);
        es2019::transform(context.ast, context.state.resolver, options.env.es2019);
        es2018::transform(&mut context, options.env.es2018);
        es2017::transform(&mut context, options.env.es2017);
        es2016::transform(&mut context, options.env.es2016);
        regexp::transform(context.ast, context.state.resolver, options.env.regexp);
        modules::transform(
            context.ast,
            &mut context.state,
            options.modules.r#type,
            options.filename.as_deref(),
            options.env.module_features,
        );

        // SWC runs hygiene once, after all transform passes, before its fixer.
        resolver.hygiene(ast);
        TransformerReturn { diagnostics }
    }
}
