import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { extname } from "node:path";
import { parse } from "@babel/parser";
import { bench } from "./bench.js";

const [file, iterations, operation = "parse"] = process.argv.slice(2);
assert(file, "usage: node babel.js <file> [iterations|--calibrate] [parse|parse-walk]");
assert(operation === "parse" || operation === "parse-walk", "unknown operation");
const source = readFileSync(file, "utf8");
const extension = extname(file);
const plugins = [];
if (/\.[cm]?tsx?$/.test(extension)) plugins.push("typescript");
if (extension === ".jsx" || extension === ".tsx") plugins.push("jsx");
const parserOptions = {
  plugins,
  sourceType: "module",
  // Babel always collects comments; skip attaching them to AST nodes.
  attachComment: false,
  createParenthesizedExpressions: true,
  // Keep recoverable early errors from rejecting fixtures accepted by SWC.
  errorRecovery: true,
};
const traverse = operation === "parse-walk"
  ? (await import("@babel/traverse")).default
  : null;
let identifiers = 0;
const visitor = {
  noScope: true,
  Identifier() { identifiers++; },
};
const run = traverse
  ? () => {
      identifiers = 0;
      traverse(parse(source, parserOptions), visitor);
      return identifiers;
    }
  : () => parse(source, parserOptions).program.body.length;
run();
bench(run, iterations);
