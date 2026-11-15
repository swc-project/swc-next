// Internal SWC Next transport; these APIs and formats do not follow semver.
// Keep in sync with crates/swc_core/src/js/transformer.rs.
import { storage } from "./buffer.js";
import { encodeParserConfig } from "./parser.js";
import { encodeGeneratorConfig } from "./generator.js";

const encoder = new TextEncoder();
const empty = new Uint8Array();
const rootKeys = ["filename", "sourceMap", "comments", "parser", "generator"];
const parserKeys = ["sourceType", "lang", "preserveParens", "comments", "semanticErrors"];
const generatorKeys = ["asciiOnly", "minify", "inlineScript"];
const comments = ["all", "none"];

function object(value, keys, name) {
  if (value == null) return;
  if (typeof value !== "object" || Array.isArray(value)) throw new TypeError(`${name} must be an object`);
  for (const key of Object.keys(value)) {
    if (!keys.includes(key)) throw new TypeError(`Unknown ${name} option: ${key}`);
  }
}
function boolean(value, name) {
  if (value == null) return 0;
  if (typeof value !== "boolean") throw new TypeError(`${name} must be a boolean`);
  return value ? 2 : 1;
}
function enumeration(value, values, name) {
  if (value == null) return 0;
  const index = values.indexOf(value);
  if (index < 0) throw new TypeError(`Invalid ${name}`);
  return index + 1;
}

let buffer = empty;
let view;

/** Encode the complete JS transform configuration into reusable scratch storage.
 * The returned buffer is valid until the next call. Option getters and Proxy traps
 * must not reenter this encoder, including via any transformSync entry point.
 */
export function encodeTransformApiOptions(options) {
  if (options == null) return empty;
  object(options, rootKeys, "options");
  const parser = options.parser;
  const generator = options.generator;
  object(parser, parserKeys, "parser");
  object(generator, generatorKeys, "generator");
  const filename = options.filename;
  if (filename != null && typeof filename !== "string") throw new TypeError("filename must be a string");
  const capacity = filename == null ? 16 : 20 + filename.length * 3;
  if (buffer.length < capacity) {
    buffer = storage(Math.max(capacity, buffer.length * 2, 256));
    view = new DataView(buffer.buffer);
  }
  view.setUint32(0, 1, true);
  view.setUint32(4, encodeParserConfig(parser), true);
  view.setUint32(8, encodeGeneratorConfig(generator), true);
  view.setUint32(12, boolean(options.sourceMap, "sourceMap")
    | enumeration(options.comments, comments, "comments") << 2, true);
  if (filename == null) return buffer.subarray(0, 16);
  const { written } = encoder.encodeInto(filename, buffer.subarray(20));
  view.setUint32(16, written, true);
  return buffer.subarray(0, 20 + written);
}
