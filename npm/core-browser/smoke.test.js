// This is the only test that runs in the browser.
// It is used to verify that the browser build exposes the correct API and works as expected.
// Don't add any other tests here

import { expect, test } from "@rstest/core";

import init, { parseSync, generateSync } from "@swc-next/core-browser";
import * as core from "@swc-next/core-browser";

test("browser exposes parsing and generation", async () => {
  await init();
  expect("Lang" in core).toBe(false);
  expect("SourceType" in core).toBe(false);
  expect("CommentMode" in core).toBe(false);
  expect(core.langFromPath("input.ts")).toBe("ts");
  expect(core.sourceTypeFromPath("input.cjs")).toBe("commonjs");
  const result = parseSync("/* 😀 */ const value = 1;", {
    lang: "ts",
    sourceType: "script",
    comments: "none",
  });
  parseSync("const other = 0;");
  const { program } = result;
  expect(program.body.get(0).declarations.get(0).id.name).toBe("value");
  program.body.get(0).declarations.get(0).init.value = 2;

  expect(generateSync(program).code).toBe("const value = 2;\n");
});
