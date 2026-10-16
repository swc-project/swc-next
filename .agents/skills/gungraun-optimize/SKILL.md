---
name: gungraun-optimize
description: "Don't trigger if not asked explicitly. Autonomously profile and optimize Rust code and the Node.js JavaScript API with Gungraun benchmarks in GitHub Actions using Callgrind, Cachegrind, DHAT, PR-free manual comparisons, and one accepted optimization per stacked pull request. Use when asked to make Rust code faster, reduce instructions, cache misses, or allocations, investigate a Gungraun CI performance regression, analyze where Rust code spends work, or create a temporary Gungraun benchmark and GitHub Actions harness before optimization."
---

# Gungraun Optimize

Act as a performance engineer. Measure provisional changes in GitHub Actions without creating pull requests, inspect Gungraun profiles, immediately publish each accepted optimization as one commit in its own draft pull request with `gh stack`, and continue while a safe profile-supported hypothesis remains.

## Obey the non-negotiable boundaries

- Use `gh stack` for all stack creation, layer addition, branch pushes, pull request submission, synchronization, rebasing, and restructuring. Do not create optimization pull requests with `gh pr create`.
- Start a dedicated stack from the repository trunk. Do not inspect, adopt, reuse, modify, rebase, wait for, or otherwise interact with unrelated pull requests or their branches.
- Keep exactly one focused optimization commit in every accepted stack layer relative to its parent. Keep exactly one accepted optimization in the corresponding pull request diff.
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
3. Read repository instructions and inspect only the worktree, trunk, remotes, stack state owned by this task, and existing user changes. Preserve unrelated work without investigating unrelated pull requests.
4. Search manifests, benchmark targets, Gungraun macros and configuration, GitHub workflows, summary renderers, and wall-time suites. Prefer `rg` and `git grep`.
5. Confirm GitHub authentication, runner access, the workflow trigger, and the installed `gh stack` commands. Never substitute another stacked-pull-request tool.
6. Freeze the starting trunk SHA for cumulative comparisons. Record the benchmark filter, base revision, Rust toolchain, Gungraun library version, runner version and architecture, build features, and relevant environment. Keep these invariant across comparisons.

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

Reuse a valid existing harness and workflow without changing them when possible. If either is missing or invalid, read [the Gungraun CI reference](references/gungraun-ci.md) and create a temporary harness layer unless the user explicitly requests a permanent harness.

- Prefer the smallest benchmark that covers the requested behavior while exercising realistic inputs.
- Keep setup and teardown outside the measured function and protect inputs and results with `std::hint::black_box`.
- Prefer a separate workspace member for library benchmarks when feasible. Choose a binary benchmark only when the executable boundary is part of the behavior under test.
- Match `gungraun-runner` to the library dependency. Use a Linux Actions runner with Valgrind.
- Save a base baseline, benchmark head against it, save machine-readable summaries, and retain profiles and flamegraphs as artifacts.
- Ensure base and head execute the same benchmark definition, toolchain, features, and environment. Never compare different workloads under the same benchmark name.
- If the base revision lacks the harness, bootstrap only benchmark plumbing that can exercise unchanged base code. If that is unsafe or the required API does not exist on base, treat the workload as non-comparable and stop.

When temporary CI is required, use a separate PR-free measurement stack. Read the [temporary harness procedure](references/gungraun-ci.md#temporary-harness-stack). Never submit this stack. Keep the publication stack based directly on trunk and transfer only accepted optimization commits into it. Record the mapping between measured and published SHAs, and verify that their production diffs match before publication.

## Run the stacked optimization loop

### 1. Analyze the current result

- Scope subsequent runs with Gungraun's benchmark filter when the workflow permits it.
- Rank target cases by the requested metric. Start with `Ir` unless another metric was requested.
- Inspect regular and differential flamegraphs or annotate the retained Callgrind data. Find functions with high self cost, unexpectedly repeated work, avoidable allocations, poor locality, expensive conversions, or unnecessary library calls.
- Before creating or measuring a provisional layer, record and freeze the comparable target portfolio, related cases, user-designated or workload-critical protected cases, and case frequencies. Use trustworthy documented production frequencies when available; otherwise assign and disclose an equal frequency of one to every case. Do not change these classifications or weights after seeing provisional results without disclosing the reason and obtaining user approval.
- Select one hypothesis that the available profile evidence supports. Do not optimize from intuition alone. Before selecting a helper optimization, locate its original definition and callers across crates. Prefer changing that shared implementation and include affected callers in the related benchmark cases before freezing the comparison portfolio.
- Reject a hypothesis that requires new or expanded architecture-specific production code or build settings unless the user explicitly requested optimization for that named architecture. Do not infer such a request from the CI runner or profile evidence.

### 2. Create one provisional layer

- Run `gh stack add <attempt-branch>` above the last accepted layer, or use `gh stack init --base <trunk> <attempt-branch>` for the first layer when no stack exists. When using a temporary harness, create attempts in the measurement stack.
- Read and understand the affected path before editing it. Reject the hypothesis immediately if it changes the protected AST representation. Do not reject a helper or implementation change solely because it is in an AST crate.
- Preserve semantics and public compatibility unless the user explicitly allows a non-AST change.
- Keep the patch small enough that its benchmark effect is attributable to one idea.
- Run narrow correctness tests first, then the broader relevant checks required by the repository. For shared helper changes, check the impact on other callers, run their relevant correctness checks, and apply the existing Gungraun comparison and acceptance criteria to the target and related cases.
- Stage only intended files and verify that the protected AST representation is unchanged and no shared helper was duplicated or shadowed. Inspect both the staged diff and each affected function or module's enclosing configuration, inherited `cfg` or `target_feature` attributes, and relevant existing Cargo configuration, build scripts, and compiler flags for `target_arch`, `CARGO_CFG_TARGET_ARCH`, `std::arch`, `core::arch`, runtime feature-detection macros, assembly, architecture-specific flags, or equivalents. Treat a change inside an existing architecture-gated scope as expanding that implementation. Accept it only under the explicit named-architecture exception, then create one provisional optimization commit.

### 3. Push and measure without a pull request

- Push the task-owned branches with `gh stack push`. Do not run `gh stack submit` while a provisional or temporary harness layer is in the current stack.
- Dispatch `gungraun.yml` with exact `base_ref` and `head_ref` SHAs and the frozen `filter`. Run one comparison against the previous accepted revision and one against the starting trunk revision; one run suffices when those bases are identical. Use the same harness for both comparisons, including harness-only counterparts when temporary plumbing is required.
- Follow the [manual dispatch procedure](references/gungraun-ci.md#base-and-head-comparison). Wait only for matching task-owned workflow runs. Match event, dispatch revision, run ID, and artifact comparison SHAs and filter; do not use the latest run without checking its identity. Ignore cancelled or superseded runs.
- Download or inspect the saved summary and profile artifacts. Treat new, removed, renamed, or failed benchmark cases as non-comparable until explained.
- Record the run URL, base/compare SHAs, benchmark identities, exact metrics, summary schema version, and artifact names.

### 4. Compare and decide

- Compare the layer's incremental effect against the previous accepted revision and the cumulative stack effect against the frozen starting trunk revision. Require both comparisons to satisfy the applicable portfolio and per-case thresholds below; never use earlier wins to justify a newly harmful layer or let individually small regressions evade the cumulative thresholds.
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
- Immediately after acceptance, before creating the next attempt, submit the accepted-only publication stack with `gh stack submit --auto`. This creates a draft PR for the accepted change. If temporary plumbing was used, first transfer only the accepted optimization commit to the publication stack and verify its production diff and correctness there. Never submit provisional, rejected, or harness layers.
- Include both comparison run links, exact metrics, and measured/published SHA mappings in the PR description. Attach the created PR to the chat. Keep each accepted layer unchanged and add the next attempt as a new layer above the last accepted measurement revision.
- After a layer is conclusively rejected, use `gh stack modify` to drop it and delete only its task-owned remote experiment branch with `git push <remote> --delete <attempt-branch>`. Use `gh stack push` if remaining branches changed. There is no rejected PR to close. Do not drop an ambiguous tradeoff while awaiting the user's decision, and do not retain a revert commit in an accepted pull request.
- After two failed hypotheses for the same hot path, move to the next profile-supported target without treating the failures as a reason to finish.

Repeat with fresh profiles because the hot path can move after each accepted optimization.

## Handle failures explicitly

- Fix deterministic build, benchmark, summary parsing, or artifact upload failures before interpreting performance.
- Retry a clearly transient Actions failure once. If it repeats, report the evidence and ask for help.
- Treat missing GitHub permissions, unavailable required runners, and invalid base/head workloads as blockers to measurement.
- For fork pull requests, accept that write-token PR comments may be unavailable; use the read-only job summary and artifacts instead.
- Never use `pull_request_target` to run untrusted pull request code.

## Remove temporary CI before finishing

If this task created temporary CI:

1. Verify each published layer has exactly one optimization commit relative to its parent. Verify the cumulative trunk-to-top diff contains none of the recorded temporary files or manifest hunks and no change to the protected AST representation or duplicated or shadowed shared helper. No temporary harness commit may be reachable from a published branch.
2. Preserve CI run URLs, artifacts, and the mapping from measured SHAs to published commits.
3. Remove rejected layers and the task-owned measurement stack with `gh stack modify`, then delete its remote measurement and harness branches. Do not delete accepted publication branches. No harness or rejected PR should have been created.
4. Run `gh stack sync --prune` on the publication stack. Verify its bottom PR targets trunk and each later PR targets the preceding accepted layer. Run normal correctness CI on the published stack.

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
- the final `gh stack` order and proof that temporary CI changes, commits, and branches were cleaned up and no temporary PR was created.
