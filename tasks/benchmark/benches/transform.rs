#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{
    Ast, ParenthesizedExpression, Program, Traverse, TraverseCtx, TraverseWith,
};
use swc_experimental_ecma_parser::{Options, parse};

fn remove_parens(program: Program, ast: &mut Ast<'_>) {
    struct RemoveParens;

    impl<'a> Traverse<'a> for RemoveParens {
        fn exit_parenthesized_expression(
            &mut self,
            node: ParenthesizedExpression,
            ctx: &mut TraverseCtx<'_, 'a>,
        ) {
            let expression = node.expression(ctx.ast());
            ctx.ast_mut().set_data(node, expression);
        }
    }

    let _ = program.traverse_with(&mut RemoveParens, &mut TraverseCtx::new(ast));
}

fn bench_transform(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/transform/remove_paren"), |b| {
            let allocator = Allocator::new();
            b.iter_batched(
                || {
                    let mut ast = Ast::new(source.as_bytes(), &allocator);
                    let program = parse(&mut ast, source.as_bytes(), Options::default()).unwrap();
                    (program, ast)
                },
                |(program, mut ast)| {
                    remove_parens(program, &mut ast);
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(transform, bench_transform);
criterion_main!(transform);
