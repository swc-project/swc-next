## SWC (Experimental)

This project is still in a very early stage; please continue to use [SWC](https://github.com/swc-project/swc).

This project is an experimental version of [SWC](https://github.com/swc-project/swc). The primary modification, based on SWC, is the redesign of the AST data structure:

1. It adopts a Flattening AST design, which allows for a higher performance.
2. The AST structure (fields) of the SWC (experimental) remains largely consistent with that of SWC, making migration relatively straightforward.

## Performance

See [codspeed](https://codspeed.io/CPunisher/swc-experimental/benchmarks) for comparison.

Local benchmark results with M3Pro:

- Parser: **1.2x-1.5x** faster.
- Pre-order visit: No apparent difference.
- Post-order visit: Compared to legacy pre-order visitor, **20x** faster.

## Roadmap

There's a long way to go, but luckly it could be progressive.

- [x] Flattening AST design
- [x] Port SWC js parser.
- [x] Port SWC jsx parser.
- [ ] Port SWC TypeScript parser.
- [ ] Port SWC minifier.
- [ ] Port SWC transformation.

## Design & Architecture

See our [docs](/docs/)

## Credits

Thanks to:

- The [zig](https://github.com/ziglang/zig) compiler and the [rowan](https://github.com/rust-analyzer/rowan), which inspire the flattening AST design.
- The [oxc](https://github.com/oxc-project/oxc), which inspires the ast-tools design.

## License

SWC (Experimental) is licensed under the Apache License, Version 2.0, which should be always same as [SWC](https://github.com/swc-project/swc).

See the [LICENSE](LICENSE) file for more details.
