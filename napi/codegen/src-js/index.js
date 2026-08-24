import { printSync as __printSync } from "./binding.js";
import { encode } from "./encode.js";

export function printSync(program, options) {
  return __printSync(encode(program), options);
}
