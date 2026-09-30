use crate::{expect_printed_ts, expect_printed_ts_minify};

#[track_caller]
fn expect_token_boundary(source: &str, expected: &str) {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Lang;
    use swc_next_ecma_codegen::{Options as PrintOptions, print};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        lang: Lang::Ts,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());
    let print_options = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };
    let compact = print(&allocator, &original.ast, print_options.clone());
    assert_eq!(compact.code, expected);

    let reparsed =
        Parser::init(&allocator, compact.code, parse_options, NoTokenParserConfig).parse();
    assert!(reparsed.diagnostics.is_empty());
    let original_pretty = print(&allocator, &original.ast, PrintOptions::default());
    let reparsed_pretty = print(&allocator, &reparsed.ast, PrintOptions::default());
    assert_eq!(reparsed_pretty.code, original_pretty.code);
    let repeated = print(&allocator, &reparsed.ast, print_options);
    assert_eq!(repeated.code, compact.code);
}

#[test]
fn test_non_null_token_boundaries() {
    expect_token_boundary("x! == y", "x! ==y;");

    expect_token_boundary("x! === y", "x! ===y;");

    expect_token_boundary("x! = y", "x! =y;");

    expect_token_boundary("x!! == y", "x!! ==y;");

    expect_token_boundary("x!! = y", "x!! =y;");

    expect_token_boundary("x! != y", "x!!=y;");

    expect_token_boundary("x! += y", "x!+=y;");
}

#[test]
fn test_type_assertion_token_boundaries() {
    expect_token_boundary("x < <T>y", "x< <T>y;");

    expect_token_boundary("x < <T><U>y", "x< <T><U>y;");

    expect_token_boundary("x << <T>y", "x<<<T>y;");

    expect_token_boundary(
        "const f = < <T>(value: T) => T > identity",
        "const f=< <T>(value:T) => T>identity;",
    );
}

#[test]
fn test_type_argument_token_boundaries() {
    expect_token_boundary("f<T> == x", "f<T> ==x;");

    expect_token_boundary("f<T> === x", "f<T> ===x;");

    expect_token_boundary("f<T> >= x", "f<T> >=x;");

    expect_token_boundary("f<T> >> x", "f<T> >>x;");

    expect_token_boundary("f<T> >>> x", "f<T> >>>x;");

    expect_token_boundary("f<Foo<T>> == x", "f<Foo<T>> ==x;");

    expect_token_boundary("f<T> * x", "f<T>*x;");

    expect_token_boundary("f<T> != x", "f<T>!=x;");

    expect_token_boundary("f<T>() == x", "f<T>()==x;");

    expect_token_boundary("f< <T>() => T>()", "f< <T>() => T>();");
}

#[test]
fn test_new_expression() {
    expect_printed_ts(
        "new parent.components!.KeepAlive();",
        "new parent.components!.KeepAlive();\n",
    );
}

#[test]
fn test_export_default() {
    expect_printed_ts(
        "export default function(): void;",
        "export default function(): void;\n",
    );
    expect_printed_ts(
        "export default async function<T>(): Promise<T>;",
        "export default async function<T>(): Promise<T>;\n",
    );
}

#[test]
fn test_import() {
    expect_printed_ts("import type {} from \"m\"", "import type {} from \"m\";\n");
}

#[test]
fn test_import_equals() {
    expect_printed_ts(
        "import value = require('value')",
        "import value = require(\"value\");\n",
    );
    expect_printed_ts(
        "export import value = require('value')",
        "export import value = require(\"value\");\n",
    );
}

#[test]
fn test_export_declarations() {
    expect_printed_ts("export type T = string;", "export type T = string;\n");
    expect_printed_ts(
        "export interface I { value: string }",
        "export interface I {\n  value: string;\n}\n",
    );
    expect_printed_ts(
        "export declare const value: string;",
        "export declare const value: string;\n",
    );
    expect_printed_ts(
        "export declare function fn(): void;",
        "export declare function fn(): void;\n",
    );
    expect_printed_ts(
        "export declare class C { value: string }",
        "export declare class C {\n  value: string;\n}\n",
    );
    expect_printed_ts(
        "export declare enum E { A }",
        "export declare enum E {\n  A,\n}\n",
    );
    expect_printed_ts(
        "export declare namespace N { const value: string }",
        "export declare namespace N {\n  const value: string;\n}\n",
    );
    expect_printed_ts(
        "export type { T } from 'module';",
        "export type { T } from \"module\";\n",
    );
    expect_printed_ts(
        "export { type T } from 'module';",
        "export { type T } from \"module\";\n",
    );
}

#[test]
fn test_optional_and_rest_tuple_elements() {
    expect_printed_ts(
        "type Tuple = [string?, ...number[]]",
        "type Tuple = [string?, ...number[]];\n",
    );
    expect_printed_ts(
        "type Tuple = [first: string, second?: number, ...rest: boolean[]]",
        "type Tuple = [first: string, second?: number, ...rest: boolean[]];\n",
    );
}

#[test]
fn test_enum_declarations_do_not_emit_semicolons() {
    expect_printed_ts("enum E {}", "enum E {}\n");
    expect_printed_ts(
        "declare const enum E { A = 1 }",
        "declare const enum E {\n  A = 1,\n}\n",
    );
    expect_printed_ts("export enum E { A }", "export enum E {\n  A,\n}\n");
    expect_printed_ts(
        "namespace N { enum E { A } }",
        "namespace N {\n  enum E {\n    A,\n  }\n}\n",
    );
}

#[test]
fn test_enum_member_names() {
    expect_printed_ts("enum E { [`x`] }", "enum E {\n  [`x`],\n}\n");

    expect_printed_ts_minify("enum E { [`\\u0078\\n`] = 1 }", "enum E{[`x\n`]=1,}");
}

#[test]
fn test_computed_enum_name_comments() {
    expect_printed_ts(
        "enum E { [/* @__PURE__ */ 'x'] = make() }",
        "enum E {\n  [ /* @__PURE__ */ \"x\"] = make(),\n}\n",
    );

    expect_printed_ts_minify(
        "enum E { [/* @__PURE__ */ `x`] = make() }",
        "enum E{[ /* @__PURE__ */ `x`]=make(),}",
    );
}

#[test]
fn test_type_assertion_generic_function_type() {
    let type_assertion = "const f = < <T>(value: T) => T > identity;";
    let type_assertion_expected = "const f = < <T>(value: T) => T>identity;\n";
    expect_printed_ts(type_assertion, type_assertion_expected);
    expect_printed_ts(type_assertion_expected, type_assertion_expected);
    expect_printed_ts_minify(type_assertion, "const f=< <T>(value:T) => T>identity;");
}

#[test]
fn test_satisfies_assignment_targets() {
    expect_printed_ts(
        "++(value satisfies number);",
        "++(value satisfies number);\n",
    );
    expect_printed_ts(
        "++(value satisfies number);\n",
        "++(value satisfies number);\n",
    );
    expect_printed_ts_minify("++(value satisfies number);", "++(value satisfies number);");
    expect_printed_ts(
        "(value satisfies number)--;",
        "(value satisfies number)--;\n",
    );
    expect_printed_ts(
        "(value satisfies number)--;\n",
        "(value satisfies number)--;\n",
    );
    expect_printed_ts_minify("(value satisfies number)--;", "(value satisfies number)--;");
    expect_printed_ts(
        "(value satisfies number) = 1;",
        "(value satisfies number) = 1;\n",
    );
    expect_printed_ts(
        "(value satisfies number) = 1;\n",
        "(value satisfies number) = 1;\n",
    );
    expect_printed_ts_minify(
        "(value satisfies number) = 1;",
        "(value satisfies number)=1;",
    );
    expect_printed_ts(
        "[(value satisfies number)] = [1];",
        "[(value satisfies number)] = [1];\n",
    );
    expect_printed_ts(
        "[(value satisfies number)] = [1];\n",
        "[(value satisfies number)] = [1];\n",
    );
    expect_printed_ts_minify(
        "[(value satisfies number)] = [1];",
        "[(value satisfies number)]=[1];",
    );
    expect_printed_ts(
        "({ key: (value satisfies number) } = { key: 1 });",
        "({ key: (value satisfies number) } = { key: 1 });\n",
    );
    expect_printed_ts(
        "({ key: (value satisfies number) } = { key: 1 });\n",
        "({ key: (value satisfies number) } = { key: 1 });\n",
    );
    expect_printed_ts_minify(
        "({ key: (value satisfies number) } = { key: 1 });",
        "({key:(value satisfies number)}={key:1});",
    );
    expect_printed_ts(
        "for ((value satisfies number) of values) {}",
        "for ((value satisfies number) of values) {\n}\n",
    );
    expect_printed_ts(
        "for ((value satisfies number) of values) {\n}\n",
        "for ((value satisfies number) of values) {\n}\n",
    );
    expect_printed_ts_minify(
        "for ((value satisfies number) of values) {}",
        "for((value satisfies number) of values){}",
    );
    expect_printed_ts(
        "for ((value satisfies number) in values) {}",
        "for ((value satisfies number) in values) {\n}\n",
    );
    expect_printed_ts(
        "for ((value satisfies number) in values) {\n}\n",
        "for ((value satisfies number) in values) {\n}\n",
    );
    expect_printed_ts_minify(
        "for ((value satisfies number) in values) {}",
        "for((value satisfies number) in values){}",
    );
}

#[test]
fn test_non_finite_numeric_literal_types() {
    let cases = [
        (
            "type Positive = 1e999;",
            "type Positive = 1e999;\n",
            "type Positive=1e999;",
        ),
        (
            "type Negative = -1e999;",
            "type Negative = -1e999;\n",
            "type Negative=-1e999;",
        ),
    ];

    for (source, expected, minified) in cases {
        expect_printed_ts(source, expected);
        expect_printed_ts(expected, expected);
        expect_printed_ts_minify(source, minified);
    }
}

#[test]
fn test_signed_numeric_literal_type_comments() {
    let source = "type T = -/* note */1;";
    let expected = "type T = /* note */ -1;\n";
    expect_printed_ts(source, expected);
    expect_printed_ts(expected, expected);
    expect_printed_ts_minify(source, "type T=/* note */-1;");
}

#[test]
fn test_legal_comments() {
    expect_printed_ts(
        "type X = number /*! license */;",
        "type X = number;\n/*! license */\n",
    );
    expect_printed_ts_minify(
        "type X = number /*! license */;",
        "type X=number;/*! license */",
    );
}
