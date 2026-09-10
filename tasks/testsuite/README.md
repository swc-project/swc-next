# Test suite

This package runs the SWC Next parser, semantic analyzer, code generator,
and transformer against upstream JavaScript and TypeScript test suites. It is a
conformance and regression harness: each run writes a deterministic summary to
`snapshots/`, which can be reviewed and committed like any other source change.

Run all commands below from the workspace root.

## Fixtures

The harness uses pinned upstream repositories under `tasks/testsuite/fixtures`:

| Directory             | Upstream suite                                                  | Used for                                                |
| --------------------- | --------------------------------------------------------------- | ------------------------------------------------------- |
| `fixtures/test262`    | [TC39 Test262](https://github.com/tc39/test262)                 | JavaScript parser and codegen coverage                  |
| `fixtures/typescript` | [microsoft/TypeScript](https://github.com/microsoft/TypeScript) | TypeScript parser and codegen coverage                  |
| `fixtures/babel`      | [babel/babel](https://github.com/babel/babel)                   | Babel parser/codegen coverage and transform comparisons |
| `fixtures/swc`        | [swc-project/swc](https://github.com/swc-project/swc)           | SWC resolver inputs and transform comparisons           |
| `fixtures/swc-conformance` | [CPunisher/swc-conformance](https://github.com/CPunisher/swc-conformance) | Expected SWC name resolution text              |
| `fixtures/estree-conformance` | [oxc-project/estree-conformance](https://github.com/oxc-project/estree-conformance) | N-API ESTree comparisons                 |

For a fresh checkout, clone the fixtures with:

```sh
./scripts/clone_fixtures.sh
```

Run this script serially across worktrees that share a Git common directory;
concurrent invocations are unsupported.

The fixture repositories are local inputs and are not part of this workspace.
Updating them can legitimately change case counts and snapshots, so review the
fixture revisions together with any resulting snapshot changes.

Fixture directories are generated, read-only inputs. Do not edit them directly;
update their pinned revisions in `scripts/clone_fixtures.sh` and regenerate them
instead.

## Parser and codegen conformance

Run the complete conformance matrix:

```sh
cargo run -p testsuite --bin conformance
```

The command combines four case sources (`test262`, `typescript`, `babel`, and `local`)
with two runners:

- `parser` parses each case and then performs semantic analysis. A case passes
  when success or failure agrees with the fixture expectation.
- `codegen` ignores cases that expect an error. With `preserve_parens` set to
  both `false` and `true`, it compares the AST before and after
  parse → print → parse, then prints again and checks that the output is unchanged.

The eight summaries are written to
`tasks/testsuite/snapshots/{parser,codegen}_{test262,typescript,babel,local}.snap`.

Project-owned JavaScript syntax regressions under `fixtures-manual/parser` use
Test262 metadata but are counted in the separate `local` suite. Keep TypeScript
syntax regressions in the existing feature tests in
`crates/swc_ecma_parser/src/parser.rs`. Retained upstream cases
under `fixtures-manual/test262-legacy` remain part of the Test262 suite.

The Babel parser and codegen runs use
`babel/packages/babel-parser/test/fixtures`. Parser cases read their expected
result from `output.json`/`output.extended.json` and `options.json`; fixtures
with Babel-specific syntax or parser options unsupported by this harness are
excluded. The process is parallel by default.

## Babel transform conformance

Run all supported Babel transform fixtures:

```sh
cargo run -p testsuite --bin babel_transform
```

For each runnable fixture, the harness:

1. parses and semantically analyzes `input.*`;
2. applies the transformer options derived from the fixture's Babel
   `options.json`;
3. prints the transformed AST;
4. parses and prints Babel's `output.*`; and
5. compares the two normalized byte sequences.

Fixtures with unsupported plugins, presets, or invalid options are counted as
skipped. The full run writes
`tasks/testsuite/snapshots/babel_transform.snap`, including totals and
per-package diagnostics.

## SWC transform conformance

Run the SWC static-block transform fixtures:

```sh
cargo run -p testsuite --bin swc_transform
```

This reads `input.js`/`output.js` pairs from
`swc/crates/swc_ecma_transforms_compat/tests/static-blocks` at the revision
pinned in `scripts/clone_fixtures.sh`. It uses the same parse → analyze →
transform → print comparison as the Babel runner. Transform options follow
SWC's `es2022_static_blocks.rs` test entry: standalone static blocks run with
`static_blocks=true` and the default `prevent_extensions=false`; fixtures under
`class-properties/` are counted as skipped until that transform is supported.

The SWC run writes `tasks/testsuite/snapshots/swc_transform.snap`. Both transform
runners write normalized `expected/`, `actual/`, and `differences.md` files under
`tasks/testsuite/outputs/{babel_transform,swc_transform}/`. Missing fixtures and
panics fail the command; ordinary output mismatches are recorded in snapshots.
CI runs both commands and checks the snapshots for changes.

To cover both static-block lowering modes locally, run:

```sh
cargo run --locked -p testsuite --bin swc_transform
cargo run --locked -p testsuite --bin babel_transform
```

The Babel static-block fixtures use `prevent_extensions=true`, exercising the
closure lowering; the SWC fixtures exercise the default private-field lowering.

## Debug one case

Run one uniquely matched case without updating snapshots:

```sh
cargo run -p testsuite --bin debug -- \
  --runner parser \
  --case test262 \
  --filter "optional-chaining - strict"
```

Supported runner/case combinations are:

| Runner | Case |
| ------ | ---- |
| `parser`, `codegen` | `test262`, `typescript`, `babel`, `local` |
| `babel_transform` | `babel` |
| `swc_transform` | `swc` |
| `resolver` | `swc_resolver` |

The filter is a contains match against the combined fixture path and case name.
If it matches multiple cases, the command lists candidates that can be used to
narrow it. For example, debug SWC private-name collision handling with:

```sh
cargo run -p testsuite --bin debug -- \
  --runner swc_transform --case swc --filter 'static-blocks/basic/name-conflicts02'
```

## Recommended workflow

1. Make a parser, semantic, codegen, or transformer change.
2. Run the complete relevant snapshot command.
3. Review the snapshot diff. New failures and panics include their fixture
   paths; pass-count changes should be explained by the implementation or by an
   intentional fixture update.
4. Run `cargo check -p testsuite` before committing.
