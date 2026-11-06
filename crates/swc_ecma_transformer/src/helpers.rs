use std::collections::BTreeSet;

use swc_next_ecma_ast::{Ast, AstUtf8, Stmt};

use crate::context::BoundIdentifier;
pub(crate) use swc_next_helpers::Helper as HelperKind;

/// How runtime helpers are provided to transformed code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HelperMode {
    /// Import each helper from `@swc-next/helpers` (or use `require` in scripts).
    #[default]
    External,
    /// Parse the packaged helper sources and inject their declarations and dependencies.
    Inline,
}

/// Prepared helper declarations, allocated on first use and attached once.
pub(crate) struct Helpers {
    pub(crate) mode: HelperMode,
    require: Option<AstUtf8>,
    entries: [Option<PreparedHelper>; HelperKind::ALL.len()],
    pub(crate) preparing: [bool; HelperKind::ALL.len()],
    pub(crate) globals: BTreeSet<String>,
}

pub(crate) struct PreparedHelper {
    pub(crate) bound_identifier: BoundIdentifier,
    pub(crate) statement: Stmt,
}

impl Helpers {
    pub(crate) fn new(mode: HelperMode) -> Self {
        Self {
            mode,
            require: None,
            entries: Default::default(),
            preparing: Default::default(),
            globals: BTreeSet::new(),
        }
    }

    pub(crate) fn bound_identifier(&self, helper: HelperKind) -> Option<BoundIdentifier> {
        self.entries[helper as usize]
            .as_ref()
            .map(|entry| entry.bound_identifier)
    }

    pub(crate) fn require(&mut self, ast: &mut Ast) -> AstUtf8 {
        *self.require.get_or_insert_with(|| ast.add_utf8("require"))
    }

    pub(crate) fn insert(&mut self, helper: HelperKind, prepared: PreparedHelper) {
        let entry = &mut self.entries[helper as usize];
        debug_assert!(entry.is_none());
        *entry = Some(prepared);
    }

    pub(crate) fn take_statements(&mut self) -> impl Iterator<Item = Stmt> + '_ {
        self.entries
            .iter_mut()
            .filter_map(|entry| entry.take().map(|helper| helper.statement))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.iter().all(Option::is_none)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, SyntaxChecks, analyze};

    use super::HelperMode;

    use crate::{
        Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
        proposal::ProposalOptions,
    };

    fn transformed_code(source: &str, source_type: SourceType) -> String {
        transformed_code_with_helpers(source, source_type, HelperMode::External)
    }

    fn inline(source: &str, source_type: SourceType) -> String {
        transformed_code_with_helpers(source, source_type, HelperMode::Inline)
    }

    fn transformed_code_with_helpers(
        source: &str,
        source_type: SourceType,
        helpers: HelperMode,
    ) -> String {
        let allocator = Allocator::new();
        let parsed = Parser::init(
            &allocator,
            source,
            ParserOptions {
                source_type,
                ..ParserOptions::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parsed.diagnostics.is_empty());
        let mut ast = parsed.ast;
        Normalizer::normalize(ast.root_program(), &mut ast);
        let analyzed = analyze(&ast, AnalyzeOptions::default());
        assert!(analyzed.diagnostics.is_empty());
        let original_references: Vec<_> = analyzed
            .semantic
            .iter_references()
            .map(|entry| (entry.id, entry.reference.node, entry.reference.symbol))
            .collect();
        let transformed = Transformer::transform(
            &mut ast,
            analyzed.semantic,
            TransformerOptions {
                helpers,
                es2022: ES2022Options {
                    static_blocks: true,
                },
                proposal: ProposalOptions {
                    prevent_extensions: true,
                },
                ..TransformerOptions::default()
            },
        );
        assert!(transformed.diagnostics.is_empty());
        for (id, node, symbol) in original_references {
            assert_eq!(transformed.semantic.reference_of(node), Some(id));
            assert_eq!(transformed.semantic.reference(id).symbol, symbol);
        }
        assert_semantic(&ast, &transformed.semantic);
        for entry in transformed.semantic.iter_references() {
            let name = ast.get_utf8(entry.reference.name);
            if name.starts_with("_set_function_name") || name.starts_with("_to_property_key") {
                let symbol = entry.reference.symbol.expect("helper reference binding");
                assert_eq!(
                    ast.get_wtf8(transformed.semantic.symbol(symbol).name)
                        .as_bytes(),
                    name.as_bytes(),
                );
            }
        }
        swc_next_ecma_codegen::print(&ast, Default::default()).code
    }

    fn assert_semantic(ast: &Ast<'_>, semantic: &ScopeModel<'_>) {
        let analyzed = analyze(
            ast,
            AnalyzeOptions {
                check_syntax: SyntaxChecks::all(),
            },
        );
        assert!(
            analyzed.diagnostics.is_empty(),
            "{:?}",
            analyzed.diagnostics
        );
        for entry in analyzed.semantic.iter_references() {
            let expected = entry.reference;
            let id = semantic
                .reference_of(expected.node)
                .expect("registered reference");
            let actual = semantic.reference(id);
            assert_eq!(ast.get_utf8(actual.name), ast.get_utf8(expected.name));
            assert_eq!(
                actual
                    .symbol
                    .map(|id| semantic.symbol(id).declaration)
                    .map(|id| semantic.declaration(id).node),
                expected
                    .symbol
                    .map(|id| analyzed.semantic.symbol(id).declaration)
                    .map(|id| analyzed.semantic.declaration(id).node)
            );
            assert_eq!(
                semantic.scope(actual.scope).node,
                analyzed.semantic.scope(expected.scope).node
            );
            assert_eq!(actual.flags.is_read(), expected.flags.is_read());
            assert_eq!(actual.flags.is_write(), expected.flags.is_write());
        }
        for entry in analyzed.semantic.iter_scopes() {
            if entry.scope.parent.is_none() {
                continue;
            }
            let id = semantic
                .scope_of(entry.scope.node)
                .expect("registered scope");
            let actual = semantic.scope(id);
            assert_eq!(actual.kind, entry.scope.kind);
            assert_eq!(actual.flags.strict, entry.scope.flags.strict);
            assert_eq!(
                actual.flags.arguments_barrier,
                entry.scope.flags.arguments_barrier
            );
            assert_eq!(
                actual.parent.map(|id| semantic.scope(id).node),
                entry
                    .scope
                    .parent
                    .map(|id| analyzed.semantic.scope(id).node)
            );
        }
    }

    #[test]
    fn inline_helpers() {
        for source_type in [SourceType::Module, SourceType::Script, SourceType::CommonJs] {
            let code = inline(
                "const obj = { [key()]: class { static { init(); } } };",
                source_type,
            );
            assert!(!code.contains("@swc-next/helpers"));
            assert!(code.contains("function _set_function_name("));
            assert!(code.contains("function _to_property_key("));
            assert!(code.contains("function _to_primitive("));
            assert!(code.contains("function _type_of("));
            assert!(code.contains("_to_property_key(key())"));

            let code = inline("const C = class { static { init(); } };", source_type);
            assert!(code.contains("function _set_function_name("));
            assert!(!code.contains("function _to_property_key("));

            let code = inline("class C {}", source_type);
            assert!(!code.contains("function _"));

            let code = inline(
                "const obj = { [first()]: class { static { init(); } }, [second()]: class { static { init(); } } };",
                source_type,
            );
            assert_eq!(code.matches("function _type_of(").count(), 1);
            assert_eq!(code.matches("function _set_function_name(").count(), 1);
        }
    }

    #[test]
    fn inline_helper_bindings() {
        let code = inline(
            "function run(_to_primitive, _type_of, _to_property_key, _set_function_name) { return { [key()]: class { static { init(); } } }; }",
            SourceType::Module,
        );
        assert!(code.contains("function _to_primitive2("));
        assert!(code.contains("function _type_of2("));
        assert!(code.contains("_to_primitive2(arg, \"string\")"));

        let code = inline(
            "const Object = other; const C = class { static { use(Object); } };",
            SourceType::Module,
        );
        assert!(code.contains("const _Object = other;"));
        assert!(code.contains("Object.defineProperty"));
        assert!(code.contains("use(_Object)"));

        let code = inline(
            "const Symbol = other; const obj = { [key()]: class { static { use(Symbol); } } };",
            SourceType::Script,
        );
        assert!(code.contains("const _Symbol = other;"));
        assert!(code.contains("input[Symbol.toPrimitive]"));

        let code = inline(
            "export const Object = other; export const C = class { static { use({ Object }); } };",
            SourceType::Module,
        );
        assert!(code.contains("_Object as Object"));
        assert!(code.contains("Object: _Object"));

        let code = inline(
            "const { Object = other } = source; export { Object }; const C = class { static { use(Object); } };",
            SourceType::Module,
        );
        assert!(code.contains("Object: _Object = other"));
        assert!(code.contains("_Object as Object"));

        let code = inline(
            "const Object = () => 1; const C = class { static { use(Object.name); } };",
            SourceType::Module,
        );
        assert!(code.contains("_set_function_name(() => 1, \"Object\")"));

        let code = inline(
            "function Object() {} const C = class { static { use(Object.name); } };",
            SourceType::Module,
        );
        assert!(code.contains("_set_function_name(_Object, \"Object\");"));

        let code = inline(
            "class Object { static value = this.name; } const C = class { static { use(Object.value); } };",
            SourceType::Module,
        );
        assert!(code.contains("_set_class_name(this, \"Object\")"));

        let code = inline(
            "const Object = class { static value = this.name; }; const C = class { static { use(Object.value); } };",
            SourceType::Module,
        );
        assert!(code.contains("_set_class_name(this, \"Object\")"));
        assert!(!code.contains("_set_function_name(class"));
    }

    #[test]
    fn imported_ast() {
        let allocator = Allocator::new();
        let mut ast = Parser::init(
            &allocator,
            "// user\noriginal;",
            ParserOptions {
                source_type: SourceType::Script,
                ..ParserOptions::default()
            },
            NoTokenParserConfig,
        )
        .parse()
        .ast;
        let root = ast.root_program();
        let count = ast.node_count();
        let imported = {
            let source_allocator = Allocator::new();
            let source = Parser::init(
                &source_allocator,
                "function helper(value = '\\uD800') { return [value, null, true, 0xFF]; }",
                ParserOptions::default(),
                NoTokenParserConfig,
            )
            .parse()
            .ast;
            let mapping = ast.import_synthesized(&source);
            for index in 0..source.node_count() {
                let node = swc_next_ecma_ast::NodeId::from_usize_unchecked(index);
                assert_eq!(ast.node_kind(mapping.node_id(node)), source.node_kind(node));
                assert_eq!(ast.span(mapping.node_id(node)), swc_next_ecma_ast::DUMMY_SP);
            }
            assert_eq!(ast.node_count(), count + source.node_count());
            mapping.node(source.root_program())
        };
        assert_eq!(ast.root_program(), root);
        assert_eq!(ast.source_type(), SourceType::Script);
        let statements: Vec<_> = root
            .body(&ast)
            .iter()
            .chain(imported.body(&ast).iter())
            .map(|slot| ast.get_node_in_sub_range(slot))
            .collect();
        let body = ast.add_sub_range(statements);
        root.set_body(&mut ast, body);
        let code = swc_next_ecma_codegen::print(&ast, Default::default()).code;
        assert!(code.contains("original;"));
        assert!(code.contains("\\ud800") || code.contains("\\uD800"));
        assert!(code.contains("0xFF"));
    }

    #[test]
    fn external_helpers() {
        let source = "const obj = { [key()]: class { static { init(); } } };";
        let code = transformed_code(source, SourceType::Module);
        assert!(code.contains("import { _to_property_key } from \"@swc-next/helpers\";"));
        assert!(code.contains("import { _set_function_name } from \"@swc-next/helpers\";"));
        assert!(code.contains("_to_property_key(key())"));

        let code = transformed_code(source, SourceType::Script);
        assert!(
            code.contains(
                "var _to_property_key = require(\"@swc-next/helpers\")._to_property_key;"
            )
        );
        assert!(code.contains(
            "var _set_function_name = require(\"@swc-next/helpers\")._set_function_name;"
        ));

        let code = transformed_code(
            "const C = class { static { init(); } };",
            SourceType::Module,
        );
        assert!(code.contains("import { _set_function_name } from \"@swc-next/helpers\";"));
        assert!(!code.contains("_to_property_key"));

        let code = transformed_code("class C {}", SourceType::Module);
        assert!(!code.contains("@swc-next/helpers"));
    }

    #[test]
    fn helper_bindings() {
        let code = transformed_code(
            "const obj = { [first()]: class { static { init(); } }, [second()]: class { static { init(); } } };",
            SourceType::Module,
        );
        assert_eq!(code.matches("import { _to_property_key }").count(), 1);
        assert_eq!(code.matches("import { _set_function_name }").count(), 1);

        let code = transformed_code(
            "function run(_to_property_key, _set_function_name) { return { [key()]: class { static { init(); } } }; }",
            SourceType::Module,
        );
        assert!(code.contains("import { _to_property_key as _to_property_key2 }"));
        assert!(code.contains("import { _set_function_name as _set_function_name2 }"));
        assert!(code.contains("_to_property_key2(key())"));
    }
}
