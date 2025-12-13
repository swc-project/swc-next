use std::collections::HashSet;

use colored::*;
use pico_args::Arguments;
use rayon::ThreadPoolBuilder;

use crate::{
    cases::{
        Case,
        misc::MiscCase,
        test262,
        test262_parser::{self},
    },
    runner::{
        parser::ParserRunner, parser_no_memory_hole::NoMemoryHoleRunner, semantic::SemanticRunner,
        transform_remove_paren::RemoveParenRunner,
    },
    suite::TestResult,
};

mod cases;
mod runner;
mod suite;
mod util;

pub struct AppArgs {
    pub debug: bool,
    pub cases: Vec<String>,
    pub runners: HashSet<String>,
    pub failures: HashSet<String>,
}

pub fn main() {
    // Initialize args
    let mut args = Arguments::from_env();
    let args = AppArgs {
        debug: args.contains("--debug"),
        cases: args.values_from_str("--cases").unwrap(),
        runners: args
            .values_from_str("--runners")
            .unwrap()
            .into_iter()
            .collect(),
        failures: args
            .values_from_str("--failures")
            .unwrap()
            .into_iter()
            .collect(),
    };

    if args.debug {
        ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .unwrap();
    }

    // Run tests
    let mut results = Vec::new();
    let misc_cases = filter(&args, MiscCase::read());
    let test262_cases = filter(&args, test262::Test262Case::read());
    let test262_parser_cases = filter(&args, test262_parser::Test262ParserCase::read());

    if args.runners.is_empty() || args.runners.contains("parser") {
        results.extend(ParserRunner::run(&args, &misc_cases));
        results.extend(ParserRunner::run(&args, &test262_cases));
        results.extend(ParserRunner::run(&args, &test262_parser_cases));
    }

    if args.runners.is_empty() || args.runners.contains("no_memory_hole") {
        results.extend(NoMemoryHoleRunner::run(&args, &misc_cases));
        results.extend(NoMemoryHoleRunner::run(&args, &test262_cases));
        results.extend(NoMemoryHoleRunner::run(&args, &test262_parser_cases));
    }

    if args.runners.is_empty() || args.runners.contains("semantic") {
        results.extend(SemanticRunner::run(&args, &misc_cases));
        results.extend(SemanticRunner::run(&args, &test262_cases));
        results.extend(SemanticRunner::run(&args, &test262_parser_cases));
    }

    if args.runners.is_empty() || args.runners.contains("remove_paren") {
        results.extend(RemoveParenRunner::run(&args, &misc_cases));
        results.extend(RemoveParenRunner::run(&args, &test262_cases));
        results.extend(RemoveParenRunner::run(&args, &test262_parser_cases));
    }

    // Collect results
    let mut passed = 0;
    let mut failed = 0;
    let mut panicked = 0;
    let mut ignored = 0;
    for result in results {
        match result {
            TestResult::Passed { .. } => {
                passed += 1;
            }
            TestResult::Failed { path, error } => {
                failed += 1;
                if args.failures.is_empty() || args.failures.contains("failed") {
                    println!("Failed: {} - {}", path.display(), error.red());
                }
            }
            TestResult::Panic { path } => {
                panicked += 1;
                if args.failures.is_empty() || args.failures.contains("panic") {
                    println!("Panic: {}", path.display());
                }
            }
            TestResult::Ignored { .. } => {
                ignored += 1;
            }
        }
    }

    println!("\n{}", "Test Summary:".bold());
    println!("Passed: {}", passed.to_string().green());
    println!("Failed: {}", failed.to_string().red());
    println!("Ignored: {}", ignored.to_string().yellow());
    println!("Panicked: {}", panicked.to_string().yellow());

    if failed > 0 {
        std::process::exit(1);
    }
}

fn filter<T: Case>(args: &AppArgs, list: Vec<T>) -> Vec<T> {
    if args.cases.is_empty() {
        return list;
    }

    list.into_iter()
        .filter(|case| {
            args.cases
                .iter()
                .any(|f| case.path().to_string_lossy().as_ref().contains(f))
        })
        .collect()
}
