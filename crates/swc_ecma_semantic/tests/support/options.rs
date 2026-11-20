use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::Options;
use swc_next_ecma_semantic::SyntaxChecks;

#[derive(Clone, Copy)]
pub(crate) struct TestOptions {
    pub(crate) parser: Options,
    pub(crate) checks: SyntaxChecks,
}

impl TestOptions {
    pub(crate) fn new(lang: Lang, source_type: SourceType) -> Self {
        Self {
            parser: Options {
                lang,
                source_type,
                ..Options::default()
            },
            checks: SyntaxChecks::all(),
        }
    }

    pub(crate) fn syntax(mut self, checks: SyntaxChecks) -> Self {
        self.checks = checks;
        self
    }
}
