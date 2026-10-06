import fs from "node:fs/promises";
import path from "node:path";

import { decode, encodeOptions } from "@swc-next/decoder";
import { encode as encodeTs } from "@swc-next/encoder";
import * as binding from "./binding/index.js";

const ROOT_DIR = path.resolve(import.meta.dirname, "../../..");
const FIXTURE_DIR = path.join(ROOT_DIR, "tasks/testsuite/fixtures");
const { makeUnitsFromTest } = await import(
  path.join(FIXTURE_DIR, "estree-conformance/src/utils/typescript-make-units-from-test.cjs")
);
const textEncoder = new TextEncoder();
const options = encodeOptions({ lang: "ts", comments: "none" });

export default async function runCase({ suite, file }) {
  const source = await readSource(file);
  const units = suite === "typescript"
    ? makeUnitsFromTest(file, source).tests
    : [{ name: path.basename(file), content: source }];
  return units.map((unit) => checkUnit(file, unit));
}

function checkUnit(file, unit) {
  const result = { file, name: unit.name, parsed: false, passed: false };
  const source = unit.content;
  const sourceBytes = textEncoder.encode(source);
  let parsed;
  try {
    parsed = decode(binding.parseSync(sourceBytes, source.length, options), source, sourceBytes);
    if (parsed.diagnostics.some((diagnostic) => diagnostic.severity === "error")) return result;
  } catch {
    return result;
  }
  result.parsed = true;
  try {
    const program = parsed.program;
    const native = binding.nativeParsePrintSync(source, options, 0);
    const codec = binding.printSync(encodeTs(program), 0);
    if (codec !== native) {
      return { ...result, error: "Codec output differs from native output" };
    }
    return { ...result, passed: true };
  } catch (error) {
    return { ...result, error: String(error) };
  }
}

async function readSource(file) {
  const bytes = await fs.readFile(file);
  if (bytes[0] === 0xff && bytes[1] === 0xfe) return new TextDecoder("utf-16le").decode(bytes.subarray(2));
  if (bytes[0] === 0xfe && bytes[1] === 0xff) return new TextDecoder("utf-16be").decode(bytes.subarray(2));
  return new TextDecoder().decode(bytes);
}
