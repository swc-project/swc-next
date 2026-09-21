use super::TypeScriptLowerer;
use crate::uid::UidGenerator;

impl<'ast> TypeScriptLowerer<'_, 'ast> {
    pub(super) fn fresh_uid(&mut self, base: &'ast str) -> swc_next_ecma_ast::AstUtf8 {
        let generator = self
            .uid_generator
            .get_or_insert_with(|| UidGenerator::new(self.ast, self.semantic));
        generator.fresh(base, self.ast)
    }
}
