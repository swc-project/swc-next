import type { Program } from "@swc-next/types";
export type * from "@swc-next/types";
export * from "@swc-next/walker";

import type {
  Diagnostic,
  GeneratorConfig,
  GenerateOutput,
  ParserConfig,
  TransformApiOptions,
  JsOutput,
} from "./binding.js";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export type {
  Assumptions,
  CommentMode,
  DecoratorConfig,
  Diagnostic,
  DiagnosticLabel,
  GeneratorConfig,
  GenerateOutput,
  JsOutput,
  JsxConfig,
  ModuleConfig,
  ParserConfig,
  ReactRefreshConfig,
  TransformApiOptions,
  TransformerConfig,
  TypeScriptConfig,
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

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): ParseResult;

export declare function generateSync(
  program: Program,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

/** Transform JavaScript or TypeScript source synchronously.
 * Syntax and transform diagnostics are returned in `diagnostics`.
 * Options encoding is not reentrant. Option getters and Proxy traps must not
 * call this transformSync again while it is encoding options.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | undefined | null,
): JsOutput;
