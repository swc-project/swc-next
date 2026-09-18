import type { AstMeta } from "@swc-next/encoder";
export type { AstMeta } from "@swc-next/encoder";
import type { Program } from "@swc-next/types";
export type {
  Program,
  Node,
  Expression,
  Statement,
  Declaration,
  Pattern,
  Identifier,
  Literal,
} from "@swc-next/types";
export type * as AST from "@swc-next/types";

export interface CodegenOptions {
  asciiOnly?: boolean;
  /** Escape closing script tags; tagged template raw text is preserved. Defaults to false. */
  inlineScript?: boolean;
  minify?: boolean;
}

export interface CodegenOutput {
  code: string;
}

export declare function printSync(
  program: Program,
  options?: CodegenOptions | undefined | null,
  meta?: Partial<AstMeta>,
): CodegenOutput;
