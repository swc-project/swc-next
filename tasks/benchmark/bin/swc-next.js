import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { langFromPath, parseSync, walk } from "@swc-next/core";

import { measure } from "./measure.js";

const [file, operation = "parse", rounds = "100", iterations = "1"] = process.argv.slice(2);
assert(file, "usage: node swc-next.js <file> [parse|parse-walk] [rounds] [iterations-per-round]");
assert(operation === "parse" || operation === "parse-walk", "unknown operation");
const source = readFileSync(file, "utf8");
const parserOptions = {
  lang: langFromPath(file),
  sourceType: "module",
  comments: "flat",
};
{
  const result = parseSync(source, parserOptions);
  assert.equal(result.diagnostics.length, 0);
}
let identifiers = 0;
const hooks = { Identifier() { identifiers++; } };
await measure(operation === "parse-walk"
  ? () => {
      identifiers = 0;
      walk(parseSync(source, parserOptions).program, hooks);
      return identifiers;
    }
  : () => parseSync(source, parserOptions).program.body.length, Number(rounds), Number(iterations));
