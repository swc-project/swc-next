import type {
  CodegenOptions,
  CodegenOutput,
  ParserOptions,
} from "./binding.js";

export * from "./binding.js";

export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface ParseResult {
  readonly program: any;
  readonly comments: Comment[];
}

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): ParseResult;

export declare function printSync(
  program: any,
  options?: CodegenOptions | undefined | null,
): CodegenOutput;
