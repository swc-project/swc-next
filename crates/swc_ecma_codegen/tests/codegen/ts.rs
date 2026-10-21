use crate::{
    expect_printed_pair, expect_printed_ts, expect_printed_ts_minify, expect_printed_ts_pair,
};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Lang;
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

#[track_caller]
fn expect_generic_parameters(source: &str, lang: Lang, pretty: &str, compact: &str) {
    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        lang,
        ..ParseOptions::default()
    };
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());

    for minify in [false, true] {
        let print_options = PrintOptions {
            minify,
            ..PrintOptions::default()
        };
        let printed = print(&original.ast, print_options.clone());
        assert_eq!(printed.code, if minify { compact } else { pretty });

        let reparsed = Parser::init(
            &allocator,
            &printed.code,
            parse_options,
            NoTokenParserConfig,
        )
        .parse();
        assert!(reparsed.diagnostics.is_empty());
        let repeated = print(&reparsed.ast, print_options);
        assert_eq!(repeated.code, printed.code);
    }
}

#[test]
fn test_tsx_generic_arrows() {
    expect_generic_parameters(
        "const f = <T,>(x: T) => x;",
        Lang::Tsx,
        "const f = <T,>(x: T) => x;\n",
        "const f=<T,>(x:T)=>x;",
    );

    expect_generic_parameters(
        "const f = <const T,>(x: T) => x;",
        Lang::Tsx,
        "const f = <const T,>(x: T) => x;\n",
        "const f=<const T,>(x:T)=>x;",
    );

    expect_generic_parameters(
        "const f = <T extends object>(x: T) => x;",
        Lang::Tsx,
        "const f = <T extends object,>(x: T) => x;\n",
        "const f=<T extends object,>(x:T)=>x;",
    );

    expect_generic_parameters(
        "const f = <T = string,>(x: T) => x;",
        Lang::Tsx,
        "const f = <T = string,>(x: T) => x;\n",
        "const f=<T=string,>(x:T)=>x;",
    );

    expect_generic_parameters(
        "const f = <const T extends object = {},>(x: T) => x;",
        Lang::Tsx,
        "const f = <const T extends object = {},>(x: T) => x;\n",
        "const f=<const T extends object={},>(x:T)=>x;",
    );

    expect_generic_parameters(
        "const f = async <T,>(x: T): Promise<T> => x;",
        Lang::Tsx,
        "const f = async <T,>(x: T): Promise<T> => x;\n",
        "const f=async<T,>(x:T):Promise<T>=>x;",
    );

    expect_generic_parameters(
        "const f = <T, U,>(x: T, y: U) => x;",
        Lang::Tsx,
        "const f = <T, U>(x: T, y: U) => x;\n",
        "const f=<T,U>(x:T,y:U)=>x;",
    );
}

#[test]
fn test_generic_parameter_contexts() {
    expect_generic_parameters(
        "const f = <T,>(x: T) => x;",
        Lang::Ts,
        "const f = <T>(x: T) => x;\n",
        "const f=<T>(x:T)=>x;",
    );

    expect_generic_parameters(
        "function f<T,>(x: T) {}",
        Lang::Tsx,
        "function f<T>(x: T) {\n}\n",
        "function f<T>(x:T){}",
    );

    expect_generic_parameters(
        "class C<T,> {}",
        Lang::Tsx,
        "class C<T> {\n}\n",
        "class C<T>{}",
    );

    expect_generic_parameters(
        "interface I<T,> {}",
        Lang::Tsx,
        "interface I<T> {}\n",
        "interface I<T>{}",
    );

    expect_generic_parameters(
        "type F = <T,>(x: T) => T;",
        Lang::Tsx,
        "type F = <T>(x: T) => T;\n",
        "type F=<T>(x:T) => T;",
    );
}

#[test]
fn test_type_aliases() {
    let source = "type T = (intrinsic);";
    expect_printed_ts(source, "type T = (intrinsic);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic);");

    let source = "type T = (intrinsic)[];";
    expect_printed_ts(source, "type T = (intrinsic[]);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic[]);");

    let source = "type T = (intrinsic)[\"x\"];";
    expect_printed_ts(source, "type T = (intrinsic[\"x\"]);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic[\"x\"]);");

    let source = "type T = (intrinsic) | string;";
    expect_printed_ts(source, "type T = (intrinsic | string);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic | string);");

    let source = "type T = (intrinsic) & string;";
    expect_printed_ts(source, "type T = (intrinsic & string);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic & string);");

    let source = "type T = (intrinsic) extends string ? 1 : 2;";
    expect_printed_ts(source, "type T = (intrinsic extends string ? 1 : 2);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic extends string ? 1 : 2);");

    let source = "type T = (intrinsic)[][number] | string;";
    expect_printed_ts(source, "type T = (intrinsic[][number] | string);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic[][number] | string);");

    let source = "type T = intrinsic;";
    expect_printed_ts(source, "type T = intrinsic;\n");
    expect_printed_ts_minify(source, "type T=intrinsic;");

    let source = "type T = (intrinsic.foo);";
    expect_printed_ts(source, "type T = intrinsic.foo;\n");
    expect_printed_ts_minify(source, "type T=intrinsic.foo;");

    let source = "type T = (intrinsic<string>);";
    expect_printed_ts(source, "type T = (intrinsic<string>);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic<string>);");

    let source = "type T = (intrinsic<string>)[];";
    expect_printed_ts(source, "type T = (intrinsic<string>[]);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic<string>[]);");

    let source = "type T = (intrinsic.foo<string>);";
    expect_printed_ts(source, "type T = intrinsic.foo<string>;\n");
    expect_printed_ts_minify(source, "type T=intrinsic.foo<string>;");

    let source = "type T = (intrinsic)?;";
    expect_printed_ts(source, "type T = ((intrinsic)?);\n");
    expect_printed_ts_minify(source, "type T=((intrinsic)?);");

    let source = "type T = (intrinsic)!;";
    expect_printed_ts(source, "type T = (intrinsic!);\n");
    expect_printed_ts_minify(source, "type T=(intrinsic!);");

    let source = "type T = ?(intrinsic);";
    expect_printed_ts(source, "type T = ?intrinsic;\n");
    expect_printed_ts_minify(source, "type T=?intrinsic;");

    let source = "type T = !(intrinsic);";
    expect_printed_ts(source, "type T = !intrinsic;\n");
    expect_printed_ts_minify(source, "type T=!intrinsic;");

    let source = "type T = string | (intrinsic);";
    expect_printed_ts(source, "type T = string | intrinsic;\n");
    expect_printed_ts_minify(source, "type T=string | intrinsic;");

    let source = "type T = keyof (intrinsic);";
    expect_printed_ts(source, "type T = keyof intrinsic;\n");
    expect_printed_ts_minify(source, "type T=keyof intrinsic;");

    let source = "type T = Array<(intrinsic)>;";
    expect_printed_ts(source, "type T = Array<intrinsic>;\n");
    expect_printed_ts_minify(source, "type T=Array<intrinsic>;");
}

#[track_caller]
fn expect_token_boundary(source: &str, expected: &str) {
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
    let compact = print(&original.ast, print_options.clone());
    assert_eq!(compact.code, expected);

    let reparsed = Parser::init(
        &allocator,
        &compact.code,
        parse_options,
        NoTokenParserConfig,
    )
    .parse();
    assert!(reparsed.diagnostics.is_empty());
    let original_pretty = print(&original.ast, PrintOptions::default());
    let reparsed_pretty = print(&reparsed.ast, PrintOptions::default());
    assert_eq!(reparsed_pretty.code, original_pretty.code);
    let repeated = print(&reparsed.ast, print_options);
    assert_eq!(repeated.code, compact.code);
}

#[test]
fn test_type_members() {
    expect_token_boundary(
        "interface A { x: string; y: number; }",
        "interface A{x:string;y:number}",
    );

    expect_token_boundary(
        "type A = { x: string; y: number; };",
        "type A={x:string;y:number};",
    );

    expect_token_boundary(
        "type A = { (): string; (): number; };",
        "type A={():string;():number};",
    );

    expect_token_boundary(
        "type A = { x(): string; y(): number; };",
        "type A={x():string;y():number};",
    );

    expect_token_boundary(
        "type A = { get x(): number; set x(value: number); };",
        "type A={get x():number;set x(value:number)};",
    );

    expect_token_boundary(
        "type A = { new(): A; new(x: number): A; };",
        "type A={new ():A;new (x:number):A};",
    );

    expect_token_boundary(
        "type A = { [x: string]: number; [x: number]: number; };",
        "type A={[x:string]:number;[x:number]:number};",
    );

    expect_token_boundary(
        "type A = { a: { b: string }; c: number };",
        "type A={a:{b:string};c:number};",
    );

    expect_token_boundary("interface A {}", "interface A{}");

    expect_token_boundary("interface A { x: string }", "interface A{x:string}");

    expect_token_boundary("type A = {};", "type A={};");

    expect_token_boundary("type A = { x: string };", "type A={x:string};");
}

#[test]
fn test_type_members_directives() {
    expect_token_boundary(
        "function f(x: { a: number }) { \"use strict\"; return this; }",
        "function f(x:{a:number}){\"use strict\";return this}",
    );

    expect_token_boundary(
        "function f(): { a: number } { \"use strict\"; return { a: 1 }; }",
        "function f():{a:number}{\"use strict\";return {a:1}}",
    );

    expect_token_boundary(
        "const f = (x: { a: number }) => { \"use strict\"; return this; };",
        "const f=(x:{a:number})=>{\"use strict\";return this};",
    );

    expect_token_boundary(
        "const f = (): { a: number } => { \"use strict\"; return { a: 1 }; };",
        "const f=():{a:number}=>{\"use strict\";return {a:1}};",
    );

    expect_token_boundary(
        "function f(x: { a: { b: number } }) { \"use strict\"; return this; }",
        "function f(x:{a:{b:number}}){\"use strict\";return this}",
    );
}

#[test]
fn test_type_members_pretty() {
    expect_printed_ts(
        "interface A { x: string; y: number; }",
        "interface A {\n  x: string;\n  y: number;\n}\n",
    );

    expect_printed_ts(
        "type A = { a: { b: string }; c: number };",
        "type A = {\n  a: {\n    b: string;\n  };\n  c: number;\n};\n",
    );

    expect_printed_ts("interface A {}", "interface A {}\n");
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
fn test_non_null_update_expressions() {
    expect_printed_ts_pair("a!--;", "a!--;\n", "a!--;");

    expect_printed_ts_pair("++a!;", "++a!;\n", "++a!;");

    expect_printed_ts("(a++)!;", "(a++)!;\n");

    expect_printed_ts_minify("(a++)!;", "(a++)!;");

    expect_printed_ts("(++a)!;", "(++a)!;\n");

    expect_printed_ts_minify("(++a)!;", "(++a)!;");
}

#[test]
fn test_non_null_optional_chains() {
    expect_printed_ts("(value?.prop)!;", "(value?.prop)!;\n");

    expect_printed_ts("(value?.[x])!;", "(value?.[x])!;\n");

    expect_printed_ts("(value?.())!;", "(value?.())!;\n");

    expect_printed_ts("(value?.prop)!.x;", "(value?.prop)!.x;\n");

    expect_printed_ts("((value?.prop)!)!;", "(value?.prop)!!;\n");

    expect_printed_ts_minify("(value?.prop)!;", "(value?.prop)!;");
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

    expect_token_boundary("(f<T>) >= x", "(f<T>)>=x;");

    expect_token_boundary("(f<T>) >> x", "(f<T>)>>x;");

    expect_token_boundary("(f<T>) >>> x", "(f<T>)>>>x;");

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

    expect_token_boundary("new C<T>() + x;", "new C<T>()+x;");

    expect_token_boundary("new C<T>() - x;", "new C<T>()-x;");

    expect_token_boundary("new C<T>() > x;", "new C<T>()>x;");

    expect_token_boundary("new C<T>() >= x;", "new C<T>()>=x;");

    expect_token_boundary("new C<T>() >> x;", "new C<T>()>>x;");

    expect_token_boundary("new C<T>() >>> x;", "new C<T>()>>>x;");

    expect_token_boundary("new C<T>();", "new C<T>();");

    expect_token_boundary("new C<T>(x) + x;", "new C<T>(x)+x;");

    expect_token_boundary("new C();", "new C;");

    expect_token_boundary("new C() + x;", "new C+x;");
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
fn test_ambient_module_declarations() {
    expect_printed_ts(
        "declare module \"a\"; declare module \"b\";",
        "declare module \"a\";\ndeclare module \"b\";\n",
    );

    expect_printed_ts_minify(
        "declare module \"a\"; declare module \"b\";",
        "declare module \"a\";declare module \"b\";",
    );

    expect_printed_ts_minify(
        "declare module \"a\"; const x = 1;",
        "declare module \"a\";const x=1;",
    );

    expect_printed_ts_minify(
        "export declare module \"a\"; export declare module \"b\";",
        "export declare module \"a\";export declare module \"b\";",
    );

    expect_printed_ts_minify("declare module \"a\";", "declare module \"a\";");

    expect_printed_ts_minify(
        "declare module \"a\" {} declare module \"b\" {}",
        "declare module \"a\"{}declare module \"b\"{}",
    );

    expect_printed_ts_minify(
        "namespace N { module \"a\"; module \"b\"; }",
        "namespace N{module \"a\";module \"b\"}",
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

    expect_printed_ts_minify(r#"enum E { "'\"" = 1 }"#, r#"enum E{"'\""=1,}"#);

    expect_printed_ts_minify(r#"enum E { ["'\""] = 1 }"#, r#"enum E{[`'"`]=1,}"#);
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
    expect_printed_ts("type Positive = 1e999;", "type Positive = 1e999;\n");
    expect_printed_ts("type Positive = 1e999;\n", "type Positive = 1e999;\n");
    expect_printed_ts_minify("type Positive = 1e999;", "type Positive=1e999;");

    expect_printed_ts("type Negative = -1e999;", "type Negative = -1e999;\n");
    expect_printed_ts("type Negative = -1e999;\n", "type Negative = -1e999;\n");
    expect_printed_ts_minify("type Negative = -1e999;", "type Negative=-1e999;");
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

#[test]
fn test_instantiation_expression_boundaries() {
    expect_printed_ts("(f<T>)<U>();", "(f<T>)<U>();\n");

    expect_printed_ts("(f<T>)!;", "(f<T>)!;\n");

    expect_printed_ts("((f<T>)!)<U>();", "(f<T>)!<U>();\n");

    expect_printed_ts("((f<T>)!)!;", "(f<T>)!!;\n");

    expect_printed_ts("(f<T>)();", "(f<T>)();\n");

    expect_printed_ts("(f<T>).x;", "(f<T>).x;\n");

    expect_printed_ts("new (f<T>)<U>();", "new (f<T>)<U>();\n");

    expect_printed_ts("(f<T>)<U>`x`;", "(f<T>)<U>`x`;\n");

    expect_printed_ts(
        "class C extends (f<T>)<U> {}",
        "class C extends (f<T>)<U> {\n}\n",
    );

    expect_printed_ts(
        "class C extends (f?.x<T>) {}",
        "class C extends (f?.x<T>) {\n}\n",
    );

    expect_printed_ts(
        "class C extends (f?.x<T>)<U> {}",
        "class C extends (f?.x<T>)<U> {\n}\n",
    );

    expect_printed_ts(
        "class C extends (f?.x!<T>)<U> {}",
        "class C extends (f?.x!<T>)<U> {\n}\n",
    );

    expect_printed_ts(
        "class C extends (f?.[x]<T>)<U> {}",
        "class C extends (f?.[x]<T>)<U> {\n}\n",
    );

    expect_printed_ts(
        "class C extends (f?.()<T>)<U> {}",
        "class C extends (f?.()<T>)<U> {\n}\n",
    );

    expect_printed_ts(
        "class C extends f?.x<T> {}",
        "class C extends f?.x<T> {\n}\n",
    );

    expect_printed_ts("class C extends f?.x {}", "class C extends f?.x {\n}\n");

    expect_printed_ts_minify(
        "class C extends (f?.x<T>)<U> {}",
        "class C extends (f?.x<T>)<U>{}",
    );

    expect_printed_ts("f<T>?.<U>();", "f<T>?.<U>();\n");

    expect_printed_ts("(f<T>)?.x;", "(f<T>)?.x;\n");

    expect_printed_ts("f<T>?.[x];", "f<T>?.[x];\n");

    expect_printed_ts("(f<T>)?.[x];", "f<T>?.[x];\n");

    expect_printed_ts_minify("(f<T>)?.x;", "(f<T>)?.x;");

    expect_printed_ts_minify("((f<T>)!)<U>();", "(f<T>)!<U>();");

    expect_printed_ts_minify("((f<T>)!)!;", "(f<T>)!!;");
}

#[test]
fn test_parenthesized_boundaries() {
    let options = ParseOptions {
        lang: Lang::Ts,
        preserve_parens: true,
        ..ParseOptions::default()
    };
    let expect = |source, pretty, compact| expect_printed_pair(source, options, pretty, compact);

    expect("(f<T>) << x;", "(f<T>) << x;\n", "(f<T>)<<x;");

    expect("(f<T>) <= x;", "(f<T>) <= x;\n", "(f<T>)<=x;");

    expect("(f<T>) && x;", "(f<T>) && x;\n", "(f<T>)&&x;");

    expect("(f<T>) * x;", "(f<T>) * x;\n", "(f<T>)*x;");

    expect("(f<T>) === x;", "(f<T>) === x;\n", "(f<T>)===x;");

    expect(
        "(await (f<T>)) ** x;",
        "(await (f<T>)) ** x;\n",
        "(await(f<T>))**x;",
    );

    expect(
        "a * (f<T>) * b >= x;",
        "a * (f<T>) * b >= x;\n",
        "a*(f<T>)*b>=x;",
    );

    expect(
        "a * (b + (f<T>)) >= x;",
        "a * (b + (f<T>)) >= x;\n",
        "a*(b+(f<T>))>=x;",
    );

    expect(
        "(<U>(f<T>)) ** x;",
        "(<U>(f<T>)) ** x;\n",
        "(<U>(f<T>))**x;",
    );

    expect(
        "(typeof (f<T>)) ** x;",
        "(typeof (f<T>)) ** x;\n",
        "(typeof (f<T>))**x;",
    );

    expect(
        "(<number>(<unknown>a))--;",
        "(<number>(<unknown>a))--;\n",
        "(<number>(<unknown>a))--;",
    );

    expect(
        "((a as unknown) as number)--;",
        "((a as unknown) as number)--;\n",
        "((a as unknown) as number)--;",
    );

    expect(
        "(value as A | (typeof f)) << other;",
        "(value as A | (typeof f)) << other;\n",
        "(value as A | (typeof f))<<other;",
    );

    expect(
        "(one < two) as typeof f << other;",
        "(one < two) as typeof f << other;\n",
        "(one<two) as typeof f<<other;",
    );

    expect(
        "(x as T) ? (/a/) : false;",
        "(x as T) ? (/a/) : false;\n",
        "(x as T)?(/a/):false;",
    );
}

#[test]
fn test_conditional_casts() {
    expect_printed_ts_pair(
        "x as T ? (() => /a/)() : false;",
        "x as T ? (() => /a/)() : false;\n",
        "x as T?(()=>/a/)():false;",
    );

    expect_printed_ts_pair(
        "(x as T) ? ~a : false;",
        "(x as T) ? ~a : false;\n",
        "(x as T)?~a:false;",
    );

    expect_printed_ts_pair(
        "x as T ? true : false;",
        "x as T ? true : false;\n",
        "x as T?true:false;",
    );

    expect_printed_ts_pair(
        "x as T ? -1 : false;",
        "x as T ? -1 : false;\n",
        "x as T?-1:false;",
    );

    expect_printed_ts_pair(
        "(x as T) ? ++a : false;",
        "(x as T) ? ++a : false;\n",
        "(x as T)?++a:false;",
    );

    expect_printed_ts_pair(
        "(x as T) ? /a/.test(s) : false;",
        "(x as T) ? /a/.test(s) : false;\n",
        "(x as T)?/a/.test(s):false;",
    );

    expect_printed_ts_pair(
        "(x as T) ? /a/ : false;",
        "(x as T) ? /a/ : false;\n",
        "(x as T)?/a/:false;",
    );

    expect_printed_ts_pair(
        "class C extends B { m() { return (x as T) ? super.m() : false; } }",
        "class C extends B {\n  m() {\n    return (x as T) ? super.m() : false;\n  }\n}\n",
        "class C extends B{m(){return (x as T)?super.m():false}}",
    );

    expect_printed_ts_pair(
        "(x as T) ? +a : false;",
        "(x as T) ? +a : false;\n",
        "(x as T)?+a:false;",
    );

    expect_printed_ts_pair(
        "x && y as T ? true : false;",
        "x && y as T ? true : false;\n",
        "x&&y as T?true:false;",
    );

    expect_printed_ts_pair(
        "x && (y as T) ? /a/ : false;",
        "x && (y as T) ? /a/ : false;\n",
        "x&&(y as T)?/a/:false;",
    );

    expect_printed_ts_pair(
        "x || y satisfies T ? true : false;",
        "x || y satisfies T ? true : false;\n",
        "x||y satisfies T?true:false;",
    );

    expect_printed_ts_pair(
        "x || (y satisfies T) ? /a/ : false;",
        "x || (y satisfies T) ? /a/ : false;\n",
        "x||(y satisfies T)?/a/:false;",
    );

    expect_printed_ts_pair(
        "x satisfies T ? true : false;",
        "x satisfies T ? true : false;\n",
        "x satisfies T?true:false;",
    );

    expect_printed_ts_pair(
        "(x satisfies T) ? class {} : false;",
        "(x satisfies T) ? class {\n} : false;\n",
        "(x satisfies T)?class{}:false;",
    );

    expect_printed_ts_pair(
        "(x satisfies T) ? delete a.b : false;",
        "(x satisfies T) ? delete a.b : false;\n",
        "(x satisfies T)?delete a.b:false;",
    );

    expect_printed_ts_pair(
        "x satisfies T ? !a : false;",
        "x satisfies T ? !a : false;\n",
        "x satisfies T?!a:false;",
    );

    expect_printed_ts_pair(
        "(x satisfies T) ? /a/ === r : false;",
        "(x satisfies T) ? /a/ === r : false;\n",
        "(x satisfies T)?/a/===r:false;",
    );

    expect_printed_ts_pair(
        "(x satisfies T) ? (/a/ as R).test(s) : false;",
        "(x satisfies T) ? (/a/ as R).test(s) : false;\n",
        "(x satisfies T)?(/a/ as R).test(s):false;",
    );

    expect_printed_ts_pair(
        "(x satisfies T) ? /a/ : false;",
        "(x satisfies T) ? /a/ : false;\n",
        "(x satisfies T)?/a/:false;",
    );

    expect_printed_ts_pair(
        "(x satisfies T) ? -a : false;",
        "(x satisfies T) ? -a : false;\n",
        "(x satisfies T)?-a:false;",
    );

    expect_printed_ts_pair(
        "let a = typeof ((\"a\" as unknown) ? /a/ : false);",
        "let a = typeof ((\"a\" as unknown) ? /a/ : false);\n",
        "let a=typeof ((\"a\" as unknown)?/a/:false);",
    );

    expect_printed_ts_pair(
        "let a = typeof ((\"a\" satisfies foo) ? /a/ : false);",
        "let a = typeof ((\"a\" satisfies foo) ? /a/ : false);\n",
        "let a=typeof ((\"a\" satisfies foo)?/a/:false);",
    );
}

#[test]
fn test_instantiation_binary_operators() {
    expect_printed_ts_pair("(f<T>) + x;", "(f<T>) + x;\n", "(f<T>)+x;");

    expect_printed_ts_pair("await (f<T>) + x;", "await (f<T>) + x;\n", "await(f<T>)+x;");

    expect_printed_ts_pair(
        "(await (f<T>)) ** x;",
        "(await f<T>) ** x;\n",
        "(await f<T>)**x;",
    );

    expect_printed_ts_pair(
        "a * b ** (f<T>) + x;",
        "a * b ** (f<T>) + x;\n",
        "a*b**(f<T>)+x;",
    );

    expect_printed_ts_pair("(f<T>) >= x;", "(f<T>) >= x;\n", "(f<T>)>=x;");

    expect_printed_ts_pair("(f<T>) > x;", "(f<T>) > x;\n", "(f<T>)>x;");

    expect_printed_ts_pair("(f<T>) << x;", "f<T> << x;\n", "f<T><<x;");

    expect_printed_ts_pair("(f<T>) <= x;", "f<T> <= x;\n", "f<T><=x;");

    expect_printed_ts_pair("(f<T>) < x;", "(f<T>) < x;\n", "(f<T>)<x;");

    expect_printed_ts_pair("(f<T>) && x;", "f<T> && x;\n", "f<T>&&x;");

    expect_printed_ts_pair(
        "a * await (f<T>) >= x;",
        "a * await (f<T>) >= x;\n",
        "a*await(f<T>)>=x;",
    );

    expect_printed_ts_pair("a * (f<T>) >= x;", "a * (f<T>) >= x;\n", "a*(f<T>)>=x;");

    expect_printed_ts_pair(
        "a * (f<T>) * b >= x;",
        "a * f<T> * b >= x;\n",
        "a*f<T>*b>=x;",
    );

    expect_printed_ts_pair(
        "a * <U>(f<T>) >= x;",
        "a * <U>(f<T>) >= x;\n",
        "a*<U>(f<T>)>=x;",
    );

    expect_printed_ts_pair("a * +(f<T>) >= x;", "a * +(f<T>) >= x;\n", "a*+(f<T>)>=x;");

    expect_printed_ts_pair("(f<T>) * x;", "f<T> * x;\n", "f<T>*x;");

    expect_printed_ts_pair("(f?.x<T>) + x;", "(f?.x<T>) + x;\n", "(f?.x<T>)+x;");

    expect_printed_ts_pair(
        "a * (b + (f<T>)) >= x;",
        "a * (b + f<T>) >= x;\n",
        "a*(b+f<T>)>=x;",
    );

    expect_printed_ts_pair("(f<T>) >> x;", "(f<T>) >> x;\n", "(f<T>)>>x;");

    expect_printed_ts_pair("(f<T>) === x;", "f<T> === x;\n", "f<T> ===x;");

    expect_printed_ts_pair("(f<T>) - x;", "(f<T>) - x;\n", "(f<T>)-x;");

    expect_printed_ts_pair("<U>(f<T>) + x;", "<U>(f<T>) + x;\n", "<U>(f<T>)+x;");

    expect_printed_ts_pair("(<U>(f<T>)) ** x;", "(<U>f<T>) ** x;\n", "(<U>f<T>)**x;");

    expect_printed_ts_pair(
        "(typeof (f<T>)) ** x;",
        "(typeof f<T>) ** x;\n",
        "(typeof f<T>)**x;",
    );

    expect_printed_ts_pair(
        "typeof (f<T>) - x;",
        "typeof (f<T>) - x;\n",
        "typeof (f<T>)-x;",
    );

    expect_printed_ts_pair("+(f<T>) + x;", "+(f<T>) + x;\n", "+(f<T>)+x;");

    expect_printed_ts_pair("(f<T>) >>> x;", "(f<T>) >>> x;\n", "(f<T>)>>>x;");
}

#[test]
fn test_type_assertion_targets() {
    expect_printed_ts_pair(
        "[(<number>a)] = values;",
        "[(<number>a)] = values;\n",
        "[(<number>a)]=values;",
    );

    expect_printed_ts_pair(
        "(<number>a) = value;",
        "(<number>a) = value;\n",
        "(<number>a)=value;",
    );

    expect_printed_ts_pair(
        "(<number>(<unknown>a))--;",
        "(<number><unknown>a)--;\n",
        "(<number><unknown>a)--;",
    );

    expect_printed_ts_pair(
        "for ((<string>a) in obj) {}",
        "for ((<string>a) in obj) {\n}\n",
        "for((<string>a) in obj){}",
    );

    expect_printed_ts_pair(
        "for ((<number>a) of values) {}",
        "for ((<number>a) of values) {\n}\n",
        "for((<number>a) of values){}",
    );

    expect_printed_ts_pair(
        "(<number>obj.value)++;",
        "(<number>obj.value)++;\n",
        "(<number>obj.value)++;",
    );

    expect_printed_ts_pair(
        "({ value: (<number>a) } = obj);",
        "({ value: (<number>a) } = obj);\n",
        "({value:(<number>a)}=obj);",
    );

    expect_printed_ts_pair("(<number>a)--;", "(<number>a)--;\n", "(<number>a)--;");

    expect_printed_ts_pair("(<number>a)++;", "(<number>a)++;\n", "(<number>a)++;");

    expect_printed_ts_pair("--(<number>a);", "--(<number>a);\n", "--(<number>a);");

    expect_printed_ts_pair("++(<number>a);", "++(<number>a);\n", "++(<number>a);");

    expect_printed_ts_pair(
        "[(a as number)] = values;",
        "[(a as number)] = values;\n",
        "[(a as number)]=values;",
    );

    expect_printed_ts_pair(
        "(a as number) = value;",
        "(a as number) = value;\n",
        "(a as number)=value;",
    );

    expect_printed_ts_pair(
        "((a as unknown) as number)--;",
        "(a as unknown as number)--;\n",
        "(a as unknown as number)--;",
    );

    expect_printed_ts_pair(
        "for ((a as string) in obj) {}",
        "for ((a as string) in obj) {\n}\n",
        "for((a as string) in obj){}",
    );

    expect_printed_ts_pair(
        "for ((a as number) of values) {}",
        "for ((a as number) of values) {\n}\n",
        "for((a as number) of values){}",
    );

    expect_printed_ts_pair(
        "(obj.value as number)++;",
        "(obj.value as number)++;\n",
        "(obj.value as number)++;",
    );

    expect_printed_ts_pair(
        "({ value: (a as number) } = obj);",
        "({ value: (a as number) } = obj);\n",
        "({value:(a as number)}=obj);",
    );

    expect_printed_ts_pair("(a as number)--;", "(a as number)--;\n", "(a as number)--;");

    expect_printed_ts_pair("(a as number)++;", "(a as number)++;\n", "(a as number)++;");

    expect_printed_ts_pair("--(a as number);", "--(a as number);\n", "--(a as number);");

    expect_printed_ts_pair("++(a as number);", "++(a as number);\n", "++(a as number);");
}

#[test]
fn test_type_query_boundaries() {
    expect_printed_ts_pair(
        "(value as A | typeof f[]) << other;",
        "(value as A | typeof f[]) << other;\n",
        "(value as A | typeof f[])<<other;",
    );

    expect_printed_ts_pair(
        "(value as A | typeof import(\"m\").f) << other;",
        "(value as A | typeof import(\"m\").f) << other;\n",
        "(value as A | typeof import(\"m\").f)<<other;",
    );

    expect_printed_ts_pair(
        "(one < two as A | typeof f) << other;",
        "(one < two as A | typeof f) << other;\n",
        "(one<two as A | typeof f)<<other;",
    );

    expect_printed_ts_pair(
        "(value as typeof f | Reference) << other;",
        "(value as typeof f | Reference) << other;\n",
        "(value as typeof f | Reference)<<other;",
    );

    expect_printed_ts_pair(
        "value as T extends U ? A : typeof f << other;",
        "value as T extends U ? A : typeof f << other;\n",
        "value as T extends U ? A : typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value satisfies new () => typeof f << other;",
        "value satisfies new () => typeof f << other;\n",
        "value satisfies new () => typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value as () => typeof f << other;",
        "value as () => typeof f << other;\n",
        "value as () => typeof f<<other;",
    );

    expect_printed_ts_pair(
        "type Query = typeof f< <T>(value: T) => T>;",
        "type Query = typeof f< <T>(value: T) => T>;\n",
        "type Query=typeof f< <T>(value:T) => T>;",
    );

    expect_printed_ts_pair(
        "(value as typeof import(\"m\").f) << other;",
        "(value as typeof import(\"m\").f) << other;\n",
        "(value as typeof import(\"m\").f)<<other;",
    );

    expect_printed_ts_pair(
        "(value as A | typeof f[\"value\"]) << other;",
        "(value as A | typeof f[\"value\"]) << other;\n",
        "(value as A | typeof f[\"value\"])<<other;",
    );

    expect_printed_ts_pair(
        "value as infer T extends typeof f << other;",
        "value as infer T extends typeof f << other;\n",
        "value as infer T extends typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value satisfies A & typeof f << other;",
        "value satisfies A & typeof f << other;\n",
        "value satisfies A & typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value as typeof f << other > (one, two, three);",
        "value as typeof f << other > (one, two, three);\n",
        "value as typeof f<<other>(one,two,three);",
    );

    expect_printed_ts_pair(
        "(one < two as typeof f) << other;",
        "(one < two as typeof f) << other;\n",
        "(one<two as typeof f)<<other;",
    );

    expect_printed_ts_pair(
        "value as typeof f << other;",
        "value as typeof f << other;\n",
        "value as typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value as typeof f\n<< other;",
        "value as typeof f << other;\n",
        "value as typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value satisfies T extends U ? A : () => B | keyof typeof f << other;",
        "value satisfies T extends U ? A : () => B | keyof typeof f << other;\n",
        "value satisfies T extends U ? A : () => B | keyof typeof f<<other;",
    );

    expect_printed_ts_pair(
        "(value as keyof (A | typeof f)) << other;",
        "(value as keyof (A | typeof f)) << other;\n",
        "(value as keyof (A | typeof f))<<other;",
    );

    expect_printed_ts_pair(
        "(value as A | (typeof f)) << other;",
        "value as A | typeof f << other;\n",
        "value as A | typeof f<<other;",
    );

    expect_printed_ts_pair(
        "(one < two) as typeof f << other;",
        "(one < two as typeof f) << other;\n",
        "(one<two as typeof f)<<other;",
    );

    expect_printed_ts_pair(
        "value as (x: unknown) => x is typeof f << other;",
        "value as (x: unknown) => x is typeof f << other;\n",
        "value as (x:unknown) => x is typeof f<<other;",
    );

    expect_printed_ts_pair(
        "(value as Wrapper<typeof f>) << other;",
        "(value as Wrapper<typeof f>) << other;\n",
        "(value as Wrapper<typeof f>)<<other;",
    );

    expect_printed_ts_pair(
        "type Reference = Wrapper<<T>(value: T) => T>;",
        "type Reference = Wrapper< <T>(value: T) => T>;\n",
        "type Reference=Wrapper< <T>(value:T) => T>;",
    );

    expect_printed_ts_pair(
        "value satisfies typeof f << other;",
        "value satisfies typeof f << other;\n",
        "value satisfies typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value as keyof typeof f << other;",
        "value as keyof typeof f << other;\n",
        "value as keyof typeof f<<other;",
    );

    expect_printed_ts_pair(
        "value as A | typeof f << other;",
        "value as A | typeof f << other;\n",
        "value as A | typeof f<<other;",
    );
}
