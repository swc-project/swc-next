use swc_next_ecma_ast::Stmt;

use crate::context::BoundIdentifier;
pub(crate) use swc_next_helpers::Helper as HelperKind;

/// Prepared external declarations, allocated on first use and attached once.
#[derive(Default)]
pub(crate) struct Helpers {
    entries: [Option<PreparedHelper>; HelperKind::ALL.len()],
}

pub(crate) struct PreparedHelper {
    pub(crate) bound_identifier: BoundIdentifier,
    pub(crate) statement: Stmt,
}

impl Helpers {
    pub(crate) fn bound_identifier(&self, helper: HelperKind) -> Option<BoundIdentifier> {
        self.entries[helper as usize]
            .as_ref()
            .map(|entry| entry.bound_identifier)
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
    use swc_next_ecma_ast::{AstCtx, Diagnostic, Lang, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, SyntaxChecks, analyze};

    use crate::{
        Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
        proposal::ProposalOptions,
    };

    fn transformed_code(source: &str, source_type: SourceType) -> String {
        let (code, diagnostics) = transform_with_helpers(source, source_type, Lang::Js);
        assert!(diagnostics.is_empty());
        code
    }

    fn transform_with_helpers(
        source: &str,
        source_type: SourceType,
        lang: Lang,
    ) -> (String, Vec<Diagnostic>) {
        let allocator = Allocator::new();
        let parsed = Parser::init(
            &allocator,
            source,
            ParserOptions {
                source_type,
                lang,
                ..ParserOptions::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parsed.diagnostics.is_empty());
        let mut ast = parsed.ast;
        Normalizer::normalize(&mut ast);
        let analyzed = analyze(&mut ast, AnalyzeOptions::default());
        assert!(analyzed.diagnostics.is_empty());
        let original_references: Vec<_> = analyzed
            .semantic
            .semantic()
            .iter_references()
            .map(|entry| (entry.id, entry.reference.node, entry.reference.symbol))
            .collect();
        let transformed = Transformer::transform(
            analyzed.semantic,
            TransformerOptions {
                es2022: ES2022Options {
                    static_blocks: true,
                },
                proposal: ProposalOptions {
                    prevent_extensions: true,
                },
                ..TransformerOptions::default()
            },
        );
        for (id, node, symbol) in original_references {
            assert_eq!(transformed.semantic.reference_of(node), Some(id));
            assert_eq!(transformed.semantic.reference(id).symbol, symbol);
        }
        assert_semantic(&ast, &transformed.semantic);
        // SAFETY: The transformed semantic model refers to this AST.
        let tree = unsafe { ast.ast() };
        for entry in transformed.semantic.iter_references() {
            let name = tree.get_utf8(entry.reference.name);
            if name.starts_with("_set_function_name") || name.starts_with("_to_property_key") {
                let symbol = entry.reference.symbol.expect("helper reference binding");
                assert_eq!(
                    tree.get_wtf8(transformed.semantic.symbol(symbol).name)
                        .as_bytes(),
                    name.as_bytes(),
                );
            }
        }
        (
            swc_next_ecma_codegen::generate(&ast, Default::default()).code,
            transformed.diagnostics,
        )
    }

    fn assert_semantic(ast: &AstCtx<'_>, semantic: &ScopeModel<'_>) {
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
        let expected_semantic = analyzed.semantic.semantic();
        // SAFETY: Both semantic models refer to this AST.
        let ast = unsafe { ast.ast() };
        for entry in expected_semantic.iter_declarations() {
            let expected = entry.declaration;
            // Annex B companion declarations share the lexical binding node.
            if expected_semantic.symbol_of(expected.node) != Some(expected.symbol) {
                continue;
            }
            let symbol = semantic
                .symbol_of(expected.node)
                .expect("registered binding");
            let actual = semantic
                .declarations(symbol)
                .find(|declaration| declaration.node == expected.node)
                .expect("registered declaration");
            assert_eq!(actual.flags, expected.flags);
            assert_eq!(
                semantic.scope(actual.scope).node,
                expected_semantic.scope(expected.scope).node,
            );
            assert_eq!(
                semantic.symbol(symbol).flags,
                expected_semantic.symbol(expected.symbol).flags,
            );
        }
        for entry in expected_semantic.iter_references() {
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
                    .map(|id| expected_semantic.symbol(id).declaration)
                    .map(|id| expected_semantic.declaration(id).node)
            );
            assert_eq!(
                semantic.scope(actual.scope).node,
                expected_semantic.scope(expected.scope).node
            );
            assert_eq!(actual.flags.is_read(), expected.flags.is_read());
            assert_eq!(actual.flags.is_write(), expected.flags.is_write());
            if let Some(symbol) = actual.symbol {
                assert!(semantic.uses(symbol).contains(&id));
            }
        }
        for entry in expected_semantic.iter_scopes() {
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
                    .map(|id| expected_semantic.scope(id).node)
            );
        }
    }

    #[test]
    fn external_helpers() {
        let source = "const obj = { [key()]: class { static { init(); } } };";
        let code = transformed_code(source, SourceType::Module);
        assert!(code.contains("import { _to_property_key } from \"@swc-next/helpers\";"));
        assert!(code.contains("import { _set_function_name } from \"@swc-next/helpers\";"));
        assert!(code.contains("_to_property_key(key())"));

        for source_type in [SourceType::Script, SourceType::CommonJs] {
            let code = transformed_code(source, source_type);
            assert!(code.contains("require(\"@swc-next/helpers\")._to_property_key"));
            assert!(code.contains("require(\"@swc-next/helpers\")._set_function_name"));
            assert!(!code.contains("function _to_property_key("));
        }

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
