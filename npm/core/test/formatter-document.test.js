import assert from "node:assert/strict";
import test from "node:test";
import { builders as b, printer } from "prettier/doc";
import { decode } from "../../../packages/prettier-plugin/decode.js";

const word = (value) => [
  value & 255,
  (value >>> 8) & 255,
  (value >>> 16) & 255,
  value >>> 24,
];
const group = (id = 0, broken = false) => [3, Number(broken), ...word(id)];
const end = (tag) => [17, tag];
const reference = (id) => [9, ...word(id)];
const condition = (target, broken, flat) => [
  21,
  ...word(target),
  ...word(broken),
  ...word(flat),
];
const alternatives = (...ids) => [
  10,
  ...word(ids.length),
  ...ids.flatMap(word),
];
const line = [2, 0];
const hard = [2, 2];
const literal = [2, 4];
const rootLiteral = [2, 5];

function wire(build, version = 0) {
  const strings = [];
  let length = 0;
  const text = (value) => {
    const bytes = Buffer.from(value);
    const op = [0, ...word(length), ...word(bytes.length)];
    strings.push(bytes);
    length += bytes.length;
    return op;
  };
  const fragments = build(text);
  return Buffer.concat([
    Buffer.from("SWCD"),
    Buffer.from([...word(version), ...word(fragments.length), ...word(length)]),
    ...fragments.flatMap((ops) => [
      Buffer.from(word(ops.length)),
      Buffer.from(ops),
    ]),
    ...strings,
  ]);
}
const print = (doc, printWidth = 80) =>
  printer.printDocToString(doc, { printWidth, tabWidth: 2, useTabs: false })
    .formatted;
const printWire = (bytes, width = 80) => print(decode(bytes, ""), width);
function graphSize(doc) {
  const seen = new Set();
  const work = [doc];
  while (work.length) {
    const item = work.pop();
    if (!item || typeof item !== "object" || seen.has(item)) continue;
    seen.add(item);
    if (Array.isArray(item)) work.push(...item);
    else
      for (const key of [
        "contents",
        "parts",
        "expandedStates",
        "breakContents",
        "flatContents",
      ])
        if (item[key] !== undefined) work.push(item[key]);
  }
  return seen.size;
}

test("literal spaces and flat lines keep their multiplicity", () => {
  const bytes = wire((t) => [
    [
      ...group(),
      ...t("a"),
      ...line,
      ...line,
      ...t("  "),
      ...t(""),
      ...t("b"),
      ...end(3),
    ],
  ]);
  assert.equal(printWire(bytes), "a    b");
  assert.equal(printWire(bytes, 1), "a\n\n  b");
});

test("line kinds own trimming and root-relative literal indentation", () => {
  const trim = wire((t) => [[...t("a  \t"), ...hard, ...t("b")]]);
  assert.equal(printWire(trim), "a\nb");
  const raw = wire((t) => [[4, ...t("a  "), ...literal, ...t("b"), ...end(4)]]);
  assert.equal(printWire(raw), "a  \nb");
  const aligned = wire((t) => [
    [4, ...t("a  "), ...rootLiteral, ...t("b"), ...end(4)],
  ]);
  assert.equal(printWire(aligned), "a  \n  b");
});

test("line suffixes retain their actual text at flush and EOF", () => {
  const eof = wire((t) => [[...t("a"), 14, ...t("S "), ...end(14), ...t("b")]]);
  assert.equal(printWire(eof), "abS ");
  const raw = wire((t) => [
    [...t("a"), 14, ...t("S "), ...end(14), ...t("b"), ...literal, ...t("c")],
  ]);
  assert.equal(printWire(raw), "abS \nc");
  const trim = wire((t) => [
    [...t("a"), 14, ...t("S "), ...end(14), ...hard, ...t("c")],
  ]);
  assert.equal(printWire(trim), "aS\nc");
});

test("Current conditions follow Fill entry mode", () => {
  const bytes = wire((t) => [
    [
      ...group(0, true),
      ...t("a"),
      11,
      12,
      ...condition(0, 1, 2),
      ...end(12),
      ...end(11),
      ...t("b"),
      ...end(3),
    ],
    t("B"),
    t("F"),
  ]);
  assert.equal(printWire(bytes), "aFb");
});

test("unknown Named modes print no branch and retain Prettier fit defaults", () => {
  const bytes = wire((t) => [
    [
      ...group(),
      ...t("a"),
      ...condition(1, 1, 2),
      ...line,
      ...t("b"),
      ...end(3),
    ],
    t("B"),
    t("long"),
  ]);
  const id = Symbol();
  const standard = b.group([
    "a",
    b.ifBreak("B", "long", { groupId: id }),
    b.line,
    "b",
  ]);
  assert.equal(printWire(bytes, 4), print(standard, 4));
  assert.equal(printWire(bytes, 80), "a b");
});

test("Named modes persist across shared group executions", () => {
  const bytes = wire((t) => [
    [
      ...t("123456"),
      ...reference(1),
      ...hard,
      ...reference(1),
      8,
      1,
      ...word(1),
      2,
      1,
      ...end(8),
      ...t("123456"),
    ],
    [...group(1), ...t("a"), ...line, ...t("b"), ...end(3)],
  ]);
  assert.equal(printWire(bytes, 8), "123456a\nb\na b123456");
});

test("both Conditional branches propagate breakParent", () => {
  const bytes = wire((t) => [
    [...group(), ...condition(0, 1, 2), ...end(3)],
    t("B"),
    [16, ...t("F")],
  ]);
  assert.equal(printWire(bytes), "B");
});

test("Alternatives retain standard conditionalGroup decisions", () => {
  const bytes = wire((t) => [
    [...alternatives(1, 2), ...t("!")],
    [...group(0, true), ...t("a"), ...line, ...t("bb"), ...end(3)],
    t("fallback"),
  ]);
  const standard = [
    b.conditionalGroup([
      b.group(["a", b.line, "bb"], { shouldBreak: true }),
      "fallback",
    ]),
    "!",
  ];
  assert.equal(printWire(bytes, 5), print(standard, 5));
  assert.equal(printWire(bytes, 20), print(standard, 20));
});

test("named indent scopes retain unknown and broken behavior", () => {
  const unknown = wire((t) => [[7, ...word(1), ...hard, ...t("x"), ...end(4)]]);
  assert.equal(printWire(unknown), "");
  const known = wire((t) => [
    [
      ...group(1, true),
      ...t("a"),
      ...end(3),
      7,
      ...word(1),
      ...hard,
      ...t("x"),
      ...end(4),
    ],
  ]);
  assert.equal(printWire(known), "a\n  x");
});

test("deep fragment DAGs remain shared and build iteratively", () => {
  const bytes = wire((t) => {
    const fragments = [];
    for (let id = 0; id < 10000; id++)
      fragments.push([...reference(id + 1), ...reference(id + 1)]);
    fragments.push(t("x"));
    return fragments;
  });
  assert.equal(graphSize(decode(bytes, "")), 10000);
});

test("v0 rejects removed extensions and malformed scopes or references", () => {
  assert.throws(() =>
    decode(
      wire((t) => [t("x")], 1),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[1]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[18, ...word(1)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[20, ...word(1), ...word(0)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[...group(), ...end(4)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[...reference(0)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[...condition(0, 0, 0)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[12, ...end(12)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[2, 6]]),
      "",
    ),
  );
});

test("Current conditions follow each caller of a shared fragment", () => {
  const bytes = wire((t) => [
    [
      ...group(0, true),
      ...reference(1),
      ...end(3),
      ...hard,
      ...group(),
      ...reference(1),
      ...end(3),
    ],
    [...condition(0, 2, 3)],
    t("B"),
    t("F"),
  ]);
  assert.equal(printWire(bytes), "B\nF");
});

test("shared fragments retain literal spaces and line trimming", () => {
  const bytes = wire((t) => [
    [
      ...t("a  "),
      ...reference(1),
      ...t("b"),
      ...reference(2),
      ...t("c"),
      ...reference(2),
      ...t("d"),
    ],
    hard,
    t("  "),
  ]);
  assert.equal(printWire(bytes), "a\nb  c  d");
});

test("absolute indentation maps to root-relative standard alignment", () => {
  const bytes = wire((t) => [
    [
      4,
      ...hard,
      19,
      ...word(3),
      ...t("a"),
      ...hard,
      ...t("b"),
      ...end(4),
      ...end(4),
    ],
  ]);
  assert.equal(printWire(bytes), "\n  a\n   b");
});

test("decoder validates all fragment bytes and string slices", () => {
  assert.throws(() => decode(Buffer.alloc(0), ""));
  const valid = wire((t) => [t("value")]);
  for (let length = 0; length < valid.length; length++)
    assert.throws(() => decode(valid.subarray(0, length), ""));
  assert.throws(() => decode(Buffer.concat([valid, Buffer.from([0])]), ""));
  assert.throws(() =>
    decode(
      wire((t) => [t("a"), [...group(), ...end(4)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[0, ...word(0), ...word(1)]]),
      "",
    ),
  );
  const utf8 = wire((t) => [t("é")]);
  utf8[utf8.length - 1] = 0xff;
  assert.throws(() => decode(utf8, ""));
  assert.throws(() =>
    decode(
      wire(() => [[3, 2, ...word(0), ...end(3)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire(() => [[7, ...word(0), ...end(4)]]),
      "",
    ),
  );
});

test("diagnostic buffers preserve UTF-16 columns and validate byte boundaries", () => {
  const error = (start, end, version = 0) =>
    Buffer.concat([
      Buffer.from("SWCE"),
      Buffer.from([...word(version), ...word(start), ...word(end)]),
      Buffer.from("message"),
    ]);
  const source = "a\r\n😀x";
  assert.throws(
    () => decode(error(7, 8), source),
    (error) => {
      assert.ok(error instanceof SyntaxError);
      assert.deepEqual(error.loc, {
        start: { line: 2, column: 3 },
        end: { line: 2, column: 4 },
      });
      return true;
    },
  );
  assert.throws(() => decode(error(4, 8), source));
  assert.throws(() => decode(error(8, 7), source));
  assert.throws(() => decode(error(0, 9), source));
  assert.throws(() => decode(error(0, 0, 1), source), /Invalid SWCFMT/);
});

test("Text rejects physical newlines including conditional token contents", () => {
  assert.throws(() =>
    decode(
      wire((t) => [t("a\nb")]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire((t) => [t("a\r\nb")]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire((t) => [t("a\rb")]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire((t) => [[8, 1, ...word(0), ...t("a\nb"), ...end(8)]]),
      "",
    ),
  );
  assert.throws(() =>
    decode(
      wire((t) => [[8, 0, ...word(0), ...t("a\rb"), ...end(8)]]),
      "",
    ),
  );
});

test("literal source boundaries normalize EOL and flush suffixes before fitting", () => {
  const bytes = wire((t) => [
    [
      ...t("1234"),
      14,
      ...t(" S"),
      ...end(14),
      ...literal,
      ...t("x"),
      ...group(),
      ...t("a"),
      ...line,
      ...t("b"),
      ...end(3),
    ],
  ]);
  const doc = decode(bytes, "");
  const options = { printWidth: 4, tabWidth: 2, useTabs: false };
  assert.equal(
    printer.printDocToString(doc, { ...options, endOfLine: "lf" }).formatted,
    "1234 S\nxa b",
  );
  assert.equal(
    printer.printDocToString(doc, { ...options, endOfLine: "crlf" }).formatted,
    "1234 S\r\nxa b",
  );
  assert.equal(
    printer.printDocToString(doc, { ...options, endOfLine: "cr" }).formatted,
    "1234 S\rxa b",
  );
});

test("suffix boundaries flush complete Docs only when a suffix is pending", () => {
  const empty = wire((t) => [[...t("a"), 15, ...t("b")]]);
  assert.equal(printWire(empty), "ab");
  const queued = wire((t) => [
    [...t("a"), 14, ...t(" S"), ...end(14), 15, ...t("b")],
  ]);
  assert.equal(printWire(queued), "a S\nb");
});
