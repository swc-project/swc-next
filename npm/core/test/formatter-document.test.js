import assert from "node:assert/strict";
import test from "node:test";
import { builders as b, printer } from "prettier/doc";
import { decode } from "../../../packages/prettier-plugin/decode.js";
import {
  formatDocBufferSync,
  printDocBufferSync,
} from "../../formatter/src-js/index.js";

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
const completeGroup = (flags, id, contents, states) => [
  22,
  flags,
  ...word(id),
  ...word(contents),
  ...(states === undefined
    ? []
    : [...word(states.length), ...states.flatMap(word)]),
];
const line = [2, 0];
const hard = [2, 2];
const rawHard = [2, 3];
const literal = [2, 4];
const rootLiteral = [2, 5];
const rawLiteral = [2, 6];

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

test("named indent scopes retain unknown and broken behavior", () => {
  const unknown = wire((t) => [
    [7, 0, ...word(1), ...hard, ...t("x"), ...end(4)],
  ]);
  assert.equal(printWire(unknown), "");
  const known = wire((t) => [
    [
      ...group(1, true),
      ...t("a"),
      ...end(3),
      7,
      0,
      ...word(1),
      ...hard,
      ...t("x"),
      ...end(4),
    ],
  ]);
  assert.equal(printWire(known), "a\n  x");
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
      wire(() => [[2, 7]]),
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

test("prefix alignment retains its exact string", () => {
  const bytes = wire((t) => [
    [23, ...t("界\r\n\t").slice(1), ...t("x"), ...end(23)],
  ]);
  const doc = decode(bytes, "");
  assert.equal(doc.n, "界\r\n\t");
  assert.equal(doc.contents, "x");
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
      wire(() => [[7, 2, ...word(0), ...end(4)]]),
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

test("Rust host prints a language-independent document buffer", () => {
  const bytes = wire((t) => [
    [...group(), ...t("λ"), 4, ...line, ...t("世界"), ...end(4), ...end(3)],
  ]);
  assert.equal(printDocBufferSync(bytes), "λ 世界");
  assert.equal(
    printDocBufferSync(bytes, {
      printWidth: 2,
      tabWidth: 4,
      endOfLine: "crlf",
    }),
    "λ\r\n    世界",
  );
  assert.equal(
    printDocBufferSync(bytes, { printWidth: 2, useTabs: true }),
    "λ\n\t世界",
  );

  const shared = new Uint8Array(new SharedArrayBuffer(bytes.length));
  shared.set(bytes);
  assert.equal(printDocBufferSync(shared), "λ 世界");

  const bom = wire((t) => [t("\uFEFFx")]);
  assert.equal(printDocBufferSync(bom), "\uFEFFx");
});

test("Rust host forwards document and option errors", () => {
  assert.throws(
    () => printDocBufferSync(Uint8Array.of(1)),
    /invalid|document/i,
  );
  assert.throws(() => printDocBufferSync(wire(() => [[]], 1)), /version/i);
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { printWidth: 65536 }),
    RangeError,
  );
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { tabWidth: 65 }),
    RangeError,
  );
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { useTabs: 1 }),
    TypeError,
  );
  assert.throws(
    () => printDocBufferSync(new Uint8Array(), { endOfLine: "auto" }),
    RangeError,
  );
});

test("Rust host prints an SWCD buffer produced by Rust", () => {
  const options = Uint8Array.of(0, 80, 0, 2, 0, 1, 0, 0, 0, 2, 1, 1, 0, 0, 0);
  const document = formatDocBufferSync("const x=1;", 0, options);
  assert.equal(printDocBufferSync(document), "const x = 1;\n");

  const source = "const x = [1, // comment\n 2];";
  const grouped = formatDocBufferSync(source, 0, options);
  const doc = decode(grouped, source);
  const group = doc.find((part) => part?.type === "group");
  assert.equal(group?.break, false);
  const expected = "const x = [\n  1, // comment\n  2\n];\n";
  assert.equal(print(doc), expected);
  assert.equal(printDocBufferSync(grouped), expected);
});
