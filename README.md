## SWC Next

This project is still in a very early stage; please continue to use [SWC](https://github.com/swc-project/swc).

This project is the next generation of [SWC](https://github.com/swc-project/swc). The primary modification, based on SWC, is the redesign of the AST data structure:

1. It adopts a Flattening AST design, which allows for a higher performance.
2. The AST structure (fields) of SWC Next remains largely consistent with that of SWC, making migration relatively straightforward.

## Roadmap

There's a long way to go, but luckly it could be progressive.

- [x] Flattening AST design
- [x] Parser, Visit, and Transformation APIs.
- [ ] Port SWC minifier.
- [ ] Port SWC transformation.

## Design & Architecture

See our [docs](/docs/)

## Credits

Thanks to:

- The [yuku](https://github.com/yuku-toolchain/yuku), which provides the prototype of ESTree AST design, parser and semantic analysis.
- The [zig](https://github.com/ziglang/zig) compiler and the [rowan](https://github.com/rust-analyzer/rowan), which inspire the flattening AST design.
- The [oxc](https://github.com/oxc-project/oxc), which provides high-performance arena data structures, regular expression parser, and inspires the ast-tools / test harness design.
- The [esbuild](https://github.com/evanw/esbuild), which provides the prototype of Code generator.

## License

SWC Next's own code is licensed under the Apache License, Version 2.0, as is [SWC](https://github.com/swc-project/swc). Included third-party code and data retain their upstream license terms; see [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).

See the [LICENSE](LICENSE) file for more details.
