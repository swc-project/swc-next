import type { Lang, Program, SourceType } from "@swc-next/types";
export type * from "@swc-next/types";

import type {
  Diagnostic,
  GeneratorConfig,
  GenerateOutput,
  ParserConfig,
  TransformApiOptions,
  JsOutput,
} from "./binding.js";

export type {
  CommentMode,
  Diagnostic,
  DiagnosticLabel,
  ParserConfig,
  GeneratorConfig,
  GenerateOutput,
  TransformApiOptions,
  JsOutput,
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

export interface ParseResult {
  readonly program: Program;
  readonly comments: Comment[];
  /** Parser diagnostics and, when semanticErrors is enabled, semantic early errors. Decoded lazily. */
  readonly diagnostics: Diagnostic[];
}

/**
 * Initializes the WebAssembly binding. Call this once before using the browser API.
 * Only the first call performs initialization; subsequent calls are idempotent no-ops.
 */
export default function init(): Promise<void>;

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): ParseResult;

export declare function generateSync(
  program: Program,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

export declare function langFromPath(path: string): Lang;
export declare function sourceTypeFromPath(path: string): SourceType;

/** Parse and print source synchronously.
 * Built-in transformations, including TypeScript type erasure and JSX lowering,
 * are not currently implemented by this API.
 * Syntax errors are returned in diagnostics.
 * Options encoding shares state across entry points and is not reentrant. Option
 * getters and Proxy traps must not call any transformSync or encodeTransformApiOptions
 * while it is encoding options.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | undefined | null,
): JsOutput;
