export { encodeParserConfig } from "./options.js";
import { decodeJs } from "./decode-js.js";
import { decodeTs } from "./decode-ts.js";

export { decodeJs, decodeTs };

const FLAGS_OFFSET = 32;
const LANG_SHIFT = 2;

export function decode(buffer, source, sourceBuffer) {
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const view = new DataView(arrayBuffer, buffer.byteOffset ?? 0, buffer.byteLength);
  const flags = view.getInt32(FLAGS_OFFSET, true);
  const lang = (flags >> LANG_SHIFT) & 7;
  return lang === 1 || lang === 3 || lang === 4
    ? decodeTs(buffer, source, sourceBuffer)
    : decodeJs(buffer, source, sourceBuffer);
}
