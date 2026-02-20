#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::rc::Rc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_core::common::{BytePos, GLOBALS, Globals, Mark};
use swc_experimental_ecma_ast::Ast;

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
        let input = StringSource::new(src);
        let mut ast = Ast::new(input.source_len(), Rc::default());
        let mut parser = Parser::new(
            &mut ast,
            swc_experimental_ecma_parser::Syntax::Es(Default::default()),
            input,
            None,
        );
        let module = parser.parse_module().unwrap();

        let semantic = resolver(module, &ast);
        swc_experimental_ecma_ast_compat::AstCompat::new(&ast, &semantic).compat_module(module);
    });
}

fn bench_files(c: &mut Criterion) {
    let bench_cases = &[
        ("colors", include_str!("../files/colors.js")),
        ("angular", include_str!("../files/angular-1.2.5.js")),
        ("backbone", include_str!("../files/backbone-1.1.0.js")),
        ("jquery", include_str!("../files/jquery-1.9.1.js")),
        (
            "jquery.mobile",
            include_str!("../files/jquery.mobile-1.4.2.js"),
        ),
        ("mootools", include_str!("../files/mootools-1.4.5.js")),
        ("underscore", include_str!("../files/underscore-1.5.2.js")),
        ("three", include_str!("../files/three-0.138.3.js")),
        ("yui", include_str!("../files/yui-3.12.0.js")),
        ("typescript", include_str!("../files/typescript.js")),
    ];

    for (name, source) in bench_cases {
        c.bench_function(&format!("{name}/ast_compat/legacy"), |b| {
            bench_legacy(b, source)
        });
        c.bench_function(&format!("{name}/ast_compat/new"), |b| bench_new(b, source));
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
