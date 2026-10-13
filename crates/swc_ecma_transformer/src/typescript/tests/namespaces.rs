use super::*;

#[test]
fn namespace_opaque_aliases() {
    let code = transformed_code(
        "import * as remote from 'pkg'; namespace N { export function f(){return value;} export import value=remote.value; }",
    );
    assert!(code.contains("N.value = remote.value"));
    assert!(code.contains("return N.value"));

    let code = transformed_code(
        "import * as remote from 'pkg'; namespace N { export import value=remote.value; export namespace M { export const result=value; } }",
    );
    assert!(code.contains("M.result = N.value"));

    let code = transformed_code(
        "import * as remote from 'pkg'; declare namespace N { import value=remote.value; export {value as x}; } namespace N { export function f(){return x;} }",
    );
    assert!(code.contains("return N.x"));

    let code = transformed_code(
        "import * as remote from 'pkg'; const x=42; declare namespace N { import value=remote.value; export type {value as x}; } namespace N { export function f(){return x;} }",
    );
    assert!(code.contains("return x"));
    assert!(!code.contains("return N.x"));

    let code = transformed_code(
        "export {}; const value=42; namespace Types { export interface T {} } namespace N { export import value=Types.T; export function f(){return value;} }",
    );
    assert!(code.contains("return value"));
    assert!(!code.contains("return N.value"));

    let code = transformed_code(
        "import * as remote from 'pkg'; namespace N { import value=remote.value; export function f(){return value;} }",
    );
    assert!(code.contains("var value = remote.value"));
    assert!(code.contains("return value"));

    let code = transformed_code(
        "import * as remote from 'pkg'; namespace N { export import value=remote.value; export function f(value:number){return value;} }",
    );
    assert!(code.contains("return value"));
    assert!(!code.contains("return N.value"));

    let code = transformed_code(
        "import * as remote from 'pkg'; namespace N { export import Infinity=remote.value; export enum E { A=Infinity } }",
    );
    assert!(code.contains("E[\"A\"] = N.Infinity"));

    let code = transformed_code_with_options(
        "import * as remote from 'pkg'; namespace N { export import value=remote.value; export function f(){return value;} }",
        TypeScriptOptions {
            verbatim_module_syntax: true,
            ..TypeScriptOptions::default()
        },
    );
    assert!(code.contains("return N.value"));
}

#[test]
fn namespace_export_aliases() {
    let code = transformed_code(
        "declare namespace N { enum E { A=42 } export { E as F }; } enum G { A=N.F.A, B }",
    );
    assert!(code.contains("G[\"B\"] = 43"));
    assert!(!code.contains("N.F"));

    let code = transformed_code(
        "declare namespace N { enum E { A=42 } import A=E.A; export { A as X }; } enum G { A=N.X, B }",
    );
    assert!(code.contains("G[\"B\"] = 43"));

    let code = transformed_code(
        "declare namespace N { const a:number; export { a as x }; } namespace N { export function f(){ return x; } }",
    );
    assert!(code.contains("return N.x"));

    let code = transformed_code(
        "declare namespace N { function f():number; export { f as g }; } namespace N { export function f(){ return 42; } export namespace M { export function h(){ return g; } } }",
    );
    assert!(code.contains("return N.g"));

    let code = transformed_code(
        "declare namespace N { const a:number; export { a as x }; } namespace N { export function f(){ const x=7; return x; } }",
    );
    assert!(code.contains("return x"));
    assert!(!code.contains("return N.x"));
}

#[test]
fn namespace_destructuring() {
    let code = transformed_code("namespace N { export let x=1; ({x}={x:2}); }");
    assert!(code.contains("{ x: N.x } = { x: 2 }"));

    let code = transformed_code("namespace N { export let x=1; ({x=3}={}); }");
    assert!(code.contains("{ x: N.x = 3 } = {}"));

    let code = transformed_code("namespace N { export let x=1; for ({x} of [{x:2}]) {} }");
    assert!(code.contains("for ({ x: N.x } of"));

    let code = transformed_code("namespace N { export const {x=3}={}; }");
    assert!(code.contains("{ x: N.x = 3 } = {}"));
}

fn assert_local_namespace_references(code: &str) {
    with_output_semantic(code, |ast, semantic| {
        let references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name) == "F");
        let mut count = 0;
        for entry in references {
            let symbol = entry
                .reference
                .resolution()
                .symbol()
                .expect("namespace reference is bound");
            assert_ne!(
                semantic.scope_of_symbol(symbol),
                swc_next_ecma_ast::ScopeId::MODULE
            );
            count += 1;
        }
        assert!(count > 0);
    });
}

#[test]
fn namespace_binding_scopes() {
    let code = transformed_code(
        "namespace N { export function F() {} } namespace N { export namespace F { export const x = 2; } }",
    );
    assert_local_namespace_references(&code);

    let code = transformed_code(
        "namespace N { export class F {} } namespace N { export namespace F { export const x = 2; } }",
    );
    assert_local_namespace_references(&code);

    let code = transformed_code(
        "namespace N { export function F() {} export namespace F { export const x = 2; } }",
    );
    assert_local_namespace_references(&code);

    let code = transformed_code(
        "namespace N { export class F {} export namespace F { export const x = 2; } }",
    );
    assert_local_namespace_references(&code);

    let code = transformed_code(
        "let F = 7; namespace N { export function F() {} } namespace N { export namespace F { export const x = 2; } }",
    );
    assert_local_namespace_references(&code);
}

fn assert_namespace_binding(code: &str) {
    with_output_semantic(code, |ast, semantic| {
        assert!(
            semantic
                .iter_symbols()
                .any(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"N")
        );
        assert!(
            semantic
                .iter_references()
                .filter(|entry| ast.get_utf8(entry.reference.name) == "N")
                .all(|entry| entry.reference.resolution().symbol().is_some())
        );
    });
}

#[test]
fn namespace_runtime_exports() {
    let code = transformed_code("namespace N { const enum E { A=0 } } export { N };");
    assert_namespace_binding(&code);
    assert!(code.contains("export { N }"));
    assert!(!code.contains("E["));

    let code = transformed_code("namespace N { const enum E { A=0 } } export default N;");
    assert_namespace_binding(&code);
    assert!(code.contains("export default N"));

    let code = transformed_code("namespace N { const enum E { A=0 } } console.log(N);");
    assert_namespace_binding(&code);

    let code = transformed_code("export namespace N { const enum E { A=0 } }");
    assert_namespace_binding(&code);
    assert!(code.contains("export var N"));

    let code = transformed_code(
        "namespace N { export namespace M { const enum E { A=0 } } } export { N };",
    );
    assert_namespace_binding(&code);
    assert!(code.contains("N.M"));

    let code = transformed_code("namespace N { const enum E { A=0 } } export type { N };");
    assert!(!code.contains("var N"));

    let code = transformed_code("namespace N { interface T {} } export type { N };");
    assert!(!code.contains("var N"));
}

#[test]
fn namespace_dotted_bindings() {
    let dotted = transformed_code(
        "let seen: boolean; namespace N.M { export const x=1; const original=M; Object.defineProperty(N, 'M', {value:{}, configurable:true}); seen=M===original; }",
    );
    let nested = transformed_code(
        "let seen: boolean; namespace N { export namespace M { export const x=1; const original=M; Object.defineProperty(N, 'M', {value:{}, configurable:true}); seen=M===original; } }",
    );
    assert_eq!(dotted, nested);
    assert!(dotted.contains("const original = M;"));
    assert!(dotted.contains("seen = M === original;"));

    let dotted = transformed_code(
        "namespace N.M.P { export const original=M; Object.defineProperty(N, 'M', {value:{}, configurable:true}); export const seen=M===original; }",
    );
    let nested = transformed_code(
        "namespace N { export namespace M { export namespace P { export const original=M; Object.defineProperty(N, 'M', {value:{}, configurable:true}); export const seen=M===original; } } }",
    );
    assert_eq!(dotted, nested);
    assert!(dotted.contains("P.original = M;"));

    let dotted = transformed_code(
        "namespace N.M { const M=1; export const value=M; } namespace N.M { export const value2=M; }",
    );
    let nested = transformed_code(
        "namespace N { export namespace M { const M=1; export const value=M; } } namespace N { export namespace M { export const value2=M; } }",
    );
    assert_eq!(dotted, nested);
}
