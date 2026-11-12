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
