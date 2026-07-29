import fs from "node:fs";
import path from "node:path";

import { parseSync } from "../src-js/index.js";

const AST_MARKER = "__ESTREE_TEST__:AST:";
const TOKENS_MARKER = "__ESTREE_TEST__:TOKENS:";

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
      lang: getLang(sourceFile),
      preserveParens: true,
      sourceType: expectedAst.sourceType,
    });

    return {
      file: sourceFile,
      parsed: true,
      passed: JSON.stringify(ast, null, 2) === expected.ast,
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

function getLang(sourceFile) {
  if (sourceFile.endsWith(".tsx")) {
    return "tsx";
  }
  if (sourceFile.endsWith(".d.ts")) {
    return "dts";
  }
  if (sourceFile.endsWith(".ts")) {
    return "ts";
  }
  if (sourceFile.endsWith(".jsx")) {
    return "jsx";
  }
  return "js";
}
