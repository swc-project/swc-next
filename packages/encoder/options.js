export function encodeGeneratorConfig(options) {
  if (options === undefined || options === null) return 0;
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("options must be an object");
  }

  let value = 1;
  value |= encodeOptionBoolean(options.asciiOnly, "asciiOnly") << 1;
  value |= encodeOptionBoolean(options.minify, "minify") << 3;
  value |= encodeOptionBoolean(options.inlineScript, "inlineScript") << 5;
  return value >>> 0;
}

function encodeOptionBoolean(value, name) {
  if (value === undefined || value === null) return 0;
  if (typeof value !== "boolean") {
    throw new TypeError(`${name} must be a boolean`);
  }
  return value ? 2 : 1;
}
