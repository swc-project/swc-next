"""Compare repeated Criterion processes on one runner, without selecting minima."""

import argparse
import json
import math
import os
from pathlib import Path
import random
import shutil
import statistics
import subprocess


BENCHES = ("parser", "visit", "semantic")
PAIRS = 5
THRESHOLD = 0.01


def read_samples(directory):
    results = {}
    for path in sorted(directory.glob("**/new/sample.json")):
        identity = json.loads(path.with_name("benchmark.json").read_text())["full_id"]
        sample = json.loads(path.read_text())
        iters, times = sample["iters"], sample["times"]
        if not iters or len(iters) != len(times):
            raise ValueError(f"Invalid samples: {path}")
        if any(not math.isfinite(x) or x <= 0 for x in [*iters, *times]):
            raise ValueError(f"Invalid sample values: {path}")
        if identity in results:
            raise ValueError(f"Duplicate benchmark: {identity}")
        # Criterion records elapsed nanoseconds and iteration counts per sample.
        results[identity] = statistics.median(t / n for t, n in zip(times, iters))
    if not results:
        raise ValueError(f"No Criterion samples in {directory}")
    return results


def compare(base, head):
    if len(base) != PAIRS or len(head) != PAIRS:
        raise ValueError(f"Expected {PAIRS} independent process pairs")
    names = set(base[0])
    if not names or any(set(run) != names for run in [*base, *head]):
        raise ValueError("Benchmark sets differ between processes or revisions")
    rows = []
    for name in sorted(names):
        before = [run[name] for run in base]
        after = [run[name] for run in head]
        # Resample whole process pairs, never the correlated samples within one process.
        rng = random.Random(0)
        ratios = []
        for _ in range(10_000):
            indices = rng.choices(range(PAIRS), k=PAIRS)
            ratios.append(statistics.median(after[i] for i in indices)
                          / statistics.median(before[i] for i in indices))
        ratios.sort()
        lower, upper = ratios[249], ratios[9749]
        ratio = statistics.median(after) / statistics.median(before)
        if lower > 1 + THRESHOLD:
            assessment = "Regression candidate"
        elif upper < 1 - THRESHOLD:
            assessment = "Improvement candidate"
        else:
            assessment = "Inconclusive"
        rows.append(dict(name=name, base_ns=statistics.median(before),
                         head_ns=statistics.median(after), ratio=ratio,
                         lower=lower, upper=upper, assessment=assessment))
    geometric_ratio = math.exp(statistics.mean(math.log(row["ratio"]) for row in rows))
    return dict(rows=rows, geometric_ratio=geometric_ratio)


def render(result, base_sha, head_sha):
    def percent(ratio):
        return f"{(ratio - 1) * 100:+.2f}%"

    lines = [
        "<!-- walltime-median -->",
        "## Wall-time median comparison", "",
        f"Base: `{base_sha}`; compare: `{head_sha}`.", "",
        f"{PAIRS} independent process pairs on one Macro Runner; execution order alternates "
        "base/head and head/base. Both revisions use the compare revision's benchmark harness "
        "and Rust toolchain. Each process median uses elapsed time per iteration; the reported "
        "revision time is the median of those process medians.", "",
        "Changes are compare / base − 1; negative is faster. Intervals are paired percentile "
        "bootstrap 95% confidence intervals, resampling processes. Candidates require the "
        "whole interval to exceed ±1%. With only five pairs these intervals are approximate; "
        "they are not corrected for multiple comparisons. Candidates need confirmation and "
        "do not fail CI.", "",
        f"**Geometric mean of time ratios: {percent(result['geometric_ratio'])}** "
        f"({len(result['rows'])} benchmarks, equal weight). Individual regressions can be "
        "hidden by this aggregate.", "",
        "This report uses medians. The separate CodSpeed report retains its own statistics.", "",
        "| Benchmark | Base median (ms) | Compare median (ms) | Change | 95% interval | Assessment |",
        "| --- | ---: | ---: | ---: | --- | --- |",
    ]
    for row in result["rows"]:
        name = row["name"].replace("|", "&#124;").replace("`", "&#96;").replace("\n", " ")
        lines.append(f"| `{name}` | {row['base_ns'] / 1e6:.6f} | "
                     f"{row['head_ns'] / 1e6:.6f} | {percent(row['ratio'])} | "
                     f"{percent(row['lower'])} to {percent(row['upper'])} | {row['assessment']} |")
    return "\n".join(lines) + "\n"


def build(checkout, output, side):
    command = ["cargo", "bench", "--locked", "-p", "benchmark", "--no-run",
               "--message-format=json"]
    for bench in BENCHES:
        command.extend(["--bench", bench])
    # Keep builds separate so neither revision can reuse the other's workspace artifacts.
    env = {**os.environ, "CARGO_TARGET_DIR": str(output / "build" / side)}
    with (output / f"build-{side}.jsonl").open("w") as log:
        subprocess.run(command, cwd=checkout, env=env, stdout=log, check=True)
    binaries = {}
    for line in (output / f"build-{side}.jsonl").read_text().splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact" and message.get("executable"):
            name = message["target"]["name"]
            if "bench" in message["target"]["kind"] and name in BENCHES:
                binaries[name] = message["executable"]
    if set(binaries) != set(BENCHES):
        raise ValueError(f"Missing benchmark executables for {side}")
    return binaries


def measure(base, head, output):
    output.mkdir(parents=True, exist_ok=False)
    # Freeze the workload, including fixtures, rather than compare different inputs.
    shutil.rmtree(base / "tasks/benchmark")
    shutil.copytree(head / "tasks/benchmark", base / "tasks/benchmark")
    binaries = {side: build(checkout, output, side)
                for side, checkout in (("base", base), ("head", head))}
    runs = {"base": [], "head": []}
    for pair in range(PAIRS):
        order = ("base", "head") if pair % 2 == 0 else ("head", "base")
        pair_runs = {"base": {}, "head": {}}
        for bench in BENCHES:
            for side in order:
                directory = output / "samples" / str(pair) / side / bench
                directory.mkdir(parents=True)
                env = {**os.environ, "CRITERION_HOME": str(directory)}
                command = [binaries[side][bench], "--bench", "--noplot",
                           "--warm-up-time", "0.5", "--measurement-time", "2",
                           "--sample-size", "50", "--nresamples", "1000"]
                print(f"Pair {pair + 1}/{PAIRS}: {side}/{bench}", flush=True)
                with (directory / "process.log").open("w") as log:
                    subprocess.run(command, cwd=base if side == "base" else head,
                                   env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
                for name, value in read_samples(directory).items():
                    pair_runs[side][f"{bench}/{name}"] = value
        for side in runs:
            runs[side].append(pair_runs[side])
        (output / "process-medians.json").write_text(json.dumps(runs, indent=2) + "\n")
    return compare(runs["base"], runs["head"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=Path)
    parser.add_argument("head", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    base, head, output = (path.resolve() for path in (args.base, args.head, args.output))
    if base == head or base in head.parents or head in base.parents:
        raise ValueError("Use separate base and head checkouts")
    revisions = {side: subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=checkout,
                                              text=True).strip()
                 for side, checkout in (("base", base), ("head", head))}
    result = measure(base, head, output)
    (output / "comparison.json").write_text(json.dumps({**result, **revisions}, indent=2) + "\n")
    report = render(result, revisions["base"], revisions["head"])
    (output / "summary.md").write_text(report)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as summary:
            summary.write(report)


if __name__ == "__main__":
    main()
