import { runTransformPipeline } from "@swc-next/protocol/plugins";
import * as transformBindings from "./binding.js";
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

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export * from "@swc-next/walker";

export function parseSync(source, options) {
  const parserConfig = encodeParserConfig(options);
  const buffer = __parseSync(encodeSource(source), source.length, parserConfig);
  const result = decode(buffer, source);
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
  if (options?.plugins !== undefined) return runTransformPipeline(source, options, transformBindings);
  const config = encodeTransformApiOptions(options);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
