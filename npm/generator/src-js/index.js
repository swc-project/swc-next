import { generateSync as __generateSync } from "./binding.js";
import { encode, encodeOptions } from "@swc-next/encoder";

export function generateSync(program, options) {
  const generatorConfig = encodeOptions(options);
  const code = __generateSync(...encode(program), generatorConfig);
  return { code };
}
