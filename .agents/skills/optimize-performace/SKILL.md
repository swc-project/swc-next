---
name: optimize-performace
description: "Run only when explicitly requested. Optimize Rust code and the Node.js API using CodSpeed wall-time measurements on the x86_64 and arm64 Macro Runners. Measure provisional changes without PRs, publish each accepted optimization in its own draft PR, and stack only dependent changes. Use for execution-time optimization, regression investigation, and temporary benchmark harnesses."
---

# Optimize Performace

Act as a performance engineer. Measure provisional changes with CodSpeed wall-time runs on both architectures without creating pull requests, compare the runs, immediately publish each accepted optimization as one commit in its own draft pull request, and continue while a safe evidence-supported hypothesis remains.

## Obey the non-negotiable boundaries

- Publish every independent accepted optimization as its own pull request based directly on the repository trunk, so that it can be reviewed and merged separately. Do not stack independent optimizations.
- Stack an accepted optimization only when it depends on another unmerged accepted optimization from this task. A dependency exists when the optimization does not apply cleanly to trunk, does not compile or stay correct without the other change, or meets the acceptance thresholds only on top of the other change. Shared files, a shared hot path, or the order of discovery alone are not dependencies. When the dependency has merged, base the optimization on trunk instead.
- Use `gh stack` for all creation, layer addition, pushes, submission, synchronization, rebasing, and restructuring of dependent stacks. Use `gh pr create --draft --base <trunk>` only for independent optimizations.
- Do not inspect, adopt, reuse, modify, rebase, wait for, or otherwise interact with unrelated pull requests or their branches.
- Keep exactly one focused optimization commit in every accepted branch relative to its parent: trunk for an independent optimization or the dependency branch for a stacked layer. Keep exactly one accepted optimization in the corresponding pull request diff.
- Never alter AST types, fields, variants, layout, schema, or serialization shape, including through generators or generated AST output. Apply this restriction to the content of the change, regardless of its path. Shared helpers and other implementations may be optimized across crates, including inside `crates/swc_ecma_ast/` and `crates/swc_ast_macros/`, when they preserve the AST representation.
- Prefer optimizing an existing shared helper at its original definition. Do not copy its functionality into a caller crate or shadow it with a local implementation to optimize it or bypass the AST restriction. For example, optimize `is_ident_continue_ascii` in `crates/swc_ecma_ast/src/syntax.rs` directly instead of duplicating it in codegen.
- Default to target-independent production changes. Unless the user explicitly requests optimization for a named architecture, do not introduce or expand architecture-specific code paths or build configuration, including `cfg(target_arch = ...)`, `cfg!(target_arch = ...)`, `CARGO_CFG_TARGET_ARCH`, `std::arch` or `core::arch` intrinsics, inline assembly, `#[target_feature]`, runtime CPU-feature detection, or architecture-specific `RUSTFLAGS`, `target-cpu`, or `target-feature` settings.
- Measure on both architectures. CodSpeed runs every benchmark on an x86_64 runner and on an arm64 runner. Never use the result of one architecture as permission to specialize production code for it, and never let a win on one architecture hide a regression on the other.
- When the user explicitly requests a named-architecture optimization, first confirm that the CodSpeed run of that architecture executes the optimized path. If it does not, stop before editing and report the measurement blocker. Scope every performance claim to the measured architecture, retain a correct portable fallback and compatibility with supported targets, run available cross-target correctness checks, and report every target that was not validated.
- Check staged and committed path lists and inspect the staged diff before every publication. Abandon any hypothesis that changes the protected AST representation; do not request an exception. Unless the user explicitly requested a named architecture, also abandon any hypothesis that violates the architecture-portability guard instead of asking for permission to retain it.
- Never push the default branch.

## Enforce the measurement contract

- Use CodSpeed wall-time runs of `.github/workflows/codspeed.yml` as the source of truth for execution-time comparisons. Read every run through the CodSpeed MCP tools or the CLI; never support a performance claim with local `cargo bench` output or any benchmark run outside CodSpeed. Use local runs only to check correctness or that a harness starts successfully.
- Compare only runs from the same architecture, the same `wall-time` mode, the same benchmark identities, and the same hardware. CodSpeed reports hardware differences in `compare_runs`; treat a reported difference as a non-comparable pair until a repeat run removes it. Never compare an x86_64 run with an arm64 run.
- A CodSpeed run measures each revision in a separate job, so unlike a paired in-job harness it does not share runner noise. Compensate with repeat runs (see the decision rules) and do not treat one pair as conclusive.
- Default to elapsed time. Instruction counts from the Profile Performance workflow (Gungraun), CodSpeed flamegraphs, and allocation profiles can explain a hypothesis, but cannot establish a speedup or override a wall-time regression.
- Keep new CI reporting-only for performance changes. Do not add regression limits that fail CI. Preserve existing repository limits unless the user authorizes changing them.
- Stop and ask for help instead of substituting uncontrolled measurements when GitHub authentication, CodSpeed authentication, runner availability, or a valid base/head comparison cannot be established.

## Inspect before changing anything

1. Identify the code and benchmark cases to optimize. Ask only if neither the request nor the repository reveals a usable target.
2. Identify the requested workload and measurement boundary. Use elapsed time when the request says "faster" or "optimize."
3. Read repository instructions and inspect only the worktree, trunk, remotes, branches and stacks owned by this task, and existing user changes. Preserve unrelated work without investigating unrelated pull requests.
4. Search manifests, Criterion benchmark targets, the CodSpeed workflow, and relevant profiling tools. Prefer `rg` and `git grep`.
5. Confirm GitHub authentication, the CodSpeed MCP server or CLI login, the `workflow_dispatch` trigger of `codspeed.yml` on trunk, and the installed `gh stack` commands. Never substitute another stacked-pull-request tool for dependent stacks.
6. Freeze the starting trunk SHA for independent attempts and cumulative comparisons. Record the benchmark identities, base revision, workflow revision, Rust toolchain, Criterion version, build features, and for each run the architecture and CPU that the CodSpeed report shows. Keep the workload and settings invariant.

## Establish a valid CodSpeed harness

Read [the CodSpeed CI reference](references/codspeed-ci.md) before dispatching or changing the harness. Reuse the existing workflow when it covers the target. It currently builds and measures the `parser`, `visit`, and `semantic` Criterion targets of the `benchmark` package on both architectures. The `type_strip` and `minifier` targets and the public JavaScript API are not measured.

- Check that the frozen target and related cases exercise the changed code. For uncovered code, add the smallest representative workload on temporary PR-free measurement branches. For a Criterion target that the workflow does not list, add it to the `--bench` lists of `codspeed.yml` there.
- Keep setup and teardown outside the measured operation unless they are part of the requested boundary. Protect inputs and results with `std::hint::black_box`.
- Ensure both revisions use identical benchmark definitions, fixtures, toolchain, features, and CodSpeed settings. Equal case names alone do not establish comparable work.
- If base lacks the harness, bootstrap only benchmark plumbing that exercises unchanged base code. If the required API does not exist on base, stop and report the non-comparable workload.
- Retain the CodSpeed run IDs and URLs, the per-benchmark results, and the environment of every run used for a decision. Keep optional profile evidence separate from acceptance measurements.

For `@swc-next/core`, measure the public N-API entry point with the same fixtures, Node version, Node flags, release settings, warm-up policy, and operation boundary on base and head. Use representative JIT-enabled execution by default. Measure lazy parse return, full AST materialization, print encoding, or path helpers according to the requested operation. A Rust-only comparison does not measure JS encoding, lazy decoding, or FFI. The existing `gungraun_js` harness with `--jitless --predictable` is diagnostic only. Establish a JS wall-time harness that CodSpeed runs on both architectures before accepting a JS API optimization; report a measurement blocker if this is unavailable.

When temporary CI is required, follow the [temporary harness procedure](references/codspeed-ci.md#temporary-harness-branches). Never create pull requests for measurement branches. Publish only accepted optimization commits on branches based on trunk or their dependency. Record measured/published SHA mappings and verify matching production diffs before publication.

## Run the optimization loop

### 1. Analyze the current result

- Run the CodSpeed workflow on the current integration revision and analyze the frozen target and related cases on each architecture.
- Rank cases by frequency-weighted elapsed time. Query flamegraphs of the hottest cases with `query_flamegraph`, starting from the whole graph and then re-rooting into the hottest subtrees, to find repeated work, avoidable allocations, poor locality, expensive conversions, or unnecessary calls. Inspect the measured paths in the code.
- Before creating or measuring a provisional attempt, record and freeze the comparable target portfolio, related cases, user-designated or workload-critical protected cases, and case frequencies. Use trustworthy documented production frequencies when available; otherwise assign and disclose an equal frequency of one to every case. Do not change these classifications or weights after seeing provisional results without disclosing the reason and obtaining user approval.
- Select one hypothesis supported by the measured workload and code inspection or available profiles. Before selecting a helper optimization, locate its original definition and callers across crates. Prefer changing that shared implementation and include affected callers in the related benchmark cases before freezing the comparison portfolio.
- Reject a hypothesis that requires new or expanded architecture-specific production code or build settings unless the user explicitly requested optimization for that named architecture. Do not infer such a request from a profile or from the result of one runner.

### 2. Create one provisional attempt

- Keep a task-owned, PR-free integration branch that starts at the frozen trunk SHA and contains every accepted optimization. Take fresh measurements and, when useful, profiles from it, because the hot path can move after each accepted optimization.
- Create each attempt as a plain branch from the frozen trunk SHA, which is its parent. Use an accepted, unmerged optimization branch as the parent only when the attempt depends on it. If the attempt depends on several accepted optimizations in different pull requests, do not combine them; wait until all but one have merged, or ask the user. When using a temporary harness, create attempts on the measurement branches.
- Read and understand the affected path before editing it. Reject the hypothesis immediately if it changes the protected AST representation. Do not reject a helper or implementation change solely because it is in an AST crate.
- Preserve semantics and public compatibility unless the user explicitly allows a non-AST change.
- Keep the patch small enough that its benchmark effect is attributable to one idea.
- Run narrow correctness tests first, then the broader relevant checks required by the repository. For shared helper changes, check the impact on other callers, run their relevant correctness checks, and apply the CodSpeed comparison and acceptance criteria to the target and related cases.
- Stage only intended files and verify that the protected AST representation is unchanged and no shared helper was duplicated or shadowed. Inspect both the staged diff and each affected function or module's enclosing configuration, inherited `cfg` or `target_feature` attributes, and relevant existing Cargo configuration, build scripts, and compiler flags for `target_arch`, `CARGO_CFG_TARGET_ARCH`, `std::arch`, `core::arch`, runtime feature-detection macros, assembly, architecture-specific flags, or equivalents. Treat a change inside an existing architecture-gated scope as expanding that implementation. Accept it only under the explicit named-architecture exception, then create one provisional optimization commit.

### 3. Push and measure without a pull request

- Push only the task-owned attempt and integration branches with `git push <remote> <branch>`. Do not create a pull request for a provisional attempt or a temporary harness.
- Dispatch `codspeed.yml` on each branch whose revision you need to measure. Run a standalone comparison of the attempt against its parent, a marginal comparison of the integration branch plus the attempt against the integration branch, and a cumulative comparison of the integration branch plus the attempt against the frozen starting trunk revision. Reuse one run for two comparisons only when the measured SHA is identical. Use the same workflow revision for all comparisons, including harness-only counterparts when temporary plumbing is required.
- Follow the [dispatch procedure](references/codspeed-ci.md#measure-and-compare-revisions). Wait only for matching task-owned workflow runs. Match event, branch, and head SHA, and confirm the CodSpeed run that belongs to each workflow run; do not use the latest run without checking its identity. Ignore cancelled or superseded runs.
- Treat new, removed, renamed, or failed benchmark cases as non-comparable until explained.
- Record the workflow run URL, CodSpeed run IDs and URLs, measured SHAs, architecture, benchmark identities, and exact metrics.

### 4. Compare and decide

- Compare, per architecture, the attempt's standalone effect against its parent, its marginal effect against the integration branch, and the cumulative effect of all accepted optimizations plus the attempt against the frozen starting trunk revision. The standalone comparison supports the published PR. The marginal comparison detects an attempt that interacts badly with accepted optimizations. Treat both as incremental comparisons: every threshold below that names incremental results applies to each of them. Require all three comparisons to satisfy the applicable portfolio and per-case thresholds below on both architectures; never use earlier wins to justify a newly harmful attempt or let individually small regressions evade the cumulative thresholds.
- If an attempt based on trunk fails the incremental thresholds but timing and code evidence show that it depends on an accepted optimization, rebase it onto that optimization's branch and measure it again before you classify it as dependent.
- For case `i`, take the base time `base[i]` and head time `head[i]` from the same statistic of the two CodSpeed runs (the one `get_benchmark_result` reports; use the same one throughout) and report the change `head[i] / base[i] - 1`. Report `compare_runs` percentages and CodSpeed's own significance flags next to these values.
- Compute the primary target portfolio ratio `P = sum(frequency[i] * head[i]) / sum(frequency[i] * base[i])`. Use `P` as the score and `1 - P` as improvement. Also report the frequency-weighted absolute time delta, the equal-weight geometric mean of the per-case ratios as a secondary statistic, and the distribution of individual changes. Never mix absolute times from different architectures.
- Require correctness checks to pass. Accept automatically when, on each architecture, every incremental and cumulative primary portfolio improves by at least 2% in two independent measurement pairs, and every individual target and related regression is below 1% in both pairs.
- Also accept an isolated non-protected regression of at least 1% but below 3% when, on each architecture, every incremental and cumulative portfolio improves by at least 5% in two independent measurement pairs, no individual regression reaches 3%, the regression is not repeated across a related workload family, and code or profile evidence explains the tradeoff.
- Do not automatically accept a protected-case regression of 1% or more or any individual regression of 3% or more. Resolve measurement noise first; for a repeatable tradeoff, keep the attempt, report it, and ask the user to decide.
- Apply the same portfolio reasoning to related cases. Inspect their raw results even when `compare_runs` omits them. Do not discard a large target improvement solely because a non-representative case outside the target portfolio has a sub-1% regression.
- Reject a repeatable incremental portfolio slowdown or broad regressions that make the overall effect negative. A portfolio improvement below 2% is eligible only when the code is equally simple or simpler, a third independent pair confirms it, and no individual regression reaches 1% in any pair. Report it as a small measured effect.
- An independent measurement pair is a fresh dispatch of both revisions, not a second reading of the same runs. Run the second pair when the first is a candidate for acceptance, and run another when the two pairs disagree in direction, a result is near a decision threshold, a run reports a hardware difference, or timing and diagnostic evidence conflict. Retain every run and its environment. Do not select favorable runs, drop outliers after seeing results, pool times from different machines, or treat CodSpeed's significance flag as proof. If the repeat remains inconclusive, do not accept the attempt; report the uncertainty and move to another supported hypothesis or finish with the measurement limitation.
- Report an improvement of 5% or more as an intermediate result with exact base/head times, percentages per architecture, the run links, and the supporting code or flamegraph evidence before continuing.
- For every accepted tradeoff, report standalone, marginal, and cumulative portfolio scores, secondary geometric means, frequency-weighted absolute time changes, individual regressions, protected-case status, and the agreement between the independent pairs, on each architecture.
- Immediately after acceptance, before creating the next attempt, publish the accepted change as a draft PR:
  - For an independent optimization, push its single-commit branch based on trunk and run `gh pr create --draft --base <trunk> --head <accepted-branch>`.
  - For a dependent optimization, run `gh stack add <accepted-branch>` when the dependency is already the top of a task-owned stack. Otherwise adopt the dependency and the new branch with `gh stack init --base <trunk> <dependency-branch> <accepted-branch>`. Then run `gh stack submit --auto`, which reuses the existing dependency PR and creates a draft PR for the new layer.
  - If temporary plumbing was used, first transfer only the accepted optimization commit to the publication branch and verify its production diff and correctness there. Never publish provisional, rejected, or harness commits.
- Do not add the `Codspeed` label to a draft PR that this skill creates, because the dispatch runs are the acceptance evidence and the label starts another measurement on every push. Add it only when the user asks.
- Cherry-pick the accepted commit onto the integration branch. Include the comparison run links, exact metrics, and measured/published SHA mappings in the PR description; for a stacked PR, also name the dependency PR and the reason for the dependency. Attach the created PR to the chat. Keep each accepted branch unchanged.
- After an attempt is conclusively rejected, delete only its task-owned local and remote experiment branches with `git branch -D <attempt-branch>` and `git push <remote> --delete <attempt-branch>`. There is no rejected PR to close. Do not drop an ambiguous tradeoff while awaiting the user's decision, and do not retain a revert commit in an accepted pull request.
- After two failed hypotheses for the same hot path, move to the next evidence-supported target without treating the failures as a reason to finish.

Repeat with fresh measurements and optional profiles from the integration branch because the hot path can move after each accepted optimization.

## Handle failures explicitly

- Fix deterministic build, benchmark, or upload failures before interpreting performance.
- Retry a clearly transient Actions failure once. If it repeats, report the evidence and ask for help.
- Treat missing GitHub or CodSpeed permissions, unavailable Macro Runners, a CodSpeed run that does not appear, results of the two architectures that CodSpeed cannot tell apart, and invalid base/head workloads as blockers to measurement.
- For fork pull requests, accept that write-token PR comments may be unavailable.
- Never use `pull_request_target` to run untrusted pull request code.

## Remove temporary CI before finishing

If this task created temporary CI:

1. Verify each published branch has exactly one optimization commit relative to its parent. Verify that the diff of every published branch against trunk contains none of the recorded temporary files or manifest hunks and no change to the protected AST representation or duplicated or shadowed shared helper. No temporary harness commit may be reachable from a published branch.
2. Preserve CodSpeed run URLs and the mapping from measured SHAs to published commits.
3. Delete the task-owned local and remote measurement, harness, integration, and rejected attempt branches. Do not delete accepted publication branches. No harness or rejected PR should have been created.
4. Verify that each independent PR targets trunk. For each dependent stack, run `gh stack sync --prune` and verify that its bottom PR targets trunk and each later PR targets its dependency. Run normal correctness CI on every published PR.

Do not finish while temporary remote branches remain or any temporary CI change or commit is included in an accepted PR.

## Finish and report

Finish only when the user is satisfied, no safe evidence-supported bottleneck remains, measurement or permissions are blocked, correctness cannot be preserved, or further work requires an unapproved architectural change. Do not finish because an improvement is small, a numeric performance target was reached or missed, several attempts failed, or returns appear to diminish.

Report:

- every accepted optimization, its single-commit pull request, and why it affects the measured hot path;
- exact before/after times, percentages, and environments for each architecture, from the associated CodSpeed runs;
- benchmark names, base/head SHAs, run links, and supporting code analysis and any flamegraph or other profile evidence;
- correctness checks and the independent repeat pairs;
- for explicitly requested architecture-specific work, the named and measured architecture, its CodSpeed runs and exact metrics, the portable fallback, target-scoped claims, the result on the other architecture, cross-target checks, and validation gaps;
- rejected attempts, remaining tradeoffs, and promising next targets;
- which PRs are independent, the order of each dependent stack with the reason for each dependency, and proof that temporary CI changes, commits, and branches were cleaned up and no temporary PR was created.
