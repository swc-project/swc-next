use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

use crate::{
    AnalyzeOptions, ExportOrigin, ExportTarget, ReferenceSpace, Semantic, SymbolFlags, analyze,
    scope::ScopeKind,
};

fn analyze_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
) -> (Ast<'a>, Semantic<'a>) {
    analyze_with_options(allocator, source, parser_options, AnalyzeOptions::default())
}

fn analyze_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
    analyze_options: AnalyzeOptions,
) -> (Ast<'a>, Semantic<'a>) {
    let parse = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
    let ast = parse.ast;
    assert!(parse.diagnostics.is_empty());

    let result = analyze(&ast, analyze_options);
    assert!(result.diagnostics.is_empty());
    (ast, result.semantic)
}

fn first_node(ast: &Ast<'_>, kind: swc_next_ecma_ast::NodeKind) -> swc_next_ecma_ast::NodeId {
    (0..ast.node_count())
        .map(swc_next_ecma_ast::NodeId::from_usize)
        .find(|&node| ast.node_kind(node) == kind)
        .unwrap()
}

fn collect_transform_info<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    features: crate::SemanticFeatureFlags,
) -> (Ast<'a>, Semantic<'a>) {
    analyze_with_options(
        allocator,
        source,
        Options::default(),
        AnalyzeOptions {
            features,
            ..AnalyzeOptions::default()
        },
    )
}

#[test]
fn class_feature_flags() {
    use crate::SemanticFeatureFlags as Features;
    use swc_next_ecma_ast::NodeKind;
    let allocator = Allocator::new();
    let source = "class C extends B { constructor() { const f = () => this; super(); } }";
    let (ast, semantic) =
        collect_transform_info(&allocator, source, Features::DERIVED_CONSTRUCTORS);
    let constructor = first_node(&ast, NodeKind::Function);
    assert!(semantic.class().is_derived_constructor(constructor));
    assert!(!semantic.class().constructor_needs_this_capture(constructor));
    assert!(
        semantic
            .class()
            .super_call_constructor(first_node(&ast, NodeKind::CallExpression))
            .is_none()
    );

    let (ast, semantic) = collect_transform_info(&allocator, source, Features::CONSTRUCTOR_THIS);
    let constructor = first_node(&ast, NodeKind::Function);
    assert!(semantic.class().constructor_needs_this_capture(constructor));
    assert!(
        semantic
            .class()
            .super_call_constructor(first_node(&ast, NodeKind::CallExpression))
            .is_none()
    );

    let (ast, semantic) = collect_transform_info(&allocator, source, Features::CONSTRUCTOR_SUPER);
    let constructor = first_node(&ast, NodeKind::Function);
    assert!(!semantic.class().constructor_needs_this_capture(constructor));
    assert_eq!(
        semantic
            .class()
            .super_call_constructor(first_node(&ast, NodeKind::CallExpression)),
        Some(constructor)
    );

    let source = "class C extends B { field = () => super.value; }";
    let (ast, semantic) = collect_transform_info(&allocator, source, Features::ARROW_ENVIRONMENTS);
    let arrow = first_node(&ast, NodeKind::ArrowFunctionExpression);
    assert!(semantic.class().property_initializer_owner(arrow).is_some());
    assert!(!semantic.class().arrow_preserves_lexical_super(arrow));

    let (ast, semantic) = collect_transform_info(&allocator, source, Features::LEXICAL_SUPER);
    let arrow = first_node(&ast, NodeKind::ArrowFunctionExpression);
    assert!(semantic.class().property_initializer_owner(arrow).is_none());
    assert!(semantic.class().arrow_preserves_lexical_super(arrow));

    let (ast, semantic) = collect_transform_info(&allocator, source, Features::empty());
    let arrow = first_node(&ast, NodeKind::ArrowFunctionExpression);
    assert!(semantic.class().property_initializer_owner(arrow).is_none());
    assert!(!semantic.class().arrow_preserves_lexical_super(arrow));
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
    use crate::SemanticFeatureFlags as Features;
    let allocator = Allocator::new();
    let features = Features::STATIC_BLOCK_LAYOUT | Features::LEXICAL_SUPER;
    let source = "class C extends B { static before = 0; static field = class { static nested = 1; static { inner(); } }; static { super.x; } static { const x = super.y; } static {} }";
    let (ast, semantic) = collect_transform_info(&allocator, source, features);
    let class = first_class(&ast);
    let layout = semantic
        .class()
        .static_block_layout(class.node_id())
        .unwrap();
    assert!(layout.has_static_blocks);
    assert_eq!(layout.trailing_blocks, 2);
    assert!(layout.trailing_lexical_super);
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

    let (ast, semantic) = collect_transform_info(&allocator, source, Features::empty());
    assert!(
        semantic
            .class()
            .static_block_layout(first_class(&ast).node_id())
            .is_none()
    );

    let source = "class C extends B { static { super.x; } static field = 1; static {} }";
    let (ast, semantic) = collect_transform_info(&allocator, source, features);
    let layout = semantic
        .class()
        .static_block_layout(first_class(&ast).node_id())
        .unwrap();
    assert_eq!(layout.trailing_blocks, 0);
    assert!(!layout.trailing_lexical_super);

    let source = "class C { static {} // keep\n}";
    let (ast, semantic) = collect_transform_info(&allocator, source, Features::STATIC_BLOCK_LAYOUT);
    let layout = semantic
        .class()
        .static_block_layout(first_class(&ast).node_id())
        .unwrap();
    assert_eq!(layout.trailing_blocks, 1);
    assert!(layout.first_static_property.is_none());
    assert!(layout.last_static_property.is_none());
    assert!(
        !semantic
            .class()
            .static_block_has_comments(first_static_block(&ast))
    );
}

#[test]
fn transform_information() {
    let allocator = Allocator::new();
    let source = "class C { static {} // keep\n}";
    let options = AnalyzeOptions {
        features: crate::SemanticFeatureFlags::STATIC_BLOCK_COMMENTS,
        ..AnalyzeOptions::default()
    };
    let (ast, semantic) = analyze_with_options(&allocator, source, Options::default(), options);
    assert!(
        semantic
            .class()
            .static_block_has_comments(first_static_block(&ast))
    );
    assert!(semantic.scope_at(first_static_block(&ast)).is_some());

    let (ast, semantic) = analyze_source(&allocator, source, Options::default());
    assert!(
        !semantic
            .class()
            .static_block_has_comments(first_static_block(&ast))
    );
    let scope = semantic.scope_at(first_static_block(&ast)).unwrap();
    assert_eq!(semantic.scope(scope).node, first_static_block(&ast));
    assert_eq!(semantic.scope(scope).kind, ScopeKind::StaticBlock);
}

#[test]
fn generated_bindings() {
    use swc_next_ecma_ast::DUMMY_SP;
    let allocator = Allocator::new();
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    let source = "let original; original;";
    let (mut ast, mut semantic) = analyze_source(&allocator, source, script);
    let original = semantic.binding(ScopeId::ROOT, b"original").unwrap();
    let original_use = semantic.uses(original)[0];
    let name = semantic.generate_name(&mut ast, "_temporary", 2);
    let binding = ast.binding_identifier(DUMMY_SP, name);
    let symbol = semantic.register_binding(
        &ast,
        binding,
        ScopeId::ROOT,
        SymbolFlags::FUNCTION_SCOPED_VAR,
    );
    let reference = ast.identifier_reference(DUMMY_SP, name);
    let reference_id = semantic.register_reference(
        &ast,
        reference,
        ScopeId::ROOT,
        Some(symbol),
        Default::default(),
    );
    assert_eq!(semantic.symbol_of(binding.node_id()), Some(symbol));
    assert_eq!(
        semantic.reference_of(reference.node_id()),
        Some(reference_id)
    );
    assert_eq!(semantic.symbol_of(reference.node_id()), None);
    assert_eq!(semantic.reference_of(binding.node_id()), None);
    assert_eq!(semantic.reference(reference_id).symbol, Some(symbol));
    assert_eq!(semantic.uses(symbol), [reference_id]);
    assert_eq!(semantic.uses(original), [original_use]);
    assert_eq!(semantic.binding(ScopeId::ROOT, b"_temporary"), Some(symbol));

    let another_reference = ast.identifier_reference(DUMMY_SP, name);
    let another_id = semantic.register_reference(
        &ast,
        another_reference,
        ScopeId::ROOT,
        Some(symbol),
        Default::default(),
    );
    assert_eq!(semantic.uses(symbol), [reference_id, another_id]);
    assert_eq!(semantic.uses(original), [original_use]);

    let unbound_name = ast.add_utf8("unbound");
    let unbound = ast.identifier_reference(DUMMY_SP, unbound_name);
    let unbound_id =
        semantic.register_reference(&ast, unbound, ScopeId::ROOT, None, Default::default());
    assert_eq!(semantic.reference_of(unbound.node_id()), Some(unbound_id));
    assert_eq!(semantic.reference(unbound_id).symbol, None);

    semantic.retarget_reference(unbound.node_id(), name, symbol);
    unbound.set_name(&mut ast, name);
    assert_eq!(semantic.reference_of(unbound.node_id()), Some(unbound_id));
    assert_eq!(semantic.reference(unbound_id).symbol, Some(symbol));
    assert_eq!(
        semantic.uses(symbol),
        [reference_id, another_id, unbound_id]
    );
    assert_eq!(semantic.uses(original), [original_use]);
}

#[test]
fn symbols() {
    let allocator = Allocator::new();
    let ts_script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    let source = "interface A {} interface B {} interface A {} interface A {}";
    let (ast, semantic) = analyze_source(&allocator, source, ts_script);
    let a = semantic.binding(ScopeId::ROOT, b"A").unwrap();
    let b = semantic.binding(ScopeId::ROOT, b"B").unwrap();
    let mut a_declarations = semantic.declaration_ids(a);
    assert_eq!(a_declarations.len(), 3);
    let first_a = a_declarations.next().unwrap();
    let last_a = a_declarations.next_back().unwrap();
    assert!(
        ast.span(semantic.declaration(first_a).node).start
            < ast.span(semantic.declaration(last_a).node).start
    );

    let mut declarations = semantic.iter_declarations();
    let first = declarations.next().unwrap();
    let second = declarations.next().unwrap();
    let third = declarations.next().unwrap();
    let fourth = declarations.next().unwrap();
    assert!(declarations.next().is_none());
    assert_eq!(
        [
            first.declaration.symbol,
            second.declaration.symbol,
            third.declaration.symbol,
            fourth.declaration.symbol,
        ],
        [a, b, a, a]
    );
    assert_eq!(semantic.symbol(a).declaration, first.id);
    assert_eq!(
        semantic.declaration_ids(a).collect::<Vec<_>>(),
        [first.id, third.id, fourth.id]
    );
}

#[test]
fn references() {
    let allocator = Allocator::new();
    let js_script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    let ts_module = Options {
        lang: Lang::Ts,
        ..Options::default()
    };

    let source = "let value; value;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(reference.flags.is_read_only());

    let source = "let value; value = 1;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(reference.flags.is_write_only());

    let source = "let value; value++;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(reference.flags.is_read_write());

    let source = "let value; delete value;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(!reference.flags.is_read());
    assert!(!reference.flags.is_write());

    let source = "import type { T } from 'mod'; type U = T;";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let symbol = semantic
        .iter_symbols()
        .find(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"T")
        .unwrap();
    assert!(symbol.symbol.flags.contains(SymbolFlags::TYPE_IMPORT));

    let reference = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"T")
        .unwrap()
        .reference;
    assert_eq!(reference.flags.space, ReferenceSpace::Type);
    assert_eq!(reference.symbol, Some(symbol.id));
    assert!(!reference.flags.is_read());

    let source = "import type { T } from 'mod'; T;";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let reference = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"T")
        .unwrap()
        .reference;
    assert_eq!(reference.flags.space, ReferenceSpace::Value);
    assert!(reference.symbol.is_none());
    assert!(reference.flags.is_read_only());
}

#[test]
fn modules() {
    let allocator = Allocator::new();
    let js_module = Options::default();
    let module_options = AnalyzeOptions {
        build_module_record: true,
        ..AnalyzeOptions::default()
    };

    let source = "export const value = 1;";
    let (_, semantic) = analyze_source(&allocator, source, js_module);
    assert!(semantic.module_record().exports().is_empty());

    let source = "export { value }; const value = 1;";
    let (ast, semantic) = analyze_with_options(&allocator, source, js_module, module_options);
    let body = ast.root_program().body(&ast);
    let statement = body.get_node(&ast, 0).unwrap();
    let declaration = statement.as_export_named_declaration(&ast).unwrap();
    let local_specifier = declaration.specifiers(&ast).get_node(&ast, 0).unwrap();
    let local_node = local_specifier.local(&ast).node_id();

    let local_reference = semantic.reference_of(local_node).unwrap();
    let local_reference_data = semantic.reference(local_reference);
    let local_symbol = local_reference_data.symbol.unwrap();
    assert_eq!(local_reference_data.flags.space, ReferenceSpace::Any);

    let local_export = semantic
        .module_record()
        .exports()
        .iter()
        .find(|entry| entry.node == local_specifier.node_id())
        .unwrap();
    assert_eq!(local_export.origin, ExportOrigin::LocalSpecifier);
    assert_eq!(
        local_export.target,
        ExportTarget::Local {
            symbol: local_symbol,
            reference: Some(local_reference),
        }
    );

    let source = "export { external as alias } from 'dep';";
    let (ast, semantic) = analyze_with_options(&allocator, source, js_module, module_options);
    let body = ast.root_program().body(&ast);
    let statement = body.get_node(&ast, 0).unwrap();
    let declaration = statement.as_export_named_declaration(&ast).unwrap();
    let re_export_specifier = declaration.specifiers(&ast).get_node(&ast, 0).unwrap();
    let re_export_node = re_export_specifier.local(&ast).node_id();
    assert_eq!(semantic.reference_of(re_export_node), None);

    let re_export = semantic
        .module_record()
        .exports()
        .iter()
        .find(|entry| entry.node == re_export_specifier.node_id())
        .unwrap();
    assert_eq!(re_export.origin, ExportOrigin::ReExportSpecifier);
    assert!(matches!(re_export.target, ExportTarget::External { .. }));
}

#[test]
fn dynamic_scopes() {
    let allocator = Allocator::new();
    let js_script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };

    let source = "let value; with ({}) { value; }";
    let (ast, semantic) = analyze_source(&allocator, source, js_script);
    let value_symbol = semantic.binding(ScopeId::ROOT, b"value").unwrap();
    let value = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"value")
        .unwrap()
        .reference;
    assert!(value.flags.is_dynamic());
    assert_eq!(value.symbol, Some(value_symbol));

    let source = "with ({}) { missing; }";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let missing = semantic.iter_references().next().unwrap().reference;
    assert!(missing.flags.is_dynamic());
    assert!(missing.symbol.is_none());

    let source = "with ({}) { let value; value; }";
    let (ast, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"value")
        .unwrap()
        .reference;
    assert!(!reference.flags.is_dynamic());
    assert!(reference.symbol.is_some());

    let source = "with ({}) { var value = 1; }";
    let (ast, semantic) = analyze_source(&allocator, source, js_script);
    let declaration = semantic.iter_declarations().next().unwrap();
    let reference_id = semantic.reference_of(declaration.declaration.node).unwrap();
    let reference = semantic.reference(reference_id);
    let symbol_id = declaration.declaration.symbol;
    assert!(reference.flags.is_write_only());
    assert!(reference.flags.is_dynamic());
    assert_eq!(reference.symbol, Some(symbol_id));
    assert_eq!(semantic.uses(symbol_id), &[reference_id]);
    assert_eq!(
        ast.get_wtf8(semantic.symbol(symbol_id).name).as_bytes(),
        b"value"
    );

    let source = "function f() { eval('direct'); }";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let function = semantic
        .iter_scopes()
        .find(|entry| entry.scope.kind == ScopeKind::Function)
        .unwrap();
    assert!(function.scope.flags.contains_direct_eval());
    assert!(semantic.scope(ScopeId::ROOT).flags.contains_direct_eval());

    let source = "function f() { (0, eval)('indirect'); }";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let function = semantic
        .iter_scopes()
        .find(|entry| entry.scope.kind == ScopeKind::Function)
        .unwrap();
    assert!(!function.scope.flags.contains_direct_eval());
    assert!(!semantic.scope(ScopeId::ROOT).flags.contains_direct_eval());
}
