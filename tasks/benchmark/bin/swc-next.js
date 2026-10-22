import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { langFromPath, parseSync, walk } from "@swc-next/core";
import { bench } from "./bench.js";

const [file, iterations, operation = "parse"] = process.argv.slice(2);
assert(file, "usage: node swc-next.js <file> [iterations|--calibrate] [parse|parse-walk]");
assert(operation === "parse" || operation === "parse-walk", "unknown operation");
const source = readFileSync(file, "utf8");
const parserOptions = {
  lang: langFromPath(file),
  sourceType: "module",
  comments: "flat",
};
let identifiers = 0;
const hooks = { Identifier() { identifiers++; } };
const run = operation === "parse-walk"
  ? () => {
      identifiers = 0;
      walk(parseSync(source, parserOptions).program, hooks);
      return identifiers;
    }
  : () => parseSync(source, parserOptions).program.body.length;
{
  const initial = parseSync(source, parserOptions);
  assert.equal(initial.diagnostics.length, 0);
  if (operation === "parse-walk") walk(initial.program, hooks);
}
bench(run, iterations);
