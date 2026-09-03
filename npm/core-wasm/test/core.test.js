import assert from "node:assert/strict";
import test from "node:test";

import * as core from "../index.js";
import { smokeTest } from "./smoke.js";

test("core-wasm exposes parseSync and printSync in Node.js", () => {
  smokeTest(core, assert.equal);
});
