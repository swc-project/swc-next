import type { AstMeta } from "@swc-next/encoder";
export type { AstMeta } from "@swc-next/encoder";

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
  program: any,
  options?: CodegenOptions | undefined | null,
  meta?: Partial<AstMeta>,
): CodegenOutput;
