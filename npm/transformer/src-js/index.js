import { encodeParserConfig } from "@swc-next/decoder";
import { encodeGeneratorConfig } from "@swc-next/encoder";
import { transformSync as __transformSync } from "./binding.js";
import {
  encodeSource,
  encodeTransformApiOptions,
  decodeJsOutput,
} from "@swc-next/encoder/transform";

export function transformSync(source, options) {
  const config = encodeTransformApiOptions(options, encodeParserConfig, encodeGeneratorConfig);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
