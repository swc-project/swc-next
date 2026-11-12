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
    let mut parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let analyzed = analyze(&mut parsed.ast, AnalyzeOptions::default());
    let (_, semantic) = analyzed.semantic.into_parts();
    // SAFETY: The inspected handles belong to this AST.
    let ast = unsafe { parsed.ast.ast() };
    let analysis = crate::typescript::analysis::TypeScriptAnalysis::analyze(
        ast.root_program(),
        ast,
        &semantic,
        &TypeScriptOptions::default(),
        Default::default(),
    );
    let scope = swc_next_ecma_ast::ScopeId::MODULE;
    let outer = semantic.binding(scope, b"Alias").unwrap();
    assert!(analysis.has_runtime_usage(outer, &semantic));
    let original = semantic.binding(scope, b"Original").unwrap();
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
            let mut ctx = parsed.ast;
            Normalizer::normalize(&mut ctx);
            let analyzed = analyze(&mut ctx, AnalyzeOptions::default()).semantic;
            let options = TypeScriptOptions {
                verbatim_module_syntax,
                ..TypeScriptOptions::default()
            };
            let mut transformer = MultiPassTransformer::new(analyzed);
            transformer.typescript(&options);
            // SAFETY: The inspected statements belong to the transformed AST.
            let ast = unsafe { transformer.ctx.ast() };
            let body = ast.root_program().body(ast);
            let retained = body.iter().any(|id| {
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
