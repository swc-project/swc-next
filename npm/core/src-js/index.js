import {
  encodeSource,
  encodeTransformApiOptions,
  decodeJsOutput,
} from "@swc-next/encoder/transform";
import {
  parseSync as __parseSync,
  generateSync as __generateSync,
  transformSync as __transformSync,
} from "./binding.js";
import {
  decode,
  encodeParserConfig,
} from "@swc-next/decoder";
import {
  encode,
  encodeGeneratorConfig,
} from "@swc-next/encoder";

const textEncoder = new TextEncoder();
let sourceBuffer = new Uint8Array();

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export * from "@swc-next/walker";

export function parseSync(source, options) {
  const parserConfig = encodeParserConfig(options);
  const capacity = source.length * 3;
  if (sourceBuffer.length < capacity) {
    const length = Math.max(capacity, sourceBuffer.length * 2, 4096);
    sourceBuffer = typeof Buffer === "function"
      ? new Uint8Array(Buffer.allocUnsafeSlow(length).buffer)
      : new Uint8Array(length);
  }
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  // Native parsing is synchronous. Lazy decoding reads the original string,
  // so this scratch buffer can be overwritten by the next parse.
  const sourceBytes = sourceBuffer.subarray(0, written);
  const buffer = __parseSync(sourceBytes, source.length, parserConfig);
  const result = decode(buffer, source, sourceBytes);
  if (!options?.errorRecovery) {
    const diagnostics = result.diagnostics;
    if (diagnostics.length) throw diagnostics[0];
  }
  return result;
}

export function generateSync(program, options) {
  const generatorConfig = encodeGeneratorConfig(options);
  const code = __generateSync(...encode(program), generatorConfig);
  return { code };
}

export function transformSync(source, options) {
  const config = encodeTransformApiOptions(options, encodeParserConfig, encodeGeneratorConfig);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
