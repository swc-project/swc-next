use super::*;

fn transformed_script(source: &str) -> String {
    transformed_code_with_source_type(source, Default::default(), Lang::Ts, SourceType::Script)
}

#[test]
fn global_aliases() {
    let source = "namespace N { export const value = 7; } import Alias = N;";
    let code = transformed_script(source);
    assert!(code.contains("var Alias = N;"));

    let code = transformed_code(source);
    assert!(!code.contains("Alias"));

    let source = "namespace N { export const value = 7; } import A = N; import B = A;";
    let code = transformed_script(source);
    assert!(code.contains("var A = N;"));
    assert!(code.contains("var B = A;"));

    let code = transformed_code(source);
    assert!(!code.contains("var A"));
    assert!(!code.contains("var B"));

    let code = transformed_script("namespace N { const enum E { A = 1 } } import Alias = N;");
    assert!(code.contains("N ||"));
    assert!(code.contains("var Alias = N;"));

    let code = transformed_script("namespace N { export const value = 7; } import type Alias = N;");
    assert!(!code.contains("Alias"));

    let code = transformed_script("namespace N { export interface T {} } import Alias = N.T;");
    assert!(!code.contains("Alias"));

    let code = transformed_script(
        "namespace N { export const value = 7; } namespace Other { import Alias = N; }",
    );
    assert!(!code.contains("Alias"));
}

#[test]
fn uncertain_alias_dependencies() {
    let allocator = Allocator::new();
    let source = "namespace Original { export const value = 1; } import Alias = Original; namespace Unknown { export interface Marker {} } namespace Use { import Alias = Unknown.Member; import Child = Alias.value; export const result = Child; }";
    let options = ParserOptions {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..ParserOptions::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let ast = parsed.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let analysis = crate::typescript::analysis::TypeScriptAnalysis::analyze(
        ast.root_program(),
        &ast,
        &semantic,
        &TypeScriptOptions::default(),
    );
    let outer = semantic
        .binding(swc_next_ecma_ast::ScopeId::MODULE, b"Alias")
        .unwrap();
    assert!(analysis.has_runtime_usage(outer, &semantic));
    let original = semantic
        .binding(swc_next_ecma_ast::ScopeId::MODULE, b"Original")
        .unwrap();
    assert!(analysis.has_runtime_usage(original, &semantic));
}

fn assert_default_export(source: &str, value: bool) {
    let allocator = Allocator::new();
    for preserve_parens in [false, true] {
        for verbatim_module_syntax in [false, true] {
            let options = ParserOptions {
                lang: Lang::Ts,
                source_type: SourceType::Module,
                preserve_parens,
                ..ParserOptions::default()
            };
            let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
            assert!(parsed.diagnostics.is_empty());
            let mut ast = parsed.ast;
            let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
            let program = ast.root_program();
            Normalizer::normalize(program, &mut ast);
            let options = TypeScriptOptions {
                verbatim_module_syntax,
                ..TypeScriptOptions::default()
            };
            transform(program, &mut ast, &semantic, options);
            let retained = program.body(&ast).iter().any(|id| {
                let statement = ast.get_node_in_sub_range(id);
                matches!(
                    ast.stmt_data(statement),
                    swc_next_ecma_ast::StmtData::ExportDefaultDeclaration(_)
                )
            });
            assert_eq!(retained, value || verbatim_module_syntax);
        }
    }
}

#[test]
fn default_export_values() {
    assert_default_export("interface X {} export default X;", false);

    assert_default_export("interface X {} export default (X);", false);

    assert_default_export("interface X {} export const X = 1; export default X;", true);

    assert_default_export(
        "export const X = 1; interface X {} export default (X);",
        true,
    );

    assert_default_export("export default unknownValue;", true);
}

#[test]
fn exported_alias_values() {
    let code = transformed_code("namespace N { export interface T {} } export import A = N.T;");
    assert!(!code.contains("N.T"));

    let code = transformed_code(
        "namespace N { export interface T {} } export namespace Use { export import A = N.T; }",
    );
    assert!(!code.contains("N.T"));
    assert!(code.contains("Use ||"));

    let code = transformed_code("namespace N { export const x = 1; } export import A = N.x;");
    assert!(code.contains("export var A = N.x;"));

    let code = transformed_code(
        "namespace N { export const x = 1; } export namespace Use { export import A = N.x; }",
    );
    assert!(code.contains("Use.A = N.x;"));

    let code = transformed_code(
        "namespace N { export namespace Inner { export interface T {} } } export import A = N.Inner.T;",
    );
    assert!(!code.contains("N.Inner.T"));

    let code = transformed_code(
        "class C { static x = 1; } namespace C { export interface x {} } export import A = C.x;",
    );
    assert!(code.contains("export var A = C.x;"));

    let code = transformed_code("import * as N from 'pkg'; export import A = N.Member;");
    assert!(code.contains("export var A = N.Member;"));
    assert!(code.contains("import * as N"));

    let code = transformed_code(
        "namespace Original { export interface T {} export const x = 1; } import Root = Original; export import A = Root.T;",
    );
    assert!(!code.contains("var Root"));
    assert!(!code.contains("Root.T"));
    assert!(code.contains("Original.x = 1"));
}

#[test]
fn module_indicators() {
    let code = transformed_code("const value = 1; export = value;");
    assert!(code.contains("export = value;"));
    assert!(!code.contains("export {};"));

    let code = transformed_code("export interface T {}");
    assert_eq!(code.trim(), "export {};");

    let code = transformed_code("import Value = require('pkg'); export = Value;");
    assert!(code.contains("export = Value;"));
    assert!(!code.contains("export {};"));
}

#[test]
fn module_side_effects() {
    let code = transformed_code("import {} from 'dep';");
    assert_eq!(code.trim(), "import \"dep\";");

    let code = transformed_code("import 'dep';");
    assert_eq!(code.trim(), "import \"dep\";");

    let code = transformed_code("export {} from 'dep';");
    assert_eq!(code.trim(), "export {} from \"dep\";");

    let code = transformed_code("import type {} from 'dep';");
    assert_eq!(code.trim(), "export {};");

    let code = transformed_code("export type {} from 'dep';");
    assert_eq!(code.trim(), "export {};");

    let verbatim = TypeScriptOptions {
        verbatim_module_syntax: true,
        ..TypeScriptOptions::default()
    };
    let code = transformed_code_with_options("import {} from 'dep';", verbatim.clone());
    assert_eq!(code.trim(), "import \"dep\";");

    let code = transformed_code_with_options("import 'dep';", verbatim.clone());
    assert_eq!(code.trim(), "import \"dep\";");

    let code = transformed_code_with_options("export {} from 'dep';", verbatim.clone());
    assert_eq!(code.trim(), "export {} from \"dep\";");

    let code = transformed_code_with_options("import type {} from 'dep';", verbatim.clone());
    assert_eq!(code.trim(), "export {};");

    let code = transformed_code_with_options("export type {} from 'dep';", verbatim);
    assert_eq!(code.trim(), "export {};");
}
