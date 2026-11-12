use super::*;

fn assert_local_namespace_references(code: &str) {
    with_output_semantic(code, |ast, semantic| {
        let references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name) == "F");
        let mut count = 0;
        for entry in references {
            let reference = entry.reference.resolution();
            let symbol = reference.symbol().unwrap();
            let scope = semantic.scope_of_symbol(symbol);
            assert_ne!(scope, swc_next_ecma_ast::ScopeId::MODULE);
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
        let has_binding = semantic.iter_symbols().any(|entry| {
            let name = ast.get_wtf8(entry.symbol.name);
            name.as_bytes() == b"N"
        });
        assert!(has_binding);
        let mut references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name) == "N");
        assert!(references.all(|entry| entry.reference.resolution().symbol().is_some()));
    });
}

#[test]
fn namespace_runtime_exports() {
    let code = transformed_code("namespace N { const enum E { A=0 } } export { N };");
    assert_namespace_binding(&code);

    let code = transformed_code("namespace N { const enum E { A=0 } } export default N;");
    assert_namespace_binding(&code);

    let code = transformed_code("namespace N { const enum E { A=0 } } console.log(N);");
    assert_namespace_binding(&code);

    let code = transformed_code("export namespace N { const enum E { A=0 } }");
    assert_namespace_binding(&code);

    let code = transformed_code(
        "namespace N { export namespace M { const enum E { A=0 } } } export { N };",
    );
    assert_namespace_binding(&code);
}
