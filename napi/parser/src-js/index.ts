import { Program } from "../dist";
import {
  parseSync as __parseSync,
  printSync as __printSync,
  ParseOptions,
  PrintOptions,
} from "./binding";
import { Encoder } from "./encode";
import { decode } from "./generated/decode";
import { Measurement } from "./measure";
import { headerSize } from "./protocol";

export function parseSync(source: string, options?: ParseOptions) {
  const buffer = __parseSync(source, options);
  return decode(buffer, source);
}

export function printSync(ast: Program, options?: PrintOptions) {
  const header = new Measurement(ast).measure();
  const buffer = Buffer.alloc(headerSize(header));
  const encoder = new Encoder(header, buffer);
  encoder.encode(ast);
  return __printSync(buffer, options);
}

export { type ParseOptions } from "./binding";
export { Visitor } from "./visit";
export * from "./node";
