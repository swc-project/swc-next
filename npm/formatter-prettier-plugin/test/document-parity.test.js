import assert from "node:assert/strict";
import test from "node:test";
import { builders as b, printer } from "prettier/doc";
import { decode } from "../src-js/decode.js";
import { printDocBufferSync } from "@swc-next/formatter";
import { formatDocBufferSync } from "@swc-next/formatter-js";
import { encodeDoc } from "@swc-next/formatter/document";
import {
  word,
  group,
  end,
  reference,
  condition,
  completeGroup,
  line,
  hard,
  rawHard,
  literal,
  rawLiteral,
  wire,
} from "./document-utils.js";

const print = (doc, printWidth = 80) =>
  printer.printDocToString(doc, { printWidth, tabWidth: 2, useTabs: false })
    .formatted;
const printWire = (bytes, width = 80) => print(decode(bytes, ""), width);

test("complete groups preserve contents and expanded states at the wire boundary", () => {
  const bytes = wire((t) => [
    [...completeGroup(2, 0, 1, [2])],
    t("contents"),
    t("state"),
  ]);
  const doc = decode(bytes, "");
  assert.equal(doc.contents, "contents");
  assert.deepEqual(doc.expandedStates, ["state"]);
  assert.equal(print(doc), "contents");
  assert.equal(printDocBufferSync(bytes), "contents");

  const absent = wire((t) => [[...completeGroup(0, 0, 1)], t("plain")]);
  assert.equal(decode(absent, "").expandedStates, undefined);
  assert.equal(printDocBufferSync(absent), printWire(absent));

  const named = wire((t) => [
    [...completeGroup(3, 1, 1, [2]), ...condition(1, 3, 4)],
    [...t("a"), ...line, ...t("b")],
    t("expanded"),
    t("B"),
    t("F"),
  ]);
  const namedDoc = decode(named, "");
  assert.equal(namedDoc[0].break, true);
  assert.equal(namedDoc[0].id, namedDoc[1].groupId);
  assert.equal(printDocBufferSync(named), print(namedDoc));
});

test("complete groups preserve shared document identity", () => {
  const bytes = wire((t) => [
    [...reference(1), ...reference(1)],
    [...completeGroup(2, 0, 2, [2])],
    [16, ...t("x")],
  ]);
  const doc = decode(bytes, "");
  assert.equal(doc[0], doc[1]);
  assert.equal(doc[0].contents, doc[0].expandedStates[0]);
  assert.equal(printDocBufferSync(bytes), print(doc));
});

test("complete groups reject invalid flags, states, and fragment cycles", () => {
  const invalid = [
    wire((t) => [[...completeGroup(4, 0, 1)], t("x")]),
    wire((t) => [[...completeGroup(2, 0, 1, [])], t("x")]),
    wire((t) => [[...completeGroup(2, 0, 1, [2])], t("x")]),
    wire((t) => [[...completeGroup(0, 0, 0)], t("x")]),
    wire((t) => [[...completeGroup(2, 0, 1, [0])], t("x")]),
    wire((t) => [[...completeGroup(2, 0, 1, [2])], t("x"), [...reference(2)]]),
  ];
  for (const bytes of invalid) {
    assert.throws(() => decode(bytes, ""));
    assert.throws(() => printDocBufferSync(bytes));
  }
});

test("conditional indentation forwards current and named modes", () => {
  for (const negate of [false, true]) {
    for (const broken of [false, true]) {
      const current = wire((t) => [
        [
          ...group(0, broken),
          7,
          Number(negate),
          ...word(0),
          ...rawHard,
          ...t("x"),
          ...end(4),
          ...end(3),
        ],
      ]);
      const named = wire((t) => [
        [
          ...group(1, broken),
          ...t("a"),
          ...end(3),
          7,
          Number(negate),
          ...word(1),
          ...rawHard,
          ...t("x"),
          ...end(4),
        ],
      ]);
      assert.equal(printDocBufferSync(current), printWire(current));
      assert.equal(printDocBufferSync(named), printWire(named));
      assert.equal(printWire(current), broken !== negate ? "\n  x" : "\nx");
      assert.equal(printWire(named), broken !== negate ? "a\n  x" : "a\nx");
    }
    const unknown = wire((t) => [
      [7, Number(negate), ...word(1), ...rawHard, ...t("x"), ...end(4)],
    ]);
    assert.equal(printWire(unknown), "");
    assert.equal(printDocBufferSync(unknown), "");
  }
});

test("conditional indentation rejects unknown flags and truncated records", () => {
  const invalid = [
    wire(() => [[7, 2, ...word(0), ...end(4)]]),
    wire(() => [[7, 1, 0, 0]]),
  ];
  for (const bytes of invalid) {
    assert.throws(() => decode(bytes, ""));
    assert.throws(() => printDocBufferSync(bytes));
  }
});

test("indent scopes preserve prefix, dedent, root, and suffix through both hosts", () => {
  const bytes = wire((t) => [
    [
      4,
      25,
      23,
      ...t("> ").slice(1),
      25,
      ...hard,
      ...t("a"),
      24,
      ...hard,
      ...t("b"),
      ...end(24),
      26,
      ...hard,
      ...t("c"),
      ...end(26),
      14,
      ...t(" S"),
      ...end(14),
      ...hard,
      ...t("d"),
      ...end(25),
      ...end(23),
      ...end(25),
      ...end(4),
    ],
  ]);
  for (const tabWidth of [0, 2, 4]) {
    for (const useTabs of [false, true]) {
      const options = { printWidth: 80, tabWidth, useTabs, endOfLine: "lf" };
      assert.equal(
        printDocBufferSync(bytes, options),
        printer.printDocToString(decode(bytes, "", options), options).formatted,
      );
    }
  }
});

test("absolute indentation resets to the current root once", () => {
  const bytes = wire((t) => [
    [
      4,
      25,
      23,
      ...t("> ").slice(1),
      25,
      19,
      ...word(2),
      ...hard,
      ...t("x"),
      ...end(4),
      ...end(25),
      ...end(23),
      ...end(25),
      ...end(4),
    ],
  ]);
  for (const tabWidth of [0, 2, 4]) {
    const options = {
      printWidth: 80,
      tabWidth,
      useTabs: false,
      endOfLine: "lf",
    };
    assert.equal(
      printDocBufferSync(bytes, options),
      printer.printDocToString(decode(bytes, "", options), options).formatted,
    );
  }
  assert.equal(
    printer.printDocToString(decode(bytes, ""), {
      printWidth: 80,
      tabWidth: 2,
      useTabs: false,
    }).formatted,
    "\n  >   x",
  );

  const zero = wire((t) => [
    [
      4,
      25,
      23,
      ...t("> ").slice(1),
      25,
      19,
      ...word(0),
      ...literal,
      ...t("x"),
      ...end(4),
      ...end(25),
      ...end(23),
      ...end(25),
      ...end(4),
    ],
  ]);
  assert.equal(printWire(zero), "\n  x");
  assert.equal(printDocBufferSync(zero), "\n  x");
});

test("indent scopes reject invalid strings and unmatched ends", () => {
  const utf8 = wire((t) => [[23, ...t("é").slice(1), ...end(23)]]);
  utf8[utf8.length - 1] = 0xff;
  const invalid = [
    utf8,
    wire(() => [[23, ...word(0), ...word(1), ...end(23)]]),
    wire((t) => [[23, ...t("x").slice(1), ...end(24)]]),
    wire(() => [[24]]),
    wire(() => [[...end(25)]]),
  ];
  for (const bytes of invalid) {
    assert.throws(() => decode(bytes, ""));
    assert.throws(() => printDocBufferSync(bytes));
  }
});

test("multiline Text remains intact across document hosts", () => {
  const bytes = wire((t) => [
    [...reference(1), ...hard, ...reference(1)],
    t("界\r\nλ\r終\n"),
  ]);
  const doc = decode(bytes, "");
  for (const [endOfLine, separator] of [
    ["lf", "\n"],
    ["crlf", "\r\n"],
    ["cr", "\r"],
  ]) {
    const options = { printWidth: 80, tabWidth: 2, useTabs: false, endOfLine };
    const text = "界\r\nλ\r終\n".replaceAll("\n", separator);
    const expected = `${text}${separator}${text}`;
    assert.equal(printer.printDocToString(doc, options).formatted, expected);
    assert.equal(printDocBufferSync(bytes, options), expected);
  }

  const conditional = wire((t) => [
    [...group(), 8, 0, ...word(0), ...t("a\nb"), ...end(8), ...end(3)],
  ]);
  assert.equal(printWire(conditional), "a\nb");
  assert.equal(printDocBufferSync(conditional), "a\nb");

  const narrow = wire((t) => [
    [...group(), ...t("界\nλ"), ...line, ...t("x"), ...end(3)],
  ]);
  assert.equal(printDocBufferSync(narrow, { printWidth: 4 }), "界\nλ\nx");
});

test("raw lines preserve parent mode across the document boundary", () => {
  for (const raw of [rawHard, rawLiteral]) {
    const bytes = wire((t) => [
      [
        ...group(),
        ...t("a"),
        ...raw,
        ...t("b"),
        ...line,
        ...t("c"),
        ...condition(0, 1, 2),
        ...end(3),
      ],
      t("B"),
      t("F"),
    ]);
    const forced = wire((t) => [
      [
        ...group(),
        ...t("a"),
        ...raw,
        16,
        ...t("b"),
        ...line,
        ...t("c"),
        ...condition(0, 1, 2),
        ...end(3),
      ],
      t("B"),
      t("F"),
    ]);
    for (const width of [1, 3, 4, 80]) {
      assert.equal(
        printDocBufferSync(bytes, { printWidth: width }),
        printWire(bytes, width),
      );
      assert.equal(
        printDocBufferSync(forced, { printWidth: width }),
        printWire(forced, width),
      );
    }
    assert.equal(printWire(bytes), "a\nb cF");
    assert.equal(printWire(forced), "a\nb\ncB");
  }
});

test("shared raw lines retain suffix and indentation behavior across hosts", () => {
  for (const raw of [rawHard, rawLiteral]) {
    const bytes = wire((t) => [
      [
        ...group(),
        4,
        ...t("a  "),
        14,
        ...t(" S "),
        ...end(14),
        ...reference(1),
        ...t("b"),
        ...line,
        ...t("c"),
        ...end(4),
        ...end(3),
        ...reference(1),
        ...t("z"),
      ],
      raw,
    ]);
    for (const width of [2, 5, 80])
      assert.equal(
        printDocBufferSync(bytes, { printWidth: width }),
        printWire(bytes, width),
      );
  }
});

test("Rust host prints an SWCD buffer produced by Rust", () => {
  const options = Uint8Array.of(0, 80, 0, 2, 0, 1, 0, 0, 0, 2, 1, 1, 0, 0, 0);
  const document = formatDocBufferSync("const x=1;", 0, options);
  assert.equal(printDocBufferSync(document), "const x = 1;\n");
});

test("JavaScript documents print through the same wire and preserve shared groups", () => {
  const id = Symbol("layout");
  const shared = b.group(["alpha", b.line, "beta"], { id });
  const doc = [
    shared,
    b.hardline,
    b.fill(["foo", b.line, "bar", b.line, "baz"]),
    b.hardline,
    shared,
    b.ifBreak(" expanded", " flat", { groupId: id }),
    b.hardline,
    b.group(["x ", b.trim, { type: "line", literal: true }, "y"]),
  ];
  const { buffer, labels } = encodeDoc(doc);
  const decoded = decode(buffer, "", undefined, labels);
  assert.equal(decoded[0], decoded[4]);
  assert.equal(decoded[4].id, decoded[5].groupId);
  for (const printWidth of [5, 12, 80]) {
    const expected = print(doc, printWidth);
    assert.equal(print(decoded, printWidth), expected);
    assert.equal(printDocBufferSync(buffer, { printWidth }), expected);
  }
});

test("document alignment values normalize across hosts", () => {
  const aligned = (alignment) =>
    b.indent(
      b.align(3, [b.hardline, "a", b.align(alignment, [b.hardline, "b"])]),
    );
  const canonical = encodeDoc(aligned(-1)).buffer;
  for (const alignment of [-1, -2, -0.5, -Number.MAX_VALUE]) {
    const doc = aligned(alignment);
    const { buffer, labels } = encodeDoc(doc);
    const expected = "\n     a\n  b";
    assert.equal(print(doc), expected);
    assert.deepEqual(buffer, canonical);
    assert.equal(print(decode(buffer, "", undefined, labels)), expected);
    assert.equal(printDocBufferSync(buffer), expected);
  }

  const rooted = b.align(
    "> ",
    b.markAsRoot(
      b.indent([b.hardline, "a", b.align(-Infinity, [b.hardline, "b"])]),
    ),
  );
  const { buffer, labels } = encodeDoc(rooted);
  const expected = "\n>   a\n> b";
  assert.equal(print(rooted), expected);
  assert.equal(print(decode(buffer, "", undefined, labels)), expected);
  assert.equal(printDocBufferSync(buffer), expected);

  assert.throws(() => encodeDoc(b.align(NaN, "x")), RangeError);
  assert.throws(() => encodeDoc(b.align(Infinity, "x")), RangeError);
  assert.throws(() => encodeDoc(b.align(0.5, "x")), RangeError);
  assert.throws(() => encodeDoc(b.align(0x1_0000_0000, "x")), RangeError);
});

test("document sidecars preserve opaque and falsy values", () => {
  const opaque = {};
  opaque.self = opaque;
  const label = (label, contents) => ({ type: "label", label, contents });
  const shared = label(opaque, "a");
  const { buffer, labels } = encodeDoc([
    shared,
    shared,
    label(0, "b"),
    label(false, "c"),
    label(null, "d"),
    label(undefined, "e"),
  ]);
  const result = decode(buffer, "", undefined, labels);
  assert.equal(result[0], result[1]);
  assert.equal(result[0].label, opaque);
  assert.equal(result[2].label, 0);
  assert.equal(result[3].label, false);
  assert.equal(result[4].label, null);
  assert.equal(result[5].label, undefined);
  assert.equal(printDocBufferSync(buffer), "aabcde");
  assert.throws(() => decode(buffer, ""));
  const missing = labels.findIndex((value) => value === undefined);
  delete labels[missing];
  assert.throws(() => decode(buffer, "", undefined, labels));
});

test("shared wire fill entries retain their logical boundaries", () => {
  const bytes = wire((text) => [
    [11, ...reference(1), ...end(11)],
    [...reference(2), ...reference(3), ...reference(2)],
    [12, ...text("ab"), ...end(12)],
    [12, 2, 0, ...end(12)],
  ]);
  const options = { printWidth: 4 };
  assert.equal(printWire(bytes, 4), "ab\nab");
  assert.equal(printDocBufferSync(bytes, options), "ab\nab");
});
