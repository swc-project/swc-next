# Gungraun Harness and GitHub Actions Reference

Use this reference only when creating, auditing, or repairing a Gungraun harness or its GitHub Actions workflow.

## Contents

- Harness selection
- Reproducible benchmark design
- Version and runner setup
- Base and head comparison
- Temporary harness branches
- Profiles and machine-readable output
- GitHub security and reporting

## Harness selection

- Use a library benchmark for a Rust function or library API. Prefer a separate, non-published workspace member so cross-crate inlining resembles downstream use.
- Use a binary benchmark when process startup, command arguments, standard input, files, or executable behavior belongs in the measured operation.
- Add `gungraun` as a development dependency and declare every target with `harness = false`.
- Keep benchmark groups and case IDs stable. Renaming a case makes it new or removed instead of comparable.
- Prefer focused cases with representative small and large inputs. Avoid a huge suite when a filter can isolate the affected path.

## Reproducible benchmark design

- Move allocation, parsing of fixtures, temporary directory creation, and other setup outside the measured body. Use Gungraun setup and teardown facilities.
- Wrap inputs and outputs with `std::hint::black_box` so the compiler cannot remove the measured work.
- Fix random seeds and inputs. Avoid network access, filesystem timing, clock reads, and mutable external services.
- Use a sandbox for file-writing benchmarks when possible. Copy deterministic fixtures into it.
- Keep the Rust toolchain, target, profile, features, allocator, environment, and benchmark source identical between base and head.
- Verify multi-threaded or multi-process totals before optimizing an individual fragment. Enable intermediate output temporarily when needed.

See the official [benchmarking best practices](https://gungraun.github.io/gungraun/latest/html/best_practices.html), [library benchmark guide](https://gungraun.github.io/gungraun/latest/html/benchmarks/library_benchmarks/structure.html), and [binary benchmark guide](https://gungraun.github.io/gungraun/latest/html/benchmarks/binary_benchmarks/quickstart.html).

## Version and runner setup

- Run on Linux because Valgrind is required. Do not present macOS or Windows substitutes as comparable measurements.
- Keep the `gungraun` library and `gungraun-runner` versions equal.
- Prefer the official `gungraun/setup-gungraun` action, which can detect the dependency version. Follow repository policy for pinning third-party actions; prefer a full commit SHA when the repository pins actions.
- Set `GUNGRAUN_COLOR=never` when capturing output. Use a unique `--home` under `RUNNER_TEMP` so base and head share results without polluting the checkout.

See the official [CI installation guide](https://gungraun.github.io/gungraun/latest/html/installation/ci.html).

## Base and head comparison

Use `workflow_dispatch` to measure experiments before creating a PR. The workflow must exist on the repository's default branch before manual dispatch is available. Land the permanent dispatch support first; if it is unavailable, report the blocker instead of opening experiment PRs to trigger measurements. See the [GitHub manual workflow documentation](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow?tool=cli).

Push the task-owned branches with `git push <remote> <branch>`, then dispatch with immutable comparison SHAs:

```bash
gh workflow run gungraun.yml --ref <workflow-branch> \
  -f base_ref=<base-sha> -f head_ref=<head-sha> -f filter='<benchmark-filter>'
```

- `--ref` selects the workflow revision. Keep it fixed across the incremental and cumulative runs. `base_ref` is required; `head_ref` defaults to the dispatch revision, and an omitted or empty `filter` runs the whole suite.
- Run standalone comparisons of the attempt against its parent SHA, marginal comparisons of the integration branch plus the attempt against the integration SHA, and cumulative comparisons of the integration branch plus the attempt against the frozen starting trunk SHA. Reuse a run for two comparisons only when both their base and head SHAs are identical; matching bases alone are not enough.
- Record dispatch time and workflow branch SHA. Find the matching `workflow_dispatch` run with `gh run list --workflow gungraun.yml --event workflow_dispatch --branch <workflow-branch> --json databaseId,headSha,createdAt,status,url`, then wait with `gh run watch <run-id> --exit-status` and download with `gh run download <run-id>`. Confirm artifact metadata `comparison.base_sha`, `comparison.compare_sha`, and `benchmark.filter` before accepting the result. If concurrent runs are candidates, inspect their metadata rather than guessing from list order.
- Resolve refs once to commit SHAs before benchmarking. Use the same quoted filter argument for both runs. The existing PR trigger continues to compare the event's base SHA with its merge SHA; it does not replace the exact-head manual evidence.
- Preserve reporting scripts from the workflow checkout before switching revisions so older benchmark revisions do not need the current reporting code.

Run the base revision first:

```bash
git checkout --detach "$BASE_SHA"
cargo bench --quiet -p <package> --bench <bench> -- \
  --home="$RUNNER_TEMP/gungraun" \
  --save-baseline=base
```

Then run the exact comparison head and retain a JSON summary:

```bash
git checkout --detach "$HEAD_SHA"
cargo bench --quiet -p <package> --bench <bench> -- \
  --home="$RUNNER_TEMP/gungraun" \
  --baseline=base \
  --save-summary=json
```

- Add a Gungraun wildcard filter after the options when only a group or case is relevant.
- Do not add `--callgrind-limits`, `--cachegrind-limits`, or `--dhat-limits` to a new reporting-only workflow.
- Preserve existing regression limits unless the user asks to change them. Distinguish their exit code from build or harness errors.
- When bootstrapping a new harness against a base that lacks it, restore only benchmark plumbing from head after checking out base. Never restore the optimized production source. Confirm that the harness calls an API available on base.
- If benchmark source, features, or inputs differ between revisions, stop and establish identical plumbing on separate temporary measurement branches before optimizing. Do not make that plumbing permanent unless the user explicitly requests a permanent harness.

See the official [baseline guide](https://gungraun.github.io/gungraun/latest/html/cli_and_env/baselines.html) and [basic CLI reference](https://gungraun.github.io/gungraun/latest/html/cli_and_env/basics.html).

## Temporary harness branches

Use a temporary harness only when existing measurement plumbing is insufficient. Keep it on task-owned measurement branches with no PRs.

- Create a harness branch from the frozen trunk SHA. Commit only benchmark definitions, workflow steps, reporting code, fixtures, and required manifest changes. Record the branch, commit, paths, and exact manifest hunks. This harness-only revision is the baseline for cumulative comparisons and for independent attempts. Marginal comparisons use the measured integration branch as their baseline.
- Push it with `git push <remote> <branch>` and dispatch the workflow on it. Establish initial profiles before editing production code. Do not create a pull request for it.
- Create each measured attempt from the harness branch, or from the measured counterpart of its dependency for a dependent attempt. Keep a measured integration branch that contains the harness and every accepted optimization. Both sides of each comparison must have identical benchmark definitions, inputs, features, and toolchain. Only bootstrap plumbing onto unchanged production code whose API supports it.
- After each acceptance, create the publication branch from trunk for an independent optimization, or from the dependency's publication branch for a dependent one. Cherry-pick only that accepted production optimization commit. Verify one commit per branch, matching production diffs, no harness ancestry or hunks, and correctness before you publish it immediately as the skill describes.
- Record measured-to-published SHA mappings and attach the comparison evidence to each PR. If transferring the change requires production edits, remeasure that revised patch before publishing it. Continue experiments on the measurement branches.
- At the end, retain run links and artifacts and delete the local and remote measurement branches. Verify all published diffs and ancestry exclude temporary plumbing. There are no temporary PRs to close.

## Profiles and machine-readable output

- Enable `FlamegraphConfig` on the selected Callgrind tool when regular or differential flamegraphs are needed.
- Upload the Gungraun home, `summary.json` files, captured text output, Callgrind data, and generated SVGs with an artifact step guarded by `if: always()`.
- Put a concise exact-count table in `GITHUB_STEP_SUMMARY`. Keep raw output and profiles in a downloadable artifact instead of truncating them into the summary.
- Inspect the summary's schema version before parsing fields. Do not assume a schema from a different Gungraun release.
- Use terminal output as a fallback presentation of the same CI run, not as a separate measurement source.

See the official [machine-readable output](https://gungraun.github.io/gungraun/latest/html/cli_and_env/output/machine_readable.html) and [flamegraph guide](https://gungraun.github.io/gungraun/latest/html/flamegraphs.html).

## GitHub security and reporting

- Use the `pull_request` event to run pull request code. Never use `pull_request_target` for the benchmark job.
- Gate PR comments explicitly on the `pull_request` event. Manual runs report through job summaries and artifacts, with `pull_request: null` in metadata schema version 2 and common revision fields under `comparison`.
- Grant `contents: read` by default. Grant `pull-requests: write` only when the workflow publishes a comment.
- Publish or update a marker-based PR comment only for trusted same-repository branches. Skip write operations for forks and restricted automation users.
- Make job summary and artifact upload work without write permissions so fork pull requests retain useful results.
- Use concurrency keyed by PR number for PR runs and by run ID for manual runs. Manual standalone, marginal, and cumulative runs must not cancel each other. Associate every result with its resolved comparison SHAs and ignore cancelled runs.
- Capture the benchmark exit status before rendering or uploading reports, perform reporting under `always()`, and finally return the original benchmark failure when the harness itself failed.

## Permanent JavaScript API CI

Use `.github/workflows/gungraun-js.yml` for Node.js N-API comparisons. The
workflow accepts `base`, `head`, and a literal case-substring `filter` on manual
runs. PR runs compare the event's exact base and head SHAs. The standalone
`tasks/benchmark/js` harness is copied from head for both runs, without
replacing either revision's product manifests or dependencies.

Use `--jitless --predictable` to disable JIT and reduce GC scheduling variation; retain the three trial counts, median and range. A range
overlap is inconclusive, including in same-SHA checks. The benchmark-only N-API
helper uses Callgrind client requests around the synchronous JS callback;
startup, input preparation and warmup are excluded. Preserve the native symbol
and debug build settings. Use the artifact metadata hashes to verify workload
identity before comparing results across runs. See the harness README for
filters, local Linux commands and limitations of interpreter profile symbolization.
Do not compare these counts directly with JIT-enabled runs or interpret them as
normal JIT-enabled speed.

The JS workflow builds both revisions once and fans out to five representative
case jobs on PRs. Manual runs cover all 24 cases unless filtered. Keep
all three base/head trials for one case on the same runner. The aggregate report
must reject missing or duplicate case artifacts and mismatched harness hashes.
Use the per-case CPU metadata when comparing profiles across runs. The same-SHA
check runs in parallel; do not wait for it before starting unrelated case jobs.
