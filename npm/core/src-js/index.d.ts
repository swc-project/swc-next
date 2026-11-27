import type { WalkHooks } from "@swc-next/walker";
import type { Node, Program } from "@swc-next/types";
export type * from "@swc-next/types";
export * from "@swc-next/walker";

import type {
  Diagnostic,
  GeneratorConfig,
  GenerateOutput,
  ParserConfig,
  TransformApiOptions as BindingTransformApiOptions,
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
 * Without plugins, syntax and transform diagnostics are returned in `diagnostics`.
 * With plugins, invalid input, enabled semantic errors, or errors from built-in
 * transforms throw SyntaxError before later hooks.
 * Nonempty plugins do not support source maps or preserved source comments.
 * Options encoding is not reentrant. Option getters and Proxy traps must not
 * call this transformSync again while it is encoding options.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | undefined | null,
): JsOutput;

/** Synchronous AST plugin. Same-stage visitors share one walk with callbacks in array order. */
export interface JsPlugin {
  name: string;
  /** Run before built-in transforms (default) or after them, just before printing. */
  stage?: "afterParse" | "beforeCodegen";
  visitor: WalkHooks<unknown, Node>;
}
export interface TransformApiOptions extends BindingTransformApiOptions {
  plugins?: readonly JsPlugin[];
}
