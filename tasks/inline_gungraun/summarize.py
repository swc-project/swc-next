import csv
import json
import math
import os
import pathlib
import statistics
import subprocess
import sys


root = pathlib.Path(sys.argv[1])
events = ["Ir", "Dr", "Dw", "Bc", "Bcm", "Bi", "Bim", "I1mr", "D1mr", "D1mw", "ILmr", "DLmr", "DLmw", "EstimatedCycles"]
rounds = []
csv_rows = []
for number in (1, 2):
    rows = []
    for path in sorted((root / f"round-{number}/profiles").rglob("summary.json")):
        data = json.loads(path.read_text())
        summaries = data if isinstance(data, list) else [data]
        for summary in summaries:
            if str(summary["version"]) != "7":
                raise ValueError(f"Unsupported Gungraun schema: {summary['version']}")
            profile = next(profile["data"] for profile in summary["profiles"] if profile["tool"] == "Callgrind")
            metrics = {}
            for event in events:
                values = profile["total"]["metrics"][event]["values"]
                base, head = values["old"], values["new"]
                if event == "Ir" and min(base, head) < 100_000:
                    raise ValueError(f"Incomplete parser collection for {summary['id']}: {base}, {head}")
                change = (head / base - 1) * 100 if base else None
                metrics[event] = {"base": base, "head": head, "delta": head - base, "delta_percent": change}
                csv_rows.append({"round": number, "case": summary["id"], "metric": event, "base": base, "head": head, "delta": head - base, "delta_percent": change})
            rows.append({"case": summary["id"], "frequency": 1, "metrics": metrics, "profile_summary": str(path.relative_to(root))})
    rows.sort(key=lambda row: row["case"])
    if len(rows) != 13 or len({row["case"] for row in rows}) != 13:
        raise ValueError(f"Expected 13 comparable cases; round {number} has {len(rows)}")
    totals = {}
    for event in events:
        base = sum(row["metrics"][event]["base"] for row in rows)
        head = sum(row["metrics"][event]["head"] for row in rows)
        totals[event] = {"base": base, "head": head, "delta": head - base, "delta_percent": (head / base - 1) * 100 if base else None}
    geometric_mean = math.expm1(statistics.mean(math.log(row["metrics"]["Ir"]["head"] / row["metrics"]["Ir"]["base"]) for row in rows)) * 100
    rounds.append({"round": number, "rows": rows, "totals": totals, "geomean_ir_delta_percent": geometric_mean})

if [row["case"] for row in rounds[0]["rows"]] != [row["case"] for row in rounds[1]["rows"]]:
    raise ValueError("Cases differ between repetitions")


def version(*command):
    return subprocess.check_output(command, text=True).strip()


report = {
    "schema_version": 1,
    "source": "GitHub Actions",
    "base_sha": os.environ["BASE_SHA"],
    "head_sha": os.environ["HEAD_SHA"],
    "run_url": f"https://github.com/{os.environ['GITHUB_REPOSITORY']}/actions/runs/{os.environ['GITHUB_RUN_ID']}",
    "architecture": version("uname", "-m"),
    "rustc": version("rustc", "-Vv"),
    "gungraun": version("gungraun-runner", "--version"),
    "valgrind": version("valgrind", "--version"),
    "profile": "bench: opt-level=3, fat LTO, codegen-units=1, debug=true, strip=none",
    "allocator": "Rust default system allocator",
    "options": {"tokens": False, "comments": "none", "preserve_parens": True, "source_type": "unambiguous"},
    "aslr": "--allow-aslr=yes on both base and head",
    "caches": {"I1": "32768,8,64", "D1": "32768,8,64", "LL": "8388608,16,64"},
    "weighting": "One parse per case (frequency=1); instruction portfolio is sum(head Ir) / sum(base Ir). Geometric mean is secondary.",
    "diff_paths": version("git", "diff", "--name-only", os.environ["BASE_SHA"], os.environ["HEAD_SHA"]).splitlines(),
    "rounds": rounds,
}
(root / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
with (root / "counts.csv").open("w", newline="") as output:
    writer = csv.DictWriter(output, fieldnames=("round", "case", "metric", "base", "head", "delta", "delta_percent"))
    writer.writeheader()
    writer.writerows(csv_rows)

print(f"Base: `{report['base_sha']}`; head: `{report['head_sha']}`; architecture: `{report['architecture']}`.")
print("\nPositive changes mean more events. Cache misses and estimated cycles are simulation metrics.\n")
for result in rounds:
    print(f"### Round {result['round']}\n")
    print("| Case | Base Ir | Head Ir | Ir change | Conditional branches | I1 misses | Estimated cycles |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in result["rows"]:
        metrics = row["metrics"]
        ir = metrics["Ir"]
        print(f"| {row['case']} | {ir['base']:,} | {ir['head']:,} | {ir['delta_percent']:+.3f}% | {metrics['Bc']['delta_percent']:+.3f}% | {metrics['I1mr']['delta_percent']:+.3f}% | {metrics['EstimatedCycles']['delta_percent']:+.3f}% |")
    print(f"\nInstruction portfolio: {result['totals']['Ir']['delta_percent']:+.3f}%; secondary geometric mean: {result['geomean_ir_delta_percent']:+.3f}%.\n")
