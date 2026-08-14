# swc_ecma_parser

## Testing

Parser unit tests live beside the implementation in the `tests` modules at the bottom of
`src/lexer.rs` and `src/parser.rs`.

Run all parser unit tests or one functional group from the workspace root:

```sh
cargo test -p swc_next_ecma_parser
cargo test -p swc_next_ecma_parser lexer::tests::template
```
