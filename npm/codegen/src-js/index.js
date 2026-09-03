import { printSync as __printSync } from "./binding.js";
import { encode, encodeOptions } from "@swc-next/encoder";

export function printSync(program, options) {
  const code = __printSync(encode(program), encodeOptions(options));
  return { code };
}
