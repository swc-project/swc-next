import { decodeJs } from "./decode-js.js";
import { decodeTs } from "./decode-ts.js";

export { decodeJs, decodeTs };

const FLAGS_OFFSET = 32;
const LANG_SHIFT = 2;

const OPTION_SOURCE_TYPES = ["module", "script", "commonjs", "unambiguous"];
const OPTION_LANGS = ["js", "ts", "jsx", "tsx", "dts"];
const OPTION_COMMENT_MODES = ["none", "flat"];

export function encodeOptions(options) {
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

export function decode(buffer, source, sourceBuffer) {
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const view = new DataView(arrayBuffer, buffer.byteOffset ?? 0, buffer.byteLength);
  const flags = view.getInt32(FLAGS_OFFSET, true);
  const lang = (flags >> LANG_SHIFT) & 7;
  return lang === 1 || lang === 3 || lang === 4
    ? decodeTs(buffer, source, sourceBuffer)
    : decodeJs(buffer, source, sourceBuffer);
}
