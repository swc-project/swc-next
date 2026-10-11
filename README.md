<p align="center">
  <a href="https://swc.rs/">
    <img alt="SWC" src="./docs/images/swc.png" width="546">
  </a>
</p>

<p align="center">
  <a href="https://discord.com/invite/GnHbXTdZz6"><img alt="Discord" src="https://img.shields.io/discord/889779439272075314?label=Discord&amp;logo=discord"></a>
  <a href="https://www.npmjs.com/package/@swc-next/core"><img alt="npm version (@swc-next/core)" src="https://img.shields.io/npm/v/@swc-next/core"></a>
  <a href="https://crates.io/crates/swc_next_ecma_parser"><img alt="crates.io version (swc_next_ecma_parser)" src="https://img.shields.io/crates/v/swc_next_ecma_parser"></a>
  <a href="./LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue"></a>
</p>

# SWC Next

SWC Next is a fast JavaScript and TypeScript toolchain written in Rust. It is a rewrite of [SWC](https://github.com/swc-project/swc) from the ground up, built around a flat, ESTree-based AST for performance and extensibility.

- **Performance:** Compact AST storage and optimized components, enabling 2x faster parsing and transformations than SWC.
- **JavaScript Extensibility:** Fast AST round trips between Rust and JavaScript enable writing plugins in JavaScript.
- **Correctness:** Tested against Test262, TypeScript, and Babel, supplemented by additional unit tests.

This project is still in a very early stage; please continue to use SWC for production.

## Roadmap

See the [project roadmap](https://github.com/swc-project/swc-next/issues/181).

## Design & Architecture

See the [Flattening AST design](docs/FlatteningAST.md) and other documents.

## Soundness

The current Rust AST API has known soundness issues:

- Node IDs and typed handles are not tied to their owning `Ast`. Using a handle with a different AST can access unrelated nodes or cause undefined behavior through unchecked access, even through safe APIs.

## Compatibility

SWC Next plans to support selected SWC [`.swcrc` options](https://swc.rs/docs/configuration/swcrc). Its Rust and JavaScript APIs will not be compatible with SWC's.

Configuration compatibility has not been implemented yet.

## Credits

Thanks to:

- [Yuku](https://github.com/yuku-toolchain/yuku), whose AST definitions, parser, and semantic analysis form the basis of the corresponding SWC Next components, with modifications for SWC Next.
- The [Zig](https://codeberg.org/ziglang/zig) compiler and [rowan](https://github.com/rust-analyzer/rowan), which inspire the flattening AST design.
- [Oxc](https://github.com/oxc-project/oxc), whose arena data structures and regular expression parser form the basis of our implementations, with modifications for SWC Next. Its AST tooling and test harness also inform our design.
- [esbuild](https://github.com/evanw/esbuild), whose printer implementation and codegen tests form the basis of ours, with modifications for SWC Next.

## License

SWC Next's own code is licensed under the Apache License, Version 2.0, as is [SWC](https://github.com/swc-project/swc). Included third-party code and data retain their upstream license terms; see [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).

See the [LICENSE](LICENSE) file for more details.
