// Internal bridge option word; this API and format do not follow semver.
// Keep in sync with crates/swc_core/src/js/parser.rs.
const OPTION_SOURCE_TYPES = ["module", "script", "commonjs", "unambiguous"];
const OPTION_LANGS = ["js", "ts", "jsx", "tsx", "dts"];
const OPTION_COMMENT_MODES = ["all", "none"];

export function encodeParserConfig(options) {
  if (options === undefined || options === null) return 0;
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("options must be an object");
  }

  encodeOptionBoolean(options.errorRecovery, "errorRecovery");

  let value = 1;
  value |= encodeOptionEnum(options.sourceType, "sourceType", OPTION_SOURCE_TYPES) << 1;
  value |= encodeOptionEnum(options.lang, "lang", OPTION_LANGS) << 4;
  value |= encodeOptionBoolean(options.preserveParens, "preserveParens") << 7;
  value |= encodeOptionEnum(options.comments, "comments", OPTION_COMMENT_MODES) << 9;
  value |= encodeOptionBoolean(options.semanticErrors, "semanticErrors") << 11;
  return value >>> 0;
}

function encodeOptionEnum(value, name, values) {
  if (value === undefined || value === null) return 0;
  const index = values.indexOf(value);
  if (index === -1) {
    throw new TypeError(`${name} must be one of: ${values.join(", ")}`);
  }
  return index + 1;
}

function encodeOptionBoolean(value, name) {
  if (value === undefined || value === null) return 0;
  if (typeof value !== "boolean") {
    throw new TypeError(`${name} must be a boolean`);
  }
  return value ? 2 : 1;
}
