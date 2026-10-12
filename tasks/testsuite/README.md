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

## Conformance

Run the complete conformance matrix:

```sh
cargo run -p testsuite --bin conformance --profile conformance
```

The command combines four case sources (`test262`, `typescript`, `babel`, and `local`)
with the parser and codegen runners:

- `parser` parses each case and then performs semantic analysis. A case passes
  when success or failure agrees with the fixture expectation.
- `codegen` ignores cases that expect an error. With `preserve_parens` set to
  both `false` and `true`, it compares the AST before and after
  parse → print → parse, then prints again and checks that the output is unchanged.

The command also runs the SWC resolver fixtures and the Babel and TypeScript
transform suites. All runners share the same snapshot and panic reporting
logic.

The eight parser/codegen summaries are written to
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

## Transform conformance

Babel and TypeScript cases use one `TransformerRunner` and the same parse,
semantic analysis, `Transformer::transform`, and codegen pipeline. Each case
provides `TransformerOptions` and optional expected output.

Both collections run as part of the `conformance` command above.

Babel cases derive options from `options.json` and compare the transformed
output with Babel's `output.*`, after parsing and printing both. Unsupported
plugins, presets, and invalid options are skipped.

The Babel collection includes `babel-plugin-transform-typescript/test/fixtures`
and `babel-generator/test/fixtures/typescript` from the pinned Babel repository.
`transform-typescript` enables the TypeScript transform: `onlyRemoveTypeImports`
maps to `verbatimModuleSyntax`, `optimizeConstEnums` maps to the inverse of
`preserveConstEnums`, and `jsxPragma`/`jsxPragmaFrag` configure JSX import retention.
These mappings use the transformer's TypeScript semantics; differences from
Babel's import or const-enum behavior remain output mismatches. Generator
fixtures use syntax-only TypeScript options and compare their preserved
TypeScript output, without enabling type erasure.

TypeScript parser/codegen cases live in `cases/typescript/parser.rs`;
transform cases live in `cases/typescript/transform.rs`. Both share fixture
parsing and compiler-option metadata in `cases/typescript/mod.rs`.
Transform cases use source inputs under the pinned repository's `compiler`
and `conformance` suites. They do not consume emitted JavaScript, diagnostics,
baselines, or compiler options other than `verbatimModuleSyntax` and
`preserveConstEnums`. Each TypeScript virtual file runs independently with
the cross product of those option variants; unspecified options default to
`false`. JavaScript, JSON, config, and package virtual files are ignored.

Without expected output, the runner checks that declaration files emit no
JavaScript and accepts them without a second transform. For other inputs,
it parses the output as JavaScript or JSX with the input's
source type mode, runs semantic analysis and the same transformer options again,
and compares the two printed outputs for idempotence. Both passes use the same
pipeline. Parser or semantic errors cause the case to be skipped; changed output
is a failure. There are no special exclusions for decorators or module syntax.

Their snapshots are `tasks/testsuite/snapshots/transform_{babel,typescript}.snap`,
using the same format as the other conformance runners. Transform pass rates
include skipped cases in the denominator. Fixture failures are recorded in
snapshots; panics cause a nonzero exit.
Failures in TypeScript fixtures containing multiple virtual files include the
virtual filename alongside the physical fixture path and transform options.

Transformed and expected output are compared and validated in memory. The debug
command uses the same runner without updating snapshots.

## Debug one case

Run one uniquely matched case without updating snapshots:

```sh
cargo run -p testsuite --bin debug -- \
  --runner parser \
  --case test262 \
  --filter "optional-chaining - strict"
```

Supported runners are `parser`, `codegen`, `transform`, and `resolver`.
The case argument selects `test262`, `typescript`, `babel`, `local`, or
`swc_resolver`; the filter is a contains match against the combined fixture
path and case name. `transform` supports `--case babel` and `--case typescript`;
`resolver` requires `--case swc_resolver`.

Each TypeScript transform case represents one virtual file and one concrete
combination of `verbatimModuleSyntax` and `preserveConstEnums`. The case name
includes both options, so a filter can select a specific combination. Pass
`--typescript-mode` to override the fixture's options with `default`, `verbatim`,
`preserve-const-enums`, or `verbatim-preserve-const-enums`. If a filter matches
multiple cases, the command lists candidates that can be used to narrow it.

## Recommended workflow

1. Make a parser, semantic, codegen, or transformer change.
2. Run the complete relevant conformance command.
3. Review any snapshot diff and invariant failures. New failures and panics
   include their fixture paths; pass-count changes should be explained by the
   implementation or by an intentional fixture update.
4. Run `cargo check -p testsuite` before committing.
