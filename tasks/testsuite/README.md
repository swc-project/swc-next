# Test suite

This package runs the experimental parser, semantic analyzer, code generator,
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

The fixture repositories are local inputs and are not part of this workspace.
Updating them can legitimately change case counts and snapshots, so review the
fixture revisions together with any resulting snapshot changes.

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
The process is parallel by default.

Use filters to narrow a local investigation:

```sh
cargo run -p testsuite --bin conformance -- \
  --runners parser \
  --cases optional-chaining \
  --debug
```

- `--runners <name>` selects `parser` or `codegen`.
- `--cases <substring>` keeps cases whose fixture path contains the substring.
- `--debug` uses one worker and prints each case path before running it.
- Repeat `--runners` or `--cases` to select more than one value.

The filtered command still overwrites the selected snapshot with the filtered
result. Treat it as a debugging aid; rerun the complete matrix before keeping
snapshot changes.

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

To inspect one case without rewriting the snapshot, pass a unique substring of
its path:

```sh
cargo run -p testsuite --bin babel_transform -- \
  --case babel-plugin-transform-shorthand-properties/test/fixtures/shorthand-properties/shorthand-single
```

This prints the selected case's status and the normalized actual and expected
outputs. If the substring matches zero or multiple cases, the command asks for
a more specific value.

## Recommended workflow

1. Make a parser, semantic, codegen, or transformer change.
2. Run a focused case while iterating.
3. Run the complete relevant command without filters.
4. Review the snapshot diff. New failures and panics include their fixture
   paths; pass-count changes should be explained by the implementation or by an
   intentional fixture update.
5. Run `cargo check -p testsuite` before committing.
