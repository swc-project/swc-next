mod analyer_and_collector;
mod analyzer;
mod ops;

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{Ast, SymbolId};

use crate::common::resolver::context::Resolver;

pub(crate) use ops::rename;

pub(crate) fn renamer_single(ast: &Ast<'_>, resolver: &Resolver) -> FxHashMap<SymbolId, String> {
    analyer_and_collector::analyzer_and_collect_unresolved(ast, resolver)
}
