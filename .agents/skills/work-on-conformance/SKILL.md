---
name: work-on-conformance
description: Diagnose and improve SWC Next parser, semantic, codegen, transformer, and ESTree conformance without changing the AST or upstream fixtures. Use for investigating conformance snapshot failures or panics, fixing failing Test262, TypeScript, or Babel cases, improving conformance pass counts, and correcting conformance harness behavior.
---

# Work on Conformance

Improve conformance by fixing implementation behavior and verifying the complete affected suite. Preserve the integrity of the upstream cases and recorded results.

## Guardrails

- Never modify `crates/swc_ecma_ast/**`, `tasks/ast_tools/**`, AST definitions, or the public AST shape. If a correct fix requires an AST change, stop and report the constraint.
- Treat `tasks/testsuite/fixtures/**` and the pinned fixture revisions in `scripts/clone_fixtures.sh` as read-only. Run the clone script only to populate missing fixtures.
- Never edit `tasks/testsuite/snapshots/*.snap` by hand. Regenerate snapshots with the owning conformance command.
- Never improve counts by adding or widening skips, ignores, filters, unsupported-case lists, or expected failures; weakening assertions; disabling runners; or reducing the cases exercised.
- Modify the testsuite harness only to correct inaccurate case selection, execution, normalization, or reporting. Ensure the change does not conceal a runnable failure.

## Workflow

1. Read `tasks/testsuite/README.md`, inspect the working tree, and ensure the pinned fixtures are available.
2. Inspect the current snapshots and select one cohesive group of related failures or panics. Keep unrelated failures out of the change.
3. Reproduce a Rust testsuite case without rewriting snapshots:

   ```sh
   cargo run --locked -p testsuite --bin debug -- \
     --runner <parser|codegen|babel_transform> \
     --case <test262|typescript|babel> \
     --filter "<unique fixture path or case name>"
   ```

   For ESTree mismatches, inspect the matching upstream oracle and reproduce through the N-API parser before running the full ESTree command.
4. Trace the mismatch to parser, semantic, codegen, transformer, or `napi/core` ESTree adapter behavior. Fix the underlying implementation within the guardrails.
5. Run focused crate or binding tests while iterating, then run the complete affected conformance command:
   - Parser or codegen: `cargo run --locked -p testsuite --bin conformance`
   - Babel transform: `cargo run --locked -p testsuite --bin babel_transform`
   - ESTree: build with `pnpm build:core --release`, then run `pnpm --filter @swc-next/core conformance`
6. Review every generated snapshot diff. Require the intended failures or panics to disappear with no new failures, panics, mismatches, or skipped cases. Explain any changed totals from the implementation.
7. Run `cargo check --locked -p testsuite` and the relevant implementation tests. Commit only implementation, legitimate harness corrections, and snapshots produced by a complete run.

If progress is impossible without changing the AST, fixtures, or failure-hiding policy, leave those files untouched and report the blocker clearly.
