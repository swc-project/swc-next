# AGENTS.md

## Generated Code

- After changing AST declarations or their generators, regenerate the Rust AST APIs
  and JavaScript types/codecs with `ast_tools`, and review the generated diff.
  Edit the declarations or generators rather than generated files.

## Testing

### Adding Rust Unit Tests

- Add tests to the existing module; group related cases by broad feature and use a coarse
  feature name for the test function.
- Keep each input small; do not combine independently testable JavaScript/TypeScript statements.
- Move repeated setup and parsing boilerplate into local helper functions so the test body only
  shows the input, result, and assertions.
- Reuse named options, contexts, and language configurations across related cases.
- Keep independent input cases and their assertions explicit rather than in a cases
  table. Loops are appropriate for running the same cases across configurations;
  reuse named options to keep the matrix clear.
- Keep each input and result acquisition on one source line, separate cases with a blank line,
  and encode input line breaks as `\n`.
- Assert observable behavior and structure; avoid exact diagnostic messages and verbose custom
  failure output unless the text itself is part of the contract.
- Do not add unit tests for behavior already covered by conformance tests or for highly specific
  cases.

### Adding JavaScript Unit Tests

- Test only JS API and FFI boundaries (encoding/decoding, strings/offsets, option forwarding). Keep parser/codegen correctness tests in Rust.
- New JavaScript tests should only be added to `@swc-next/core`. `test:wasm` runs the same tests with `NAPI_RS_FORCE_WASI=error`. The browser package should only have a simple smoke test.

## README Writing

- Follow the root README: briefly describe purpose, capabilities, basic usage,
  essential limitations, and crate testing where relevant.
- Write necessary document API, options, and attributes in Rust doc rather than README.
- Document commands in the relevant component README
- Omit development history, step-by-step contributor workflows, implementation
  walkthroughs, and exhaustive command lists.

## Code Review

- Before completing implementation or review, check changed tests and docs against
  the conventions above. Report violations with the relevant rule and location.
- For test refactors, compare inputs, expected results, configuration combinations,
  and assertions with the original. Passing tests or unchanged test counts alone
  do not establish equivalent coverage; preserve checks when reorganizing tests.
- Allow configuration matrices and traversal of actual results. Do not weaken
  assertions or expose private APIs merely to change test layout.
