---
name: code-review
description: Review pull requests in this repository for correctness, simplicity, behavior changes, preserved test coverage, and compliance with AGENTS.md, including optional performance and code quality improvements.
---

# Code Review

Read the applicable `AGENTS.md` and review the requested diff with its surrounding
code. Check changed tests and documentation against the repository conventions;
report actionable violations with the relevant rule and source location.

## Design and Simplicity

Treat added complexity as something to justify. For new abstractions, layers,
branches, and special cases, identify the requirement they serve and consider
whether removing them or reusing an existing path would still satisfy it.
Recommend the smallest design that preserves required behavior; support
simplification suggestions with a concrete alternative and its tradeoffs.

## Behavior Changes

Print an input as example and its actual before/after results. For behavior-preserving refactors, report
the evidence for equivalence.

## Test Coverage

- When reviewing a GitHub pull request link, do not rerun tests locally.
- For test refactors, compare inputs, expected results, configuration combinations,
  and assertions with the original. Passing tests or unchanged test counts alone
  do not establish equivalent coverage; preserve checks when reorganizing tests.
- Allow loops for configuration matrices and traversal of actual results. Keep
  independent input cases explicit as required by `AGENTS.md`.
- Do not weaken assertions or expose private APIs merely to change test layout.
  Check that assertions through public APIs preserve the original guarantees.

## Optional Improvements

Include a separate list of optional performance and code quality improvements
when supported by the diff. For each, identify the relevant code, a concrete
change, its expected benefit, and material tradeoffs. Distinguish measured gains
from hypotheses that need profiling or benchmarks. Keep these suggestions
separate from correctness findings and required fixes; do not pad the review
with speculative optimizations or style preferences.

## Conclusion

End the review with one sentence stating whether the change is ready to merge
or needs fixes, naming the main blocker or any material validation gap. Optional
improvements alone should not prevent a positive conclusion.
