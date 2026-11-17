import type { WalkHooks } from "@swc-next/walker";
import type { Node } from "@swc-next/types";
import type { Program } from "@swc-next/types";
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
  CommentMode,
  Diagnostic,
  DiagnosticLabel,
  ParserConfig,
  GeneratorConfig,
  GenerateOutput,
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

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): ParseResult;

export declare function generateSync(
  program: Program,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

/** Parse and print source synchronously.
 * Built-in transformations, including TypeScript type erasure and JSX lowering,
 * are not currently implemented by this API.
 * Without plugins, syntax errors are returned in diagnostics. With plugins,
 * invalid input or enabled semantic errors throw SyntaxError before later hooks.
 * afterParse runs before beforeCodegen; native transforms remain unavailable.
 * Nonempty plugins do not support source maps or preserved source comments.
 * Options encoding is not reentrant. Option getters and Proxy traps must not
 * call this transformSync again while it is encoding options.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | undefined | null,
): JsOutput;

/** A synchronous AST plugin. Plugins finish one walk each, in stage array order. */
export interface JsPlugin {
  name: string;
  stage?: "afterParse" | "beforeCodegen";
  visitor: WalkHooks<unknown, Node>;
}
export interface TransformApiOptions extends BindingTransformApiOptions {
  plugins?: readonly JsPlugin[];
}
