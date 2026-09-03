import { beforeAll, expect, test } from "@rstest/core";

import init, * as core from "../index.js";
import { smokeTest } from "./smoke.js";

beforeAll(async () => {
  await init();
});

test("core-wasm exposes parseSync and printSync in browsers", () => {
  smokeTest(core, (actual, expected) => {
    expect(actual).toBe(expected);
  });
});
