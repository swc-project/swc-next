## SWC (Experimental)

This project is still in a very early stage; please continue to use [SWC](https://github.com/swc-project/swc).

This project is an experimental version of [SWC](https://github.com/swc-project/swc). The primary modification, based on SWC, is the redesign of the AST data structure:

1. It adopts arena-based AST design, which allows for a higher performance.
2. The AST structure (fields) of the SWC (experimental) remains largely consistent with that of SWC, making migration relatively straightforward.

## Performance

See [codspeed](https://codspeed.io/CPunisher/swc-experimental/benchmarks) for details.

Local benchmark results with M3Pro:

- Parser: **~1.5x** faster.
- Visit: **~1.3x** faster.

## Roadmap

There's a long way to go, but luckly it could be progressive.

- [x] Arena-based AST design
- [x] Port SWC js parser.
- [x] Port SWC jsx parser.
- [ ] Port SWC TypeScript parser.
- [ ] Port SWC minifier.
- [ ] Port SWC transformation.

## Design & Architecture

See our [docs](/docs/)

## Credits

Thanks to:

- The [oxc](https://github.com/oxc-project/oxc), which inspires the arena-based ast design and ast-tools design.

## License

SWC (Experimental) is licensed under the Apache License, Version 2.0, which should be always same as [SWC](https://github.com/swc-project/swc).

See the [LICENSE](LICENSE) file for more details.
