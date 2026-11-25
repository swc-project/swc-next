//! TypeScript-specific AST adapters.
pub(super) use crate::common::build::{
    alias_ident_for, assign, binding_shorthand, call, clone_expr, computed, export, find_pat_ids,
    function, member, target, unwrap, variable,
};
use swc_next_ecma_ast::*;

/// Next's dotted namespace names are one qualified node; SWC represents each
/// segment as a nested declaration.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1265>.
pub(super) fn namespace_name_nodes(ast: &Ast<'_>, name: TsModuleName) -> Vec<NodeId> {
    fn qualified(ast: &Ast<'_>, name: TsQualifiedName, names: &mut Vec<NodeId>) {
        match ast.ts_qualified_name_left_data(name.left(ast)) {
            TsQualifiedNameLeftData::BindingIdentifier(id) => names.push(id.node_id()),
            TsQualifiedNameLeftData::IdentifierReference(id) => names.push(id.node_id()),
            TsQualifiedNameLeftData::IdentifierName(id) => names.push(id.node_id()),
            TsQualifiedNameLeftData::TsQualifiedName(name) => qualified(ast, name, names),
            TsQualifiedNameLeftData::ThisExpression(_) => unreachable!(),
        }
        let right = name.right(ast);
        names.push(right.node_id());
    }
    let mut names = Vec::new();
    match ast.ts_module_name_data(name) {
        TsModuleNameData::BindingIdentifier(id) => names.push(id.node_id()),
        TsModuleNameData::TsQualifiedName(name) => qualified(ast, name, &mut names),
        TsModuleNameData::StringLiteral(_) => {}
    }
    names
}
