mod analyer_and_collector;
mod analyzer;
mod ops;

use std::collections::HashMap;
use swc_next_ecma_ast::Ast;

use crate::common::resolver::context::{Id, Resolver};

pub(crate) use ops::rename;

pub(crate) fn renamer_single(ast: &Ast<'_>, resolver: &Resolver) -> HashMap<Id, String> {
    analyer_and_collector::analyzer_and_collect_unresolved(ast, resolver)
}
