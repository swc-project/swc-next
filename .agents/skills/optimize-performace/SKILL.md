---
name: optimize-performace
description: "Run only when explicitly requested. Optimize Rust code and the Node.js API using paired wall-time measurements in GitHub Actions. Measure provisional changes without PRs, publish each accepted optimization in its own draft PR, and stack only dependent changes. Use for execution-time optimization, regression investigation, and temporary benchmark harnesses."
---

# Optimize Performace

Act as a performance engineer. Measure provisional changes in GitHub Actions without creating pull requests, compare paired wall times, immediately publish each accepted optimization as one commit in its own draft pull request, and continue while a safe evidence-supported hypothesis remains.

## Obey the non-negotiable boundaries

- Publish every independent accepted optimization as its own pull request based directly on the repository trunk, so that it can be reviewed and merged separately. Do not stack independent optimizations.
- Stack an accepted optimization only when it depends on another unmerged accepted optimization from this task. A dependency exists when the optimization does not apply cleanly to trunk, does not compile or stay correct without the other change, or meets the acceptance thresholds only on top of the other change. Shared files, a shared hot path, or the order of discovery alone are not dependencies. When the dependency has merged, base the optimization on trunk instead.
- Use `gh stack` for all creation, layer addition, pushes, submission, synchronization, rebasing, and restructuring of dependent stacks. Use `gh pr create --draft --base <trunk>` only for independent optimizations.
- Do not inspect, adopt, reuse, modify, rebase, wait for, or otherwise interact with unrelated pull requests or their branches.
- Keep exactly one focused optimization commit in every accepted branch relative to its parent: trunk for an independent optimization or the dependency branch for a stacked layer. Keep exactly one accepted optimization in the corresponding pull request diff.
- Never alter AST types, fields, variants, layout, schema, or serialization shape, including through generators or generated AST output. Apply this restriction to the content of the change, regardless of its path. Shared helpers and other implementations may be optimized across crates, including inside `crates/swc_ecma_ast/` and `crates/swc_ast_macros/`, when they preserve the AST representation.
- Prefer optimizing an existing shared helper at its original definition. Do not copy its functionality into a caller crate or shadow it with a local implementation to optimize it or bypass the AST restriction. For example, optimize `is_ident_continue_ascii` in `crates/swc_ecma_ast/src/syntax.rs` directly instead of duplicating it in codegen.
- Default to target-independent production changes. Unless the user explicitly requests optimization for a named architecture, do not introduce or expand architecture-specific code paths or build configuration, including `cfg(target_arch = ...)`, `cfg!(target_arch = ...)`, `CARGO_CFG_TARGET_ARCH`, `std::arch` or `core::arch` intrinsics, inline assembly, `#[target_feature]`, runtime CPU-feature detection, or architecture-specific `RUSTFLAGS`, `target-cpu`, or `target-feature` settings.
- Treat the Linux x86_64 paired wall-time runner as a measurement environment only. Never use its architecture, profiles, or an existing architecture-specific implementation as permission to specialize production code for x86_64.
- When the user explicitly requests a named-architecture optimization, first confirm that an Actions runner and a valid paired wall-time harness are available on that architecture. Require a GitHub Actions base/head comparison on that architecture which executes the optimized path; if it cannot be established, stop before editing and report the measurement blocker. Scope every performance claim to the measured architecture, retain a correct portable fallback and compatibility with supported targets, run available cross-target correctness checks, and report every target that was not validated.
- Check staged and committed path lists and inspect the staged diff before every publication. Abandon any hypothesis that changes the protected AST representation; do not request an exception. Unless the user explicitly requested a named architecture, also abandon any hypothesis that violates the architecture-portability guard instead of asking for permission to retain it.
- Never push the default branch.

## Enforce the measurement contract

- Use the paired wall-time results of `.github/workflows/profile-perf.yml` with exact base/head SHAs as the source of truth for execution-time comparisons. Both revisions must execute the evaluated path on the same runner in the same job.
- Use local runs only to check correctness or that a harness starts successfully. Never support a performance claim with local benchmark output.
- Default to elapsed time. Gungraun instruction counts, simulated cache metrics, estimated cycles, and allocation profiles can explain a hypothesis, but cannot establish a speedup or override a wall-time regression. By default, the same run also uploads Gungraun instruction counts and Callgrind data from the same build as diagnostic evidence; dispatch with `suite=walltime` when you do not need them. Treat a Gungraun failure as diagnostic only when the wall-time results are complete and valid.
- Read all measurements in `paired-walltime/results.json`, including cases omitted from `summary.md`. The summary reports a change only when every round agrees on its direction and the median paired change is at least 2%. Its `unchanged` label does not prove equivalence or absence of regressions.
- Keep new CI reporting-only for performance changes. Do not add regression limits that fail CI. Preserve existing repository limits unless the user authorizes changing them.
- Stop and ask for help instead of substituting uncontrolled measurements when GitHub authentication, permissions, runner availability, or a valid base/head comparison cannot be established.

## Inspect before changing anything

1. Identify the code and benchmark cases to optimize. Ask only if neither the request nor the repository reveals a usable target.
2. Identify the requested workload and measurement boundary. Use elapsed time when the request says "faster" or "optimize."
3. Read repository instructions and inspect only the worktree, trunk, remotes, branches and stacks owned by this task, and existing user changes. Preserve unrelated work without investigating unrelated pull requests.
4. Search manifests, Criterion benchmark targets, GitHub workflows, the paired wall-time script, and relevant profiling tools. Prefer `rg` and `git grep`.
5. Confirm GitHub authentication, runner access, the workflow trigger, and the installed `gh stack` commands. Never substitute another stacked-pull-request tool for dependent stacks.
6. Freeze the starting trunk SHA for independent attempts and cumulative comparisons. Record the benchmark identities, base revision, workflow/script revision, Rust toolchain, Criterion version, rounds, warm-up and measurement durations, build features, runner image, CPU and architecture. Keep the workload and settings invariant. Inspect the recorded runner environment on each run; hosted runners can use different CPUs.

## Establish a valid CI harness

Read [the paired wall-time CI reference](references/paired-walltime-ci.md) before dispatching or changing the harness. Reuse the existing workflow and harness when they cover the target. Its wall-time suite currently measures the Rust `parser`, `visit`, `semantic`, `type_strip`, and `minifier` Criterion targets; `targets` selects the wall-time targets and defaults to `affected` (`filter` applies only to Gungraun) and does not measure the public JavaScript API or every compiler pipeline.

- Check that the frozen target and related cases exercise the changed code. For uncovered code, add the smallest representative workload on temporary PR-free measurement branches. Extend the paired workflow/script only as needed; do not substitute Gungraun counts for elapsed time.
- Keep setup and teardown outside the measured operation unless they are part of the requested boundary. Protect inputs and results with `std::hint::black_box`.
- Ensure both revisions use identical benchmark definitions, fixtures, toolchain, features, and timing settings. The workflow preserves its driver script, but builds benchmark source from each revision; equal case names alone do not establish comparable work.
- If base lacks the harness, bootstrap only benchmark plumbing that exercises unchanged base code. If the required API does not exist on base, stop and report the non-comparable workload.
- Retain raw per-round times, the summary, run URL, and environment. Keep optional profile evidence separate from acceptance measurements.

For `@swc-next/core`, measure the public N-API entry point with the same fixtures, Node version, Node flags, release settings, warm-up policy, and operation boundary on base and head. Use representative JIT-enabled execution by default. Measure lazy parse return, full AST materialization, print encoding, or path helpers according to the requested operation. A Rust-only comparison does not measure JS encoding, lazy decoding, or FFI. The existing `gungraun_js` harness with `--jitless --predictable` is diagnostic only. Establish a paired elapsed-time JS harness in Actions before accepting a JS API optimization; report a measurement blocker if this is unavailable.

When temporary CI is required, follow the [temporary harness procedure](references/paired-walltime-ci.md#temporary-harness-branches). Never create pull requests for measurement branches. Publish only accepted optimization commits on branches based on trunk or their dependency. Record measured/published SHA mappings and verify matching production diffs before publication.

## Run the optimization loop

### 1. Analyze the current result

- Run the full existing paired suite and analyze the frozen target and related cases. Do not invent a wall-time filter input.
- Rank cases by frequency-weighted elapsed time. Inspect the measured paths for repeated work, avoidable allocations, poor locality, expensive conversions, or unnecessary calls. Use available profiles to locate costs when useful.
- Before creating or measuring a provisional attempt, record and freeze the comparable target portfolio, related cases, user-designated or workload-critical protected cases, and case frequencies. Use trustworthy documented production frequencies when available; otherwise assign and disclose an equal frequency of one to every case. Do not change these classifications or weights after seeing provisional results without disclosing the reason and obtaining user approval.
- Select one hypothesis supported by the measured workload and code inspection or available profiles. Before selecting a helper optimization, locate its original definition and callers across crates. Prefer changing that shared implementation and include affected callers in the related benchmark cases before freezing the comparison portfolio.
- Reject a hypothesis that requires new or expanded architecture-specific production code or build settings unless the user explicitly requested optimization for that named architecture. Do not infer such a request from the CI runner or profile evidence.

### 2. Create one provisional attempt

- Keep a task-owned, PR-free integration branch that starts at the frozen trunk SHA and contains every accepted optimization. Take fresh timing measurements and, when useful, profiles from it, because the hot path can move after each accepted optimization.
- Create each attempt as a plain branch from the frozen trunk SHA, which is its parent. Use an accepted, unmerged optimization branch as the parent only when the attempt depends on it. If the attempt depends on several accepted optimizations in different pull requests, do not combine them; wait until all but one have merged, or ask the user. When using a temporary harness, create attempts on the measurement branches.
- Read and understand the affected path before editing it. Reject the hypothesis immediately if it changes the protected AST representation. Do not reject a helper or implementation change solely because it is in an AST crate.
- Preserve semantics and public compatibility unless the user explicitly allows a non-AST change.
- Keep the patch small enough that its benchmark effect is attributable to one idea.
- Run narrow correctness tests first, then the broader relevant checks required by the repository. For shared helper changes, check the impact on other callers, run their relevant correctness checks, and apply the paired wall-time comparison and acceptance criteria to the target and related cases.
- Stage only intended files and verify that the protected AST representation is unchanged and no shared helper was duplicated or shadowed. Inspect both the staged diff and each affected function or module's enclosing configuration, inherited `cfg` or `target_feature` attributes, and relevant existing Cargo configuration, build scripts, and compiler flags for `target_arch`, `CARGO_CFG_TARGET_ARCH`, `std::arch`, `core::arch`, runtime feature-detection macros, assembly, architecture-specific flags, or equivalents. Treat a change inside an existing architecture-gated scope as expanding that implementation. Accept it only under the explicit named-architecture exception, then create one provisional optimization commit.

### 3. Push and measure without a pull request

- Push only the task-owned attempt and integration branches with `git push <remote> <branch>`. Do not create a pull request for a provisional attempt or a temporary harness.
- Dispatch `profile-perf.yml` with exact `base` and `head` SHAs, the frozen positive even `rounds` count (default 6), and `suite=all` or `suite=walltime`. Run a standalone comparison of the attempt against its parent, a marginal comparison of the integration branch plus the attempt against the integration branch, and a cumulative comparison of the integration branch plus the attempt against the frozen starting trunk revision. Reuse one run for two comparisons only when both their base and head SHAs are identical. Use the same harness for all comparisons, including harness-only counterparts when temporary plumbing is required.
- Follow the [manual dispatch procedure](references/paired-walltime-ci.md#base-and-head-comparison). Wait only for matching task-owned workflow runs. Match event, dispatch revision, run ID, and artifact `base_sha`, `head_sha`, rounds, timing settings, and benchmark identities; do not use the latest run without checking its identity. Ignore cancelled or superseded runs.
- Download or inspect the saved `paired-walltime` artifact (`results.json` and `summary.md`). Treat new, removed, renamed, or failed benchmark cases as non-comparable until explained.
- Record the run URL, base/head SHAs, benchmark identities, exact metrics, summary schema version, and artifact names.

### 4. Compare and decide

- Compare the attempt's standalone effect against its parent, its marginal effect against the integration branch, and the cumulative effect of all accepted optimizations plus the attempt against the frozen starting trunk revision. The standalone comparison supports the published PR. The marginal comparison detects an attempt that interacts badly with accepted optimizations. Treat both as incremental comparisons: every threshold below that names incremental results applies to each of them. Require all three comparisons to satisfy the applicable portfolio and per-case thresholds below; never use earlier wins to justify a newly harmful attempt or let individually small regressions evade the cumulative thresholds.
- If an attempt based on trunk fails the incremental thresholds but timing and code evidence show that it depends on an accepted optimization, rebase it onto that optimization's branch and measure it again before you classify it as dependent.
- For case `i` and round `r`, use `q[i,r] = head[i,r] / base[i,r]`. Report its change as `median(q[i,*]) - 1`, as the renderer does. Also report the separate base/head median times and the range of paired ratios; the ratio of medians is not the paired median ratio.
- Compute the primary target portfolio ratio in each round as `P[r] = sum(frequency[i] * head[i,r]) / sum(frequency[i] * base[i,r])`. Use `median(P)` as the score and `1 - median(P)` as improvement. Also report the median per-round frequency-weighted absolute time delta, the equal-weight geometric mean of per-case paired median ratios as a secondary statistic, and the distribution of individual changes. Never mix absolute times from different Actions jobs.
- Require correctness checks to pass. Accept automatically when every incremental and cumulative primary portfolio improves in every round, its median improvement is at least 2%, and every individual target and related median regression is below 1%.
- Also accept an isolated non-protected median regression of at least 1% but below 3% when every incremental and cumulative portfolio improves in every round by a median of at least 5%, no individual median regression reaches 3%, the regression is not repeated across a related workload family, and code or profile evidence explains the tradeoff.
- Do not automatically accept a protected-case median regression of 1% or more or any individual median regression of 3% or more. Resolve measurement noise first; for a repeatable tradeoff, keep the attempt, report it, and ask the user to decide.
- Apply the same portfolio reasoning to related cases. Inspect their raw times even when the summary omits them. Do not discard a large target improvement solely because a non-representative case outside the target portfolio has a sub-1% regression.
- Reject a repeatable incremental portfolio slowdown or broad regressions that make the overall effect negative. An improvement below 2% is eligible only when the code is equally simple or simpler, a matching repeat run confirms improvement in every round of each incremental and cumulative comparison, and no individual median regression reaches 1% in either run. Report it as a small measured effect below the summary's reporting threshold.
- Re-run each affected exact-SHA comparison once when rounds disagree, a result is near a decision threshold, or timing and diagnostic evidence conflict. Retain both runs and their environments. Do not select favorable rounds, drop outliers after seeing results, pool times from different runners, or treat the 2% reporting rule as a confidence interval. If the repeat remains inconclusive, do not accept the attempt; report the uncertainty and move to another supported hypothesis or finish with the measurement limitation.
- Report an improvement of 5% or more as an intermediate result with exact base/head times, paired percentages and ranges, the run link, and the supporting code or profile evidence before continuing.
- For every accepted tradeoff, report standalone, marginal, and cumulative portfolio scores, secondary geometric means, frequency-weighted absolute time changes, individual regressions, protected-case status, round consistency, and repeat-run evidence when required.
- Immediately after acceptance, before creating the next attempt, publish the accepted change as a draft PR:
  - For an independent optimization, push its single-commit branch based on trunk and run `gh pr create --draft --base <trunk> --head <accepted-branch>`.
  - For a dependent optimization, run `gh stack add <accepted-branch>` when the dependency is already the top of a task-owned stack. Otherwise adopt the dependency and the new branch with `gh stack init --base <trunk> <dependency-branch> <accepted-branch>`. Then run `gh stack submit --auto`, which reuses the existing dependency PR and creates a draft PR for the new layer.
  - If temporary plumbing was used, first transfer only the accepted optimization commit to the publication branch and verify its production diff and correctness there. Never publish provisional, rejected, or harness commits.
- Cherry-pick the accepted commit onto the integration branch. Include the comparison run links, exact metrics, and measured/published SHA mappings in the PR description; for a stacked PR, also name the dependency PR and the reason for the dependency. Attach the created PR to the chat. Keep each accepted branch unchanged.
- After an attempt is conclusively rejected, delete only its task-owned local and remote experiment branches with `git branch -D <attempt-branch>` and `git push <remote> --delete <attempt-branch>`. There is no rejected PR to close. Do not drop an ambiguous tradeoff while awaiting the user's decision, and do not retain a revert commit in an accepted pull request.
- After two failed hypotheses for the same hot path, move to the next evidence-supported target without treating the failures as a reason to finish.

Repeat with fresh timing measurements and optional profiles from the integration branch because the hot path can move after each accepted optimization.

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

Finish only when the user is satisfied, no safe evidence-supported bottleneck remains, measurement or permissions are blocked, correctness cannot be preserved, or further work requires an unapproved architectural change. Do not finish because an improvement is small, a numeric performance target was reached or missed, several attempts failed, or returns appear to diminish.

Report:

- every accepted optimization, its single-commit pull request, and why it affects the measured hot path;
- exact before/after times, paired percentages, round ranges, and environments from the associated Actions runs;
- benchmark names, base/head SHAs, run links, and supporting code analysis and any optional profile or flamegraph evidence;
- correctness checks and any required repeat wall-time comparisons;
- for explicitly requested architecture-specific work, the named and measured architecture, matching-architecture Actions run and exact base/head metrics, portable fallback, target-scoped claims, cross-target checks, and validation gaps;
- rejected attempts, remaining tradeoffs, and promising next targets;
- which PRs are independent, the order of each dependent stack with the reason for each dependency, and proof that temporary CI changes, commits, and branches were cleaned up and no temporary PR was created.
