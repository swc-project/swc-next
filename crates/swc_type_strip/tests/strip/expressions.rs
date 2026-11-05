use super::*;

const CHAIN_LENGTH: usize = 50_000;

fn strip_with_small_stack(source: String) -> (String, usize) {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(move || {
            let result = strip_types(&source, options(Lang::Ts));
            (result.code.into_owned(), result.diagnostics.len())
        })
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn assertions() {
    assert_strip("const x = value as string;", "const x = value          ;");

    assert_strip(
        "const value = [1, 2] as const;",
        "const value = [1, 2]         ;",
    );

    assert_strip(
        "const x = value satisfies Type;",
        "const x = value               ;",
    );

    assert_strip("const x = value!;", "const x = value ;");

    assert_strip("const value = expr!!!;", "const value = expr   ;");

    assert_strip(
        "const value = (expr as A)! satisfies B;",
        "const value = (expr     )             ;",
    );

    assert_strip_error_at(
        "const x = <number>value;",
        "const x =         value;",
        "<number>",
    );

    assert_strip(
        "const value = input as unknown satisfies Type;",
        "const value = input                          ;",
    );

    assert_strip(
        "let value = ['csv', 'json'] as (`csv` | `json`)[];",
        "let value = ['csv', 'json']                      ;",
    );

    assert_strip(
        "const value = 1 * 1 as number + 2;",
        "const value = 1 * 1           + 2;",
    );

    assert_strip(
        "const value = 2 * 3 as number * 2;",
        "const value = 2 * 3           * 2;",
    );

    let ts = options(Lang::Ts);
    let result = strip_types("const value = -input as number ** 2;", ts);
    assert_eq!(result.code, "const value = -input           ** 2;");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
    assert_eq!(result.diagnostics[0].span, Span::new(14, 20));

    let result = strip_types("await input as any satisfies number ** 2;", ts);
    assert_eq!(result.code, "await input                         ** 2;");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
    assert_eq!(result.diagnostics[0].span, Span::new(0, 11));

    assert_strip(
        "const value = (-input) as number ** 2;",
        "const value = (-input)           ** 2;",
    );

    assert_strip(
        "const value = (await input) as number ** 2;",
        "const value = (await input)           ** 2;",
    );

    assert_strip("value as T\n/re/g.test(s);", "value     \n/re/g.test(s);");

    assert_strip("value!\n/'re'/g;", "value \n/'re'/g;");

    assert_strip("`${value as T}}`;", "`${value     }}`;");
}

#[test]
fn type_arguments() {
    assert_strip_error_at("hello<T>.world<U>;", "hello   .world   ;", ".");

    assert_strip("(hello<T>).world<U>;", "(hello   ).world   ;");

    assert_strip(
        "const value = (fn<A>)! as B;",
        "const value = (fn   )      ;",
    );

    assert_strip(
        "const value = (make!)<Type>;",
        "const value = (make )      ;",
    );

    assert_strip(
        "const value = (fn!)<T>(argument!);",
        "const value = (fn )   (argument );",
    );

    assert_strip("fn!?.<Type>(value);", "fn ?.      (value);");

    assert_strip(
        "const value = new (Map!)<Key, Value>();",
        "const value = new (Map )            ();",
    );

    assert_strip("@decorate<Type>\nclass C {}", "@decorate      \nclass C {}");
}

#[test]
fn assignment_targets() {
    assert_strip("[left!, right!] = values;", "[left , right ] = values;");

    assert_strip("[left as Type] = values;", "[left        ] = values;");

    assert_strip(
        "[left satisfies Type] = values;",
        "[left               ] = values;",
    );
}

#[test]
fn jsx() {
    assert_strip_tsx(
        "const element = <div>{value as string}</div>;",
        "const element = <div>{value          }</div>;",
    );

    assert_strip_tsx(
        "const element = <Component<Type> />;",
        "const element = <Component       />;",
    );

    assert_strip_tsx(
        "const element = <Component render={(value: Type) => value} />;",
        "const element = <Component render={(value      ) => value} />;",
    );
}

#[test]
fn literals() {
    assert_strip(
        r"const value = [/as\s+Type/, input as Type];",
        r"const value = [/as\s+Type/, input        ];",
    );

    assert_strip(
        "const value = `as Type ${input as Type}`;",
        "const value = `as Type ${input        }`;",
    );
}

#[test]
fn expression_chains() {
    let initializer = vec!["a"; CHAIN_LENGTH].join("+");
    let source = format!("declare namespace N {{ const value = {initializer} + <number>1; }}");
    let expected = format!(";{}", " ".repeat(source.len() - 1));
    let (code, diagnostics) = strip_with_small_stack(source);
    assert_eq!(code, expected);
    assert_eq!(diagnostics, 1);

    let source = format!("x = {};", vec!["a"; CHAIN_LENGTH].join("+"));
    let (code, diagnostics) = strip_with_small_stack(source.clone());
    assert_eq!(code, source);
    assert_eq!(diagnostics, 0);

    let source = format!("x = {};", vec!["a"; CHAIN_LENGTH].join("||"));
    let (code, diagnostics) = strip_with_small_stack(source.clone());
    assert_eq!(code, source);
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", ".b".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source.clone());
    assert_eq!(code, source);
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", "[b]".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source.clone());
    assert_eq!(code, source);
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", "()".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source.clone());
    assert_eq!(code, source);
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", "?.b".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source.clone());
    assert_eq!(code, source);
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", "`x`".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source.clone());
    assert_eq!(code, source);
    assert_eq!(diagnostics, 0);
}

#[test]
fn type_expression_chains() {
    let source = format!("x = a{};", "!".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source);
    assert_eq!(code, format!("x = a{};", " ".repeat(CHAIN_LENGTH)));
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", " as any".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source);
    assert_eq!(code, format!("x = a{};", "       ".repeat(CHAIN_LENGTH)));
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", " satisfies any".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source);
    assert_eq!(
        code,
        format!("x = a{};", "              ".repeat(CHAIN_LENGTH))
    );
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", "<T>?.()".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source);
    assert_eq!(code, format!("x = a{};", "   ?.()".repeat(CHAIN_LENGTH)));
    assert_eq!(diagnostics, 0);

    let source = format!("x = a{};", "<T>(b!)[c as C]?.d!".repeat(CHAIN_LENGTH));
    let (code, diagnostics) = strip_with_small_stack(source);
    assert_eq!(
        code,
        format!("x = a{};", "   (b )[c     ]?.d ".repeat(CHAIN_LENGTH))
    );
    assert_eq!(diagnostics, 0);
}

#[test]
fn chain_children() {
    assert_strip(
        "x = f<A>(b! + c!, d as D)<B>(e!)[f as F]!.g satisfies G;\n",
        "x = f   (b  + c , d     )   (e )[f     ] .g            ;\n",
    );

    assert_strip(
        "x = tag<T>`a${b as B}`<U>`c${d!}`;\n",
        "x = tag   `a${b     }`   `c${d }`;\n",
    );

    assert_strip(
        "f<T>(a! as A)?.[b as B]?.(c satisfies C);\n",
        "f   (a      )?.[b     ]?.(c            );\n",
    );

    assert_strip("a as A as B\n(c)\n", "a     ;    \n(c)\n");
}
