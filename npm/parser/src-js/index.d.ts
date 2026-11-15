import type { Program } from "@swc-next/types";
export type * from "@swc-next/types";

import type { Diagnostic, ParserConfig } from "./binding.js";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export type { CommentMode, Diagnostic, DiagnosticLabel, ParserConfig } from "./binding.js";

export interface ParseApiOptions extends ParserConfig {
  /** Return diagnostics and the recovered AST instead of throwing the first diagnostic. @default false */
  errorRecovery?: boolean;
}

export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface ParseResult {
  readonly program: Program;
  readonly comments: Comment[];
  /** Parser diagnostics and, when semanticErrors is enabled, semantic early errors. Decoded lazily. */
  readonly diagnostics: Diagnostic[];
}

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): ParseResult;
