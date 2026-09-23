import { decode_js } from "./decode_js.js";
import { decode_ts } from "./decode_ts.js";

export { decode_js, decode_ts };

const FLAGS_OFFSET = 32;
const TYPESCRIPT_FLAG = 1;

const OPTION_SOURCE_TYPES = ["module", "script", "commonjs", "unambiguous"];
const OPTION_LANGS = ["js", "ts", "jsx", "tsx", "dts"];
const OPTION_COMMENT_MODES = ["none", "flat"];

export function encodeOptions(options) {
  if (options === undefined || options === null) return 0;
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("options must be an object");
  }

  let value = 1;
  value |= encodeOptionEnum(options.sourceType, "sourceType", OPTION_SOURCE_TYPES) << 1;
  value |= encodeOptionEnum(options.lang, "lang", OPTION_LANGS) << 4;
  value |= encodeOptionBoolean(options.preserveParens, "preserveParens") << 7;
  value |= encodeOptionEnum(options.comments, "comments", OPTION_COMMENT_MODES) << 9;
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

export function decode(buffer, source) {
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const view = new DataView(arrayBuffer, buffer.byteOffset ?? 0, buffer.byteLength);
  const flags = view.getUint32(FLAGS_OFFSET, true);
  return (flags & TYPESCRIPT_FLAG) !== 0
    ? decode_ts(buffer, source)
    : decode_js(buffer, source);
}
