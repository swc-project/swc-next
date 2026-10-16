import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { extname } from "node:path";
import { parseSync } from "@swc/core";

const [file, operation] = process.argv.slice(2);
assert(file && operation === "parse", "usage: node swc.js <file> parse");
const extension = extname(file);
const options = /\.[cm]?tsx?$/.test(extension)
  ? { syntax: "typescript", tsx: extension === ".tsx" }
  : { syntax: "ecmascript", jsx: extension === ".jsx" };
parseSync(readFileSync(file, "utf8"), {
  ...options,
  isModule: true,
  comments: false,
  target: "esnext",
});
