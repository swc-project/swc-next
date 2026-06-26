import { parseSync as __parseSync, ParseOptions } from "./binding";
import { decode } from "./decode";

export function parseSync(source: string, options?: ParseOptions) {
  const buffer = __parseSync(source, options);
  return decode(buffer, source);
}

export { type ParseOptions } from "./binding";
export { Visitor } from "./visit";
export * from "./node";
