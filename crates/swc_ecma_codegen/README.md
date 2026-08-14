# swc_ecma_codegen

## Testing

Local codegen integration tests live in `tests/codegen/js.rs` and `tests/codegen/ts.rs`.
`tests/codegen/main.rs` registers the suites, while `tests/codegen/tester.rs` provides the shared
assertion helpers. Files under `tests/codegen/esbuild/` mirror upstream esbuild tests and should
normally change only when syncing with esbuild; add project-specific coverage to `js.rs` or
`ts.rs` instead.

Choose the helper that matches the syntax and printer mode under test:

| Helper                                                                            | Mode                           |
| --------------------------------------------------------------------------------- | ------------------------------ |
| `expect_printed`                                                                  | JavaScript module              |
| `expect_printed_script`                                                           | JavaScript script              |
| `expect_printed_ts`                                                               | TypeScript module              |
| `expect_printed_jsx`                                                              | JSX module                     |
| `expect_printed_minify`, `expect_printed_ts_minify`, `expect_printed_jsx_minify`  | Minified output                |
| `expect_printed_ascii`, `expect_printed_minify_ascii`, `expect_printed_jsx_ascii` | ASCII-only output              |
| `expect_stable`                                                                   | Pretty output is a fixed point |

Run the suite from the workspace root:

```sh
cargo test -p swc_next_ecma_codegen --test codegen
```
