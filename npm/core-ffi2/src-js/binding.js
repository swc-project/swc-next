import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const forceWasi = process.env.NAPI_RS_FORCE_WASI;
const binding = forceWasi === "true" || forceWasi === "error"
  ? require("./swc-next-core-ffi2.node.mjs")
  : require("./swc-next-core-ffi2.node");

export const {
  generateSync2,
  generateSync2Unchecked,
  langFromPath,
  parseSync2,
  sourceTypeFromPath,
  transformSync,
} = binding;
