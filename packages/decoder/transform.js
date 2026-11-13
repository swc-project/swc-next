import { encodeParserOptions } from "./options.js";
import { encodeGeneratorOptions } from "@swc-next/encoder/options";
import { decodeDiagnostics } from "./diagnostics.js";
import { utf8Slice } from "./utf8.js";

// Binary transform transport. Keep the layout aligned with
// crates/swc_core/src/js/transformer.rs.
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
function storage(length) {
  return typeof Buffer === "function"
    ? new Uint8Array(Buffer.allocUnsafeSlow(length).buffer)
    : new Uint8Array(length);
}

/** Create an options encoder with synchronous, reusable scratch storage.
 * The encoder is not reentrant. Option getters and Proxy traps must not
 * reenter it, including via its transformSync wrapper.
 */
export function createTransformOptionsEncoder() {
  let buffer = empty;
  let view;
  return function encodeOptions(options) {
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
    view.setUint32(4, encodeParserOptions(parser), true);
    view.setUint32(8, encodeGeneratorOptions(generator), true);
    view.setUint32(12, boolean(options.sourceMap, "sourceMap")
      | enumeration(options.comments, comments, "comments") << 2, true);
    if (filename == null) return buffer.subarray(0, 16);
    const { written } = encoder.encodeInto(filename, buffer.subarray(20));
    view.setUint32(16, written, true);
    return buffer.subarray(0, 20 + written);
  };
}

const encodeOptions = createTransformOptionsEncoder();
/** The returned buffer is scratch storage, valid until the next encoding call.
 * Option getters and Proxy traps must not reenter this shared encoder.
 */
export { encodeOptions as encodeTransformOptions };

export function decodeTransform(buffer) {
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const codeLength = view.getUint32(0, true);
  const mapLength = view.getUint32(4, true);
  const count = view.getUint32(8, true);
  const code = utf8Slice.call(buffer, 12, 12 + codeLength);
  let offset = 12 + codeLength;
  const map = mapLength === 0xffffffff ? undefined : utf8Slice.call(buffer, offset, offset + mapLength);
  if (map !== undefined) offset += mapLength;
  const diagnostics = decodeDiagnostics(buffer, view, offset, count, true);
  return map === undefined ? { code, diagnostics } : { code, map, diagnostics };
}

/** Build a synchronous transform API around a binary native or Wasm binding.
 * Each wrapper owns a non-reentrant options encoder. Option getters and Proxy
 * traps must not reenter the same wrapper while it is encoding options.
 */
export function createTransformSync(binding) {
  const encodeOptions = createTransformOptionsEncoder();
  let sourceBuffer = empty;
  return function transformSync(source, options) {
    if (typeof source !== "string") throw new TypeError("source must be a string");
    const config = encodeOptions(options);
    const capacity = source.length * 3;
    if (sourceBuffer.length < capacity) {
      sourceBuffer = storage(Math.max(capacity, sourceBuffer.length * 2, 4096));
    }
    const { written } = encoder.encodeInto(source, sourceBuffer);
    // Synchronous Rust processing borrows these bytes only for this call.
    const result = binding(sourceBuffer.subarray(0, written), source.length, config);
    return typeof result === "string" ? { code: result, diagnostics: [] } : decodeTransform(result);
  };
}
