import type { Program } from "@swc-next/types";
export type * from "@swc-next/types";
export * from "@swc-next/walker";

import type {
  GeneratorConfig,
  GenerateOutput,
  ParserConfig,
  TransformApiOptions,
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

export declare function generateSync(
  program: Program,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

export type {
  TransformApiOptions,
  TransformerConfig,
  TypeScriptConfig,
  JsxConfig,
  JsxRuntime,
  ReactRefreshConfig,
  JsFeature,
  Assumptions,
  ProposalConfig,
} from "./binding.js";

export interface Output extends GenerateOutput {
  /** Source map as JSON when sourceMap is enabled. */
  map?: string;
  diagnostics: Diagnostic[];
}

/**
 * Parse, transform, and print synchronously using the Rust core options.
 * TypeScript and JSX lowering follow the input language. Set parser.semanticErrors
 * to true for additional semantic error checks; transform binding analysis always runs.
 * @throws {SyntaxError} On parser, enabled semantic, or transform errors, with a
 * diagnostics array containing UTF-16 source offsets. Invalid options throw TypeError.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | null,
): Output;
