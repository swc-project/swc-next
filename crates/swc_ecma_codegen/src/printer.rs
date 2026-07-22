use std::path::PathBuf;

use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, Comment, Program};

use crate::SourceMap;

#[derive(Clone)]
pub struct Options {
    pub ascii_only: bool,
    pub minify: bool,
    pub source_map_path: Option<PathBuf>,
    pub comments: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            ascii_only: false,
            minify: false,
            source_map_path: None,
            comments: true,
        }
    }
}

pub struct PrintResult<'a> {
    pub code: &'a [u8],
    pub legal_comments: Vec<Comment>,
    pub map: Option<SourceMap<'a>>,
}

pub fn print<'a>(
    allocator: &'a Allocator,
    tree: &'a Ast<'a>,
    root: Program,
    options: Options,
) -> PrintResult<'a> {
    todo!()
}
