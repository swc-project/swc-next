import {
  parseSync as __parseSync,
  printSync as __printSync,
} from "./swc-next-core.browser.mjs";
import {
  decode,
  encodeOptions as encodeParserOptions,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeCodegenOptions,
} from "@swc-next/encoder";

const textEncoder = new TextEncoder();

export {
  default,
  CommentMode,
  Lang,
  SourceType,
  langFromPath,
  sourceTypeFromPath,
} from "./swc-next-core.browser.mjs";

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
