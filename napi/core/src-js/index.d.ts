import type { ParserOptions as BindingParserOptions } from "./binding.js";

export * from "./binding.js";

export type ParserOptions = Omit<BindingParserOptions, "comments"> & {
  comments?: "flat";
};

export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface ParseResult {
  program: any;
  readonly comments: Comment[];
}

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): ParseResult;
