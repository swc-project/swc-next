import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { extname } from "node:path";
import { parseSync } from "@swc/core";
import { bench } from "./bench.js";

const [file, iterations] = process.argv.slice(2);
assert(file, "usage: node swc.js <file> [iterations|--calibrate]");
const source = readFileSync(file, "utf8");
const extension = extname(file);
const options = /\.[cm]?tsx?$/.test(extension)
  ? { syntax: "typescript", tsx: extension === ".tsx" }
  : { syntax: "ecmascript", jsx: extension === ".jsx" };
const parserOptions = {
  ...options,
  isModule: true,
  comments: false,
  target: "esnext",
};
parseSync(source, parserOptions);
bench(() => parseSync(source, parserOptions).body.length, iterations);
