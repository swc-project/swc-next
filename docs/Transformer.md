# Transformer Implementation

This document describes the transformer architecture in
`crates/swc_ecma_transformer`, its integration with Semantic, and the conventions
for implementing transforms. Paths below are relative to the repository root.
See [AGENTS.md](../AGENTS.md) for repository-wide development and testing commands.

## Object relationships

```text
Transformer                              TraverseMutCtx<TransformerState>
| implements TraverseMut                 |
+-- ES2015 transforms                     +-- ast: &mut Ast
|   +-- ArrowFunctions, ...               +-- state: TransformerState
+-- ES2022 transforms                     |
    +-- StaticBlocks                         |
                                             +-- path: Vec<NodeId>
                                             +-- semantic: &mut Semantic
Visitor callback receives TraverseMutCtx     +-- assumptions
             |                               +-- generated_bindings
             | ctx.mutate()                  +-- helpers
             v                               +-- statement_injector
          Mutation
          +-- borrows ast mutably
          +-- borrows state mutably
          +-- borrows transform path
```

- `Transformer` owns the concrete transform implementations and dispatches visitor
  callbacks. Each transform keeps its own traversal state, such as arrow capture
  frames or pending static-block closure state.
- `TraverseMutCtx` belongs to `swc_ecma_ast`. It carries AST and the generic state value. The generic traversal does not implement transform
  replacement policies or depend on Semantic.
- `TransformerState` holds shared configuration, Semantic's mutable borrow,
  generated bindings, helper imports, and the statement injection queue. It
  stores the traversal path through the generic node hooks and does not own another AST.
- `Mutate` is a trait defined in the transform crate and implemented for
  `TraverseMutCtx<TransformerState>`. Its `mutate()` method returns a temporary
  `Mutation` borrowing the existing data.
- `Mutation` provides read access through `ast()` and `semantic()`, and methods
  that coordinate AST edits with Semantic updates. It is not a second traversal
  engine or a snapshot of the tree.

## Execution flow

```text
TransformerOptions
  semantic_features() selects required SemanticFeatureFlags (bitflags)
             |
             v
Semantic analysis
  collect scope, binding, reference, and optional transform information
             |
             v
Transformer::transform
  check required features; create TransformerState and TraverseMutCtx
  normalize through Mutation and synchronize affected export metadata
             |
             v
Program.traverse_mut_with(Transformer, TraverseMutCtx)
  enter / exit callbacks dispatch to concrete transforms
             |
             v
Mutation operations
  allocate/register new nodes and bindings; synchronize replacements/moves
  prepare helper imports on first use; queue statements in destination scopes
             |
             v
Exit the corresponding statement container
  attach queued statements in a batch
  inject prepared helper imports at Program exit
             |
             v
Mutation::finish
  debug-check empty queues/caches and Semantic invariants
```

The statement queue defers attachment to AST lists, not binding allocation or
reference registration. Ordinary replacements happen during the visitor. Flush
is per destination container; it is not one global replay of all AST mutations.

Semantic always collects node evaluation scopes in an arena-backed index. Transforms
request feature flags only for optional information beyond these scope indexes.

## Implementing a transform

1. **Identify the needed information.** Reuse Semantic's existing scopes,
   bindings, references, class metadata, and name allocation. Collect additional
   program information in the existing semantic traversal. Gate optional
   collection with narrowly scoped `SemanticFeatureFlags` and request those flags
   from `TransformerOptions::semantic_features()`. Avoid separate information
   collection walks in the transform.
2. **Keep lowering in the visitor.** Follow the existing ES2015/ES2022 dispatch
   pattern. Borrow `Mutation` in callbacks and pass it to the concrete transform.
   Keep transform-specific capture/layout logic with that transform. Add shared
   state or Mutation methods only when their responsibility is actually shared.
3. **Choose the edit's scope and ownership before building nodes.** An expression's
   evaluation scope can differ from the surrounding node's own scope. Reuse
   Semantic's scope queries rather than reconstructing lexical environments from
   syntax. Preserve existing node, binding, and reference identities when moving
   or wrapping them.
4. **Apply edits through Mutation.** Extend it with a concrete atomic operation
   when needed, including the corresponding Semantic update. Do not perform raw
   AST edits followed by separate Semantic repairs in a visitor. Although the
   generic `TraverseMutCtx.ast` field is public, transforms use the Mutation boundary.

### Existing mutation contracts

- `add_node(scope, |ast| ...)` allocates nodes and registers the new allocation
  range. The closure constructs new AST nodes; it must not modify existing edges.
  Use dedicated binding/reference and scope-owner APIs for those identities.
- `generate_binding` returns a `GeneratedBinding` containing its declaration and
  symbol. Register it on first use, then reuse it through `generated_bindings`.
  Use `add_scope_node` to create or transfer a scope owner.
- `replace_current_expr` supports leaf replacement. `wrap_current_expr` retains
  the old expression as a direct child. Use
  `replace_current_expr_preserving_children` only after preserving the children
  and transferring the old owner's scope as required by its contract.
- `filter_map_class_elements` distinguishes `Unchanged`, `Update`, and `Delete`.
  It compacts node-ID slots in place and synchronizes each edit. Materialize or
  move retained contents before deleting their old class-element attachment.
- Use `inject_statement`, `inject_var`, or the after-insertion methods for
  deferred statements. Prepare them in their destination scopes and let the
  existing visitor lifecycle flush them. `insert_statement` is an immediate edit;
  do not confuse it with queued injection.

Detached AST shells remain in arena storage until the AST is destroyed. Keeping
storage alive does not justify stale semantic indexes for detached references or
scope owners. Update the affected semantic identities without adding subtree
snapshots, repeated subtree scans, or a separate node-retirement lifecycle.

## Helpers

Helpers are currently external only. `Mutation::helper_callee` allocates the name,
registers its binding, and prepares the import on first use. `Helpers` caches that
prepared declaration; `inject_helpers` feeds it into the statement injector at
Program exit. Repeated uses must reuse the binding and import.

Module output uses `import`; script output uses `require`. Currently
`setFunctionName` comes from `@babel/runtime/helpers/setFunctionName`, while
`toPropertyKey` uses `@swc/helpers`. Verify the actual runtime export when adding a
helper. Inline helpers and a swc-next-owned helper library remain an explicit
TODO; do not silently reintroduce inline implementations for ordinary transforms.

## Source map

| Responsibility | File |
| --- | --- |
| Entry point, feature requirements, visitor dispatch | [crates/swc_ecma_transformer/src/lib.rs](../crates/swc_ecma_transformer/src/lib.rs) |
| Shared transform state | [crates/swc_ecma_transformer/src/state.rs](../crates/swc_ecma_transformer/src/state.rs) |
| Mutate trait, borrowed Mutation, names and helpers | [crates/swc_ecma_transformer/src/context/mod.rs](../crates/swc_ecma_transformer/src/context/mod.rs) |
| Allocation and registration | [crates/swc_ecma_transformer/src/context/construction.rs](../crates/swc_ecma_transformer/src/context/construction.rs) |
| Expression replacement contracts | [crates/swc_ecma_transformer/src/context/expressions.rs](../crates/swc_ecma_transformer/src/context/expressions.rs) |
| Statement/class edits and scope operations | [crates/swc_ecma_transformer/src/context/mutation.rs](../crates/swc_ecma_transformer/src/context/mutation.rs), [crates/swc_ecma_transformer/src/context/scopes.rs](../crates/swc_ecma_transformer/src/context/scopes.rs) |
| Deferred statement attachment | [crates/swc_ecma_transformer/src/context/statement_injector.rs](../crates/swc_ecma_transformer/src/context/statement_injector.rs) |
| External helper cache and imports | [crates/swc_ecma_transformer/src/helpers.rs](../crates/swc_ecma_transformer/src/helpers.rs), [crates/swc_ecma_transformer/src/helpers/builder.rs](../crates/swc_ecma_transformer/src/helpers/builder.rs) |
| Concrete lowering examples | [crates/swc_ecma_transformer/src/es2015/arrow.rs](../crates/swc_ecma_transformer/src/es2015/arrow.rs), [crates/swc_ecma_transformer/src/es2022/static_blocks.rs](../crates/swc_ecma_transformer/src/es2022/static_blocks.rs) |
| Semantic collection and synchronization | [crates/swc_ecma_semantic/src/builder.rs](../crates/swc_ecma_semantic/src/builder.rs), [crates/swc_ecma_semantic/src/class_semantic.rs](../crates/swc_ecma_semantic/src/class_semantic.rs), [crates/swc_ecma_semantic/src/mutation.rs](../crates/swc_ecma_semantic/src/mutation.rs) |
| Generic traversal context | [crates/swc_ecma_ast/src/visit.rs](../crates/swc_ecma_ast/src/visit.rs) |

## Validation

Keep test code under `crates/swc_ecma_transformer/tests/`, mirroring the source
module layout:

```text
tests/
|-- lib.rs                 Whole-transform tests for src/lib.rs
|-- helpers.rs             Tests for src/helpers.rs
`-- context/
    |-- mutation.rs        Tests for src/context/mutation.rs
    `-- support.rs         Shared context test utilities
```

Source modules load these files with `#[cfg(test)]` and `#[path]`, retaining unit
test access to private implementation details. The crate sets `autotests = false`
so Cargo does not compile these files as separate integration-test crates. Follow
this pattern for additional module tests without widening production visibility.

Reuse conformance fixtures for covered behavior; add focused tests for new
semantic invariants or observable behavior, following `AGENTS.md`. Scope-sensitive
changes should compare the transformed Semantic with fresh analysis, following
`assert_reference_scopes` in `crates/swc_ecma_transformer/tests/lib.rs`.

Run relevant Rust tests and the repository's required formatting/lint checks.
For static-block/helper changes, run both `babel_transform` and `swc_transform`;
see [AGENTS.md](../AGENTS.md) for commands and fixture setup. Inspect
`tasks/testsuite/snapshots/` and `tasks/testsuite/outputs/<runner>/differences.md`:
a successful runner exit does not mean every fixture matches.

Separate helper spelling/import differences from semantic differences. When
binding, evaluation order, or constructor behavior is in question, compare
execution with native JavaScript and the requested upstream configuration before
changing behavior merely to match expected text.
