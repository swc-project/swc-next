import type { Node } from "@swc-next/types";
import type { WalkHooks } from "@swc-next/walker";
export type * from "@swc-next/types";

import type { CodegenOutput } from "./binding.js";
export type { CodegenOutput } from "./binding.js";

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

/** Supported parser configuration. Explicit settings override filename inference. */
export type TransformParserOptions =
  | { syntax: "ecmascript"; jsx?: boolean }
  | { syntax: "typescript"; tsx?: boolean };

export interface TransformJscOptions {
  parser?: TransformParserOptions;
  /** Defaults to "esnext". Other targets are not implemented. */
  target?: "esnext";
  transform?: {
    /**
     * Preserve value imports/exports. Defaults to false.
     * SWC Next also disables const-enum inlining/removal; SWC 1.16.18 still
     * inlines const-enum uses with this enabled.
     */
    verbatimModuleSyntax?: boolean;
  };
  output?: {
    /** Defaults to "utf8". */
    charset?: "ascii" | "utf8";
  };
}

/** SWC Next extensions, separate from the supported SWC configuration subset. */
export interface SwcNextTransformOptions {
  /** Lower TypeScript. Defaults to true for TS input; false retains TS syntax. */
  typescript?: boolean;
  /** Lower class static blocks. Defaults to false. */
  staticBlocks?: boolean;
  /** Preserve explicit parentheses in plugin ASTs. Defaults to true. */
  preserveParens?: boolean;
  /** Escape closing script tags; tagged template raw text is preserved. Defaults to false. */
  inlineScript?: boolean;
  /** Retain const-enum declarations while allowing inlining. Defaults to false. */
  preserveConstEnums?: boolean;
}

/**
 * Strict SWC configuration subset. Unknown fields and unsupported values throw
 * TypeError with the option path. No config files are discovered or loaded.
 */
export interface TransformOptions {
  /**
   * Used for diagnostics and language inference when jsc.parser is omitted.
   * Explicit parser settings win. TS declaration filenames select ambient
   * parsing with TypeScript syntax. Does not select the input module mode.
   */
  filename?: string;
  /** Defaults to true (module). false selects script; "commonjs" allows top-level return. */
  isModule?: boolean | "commonjs";
  jsc?: TransformJscOptions;
  /** Compact printing only; does not compress code or mangle names. Defaults to false. */
  minify?: boolean;
  swcNext?: SwcNextTransformOptions;
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
 * containing UTF-16 source offsets. Source and before-plugin ASTs are checked;
 * after-plugin edits are printed without another semantic check.
 * No later phase runs after an error. Invalid options throw TypeError.
 */
export declare function transformSync(
  source: string,
  options?: TransformOptions | null,
): TransformOutput;
