# AGENTS.md

## Commands

Run from the workspace root.

### Rust Development

| Task                                               | Command                                                              |
| -------------------------------------------------- | -------------------------------------------------------------------- |
| Check                                              | `cargo check --workspace`                                            |
| Format / lint                                      | `cargo fmt --all`; `cargo clippy --all --all-targets -- -D warnings` |
| Regenerate AST code                                | `cargo run -p ast_tools`                                             |
| Build                                              | `cargo build --workspace [--release]`                                |
| Test                                               | `cargo test --workspace --locked`                                    |
| Benchmark wall time                                | `cargo bench -p benchmark --bench <parser\|visit\|semantic>`         |
| Benchmark instructions (Linux + Valgrind/Gungraun) | `cargo bench -p benchmark --bench gungraun`                          |

### JavaScript Development

| Task                     | Command                                                              |
| ------------------------ | -------------------------------------------------------------------- |
| Install                  | `pnpm install`                                                       |
| Build all N-API packages | `pnpm build [--release]`                                             |
| Build one package        | `pnpm --filter <@swc-next/core\|@swc-next/parser> build [--release]` |
| Test                     | `pnpm test`                                                          |
| Test parser package      | `pnpm --filter @swc-next/parser test`                                |

## Testing & Examples

### Conformance Testing

First run `./scripts/clone_fixtures.sh`. Review changes under `tasks/testsuite/snapshots/`.

| Runner    | Suites                     | Check                                                | Command                                                   |
| --------- | -------------------------- | ---------------------------------------------------- | --------------------------------------------------------- |
| Parser    | Test262, TypeScript, Babel | Parse + semantic result matches upstream expectation | `cargo run -p testsuite --bin conformance --profile test` |
| Codegen   | Test262, TypeScript, Babel | `parse -> print` is idempotent                       | Same as above                                             |
| Resolver  | SWC resolver fixtures      | Name resolution text matches `swc-conformance`       | Same as above                                             |
| Transform | Babel                      | Normalized output matches Babel fixtures             | `cargo run -p testsuite --bin babel_transform`            |
| ESTree    | Test262, TypeScript        | N-API AST matches `estree-conformance`               | `pnpm --filter @swc-next/core conformance`                |

Debug one fixture with:

```sh
cargo run -p testsuite --bin debug -- --runner <parser|codegen|babel_transform|resolver> --case <test262|typescript|babel|swc_resolver> --filter '<unique substring>'
```

### Debugging with Examples

Use a small `input.js`/`.ts`/`.tsx` file.

| Scenario                                            | Command                                                                                                          |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Parser diagnostics, tokens, AST counts              | `cargo run -p swc_next_ecma_parser --example parser -- input.js [--preserve-parens]`                             |
| Printed output, minification, escaping, parentheses | `cargo run -p swc_next_ecma_codegen --example codegen -- input.js [--minify] [--ascii-only] [--preserve-parens]` |
| Scopes, bindings, semantic diagnostics              | `cargo run -p swc_next_ecma_semantic --example semantic -- input.js`                                             |
| Compare/profile Rspack-like pipelines               | `TARGET=<legacy\|next\|oxc> cargo run -p benchmark --release --example rspack_simulator [--features tracy]`      |

### Adding Unit Tests

- Add tests to the existing module; group related cases by broad feature and use a coarse
  feature name for the test function.
- Keep each input small; do not combine independently testable JavaScript/TypeScript statements.
- Move repeated setup and parsing boilerplate into local helper functions so the test body only
  shows the input, result, and assertions.
- Reuse named options, contexts, and language configurations across related cases.
- Write assertions flat; do not define a cases table and iterate over it.
- Keep each input and result acquisition on one source line, separate cases with a blank line,
  and encode input line breaks as `\n`.
- Assert observable behavior and structure; avoid exact diagnostic messages and verbose custom
  failure output unless the text itself is part of the contract.
- Do not add unit tests for behavior already covered by conformance tests or for highly specific
  cases.

Update this file when the commands or workflows above change; otherwise do not edit it.
