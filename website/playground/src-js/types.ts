import type { Diagnostic } from "@swc-next/decoder";
import type { Lang, Program, SourceType } from "@swc-next/types";

export interface Options {
  parser: {
    lang: Lang;
    sourceType: SourceType;
    preserveParens: boolean;
    semanticErrors: boolean;
  };
  transform: { ts: boolean; es2022: { classStaticBlock: boolean } };
  codegen: { minify: boolean; asciiOnly: boolean; inlineScript: boolean };
}

export interface PlaygroundState {
  source: string;
  options: Options;
}
export interface CompileRequest extends PlaygroundState {
  id: number;
}
export type CompileResponse =
  | {
      id: number;
      code: string;
      ast: Program;
      diagnostics: Diagnostic[];
      duration: number;
    }
  | { id: number; error: string };
export type CompileResult = Extract<CompileResponse, { code: string }>;
export type SourceRange = { start: number; end: number };
