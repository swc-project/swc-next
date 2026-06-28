import { Program } from "../dist";
import {
  parseSync as __parseSync,
  printSync as __printSync,
  ParseOptions,
  PrintOptions,
} from "./binding";
import { decode } from "./decode";
import { encode } from "./encode";

export function parseSync(source: string, options?: ParseOptions) {
  const buffer = __parseSync(source, options);
  return decode(buffer, source);
}

export function printSync(ast: Program, options?: PrintOptions) {
  const buffer = encode(ast);
  return __printSync(buffer, options);
}

export { type ParseOptions } from "./binding";
export { Visitor } from "./visit";
export * from "./node";
