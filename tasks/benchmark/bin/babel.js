import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { extname } from "node:path";
import { parse } from "@babel/parser";
import { bench } from "./bench.js";

const [file, iterations] = process.argv.slice(2);
assert(file, "usage: node babel.js <file> [iterations|--calibrate]");
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
parse(source, parserOptions);
bench(() => parse(source, parserOptions).program.body.length, iterations);
