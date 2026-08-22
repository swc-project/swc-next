# Test suite

This package runs the SWC Next parser, semantic analyzer, code generator,
and transformer against upstream JavaScript and TypeScript test suites. It is a
conformance and regression harness: each run writes a deterministic summary to
`snapshots/`, which can be reviewed and committed like any other source change.

Run all commands below from the workspace root.

## Fixtures

The harness expects three upstream repositories under `tasks/testsuite/fixtures`:

| Directory             | Upstream suite                                                  | Used for                                                |
| --------------------- | --------------------------------------------------------------- | ------------------------------------------------------- |
| `fixtures/test262`    | [TC39 Test262](https://github.com/tc39/test262)                 | JavaScript parser and codegen coverage                  |
| `fixtures/typescript` | [microsoft/TypeScript](https://github.com/microsoft/TypeScript) | TypeScript parser and codegen coverage                  |
| `fixtures/babel`      | [babel/babel](https://github.com/babel/babel)                   | Babel parser/codegen coverage and transform comparisons |

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

The command combines three case sources (`test262`, `typescript`, and `babel`)
with two runners:

- `parser` parses each case and then performs semantic analysis. A case passes
  when success or failure agrees with the upstream expectation.
- `codegen` ignores cases that expect an error, then checks that
  parse → print → parse → print reaches the same byte output after the first
  print.

The six summaries are written to
`tasks/testsuite/snapshots/{parser,codegen}_{test262,typescript,babel}.snap`.
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

## Debug one case

Run one uniquely matched case without updating snapshots:

```sh
cargo run -p testsuite --bin debug -- \
  --runner parser \
  --case test262 \
  --filter "optional-chaining - strict"
```

Supported runners are `parser`, `codegen`, and `babel_transform`. The case
argument selects `test262`, `typescript`, or `babel`; the filter is a contains
match against the combined fixture path and case name. If it matches multiple
cases, the command lists candidates that can be used to narrow it.

## Recommended workflow

1. Make a parser, semantic, codegen, or transformer change.
2. Run the complete relevant snapshot command.
3. Review the snapshot diff. New failures and panics include their fixture
   paths; pass-count changes should be explained by the implementation or by an
   intentional fixture update.
4. Run `cargo check -p testsuite` before committing.
