use clap::{Parser, ValueEnum};
use testsuite::{
    cases::{
        Case, babel::BabelTransformCase, babel_parser::BabelParserCase, local::LocalCase,
        swc_resolver::SwcResolverCase, swc_transform::SwcTransformCase, test262::Test262Case,
        typescript::TypeScriptCase,
    },
    runner::{
        Runner, babel_transform::BabelTransformRunner, codegen::CodegenRunner,
        parser::ParserRunner, resolver::ResolverRunner, swc_transform::SwcTransformRunner,
    },
};

#[derive(Clone, ValueEnum)]
enum RunnerName {
    Parser,
    Codegen,
    #[value(name = "babel_transform")]
    BabelTransform,
    #[value(name = "swc_transform")]
    SwcTransform,
    Resolver,
}

#[derive(Clone, Copy, ValueEnum)]
enum CaseSet {
    #[value(name = "test262")]
    Test262,
    Local,
    #[value(name = "typescript")]
    TypeScript,
    Babel,
    Swc,
    #[value(name = "swc_resolver")]
    SwcResolver,
}

#[derive(Parser)]
#[command(about = "Run one testsuite case without updating snapshots")]
struct AppArgs {
    /// Runner to execute.
    #[arg(long, value_enum)]
    runner: RunnerName,

    /// Case collection to search.
    #[arg(long, value_enum)]
    case: CaseSet,

    /// Contains filter applied to the fixture path and case name.
    #[arg(long)]
    filter: String,
}

fn main() {
    let args = AppArgs::parse();
    let res = match args.runner {
        RunnerName::Parser => run_conformance::<ParserRunner>(args.case, &args.filter),
        RunnerName::Codegen => run_conformance::<CodegenRunner>(args.case, &args.filter),
        RunnerName::BabelTransform if matches!(args.case, CaseSet::Babel) => {
            run_filtered::<BabelTransformRunner, BabelTransformCase>(&args.filter)
        }
        RunnerName::BabelTransform => {
            Err("`babel_transform` runner requires `--case babel`".into())
        }
        RunnerName::SwcTransform if matches!(args.case, CaseSet::Swc) => {
            run_filtered::<SwcTransformRunner, SwcTransformCase>(&args.filter)
        }
        RunnerName::SwcTransform => Err("`swc_transform` runner requires `--case swc`".into()),
        RunnerName::Resolver if matches!(args.case, CaseSet::SwcResolver) => {
            run_filtered::<ResolverRunner, SwcResolverCase>(&args.filter)
        }
        RunnerName::Resolver => Err("`resolver` runner requires `--case swc_resolver`".into()),
    };
    println!("{:?}", res);
}

fn run_conformance<R>(case_set: CaseSet, filter: &str) -> Result<(), String>
where
    R: Runner<Test262Case> + Runner<LocalCase> + Runner<TypeScriptCase> + Runner<BabelParserCase>,
{
    match case_set {
        CaseSet::Test262 => run_filtered::<R, Test262Case>(filter),
        CaseSet::Local => run_filtered::<R, LocalCase>(filter),
        CaseSet::TypeScript => run_filtered::<R, TypeScriptCase>(filter),
        CaseSet::Babel => run_filtered::<R, BabelParserCase>(filter),
        CaseSet::Swc => Err("parser/codegen do not run SWC transform cases".into()),
        CaseSet::SwcResolver => Err("parser/codegen do not run SWC resolver cases".into()),
    }
}

fn run_filtered<R, C>(filter: &str) -> Result<(), String>
where
    C: Case,
    R: Runner<C>,
{
    let mut cases = C::read()
        .into_iter()
        .filter(|case| case_label(case).contains(filter))
        .collect::<Vec<_>>();
    if cases.len() > 1 {
        let candidates = cases.iter().map(case_label).collect::<Vec<_>>().join("\n");
        return Err(format!(
            "Multiple cases found for filter `{filter}`; narrow it to one case:\n{candidates}"
        ));
    }
    let case = cases
        .pop()
        .ok_or_else(|| format!("No case found for filter: {filter}"))?;
    println!("Runner: {}", R::NAME);
    println!("Case: {}", case_label(&case));
    if R::should_run(&case) {
        println!("{}", R::run(&case));
    } else {
        println!("Skipped");
    }
    Ok(())
}

fn case_label<C: Case>(case: &C) -> String {
    format!("{} - {}", case.relative_path().display(), case.name())
}
