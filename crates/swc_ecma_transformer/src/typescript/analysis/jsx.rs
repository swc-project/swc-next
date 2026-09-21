use super::super::JsxImportOptions;
use crate::jsx_pragma::{PragmaKind, pragmas};
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{Ast, Program, SymbolId};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic};

pub(in crate::typescript) fn dependencies(
    program: Program,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    options: &JsxImportOptions,
) -> FxHashSet<SymbolId> {
    let mut dependencies = FxHashSet::default();
    if semantic.jsx_usages().is_empty() {
        return dependencies;
    }
    let mut automatic = options.automatic;
    let mut pragma = options.pragma.as_deref().unwrap_or("React.createElement");
    let mut fragment = options.pragma_frag.as_deref().unwrap_or("React.Fragment");
    for (kind, value) in pragmas(program, ast) {
        match kind {
            PragmaKind::Jsx => pragma = value,
            PragmaKind::Fragment => fragment = value,
            PragmaKind::Runtime => match value {
                "automatic" => automatic = true,
                "classic" => automatic = false,
                _ => {}
            },
            PragmaKind::ImportSource => {}
        }
    }
    if automatic {
        return dependencies;
    }
    fn root(name: &str) -> &[u8] {
        name.split_once('.')
            .map_or(name, |(root, _)| root)
            .as_bytes()
    }
    let factory = root(pragma);
    let fragment = root(fragment);
    for (scope, usages) in semantic.jsx_usage_groups() {
        if let Some(symbol) = semantic.lookup(scope, factory, ReferenceSpace::Value) {
            dependencies.insert(symbol);
        }
        if usages.iter().any(|usage| usage.is_fragment)
            && let Some(symbol) = semantic.lookup(scope, fragment, ReferenceSpace::Value)
        {
            dependencies.insert(symbol);
        }
    }
    dependencies
}
