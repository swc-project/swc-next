import assert from "node:assert/strict";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { loadSummaries, renderFailureSummary } from "./render-gungraun-summary.mjs";
import { compare, render } from "./render-gungraun-js-summary.mjs";

const directory = process.argv[2];
const expected = JSON.parse(process.env.JS_BENCH_MATRIX).include.map(item => item.case).sort();
let summary;
try {
  if (process.env.JS_BENCH_REPEATABILITY) assert.equal(process.env.JS_BENCH_REPEATABILITY, "success", "same-SHA check failed");
  const metadata = readdirSync(directory).filter(name => name.startsWith("gungraun-js-results-"))
    .map(name => JSON.parse(readFileSync(join(directory, name, "metadata.json"), "utf8")));
  assert.deepEqual(metadata.map(item => item.case).sort(), expected, "missing or duplicate shards");
  for (const item of metadata) {
    assert.equal(item.status, "passed", `failed shard: ${item.case}`);
    assert.equal(item.base_sha, metadata[0].base_sha);
    assert.equal(item.head_sha, metadata[0].head_sha);
    assert.deepEqual(item.hashes, metadata[0].hashes, "shards used different harnesses");
  }
  const rows = compare(await loadSummaries(directory));
  assert.deepEqual(rows.map(row => row.name).sort(), expected.map(name => name.replace(/[.-]/g, "_")).sort());
  writeFileSync(join(directory, "comparison.json"), JSON.stringify(rows, null, 2));
  writeFileSync(join(directory, "metadata.json"), JSON.stringify(metadata, null, 2));
  writeFileSync(join(directory, "revisions.txt"), `Base: \`${metadata[0].base_sha}\`; compare: \`${metadata[0].head_sha}\`\n`);
  summary = render(rows);
} catch (error) {
  summary = renderFailureSummary(error.message);
  console.error(error);
  process.exitCode = 1;
} finally {
  writeFileSync(join(directory, "summary.md"), summary);
}
