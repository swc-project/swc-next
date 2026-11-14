use std::{fs, path::Path};

use swc_next_allocator::Allocator;
use swc_next_core::js::{JsTransformApiOptions, transformer::transform_js};
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};

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
            let output = transform_js(&input, options);
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
