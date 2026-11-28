import assert from "node:assert/strict";
import test from "node:test";
import * as prettier from "prettier";
import { builders } from "prettier/doc";
import * as javascript from "@swc-next/formatter-prettier-plugin/javascript";
import * as markdown from "@swc-next/formatter-prettier-plugin/markdown";
import { encodeDoc, readDocument } from "@swc-next/protocol/document";
import { decodeDoc } from "../src-js/decode.js";

const formatWithPlugin = (source, options = {}) =>
  prettier.format(source, {
    filepath: "readme.md",
    plugins: [markdown],
    ...options,
  });

// Keep Prettier's language registry and Markdown parent context. The one
// parser override also applies whenever a child requests Markdown.
const markdownWithPrettierGuests = {
  parsers: { markdown: markdown.parsers["swc-markdown"] },
  printers: markdown.printers,
};

const formatWithPrettierGuests = (source, options = {}) =>
  prettier.format(source, {
    filepath: "readme.md",
    ...options,
    parser: "markdown",
    plugins: [markdownWithPrettierGuests],
  });

function appendLabelData(buffer, ...records) {
  const length = records.reduce((size, record) => size + 4 + record.length, 4);
  const framed = new Uint8Array(buffer.length + 4 + length + 1).subarray(1);
  const view = new DataView(
    framed.buffer,
    framed.byteOffset,
    framed.byteLength,
  );
  framed.set(buffer);
  let offset = buffer.length;
  view.setUint32(offset, length, true);
  view.setUint32(offset + 4, records.length, true);
  offset += 8;
  for (const record of records) {
    view.setUint32(offset, record.length, true);
    offset += 4;
    framed.set(record, offset);
    offset += record.length;
  }
  return framed;
}

test("reads SWCD label records and retains external label identities", () => {
  const opaque = {};
  const symbol = Symbol();
  const shared = builders.label(opaque, "doc");
  const { buffer, labels } = encodeDoc([
    shared,
    shared,
    builders.label(symbol, "other"),
  ]);
  assert.deepEqual(readDocument(buffer).labelData, []);
  const { doc: decoded, labelData: external } = decodeDoc(
    buffer,
    undefined,
    labels,
  );
  assert.deepEqual(external, []);
  assert.equal(decoded[0].label, opaque);
  assert.equal(decoded[0], decoded[1]);
  assert.equal(decoded[2].label, symbol);

  const record = Uint8Array.of(255, 0);
  const empty = new Uint8Array();
  const framed = appendLabelData(buffer, record, empty);
  const { doc, labelData } = decodeDoc(framed);
  assert.deepEqual(labelData, [record, empty]);
  assert.equal(labelData[0].buffer, framed.buffer);
  assert.equal(labelData[0].byteOffset, framed.byteOffset + buffer.length + 12);
  assert.deepEqual(
    doc.map(({ label }) => label),
    [0, 0, 1],
  );
});

test("rejects malformed SWCD label tables and invalid label references", () => {
  const label = builders.label({}, "doc");
  const { buffer } = encodeDoc(label);
  const framed = appendLabelData(buffer, Uint8Array.of(7));
  for (let end = buffer.length + 1; end < framed.length; end++) {
    assert.throws(() => readDocument(framed.subarray(0, end)));
  }
  assert.throws(() => readDocument(appendLabelData(buffer)));

  const badLength = framed.slice();
  new DataView(badLength.buffer).setUint32(buffer.length + 8, 2, true);
  assert.throws(() => readDocument(badLength));

  const trailing = new Uint8Array(framed.length + 1);
  trailing.set(framed);
  new DataView(trailing.buffer).setUint32(
    buffer.length,
    trailing.length - buffer.length - 4,
    true,
  );
  assert.throws(() => decodeDoc(trailing));

  const twoLabels = encodeDoc([label, builders.label({}, "other")]).buffer;
  assert.throws(() => decodeDoc(appendLabelData(twoLabels, new Uint8Array())));
});

function probe(parse, print, embed) {
  return {
    languages: [
      {
        name: "Probe",
        aliases: ["example"],
        extensions: [".probe"],
        parsers: ["probe"],
      },
    ],
    parsers: {
      probe: {
        astFormat: "probe",
        locStart: () => 0,
        locEnd: (node) => node.text.length,
        parse(text, options) {
          parse?.(text, options);
          return { text };
        },
      },
    },
    printers: {
      probe: { print, embed, getVisitorKeys: () => [] },
    },
  };
}

test("Markdown routing is opt-in and composes TypeScript in a quote and list", async () => {
  assert.equal(javascript.parsers["swc-markdown"], undefined);
  const source =
    "\uFEFF# Café 😀\r\n\r\n> - ```ts example\r\n>   const answer:number=42\r\n>   ```\r\n";
  const options = {
    endOfLine: "auto",
    tabWidth: 4,
    printWidth: 32,
  };
  const result = await formatWithPrettierGuests(source, options);
  assert.equal(
    result,
    "\uFEFF# Café 😀\r\n\r\n> - ```ts example\r\n>   const answer: number = 42;\r\n>   ```\r\n",
  );
  assert.equal(await formatWithPrettierGuests(result, options), result);
});

test("TypeScript and TSX use distinct virtual filenames", async () => {
  const source =
    "```ts\nconst id=<T,>(value:T)=>value\n```\n\n```tsx\nconst id=<T,>(value:T)=>value\n```\n";
  assert.equal(
    await formatWithPrettierGuests(source),
    "```ts\nconst id = <T>(value: T) => value;\n```\n\n```tsx\nconst id = <T,>(value: T) => value;\n```\n",
  );
});

test("keeps the opening fence on one line when replacing the guest Doc", async () => {
  const source = "```j&#115; flags&#10;note\nconst n=1\n```\n";
  const formatted = await formatWithPrettierGuests(source);
  assert.equal(formatted, "```j&#115; flags&#10;note\nconst n = 1;\n```\n");
  assert.equal(await formatWithPrettierGuests(formatted), formatted);
  assert.equal(
    await formatWithPrettierGuests(source, {
      embeddedLanguageFormatting: "off",
    }),
    source,
  );
});

test("passes a complete front matter document to the guest", async () => {
  const sequence = "---\n  - a\n  - b\n---\n";
  assert.equal(
    await formatWithPrettierGuests(sequence),
    "---\n- a\n- b\n---\n",
  );

  const scalar = "---\nmessage: |+\n  value  \n\n---\n";
  assert.equal(await formatWithPrettierGuests(scalar), scalar);
});

test("resolves Angular fences and honors explicitly registered languages", async () => {
  const source =
    "```angular-ts\ninterface Example{value:string}\n```\n\n```angular-html\n@if (ready) {<div>ready</div>}\n```\n";
  assert.equal(
    await formatWithPrettierGuests(source),
    await prettier.format(source, { parser: "markdown" }),
  );

  const child = probe(undefined, () => "custom");
  child.languages = ["angular-ts", "angular-html"].map((name) => ({
    ...child.languages[0],
    name,
  }));
  assert.equal(
    await formatWithPlugin(source, { plugins: [markdown, child] }),
    "```angular-ts\ncustom\n```\n\n```angular-html\ncustom\n```\n",
  );
});

test("routes an alias to the loaded plugin and replaces its empty Doc", async () => {
  const child = probe(
    (text, options) => {
      assert.equal(text, "old body");
      assert.equal(options.filepath, "embedded.probe");
    },
    () => "",
  );
  assert.equal(
    await formatWithPlugin("```example\nold body\n```\n", {
      plugins: [markdown, child],
    }),
    "```example\n\n```\n",
  );
});

test("preserves fallback fences when the guest is ignored, disabled, unknown, or fails", async () => {
  let calls = 0;
  const child = probe(
    () => {
      calls++;
      throw new Error("guest parse failed");
    },
    () => "unreachable",
  );
  const options = { plugins: [markdown, child] };
  const source = "```example\nold  body\n```\n";
  const disabled = { ...options, embeddedLanguageFormatting: "off" };
  const root = await markdown.parsers["swc-markdown"].parse(source, disabled);
  assert.deepEqual(root.embeds, []);
  assert.equal(await formatWithPlugin(source, disabled), source);
  assert.equal(
    await formatWithPlugin("<!-- prettier-ignore -->\n\n" + source, options),
    "<!-- prettier-ignore -->\n" + source,
  );
  const unknown = "```unregistered-guest\nold  body\n```\n";
  assert.equal(await formatWithPlugin(unknown, options), unknown);
  assert.equal(calls, 0);
  assert.equal(await formatWithPlugin(source, options), source);
  assert.equal(calls, 1);
});

test("chooses delimiters from the guest Doc and the fence info", async () => {
  const child = probe(undefined, ({ node }) =>
    node.text === "collision"
      ? builders.group(["``", builders.ifBreak("`", "x"), "`"], {
          shouldBreak: true,
        })
      : "replacement",
  );
  const options = { plugins: [markdown, child] };
  assert.equal(
    await formatWithPlugin("```probe\ncollision\n```\n", options),
    "`````probe\n````\n`````\n",
  );
  const shortened = await formatWithPlugin(
    "``````probe\nold\n``````\n",
    options,
  );
  assert.equal(shortened, "```probe\nreplacement\n```\n");
  assert.equal(await formatWithPlugin(shortened, options), shortened);
  assert.equal(
    await formatWithPlugin("~~~probe ticks=`value`\nold\n~~~\n", options),
    "~~~probe ticks=`value`\nreplacement\n~~~\n",
  );
});

test("shares the host's width and indentation with the child formatter", async () => {
  const source =
    "alpha beta gamma delta epsilon zeta\n\n```js\nif (ready) { run() }\n```\n";
  assert.equal(
    await formatWithPrettierGuests(source, {
      proseWrap: "always",
      printWidth: 20,
      tabWidth: 4,
      useTabs: true,
    }),
    "alpha beta gamma\ndelta epsilon zeta\n\n```js\nif (ready) {\n\trun();\n}\n```\n",
  );
  assert.equal(
    await formatWithPrettierGuests('[link](target "title")\n', {
      singleQuote: true,
    }),
    "[link](target 'title')\n",
  );
});

test("bounds nested Markdown reentry through another plugin", async () => {
  let calls = 0;
  const source = "```probe\nnested\n```\n";
  const child = probe(
    () => {
      if (++calls > 8) throw new Error("depth budget was lost");
    },
    () => "",
    () => (textToDoc) => textToDoc(source, { parser: "swc-markdown" }),
  );
  const result = await formatWithPlugin(source, { plugins: [markdown, child] });
  assert.equal(calls, 8);
  assert(result.includes("\nnested\n"));
});
