export type * from "#protocol/index";
export * from "./walker/index.js";
export { readAst, AstView, NodeView, NodeKind, AstList } from "#protocol/index";
import type { AstView, AstNode, Program } from "#protocol/index";

import type {
  GeneratorConfig,
  GenerateOutput,
  ParserConfig,
  TransformApiOptions,
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
  TransformApiOptions,
  JsOutput,
} from "./binding.js";

export type GenerateApiOptions = GeneratorConfig;

export interface ParseApiOptions extends ParserConfig {
  /** Return diagnostics and the recovered AST instead of throwing the first diagnostic. @default false */
  errorRecovery?: boolean;
}

export type ParseResult = AstView;

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): AstView;

/** Serialize source, native AST and diagnostics into one self-contained AST buffer. */
export declare function parseBufferSync(
  source: string,
  options?: Omit<ParseApiOptions, "errorRecovery"> | undefined | null,
): Uint8Array;

export declare function generateSync(
  program: AstNode<Program>,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

/** Parse and print source synchronously.
 * Built-in transformations, including TypeScript type erasure and JSX lowering,
 * are not currently implemented by this API.
 * Syntax errors are returned in diagnostics.
 * Options encoding is not reentrant. Option getters and Proxy traps must not
 * call this transformSync again while it is encoding options.
 */
export declare function transformSync(
  source: string,
  options?: TransformApiOptions | undefined | null,
): JsOutput;

/**
 * Generate a validated AST base/extension pair produced by AstView.encode().
 * Pass immutable buffers; copy shared backing stores such as SharedArrayBuffer
 * into a non-shared Uint8Array first.
 */
export declare function generateBuffersSync(
  buffer: Uint8Array,
  extension?: Uint8Array | null,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;
