import { generateSync as __generateSync } from "./binding.js";
import { encode, encodeGeneratorConfig } from "@swc-next/encoder";

export function generateSync(program, options) {
  const generatorConfig = encodeGeneratorConfig(options);
  const code = __generateSync(...encode(program), generatorConfig);
  return { code };
}
