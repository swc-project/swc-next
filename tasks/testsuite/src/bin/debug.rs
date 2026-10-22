use clap::{Parser, ValueEnum};
use testsuite::{
    cases::{
        Case,
        babel::BabelTransformCase,
        babel_parser::BabelParserCase,
        swc_resolver::SwcResolverCase,
        swc_transform::SwcTransformCase,
        test262::Test262Case,
        typescript::{
            parser::TypeScriptParserCase,
            transform::{TypeScriptTransformCase, TypeScriptTransformMode},
        },
    },
    runner::{
        Runner, codegen::CodegenRunner, parser::ParserRunner, semantic::SemanticRunner,
        transform::TransformerRunner,
    },
};

#[derive(Clone, ValueEnum)]
enum RunnerName {
    Parser,
    Codegen,
    Transform,
    Semantic,
}

#[derive(Clone, Copy, ValueEnum)]
enum CaseSet {
    #[value(name = "test262")]
    Test262,
    #[value(name = "typescript")]
    TypeScript,
    Babel,
    #[value(name = "swc_resolver")]
    SwcResolver,
    Swc,
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
    /// Override the fixture's TypeScript transform options.
    #[arg(long, value_enum)]
    typescript_mode: Option<TypeScriptMode>,
}

#[derive(Clone, Copy, ValueEnum)]
enum TypeScriptMode {
    Default,
    Verbatim,
    PreserveConstEnums,
    VerbatimPreserveConstEnums,
}

impl From<TypeScriptMode> for TypeScriptTransformMode {
    fn from(mode: TypeScriptMode) -> Self {
        match mode {
            TypeScriptMode::Default => Self::DEFAULT,
            TypeScriptMode::Verbatim => Self::VERBATIM,
            TypeScriptMode::PreserveConstEnums => Self::PRESERVE_CONST_ENUMS,
            TypeScriptMode::VerbatimPreserveConstEnums => Self::VERBATIM_PRESERVE_CONST_ENUMS,
        }
    }
}

fn main() {
    let args = AppArgs::parse();
    let res = match args.runner {
        RunnerName::Parser => run_conformance::<ParserRunner>(args.case, &args.filter),
        RunnerName::Codegen => run_conformance::<CodegenRunner>(args.case, &args.filter),
        RunnerName::Transform => match args.case {
            CaseSet::Babel => run_filtered::<TransformerRunner, BabelTransformCase>(&args.filter),
            CaseSet::TypeScript => TypeScriptTransformCase::read_all(
                args.typescript_mode.map(TypeScriptTransformMode::from),
            )
            .and_then(|cases| run_filtered_cases::<TransformerRunner, _>(&args.filter, cases)),
            CaseSet::Swc => run_filtered::<TransformerRunner, SwcTransformCase>(&args.filter),
            _ => Err(
                "`transform` runner requires `--case babel`, `--case typescript` or `--case swc`"
                    .into(),
            ),
        },
        RunnerName::Semantic if matches!(args.case, CaseSet::SwcResolver) => {
            run_filtered::<SemanticRunner, SwcResolverCase>(&args.filter)
        }
        RunnerName::Semantic => Err("`semantic` runner requires `--case swc_resolver`".into()),
    };
    println!("{:?}", res);
}

fn run_conformance<R>(case_set: CaseSet, filter: &str) -> Result<(), String>
where
    R: Runner<Test262Case> + Runner<TypeScriptParserCase> + Runner<BabelParserCase>,
{
    match case_set {
        CaseSet::Test262 => run_filtered::<R, Test262Case>(filter),
        CaseSet::TypeScript => run_filtered::<R, TypeScriptParserCase>(filter),
        CaseSet::Babel => run_filtered::<R, BabelParserCase>(filter),
        CaseSet::SwcResolver | CaseSet::Swc => {
            Err("parser/codegen do not run SWC transform/semantic cases".into())
        }
    }
}

fn run_filtered<R, C>(filter: &str) -> Result<(), String>
where
    C: Case,
    R: Runner<C>,
{
    run_filtered_cases::<R, C>(filter, C::read())
}

fn run_filtered_cases<R, C>(filter: &str, cases: Vec<C>) -> Result<(), String>
where
    C: Case,
    R: Runner<C>,
{
    let mut cases = cases
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
