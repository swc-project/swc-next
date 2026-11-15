# swc_formatter_host

Formatting sessions for SWC Next document producers. Provides shared layout options, document decoding, and embedded-language dispatch.

**Experimental and unstable.** APIs and document formats may change.

Use `FormatSession` with a language-specific producer or an encoded document. Language formatters provide parsing and syntax options; applications provide embedded-language dispatch. See the [crate API](src/lib.rs) for session usage and document lifetimes.

## Testing

Unit tests cover formatting sessions, embedded-language dispatch, and error handling.

```sh
cargo test -p swc_next_formatter_host --locked
```
