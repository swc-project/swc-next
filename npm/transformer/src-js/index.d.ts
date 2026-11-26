import type { TransformApiOptions, JsOutput } from "./binding.js";

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
  TransformApiOptions,
  TransformerConfig,
  TypeScriptConfig,
} from "./binding.js";

/** Transform JavaScript or TypeScript source synchronously.
 * Syntax and transform diagnostics are returned in `diagnostics`.
 * Options encoding is not reentrant. Option getters and Proxy traps must not
 * call this transformSync again while it is encoding options.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | undefined | null,
): JsOutput;
