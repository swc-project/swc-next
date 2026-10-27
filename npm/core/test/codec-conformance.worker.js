import fs from "node:fs/promises";
import path from "node:path";
import { readExpected } from "./conformance-utils.js";

import { decode, encodeOptions } from "@swc-next/decoder";
import { encode } from "@swc-next/encoder";
import * as binding from "./binding/index.js";

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
  const options = encodeOptions({
    lang: getLang(unit, suite),
    sourceType: sourceType ?? (unit.sourceType.module ? "module" : "script"),
    preserveParens: suite !== "typescript",
    comments: "none",
  });
  let parsed;
  try {
    parsed = decode(binding.parseSync(sourceBytes, source.length, options), source, sourceBytes);
    if (parsed.diagnostics.some((diagnostic) => diagnostic.severity === "error")) return result;
  } catch (error) {
    return { ...result, error: String(error) };
  }
  result.parsed = true;
  try {
    const program = parsed.program;
    const native = binding.nativeParsePrintSync(source, options, 0);
    const codec = binding.printSync(...encode(program, parsed.meta), 0);
    if (codec !== native) {
      return { ...result, error: "Codec output differs from native output" };
    }
    return { ...result, passed: true };
  } catch (error) {
    return { ...result, error: String(error) };
  }
}

function getLang(unit, suite) {
  let lang = binding.langFromPath(unit.name);
  if (suite === "typescript" && lang !== "dts") lang = "ts";
  if (!unit.sourceType.jsx) return lang;
  if (lang === "ts" || lang === "dts") return "tsx";
  if (lang === "js") return "jsx";
  return lang;
}

async function readSource(file) {
  const bytes = await fs.readFile(file);
  if (bytes[0] === 0xff && bytes[1] === 0xfe) return new TextDecoder("utf-16le").decode(bytes.subarray(2));
  if (bytes[0] === 0xfe && bytes[1] === 0xff) return new TextDecoder("utf-16be").decode(bytes.subarray(2));
  return new TextDecoder().decode(bytes);
}
