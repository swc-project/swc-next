import type { WalkHooks } from "@swc-next/walker";
import type { Node } from "@swc-next/types";
import type { TransformApiOptions as BindingTransformApiOptions, JsOutput } from "./binding.js";

export type {
  Assumptions,
  CommentMode,
  DecoratorConfig,
  Diagnostic,
  DiagnosticLabel,
  GeneratorConfig,
  JsOutput,
  JsxConfig,
  Lang,
  ModuleConfig,
  ParserConfig,
  ReactRefreshConfig,
  SourceType,
  TransformerConfig,
  TypeScriptConfig,
} from "./binding.js";

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
