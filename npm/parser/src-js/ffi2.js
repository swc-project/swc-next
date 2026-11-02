import { encodeSource } from "@swc-next/protocol/source";
import { encodeParserConfig } from "@swc-next/protocol/parser";
import { readAst } from "@swc-next/ast2";
export { readAst, AstView, NodeView, NodeKind, AstList } from "@swc-next/ast2";
import { parseSync2 as __parseSync } from "./binding.js";
export { langFromPath, sourceTypeFromPath } from "./binding.js";

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
