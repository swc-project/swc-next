import assert from "node:assert/strict";
import test from "node:test";
import * as prettier from "prettier";
import { builders } from "prettier/doc";
import { formatDocBufferSync } from "@swc-next/formatter";
import * as plugin from "../index.js";
import { formatNativeSync, normalizeInput } from "./native.js";

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

test("SWCFMT honors both JSX bracket option names with Prettier precedence", async () => {
  const source = 'const a=<Box a="one" b="two">text</Box>;';
  await matches(source, tsx, { printWidth: 20, jsxBracketSameLine: true });
  await matches(source, tsx, {
    printWidth: 20,
    bracketSameLine: false,
    jsxBracketSameLine: true,
  });
  await matches(source, tsx, {
    printWidth: 20,
    bracketSameLine: true,
    jsxBracketSameLine: false,
  });
  await matches(source, tsx, {
    printWidth: 20,
    bracketSameLine: false,
    jsxBracketSameLine: false,
  });
});

test("SWCFMT leaves line endings and BOM to Prettier", async () => {
  await matches("\uFEFFconst x=1;\r\nconst y=2;\r\n", js, {
    endOfLine: "auto",
  });
  await matches("const x=1;\nconst y=2;\n", js, { endOfLine: "cr" });
  await matches("", js);
});

test("native N-API oracle returns exact strings and reports invalid inputs", () => {
  assert.equal(formatNativeSync("", "file.js", {}), "");
  assert.equal(
    formatNativeSync("const x=1;", "file.js", { endOfLine: "crlf" }),
    "const x = 1;\r\n",
  );
  assert.equal(
    formatNativeSync('const x="😀";', "file.js", { endOfLine: "cr" }),
    'const x = "😀";\r',
  );
  assert.throws(() => formatNativeSync("const =", "file.js", {}));
  assert.throws(() =>
    formatNativeSync("", "file.js", { endOfLine: "invalid" }),
  );
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

async function observedInput(source, options) {
  let received;
  const observer = {
    ...plugin,
    parsers: {
      ...plugin.parsers,
      swc: {
        ...plugin.parsers.swc,
        parse(source, options) {
          received = { source, options: { endOfLine: options.endOfLine } };
          return plugin.parsers.swc.parse(source, options);
        },
      },
    },
  };
  await prettier.format(source, { ...js, ...options, plugins: [observer] });
  return received;
}

test("native parity uses the same normalized input and resolved EOL as Prettier", async () => {
  const auto = { endOfLine: "auto" };
  const lf = { endOfLine: "lf" };
  const mixed = "\uFEFFconst x=1;\nconst y=2;\r\n";
  assert.deepEqual(
    await observedInput(mixed, auto),
    normalizeInput(mixed, auto),
  );

  const cr = "const x=1;\rconst y=2;\n";
  assert.deepEqual(await observedInput(cr, auto), normalizeInput(cr, auto));

  const crlf = "const x=1;\r\nconst y=2;\r\n";
  assert.deepEqual(await observedInput(crlf, lf), normalizeInput(crlf, lf));

  const unix = "const x=1;\n";
  assert.deepEqual(await observedInput(unix, auto), normalizeInput(unix, auto));
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

function documentGraphSize(root) {
  const seen = new Set();
  const work = [root];
  while (work.length) {
    const doc = work.pop();
    if (doc === undefined || typeof doc === "string" || seen.has(doc)) continue;
    seen.add(doc);
    if (Array.isArray(doc)) work.push(...doc);
    else if (doc.type === "group" && doc.expandedStates)
      work.push(...doc.expandedStates);
    else if (doc.type === "fill") work.push(...doc.parts);
    else if (doc.type === "if-break")
      work.push(doc.breakContents, doc.flatContents);
    else work.push(doc.contents);
  }
  return seen.size;
}

test("SWCFMT retains bounded document sharing across nested alternatives", async () => {
  let source = "done();";
  for (let depth = 0; depth < 8; depth++)
    source = `f(x, () => {\n${source}\n});`;
  let doc;
  const observer = {
    ...plugin,
    printers: {
      SWCFMT: {
        ...plugin.printers.SWCFMT,
        print(path) {
          doc = plugin.printers.SWCFMT.print(path);
          return doc;
        },
      },
    },
  };
  const actual = await prettier.format(source, { ...js, plugins: [observer] });
  assert.equal(actual, formatNativeSync(source, "file.js", {}));
  assert.ok(documentGraphSize(doc) < source.length * 10);
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
