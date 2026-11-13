import { transformSync as __transformSync } from "./binding.js";
import {
  encodeSource,
  encodeTransformApiOptions,
  decodeJsOutput,
} from "@swc-next/transformer-bridge";

export function transformSync(source, options) {
  const config = encodeTransformApiOptions(options);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
