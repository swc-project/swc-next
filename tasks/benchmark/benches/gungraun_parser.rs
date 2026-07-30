#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use gungraun::prelude::*;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Ast;
use swc_next_ecma_parser::{Options, parse};

const ANGULAR: &str = include_str!("../files/angular-1.2.5.js");
const BACKBONE: &str = include_str!("../files/backbone-1.1.0.js");
const JQUERY: &str = include_str!("../files/jquery-1.9.1.js");

#[library_benchmark]
#[bench::angular(ANGULAR)]
#[bench::backbone(BACKBONE)]
#[bench::jquery(JQUERY)]
fn bench_parser(source: &str) {
    let source = black_box(source);
    let allocator = Allocator::new();
    let mut ast = Ast::new(source.as_bytes(), &allocator);
    black_box(parse(&mut ast, source.as_bytes(), Options::default()));
}

library_benchmark_group!(
    name = parser;
    benchmarks = bench_parser
);

main!(library_benchmark_groups = parser);
