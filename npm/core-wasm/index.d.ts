import type {
  CodegenOptions,
  CodegenOutput,
  ParserOptions,
} from "./binding.js";

export { CommentMode, Lang, SourceType } from "./binding.js";
export type { CodegenOptions, CodegenOutput, ParserOptions } from "./binding.js";

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

/**
 * Initializes the WebAssembly binding. Call this once before using the browser API.
 * Only the first call performs initialization; subsequent calls are idempotent no-ops.
 */
export default function init(): Promise<void>;

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): ParseResult;

export declare function printSync(
  program: any,
  options?: CodegenOptions | undefined | null,
): CodegenOutput;

export declare function langFromPath(path: string): import("./binding.js").Lang;
export declare function sourceTypeFromPath(
  path: string,
): import("./binding.js").SourceType;
