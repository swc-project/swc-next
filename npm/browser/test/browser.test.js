import { beforeAll, expect, test } from "@rstest/core";

import init, * as core from "@swc-next/browser";
import { smokeTest } from "./smoke.js";

beforeAll(async () => {
  await init();
});

test("browser exposes parseSync and printSync", () => {
  smokeTest(core, (actual, expected) => {
    expect(actual).toBe(expected);
  });
});
