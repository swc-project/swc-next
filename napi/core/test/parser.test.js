import assert from "node:assert/strict";
import test from "node:test";

import { parseSync } from "../src-js/index.js";

function parseRegExpLiterals(source) {
  const { program } = parseSync(source, {
    comments: "none",
    sourceType: "script",
  });

  const literals = [];
  const pending = [program];
  while (pending.length > 0) {
    const value = pending.pop();
    if (value?.type === "Literal" && value.regex !== undefined) {
      literals.push(value);
    }
    if (value === null || typeof value !== "object") continue;
    pending.push(...Object.values(value));
  }
  return literals;
}

function assertRegExpLiteral(source, raw, pattern, flags) {
  const start = source.indexOf(raw);
  assert.deepStrictEqual(parseRegExpLiterals(source), [
    {
      type: "Literal",
      start,
      end: start + raw.length,
      value: null,
      raw,
      regex: { pattern, flags },
    },
  ]);
}

test("adapts RegExp property escapes to ESTree literals", () => {
  assertRegExpLiteral(
    "/[\\p{Script=Greek}]/u",
    "/[\\p{Script=Greek}]/u",
    "[\\p{Script=Greek}]",
    "u",
  );
});

test("adapts Unicode Sets to ESTree literals", () => {
  assertRegExpLiteral(
    "/[[a-z]--[aeiou]]/v",
    "/[[a-z]--[aeiou]]/v",
    "[[a-z]--[aeiou]]",
    "v",
  );
});

test("adapts RegExp literals used in prototype operations", () => {
  assertRegExpLiteral(
    'RegExp.prototype.exec.call(/./vg, "x")',
    "/./vg",
    ".",
    "gv",
  );
});

test("adapts Annex B regular expressions to ESTree literals", () => {
  assertRegExpLiteral("/\\8/", "/\\8/", "\\8", "");
});
