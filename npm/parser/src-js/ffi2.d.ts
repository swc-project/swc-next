export { readAst, AstView, NodeView, NodeKind, AstList } from "@swc-next/ast2";
import type { AstView } from "@swc-next/ast2";
export type * from "@swc-next/ast2";

import type { ParserConfig } from "./binding.js";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export type { CommentMode, Diagnostic, DiagnosticLabel, ParserConfig } from "./binding.js";

export interface ParseApiOptions extends ParserConfig {
  /** Return diagnostics and the recovered AST instead of throwing the first diagnostic. @default false */
  errorRecovery?: boolean;
}

export type ParseResult = AstView;

export declare function parseSync(
  source: string,
  options?: ParseApiOptions | undefined | null,
): AstView;

/** Parse source into a self-contained AST buffer, retaining diagnostics. */
export declare function parseBufferSync(source: string, options?: Omit<ParseApiOptions, "errorRecovery"> | null): Uint8Array;
