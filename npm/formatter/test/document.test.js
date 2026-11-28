import assert from "node:assert/strict";
import test from "node:test";
import {
  printDocBufferSync,
  printDocBufferWithCursorSync,
} from "@swc-next/formatter";
import { encodeDoc } from "@swc-next/formatter/document";

test("document printing forwards layout options and preserves input strings", () => {
  const { buffer } = encodeDoc({
    type: "group",
    contents: ["λ", { type: "indent", contents: [{ type: "line" }, "世界"] }],
  });
  assert.equal(printDocBufferSync(buffer), "λ 世界");
  assert.equal(
    printDocBufferSync(buffer, {
      lineWidth: 2,
      indentWidth: 4,
      indentStyle: "space",
      lineEnding: "crlf",
    }),
    "λ\r\n    世界",
  );
  assert.equal(
    printDocBufferSync(buffer, { lineWidth: 2, indentStyle: "tab" }),
    "λ\n\t世界",
  );

  const shared = new Uint8Array(new SharedArrayBuffer(buffer.length));
  shared.set(buffer);
  assert.equal(printDocBufferSync(shared), "λ 世界");

  const bom = encodeDoc("\uFEFFx").buffer;
  assert.equal(printDocBufferSync(bom), "\uFEFFx");
});

test("document printing forwards document and option errors", () => {
  assert.throws(
    () => printDocBufferSync(Uint8Array.of(1)),
    /invalid|document/i,
  );
  const unsupportedVersion = encodeDoc("").buffer;
  new DataView(unsupportedVersion.buffer).setUint32(4, 1, true);
  assert.throws(() => printDocBufferSync(unsupportedVersion), /version/i);
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { lineWidth: 65536 }),
    RangeError,
  );
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { indentWidth: 65 }),
    RangeError,
  );
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { indentStyle: 1 }),
    TypeError,
  );
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { lineEnding: "auto" }),
    TypeError,
  );
});

test("document encoding rejects cycles and isolated surrogates", () => {
  const cyclic = [];
  cyclic.push({ type: "indent", contents: cyclic });
  assert.throws(() => encodeDoc(cyclic), TypeError);
  assert.throws(() => encodeDoc("\ud800"), TypeError);
  assert.throws(
    () => encodeDoc({ type: "align", n: "\udc00", contents: "x" }),
    TypeError,
  );
});

test("document encoding rejects invalid alignment values", () => {
  const align = (n) => ({ type: "align", n, contents: "x" });
  assert.throws(() => encodeDoc(align(NaN)), RangeError);
  assert.throws(() => encodeDoc(align(Infinity)), RangeError);
  assert.throws(() => encodeDoc(align(0.5)), RangeError);
  assert.throws(() => encodeDoc(align(0x1_0000_0000)), RangeError);
});

test("cursor positions are reported in UTF-16 after trimming and line ending conversion", () => {
  const cursor = { type: "cursor" };
  const { buffer } = encodeDoc([
    "😀 ",
    cursor,
    { type: "trim" },
    { type: "line", hard: true, literal: true },
    "中",
    cursor,
  ]);
  const options = { lineEnding: "crlf" };
  assert.deepEqual(printDocBufferWithCursorSync(buffer, options), {
    formatted: "😀\r\n中",
    cursorNodeStart: 2,
    cursorNodeText: "\r\n中",
  });

  const single = encodeDoc(["😀", cursor]).buffer;
  assert.deepEqual(printDocBufferWithCursorSync(single), { formatted: "😀" });

  const extra = encodeDoc([
    cursor,
    cursor,
    { type: "line-suffix", contents: cursor },
  ]).buffer;
  assert.equal(printDocBufferSync(extra), "");
  assert.throws(() => printDocBufferWithCursorSync(extra), /cursor/i);
});
