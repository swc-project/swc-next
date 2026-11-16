// Internal SWC Next transport; use matching native/Wasm bindings.
// Keep in sync with crates/swc_formatter/src/{lib,js}.rs.
const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const encoder = new TextEncoder();
const layout = [
  ["lineWidth", 65535],
  ["indentStyle", ["tab", "space"]],
  ["indentWidth", 64],
  ["lineEnding", ["lf", "crlf", "cr"]],
];
const syntax = [
  ["semicolons", ["always", "as-needed"]],
  ["quoteStyle", ["double", "single"]],
  ["jsxQuoteStyle", ["double", "single"]],
  ["quoteProperties", ["as-needed", "consistent", "preserve"]],
  ["trailingCommas", ["all", "es5", "none"]],
  ["arrowParentheses", ["always", "as-needed"]],
  ["bracketSpacing", [false, true]],
  ["bracketSameLine", [false, true]],
  ["singleAttributePerLine", [false, true]],
];

function encodeConfig(options, fields) {
  if (options == null) return new Uint8Array();
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("options must be an object");
  }
  // Version u32, presence mask u32, line width u16, then one byte per field.
  const buffer = new Uint8Array(9 + fields.length);
  const view = new DataView(buffer.buffer);
  view.setUint32(0, 1, true);
  let mask = 0;
  for (let index = 0; index < fields.length; index++) {
    const [name, domain] = fields[index];
    const value = options[name];
    if (value == null) continue;
    let encoded;
    if (typeof domain === "number") {
      if (!Number.isInteger(value) || value < 0 || value > domain) {
        throw new RangeError(`${name} must be between 0 and ${domain}`);
      }
      encoded = value;
    } else {
      encoded = domain.indexOf(value);
      if (encoded < 0) throw new TypeError(`Invalid ${name}`);
    }
    mask |= 1 << index;
    if (index === 0) view.setUint16(8, encoded, true);
    else buffer[9 + index] = encoded;
  }
  view.setUint32(4, mask, true);
  return buffer;
}

export function encodePrintConfig(options) {
  return encodeConfig(options, layout);
}

export function encodeFormatterConfig(options) {
  return encodeConfig(options, [...layout, ...syntax]);
}

/** Decode SWCD success or SWCE failure without constructing a printer Doc. */
export function decodeFormatDocOutput(buffer, source) {
  const invalid = () => new Error("Invalid formatter result buffer");
  if (!(buffer instanceof Uint8Array) || buffer.length < 8) throw invalid();
  const view = new DataView(
    buffer.buffer,
    buffer.byteOffset,
    buffer.byteLength,
  );
  if (view.getUint32(4, true) !== 0) throw invalid();
  const magic = view.getUint32(0, true);
  if (magic === 0x44435753) return { buffer };
  if (magic !== 0x45435753 || buffer.length < 16) throw invalid();
  const start = view.getUint32(8, true);
  const end = view.getUint32(12, true);
  const bytes = encoder.encode(source);
  if (start > end || end > bytes.length) throw invalid();
  return {
    error: {
      message: utf8.decode(buffer.subarray(16)),
      start: utf8.decode(bytes.subarray(0, start)).length,
      end: utf8.decode(bytes.subarray(0, end)).length,
    },
  };
}

export function decodePrintDocOutput(buffer) {
  const invalid = () => new Error("Invalid printer result buffer");
  if (!(buffer instanceof Uint8Array) || buffer.length < 1) throw invalid();
  if (buffer[0] === 0) return { formatted: utf8.decode(buffer.subarray(1)) };
  if (buffer[0] !== 1 || buffer.length < 9) throw invalid();
  const view = new DataView(
    buffer.buffer,
    buffer.byteOffset,
    buffer.byteLength,
  );
  const start = view.getUint32(1, true);
  const end = view.getUint32(5, true);
  const bytes = buffer.subarray(9);
  if (start > end || end > bytes.length) throw invalid();
  return {
    formatted: utf8.decode(bytes),
    cursorNodeStart: utf8.decode(bytes.subarray(0, start)).length,
    cursorNodeText: utf8.decode(bytes.subarray(start, end)),
  };
}
