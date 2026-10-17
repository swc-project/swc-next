import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { langFromPath, parseSync } from "@swc-next/core";
import { bench } from "./bench.js";

const [file, iterations] = process.argv.slice(2);
assert(file, "usage: node swc-next.js <file> [iterations|--calibrate]");
const source = readFileSync(file, "utf8");
const parserOptions = {
  lang: langFromPath(file),
  sourceType: "module",
  comments: "none",
};
assert.equal(parseSync(source, parserOptions).diagnostics.length, 0);
bench(() => parseSync(source, parserOptions).program.body.length, iterations);
