# Legacy visitor and pass symbol tables

Snapshot: [`c682b13a8ea7`](https://github.com/swc-project/swc/tree/c682b13a8ea7372203970d315df38b529ce5b7f0).

See the [feature map and scope](transform-inventory.md) for implementation owners,
wrapper relationships, placeholder hooks, and ordering constraints.

The inventory inspected 262 production source files. It lists
177 pass/hook factory or builder functions and 210 explicit trait
implementations. These counts include internal utilities and adapters; they are
not counts of independent transforms. Multiple traits on one type are separate
implementation rows. Conditional implementations are listed even when their
feature is disabled. Macro-expanded implementations are not expanded here.

Factories include functions returning `Pass`, `VisitMut`, `Fold`,
`VisitMutHook`, a container of these, or the concrete `OptionalChaining` visitor.
Visibility is the declaration visibility, not a claim that the containing module
is publicly reachable. `private/method` denotes a declaration without `pub`.

Implementation rows cover `Visit`, `VisitMut`, `Fold`, `VisitMutHook`, `Pass`,
`CompilerPass`, `Repeated`, `Check`, `ParVisit`, `ParVisitMut`, and `ParFold`.
Each source link points at the exact declaration, including nested/local visitors.
The RegExp transformer uses a different AST but is retained as a dependency.

## Counts by crate

| Crate | Factories / builders | Explicit implementations |
| --- | ---: | ---: |
| `swc` | 6 | 3 |
| `swc_ecma_compat_bugfixes` | 5 | 4 |
| `swc_ecma_compat_common` | 1 | 0 |
| `swc_ecma_compat_es2015` | 21 | 34 |
| `swc_ecma_compat_es2016` | 2 | 0 |
| `swc_ecma_compat_es2017` | 2 | 0 |
| `swc_ecma_compat_es2018` | 2 | 0 |
| `swc_ecma_compat_es2019` | 2 | 0 |
| `swc_ecma_compat_es2020` | 4 | 0 |
| `swc_ecma_compat_es2021` | 2 | 0 |
| `swc_ecma_compat_es2022` | 5 | 12 |
| `swc_ecma_compat_es3` | 4 | 0 |
| `swc_ecma_compat_regexp` | 0 | 1 |
| `swc_ecma_ext_transforms` | 1 | 1 |
| `swc_ecma_preset_env` | 4 | 6 |
| `swc_ecma_transformer` | 64 | 35 |
| `swc_ecma_transforms` | 0 | 0 |
| `swc_ecma_transforms_base` | 10 | 19 |
| `swc_ecma_transforms_classes` | 0 | 2 |
| `swc_ecma_transforms_compat` | 2 | 4 |
| `swc_ecma_transforms_module` | 7 | 11 |
| `swc_ecma_transforms_optimization` | 9 | 23 |
| `swc_ecma_transforms_proposal` | 7 | 15 |
| `swc_ecma_transforms_react` | 13 | 11 |
| `swc_ecma_transforms_typescript` | 4 | 7 |
| `swc_ecma_utils` | 0 | 22 |

## Declarations by crate

### swc

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `build_as_input` | pub | `Result<BuiltInput<Box<dyn 'a + Pass>>, Error>` | [config/mod.rs:264](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/config/mod.rs#L264) |
| `build` | pub | `Box<dyn Pass + 'cmt>` | [config/mod.rs:1640](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/config/mod.rs#L1640) |
| `build` | pub | `Box<dyn Pass + 'cmt>` | [config/mod.rs:1791](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/config/mod.rs#L1791) |
| `build` | pub | `impl 'static + Pass` | [config/mod.rs:2259](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/config/mod.rs#L2259) |
| `dropped_comments_preserver` | pub | `impl Pass` | [dropped_comments_preserver.rs:19](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/dropped_comments_preserver.rs#L19) |
| `plugins` | restricted | `impl Pass` | [plugin.rs:50](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/plugin.rs#L50) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `MinifierPass<'_>` | `Pass` | [builder.rs:16](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/builder.rs#L16) |
| `DroppedCommentsPreserver` | `VisitMut` | [dropped_comments_preserver.rs:35](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/dropped_comments_preserver.rs#L35) |
| `RustPlugins` | `Fold` | [plugin.rs:203](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc/src/plugin.rs#L203) |

### swc_ecma_compat_bugfixes

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `async_arrows_in_class` | pub | `impl Pass` | [async_arrows_in_class.rs:14](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/async_arrows_in_class.rs#L14) |
| `edge_default_param` | pub | `impl Pass` | [edge_default_param.rs:10](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/edge_default_param.rs#L10) |
| `bugfixes` | pub | `impl Pass` | [lib.rs:15](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/lib.rs#L15) |
| `safari_id_destructuring_collision_in_function_expression` | pub | `impl Pass` | [safari_id_destructuring_collision_in_function_expression.rs:10](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/safari_id_destructuring_collision_in_function_expression.rs#L10) |
| `template_literal_caching` | pub | `impl Pass` | [template_literal_caching.rs:21](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/template_literal_caching.rs#L21) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `AsyncArrowsInClass` | `Fold` | [async_arrows_in_class.rs:28](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/async_arrows_in_class.rs#L28) |
| `EdgeDefaultParam` | `VisitMut` | [edge_default_param.rs:18](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/edge_default_param.rs#L18) |
| `SafariIdDestructuringCollisionInFunctionExpression` | `VisitMut` | [safari_id_destructuring_collision_in_function_expression.rs:32](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/safari_id_destructuring_collision_in_function_expression.rs#L32) |
| `TemplateLiteralCaching` | `Fold` | [template_literal_caching.rs:59](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_bugfixes/src/template_literal_caching.rs#L59) |

### swc_ecma_compat_common

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `regexp` | pub | `impl Pass` | [regexp.rs:3](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_common/src/regexp.rs#L3) |

### swc_ecma_compat_es2015

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `arrow` | pub | `impl Pass + VisitMut + InjectVars` | [arrow.rs:59](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/arrow.rs#L59) |
| `block_scoped_functions` | pub | `impl Pass` | [block_scoped_fn.rs:6](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoped_fn.rs#L6) |
| `block_scoping` | pub | `impl Pass` | [block_scoping/mod.rs:37](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoping/mod.rs#L37) |
| `block_scoped_vars` | restricted | `impl VisitMut` | [block_scoping/vars.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoping/vars.rs#L9) |
| `classes` | pub | `impl Pass` | [classes/mod.rs:28](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/classes/mod.rs#L28) |
| `computed_properties` | pub | `impl Pass` | [computed_props.rs:41](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/computed_props.rs#L41) |
| `destructuring` | pub | `impl Pass` | [destructuring.rs:38](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/destructuring.rs#L38) |
| `duplicate_keys` | pub | `impl Pass` | [duplicate_keys.rs:3](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/duplicate_keys.rs#L3) |
| `for_of` | pub | `impl Pass` | [for_of.rs:52](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/for_of.rs#L52) |
| `function_name` | pub | `impl Pass` | [function_name.rs:20](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/function_name.rs#L20) |
| `generator` | pub | `impl Pass` | [generator.rs:26](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/generator.rs#L26) |
| `instance_of` | pub | `impl Pass` | [instanceof.rs:28](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/instanceof.rs#L28) |
| `es2015` | pub | `impl Pass` | [lib.rs:47](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/lib.rs#L47) |
| `new_target` | pub | `impl Pass` | [new_target.rs:11](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/new_target.rs#L11) |
| `object_super` | pub | `impl Pass` | [object_super.rs:15](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/object_super.rs#L15) |
| `parameters` | pub | `impl 'static + Pass` | [parameters.rs:18](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/parameters.rs#L18) |
| `shorthand` | pub | `impl Pass` | [shorthand_property.rs:37](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/shorthand_property.rs#L37) |
| `spread` | pub | `impl Pass` | [spread.rs:16](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/spread.rs#L16) |
| `sticky_regex` | pub | `impl Pass` | [sticky_regex.rs:17](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/sticky_regex.rs#L17) |
| `template_literal` | pub | `impl Pass` | [template_literal.rs:11](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/template_literal.rs#L11) |
| `typeof_symbol` | pub | `impl Pass` | [typeof_symbol.rs:4](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/typeof_symbol.rs#L4) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `Arrow` | `VisitMut` | [arrow.rs:72](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/arrow.rs#L72) |
| `BlockScopedFns` | `VisitMut` | [block_scoped_fn.rs:13](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoped_fn.rs#L13) |
| `BlockScoping` | `VisitMut` | [block_scoping/mod.rs:449](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoping/mod.rs#L449) |
| `FlowHelper<'_>` | `VisitMut` | [block_scoping/mod.rs:700](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoping/mod.rs#L700) |
| `MutationHandler<'_>` | `VisitMut` | [block_scoping/mod.rs:932](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoping/mod.rs#L932) |
| `NestedMutationCollector<'_, '_>` | `Visit` | [block_scoping/nested_mutation.rs:21](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoping/nested_mutation.rs#L21) |
| `BlockScopedVars` | `VisitMut` | [block_scoping/vars.rs:248](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/block_scoping/vars.rs#L248) |
| `ConstructorFolder` | `VisitMut` | [classes/constructor.rs:235](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/classes/constructor.rs#L235) |
| `Classes` | `VisitMut` | [classes/mod.rs:203](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/classes/mod.rs#L203) |
| `ClassFinder` | `Visit` | [classes/mod.rs:972](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/classes/mod.rs#L972) |
| `ClassFinder` | `Check` | [classes/mod.rs:1004](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/classes/mod.rs#L1004) |
| `ComputedProps` | `VisitMut` | [computed_props.rs:62](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/computed_props.rs#L62) |
| `ComplexVisitor` | `Visit` | [computed_props.rs:331](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/computed_props.rs#L331) |
| `ShouldWork` | `Visit` | [computed_props.rs:419](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/computed_props.rs#L419) |
| `Destructuring` | `VisitMut` | [destructuring.rs:476](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/destructuring.rs#L476) |
| `AssignFolder` | `VisitMut` | [destructuring.rs:600](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/destructuring.rs#L600) |
| `DestructuringVisitor` | `Visit` | [destructuring.rs:1336](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/destructuring.rs#L1336) |
| `DestructuringVisitor` | `Check` | [destructuring.rs:1383](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/destructuring.rs#L1383) |
| `ForOf` | `VisitMut` | [for_of.rs:691](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/for_of.rs#L691) |
| `Wrapper` | `VisitMut` | [generator.rs:54](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/generator.rs#L54) |
| `Generator` | `VisitMut` | [generator.rs:402](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/generator.rs#L402) |
| `YieldFinder` | `Visit` | [generator.rs:3708](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/generator.rs#L3708) |
| `InvalidToLit<'_>` | `VisitMut` | [generator.rs:3737](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/generator.rs#L3737) |
| `NewTarget` | `VisitMut` | [new_target.rs:37](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/new_target.rs#L37) |
| `NewTarget` | `CompilerPass` | [new_target.rs:140](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/new_target.rs#L140) |
| `ShouldWork` | `Visit` | [new_target.rs:151](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/new_target.rs#L151) |
| `ShouldWork` | `Check` | [new_target.rs:165](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/new_target.rs#L165) |
| `ObjectSuper` | `VisitMut` | [object_super.rs:21](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/object_super.rs#L21) |
| `SuperReplacer` | `VisitMut` | [object_super.rs:133](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/object_super.rs#L133) |
| `Params` | `VisitMut` | [parameters.rs:490](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/parameters.rs#L490) |
| `Spread` | `VisitMut` | [spread.rs:39](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/spread.rs#L39) |
| `SpreadFinder` | `Visit` | [spread.rs:527](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/spread.rs#L527) |
| `SpreadFinder` | `Check` | [spread.rs:537](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/spread.rs#L537) |
| `TemplateLiteral` | `VisitMut` | [template_literal.rs:43](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2015/src/template_literal.rs#L43) |

### swc_ecma_compat_es2016

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `exponentiation` | pub | `impl Pass` | [exponentiation.rs:22](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2016/src/exponentiation.rs#L22) |
| `es2016` | pub | `impl Pass` | [lib.rs:10](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2016/src/lib.rs#L10) |

### swc_ecma_compat_es2017

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `async_to_generator` | pub | `impl Pass` | [async_to_generator.rs:25](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2017/src/async_to_generator.rs#L25) |
| `es2017` | pub | `impl Pass` | [lib.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2017/src/lib.rs#L9) |

### swc_ecma_compat_es2018

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `es2018` | pub | `impl Pass` | [lib.rs:12](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2018/src/lib.rs#L12) |
| `object_rest_spread` | pub | `impl Pass` | [object_rest_spread.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2018/src/object_rest_spread.rs#L9) |

### swc_ecma_compat_es2019

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `es2019` | pub | `impl Pass` | [lib.rs:7](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2019/src/lib.rs#L7) |
| `optional_catch_binding` | pub | `impl Pass` | [optional_catch_binding.rs:3](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2019/src/optional_catch_binding.rs#L3) |

### swc_ecma_compat_es2020

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `export_namespace_from` | pub | `impl Pass` | [export_namespace_from.rs:3](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2020/src/export_namespace_from.rs#L3) |
| `es2020` | pub | `impl Pass` | [lib.rs:14](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2020/src/lib.rs#L14) |
| `nullish_coalescing` | pub | `impl Pass + 'static` | [nullish_coalescing.rs:15](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2020/src/nullish_coalescing.rs#L15) |
| `optional_chaining` | pub | `impl Pass` | [optional_chaining.rs:7](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2020/src/optional_chaining.rs#L7) |

### swc_ecma_compat_es2021

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `es2021` | pub | `impl Pass` | [lib.rs:7](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2021/src/lib.rs#L7) |
| `logical_assignments` | pub | `impl Pass` | [logical_assignments.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2021/src/logical_assignments.rs#L9) |

### swc_ecma_compat_es2022

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `class_properties` | pub | `impl Pass` | [class_properties/mod.rs:44](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L44) |
| `es2022` | pub | `impl Pass` | [lib.rs:16](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/lib.rs#L16) |
| `optional_chaining_impl` | pub | `OptionalChaining` | [optional_chaining_impl.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L9) |
| `private_in_object` | pub | `impl Pass` | [private_in_object.rs:4](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/private_in_object.rs#L4) |
| `static_blocks` | pub | `impl Pass` | [static_blocks.rs:3](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/static_blocks.rs#L3) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `ClassNameTdzFolder<'_>` | `VisitMut` | [class_properties/class_name_tdz.rs:11](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/class_name_tdz.rs#L11) |
| `ClassProperties` | `VisitMut` | [class_properties/mod.rs:140](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L140) |
| `PrivateNameFinder` | `Visit` | [class_properties/mod.rs:1076](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L1076) |
| `ShouldWork` | `Visit` | [class_properties/mod.rs:1089](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L1089) |
| `ShouldWork` | `Check` | [class_properties/mod.rs:1151](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L1151) |
| `SuperVisitor` | `Visit` | [class_properties/mod.rs:1162](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/mod.rs#L1162) |
| `BrandCheckHandler<'_>` | `VisitMut` | [class_properties/private_field.rs:83](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L83) |
| `PrivateAccessVisitor<'_>` | `VisitMut` | [class_properties/private_field.rs:194](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L194) |
| `ThisInStaticFolder` | `VisitMut` | [class_properties/this_in_static.rs:8](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/this_in_static.rs#L8) |
| `NewTargetInProp` | `VisitMut` | [class_properties/this_in_static.rs:27](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/this_in_static.rs#L27) |
| `UsedNameCollector<'_>` | `Visit` | [class_properties/used_name.rs:16](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/class_properties/used_name.rs#L16) |
| `OptionalChaining` | `VisitMut` | [optional_chaining_impl.rs:37](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L37) |

### swc_ecma_compat_es3

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `es3` | pub | `impl Pass` | [lib.rs:26](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es3/src/lib.rs#L26) |
| `member_expression_literals` | pub | `impl Pass` | [member_expr_lits.rs:22](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es3/src/member_expr_lits.rs#L22) |
| `property_literals` | pub | `impl Pass` | [prop_lits.rs:32](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es3/src/prop_lits.rs#L32) |
| `reserved_words` | pub | `impl Pass` | [reserved_word.rs:22](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_es3/src/reserved_word.rs#L22) |

### swc_ecma_compat_regexp

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `UnicodePropertyTransformer` | `VisitMut` | [unicode_property.rs:45](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_compat_regexp/src/unicode_property.rs#L45) |

### swc_ecma_ext_transforms

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `jest` | pub | `impl Pass` | [jest.rs:15](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_ext_transforms/src/jest.rs#L15) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `Jest` | `VisitMut` | [jest.rs:81](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_ext_transforms/src/jest.rs#L81) |

### swc_ecma_preset_env

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `transform_internal` | private/method | `impl Pass` | [lib.rs:44](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/lib.rs#L44) |
| `transform_from_env` | pub | `impl Pass` | [lib.rs:331](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/lib.rs#L331) |
| `transform_from_es_version` | pub | `impl Pass` | [lib.rs:389](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/lib.rs#L389) |
| `strip_node_colon_prefix` | pub | `impl Pass` | [node_colon_prefix_strip.rs:5](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/node_colon_prefix_strip.rs#L5) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `Entry` | `VisitMut` | [corejs2/entry.rs:70](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/corejs2/entry.rs#L70) |
| `UsageVisitor` | `Visit` | [corejs2/mod.rs:124](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/corejs2/mod.rs#L124) |
| `Entry` | `VisitMut` | [corejs3/entry.rs:102](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/corejs3/entry.rs#L102) |
| `UsageVisitor` | `Visit` | [corejs3/usage.rs:165](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/corejs3/usage.rs#L165) |
| `Polyfills` | `VisitMut` | [lib.rs:509](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/lib.rs#L509) |
| `NodeColonPrefixStrip` | `VisitMut` | [node_colon_prefix_strip.rs:15](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_preset_env/src/node_colon_prefix_strip.rs#L15) |

### swc_ecma_transformer

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [bugfix/mod.rs:17](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/bugfix/mod.rs#L17) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [decorators/mod.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/decorators/mod.rs#L9) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2015/duplicate_keys.rs:43](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/duplicate_keys.rs#L43) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2015/function_name.rs:37](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/function_name.rs#L37) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2015/instanceof.rs:41](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/instanceof.rs#L41) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2015/mod.rs:49](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/mod.rs#L49) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2015/shorthand.rs:39](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/shorthand.rs#L39) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2015/sticky_regex.rs:29](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/sticky_regex.rs#L29) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2015/typeof_symbol.rs:33](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/typeof_symbol.rs#L33) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2016/exponentiation_operator.rs:40](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2016/exponentiation_operator.rs#L40) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2016/mod.rs:20](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2016/mod.rs#L20) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2017/async_to_generator.rs:19](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L19) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2017/mod.rs:26](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2017/mod.rs#L26) |
| `hook` | restricted | `impl VisitMutHook<TraverseCtx>` | [es2018/mod.rs:21](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2018/mod.rs#L21) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2018/object_rest_spread.rs:21](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L21) |
| `hook` | restricted | `impl VisitMutHook<TraverseCtx>` | [es2019/mod.rs:20](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2019/mod.rs#L20) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2019/optional_catch_binding.rs:8](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2019/optional_catch_binding.rs#L8) |
| `pass` | restricted | `impl Pass` | [es2019/optional_catch_binding.rs:12](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2019/optional_catch_binding.rs#L12) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2020/export_namespace_from.rs:8](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2020/export_namespace_from.rs#L8) |
| `pass` | restricted | `impl Pass` | [es2020/export_namespace_from.rs:12](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2020/export_namespace_from.rs#L12) |
| `hook` | restricted | `impl VisitMutHook<TraverseCtx>` | [es2020/mod.rs:30](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2020/mod.rs#L30) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2020/nullish_coalescing.rs:48](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2020/nullish_coalescing.rs#L48) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2021/logical_assignment_operators.rs:45](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2021/logical_assignment_operators.rs#L45) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2021/mod.rs:23](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2021/mod.rs#L23) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2022/class_static_block.rs:48](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/class_static_block.rs#L48) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2022/mod.rs:29](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/mod.rs#L29) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2022/private_property_in_object.rs:47](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L47) |
| `pass` | restricted | `impl Pass` | [es2022/private_property_in_object.rs:51](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L51) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es2026/mod.rs:17](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2026/mod.rs#L17) |
| `hook` | restricted | `impl VisitMutHook<TraverseCtx>` | [es3/member_expr_lits.rs:26](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es3/member_expr_lits.rs#L26) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [es3/mod.rs:53](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es3/mod.rs#L53) |
| `hook` | restricted | `impl VisitMutHook<TraverseCtx>` | [es3/prop_lits.rs:36](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es3/prop_lits.rs#L36) |
| `hook` | restricted | `impl VisitMutHook<TraverseCtx>` | [es3/reserved_word.rs:43](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es3/reserved_word.rs#L43) |
| `chain` | pub | `HookBuilder<impl VisitMutHook<TraverseCtx>>` | [hook_utils.rs:156](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/hook_utils.rs#L156) |
| `chain_optional` | pub | `HookBuilder<impl VisitMutHook<TraverseCtx>>` | [hook_utils.rs:165](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/hook_utils.rs#L165) |
| `build` | pub | `impl VisitMutHook<TraverseCtx>` | [hook_utils.rs:172](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/hook_utils.rs#L172) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [jsx/mod.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/jsx/mod.rs#L9) |
| `transform_hook` | pub | `impl VisitMutHook<TraverseCtx>` | [lib.rs:40](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L40) |
| `hook_pass` | pub | `impl Pass` | [lib.rs:84](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L84) |
| `hook_pass_with_var_declarations` | private/method | `impl Pass` | [lib.rs:90](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L90) |
| `es2022_static_blocks` | pub | `impl Pass` | [lib.rs:114](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L114) |
| `es2022_private_in_object` | pub | `impl Pass` | [lib.rs:118](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L118) |
| `es2022_runtime_transforms` | pub | `impl Pass` | [lib.rs:122](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L122) |
| `es2021_logical_assignments` | pub | `impl Pass` | [lib.rs:134](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L134) |
| `es2020_export_namespace_from` | pub | `impl Pass` | [lib.rs:138](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L138) |
| `es2020_nullish_coalescing` | pub | `impl Pass` | [lib.rs:142](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L142) |
| `es2020_runtime_transforms` | pub | `impl Pass` | [lib.rs:146](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L146) |
| `es2019_optional_catch_binding` | pub | `impl Pass` | [lib.rs:155](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L155) |
| `es2018_object_rest_spread` | pub | `impl Pass` | [lib.rs:159](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L159) |
| `es2018_runtime_transforms` | pub | `impl Pass` | [lib.rs:163](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L163) |
| `es2017_async_to_generator` | pub | `impl Pass` | [lib.rs:175](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L175) |
| `es2016_exponentiation` | pub | `impl Pass` | [lib.rs:187](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L187) |
| `es2015_shorthand` | pub | `impl Pass` | [lib.rs:191](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L191) |
| `es2015_function_name` | pub | `impl Pass` | [lib.rs:195](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L195) |
| `es2015_duplicate_keys` | pub | `impl Pass` | [lib.rs:199](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L199) |
| `es2015_sticky_regex` | pub | `impl Pass` | [lib.rs:203](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L203) |
| `es2015_instanceof` | pub | `impl Pass` | [lib.rs:210](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L210) |
| `es2015_typeof_symbol` | pub | `impl Pass` | [lib.rs:214](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L214) |
| `es2015_runtime_transforms` | pub | `impl Pass` | [lib.rs:218](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L218) |
| `regexp_pass` | pub | `impl Pass` | [lib.rs:233](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L233) |
| `into_pass` | pub | `impl Pass` | [lib.rs:243](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L243) |
| `hook` | restricted | `Option<impl VisitMutHook<TraverseCtx>>` | [regexp.rs:61](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/regexp.rs#L61) |
| `pass` | restricted | `impl Pass` | [regexp.rs:78](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/regexp.rs#L78) |
| `hook` | pub | `impl VisitMutHook<TraverseCtx>` | [typescript/mod.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/typescript/mod.rs#L9) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `BugfixPass` | `VisitMutHook` | [bugfix/mod.rs:25](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/bugfix/mod.rs#L25) |
| `StmtInjector` | `VisitMutHook` | [common/statement_injector.rs:84](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/common/statement_injector.rs#L84) |
| `VarDeclarations` | `VisitMutHook` | [common/var_declarations.rs:49](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/common/var_declarations.rs#L49) |
| `DecoratorPass` | `VisitMutHook` | [decorators/mod.rs:17](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/decorators/mod.rs#L17) |
| `DuplicateKeysPass` | `VisitMutHook` | [es2015/duplicate_keys.rs:49](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/duplicate_keys.rs#L49) |
| `FunctionNamePass` | `VisitMutHook` | [es2015/function_name.rs:52](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/function_name.rs#L52) |
| `InstanceOfPass` | `VisitMutHook` | [es2015/instanceof.rs:52](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/instanceof.rs#L52) |
| `ShorthandPass` | `VisitMutHook` | [es2015/shorthand.rs:45](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/shorthand.rs#L45) |
| `StickyRegexPass` | `VisitMutHook` | [es2015/sticky_regex.rs:35](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/sticky_regex.rs#L35) |
| `TypeofSymbolPass` | `VisitMutHook` | [es2015/typeof_symbol.rs:49](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2015/typeof_symbol.rs#L49) |
| `ExponentiationOperatorPass` | `VisitMutHook` | [es2016/exponentiation_operator.rs:46](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2016/exponentiation_operator.rs#L46) |
| `AsyncToGeneratorPass` | `VisitMutHook` | [es2017/async_to_generator.rs:76](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L76) |
| `ObjectRestSpreadPass` | `VisitMutHook` | [es2018/object_rest_spread.rs:39](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L39) |
| `OptionalCatchBindingPass` | `VisitMutHook` | [es2019/optional_catch_binding.rs:28](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2019/optional_catch_binding.rs#L28) |
| `OptionalCatchBindingPass` | `VisitMut` | [es2019/optional_catch_binding.rs:34](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2019/optional_catch_binding.rs#L34) |
| `ExportNamespaceFromPass` | `VisitMutHook` | [es2020/export_namespace_from.rs:129](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2020/export_namespace_from.rs#L129) |
| `ExportNamespaceFromPass` | `VisitMut` | [es2020/export_namespace_from.rs:139](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2020/export_namespace_from.rs#L139) |
| `NullishCoalescingPass` | `VisitMutHook` | [es2020/nullish_coalescing.rs:56](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2020/nullish_coalescing.rs#L56) |
| `LogicalAssignmentOperatorsPass` | `VisitMutHook` | [es2021/logical_assignment_operators.rs:51](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2021/logical_assignment_operators.rs#L51) |
| `ClassStaticBlock` | `VisitMutHook` | [es2022/class_static_block.rs:174](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/class_static_block.rs#L174) |
| `PrivatePropertyInObjectFastPass` | `Pass` | [es2022/private_property_in_object.rs:57](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L57) |
| `PrivatePropertyInObjectPass` | `VisitMutHook` | [es2022/private_property_in_object.rs:244](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L244) |
| `PrivateInFinder` | `Visit` | [es2022/private_property_in_object.rs:599](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L599) |
| `ClassAnalyzer<'_>` | `Visit` | [es2022/private_property_in_object.rs:629](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2022/private_property_in_object.rs#L629) |
| `Es2026Pass` | `VisitMutHook` | [es2026/mod.rs:25](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es2026/mod.rs#L25) |
| `MemberExprLitHook` | `VisitMutHook` | [es3/member_expr_lits.rs:32](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es3/member_expr_lits.rs#L32) |
| `PropertyLiteralHook` | `VisitMutHook` | [es3/prop_lits.rs:42](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es3/prop_lits.rs#L42) |
| `ReservedWordHook` | `VisitMutHook` | [es3/reserved_word.rs:51](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/es3/reserved_word.rs#L51) |
| `OrderedChain<A, B>` | `VisitMutHook` | [hook_utils.rs:28](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/hook_utils.rs#L28) |
| `NoopHook` | `VisitMutHook` | [hook_utils.rs:139](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/hook_utils.rs#L139) |
| `JsxPass` | `VisitMutHook` | [jsx/mod.rs:17](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/jsx/mod.rs#L17) |
| `OptionalPass<P>` | `Pass` | [lib.rs:103](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/lib.rs#L103) |
| `RegexpPass` | `VisitMutHook` | [regexp.rs:274](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/regexp.rs#L274) |
| `RegexpPass` | `VisitMut` | [regexp.rs:280](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/regexp.rs#L280) |
| `TypeScriptPass` | `VisitMutHook` | [typescript/mod.rs:17](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transformer/src/typescript/mod.rs#L17) |

### swc_ecma_transforms

Reexports only; declarations belong to the crates below.

### swc_ecma_transforms_base

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `fixer` | pub | `impl '_ + Pass + VisitMut` | [fixer.rs:21](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/fixer.rs#L21) |
| `paren_remover` | pub | `impl '_ + Pass + VisitMut` | [fixer.rs:33](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/fixer.rs#L33) |
| `inject_helpers` | pub | `impl Pass + VisitMut` | [helpers/mod.rs:151](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/helpers/mod.rs#L151) |
| `hygiene` | pub | `impl Pass + VisitMut` | [hygiene/mod.rs:38](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/hygiene/mod.rs#L38) |
| `hygiene_with_config` | pub | `impl 'static + Pass + VisitMut` | [hygiene/mod.rs:68](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/hygiene/mod.rs#L68) |
| `rename` | pub | `impl '_ + Pass + VisitMut` | [rename/mod.rs:57](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/mod.rs#L57) |
| `rename_with_config` | pub | `impl '_ + Pass + VisitMut` | [rename/mod.rs:61](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/mod.rs#L61) |
| `renamer` | pub | `impl Pass + VisitMut` | [rename/mod.rs:72](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/mod.rs#L72) |
| `renamer_keep_contexts` | pub | `impl Pass + VisitMut` | [rename/mod.rs:91](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/mod.rs#L91) |
| `resolver` | pub | `impl 'static + Pass + VisitMut` | [resolver/mod.rs:136](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L136) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `Fixer<'_>` | `VisitMut` | [fixer.rs:159](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/fixer.rs#L159) |
| `InjectHelpers` | `VisitMut` | [helpers/mod.rs:340](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/helpers/mod.rs#L340) |
| `Marker` | `VisitMut` | [helpers/mod.rs:382](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/helpers/mod.rs#L382) |
| `HygieneRemover` | `VisitMut` | [hygiene/mod.rs:96](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/hygiene/mod.rs#L96) |
| `T` | `ParVisit` | [perf.rs:46](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/perf.rs#L46) |
| `T` | `ParVisitMut` | [perf.rs:85](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/perf.rs#L85) |
| `T` | `ParFold` | [perf.rs:125](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/perf.rs#L125) |
| `T` | `ParVisit` | [perf.rs:189](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/perf.rs#L189) |
| `T` | `ParVisitMut` | [perf.rs:204](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/perf.rs#L204) |
| `T` | `ParFold` | [perf.rs:219](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/perf.rs#L219) |
| `AnalyzerAndCollector` | `Visit` | [rename/analyer_and_collector.rs:83](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/analyer_and_collector.rs#L83) |
| `EvalFinder` | `Visit` | [rename/eval.rs:23](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/eval.rs#L23) |
| `RenamePass<R, V>` | `VisitMut` | [rename/mod.rs:363](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/mod.rs#L363) |
| `Operator<'_, V>` | `VisitMut` | [rename/ops.rs:97](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/ops.rs#L97) |
| `VarFolder<'_, '_, V>` | `VisitMut` | [rename/ops.rs:613](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/rename/ops.rs#L613) |
| `Resolver<'_>` | `VisitMut` | [resolver/mod.rs:530](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L530) |
| `Hoister<'_, '_>` | `VisitMut` | [resolver/mod.rs:1810](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1810) |
| `BindingResolver<'_>` | `VisitMut` | [resolver/typescript/bindings.rs:11](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/resolver/typescript/bindings.rs#L11) |
| `NamespaceResolver` | `VisitMut` | [resolver/typescript.rs:120](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_base/src/resolver/typescript.rs#L120) |

### swc_ecma_transforms_classes

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `SuperFieldAccessFolder<'_>` | `VisitMut` | [super_field.rs:66](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_classes/src/super_field.rs#L66) |
| `MovedStaticSuperFieldAccessFolder<'_>` | `VisitMut` | [super_field.rs:192](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_classes/src/super_field.rs#L192) |

### swc_ecma_transforms_compat

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `class_fields_use_set` | pub | `impl Pass` | [class_fields_use_set.rs:87](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L87) |
| `reserved_words` | pub | `impl 'static + Pass + VisitMut` | [reserved_words.rs:8](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_compat/src/reserved_words.rs#L8) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `ClassFieldsUseSet` | `VisitMut` | [class_fields_use_set.rs:96](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L96) |
| `FieldsHandler` | `VisitMut` | [class_fields_use_set.rs:167](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L167) |
| `ComputedFieldsHandler` | `VisitMut` | [class_fields_use_set.rs:274](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L274) |
| `EsReservedWord` | `VisitMut` | [reserved_words.rs:23](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_compat/src/reserved_words.rs#L23) |

### swc_ecma_transforms_module

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `amd` | pub | `impl Pass` | [amd.rs:58](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/amd.rs#L58) |
| `common_js` | pub | `impl Pass` | [common_js.rs:39](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/common_js.rs#L39) |
| `import_analyzer` | pub | `impl Pass` | [import_analysis.rs:12](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/import_analysis.rs#L12) |
| `swc_import_rewriter` | pub | `impl Pass` | [rewriter/import_rewriter_swc.rs:11](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/rewriter/import_rewriter_swc.rs#L11) |
| `typescript_import_rewriter` | pub | `impl Pass` | [rewriter/import_rewriter_typescript.rs:14](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/rewriter/import_rewriter_typescript.rs#L14) |
| `system_js` | pub | `impl Pass` | [system_js.rs:19](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/system_js.rs#L19) |
| `umd` | pub | `impl Pass` | [umd.rs:36](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/umd.rs#L36) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `Amd<C>` | `VisitMut` | [amd.rs:120](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/amd.rs#L120) |
| `Cjs` | `VisitMut` | [common_js.rs:66](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/common_js.rs#L66) |
| `ImportAnalyzer` | `VisitMut` | [import_analysis.rs:30](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/import_analysis.rs#L30) |
| `ImportAnalyzer` | `Visit` | [import_analysis.rs:38](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/import_analysis.rs#L38) |
| `Rewriter` | `VisitMut` | [rewriter/import_rewriter_swc.rs:20](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/rewriter/import_rewriter_swc.rs#L20) |
| `Rewriter` | `VisitMut` | [rewriter/import_rewriter_typescript.rs:22](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/rewriter/import_rewriter_typescript.rs#L22) |
| `SpecialRefRewriter<'_>` | `VisitMut` | [system_js/rewrite.rs:64](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L64) |
| `ExportBindingRewriter` | `VisitMut` | [system_js/rewrite.rs:179](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L179) |
| `SystemJs` | `VisitMut` | [system_js.rs:33](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/system_js.rs#L33) |
| `TopLevelThis` | `VisitMut` | [top_level_this.rs:21](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/top_level_this.rs#L21) |
| `Umd` | `VisitMut` | [umd.rs:74](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_module/src/umd.rs#L74) |

### swc_ecma_transforms_optimization

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `const_modules` | pub | `impl Pass` | [const_modules.rs:22](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/const_modules.rs#L22) |
| `inline_globals` | pub | `impl Pass` | [inline_globals.rs:19](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/inline_globals.rs#L19) |
| `json_parse` | pub | `impl Pass` | [json_parse.rs:37](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/json_parse.rs#L37) |
| `dead_branch_remover` | pub | `impl Repeated + Pass + CompilerPass + VisitMut + 'static` | [simplify/branch/mod.rs:27](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/branch/mod.rs#L27) |
| `constant_propagation` | pub | `impl 'static + Pass + VisitMut` | [simplify/const_propagation.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/const_propagation.rs#L9) |
| `dce` | pub | `impl Pass + VisitMut + Repeated + CompilerPass` | [simplify/dce/mod.rs:26](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/dce/mod.rs#L26) |
| `expr_simplifier` | pub | `impl Repeated + Pass + CompilerPass + VisitMut + 'static` | [simplify/expr/mod.rs:47](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/expr/mod.rs#L47) |
| `inlining` | pub | `impl 'static + Repeated + CompilerPass + Pass + VisitMut` | [simplify/inlining/mod.rs:37](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/inlining/mod.rs#L37) |
| `simplifier` | pub | `impl CompilerPass + Pass + Repeated` | [simplify/mod.rs:28](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/mod.rs#L28) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `ConstModules` | `VisitMut` | [const_modules.rs:95](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/const_modules.rs#L95) |
| `AssertValid` | `Visit` | [debug.rs:30](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/debug.rs#L30) |
| `InlineGlobals` | `VisitMut` | [inline_globals.rs:59](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/inline_globals.rs#L59) |
| `JsonParse` | `VisitMut` | [json_parse.rs:61](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/json_parse.rs#L61) |
| `Remover` | `CompilerPass` | [simplify/branch/mod.rs:42](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/branch/mod.rs#L42) |
| `Remover` | `Repeated` | [simplify/branch/mod.rs:48](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/branch/mod.rs#L48) |
| `Remover` | `VisitMut` | [simplify/branch/mod.rs:74](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/branch/mod.rs#L74) |
| `Visitor` | `Visit` | [simplify/branch/mod.rs:1942](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/branch/mod.rs#L1942) |
| `ConstPropagation<'_>` | `VisitMut` | [simplify/const_propagation.rs:41](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/const_propagation.rs#L41) |
| `TreeShaker` | `CompilerPass` | [simplify/dce/mod.rs:102](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/dce/mod.rs#L102) |
| `Dropper<'a>` | `Visit` | [simplify/dce/mod.rs:281](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/dce/mod.rs#L281) |
| `Analyzer<'_>` | `Visit` | [simplify/dce/mod.rs:612](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/dce/mod.rs#L612) |
| `TreeShaker` | `Repeated` | [simplify/dce/mod.rs:908](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/dce/mod.rs#L908) |
| `RootHoistedVars` | `Visit` | [simplify/dce/mod.rs:1061](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/dce/mod.rs#L1061) |
| `TreeShaker` | `VisitMut` | [simplify/dce/mod.rs:1080](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/dce/mod.rs#L1080) |
| `SimplifyExpr` | `CompilerPass` | [simplify/expr/mod.rs:87](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/expr/mod.rs#L87) |
| `SimplifyExpr` | `Repeated` | [simplify/expr/mod.rs:93](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/expr/mod.rs#L93) |
| `SimplifyExpr` | `VisitMut` | [simplify/expr/mod.rs:103](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/expr/mod.rs#L103) |
| `Inlining<'_>` | `CompilerPass` | [simplify/inlining/mod.rs:57](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/inlining/mod.rs#L57) |
| `Inlining<'_>` | `Repeated` | [simplify/inlining/mod.rs:63](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/inlining/mod.rs#L63) |
| `Inlining<'_>` | `VisitMut` | [simplify/inlining/mod.rs:106](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/inlining/mod.rs#L106) |
| `IdentListVisitor<'_, '_>` | `Visit` | [simplify/inlining/mod.rs:758](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/inlining/mod.rs#L758) |
| `WriteVisitor<'_, '_>` | `Visit` | [simplify/inlining/mod.rs:773](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_optimization/src/simplify/inlining/mod.rs#L773) |

### swc_ecma_transforms_proposal

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `decorator_2022_03` | pub | `impl Pass` | [decorator_2022_03.rs:5](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_2022_03.rs#L5) |
| `decorator_2023_11` | pub | `impl Pass` | [decorator_2023_11.rs:5](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_2023_11.rs#L5) |
| `decorator_impl` | restricted | `impl Pass` | [decorator_impl.rs:20](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L20) |
| `decorators` | pub | `impl Pass` | [decorators/mod.rs:60](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorators/mod.rs#L60) |
| `explicit_resource_management` | pub | `impl Pass` | [explicit_resource_management.rs:10](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/explicit_resource_management.rs#L10) |
| `export_default_from` | pub | `impl Pass` | [export_default_from.rs:6](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/export_default_from.rs#L6) |
| `import_attributes` | pub | `impl Pass` | [import_attributes.rs:4](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/import_attributes.rs#L4) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `PrivateNameFinder` | `Visit` | [decorator_impl.rs:110](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L110) |
| `YieldAwaitFinder` | `Visit` | [decorator_impl.rs:123](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L123) |
| `InstancePrivateNameFinder<'_>` | `Visit` | [decorator_impl.rs:140](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L140) |
| `CurrentClassNameFinder` | `Visit` | [decorator_impl.rs:155](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L155) |
| `MovedStaticSuperFinder` | `Visit` | [decorator_impl.rs:170](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L170) |
| `DecoratorPass` | `VisitMut` | [decorator_impl.rs:2287](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorator_impl.rs#L2287) |
| `ParamMetadata` | `VisitMut` | [decorators/legacy/metadata.rs:19](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L19) |
| `Metadata<'_>` | `VisitMut` | [decorators/legacy/metadata.rs:139](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorators/legacy/metadata.rs#L139) |
| `TscDecorator` | `Visit` | [decorators/legacy/mod.rs:219](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L219) |
| `TscDecorator` | `VisitMut` | [decorators/legacy/mod.rs:264](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorators/legacy/mod.rs#L264) |
| `Decorators` | `Fold` | [decorators/mod.rs:92](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorators/mod.rs#L92) |
| `DecoratorFinder` | `Visit` | [decorators/mod.rs:672](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/decorators/mod.rs#L672) |
| `ExplicitResourceManagement` | `VisitMut` | [explicit_resource_management.rs:366](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/explicit_resource_management.rs#L366) |
| `ExportDefaultFrom` | `Pass` | [export_default_from.rs:12](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/export_default_from.rs#L12) |
| `ImportAssertions` | `VisitMut` | [import_attributes.rs:10](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_proposal/src/import_attributes.rs#L10) |

### swc_ecma_transforms_react

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `hook` | pub | `impl VisitMutHook<()>` | [display_name.rs:12](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/display_name.rs#L12) |
| `hook` | pub | `impl VisitMutHook<()>` | [jsx/mod.rs:212](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L212) |
| `jsx` | pub | `impl Pass + VisitMut` | [jsx/mod.rs:256](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L256) |
| `hook` | pub | `impl VisitMutHook<()>` | [jsx_self.rs:9](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/jsx_self.rs#L9) |
| `hook` | pub | `impl VisitMutHook<()>` | [jsx_src.rs:7](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/jsx_src.rs#L7) |
| `display_name` | pub | `impl Pass` | [lib.rs:22](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/lib.rs#L22) |
| `jsx_self` | pub | `impl Pass` | [lib.rs:29](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/lib.rs#L29) |
| `jsx_src` | pub | `impl Pass` | [lib.rs:36](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/lib.rs#L36) |
| `pure_annotations` | pub | `impl Pass` | [lib.rs:43](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/lib.rs#L43) |
| `refresh` | pub | `impl Pass` | [lib.rs:53](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/lib.rs#L53) |
| `react` | pub | `impl Pass` | [lib.rs:79](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/lib.rs#L79) |
| `hook` | pub | `impl VisitMutHook<()>` | [pure_annotations.rs:13](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/pure_annotations.rs#L13) |
| `hook` | pub | `impl VisitMutHook<()>` | [refresh/mod.rs:49](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L49) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `DisplayName` | `VisitMutHook` | [display_name.rs:18](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/display_name.rs#L18) |
| `Folder` | `VisitMut` | [display_name.rs:117](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/display_name.rs#L117) |
| `Jsx<C>` | `VisitMutHook` | [jsx/mod.rs:1207](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1207) |
| `JsxSelf` | `VisitMutHook` | [jsx_self.rs:95](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/jsx_self.rs#L95) |
| `JsxSrc` | `VisitMutHook` | [jsx_src.rs:17](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/jsx_src.rs#L17) |
| `PureAnnotations<C>` | `VisitMutHook` | [pure_annotations.rs:31](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/pure_annotations.rs#L31) |
| `HookRegister<'_>` | `VisitMut` | [refresh/hook.rs:248](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L248) |
| `HookCollector<'_>` | `Visit` | [refresh/hook.rs:541](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/refresh/hook.rs#L541) |
| `Refresh<C>` | `VisitMutHook` | [refresh/mod.rs:233](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L233) |
| `SpanVisitor<'_, C>` | `Visit` | [refresh/mod.rs:256](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L256) |
| `UsedInJsx` | `Visit` | [refresh/util.rs:95](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_react/src/refresh/util.rs#L95) |

### swc_ecma_transforms_typescript

Factories and builders:

| Function | Declared visibility | Return type | Source |
| --- | --- | --- | --- |
| `transform` | pub | `impl Pass + 'a` | [transform.rs:168](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/transform.rs#L168) |
| `typescript` | pub | `impl Pass` | [typescript.rs:19](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/typescript.rs#L19) |
| `strip` | pub | `impl Pass` | [typescript.rs:30](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/typescript.rs#L30) |
| `tsx` | pub | `impl Pass` | [typescript.rs:140](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/typescript.rs#L140) |

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `DefinitionCollector<'_>` | `VisitMut` | [semantic/enums/definitions.rs:131](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/semantic/enums/definitions.rs#L131) |
| `UsageCollector<'_>` | `VisitMut` | [semantic/usage.rs:221](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/semantic/usage.rs#L221) |
| `SemanticAnalyzer` | `Visit` | [semantic.rs:427](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/semantic.rs#L427) |
| `Transform<'_>` | `VisitMut` | [transform.rs:208](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/transform.rs#L208) |
| `AwaitBindingCollector` | `Visit` | [transform.rs:1085](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/transform.rs#L1085) |
| `TypeScript` | `Pass` | [typescript.rs:42](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/typescript.rs#L42) |
| `TypeScriptReact<C>` | `VisitMut` | [typescript.rs:187](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_transforms_typescript/src/typescript.rs#L187) |

### swc_ecma_utils

Explicit implementations:

| Type | Trait | Source |
| --- | --- | --- |
| `Injector` | `VisitMut` | [constructor.rs:38](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/constructor.rs#L38) |
| `FnEnvHoister` | `VisitMut` | [function/fn_env_hoister.rs:315](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/function/fn_env_hoister.rs#L315) |
| `InitThis<'_>` | `VisitMut` | [function/fn_env_hoister.rs:599](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/function/fn_env_hoister.rs#L599) |
| `ThisVisitor` | `Visit` | [lib.rs:77](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L77) |
| `IdentRefFinder<'_>` | `Visit` | [lib.rs:129](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L129) |
| `ArgumentsFinder` | `Visit` | [lib.rs:158](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L158) |
| `Hoister` | `Visit` | [lib.rs:665](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L665) |
| `RestPatVisitor` | `Visit` | [lib.rs:1135](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L1135) |
| `LiteralVisitor` | `Visit` | [lib.rs:1172](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L1172) |
| `DestructuringFinder<I>` | `Visit` | [lib.rs:1734](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L1734) |
| `BindingIdentifierVisitor<F>` | `Visit` | [lib.rs:1782](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L1782) |
| `DropSpan` | `Pass` | [lib.rs:1818](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L1818) |
| `DropSpan` | `VisitMut` | [lib.rs:1824](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L1824) |
| `IdentUsageFinder<'_>` | `Visit` | [lib.rs:1849](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L1849) |
| `IdentReplacer<'_>` | `VisitMut` | [lib.rs:2169](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L2169) |
| `BindingCollector<I>` | `Visit` | [lib.rs:2223](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L2223) |
| `TopLevelAwait` | `Visit` | [lib.rs:2361](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L2361) |
| `Remapper<'_>` | `VisitMut` | [lib.rs:2459](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L2459) |
| `IdentRenamer<'_>` | `VisitMut` | [lib.rs:2480](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L2480) |
| `RefRewriter<T>` | `VisitMut` | [lib.rs:2713](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/lib.rs#L2713) |
| `TsBindingCollector<O, TYPE_OWNERS>` | `Visit` | [ts_bindings/collect.rs:197](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/ts_bindings/collect.rs#L197) |
| `VarCollector<'_, I>` | `Visit` | [var.rs:12](https://github.com/swc-project/swc/blob/c682b13a8ea7372203970d315df38b529ce5b7f0/crates/swc_ecma_utils/src/var.rs#L12) |
