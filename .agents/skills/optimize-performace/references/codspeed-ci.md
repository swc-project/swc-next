# CodSpeed CI Reference

Read this before dispatching, auditing, or extending the CodSpeed wall-time measurement in `.github/workflows/codspeed.yml`.
Component usage is in [tasks/benchmark/README.md](../../../../tasks/benchmark/README.md).

## Existing measurement boundary

- The workflow builds the `benchmark` package's `parser`, `visit`, and `semantic` Criterion targets with `cargo codspeed build -m walltime` and runs them with the CodSpeed action in `walltime` mode. The `type_strip`, `minifier`, and `gungraun` targets are not part of it. The `criterion` dependency is `codspeed-criterion-compat`.
- A matrix runs the same job on two CodSpeed Macro Runners: AMD Ryzen 9950X (x86_64, `codspeed-macro-x64-ryzen-9950x-ubuntu-24-04`) and Arm Cortex-A72 (arm64, `codspeed-macro-arm64-graviton-ubuntu-22-04`). The job names are `Measure walltime (x86_64)` and `Measure walltime (arm64)`. CodSpeed documents that results are comparable only when both runs used the same machine, so keep each architecture on its label.
- Each run measures one revision. Base and head are measured by separate runs, in separate jobs, possibly on different machines of the same label. The runs share no noise, and the report contains no per-round pairs. Use repeat runs, not a single pair, to judge noise.
- The workflow triggers on pushes to `main`, on pull requests with the `Codspeed` label, and on `workflow_dispatch`. A dispatch measures the head of the branch that it runs on. `workflow_dispatch` must exist in `codspeed.yml` on the default branch before `gh workflow run` can find the workflow.
- CodSpeed documents that it does not support running the same benchmarks more than once within one workflow, and that duplicate runs can produce a warning. The matrix does exactly that on two architectures. Before you rely on a result, confirm with `list_runs` and `get_run` that the x86_64 and arm64 results are separate and identifiable. If they are merged, duplicated, or cannot be told apart, this is a measurement blocker: report it and do not guess which result belongs to which architecture.

## Measure and compare revisions

Set up the tools once: the CodSpeed MCP server (`claude mcp add --transport http CodSpeed https://mcp.codspeed.io/mcp`, then OAuth in the browser) or the CodSpeed CLI (`codspeed auth login`). Use `list_repositories` to confirm that the repository is visible.

Push task-owned branches, one per revision that needs a measurement (an attempt, its parent, the integration branch, and the frozen trunk revision), and dispatch the workflow on each:

```bash
gh workflow run codspeed.yml --ref <branch>
gh run list --workflow codspeed.yml --event workflow_dispatch \
  --branch <branch> --json databaseId,headSha,createdAt,status,url
```

- A dispatch cannot take a bare SHA, so create a task-owned branch at the SHA that you want to measure. Record the full `headSha` of every run.
- Keep the workflow revision identical across comparable runs. A branch that contains different `codspeed.yml` or benchmark code than its counterpart is a different workload.
- Follow the environment's supported run-wait mechanism for each run ID. Ignore cancelled or superseded runs. Retry a clearly transient failure once.
- Find the CodSpeed run of each workflow run with `list_runs(repository, limit)`: match the commit hash, the branch, and the event type. Check the run's architecture and CPU in the data that `get_run` and `compare_runs` return. Never choose the latest run by list order.
- Compare with `compare_runs(repository, head_run_id, base_run_id)`, once per architecture. Read its warnings about environment differences. For the numbers behind a decision, call `get_benchmark_result(repository, run_id, benchmark_uri, runner_mode)` with runner mode `WallTime` for each case in the frozen portfolio, including cases that the comparison report omits. Use one statistic consistently.
- Use `query_flamegraph(repository, run_id, benchmark_uri, runner_mode)` on the hottest benchmarks to locate costs, and its `base_run_id` option for a differential view of an attempt. Flamegraphs explain a result; they do not establish a speedup.
- Retain run IDs, run URLs, measured SHAs, the architecture and CPU of every run, and any measured-to-published SHA mapping. A missing or failed baseline is a measurement failure, not a regression.
- Push runs on `main` and labeled PR runs also create CodSpeed runs, and a baseline for trunk may already exist. Use such a run as the trunk baseline only when its SHA is the frozen trunk SHA and it ran on the same architecture and hardware. Prefer a fresh dispatch at the same time as the head run.

## Read results and account for noise

Validate that every frozen target and related benchmark appears in both runs on each architecture. Compare the same cases and operations in both revisions. New, removed, renamed, skipped, or failed cases do not establish an improvement.

Use the per-case and portfolio calculations and the acceptance rules in `SKILL.md`. Apply them separately for each architecture. A `compare_runs` entry marked as an improvement or a regression is CodSpeed's own significance judgment on one pair; it is not a confidence interval. Retain both the first and the repeat measurement pair and do not repeat until a favorable result appears. If noise or hardware differences leave the effect unresolved, report it as inconclusive.

Instruction counts from `.github/workflows/profile-perf.yml` (Gungraun, dispatched with `base`, `head`, and `filter`) can explain a change in a hot path. They never decide acceptance.

## Temporary harness branches

Use temporary plumbing only when the existing suite does not cover the requested operation or cannot compare base and head. Keep it on task-owned branches without PRs.

- Create a harness branch from frozen trunk. Commit only benchmark definitions, fixtures, required manifests, and workflow changes (for example, adding a `--bench` target to the lists in `codspeed.yml`). Record all temporary paths and manifest hunks. This harness-only revision supplies the cumulative baseline and independent attempt parent for measurement.
- A new Rust benchmark must use the `criterion` dependency (CodSpeed's compatible crate) and be built by `cargo codspeed build -m walltime`. For a JS boundary, add a benchmark that exercises the public N-API operation with consistent Node settings and runs under CodSpeed on both runners. Verify that the base API supports the identical workload before changing production code.
- Keep setup outside the measured operation unless it belongs to the requested boundary. Prevent elimination of Rust inputs and results with `std::hint::black_box`. Keep case identities and fixtures stable. Do not compare changed workloads under the same ID.
- Push the task-owned harness branch and dispatch the workflow there. Keep the workflow revision fixed across that set of comparisons. If measurement settings or plumbing must change, document why and remeasure all affected comparisons with matching plumbing.
- Create attempts from the harness baseline or a measured dependency. Keep a measured integration branch containing the same harness and accepted optimizations. Every baseline and head must contain identical plumbing; never bootstrap optimized production code onto a baseline.
- On acceptance, create a publication branch from trunk or the dependency's publication branch. Transfer only the accepted production commit. Verify one optimization commit relative to its parent, identical production diffs, correctness, and no temporary harness ancestry or hunks. If transfer changes production code, remeasure before publication.
- Preserve measured/published SHA mappings and CodSpeed evidence. Delete only task-owned temporary branches after finishing; retain accepted publication branches. Verify no temporary changes or commits entered any published branch.

## Failures and reporting

Fix build, harness, and upload failures before interpreting results. Retry a clearly transient Actions failure once. Treat repeated failures, missing runner or CodSpeed access, missing permissions, or non-comparable workloads as measurement blockers.

Keep performance reporting non-blocking; do not add failing regression limits. Keep workflow token permissions minimal, do not execute untrusted code with `pull_request_target`, and use the CodSpeed report when fork PR comments are unavailable. A manual dispatch produces CodSpeed runs without creating an experiment PR.
