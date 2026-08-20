use crate::{
    derive::ast_clone_in::ast_clone_in,
    generator::{
        ast_builder::ast_builder, ast_extra_compact::ast_extra_compact, ast_node::ast_node,
        ast_property::ast_property, ast_visitor::ast_visitor, estree_decoder::estree_decoder,
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
    "crates/swc_ecma_ast/src/ast/js.rs",
    "crates/swc_ecma_ast/src/ast/ts.rs",
    "crates/swc_ecma_ast/src/ast/jsx.rs",
];

const AST_CRATE_PATH: &str = "swc_ecma_ast";

fn main() {
    let schema = parse_files(SOURCE_PATHS);

    let ast_builder_ret = ast_builder(&schema);
    ast_builder_ret.write_to_file().unwrap();

    let ast_property_ret = ast_property(&schema);
    ast_property_ret.write_to_file().unwrap();

    let ast_node = ast_node(&schema);
    ast_node.write_to_file().unwrap();

    let ast_clone_in = ast_clone_in(&schema);
    ast_clone_in.write_to_file().unwrap();

    let ast_visitor = ast_visitor(&schema);
    ast_visitor.write_to_file().unwrap();

    let ast_extra_data_compact = ast_extra_compact(&schema);
    ast_extra_data_compact.write_to_file().unwrap();

    let (
        core_estree_decoder_pretty,
        core_estree_decoder_minified,
        parser_estree_decoder_pretty,
        parser_estree_decoder_minified,
    ) = estree_decoder(&schema);
    core_estree_decoder_pretty.write_to_file().unwrap();
    core_estree_decoder_minified.write_to_file().unwrap();
    parser_estree_decoder_pretty.write_to_file().unwrap();
    parser_estree_decoder_minified.write_to_file().unwrap();
}
