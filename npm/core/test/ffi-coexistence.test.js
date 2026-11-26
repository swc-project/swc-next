import assert from "node:assert/strict";
import test from "node:test";
import * as legacy from "@swc-next/core";
import * as ffi2 from "@swc-next/core/ffi2";

// Both imports use the same native module, but their ASTs are independent.
test("legacy and FFI2 APIs coexist", () => {
  const source = 'const name = "é😀";';
  const plain = legacy.parseSync(source);
  const view = ffi2.parseSync(source);

  assert(Array.isArray(plain.program.body));
  assert(view instanceof ffi2.AstView);
  assert.equal(legacy.generateSync(plain.program).code, `${source}\n`);
  assert.equal(ffi2.generateSync(view.program).code, `${source}\n`);

  plain.program.body[0].declarations[0].id.name = "legacy";
  view.program.body.get(0).declarators.get(0).id.name = "view";
  assert.equal(legacy.generateSync(plain.program).code, 'const legacy = "é😀";\n');
  assert.equal(ffi2.generateSync(view.program).code, 'const view = "é😀";\n');
});
