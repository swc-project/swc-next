use super::{TypeScriptOptions, analysis::TypeScriptAnalysis};
use crate::{comments::CommentRetargets, uid::UidGenerator};
use swc_next_ecma_ast::{Ast, Program, SourceType, VisitMut};
use swc_next_ecma_semantic::Semantic;

mod erase;
mod lexical_env;
mod modules;
mod names;
mod parameters;
mod statements;
mod visit;

use statements::StatementExpansion;

pub(super) struct TypeScriptLowerer<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    options: &'visit TypeScriptOptions,
    analysis: TypeScriptAnalysis,
    semantic: &'visit Semantic<'ast>,
    comment_retargets: CommentRetargets,
    uid_generator: Option<UidGenerator<'ast>>,
}

impl<'visit, 'ast> TypeScriptLowerer<'visit, 'ast> {
    pub(super) fn new(
        options: &'visit TypeScriptOptions,
        analysis: TypeScriptAnalysis,
        ast: &'visit mut Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
    ) -> Self {
        Self {
            ast,
            options,
            analysis,
            semantic,
            comment_retargets: CommentRetargets::default(),
            uid_generator: None,
        }
    }

    pub(super) fn lower_program(mut self, program: Program) {
        let body = program.body(self.ast());
        if let Some(body) = self.lower_statement_list(body) {
            program.set_body(self.ast(), body);
        }
        if self.ast_ref().source_type() == SourceType::Module
            && !self.has_module_indicator(program.body(self.ast_ref()))
        {
            self.append_esm_marker(program);
        }

        let retargets = std::mem::take(&mut self.comment_retargets);
        retargets.apply(self.ast());
    }

    pub(super) fn ast_ref(&self) -> &Ast<'ast> {
        self.ast
    }
}
