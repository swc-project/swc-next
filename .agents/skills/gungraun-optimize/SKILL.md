---
name: gungraun-optimize
description: "Don't trigger if not asked explicitly. Autonomously profile and optimize Rust code and the Node.js JavaScript API with Gungraun benchmarks in GitHub Actions using Callgrind, Cachegrind, DHAT, PR-free manual comparisons, and one accepted optimization per pull request, stacking only dependent optimizations. Use when asked to make Rust code faster, reduce instructions, cache misses, or allocations, investigate a Gungraun CI performance regression, analyze where Rust code spends work, or create a temporary Gungraun benchmark and GitHub Actions harness before optimization."
---

# Gungraun Optimize

Act as a performance engineer. Measure provisional changes in GitHub Actions without creating pull requests, inspect Gungraun profiles, immediately publish each accepted optimization as one commit in its own draft pull request, and continue while a safe profile-supported hypothesis remains.

## Obey the non-negotiable boundaries

- Publish every independent accepted optimization as its own pull request based directly on the repository trunk, so that it can be reviewed and merged separately. Do not stack independent optimizations.
- Stack an accepted optimization only when it depends on another unmerged accepted optimization from this task. A dependency exists when the optimization does not apply cleanly to trunk, does not compile or stay correct without the other change, or meets the acceptance thresholds only on top of the other change. Shared files, a shared hot path, or the order of discovery alone are not dependencies. When the dependency has merged, base the optimization on trunk instead.
- Use `gh stack` for all creation, layer addition, pushes, submission, synchronization, rebasing, and restructuring of dependent stacks. Use `gh pr create --draft --base <trunk>` only for independent optimizations.
- Do not inspect, adopt, reuse, modify, rebase, wait for, or otherwise interact with unrelated pull requests or their branches.
- Keep exactly one focused optimization commit in every accepted branch relative to its parent: trunk for an independent optimization or the dependency branch for a stacked layer. Keep exactly one accepted optimization in the corresponding pull request diff.
- Never alter AST types, fields, variants, layout, schema, or serialization shape, including through generators or generated AST output. Apply this restriction to the content of the change, regardless of its path. Shared helpers and other implementations may be optimized across crates, including inside `crates/swc_ecma_ast/` and `crates/swc_ast_macros/`, when they preserve the AST representation.
- Prefer optimizing an existing shared helper at its original definition. Do not copy its functionality into a caller crate or shadow it with a local implementation to optimize it or bypass the AST restriction. For example, optimize `is_ident_continue_ascii` in `crates/swc_ecma_ast/src/syntax.rs` directly instead of duplicating it in codegen.
- Default to target-independent production changes. Unless the user explicitly requests optimization for a named architecture, do not introduce or expand architecture-specific code paths or build configuration, including `cfg(target_arch = ...)`, `cfg!(target_arch = ...)`, `CARGO_CFG_TARGET_ARCH`, `std::arch` or `core::arch` intrinsics, inline assembly, `#[target_feature]`, runtime CPU-feature detection, or architecture-specific `RUSTFLAGS`, `target-cpu`, or `target-feature` settings.
- Treat the Linux x86_64 Gungraun runner as a measurement environment only. Never use its architecture, profiles, or an existing architecture-specific implementation as permission to specialize production code for x86_64.
- When the user explicitly requests a named-architecture optimization, first confirm that a Linux Actions runner, Valgrind, and a valid Gungraun harness are available on that architecture. Require a GitHub Actions base/head comparison on that architecture which executes the optimized path; if it cannot be established, stop before editing and report the measurement blocker. Scope every performance claim to the measured architecture, retain a correct portable fallback and compatibility with supported targets, run available cross-target correctness checks, and report every target that was not validated.
- Check staged and committed path lists and inspect the staged diff before every publication. Abandon any hypothesis that changes the protected AST representation; do not request an exception. Unless the user explicitly requested a named architecture, also abandon any hypothesis that violates the architecture-portability guard instead of asking for permission to retain it.
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
3. Read repository instructions and inspect only the worktree, trunk, remotes, branches and stacks owned by this task, and existing user changes. Preserve unrelated work without investigating unrelated pull requests.
4. Search manifests, benchmark targets, Gungraun macros and configuration, GitHub workflows, summary renderers, and wall-time suites. Prefer `rg` and `git grep`.
5. Confirm GitHub authentication, runner access, the workflow trigger, and the installed `gh stack` commands. Never substitute another stacked-pull-request tool for dependent stacks.
6. Freeze the starting trunk SHA for independent attempts and cumulative comparisons. Record the benchmark filter, base revision, Rust toolchain, Gungraun library version, runner version and architecture, build features, and relevant environment. Keep these invariant across comparisons.

## JavaScript API measurements

For `@swc-next/core` JavaScript API work, use the permanent `JavaScript API
Gungraun` workflow and `gungraun_js` binary harness. Read
[tasks/benchmark/js/README.md](../../../tasks/benchmark/js/README.md) for commands,
case filters and the measurement boundary. PRs run five representative cases;
use manual dispatch without a filter for all 24 cases. Use the N-API public JS entry point;
a Rust library-only result does not measure JS encoding, lazy decoding or FFI.

Use `--jitless --predictable` to disable JIT and reduce GC scheduling variation. Compare exact base/head SHAs with the same head harness,
fixtures, Node flags and release settings. The workflow reports three fresh
processes per case. Check the individual trials and median/range before applying
the optimization thresholds below. Overlapping ranges are inconclusive; do not
accept an optimization from those counts. Repeat the matching CI comparison and
inspect profiles. Interpreter frames may lack JS source names. Do not compare
these counts directly with JIT-enabled profiles. These main-thread Ir
profiles do not establish wall-time speedups or background V8 costs.

Select `parse` for the lazy API return, `parse_full` for AST materialization,
`print` for JS AST encoding and native printing, or the path helper cases.
Record `comparison.json`, `metadata.json`, base summaries, run URL and raw
profiles as evidence. Keep a missing/failed baseline distinct from a regression.

## Establish a valid CI harness

Reuse a valid existing harness and workflow without changing them when possible. If either is missing or invalid, read [the Gungraun CI reference](references/gungraun-ci.md) and create a temporary harness branch unless the user explicitly requests a permanent harness.

- Prefer the smallest benchmark that covers the requested behavior while exercising realistic inputs.
- Keep setup and teardown outside the measured function and protect inputs and results with `std::hint::black_box`.
- Prefer a separate workspace member for library benchmarks when feasible. Choose a binary benchmark only when the executable boundary is part of the behavior under test.
- Match `gungraun-runner` to the library dependency. Use a Linux Actions runner with Valgrind.
- Save a base baseline, benchmark head against it, save machine-readable summaries, and retain profiles and flamegraphs as artifacts.
- Ensure base and head execute the same benchmark definition, toolchain, features, and environment. Never compare different workloads under the same benchmark name.
- If the base revision lacks the harness, bootstrap only benchmark plumbing that can exercise unchanged base code. If that is unsafe or the required API does not exist on base, treat the workload as non-comparable and stop.

When temporary CI is required, use separate PR-free measurement branches. Read the [temporary harness procedure](references/gungraun-ci.md#temporary-harness-branches). Never create pull requests for them. Publish only accepted optimization commits on branches based on trunk or, for dependent optimizations, on their dependency. Record the mapping between measured and published SHAs, and verify that their production diffs match before publication.

## Run the optimization loop

### 1. Analyze the current result

- Scope subsequent runs with Gungraun's benchmark filter when the workflow permits it.
- Rank target cases by the requested metric. Start with `Ir` unless another metric was requested.
- Inspect regular and differential flamegraphs or annotate the retained Callgrind data. Find functions with high self cost, unexpectedly repeated work, avoidable allocations, poor locality, expensive conversions, or unnecessary library calls.
- Before creating or measuring a provisional attempt, record and freeze the comparable target portfolio, related cases, user-designated or workload-critical protected cases, and case frequencies. Use trustworthy documented production frequencies when available; otherwise assign and disclose an equal frequency of one to every case. Do not change these classifications or weights after seeing provisional results without disclosing the reason and obtaining user approval.
- Select one hypothesis that the available profile evidence supports. Do not optimize from intuition alone. Before selecting a helper optimization, locate its original definition and callers across crates. Prefer changing that shared implementation and include affected callers in the related benchmark cases before freezing the comparison portfolio.
- Reject a hypothesis that requires new or expanded architecture-specific production code or build settings unless the user explicitly requested optimization for that named architecture. Do not infer such a request from the CI runner or profile evidence.

### 2. Create one provisional attempt

- Keep a task-owned, PR-free integration branch that starts at the frozen trunk SHA and contains every accepted optimization. Take fresh profiles from it, because the hot path can move after each accepted optimization.
- Create each attempt as a plain branch from the frozen trunk SHA, which is its parent. Use an accepted, unmerged optimization branch as the parent only when the attempt depends on it. If the attempt depends on several accepted optimizations in different pull requests, do not combine them; wait until all but one have merged, or ask the user. When using a temporary harness, create attempts on the measurement branches.
- Read and understand the affected path before editing it. Reject the hypothesis immediately if it changes the protected AST representation. Do not reject a helper or implementation change solely because it is in an AST crate.
- Preserve semantics and public compatibility unless the user explicitly allows a non-AST change.
- Keep the patch small enough that its benchmark effect is attributable to one idea.
- Run narrow correctness tests first, then the broader relevant checks required by the repository. For shared helper changes, check the impact on other callers, run their relevant correctness checks, and apply the existing Gungraun comparison and acceptance criteria to the target and related cases.
- Stage only intended files and verify that the protected AST representation is unchanged and no shared helper was duplicated or shadowed. Inspect both the staged diff and each affected function or module's enclosing configuration, inherited `cfg` or `target_feature` attributes, and relevant existing Cargo configuration, build scripts, and compiler flags for `target_arch`, `CARGO_CFG_TARGET_ARCH`, `std::arch`, `core::arch`, runtime feature-detection macros, assembly, architecture-specific flags, or equivalents. Treat a change inside an existing architecture-gated scope as expanding that implementation. Accept it only under the explicit named-architecture exception, then create one provisional optimization commit.

### 3. Push and measure without a pull request

- Push only the task-owned attempt and integration branches with `git push <remote> <branch>`. Do not create a pull request for a provisional attempt or a temporary harness.
- Dispatch `gungraun.yml` with exact `base_ref` and `head_ref` SHAs and the frozen `filter`. Run a standalone comparison of the attempt against its parent, a marginal comparison of the integration branch plus the attempt against the integration branch, and a cumulative comparison of the integration branch plus the attempt against the frozen starting trunk revision. Reuse one run for two comparisons only when both their base and head SHAs are identical. Use the same harness for all comparisons, including harness-only counterparts when temporary plumbing is required.
- Follow the [manual dispatch procedure](references/gungraun-ci.md#base-and-head-comparison). Wait only for matching task-owned workflow runs. Match event, dispatch revision, run ID, and artifact comparison SHAs and filter; do not use the latest run without checking its identity. Ignore cancelled or superseded runs.
- Download or inspect the saved summary and profile artifacts. Treat new, removed, renamed, or failed benchmark cases as non-comparable until explained.
- Record the run URL, base/compare SHAs, benchmark identities, exact metrics, summary schema version, and artifact names.

### 4. Compare and decide

- Compare the attempt's standalone effect against its parent, its marginal effect against the integration branch, and the cumulative effect of all accepted optimizations plus the attempt against the frozen starting trunk revision. The standalone comparison supports the published PR. The marginal comparison detects an attempt that interacts badly with accepted optimizations. Treat both as incremental comparisons: every threshold below that names incremental results applies to each of them. Require all three comparisons to satisfy the applicable portfolio and per-case thresholds below; never use earlier wins to justify a newly harmful attempt or let individually small regressions evade the cumulative thresholds.
- If an attempt based on trunk fails the incremental thresholds but profile evidence shows that it depends on an accepted optimization, rebase it onto that optimization's branch and measure it again before you classify it as dependent.
- Compute the primary target portfolio score as `sum(frequency * head) / sum(frequency * base)`, equivalently the arithmetic mean of `head / base` ratios weighted by `frequency * base`. Express portfolio improvement as `1 - score`. Use the equal-weight geometric mean of case ratios only as a secondary suite statistic.
- Also inspect the frequency-weighted absolute metric delta, the count and distribution of improvements and regressions, the largest individual regression, and profile evidence.
- Require correctness checks to pass. Accept automatically when every incremental and cumulative primary target portfolio score improves by at least 1% and every incremental and cumulative individual target and related regression is below 1%.
- Also accept an isolated non-protected regression of at least 1% but below 3% when every incremental and cumulative primary target portfolio score improves by at least 5%, no incremental or cumulative regression reaches 3%, the regression is not repeated across a related workload family, and profile evidence explains the tradeoff.
- Do not automatically accept when any comparison has a protected-case regression of 1% or more or any individual regression of 3% or more. Keep the provisional attempt, report the exact tradeoff, and ask the user to decide.
- Apply the same portfolio reasoning to related cases. Do not discard a large target improvement solely because a non-representative case outside the target portfolio has a sub-1% regression.
- Reject an attempt when any incremental primary target portfolio worsens or when small regressions are broad enough to make the overall effect negative. Accept an improvement below 1% only when the code is equally simple or simpler, every incremental and cumulative primary portfolio score still improves, and no incremental or cumulative individual regression reaches 1%.
- Re-run the matching CI comparison once before deciding when a result is near a threshold or the aggregate, individual cases, and profile evidence conflict.
- Report an improvement of 5% or more as an intermediate result with exact base/head counts, percentages, the run link, and the profile evidence before continuing.
- For every accepted tradeoff, report the standalone, marginal, and cumulative primary portfolio scores, the secondary geometric means, frequency-weighted absolute metric changes, individual regressions, protected-case status, and profile evidence.
- Immediately after acceptance, before creating the next attempt, publish the accepted change as a draft PR:
  - For an independent optimization, push its single-commit branch based on trunk and run `gh pr create --draft --base <trunk> --head <accepted-branch>`.
  - For a dependent optimization, run `gh stack add <accepted-branch>` when the dependency is already the top of a task-owned stack. Otherwise adopt the dependency and the new branch with `gh stack init --base <trunk> <dependency-branch> <accepted-branch>`. Then run `gh stack submit --auto`, which reuses the existing dependency PR and creates a draft PR for the new layer.
  - If temporary plumbing was used, first transfer only the accepted optimization commit to the publication branch and verify its production diff and correctness there. Never publish provisional, rejected, or harness commits.
- Cherry-pick the accepted commit onto the integration branch. Include the comparison run links, exact metrics, and measured/published SHA mappings in the PR description; for a stacked PR, also name the dependency PR and the reason for the dependency. Attach the created PR to the chat. Keep each accepted branch unchanged.
- After an attempt is conclusively rejected, delete only its task-owned local and remote experiment branches with `git branch -D <attempt-branch>` and `git push <remote> --delete <attempt-branch>`. There is no rejected PR to close. Do not drop an ambiguous tradeoff while awaiting the user's decision, and do not retain a revert commit in an accepted pull request.
- After two failed hypotheses for the same hot path, move to the next profile-supported target without treating the failures as a reason to finish.

Repeat with fresh profiles from the integration branch because the hot path can move after each accepted optimization.

## Handle failures explicitly

- Fix deterministic build, benchmark, summary parsing, or artifact upload failures before interpreting performance.
- Retry a clearly transient Actions failure once. If it repeats, report the evidence and ask for help.
- Treat missing GitHub permissions, unavailable required runners, and invalid base/head workloads as blockers to measurement.
- For fork pull requests, accept that write-token PR comments may be unavailable; use the read-only job summary and artifacts instead.
- Never use `pull_request_target` to run untrusted pull request code.

## Remove temporary CI before finishing

If this task created temporary CI:

1. Verify each published branch has exactly one optimization commit relative to its parent. Verify that the diff of every published branch against trunk contains none of the recorded temporary files or manifest hunks and no change to the protected AST representation or duplicated or shadowed shared helper. No temporary harness commit may be reachable from a published branch.
2. Preserve CI run URLs, artifacts, and the mapping from measured SHAs to published commits.
3. Delete the task-owned local and remote measurement, harness, integration, and rejected attempt branches. Do not delete accepted publication branches. No harness or rejected PR should have been created.
4. Verify that each independent PR targets trunk. For each dependent stack, run `gh stack sync --prune` and verify that its bottom PR targets trunk and each later PR targets its dependency. Run normal correctness CI on every published PR.

Do not finish while temporary remote branches remain or any temporary CI change or commit is included in an accepted PR.

## Finish and report

Finish only when the user is satisfied, no safe profile-supported bottleneck remains, measurement or permissions are blocked, correctness cannot be preserved, or further work requires an unapproved architectural change. Do not finish because an improvement is small, a numeric performance target was reached or missed, several attempts failed, or returns appear to diminish.

Report:

- every accepted optimization, its single-commit pull request, and why it affects the measured hot path;
- exact before/after metrics and percentages from the associated Actions runs;
- benchmark names, base/head SHAs, run links, and relevant profile or flamegraph evidence;
- correctness checks and any existing wall-time corroboration;
- for explicitly requested architecture-specific work, the named and measured architecture, matching-architecture Actions run and exact base/head metrics, portable fallback, target-scoped claims, cross-target checks, and validation gaps;
- rejected attempts, remaining tradeoffs, and promising next targets;
- which PRs are independent, the order of each dependent stack with the reason for each dependency, and proof that temporary CI changes, commits, and branches were cleaned up and no temporary PR was created.
