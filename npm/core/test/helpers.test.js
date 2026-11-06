import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

import {
  _set_function_name as esmSetFunctionName,
  _set_class_name as esmSetClassName,
  _to_property_key as esmToPropertyKey,
} from "@swc-next/helpers";

const require = createRequire(import.meta.url);
const {
  _set_function_name: cjsSetFunctionName,
  _set_class_name: cjsSetClassName,
  _to_property_key: cjsToPropertyKey,
} = require("@swc-next/helpers");

for (const [format, setFunctionName, toPropertyKey, setClassName] of [
  ["ESM", esmSetFunctionName, esmToPropertyKey, esmSetClassName],
  ["CommonJS", cjsSetFunctionName, cjsToPropertyKey, cjsSetClassName],
]) {
  test(`${format} helper exports preserve property keys`, () => {
    assert.equal(toPropertyKey(12), "12");
    assert.equal(toPropertyKey(null), "null");
    assert.equal(toPropertyKey("name"), "name");

    const key = Symbol("name");
    assert.equal(toPropertyKey(key), key);

    let calls = 0;
    const value = {
      [Symbol.toPrimitive](hint) {
        assert.equal(this, value);
        assert.equal(hint, "string");
        calls++;
        return key;
      },
    };
    assert.equal(toPropertyKey(value), key);
    assert.equal(calls, 1);

    const ordinary = { toString: () => ({}), valueOf: () => 42 };
    assert.equal(toPropertyKey(ordinary), "42");

    const invalid = { [Symbol.toPrimitive]: () => ({}) };
    assert.throws(() => toPropertyKey(invalid), TypeError);

    const error = new Error("conversion failed");
    const throwing = { [Symbol.toPrimitive]() { throw error; } };
    assert.throws(() => toPropertyKey(throwing), (caught) => caught === error);
  });

  test(`${format} helper exports set function names`, () => {
    const fn = function () {};
    assert.equal(setFunctionName(fn, "renamed"), fn);
    assert.deepEqual(Object.getOwnPropertyDescriptor(fn, "name"), {
      value: "renamed",
      configurable: true,
      enumerable: false,
      writable: false,
    });

    const cls = class {};
    assert.equal(setFunctionName(cls, Symbol("Widget")), cls);
    assert.equal(cls.name, "[Widget]");

    const unnamed = function () {};
    setFunctionName(unnamed, Symbol());
    assert.equal(unnamed.name, "");

    const getter = function () {};
    setFunctionName(getter, "value", "get");
    assert.equal(getter.name, "get value");

    const frozen = Object.freeze(function original() {});
    assert.equal(setFunctionName(frozen, "renamed"), frozen);
    assert.equal(frozen.name, "original");
  });

  test(`${format} helper exports restore class names before static fields`, () => {
    const Renamed = class {
      static #name = setClassName(this, "Original");
      static value = this.name;
    };
    assert.equal(Renamed.value, "Original");

    const key = "name";
    const Method = class {
      static #name = setClassName(this, "Original");
      static [key]() { return "method"; }
      static value = this.name();
    };
    assert.equal(Method.value, "method");

    let reads = 0;
    const Accessor = class {
      static #name = setClassName(this, "Original");
      static get [key]() { reads++; return "accessor"; }
      static value = this.name;
    };
    assert.equal(Accessor.value, "accessor");
    assert.equal(reads, 1);
  });
}
