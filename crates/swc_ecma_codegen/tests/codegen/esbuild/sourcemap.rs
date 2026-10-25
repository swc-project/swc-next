// Based on esbuild's scripts/verify-source-map.js, limited to parser/codegen behavior.
// https://github.com/evanw/esbuild/blob/6ff1d8b0/scripts/verify-source-map.js
// Decode with an independent consumer and check positions computed from the source and output.
use sourcemap::{SourceMap as DecodedSourceMap, Token};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Lang;
use swc_next_ecma_codegen::{Options as PrintOptions, SourceMap, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParseOptions, Parser};

const SOURCE_PATH: &str = "file name with spaces.js";

struct Config {
    print: PrintOptions,
    newline: &'static str,
}

fn configurations() -> impl Iterator<Item = Config> {
    // Match the upstream LF/CRLF and minify matrix.
    [false, true].into_iter().flat_map(|minify| {
        ["\n", "\r\n"].into_iter().map(move |newline| Config {
            print: PrintOptions {
                minify,
                source_map_path: Some(SOURCE_PATH.into()),
                ..PrintOptions::default()
            },
            newline,
        })
    })
}

struct Output {
    source: String,
    code: String,
    map: DecodedSourceMap,
}

fn decode_map(map: &SourceMap<'_>) -> DecodedSourceMap {
    let text = |bytes| std::str::from_utf8(bytes).unwrap();
    let json = serde_json::json!({
        "version": map.version(),
        "file": map.file().map(text),
        "sourceRoot": map.source_root().map(text),
        "sources": map.sources().iter().map(|source| text(source)).collect::<Vec<_>>(),
        "sourcesContent": map.sources_content().map(|sources| sources.iter().map(|source| source.map(text)).collect::<Vec<_>>()),
        "names": map.names().iter().map(|name| text(name)).collect::<Vec<_>>(),
        "mappings": text(map.mappings()),
    });
    DecodedSourceMap::from_slice(&serde_json::to_vec(&json).unwrap()).unwrap()
}

#[track_caller]
fn generate(source: &str, lang: Lang, config: &Config) -> Output {
    let source = source.replace('\n', config.newline);
    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        lang,
        comments: CommentMode::Flat,
        ..ParseOptions::default()
    };
    let parsed = Parser::init(&allocator, &source, parse_options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let printed = print(&parsed.ast, config.print.clone());
    let map = decode_map(printed.map.as_ref().unwrap());
    assert_eq!(map.get_source_count(), 1);
    assert_eq!(map.get_source(0), Some(SOURCE_PATH));
    assert_eq!(map.get_source_contents(0), Some(source.as_str()));
    let code = printed.code;
    let output = Output { source, code, map };
    output.expect_coverage();
    output
}

fn position(text: &str, offset: usize) -> (u32, u32) {
    let mut line = 0;
    let mut column = 0;
    let mut chars = text[..offset].chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                line += 1;
                column = 0;
            }
            '\n' | '\u{2028}' | '\u{2029}' => {
                line += 1;
                column = 0;
            }
            _ => column += ch.len_utf16() as u32,
        }
    }
    (line, column)
}

impl Output {
    #[track_caller]
    fn expect_mapping(&self, original: &str, generated: &str) -> Token<'_> {
        let original = position(&self.source, self.source.find(original).unwrap());
        let generated = position(&self.code, self.code.find(generated).unwrap());
        let token = self.map.lookup_token(generated.0, generated.1).unwrap();
        assert_eq!(token.get_src(), original);
        assert_eq!(token.get_source(), Some(SOURCE_PATH));
        assert!(self.map.tokens().any(|token| {
            token.get_src_id() == 0 && token.get_src() == original && token.get_dst() == generated
        }));
        token
    }

    #[track_caller]
    fn expect_string(&self, value: &str) {
        let quoted = format!("\"{value}\"");
        let original = if self.source.contains(&quoted) {
            quoted.clone()
        } else {
            format!("'{value}'")
        };
        self.expect_mapping(&original, &quoted);
    }

    #[track_caller]
    fn expect_name(&self, name: &str) {
        let token = self.expect_mapping(name, name);
        // Like esbuild, accept omitted names when the printed spelling is unchanged.
        assert_eq!(token.get_name().unwrap_or(name), name);
    }

    #[track_caller]
    fn expect_coverage(&self) {
        for (line, text) in self.code.lines().enumerate() {
            if text.starts_with("#!") || text.starts_with("//") {
                continue;
            }
            for column in 0..=text.encode_utf16().count() {
                let token = self.map.lookup_token(line as u32, column as u32).unwrap();
                assert_eq!(token.get_dst_line(), line as u32);
                assert!(token.has_source());
            }
        }
    }
}

#[test]
fn test_mappings() {
    let js = Lang::Js;
    for config in configurations() {
        // testCaseFileNameWithSpaces.
        let output = generate("console . log ( \"test\" )", js, &config);
        output.expect_string("test");

        // testCaseES6 and testCaseCommonJS.
        let output = generate("function a0() { a1(\"a0\") }", js, &config);
        output.expect_string("a0");

        let output = generate("export function b0() { b1(\"b0\") }", js, &config);
        output.expect_string("b0");

        let output = generate("exports.c0 = function() { c1(\"c0\") }", js, &config);
        output.expect_string("c0");

        let output = generate("function c2() { throw new Error(\"c2\") }", js, &config);
        output.expect_string("c2");
    }
}

#[test]
fn test_imports() {
    let js = Lang::Js;
    for config in configurations() {
        // testCaseES6.
        let output = generate("import { b0 } from './b-dir/b'", js, &config);
        output.expect_string("./b-dir/b");
        output.expect_name("b0");

        // testCaseCommonJS.
        let output = generate("const { b0 } = require('./b-dir/b')", js, &config);
        output.expect_string("./b-dir/b");

        // testCaseDynamicImport.
        let output = generate("import(\"./ext/a.js\").then(then)", js, &config);
        output.expect_string("./ext/a.js");

        // testCaseEmptyFile.
        let output = generate("export { default as fn } from './test'", js, &config);
        output.expect_string("./test");
    }
}

#[test]
fn test_unicode() {
    // testCaseUnicode.
    let js = Lang::Js;
    for config in configurations() {
        let output = generate("console.log('🍕🍕🍕', \"a\")", js, &config);
        output.expect_string("a");

        let output = generate("console.log({𐀀: \"b\"})", js, &config);
        output.expect_string("b");
    }
}

#[test]
fn test_names() {
    // testCaseNames, with the /**/ position markers removed.
    let js = Lang::Js;
    for config in configurations() {
        let output = generate("var foo = foo || 0", js, &config);
        output.expect_name("foo");

        let output = generate("function fn(bar) {}", js, &config);
        output.expect_name("fn");
        output.expect_name("bar");

        let output = generate("class cls {}", js, &config);
        output.expect_name("cls");

        let output = generate("export let foo = bar => bar()", js, &config);
        output.expect_name("foo");
        output.expect_name("bar");

        let output = generate("var { mangle_: bar } = foo", js, &config);
        output.expect_name("mangle_");
        output.expect_name("bar");

        let output = generate("var { 'mangle_': bar } = foo", js, &config);
        output.expect_string("mangle_");
        output.expect_name("bar");

        let output = generate("foo.mangle_ = 1", js, &config);
        output.expect_name("mangle_");

        let output = generate("foo['mangle_']", js, &config);
        output.expect_string("mangle_");

        let output = generate("foo = { mangle_: 0 }", js, &config);
        output.expect_name("mangle_");

        let output = generate("foo = { 'mangle_': 0 }", js, &config);
        output.expect_string("mangle_");

        let output = generate("foo = class { mangle_ = 0 }", js, &config);
        output.expect_name("mangle_");

        let output = generate("foo = class { 'mangle_' = 0 }", js, &config);
        output.expect_string("mangle_");

        let output = generate("foo = 'mangle_' in bar", js, &config);
        output.expect_string("mangle_");
    }
}

#[test]
fn test_jsx() {
    // JSX expressions extracted from testCaseJSXRuntime.
    let jsx = Lang::Jsx;
    for config in configurations() {
        let output = generate("<Fragment id=\"A0\"><>a0</></Fragment>", jsx, &config);
        output.expect_string("A0");

        let output = generate("<div {...jsx} data-testid=\"A1\">a1</div>", jsx, &config);
        output.expect_string("A1");
    }
}
