import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";

// Adapted from .agents/skills/js-parser-benchmark/scripts/bench.mjs.
// Calibration keeps the best ~10 ms round. Hyperfine then runs a fixed number
// of parses, shared by both engines, so it measures the same amount of work.
export function bench(run, argument = "10") {
  const calibrate = argument === "--calibrate";
  const iterations = calibrate ? Infinity : Number(argument);
  assert(
    calibrate || (Number.isSafeInteger(iterations) && iterations > 0),
    "iterations must be a positive integer or --calibrate",
  );

  let checksum = 0;
  let warmupMs = 0;
  for (let i = 0; i < 5; i++) {
    const start = performance.now();
    checksum += run();
    warmupMs += performance.now() - start;
  }
  const cyclesPerRound = Math.max(
    1,
    Math.round(10 / Math.max(warmupMs / 5, 0.001)),
  );
  const deadline = performance.now() + 250;
  let completed = 0;
  let rounds = 0;
  let bestMs = Infinity;
  while (
    calibrate
      ? rounds < 5 || performance.now() < deadline
      : completed < iterations
  ) {
    const cycles = Math.min(cyclesPerRound, iterations - completed);
    const start = performance.now();
    for (let i = 0; i < cycles; i++) checksum += run();
    bestMs = Math.min(bestMs, (performance.now() - start) / cycles);
    completed += cycles;
    rounds++;
  }
  assert(Number.isFinite(checksum));
  if (calibrate) console.log(bestMs);
}
