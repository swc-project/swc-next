use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BindingIdentifier, Function, IdentifierReference, Lang, NodeId,
    NodeKindData, SourceType, Visit, VisitWith,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, SyntaxChecks, analyze};

use super::ES2017Options;
use crate::{
    Transformer, TransformerOptions, normalize::Normalizer, typescript::TypeScriptOptions,
};

fn transformed_code(source: &str) -> String {
    transformed_code_with_source_type(source, SourceType::Module)
}

fn transformed_code_with_source_type(source: &str, source_type: SourceType) -> String {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            lang: Lang::Js,
            source_type,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    let mut ast = parsed.ast;
    Normalizer::normalize(ast.root_program(), &mut ast);
    let transform_options = TransformerOptions {
        es2017: ES2017Options {
            async_to_generator: true,
        },
        ..TransformerOptions::default()
    };
    let analyzed = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
            features: transform_options.semantic_features(),
        },
    );
    assert!(
        analyzed.diagnostics.is_empty(),
        "{:#?}",
        analyzed.diagnostics
    );
    let transformed = Transformer::transform(&mut ast, analyzed.semantic, transform_options);
    assert_transform_semantic(&ast, &transformed.semantic);
    let code = swc_next_ecma_codegen::print(&ast, Default::default()).code;

    let output_allocator = Allocator::new();
    let output = Parser::init(
        &output_allocator,
        &code,
        ParserOptions {
            source_type,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(
        output.diagnostics.is_empty(),
        "{code}\n{:#?}",
        output.diagnostics
    );
    let analyzed = analyze(
        &output.ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
            ..AnalyzeOptions::default()
        },
    );
    assert!(
        analyzed.diagnostics.is_empty(),
        "{code}\n{:#?}",
        analyzed.diagnostics
    );

    code
}

fn transformed_typescript_code(source: &str) -> String {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            lang: Lang::Ts,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    let mut ast = parsed.ast;
    Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    assert!(semantic.transform_semantic().is_none());
    let transformed = Transformer::transform(
        &mut ast,
        semantic,
        TransformerOptions {
            es2017: ES2017Options {
                async_to_generator: true,
            },
            typescript: Some(TypeScriptOptions::default()),
            ..TransformerOptions::default()
        },
    );
    assert_transform_semantic(&ast, &transformed.semantic);
    swc_next_ecma_codegen::print(&ast, Default::default()).code
}

fn assert_transform_semantic(ast: &Ast<'_>, semantic: &ScopeModel<'_>) {
    struct Identities<'ast> {
        ast: &'ast Ast<'ast>,
        owners: Vec<NodeId>,
        bindings: Vec<NodeId>,
        references: Vec<NodeId>,
    }
    impl<'ast> Visit<'ast> for Identities<'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_function(&mut self, function: Function) {
            self.owners.push(function.node_id());
            function.visit_children_with(self);
        }

        fn visit_arrow_function_expression(&mut self, arrow: ArrowFunctionExpression) {
            self.owners.push(arrow.node_id());
            arrow.visit_children_with(self);
        }

        fn visit_binding_identifier(&mut self, binding: BindingIdentifier) {
            self.bindings.push(binding.node_id());
        }

        fn visit_identifier_reference(&mut self, reference: IdentifierReference) {
            self.references.push(reference.node_id());
        }
    }
    let mut identities = Identities {
        ast,
        owners: Vec::new(),
        bindings: Vec::new(),
        references: Vec::new(),
    };
    ast.root_program().visit_with(&mut identities);
    for owner in identities.owners {
        assert!(semantic.scope_of(owner).is_some(), "missing function scope");
    }
    for binding in identities.bindings {
        assert!(
            semantic.symbol_of(binding).is_some(),
            "missing binding identity"
        );
    }
    for reference in identities.references {
        assert!(
            semantic.reference_of(reference).is_some(),
            "missing reference identity"
        );
    }
}

#[test]
fn async_functions_and_await() {
    let code = transformed_code("async function load(value) { return await value; }");
    assert!(code.contains("import * as _swc_helpers from \"@swc/helpers\";"));
    assert!(code.contains("function load(_0)"), "{code}");
    assert!(
        code.contains("_swc_helpers._async_to_generator(function* (value)"),
        "{code}"
    );
    assert!(code.contains(").apply(this, arguments)"), "{code}");
    assert!(code.contains("return yield value;"));
    assert!(!code.contains("async function"));

    let code = transformed_code(
        "const expression = async function named() { await task(); }; const arrow = async () => await task();",
    );
    assert!(code.contains("function named()"));
    assert!(
        code.contains("const arrow = () => _swc_helpers._async_to_generator(function* ()"),
        "{code}"
    );
    assert_eq!(code.matches("yield task()").count(), 2);
    assert!(!code.contains("async "));

    let code = transformed_code(
        "const object = { async method(value) { await value; return this; } }; class C { async method(value) { await value; return this; } }",
    );
    assert_eq!(
        code.matches("_async_to_generator(function* (value)")
            .count(),
        2
    );
    assert_eq!(code.matches(").apply(this, arguments)").count(), 2);
    assert!(!code.contains("async method"));

    let code = transformed_code(
        "await top(); async function outer() { await one(); function sync() { return two(); } async function inner() { await three(); } }",
    );
    assert!(code.contains("await top();"));
    assert!(code.contains("function sync()"));
    assert!(code.contains("return two();"));
    assert_eq!(code.matches("yield one()").count(), 1);
    assert_eq!(code.matches("yield three()").count(), 1);
    assert_eq!(code.matches("_async_to_generator(function* ()").count(), 2);

    let code = transformed_typescript_code(
        "async function typed(value: number): Promise<number> { return await value; }",
    );
    assert!(code.contains("function typed(_0)"), "{code}");
    assert!(code.contains("function* (value)"), "{code}");
    assert!(code.contains("return yield value"), "{code}");
}

#[test]
fn async_function_parameters_and_lexical_values() {
    let code = transformed_code(
        "async function read(a, { value } = fail()) { return [arguments[0], this, await value]; }",
    );
    assert!(code.contains("function read(_0)"));
    assert!(code.contains("function* (a, { value } = fail())"), "{code}");
    assert!(code.contains(").apply(this, arguments)"));
    assert!(code.contains("yield value"));

    let code = transformed_code(
        "function outer() { return async (value) => [this, arguments, await value]; }",
    );
    assert!(
        code.contains("(value) => _swc_helpers._async_to_generator(function* ()"),
        "{code}"
    );
    assert!(code.contains(").apply(this, arguments)"));
    assert!(code.contains("yield value"));

    let code = transformed_code("async function strictThis() { 'use strict'; return this; }");
    assert_eq!(code.matches("use strict").count(), 2, "{code}");
    assert!(code.contains(").call(this)"), "{code}");

    let code = transformed_code(
        "async function safe(value = 1, callback = () => this) { return await callback(value); }",
    );
    assert!(code.contains("function safe()"), "{code}");
    assert!(
        code.contains("function* (value = 1, callback = () => this)"),
        "{code}"
    );
    assert!(code.contains(").apply(this, arguments)"), "{code}");

    let code = transformed_code_with_source_type(
        "async function shadow(arguments) { arguments = await arguments; return arguments; }",
        SourceType::Script,
    );
    assert!(code.contains(".apply(this, arguments)"), "{code}");

    let code = transformed_code_with_source_type(
        "async function mutate() { arguments = []; await value; }",
        SourceType::Script,
    );
    assert!(code.contains(".apply(this, arguments)"), "{code}");
}

#[test]
fn generated_semantic_indexes() {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        "async function declared(value) { return await value; } const expression = async function named(value) { return named(await value); }; class Base {} class Derived extends Base { async load(value, key) { super[key] += await value; return [arguments, this, super.result]; } }",
        ParserOptions {
            source_type: SourceType::Module,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    Normalizer::normalize(ast.root_program(), &mut ast);
    let transform_options = TransformerOptions {
        es2017: ES2017Options {
            async_to_generator: true,
        },
        ..TransformerOptions::default()
    };
    let semantic = analyze(
        &ast,
        AnalyzeOptions {
            features: transform_options.semantic_features(),
            ..AnalyzeOptions::default()
        },
    )
    .semantic;
    let original_functions: Vec<_> = semantic
        .transform_semantic()
        .expect("async transform semantic data")
        .async_functions_post_order()
        .iter()
        .map(|info| {
            (
                info.node,
                semantic
                    .scope_of(info.node)
                    .expect("original async function scope"),
            )
        })
        .collect();
    let original_reference = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name) == "value")
        .expect("value reference");
    let original_reference_id = original_reference.id;
    let original_reference_node = original_reference.reference.node;
    let original_symbol = original_reference.reference.symbol;
    let transformed = Transformer::transform(&mut ast, semantic, transform_options);

    struct GeneratedFunction<'ast> {
        ast: &'ast Ast<'ast>,
        owners: Vec<NodeId>,
        bindings: Vec<NodeId>,
        references: Vec<NodeId>,
    }
    impl<'ast> Visit<'ast> for GeneratedFunction<'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_function(&mut self, function: Function) {
            self.owners.push(function.node_id());
            function.visit_children_with(self);
        }

        fn visit_arrow_function_expression(&mut self, arrow: ArrowFunctionExpression) {
            self.owners.push(arrow.node_id());
            arrow.visit_children_with(self);
        }

        fn visit_binding_identifier(&mut self, binding: BindingIdentifier) {
            self.bindings.push(binding.node_id());
        }

        fn visit_identifier_reference(&mut self, reference: IdentifierReference) {
            self.references.push(reference.node_id());
        }
    }
    let mut generated = GeneratedFunction {
        ast: &ast,
        owners: Vec::new(),
        bindings: Vec::new(),
        references: Vec::new(),
    };
    ast.root_program().visit_with(&mut generated);
    for (node, scope) in original_functions {
        let NodeKindData::Function(function) = ast.kind_data(node) else {
            panic!("original async function must remain a function");
        };
        assert!(function.generator(&ast));
        assert_eq!(transformed.semantic.scope_of(node), Some(scope));
    }
    assert!(
        transformed
            .semantic
            .transform_semantic()
            .expect("async transform semantic data")
            .async_functions_post_order()
            .is_empty()
    );
    for owner in generated.owners {
        assert!(
            transformed.semantic.scope_of(owner).is_some(),
            "missing function scope for {owner:?}"
        );
    }
    for binding in generated.bindings {
        assert!(
            transformed.semantic.symbol_of(binding).is_some(),
            "missing binding identity for {binding:?}"
        );
    }
    for reference in generated.references {
        assert!(
            transformed.semantic.reference_of(reference).is_some(),
            "missing reference identity for {reference:?}"
        );
    }
    for entry in transformed.semantic.iter_references() {
        let Some(symbol) = entry.reference.symbol else {
            continue;
        };
        let name = ast.get_utf8(entry.reference.name).as_bytes();
        let mut scope = Some(entry.reference.scope);
        let mut visible = false;
        while let Some(current) = scope {
            if transformed.semantic.binding(current, name) == Some(symbol) {
                visible = true;
                break;
            }
            scope = transformed.semantic.scope(current).parent;
        }
        assert!(visible, "reference {:?} cannot see its symbol", entry.id);
    }
    assert_eq!(
        transformed.semantic.reference_of(original_reference_node),
        Some(original_reference_id)
    );
    assert_eq!(
        transformed.semantic.reference(original_reference_id).symbol,
        original_symbol
    );
}

#[test]
fn async_methods_with_super() {
    let code = transformed_code(
        "class Base { method(value) { return value; } } class Derived extends Base { async method(key, value) { const result = await super.method(value); const current = await super[key]; super.value = current; return result; } }",
    );
    assert!(code.contains("_superprop_get_method"), "{code}");
    assert!(code.contains("_superprop_get"), "{code}");
    assert!(code.contains("_superprop_set_value"), "{code}");
    assert!(code.contains(".call(this, value)"), "{code}");
    assert!(!code.contains("function* () {\n      const result = yield super"));

    let code = transformed_code(
        "class Base { method() {} } class Derived extends Base { method() { return async () => await super.method(); } }",
    );
    assert!(code.contains("_superprop_get_method"), "{code}");
    assert!(
        code.contains("yield _superprop_get_method().call(this)"),
        "{code}"
    );

    let code = transformed_code(
        "class Base { count = 0; } class Derived extends Base { async update(key) { super.count += await step(); const old = super[key]++; return ++super.count || old; } }",
    );
    assert!(code.contains("_superprop_set_count"), "{code}");
    assert!(code.contains("_superprop_update"), "{code}");
    assert!(code.contains("yield step()"), "{code}");
    assert!(!code.contains("function* () {\n      super"), "{code}");

    let code = transformed_code(
        "class Base { count = 0; } class Derived extends Base { update = async (key) => super[key] += await step(); }",
    );
    assert!(code.contains("var _key"), "{code}");
    assert!(code.contains("_superprop_get"), "{code}");
    assert!(code.contains("_superprop_set"), "{code}");

    let code = transformed_code(
        "class Base { count = 0; } class Derived extends Base { field = async (value = 1) => super.count + await value; }",
    );
    assert!(code.contains("(value = 1) =>"), "{code}");
    assert!(code.contains("_superprop_get_count"), "{code}");
}
