import { runTransformPipeline } from "@swc-next/protocol/plugins";
import * as transformBindings from "./binding.js";
import { transformSync as __transformSync } from "./binding.js";
import { encodeSource } from "@swc-next/protocol/source";
import { encodeTransformApiOptions } from "@swc-next/protocol/transform";
import { decodeJsOutput } from "@swc-next/protocol/output";

export function transformSync(source, options) {
  if (options?.plugins !== undefined) return runTransformPipeline(source, options, transformBindings);
  const config = encodeTransformApiOptions(options);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
