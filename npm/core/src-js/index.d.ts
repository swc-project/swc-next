import type { Program } from "@swc-next/types";
export type * from "@swc-next/types";
export * from "@swc-next/walker";
export { readAst, AstView, NodeView, NodeKind } from "@swc-next/ast";
export type { AstComment, AstDiagnostic } from "@swc-next/ast";
import type { AstView } from "@swc-next/ast";

import type {
  GeneratorConfig,
  GenerateOutput,
  ParserConfig,
} from "./binding.js";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export type {
  CommentMode,
  ParserConfig,
  GeneratorConfig,
  GenerateOutput,
} from "./binding.js";

export type GenerateApiOptions = GeneratorConfig;

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
  readonly program: Program;
  readonly comments: Comment[];
  /** Parser diagnostics and, when semanticErrors is enabled, semantic early errors. Decoded lazily. */
  readonly diagnostics: Diagnostic[];
}

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): ParseResult;

/** Serialize source, native AST and diagnostics into one self-contained FFI2 buffer. */
export declare function parseBufferSync(
  source: string,
  options?: Omit<ParseApiOptions, "errorRecovery"> | undefined | null,
): Uint8Array;

/** Editable views over FFI2. Source offsets convert to UTF-16 lazily on access. */
export declare function parseAstSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): AstView;

export declare function generateSync(
  program: Program,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

/** Generate a validated FFI2 base/extension pair produced by AstView.encode(). */
export declare function generateBuffersSync(
  buffer: Uint8Array,
  extension?: Uint8Array,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;
