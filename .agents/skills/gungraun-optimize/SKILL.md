---
name: gungraun-optimize
description: "Autonomously optimize Rust code with Gungraun benchmarks in GitHub Actions using Callgrind, Cachegrind, DHAT, baseline comparisons, profiles, and iterative commits. Use when asked to make Rust code faster, reduce instructions, cache misses, or allocations, investigate a Gungraun CI performance regression, analyze where Rust code spends work, or create or repair the Gungraun benchmark and GitHub Actions harness needed before optimization."
---

# Gungraun Optimize

Act as a performance engineer. Measure in GitHub Actions, inspect Gungraun profiles, make one focused change, re-measure, and continue until the target is met or further work has diminishing value.

## Enforce the measurement contract

- Treat Gungraun results produced by GitHub Actions for the relevant base and head revisions as the source of truth for comparative performance.
- Use local runs only to check correctness or that a harness starts successfully. Never support a performance claim with local benchmark output.
- Default to Callgrind `Ir` (executed instructions). Use cache metrics or Cachegrind for cache work and DHAT for heap allocation or memory work.
- Describe `Ir`, cache simulation, and estimated cycles as profiling metrics, not wall-clock time. Use an existing wall-time suite only as optional final corroboration; do not create one unless the user asks.
- Keep new CI reporting-only for performance changes. Do not add regression limits that fail CI. Preserve existing repository limits unless the user authorizes changing them.
- Stop and ask for help instead of substituting uncontrolled measurements when GitHub authentication, permissions, runner availability, or a valid base/head comparison cannot be established.

## Inspect before changing anything

1. Identify the code and benchmark cases to optimize. Ask only if neither the request nor the repository reveals a usable target.
2. Identify the requested metric. Use `Ir` when the request only says "faster" or "optimize."
3. Read repository instructions and inspect the worktree, current branch, default branch, remotes, open pull request, and existing user changes. Preserve unrelated work.
4. Search manifests, benchmark targets, Gungraun macros and configuration, GitHub workflows, summary renderers, and wall-time suites. Prefer `rg` and `git grep`.
5. Inspect GitHub authentication and the workflow trigger. Determine whether an existing pull request and workflow run can be reused.
6. Record the benchmark filter, base revision, Rust toolchain, Gungraun library version, runner version, build features, and relevant environment. Keep these invariant across comparisons.

## Establish a valid CI harness

Adapt an existing harness and workflow instead of replacing them. If either is missing or invalid, read [the Gungraun CI reference](references/gungraun-ci.md) before creating or repairing it.

- Prefer the smallest benchmark that covers the requested behavior while exercising realistic inputs.
- Keep setup and teardown outside the measured function and protect inputs and results with `std::hint::black_box`.
- Prefer a separate workspace member for library benchmarks when feasible. Choose a binary benchmark only when the executable boundary is part of the behavior under test.
- Match `gungraun-runner` to the library dependency. Use a Linux Actions runner with Valgrind.
- Save a base baseline, benchmark head against it, save machine-readable summaries, and retain profiles and flamegraphs as artifacts.
- Ensure base and head execute the same benchmark definition, toolchain, features, and environment. Never compare different workloads under the same benchmark name.
- If the base revision lacks the harness, bootstrap only benchmark plumbing that can exercise unchanged base code. If that is unsafe or the required API does not exist on base, create a harness-only prerequisite change and do not claim a comparison until it is available on the target branch.

Commit a new or repaired harness as its own coherent change. Run its correctness checks before beginning optimization.

## Prepare the GitHub Actions loop

Reuse the current non-default branch and pull request when they match the task. Otherwise:

1. Create a descriptive branch that follows repository naming rules.
2. Commit only the intended harness or optimization changes.
3. Push the branch and create a draft pull request so the benchmark workflow runs.

Never push the default branch. Keep one optimization attempt per commit. After publishing a failed attempt, revert it with a new commit instead of rewriting public history.

Establish an initial CI result before optimizing. Record the run URL, commit SHA, benchmark identities, exact metrics, summary schema version, and artifact names. If the workflow compares only against the pull request base, retain each accepted head summary so the exact delta from the preceding accepted commit can also be calculated.

## Run the optimization loop

### 1. Analyze the current result

- Scope subsequent runs with Gungraun's benchmark filter when the workflow permits it.
- Rank target cases by the requested metric. Start with `Ir` unless another metric was requested.
- Inspect regular and differential flamegraphs or annotate the retained Callgrind data. Find functions with high self cost, unexpectedly repeated work, avoidable allocations, poor locality, expensive conversions, or unnecessary library calls.
- Select one hypothesis that the available profile evidence supports. Do not optimize from intuition alone.

### 2. Make one focused change

- Read and understand the affected path before editing it.
- Preserve semantics and public compatibility unless the user explicitly allows a change.
- Keep the patch small enough that its benchmark effect is attributable to one idea.
- Run the narrow correctness tests first, then the broader relevant checks required by the repository.

### 3. Commit, push, and wait

- Commit the working change using the repository's message convention.
- Push the branch and wait for the matching GitHub Actions run to finish.
- Ignore superseded or cancelled runs and associate results with the exact head SHA.
- Download or inspect the saved summary and profile artifacts. Treat new, removed, renamed, or failed benchmark cases as non-comparable until explained.

### 4. Compare and decide

- Compare the new head with the pull request base and with the preceding accepted head when available.
- Accept a change when at least one target case improves by 1% or more, no target case regresses, no related case regresses by 1% or more, and correctness checks pass.
- Keep an exact improvement below 1% only when the code is equally simple or simpler and no target or related case regresses. Otherwise revert it as complexity without meaningful benefit.
- Reject any target regression. Reject a related regression of 1% or more unless the user explicitly accepts the tradeoff.
- Report an improvement of 5% or more as an intermediate result with exact base/head counts, percentages, the run link, and the profile evidence before continuing.
- Revert an ineffective or regressing published commit, push the revert, and confirm CI returns to the last accepted state.
- After two failed hypotheses for the same hot path, move to the next profile-supported target.

Repeat with fresh profiles because the hot path can move after each accepted optimization.

## Handle failures explicitly

- Fix deterministic build, benchmark, summary parsing, or artifact upload failures before interpreting performance.
- Retry a clearly transient Actions failure once. If it repeats, report the evidence and ask for help.
- Treat missing GitHub permissions, unavailable required runners, and invalid base/head workloads as blockers to measurement.
- For fork pull requests, accept that write-token PR comments may be unavailable; use the read-only job summary and artifacts instead.
- Never use `pull_request_target` to run untrusted pull request code.

## Finish and report

Stop when the user is satisfied, no profile-supported bottleneck remains, two consecutive targets yield less than 1% without simplifying code, or further work requires an unapproved architectural change.

Report:

- every accepted optimization and why it affects the measured hot path;
- exact before/after metrics and percentages from the associated Actions runs;
- benchmark names, base/head SHAs, run links, and relevant profile or flamegraph evidence;
- correctness checks and any existing wall-time corroboration;
- reverted attempts, remaining tradeoffs, and promising next targets;
- the draft pull request and final branch state.
