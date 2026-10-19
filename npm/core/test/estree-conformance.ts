import path from "node:path";
import fs from "node:fs/promises";
import Tinypool from "tinypool";

// =========================== Roots ==============================
const ROOT_DIR = path.resolve(import.meta.dirname, "../../..");
const FIXTURE_DIR = path.resolve(ROOT_DIR, "tasks/testsuite/fixtures");
const ORACLE_DIR = path.resolve(FIXTURE_DIR, "estree-conformance/tests");
const SNAPSHOT_DIR = path.resolve(ROOT_DIR, "tasks/testsuite/snapshots");

// ========================== Oracles ==============================
const ORACLE_TEST262 = path.resolve(ORACLE_DIR, "test262/test");
const ORACLE_TS = path.resolve(ORACLE_DIR, "typescript/tsc/testdata/tests/cases");

// ========================== Sources ==============================
const SOURCE_TEST262 = path.resolve(FIXTURE_DIR, "test262/test");
const SOURCE_TS = path.resolve(FIXTURE_DIR, "typescript/tsc/testdata/tests/cases");

// ====================== Output Snapshots ==========================
const SNAPSHOT_TEST262 = path.resolve(SNAPSHOT_DIR, "estree_test262.snap");
const SNAPSHOT_TS = path.resolve(SNAPSHOT_DIR, "estree_typescript.snap");

// ========================= Test Runner ============================
const pool = new Tinypool({
  filename: path.resolve(import.meta.dirname, "estree-conformance.worker.js"),
});

async function runTestSuite(oracle: string, source: string, snapshot: string) {
  const oracles = await fs.readdir(oracle, { recursive: true });
  const results: TestResult[] = await Promise.all(
    oracles
      .filter((file) => file.endsWith(".json") || file.endsWith(".md"))
      // Match the Rust Test262 runner: call expressions as assignment targets
      // in non-strict mode are intentionally unsupported.
      .filter(
        (file) =>
          source !== SOURCE_TEST262 ||
          !file
            .split(path.sep)
            .join("/")
            .startsWith("annexB/language/expressions/assignmenttargettype/"),
      )
      .sort()
      .map((file) => pool.run({ oracle, source, file })),
  );
  const output = toSnapshot(results);
  const expected = await fs.readFile(snapshot, "utf8");
  await fs.mkdir(path.dirname(snapshot), { recursive: true });
  await fs.writeFile(snapshot, output);
  if (output !== expected) {
    console.error(`Snapshot changed: ${snapshot}`);
    process.exitCode = 1;
  }
}

try {
  await Promise.all([
    runTestSuite(ORACLE_TEST262, SOURCE_TEST262, SNAPSHOT_TEST262),
    runTestSuite(ORACLE_TS, SOURCE_TS, SNAPSHOT_TS),
  ]);
} finally {
  await pool.destroy();
}

// ======================== Test Result ============================
interface TestResult {
  error?: string;
  file: string;
  parsed: boolean;
  passed: boolean;
}

function toSnapshot(results: TestResult[]) {
  const total = results.length;
  const parsed = results.filter((result) => result.parsed).length;
  const passed = results.filter((result) => result.passed).length;
  const diagnostics = results
    .filter((result) => !result.passed)
    .map((result) => {
      const file = path
        .relative(ROOT_DIR, result.file)
        .split(path.sep)
        .join("/");
      return `Mismatch: ${file}`;
    })
    .sort();

  function percentage(count: number, total: number) {
    return total === 0 ? 0 : (count / total) * 100;
  }

  return [
    `AST Parsed     : ${parsed}/${total} (${percentage(parsed, total).toFixed(2)}%)`,
    `Positive Passed: ${passed}/${total} (${percentage(passed, total).toFixed(2)}%)`,
    ...diagnostics,
    "",
  ].join("\n");
}
