import {
  parseSync as __parseSync,
  generateSync as __generateSync,
} from "./binding.js";
import {
  decode,
  encodeOptions as encodeParserConfig,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeGeneratorConfig,
} from "@swc-next/encoder";

const textEncoder = new TextEncoder();
const canTransferToFixedLength =
  typeof ArrayBuffer.prototype.transferToFixedLength === "function";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export * from "@swc-next/walker";

export function parseSync(source, options) {
  // Every byte read below is initialized by encodeInto; avoid zeroing the
  // oversized scratch buffer on Node. Keep the portable allocation for browsers.
  const sourceBuffer = typeof source === "string" && typeof Buffer === "function"
    ? new Uint8Array(Buffer.allocUnsafeSlow(source.length * 3).buffer)
    : new Uint8Array(source.length * 3);
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  // Lazy decoder getters retain sourceBytes, so drop the unused capacity of
  // the encodeInto buffer instead of keeping it alive through a subarray.
  const sourceBytes = canTransferToFixedLength
    ? new Uint8Array(sourceBuffer.buffer.transferToFixedLength(written))
    : sourceBuffer.slice(0, written);
  const buffer = __parseSync(sourceBytes, source.length, encodeParserConfig(options));
  const result = decode(buffer, source, sourceBytes);
  if (!options?.errorRecovery) {
    const diagnostics = result.diagnostics;
    if (diagnostics.length) throw diagnostics[0];
  }
  return result;
}

export function generateSync(program, options, meta) {
  const code = __generateSync(...encode(program, meta), encodeGeneratorConfig(options));
  return { code };
}
