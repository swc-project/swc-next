import { test, expect } from "@rstest/core";
import { parseSync as swcExpParseSync } from "./index";
import { parseSync as swcParseSync } from "@swc/core";

test("should sayHi correctly", () => {
  const source = "let a = 1";
  expect(swcExpParseSync(source)).toStrictEqual(swcParseSync(source));
});
