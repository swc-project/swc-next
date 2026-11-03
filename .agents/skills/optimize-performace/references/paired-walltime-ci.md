# Paired Wall-Time CI Reference

Read this before dispatching, auditing, or extending the paired wall-time measurement in `.github/workflows/profile-perf.yml`.
The implementation is `.github/scripts/paired-walltime.mjs`; component usage is in
[tasks/benchmark/README.md](../../../../tasks/benchmark/README.md).

## Existing measurement boundary

- The script builds the `benchmark` package's available `parser`, `visit`, `semantic`, and `type_strip` Criterion targets with `cargo bench --locked --no-run`. It measures cases present in both builds. Check `head_only` and `base_only`; a successful run can still omit required cases.
- Each round runs each case once with each binary in the same job. Odd rounds run base first; even rounds run head first. Each recorded value is Criterion's median point estimate in nanoseconds, not process startup time or the workflow duration.
- Defaults are 6 rounds, 1 second of warm-up, and 2 seconds of measurement. `rounds` must be a positive even integer. The script also reads `PAIRED_WALLTIME_WARM_UP` and `PAIRED_WALLTIME_MEASUREMENT`, but the existing workflow exposes neither as dispatch inputs.
- One `ubuntu-24.04` (x86_64) build job checks out and builds each revision once. With `suite=all`, it also builds the Gungraun benchmark from the same Cargo target directory. Separate jobs then run the uploaded Gungraun and wall-time executables in parallel, so a Gungraun failure does not stop the wall-time rounds; check the `paired-walltime` artifact before you treat such a run as a measurement failure. The wall-time job runs both revisions on one runner and records CPU and environment information; hosted runners can differ across runs. Pair base/head on the same runner and retain each run's environment. Never compare a base time from one job with a head time from another.
- The driver scripts come from the dispatch revision; the build job copies its script outside the checkout before switching revisions. Manual runs use head's Rust toolchain for both builds. Benchmark definitions, fixtures, manifests, Cargo configuration, and features still come from each measured revision: inspect them for identical workloads and settings.
- The artifact contains raw round estimates and a rendered summary. It does not include statistical confidence intervals, raw Criterion samples, flamegraphs, or allocation profiles. Use optional diagnostic tools separately when needed.

## Base and head comparison

The workflow must exist on the default branch for manual dispatch. If dispatch is unavailable, report the blocker; do not create experiment PRs to trigger measurement.

Push task-owned branches, resolve comparison refs to immutable SHAs, and dispatch:

```bash
gh workflow run profile-perf.yml --ref <workflow-branch> \
  -f base=<base-sha> -f head=<head-sha> -f rounds=6 -f suite=walltime
```

- The inputs are `base`, `head`, `rounds`, `suite` (`all`, `walltime`, or `gungraun`; default `all`), and `filter`. `filter` selects Gungraun benchmarks only; the wall-time suite has no filter. Use `suite=all` when Gungraun counts can help explain the result. Always pass `head` explicitly; it defaults to the dispatch revision. Keep the workflow branch at a recorded fixed revision across comparable runs.
- Run standalone (attempt versus parent), marginal (integration plus attempt versus integration), and cumulative (integration plus attempt versus frozen trunk) comparisons. Reuse a run only when both base and head SHAs match exactly.
- Record dispatch time and workflow revision. Identify the matching task-owned run by event, dispatch SHA, time, and run ID:

```bash
gh run list --workflow profile-perf.yml --event workflow_dispatch \
  --branch <workflow-branch> --json databaseId,headSha,createdAt,status,url
```

- Follow the environment's supported run-wait mechanism for that run ID. Ignore cancelled or superseded runs. Download each run into a separate directory:

```bash
gh run download <run-id> --name paired-walltime --dir <run-artifact-directory>
```

- With `suite=all`, the run also uploads `gungraun-valgrind-<run-id>-<attempt>`, which contains Gungraun summaries and raw Callgrind data. Use it only as diagnostic evidence.

- Verify full `base_sha` and `head_sha`, rounds, timing settings, and benchmark identities in `results.json`. The Actions run's `headSha` is the dispatch revision, which need not be the measured head. If concurrent dispatches are possible, inspect artifact identities; never choose the latest run by list order.
- Retain `results.json`, `summary.md`, run URL, workflow/script revision, build settings, and any measured-to-published SHA mapping. Missing or failed baselines are measurement failures, not regressions.
- PR wall-time runs require the `Codspeed` label; Gungraun runs for every PR. PR wall-time runs compare the PR merge SHA with its base SHA. Push runs on `main` compare the pushed commit with its first parent. Neither substitutes for an attempt's exact-SHA manual comparison.

## Read results and account for noise

For `schema_version: 1`, `results.json` contains:

- `base_sha`, `head_sha`, and `environment` (CPU model, topology, caches, SIMD flags, OS/kernel, runner image/name, and Rust version);
- `rounds`, `warm_up_seconds`, `measurement_seconds`, and `threshold` (currently `0.02`);
- `benchmarks`, each with `target`, `id`, and aligned `base` and `head` arrays of per-round nanosecond estimates;
- `head_only` and `base_only` lists of unmatched case IDs.

Validate the schema and require every frozen target and related `(target, id)` pair to have the expected number of finite positive estimates. Compare the same cases and operations in both revisions. New, removed, renamed, skipped, or failed cases do not establish an improvement.

For each case, calculate `head[r] / base[r]` per round. The summary's change is the median paired ratio minus one. Its displayed base/head times are separate medians, so their ratio need not equal the reported change. A case appears as faster or slower only when all rounds agree on direction and the absolute median change is at least 2%. Omitted cases still need inspection for the skill's portfolio and regression rules.

Use the per-round portfolio calculations and acceptance rules in `SKILL.md`. Preserve pairing when aggregating. The agreement rule and observed range are noise checks, not confidence intervals. Retain both original and repeat runs; do not repeat until a favorable result appears. If noise or CPU differences leave the effect unresolved, report it as inconclusive.

## Temporary harness branches

Use temporary plumbing only when the existing suite does not cover the requested operation or cannot compare base and head. Keep it on task-owned branches without PRs.

- Create a harness branch from frozen trunk. Commit only benchmark definitions, fixtures, required manifests, and workflow/script changes. Record all temporary paths and manifest hunks. This harness-only revision supplies the cumulative baseline and independent attempt parent for measurement.
- Reuse the paired workflow's same-job, alternating-order design. Add representative Criterion targets to the driver's target list when needed. For a JS boundary, add a paired elapsed-time harness that exercises the public N-API operation with consistent Node settings. Verify that the base API supports the identical workload before changing production code.
- Keep setup outside the measured operation unless it belongs to the requested boundary. Prevent elimination of Rust inputs/results with `std::hint::black_box`. Keep case identities and fixtures stable. Do not compare changed workloads under the same ID.
- Push the task-owned harness branch and dispatch the workflow there. Keep the driver revision fixed across that set of comparisons. If measurement settings or plumbing must change, document why and remeasure all affected comparisons with matching plumbing.
- Create attempts from the harness baseline or a measured dependency. Keep a measured integration branch containing the same harness and accepted optimizations. Every baseline and head must contain identical plumbing; never bootstrap optimized production code onto a baseline.
- On acceptance, create a publication branch from trunk or the dependency's publication branch. Transfer only the accepted production commit. Verify one optimization commit relative to its parent, identical production diffs, correctness, and no temporary harness ancestry or hunks. If transfer changes production code, remeasure before publication.
- Preserve measured/published SHA mappings and CI evidence. Delete only task-owned temporary branches after finishing; retain accepted publication branches. Verify no temporary changes or commits entered any published branch.

## Failures and reporting

Fix build, harness, parsing, and artifact failures before interpreting results. Retry a clearly transient Actions failure once. Treat repeated failures, missing runner access, missing permissions, or non-comparable workloads as measurement blockers.

Keep performance reporting non-blocking; do not add failing regression limits. Keep workflow token permissions minimal, do not execute untrusted code with `pull_request_target`, and use summaries/artifacts when fork PR comments are unavailable. Manual dispatch produces a summary and artifact without creating an experiment PR.
