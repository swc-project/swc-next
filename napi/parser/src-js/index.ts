import { parseSync as __parseSync, ParseOptions } from "./binding.js";
import { decode } from "./decode.js";

export function parseSync(source: string, options?: ParseOptions) {
  const buffer = __parseSync(source, options);
  return decode(buffer, source);
}

export { type ParseOptions } from "./binding.js";
