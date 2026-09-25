import {
  parseSync as __parseSync,
  printSync as __printSync,
} from "./binding.js";
import {
  decode,
  encodeOptions as encodeParserOptions,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeCodegenOptions,
} from "@swc-next/encoder";

const textEncoder = new TextEncoder();

export * from "./binding.js";

export function parseSync(source, options) {
  const sourceBuffer = new Uint8Array(source.length * 3);
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  const sourceBytes = sourceBuffer.subarray(0, written);
  const buffer = __parseSync(sourceBytes, encodeParserOptions(options));
  return decode(buffer, sourceBytes);
}

export function printSync(program, options) {
  const code = __printSync(encode(program), encodeCodegenOptions(options));
  return { code };
}
