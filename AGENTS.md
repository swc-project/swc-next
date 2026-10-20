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

Run Gungraun comparisons without a PR after pushing the comparison commits:

```sh
gh workflow run gungraun.yml --ref <workflow-branch> -f base_ref=<base-sha> -f head_ref=<head-sha> -f filter='<benchmark-filter>'
```

The workflow must first exist on the default branch. Omit `head_ref` to compare
against the dispatch revision; omit `filter` to run all benchmarks. Results and
resolved SHAs are available in the run summary and artifacts. PRs still run the
base-versus-merge comparison automatically.

The `jsx_lowering` Gungraun group measures classic, automatic, and development
JSX lowering. Parsing, TypeScript erasure, normalization, and semantic analysis
run in setup; output validation and cleanup run in teardown. The `jsx_transform`
group measures the combined TypeScript, JSX, and optional React Refresh pipeline.
Run the isolated pass on Linux with Valgrind, or smoke-test its cases without
Valgrind on any supported host:

```sh
cargo bench --locked -p benchmark --bench gungraun -- --allow-aslr=yes '*::jsx_lowering::*'
cargo test --locked -p benchmark --bench gungraun -- --test '*::jsx_lowering::*'
```

For JavaScript API instruction profiles, use the `JavaScript API Gungraun`
workflow or, on Linux with Valgrind,
`BASE_REF=main HEAD_REF=HEAD node .github/scripts/run-gungraun-js.mjs`.
PRs run four representative cases; manual workflow runs default to all 20.
Set `JS_BENCH_FILTER` to a case substring such as `parse/underscore`.
See `tasks/benchmark/js/README.md` for setup and artifact interpretation.

### JavaScript Development

| Task                     | Command                                                                                                   |
| ------------------------ | --------------------------------------------------------------------------------------------------------- |
| Install                  | `pnpm install`                                                                                            |
| Build all N-API packages | `pnpm build [--release]`                                                                                  |
| Build one package        | `pnpm --filter <@swc-next/core\|@swc-next/parser\|@swc-next/codegen> build [--release]`                   |
| Build browser package    | `pnpm build:browser [--profile wasm-release]`                                                             |
| Assemble browser package | `node scripts/wasm-package/cli.mjs prepare-browser --source-dir target/wasm-package/core`                 |
| Build all WASM packages  | `pnpm build:wasm [--profile wasm-release]`                                                                |
| Build one WASM package   | `pnpm --filter <@swc-next/core\|@swc-next/parser\|@swc-next/codegen> build:wasm [--profile wasm-release]` |
| Test Node.js API         | `pnpm test` (Node.js API tests live in `npm/core/test`)                                                   |
| Test Node.js WASM        | `pnpm test:wasm` (after `pnpm build:wasm`)                                                                |
| Smoke test browser       | `pnpm test:browser` (after building the browser package)                                                  |
| Develop playground       | `pnpm dev:playground` (requires `wasm32-unknown-unknown`)                                                  |
| Build playground         | `pnpm build:playground`                                                                                  |
| Check playground types   | `pnpm --filter playground check`                                                                         |
| Deploy playground        | `gh workflow run deploy-playground.yml --ref main`                                                       |

The Deploy Playground workflow builds and deploys to `https://play.next.swc.rs`
only when run manually. It requires the `CLOUDFLARE_ACCOUNT_ID` and
`CLOUDFLARE_API_TOKEN` Actions secrets. See `website/playground/README.md` for
Cloudflare setup and deployment details.

Wasm builds use `cargo run --locked -p bridge_cli -- build <component> --output-dir <directory> [cargo build options...]`.
`core` into `target/wasm-package/core`; explicitly run the browser assembly command afterward.
To reuse an existing core Wasm build, run
`node scripts/wasm-package/cli.mjs prepare-browser --source-dir npm/core/src-js`.

## Testing & Examples

### Conformance Testing

First run `./scripts/clone_fixtures.sh`. Review changes under `tasks/testsuite/snapshots/`.

The Test262 parser/codegen runner also discovers `.js` files with Test262 metadata under
`tasks/testsuite/fixtures-manual/test262-legacy/`, including tests retained after upstream removal.
Local syntax regressions use the same metadata format under
`tasks/testsuite/fixtures-manual/parser/`; use `flags: [module]` for Module cases,
otherwise the runner checks both sloppy and strict Script modes. These project-owned
cases are reported separately in `{parser,codegen}_local.snap`, not Test262 totals.
Use `.js` for JavaScript, `.ts` for TypeScript, or `.tsx` for TSX local cases.
The extension selects the parser language.

| Runner    | Suites                            | Check                                                          | Command                                                          |
| --------- | --------------------------------- | -------------------------------------------------------------- | ---------------------------------------------------------------- |
| Parser    | Test262, TypeScript, Babel, local | Parse + semantic result matches upstream expectation           | `cargo run -p testsuite --bin conformance --profile conformance` |
| Codegen   | Test262, TypeScript, Babel, local | AST round trip and print idempotence in both parenthesis modes | Same as above                                                    |
| Resolver  | SWC resolver fixtures             | Name resolution text matches `swc-conformance`                 | Same as above                                                    |
| Transform | Babel                             | Normalized output matches Babel fixtures                       | Same as above                                                    |
| Transform | TypeScript                        | Runtime JavaScript syntax and transform idempotence            | Same as above                                                    |
| Transform | SWC static blocks                 | Normalized output matches SWC fixtures                         | Same as above                                                    |
| ESTree    | Test262, TypeScript               | N-API AST matches `estree-conformance`                         | `pnpm test:conformance`                                          |
| Codec     | Test262, TypeScript               | JS decode/encode codegen output matches native parse/codegen   | `pnpm test:codec`                                                |

Debug one fixture with:

```sh
cargo run -p testsuite --bin debug -- --runner <parser|codegen|transform|resolver> --case <test262|typescript|babel|local|swc|swc_resolver> --filter '<unique substring>'
```

### Minifier Fixtures

Run `cargo test -p swc_next_ecma_minifier --locked` for the local minifier suite.
Filter individual cases with
`cargo test -p swc_next_ecma_minifier --test fixtures -- '<feature>/<case>'`.
Add `input.js`, `output.js`, and optional `options.json` under
`crates/swc_ecma_minifier/tests/fixtures/<feature>/<case>/`; see the crate README
for options. The runner validates syntax and semantics, compares compact output,
and checks minification stability. Expected outputs are maintained manually.
These tests run in the existing workspace test CI without external fixtures.

### Debugging with Examples

Use a small `input.js`/`.ts`/`.tsx` file.

| Scenario                                            | Command                                                                                                                            |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Parser diagnostics, tokens, AST counts              | `cargo run -p swc_next_ecma_parser --example parser -- input.js [--preserve-parens]`                                               |
| Printed output, minification, escaping, parentheses | `cargo run -p swc_next_ecma_codegen --example codegen -- input.js [--minify] [--ascii-only] [--inline-script] [--preserve-parens]` |
| Scopes, bindings, semantic diagnostics              | `cargo run -p swc_next_ecma_semantic --example semantic -- input.js`                                                               |
| TypeScript lowering, class static blocks            | `cargo run -p swc_next_ecma_transformer --example transformer -- input.ts`                                                        |
| Compare/profile Rspack-like pipelines               | `TARGET=<legacy\|next\|oxc> cargo run -p benchmark --release --example rspack_simulator [--features tracy]`                        |

### Adding Rust Unit Tests

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

### Adding JavaScript Unit Tests

- Test only JS API and FFI boundaries (encoding/decoding, strings/offsets, option forwarding). Keep parser/codegen correctness tests in Rust.
- New JavaScript tests should only be added to `@swc-next/core`. `test:wasm` runs the same tests with `NAPI_RS_FORCE_WASI=error`. The browser package should only have a simple smoke test.

Update this file when the commands or workflows above change; otherwise do not edit it.
