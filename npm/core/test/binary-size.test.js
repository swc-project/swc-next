import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  measureBinarySizes,
  renderBinarySizeDiff,
} from "../../../.github/scripts/binary-size.mjs";

const baseSha = "a".repeat(40);
const mergeSha = "b".repeat(40);
const rustc = "rustc 1.97.1 (example 2026-09-01)";

function measurement(revision, sizes) {
  return {
    schemaVersion: 1,
    revision,
    target: "x86_64-unknown-linux-gnu",
    profile: "release",
    rustc,
    artifacts: ["core", "parser", "codegen"].map((component, index) => ({
      name: `@swc-next/${component}`,
      bytes: sizes[index],
    })),
  };
}

test("binary size diff reports growth, reduction, and unchanged exact bytes", () => {
  const base = measurement(baseSha, [1024, 2048, 4096]);
  const merge = measurement(mergeSha, [1025, 1536, 4096]);
  merge.artifacts.reverse();
  const report = renderBinarySizeDiff(base, merge);

  assert.ok(report.includes(`Merge: \`${mergeSha}\`; base: \`${baseSha}\``));
  assert.ok(
    report.includes(
      "| `@swc-next/core` | 1.00 KiB | 1.00 KiB | +1 | +0.098% 🔴 |",
    ),
  );
  assert.ok(
    report.includes(
      "| `@swc-next/parser` | 1.50 KiB | 2.00 KiB | -512 | -25.000% 🟢 |",
    ),
  );
  assert.ok(
    report.includes(
      "| `@swc-next/codegen` | 4.00 KiB | 4.00 KiB | 0 | 0.000% ⚪ |",
    ),
  );
});

test("binary size diff rejects incompatible builds", () => {
  const base = measurement(baseSha, [1024, 2048, 4096]);
  const merge = measurement(mergeSha, [1024, 2048, 4096]);

  assert.throws(() =>
    renderBinarySizeDiff(base, { ...merge, rustc: "rustc 1.98.0" }),
  );
  assert.throws(() =>
    renderBinarySizeDiff(base, { ...merge, profile: "bench" }),
  );
  assert.throws(() =>
    renderBinarySizeDiff(base, { ...merge, target: "aarch64-apple-darwin" }),
  );
  assert.throws(() =>
    renderBinarySizeDiff(base, { ...merge, revision: "unknown" }),
  );
});

test("binary size diff rejects incomplete and invalid measurements", () => {
  const base = measurement(baseSha, [1024, 2048, 4096]);
  const merge = measurement(mergeSha, [1024, 2048, 4096]);

  assert.throws(() =>
    renderBinarySizeDiff(base, measurement(mergeSha, [0, 2048, 4096])),
  );
  assert.throws(() =>
    renderBinarySizeDiff(base, measurement(mergeSha, [-1, 2048, 4096])),
  );
  assert.throws(() =>
    renderBinarySizeDiff(base, measurement(mergeSha, [1.5, 2048, 4096])),
  );
  assert.throws(() =>
    renderBinarySizeDiff(base, {
      ...merge,
      artifacts: merge.artifacts.slice(1),
    }),
  );
  assert.throws(() =>
    renderBinarySizeDiff(base, {
      ...merge,
      artifacts: [merge.artifacts[0], merge.artifacts[0], merge.artifacts[2]],
    }),
  );
});

test("binary size collection measures only the expected Linux bindings", (t) => {
  const root = mkdtempSync(join(tmpdir(), "swc-next-binary-size-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const coreDir = join(root, "npm/core/src-js");
  const parserDir = join(root, "npm/parser/src-js");
  const codegenDir = join(root, "npm/codegen/src-js");
  mkdirSync(coreDir, { recursive: true });
  mkdirSync(parserDir, { recursive: true });
  mkdirSync(codegenDir, { recursive: true });
  writeFileSync(
    join(coreDir, "swc-next-core.linux-x64-gnu.node"),
    Buffer.alloc(1024),
  );
  writeFileSync(
    join(parserDir, "swc-next-parser.linux-x64-gnu.node"),
    Buffer.alloc(2048),
  );
  writeFileSync(
    join(codegenDir, "swc-next-codegen.linux-x64-gnu.node"),
    Buffer.alloc(4096),
  );
  writeFileSync(
    join(coreDir, "swc-next-core.darwin-arm64.node"),
    Buffer.alloc(8192),
  );

  const result = measureBinarySizes(root, baseSha, rustc);
  assert.deepEqual(
    result.artifacts.map(({ bytes }) => bytes),
    [1024, 2048, 4096],
  );
  assert.equal(result.revision, baseSha);
  assert.equal(result.rustc, rustc);
  assert.ok(renderBinarySizeDiff(result, result).includes("0.000% ⚪"));

  writeFileSync(join(coreDir, "swc-next-core.linux-x64-gnu.node"), "");
  assert.throws(() => measureBinarySizes(root, baseSha, rustc));

  rmSync(join(coreDir, "swc-next-core.linux-x64-gnu.node"));
  assert.throws(() => measureBinarySizes(root, baseSha, rustc));
});
