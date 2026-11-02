export { readAst, AstView, NodeView, NodeKind, AstList } from "@swc-next/ast2";
import type { AstView, AstNode } from "@swc-next/ast2";
import type { Lang, Program, SourceType } from "@swc-next/types";
export type * from "@swc-next/types";

import type {
  GeneratorConfig,
  GenerateOutput,
  ParserConfig,
  TransformApiOptions,
  JsOutput,
} from "./binding.js";

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

export default function init(): Promise<void>;

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): AstView;

export declare function generateSync(
  program: AstNode<Program>,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

export declare function langFromPath(path: string): Lang;
export declare function sourceTypeFromPath(path: string): SourceType;

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

/** Parse source into a self-contained AST buffer, retaining diagnostics. */
export declare function parseBufferSync(source: string, options?: Omit<ParseApiOptions, "errorRecovery"> | null): Uint8Array;

export declare function generateBuffersSync(buffer: Uint8Array, extension?: Uint8Array | null, options?: GenerateApiOptions | null): GenerateOutput;
