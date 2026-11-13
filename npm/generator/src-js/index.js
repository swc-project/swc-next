import { generateSync as __generateSync } from "./binding.js";
import { encode, encodeGeneratorOptions } from "@swc-next/encoder";

export function generateSync(program, options) {
  const generatorConfig = encodeGeneratorOptions(options);
  const code = __generateSync(...encode(program), generatorConfig);
  return { code };
}
