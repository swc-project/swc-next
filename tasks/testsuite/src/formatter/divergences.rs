use std::collections::BTreeSet;

use super::{CaseResult, CaseStatus, ConformanceConfig};

struct Divergence {
    reason: &'static str,
    fixtures: &'static [Fixture],
}

struct Fixture {
    path: &'static str,
    options: &'static [&'static str],
}

const DEFAULT: &[&str] = &[""];
const BOTH_SEMI_OPTIONS: &[&str] = &["", "semi: false"];

const fn fixture(path: &'static str, options: &'static [&'static str]) -> Fixture {
    Fixture { path, options }
}

// List only snapshot mismatches with a verified reason. The option strings match
// the normalized options shown by PRETTIER_FILTER. An empty string denotes default options.
const KNOWN_DIVERGENCES: &[Divergence] = &[
    Divergence {
        reason: "Preserve comments on their original side of operator and terminator tokens",
        fixtures: &[
            fixture("js/babel-plugins/bigint.js", DEFAULT),
            fixture("js/binary-expressions/chain-expression.js", DEFAULT),
            fixture("js/comments/binary-expressions.js", BOTH_SEMI_OPTIONS),
            fixture("js/comments/empty-statements.js", BOTH_SEMI_OPTIONS),
            fixture("js/comments/variable-declarator.js", BOTH_SEMI_OPTIONS),
            fixture("typescript/binary-expressions/chain-expression.ts", DEFAULT),
        ],
    },
    Divergence {
        reason: "Separate opening angles in heritage type arguments to preserve TypeScript syntax",
        fixtures: &[fixture(
            "typescript/type-arguments-bit-shift-left-like/3.ts",
            DEFAULT,
        )],
    },
];

pub(super) fn reason_for(result: &CaseResult) -> Option<&'static str> {
    if !matches!(result.status, CaseStatus::Mismatched) || !result.is_valid_output() {
        return None;
    }
    KNOWN_DIVERGENCES
        .iter()
        .find(|divergence| {
            divergence.fixtures.iter().any(|fixture| {
                fixture.path == result.path && fixture.options.contains(&result.options.as_str())
            })
        })
        .map(|divergence| divergence.reason)
}

pub(super) fn stale_errors(config: ConformanceConfig, results: &[CaseResult]) -> Vec<String> {
    let mismatches = results
        .iter()
        .filter(|result| matches!(result.status, CaseStatus::Mismatched))
        .map(|result| (result.path.as_str(), result.options.as_str()))
        .collect::<BTreeSet<_>>();
    let mut errors = Vec::new();
    for divergence in KNOWN_DIVERGENCES {
        for fixture in divergence.fixtures {
            let (root, _) = fixture.path.split_once('/').unwrap();
            if !config.fixture_roots.contains(&root) {
                continue;
            }
            for &options in fixture.options {
                if !mismatches.contains(&(fixture.path, options)) {
                    let label = option_label(options);
                    errors.push(format!(
                        "{} ({label}): stale known divergence; no output mismatch",
                        fixture.path
                    ));
                }
            }
        }
    }
    errors.sort();
    errors
}

pub(super) fn option_label(options: &str) -> &str {
    if options.is_empty() {
        "default options"
    } else {
        options
    }
}
