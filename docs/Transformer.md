# Transformer Implementation

This document describes `crates/swc_ecma_transformer`, its integration with
Semantic, and the conventions for implementing transforms. See
[AGENTS.md](../AGENTS.md) for repository-wide commands and testing conventions.

## Object relationships

```text
Transformer                         TraverseMutCtx<TransformerState>
| implements TraverseMut            |
+-- ES2022                          +-- ast: &mut Ast
    +-- StaticBlocks                +-- state: TransformerState
                                         +-- path
                                         +-- semantic: &mut ScopeModel
                                         +-- used_names
                                         +-- helpers
                                         +-- comments
                                         +-- container
                                         +-- statement_injector

Visitor callback -> ctx.mutate() -> Mutation
                                    +-- borrows ast
                                    +-- borrows state, including path
```

`Transformer::transform_with_scopes` dispatches callbacks to the enabled concrete passes.
This entry point requires TypeScript lowering to have already run; its
options must disable TypeScript. It accepts an existing ScopeModel for callers
that only need synchronized scope transforms, including static-block benchmarks.

Each pass keeps its own traversal state, such as calls to trailing static-block closures.
`TraverseMutCtx` belongs to `swc_ecma_ast`; generic traversal does not depend on
Semantic or implement transform replacement policies.

`ScopeModel` consumes Semantic's basic scope, binding, declaration and reference
indexes. It preserves their identities without retaining Semantic's TypeScript
member/alias lookup indexes or hoisting conflict tables. Its mutation operations
take semantic records and IDs, not AST syntax.

`Mutate` is a transform-crate trait implemented for
`TraverseMutCtx<TransformerState>`. Its `mutate()` method returns a temporary
`Mutation` borrowing the existing AST and state. `Mutation` reads syntax and
coordinates edits with ScopeModel. It is not a second traversal engine or a tree
snapshot. Visitors read AST through `Mutation::ast()` and perform edits through
the Mutation boundary.

## Execution flow

1. Normalize parser AST shapes with `Normalizer::normalize_runtime` when
   TypeScript lowering is enabled, otherwise `Normalizer::normalize`.
2. Analyze that AST with Semantic and pass it by value to `Transformer::transform`.
3. Run main's TypeScript lowering with full Semantic, then refresh Semantic for
   the resulting runtime AST. Construct ScopeModel only after that phase.
   This existing phase retains its Semantic/VisitMut contract; migrating it
   to Mutation is separate work. Static-block lowering uses synchronized Mutation.
   `Transformer::transform` returns the final ScopeModel.
4. Select enabled passes through `TransformerOptions::transform_features` and
   traverse with `TraverseMutCtx<TransformerState>`.
5. Prepare generated nodes, scopes, bindings and references through Mutation.
6. Attach queued statements when exiting their destination containers; inject
   prepared helper imports at Program exit.
7. `Mutation::finish` applies deferred comment transfers and debug-checks that
   statement, helper and container queues are empty.

There is no program-wide class-semantic preparation traversal. At ClassBody entry,
static-block lowering reads runtime static-field order, empty-block comments and
private-name collisions from the current class. This information stays local to
the pass. Lexical `this` and `super` remain in arrow syntax.

## Traversal and attachment

Mutable typed hooks return `TraverseAction<T>`: `Continue`, `Skip`, `Stop`,
`ReplaceContinue(T)`, `ReplaceSkip(T)` or `ReplaceStop(T)`. Untyped
`enter_node`/`exit_node` hooks maintain the path and enclose every visit, including
skipped, stopped and replaced visits.

`Skip` ends the current node's remaining hooks while siblings continue. In enter
it also skips children and exit. `Stop` additionally stops remaining fields/list
entries and propagates outward; already entered ancestors still run exit hooks.
Replacement ends the old visit. The parent attaches it before applying the
requested disposition. `ReplaceContinue` restarts traversal of the replacement;
`ReplaceSkip` continues with the next sibling; `ReplaceStop` stops traversal.
Returning a replacement applies it even when its NodeId is unchanged.

Static-block lowering returns `(class, call)` with `ReplaceContinue` from
`exit_expr`, preserving the class's identity inside the sequence. Declarations
queue the call through `insert_after` at `exit_decl`. The default-export kind hook
forwards class declarations because its Class variant bypasses the Decl hook.

Class-body edits use `container_class_body_mut`; reads use
`container_class_body`, which includes pending edits. `push_container` and
`unshift_container` also edit that logical list. `container_flush` attaches it
with one `add_sub_range` call and releases cached handles. Unedited lists keep
their original range; repeated flush without edits does nothing. ClassBody enter
flushes before generic traversal reads the member range, and exit flushes any
edits made while visiting children.

The container and statement queues defer attachment only. Materialize retained
contents and synchronize semantic information before removing or replacing old
attachments. Detached arena storage is not a reason to retain stale node,
reference or scope-owner mappings.

## Mutation contracts

- `add_node(scope, |ast| ...)` allocates nodes and registers the new NodeId range
  in their evaluation scope. Its closure must not change existing edges. Binding,
  reference and scope-owner identities use their dedicated APIs.
- `generate_var(scope, base)` registers a `BoundIdentifier` in the destination
  hoist scope and queues its declaration. The identifier retains its declaration
  and symbol. `reference_read`, `reference_write` and `assign` create fresh
  references to that same symbol.
- Generated names share a collision domain initialized lazily from ScopeModel's
  symbols, declarations and references, including unresolved references. Private
  names use a separate domain from the current class's own member keys.
- `register_binding` reads the AST name and passes its ID, arena-lived text and
  flags to ScopeModel. `register_reference` constructs a Reference record.
  Mutation registers pending allocations; ScopeModel does not read syntax.
- Prepared initializer and computed-key wrappers retain the original subtree
  and attach through the AST setters at the lowering site. Before replacing an
  arrow's empty placeholder body, clear that detached body's scope mapping.
- `inject_statement`, `inject_variable` and `insert_after` queue prepared nodes.
  The visitor lifecycle flushes each destination container. `insert_after`
  forwards through labels and exports to the containing statement-list anchor.
- `inherits_comments(child, parent)` queues the parent's leading/trailing anchors
  for inheritance and returns the child. `CommentState` composes transfers in
  call order; finish applies them with one `Ast::retarget_comments` call. Inner
  comments retain child/token anchors. Codegen support for retargeted leading
  anchors remains incomplete; see `context/comments.rs`.

## Scope operations

ScopeModel implements the index operations below, and Mutation composes them with
AST traversal and allocation registration. Subtree moves accept typed AST handles
and reuse `Traverse`; only nodes evaluated directly in the source scope move.
Nested scopes keep their contents and receive a new parent when needed.

| Operation | Contract |
| --- | --- |
| `create_child_scope` | Register a new scope owner below an active parent; move existing content separately. |
| `convert_scope` | Transfer a scope to a replacement owner and change kind/flags while preserving bindings, references and child scopes. |
| `move_node_to_scope` | Move one evaluation mapping and its reference scope while preserving identity and target. |
| `reparent_scope` | Change ancestry and refresh affected hoist targets without changing retained content identities. |
| `set_scope_flags` | Apply caller-computed flags to one scope. |
| `remove_scope` | Retire an empty scope and clear its detached owner's mapping after retained content has moved. |
| `clear_node_scope` | Clear a detached syntax shell's mapping without visiting retained children. |

The caller maintains strictness inheritance. `create_child_scope` and
`convert_scope` apply supplied flags as-is; moves and reparenting leave flags
unchanged. When a syntax or ancestry change affects strictness, the caller must
compute and apply flags to each affected scope through `set_scope_flags`.

`convert_scope` requires its current owner to own only that scope. When replacing
the owner, it clears the old node's mapping and maps the new node to the same
ScopeId. A replacement owner must not already own another scope.

A single-expression static block moves its expression into the destination
scope, clears the discarded ExpressionStatement mapping and retires its empty
scope. A block reused as an arrow body converts its scope to Function and changes
the owner to the arrow; its parameters/body are allocated in that same scope.
When moving that arrow under an additional wrapping closure, reparent its scope.
A new trailing closure creates its own scope, moves its prepared parameters/body
there, and receives each retained expression or child scope separately.

## Helpers

Helpers are external. `Mutation::helper_callee` registers a binding and prepares
its import on first use. `Helpers` caches the declaration; Program exit queues it
through `inject_helpers`. Repeated uses share the binding/import. Module output
uses `import`; script output uses `require`.

`call_set_function_name` uses `@babel/runtime/helpers/setFunctionName` and
`call_to_property_key` uses `@swc/helpers`. Static-block name restoration places
helper expressions in existing field initializers or the trailing closure; it
does not synthesize a named-evaluation static block. Inline helpers and a
swc-next-owned helper library remain TODOs.

## Source map

| Responsibility | File under `crates/` |
| --- | --- |
| Options and visitor dispatch | `swc_ecma_transformer/src/lib.rs` |
| Shared state | `swc_ecma_transformer/src/state.rs` |
| Borrowed Mutation and statement insertion | `swc_ecma_transformer/src/context/mod.rs` |
| Allocation, names, semantic registration and scope adapters | `swc_ecma_transformer/src/context/semantic.rs` |
| Class containers and expression attachment | `swc_ecma_transformer/src/context/container.rs` |
| Deferred statements | `swc_ecma_transformer/src/context/statement_injector.rs` |
| Comment transfers | `swc_ecma_transformer/src/context/comments.rs` |
| Helper cache and construction | `swc_ecma_transformer/src/helpers.rs`, `swc_ecma_transformer/src/context/helpers.rs` |
| Static blocks and local class information | `swc_ecma_transformer/src/es2022/static_blocks.rs` |
| Semantic indexes and scope operations | `swc_ecma_semantic/src/scope_model.rs` |
| Generic traversal | `swc_ecma_ast/src/visit.rs` |

## Validation

Keep tests under `crates/swc_ecma_transformer/tests/`, mirroring source layout.
Source modules load them with `#[cfg(test)]` and `#[path]`, retaining private
implementation access. The crate sets `autotests = false`.

Reuse conformance fixtures for covered behavior; add focused tests for semantic
invariants or observable behavior following AGENTS.md. For scope changes, compare
transformed ScopeModel node/reference scopes, binding ownership and reference
targets with fresh Semantic analysis, and compare generated JavaScript execution
with native input.

Run Rust tests and required formatting/lint checks. For static-block/helper
changes, prepare fixtures with `./scripts/clone_fixtures.sh` and run both
`babel_transform` and `swc_transform`; both use the shared `transform` runner
and write `transform_babel.snap` / `transform_swc.snap`. Inspect snapshots and
`tasks/testsuite/outputs/transform/{babel,swc}/differences.md`; successful exit alone does not
mean fixtures match. Separate helper spelling/import differences from semantic
differences. Verify binding, evaluation order and constructor behavior against
native JavaScript before changing behavior solely to match output text.
