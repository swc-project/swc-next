// This is the only test that runs in the browser.
// It is used to verify that the browser build exposes the correct API and works as expected.
// Don't add any other tests here

import { expect, test } from "@rstest/core";

import init, { parseSync, printSync } from "@swc-next/core-browser";
import * as core from "@swc-next/core-browser";

test("browser exposes parseSync and printSync", async () => {
  await init();
  expect("Lang" in core).toBe(false);
  expect("SourceType" in core).toBe(false);
  expect("CommentMode" in core).toBe(false);
  expect(core.langFromPath("input.ts")).toBe("ts");
  expect(core.sourceTypeFromPath("input.cjs")).toBe("commonjs");
  const { program } = parseSync("/* 😀 */ const value = 1;", {
    lang: "ts",
    sourceType: "script",
    comments: "none",
  });
  expect(program.body[0].declarations[0].id.name).toBe("value");
  program.body[0].declarations[0].init.value = 2;

  expect(printSync(program).code).toBe("const value = 2;\n");
});
