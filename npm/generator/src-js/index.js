import { generateSync as __generateSync } from "./binding.js";
import { encode, encodeOptions } from "@swc-next/encoder";

export function generateSync(program, options, meta) {
  const code = __generateSync(...encode(program, meta), encodeOptions(options));
  return { code };
}
