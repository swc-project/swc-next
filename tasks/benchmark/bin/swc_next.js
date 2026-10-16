import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { langFromPath, parseSync } from "../../../npm/core/src-js/index.js";

const [file, operation] = process.argv.slice(2);
assert(file && operation === "parse", "usage: node swc_next.js <file> parse");
const result = parseSync(readFileSync(file, "utf8"), {
  lang: langFromPath(file),
  sourceType: "module",
  comments: "none",
});
assert.equal(result.diagnostics.length, 0);
result.program;
