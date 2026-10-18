import assert from "node:assert/strict";
import test from "node:test";
import * as prettier from "prettier";
import { builders } from "prettier/doc";
import { formatDocBufferSync } from "@swc-next/formatter";
import * as plugin from "../index.js";
import { decode } from "../decode.js";

const js = { parser: "swc", plugins: [plugin] };
const ts = { ...js, parser: "swc-ts" };
const tsx = { ...js, parser: "swc-tsx" };

async function matches(source, options = js, extra = {}) {
  const nativeParser = options.parser === "swc" ? "babel" : "typescript";
  const expected = await prettier.format(source, {
    ...extra,
    parser: nativeParser,
  });
  const actual = await prettier.format(source, { ...options, ...extra });
  assert.equal(actual, expected);
  assert.equal(await prettier.format(actual, { ...options, ...extra }), actual);
}

test("SWCFMT formats programs using Prettier documents", async () => {
  assert.equal(plugin.parsers["swc"].astFormat, "SWCFMT");
  const doc = builders.group(["a", builders.line, "b"]);
  assert.equal(plugin.printers.SWCFMT.print({ node: { doc } }), doc);
  await matches("const x={a:1,b:[2,3]};");
  await matches(
    "const x = (a, b, c) => ({ alpha: a, beta: b, gamma: c });",
    js,
    { printWidth: 25 },
  );
});

test("SWCFMT transports groups, alternatives and conditional layouts", async () => {
  await matches(
    "const result = foo.bar().baz(one, two, three).qux(four, five);",
    js,
    { printWidth: 25 },
  );
  await matches(
    "type T = Foo | Bar | Baz | Qux; const value = input as Foo | Bar | Baz | Qux;",
    ts,
    { printWidth: 22 },
  );
  await matches("type T<A> = A extends string ? {foo:A} : {bar:A};", ts, {
    printWidth: 30,
  });
});

test("SWCFMT preserves comments, ignored source and template text", async () => {
  await matches("// header\nconst x=1; // end\n");
  await matches("// prettier-ignore\nconst x = {  a:1 };\n");
  await matches("const x=`a\n  b ${foo + bar}\nc`;\n");
  await matches(
    "const x = <div>hello world here and there and everywhere</div>;",
    tsx,
    { printWidth: 25 },
  );
});

test("SWCFMT selects source type from module and CommonJS extensions", async () => {
  const formatFile = (source, filepath) =>
    prettier.format(source, { plugins: [plugin], filepath });

  await assert.rejects(formatFile("return 1;", "file.mjs"), SyntaxError);
  await assert.rejects(formatFile("const await = 1;", "file.mts"), SyntaxError);
  await assert.rejects(formatFile("return 1;", "file.d.mts"), SyntaxError);

  assert.equal(await formatFile("return 1;", "file.cjs"), "return 1;\n");
  assert.equal(await formatFile("return 1;", "file.cts"), "return 1;\n");
  assert.equal(
    await formatFile("const await = 1;", "file.js"),
    "const await = 1;\n",
  );
  assert.equal(
    await formatFile("const await = 1;", "file.ts"),
    "const await = 1;\n",
  );

  assert.equal(
    await formatFile("export const x=1;", "file.mjs"),
    "export const x = 1;\n",
  );
  assert.equal(
    await formatFile("export const x:number=1;", "file.mts"),
    "export const x: number = 1;\n",
  );
  assert.equal(
    await formatFile("export declare const x:number;", "file.d.mts"),
    "export declare const x: number;\n",
  );
});

test("SWCFMT follows formatting options and file inference", async () => {
  await matches('const x = {foo:"bar", nested:[1,2]};', js, {
    semi: false,
    singleQuote: true,
    trailingComma: "none",
    useTabs: true,
    printWidth: 20,
  });
  assert.equal(
    await prettier.format("const x:number=1", {
      plugins: [plugin],
      filepath: "file.ts",
    }),
    "const x: number = 1;\n",
  );
  assert.equal(
    await prettier.format("declare const x:number;", {
      plugins: [plugin],
      filepath: "file.d.ts",
    }),
    "declare const x: number;\n",
  );
  await matches("#!/usr/bin/env node\nconst x=1;", js);
  await matches('const x = {"a-b": 1, plain: 2};', js, {
    quoteProps: "consistent",
  });
  await matches('const x = <Box a="x" b={value} />;', tsx, {
    singleAttributePerLine: true,
  });
});

test("SWCFMT leaves line endings and BOM to Prettier", async () => {
  await matches("\uFEFFconst x=1;\r\nconst y=2;\r\n", js, {
    endOfLine: "auto",
  });
  await matches("const x=1;\nconst y=2;\n", js, { endOfLine: "cr" });
  await matches("", js);
});

test("SWCFMT forwards line endings during JSX document construction", async () => {
  assert.equal(
    await prettier.format('<Box title="a\nb" />;', { ...js, endOfLine: "lf" }),
    '<Box\n  title="a\nb"\n/>;\n',
  );
  assert.equal(
    await prettier.format('<Box title="a\nb" />;', {
      ...js,
      endOfLine: "crlf",
    }),
    '<Box\r\n  title="a&#10;b"\r\n/>;\r\n',
  );
  assert.equal(
    await prettier.format('<Box title="a\nb" />;', { ...js, endOfLine: "cr" }),
    '<Box\r  title="a&#10;b"\r/>;\r',
  );
  assert.equal(
    await prettier.format('<Box title="a\r\nb" />;', {
      ...js,
      endOfLine: "auto",
    }),
    '<Box\r\n  title="a&#10;b"\r\n/>;\r\n',
  );
  assert.equal(
    await prettier.format('<Box title="a\rb" />;', {
      ...js,
      endOfLine: "auto",
    }),
    '<Box\r  title="a&#10;b"\r/>;\r',
  );
});

test("SWCFMT reports UTF-16 diagnostic columns and rejects unsupported options", async () => {
  await assert.rejects(
    prettier.format('const x="😀"; const y = ;', js),
    (error) => {
      assert.equal(error.loc.start.line, 1);
      assert.equal(error.loc.start.column, 25);
      return true;
    },
  );
  await assert.rejects(
    prettier.format("const x=1; const y=2;", { ...js, rangeStart: 11 }),
    /whole-file/,
  );
  await assert.rejects(
    prettier.format("const x={a:1};", { ...js, objectWrap: "collapse" }),
    /layout option/,
  );
});

test("SWCFMT decoder rejects truncated and cyclic transport", () => {
  assert.throws(() => decode(new Uint8Array(), ""), /Invalid SWCFMT/);
  const unsupportedVersion = documentBuffer([...textA]);
  new DataView(unsupportedVersion.buffer).setUint32(4, 1, true);
  assert.throws(() => decode(unsupportedVersion, ""), /Invalid SWCFMT/);
  const bytes = new Uint8Array(25);
  bytes.set([83, 87, 67, 68]);
  const view = new DataView(bytes.buffer);
  view.setUint32(4, 0, true);
  view.setUint32(8, 1, true);
  view.setUint32(16, 5, true);
  bytes[20] = 9; // reference to root itself
  assert.throws(() => decode(bytes, ""), /Invalid SWCFMT/);
});

// Small transport fixtures exercise behavior independent of syntax formatting.
function documentBuffer(instructions, strings = "ab") {
  return fragmentBuffer([instructions], strings);
}

function fragmentBuffer(fragments, strings = "ab") {
  const pool = new TextEncoder().encode(strings);
  const length = fragments.reduce(
    (length, fragment) => length + 4 + fragment.length,
    16,
  );
  const output = new Uint8Array(length + pool.length);
  output.set([83, 87, 67, 68]);
  const view = new DataView(output.buffer);
  view.setUint32(4, 0, true);
  view.setUint32(8, fragments.length, true);
  view.setUint32(12, pool.length, true);
  let offset = 16;
  for (const fragment of fragments) {
    view.setUint32(offset, fragment.length, true);
    offset += 4;
    output.set(fragment, offset);
    offset += fragment.length;
  }
  output.set(pool, offset);
  return output;
}

const word = (value) => [
  value & 255,
  (value >>> 8) & 255,
  (value >>> 16) & 255,
  value >>> 24,
];
const textAt = (start, length) => [0, ...word(start), ...word(length)];
const anonymousGroup = (broken) => [3, Number(broken), ...word(0)];
const nearestCondition = (broken) => [8, Number(broken), ...word(0xffffffff)];
const textA = textAt(0, 1);
const textB = textAt(1, 1);
const printDoc = (bytes, options = {}) =>
  prettier.doc.printer.printDocToString(decode(bytes, ""), {
    printWidth: 80,
    tabWidth: 2,
    useTabs: false,
    ...options,
  }).formatted;

test("SWCFMT transports suffix documents without implicit spaces or breaks", () => {
  assert.equal(
    printDoc(
      documentBuffer([...textA, 14, ...textAt(2, 1), 17, 14, ...textB], "abc"),
    ),
    "abc",
  );

  const long = documentBuffer(
    [
      ...anonymousGroup(false),
      ...textA,
      14,
      ...textAt(2, 18),
      17,
      14,
      2,
      0,
      ...textB,
      17,
      3,
    ],
    "ab// suffix too long",
  );
  assert.equal(printDoc(long, { printWidth: 4 }), "a b// suffix too long");

  const nested = documentBuffer(
    [
      ...textA,
      14,
      ...textAt(2, 1),
      14,
      ...textAt(3, 1),
      17,
      14,
      ...textAt(4, 1),
      17,
      14,
      ...textB,
    ],
    "abxyz",
  );
  assert.equal(printDoc(nested), "abxzy");
});

test("SWCFMT preserves suffix boundaries and explicit separating spaces", () => {
  assert.equal(printDoc(documentBuffer([...textA, 1, 15, ...textB])), "a b");

  const suffix = [14, 1, ...textAt(1, 8), 17, 14];
  const separated = documentBuffer(
    [...textA, ...suffix, 15, ...textAt(9, 1)],
    "a// firstb",
  );
  assert.equal(printDoc(separated), "a // first\nb");

  const consecutive = documentBuffer(
    [
      ...textA,
      ...suffix,
      15,
      14,
      1,
      ...textAt(9, 9),
      17,
      14,
      2,
      3,
      ...textAt(18, 1),
    ],
    "a// first// secondb",
  );
  assert.equal(printDoc(consecutive), "a // first\n // second\nb");

  const grouped = documentBuffer(
    [
      ...textA,
      ...suffix,
      ...anonymousGroup(false),
      ...nearestCondition(true),
      ...textAt(9, 1),
      17,
      8,
      15,
      ...textAt(10, 1),
      2,
      0,
      ...textAt(11, 1),
      ...nearestCondition(true),
      ...textAt(12, 1),
      17,
      8,
      17,
      3,
    ],
    "a// first(bc)",
  );
  assert.equal(printDoc(grouped), "a( // first\nb\nc)");

  const shared = fragmentBuffer(
    [
      [...textA, ...suffix, ...reference(1)],
      [15, ...textAt(9, 1)],
    ],
    "a// firstb",
  );
  assert.equal(printDoc(shared), "a // first\nb");
});

test("SWCFMT measures current-group conditions during lookahead", () => {
  const bytes = documentBuffer(
    [
      ...anonymousGroup(false),
      ...textAt(0, 6),
      2,
      0,
      ...textAt(6, 2),
      17,
      3,
      ...anonymousGroup(true),
      ...nearestCondition(true),
      ...textAt(8, 3),
      17,
      8,
      2,
      3,
      17,
      3,
    ],
    "012345xxabc",
  );
  // The forced group's conditional contributes three columns while fitting
  // the preceding group, before that forced group has actually been printed.
  assert.equal(printDoc(bytes, { printWidth: 10 }), "012345\nxxabc\n");
});

test("SWCFMT lowers pending spaces at content and scope boundaries", () => {
  assert.equal(
    printDoc(documentBuffer([1, ...textA, 1, 1, ...textB, 1])),
    " a b",
  );
  assert.equal(
    printDoc(documentBuffer([...textA, 1, 2, 2, 1, ...textB])),
    "a\n b",
  );
  assert.equal(
    printDoc(documentBuffer([...textA, 4, 1, 17, 4, 1, ...textB])),
    "a b",
  );
  assert.equal(printDoc(documentBuffer([...textA, 1, 2, 0, ...textB])), "a\nb");
});

const reference = (id) => [9, ...word(id)];

test("SWCFMT separates canonical fitting from source-only expansion", () => {
  const alternatives = [20, ...word(2), ...word(1), ...word(2)];
  const sourceGroup = [3, 2, ...word(0)];
  const source = fragmentBuffer(
    [
      alternatives,
      [...sourceGroup, ...textAt(0, 6), 2, 0, ...textAt(6, 6), 17, 3],
      textAt(12, 8),
    ],
    "aaaaaabbbbbbfallback",
  );
  assert.equal(printDoc(source, { printWidth: 10 }), "fallback");
  assert.equal(printDoc(source, { printWidth: 20 }), "aaaaaa\nbbbbbb");

  const required = fragmentBuffer(
    [
      alternatives,
      [...anonymousGroup(true), ...textAt(0, 6), 2, 0, ...textAt(6, 6), 17, 3],
      textAt(12, 8),
    ],
    "aaaaaabbbbbbfallback",
  );
  assert.equal(printDoc(required, { printWidth: 10 }), "aaaaaa\nbbbbbb");

  const fallback = fragmentBuffer([[20, ...word(1), ...word(1)], textA]);
  assert.equal(printDoc(fallback), "a");

  const named = fragmentBuffer(
    [
      alternatives,
      [
        ...anonymousGroup(false),
        3,
        2,
        ...word(1),
        ...textAt(0, 1),
        2,
        0,
        ...nearestCondition(false),
        ...textAt(1, 4),
        17,
        8,
        ...nearestCondition(true),
        ...textAt(5, 1),
        17,
        8,
        17,
        3,
        8,
        1,
        ...word(1),
        2,
        1,
        17,
        8,
        ...textAt(6, 2),
        17,
        3,
      ],
      textAt(8, 8),
    ],
    "affffbccfallback",
  );
  assert.equal(printDoc(named, { printWidth: 8 }), "a\nb\ncc");
  assert.equal(printDoc(named, { printWidth: 7 }), "fallback");
});

test("SWCFMT builds canonical views of deep shared fragments iteratively", () => {
  const fragments = [[20, ...word(2), ...word(1), ...word(10002)]];
  for (let id = 1; id < 10001; id++) fragments.push(reference(id + 1));
  fragments.push([3, 2, ...word(0), ...textA, 2, 0, ...textB, 17, 3]);
  fragments.push(textB);
  assert.doesNotThrow(() => decode(fragmentBuffer(fragments), ""));
});

test("SWCFMT carries whitespace through shared fragment boundaries", () => {
  assert.equal(
    printDoc(
      fragmentBuffer([
        [...reference(1), ...textB],
        [...textA, 1],
      ]),
    ),
    "a b",
  );
  assert.equal(
    printDoc(
      fragmentBuffer([
        [...reference(1), 1, ...textB],
        [...textA, 2, 2],
      ]),
    ),
    "a\n b",
  );
  assert.equal(
    printDoc(
      fragmentBuffer([
        [...textA, 1, ...reference(1)],
        [1, ...textB],
      ]),
    ),
    "a b",
  );
  assert.equal(
    printDoc(
      fragmentBuffer([
        [...textA, 1, ...reference(1), ...textB],
        [2, 2],
      ]),
    ),
    "a\nb",
  );
  assert.equal(
    printDoc(fragmentBuffer([[...textA, 1, ...reference(1), ...textB], []])),
    "a b",
  );
  assert.equal(
    printDoc(fragmentBuffer([[...reference(1)], [...textA, 1]])),
    "a",
  );
});

test("SWCFMT caches shared documents by incoming whitespace state", () => {
  const bytes = fragmentBuffer([
    [...reference(1), ...textA, ...reference(1), 2, 2, ...reference(1)],
    [...reference(2)],
    [1, ...textB],
  ]);
  assert.equal(printDoc(bytes), " ba b\n b");

  const repeated = fragmentBuffer([
    [...textA, 1, ...reference(1), 1, ...reference(1)],
    [...textB],
  ]);
  const doc = decode(repeated, "");
  assert.equal(printDoc(repeated), "a b b");
  assert.equal(doc[1], doc[2]);
});

test("SWCFMT preserves logical group conditions across shared fill entries", () => {
  const bytes = fragmentBuffer([
    [
      ...anonymousGroup(true),
      ...reference(1),
      11,
      12,
      ...reference(1),
      17,
      12,
      17,
      11,
      17,
      3,
    ],
    [
      ...nearestCondition(true),
      ...textA,
      17,
      8,
      ...nearestCondition(false),
      ...textB,
      17,
      8,
    ],
  ]);
  // Fill can print an entry flat inside a broken logical group. Reusing the
  // direct reference must still test the logical group's broken mode there.
  assert.equal(printDoc(bytes), "aa");
});

test("SWCFMT discards conditional soft-line gaps during lookahead", () => {
  const bytes = documentBuffer(
    [
      ...anonymousGroup(false),
      ...textAt(0, 6),
      2,
      0,
      ...textAt(6, 1),
      17,
      3,
      ...anonymousGroup(true),
      1,
      ...nearestCondition(true),
      ...textAt(7, 1),
      17,
      8,
      2,
      1,
      ...textAt(8, 1),
      17,
      3,
    ],
    "012345xaz",
  );
  // The break branch consumes the gap. Its inactive flat branch must not
  // contribute another space when fitting up to the following soft line.
  assert.equal(printDoc(bytes, { printWidth: 10 }), "012345 x a\nz");
});

test("SWCFMT resolves deeply nested references without recursive calls", () => {
  const fragments = Array.from({ length: 10000 }, (_, id) => reference(id + 1));
  fragments.push([...textA, 1]);
  fragments[0].push(...textB);
  // Inspect the decoded structure; Prettier's own traversal has separate limits.
  const doc = decode(fragmentBuffer(fragments), "");
  assert.equal(doc.at(-2), " ");
  assert.equal(doc.at(-1), "b");
});

// A deferred space must be consumed only by a branch that actually prints.
test("SWCFMT carries conditional gaps through shared documents", () => {
  const group = [3, 0, 1, 0, 0, 0];
  const broken = [8, 1, 1, 0, 0, 0];
  const flat = [8, 0, 1, 0, 0, 0];
  const bytes = fragmentBuffer([
    [
      ...group,
      ...textA,
      1,
      ...broken,
      ...reference(1),
      17,
      8,
      ...flat,
      ...reference(1),
      17,
      8,
      ...textA,
      17,
      3,
    ],
    [...textB],
  ]);
  assert.equal(printDoc(bytes), "a ba");
});

test("SWCFMT maps absolute indentation and flat named groups", () => {
  assert.equal(
    printDoc(
      documentBuffer([
        4,
        2,
        3,
        19,
        3,
        0,
        0,
        0,
        ...textA,
        2,
        3,
        ...textB,
        17,
        4,
        17,
        4,
      ]),
    ),
    "\n  a\n   b",
  );
  assert.equal(
    printDoc(
      documentBuffer([
        18,
        1,
        0,
        0,
        0,
        8,
        0,
        1,
        0,
        0,
        0,
        ...textA,
        17,
        8,
        8,
        1,
        1,
        0,
        0,
        0,
        ...textB,
        17,
        8,
      ]),
    ),
    "a",
  );
});

test("SWCFMT rejects malformed formatter option records at the bridge", () => {
  assert.throws(
    () =>
      formatDocBufferSync(
        "",
        0,
        Uint8Array.of(1, 80, 0, 2, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0),
      ),
    /invalid formatter options/,
  );
  assert.throws(
    () => formatDocBufferSync("", 0, Uint8Array.of(0)),
    /invalid formatter options/,
  );
  assert.throws(
    () =>
      formatDocBufferSync(
        "",
        0,
        Uint8Array.of(0, 80, 0, 2, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0),
      ),
    /invalid formatter options/,
  );
  assert.throws(
    () =>
      formatDocBufferSync(
        "",
        0,
        Uint8Array.of(0, 80, 0, 2, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 3),
      ),
    /invalid formatter options/,
  );
});

test("SWCFMT preserves raw trailing spaces across cached fragment boundaries", () => {
  const rawSpace = [0, 1, 0, 0, 0, 2, 0, 0, 0];
  const textAfterSpace = [0, 3, 0, 0, 0, 1, 0, 0, 0];
  assert.equal(
    printDoc(
      fragmentBuffer(
        [
          [
            ...textA,
            ...reference(1),
            ...textA,
            ...rawSpace,
            ...reference(1),
            ...textAfterSpace,
          ],
          [2, 3],
        ],
        "a  b",
      ),
    ),
    "a\na  \nb",
  );
});
