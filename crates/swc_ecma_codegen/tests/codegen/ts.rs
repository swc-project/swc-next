use crate::{expect_printed_ts, expect_printed_ts_minify};

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
fn test_type_assertion_generic_function_type() {
    let type_assertion = "const f = < <T>(value: T) => T > identity;";
    let type_assertion_expected = "const f = < <T>(value: T) => T>identity;\n";
    expect_printed_ts(type_assertion, type_assertion_expected);
    expect_printed_ts(type_assertion_expected, type_assertion_expected);
    expect_printed_ts_minify(type_assertion, "const f=< <T>(value:T) => T>identity;");
}

#[test]
fn test_non_finite_numeric_literal_type() {
    let non_finite_literals = "type Positive = 1e999; type Negative = -1e999;";
    let non_finite_literals_expected = "type Positive = 1e999;\ntype Negative = -1e999;\n";
    expect_printed_ts(non_finite_literals, non_finite_literals_expected);
    expect_printed_ts(non_finite_literals_expected, non_finite_literals_expected);
    expect_printed_ts_minify(
        non_finite_literals,
        "type Positive=1e999;type Negative=-1e999;",
    );
}

#[test]
fn test_satisfies_assignment_targets() {
    let satisfies_targets = "let value = 0; ++(value satisfies number); (value satisfies number)--; (value satisfies number) = 1; [(value satisfies number)] = [1]; ({ key: (value satisfies number) } = { key: 1 }); for ((value satisfies number) of values) {} for ((value satisfies number) in values) {}";
    let satisfies_targets_expected = "let value = 0;\n++(value satisfies number);\n(value satisfies number)--;\n(value satisfies number) = 1;\n[(value satisfies number)] = [1];\n({ key: (value satisfies number) } = { key: 1 });\nfor ((value satisfies number) of values) {\n}\nfor ((value satisfies number) in values) {\n}\n";
    expect_printed_ts(satisfies_targets, satisfies_targets_expected);
    expect_printed_ts(satisfies_targets_expected, satisfies_targets_expected);
    expect_printed_ts_minify(
        satisfies_targets,
        "let value=0;++(value satisfies number);(value satisfies number)--;(value satisfies number)=1;[(value satisfies number)]=[1];({key:(value satisfies number)}={key:1});for((value satisfies number) of values){}for((value satisfies number) in values){}",
    );
}
