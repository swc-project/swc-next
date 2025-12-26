use crate::{
    derive::{ast_clone_in::ast_clone_in, ast_node_id::ast_node_id},
    generator::{
        ast_builder::ast_builder, ast_extra_compact::ast_extra_compact, ast_property::ast_property,
        ast_visitor::ast_visitor,
    },
    parse::parse_files,
};

pub(crate) mod derive;
pub(crate) mod generator;
pub(crate) mod output;
pub(crate) mod parse;
pub(crate) mod schema;
pub(crate) mod util;

const SOURCE_PATHS: &[&str] = &[
    "crates/swc_ecma_ast/src/ast/module.rs",
    "crates/swc_ecma_ast/src/ast/module_decl.rs",
    "crates/swc_ecma_ast/src/ast/stmt.rs",
    "crates/swc_ecma_ast/src/ast/decl.rs",
    "crates/swc_ecma_ast/src/ast/expr.rs",
    "crates/swc_ecma_ast/src/ast/function.rs",
    "crates/swc_ecma_ast/src/ast/class.rs",
    "crates/swc_ecma_ast/src/ast/prop.rs",
    "crates/swc_ecma_ast/src/ast/pat.rs",
    "crates/swc_ecma_ast/src/ast/ident.rs",
    "crates/swc_ecma_ast/src/ast/lit.rs",
    "crates/swc_ecma_ast/src/ast/jsx.rs",
    "crates/swc_ecma_ast/src/ast/operator.rs",
    // "crates/swc_ecma_ast/src/ast/typescript.rs",
];

const AST_CRATE_PATH: &str = "swc_ecma_ast";

fn main() {
    let schema = parse_files(SOURCE_PATHS);

    let ast_builder_ret = ast_builder(&schema);
    ast_builder_ret.write_to_file().unwrap();

    let ast_property_ret = ast_property(&schema);
    ast_property_ret.write_to_file().unwrap();

    let ast_node_id = ast_node_id(&schema);
    ast_node_id.write_to_file().unwrap();

    let ast_clone_in = ast_clone_in(&schema);
    ast_clone_in.write_to_file().unwrap();

    let ast_visitor = ast_visitor(&schema);
    ast_visitor.write_to_file().unwrap();

    let ast_extra_data_compact = ast_extra_compact(&schema);
    ast_extra_data_compact.write_to_file().unwrap();
}
