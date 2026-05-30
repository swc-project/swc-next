#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_core::common::{BytePos, GLOBALS, Globals, Mark};
use swc_experimental_allocator::{Allocator, boxed::Box as AstBox};
use swc_experimental_ecma_ast::Program;

fn bench_legacy(b: &mut Bencher, src: &'static str) {
    use swc_core::ecma::parser::{Parser, StringInput, Syntax, lexer::Lexer};
    use swc_core::ecma::transforms::base::resolver;
    use swc_core::ecma::visit::VisitMut;
    GLOBALS.set(&Globals::new(), || {
        b.iter(|| {
            let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32));
            let lexer = Lexer::new(
                Syntax::Es(Default::default()),
                Default::default(),
                input,
                None,
            );
            let mut parser = Parser::new_from(lexer);
            let mut module = parser.parse_module().unwrap();
            resolver(Mark::new(), Mark::new(), false).visit_mut_module(&mut module);
        });
    });
}

fn bench_new(b: &mut Bencher, src: &'static str) {
    use swc_experimental_ecma_parser::Parser;
    use swc_experimental_ecma_parser::StringSource;
    use swc_experimental_ecma_semantic::resolver::resolver;

    b.iter(|| {
        let allocator = Allocator::new();
        let input = StringSource::new(src);
        let mut parser = Parser::new(
            &allocator,
            swc_experimental_ecma_parser::Syntax::Es(Default::default()),
            input,
            None,
        );
        let module = parser.parse_module().unwrap();
        let program = Program::Module(allocator.boxed(module));

        let semantic = resolver(&program);
        match program {
            Program::Module(module) => {
                swc_experimental_ecma_ast_compat::AstCompat::new(&semantic)
                    .compat_module(AstBox::into_inner(module));
            }
            Program::Script(_) => unreachable!(),
        }
    });
}

fn bench_new_unsafe(b: &mut Bencher, src: &'static str) {
    use swc_experimental_ecma_parser::Parser;
    use swc_experimental_ecma_parser::StringSource;
    use swc_experimental_ecma_semantic::resolver::resolver;

    b.iter(|| {
        let allocator = Allocator::new();
        let input = StringSource::new(src);
        let mut parser = Parser::new(
            &allocator,
            swc_experimental_ecma_parser::Syntax::Es(Default::default()),
            input,
            None,
        );
        let module = parser.parse_module().unwrap();
        let program = Program::Module(allocator.boxed(module));

        let semantic = resolver(&program);
        match program {
            Program::Module(module) => std::hint::black_box(
                swc_experimental_ecma_ast_compat::UnsafeArenaAstCompat::new(&semantic)
                    .compat_module(AstBox::into_inner(module)),
            ),
            Program::Script(_) => unreachable!(),
        }
    });
}

fn bench_files(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    for (name, source) in bench_cases {
        c.bench_function(&format!("{name}/ast_compat/legacy"), |b| {
            bench_legacy(b, source)
        });
        c.bench_function(&format!("{name}/ast_compat/new"), |b| bench_new(b, source));
        c.bench_function(&format!("{name}/ast_compat/new_unsafe"), |b| {
            bench_new_unsafe(b, source)
        });
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
