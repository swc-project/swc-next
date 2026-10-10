# Transformer Implementation

This document describes the transformer architecture in
`crates/swc_ecma_transformer`, its integration with Semantic, and the conventions
for implementing transforms. Paths below are relative to the repository root.
See [AGENTS.md](../AGENTS.md) for repository-wide development and testing commands.

## Object relationships

```text
Transformer                              TraverseMutCtx<TransformerState>
| implements TraverseMut                 |
+-- ES2022 transforms                     +-- ast: &mut Ast
    +-- StaticBlocks                     +-- state: TransformerState
                                             |
                                             +-- path: Vec<NodeId>
                                             +-- semantic: &mut Semantic
Visitor callback receives TraverseMutCtx     +-- assumptions
             |                               +-- generated_bindings
             |                               +-- used_names
             |                               +-- analysis: TransformAnalysis
             | ctx.mutate()                  +-- helpers
             v                               +-- statement_injector
          Mutation
          +-- borrows ast mutably
          +-- borrows state mutably
          +-- borrows transform path
```

- `Transformer` owns the concrete transform implementations and dispatches visitor
  callbacks. Each transform keeps its own traversal state, such as pending
  static-block closure state.
- `TraverseMutCtx` belongs to `swc_ecma_ast`. It carries AST and the generic state value. The generic traversal does not implement transform
  replacement policies or depend on Semantic.
- `TransformerState` holds shared configuration, Semantic's mutable borrow,
  optional transform analysis,
  generated bindings, used names, helper imports, and the statement injection
  queue. It stores the traversal path through the generic node hooks and does
  not own another AST.
- `Mutate` is a trait defined in the transform crate and implemented for
  `TraverseMutCtx<TransformerState>`. Its `mutate()` method returns a temporary
  `Mutation` borrowing the existing data.
- `Mutation` provides read access through `ast()` and `semantic()`, and methods
  that coordinate AST edits with Semantic updates. It is not a second traversal
  engine or a snapshot of the tree.

## Execution flow

```text
TransformerOptions
  transform_features() selects enabled passes (bitflags)
             |
             v
Normalizer::normalize
  normalize parser AST shapes before building semantic indexes
             |
             v
Semantic analysis
  resolve names; collect scopes, bindings, declarations, and references
             |
             v
Transformer::transform
  collect optional TransformAnalysis for enabled passes
  create TransformerState and TraverseMutCtx
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

Callers must normalize the AST before `analyze`, then pass that same AST and
Semantic to `Transformer::transform`. Normalization removes parentheses and
flattens associative containers, so building Semantic afterward records the final
scope owners and export expression roots directly.

The statement queue defers attachment to AST lists, not binding allocation or
reference registration. Mutable enter and exit hooks return `TraverseAction<T>`:
`Continue`, `Skip`, `Stop`, `ReplaceContinue(T)`, `ReplaceSkip(T)`, or
`ReplaceStop(T)`. Untyped `enter_node`/`exit_node` hooks
return no action and always enclose each visit, including skipped, stopped, or
replaced visits, so path bookkeeping does not depend on typed exit hooks.

`Skip` ends the current node's remaining typed hooks, while siblings continue.
In enter it also skips children and exit; in exit the children were already
visited. `Stop` also stops remaining fields and list entries and propagates
outward. Already entered ancestors still run their exit hooks; `Continue` cannot
cancel a descendant's stop. If an ancestor exit requests a replacement while
unwinding a stop, the parent attaches it without revisiting it. Enum and concrete
hooks for the same node are one visit: a skip, stop, or replacement returned by
an inner hook ends that node's remaining hooks, rather than being merged with
an outer exit action.

All replacement variants end the old node's visit. The parent applies the typed
replacement before processing its disposition: `ReplaceContinue` restarts the
replacement from enter, `ReplaceSkip` continues with the next sibling, and
`ReplaceStop` stops. Returning a replacement always applies it, even when its
NodeId is unchanged. Return `Continue` to keep traversing the current node.
Scalar fields, optional fields, and list slots share the same attachment loop.
`TraverseMutWith` returns `TraverseAction<Self>` as the disposition for its
owning slot, consuming a local `Skip` as `Continue`. A direct root caller must
attach returned replacements through the root setter and restart traversal for
`ReplaceContinue`. Flush is per destination container, not one global replay of
all AST mutations.

`push_container` and `unshift_container` currently support `ClassBody.body`.
They edit the logical member list returned by `container_class_body` in
transformer state, including pending insertions. `container_flush` attaches
the list with one `add_sub_range` call and releases the cached handles. Unedited
lists keep their original ranges, and repeated write-back without further edits
does nothing. The class-body entry callback writes back before the generic
traversal reads the member range, so traversal sees the updated members. The exit
callback writes back any edits made while visiting children; `Mutation::finish`
checks that no containers remain pending. These operations
defer list attachment only; new scope owners, bindings and references must be
registered when nodes are prepared. Scope registration for the synthetic
named-evaluation static block remains unimplemented. Removal and replacement of
class members must update the same logical list so final attachment retains all
edits.

Semantic always collects node evaluation scopes in an arena-backed index. It has
no transform collection flags. Transformer preparation uses the enabled pass flags
to collect static-block comments and closure-lowering layouts in one optional
traversal. Private-name collision domains come from local class member lists.
Mutation initializes shared generated-name state on first use from Semantic's
symbols, binding declarations, and references, including unresolved references.
It reserves each generated name immediately and observes newly introduced names
so later transforms use the same collision domain. Class private-name allocation
belongs to TransformAnalysis's separate per-class collision domains.

## Implementing a transform

1. **Identify the needed information.** Reuse Semantic's existing scopes,
   bindings, declarations, and references; allocate names through Mutation.
   Keep additional information in the transformer. Use traversal state or local
   member lists when possible; combine necessary lookahead in TransformAnalysis's
   preparation traversal and gate it with enabled transform flags.
2. **Keep lowering in the visitor.** Follow the existing ES2022 dispatch
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
   AST edits followed by separate Semantic repairs in a visitor. Mutation also
   retires replaced static-block metadata and invalidates affected member layouts.
   Although the generic `TraverseMutCtx.ast` field is public, transforms use the
   Mutation boundary.

### Existing mutation contracts

- `inherits_comments(child, parent)` returns `child` and queues the parent's
  leading/trailing attachment offsets for inheritance, retaining the child's
  existing comments. `CommentState` lives in traversal state and composes queued
  transfers in call order. `Mutation::finish` applies them with one
  `Ast::retarget_comments` call, preserving source order and avoiding duplicate
  comment records. This is relocation for replaced parents: unlike Babel's
  shared comment objects, the flat AST gives each comment one attachment.
  Inner comments retain their child/token anchors because the AST has no
  separate inner-comment list.
- `add_node(scope, |ast| ...)` allocates nodes and registers the new allocation
  range. The closure constructs new AST nodes; it must not modify existing edges.
  Use dedicated binding/reference and scope-owner APIs for those identities.
- `generate_binding` returns a `GeneratedBinding` containing its declaration and
  symbol. Register it on first use, then reuse it through `generated_bindings`.
  Use `add_scope_node` to create or transfer a scope owner.
- Return `ReplaceContinue`, `ReplaceSkip`, or `ReplaceStop` from enter or exit
  to edit the current attachment. Prepare the replacement's nodes,
  scopes, bindings and references before returning it; traversal only applies
  the typed AST edge and does not allocate semantic identities. Static-block
  lowering returns `(class, call)` as `TraverseAction::ReplaceContinue` from
  `exit_expr`, preserving the original class's NodeId and semantic identities
  inside the sequence, with new nodes evaluated in the class's outside scope.
  Class declarations use `exit_decl`; the default-export kind hook forwards
  default-exported class declarations there because its Class variant bypasses
  the Decl traversal hook.
  Concrete hooks return their concrete type; use a union hook for replacements
  between different node kinds. Untyped hooks only maintain traversal state.
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

### Planned scope mutation contracts

`ScopeModel` declares index-only operations with `todo!()` bodies. Mutation's
AST-facing adapters live in `context/semantic.rs`; the planned scope operations
are not connected to transforms yet:

- `create_child_scope(parent, owner, kind, flags)` registers an allocated scope
  owner and returns its scope ID. Existing content is moved separately.
- `convert_scope(scope, owner, kind, flags)` reuses a scope under a replacement
  owner, preserving binding/reference identities while updating scope properties.
- `move_node_to_scope(node, from, to)` updates one evaluation mapping and its
  reference's scope, preserving reference identity and target.
- `move_binding(from, to, symbol)` moves the binding-table entry and declarations
  owned by `from`, preserving the symbol, references and other merged declarations.
  Declaration ownership is separate from node evaluation scope.
- `reparent_scope(scope, parent)` changes ancestry and refreshes affected hoist
  targets, preserving the scope's owner and contents.
- `set_scope_flags(scope, flags)` applies flags computed from syntax by Mutation.
- `remove_scope(scope)` marks an empty scope inactive and clears its detached
  owner's mapping; retained contents must have moved first.
- `clear_node_scope(node)` clears a detached syntax shell's mapping without
  visiting children retained elsewhere.

Mutation's `move_subtree_to_scope(root, from, to)` reads AST edges and composes
node moves and scope reparenting. Only nodes directly in `from` move; nested
scopes keep their internal mappings and bindings. Mutation coordinates any
syntax-dependent strictness and direct-eval changes. Semantic does not traverse
syntax to infer these updates.

Existing binding/reference registration also follows this boundary.
`Mutation::register_binding` reads the identifier's name and arena-lived text;
`ScopeModel::register_binding` receives the node ID, scope, name handle, text and
flags. `Mutation::register_reference` constructs a `Reference` record for
`ScopeModel::register_reference`. Mutation registers the full AST allocation
range; ScopeModel only ensures capacity through the supplied node ID. Neither
ScopeModel registration method takes `Ast` or reads node attributes.

For single-expression static blocks, move the expression, clear the discarded
statement shell and retire the empty static-block scope. For a whole block reused
as an arrow body, convert its scope and register new parameters/body nodes there.
For an additional wrapping arrow, create its scope and move each retained subtree
into it, preserving independent scopes belonging to the original blocks.

## Helpers

Helpers are currently external only. `Mutation::helper_callee` allocates the name,
registers its binding, and prepares the import on first use. `Helpers` caches that
prepared declaration; `inject_helpers` feeds it into the statement injector at
Program exit. Repeated uses must reuse the binding and import. Helper construction
and calls are methods on `Mutation`, implemented in `context/helpers.rs`; transforms
use `call_set_function_name` and `call_to_property_key` through that interface.

Module output uses `import`; script output uses `require`. Currently
`setFunctionName` comes from `@babel/runtime/helpers/setFunctionName`, while
`toPropertyKey` uses `@swc/helpers`. Verify the actual runtime export when adding a
helper. Inline helpers and a swc-next-owned helper library remain an explicit
TODO; do not silently reintroduce inline implementations for ordinary transforms.

## Source map

| Responsibility | File |
| --- | --- |
| Entry point, enabled passes, visitor dispatch | [crates/swc_ecma_transformer/src/lib.rs](../crates/swc_ecma_transformer/src/lib.rs) |
| Optional transform preparation | [crates/swc_ecma_transformer/src/analysis.rs](../crates/swc_ecma_transformer/src/analysis.rs) |
| Shared transform state | [crates/swc_ecma_transformer/src/state.rs](../crates/swc_ecma_transformer/src/state.rs) |
| Mutate trait, borrowed Mutation, and names | [crates/swc_ecma_transformer/src/context/mod.rs](../crates/swc_ecma_transformer/src/context/mod.rs) |
| Shared name allocation from Semantic indexes | [crates/swc_ecma_transformer/src/context/names.rs](../crates/swc_ecma_transformer/src/context/names.rs) |
| Generated-name collision domains | [crates/swc_ecma_transformer/src/used_names.rs](../crates/swc_ecma_transformer/src/used_names.rs) |
| Post-transform semantic invariant checks | [crates/swc_ecma_transformer/src/context/invariants.rs](../crates/swc_ecma_transformer/src/context/invariants.rs) |
| Allocation and registration | [crates/swc_ecma_transformer/src/context/construction.rs](../crates/swc_ecma_transformer/src/context/construction.rs) |
| Typed traversal actions and parent write-back | [crates/swc_ecma_ast/src/visit.rs](../crates/swc_ecma_ast/src/visit.rs), [tasks/ast_tools/src/generator/ast_visitor.rs](../tasks/ast_tools/src/generator/ast_visitor.rs) |
| Statement/class edits and scope operations | [crates/swc_ecma_transformer/src/context/mutation.rs](../crates/swc_ecma_transformer/src/context/mutation.rs), [crates/swc_ecma_transformer/src/context/scopes.rs](../crates/swc_ecma_transformer/src/context/scopes.rs) |
| Deferred statement attachment | [crates/swc_ecma_transformer/src/context/statement_injector.rs](../crates/swc_ecma_transformer/src/context/statement_injector.rs) |
| External helper cache and kinds | [crates/swc_ecma_transformer/src/helpers.rs](../crates/swc_ecma_transformer/src/helpers.rs) |
| Helper construction, imports, and calls | [crates/swc_ecma_transformer/src/context/helpers.rs](../crates/swc_ecma_transformer/src/context/helpers.rs) |
| Concrete lowering examples | [crates/swc_ecma_transformer/src/es2022/static_blocks.rs](../crates/swc_ecma_transformer/src/es2022/static_blocks.rs) |
| Name resolution and semantic synchronization | [crates/swc_ecma_semantic/src/builder.rs](../crates/swc_ecma_semantic/src/builder.rs), [crates/swc_ecma_semantic/src/mutation.rs](../crates/swc_ecma_semantic/src/mutation.rs) |
| Generic traversal context | [crates/swc_ecma_ast/src/visit.rs](../crates/swc_ecma_ast/src/visit.rs) |

## Validation

Keep test code under `crates/swc_ecma_transformer/tests/`, mirroring the source
module layout:

```text
tests/
|-- lib.rs                 Whole-transform tests for src/lib.rs
|-- analysis.rs            Optional preparation and class layout tests
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
