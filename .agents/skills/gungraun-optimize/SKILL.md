---
name: gungraun-optimize
description: "Autonomously profile and optimize Rust code with Gungraun benchmarks in GitHub Actions using Callgrind, Cachegrind, DHAT, baseline comparisons, and one accepted optimization per stacked pull request. Use when asked to make Rust code faster, reduce instructions, cache misses, or allocations, investigate a Gungraun CI performance regression, analyze where Rust code spends work, or create a temporary Gungraun benchmark and GitHub Actions harness before optimization."
---

# Gungraun Optimize

Act as a performance engineer. Measure in GitHub Actions, inspect Gungraun profiles, publish each accepted optimization as one commit in its own pull request with `gh stack`, and continue while a safe profile-supported hypothesis remains.

## Obey the non-negotiable boundaries

- Use `gh stack` for all stack creation, layer addition, pull request submission, synchronization, rebasing, and restructuring. Do not create optimization pull requests with `gh pr create`.
- Start a dedicated stack from the repository trunk. Do not inspect, adopt, reuse, modify, rebase, wait for, or otherwise interact with unrelated pull requests or their branches.
- Keep exactly one focused optimization commit in every accepted stack layer relative to its parent. Keep exactly one accepted optimization in the corresponding pull request diff.
- Never modify AST code or its generated representation. In this repository, treat `crates/swc_ecma_ast/**` and `crates/swc_ast_macros/**` as forbidden paths. Also reject changes elsewhere that alter AST types, fields, variants, layout, schema, serialization shape, generators, or generated AST output.
- Default to target-independent production changes. Unless the user explicitly requests optimization for a named architecture, do not introduce or expand architecture-specific code paths or build configuration, including `cfg(target_arch = ...)`, `cfg!(target_arch = ...)`, `CARGO_CFG_TARGET_ARCH`, `std::arch` or `core::arch` intrinsics, inline assembly, `#[target_feature]`, runtime CPU-feature detection, or architecture-specific `RUSTFLAGS`, `target-cpu`, or `target-feature` settings.
- Treat the Linux x86_64 Gungraun runner as a measurement environment only. Never use its architecture, profiles, or an existing architecture-specific implementation as permission to specialize production code for x86_64.
- When the user explicitly requests a named-architecture optimization, first confirm that a Linux Actions runner, Valgrind, and a valid Gungraun harness are available on that architecture. Require a GitHub Actions base/head comparison on that architecture which executes the optimized path; if it cannot be established, stop before editing and report the measurement blocker. Scope every performance claim to the measured architecture, retain a correct portable fallback and compatibility with supported targets, run available cross-target correctness checks, and report every target that was not validated.
- Check staged and committed path lists and inspect the staged diff before every publication. Abandon any hypothesis that requires an AST change; do not request an exception. Unless the user explicitly requested a named architecture, also abandon any hypothesis that violates the architecture-portability guard instead of asking for permission to retain it.
- Never push the default branch.

## Enforce the measurement contract

- Treat Gungraun results produced by GitHub Actions for the relevant base and head revisions on the architecture that executes the evaluated code path as the source of truth for comparative performance.
- Use local runs only to check correctness or that a harness starts successfully. Never support a performance claim with local benchmark output.
- Default to Callgrind `Ir` (executed instructions). Use cache metrics or Cachegrind for cache work and DHAT for heap allocation or memory work.
- Describe `Ir`, cache simulation, and estimated cycles as profiling metrics, not wall-clock time. Use an existing wall-time suite only as optional final corroboration; do not create one unless the user asks.
- Keep new CI reporting-only for performance changes. Do not add regression limits that fail CI. Preserve existing repository limits unless the user authorizes changing them.
- Stop and ask for help instead of substituting uncontrolled measurements when GitHub authentication, permissions, runner availability, or a valid base/head comparison cannot be established.

## Inspect before changing anything

1. Identify the code and benchmark cases to optimize. Ask only if neither the request nor the repository reveals a usable target.
2. Identify the requested metric. Use `Ir` when the request only says "faster" or "optimize."
3. Read repository instructions and inspect only the worktree, trunk, remotes, stack state owned by this task, and existing user changes. Preserve unrelated work without investigating unrelated pull requests.
4. Search manifests, benchmark targets, Gungraun macros and configuration, GitHub workflows, summary renderers, and wall-time suites. Prefer `rg` and `git grep`.
5. Confirm GitHub authentication, runner access, the workflow trigger, and the installed `gh stack` commands. Never substitute another stacked-pull-request tool.
6. Record the benchmark filter, base revision, Rust toolchain, Gungraun library version, runner version and architecture, build features, and relevant environment. Keep these invariant across comparisons.

## Establish a valid CI harness

Reuse a valid existing harness and workflow without changing them when possible. If either is missing or invalid, read [the Gungraun CI reference](references/gungraun-ci.md) and create a temporary harness layer unless the user explicitly requests a permanent harness.

- Prefer the smallest benchmark that covers the requested behavior while exercising realistic inputs.
- Keep setup and teardown outside the measured function and protect inputs and results with `std::hint::black_box`.
- Prefer a separate workspace member for library benchmarks when feasible. Choose a binary benchmark only when the executable boundary is part of the behavior under test.
- Match `gungraun-runner` to the library dependency. Use a Linux Actions runner with Valgrind.
- Save a base baseline, benchmark head against it, save machine-readable summaries, and retain profiles and flamegraphs as artifacts.
- Ensure base and head execute the same benchmark definition, toolchain, features, and environment. Never compare different workloads under the same benchmark name.
- If the base revision lacks the harness, bootstrap only benchmark plumbing that can exercise unchanged base code. If that is unsafe or the required API does not exist on base, treat the workload as non-comparable and stop.

When temporary CI is required:

1. Run `gh stack init --base <trunk> <temporary-ci-branch>` to make the harness-only bottom layer.
2. Commit only benchmark and reporting plumbing, run its correctness checks, and publish it as a draft pull request with `gh stack submit --auto`.
3. Record the temporary branch, pull request, commit, files, and manifest hunks so all of them can be removed later.
4. Establish the initial CI result and inspect its profiles before changing production code.
5. Keep all optimization layers above this temporary layer until profiling and optimization work ends.

## Run the stacked optimization loop

### 1. Analyze the current result

- Scope subsequent runs with Gungraun's benchmark filter when the workflow permits it.
- Rank target cases by the requested metric. Start with `Ir` unless another metric was requested.
- Inspect regular and differential flamegraphs or annotate the retained Callgrind data. Find functions with high self cost, unexpectedly repeated work, avoidable allocations, poor locality, expensive conversions, or unnecessary library calls.
- Before creating or measuring a provisional layer, record and freeze the comparable target portfolio, related cases, user-designated or workload-critical protected cases, and case frequencies. Use trustworthy documented production frequencies when available; otherwise assign and disclose an equal frequency of one to every case. Do not change these classifications or weights after seeing provisional results without disclosing the reason and obtaining user approval.
- Select one hypothesis that the available profile evidence supports. Do not optimize from intuition alone.
- Reject a hypothesis that requires new or expanded architecture-specific production code or build settings unless the user explicitly requested optimization for that named architecture. Do not infer such a request from the CI runner or profile evidence.

### 2. Create one provisional layer

- Run `gh stack add <attempt-branch>` above the last accepted layer, or use `gh stack init --base <trunk> <attempt-branch>` for the first layer when no temporary CI layer exists.
- Read and understand the affected path before editing it. Reject the hypothesis immediately if it touches a forbidden AST path or representation.
- Preserve semantics and public compatibility unless the user explicitly allows a non-AST change.
- Keep the patch small enough that its benchmark effect is attributable to one idea.
- Run narrow correctness tests first, then the broader relevant checks required by the repository.
- Stage only intended files and verify the AST path guard. Inspect both the staged diff and each affected function or module's enclosing configuration, inherited `cfg` or `target_feature` attributes, and relevant existing Cargo configuration, build scripts, and compiler flags for `target_arch`, `CARGO_CFG_TARGET_ARCH`, `std::arch`, `core::arch`, runtime feature-detection macros, assembly, architecture-specific flags, or equivalents. Treat a change inside an existing architecture-gated scope as expanding that implementation. Accept it only under the explicit named-architecture exception, then create one provisional optimization commit.

### 3. Submit and measure

- Publish or update the dedicated stack with `gh stack submit --auto`; keep pull requests in draft while measuring.
- Wait only for the matching task-owned pull request run. Ignore unrelated pull requests and superseded or cancelled runs, and associate results with the exact head SHA.
- Download or inspect the saved summary and profile artifacts. Treat new, removed, renamed, or failed benchmark cases as non-comparable until explained.
- Record the run URL, commit SHA, benchmark identities, exact metrics, summary schema version, and artifact names.

### 4. Compare and decide

- Compare the layer's incremental effect against its pull request base and the cumulative stack effect against trunk. Require both comparisons to satisfy the applicable portfolio and per-case thresholds below; never use earlier wins to justify a newly harmful layer or let individually small regressions evade the cumulative thresholds.
- Compute the primary target portfolio score as `sum(frequency * head) / sum(frequency * base)`, equivalently the arithmetic mean of `head / base` ratios weighted by `frequency * base`. Express portfolio improvement as `1 - score`. Use the equal-weight geometric mean of case ratios only as a secondary suite statistic.
- Also inspect the frequency-weighted absolute metric delta, the count and distribution of improvements and regressions, the largest individual regression, and profile evidence.
- Require correctness checks to pass. Accept automatically when both incremental and cumulative primary target portfolio scores improve by at least 1% and every incremental and cumulative individual target and related regression is below 1%.
- Also accept an isolated non-protected regression of at least 1% but below 3% when both primary target portfolio scores improve by at least 5%, neither the incremental nor cumulative regression reaches 3%, the regression is not repeated across a related workload family, and profile evidence explains the tradeoff.
- Do not automatically accept when either comparison has a protected-case regression of 1% or more or any individual regression of 3% or more. Keep the provisional layer, report the exact tradeoff, and ask the user to decide.
- Apply the same portfolio reasoning to related cases. Do not discard a large target improvement solely because a non-representative case outside the target portfolio has a sub-1% regression.
- Reject a layer when its incremental primary target portfolio worsens or when small regressions are broad enough to make the overall effect negative. Accept an improvement below 1% only when the code is equally simple or simpler, both primary portfolio scores still improve, and no incremental or cumulative individual regression reaches 1%.
- Re-run the matching CI comparison once before deciding when a result is near a threshold or the aggregate, individual cases, and profile evidence conflict.
- Report an improvement of 5% or more as an intermediate result with exact base/head counts, percentages, the run link, and the profile evidence before continuing.
- For every accepted tradeoff, report both incremental and cumulative primary portfolio scores, the secondary geometric means, frequency-weighted absolute metric changes, individual regressions, protected-case status, and profile evidence.
- Keep an accepted layer unchanged and add the next attempt as a new layer above it.
- After a layer is conclusively rejected, use `gh stack modify` to drop it, resubmit the remaining stack with `gh stack submit --auto`, then use `gh pr close <task-owned-pr> --delete-branch` only to close and delete that rejected pull request branch. Do not drop an ambiguous tradeoff while awaiting the user's decision, and do not retain a revert commit in an accepted pull request.
- After two failed hypotheses for the same hot path, move to the next profile-supported target without treating the failures as a reason to finish.

Repeat with fresh profiles because the hot path can move after each accepted optimization.

## Handle failures explicitly

- Fix deterministic build, benchmark, summary parsing, or artifact upload failures before interpreting performance.
- Retry a clearly transient Actions failure once. If it repeats, report the evidence and ask for help.
- Treat missing GitHub permissions, unavailable required runners, and invalid base/head workloads as blockers to measurement.
- For fork pull requests, accept that write-token PR comments may be unavailable; use the read-only job summary and artifacts instead.
- Never use `pull_request_target` to run untrusted pull request code.

## Remove temporary CI before finishing

If this task created temporary CI, complete every step below before presenting the accepted pull requests as final:

1. Use `gh stack modify` to drop every rejected layer and the temporary CI bottom layer. Allow the command to cascade-rebase accepted optimization layers directly onto trunk while preserving one optimization commit per layer.
2. Verify each accepted branch contains exactly one commit relative to its new parent and contains no temporary CI commit.
3. Verify the cumulative trunk-to-top diff contains none of the recorded temporary files or manifest hunks and no forbidden AST change.
4. Run `gh stack submit --auto` to push the rewritten task-owned branches and update every accepted pull request base.
5. Close the temporary pull request and delete its remote branch with `gh pr close <temporary-pr> --delete-branch`.
6. Run `gh stack sync --prune`, then verify the active stack contains only accepted optimization pull requests, the bottom pull request targets trunk, and every later pull request targets the preceding accepted layer.
7. Preserve the CI run URLs and artifact evidence gathered before cleanup. Run the repository's normal correctness CI on the final rewritten stack.

Do not finish if the temporary pull request is open, its remote branch exists, its commit is reachable from an accepted branch, or any temporary CI change remains in the final diff. The closed pull request record may remain on GitHub.

## Finish and report

Finish only when the user is satisfied, no safe profile-supported bottleneck remains, measurement or permissions are blocked, correctness cannot be preserved, or further work requires an unapproved architectural change. Do not finish because an improvement is small, a numeric performance target was reached or missed, several attempts failed, or returns appear to diminish.

Report:

- every accepted optimization, its single-commit pull request, and why it affects the measured hot path;
- exact before/after metrics and percentages from the associated Actions runs;
- benchmark names, base/head SHAs, run links, and relevant profile or flamegraph evidence;
- correctness checks and any existing wall-time corroboration;
- for explicitly requested architecture-specific work, the named and measured architecture, matching-architecture Actions run and exact base/head metrics, portable fallback, target-scoped claims, cross-target checks, and validation gaps;
- rejected attempts, remaining tradeoffs, and promising next targets;
- the final `gh stack` order and proof that temporary CI changes, commits, pull requests, and branches were cleaned up as required.
