use super::*;

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
