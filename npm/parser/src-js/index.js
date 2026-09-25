import { parseSync as __parseSync } from "./binding.js";
import { decode, encodeOptions } from "@swc-next/decoder";

const textEncoder = new TextEncoder();

export * from "./binding.js";

export function parseSync(source, options) {
  const sourceBuffer = new Uint8Array(source.length * 3);
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  const sourceBytes = sourceBuffer.subarray(0, written);
  const buffer = __parseSync(sourceBytes, encodeOptions(options));
  return decode(buffer, source, sourceBytes);
}
