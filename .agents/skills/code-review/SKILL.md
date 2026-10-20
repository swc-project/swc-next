---
name: code-review
description: Review changes in this repository for correctness, preserved test coverage, and compliance with AGENTS.md. Use only for pull request.
---

# Code Review

- Read the applicable `AGENTS.md` and review the requested diff with its surrounding
  code. Check changed tests and documentation against the repository conventions;
  report actionable violations with the relevant rule and source location.
- For test refactors, compare inputs, expected results, configuration combinations,
  and assertions with the original. Passing tests or unchanged test counts alone
  do not establish equivalent coverage; preserve checks when reorganizing tests.
- Allow loops for configuration matrices and traversal of actual results. Keep
  independent input cases explicit as required by `AGENTS.md`.
- Do not weaken assertions or expose private APIs merely to change test layout.
  Check that assertions through public APIs preserve the original guarantees.
