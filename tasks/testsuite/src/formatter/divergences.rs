use std::collections::BTreeSet;

use super::{CaseResult, CaseStatus, ConformanceConfig};

pub(super) struct Divergences(&'static [Divergence]);

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

const NO_SEMI: &[&str] = &["semi: false"];
const BOTH_ARROW_PARENS: &[&str] = &["arrowParens: \"always\"", "arrowParens: \"avoid\""];
const ES5_TRAILING_COMMA: &[&str] = &["trailingComma: \"es5\""];

const fn fixture(path: &'static str, options: &'static [&'static str]) -> Fixture {
    Fixture { path, options }
}

// List only snapshot mismatches with a verified reason. The option strings match
// the normalized options shown by PRETTIER_FILTER. An empty string denotes default options.
pub(super) const KNOWN_DIVERGENCES: Divergences = Divergences(&[
    Divergence {
        reason: "Preserve comments on their original side of operator and terminator tokens",
        fixtures: &[
            fixture("js/binary-expression/chain-expression.js", DEFAULT),
            fixture("js/comments/binary-expressions.js", BOTH_SEMI_OPTIONS),
            fixture("js/comments/empty-statements.js", BOTH_SEMI_OPTIONS),
            fixture("js/comments/variable-declarator.js", BOTH_SEMI_OPTIONS),
            fixture("typescript/as/comments/18160.ts", DEFAULT),
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
    Divergence {
        reason: "Prettier changes these outputs on a second pass",
        fixtures: &[
            fixture("jsx/comments/in-attributes.js", &["bracketSameLine: true"]),
            fixture(
                "typescript/import-type/long-module-name/long-module-name4.ts",
                DEFAULT,
            ),
        ],
    },
    Divergence {
        reason: "Keep ignore markers on their original verbatim target",
        fixtures: &[
            fixture("js/ignore/issue-11077.js", DEFAULT),
            fixture("js/ignore/issue-13737.js", DEFAULT),
            fixture("js/ignore/issue-14404.js", DEFAULT),
            fixture(
                "js/ignore/semi/class-expression-decorator.js",
                BOTH_SEMI_OPTIONS,
            ),
            fixture("js/ignore/semi/head-ignored.js", BOTH_SEMI_OPTIONS),
            fixture("typescript/prettier-ignore/issue-14238.ts", DEFAULT),
            fixture("typescript/prettier-ignore/mapped-types.ts", DEFAULT),
            fixture(
                "typescript/prettier-ignore/prettier-ignore-nested-unions.ts",
                DEFAULT,
            ),
        ],
    },
    Divergence {
        reason: "Keep JSDoc casts bound to their original receiver",
        fixtures: &[
            fixture(
                "js/comments-closure-typecast/comment-in-the-middle.js",
                DEFAULT,
            ),
            fixture(
                "js/comments-closure-typecast/no-semi/not-on-same-line.js",
                NO_SEMI,
            ),
            fixture(
                "js/comments-closure-typecast/no-semi/with-other-comments.js",
                NO_SEMI,
            ),
        ],
    },
    Divergence {
        reason: "Keep comments before their array elision comma",
        fixtures: &[fixture("js/array-exprssion/numbers-with-holes.js", DEFAULT)],
    },
    Divergence {
        reason: "Preserve which lines TypeScript diagnostic directives cover",
        fixtures: &[fixture("js/comments/issue-11050.js", BOTH_SEMI_OPTIONS)],
    },
    Divergence {
        reason: "Preserve the original rendered JSX text",
        fixtures: &[
            fixture("js/unicode/nbsp-jsx.js", DEFAULT),
            fixture("jsx/escape/nbsp.js", DEFAULT),
            fixture("jsx/text-wrap/test.js", DEFAULT),
        ],
    },
    Divergence {
        reason: "Keep closing-tag line comments inside JSX",
        fixtures: &[fixture(
            "jsx/comments/in-end-tag.js",
            &["bracketSameLine: true"],
        )],
    },
    Divergence {
        reason: "Keep source parentheses around commented curried arrow bodies",
        fixtures: &[fixture(
            "js/arrow-function/currying-4.js",
            BOTH_ARROW_PARENS,
        )],
    },
    Divergence {
        reason: "Preserve the complete hashbang payload",
        fixtures: &[fixture("js/comments/trailing_space.js", BOTH_SEMI_OPTIONS)],
    },
    Divergence {
        reason: "Keep signature comments outside the body they precede",
        fixtures: &[
            fixture("js/class-comment/misc.js", DEFAULT),
            fixture("js/comments/function-declaration.js", BOTH_SEMI_OPTIONS),
            fixture("js/comments/function/18146.js", DEFAULT),
            fixture(
                "js/comments/function/between-parentheses-and-function-body.js",
                DEFAULT,
            ),
            fixture(
                "typescript/interface/comments-generic.ts",
                BOTH_SEMI_OPTIONS,
            ),
            fixture(
                "typescript/interface2/comments-declare.ts",
                ES5_TRAILING_COMMA,
            ),
            fixture("typescript/interface2/comments.ts", ES5_TRAILING_COMMA),
        ],
    },
]);

impl Divergences {
    pub(super) fn reason_for(&self, result: &CaseResult) -> Option<&'static str> {
        if !matches!(result.status, CaseStatus::Mismatched) || !result.is_valid_output() {
            return None;
        }
        self.0
            .iter()
            .find(|divergence| {
                divergence.fixtures.iter().any(|fixture| {
                    fixture.path == result.path
                        && fixture.options.contains(&result.options.as_str())
                })
            })
            .map(|divergence| divergence.reason)
    }

    pub(super) fn stale_errors(
        &self,
        config: ConformanceConfig,
        results: &[CaseResult],
    ) -> Vec<String> {
        let mismatches = results
            .iter()
            .filter(|result| matches!(result.status, CaseStatus::Mismatched))
            .map(|result| (result.path.as_str(), result.options.as_str()))
            .collect::<BTreeSet<_>>();
        let mut errors = Vec::new();
        for divergence in self.0 {
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
}

pub(super) fn option_label(options: &str) -> &str {
    if options.is_empty() {
        "default options"
    } else {
        options
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT, Divergence, Divergences, fixture};
    use crate::formatter::{CaseResult, CaseStatus, JAVASCRIPT};

    const PATH: &str = "js/example.js";
    const DIVERGENCES: Divergences = Divergences(&[Divergence {
        reason: "Example divergence",
        fixtures: &[fixture(PATH, DEFAULT)],
    }]);

    fn case_result(options: &str, status: CaseStatus) -> CaseResult {
        CaseResult {
            path: PATH.to_string(),
            options: options.to_string(),
            status,
            debug_diff: None,
            ast_debug_diff: None,
            ast_parity: Some(true),
            ast_error: None,
            idempotent: Some(true),
            second_pass_error: false,
        }
    }

    #[test]
    fn divergence_options() {
        let mismatch = case_result("", CaseStatus::Mismatched);
        assert_eq!(
            DIVERGENCES.reason_for(&mismatch),
            Some("Example divergence")
        );
        assert!(DIVERGENCES.stale_errors(JAVASCRIPT, &[mismatch]).is_empty());

        let matched = case_result("", CaseStatus::Matched);
        let other_option = case_result("semi: false", CaseStatus::Mismatched);
        assert_eq!(DIVERGENCES.reason_for(&other_option), None);
        assert_eq!(
            DIVERGENCES
                .stale_errors(JAVASCRIPT, &[matched, other_option])
                .len(),
            1
        );
    }

    #[test]
    fn divergence_output_checks() {
        let mut ast_mismatch = case_result("", CaseStatus::Mismatched);
        ast_mismatch.ast_parity = Some(false);
        assert_eq!(DIVERGENCES.reason_for(&ast_mismatch), None);

        let mut unstable = case_result("", CaseStatus::Mismatched);
        unstable.idempotent = Some(false);
        assert_eq!(DIVERGENCES.reason_for(&unstable), None);

        let invalid = case_result("", CaseStatus::StructuralError(String::new()));
        assert_eq!(DIVERGENCES.reason_for(&invalid), None);

        let panic = case_result("", CaseStatus::Panic(String::new()));
        assert_eq!(DIVERGENCES.reason_for(&panic), None);
    }
}
