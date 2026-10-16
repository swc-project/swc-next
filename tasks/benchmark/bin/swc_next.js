import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { langFromPath, parseSync } from "@swc-next/core";
import { bench } from "./bench.js";

const [file, operation, iterations] = process.argv.slice(2);
assert(
  file && operation === "parse",
  "usage: node swc_next.js <file> parse [iterations|--calibrate]",
);
const source = readFileSync(file, "utf8");
const parserOptions = {
  lang: langFromPath(file),
  sourceType: "module",
  comments: "none",
};
bench(() => {
  const result = parseSync(source, parserOptions);
  assert.equal(result.diagnostics.length, 0);
  return result.program.body.length;
}, iterations);
