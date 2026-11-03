use super::*;

#[test]
fn async_function_lexical_environment_facts() {
    let allocator = Allocator::new();
    let (_, semantic) = analyze_source(
        &allocator,
        "async function outer() { const inner = async () => [this, arguments, await value]; await done; }",
        Options::default(),
    );
    let facts = semantic.transform_info().async_functions_post_order();
    assert_eq!(facts.len(), 2);
    assert!(facts[0].uses_this);
    assert!(facts[0].uses_arguments);
    assert!(facts[1].uses_this);
    assert!(facts[1].uses_arguments);

    let allocator = Allocator::new();
    let (_, semantic) = analyze_source(
        &allocator,
        "async function outer() { function barrier() { return async () => arguments; } await value; }",
        Options::default(),
    );
    let facts = semantic.transform_info().async_functions_post_order();
    assert_eq!(facts.len(), 2);
    assert!(facts[0].uses_arguments);
    assert!(!facts[1].uses_arguments);

    let allocator = Allocator::new();
    let (_, semantic) = analyze_source(
        &allocator,
        "async function shadow(arguments) { await arguments; }",
        Options::default(),
    );
    let facts = semantic.transform_info().async_functions_post_order();
    assert_eq!(facts.len(), 1);
    assert!(!facts[0].uses_arguments);

    let allocator = Allocator::new();
    let (_, semantic) = analyze_source(
        &allocator,
        "const arrow = async (value = 1) => value;",
        Options::default(),
    );
    let facts = semantic.transform_info().async_functions_post_order();
    assert_eq!(facts.len(), 1);
    assert!(facts[0].has_parameter_expressions);
}
