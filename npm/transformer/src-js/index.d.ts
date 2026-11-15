import type { TransformApiOptions, JsOutput } from "./binding.js";

export type {
  CommentMode,
  Diagnostic,
  DiagnosticLabel,
  GeneratorConfig,
  Lang,
  ParserConfig,
  SourceType,
  TransformApiOptions,
  JsOutput,
} from "./binding.js";

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
