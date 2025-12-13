use std::collections::{HashMap, hash_map::Entry};

use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_ecma_ast::{Ast, NodeId, Visit};

use crate::{
    AppArgs,
    cases::Case,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct NoMemoryHoleRunner;

impl NoMemoryHoleRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        cases
            .par_iter()
            .filter_map(|case| {
                if args.debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                if case.should_ignore() {
                    return Some(TestResult::Ignored {
                        path: case.relative_path().to_owned(),
                    });
                }

                let (root, ast) = match parse(case) {
                    ParseResult::Succ(ret) => ret,
                    _ => return None,
                };

                let mut use_visitor = Use {
                    ast: &ast,
                    used: Default::default(),
                };

                use_visitor.visit_program(root);

                if use_visitor.used.len() != ast.nodes_len() as usize {
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
