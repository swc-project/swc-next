import * as binding from "../dist/swc-next-core-ffi2.browser.mjs";
import { createApi } from "./api.js";

export { default, langFromPath, sourceTypeFromPath } from "../dist/swc-next-core-ffi2.browser.mjs";
export { readAst, AstView, NodeView, NodeKind, AstList } from "./protocol/index.ts";
export * from "./walker/index.js";
export const { parseSync, parseBufferSync, generateSync, generateBuffersSync, transformSync } =
  createApi(binding);
