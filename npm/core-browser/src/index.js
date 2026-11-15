import {
  encodeSource,
  encodeTransformApiOptions,
  decodeJsOutput,
} from "@swc-next/encoder/transform";
import {
  parseSync as __parseSync,
  generateSync as __generateSync,
  transformSync as __transformSync,
} from "./swc-next-core.browser.mjs";
import {
  decode,
  encodeParserConfig,
} from "@swc-next/decoder";
import {
  encode,
  encodeGeneratorConfig,
} from "@swc-next/encoder";

const textEncoder = new TextEncoder();
const canTransferToFixedLength =
  typeof ArrayBuffer.prototype.transferToFixedLength === "function";

export {
  default,
  langFromPath,
  sourceTypeFromPath,
} from "./swc-next-core.browser.mjs";

export function parseSync(source, options) {
  const sourceBuffer = new Uint8Array(source.length * 3);
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

export function generateSync(program, options) {
  const code = __generateSync(...encode(program), encodeGeneratorConfig(options));
  return { code };
}

export function transformSync(source, options) {
  const config = encodeTransformApiOptions(options, encodeParserConfig, encodeGeneratorConfig);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
