#!/usr/bin/env bash
set -euo pipefail

output="$RUNNER_TEMP/expr-inline-counts"
mkdir -p "$output"

for round in 1 2; do
  data="$output/round-$round"
  mkdir -p "$data"
  git checkout --detach "$BASE_SHA"
  git checkout "$HEAD_SHA" -- Cargo.toml Cargo.lock tasks/inline_gungraun
  sha256sum tasks/inline_gungraun/benches/parser.rs tasks/benchmark/files/* > "$data/base-inputs.sha256"
  CARGO_TARGET_DIR="$RUNNER_TEMP/expr-inline-target-base" cargo bench --locked -p inline_gungraun --bench parser -- \
    --allow-aslr=yes --home="$data/profiles" --save-baseline=base \
    --callgrind-metrics=Ir,Dr,Dw,Bc,Bcm,Bi,Bim,I1mr,D1mr,D1mw,ILmr,DLmr,DLmw,EstimatedCycles \
    --save-summary=json 2>&1 | tee "$data/base.txt"
  git checkout --detach "$HEAD_SHA"
  sha256sum tasks/inline_gungraun/benches/parser.rs tasks/benchmark/files/* > "$data/head-inputs.sha256"
  diff -u "$data/base-inputs.sha256" "$data/head-inputs.sha256"
  CARGO_TARGET_DIR="$RUNNER_TEMP/expr-inline-target-head" cargo bench --locked -p inline_gungraun --bench parser -- \
    --allow-aslr=yes --home="$data/profiles" --baseline=base \
    --callgrind-metrics=Ir,Dr,Dw,Bc,Bcm,Bi,Bim,I1mr,D1mr,D1mw,ILmr,DLmr,DLmw,EstimatedCycles \
    --save-summary=json 2>&1 | tee "$data/head.txt"
done

python3 tasks/inline_gungraun/summarize.py "$output" | tee -a "$GITHUB_STEP_SUMMARY"
