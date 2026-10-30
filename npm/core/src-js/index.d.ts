import type { Program } from "@swc-next/types";
export type * from "@swc-next/types";
export * from "@swc-next/walker";

import type {
  CodegenOptions,
  CodegenOutput,
  ParserOptions as BindingParserOptions,
} from "./binding.js";
import type { AstMeta } from "@swc-next/decoder";
export type { AstMeta } from "@swc-next/decoder";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export type {
  CommentMode,
  CodegenOptions,
  CodegenOutput,
} from "./binding.js";

export interface ParserOptions extends BindingParserOptions {
  /** Return diagnostics and the recovered AST instead of throwing the first diagnostic. @default false */
  errorRecovery?: boolean;
}

export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface DiagnosticLabel {
  message: string;
  /** UTF-16 offset in the source. */
  start: number;
  /** UTF-16 offset in the source. */
  end: number;
}

export interface Diagnostic extends DiagnosticLabel {
  severity: "error" | "warning" | "hint" | "info";
  help?: string;
  labels: DiagnosticLabel[];
}

export interface ParseResult {
  readonly meta: AstMeta;
  readonly program: Program;
  readonly comments: Comment[];
  /** Parser diagnostics and, when semanticErrors is enabled, semantic early errors. Decoded lazily. */
  readonly diagnostics: Diagnostic[];
}

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): ParseResult;

export declare function printSync(
  program: Program,
  options?: CodegenOptions | undefined | null,
  meta?: Partial<AstMeta>,
): CodegenOutput;
