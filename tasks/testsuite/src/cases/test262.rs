use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use serde::Deserialize;
use swc_next_ecma_ast::{Lang, SourceType, Span};

use crate::cases::{Case, fixtures, json_paths, read_json, swc_conformance};

pub struct Test262Case {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    expects_error: bool,
    error_spans: Option<Vec<Span>>,
    name: String,
}

#[derive(Deserialize)]
struct Input {
    path: PathBuf,
    metadata: Option<Meta>,
    modes: Vec<String>,
}

#[derive(Deserialize)]
struct Meta {
    negative: Option<Negative>,
}

#[derive(Deserialize)]
struct Negative {
    phase: String,
    #[serde(rename = "type")]
    kind: Option<String>,
}

impl Test262Case {
    fn read_cases() -> Vec<Self> {
        let mut inputs = Vec::new();
        for suite in ["test262", "test262-legacy"] {
            let root = fixtures().join(suite).join("test");
            for path in json_paths(suite) {
                let input: Input = read_json(&path);
                inputs.push((root.join(&input.path), input, suite));
            }
        }
        inputs
            .into_par_iter()
            .filter(|(path, _, _)| {
                let path = path.to_string_lossy().replace('\\', "/");
                // These Annex B cases require call expressions to be accepted as assignment
                // targets in non-strict mode, which this parser intentionally does not support.
                !path.contains("_FIXTURE")
                    && !path.contains("test262/test/staging")
                    && !path
                        .contains("test262/test/annexB/language/expressions/assignmenttargettype")
            })
            .flat_map(|(path, input, suite)| Self::from_input(path, input, suite))
            .collect()
    }

    fn from_input(path: PathBuf, input: Input, suite: &str) -> Vec<Self> {
        let Some(meta) = input.metadata else {
            return Vec::new();
        };
        let code = read_to_string(&path).unwrap();
        let expects_error = meta
            .negative
            .as_ref()
            .is_some_and(|negative| matches!(negative.phase.as_str(), "parse" | "early"));
        let has_reference = meta.negative.is_some_and(|negative| {
            expects_error && negative.kind.as_deref() == Some("SyntaxError")
        });
        let mut oracle_root = swc_conformance().join("tests/parser/test262");
        if suite == "test262-legacy" {
            oracle_root.push("legacy");
        }
        let mut cases = Vec::new();
        for name in input.modes {
            let source_type = if name == "module" {
                SourceType::Module
            } else {
                SourceType::Script
            };
            let source = if name == "strict" {
                format!("\"use strict\";\n{code}")
            } else {
                code.clone()
            };
            let error_spans = if has_reference {
                let snapshot = input.path.with_added_extension(format!("{name}.json"));
                read_error_spans(&oracle_root.join(snapshot), source.len())
            } else {
                None
            };
            cases.push(Self {
                path: path.clone(),
                code: source,
                source_type,
                expects_error,
                error_spans,
                name,
            });
        }
        cases
    }
}

#[derive(Deserialize)]
struct ReferenceDiagnostic {
    start: Option<u32>,
    end: Option<u32>,
}

fn read_error_spans(path: &Path, source_len: usize) -> Option<Vec<Span>> {
    let diagnostics: Vec<ReferenceDiagnostic> = read_json(path);
    let mut spans = Vec::new();
    for diagnostic in diagnostics {
        if let (Some(start), Some(end)) = (diagnostic.start, diagnostic.end) {
            assert!(
                start <= end && end as usize <= source_len,
                "invalid diagnostic span: {}",
                path.display()
            );
            spans.push(Span::new(start, end));
        }
    }
    (!spans.is_empty()).then_some(spans)
}

impl Case for Test262Case {
    const NAME: &'static str = "test262";

    fn read() -> Vec<Self> {
        Self::read_cases()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn source_type(&self) -> SourceType {
        self.source_type
    }

    fn lang(&self) -> Lang {
        Lang::Js
    }

    fn expects_error(&self) -> bool {
        self.expects_error
    }

    fn expected_error_spans(&self) -> Option<&[Span]> {
        self.error_spans.as_deref()
    }
}
