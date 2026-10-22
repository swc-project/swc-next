import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { extname } from "node:path";
import { parseSync } from "@swc/core";

import { measure } from "./measure.js";

const [file, operation = "parse", rounds = "100", iterations = "1"] = process.argv.slice(2);
assert(file, "usage: node swc.js <file> [parse|parse-walk] [rounds] [iterations-per-round]");
assert(operation === "parse" || operation === "parse-walk", "unknown operation");
const source = readFileSync(file, "utf8");
const extension = extname(file);
const options = /\.[cm]?tsx?$/.test(extension)
  ? { syntax: "typescript", tsx: extension === ".tsx" }
  : { syntax: "ecmascript", jsx: extension === ".jsx" };
const parserOptions = {
  ...options,
  isModule: true,
  comments: true,
  target: "esnext",
};
let visitor;
if (operation === "parse-walk") {
  assert.equal(options.syntax, "ecmascript", "SWC's JS Visitor does not support TypeScript types");
  const { Visitor } = await import("@swc/core/Visitor.js");
  visitor = new class extends Visitor {
    identifiers = 0;
    visitIdentifier(node) {
      this.identifiers++;
      return node;
    }
  }();
}
await measure(visitor
  ? () => {
      visitor.identifiers = 0;
      visitor.visitProgram(parseSync(source, parserOptions));
      return visitor.identifiers;
    }
  : () => parseSync(source, parserOptions).body.length, Number(rounds), Number(iterations));
