import * as binding from "./binding.js";
import { createApi } from "./api.js";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export { readAst, AstView, NodeView, NodeKind, AstList } from "./protocol/index.ts";
export * from "./walker/index.js";
export const { parseSync, parseBufferSync, generateSync, generateBuffersSync, transformSync } =
  createApi(binding);
