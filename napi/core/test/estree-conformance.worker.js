import fs from "node:fs";
import path from "node:path";

import { langFromPath, parseSync } from "../src-js/index.js";

// =========================== Paths ==============================
const ROOT_DIR = path.resolve(import.meta.dirname, "../../..");
const FIXTURE_DIR = path.resolve(ROOT_DIR, "tasks/testsuite/fixtures");
const TEST_HARNESS = path.resolve(FIXTURE_DIR, "estree-conformance/src");
const FIELD_ORDERS_FILE = path.resolve(TEST_HARNESS, "field-orders.json");

// ========================= Runner =============================
export default async function runCase({ oracle, source, file }) {
  const oracleFile = path.resolve(oracle, file);
  const sourceFile = getSourceFile(source, file);

  try {
    const expected = readExpected(oracleFile);
    const expectedAst = JSON.parse(expected.ast);
    // Parse the tokens now as well so malformed token fixtures are reported.
    // Token conformance will be compared separately once token output is exposed.
    JSON.parse(expected.tokens);

    const sourceCode = fs.readFileSync(sourceFile, "utf8");
    const { program: ast } = parseSync(sourceCode, {
      comments: "none",
      lang: langFromPath(sourceFile),
      preserveParens: true,
      sourceType: expectedAst.sourceType,
    });

    return {
      file: sourceFile,
      parsed: true,
      passed: JSON.stringify(ast, reorderAstFields, 2) === expected.ast,
    };
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : String(error),
      file: sourceFile,
      parsed: false,
      passed: false,
    };
  }
}

// ========================= Adapters =============================
const AST_MARKER = "__ESTREE_TEST__:AST:";
const TOKENS_MARKER = "__ESTREE_TEST__:TOKENS:";

const fieldOrders = JSON.parse(fs.readFileSync(FIELD_ORDERS_FILE, "utf8"));

function reorderAstFields(_key, value) {
  if (
    typeof value !== "object" ||
    value === null ||
    !Object.hasOwn(value, "type")
  ) {
    return value;
  }

  const keys = fieldOrders[value.type];
  if (!keys) return value;

  const reordered = { type: value.type };
  for (const key of keys) {
    if (Object.hasOwn(value, key)) reordered[key] = value[key];
  }
  for (const key of Object.keys(value)) {
    if (
      key === "type" ||
      key === "start" ||
      key === "end" ||
      keys.includes(key)
    )
      continue;
    reordered[key] = value[key];
  }
  if (Object.hasOwn(value, "start")) reordered.start = value.start;
  if (Object.hasOwn(value, "end")) reordered.end = value.end;

  return reordered;
}

function readExpected(oracleFile) {
  const contents = fs.readFileSync(oracleFile, "utf8");
  switch (path.extname(oracleFile)) {
    case ".json":
      return { ast: contents, tokens: "[]" };
    case ".md":
      return {
        ast: readJsonBlock(contents, AST_MARKER, oracleFile),
        tokens: readJsonBlock(contents, TOKENS_MARKER, oracleFile),
      };
    default:
      throw new Error(`Unsupported oracle file: ${oracleFile}`);
  }

  function readJsonBlock(markdown, marker, file) {
    const markerIndex = markdown.indexOf(marker);
    if (markerIndex === -1) {
      throw new Error(`Missing ${marker} in ${file}`);
    }

    const block = markdown
      .slice(markerIndex + marker.length)
      .match(/^\s*```json\r?\n([\s\S]*?)\r?\n```/);
    if (!block) {
      throw new Error(`Invalid JSON block after ${marker} in ${file}`);
    }

    return block[1];
  }
}

function getSourceFile(source, file) {
  switch (path.extname(file)) {
    case ".json":
      return path.resolve(source, file.slice(0, -".json".length) + ".js");
    case ".md":
      return path.resolve(source, file.slice(0, -".md".length));
    default:
      throw new Error(`Unsupported oracle file: ${file}`);
  }
}
