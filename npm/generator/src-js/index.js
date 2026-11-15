import { generateSync as __generateSync } from "./binding.js";
import { encode } from "@swc-next/protocol/ast/encode";
import { encodeGeneratorConfig } from "@swc-next/protocol/generator";

export function generateSync(program, options) {
  const generatorConfig = encodeGeneratorConfig(options);
  const code = __generateSync(...encode(program), generatorConfig);
  return { code };
}
