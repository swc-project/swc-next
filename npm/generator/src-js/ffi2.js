import { generateSync2 as __generateSync } from "./binding.js";
import { NodeView } from "@swc-next/ast2";
import { encodeGeneratorConfig } from "@swc-next/protocol/generator";

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
