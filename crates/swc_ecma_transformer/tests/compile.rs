use std::{fs, path::Path};

use swc_next_allocator::Allocator;
use swc_next_core::js::{JsTransformApiOptions, transformer::transform_js};
use swc_next_ecma_ast::{
    Ast, GetSpan, Lang, SetSpan, SourceType, Span, TsEnumMember, VariableDeclarator, VisitMut,
    VisitMutWith,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions, normalize::Normalizer, typescript::TypeScriptOptions,
};

fn options(mut directory: &Path, root: &Path) -> JsTransformApiOptions {
    loop {
        let options = directory.join("options.json");
        if options.exists() {
            let source = fs::read_to_string(options).unwrap();
            return serde_json::from_str(&source).unwrap();
        }
        assert_ne!(directory, root);
        directory = directory.parent().unwrap();
    }
}

#[test]
fn compile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/compile");
    insta::glob!(
        &root,
        "**/input.{js,jsx,ts,tsx,mjs,cjs,mts,cts,d.ts}",
        |path| {
            let id = path.display();
            let input = fs::read_to_string(path).unwrap();
            let directory = path.parent().unwrap();
            let options = options(directory, &root);
            let parser = options.parser.unwrap_or_default();
            let filename = options.filename.as_deref();
            let lang = parser.lang.map(Into::into);
            let lang = lang.or(filename.map(Lang::from_path));
            let source_type = parser.source_type.map(Into::into);
            let source_type = source_type.or(filename.map(SourceType::from_path));
            let parser = ParserOptions {
                lang: lang.unwrap_or_default(),
                source_type: source_type.unwrap_or_default(),
                ..parser.into()
            };
            let output = transform_js::<true>(&input, options);
            let allocator = Allocator::new();
            let reparsed =
                Parser::init(&allocator, &output.code, parser, NoTokenParserConfig).parse();
            assert!(reparsed.diagnostics.is_empty(), "{id}");
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_path(directory);
            settings.set_prepend_module_to_snapshot(false);
            settings.set_omit_expression(true);
            settings.remove_snapshot_suffix();
            settings.remove_input_file();
            settings.bind(|| {
                insta::assert_snapshot!("output.js", output.code);
                if output.diagnostics.is_empty() {
                    assert!(!directory.join("diagnostics.snap").exists(), "{id}");
                } else {
                    insta::assert_debug_snapshot!("diagnostics", output.diagnostics);
                }
            });
        }
    );
}

#[derive(Clone, Copy)]
enum InitializationSpans {
    Collapsed,
    Reversed,
}

struct RemapInitializations<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    spans: InitializationSpans,
}

impl RemapInitializations<'_, '_> {
    fn remap(&mut self, node: impl GetSpan + SetSpan) {
        let span = match self.spans {
            InitializationSpans::Collapsed => Span::new(0, 0),
            InitializationSpans::Reversed => {
                let span = node.span(self.ast);
                let end = self.ast.source().len() as u32;
                Span::new(end - span.end, end - span.start)
            }
        };
        node.set_span(self.ast, span);
    }
}

impl<'ast> VisitMut<'ast> for RemapInitializations<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_ts_enum_member(&mut self, member: TsEnumMember) -> Option<TsEnumMember> {
        self.remap(member);
        member.visit_mut_children_with(self)
    }

    fn visit_mut_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
    ) -> Option<VariableDeclarator> {
        self.remap(declarator);
        declarator.visit_mut_children_with(self)
    }
}

fn transform_with_initialization_spans(source: &str, spans: InitializationSpans) -> String {
    let allocator = Allocator::new();
    let options = ParserOptions {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..ParserOptions::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    Normalizer::normalize_runtime(&mut ast);
    let program = ast.root_program();
    // SAFETY: This visitor uses only handles from this AST and changes source spans.
    let tree = unsafe { ast.ast_mut() };
    program.visit_mut_with(&mut RemapInitializations { ast: tree, spans });
    let analyzed = analyze(&mut ast, AnalyzeOptions::default());
    assert!(analyzed.diagnostics.is_empty());
    let options = TransformerOptions {
        typescript: Some(TypeScriptOptions::default()),
        ..TransformerOptions::default()
    };
    Transformer::transform(analyzed.semantic, options);
    swc_next_ecma_codegen::generate(&ast, Default::default()).code
}

// Source fixtures cannot express synthetic AST spans.
#[test]
fn enum_initialization_order() {
    let input = "const value = 'x'; enum E { A = value, B = A }";
    let expected = r#"const value = "x";
var E;
(function(E) {
  E["A"] = "x";
  E["B"] = "x";
})(E || (E = {}));
"#;
    for spans in [
        InitializationSpans::Collapsed,
        InitializationSpans::Reversed,
    ] {
        let output = transform_with_initialization_spans(input, spans);
        assert_eq!(output, expected);
    }
}
