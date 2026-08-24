import {
  parseSync as __parseSync,
  printSync as __printSync,
} from "./binding.js";
import { decode } from "./decode.js";
import { encode } from "./encode.js";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, options);
  return decode(buffer, source);
}

export function printSync(program, options) {
  return __printSync(encode(program), options);
}
