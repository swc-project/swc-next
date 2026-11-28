import assert from "node:assert/strict";
import test from "node:test";
import { printDocBufferSync } from "@swc-next/formatter";
import { readDocument } from "@swc-next/protocol/document";
import { formatMarkdownDocBufferSync } from "../dist/binding.js";
import {
  decodeEmbeddedBlocks,
  formatMarkdownDocBuffer,
} from "../src-js/markdown/format.js";

function formatDocSync(source, options) {
  const doc = formatMarkdownDocBuffer(
    formatMarkdownDocBufferSync,
    source,
    options,
  );
  const embeds = decodeEmbeddedBlocks(readDocument(doc).labelData, source);
  return { doc, embeds };
}

test("returns labeled documents and UTF-16 fence spans", () => {
  const source =
    "文😀\r\n\r\n```ts example\r\nconst café=1\r\n```\r\n\r\n~~~text\r\nλ\r\n~~~";
  const { doc, embeds } = formatDocSync(source, {
    proseWrap: "always",
    lineWidth: 31,
    indentStyle: "tab",
    indentWidth: 4,
    lineEnding: "crlf",
  });
  assert(doc instanceof Uint8Array);
  assert.equal(new TextDecoder().decode(doc.subarray(0, 4)), "SWCD");
  assert.deepEqual(embeds, [
    {
      start: source.indexOf("```ts"),
      end: source.indexOf("```\r\n") + 3,
      language: "ts",
      meta: "example",
      body: "const café=1",
    },
    {
      start: source.indexOf("~~~text"),
      end: source.length,
      language: "text",
      meta: undefined,
      body: "λ",
    },
  ]);
});

test("validates Markdown options before invoking the binding", () => {
  assert.throws(() => formatDocSync(123), TypeError);
  assert.throws(() => formatDocSync("", { proseWrap: "unknown" }), TypeError);
  assert.throws(() => formatDocSync("", { includeEmbeds: "off" }), TypeError);
  assert.throws(() => formatDocSync("", { indentWidth: 65 }), RangeError);
});

test("keeps the printable fence info alongside decoded fields", () => {
  const source = "```js label&#10;日本\nbody\n```\n";
  const {
    embeds: [fence],
  } = formatDocSync(source);
  assert.equal(fence.language, "js");
  assert.equal(fence.meta, "label\n日本");
  assert.equal(fence.sourceInfo, "js label&#10;日本");

  const {
    embeds: [escaped],
  } = formatDocSync("```ab&#32;cd note=&amp;copy;\nbody\n```\n");
  assert.equal(escaped.language, "ab cd");
  assert.equal(escaped.meta, "note=&copy;");
  assert.equal(escaped.sourceInfo, "ab&#32;cd note=&amp;copy;");
});

test("indexes front matter and fences in the same document", () => {
  const source = "---\r\nname: 文😀\r\n...  \r\n\r\n```js\r\n42\r\n```\r\n";
  const { doc, embeds } = formatDocSync(source);
  const buffer = formatMarkdownDocBufferSync(
    source,
    0,
    false,
    80,
    2,
    false,
    0,
    true,
  );
  assert.deepEqual(doc, buffer);
  assert.equal(new DataView(doc.buffer, doc.byteOffset).getUint32(4, true), 0);
  assert.equal(readDocument(doc).labelData.length, 2);
  assert.equal(
    printDocBufferSync(doc),
    "---\nname: 文😀\n...\n\n```js\n42\n```\n",
  );
  assert.deepEqual(embeds[0], {
    start: 0,
    end: source.indexOf("...") + 3,
    language: "yaml",
    body: "name: 文😀\n",
    opening: "---",
    closing: "...",
  });
  assert.equal(embeds[1].start, source.indexOf("```js"));

  const fallback = formatDocSync(source, { includeEmbeds: false });
  assert.deepEqual(fallback.embeds, []);
  assert.deepEqual(readDocument(fallback.doc).labelData, []);
  assert.equal(printDocBufferSync(fallback.doc), printDocBufferSync(doc));
});

test("omits the label table for plain Markdown", () => {
  const { doc, embeds } = formatDocSync("# title\n");
  const { stringPool, labelData } = readDocument(doc);
  assert.deepEqual(embeds, []);
  assert.deepEqual(labelData, []);
  assert.equal(
    stringPool.byteOffset + stringPool.byteLength,
    doc.byteOffset + doc.byteLength,
  );
  assert.equal(printDocBufferSync(doc), "# title\n");
});
