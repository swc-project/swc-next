# Formatter tests

`api.rs` checks public option values, parser options, and error handling.

Local inputs live under `fixtures/<language>/<syntax>/`. `build.rs` discovers
supported source files and generates one test per input. Each input has an
adjacent `.snap` containing its source, option sets, and formatted output.
An `options.json` array selects Prettier-style options; the nearest ancestor
file wins, and absent options use Prettier defaults.

The fixture runner formats twice. Both passes must parse, retain the comment
count, and agree between the source and AST entry points. Non-idempotent output
fails the test rather than being accepted in a snapshot. Comment counts and parsing
alone do not prove that formatting preserved semantics.

Run the local tests with:

```sh
cargo test -p swc_next_ecma_formatter --locked
```

Generate snapshots after adding an input or intentionally changing output:

```sh
INSTA_UPDATE=always cargo test -p swc_next_ecma_formatter --test fixtures --locked
```

Review the output before accepting a snapshot. Keep each input small and place
comment boundaries beside the syntax they exercise. Use a few representative
widths and options for relevant layout branches.

The full Prettier 3.9.9 conformance matrix lives in `tasks/testsuite`. It reads
upstream fixture specifications and committed outputs independently of local
snapshots. Unsupported syntax remains a reported failure while support grows.
Local and upstream tests share option decoding in `support/prettier_options.rs`.
