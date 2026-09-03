import { parseSync as __parseSync } from "./binding.js";
import { decode, encodeOptions } from "@swc-next/decoder";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, encodeOptions(options));
  return decode(buffer, source);
}
