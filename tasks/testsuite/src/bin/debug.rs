use clap::{Parser, ValueEnum};
use testsuite::{
    cases::{
        Case,
        babel::BabelTransformCase,
        babel_parser::BabelParserCase,
        local::LocalCase,
        swc_resolver::SwcResolverCase,
        test262::Test262Case,
        typescript::TypeScriptCase,
        typescript_transform::{TypeScriptTransformInput, TypeScriptTransformMode},
    },
    runner::{
        Runner, babel_transform::BabelTransformRunner, codegen::CodegenRunner,
        parser::ParserRunner, resolver::ResolverRunner,
        typescript_transform::TypeScriptTransformRunner,
    },
};

#[derive(Clone, ValueEnum)]
enum RunnerName {
    Parser,
    Codegen,
    #[value(name = "babel_transform")]
    BabelTransform,
    Resolver,
    #[value(name = "typescript_transform")]
    TypeScriptTransform,
}

#[derive(Clone, Copy, ValueEnum)]
enum CaseSet {
    #[value(name = "test262")]
    Test262,
    Local,
    #[value(name = "typescript")]
    TypeScript,
    Babel,
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
        RunnerName::BabelTransform if matches!(args.case, CaseSet::Babel) => {
            run_filtered::<BabelTransformRunner, BabelTransformCase>(&args.filter)
        }
        RunnerName::BabelTransform => {
            Err("`babel_transform` runner requires `--case babel`".into())
        }
        RunnerName::Resolver if matches!(args.case, CaseSet::SwcResolver) => {
            run_filtered::<ResolverRunner, SwcResolverCase>(&args.filter)
        }
        RunnerName::Resolver => Err("`resolver` runner requires `--case swc_resolver`".into()),
        RunnerName::TypeScriptTransform if matches!(args.case, CaseSet::TypeScript) => {
            run_typescript_transform(
                &args.filter,
                args.typescript_mode.map(TypeScriptTransformMode::from),
            )
        }
        RunnerName::TypeScriptTransform => {
            Err("`typescript_transform` runner requires `--case typescript`".into())
        }
    };
    println!("{:?}", res);
}

fn run_typescript_transform(
    filter: &str,
    mode_override: Option<TypeScriptTransformMode>,
) -> Result<(), String> {
    let inputs = TypeScriptTransformInput::read_all()?;
    let mut units = inputs
        .iter()
        .flat_map(|input| {
            input
                .units
                .iter()
                .map(move |unit| (&input.relative_path, &input.modes, unit))
        })
        .filter(|(relative_path, _, unit)| {
            format!("{} :: {}", relative_path.display(), unit.name).contains(filter)
        })
        .collect::<Vec<_>>();
    if units.len() > 1 {
        let candidates = units
            .iter()
            .map(|(relative_path, _, unit)| format!("{} :: {}", relative_path.display(), unit.name))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "Multiple cases found for filter `{filter}`; narrow it to one case:\n{candidates}"
        ));
    }
    let (relative_path, modes, unit) = units
        .pop()
        .ok_or_else(|| format!("No case found for filter: {filter}"))?;
    let mode = match (mode_override, modes.as_slice()) {
        (Some(mode), _) => mode,
        (None, [mode]) => *mode,
        (None, modes) => {
            let modes = modes
                .iter()
                .map(|mode| mode.name())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "Fixture has multiple TypeScript transform option variants ({modes}); pass `--typescript-mode` to select one"
            ));
        }
    };
    println!("Runner: {}", TypeScriptTransformRunner::NAME);
    println!(
        "Case: {} :: {} [{}]",
        relative_path.display(),
        unit.name,
        mode.name()
    );
    let output = TypeScriptTransformRunner::transform(mode, unit)?;
    println!("{output}");
    Ok(())
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
