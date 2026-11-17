export function _set_function_name(fn, name, prefix) {
  if (typeof name === "symbol") {
    name = name.description;
    // ECMA-262 SetFunctionName, step 2: only an undefined Symbol description
    // produces an empty name; an empty description produces "[]".
    // https://tc39.es/ecma262/#sec-setfunctionname
    name = name === void 0 ? "" : "[" + name + "]";
  }
  // Match SWC: an empty prefix means no prefix.
  // Older engines can expose a non-configurable name property.
  try {
    Object.defineProperty(fn, "name", {
      configurable: true,
      value: prefix ? prefix + " " + name : name,
    });
  } catch (_) {}
  return fn;
}
