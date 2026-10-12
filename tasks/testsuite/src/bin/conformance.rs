use std::{fs, process::ExitCode};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use testsuite::{
    cases::{
        Case,
        babel::BabelTransformCase,
        babel_parser::BabelParserCase,
        local::LocalCase,
        swc_resolver::SwcResolverCase,
        swc_transform::SwcTransformCase,
        test262::Test262Case,
        typescript::{parser::TypeScriptParserCase, transform::TypeScriptTransformCase},
    },
    runner::{
        Runner, codegen::CodegenRunner, parser::ParserRunner, resolver::ResolverRunner,
        transform::TransformerRunner,
    },
    suite::{TestResult, to_snapshot},
    util::crate_root,
};

pub fn main() -> ExitCode {
    rayon::ThreadPoolBuilder::new()
        .stack_size(32 * 1024 * 1024)
        .build_global()
        .unwrap();
    let panicked = [
        update_snapshots::<Test262Case>(),
        update_snapshots::<LocalCase>(),
        update_snapshots::<TypeScriptParserCase>(),
        update_snapshots::<BabelParserCase>(),
        update_snapshot::<ResolverRunner, SwcResolverCase>(&SwcResolverCase::read()),
        update_snapshot::<TransformerRunner, BabelTransformCase>(&BabelTransformCase::read()),
        update_snapshot::<TransformerRunner, SwcTransformCase>(&SwcTransformCase::read()),
        update_snapshot::<TransformerRunner, TypeScriptTransformCase>(
            &TypeScriptTransformCase::read(),
        ),
    ]
    .into_iter()
    .any(|panicked| panicked);

    if panicked {
        eprintln!("Conformance run contained one or more panics");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn update_snapshots<C: Case>() -> bool {
    let cases = C::read();
    [
        update_snapshot::<ParserRunner, C>(&cases),
        update_snapshot::<CodegenRunner, C>(&cases),
    ]
    .into_iter()
    .any(|panicked| panicked)
}

fn update_snapshot<R: Runner<C>, C: Case>(cases: &[C]) -> bool {
    let results = cases
        .par_iter()
        .filter(|case| R::should_run(*case))
        .map(R::run)
        .collect::<Vec<_>>();
    let panicked = results
        .iter()
        .any(|result| matches!(result, TestResult::Panic { .. }));
    let snapshot = to_snapshot(R::NAME, C::NAME, &results);
    let snapshot_path =
        crate_root()
            .join("snapshots")
            .join(format!("{}_{}.snap", R::NAME, C::NAME));
    fs::write(&snapshot_path, snapshot).unwrap();
    println!("Updated {}", snapshot_path.display());
    panicked
}
