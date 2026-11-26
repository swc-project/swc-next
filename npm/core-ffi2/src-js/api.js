import { encodeSource } from "@swc-next/protocol/source";
import { encodeParserConfig } from "@swc-next/protocol/parser";
import { encodeTransformApiOptions } from "@swc-next/protocol/transform";
import { decodeJsOutput } from "@swc-next/protocol/output";
import { encodeGeneratorConfig } from "@swc-next/protocol/generator";
import { readAst, NodeView } from "#protocol/index";

export function createApi(binding) {
  function parseBufferSync(source, options) {
    const config = encodeParserConfig(options);
    return binding.parseSync2(encodeSource(source), source.length, config);
  }

  function parseSync(source, options) {
    const ast = readAst(parseBufferSync(source, options), source);
    if (!options?.errorRecovery && ast.diagnostics.length) {
      throw ast.diagnostics[0];
    }
    return ast;
  }

  function generateSync(program, options) {
    if (!(program instanceof NodeView) || program.nodeId !== program.ast.rootId) {
      throw new TypeError("Expected the root Program view of an AST");
    }
    const config = encodeGeneratorConfig(options);
    return { code: binding.generateSync2(...program.ast.encode(), config) };
  }

  function generateBuffersSync(buffer, extension = new Uint8Array(), options) {
    const config = encodeGeneratorConfig(options);
    return { code: binding.generateSync2(buffer, extension ?? new Uint8Array(), config) };
  }

  function transformSync(source, options) {
    const config = encodeTransformApiOptions(options);
    const output = binding.transformSync(encodeSource(source), source.length, config);
    return decodeJsOutput(output);
  }

  return { parseSync, parseBufferSync, generateSync, generateBuffersSync, transformSync };
}
