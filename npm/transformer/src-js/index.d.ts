import type {
  GeneratorConfig as GeneratorOptions,
  GenerateOutput,
  TransformApiOptions,
  ParserConfig as BindingParserOptions,
} from "./binding.js";
export type { GeneratorConfig as GeneratorOptions, GenerateOutput } from "./binding.js";

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

export type {
  TransformApiOptions,
  TransformerConfig,
  TypeScriptConfig,
  TypeScriptConfig as TypeScriptOptions,
  JsxConfig,
  JsxRuntime,
  ReactRefreshConfig,
  ReactRefreshConfig as ReactRefreshOptions,
  JsFeature,
  Assumptions,
  ProposalConfig,
} from "./binding.js";

export interface Output extends GenerateOutput {
  /** Source map as JSON when sourceMap is enabled. */
  map?: string;
  diagnostics: Diagnostic[];
}

export type TransformOutput = Output;

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

export type { ParserConfig, GeneratorConfig } from "./binding.js";
export type ParserApiOptions = BindingParserOptions;
export type GenerateApiOptions = GeneratorOptions;
