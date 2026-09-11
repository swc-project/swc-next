use crate::{
    TransformerOptions,
    tests::{assert_reference_scopes, transform_source},
};
use swc_next_allocator::Allocator;

fn transform(source: &str, arrows: bool) -> String {
    let allocator = Allocator::new();
    let mut options = TransformerOptions::default();
    options.es2022.static_blocks = true;
    options.proposal.prevent_extensions = true;
    if arrows {
        options.es2015.arrow_functions = Some(Default::default());
    }
    let (ast, semantic) = transform_source(&allocator, source, options);
    assert_reference_scopes(&ast, &semantic);
    swc_next_ecma_codegen::print(&allocator, &ast, Default::default())
        .code
        .to_owned()
}

#[test]
fn prototype_setter_names() {
    let source = "let seen; ({ __proto__: class { static { seen = this.name; 0; } } });";
    let output = transform(source, false);
    assert!(!output.contains("_set_function_name"));

    let output = transform("({ '__proto__': class { static { 0; 0; } } });", false);
    assert!(!output.contains("_set_function_name"));

    let output = transform("({ ['__proto__']: class { static { 0; 0; } } });", false);
    assert!(output.contains("_set_function_name(this, \"__proto__\")"));

    let output = transform("class C { __proto__ = class { static { 0; 0; } }; }", false);
    assert!(output.contains("_set_function_name(this, \"__proto__\")"));
}

#[test]
fn computed_instance_field_names() {
    let source = "const classes = []; for (const key of ['a', 'b']) classes.push(class { [key] = class { static { 0; 0; } } });";
    let output = transform(source, false);
    assert!(output.contains("[key] = class {"));
    assert!(!output.contains("_computedKey"));
    assert!(!output.contains("_set_function_name"));

    let source =
        "class C { [key()] = (class { static { Object.freeze(this); } static { use(this); } }); }";
    let output = transform(source, false);
    assert!(output.contains("[key()] = class {"));
    assert!(!output.contains("_computedKey"));

    let source = "class C { [key] = class { static { use(this); 0; } }; }";
    let output = transform(source, true);
    assert!(output.contains("[key] = class {"));
    assert!(!output.contains("_computedKey"));

    let source = "class C { ['literal'] = class { static { 0; 0; } }; }";
    let output = transform(source, false);
    assert!(output.contains("_set_function_name(this, \"literal\")"));

    let source = "class C { [key] = class { static { init(); } static x = 1; }; }";
    let output = transform(source, false);
    assert!(output.contains("static x = (init(), 1)"));
    assert!(!output.contains("static #"));
}
