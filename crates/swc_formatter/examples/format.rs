use std::{
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, ValueEnum};
use swc_next_ecma_ast::{Lang, Severity, SourceType};
use swc_next_ecma_parser::Options as ParserOptions;
use swc_next_formatter::{js, markdown};
use tempfile::NamedTempFile;

#[derive(Parser)]
#[command(about = "Format JavaScript, JSX, TypeScript, TSX, or Markdown")]
struct Args {
    /// Files to format. With no files, read stdin.
    #[arg(required_unless_present_any = ["stdin_filepath", "parser"])]
    files: Vec<PathBuf>,

    /// Infer the parser and source type for stdin from this path.
    #[arg(long, conflicts_with = "files")]
    stdin_filepath: Option<PathBuf>,

    /// Override the parser inferred from the input path.
    #[arg(long, value_enum)]
    parser: Option<InputParser>,

    /// Report inputs that would change.
    #[arg(long, conflicts_with = "write")]
    check: bool,

    /// Replace each file whose formatting changes.
    #[arg(long, requires = "files")]
    write: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum InputParser {
    #[value(alias = "javascript")]
    Js,
    Jsx,
    #[value(alias = "typescript")]
    Ts,
    Tsx,
    Dts,
    #[value(alias = "md")]
    Markdown,
}

enum InputFormat {
    Ecma(ParserOptions),
    Markdown,
}

impl InputFormat {
    fn resolve(path: Option<&Path>, parser: Option<InputParser>) -> Result<Self> {
        let filename = path
            .and_then(Path::file_name)
            .map(|filename| filename.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let extension = Path::new(&filename).extension().and_then(OsStr::to_str);
        let lang = match (parser, extension) {
            (Some(InputParser::Markdown), _)
            | (
                None,
                Some("md" | "markdown" | "mdown" | "mdwn" | "mkd" | "mkdn" | "mkdown" | "ronn"),
            ) => {
                return Ok(Self::Markdown);
            }
            (Some(InputParser::Js | InputParser::Jsx), _)
            | (None, Some("js" | "jsx" | "mjs" | "cjs")) => Lang::Jsx,
            (Some(InputParser::Ts), _) if Lang::from_path(&filename) == Lang::Dts => Lang::Dts,
            (Some(InputParser::Ts), _) => Lang::Ts,
            (Some(InputParser::Tsx), _) => Lang::Tsx,
            (Some(InputParser::Dts), _) => Lang::Dts,
            (None, Some("ts" | "tsx" | "mts" | "cts")) => Lang::from_path(&filename),
            (None, _) => bail!("cannot infer a parser from this path. Specify --parser"),
        };
        Ok(Self::Ecma(ParserOptions {
            lang,
            source_type: SourceType::from_path(&filename),
            ..Default::default()
        }))
    }

    fn format(self, source: &str) -> Result<String> {
        match self {
            Self::Ecma(options) => {
                js::format(source, options, js::FormatOptions::prettier()).map_err(format_error)
            }
            Self::Markdown => Ok(markdown::format(source, Default::default())?),
        }
    }
}

#[derive(Clone, Copy)]
enum Input<'a> {
    File(&'a Path),
    Stdin(Option<&'a Path>),
}

impl<'a> Input<'a> {
    fn path(self) -> Option<&'a Path> {
        match self {
            Self::File(path) => Some(path),
            Self::Stdin(path) => path,
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    let mut stdout = io::stdout().lock();
    let mut status = 0;
    if args.files.is_empty() {
        status = run_input(
            Input::Stdin(args.stdin_filepath.as_deref()),
            &args,
            &mut stdout,
        );
    } else {
        for path in &args.files {
            status = status.max(run_input(Input::File(path), &args, &mut stdout));
        }
    }
    if let Err(error) = stdout.flush() {
        eprintln!("stdout: {error}");
        return ExitCode::from(2);
    }
    ExitCode::from(status)
}

fn run_input(input: Input<'_>, args: &Args, stdout: &mut impl Write) -> u8 {
    let label = input
        .path()
        .unwrap_or_else(|| Path::new("<stdin>"))
        .display();
    match format_input(input, args, stdout) {
        Ok(true) => {
            eprintln!("{label} needs formatting");
            1
        }
        Ok(false) => 0,
        Err(error) => {
            eprintln!("{label}: {error:#}");
            2
        }
    }
}

fn format_input(input: Input<'_>, args: &Args, stdout: &mut impl Write) -> Result<bool> {
    let format = InputFormat::resolve(input.path(), args.parser)?;
    let source = match input {
        Input::File(path) => fs::read_to_string(path).context("read file")?,
        Input::Stdin(_) => io::read_to_string(io::stdin().lock()).context("read stdin")?,
    };
    let output = format.format(&source)?;
    if args.check {
        return Ok(source != output);
    }
    if args.write {
        let Input::File(path) = input else {
            bail!("--write requires a file path");
        };
        if source != output {
            replace_file(path, output.as_bytes())?;
        }
    } else {
        stdout
            .write_all(output.as_bytes())
            .context("write stdout")?;
    }
    Ok(false)
}

fn replace_file(path: &Path, output: &[u8]) -> Result<()> {
    let path = path.canonicalize().context("resolve input path")?;
    let permissions = fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .context("open input for writing")?
        .metadata()
        .context("read input metadata")?
        .permissions();
    let directory = path.parent().context("find the input directory")?;
    let mut temporary = NamedTempFile::new_in(directory).context("create temporary output")?;
    temporary
        .write_all(output)
        .context("write temporary output")?;
    temporary
        .as_file()
        .set_permissions(permissions)
        .context("preserve file permissions")?;
    temporary.persist(path).context("replace input file")?;
    Ok(())
}

fn format_error(error: js::FormatError) -> anyhow::Error {
    if let js::FormatError::Parse(diagnostics) = &error
        && let Some(first) = diagnostics
            .iter()
            .find(|entry| entry.severity == Severity::Error)
    {
        return anyhow!(
            "{} at {}..{}",
            first.message,
            first.span.start,
            first.span.end
        );
    }
    error.into()
}
