---
name: code-review
description: Review pull requests in this repository for correctness, simplicity, behavior changes, performance, and preserved coverage. Use only for PR reviews.
---

# Code Review

Default to English unless requested otherwise.

## Review

- Read the applicable `AGENTS.md`.
- Omit routine bot comments and unrelated diffs. Fetch linked issues or discussions when they resolve a concrete uncertainty. Don't read CI results.
- Prefer removal or reuse over added flags, layers, and special cases. Suggest concrete simpler alternatives without weakening behavior or safety.
- Internal binary protocols and their transport APIs do not follow semver; do not flag their changes alone as breaking changes. Still verify that producers and consumers remain consistent within the same version.

## Explain

- Explain the problem and core implementation idea, not a file inventory.
- Show behavior changes with complete and small inputs, relevant options, and explicit before/after outputs or structures. Include recovery differences when relevant; distinguish execution, fixture expectations, and inference.
- For optimizations, show short before/after code excerpts explaining avoided work and preserved behavior.

## Report

Give actionable findings by severity, with code location, trigger, consequence, and correction. State when none are found; still explain the change.

Separate optional performance and code quality improvements from required fixes, with concrete benefits and tradeoffs. Omit speculative suggestions.

End with one sentence stating merge readiness and any blocker or material validation gap. Optional improvements alone do not block merging.
