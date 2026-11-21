# Legacy SWC transform inventory

Source: local `swc` checkout at
[`c682b13a8ea7372203970d315df38b529ce5b7f0`](https://github.com/swc-project/swc/tree/c682b13a8ea7372203970d315df38b529ce5b7f0),
inspected on 2026-11-21. Source links in the [symbol tables](visitor-pass-table.md)
are pinned to this revision. This is a source inventory, not a claim of tested
behavioral parity.

## Scope and counting

The tables cover production Rust sources in `swc_ecma_transforms*`,
`swc_ecma_compat_*`, `swc_ecma_transformer`, `swc_ecma_preset_env`,
`swc_ecma_ext_transforms`, and `swc_ecma_utils`, plus the compiler's pass assembly,
minifier adapter, plugin adapter, and dropped-comment preservation in `swc`.
The umbrella `swc_ecma_transforms` crate only reexports other crates.

The [complete symbol tables](visitor-pass-table.md) separate factories from
explicit visitor/pass implementations, including private analysis visitors,
fast-path checks, repeat metadata, and parallel traversal adapters. A function
returning a pass, an implementation of a visitor trait, and a transform feature
are different counts. Compatibility wrappers and hook implementations must not
be counted as independent porting tasks.

Test-only code, generated helper ASTs, proc-macro implementations, and generated
traversal boilerplate are excluded. `swc_ecma_transforms_testing` is test support;
`swc_ecma_transforms_macros`, `swc_ecma_visit`, and `swc_ecma_hooks` are traversal
infrastructure, summarized separately below. Minifier internals, lint rules,
bundler passes, code generation, and external plugin implementations are outside
this JavaScript transpilation inventory; only their compiler integration points
are listed.

## Feature and entry-point map

Paths below are relative to the source checkout's `crates/` directory. `Hook`
means `VisitMutHook`; `VM` means `VisitMut`; `F` means `Fold`. A wrapper entry
delegates to the implementation named in its row. The symbol tables give each
factory and implementation its own source link.

### Base and shared transforms

| Feature / entry | Actual implementation | Role |
| --- | --- | --- |
| `resolver` | `swc_ecma_transforms_base/src/resolver/`: `Resolver`, `Hoister`, `BindingResolver`, `NamespaceResolver` (VM) | Bind identifiers and track TS namespaces before transforms |
| `hygiene`, `hygiene_with_config` | `hygiene/mod.rs`, `rename/`: `RenamePass`, `HygieneRemover` (VM) | Allocate collision-free names and remove contexts |
| `rename`, `rename_with_config`, `renamer`, `renamer_keep_contexts` | `rename/mod.rs`, `rename/ops.rs` (VM) | Shared identifier renaming; includes analysis and eval detection |
| `fixer`, `paren_remover` | `swc_ecma_transforms_base/src/fixer.rs`: `Fixer` (VM) | Normalize AST parentheses and output structure |
| `inject_helpers` | `swc_ecma_transforms_base/src/helpers/mod.rs`: `InjectHelpers`, `Marker` (VM) | Inject or import runtime helpers |
| Super access rewriting | `swc_ecma_transforms_classes/src/super_field.rs`: `SuperFieldAccessFolder`, `MovedStaticSuperFieldAccessFolder` (VM) | Shared class/decorator lowering utilities |
| `class_fields_use_set` | `swc_ecma_transforms_compat/src/class_fields_use_set.rs`: `ClassFieldsUseSet`, `FieldsHandler`, `ComputedFieldsHandler` (VM) | Implement set semantics for class fields |
| `reserved_words` | `swc_ecma_transforms_compat/src/reserved_words.rs`: `EsReservedWord` (VM) | General reserved binding renaming; distinct from the ES3 wrapper |

### ECMAScript compatibility

`swc_ecma_compat_esYYYY` exports the individual functions and a same-year preset.
The implementation column identifies the actual owner when an export is a shim.

| Group | Entry / feature | Actual implementation |
| --- | --- | --- |
| ES3 | `property_literals` | `swc_ecma_transformer/src/es3/prop_lits.rs`: `PropertyLiteralHook` |
| ES3 | `member_expression_literals` | `swc_ecma_transformer/src/es3/member_expr_lits.rs`: `MemberExprLitHook` |
| ES3 | `reserved_words` | `swc_ecma_transformer/src/es3/reserved_word.rs`: `ReservedWordHook` |
| ES2015 | `arrow` | `swc_ecma_compat_es2015/src/arrow.rs`: `Arrow` (VM), shared `FnEnvHoister` |
| ES2015 | `block_scoped_functions` | `block_scoped_fn.rs`: `BlockScopedFns` (VM) |
| ES2015 | `block_scoping`, internal `block_scoped_vars` | `block_scoping/`: `BlockScoping`, `BlockScopedVars`, `FlowHelper`, `MutationHandler` (VM) |
| ES2015 | `classes` | `classes/`: `Classes`, `ConstructorFolder` (VM) |
| ES2015 | `computed_properties` | `computed_props.rs`: `ComputedProps` (VM) |
| ES2015 | `destructuring` | `destructuring.rs`: `Destructuring`, `AssignFolder` (VM) |
| ES2015 | `duplicate_keys` | `swc_ecma_transformer/src/es2015/duplicate_keys.rs`: `DuplicateKeysPass` (Hook) |
| ES2015 | `for_of` | `swc_ecma_compat_es2015/src/for_of.rs`: `ForOf` (VM) |
| ES2015 | `function_name` | `swc_ecma_transformer/src/es2015/function_name.rs`: `FunctionNamePass` (Hook) |
| ES2015 | `generator::generator` | `swc_ecma_compat_es2015/src/generator.rs`: `Wrapper`, `Generator`, `InvalidToLit` (VM) |
| ES2015 | `instance_of` | `swc_ecma_transformer/src/es2015/instanceof.rs`: `InstanceOfPass` (Hook) |
| ES2015 | `new_target` | `swc_ecma_compat_es2015/src/new_target.rs`: `NewTarget` (VM) |
| ES2015 | `object_super` | `object_super.rs`: `ObjectSuper`, `SuperReplacer` (VM) |
| ES2015 | `parameters` | `parameters.rs`: `Params` (VM) |
| ES2015 | `shorthand` | `swc_ecma_transformer/src/es2015/shorthand.rs`: `ShorthandPass` (Hook) |
| ES2015 | `spread` | `swc_ecma_compat_es2015/src/spread.rs`: `Spread` (VM) |
| ES2015 | `sticky_regex` | Wrapper calls `swc_ecma_transformer::regexp_pass`; an additional `StickyRegexPass` hook exists in `es2015/sticky_regex.rs` |
| ES2015 | `template_literal` | `swc_ecma_compat_es2015/src/template_literal.rs`: `TemplateLiteral` (VM) |
| ES2015 | `typeof_symbol` | `swc_ecma_transformer/src/es2015/typeof_symbol.rs`: `TypeofSymbolPass` (Hook) |
| ES2016 | `exponentiation` | `swc_ecma_transformer/src/es2016/exponentiation_operator.rs`: `ExponentiationOperatorPass` (Hook) |
| ES2017 | `async_to_generator` | `swc_ecma_transformer/src/es2017/async_to_generator.rs`: `AsyncToGeneratorPass` (Hook), handles async functions and async generators |
| ES2018 | `object_rest_spread` | `swc_ecma_transformer/src/es2018/object_rest_spread.rs`: `ObjectRestSpreadPass` (Hook) |
| ES2018 | RegExp compatibility | `es2018` preset enables RegExp options as well as object rest/spread |
| ES2019 | `optional_catch_binding` | `swc_ecma_transformer/src/es2019/optional_catch_binding.rs`: `OptionalCatchBindingPass` (Hook + VM) |
| ES2020 | `export_namespace_from` | `swc_ecma_transformer/src/es2020/export_namespace_from.rs`: `ExportNamespaceFromPass` (Hook + VM) |
| ES2020 | `nullish_coalescing` | `swc_ecma_transformer/src/es2020/nullish_coalescing.rs`: `NullishCoalescingPass` (Hook) |
| ES2020 | `optional_chaining` | Wrapper instantiates `swc_ecma_compat_es2022/src/optional_chaining_impl.rs`: `OptionalChaining` (VM) |
| ES2021 | `logical_assignments` | `swc_ecma_transformer/src/es2021/logical_assignment_operators.rs`: `LogicalAssignmentOperatorsPass` (Hook) |
| ES2022 | `static_blocks` | `swc_ecma_transformer/src/es2022/class_static_block.rs`: `ClassStaticBlock` (Hook) |
| ES2022 | `class_properties` | `swc_ecma_compat_es2022/src/class_properties/`: `ClassProperties` plus private-field, class-name-TDZ, static-this, and name-analysis visitors |
| ES2022 | `private_in_object` | `swc_ecma_transformer/src/es2022/private_property_in_object.rs`: `PrivatePropertyInObjectFastPass` (Pass), `PrivatePropertyInObjectPass` (Hook), `PrivateInFinder`, `ClassAnalyzer` |
| RegExp | `swc_ecma_compat_common::regexp::regexp`, `regexp_pass` | `swc_ecma_transformer/src/regexp.rs`: `RegexpPass` (Hook + VM); flags include dotAll, indices, lookbehind, named/duplicate named groups, sticky, Unicode properties, Unicode, Unicode sets |
| RegExp | `transform_unicode_property_escapes` | `swc_ecma_compat_regexp/src/unicode_property.rs`: `UnicodePropertyTransformer`; visits the RegExp AST rather than the JavaScript AST |

`swc_ecma_compat_es2015/src/regenerator/mod.rs` is empty at this revision.
The active generator transform is `generator::generator`. Likewise the files
`swc_ecma_transformer/src/es2020/optional_chaining.rs` and
`es2022/class_properties/mod.rs` are empty; their active implementations are in
the compatibility crates listed above. Option names and file names alone do not
establish implemented support.

### Browser bug fixes

All four standalone passes live in `swc_ecma_compat_bugfixes`; `bugfixes` composes
them. The similarly named hook module in `swc_ecma_transformer` is a placeholder.

| Entry | Visitor | Purpose |
| --- | --- | --- |
| `async_arrows_in_class` | `AsyncArrowsInClass` (F) | Async arrows in class methods |
| `edge_default_param` | `EdgeDefaultParam` (VM) | Edge default-parameter destructuring |
| `template_literal_caching` | `TemplateLiteralCaching` (F) | Tagged-template object caching |
| `safari_id_destructuring_collision_in_function_expression` | `SafariIdDestructuringCollisionInFunctionExpression` (VM) | Safari function-expression/destructuring name collision |

### Proposals, TypeScript, and React

| Group | Entry | Actual implementation / behavior |
| --- | --- | --- |
| Proposal | `decorators` | `swc_ecma_transforms_proposal/src/decorators/`: `Decorators` (F); legacy branch uses `TscDecorator` (Visit + VM), `Metadata`, `ParamMetadata` |
| Proposal | `decorator_2022_03`, `decorator_2023_11` | Both select a version of `decorator_impl.rs`: `DecoratorPass` (VM) and shared analysis visitors |
| Proposal | `explicit_resource_management` | `ExplicitResourceManagement` (VM); lowers `using` / `await using` |
| Proposal | `export_default_from` | `ExportDefaultFrom` (Pass) |
| Proposal | `import_attributes` | `ImportAssertions` (VM); removes import attributes/assertions |
| TypeScript | `typescript`, `strip` | `swc_ecma_transforms_typescript`: `TypeScript` (Pass), semantic analysis, then `Transform` (VM); includes enums, namespaces, parameter properties, import/export assignments, type erasure, and optional Flow cleanup |
| TypeScript | `tsx` | `TypeScriptReact` (VM); records JSX pragma usage before TypeScript lowering |
| React | `jsx`, `jsx::hook` | `swc_ecma_transforms_react/src/jsx/mod.rs`: `Jsx` (Hook); classic/automatic runtime, pragmas, development output |
| React | `jsx_self` | `JsxSelf` (Hook); development `__self` metadata |
| React | `jsx_src` | `JsxSrc` (Hook); development `__source` metadata |
| React | `display_name` | `DisplayName` (Hook), nested `Folder` (VM) |
| React | `pure_annotations` | `PureAnnotations` (Hook); comment-based purity annotations |
| React | `refresh` | `Refresh` (Hook), `HookRegister` (VM), hook/JSX/span collectors |
| React | `react` | Nested `CompositeHook`: source → self → refresh → JSX → display name → pure annotations on enter; reverse order on exit |

The legacy `swc_ecma_transformer` hooks `TypeScriptPass`, `DecoratorPass`,
`JsxPass`, `BugfixPass`, and `Es2026Pass` have empty implementations. They are
listed in the symbol inventory as placeholders, not working feature ports.

### Modules

| Entry / stage | Implementation in `swc_ecma_transforms_module/src/` | Role |
| --- | --- | --- |
| `common_js` | `common_js.rs`: `Cjs` (VM) | ESM to CommonJS |
| `amd` | `amd.rs`: `Amd` (VM), `amd/emit.rs` | ESM to AMD |
| `umd` | `umd.rs`: `Umd` (VM), `umd/emit.rs` | ESM to UMD |
| `system_js` | `system_js.rs`: `SystemJs` (VM), `system_js/` IR/lower/emit/rewrite | ESM to System.register; live export and special-reference rewriting |
| `import_analyzer` | `import_analysis.rs`: `ImportAnalyzer` (Visit + VM) | Record interop helper requirements |
| `swc_import_rewriter` | `rewriter/import_rewriter_swc.rs`: `Rewriter` (VM) | Rewrite import paths using resolver |
| `typescript_import_rewriter` | `rewriter/import_rewriter_typescript.rs`: `Rewriter` (VM) | Rewrite relative TS import extensions |
| Top-level `this` | `top_level_this.rs`: `TopLevelThis` (VM) | Shared module `this` handling |
| Source analysis / lowering | `module_record.rs`, `module_ref_rewriter.rs`, `syntax_strip.rs` | Ordinary functions shared by module backends; not standalone visitor factories |

ESM preservation / NodeNext are compiler configuration branches, not additional
ESM-to-module visitor implementations. Dynamic import handling is part of the
module backends, not a separate transform crate.

### Optimization and extra compiler passes

| Entry / stage | Implementation | Role |
| --- | --- | --- |
| `const_modules` | `swc_ecma_transforms_optimization/src/const_modules.rs`: `ConstModules` (VM) | Inline configured module constants |
| `inline_globals` | `inline_globals.rs`: `InlineGlobals` (VM) | Inline configured globals / environment values |
| `json_parse` | `json_parse.rs`: `JsonParse` (VM) | Replace sufficiently large literals with JSON.parse |
| `simplifier` | `simplify/mod.rs` | Repeat expression simplification → dead-branch removal → DCE |
| `expr_simplifier` | `simplify/expr/mod.rs`: `SimplifyExpr` (VM + Repeated + CompilerPass) | Expression folding/simplification |
| `dead_branch_remover` | `simplify/branch/mod.rs`: `Remover` (VM + Repeated + CompilerPass) | Remove unreachable branches |
| `dce` | `simplify/dce/mod.rs`: `TreeShaker` (VM + Repeated + CompilerPass), analysis visitors | Eliminate dead declarations/code |
| `inlining` | `simplify/inlining/mod.rs`: `Inlining` (VM + Repeated + CompilerPass) | Separate inliner; not called by current `simplifier` body |
| `constant_propagation` | `simplify/const_propagation.rs`: `ConstPropagation` (VM) | Separate lightweight constant propagation |
| `debug_assert_valid` | `debug.rs`: `AssertValid` (Visit) | Debug AST validation; not a lowering pass |
| `jest` | `swc_ecma_ext_transforms/src/jest.rs`: `Jest` (VM) | Hoist Jest calls |
| `dropped_comments_preserver` | `swc/src/dropped_comments_preserver.rs`: `DroppedCommentsPreserver` (VM) | Preserve comments from removed nodes |
| `plugins` | `swc/src/plugin.rs`: `RustPlugins` (F) | Compiler's external plugin execution adapter |
| Minification | `swc/src/builder.rs`: `MinifierPass` (Pass) | Calls the minifier; compression/mangling internals are separate scope |
| Global pass options | `swc/src/config/mod.rs`: `GlobalPassOption::build` | Builds configured global/environment inlining |
| Optimizer assembly | `swc/src/config/mod.rs`: `build_as_input` | Combines constant modules, global inlining, simplification, and JSON optimization |

### Environment selection and polyfills

| Entry / stage | Implementation in `swc_ecma_preset_env/src/` | Role |
| --- | --- | --- |
| `transform_from_env` | `lib.rs` | Select transforms from browser targets; append polyfill pass |
| `transform_from_es_version` | `lib.rs` | Select transforms from ES target; no core-js polyfill pass |
| `transform_internal` | `lib.rs` | Shared ordered compatibility pipeline |
| `strip_node_colon_prefix` | `node_colon_prefix_strip.rs`: `NodeColonPrefixStrip` (VM) | Rewrite `node:` specifiers for older Node targets |
| Polyfills | `lib.rs`: `Polyfills` (VM) | Select/inject configured core-js imports |
| Core-js 2 | `corejs2/entry.rs`: `Entry` (VM); `corejs2/mod.rs`: `UsageVisitor` (Visit) | Entry expansion / usage analysis |
| Core-js 3 | `corejs3/entry.rs`: `Entry` (VM); `corejs3/usage.rs`: `UsageVisitor` (Visit) | Entry expansion / usage analysis |

### Traversal infrastructure and utility visitors

| Component | Relevant abstractions / visitors | Porting relationship |
| --- | --- | --- |
| `swc_ecma_ast` | `Pass::process`, tuples/options/either/noop adapters | Legacy pipeline execution contract |
| `swc_ecma_visit` | `Visit`, `VisitMut`, `Fold`, `visit_pass`, `visit_mut_pass`, `fold_pass`, `InjectVars` | Generated traversal and pass adapters |
| `swc_ecma_hooks` | `VisitMutHook`, `VisitMutWithHook`, `CompositeHook`, `NoopHook` | Hook traversal and composition |
| `swc_common::pass` | `CompilerPass`, `Repeated`, `Repeat`, `Optional` | Repetition and conditional execution |
| `swc_ecma_transforms_macros` | `fast_path`, `parallel` | Generated fast-path and parallel visitation support |
| `swc_ecma_transforms_base::perf` | `Check`, `should_work`, `ParVisit`, `ParVisitMut`, `ParFold`, `Parallel`, `ParExplode` | Fast-path analysis / parallel traversal |
| `swc_ecma_transformer` | `TraverseCtx`, `HookBuilder`, `OrderedChain`, `NoopHook`, `OptionalPass`, `StmtInjector`, `VarDeclarations` | Hook ordering and generated statement/declaration storage |
| `swc_ecma_utils` | `FnEnvHoister`, `InitThis`, constructor `Injector`, `Hoister`, identifier/binding/this/arguments collectors, `RefRewriter`, `Remapper`, `IdentRenamer`, `IdentReplacer`, `DropSpan` | Shared rewriting and analysis; every explicit visitor implementation is in the symbol tables |

## Pipeline constraints to preserve

These are observations of the pinned compiler source, not a proposed new API.
The source of truth for assembly is
[`swc/src/config/mod.rs`](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/config/mod.rs)
and
[`swc_ecma_preset_env/src/lib.rs`](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/lib.rs).

| Relationship | Reason / observation |
| --- | --- |
| Resolver before custom passes and lowering | Generated identifiers and references rely on resolved contexts |
| Decorators before TypeScript erasure | Decorator metadata can require type information |
| TS/TSX preparation before React lowering | JSX pragma usage affects import retention |
| React composite hook order | Enter: source → self → refresh → JSX → display name → purity; exit: reverse order; callbacks share traversal |
| Class static blocks before class properties | Static blocks lower into private static properties |
| `for_of` before `parameters`; `parameters` before `arrow` | Order is explicit in the ES2015 preset |
| Hook transforms before `VarDeclarations` | Newly queued declarations must be flushed after their producers |
| Import analysis → import rewrite → helper injection → module lowering | Import extension rewriting may request a helper; module conversion needs helper/interop information |
| Compatibility → modules → minifier → hygiene → fixer | Order in `built_pass`; hygiene is conditional and skipped when mangling is enabled |
| Final Jest / dropped comments passes | Run after `built_pass` in compiler assembly |
| `simplifier` repeats to a fixed point | Its actual tuple is expression simplifier, branch remover, DCE; inlining is not in the tuple |

The target-driven pipeline does not simply concatenate all same-year presets.
It places class fields and optional chaining before a shared ES2022–ES2016 hook
pass, then performs the ES2015 stages with a separate ES2015 hook pass, followed
by computed properties, destructuring, block scoping, generators, ES3, and browser
bug fixes. `function_name` is in that ES2015 hook group in `preset_env`, whereas
the standalone `es2015` preset schedules it earlier. Preserve the ordering of the
entry being ported rather than inferring order from directory names.

`OrderedChain` calls A then B on both enter and exit. React's `CompositeHook`
calls A then B on enter, and B then A on exit. Both compose callbacks within a
traversal. Treating every hook as an independent full AST pass can change
behavior.

## Existing SWC Next coverage

These are existing source counterparts in this workspace, not verified parity
or implementations in the new crate. Everything in the new legacy crate is
still unported.

| Legacy feature | Existing SWC Next source | Status for this inventory |
| --- | --- | --- |
| Optional catch binding | `swc_ecma_transformer/src/es2019/optional_catch_binding.rs` | Counterpart exists |
| Class static blocks | `swc_ecma_transformer/src/es2022/static_blocks.rs` | Counterpart exists |
| JSX, source/self, display name, refresh | `swc_ecma_transformer/src/jsx/` | Counterparts exist; purity behavior needs parity review |
| TS type erasure and runtime constructs | `swc_ecma_transformer/src/typescript/`, `swc_type_strip` | Counterparts exist; legacy Flow/config behavior needs separate review |
| Identifier/scope management | `swc_ecma_semantic`, transformer `context/` and `uid.rs` | Different AST/semantic model; adapt legacy context-dependent algorithms |
| Runtime helpers | `packages/helpers` and transformer helper injection | Existing helper system; audit helpers needed by each port |
| Simplification / DCE / mangling | `swc_ecma_minifier` | Related optimizer exists; no parity claim for legacy optimization passes |
| Other compatibility/proposal/module transforms | No corresponding transform exposed in current `TransformerOptions` | Pending migration assessment |

The new crate currently has no dependency on legacy SWC, no AST conversion
bridge, and no placeholder transform methods. Implementations can be added after
the feature and shared infrastructure boundaries have been chosen.
