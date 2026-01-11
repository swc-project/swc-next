use std::{env, fs::read_to_string, path::PathBuf};

use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax, TsSyntax};
use swc_experimental_ecma_semantic::resolver::resolver;

pub fn main() {
    let path = PathBuf::from(
        env::args()
            .nth(1)
            .unwrap_or("../files/typescript.js".to_owned()),
    );
    let source = read_to_string(&path).unwrap();
    let input = StringSource::new(&source);
    let syntax = match path.extension().and_then(|s| s.to_str()) {
        Some("ts") => Syntax::Typescript(TsSyntax::default()),
        Some("tsx") => Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        Some("js") => Syntax::Es(EsSyntax::default()),
        Some("jsx") => Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        }),
        _ => Syntax::Es(EsSyntax::default()),
    };

    let parser = Parser::new(syntax, input, None);
    let ret = parser.parse_module().unwrap();

    let _semantic = resolver(ret.root, &ret.ast);
}
