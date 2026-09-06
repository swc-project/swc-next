// This is the only test that runs in the browser.
// It is used to verify that the browser build exposes the correct API and works as expected.
// Don't add any other tests here

import { expect, test } from "@rstest/core";

import init, { parseSync, printSync } from "@swc-next/core-browser";

test("browser exposes parseSync and printSync", async () => {
  await init();
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].init.value = 2;

  expect(printSync(program).code).toBe("const value = 2;\n");
});
