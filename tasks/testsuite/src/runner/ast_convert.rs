use std::panic::{AssertUnwindSafe, catch_unwind};

use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, Program};
use swc_experimental_ecma_ast_compat::AstCompat;
use swc_experimental_ecma_semantic::resolver::resolver;

use crate::{
    AppArgs,
    ast_compare::ast_compare,
    cases::Case,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct AstConvertRunner;

impl AstConvertRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        let iter = cases.par_iter();
        iter.filter_map(|case| {
            if args.debug {
                println!("[{}] {:?}", "Debug".green(), case.relative_path());
            }

            if case.should_ignore() {
                return Some(TestResult::Ignored {
                    path: case.relative_path().to_owned(),
                });
            }

            if case.should_fail() {
                return None;
            }

            let allocator = Allocator::new();
            let (experimental_root, mut ast) = match parse(&allocator, case) {
                ParseResult::Succ(ret, ast) => (ret, ast),
                ParseResult::Panic => {
                    return Some(TestResult::Panic {
                        path: case.relative_path().to_owned(),
                    });
                }
                ParseResult::Fail(_) | ParseResult::Ignore => return None,
            };

            let ret = catch_unwind(AssertUnwindSafe(|| {
                let legacy_root = convert_experimental_to_legacy(experimental_root, &mut ast);
                let code = swc_core::ecma::codegen::to_code(&legacy_root);

                let roundtrip_allocator = Allocator::new();
                let (roundtrip_root, mut roundtrip_ast) = match crate::runner::parse_code(
                    &roundtrip_allocator,
                    &code,
                    case.syntax(),
                    case.is_module(),
                ) {
                    ParseResult::Succ(root, ast) => (root, ast),
                    _ => panic!("generated legacy AST failed to parse"),
                };
                let roundtrip_legacy_root =
                    convert_experimental_to_legacy(roundtrip_root, &mut roundtrip_ast);

                (legacy_root, roundtrip_legacy_root)
            }));

            let (legacy_root, roundtrip_legacy_root) = match ret {
                Ok(ret) => ret,
                Err(_) => {
                    return Some(TestResult::Panic {
                        path: case.relative_path().to_owned(),
                    });
                }
            };

            if let Some(error) = ast_compare(&legacy_root, &roundtrip_legacy_root) {
                return Some(TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error,
                });
            }

            Some(TestResult::Passed {
                path: case.relative_path().to_owned(),
            })
        })
        .collect()
    }
}

fn convert_experimental_to_legacy(
    program: Program,
    ast: &mut Ast<'_>,
) -> swc_core::ecma::ast::Program {
    let semantic = resolver(program, ast);
    AstCompat::new(ast, &semantic).compat_program(program)
}
