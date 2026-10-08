use super::*;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

fn collect_transform_info<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    features: TransformFeatureFlags,
) -> (Ast<'a>, TransformAnalysis<'a>) {
    let parsed = Parser::init(allocator, source, Options::default(), NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let analysis = TransformAnalysis::collect(&parsed.ast, features);
    (parsed.ast, analysis)
}

fn first_class(ast: &Ast<'_>) -> swc_next_ecma_ast::Class {
    ast.root_program()
        .body(ast)
        .get_node(ast, 0)
        .unwrap()
        .as_declaration(ast)
        .unwrap()
        .as_class(ast)
        .unwrap()
}

fn first_static_block(ast: &Ast<'_>) -> swc_next_ecma_ast::NodeId {
    first_class(ast)
        .body(ast)
        .body(ast)
        .get_node(ast, 0)
        .unwrap()
        .node_id()
}

#[test]
fn static_block_layout() {
    use crate::TransformFeatureFlags as Features;
    let allocator = Allocator::new();
    let features = Features::ES2022_STATIC_BLOCKS | Features::PROPOSAL_PREVENT_EXTENSIONS;
    let source = "class C extends B { static before = 0; static field = class { static nested = 1; static { inner(); } }; static { super.x; } static { const x = super.y; } static {} }";
    let (ast, analysis) = collect_transform_info(&allocator, source, features);
    let class = first_class(&ast);
    let layout = analysis.static_block_layout(class.node_id()).unwrap();
    assert!(layout.has_static_blocks);
    assert_eq!(layout.trailing_blocks, 2);

    assert_eq!(
        ast.get_utf8(
            layout
                .first_static_property
                .unwrap()
                .key(&ast)
                .as_identifier_name(&ast)
                .unwrap()
                .name(&ast)
        ),
        "before"
    );
    assert_eq!(
        ast.get_utf8(
            layout
                .last_static_property
                .unwrap()
                .key(&ast)
                .as_identifier_name(&ast)
                .unwrap()
                .name(&ast)
        ),
        "field"
    );

    let (ast, analysis) = collect_transform_info(&allocator, source, Features::empty());
    assert!(
        analysis
            .static_block_layout(first_class(&ast).node_id())
            .is_none()
    );

    let source = "class C extends B { static { super.x; } static field = 1; static {} }";
    let (ast, analysis) = collect_transform_info(&allocator, source, features);
    let layout = analysis
        .static_block_layout(first_class(&ast).node_id())
        .unwrap();
    assert_eq!(layout.trailing_blocks, 0);

    let source = "class C { static {} // keep\n}";
    let (ast, analysis) = collect_transform_info(
        &allocator,
        source,
        Features::ES2022_STATIC_BLOCKS | Features::PROPOSAL_PREVENT_EXTENSIONS,
    );
    let layout = analysis
        .static_block_layout(first_class(&ast).node_id())
        .unwrap();
    assert_eq!(layout.trailing_blocks, 1);
    assert!(layout.first_static_property.is_none());
    assert!(layout.last_static_property.is_none());
    assert!(analysis.static_block_has_comments(first_static_block(&ast)));
}
