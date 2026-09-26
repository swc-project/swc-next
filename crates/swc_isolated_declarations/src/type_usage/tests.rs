use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

use crate::transform;

fn declaration_output(source: &str) -> (String, usize) {
    declaration_output_with_source_type(source, SourceType::Module)
}

fn declaration_output_with_source_type(source: &str, source_type: SourceType) -> (String, usize) {
    let allocator = Allocator::new();
    let options = Options {
        lang: Lang::Ts,
        source_type,
        comments: CommentMode::Flat,
        ..Options::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    let result = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: true,
            ..AnalyzeOptions::default()
        },
    );
    assert!(result.diagnostics.is_empty());
    let diagnostics = transform(&mut ast, &result.semantic);
    let code = print(
        &allocator,
        &ast,
        PrintOptions {
            comments: false,
            ..PrintOptions::default()
        },
    )
    .code;
    (code.to_owned(), diagnostics.len())
}

fn assert_export_dependencies(source: &str) {
    let allocator = Allocator::new();
    let options = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let ast = parsed.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let scope = swc_next_ecma_ast::ScopeId::MODULE;
    let local = semantic
        .lookup(scope, b"X", swc_next_ecma_semantic::ReferenceSpace::Type)
        .unwrap();
    let exported = semantic
        .lookup(scope, b"X", swc_next_ecma_semantic::ReferenceSpace::Value)
        .unwrap();
    assert_ne!(local, exported);
    let (used, diagnostics, _) = super::TypeUsageAnalyzer::analyze(&ast, &semantic);
    assert!(diagnostics.is_empty());
    assert!(used.contains_type(local));
    assert!(!used.contains_value(local));
    assert!(used.contains_value(exported));
}

#[test]
fn export_dependencies() {
    let (code, diagnostics) = declaration_output(
        "import type { T } from 'dual' with { 'resolution-mode': 'require' }; export type Public = T;",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("with {"));
    assert!(code.contains("resolution-mode"));

    let (code, diagnostics) =
        declaration_output("export type { T } from 'dual' with { 'resolution-mode': 'require' };");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("resolution-mode"));

    let (code, diagnostics) =
        declaration_output("export type * from 'dual' with { 'resolution-mode': 'import' };");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("resolution-mode"));

    assert_export_dependencies("interface X {} export const X = 1; export default X;");

    assert_export_dependencies("export const X = 1; interface X {} export { X as Y };");

    let (code, diagnostics) =
        declaration_output("interface X {} export const X = 1; export default X;");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("interface X"));
    assert!(code.contains("export declare const X = 1;"));
    assert!(code.contains("export default X;"));

    let (code, diagnostics) = declaration_output("export default 1 as typeof _default;");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("const _default_1: typeof _default"));
    assert!(code.contains("export default _default_1"));

    let (code, diagnostics) = declaration_output(
        "declare const _default_1: number; export default 1 as typeof _default;",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("const _default_2: typeof _default"));

    let (code, diagnostics) =
        declaration_output("const hidden = [1, 2]; export default hidden as number[];");
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("hidden"));
    assert!(code.contains("const _default: number[]"));

    let (code, diagnostics) =
        declaration_output("const hidden = [1, 2]; export = hidden as number[];");
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("hidden"));
    assert!(code.contains("export = _default"));
}

#[test]
fn exported_alias_declarations() {
    let (code, diagnostics) =
        declaration_output("namespace N { export interface T {} } export import A = N.T;");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("namespace N"));
    assert!(code.contains("interface T"));
    assert!(code.contains("export import A = N.T;"));

    let (code, diagnostics) = declaration_output(
        "namespace N { export interface T {} } export namespace Use { export import A = N.T; }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("namespace N"));
    assert!(code.contains("namespace Use"));
    assert!(code.contains("export import A = N.T;"));
}

#[test]
fn expando_type_dependencies() {
    let (code, diagnostics) = declaration_output(
        "type Alias = number; function Root(): void {} namespace Root { export function child(): void {} export namespace child { export interface Marker {} } } Root.child.value = 1; export namespace Use { import Alias = Root.child.value; export type T = Alias; }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type Alias = number;"));
    assert!(!code.contains("import Alias"));
    assert!(!code.contains("namespace Root"));
}

#[test]
fn uncertain_declaration_dependencies() {
    let (code, diagnostics) = declaration_output(
        "import * as Holder from 'pkg'; type Alias = number; export namespace Use { import Alias = Holder.Member; export type T = Alias; }",
    );
    assert_eq!(diagnostics, 1);
    assert!(code.contains("type Alias = number;"));
    assert!(code.contains("import Alias = Holder.Member;"));

    let (code, diagnostics) = declaration_output(
        "import * as Holder from 'pkg'; type Alias = number; namespace Private { import Alias = Holder.Member; export type T = Alias; } export {};",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("namespace Private"));
}

#[test]
fn inferred_declaration_dependencies() {
    let (code, diagnostics) = declaration_output(
        "type Hidden = { x: number }; export class C { value = ({} as Hidden); }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type Hidden"));
    assert!(code.contains("value: Hidden"));

    let (code, diagnostics) = declaration_output(
        "type Hidden = { x: number }; export function f(x = ({} as Hidden)): void {}",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type Hidden"));
    assert!(code.contains("x?: Hidden"));

    let (code, diagnostics) = declaration_output(
        "type Hidden = { x: number }; export function f() { return ({} as Hidden); }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type Hidden"));
    assert!(code.contains("f(): Hidden"));

    let (code, diagnostics) = declaration_output(
        "function runtime() { return [1]; } export const value: number = runtime()[0];",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("runtime"));
    assert!(code.contains("value: number"));
}

#[test]
fn namespace_value_dependencies() {
    let (code, diagnostics) = declaration_output(
        "type Hidden = { x: number }; namespace N { export const x: Hidden = { x: 1 }; } export type T = typeof N;",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("namespace N"));
    assert!(code.contains("type Hidden"));
    assert!(code.contains("x: Hidden"));
    let (code, diagnostics) = declaration_output(
        "import type { T } from './dep'; export declare namespace API { namespace Nested { function use(value: T): void; } }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("import type { T }"));
    assert!(code.contains("use(value: T)"));

    let (code, diagnostics) = declaration_output_with_source_type(
        "namespace API { interface T { value: string } export function use(value: T): void {} }",
        SourceType::Script,
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("interface T"));
    assert!(code.contains("use(value: T)"));
}

#[test]
fn method_overload_dependencies() {
    let (code, diagnostics) = declaration_output(
        "const hidden = (() => ({ value: 1 }))(); export function f(x: string): void; export function f(x: typeof hidden | string): void {}",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("hidden"));

    let (code, diagnostics) = declaration_output(
        "function compute(): number { return 1; } export class C { constructor(x?: number); constructor(x = compute()) {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("compute"));
    assert_eq!(code.matches("constructor(").count(), 1);

    let (code, diagnostics) = declaration_output(
        "type T = { value: number }; export class C { constructor(value: T); constructor(public value: T) {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type T"));
    assert!(code.contains("value: T;"));
    assert_eq!(code.matches("constructor(").count(), 1);

    let (code, diagnostics) = declaration_output(
        "export default function f(x: number): void; export default function f(x = compute()): void {} declare function compute(): number;",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("compute"));

    let (code, diagnostics) = declaration_output(
        "export namespace N { const hidden = (() => 1)(); export function f(x: string): void; export function f(x: typeof hidden | string): void {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("hidden"));

    let (code, diagnostics) = declaration_output(
        "type Hidden = { value: string }; export declare class C { private m(x: Hidden): Hidden; }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("private m;"));
    assert!(!code.contains("Hidden"));

    let (code, diagnostics) = declaration_output(
        "type Hidden = { value: string }; export class C { private m(x: Hidden): Hidden; private m(x: string): string; private m(x: Hidden | string): Hidden | string { return x; } }",
    );
    assert_eq!(diagnostics, 0);
    assert_eq!(code.matches("private m;").count(), 1);
    assert!(!code.contains("Hidden"));

    let (code, diagnostics) = declaration_output(
        "type Hidden = number[]; export class C { m(x: string): string; m(x: number): number; m(x = [] as Hidden) { return [x]; } }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));
    assert!(code.contains("m(x: string): string"));
    assert!(code.contains("m(x: number): number"));
    assert_eq!(code.matches("m(").count(), 2);

    let (code, diagnostics) = declaration_output(
        "export default class { m(x: string): string; m(x = []) { return [x]; } }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("m(x: string): string"));
    assert_eq!(code.matches("m(").count(), 1);
}

#[test]
fn module_contexts() {
    let (code, diagnostics) = declaration_output(
        "import type { Unused } from 'pkg'; declare global { interface Window { value: string } }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("import"));
    assert!(code.contains("declare global"));
    assert!(code.contains("export {}"));

    let common_js = SourceType::from_path("input.cts");
    let (code, diagnostics) = declaration_output_with_source_type("const hidden = 1;", common_js);
    assert_eq!(diagnostics, 0);
    assert_eq!(code, "export {};\n");

    let (code, diagnostics) =
        declaration_output_with_source_type("function hidden() { return [1]; }", common_js);
    assert_eq!(diagnostics, 0);
    assert_eq!(code, "export {};\n");

    let (code, diagnostics) = declaration_output_with_source_type(
        "type T = string; const visible: T = 'x'; const hidden = 1; export = visible;",
        common_js,
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type T = string;"));
    assert!(code.contains("declare const visible: T;"));
    assert!(code.contains("export = visible;"));
    assert!(!code.contains("hidden"));

    let (code, diagnostics) = declaration_output_with_source_type(
        "declare global { interface Window { value: string } }",
        common_js,
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("declare global"));
    assert!(code.contains("export {}"));

    let (code, diagnostics) =
        declaration_output_with_source_type("const hidden = 1;", SourceType::Script);
    assert_eq!(diagnostics, 0);
    assert_eq!(code, "declare const hidden = 1;\n");
}

#[test]
fn transparent_expression_parentheses() {
    let (plain, plain_diagnostics) =
        declaration_output("export const value = [-1, 2, `text`] as const;");
    let (wrapped, wrapped_diagnostics) =
        declaration_output("export const value = ((([-(1), ((2)), (`text`)])) as const);");
    assert_eq!(plain_diagnostics, 0);
    assert_eq!(wrapped_diagnostics, 0);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) = declaration_output("export const value = -1;");
    let (wrapped, wrapped_diagnostics) = declaration_output("export const value = (-(1));");
    assert_eq!(plain_diagnostics, 0);
    assert_eq!(wrapped_diagnostics, 0);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) = declaration_output(
        "export const value = { get ['x'](): number { return 1; }, set x(value: number) {} };",
    );
    let (wrapped, wrapped_diagnostics) = declaration_output(
        "export const value = { get [(('x'))](): number { return 1; }, set x(value: number) {} };",
    );
    assert_eq!(plain_diagnostics, 0);
    assert_eq!(wrapped_diagnostics, 0);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) = declaration_output("export function f(): void {} f.x = 1;");
    let (wrapped, wrapped_diagnostics) =
        declaration_output("export function f(): void {} (((f).x = 1));");
    assert_eq!(plain_diagnostics, 1);
    assert_eq!(wrapped_diagnostics, plain_diagnostics);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) =
        declaration_output("export const f = (): void => {}; f.x = 1;");
    let (wrapped, wrapped_diagnostics) =
        declaration_output("export const f = ((((): void => {}))); f.x = 1;");
    assert_eq!(plain_diagnostics, 1);
    assert_eq!(wrapped_diagnostics, plain_diagnostics);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) =
        declaration_output("export const f = function(): void {}; f.x = 1;");
    let (wrapped, wrapped_diagnostics) =
        declaration_output("export const f = (((function(): void {}))); f.x = 1;");
    assert_eq!(plain_diagnostics, 1);
    assert_eq!(wrapped_diagnostics, plain_diagnostics);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) = declaration_output(
        "export function f(): void {} export namespace f { export const x: number = 0; } f.x = 1;",
    );
    let (wrapped, wrapped_diagnostics) = declaration_output(
        "export function f(): void {} export namespace f { export const x: number = 0; } (((f).x = 1));",
    );
    assert_eq!(plain_diagnostics, 0);
    assert_eq!(wrapped_diagnostics, plain_diagnostics);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) = declaration_output("export const value = { n: +1 } as const;");
    let (wrapped, wrapped_diagnostics) =
        declaration_output("export const value = { n: +((1)) } as const;");
    assert_eq!(plain_diagnostics, 0);
    assert_eq!(wrapped_diagnostics, plain_diagnostics);
    assert_eq!(wrapped, plain);

    let (plain, plain_diagnostics) = declaration_output("export const value = +1;");
    let (wrapped, wrapped_diagnostics) = declaration_output("export const value = +((1));");
    assert_eq!(plain_diagnostics, 0);
    assert_eq!(wrapped_diagnostics, plain_diagnostics);
    assert_eq!(wrapped, plain);

    let (_, plain_diagnostics) = declaration_output("export class C extends null {}");
    let (_, wrapped_diagnostics) = declaration_output("export class C extends ((null)) {}");
    assert_eq!(plain_diagnostics, 0);
    assert_eq!(wrapped_diagnostics, plain_diagnostics);
}

#[test]
fn inferred_reference_identity() {
    let (code, diagnostics) = declaration_output("export function f(x?: number) { return x; }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("f(x?: number): number | undefined"));

    let (code, diagnostics) =
        declaration_output("export function f(undefined?: number) { return (undefined); }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("f(undefined?: number): number | undefined"));

    let (code, diagnostics) = declaration_output("export function f(x: number = 1) { return x; }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("f(x?: number): number;"));

    let (code, diagnostics) =
        declaration_output("type T = { value: string }; export function f(x?: T) { return x; }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type T"));
    assert!(code.contains("f(x?: T): T | undefined"));

    let (code, diagnostics) =
        declaration_output("export declare namespace N { enum E { A = 5, B, C } }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("A = 5"));
    assert!(!code.contains("B ="));
    assert!(!code.contains("C ="));

    let (code, diagnostics) = declaration_output("export declare const enum E { A = 5, B }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("B = 6"));

    let (code, diagnostics) =
        declaration_output("export function f(undefined: number) { return (undefined); }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("f(undefined: number): number"));

    let (code, diagnostics) =
        declaration_output("export enum E { Infinity = 2, A = Infinity, NaN = 3, B = NaN }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("A = 2"));
    assert!(code.contains("B = 3"));

    let (code, diagnostics) = declaration_output("export enum E { A = '' + 1e21, B = '' + 1e-7 }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("\"1e+21\""));
    assert!(code.contains("\"1e-7\""));

    let (code, diagnostics) = declaration_output(
        "export const value = { get '\\uD800'(): number { return 1; }, set '\\uD801'(value: string) {} };",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("readonly \"\\uD800\": number"));
    assert!(code.contains("\"\\uD801\": string"));
    let (code, diagnostics) =
        declaration_output("export enum E { A = +\"1\", B = -\"2\", C = ~\"3\" }");
    assert_eq!(diagnostics, 0);
    assert!(!code.contains('='));
}

#[test]
fn computed_object_dependencies() {
    let (code, diagnostics) = declaration_output(
        "declare const key: unique symbol; export const o = { get [key](): number { return 1; }, set key(v: string) {} };",
    );
    assert_eq!(diagnostics, 1);
    assert!(!code.contains("declare const key"));
    assert!(code.contains("key: string"));
}

#[test]
fn accessor_declaration_dependencies() {
    let (code, diagnostics) = declaration_output(
        "type Hidden = { x: number }; type Public = object; export const obj = { get value() { return {} as Hidden; }, set value(v: Public) {} };",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));
    assert!(code.contains("type Public"));
    assert!(code.contains("value: Public"));

    let (code, diagnostics) = declaration_output(
        "type Hidden = { x: number }; type Public = object; export class C { get value() { return {} as Hidden; } set value(v: Public) {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));
    assert!(code.contains("type Public"));
    assert!(code.contains("get value(): Public"));
    assert!(code.contains("set value(v: Public)"));

    let (code, diagnostics) = declaration_output(
        "type Read = { x: number }; type Write = object; export const obj = { get value(): Read { return { x: 1 }; }, set value(v: Write) {} };",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type Read"));
    assert!(code.contains("type Write"));
    assert!(code.contains("get value(): Read"));
    assert!(code.contains("set value(v: Write)"));

    let (code, diagnostics) = declaration_output(
        "type Read = { x: number }; type Write = object; export const obj = { set value(v: Write) {}, get value(): Read { return { x: 1 }; } };",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("type Read"));
    assert!(code.contains("type Write"));
    assert!(code.contains("get value(): Read"));
    assert!(code.contains("set value(v: Write)"));
    let (code, diagnostics) = declaration_output(
        "import type { T } from './dep'; export declare class C { get value(): T; set value(v); }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("import type { T }"));
    assert!(code.contains("set value(v: T)"));

    let (code, diagnostics) = declaration_output(
        "export abstract class C { abstract set value(v); abstract get value(): string; }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("set value(v: string)"));
}

#[test]
fn numeric_member_identity() {
    let (code, diagnostics) = declaration_output(
        "export const obj = { get [-1](): number { return 1; }, set ['-1'](v: string) {} };",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("get [-1](): number"));
    assert!(code.contains("set [\"-1\"](v: string)"));

    let (code, diagnostics) = declaration_output(
        "export class C { [-1](x: string): string; ['-1'](x: number): number; [-(1)](x: string | number): string | number { return x; } }",
    );
    assert_eq!(diagnostics, 0);
    assert_eq!(code.matches("(x:").count(), 2);
    assert!(!code.contains("string | number"));

    let (code, diagnostics) = declaration_output(
        "export const obj = { get [-(0)](): number { return 1; }, set [0](v: string) {} };",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("get [0](): number"));
    assert!(code.contains("set [0](v: string)"));

    let (code, diagnostics) = declaration_output(
        "export class C { get [-0](): number { return 1; } set [(0)](v: string) {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("get [0](): number"));
    assert!(code.contains("set [0](v: string)"));
}

#[test]
fn private_setter_type_dependencies() {
    let (code, diagnostics) = declaration_output(
        "type Public = object; type Hidden = { x: number }; export class C { get value() { return {} as Hidden; } private set value(v: Public) {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));
    assert!(code.contains("type Public"));
    assert!(code.contains("get value(): Public"));
    assert!(code.contains("private set value(value)"));

    let (code, diagnostics) = declaration_output(
        "type Public = object; type Hidden = { x: number }; export class C { private set value(v: Public) {} get value() { return {} as Hidden; } }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));
    assert!(code.contains("get value(): Public"));

    let (code, diagnostics) = declaration_output(
        "type Hidden = object; export class C { private set value(v: Hidden) {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));

    let (code, diagnostics) = declaration_output(
        "interface Hidden { x: string } export declare class C { private get value(): Hidden; private set value(v: Hidden); }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));
    assert!(code.contains("private get value()"));
    assert!(code.contains("private set value(value)"));

    let (code, diagnostics) = declaration_output(
        "interface Hidden { x: string } export declare class C { private get value(): Hidden; }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));

    let (code, diagnostics) = declaration_output(
        "interface Hidden { x: string } export declare class C { private set value(v: Hidden); }",
    );
    assert_eq!(diagnostics, 0);
    assert!(!code.contains("Hidden"));

    let (code, diagnostics) = declaration_output(
        "interface Hidden { x: string } export declare class C { get value(); private set value(v: Hidden); }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("interface Hidden"));
    assert!(code.contains("get value(): Hidden"));
    assert!(code.contains("private set value(value)"));
}

#[test]
fn parameter_properties() {
    let (code, diagnostics) = declaration_output(
        "export class C { constructor(public value = undefined as number | undefined) {} }",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("value: number | undefined;"));
    assert!(code.contains("constructor(value?: number | undefined)"));

    let (code, diagnostics) =
        declaration_output("export class C { constructor(public value = 1, required: string) {} }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("value: number;"));
    assert!(code.contains("value: number | undefined, required: string"));

    let (_, diagnostics) = declaration_output(
        "declare function compute(): number; export class C { constructor(public value = compute()) {} }",
    );
    assert_eq!(diagnostics, 1);
}

#[test]
fn declaration_diagnostics() {
    let (code, diagnostics) =
        declaration_output("export = class C { clone(): C { return this; } };");
    assert!(diagnostics > 0);
    assert!(!code.contains("clone(): C"));

    let (code, diagnostics) =
        declaration_output("export = (class C { clone(): C { return this; } });");
    assert!(diagnostics > 0);
    assert!(!code.contains("clone(): C"));

    let (code, diagnostics) = declaration_output("export = class C { m(): void { new C(); } };");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("m(): void"));

    let (code, diagnostics) = declaration_output("export = (class { x: number = 1; });");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("x: number"));

    let (code, diagnostics) =
        declaration_output("export default class C { clone(): C { return this; } }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("clone(): C"));

    let (code, diagnostics) = declaration_output(
        "interface C { value: string }; export const value = {} as C; export default (class C { x: number = 1; });",
    );
    assert_eq!(diagnostics, 0);
    assert!(code.contains("interface C"));
    assert!(code.contains("x: number"));

    let (_, diagnostics) =
        declaration_output("export function f() { type Local = string; return 'hello' as Local; }");
    assert_eq!(diagnostics, 1);

    let (_, diagnostics) =
        declaration_output("export function f() { const value = 1; return 1 as typeof value; }");
    assert_eq!(diagnostics, 1);

    let (_, diagnostics) = declaration_output(
        "export function f() { type Local = string; return { get value(): Local { return ''; }, set value(v) {} }; }",
    );
    assert_eq!(diagnostics, 2);

    let (code, diagnostics) =
        declaration_output("export function f<T>(value: T) { return value as T; }");
    assert_eq!(diagnostics, 0);
    assert!(code.contains("f<T>(value: T): T"));

    let (_, diagnostics) =
        declaration_output("export function f() { return <T>(value: T): T => value; }");
    assert_eq!(diagnostics, 0);

    let (_, diagnostics) = declaration_output(
        "function hidden() { type Local = string; return '' as Local; } export const value = 1;",
    );
    assert_eq!(diagnostics, 0);

    let (_, diagnostics) = declaration_output("export let value;");
    assert_eq!(diagnostics, 1);

    let (_, diagnostics) = declaration_output("export class C { value; }");
    assert_eq!(diagnostics, 1);
}
