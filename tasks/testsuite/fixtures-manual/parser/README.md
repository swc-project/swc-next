# Local parser conformance cases

These project-owned syntax regressions use Test262 metadata and run through
both the parser/semantic and codegen conformance runners. They are not upstream
Test262 tests. Results are reported separately in `parser_local.snap` and
`codegen_local.snap`; use `--case local` with the debug runner. Runtime evaluation is not performed: a positive syntax fixture
may throw at runtime.

Keep one scenario per file. Use `flags: [module]` for Module mode; without it,
the runner checks sloppy and strict Script modes. Negative syntax fixtures use
`negative: { phase: parse, type: SyntaxError }` and skip codegen.

Use `.js` for JavaScript and `.ts` for TypeScript. The local runner selects the
parser language from the file extension; both use the same metadata and modes.

The runners check diagnostics and printing stability, not expected AST shapes.
Keep general AST relationship checks in the existing Rust feature tests.
