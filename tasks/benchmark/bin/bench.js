import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";

// Adapted from .agents/skills/js-parser-benchmark/scripts/bench.mjs.
// Calibration keeps the best ~10 ms round. Hyperfine then runs a fixed number
// of operations, shared by all engines, so it measures the same amount of work.
export function bench(run, argument = "10") {
  if (argument === "--calibrate") {
    calibrate(run);
    return;
  }

  const iterations = Number(argument);
  assert(
    Number.isSafeInteger(iterations) && iterations > 0,
    "iterations must be a positive integer or --calibrate",
  );

  let checksum = 0;
  for (let i = 0; i < 5; i++) checksum += run();
  for (let i = 0; i < iterations; i++) checksum += run();
  assert(Number.isFinite(checksum));
}

function calibrate(run) {
  let checksum = 0;
  const warmupStart = performance.now();
  for (let i = 0; i < 5; i++) checksum += run();
  const warmupMs = (performance.now() - warmupStart) / 5;
  const cyclesPerRound = Math.max(
    1,
    Math.round(10 / Math.max(warmupMs, 0.001)),
  );
  const deadline = performance.now() + 250;
  let bestMs = Infinity;
  for (let round = 0; round < 5 || performance.now() < deadline; round++) {
    const start = performance.now();
    for (let i = 0; i < cyclesPerRound; i++) checksum += run();
    bestMs = Math.min(bestMs, (performance.now() - start) / cyclesPerRound);
  }
  assert(Number.isFinite(checksum));
  console.log(bestMs);
}
