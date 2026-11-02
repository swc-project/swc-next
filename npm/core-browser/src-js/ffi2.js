import { encodeSource } from "@swc-next/protocol/source";
import { encodeParserConfig } from "@swc-next/protocol/parser";
import { readAst, NodeView } from "@swc-next/ast2";
export { readAst, AstView, NodeView, NodeKind, AstList } from "@swc-next/ast2";
import { encodeTransformApiOptions } from "@swc-next/protocol/transform";
import { decodeJsOutput } from "@swc-next/protocol/output";
import { encodeGeneratorConfig } from "@swc-next/protocol/generator";
import {
  parseSync2 as __parseSync,
  generateSync2 as __generateSync,
  transformSync as __transformSync,
} from "./swc-next-core.browser.mjs";
export { default, langFromPath, sourceTypeFromPath } from "./swc-next-core.browser.mjs";

export function parseBufferSync(source, options) {
  const config = encodeParserConfig(options);
  return __parseSync(encodeSource(source), source.length, config);
}

export function parseSync(source, options) {
  const ast = readAst(parseBufferSync(source, options));
  if (!options?.errorRecovery && ast.diagnostics.length) {
    throw ast.diagnostics[0];
  }
  return ast;
}

export function generateSync(program, options) {
  if (!(program instanceof NodeView) || program.nodeId !== program.ast.rootId) {
    throw new TypeError("Expected the root Program view of an AST");
  }
  const config = encodeGeneratorConfig(options);
  const buffers = program.ast.encode();
  return { code: __generateSync(...buffers, config) };
}

export function generateBuffersSync(buffer, extension = new Uint8Array(), options) {
  return { code: __generateSync(buffer, extension ?? new Uint8Array(), encodeGeneratorConfig(options)) };
}

export function transformSync(source, options) {
  const config = encodeTransformApiOptions(options);
  const output = __transformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
