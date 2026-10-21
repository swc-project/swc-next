import type { Node } from "@swc-next/types";
import type { WalkHooks } from "@swc-next/walker";
export type * from "@swc-next/types";

import type {
  CodegenOptions,
  CodegenOutput,
  ParserOptions,
  TransformPassOptions,
} from "./binding.js";
export type {
  CommentMode,
  CodegenOptions,
  CodegenOutput,
  ParserOptions,
  TransformPassOptions,
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

/** A synchronous AST plugin. Each plugin completes its walk in array order. */
export interface JsPlugin {
  name: string;
  /** Run before or after the Rust transform. Defaults to "before". */
  phase?: "before" | "after";
  /** Full ESTree/TS-ESTree hooks, including Program. Hooks must return undefined. */
  visitors: WalkHooks<unknown, Node>;
}

export interface TransformOptions {
  /** Infer language/source type from this path. Without a filename, parse as a JS module. */
  filename?: string;
  /** Override filename inference. Transform always checks semantic early errors. */
  parser?: Omit<ParserOptions, "semanticErrors"> | null;
  transform?: TransformPassOptions | null;
  codegen?: CodegenOptions | null;
  /** Before plugins run first, then Rust transforms, then after plugins.
   * Stages without plugins stay in Rust without transferring their AST.
   */
  plugins?: readonly JsPlugin[];
}

export interface TransformOutput extends CodegenOutput {
  diagnostics: Diagnostic[];
}

/**
 * Parse, run JS plugins around Rust transforms, and print synchronously.
 * TypeScript is lowered by default; JSX is preserved. Source maps and comment
 * printing are not yet supported. Plugin errors propagate unchanged.
 * With no JS plugins, the entire pipeline runs in Rust without AST transfers.
 * @throws {SyntaxError} On parser or semantic errors, with a diagnostics array
 * containing UTF-16 source offsets. No later phase runs after an error.
 */
export declare function transformSync(
  source: string,
  options?: TransformOptions | null,
): TransformOutput;
