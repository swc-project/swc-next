# swc_ecma_semantic

## Testing

Semantic unit tests live in the `tests` module at the bottom of `src/lib.rs`. Keep them grouped by
broad semantic-model features such as symbols, references, modules, and dynamic scopes.

Use unit tests to cover observable behavior and structural contracts of the public `Semantic` API.
Parser behavior, early-error variants, exact diagnostic spans or messages, and narrow syntax cases
belong in the conformance suites and should not be duplicated here.

When adding or updating a unit test:

- Keep each input minimal and do not combine independently testable statements.
- Write the input and semantic result acquisition on separate single lines, followed by flat
  assertions. Separate cases with a blank line.
- Reuse the parsing helpers and named parser or analyzer options across related cases.
- Keep parsing and analysis boilerplate out of test bodies.
- Do not use case tables or loops; write each case explicitly.
- Assert public behavior and structure rather than exact diagnostic text or verbose failure output.

Run the semantic unit tests from the workspace root:

```sh
cargo test -p swc_next_ecma_semantic --lib --locked
```
