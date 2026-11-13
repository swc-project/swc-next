import type { TransformApiOptions, TransformOutput } from "./binding.js";

export type {
  Assumptions,
  CommentMode,
  Diagnostic,
  DiagnosticLabel,
  GeneratorConfig,
  JsxConfig,
  JsxRuntime,
  Lang,
  ParserConfig,
  ProposalConfig,
  ReactRefreshConfig,
  SourceType,
  TransformApiOptions,
  TransformerConfig,
  TransformOutput,
  TypeScriptConfig,
} from "./binding.js";

/** Transform source synchronously. Syntax errors are returned in diagnostics.
 * Options encoding reuses state and is not reentrant. Option getters and Proxy
 * traps must not reenter this transformSync function while it is encoding options.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | undefined | null,
): TransformOutput;
