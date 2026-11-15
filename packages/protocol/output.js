// Internal SWC Next transport; these APIs and formats do not follow semver.
// Keep in sync with crates/swc_core/src/js/transformer.rs.
const decoder = new TextDecoder("utf-8", { ignoreBOM: true });
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
