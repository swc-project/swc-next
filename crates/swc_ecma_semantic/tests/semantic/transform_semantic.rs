use super::*;

fn analyze_transform_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (Ast<'a>, Semantic<'a>) {
    analyze_with_options(
        allocator,
        source,
        Options::default(),
        AnalyzeOptions {
            features: swc_next_ecma_semantic::SemanticFeatureFlags::ASYNC_TO_GENERATOR,
            ..AnalyzeOptions::default()
        },
    )
}

#[test]
fn async_transform_semantic_is_opt_in() {
    let allocator = Allocator::new();
    let (_, semantic) = analyze_source(
        &allocator,
        "async function unused() { await value; }",
        Options::default(),
    );
    assert!(semantic.transform_semantic().is_none());

    let allocator = Allocator::new();
    let (_, semantic) = analyze_transform_source(
        &allocator,
        "async function outer() { const inner = async () => [this, arguments, await value]; await done; }",
    );
    let transform_semantic = semantic
        .transform_semantic()
        .expect("async transform semantic data");
    assert_eq!(
        transform_semantic.features(),
        swc_next_ecma_semantic::SemanticFeatureFlags::ASYNC_TO_GENERATOR
    );
    let facts = transform_semantic.async_functions_post_order();
    assert_eq!(facts.len(), 2);
    assert!(facts[0].uses_this);
    assert!(facts[0].uses_arguments);
    assert!(facts[1].uses_this);
    assert!(facts[1].uses_arguments);

    let allocator = Allocator::new();
    let (_, semantic) = analyze_transform_source(
        &allocator,
        "async function outer() { function barrier() { return async () => arguments; } await value; }",
    );
    let facts = semantic
        .transform_semantic()
        .expect("async transform semantic data")
        .async_functions_post_order();
    assert_eq!(facts.len(), 2);
    assert!(facts[0].uses_arguments);
    assert!(!facts[1].uses_arguments);

    let allocator = Allocator::new();
    let (_, semantic) = analyze_transform_source(
        &allocator,
        "async function shadow(arguments) { await arguments; }",
    );
    let facts = semantic
        .transform_semantic()
        .expect("async transform semantic data")
        .async_functions_post_order();
    assert_eq!(facts.len(), 1);
    assert!(!facts[0].uses_arguments);

    let allocator = Allocator::new();
    let (_, semantic) =
        analyze_transform_source(&allocator, "const arrow = async (value = 1) => value;");
    let facts = semantic
        .transform_semantic()
        .expect("async transform semantic data")
        .async_functions_post_order();
    assert_eq!(facts.len(), 1);
    assert!(facts[0].has_parameter_expressions);
}
