use std::collections::{HashMap, hash_map::Entry};

use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_core::common::comments::SingleThreadedComments;
use swc_experimental_ecma_ast::{Ast, NodeId, Program, Visit};
use swc_experimental_ecma_parser::{Lexer, Parser, StringSource};

use crate::{AppArgs, cases::Case, suite::TestResult};

pub struct NoMemoryHoleRunner;

impl NoMemoryHoleRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        cases
            .par_iter()
            .filter_map(|case| {
                if args.debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                if case.should_fail() {
                    return None;
                }

                let syntax = case.syntax();
                let input = StringSource::new(case.code());
                let comments = SingleThreadedComments::default();
                let lexer = Lexer::new(syntax, Default::default(), input, Some(&comments));
                let parser = Parser::new_from(lexer);
                let ret = match case.ext().as_str() {
                    "js" | "jsx" => parser.parse_program(),
                    "cjs" => parser
                        .parse_script()
                        .map(|ret| ret.map_root(Program::Script)),
                    "mjs" => parser
                        .parse_module()
                        .map(|ret| ret.map_root(Program::Module)),
                    "ts" | "tsx" => {
                        return Some(TestResult::Ignored {
                            path: case.path().to_owned(),
                        });
                    }
                    _ => unreachable!(),
                };

                let Ok(ret) = ret else {
                    return None;
                };

                let mut use_visitor = Use {
                    ast: &ret.ast,
                    used: Default::default(),
                };

                use_visitor.visit_program(ret.root);

                if use_visitor.used.len() != ret.ast.nodes_len() as usize {
                    return Some(TestResult::Failed {
                        path: case.path().to_owned(),
                        error: "Memory hole is detected".to_string(),
                    });
                }

                for values in use_visitor.used.values() {
                    if *values != 1 {
                        return Some(TestResult::Failed {
                            path: case.path().to_owned(),
                            error: "Shared node is detected".to_string(),
                        });
                    }
                }

                Some(TestResult::Passed {
                    path: case.path().to_owned(),
                })
            })
            .collect()
    }
}

struct Use<'a> {
    ast: &'a Ast,
    used: HashMap<NodeId, u32>,
}

impl Visit for Use<'_> {
    fn ast(&self) -> &swc_experimental_ecma_ast::Ast {
        self.ast
    }

    fn enter_node(&mut self, node_id: NodeId) {
        match self.used.entry(node_id) {
            Entry::Occupied(mut occupied_entry) => *occupied_entry.get_mut() += 1,
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(1);
            }
        }
    }
}
