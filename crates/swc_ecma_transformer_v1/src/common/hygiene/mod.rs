use swc_next_ecma_ast::Ast;

use super::{
    rename::{rename, renamer_single},
    resolver::context::Resolver,
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/hygiene/mod.rs#L68>.
pub(crate) fn hygiene(ast: &mut Ast<'_>, resolver: &mut Resolver) {
    let renames = renamer_single(ast, resolver)
        .into_iter()
        .map(|(id, name)| (id, ast.add_utf8(&name)))
        .collect();
    rename(ast, resolver, &renames);
}
