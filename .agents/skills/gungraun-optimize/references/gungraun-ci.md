# Gungraun Harness and GitHub Actions Reference

Use this reference only when creating, auditing, or repairing a Gungraun harness or its GitHub Actions workflow.

## Contents

- Harness selection
- Reproducible benchmark design
- Version and runner setup
- Base and head comparison
- Temporary harness stack
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

Use a pull request workflow with full history. Resolve the base and head from the event payload, not from an ambiguous local branch name.

Run the base revision first:

```bash
git checkout --detach "$BASE_SHA"
cargo bench --quiet -p <package> --bench <bench> -- \
  --home="$RUNNER_TEMP/gungraun" \
  --save-baseline=base
```

Then run the exact pull request head and retain a JSON summary:

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
- If benchmark source, features, or inputs differ between revisions, stop and establish a temporary harness-only bottom layer before optimizing. Do not make that layer permanent unless the user explicitly requests a permanent harness.

See the official [baseline guide](https://gungraun.github.io/gungraun/latest/html/cli_and_env/baselines.html) and [basic CLI reference](https://gungraun.github.io/gungraun/latest/html/cli_and_env/basics.html).

## Temporary harness stack

Use a temporary harness only when the repository lacks valid measurement plumbing. Manage it as the bottom layer of a dedicated `gh stack`, never as an unrelated prerequisite pull request.

- Initialize the temporary branch from trunk with `gh stack init --base <trunk> <temporary-ci-branch>` and publish it as a draft pull request with `gh stack submit --auto`.
- Keep only benchmark definitions, workflow steps, reporting code, fixtures, and required manifest changes in the temporary commit. Do not mix production optimizations into it.
- Record its branch, pull request, commit, paths, and exact manifest hunks before adding optimization layers above it.
- Use the temporary layer to produce the baseline, summaries, profiles, and artifact URLs needed to judge each optimization pull request.
- At the end, use `gh stack modify` to drop the temporary layer and cascade-rebase accepted optimization layers onto trunk. Resubmit it with `gh stack submit --auto` so every pull request base is updated.
- Close the temporary pull request, delete its remote branch, and run `gh stack sync --prune` after the accepted stack has been resubmitted.
- Verify the temporary commit is unreachable from every accepted branch and the recorded paths and hunks are absent from the cumulative trunk-to-top diff. A cleanup revert commit is not acceptable because it leaves temporary CI in the final history.
- Retain links to the pre-cleanup CI runs and artifacts as measurement evidence, then run normal correctness CI against the rewritten stack.

## Profiles and machine-readable output

- Enable `FlamegraphConfig` on the selected Callgrind tool when regular or differential flamegraphs are needed.
- Upload the Gungraun home, `summary.json` files, captured text output, Callgrind data, and generated SVGs with an artifact step guarded by `if: always()`.
- Put a concise exact-count table in `GITHUB_STEP_SUMMARY`. Keep raw output and profiles in a downloadable artifact instead of truncating them into the summary.
- Inspect the summary's schema version before parsing fields. Do not assume a schema from a different Gungraun release.
- Use terminal output as a fallback presentation of the same CI run, not as a separate measurement source.

See the official [machine-readable output](https://gungraun.github.io/gungraun/latest/html/cli_and_env/output/machine_readable.html) and [flamegraph guide](https://gungraun.github.io/gungraun/latest/html/flamegraphs.html).

## GitHub security and reporting

- Use the `pull_request` event to run pull request code. Never use `pull_request_target` for the benchmark job.
- Grant `contents: read` by default. Grant `pull-requests: write` only when the workflow publishes a comment.
- Publish or update a marker-based PR comment only for trusted same-repository branches. Skip write operations for forks and restricted automation users.
- Make job summary and artifact upload work without write permissions so fork pull requests retain useful results.
- Use concurrency keyed by pull request number and cancel superseded runs. Associate every result with its head SHA and ignore cancelled older runs.
- Capture the benchmark exit status before rendering or uploading reports, perform reporting under `always()`, and finally return the original benchmark failure when the harness itself failed.
