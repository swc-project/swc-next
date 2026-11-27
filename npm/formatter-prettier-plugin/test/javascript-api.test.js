import assert from "node:assert/strict";
import test from "node:test";
import { printDocBufferSync } from "@swc-next/formatter";
import { formatJsDocBufferSync } from "../dist/binding.js";
import { formatJsDoc } from "../src-js/javascript/format.js";

const formatDocBufferSync = (source, parser, options) =>
  formatJsDoc(formatJsDocBufferSync, source, parser, options);

const script = { sourceType: "script" };
const syntax = { semicolons: "as-needed", quoteStyle: "single" };

test("configuration objects cross the formatter binding", () => {
  const source = 'const text = "λ😀";';
  const buffer = formatDocBufferSync(source, script, syntax);
  assert.equal(printDocBufferSync(buffer), "const text = 'λ😀'\n");

  const defaults = formatDocBufferSync(source);
  const omitted = formatDocBufferSync(source, null, null);
  assert.deepEqual(defaults, omitted);

  const empty = formatDocBufferSync(source, {}, {});
  assert.deepEqual(defaults, empty);

  const typescript = formatDocBufferSync("let value: number;", { lang: "ts" });
  assert.equal(printDocBufferSync(typescript), "let value: number;\n");
});

test("invalid API options are rejected before formatting", () => {
  assert.throws(() => formatDocBufferSync(1), TypeError);
  assert.throws(() => formatDocBufferSync("", 1), TypeError);
  assert.throws(() => formatDocBufferSync("", script, []), TypeError);
  assert.throws(
    () => formatDocBufferSync("", script, { semicolons: false }),
    TypeError,
  );
  assert.throws(
    () => formatDocBufferSync("", script, { bracketSpacing: 1 }),
    TypeError,
  );
  assert.throws(
    () => formatDocBufferSync("", script, { quoteStyle: "invalid" }),
    TypeError,
  );
  assert.throws(
    () => formatDocBufferSync("", script, { indentWidth: 65 }),
    RangeError,
  );
});

test("source failures expose UTF-16 offsets without leaking error buffers", () => {
  const source = '"😀"; const = 1;';
  assert.throws(
    () => formatDocBufferSync(source),
    (error) => {
      assert.ok(error instanceof SyntaxError);
      assert.equal(source.slice(error.start, error.end), "=");
      assert.equal(error.loc, undefined);
      return true;
    },
  );
});
