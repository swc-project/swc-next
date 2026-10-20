use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Expr, NodeKind};
use swc_next_ecma_parser::{Diagnostic, Lang, NoTokenParserConfig, Options, Parser};

#[test]
fn type_argument_speculation() {
    fn parse<T: Send + 'static>(
        source: String,
        options: Options,
        inspect: fn(&Ast<'_>) -> T,
    ) -> (Vec<Diagnostic<'static>>, T) {
        std::thread::Builder::new()
            .stack_size(2 << 20)
            .spawn(move || {
                let allocator = Allocator::new();
                let parsed =
                    Parser::init(&allocator, &source, options, NoTokenParserConfig).parse();
                let result = inspect(&parsed.ast);
                (parsed.diagnostics, result)
            })
            .unwrap()
            .join()
            .unwrap()
    }

    fn expression(ast: &Ast<'_>) -> Expr {
        ast.root_program()
            .body(ast)
            .get_node(ast, 0)
            .unwrap()
            .as_expression_statement(ast)
            .unwrap()
            .expression(ast)
    }

    fn array_length(ast: &Ast<'_>) -> usize {
        expression(ast)
            .as_array_expression(ast)
            .unwrap()
            .elements(ast)
            .len()
    }

    fn arguments(ast: &Ast<'_>) -> (usize, NodeKind) {
        let call = expression(ast).as_call_expression(ast).unwrap();
        let arguments = call.arguments(ast);
        let last = arguments.get_node(ast, arguments.len() - 1).unwrap();
        (arguments.len(), ast.node_kind(last.node_id()))
    }

    fn type_argument_depth(ast: &Ast<'_>) -> usize {
        let call = expression(ast).as_call_expression(ast).unwrap();
        let mut arguments = call.type_arguments(ast);
        let mut depth = 0;
        while let Some(list) = arguments {
            depth += 1;
            let ty = list.params(ast).get_node(ast, 0).unwrap();
            arguments = ty.as_ts_type_reference(ast).unwrap().type_arguments(ast);
        }
        depth
    }

    let typescript = Options {
        lang: Lang::Ts,
        ..Options::default()
    };
    let tsx = Options {
        lang: Lang::Tsx,
        ..typescript
    };
    let mut computed_keys = "x".to_owned();
    for _ in 0..24 {
        computed_keys = format!("a<{{[{computed_keys}]:1}}");
    }
    computed_keys.push(';');

    for options in [typescript, tsx] {
        let source = format!("[{}];", "a < b, ".repeat(100_000));
        let (errors, length) = parse(source, options, array_length);
        assert!(errors.is_empty());
        assert_eq!(length, 100_000);

        let source = format!("f({}1);", "i < n, j < m, ".repeat(10_000));
        let (errors, args) = parse(source, options, arguments);
        assert!(errors.is_empty());
        assert_eq!(args, (20_001, NodeKind::NumericLiteral));

        let source = format!("[{}];", "a < 1, ".repeat(20_000));
        let (errors, length) = parse(source, options, array_length);
        assert!(errors.is_empty());
        assert_eq!(length, 20_000);

        let source = format!("f({}g<A<B>>(c));", "i < n, ".repeat(20_000));
        let (errors, args) = parse(source, options, arguments);
        assert!(errors.is_empty());
        assert_eq!(args, (20_001, NodeKind::CallExpression));

        let source = format!(
            "f({}g<{{[(/>/).source]: string}}>(c));",
            "i < n, ".repeat(20_000)
        );
        let (errors, args) = parse(source, options, arguments);
        assert!(errors.is_empty());
        assert_eq!(args, (20_001, NodeKind::CallExpression));

        let source = format!("f<{}a{}>();", "A<".repeat(99), ">".repeat(99));
        let (errors, depth) = parse(source, options, type_argument_depth);
        assert!(errors.is_empty());
        assert_eq!(depth, 100);

        let source = format!("f<{}a{}>();", "A<".repeat(9_999), ">".repeat(9_999));
        let (errors, depth) = parse(source, options, type_argument_depth);
        assert!(errors.is_empty());
        assert_eq!(depth, 10_000);

        let (errors, ()) = parse(computed_keys.clone(), options, |_| ());
        assert!(errors.is_empty());
    }
}
