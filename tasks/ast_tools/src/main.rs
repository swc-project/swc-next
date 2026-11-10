use crate::{
    derive::ast_clone_in::ast_clone_in,
    generator::{
        ast_builder::ast_builder, ast_compare::ast_compare, ast_node::ast_node,
        ast_payload::ast_payload, ast_property::ast_property, ast_visitor::ast_visitor,
        estree_decoder::estree_decoder, estree_encoder::estree_encoder, estree_types::estree_types,
    },
    parse::parse_files,
};

pub(crate) mod derive;
pub(crate) mod estree;
pub(crate) mod generator;
pub(crate) mod output;
pub(crate) mod parse;
pub(crate) mod schema;
pub(crate) mod util;

const SOURCE_PATHS: &[&str] = &[
    "crates/swc_ecma_ast/src/ast/mod.rs",
    "crates/swc_ecma_ast/src/ast/js.rs",
    "crates/swc_ecma_ast/src/ast/ts.rs",
    "crates/swc_ecma_ast/src/ast/jsx.rs",
];

const AST_CRATE_PATH: &str = "swc_ecma_ast";

fn main() -> std::io::Result<()> {
    let schema = parse_files(SOURCE_PATHS);

    estree_types(&schema).write_to_file()?;

    let ast_builder = ast_builder(&schema);
    ast_builder.write_to_file().unwrap();

    let ast_payload = ast_payload(&schema);
    ast_payload.write_to_file().unwrap();

    let ast_property_ret = ast_property(&schema);
    ast_property_ret.write_to_file()?;

    let ast_node = ast_node(&schema);
    ast_node.write_to_file()?;

    let ast_clone_in = ast_clone_in(&schema);
    ast_clone_in.write_to_file()?;

    let ast_visitor = ast_visitor(&schema);
    ast_visitor.write_to_file()?;

    ast_compare(&schema).write_to_file()?;

    for output in estree_decoder(&schema) {
        output.write_to_file()?;
    }

    for output in estree_encoder(&schema) {
        output.write_to_file()?;
    }
    for output in generator::estree_visitor_keys::estree_visitor_keys(&schema) {
        output.write_to_file()?;
    }
    Ok(())
}
