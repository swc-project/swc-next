use swc_next_ecma_ast::{
    AstCtx, Diagnostic, Program, Span, TraverseMut, TraverseMutCtx, TraverseMutWith,
};

use crate::{
    TransformerOptions,
    common::{TraverseCtx, resolver::context::Resolver, var_declarations::VarDeclarations},
    jsx, typescript,
};

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
    /// AST. Analysis needed by the selected passes belongs inside this pipeline.
    /// Previously computed semantic indexes must be recomputed after mutation.
    /// Name resolution runs before the selected passes; hygiene runs once after
    /// all passes have finished.
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
    /// Supports the selected JSX, TypeScript, ES2016–ES2022 and RegExp passes.
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
        if let Some(typescript) = &options.typescript {
            diagnostics.extend(typescript::typescript(
                ast,
                typescript,
                options.jsx.is_some(),
                &mut resolver,
            ));
        }
        if let Some(jsx) = &options.jsx {
            diagnostics.extend(jsx::jsx(
                ast,
                jsx,
                options.filename.as_deref(),
                &mut resolver,
            ));
            if jsx.runtime != Some(jsx::JsxRuntime::Preserve)
                && jsx.development == Some(true)
                && jsx.refresh.is_some()
            {
                let mut diagnostic =
                    Diagnostic::error("Legacy transform has not been ported yet", Span::new(0, 0));
                diagnostic.help = Some("react_refresh".into());
                diagnostics.push(diagnostic);
            }
        }
        let mut context = TraverseMutCtx::new(ast, TraverseCtx::new(&mut resolver));
        let program = context.ast.root_program();
        if let Some(options) = options.env.es2022.class_fields_use_set {
            crate::es2022::class_fields_use_set::class_fields_use_set(
                context.ast,
                context.state.resolver,
                options,
            );
        }
        if options.env.es2022.static_blocks {
            let _ = program.traverse_mut_with(
                &mut crate::es2022::class_static_block::ClassStaticBlock,
                &mut context,
            );
        }
        if let Some(options) = options.env.es2022.class_properties {
            diagnostics.extend(crate::es2022::class_properties::class_properties(
                context.ast,
                &mut context.state,
                options,
            ));
        }
        if options.env.es2022.private_in_object {
            let _ = program.traverse_mut_with(
                &mut VarDeclarations {
                    hook:
                        crate::es2022::private_property_in_object::PrivatePropertyInObjectPass::new(
                        ),
                    scope_arrow_exprs: false,
                },
                &mut context,
            );
        }
        if options.env.es2021.logical_assignments {
            hook_pass_with_var_declarations(
                program,
                crate::es2021::logical_assignment_operators::LogicalAssignmentOperatorsPass,
                &mut context,
            );
        }
        if let Some(options) = options.env.es2020.optional_chaining {
            crate::es2022::optional_chaining_impl::optional_chaining_impl(
                context.ast,
                context.state.resolver,
                options,
            );
        }
        if options.env.es2020.export_namespace_from {
            crate::es2020::export_namespace_from::pass(context.ast, context.state.resolver);
        }
        if let Some(options) = options.env.es2020.nullish_coalescing {
            hook_pass_with_var_declarations(
                program,
                crate::es2020::nullish_coalescing::NullishCoalescingPass(options),
                &mut context,
            );
        }
        if options.env.es2019.optional_catch_binding {
            crate::es2019::optional_catch_binding::pass(context.ast, context.state.resolver);
        }
        if let Some(options) = options.env.es2018.object_rest_spread {
            hook_pass_with_var_declarations(
                program,
                crate::es2018::object_rest_spread::ObjectRestSpreadPass::new(options),
                &mut context,
            );
        }
        let es2017 = options.env.es2017;
        if let Some(options) = es2017
            .async_to_generator
            .or(es2017.async_generator_functions)
        {
            let _ = program.traverse_mut_with(
                &mut crate::es2017::async_to_generator::AsyncToGeneratorPass::new(
                    es2017.async_to_generator.is_some(),
                    options,
                ),
                &mut context,
            );
        }
        if options.env.es2016.exponentiation {
            hook_pass_with_var_declarations(
                program,
                crate::es2016::exponentiation_operator::ExponentiationOperatorPass,
                &mut context,
            );
        }
        let _ = program.traverse_mut_with(
            &mut crate::regexp::RegexpPass(options.env.regexp),
            &mut context,
        );
        context
            .state
            .helpers
            .inject_helpers(context.ast, context.state.resolver);
        context.state.resolver.register(context.ast);
        // SWC runs hygiene once, after all transform passes, before its fixer.
        resolver.hygiene(ast);
        TransformerReturn { diagnostics }
    }
}

/// SWC composes the lowering hook with VarDeclarations in a single traversal.
/// <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/lib.rs#L90>.
fn hook_pass_with_var_declarations<'ast, 'r>(
    program: Program,
    hook: impl TraverseMut<'ast, TraverseCtx<'r>>,
    ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
) {
    let _ = program.traverse_mut_with(
        &mut VarDeclarations {
            hook,
            scope_arrow_exprs: true,
        },
        ctx,
    );
}
