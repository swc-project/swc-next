## SWC Next

SWC Next is a fast JavaScript and TypeScript toolchain written in Rust. It is a brand-new rewrite and redesign of [SWC](https://github.com/swc-project/swc), built around a flat, ESTree-based AST for performance and extensibility.

- **Performance:** Compact AST storage and optimized components, enabling 2x faster parsing and transformations than SWC.
- **JavaScript Extensibility:** Fast AST round trips between Rust and JavaScript.
- **Correctness:** Tested against Test262, TypeScript, and Babel, supplemented by additional unit tests.

This project is still in a very early stage; please continue to use SWC for production.

## Roadmap

See https://github.com/swc-project/swc-next/issues/181

## Design & Architecture

See our [docs](/docs/)

## Soundness

The current Rust AST API has known soundness issues:

- Node IDs and typed handles are not tied to their owning `Ast`. Using a handle with a different AST can access unrelated nodes or cause undefined behavior through unchecked access, even through safe APIs.

## Compatibility

SWC Next aims to provide capabilities while supporting parts of SWC's [`.swcrc` configuration](https://swc.rs/docs/configuration/swcrc). Its Rust and JavaScript APIs will not be compatible with SWC's.

This is planned but has not been implemented yet.

## Credits

Thanks to:

- The [yuku](https://github.com/yuku-toolchain/yuku), which provides the prototype of ESTree AST design, parser and semantic analysis.
- The [zig](https://github.com/ziglang/zig) compiler and the [rowan](https://github.com/rust-analyzer/rowan), which inspire the flattening AST design.
- The [oxc](https://github.com/oxc-project/oxc), which provides high-performance arena data structures, regular expression parser, and inspires the ast-tools / test harness design.
- The [esbuild](https://github.com/evanw/esbuild), which provides the prototype of Code generator.

## License

SWC Next's own code is licensed under the Apache License, Version 2.0, as is [SWC](https://github.com/swc-project/swc). Included third-party code and data retain their upstream license terms; see [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).

See the [LICENSE](LICENSE) file for more details.
