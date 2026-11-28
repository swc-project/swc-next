//! Compare source-level targets of formatter-sensitive directives.

mod source;
#[cfg(test)]
mod tests;

use std::{fs, ops::Range, path::PathBuf, process::ExitCode};

use anyhow::{Context, Result};
use clap::Parser;
use swc_next_ecma_ast::{Lang, NodeKind, SourceType, Span};
use swc_next_ecma_parser::{CommentMode, Options, TokenTag};
use swc_next_formatter::js::{self, FormatOptions, JsSemicolons};

use source::inspect;

#[derive(Parser)]
#[command(about = "Check directive targets using the SWC AST and source tokens")]
struct Args {
    /// Original JavaScript, JSX, TypeScript or TSX file.
    input: PathBuf,
    /// Compare this output with the input. Without it, check two formatter passes.
    candidate: Option<PathBuf>,
    /// Print resolved AST targets and token coverage, including preserved targets.
    #[arg(long)]
    show_targets: bool,
    #[arg(long, conflicts_with = "candidate")]
    no_semi: bool,
    #[arg(long, conflicts_with = "candidate")]
    width: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Marker {
    Pure,
    Cast,
    Ignore,
    ExpectError,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Target {
    Node(usize, NodeKind),
    /// Half-open range of aligned source tokens with starts on the covered line.
    Line {
        first: usize,
        end: usize,
        punctuation: Vec<(usize, usize)>,
    },
    Inactive,
    Unknown(&'static str),
}

#[derive(Debug)]
struct Occurrence {
    marker: Marker,
    text: String,
    line: usize,
    target: Target,
    target_span: Option<Span>,
}

impl Occurrence {
    fn same_marker(&self, other: &Self) -> bool {
        if self.marker != other.marker {
            return false;
        }
        if matches!(self.marker, Marker::Ignore | Marker::ExpectError) {
            // Reindenting a multiline block preserves its diagnostic directive.
            return self
                .text
                .split_whitespace()
                .eq(other.text.split_whitespace());
        }
        self.text == other.text
    }
}

#[derive(Default)]
struct ResultCounts {
    preserved: usize,
    activated: usize,
    changed: usize,
    unverified: usize,
}

impl ResultCounts {
    fn status(&self) -> u8 {
        if self.changed > 0 {
            1
        } else if self.unverified > 0 {
            2
        } else {
            0
        }
    }
}

struct Snapshot {
    shape: Vec<(NodeKind, usize)>,
    code: Vec<(TokenTag, Vec<u8>)>,
    directives: Vec<Occurrence>,
    prologues: Vec<(usize, Vec<String>)>,
    gaps: Vec<Vec<TokenTag>>,
}

impl Snapshot {
    fn covered_code(&self, range: Range<usize>) -> impl Iterator<Item = usize> + '_ {
        range.filter(|&i| {
            // Parentheses, commas and semicolons are recorded in gaps. Braces and
            // brackets also delimit syntax, but still participate in global AST/token alignment.
            !matches!(
                self.code[i].0,
                TokenTag::LeftBrace
                    | TokenTag::RightBrace
                    | TokenTag::LeftBracket
                    | TokenTag::RightBracket
            )
        })
    }
}

fn compare(
    label: &str,
    before: &Snapshot,
    after: &Snapshot,
    stable: bool,
    result: &mut ResultCounts,
) {
    let aligned = before.shape == after.shape && before.code == after.code;
    if !aligned {
        eprintln!("{label}: unverified: AST or significant tokens cannot be aligned");
        result.unverified += 1;
    }
    for ((scope, old), (_, new)) in before.prologues.iter().zip(&after.prologues) {
        // The program always corresponds. Function identities require structural alignment.
        if *scope != 0 && !aligned {
            break;
        }
        if old != new {
            eprintln!("{label}: changed directive prologue at scope #{scope}: {old:?} -> {new:?}");
            result.changed += 1;
        } else if !old.is_empty() {
            result.preserved += 1;
        }
    }
    if before.directives.len() != after.directives.len() {
        eprintln!(
            "{label}: changed marker count {} -> {}",
            before.directives.len(),
            after.directives.len()
        );
        result.changed += 1;
    }
    for (old, new) in before.directives.iter().zip(&after.directives) {
        if old.marker == Marker::Unsupported || new.marker == Marker::Unsupported {
            eprintln!(
                "{label}: unverified directive form at source line {} and candidate line {}",
                old.line, new.line
            );
            result.unverified += 1;
            continue;
        }
        if !old.same_marker(new) {
            eprintln!(
                "{label}: changed marker text or order at source line {} and candidate line {}",
                old.line, new.line
            );
            result.changed += 1;
            continue;
        }
        let unknown = match (&old.target, &new.target) {
            (Target::Unknown(reason), _) | (_, Target::Unknown(reason)) => Some(*reason),
            (Target::Node(..) | Target::Line { .. }, Target::Node(..) | Target::Line { .. })
                if !aligned =>
            {
                // A missing marker or a newly inactive target can still be established locally.
                continue;
            }
            (
                Target::Line {
                    first: a,
                    end: b,
                    punctuation: p,
                },
                Target::Line {
                    first: c,
                    end: d,
                    punctuation: q,
                },
            ) => {
                let mut original = before.covered_code(*a..*b).peekable();
                let mut candidate = after.covered_code(*c..*d).peekable();
                if original.peek().is_none() && candidate.peek().is_some() {
                    Some("the source diagnostic line contained only delimiters")
                } else if original.eq(candidate)
                    && (old.target != new.target
                        || p.iter()
                            .chain(q)
                            .any(|&(gap, _)| before.gaps[gap] != after.gaps[gap]))
                {
                    Some("only diagnostic-line delimiters changed")
                } else {
                    None
                }
            }
            (Target::Line { first, end, .. }, Target::Inactive)
                if before.covered_code(*first..*end).next().is_none() =>
            {
                Some("the source diagnostic line contained only delimiters")
            }
            (Target::Inactive, Target::Line { first, end, .. })
                if after.covered_code(*first..*end).next().is_none() =>
            {
                Some("the candidate diagnostic line contains only delimiters")
            }
            _ => None,
        };
        if let Some(reason) = unknown {
            eprintln!(
                "{label}: unverified {:?} at source line {} and candidate line {}: {reason}",
                old.marker, old.line, new.line
            );
            result.unverified += 1;
        } else if old.target == new.target {
            result.preserved += 1;
        } else if !stable && old.target == Target::Inactive {
            result.activated += 1;
            println!(
                "{label}: activated {:?} at candidate line {}",
                new.marker, new.line
            );
        } else {
            result.changed += 1;
            eprintln!(
                "{label}: changed {:?} at source line {} and candidate line {}: {:?} -> {:?}",
                old.marker, old.line, new.line, old.target, new.target
            );
            if let (
                Target::Line {
                    first: a, end: b, ..
                },
                Target::Line {
                    first: c, end: d, ..
                },
            ) = (&old.target, &new.target)
            {
                report_coverage("lost", *a..*b, *c..*d, before);
                report_coverage("added", *c..*d, *a..*b, after);
            }
        }
    }
}

fn show_targets(label: &str, source: &str, snapshot: &Snapshot) {
    for (scope, values) in &snapshot.prologues {
        if !values.is_empty() {
            println!("{label} scope #{scope}: directive prologue {values:?}");
        }
    }
    for entry in &snapshot.directives {
        if let Some(span) = entry.target_span {
            println!(
                "{label} line {} {:?}: {:?} at {}..{} {:?}",
                entry.line,
                entry.marker,
                entry.target,
                span.start,
                span.end,
                source[span.start as usize..span.end as usize]
                    .chars()
                    .take(80)
                    .collect::<String>()
            );
        } else {
            println!(
                "{label} line {} {:?}: {:?}",
                entry.line, entry.marker, entry.target
            );
        }
    }
}

fn report_coverage(change: &str, here: Range<usize>, other: Range<usize>, snapshot: &Snapshot) {
    for index in snapshot
        .covered_code(here)
        .filter(|i| !other.contains(i))
        .take(8)
    {
        eprintln!(
            "  {change} code token #{index}: {}",
            String::from_utf8_lossy(&snapshot.code[index].1)
        );
    }
}

fn run(args: Args) -> Result<u8> {
    let path = args.input.to_string_lossy();
    let options = Options {
        lang: Lang::from_path(&path),
        source_type: SourceType::from_path(&path),
        comments: CommentMode::Flat,
        preserve_parens: true,
    };
    let input = fs::read_to_string(&args.input).context("read input")?;
    let before = inspect(&input, options)?;
    if args.show_targets {
        show_targets("input", &input, &before);
    }
    let mut result = ResultCounts::default();
    if let Some(candidate) = args.candidate {
        let output = fs::read_to_string(candidate).context("read candidate")?;
        let after = inspect(&output, options)?;
        if args.show_targets {
            show_targets("candidate", &output, &after);
        }
        compare("input/candidate", &before, &after, false, &mut result);
    } else {
        let mut style = FormatOptions::prettier();
        if args.no_semi {
            style.semicolons = JsSemicolons::AsNeeded.into();
        }
        if let Some(width) = args.width {
            style.line_width = width;
        }
        let first = js::format(&input, options, style).context("format first pass")?;
        let second = js::format(&first, options, style).context("format second pass")?;
        let first_snapshot = inspect(&first, options)?;
        let second_snapshot = inspect(&second, options)?;
        if args.show_targets {
            show_targets("F1", &first, &first_snapshot);
            show_targets("F2", &second, &second_snapshot);
        }
        compare("input/F1", &before, &first_snapshot, false, &mut result);
        compare(
            "F1/F2",
            &first_snapshot,
            &second_snapshot,
            true,
            &mut result,
        );
        if first != second {
            eprintln!("F1/F2: output bytes changed");
            result.changed += 1;
        }
    }
    println!(
        "preserved={} activated={} changed={} unverified={}",
        result.preserved, result.activated, result.changed, result.unverified
    );
    Ok(result.status())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("unverified: {error:#}");
            ExitCode::from(2)
        }
    }
}
