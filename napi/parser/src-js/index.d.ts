import type { FragmentContext, ParserOptions } from "./binding.js";

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

export interface ParseExpressionFragmentResult {
  readonly expression: any;
  readonly comments: Comment[];
}

export interface ParseStatementFragmentResult {
  readonly statement: any;
  readonly comments: Comment[];
}

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): ParseResult;

export declare function parseExpressionFragmentSync(
  source: string,
  fragmentContext: FragmentContext,
  options?: ParserOptions | undefined | null,
): ParseExpressionFragmentResult;

export declare function parseStatementFragmentSync(
  source: string,
  fragmentContext: FragmentContext,
  options?: ParserOptions | undefined | null,
): ParseStatementFragmentResult;
