import { describe, it } from "@rstest/core";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, relative, resolve, sep } from "node:path";

import { parseSync, printSync } from "./index";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const TEST262_DIR = resolve(
  __dirname,
  "../../../tasks/testsuite/fixtures/test262/test",
);

describe("parse <-> codegen idempotent", () => {
  for (const testCase of readTest262Cases(TEST262_DIR)) {
    it(testCase.name, () => {
      let ast: ReturnType<typeof parseSync>;
      try {
        ast = parseSync(testCase.code);
      } catch {
        return;
      }

      const firstPrint = printSync(ast);
      const secondPrint = printSync(parseSync(firstPrint));

      assert.equal(secondPrint, firstPrint);
    });
  }
});

// --------------------------------- Test Harness ---------------------------------
interface TestCase {
  name: string;
  code: string;
}

function readTest262Cases(dir: string): TestCase[] {
  const cases: TestCase[] = [];

  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = resolve(dir, entry.name);

    if (entry.isDirectory()) {
      cases.push(...readTest262Cases(path));
      continue;
    }

    if (!entry.isFile() || !path.endsWith(".js") || path.includes("_FIXTURE")) {
      continue;
    }

    const name = relative(TEST262_DIR, path).split(sep).join("/");
    if (name.startsWith("staging/")) {
      continue;
    }

    const code = readFileSync(path, "utf8");
    cases.push({ name, code });
  }

  cases.sort((a, b) => a.name.localeCompare(b.name));
  return cases;
}
