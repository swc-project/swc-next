import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { extname } from "node:path";

const require = createRequire(import.meta.url);
const { parseSync } = require(
  process.env.SWC_CORE_ENTRY ??
    "../../../target/website-benchmark/core-js/index.js",
);
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
