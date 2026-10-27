import { printSync as __printSync } from "./binding.js";
import { encode, encodeOptions } from "@swc-next/encoder";

export function printSync(program, options, meta) {
  const code = __printSync(...encode(program, meta), encodeOptions(options));
  return { code };
}
