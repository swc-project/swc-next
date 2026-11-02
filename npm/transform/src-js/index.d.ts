import type { Node } from "@swc-next/types";
import type { WalkHooks } from "@swc-next/walker";
export type * from "@swc-next/types";

import type { CodegenOptions, CodegenOutput, ParserOptions as BindingParserOptions } from "./binding.js";
export type { CodegenOptions, CodegenOutput } from "./binding.js";

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

/** Rust parser options. Transform always collects diagnostics and omits comments. */
export type TransformParserOptions = Omit<BindingParserOptions, "comments" | "semanticErrors"> & {
  comments?: "none";
  semanticErrors?: true;
  errorRecovery?: true;
};

/** Matches Rust TypeScriptOptions. Omitted fields use Rust defaults. */
export interface TypeScriptOptions {
  /** Preserve value imports/exports and disable const-enum inlining/removal. Defaults to false. */
  verbatimModuleSyntax?: boolean;
  /** Retain const-enum declarations while allowing inlining. Defaults to false. */
  preserveConstEnums?: boolean;
  /** Factory dependencies for JSX preserved for a later transform. */
  jsx?: JsxImportOptions;
}

/** Matches Rust JsxImportOptions; this controls import retention, not JSX lowering. */
export interface JsxImportOptions {
  automatic?: boolean;
  pragma?: string | null;
  pragmaFrag?: string | null;
}

/** Matches Rust ES2022Options. */
export interface ES2022Options {
  /** Lower class static blocks. Defaults to false. */
  staticBlocks?: boolean;
}

/** Matches Rust ProposalOptions. */
export interface ProposalOptions {
  /** Preserve static-block behavior when a block makes its class object non-extensible. */
  preventExtensions?: boolean;
}

/** Rust-aligned configuration. Unsupported fields and values throw TypeError. */
export interface TransformOptions {
  /** Used for diagnostics and language inference when parser.lang is omitted. */
  filename?: string;
  parser?: TransformParserOptions;
  /** Enabled by default for TypeScript input. false retains TS syntax; {} uses Rust defaults. */
  typescript?: TypeScriptOptions | false;
  es2022?: ES2022Options;
  proposal?: ProposalOptions;
  codegen?: CodegenOptions;
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
 * TypeScript inputs are lowered by default; JSX is preserved. Source maps and comment
 * printing are not yet supported. Plugin errors propagate unchanged.
 * With no JS plugins, the entire pipeline runs in Rust without AST transfers.
 * @throws {SyntaxError} On parser or semantic errors, with a diagnostics array
 * containing UTF-16 source offsets. Before-plugin ASTs undergo semantic checks;
 * parser-only context checks are not rerun. After-plugin edits are printed
 * without another semantic check.
 * No later phase runs after an error. Invalid options throw TypeError.
 */
export declare function transformSync(
  source: string,
  options?: TransformOptions | null,
): TransformOutput;
