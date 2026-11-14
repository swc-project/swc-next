// Binary transform transport. Keep the layout aligned with
// crates/swc_core/src/js/transformer.rs.
const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { ignoreBOM: true });
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

let sourceBuffer = empty;

/** Encode source as UTF-8 into reusable scratch storage. Consume the returned
 * bytes synchronously before the next call; do not retain or modify them.
 */
export function encodeSource(source) {
  if (typeof source !== "string") throw new TypeError("source must be a string");
  const capacity = source.length * 3;
  if (sourceBuffer.length < capacity) {
    sourceBuffer = storage(Math.max(capacity, sourceBuffer.length * 2, 4096));
  }
  const { written } = encoder.encodeInto(source, sourceBuffer);
  return sourceBuffer.subarray(0, written);
}

let buffer = empty;
let view;

/** Encode the complete JS transform configuration into reusable scratch storage.
 * The returned buffer is valid until the next call. Option getters and Proxy traps
 * must not reenter this encoder, including via any transformSync entry point.
 */
export function encodeTransformApiOptions(options, encodeParserConfig, encodeGeneratorConfig) {
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

const utf8Slice = typeof Buffer !== "undefined" && typeof Buffer.prototype.utf8Slice === "function"
  ? Buffer.prototype.utf8Slice
  : function(start, end) { return decoder.decode(this.subarray(start, end)); };
const severities = ["error", "warning", "hint", "info"];

/** Decode either the native code-only string or a packed native/Wasm result. */
export function decodeJsOutput(buffer) {
  if (typeof buffer === "string") return { code: buffer, diagnostics: [] };
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const codeLength = view.getUint32(0, true);
  const mapLength = view.getUint32(4, true);
  const count = view.getUint32(8, true);
  const code = utf8Slice.call(buffer, 12, 12 + codeLength);
  let offset = 12 + codeLength;
  const map = mapLength === 0xffffffff ? undefined : utf8Slice.call(buffer, offset, offset + mapLength);
  if (map !== undefined) offset += mapLength;
  const diagnostics = new Array(count);
  const readU32 = () => { const value = view.getUint32(offset, true); offset += 4; return value; };
  const readString = () => {
    const length = readU32();
    const value = utf8Slice.call(buffer, offset, offset + length);
    offset += length;
    return value;
  };
  for (let i = 0; i < count; i++) {
    const severity = severities[buffer[offset++]];
    if (severity === undefined) throw new RangeError("Unknown diagnostic severity");
    const start = readU32();
    const end = readU32();
    const message = readString();
    const help = buffer[offset++] === 0 ? undefined : readString();
    const labels = new Array(readU32());
    for (let j = 0; j < labels.length; j++) labels[j] = { start: readU32(), end: readU32(), message: readString() };
    diagnostics[i] = help === undefined ? { severity, message, start, end, labels } : { severity, message, start, end, help, labels };
  }
  return map === undefined ? { code, diagnostics } : { code, map, diagnostics };
}
