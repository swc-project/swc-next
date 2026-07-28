import { parseSync as __parseSync } from "./binding.js";
import { decode } from "./decode.js";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, options);
  return {
    program: decode(buffer, source),
  };
}
