import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { setImmediate } from "node:timers/promises";

// Adapted from .agents/skills/js-parser-benchmark/scripts/bench.mjs.
export async function measure(run, rounds = 100, iterations = 1) {
  assert(Number.isSafeInteger(rounds) && rounds > 0, "rounds must be a positive integer");
  assert(Number.isSafeInteger(iterations) && iterations > 0, "iterations must be a positive integer");
  let checksum = 0;
  for (let i = 0; i < 5; i++) {
    checksum += run();
    await setImmediate();
  }
  let bestMs = Infinity;
  for (let round = 0; round < rounds; round++) {
    const roundStart = performance.now();
    for (let i = 0; i < iterations; i++) checksum += run();
    bestMs = Math.min(bestMs, (performance.now() - roundStart) / iterations);
    // Drain deferred native buffer finalizers outside the timed round.
    await setImmediate();
  }
  assert(Number.isFinite(checksum));
  console.log(JSON.stringify({ bestMs }));
}
