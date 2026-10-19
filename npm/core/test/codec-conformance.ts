import fs from "node:fs/promises";
import path from "node:path";
import Tinypool from "tinypool";

const ROOT_DIR = path.resolve(import.meta.dirname, "../../..");
const FIXTURE_DIR = path.join(ROOT_DIR, "tasks/testsuite/fixtures");
const ORACLE_DIR = path.join(FIXTURE_DIR, "estree-conformance/tests");
const SNAPSHOT_DIR = path.join(ROOT_DIR, "tasks/testsuite/snapshots");

const ORACLE_TEST262 = path.join(ORACLE_DIR, "test262/test");
const ORACLE_TS = path.join(ORACLE_DIR, "typescript/tsc/testdata/tests/cases");
const SOURCE_TEST262 = path.join(FIXTURE_DIR, "test262/test");
const SOURCE_TS = path.join(FIXTURE_DIR, "typescript/tsc/testdata/tests/cases");
const SNAPSHOT_TEST262 = path.join(SNAPSHOT_DIR, "codec_test262.snap");
const SNAPSHOT_TS = path.join(SNAPSHOT_DIR, "codec_typescript.snap");

interface TestResult {
  file: string;
  name: string;
  parsed: boolean;
  passed: boolean;
  error?: string;
}

// Fail before dispatching fixtures if a production binding was used by mistake.
const binding = await import("./binding/index.js");
if (typeof binding.nativeParsePrintSync !== "function") {
  throw new Error("Build core with --features test-codec (or run pnpm test:codec)");
}

const pool = new Tinypool({
  filename: path.join(import.meta.dirname, "codec-conformance.worker.js"),
  // Deep expression fixtures need more stack than the worker default.
  resourceLimits: { stackSizeMb: 16 },
});

async function runTestSuite(suite: string, oracle: string, source: string, snapshot: string) {
  const files = await fs.readdir(oracle, { recursive: true });
  const results: TestResult[] = (
    await Promise.all(
      files
        .filter((file) => file.endsWith(".json") || file.endsWith(".md"))
        .sort()
        .map((file) => {
          const sourceFile = file.endsWith(".json")
            ? file.slice(0, -".json".length) + ".js"
            : file.slice(0, -".md".length);
          return pool.run({ suite, file: path.join(source, sourceFile), oracle: path.join(oracle, file) });
        }),
    )
  ).flat().filter((result) => result.parsed || result.error !== undefined);
  await writeResults(suite, snapshot, results);
}

try {
  await Promise.all([
    runTestSuite("test262", ORACLE_TEST262, SOURCE_TEST262, SNAPSHOT_TEST262),
    runTestSuite("typescript", ORACLE_TS, SOURCE_TS, SNAPSHOT_TS),
  ]);
} finally {
  await pool.destroy();
}

async function writeResults(suite: string, snapshot: string, results: TestResult[]) {
  const failed = results.filter((result) => !result.passed);
  const passed = results.length - failed.length;
  const lines = [
    `Cases         : ${results.length}`,
    `Codec Passed  : ${passed}/${results.length}`,
  ];
  const failures: string[] = [];
  for (const result of failed) {
    const file = path.relative(ROOT_DIR, result.file).split(path.sep).join("/");
    failures.push(`${result.error ?? "Mismatch"}: ${file} (${result.name})`);
  }
  const output = [...lines, ...failures.sort(), ""].join("\n");
  const expected = await fs.readFile(snapshot, "utf8");
  await fs.mkdir(SNAPSHOT_DIR, { recursive: true });
  await fs.writeFile(snapshot, output);
  console.log(`${suite}: ${passed}/${results.length} passed, ${failed.length} failed cases`);
  if (output !== expected) {
    console.error(`Snapshot changed: ${snapshot}`);
    process.exitCode = 1;
  }
}
