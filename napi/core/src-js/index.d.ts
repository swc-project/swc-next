import type { ParserOptions } from "./binding.js";

export * from "./binding.js";

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): {
  program: any;
};
