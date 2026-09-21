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

## TypeScript-only transform tests

Run the dedicated TypeScript-only transform suite:

```sh
cargo run -p testsuite --bin typescript_transform
```

The suite reads only source inputs from the pinned TypeScript repository. It
does not consume TypeScript's emitted JavaScript, diagnostics, compiler
options other than `verbatimModuleSyntax` and `preserveConstEnums`, baselines,
or other oracle metadata.
Every TypeScript input under the `compiler` and `conformance` suites runs
through parse, read-only semantic analysis, normalization, the TypeScript
transform, and codegen. The emitted text must also parse as JavaScript or JSX.
Input parse and semantic errors are reported as rejected negative inputs.
Output syntax that still requires the decorators or module transform is
reported separately; every such input has already run through the TS-only
pipeline, and no other transform is invoked.

Physical TypeScript cases may contain multiple `// @filename:` virtual files.
The harness transforms each TypeScript virtual file independently. JavaScript,
JSON, config, and package virtual files are ignored. Every eligible input is
checked with the cross product of its fixture-level `verbatimModuleSyntax` and
`preserveConstEnums` option variants; unspecified options use TypeScript's
`false` defaults. Declaration files must emit no JavaScript; all other output
must parse as JavaScript or JSX. Exact output belongs in a focused unit test
only when it expresses a transform boundary or invariant.

The full run writes `tasks/testsuite/snapshots/typescript_transform.snap` with
total result counts plus any failure or panic diagnostics. Full transform
output is written to the Git-ignored directory
`tasks/testsuite/outputs/typescript_transform/`, with one output file per
physical fixture. Outputs contain one section per virtual file and show option
variants only when a fixture requests multiple transform option combinations.
Each output is accompanied by a relative symbolic link to its physical fixture
so the input and output can be reviewed side by side.

By default, at most 100 full-corpus failures are printed. Pass `--verbose` to
print all of them.

## Debug one case

Run one uniquely matched case without updating snapshots:

```sh
cargo run -p testsuite --bin debug -- \
  --runner parser \
  --case test262 \
  --filter "optional-chaining - strict"
```

Supported runners are `parser`, `codegen`, `babel_transform`, and
`typescript_transform`. The case argument selects `test262`, `typescript`, `babel`, or
`local`; the filter is a contains match against the combined fixture path and
case name. `typescript_transform` requires the `typescript` case set; pass
`--typescript-mode` to override the fixture's `verbatimModuleSyntax` metadata.
If a filter matches multiple virtual files, the command lists candidates that
can be used to narrow it.

## Recommended workflow

1. Make a parser, semantic, codegen, or transformer change.
2. Run the complete relevant conformance command.
3. Review any snapshot diff and invariant failures. New failures and panics
   include their fixture paths; pass-count changes should be explained by the
   implementation or by an intentional fixture update.
4. Run `cargo check -p testsuite` before committing.
