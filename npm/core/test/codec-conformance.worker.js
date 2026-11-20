import fs from "node:fs/promises";
import path from "node:path";
import { readExpected } from "./conformance-utils.js";

import { readAst } from "@swc-next/ast";
import { encodeParserConfig } from "@swc-next/protocol/parser";
import assert from "node:assert/strict";
import * as binding from "../dist/binding.js";

const ROOT_DIR = path.resolve(import.meta.dirname, "../../..");
const FIXTURE_DIR = path.join(ROOT_DIR, "tasks/testsuite/fixtures");
const { makeUnitsFromTest } = await import(
  path.join(FIXTURE_DIR, "estree-conformance/src/utils/typescript-make-units-from-test.ts")
);
const textEncoder = new TextEncoder();

export default async function runCase({ suite, file, oracle }) {
  const source = await readSource(file);
  const expectedUnits = readExpected(oracle).map((expected) => JSON.parse(expected.ast));
  const units = suite === "typescript"
    ? makeUnitsFromTest(file, source).tests
    : [{
        name: path.basename(file),
        content: source,
        sourceType: { jsx: false },
      }];
  return units.map((unit, index) => checkUnit(file, unit, suite, expectedUnits[index]?.sourceType));
}

function checkUnit(file, unit, suite, sourceType) {
  const result = { file, name: unit.name, parsed: false, passed: false };
  const source = unit.content;
  const sourceBytes = textEncoder.encode(source);
  const options = encodeParserConfig({
    lang: getLang(unit, suite),
    sourceType: sourceType ?? (unit.sourceType.module ? "module" : "script"),
    preserveParens: suite !== "typescript",
    comments: "none",
  });
  let parsed;
  try {
    parsed = readAst(binding.parseSync(sourceBytes, options));
    if (parsed.diagnostics.some((diagnostic) => diagnostic.severity === "error")) { return result; }
  } catch (error) {
    return { ...result, error: String(error) };
  }
  result.parsed = true;
  try {
    const [buffer, extension] = parsed.encode();
    assert.equal(extension.length, 0);
    const decoded = readAst(buffer.slice());
    assert.deepEqual(decoded.materialize(), parsed.materialize());
    assert.deepEqual(decoded.comments, parsed.comments);
    assert.equal(binding.generateSync(...decoded.encode(), 0), binding.generateSync(buffer, extension, 0));
    return { ...result, passed: true };
  } catch (error) {
    return { ...result, error: String(error) };
  }
}

function getLang(unit, suite) {
  let lang = binding.langFromPath(unit.name);
  if (suite === "typescript" && lang !== "dts") { lang = "ts"; }
  if (!unit.sourceType.jsx) { return lang; }
  if (lang === "ts" || lang === "dts") { return "tsx"; }
  if (lang === "js") { return "jsx"; }
  return lang;
}

async function readSource(file) {
  const bytes = await fs.readFile(file);
  if (bytes[0] === 0xff && bytes[1] === 0xfe) { return new TextDecoder("utf-16le").decode(bytes.subarray(2)); }
  if (bytes[0] === 0xfe && bytes[1] === 0xff) { return new TextDecoder("utf-16be").decode(bytes.subarray(2)); }
  return new TextDecoder().decode(bytes);
}
