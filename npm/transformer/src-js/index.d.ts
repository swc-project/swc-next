import type { WalkHooks } from "@swc-next/walker";
import type { Node } from "@swc-next/types";
import type {
  GenerateOutput,
  TransformApiOptions as BindingTransformApiOptions,
} from "./binding.js";

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
 * Parse, run synchronous JavaScript plugins around Rust transforms, and print.
 * Empty plugin lists keep the AST in Rust. Plugins with preserved source comments
 * require comments: "none" and do not support source maps. Plugin errors propagate.
 * Enabled semantic checks run after afterParse edits; beforeCodegen edits print
 * without another semantic check.
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

/** Synchronous plugin; each completes a walk in array order within its stage. */
export interface JsPlugin {
  name: string;
  stage?: "afterParse" | "beforeCodegen";
  visitor: WalkHooks<unknown, Node>;
}
export interface TransformApiOptions extends BindingTransformApiOptions {
  plugins?: readonly JsPlugin[];
}
