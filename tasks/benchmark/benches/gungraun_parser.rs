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
const JQUERY_MOBILE: &str = include_str!("../files/jquery.mobile-1.4.2.js");
const MOOTOOLS: &str = include_str!("../files/mootools-1.4.5.js");
const UNDERSCORE: &str = include_str!("../files/underscore-1.5.2.js");
const THREE: &str = include_str!("../files/three-0.138.3.js");
const YUI: &str = include_str!("../files/yui-3.12.0.js");
const TYPESCRIPT: &str = include_str!("../files/typescript.js");
const CAL_COM_TSX: &str = include_str!("../files/cal.com.tsx");

#[library_benchmark]
#[bench::angular(ANGULAR)]
#[bench::backbone(BACKBONE)]
#[bench::jquery(JQUERY)]
#[bench::jquery_mobile(JQUERY_MOBILE)]
#[bench::mootools(MOOTOOLS)]
#[bench::underscore(UNDERSCORE)]
#[bench::three(THREE)]
#[bench::yui(YUI)]
#[bench::typescript(TYPESCRIPT)]
#[bench::cal_com_tsx(CAL_COM_TSX)]
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
