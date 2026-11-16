# swc_formatter

Formatting configuration and document transport for SWC Next. Provides shared print options, with JavaScript and TypeScript configuration behind the `js` feature.

**Experimental and unstable.** APIs, binary formats, and formatting output may change. Use matching SWC Next wrappers and bindings.

Convert `PrintConfig` into `swc_next_formatter_core::PrintOptions`, or enable `js` and convert `js::JsFormatterConfig` into `swc_next_formatter_js::FormatOptions`. Enable `serde` for configuration deserialization and `bridge` for internal binary document transport. See the [crate API](src/lib.rs) for options and defaults.

## Testing

Unit tests cover binary print options and document printing.

```sh
cargo test -p swc_next_formatter --all-features --locked
```
