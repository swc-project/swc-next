import assert from "node:assert/strict";
import test from "node:test";
import * as prettier from "prettier";
import { formatDocBufferSync } from "@swc-next/ecma-formatter";
import * as plugin from "../index.js";

const js = { parser: "swc", plugins: [plugin] };
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
  await matches("const x=1;");
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
