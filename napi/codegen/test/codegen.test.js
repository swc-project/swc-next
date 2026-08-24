import assert from "node:assert/strict";
import test from "node:test";

import { Lang, parseSync } from "../../parser/src-js/index.js";
import { printSync } from "../src-js/index.js";

test("printSync prints parser-produced ESTree", () => {
  const { program } = parseSync("const answer = 1 + 2;");

  assert.deepEqual(printSync(program), { code: "const answer = 1 + 2;\n" });
});

test("printSync observes ESTree modifications", () => {
  const { program } = parseSync("const answer = 1;");
  program.body[0].declarations[0].id.name = "changed";
  program.body[0].declarations[0].init.value = 42;

  assert.equal(printSync(program).code, "const changed = 42;\n");
});

test("printSync omits parser comments", () => {
  const { program } = parseSync("/* omitted */ const value = 1;");

  assert.equal(printSync(program).code, "const value = 1;\n");
});

test("printSync prints TypeScript and JSX", () => {
  const { program } = parseSync(
    "const element: JSX.Element = <Box value={1} />;",
    { lang: Lang.Tsx },
  );

  assert.equal(
    printSync(program).code,
    "const element: JSX.Element = <Box value={1} />;\n",
  );
});

test("printSync prints TypeScript module blocks", () => {
  const { program } = parseSync("namespace NS { export const value = 1; }", {
    lang: Lang.Ts,
  });

  assert.equal(
    printSync(program).code,
    "namespace NS {\n  export const value = 1;\n}\n",
  );
});

test("printSync observes JSX string modifications", () => {
  const { program } = parseSync('const element = <Box label="before" />;', {
    lang: Lang.Tsx,
  });
  program.body[0].declarations[0].init.openingElement.attributes[0].value.value =
    "after";

  assert.equal(
    printSync(program).code,
    'const element = <Box label="after" />;\n',
  );
});

test("printSync forwards codegen options", () => {
  const { program } = parseSync("const emoji = '😀';");

  assert.equal(
    printSync(program, { asciiOnly: true, minify: true }).code,
    'const emoji="\\u{1F600}";',
  );
});

test("printSync rejects unsupported ESTree nodes", () => {
  assert.throws(
    () =>
      printSync({
        type: "Program",
        sourceType: "script",
        body: [{ type: "Unsupported" }],
      }),
    /Unsupported ESTree node type Unsupported/,
  );
});
