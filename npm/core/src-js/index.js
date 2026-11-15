import { encodeSource } from "@swc-next/protocol/source";
import { encodeTransformApiOptions } from "@swc-next/protocol/transform";
import { decodeJsOutput } from "@swc-next/protocol/output";
import {
  parseSync as __parseSync,
  generateSync as __generateSync,
  transformSync as __transformSync,
} from "./binding.js";
import { decode } from "@swc-next/protocol/ast/decode";
import { encodeParserConfig } from "@swc-next/protocol/parser";
import { encode } from "@swc-next/protocol/ast/encode";
import { encodeGeneratorConfig } from "@swc-next/protocol/generator";

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
  const config = encodeTransformApiOptions(options);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
