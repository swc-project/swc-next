// Adapted from Babel's setFunctionName helper. See THIRD_PARTY_NOTICES.
function _set_function_name(fn, name, prefix) {
  if (typeof name === "symbol") {
    name = name.description;
    name = name ? "[" + name + "]" : "";
  }
  // Older engines can expose a non-configurable name property.
  try {
    Object.defineProperty(fn, "name", {
      configurable: true,
      value: prefix ? prefix + " " + name : name,
    });
  } catch (_) {}
  return fn;
}
export { _set_function_name as _ };
